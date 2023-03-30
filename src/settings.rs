//! Read settings for kneard

use crate::near_config::{read_near_config, NearKey};
use anyhow::{bail, Context, Result};
use clap::Parser;
use near_primitives::types::AccountId;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

// set by systemd LoadCredential
const CREDENTIALS_DIRECTORY: &str = "CREDENTIALS_DIRECTORY";

/// Setting options for kneard
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Settings {
    /// The consul agent url
    #[clap(
        long,
        default_value = "http://localhost:8500",
        env = "KUUTAMO_CONSUL_URL"
    )]
    pub consul_url: String,
    /// Consul token used for authentication, also see `https://www.consul.io/docs/security/acl/acl-tokens`
    #[clap(long, env = "KUUTAMO_CONSUL_TOKEN_FILE")]
    pub consul_token_file: Option<PathBuf>,
    /// Contains the content of `consul_token_file`
    #[clap(skip = None)]
    pub consul_token: Option<String>,

    /// Node id of the kuutamo instance
    #[clap(long, default_value = "node", env = "KUUTAMO_NODE_ID")]
    pub node_id: String,
    /// NEAR Account id of the validator. This ID will be used to acquire
    /// leadership in consul. It should be the same for all nodes that share the
    /// same validator key
    #[clap(long, default_value = "default", env = "KUUTAMO_ACCOUNT_ID")]
    pub account_id: AccountId,
    /// The exporter address, that kneard will listen to: format: ip:host
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
    /// The neard validator key that we will pass to neard, when kneard becomes validator
    #[clap(long, default_value = ".", env = "KUUTAMO_VALIDATOR_KEY")]
    pub validator_key: PathBuf,
    /// The neard node key that we will pass to neard, when kneard becomes validator
    #[clap(long, default_value = ".", env = "KUUTAMO_VALIDATOR_NODE_KEY")]
    pub validator_node_key: PathBuf,

    /// The public key of the node key that we will write in the public address
    /// of our neard configuration when kneard becomes validator
    #[clap(skip)]
    pub validator_node_public_key: String,

    /// The address neard will listen, when being a validator
    #[clap(
        long,
        default_value = "0.0.0.0:24567",
        env = "KUUTAMO_VALIDATOR_NETWORK_ADDR"
    )]
    pub validator_network_addr: SocketAddr,
    /// The neard node key that we will pass to neard, when kneard is not a validator
    #[clap(long, default_value = ".", env = "KUUTAMO_VOTER_NODE_KEY")]
    pub voter_node_key: PathBuf,
    /// The address neard will listen, when kneard is not a validator. At least the port should be different from `validator_network_addr`
    #[clap(
        long,
        default_value = "0.0.0.0:24568",
        env = "KUUTAMO_VOTER_NETWORK_ADDR"
    )]
    pub voter_network_addr: SocketAddr,

    /// The ip addresses of the validator is *directly* reachable.
    /// Kuutamod will add the configured validator node key and port number of this node to these
    /// addresses and expects each entry to be an ip address without the public key part
    #[clap(long, env = "KUUTAMO_PUBLIC_ADDRESS")]
    pub public_address: Option<IpAddr>,

    /// Bootnodes passed to neard
    #[clap(long, env = "KUUTAMO_NEARD_BOOTNODES")]
    pub near_boot_nodes: Option<String>,

    /// Unix socket path where kneard will listen for remote control commands
    #[clap(
        long,
        default_value = "/var/lib/neard/kuutamod.sock",
        env = "KUUTAMO_CONTROL_SOCKET"
    )]
    pub control_socket: PathBuf,
}

fn get_near_key(key: &str, val: &mut PathBuf, credential_filename: &str) -> Result<String> {
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

    // compute absolute path for symlinking
    *val = fs::canonicalize(&val)
        .with_context(|| format!("cannot resolve path for {}", val.display()))?;

    let key = NearKey::read_from_file(val).context("failed to read near key")?;

    Ok(key.public_key)
}

/// Read and returns settings from environment variables and the filesystem
pub fn parse_settings() -> Result<Settings> {
    let mut settings = Settings::parse();

    get_near_key(
        "--validator-key",
        &mut settings.validator_key,
        "validator_key.json",
    )?;
    settings.validator_node_public_key = get_near_key(
        "--validator-node-key",
        &mut settings.validator_node_key,
        "validator_node_key.json",
    )?;
    get_near_key(
        "--voter-node-key",
        &mut settings.voter_node_key,
        "voter_node_key.json",
    )?;

    settings.consul_token = match settings.consul_token_file {
        Some(ref file) => {
            let s = fs::read_to_string(file)
                .with_context(|| format!("cannot read consul token file {}", file.display()))?;
            Some(s.trim_end().to_string())
        }
        None => None,
    };

    let config_path = &settings.neard_home.join("config.json");
    let config = read_near_config(config_path).context("failed to parse near config")?;
    settings.near_rpc_addr = config.rpc_addr;
    settings.control_socket = settings.neard_home.join("kuutamod.sock");

    Ok(settings)
}
