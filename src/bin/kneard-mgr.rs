//! kneard-ctl - a cli for kneard

#![deny(missing_docs)]

use crate::utils::ssh::ssh_with_timeout;
use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use kneard::deploy::{self, generate_nixos_flake, Config, Host, NixosFlake};
use kneard::proxy;
use kneard::utils;
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Output;

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct InstallArgs {
    /// Comma-separated lists of hosts to perform the install
    #[clap(long, default_value = "")]
    hosts: String,

    /// Kexec-tarball url to install from
    #[clap(
        long,
        default_value = "https://github.com/nix-community/nixos-images/releases/download/nixos-22.11/nixos-kexec-installer-x86_64-linux.tar.gz"
    )]
    kexec_url: String,

    /// Enables debug output in nixos-anywhere
    #[clap(long, action)]
    debug: bool,

    /// Do not reboot after installation
    #[clap(long, action)]
    no_reboot: bool,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct GenerateConfigArgs {
    /// Directory where to copy the configuration to.
    directory: PathBuf,
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

    /// Immediately update without finding maintenance windows
    #[clap(long)]
    immediately: bool,

    /// If not immediately, please specify time in blocks to update, it takes 1~2 seconds for a near block.
    /// For active-passive pairs, the time needs to cover switching nodes.
    /// For single nodes, the time need to cover copy binaries.
    /// If 0 or not provided, kneard will try to update in the longest maintenance window in the current epoch,
    /// but it can not guarantee the  maintenance window is enough.
    #[clap(default_value = "0")]
    required_time_in_blocks: u64,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct RollbackArgs {
    /// Comma-separated lists of hosts to perform the rollback
    #[clap(long, default_value = "")]
    hosts: String,

    /// Immediately update without finding maintenance windows
    #[clap(long)]
    immediately: bool,

    /// If not immediately, please specify time in blocks to rollback, it takes 1~2 seconds for a near block.
    /// For active-passive pairs, the time needs to cover switching nodes.
    /// For single nodes, the time need to cover copy binaries.
    /// If 0 or not provided, kneard will try to rollback in the longest maintenance window in the current epoch,
    /// but it can not guarantee the  maintenance window is enough.
    #[clap(default_value = "0")]
    required_time_in_blocks: u64,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct ProxyArgs {
    /// The host to proxy rpc
    #[clap(long, default_value = "")]
    host: String,

    /// Proxy to local port
    #[clap(long, action, default_value = "3030")]
    local_port: u16,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct SshArgs {
    /// Host to ssh into
    #[clap(long, default_value = "")]
    hosts: String,

    /// Additional arguments to pass to ssh
    command: Option<Vec<String>>,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
struct RestartArgs {
    /// Comma-separated lists of hosts to perform the restart
    #[clap(long, default_value = "")]
    hosts: String,

    /// Specify the minimum length in blocks of maintenance window to restart at, if not provided,
    /// kneard will try to pick the longest maintenance window in the current epoch.
    pub minimum_length: Option<u64>,

    /// Specify the block height to restart at, and will not check on it in maintenance window or
    /// not.
    #[arg(long)]
    pub restart_at: Option<u64>,

    /// Gracefully restart immediately
    #[arg(long)]
    pub immediately: bool,

    /// Cancel the restart setting
    #[arg(long)]
    pub cancel: bool,

    /// Cli will wait for restart
    #[clap(long)]
    pub wait: bool,
}

/// Subcommand to run
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(clap::Subcommand, PartialEq, Debug, Clone)]
enum Command {
    /// Generate NixOS configuration
    GenerateConfig(GenerateConfigArgs),
    /// Install Validator on a given machine. This will remove all data of the current system!
    Install(InstallArgs),
    /// Upload update to host and show which actions would be performed on an update
    DryUpdate(DryUpdateArgs),
    /// Update validator
    Update(UpdateArgs),
    /// Rollback validator
    Rollback(RollbackArgs),
    /// Proxy remote rpc to local
    Proxy(ProxyArgs),
    /// Schedule a restart in a window where no blocks or chunks are expected to be produced by the validator
    Restart(RestartArgs),
    /// SSH into a host
    Ssh(SshArgs),
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// configuration file to load
    #[clap(long, default_value = "kneard.toml", env = "KUUTAMO_CONFIG")]
    config: PathBuf,

    /// skip interactive dialogs by assuming the answer is yes
    #[clap(long, default_value = "false")]
    yes: bool,

    #[clap(subcommand)]
    action: Command,
}

fn ask_yes_no(prompt_text: &str) -> bool {
    println!("{prompt_text} ");
    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return false;
    }
    let normalized = line.trim_end_matches('\n').to_string().to_ascii_lowercase();

    matches!(normalized.as_str(), "y" | "yes")
}

fn filter_hosts(host_spec: &str, hosts: &HashMap<String, Host>) -> Result<Vec<Host>> {
    if host_spec.is_empty() {
        return Ok(hosts.values().map(Clone::clone).collect::<Vec<_>>());
    }
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
    flake: &NixosFlake,
) -> Result<()> {
    let hosts = filter_hosts(&install_args.hosts, &config.hosts)?;
    if !args.yes && !ask_yes_no(
            "Installing will remove any existing data from the configured hosts. Do you want to continue? (y/n)"
        ) {
        return Ok(());
    }
    deploy::install(
        &hosts,
        &install_args.kexec_url,
        flake,
        install_args.debug,
        install_args.no_reboot,
    )
}
fn generate_config(
    _args: &Args,
    config_args: &GenerateConfigArgs,
    _config: &Config,
    flake: &NixosFlake,
) -> Result<()> {
    deploy::generate_config(&config_args.directory, flake)
}

fn dry_update(
    _args: &Args,
    dry_update_args: &DryUpdateArgs,
    config: &Config,
    flake: &NixosFlake,
) -> Result<()> {
    let hosts = filter_hosts(&dry_update_args.hosts, &config.hosts)?;
    deploy::dry_update(&hosts, flake)
}

async fn update(
    _args: &Args,
    update_args: &UpdateArgs,
    config: &Config,
    flake: &NixosFlake,
) -> Result<()> {
    let hosts = filter_hosts(&update_args.hosts, &config.hosts)?;
    deploy::update(
        &hosts,
        flake,
        update_args.immediately,
        update_args.required_time_in_blocks,
    )
    .await
}

async fn rollback(
    _args: &Args,
    rollback_args: &RollbackArgs,
    config: &Config,
    flake: &NixosFlake,
) -> Result<()> {
    let hosts = filter_hosts(&rollback_args.hosts, &config.hosts)?;
    deploy::rollback(
        &hosts,
        flake,
        rollback_args.immediately,
        rollback_args.required_time_in_blocks,
    )
    .await
}

async fn proxy(proxy_args: &ProxyArgs, config: &Config) -> Result<()> {
    let hosts = filter_hosts(&proxy_args.host, &config.hosts)?;
    if hosts.len() > 1 {
        println!(
            "Multiple hosts detected in configuration. Will proxy to machine: {}",
            hosts[0].name
        );
    }
    proxy::rpc(&hosts[0], proxy_args.local_port).await
}

/// For v0.1
fn maintenance_shutdown(
    host: &Host,
    minimum_length: Option<u64>,
    schedule_at: Option<u64>,
) -> Result<Output> {
    match (minimum_length, schedule_at) {
        (Some(_), Some(_)) => bail!(
            "We can not guarantee minimum maintenance window for a specified shutdown block height"
        ),
        (Some(minimum_length), None) => utils::ssh::ssh_with_timeout(
            host,
            &[
                "kuutamoctl",
                "maintenance-shutdown",
                &minimum_length.to_string(),
            ],
            true,
        ),
        (None, None) => {
            utils::ssh::ssh_with_timeout(host, &["kuutamoctl", "maintenance-shutdown"], true)
        }
        (None, Some(schedule_at)) => utils::ssh::ssh_with_timeout(
            host,
            &[
                "kuutamoctl",
                "maintenance-shutdown",
                "--shutdown-at",
                &schedule_at.to_string(),
            ],
            true,
        ),
    }
}

/// For v0.2.1
fn schedule_restart(
    host: &Host,
    minimum_length: Option<u64>,
    schedule_at: Option<u64>,
) -> Result<Output> {
    match (minimum_length, schedule_at) {
        (Some(_), Some(_)) => bail!(
            "We can not guarantee minimum maintenance window for a specified shutdown block height"
        ),
        (Some(minimum_length), None) => utils::ssh::ssh_with_timeout(
            host,
            &["kuutamoctl", "restart", &minimum_length.to_string()],
            true,
        ),
        (None, None) => utils::ssh::ssh_with_timeout(host, &["kuutamoctl", "restart"], true),
        (None, Some(schedule_at)) => utils::ssh::ssh_with_timeout(
            host,
            &[
                "kuutamoctl",
                "restart",
                "--schedule-at",
                &schedule_at.to_string(),
            ],
            true,
        ),
    }
}

fn restart(args: &RestartArgs, config: &Config) -> Result<()> {
    let schedule_at = if args.immediately {
        // scheule at a past block height to gracefully restart immediately
        Some(1)
    } else {
        args.restart_at
    };

    let hosts = filter_hosts(&args.hosts, &config.hosts)?;

    for host in hosts.iter() {
        let Output { stdout, .. } = ssh_with_timeout(host, &["kuutamoctl", "-V"], true)
            .context("Failed to fetch kuutamoctl version")?;
        let version_str =
            std::str::from_utf8(&stdout).map(|s| s.rsplit_once(' ').map(|(_, v)| v.trim()))?;
        let version =
            Version::parse(version_str.ok_or(anyhow!("version is not prefix with binary name"))?)
                .context("Failed to parse kuutamoctl version")?;

        let output = if VersionReq::parse(">=0.2.1")?.matches(&version) {
            schedule_restart(host, args.minimum_length, schedule_at)?
        } else {
            maintenance_shutdown(host, args.minimum_length, schedule_at)?
        };

        io::stdout()
            .write_all(&output.stdout)
            .with_context(|| "Fail to dump stdout of kuutamctl")?;
        if output.status.success() {
            println!("{} restart", host.name);
        } else {
            io::stdout()
                .write_all(&output.stderr)
                .with_context(|| "Fail to dump stderr of kuutamctl")?;
            bail!("Fail to trigger restart");
        }
    }
    Ok(())
}

fn ssh(_args: &Args, ssh_args: &SshArgs, config: &Config) -> Result<()> {
    let hosts = filter_hosts(&ssh_args.hosts, &config.hosts)?;
    let command = ssh_args
        .command
        .as_ref()
        .map_or_else(|| [].as_slice(), |v| v.as_slice());
    let command = command.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    kneard::utils::ssh::ssh(&hosts, command.as_slice())
}

/// The kuutamo program entry point
#[tokio::main]
pub async fn main() -> Result<()> {
    let mut args = Args::parse();

    if args.config.to_str() == Some("kneard.toml")
        && !Path::new("kneard.toml").exists()
        && Path::new("kuutamo.toml").exists()
    {
        println!("`kuutamo.toml` is deprecated, please switch to `kneard.toml`");
        args.config = "kuutamo.toml".into();
    }

    let res = match args.action {
        Command::GenerateConfig(_)
        | Command::Install(_)
        | Command::DryUpdate(_)
        | Command::Update(_)
        | Command::Rollback(_) => {
            let config = deploy::load_configuration(&args.config, true).with_context(|| {
                format!(
                    "failed to parse configuration file: {}",
                    &args.config.display()
                )
            })?;
            let flake = generate_nixos_flake(&config).context("failed to generate flake")?;
            match args.action {
                Command::GenerateConfig(ref config_args) => {
                    generate_config(&args, config_args, &config, &flake)
                }
                Command::Install(ref install_args) => install(&args, install_args, &config, &flake),
                Command::DryUpdate(ref dry_update_args) => {
                    dry_update(&args, dry_update_args, &config, &flake)
                }
                Command::Update(ref update_args) => {
                    update(&args, update_args, &config, &flake).await
                }
                Command::Rollback(ref rollback_args) => {
                    rollback(&args, rollback_args, &config, &flake).await
                }
                _ => unreachable!(),
            }
        }
        Command::Proxy(_) | Command::Restart(_) | Command::Ssh(_) => {
            let config = deploy::load_configuration(&args.config, false).with_context(|| {
                format!(
                    "failed to load configuration file: {}",
                    &args.config.display()
                )
            })?;
            match args.action {
                Command::Proxy(ref proxy_args) => proxy(proxy_args, &config).await,
                Command::Ssh(ref ssh_args) => ssh(&args, ssh_args, &config),
                Command::Restart(ref args) => restart(args, &config),
                _ => unreachable!(),
            }
        }
    };
    res.with_context(|| format!("kuutamo failed doing {:?}", args.action))
}
