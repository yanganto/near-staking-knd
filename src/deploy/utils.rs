///! utils for deploy and control remote machines
use anyhow::{Context, Result};
use std::process::{Command, ExitStatus};

use super::Host;

/// execute remote ssh
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
