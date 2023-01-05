use anyhow::Result;
use log::info;

use crate::deploy::nixos_rebuild;

use super::{Host, NixosFlake};

/// Update Validator on a given machine
pub fn update(hosts: &[Host], flake: &NixosFlake) -> Result<()> {
    hosts
        .iter()
        .map(|host| {
            info!("Update {}", host.name);

            nixos_rebuild("switch", host, flake, true)?;

            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
