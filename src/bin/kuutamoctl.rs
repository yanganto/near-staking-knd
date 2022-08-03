//! kuutamoctl - a cli for kuutamod

#![deny(missing_docs)]

use anyhow::{Context, Result};
use clap::Parser;
use kuutamod::{consul_client::ConsulClient, leader_protocol::consul_leader_key};
use serde_json::to_string_pretty;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Consul url to interact with
    #[clap(
        long,
        default_value = "http://localhost:8500",
        env = "KUUTAMO_CONSUL_URL"
    )]
    consul_url: String,

    /// The consul token to authenticate, used for authentication https://www.consul.io/docs/security/acl/acl-tokens
    #[clap(long, env = "KUUTAMO_CONSUL_TOKEN")]
    pub consul_token: Option<String>,

    #[clap(long, action, help = "output in json format")]
    json: bool,

    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Show the current voted validator
    ActiveValidator,
}
const ACCOUNT_ID: &str = "KUUTAMO_ACCOUNT_ID";

async fn show_active_validator(args: &Args) -> Result<i32> {
    let token = args.consul_token.as_deref();
    let client =
        ConsulClient::new(&args.consul_url, token).context("Failed to create consul client")?;

    let account_id = std::env::var(ACCOUNT_ID).unwrap_or_else(|_| "default".to_string());

    let res = client
        .get(&consul_leader_key(&account_id))
        .await
        .context("Failed to get leader key from consul")?;
    let value = match res {
        None => {
            eprintln!("No leader found");
            return Ok(1);
        }
        Some(session) => session,
    };
    let uuid = match value.session {
        None => {
            eprintln!("Last leader session was expired");
            return Ok(2);
        }
        Some(val) => val,
    };
    let res = client
        .get_session(&uuid)
        .await
        .context("Failed to get leader key from consul")?;

    let session = match res {
        None => {
            eprintln!("Last leader session was expired");
            return Ok(2);
        }
        Some(session) => session,
    };

    if args.json {
        println!(
            "{}",
            to_string_pretty(&session).context("Failed to serialize json")?
        );
    } else {
        println!("Name: {}", session.name());
    }

    Ok(0)
}

/// The kuutamoctl program entry point
#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Args::parse();
    let exit_code = match args.action {
        Action::ActiveValidator => show_active_validator(&args).await?,
    };
    std::process::exit(exit_code);
}
