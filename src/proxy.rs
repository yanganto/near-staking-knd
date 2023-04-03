//! Proxy services of validator
//!

use crate::deploy::Host;
use crate::utils::ssh::async_timeout_ssh;
use anyhow::{Context, Result};
use std::process::Command;

async fn proxy(host: &Host, local_port: u16) -> Result<()> {
    let address = host.ipv4_address;
    let user = &host.install_ssh_user;
    println!(
        "The Near RPC api of {:} is now reachable at http://localhost:{:}",
        host.name, local_port
    );
    println!("Press Ctrl-C to close it");
    let _ = Command::new("ssh")
        .args([
            "-L",
            &format!("{local_port:}:localhost:3030"),
            &format!("{user:}@{address:}"),
            "-N",
        ])
        .status()
        .context("Failed to setup ssh tunnel")?;
    Ok(())
}

/// Proxy RPC service
pub async fn rpc(host: &Host, local_port: u16) -> Result<()> {
    tokio::select! {
        _ = async_timeout_ssh(host, vec!["kuutamoctl".into(), "check-rpc".into(), "--watch".into()], true) => println!("Could not proxy, because neard does not provide rpc service now."),
        _ = proxy(host, local_port) => (),
    }
    Ok(())
}
