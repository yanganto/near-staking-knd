use anyhow::{Context, Result};
use ctrlc;
use lazy_static::lazy_static;
use log::info;
use std::sync::Mutex;
use std::{
    process::Command,
    sync::mpsc::{channel, Receiver, RecvTimeoutError},
    time::Duration,
};

use crate::deploy::command::status_to_pretty_err;
use crate::utils::ssh::ssh_with_timeout;

use super::{Host, NixosFlake};

lazy_static! {
    static ref CTRL_WAS_PRESSED: Mutex<Receiver<()>> = {
        let (ctrlc_tx, ctrlc_rx) = channel();
        ctrlc::set_handler(move || {
            info!("received ctrl-c!. Stopping program...");
            let _ = ctrlc_tx.send(());
        })
        .expect("Error setting ctrl-C handler");
        Mutex::new(ctrlc_rx)
    };
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
            if cfg!(target_os = "macos") {
                args.push("--build-on-remote");
            }
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

            // remove potential old ssh keys before adding new ones...
            let _ = Command::new("ssh-keygen")
                .args(["-R", &host.ssh_hostname])
                .status()
                .context("Failed to run ssh-keygen to remove old keys...")?;

            // Wait for the machine to come back and learn add it's ssh key to our host
            loop {
                if ssh_with_timeout(host, &["exit", "0"], true, true)?
                    .status
                    .success()
                {
                    break;
                }
                if let Ok(chan) = CTRL_WAS_PRESSED.lock() {
                    if !matches!(
                        chan.recv_timeout(Duration::from_millis(500)),
                        Err(RecvTimeoutError::Timeout)
                    ) {
                        break;
                    }
                }
            }

            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
