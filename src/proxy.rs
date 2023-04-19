//! Proxy services of validator
//!

use crate::deploy::Host;
use crate::utils::ssh::ssh_with_timeout_async;
use crate::utils::version::require_async;
use anyhow::{bail, Context, Result};
use std::process::{Command, Output};

async fn proxy(host: &Host, local_port: u16) -> Result<()> {
    let address = host.ipv4_address;
    println!(
        "The Near RPC api of {:} is now reachable at http://localhost:{:}",
        host.name, local_port
    );
    println!("Press Ctrl-C to close it");
    // always use root for proxy, because this is the only user in our image
    let _ = Command::new("ssh")
        .args([
            "-L",
            &format!("{local_port:}:localhost:3030"),
            &format!("root@{address:}"),
            "-N",
        ])
        .status()
        .context("Failed to setup ssh tunnel")?;
    Ok(())
}

/// Proxy RPC service
pub async fn rpc(host: &Host, local_port: u16) -> Result<()> {
    match require_async(host, ">=0.2").await? {
        (true, _) => {
            tokio::select! {
                _ = ssh_with_timeout_async(host, vec!["kuutamoctl".into(), "check-rpc".into(), "--watch".into()], true, true) => println!("rpc service of neard is not running, cannot proxy rpc service. Check `systemctl status kuutamod` on the server for more details."),
                _ = proxy(host, local_port) => (),
            }
        }
        (false, version) => {
            // check on kuutamod if there is no check-rpc command
            println!("{version:} version is not supported for check rpc service status, so we check on kuutamod status before proxy");
            let Output { status, .. } = ssh_with_timeout_async(
                host,
                vec!["systemctl".into(), "is-active".into(), "kuutamod".into()],
                true,
                true,
            )
            .await
            .context("Failed to fetch kuutamod status")?;
            if status.success() {
                proxy(host, local_port).await?;
            } else {
                bail!("kuutamod is not running, cannot proxy rpc service. Check `systemctl status kuutamod` on the server for more details.")
            }
        }
    }
    Ok(())
}
