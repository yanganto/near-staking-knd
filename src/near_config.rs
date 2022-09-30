//! Module for parsing neard config and keys

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::Write;
use std::net::{IpAddr, SocketAddr};
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

/// Update public addresses, RPC and network port in existing neard configuration
pub fn update_neard_config<P: AsRef<Path>>(
    path: P,
    public_ips: &[IpAddr],
    port: u16,
    node_key: &str,
    listen_addr: &SocketAddr,
) -> Result<()> {
    let content = fs::read_to_string(path.as_ref())
        .with_context(|| format!("cannot read {}", path.as_ref().display()))?;
    let mut current_config: Value = serde_json::from_str(&content)
        .with_context(|| format!("error parsing near key {}", path.as_ref().display()))?;

    if let Some(o) = current_config
        .get_mut("network")
        .and_then(|o| o.get_mut("addr"))
    {
        *o = json!(listen_addr);
    }
    match current_config.get_mut("network") {
        Some(serde_json::Value::Object(map)) => {
            let public_addrs = public_ips
                .iter()
                .map(|ip| {
                    if ip.is_ipv6() {
                        format!("{}@[{}]:{}", node_key, ip, port)
                    } else {
                        format!("{}@{}:{}", node_key, ip, port)
                    }
                })
                .collect::<Vec<_>>();
            map.insert("public_addrs".to_string(), json!(public_addrs));
        }
        None => {
            bail!("no network section found in neard configuration");
        }
        val => {
            bail!(
                "network section found in neard configuration but an Map but {:?}",
                val
            );
        }
    }

    let mut file = File::create(path.as_ref())?;
    file.write_all(
        serde_json::to_string_pretty(&current_config)
            .context("failed to serialize neard configuration")?
            .as_bytes(),
    )
    .with_context(|| format!("failed to write to {}", path.as_ref().display()))?;

    Ok(())
}
