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
use tokio::time::{sleep_until, Duration, Instant};

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

    /// Update dynamic config
    /// NOTE: currently only expected shutdown in the config, so input parameter is only
    /// expected_shutdown
    pub async fn update_dynamic_config(
        client: &NeardClient,
        pid: Pid,
        neard_home: &Path,
        expected_shutdown: BlockHeight,
    ) -> Result<()> {
        // We need neard metric to make sure the dyn config is correctly applied.
        // If we can not get the neard metric at this moment, we will not try to apply the dynamic
        // config and abort early.
        let metrics = client.metrics().await?;

        let changes = metrics
            .get("near_dynamic_config_changes")
            .map(|s| s.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);

        let op = ApplyDynConfig::new(pid, neard_home, expected_shutdown).await?;
        op.run_uncheck().await?;

        // Check the dynamic config effect and show in metrics
        // Actually, the dynamic config is applied when SIGHUP sent, and we can check it change on
        // log in the same time, however, the metics takes much more time to reflect these, so we
        // take 5 seconds to check on this.
        let mut check = 0;
        while check < 5
            && changes + 1
                != client
                    .metrics()
                    .await?
                    .get("near_dynamic_config_changes")
                    .map(|s| s.parse::<usize>().unwrap_or(0))
                    .unwrap_or(0)
        {
            check += 1;
            sleep_until(Instant::now() + Duration::from_secs(1)).await;
        }
        if check == 5 {
            anyhow::bail!("fail check on dynamic config change")
        }
        Ok(())
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
struct ApplyDynConfig {
    dyn_config_path: PathBuf,
    pid: Pid,
}

impl ApplyDynConfig {
    pub async fn new(pid: Pid, neard_home: &Path, expected_shutdown: BlockHeight) -> Result<Self> {
        let dyn_config_path = neard_home.join("dyn_config.json");
        let dynamic_config = serde_json::json!({ "expected_shutdown": expected_shutdown });

        // The previous config file will be truncated if existing
        let mut file = File::create(&dyn_config_path).await?;
        file.write_all(dynamic_config.to_string().as_bytes())
            .await?;

        Ok(Self {
            dyn_config_path,
            pid,
        })
    }

    /// Trigger neard to load the dynamic config without check after load
    pub async fn run_uncheck(&self) -> Result<()> {
        // FIXME refactor with inspect_err after following PR shiped
        // https://github.com/rust-lang/rust/issues/91345
        match signal::kill(self.pid, Signal::SIGHUP) {
            Ok(s) => Ok(s),
            Err(e) => {
                warn!("Try send SIGHUP to neard({:?}): {e:?}", self.pid);
                Err(e.into())
            }
        }
    }
}

impl Drop for ApplyDynConfig {
    /// Make sure the config file be deleted to avoid neard reload it when restarting,
    /// because neard process will load any existing dyn_config.json when it start.
    fn drop(&mut self) {
        if let Err(e) = force_unlink(&self.dyn_config_path) {
            error!("dyn_config.json can not force remove as expected: {e:?}");
        }
    }
}
