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
    /// NOTE: currently only expected shutdown in the config, so input parameter is only expected_shutdown
    pub async fn update_dynamic_config(
        client: NeardClient,
        pid: Pid,
        neard_home: &Path,
        expected_shutdown: BlockHeight,
    ) -> Result<()> {
        let mut check = 0;
        let metrics = loop {
            check += 1;
            if let Ok(metrics) = client.metrics().await {
                break metrics;
            } else if check > 50 {
                anyhow::bail!("fail to get neard metrics");
            }
            sleep_until(Instant::now() + Duration::from_millis(100)).await;
        };
        let changes = metrics
            .get("near_dynamic_config_changes")
            .map(|s| s.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        let dyn_config_path = neard_home.join("dyn_config.json");

        let _guard = scopeguard::guard((), |_| {
            if let Err(e) = force_unlink(&dyn_config_path) {
                error!("dyn_config.json can not force remove as expected: {e:?}");
            }
        });

        force_unlink(&dyn_config_path).context("failed to remove previous dynamic config")?;
        let mut file = File::create(&dyn_config_path).await?;
        let dynamic_config = serde_json::json!({ "expected_shutdown": expected_shutdown });
        file.write_all(dynamic_config.to_string().as_bytes())
            .await?;
        let mut result = signal::kill(pid, Signal::SIGHUP);
        if let Err(e) = result {
            warn!("Try send SIGHUP to neard({pid:?}): {e:?}");
            result = signal::kill(pid, Signal::SIGHUP);
        }

        let mut check = 0;
        while check < 50
            && changes + 1
                != client
                    .metrics()
                    .await?
                    .get("near_dynamic_config_changes")
                    .map(|s| s.parse::<usize>().unwrap_or(0))
                    .unwrap_or(0)
        {
            check += 1;
            sleep_until(Instant::now() + Duration::from_millis(100)).await;
        }
        if check == 50 {
            anyhow::bail!("fail check on dynamic config change")
        } else {
            Ok(result?)
        }
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
