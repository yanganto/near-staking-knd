use anyhow::Result;
use log::info;

use crate::deploy::nixos_rebuild;

use super::{utils::handle_maintenance_shutdown, Host, NixosFlake};

/// Rollback a nixos machine
pub async fn rollback(
    hosts: &[Host],
    flake: &NixosFlake,
    immediately: bool,
    required_time_in_blocks: u64,
) -> Result<()> {
    flake.show()?;
    for host in hosts.iter() {
        info!("Rollback {}", host.name);

        if immediately {
            nixos_rebuild("rollback", host, flake, false)?;
        } else {
            nixos_rebuild("build", host, flake, true)?;
            handle_maintenance_shutdown(host, required_time_in_blocks)
                .await
                .and_then(|_| nixos_rebuild("rollback", host, flake, true))?
        }
    }
    Ok(())
}
