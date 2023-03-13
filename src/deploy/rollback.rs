use anyhow::{bail, Result};
use log::info;

use crate::deploy::nixos_rebuild;

use super::{
    utils::{handle_maintenance_shutdown, timeout_ssh},
    Host, NixosFlake,
};

/// Rollback a nixos machine
pub async fn rollback(
    hosts: &[Host],
    flake: &NixosFlake,
    immediately: bool,
    required_time_in_blocks: Option<u64>,
) -> Result<()> {
    flake.show()?;
    for host in hosts.iter() {
        info!("Rollback {}", host.name);

        if immediately {
            nixos_rebuild("rollback", host, flake, false)?;
        } else if let Some(required_time_in_blocks) = required_time_in_blocks {
            nixos_rebuild("build", host, flake, true)?;
            let r = match timeout_ssh(host, &["systemctl", "disable", "kuutamod"], true) {
                Ok(_) => handle_maintenance_shutdown(host, required_time_in_blocks)
                    .await
                    .and_then(|_| nixos_rebuild("rollback", host, flake, true)),
                Err(e) => Err(e),
            };
            let _ = timeout_ssh(host, &["systemctl", "enable", "kuutamod"], true)?;
            r?
        } else {
            bail!("please specify [REQUIRED_TIME_IN_BLOCKS] or pass `--immediately`, or `--help` to learn more.")
        }
    }
    Ok(())
}
