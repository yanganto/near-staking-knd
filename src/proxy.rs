//! Proxy services of validator
//!

use crate::deploy::Host;
use crate::utils::ssh::async_timeout_ssh;
use anyhow::{anyhow, bail, Context, Result};
use semver::{Version, VersionReq};
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
    let Output { stdout, .. } =
        async_timeout_ssh(host, vec!["kuutamoctl".into(), "-V".into()], true)
            .await
            .context("Failed to fetch kuutamoctl version")?;
    let version_str =
        std::str::from_utf8(&stdout).map(|s| s.rsplit_once(' ').map(|(_, v)| v.trim()))?;
    let version =
        Version::parse(version_str.ok_or(anyhow!("version is not prefix with binary name"))?)
            .context("Failed to parse kuutamoctl version")?;

    if VersionReq::parse(">=0.2")?.matches(&version) {
        tokio::select! {
            _ = async_timeout_ssh(host, vec!["kuutamoctl".into(), "check-rpc".into(), "--watch".into()], true) => println!("Could not proxy, because neard does not provide rpc service now."),
            _ = proxy(host, local_port) => (),
        }
    } else {
        // check on kuutamod if there is no check-rpc command
        println!("{:} version is not supported for check rpc service status, so we check on kuutamod status before proxy", version_str.unwrap_or("Current"));
        let Output { status, .. } = async_timeout_ssh(
            host,
            vec!["systemctl".into(), "is-active".into(), "kuutamod".into()],
            true,
        )
        .await
        .context("Failed to fetch kuutamod status")?;
        if status.success() {
            proxy(host, local_port).await?;
        } else {
            bail!("kuutamod is not running, cannot proxy rpc service. Check `systemctl status kuutamod` on the server for more details")
        }
    }
    Ok(())
}
