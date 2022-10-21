//! Setups neard in validator or voter mode

use crate::near_config::update_neard_config;
use crate::proc::{graceful_stop_neard, run_neard};
use crate::settings::Settings;
use anyhow::{Context, Result};
use log::warn;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::ExitStatus;
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
