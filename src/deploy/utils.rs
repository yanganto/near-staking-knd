use crate::utils::ssh::{ssh_with_timeout, ssh_with_timeout_async};
///! utils for deploy and control remote machines
use anyhow::{anyhow, Context, Result};
use semver::{Version, VersionReq};
use std::process::Output;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

use super::Host;

async fn watch_maintenance_status(host: &Host, flag: &AtomicBool) {
    while flag.load(Ordering::Relaxed) {
        sleep(Duration::from_secs(1)).await;
        if let Ok(output) =
            ssh_with_timeout(host, &["kuutamoctl", "maintenance-status"], true, true)
        {
            let _ = tokio::io::stdout().write_all(&output.stdout).await;
            let _ = tokio::io::stdout().write_all(&output.stderr).await;
        }
    }
}

/// Keep printing maintenance status before maintenance shutdown
pub async fn handle_maintenance_shutdown(host: &Host, required_time_in_blocks: u64) -> Result<()> {
    let flag = AtomicBool::new(true);

    let Output { stdout, .. } = ssh_with_timeout(host, &["kuutamoctl", "-V"], true, true)
        .context("Failed to fetch kuutamoctl version")?;
    let version_str =
        std::str::from_utf8(&stdout).map(|s| s.rsplit_once(' ').map(|(_, v)| v.trim()))?;
    let version =
        Version::parse(version_str.ok_or(anyhow!("version is not prefix with binary name"))?)
            .context("Failed to parse kuutamoctl version")?;

    if VersionReq::parse(">=0.2.0")?.matches(&version) {
        tokio::select! {
            _ = watch_maintenance_status(host, &flag) => (),
            r = ssh_with_timeout_async(
                host,
                vec![
                    // TODO:
                    // use kuutamoctl (v0.1.0) for backward compatible
                    "kuutamoctl".into(),
                    "restart".into(),
                    "--wait".to_string(),
                    required_time_in_blocks.to_string(),
                ],
                true,
                true,
            ) => {
                flag.store(false, Ordering::Relaxed);
                let Output{stdout, stderr, status} = r?;
                let _ = tokio::io::stdout().write_all(&stdout).await;
                if status.success() {
                    println!("restart complete");
                }
                else {
                    let _ = tokio::io::stderr().write_all(&stderr).await;
                    anyhow::bail!("Could not find a suitable maintenance window, please try again later")
                }
            }
        }
    } else {
        tokio::select! {
            _ = watch_maintenance_status(host, &flag) => (),
            r = ssh_with_timeout_async(
                host,
                vec![
                    // TODO:
                    // use kuutamoctl (v0.1.0) for backward compatible
                    "kuutamoctl".into(),
                    "maintenance-shutdown".into(),
                    "--wait".to_string(),
                    required_time_in_blocks.to_string(),
                ],
                true,
                true,
            ) => {
                flag.store(false, Ordering::Relaxed);
                let Output{stdout, stderr, status} = r?;
                let _ = tokio::io::stdout().write_all(&stdout).await;
                if status.success() {
                    println!("maintenance shutdown complete");
                }
                else {
                    let _ = tokio::io::stderr().write_all(&stderr).await;
                    anyhow::bail!("Could not find a suitable maintenance window, please try again later")
                }
            }
        }
    };

    Ok(())
}
