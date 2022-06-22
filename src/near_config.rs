//! Module for parsing neard config and keys

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::net::SocketAddr;
use std::path::Path;

/// A key used neard i.e. node key, validator key etc
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NearKey {
    /// Human readable account identifier
    pub account_id: String,
    /// ed25519 public key
    pub public_key: String,
    /// ed25519 private key
    pub secret_key: String,
}

impl NearKey {
    /// Reads and returns a near key in json format from path
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<NearKey> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.as_ref().display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("error parsing near key {}", path.as_ref().display()))
    }
    /// Writes near key in json format to path
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(&path)
            .with_context(|| format!("Failed to open {}", &path.as_ref().display()))?;
        serde_json::to_writer_pretty(&file, &self).context("cannot serialize key")?;
        Ok(())
    }
}

/// Subset of data stored in neard's config.json
pub struct NearConfig {
    /// TCP Port of the neard rpc service
    pub rpc_addr: SocketAddr,
}

/// Reads, parses and returns neard's config.json
pub fn read_near_config<P: AsRef<Path>>(path: P) -> Result<NearConfig> {
    let content = fs::read_to_string(path.as_ref())
        .with_context(|| format!("cannot read {}", path.as_ref().display()))?;
    let config: Value = serde_json::from_str(&content)
        .with_context(|| format!("error parsing near key {}", path.as_ref().display()))?;

    let rpc_addr_string = config
        .get("rpc")
        .and_then(|o| o.get("addr"))
        .and_then(|o| o.as_str())
        .unwrap_or("0.0.0.0:3030");
    let rpc_addr = rpc_addr_string
        .parse::<SocketAddr>()
        .with_context(|| format!("failed to parse rpc addr {}", rpc_addr_string))?;

    Ok(NearConfig { rpc_addr })
}

/// Update RPC and network port in existing neard configuration
pub fn update_near_network_addr<P: AsRef<Path>>(path: P, addr: &SocketAddr) -> Result<()> {
    let content = fs::read_to_string(path.as_ref())
        .with_context(|| format!("cannot read {}", path.as_ref().display()))?;
    let mut current_config: Value = serde_json::from_str(&content)
        .with_context(|| format!("error parsing near key {}", path.as_ref().display()))?;

    if let Some(o) = current_config
        .get_mut("network")
        .and_then(|o| o.get_mut("addr"))
    {
        *o = json!(addr);
    }

    let file = File::create(path.as_ref())?;
    serde_json::to_writer(file, &current_config)
        .with_context(|| format!("failed to write to {}", path.as_ref().display()))?;

    Ok(())
}
