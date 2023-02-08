use anyhow::{Context, Result};
use ctrlc;
use log::info;
use std::{
    process::{Command, ExitStatus},
    sync::mpsc::{channel, RecvTimeoutError},
    time::Duration,
};

use crate::deploy::command::status_to_pretty_err;

use super::{Host, NixosFlake};

pub fn timeout_ssh(
    host: &Host,
    command: &[&str],
    learn_known_host_key: bool,
) -> Result<ExitStatus> {
    let target = host.deploy_ssh_target();
    let mut args = vec!["-o", "ConnectTimeout=10", "-o", "StrictHostKeyChecking=no"];
    if !learn_known_host_key {
        args.push("-o");
        args.push("UserKnownHostsFile=/dev/null");
    }
    args.push(&target);
    args.push("--");
    args.extend(command);
    println!("$ ssh {}", args.join(" "));
    let status = Command::new("ssh")
        .args(args)
        .status()
        .context("Failed to run ssh...")?;
    Ok(status)
}

/// Install a Validator on a given machine
pub fn install(
    hosts: &[Host],
    kexec_url: &str,
    flake: &NixosFlake,
    debug: bool,
    no_reboot: bool,
) -> Result<()> {
    flake.show()?;
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
                "--extra-files",
                &extra_files,
                "--kexec",
                kexec_url,
                "--flake",
                &flake_uri,
                "--option",
                "accept-flake-config",
                "true",
            ];
            if debug {
                args.push("--debug");
            }
            if no_reboot {
                args.push("--no-reboot");
            }
            args.push(&connection_string);
            println!("$ nixos-anywhere {}", args.join(" "));
            let status = Command::new("nixos-anywhere").args(&args).status();
            status_to_pretty_err(status, "nixos-anywhere", &args)?;

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

            // wait for the machine to go down
            loop {
                if !timeout_ssh(host, &["exit", "0"], false)?.success() {
                    break;
                }
                if !matches!(
                    ctrlc_rx.recv_timeout(Duration::from_millis(500)),
                    Err(RecvTimeoutError::Timeout)
                ) {
                    break;
                }
            }

            // remove potential old ssh keys before adding new ones...
            let _ = Command::new("ssh-keygen")
                .args(["-R", &host.ssh_hostname])
                .status()
                .context("Failed to run ssh-keygen to remove old keys...")?;

            // Wait for the machine to come back and learn add it's ssh key to our host
            loop {
                if timeout_ssh(host, &["exit", "0"], true)?.success() {
                    break;
                }
                if !matches!(
                    ctrlc_rx.recv_timeout(Duration::from_millis(500)),
                    Err(RecvTimeoutError::Timeout)
                ) {
                    break;
                }
            }

            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
