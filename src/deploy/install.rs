use anyhow::Result;
use log::info;
use std::process::Command;

use crate::deploy::command::status_to_pretty_err;

use super::{Host, NixosFlake};

/// Install a Validator on a given machine
pub fn install(hosts: &[Host], kexec_url: &str, flake: &NixosFlake) -> Result<()> {
    hosts
        .iter()
        .map(|host| {
            info!("Install {}", host.name);
            let connection_string = if host.install_ssh_user.is_empty() {
                host.ssh_hostname.clone()
            } else {
                format!("{}@{}", host.install_ssh_user, host.ssh_hostname)
            };

            let secrets = host.secrets()?;
            let flake_uri = format!("{}#{}", flake.path().display(), host.name);
            let args = &[
                "--debug",
                "--no-ssh-copy-id",
                "--extra-files",
                &format!("{}", secrets.path().display()),
                "--kexec",
                kexec_url,
                "--flake",
                &flake_uri,
                &connection_string,
            ];
            println!("$ nixos-remote {}", args.join(" "));
            let status = Command::new("nixos-remote").args(args).status();
            status_to_pretty_err(status, "nixos-remote", args)?;
            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
