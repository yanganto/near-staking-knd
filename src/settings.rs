//! Read settings for kuutamod

use crate::near_config::read_near_config;
use anyhow::{bail, Context, Result};
use nix::unistd::{access, AccessFlags};
use std::{net::SocketAddr, path::PathBuf};

// Environment variables used by kuutamod

const NODE_ID: &str = "KUUTAMO_NODE_ID";
const ACCOUNT_ID: &str = "KUUTAMO_ACCOUNT_ID";
const CONSUL_URL: &str = "KUUTAMO_CONSUL_URL";
const EXPORTER_ADDRESS: &str = "KUUTAMO_EXPORTER_ADDRESS";
const VALIDATOR_KEY: &str = "KUUTAMO_VALIDATOR_KEY";
const VALIDATOR_NODE_KEY: &str = "KUUTAMO_VALIDATOR_NODE_KEY";
const VALIDATOR_NETWORK_ADDR: &str = "KUUTAMO_VALIDATOR_NETWORK_ADDR";
const VOTER_NODE_KEY: &str = "KUUTAMO_VOTER_NODE_KEY";
const VOTER_NETWORK_ADDR: &str = "KUUTAMO_VOTER_NETWORK_ADDR";
// set by systemd LoadCredential
const CREDENTIALS_DIRECTORY: &str = "CREDENTIALS_DIRECTORY";
const NEARD_HOME: &str = "KUUTAMO_NEARD_HOME";
const BOOT_NODES: &str = "KUUTAMO_NEARD_BOOTNODES";

/// Setting options for kuutamod
#[derive(Debug)]
pub struct Settings {
    /// The consul agent url
    pub consul_url: String,
    /// Node id of the kuutamo instance
    pub node_id: String,
    /// NEAR Account id of the validator. This ID will be used to acquire
    /// leadership in consul. It should be the same for all nodes that share the
    /// same validator key
    pub account_id: String,
    /// The exporter address, that kuutamod will listen to: format: ip:host
    pub exporter_address: String,
    /// Location where keys and chain data for neard is stored
    pub neard_home: PathBuf,
    /// RPC address of the neard daemon
    pub near_rpc_addr: SocketAddr,
    /// The neard validator key that we will pass to neard, when kuutamod becomes validator
    pub validator_key: PathBuf,
    /// The neard node key that we will pass to neard, when kuutamod becomes validator
    pub validator_node_key: PathBuf,
    /// The address neard will listen, when beeing a validator
    pub validator_network_addr: SocketAddr,
    /// The neard node key that we will pass to neard, when kuutamod is not a validator
    pub voter_node_key: PathBuf,
    /// The address neard will listen, when kuutamod is not a validator. At least the port should be different from `validator_network_addr`
    pub voter_network_addr: SocketAddr,
    /// Bootnodes passed to neard
    pub near_boot_nodes: Option<String>,
}

fn get_near_key(env_var: &str, credential_filename: &str) -> Result<PathBuf> {
    let path = match std::env::var_os(env_var) {
        Some(val) => PathBuf::from(val),
        None => {
            // Use systemd's LoadCredential environment variable, if it exits:
            // TODO: replace this with KUUTAMO_NEAR_VALIDATOR_FILE=%d/validator_key.json in systemd's Environment after the next systemd upgrade:
            // see: https://www.freedesktop.org/software/systemd/man/systemd.exec.html
            match std::env::var_os(CREDENTIALS_DIRECTORY) {
                Some(val) => PathBuf::from(val).join(credential_filename),
                None => {
                    bail!("{} environment variable is not set but required", env_var);
                }
            }
        }
    };
    access(&path, AccessFlags::R_OK | AccessFlags::F_OK)
        .with_context(|| format!("cannot open {} as a file", path.display()))?;
    Ok(path)
}

fn parse_sock_addr_from_env(key: &str, default: &str) -> Result<SocketAddr> {
    let addr_str = std::env::var(key).unwrap_or_else(|_| default.to_string());
    addr_str
        .parse::<SocketAddr>()
        .with_context(|| format!("failed to parse addr {}", addr_str))
}

/// Read and returns settings from environment variables and the filesystem
pub fn settings_from_env() -> Result<Settings> {
    let consul_url =
        std::env::var(CONSUL_URL).unwrap_or_else(|_| "http://localhost:8500".to_string());
    let exporter_address =
        std::env::var(EXPORTER_ADDRESS).unwrap_or_else(|_| "127.0.0.1:2233".to_string());

    let node_id = std::env::var(NODE_ID).unwrap_or_else(|_| "node".to_string());
    let account_id = std::env::var(ACCOUNT_ID).unwrap_or_else(|_| "default".to_string());

    // This is the default neard port
    let validator_network_addr = parse_sock_addr_from_env(VALIDATOR_NETWORK_ADDR, "0.0.0.0:24567")
        .with_context(|| format!("failed to parse ${}", VALIDATOR_NETWORK_ADDR))?;

    // We use a different port for non-validator to cause no confusion in other peers routing tables
    let voter_network_addr = parse_sock_addr_from_env(VOTER_NETWORK_ADDR, "0.0.0.0:24568")
        .with_context(|| format!("failed to parse ${}", VOTER_NETWORK_ADDR))?;

    let boot_nodes = std::env::var(BOOT_NODES).ok();

    let validator_key = get_near_key(VALIDATOR_KEY, "validator_key.json")?;
    let validator_node_key = get_near_key(VALIDATOR_NODE_KEY, "validator_node_key.json")?;
    let voter_node_key = get_near_key(VOTER_NODE_KEY, "voter_node_key.json")?;

    let _neard_home = std::env::var(NEARD_HOME).map(PathBuf::from);
    let neard_home = match _neard_home {
        Ok(v) => v,
        Err(_) => std::env::current_dir().context("Cannot get current working directory")?,
    };

    let config_path = &neard_home.join("config.json");
    let config = read_near_config(config_path).context("failed to parse near config")?;

    Ok(Settings {
        node_id,
        account_id,
        consul_url,
        exporter_address,
        neard_home,
        near_rpc_addr: config.rpc_addr,
        validator_key,
        validator_node_key,
        validator_network_addr,
        voter_node_key,
        voter_network_addr,
        near_boot_nodes: boot_nodes,
    })
}
