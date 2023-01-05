use anyhow::{Context, Result};
use ctrlc;
use log::info;
use std::{
    process::Command,
    sync::mpsc::{channel, RecvTimeoutError},
    time::Duration,
};

use crate::deploy::command::status_to_pretty_err;

use super::{Host, NixosFlake};

/// Install a Validator on a given machine
pub fn install(hosts: &[Host], kexec_url: &str, flake: &NixosFlake, no_reboot: bool) -> Result<()> {
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
            let extra_files = format!("{}", secrets.path().display());
            let mut args = vec![
                "--debug",
                "--no-ssh-copy-id",
                "--extra-files",
                &extra_files,
                "--kexec",
                kexec_url,
                "--flake",
                &flake_uri,
            ];
            if no_reboot {
                args.push("--no-reboot");
            }
            args.push(&connection_string);
            println!("$ nixos-remote {}", args.join(" "));
            let status = Command::new("nixos-remote").args(&args).status();
            status_to_pretty_err(status, "nixos-remote", &args)?;

            if no_reboot {
                return Ok(());
            }

            info!(
                "Installation of {} finished. Waiting for reboot.",
                host.name
            );

            let (ctrlc_tx, ctrlc_rx) = channel();
            ctrlc::set_handler(move || {
                info!("received ctrl-c!. Stopping program...");
                let _ = ctrlc_tx.send(());
            })
            .context("Error setting ctrl-C handler")?;

            // Wait for the machine to come back and learn add it's ssh key to our host
            while matches!(
                ctrlc_rx.recv_timeout(Duration::from_millis(0)),
                Err(RecvTimeoutError::Timeout)
            ) {
                let args = &[
                    "-o",
                    "ConnectTimeout=10",
                    "-o",
                    "StrictHostKeyChecking=no",
                    &host.deploy_ssh_target(),
                ];
                println!("$ ssh {}", args.join(" "));
                let status = Command::new("ssh")
                    .args(args)
                    .status()
                    .context("Failed to run ssh...")?;
                if status.success() {
                    break;
                }
            }

            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
