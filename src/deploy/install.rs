use anyhow::{bail, Context, Result};
use log::info;
use std::{path::PathBuf, process::Command};

use crate::deploy::secrets::Secrets;

use super::{Host, NixosFlake};

/// Install a Validator on a given machine
pub fn install(hosts: &[Host], kexec_url: &str, flake: &NixosFlake) -> Result<()> {
    hosts
        .iter()
        .map(|host| {
            info!("Deploying {}", host.name);
            let connection_string = if host.install_ssh_user.is_empty() {
                host.ssh_hostname.clone()
            } else {
                format!("{}@{}", host.install_ssh_user, host.ssh_hostname)
            };

            let mut secret_files = vec![];
            let validator_key: Option<PathBuf>;
            let node_key: Option<PathBuf>;
            if let Some(keys) = &host.validator_keys {
                validator_key = Some(PathBuf::from("/var/lib/secrets/validator_key.json"));
                node_key = Some(PathBuf::from("/var/lib/secrets/node_key.json"));
                secret_files.push((
                    validator_key.as_ref().unwrap().as_path(),
                    keys.key_file.as_path(),
                ));
                secret_files.push((
                    node_key.as_ref().unwrap().as_path(),
                    keys.node_key_file.as_path(),
                ));
            }
            let secrets =
                Secrets::new(secret_files.iter()).context("failed to prepare uploading secrets")?;
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
            let status = status.with_context(|| {
                format!("nixos-remote failed (nixos-remote {})", args.join(" "))
            })?;
            if !status.success() {
                match status.code() {
                    Some(code) => bail!(
                        "nixos-remote failed (nixos-remote {}) with exit code: {}",
                        args.join(" "),
                        code
                    ),
                    None => bail!(
                        "nixos-remote (nixos-remote {}) was terminated by a signal",
                        args.join(" ")
                    ),
                }
            }
            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}
