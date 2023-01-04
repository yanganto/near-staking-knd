use std::process::Command;

use crate::deploy::command;

use super::{Host, NixosFlake};
use anyhow::{Context, Result};

/// Runs nixos-rebuild on the given host
pub fn nixos_rebuild(
    action: &str,
    host: &Host,
    flake: &NixosFlake,
    collect_garbage: bool,
) -> Result<()> {
    let secrets = host.secrets()?;
    let target = host.deploy_ssh_target();
    secrets
        .upload(&target)
        .context("Failed to upload secrets")?;
    let flake_uri = host.flake_uri(flake);
    let mut args = vec![
        if action == "rollback" {
            "switch"
        } else {
            action
        },
        "--flake",
        &flake_uri,
        "--option",
        "accept-flake-config",
        "true",
        "--target-host",
        &target,
        "--fast",
    ];
    if action == "rollback" {
        args.push("--rollback");
    }
    for i in 1..3 {
        println!("$ nixos-remote {}", &args.join(" "));
        let status = Command::new("nixos-rebuild").args(&args).status();
        if let Err(e) = command::status_to_pretty_err(status, "nixos-rebuild", &args) {
            if i == 1 {
                eprintln!("{}", e);
                eprintln!("Retry...");
            } else {
                return Err(e);
            }
        }
        if collect_garbage {
            let gc_args = ["--delete-older-than", "14d"];
            println!("$ nix-collect-garbage {}", gc_args.join(" "));
            let status = Command::new("nix-collect-garbage").args(gc_args).status();
            if let Err(e) = command::status_to_pretty_err(status, "nix-collect-garbage", &gc_args) {
                eprintln!("garbage collection failed, but continue...: {}", e);
            }
        }
    }
    Ok(())
}
