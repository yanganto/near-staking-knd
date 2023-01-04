use anyhow::Result;
use log::info;

use crate::deploy::nixos_rebuild;

use super::{Host, NixosFlake};

/// Push update to server but do not activate it yet.
pub fn dry_update(hosts: &[Host], flake: &NixosFlake) -> Result<()> {
    hosts
        .iter()
        .map(|host| {
            info!("Dry-update {}", host.name);

            nixos_rebuild("dry-activate", host, flake, false)?;

            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
