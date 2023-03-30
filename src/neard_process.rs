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
use std::path::Path;
use std::process::ExitStatus;
use tokio::fs::{read_to_string, write};
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
        settings.neard_home.join("validator_key.json"),
    )
    .context("failed to set validator key")?;
    force_symlink(
        &settings.validator_node_key,
        settings.neard_home.join("node_key.json"),
    )
    .context("failed to set validator node key")?;
    let addresses = if let Some(addr) = settings.public_address {
        vec![addr]
    } else {
        vec![]
    };
    update_neard_config(
        settings.neard_home.join("config.json"),
        &addresses,
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
    force_unlink(settings.neard_home.join("validator_key.json"))
        .context("failed to remove validator")?;

    force_symlink(
        &settings.voter_node_key,
        settings.neard_home.join("node_key.json"),
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
    return Ok(metrics
        .get("near_config_reloads_total")
        .map(|s| s.parse::<u64>().unwrap_or(0))
        .unwrap_or(0));
}

/// Trigger neard to load the dynamic config
pub fn reload_neard(pid: Pid) -> Result<()> {
    // FIXME refactor with inspect_err after following PR shipped
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

/// Apply dynamic part, which is a part of `config.json`
pub async fn apply_dynamic_config(
    client: &NeardClient,
    pid: Pid,
    neard_home: &Path,
    expected_shutdown: Option<BlockHeight>,
) -> Result<()> {
    let changes = get_neard_config_changes(client).await?;

    let config_path = neard_home.join("config.json");
    let config_content = read_to_string(&config_path).await?;
    if let Some(part) = config_content.strip_suffix("\n}") {
        if let Some(expected_shutdown) = expected_shutdown {
            let new_config = format!("{part:},\n  \"expected_shutdown\": {expected_shutdown}\n}}");
            write(&config_path, &new_config)
                .await
                .with_context(|| "Can not write new config with expect_shutdown")?;
            println!("new config({}):\n{}", config_path.display(), new_config);
        };
        reload_neard(pid)?;
    } else {
        anyhow::bail!("config.json is not end with `}}` as expected json object")
    }

    let apply_timeout = Instant::now() + Duration::from_secs(5);
    tokio::select! {
        res = wait_until_config_applied(client, changes) => {
            if write(&config_path, config_content).await.is_err() {
                error!("Can not write back the original config after expect_shutdown applied");
            }
            res
        },
        // startup timeout
        _ = sleep_until(apply_timeout) => {
            if write(&config_path, config_content).await.is_err() {
                error!("Can not write back the original config after expect_shutdown applied");
            }
            anyhow::bail!("dynamic config change was not applied within 5s")
        },
    }
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
