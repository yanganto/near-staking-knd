//! Read settings for kuutamod

use crate::near_config::read_near_config;
use anyhow::{bail, Context, Result};
use clap::Parser;
use nix::unistd::{access, AccessFlags};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

// set by systemd LoadCredential
const CREDENTIALS_DIRECTORY: &str = "CREDENTIALS_DIRECTORY";

/// Setting options for kuutamod
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Settings {
    /// The consul agent url
    #[clap(
        long,
        default_value = "http://localhost:8500",
        env = "KUUTAMO_CONSUL_URL"
    )]
    pub consul_url: String,
    /// Node id of the kuutamo instance
    #[clap(long, default_value = "node", env = "KUUTAMO_NODE_ID")]
    pub node_id: String,
    /// NEAR Account id of the validator. This ID will be used to acquire
    /// leadership in consul. It should be the same for all nodes that share the
    /// same validator key
    #[clap(long, default_value = "default", env = "KUUTAMO_ACCOUNT_ID")]
    pub account_id: String,
    /// The exporter address, that kuutamod will listen to: format: ip:host
    #[clap(
        long,
        default_value = "127.0.0.1:2233",
        env = "KUUTAMO_EXPORTER_ADDRESS"
    )]
    pub exporter_address: String,
    /// Location where keys and chain data for neard is stored
    #[clap(long, default_value = ".", env = "KUUTAMO_NEARD_HOME")]
    pub neard_home: PathBuf,
    /// RPC address of the neard daemon
    #[clap(skip = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 80))]
    pub near_rpc_addr: SocketAddr,
    /// The neard validator key that we will pass to neard, when kuutamod becomes validator
    #[clap(long, default_value = "", env = "KUUTAMO_VALIDATOR_KEY")]
    pub validator_key: PathBuf,
    /// The neard node key that we will pass to neard, when kuutamod becomes validator
    #[clap(long, default_value = "", env = "KUUTAMO_VALIDATOR_NODE_KEY")]
    pub validator_node_key: PathBuf,
    /// The address neard will listen, when beeing a validator
    #[clap(
        long,
        default_value = "0.0.0.0:24567",
        env = "KUUTAMO_VALIDATOR_NETWORK_ADDR"
    )]
    pub validator_network_addr: SocketAddr,
    /// The neard node key that we will pass to neard, when kuutamod is not a validator
    #[clap(long, default_value = "", env = "KUUTAMO_VOTER_NODE_KEY")]
    pub voter_node_key: PathBuf,
    /// The address neard will listen, when kuutamod is not a validator. At least the port should be different from `validator_network_addr`
    #[clap(
        long,
        default_value = "0.0.0.0:24568",
        env = "KUUTAMO_VOTER_NETWORK_ADDR"
    )]
    pub voter_network_addr: SocketAddr,
    /// Bootnodes passed to neard
    #[clap(long, env = "KUUTAMO_NEARD_BOOTNODES")]
    pub near_boot_nodes: Option<String>,
}

fn get_near_key(key: &str, val: &mut PathBuf, credential_filename: &str) -> Result<()> {
    if val == Path::new("") {
        // Use systemd's LoadCredential environment variable, if it exits:
        // TODO: replace this with KUUTAMO_NEAR_VALIDATOR_FILE=%d/validator_key.json in systemd's Environment after the next systemd upgrade:
        // see: https://www.freedesktop.org/software/systemd/man/systemd.exec.html
        match std::env::var_os(CREDENTIALS_DIRECTORY) {
            Some(v) => {
                *val = PathBuf::from(v).join(credential_filename);
            }
            None => {
                bail!("{} option is not set but required", key);
            }
        }
    };
    access(val, AccessFlags::R_OK | AccessFlags::F_OK)
        .with_context(|| format!("cannot open {} as a file", val.display()))?;

    Ok(())
}

/// Read and returns settings from environment variables and the filesystem
pub fn parse_settings() -> Result<Settings> {
    let mut settings = Settings::parse();

    get_near_key(
        "--validator-key",
        &mut settings.validator_key,
        "validator_key.json",
    )?;
    get_near_key(
        "--validator-node-key",
        &mut settings.validator_node_key,
        "validator_node_key.json",
    )?;
    get_near_key(
        "--voter-node-key",
        &mut settings.voter_node_key,
        "voter_node_key.json",
    )?;

    let config_path = &settings.neard_home.join("config.json");
    let config = read_near_config(config_path).context("failed to parse near config")?;

    settings.near_rpc_addr = config.rpc_addr;

    Ok(settings)
}
