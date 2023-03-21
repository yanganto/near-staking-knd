//! kuutamoctl - a cli for kneard

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use kneard::commands::control_commands::{Command, MaintenanceOperationArgs};
use kneard::commands::CommandClient;
use std::path::PathBuf;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, action, help = "output in json format")]
    json: bool,

    /// Kuutamod control socket to interact with
    #[clap(
        long,
        env = "KUUTAMO_CONTROL_SOCKET",
        default_value = "/var/lib/neard/kneard.sock"
    )]
    pub control_socket: PathBuf,

    #[clap(subcommand)]
    action: Command,
}

async fn show_active_validator(kuutamo_client: &CommandClient, args: &Args) -> Result<()> {
    let validator = kuutamo_client.active_validator().await?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string(&validator).context("Failed to serialize json")?
        );
    } else {
        match validator {
            Some(v) => println!("Name: {}\nNode: {}", v.name, v.node),
            None => println!("No active validator"),
        }
    }
    Ok(())
}

async fn show_maintenance_status(kuutamo_client: &CommandClient) -> Result<()> {
    Ok(println!("{}", &kuutamo_client.maintenance_status().await?))
}

/// The kuutamoctl program entry point
#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    let kuutamo_client = CommandClient::new(&args.control_socket);
    let res = match args.action {
        Command::ActiveValidator => show_active_validator(&kuutamo_client, &args).await,
        Command::MaintenanceShutdown(MaintenanceOperationArgs {
            minimum_length,
            shutdown_at,
            cancel,
            wait,
            ..
        }) => {
            if minimum_length.is_some() && shutdown_at.is_some() {
                Err(anyhow!("We can not guarantee minimum maintenance window for a specified shutdown block height"))
            } else {
                let r = kuutamo_client
                    .maintenance_operation(minimum_length, shutdown_at, cancel, false)
                    .await;
                if r.is_ok() && wait {
                    // Wait for kneard to terminate
                    while UnixStream::connect(&args.control_socket).await.is_ok() {
                        sleep(Duration::from_millis(100)).await
                    }
                }
                r
            }
        }
        Command::MaintenanceRestart(MaintenanceOperationArgs {
            minimum_length,
            shutdown_at,
            cancel,
            wait,
            ..
        }) => {
            if minimum_length.is_some() && shutdown_at.is_some() {
                Err(anyhow!("We can not guarantee minimum maintenance window for a specified shutdown block height"))
            } else {
                let r = kuutamo_client
                    .maintenance_operation(minimum_length, shutdown_at, cancel, true)
                    .await;
                if r.is_ok() && wait {
                    eprintln!("waiting for neard restart is not implemented");
                }
                r
            }
        }
        Command::MaintenanceStatus => show_maintenance_status(&kuutamo_client).await,
    };
    if let Err(e) = res {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
