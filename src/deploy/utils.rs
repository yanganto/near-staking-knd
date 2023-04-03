use crate::utils::ssh::{async_timeout_ssh, timeout_ssh};
///! utils for deploy and control remote machines
use anyhow::Result;
use std::process::Output;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

use super::Host;

async fn watch_maintenance_status(host: &Host, flag: &AtomicBool) {
    while flag.load(Ordering::Relaxed) {
        sleep(Duration::from_secs(1)).await;
        // TODO:
        // use kuutamoctl (v0.1.0) for backward compatible, change to "kneard-ctl" after (v0.2.1)
        if let Ok(output) = timeout_ssh(host, &["kuutamoctl", "maintenance-status"], true) {
            let _ = tokio::io::stdout().write_all(&output.stdout).await;
        }
    }
}

/// Keep printing maintenance status before maintenance shutdown
pub async fn handle_maintenance_shutdown(host: &Host, required_time_in_blocks: u64) -> Result<()> {
    let flag = AtomicBool::new(true);

    tokio::select! {
        _ = watch_maintenance_status(host, &flag) => (),
        r = async_timeout_ssh(
            host,
            vec![
                // TODO:
                // use kuutamoctl (v0.1.0) for backward compatible, change to "kneard-ctl" after (v0.2.1)
                "kuutamoctl".into(),
                "maintenance-shutdown".into(),
                "--wait".to_string(),
                required_time_in_blocks.to_string(),
            ],
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
                anyhow::bail!("could not execute maintenance shutdown")
            }
        }
    }
    Ok(())
}
