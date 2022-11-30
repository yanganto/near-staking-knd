//! Setups neard in validator or voter mode

use crate::near_client::NeardClient;
use crate::near_config::update_neard_config;
use crate::proc::{graceful_stop_neard, run_neard};
use crate::settings::Settings;
use anyhow::{Context, Result};
use log::{error, warn};
use near_primitives::types::BlockHeight;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Child;
use tokio::time::{sleep, sleep_until, Duration, Instant};

/// A neard validator process
#[derive(Debug)]
pub struct NeardProcess {
    process: Child,
    sent_kill: bool,
}

// ignores non-existing files
fn force_unlink<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Err(e) = remove_file(path.as_ref()) {
        if e.kind() != ErrorKind::NotFound {
            Err(e).context(format!(
                "Cannot remove existing file at {}",
                path.as_ref().display()
            ))?;
        }
    }
    Ok(())
}

fn force_symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> Result<()> {
    force_unlink(link.as_ref()).context("failed to unlink old path")?;
    symlink(original, link.as_ref())
        .with_context(|| format!("failed to create symlink at: {}", link.as_ref().display()))?;
    Ok(())
}

/// Setup a neard process as a validator (neard process with a validator key)
pub fn setup_validator(settings: &Settings) -> Result<NeardProcess> {
    force_symlink(
        &settings.validator_key,
        &settings.neard_home.join("validator_key.json"),
    )
    .context("failed to set validator key")?;
    force_symlink(
        &settings.validator_node_key,
        &settings.neard_home.join("node_key.json"),
    )
    .context("failed to set validator node key")?;

    update_neard_config(
        settings.neard_home.join("config.json"),
        &settings.public_addresses,
        settings.validator_network_addr.port(),
        &settings.validator_node_public_key,
        &settings.validator_network_addr,
    )
    .context("failed to update network addr in near config")?;

    let process = run_neard(&settings.neard_home, &settings.near_boot_nodes)
        .context("Cannot start validator neard")?;

    Ok(NeardProcess {
        process,
        sent_kill: false,
    })
}

/// Setup a neard process as a voter (neard process without a validator key)
pub fn setup_voter(settings: &Settings) -> Result<NeardProcess> {
    // remove any old validator key as we start in non-validation mode.
    force_unlink(&settings.neard_home.join("validator_key.json"))
        .context("failed to remove validator")?;

    force_symlink(
        &settings.voter_node_key,
        &settings.neard_home.join("node_key.json"),
    )
    .context("failed to set validator node key")?;

    // We set the public address to an empty list as it is not broadcasted to the network if we are not validating
    update_neard_config(
        settings.neard_home.join("config.json"),
        &[],
        0,
        "",
        &settings.voter_network_addr,
    )
    .context("failed to update network addr in near config")?;

    let process = run_neard(&settings.neard_home, &settings.near_boot_nodes)
        .context("Cannot start voter neard")?;

    Ok(NeardProcess {
        process,
        sent_kill: false,
    })
}

async fn get_neard_config_changes(client: &NeardClient) -> Result<u64> {
    let metrics = client.metrics().await?;

    metrics
        .get("near_dynamic_config_changes")
        .context("metrics do not contain the near_dynamic_config_changes field")?
        .parse::<u64>()
        .context("near_dynamic_config_changes")
}

/// Trigger neard to load the dynamic config
pub fn reload_neard(pid: Pid) -> Result<()> {
    // FIXME refactor with inspect_err after following PR shiped
    // https://github.com/rust-lang/rust/issues/91345
    match signal::kill(pid, Signal::SIGHUP) {
        Ok(s) => Ok(s),
        Err(e) => {
            warn!("Try send SIGHUP to neard({:?}): {e:?}", pid);
            Err(e.into())
        }
    }
}

async fn wait_until_config_applied(client: &NeardClient, initial_change_value: u64) -> Result<()> {
    loop {
        let latest_changes = get_neard_config_changes(client).await?;
        if latest_changes > initial_change_value {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
}

/// Apply dynamic config
/// NOTE: currently only expected shutdown in the config, so input parameter is only
/// expected_shutdown
pub async fn apply_dynamic_config(
    client: &NeardClient,
    pid: Pid,
    neard_home: &Path,
    expected_shutdown: BlockHeight,
) -> Result<()> {
    // We need neard metric to make sure the dyn config is correctly applied.
    // If we can not get the neard metric at this moment, we will not try to apply the dynamic
    // config and abort early.
    let changes = get_neard_config_changes(client).await?;

    let dyn_config = DynConfig::new(neard_home, expected_shutdown).await?;
    reload_neard(pid)?;

    // Check the dynamic config effect and show in metrics
    // Actually, the dynamic config is applied when SIGHUP sent, and we can check it change on
    // log in the same time, however, the metrics takes much more time to reflect these, so we
    // wait 5 seconds to check on this.
    let apply_timeout = Instant::now() + Duration::from_secs(5);
    tokio::select! {
        res = wait_until_config_applied(client, changes) => {
            drop(dyn_config);
            return res;
        }
        // startup timeout
        _ = sleep_until(apply_timeout) => {},
    }
    anyhow::bail!("dynamic config change was not applied within 5s")
}

impl NeardProcess {
    /// Return handle on the neard process
    pub fn process(&mut self) -> &mut Child {
        &mut self.process
    }

    /// Stops a process by first sending SIGTERM and than after `NEARD_STOP_TIMEOUT`
    pub fn graceful_stop(mut self) -> Result<()> {
        let res = graceful_stop_neard(&mut self.process);
        self.sent_kill = true;
        res
    }

    /// Wait for process to stop
    pub async fn wait(&mut self) -> std::result::Result<ExitStatus, std::io::Error> {
        self.process.wait().await
    }

    /// Get Pid of neard
    pub fn pid(&self) -> Option<Pid> {
        if let Some(pid) = self.process.id() {
            if let Ok(i) = pid.try_into() {
                return Some(Pid::from_raw(i));
            }
        }
        None
    }

    /// Restart by sending terminate signal without update `self.sent_kill` such that it will restart by kuutamod
    pub async fn restart(pid: Pid) -> Result<()> {
        let mut result = signal::kill(pid, Signal::SIGTERM);
        if result.is_err() {
            result = signal::kill(pid, Signal::SIGTERM);
        }
        warn!("Try send SIGTERM to neard({:?})", pid);
        Ok(result?)
    }
}

impl Drop for NeardProcess {
    fn drop(&mut self) {
        if !self.sent_kill {
            if let Err(err) = graceful_stop_neard(&mut self.process) {
                warn!("Failed to stop near process: {err:?}");
            }
        }
    }
}

/// Safty operation to apply dyanic config
struct DynConfig {
    dyn_config_path: PathBuf,
}

impl DynConfig {
    pub async fn new(neard_home: &Path, expected_shutdown: BlockHeight) -> Result<Self> {
        let dyn_config_path = neard_home.join("dyn_config.json");
        let dynamic_config = serde_json::json!({ "expected_shutdown": expected_shutdown });

        // The previous config file will be truncated if existing
        let mut file = File::create(&dyn_config_path).await?;
        file.write_all(dynamic_config.to_string().as_bytes())
            .await?;

        Ok(Self { dyn_config_path })
    }
}

impl Drop for DynConfig {
    /// Make sure the config file be deleted to avoid neard reload it when restarting,
    /// because neard process will load any existing dyn_config.json when it start.
    fn drop(&mut self) {
        if let Err(e) = force_unlink(&self.dyn_config_path) {
            error!("dyn_config.json can not force remove as expected: {e:?}");
        }
    }
}
