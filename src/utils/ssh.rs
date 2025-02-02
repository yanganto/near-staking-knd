//! utils for execution via ssh
use crate::deploy::command::status_to_pretty_err;
use crate::deploy::Host;
use anyhow::{Context, Result};
use std::process::{Command, Output};

/// set up ssh connection to a host
pub fn ssh(hosts: &[Host], command: &[&str]) -> Result<()> {
    for host in hosts {
        let target = host.deploy_ssh_target();
        let mut args = vec![];
        args.push(target.as_str());
        args.push("--");
        args.extend(command);
        let status = std::process::Command::new("ssh").args(&args).status();
        status_to_pretty_err(status, "ssh", &args)?;
    }
    Ok(())
}

/// execute remote ssh
pub fn ssh_with_timeout(
    host: &Host,
    command: &[&str],
    learn_known_host_key: bool,
    verbose: bool,
) -> Result<Output> {
    let target = host.deploy_ssh_target();
    let mut args = vec!["-o", "ConnectTimeout=10", "-o", "StrictHostKeyChecking=no"];
    if !learn_known_host_key {
        args.push("-o");
        args.push("UserKnownHostsFile=/dev/null");
    }
    args.push(&target);
    args.push("--");
    args.extend(command);
    if verbose {
        println!("$ ssh {}", args.join(" "));
    }
    let output = Command::new("ssh")
        .args(args)
        .output()
        .context("Failed to run ssh...")?;
    Ok(output)
}

/// execute remote ssh async
pub async fn ssh_with_timeout_async(
    host: &Host,
    mut command: Vec<String>,
    learn_known_host_key: bool,
    verbose: bool,
) -> Result<Output> {
    let target = host.deploy_ssh_target();
    let mut args = vec![
        "-o".to_string(),
        "ConnectTimeout=10".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=no".to_string(),
    ];
    if !learn_known_host_key {
        args.push("-o".to_string());
        args.push("UserKnownHostsFile=/dev/null".to_string());
    }
    args.push(target);
    args.push("--".to_string());
    args.append(&mut command);
    if verbose {
        println!("$ ssh {}", args.join(" "));
    }
    let output = tokio::process::Command::new("ssh")
        .args(args)
        .output()
        .await?;
    Ok(output)
}
