//! kuutamoctl - a cli for kuutamod

#![deny(missing_docs)]

use anyhow::{Context, Result};
use clap::Parser;
use kuutamod::commands::CommandClient;
use serde_json::to_string_pretty;
use std::path::PathBuf;

/// Subcommand to run
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(clap::Subcommand, PartialEq, Debug, Clone)]
pub enum Command {
    /// Initiate maintenance shutdown
    MaintenanceShutdown {
        /// Specify the minimum length in blockheight for the maintenance shutdown
        minimum_length: Option<u64>,
    },
    /// Show the current voted validator
    ActiveValidator,
}

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
            to_string_pretty(&validator).context("Failed to serialize json")?
        );
    } else {
        match validator {
            Some(v) => println!("Name: {}\nId: {}", v.name, v.id),
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
        Command::MaintenanceShutdown { minimum_length } => {
            kuutamo_client.maintenance_shutdown(minimum_length).await
        }
    };
    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
