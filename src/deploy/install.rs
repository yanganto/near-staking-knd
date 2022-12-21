use anyhow::{Context, Result};
use log::info;
use std::process::Command;

use super::{Host, NixosFlake};

/// Install a Validator on a given machine
pub fn install(hosts: &[Host], flake: &NixosFlake) -> Result<()> {
    hosts
        .iter()
        .map(|host| {
            info!("Deploying {}", host.name);
            let connection_string = if host.install_ssh_user.is_empty() {
                host.ssh_hostname.clone()
            } else {
                format!("{}@{}", host.install_ssh_user, host.ssh_hostname)
            };
            let flake_uri = format!("{}#{}", flake.path().display(), host.nixos_module);
            let args = &["--flake", &flake_uri, &connection_string];
            println!("$ nixos-remote {}", args.join(" "));
            let status = Command::new("nixos-remote").args(args).status();
            status.with_context(|| format!("nixos-remote failed (nixos-remote {})", args.join(" ")))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
