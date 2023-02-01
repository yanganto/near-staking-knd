//! Proxy services of validator
//!

use crate::deploy::Host;
use anyhow::{Context, Result};
use std::process::Command;

/// Proxy RPC service
pub fn rpc(host: &Host, local_port: u16) -> Result<()> {
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
