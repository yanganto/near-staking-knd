///! utils for deploy and control remote machines
use anyhow::{Context, Result};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::stdout;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

use super::Host;

/// execute remote ssh
pub fn timeout_ssh(host: &Host, command: &[&str], learn_known_host_key: bool) -> Result<Output> {
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
    let output = Command::new("ssh")
        .args(args)
        .output()
        .context("Failed to run ssh...")?;
    Ok(output)
}

async fn async_timeout_ssh(
    host: &Host,
    mut command: Vec<String>,
    learn_known_host_key: bool,
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
    println!("$ ssh {}", args.join(" "));
    let output = tokio::process::Command::new("ssh")
        .args(args)
        .output()
        .await?;
    Ok(output)
}

async fn watch_maintenance_status(host: &Host, flag: &AtomicBool) {
    while flag.load(Ordering::Relaxed) {
        sleep(Duration::from_secs(1)).await;
        if let Ok(output) = timeout_ssh(host, &["kuutamoctl", "maintenance-status"], true) {
            let _ = stdout().write_all(&output.stdout).await;
        }
    }
}

/// Keep printing maintenance status before maintenance shutdown
pub async fn handle_maintenance_shutdown(
    host: &Host,
    required_time_in_blocks: Option<u64>,
) -> Result<()> {
    let flag = AtomicBool::new(true);
    let shutdown_cmd = if let Some(blocks) = required_time_in_blocks {
        async_timeout_ssh(
            host,
            vec![
                "kuutamoctl".into(),
                "maintenance-shutdown".into(),
                blocks.to_string(),
            ],
            true,
        )
    } else {
        async_timeout_ssh(
            host,
            vec!["kuutamoctl".into(), "maintenance-shutdown".into()],
            true,
        )
    };

    tokio::select! {
        _ = watch_maintenance_status(host, &flag) => (),
        r = shutdown_cmd => {
            flag.store(false, Ordering::Relaxed);
            let _ = stdout().write_all(&r?.stdout).await;
        }
    }
    Ok(())
}
