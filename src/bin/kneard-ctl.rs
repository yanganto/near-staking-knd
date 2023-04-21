//! kneard-ctl - a cli for kneard

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use kneard::commands::control_commands::{CheckRpcArgs, Command, MaintenanceOperationArgs};
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
        default_value = "/var/lib/neard/kuutamod.sock"
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

async fn set_maintenance_shutdown(
    kuutamo_client: &CommandClient,
    control_socket: &PathBuf,
    operation_arg: MaintenanceOperationArgs,
) -> Result<()> {
    let MaintenanceOperationArgs {
        minimum_length,
        shutdown_at,
        cancel,
        wait,
        ..
    } = operation_arg;
    if minimum_length.is_some() && shutdown_at.is_some() {
        Err(anyhow!(
            "We can not guarantee minimum maintenance window for a specified shutdown block height"
        ))
    } else {
        let r = kuutamo_client
            .maintenance_operation(minimum_length, shutdown_at, cancel, false)
            .await;
        if r.is_ok() && wait {
            // Wait for kuutamod to terminate
            while UnixStream::connect(&control_socket).await.is_ok() {
                sleep(Duration::from_millis(100)).await
            }
        }
        r
    }
}

async fn set_maintenance_restart(
    kuutamo_client: &CommandClient,
    operation_arg: MaintenanceOperationArgs,
) -> Result<()> {
    let MaintenanceOperationArgs {
        minimum_length,
        shutdown_at,
        cancel,
        wait,
        ..
    } = operation_arg;
    if minimum_length.is_some() && shutdown_at.is_some() {
        Err(anyhow!(
            "We can not guarantee minimum maintenance window for a specified shutdown block height"
        ))
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

async fn show_maintenance_status(kuutamo_client: &CommandClient) -> Result<()> {
    Ok(println!("{}", &kuutamo_client.maintenance_status().await?))
}

async fn check_rpc_status(kuutamo_client: &CommandClient, watch: bool) -> Result<()> {
    if watch {
        while kuutamo_client.rpc_status().await.is_ok() {
            sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    } else {
        Ok(println!("{}", &kuutamo_client.rpc_status().await?))
    }
}

/// The kneard-ctl program entry point
#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    let kuutamo_client = CommandClient::new(&args.control_socket);
    let res = match args.action {
        Command::ActiveValidator => show_active_validator(&kuutamo_client, &args).await,
        Command::MaintenanceShutdown(operation_arg) => {
            set_maintenance_shutdown(&kuutamo_client, &args.control_socket, operation_arg).await
        }
        Command::MaintenanceRestart(operation_arg) => {
            set_maintenance_restart(&kuutamo_client, operation_arg).await
        }
        Command::MaintenanceStatus => show_maintenance_status(&kuutamo_client).await,
        Command::CheckRpc(CheckRpcArgs { watch }) => check_rpc_status(&kuutamo_client, watch).await,
    };
    if let Err(e) = res {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
