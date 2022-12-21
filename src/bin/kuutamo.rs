//! kuutamoctl - a cli for kuutamod

#![deny(missing_docs)]

use anyhow::{bail, Context, Result};
use clap::Parser;
use kuutamod::deploy::{self, generate_nixos_flake, Config, Host, NixosFlake};
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::path::PathBuf;

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct InstallArgs {
    /// Comma-separated lists of hosts to perform the install
    #[clap(long, default_value = "")]
    hosts: String,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct DryUpdateArgs {
    /// Comma-separated lists of hosts to perform the dry-update
    #[clap(long, default_value = "")]
    hosts: String,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct UpdateArgs {
    /// Comma-separated lists of hosts to perform the update
    #[clap(long, default_value = "")]
    hosts: String,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct RollbackArgs {
    /// Comma-separated lists of hosts to perform the rollback
    #[clap(long, default_value = "")]
    hosts: String,
}

/// Subcommand to run
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(clap::Subcommand, PartialEq, Debug, Clone)]
enum Command {
    /// Install Validator on a given machine. This will remove all data of the current system!
    Install(InstallArgs),
    /// Upload update to host and show which actions would be performed on an update
    DryUpdate(DryUpdateArgs),
    /// Update validator
    Update(UpdateArgs),
    /// Rollback validator
    Rollback(RollbackArgs),
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// configuration file to load
    #[clap(long, default_value = "kuutamo.toml", env = "KUUTAMO_CONFIG")]
    config: PathBuf,

    /// skip interactive dialogs by assuming the answer is yes
    #[clap(long, default_value = "false")]
    yes: bool,

    #[clap(subcommand)]
    action: Command,
}

fn ask_yes_no(prompt_text: &str) -> bool {
    print!("{} ", prompt_text);
    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return false;
    }
    let normalized = line.trim_end_matches('\n').to_string().to_ascii_lowercase();

    matches!(normalized.as_str(), "y" | "yes")
}

fn filter_hosts(host_spec: &str, hosts: &HashMap<String, Host>) -> Result<Vec<Host>> {
    let mut filtered = vec![];
    for name in host_spec.split(',') {
        match hosts.get(name) {
            Some(v) => {
                filtered.push(v.clone());
            }
            None => {
                bail!("no host named '{}' found in configuration", name)
            }
        }
    }
    Ok(filtered)
}

fn install(
    args: &Args,
    install_args: &InstallArgs,
    config: &Config,
    _flake: &NixosFlake,
) -> Result<()> {
    if !args.yes && !ask_yes_no(
            "Installing will remove any existing data from the configured hosts. Do you want to continue? (y/n)"
        ) {
        return Ok(());
    }
    let hosts = filter_hosts(&install_args.hosts, &config.hosts)?;
    deploy::install(&hosts)
}

fn dry_update(
    _args: &Args,
    dry_update_args: &DryUpdateArgs,
    config: &Config,
    _flake: &NixosFlake,
) -> Result<()> {
    let hosts = filter_hosts(&dry_update_args.hosts, &config.hosts)?;
    deploy::dry_update(&hosts)
}

fn update(
    _args: &Args,
    update_args: &UpdateArgs,
    config: &Config,
    _flake: &NixosFlake,
) -> Result<()> {
    let hosts = filter_hosts(&update_args.hosts, &config.hosts)?;
    deploy::update(&hosts)
}

fn rollback(
    _args: &Args,
    rollback_args: &RollbackArgs,
    config: &Config,
    _flake: &NixosFlake,
) -> Result<()> {
    let hosts = filter_hosts(&rollback_args.hosts, &config.hosts)?;
    deploy::rollback(&hosts)
}

fn run_deploy() -> Result<()> {
    let args = Args::parse();
    let config = deploy::load_configuration(&args.config).with_context(|| {
        format!(
            "failed to parse configuration file: {}",
            &args.config.display()
        )
    })?;
    let flake = generate_nixos_flake(&config).context("failed to generate flake")?;

    match args.action {
        Command::Install(ref install_args) => install(&args, install_args, &config, &flake),
        Command::DryUpdate(ref dry_update_args) => {
            dry_update(&args, dry_update_args, &config, &flake)
        }
        Command::Update(ref update_args) => update(&args, update_args, &config, &flake),
        Command::Rollback(ref rollback_args) => rollback(&args, rollback_args, &config, &flake),
    }
}

/// The kuutamo program entry point
pub fn main() {
    let res = run_deploy();
    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
