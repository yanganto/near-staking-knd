//! Setups neard in validator or voter mode

use crate::near_config::update_neard_config;
use crate::proc::{graceful_stop_neard, run_neard};
use crate::settings::Settings;
use anyhow::{Context, Result};
use log::warn;
use near_primitives::types::BlockHeight;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Child;

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
    pub fn pid(&self) -> Pid {
        // FIXME: correctly handle unwrap here
        let pid: i32 = self.process.id().unwrap().try_into().unwrap();
        Pid::from_raw(pid)
    }

    /// Update dynamic config
    /// NOTE: currently only expected shutdown in the config, so input parameter is only expected_shutdown
    pub async fn update_dynamic_config(
        pid: Pid,
        dyn_config_path: &PathBuf,
        expected_shutdown: BlockHeight,
    ) -> Result<()> {
        force_unlink(&dyn_config_path).context("failed to remove previous dynamic config")?;
        let mut file = File::create(&dyn_config_path).await?;
        let dynamicConfig = json!({
          "expected_shutdown": expected_shutdown
        });
        file.write_all(dynamicConfig.to_string().as_bytes()).await?;
        let mut result = signal::kill(pid, Signal::SIGHUP);
        for i in 1..=3 {
            if let Err(e) = result {
                warn!("{i} time try send SIGHUP to neard({pid:?}): {e:?}");
                result = signal::kill(pid, Signal::SIGHUP);
            }
        }
        // TODO check dyn_config setup correctly, when this issue is fixed
        // https://github.com/near/nearcore/issues/7990
        Ok(result?)
    }

    /// Restart by sending terminate signal without update `self.sent_kill` such that it will restart by kuutamod
    pub async fn restart(pid: Pid) -> Result<()> {
        let mut result = signal::kill(pid, Signal::SIGTERM);
        for i in 1..=3 {
            if result.is_err() {
                result = signal::kill(pid, Signal::SIGTERM);
            }
            warn!("{i} time try send SIGTERM to neard({:?})", pid);
        }
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
