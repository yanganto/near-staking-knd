//! kuutamoctl - a cli for kuutamod

#![deny(missing_docs)]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use kuutamod::commands::control_commands::{Command, MaintenanceShutdownArgs};
use kuutamod::commands::CommandClient;
use std::path::PathBuf;

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

/// The kuutamoctl program entry point
#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    let kuutamo_client = CommandClient::new(&args.control_socket);
    let res = match args.action {
        Command::ActiveValidator => show_active_validator(&kuutamo_client, &args).await,
        Command::MaintenanceShutdown(MaintenanceShutdownArgs {
            minimum_length,
            shutdown_at,
            cancel,
            ..
        }) => {
            if minimum_length.is_some() && shutdown_at.is_some() {
                Err(anyhow!("We can not guarantee minimum maintenance window for a specified shutdown block height"))
            } else {
                kuutamo_client
                    .maintenance_shutdown(minimum_length, shutdown_at, cancel)
                    .await
            }
        }
    };
    if let Err(e) = res {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
