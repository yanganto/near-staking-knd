use anyhow::{bail, Context, Result};
use format_serde_error::SerdeError;
use regex::Regex;
use serde::Serialize;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use toml;

use super::secrets::Secrets;
use super::NixosFlake;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    global: GlobalConfig,

    #[serde(default)]
    host_defaults: HostConfig,
    #[serde(default)]
    hosts: HashMap<String, HostConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct HostConfig {
    #[serde(default)]
    ipv4_address: Option<IpAddr>,
    #[serde(default)]
    ipv4_gateway: Option<IpAddr>,
    #[serde(default)]
    ipv4_cidr: Option<u8>,
    #[serde(default)]
    nixos_module: Option<String>,
    #[serde(default)]
    extra_nixos_modules: Vec<String>,

    #[serde(default)]
    pub mac_address: Option<String>,
    #[serde(default)]
    ipv6_address: Option<IpAddr>,
    #[serde(default)]
    ipv6_gateway: Option<IpAddr>,
    #[serde(default)]
    ipv6_cidr: Option<u8>,

    #[serde(default)]
    public_ssh_keys: Vec<String>,

    #[serde(default)]
    install_ssh_user: Option<String>,

    #[serde(default)]
    ssh_hostname: Option<String>,

    #[serde(default)]
    validator_key_file: Option<PathBuf>,
    #[serde(default)]
    validator_node_key_file: Option<PathBuf>,

    #[serde(default)]
    pub disks: Option<Vec<PathBuf>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct ValidatorKeys {
    // Near validator key
    pub key_file: PathBuf,
    // Near validator node key
    pub node_key_file: PathBuf,
}

/// Global configuration affecting all hosts
#[derive(Debug, Default, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    flake: Option<String>,
}

/// NixOS host configuration
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Host {
    /// Name identifying the host
    pub name: String,

    /// NixOS module to use as a base for the host from the flake
    pub nixos_module: String,

    /// Extra NixOS modules to include in the system
    pub extra_nixos_modules: Vec<String>,

    /// Mac address of the public interface to use
    pub mac_address: Option<String>,

    /// Public ipv4 address of the host
    pub ipv4_address: IpAddr,
    /// Cidr of the public ipv4 address
    pub ipv4_cidr: u8,
    /// Public ipv4 gateway ip address
    pub ipv4_gateway: IpAddr,

    /// Public ipv6 address of the host
    pub ipv6_address: IpAddr,
    /// Cidr of the public ipv6 address
    pub ipv6_cidr: u8,
    /// Public ipv6 gateway address of the host
    pub ipv6_gateway: IpAddr,

    /// SSH Username used when connecting during installation
    pub install_ssh_user: String,

    /// SSH hostname used for connecting
    pub ssh_hostname: String,

    /// Public ssh keys that will be added to the nixos configuration
    pub public_ssh_keys: Vec<String>,

    /// Block device paths to use for installing
    pub disks: Vec<PathBuf>,

    /// Validator keys used by neard
    pub validator_keys: Option<ValidatorKeys>,
}

impl Host {
    /// Returns prepared secrets directory for host
    pub fn secrets(&self) -> Result<Secrets> {
        let mut secret_files = vec![];
        let validator_key: Option<PathBuf>;
        let node_key: Option<PathBuf>;
        if let Some(keys) = &self.validator_keys {
            validator_key = Some(PathBuf::from("/var/lib/secrets/validator_key.json"));
            node_key = Some(PathBuf::from("/var/lib/secrets/node_key.json"));
            secret_files.push((
                validator_key.as_ref().unwrap().as_path(),
                keys.key_file.as_path(),
            ));
            secret_files.push((
                node_key.as_ref().unwrap().as_path(),
                keys.node_key_file.as_path(),
            ));
        }

        Secrets::new(secret_files.iter()).context("failed to prepare uploading secrets")
    }
    /// The hostname to which we will deploy
    pub fn deploy_ssh_target(&self) -> String {
        format!("root@{}", self.ssh_hostname)
    }
    /// The hostname to which we will deploy
    pub fn flake_uri(&self, flake: &NixosFlake) -> String {
        format!("{}#{}", flake.path().display(), self.name)
    }
}

/// Global configuration affecting all hosts
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct Global {
    /// Flake url where the nixos configuration is
    #[serde(default)]
    pub flake: String,
}

fn validate_global(global_config: &GlobalConfig) -> Result<Global> {
    let default_flake = "github:kuutamolabs/near-staking-knd";
    let flake = global_config
        .flake
        .as_deref()
        .unwrap_or(default_flake)
        .to_string();
    Ok(Global { flake })
}

fn validate_host(name: &str, host: &HostConfig, default: &HostConfig) -> Result<Host> {
    let name = name.to_string();

    if name.is_empty() || name.len() > 63 {
        bail!(
            "a host's name must be between 1 and 63 characters long, got: '{}'",
            name
        );
    }
    let hostname_regex = Regex::new(r"^[a-z0-9][a-z0-9\-]{0,62}$").unwrap();
    if !hostname_regex.is_match(&name) {
        bail!("a host's name must only contain letters from a to z, the digits from 0 to 9, and the hyphen (-). But not starting with a hyphen. got: '{}'", name);
    }
    let mac_address = if let Some(ref a) = &host.mac_address {
        let mac_address_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
        if !mac_address_regex.is_match(a) {
            bail!("mac address does match a valid format: {} (valid example value: 02:42:34:d1:18:7a)", a);
        }
        Some(a.clone())
    } else {
        None
    };
    let ipv4_address = host
        .ipv4_address
        .or(default.ipv4_address)
        .with_context(|| format!("no ipv4_address provided for host.{}", name))?;
    let ipv4_cidr = host
        .ipv4_cidr
        .or(default.ipv4_cidr)
        .with_context(|| format!("no ipv4_cidr provided for hosts.{}", name))?;

    if !ipv4_address.is_ipv4() {
        format!(
            "ipv4_address provided for hosts.{} is not an ipv4 address: {}",
            name, ipv4_address
        );
    }

    // FIXME: this is currently an unstable feature
    //if ipv4_address.is_global() {
    //    warn!("ipv4_address provided for hosts.{} is not a public ipv4 address: {}. This might not work with near mainnet", name, ipv4_address);
    //}

    if !(0..32_u8).contains(&ipv4_cidr) {
        bail!(
            "ipv4_cidr for hosts.{} is not between 0 and 32: {}",
            name,
            ipv4_cidr
        )
    }

    let default_module = "single-node-validator-mainnet";
    let nixos_module = host
        .nixos_module
        .as_deref()
        .or(default.nixos_module.as_deref())
        .unwrap_or(default_module)
        .to_string();

    let mut extra_nixos_modules = vec![];
    extra_nixos_modules.extend_from_slice(&host.extra_nixos_modules);
    extra_nixos_modules.extend_from_slice(&default.extra_nixos_modules);

    let ipv4_gateway = host
        .ipv4_gateway
        .or(default.ipv4_gateway)
        .with_context(|| format!("no ipv4_gateway provided for hosts.{}", name))?;

    let ipv6_address = host
        .ipv6_address
        .or(default.ipv6_address)
        .with_context(|| format!("no ipv6_address provided for hosts.{}", name))?;
    if !ipv6_address.is_ipv6() {
        format!(
            "ipv6_address provided for hosts.{} is not an ipv6 address: {}",
            name, ipv6_address
        );
    }
    // FIXME: this is currently an unstable feature
    //if ipv6_address.is_global() {
    //    warn!("ipv6_address provided for hosts.{} is not a public ipv6 address: {}. This might not work with near mainnet", name, ipv6_address);
    //}
    let ipv6_cidr = host
        .ipv6_cidr
        .or(default.ipv6_cidr)
        .with_context(|| format!("no ipv6_cidr provided for hosts.{}", name))?;
    if !(0..128_u8).contains(&ipv6_cidr) {
        bail!(
            "ipv6_cidr for hosts.{} is not between 0 and 128: {}",
            name,
            ipv6_cidr
        )
    }
    let ipv6_gateway = host
        .ipv6_gateway
        .or(default.ipv6_gateway)
        .with_context(|| format!("no ipv6_gateway provided for hosts.{}", name))?;

    let ssh_hostname = host
        .ssh_hostname
        .as_ref()
        .or(default.ssh_hostname.as_ref())
        .cloned()
        .unwrap_or_else(|| ipv4_address.to_string());

    let install_ssh_user = host
        .install_ssh_user
        .as_ref()
        .or(default.install_ssh_user.as_ref())
        .cloned()
        .unwrap_or_else(|| String::from("root"));

    let mut public_ssh_keys = vec![];
    public_ssh_keys.extend_from_slice(&host.public_ssh_keys);
    public_ssh_keys.extend_from_slice(&default.public_ssh_keys);
    if public_ssh_keys.is_empty() {
        bail!("no public_ssh_keys provided for hosts.{}", name);
    }

    let default_disks = vec![PathBuf::from("/dev/nvme0n1"), PathBuf::from("/dev/nvme1n1")];
    let disks = host
        .disks
        .as_ref()
        .or(default.disks.as_ref())
        .unwrap_or(&default_disks)
        .to_vec();

    if disks.is_empty() {
        bail!("no disks specified for hosts.{}", name);
    }

    let validator_key_file = host
        .validator_key_file
        .as_ref()
        .or(default.validator_key_file.as_ref())
        .map(|v| v.to_path_buf());

    let validator_node_key_file = host
        .validator_node_key_file
        .as_ref()
        .or(default.validator_node_key_file.as_ref())
        .map(|v| v.to_path_buf());

    let validator_keys = if let Some(validator_key_file) = validator_key_file {
        if let Some(validator_node_key_file) = validator_node_key_file {
            Some(ValidatorKeys {
                key_file: validator_key_file,
                node_key_file: validator_node_key_file,
            })
        } else {
            bail!(
                "hosts.{} has a validator_key_file but not a validator_node_key_file",
                name
            );
        }
    } else {
        if validator_node_key_file.is_some() {
            bail!(
                "hosts.{} has a validator_node_key_file but not a validator_key_file",
                name
            );
        }
        None
    };

    Ok(Host {
        name,
        nixos_module,
        extra_nixos_modules,
        install_ssh_user,
        ssh_hostname,
        mac_address,
        ipv4_address,
        ipv4_cidr,
        ipv4_gateway,
        ipv6_address,
        ipv6_cidr,
        ipv6_gateway,
        validator_keys,
        public_ssh_keys,
        disks,
    })
}

/// Validated configuration
pub struct Config {
    /// Hosts as defined in the configuration
    pub hosts: HashMap<String, Host>,
    /// Configuration affecting all hosts
    pub global: Global,
}

/// Parse toml configuration
pub fn parse_config(content: &str) -> Result<Config> {
    let config: ConfigFile = toml::from_str(content)
        // pretty print our error message.
        .map_err(|e| SerdeError::new(content.to_string(), e))?;
    let hosts = config
        .hosts
        .iter()
        .map(|(name, host)| {
            Ok((
                name.clone(),
                validate_host(name, host, &config.host_defaults)?,
            ))
        })
        .collect::<Result<_>>()?;

    let global = validate_global(&config.global)?;
    Ok(Config { hosts, global })
}

/// Load configuration from path
pub fn load_configuration(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path).context("Cannot read file")?;
    parse_config(&content)
}

#[test]
pub fn test_parse_config() -> Result<()> {
    use std::str::FromStr;

    let config = parse_config(
        r#"
[global]
flake = "github:myfork/near-staking-knd"

[host_defaults]
public_ssh_keys = [
  '''ssh-ed25519 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA foobar'''
]
ipv4_cidr = 24
ipv6_cidr = 48
ipv4_gateway = "199.127.64.1"
ipv6_gateway = "2605:9880:400::1"

[hosts]
[hosts.validator-00]
ipv4_address = "199.127.64.2"
ipv6_address = "2605:9880:400::2"
ipv6_cidr = 48
validator_key_file = "validator_key.json"
validator_node_key_file = "node_key.json"

[hosts.validator-01]
ipv4_address = "199.127.64.3"
ipv6_address = "2605:9880:400::3"
"#,
    )?;
    assert_eq!(config.global.flake, "github:myfork/near-staking-knd");

    let hosts = &config.hosts;
    assert_eq!(hosts.len(), 2);
    assert_eq!(
        hosts["validator-00"].ipv4_address,
        IpAddr::from_str("199.127.64.2").unwrap()
    );
    assert_eq!(hosts["validator-00"].ipv4_cidr, 24);
    assert_eq!(
        hosts["validator-00"].ipv4_gateway,
        IpAddr::from_str("199.127.64.1").unwrap()
    );
    assert_eq!(
        hosts["validator-00"].ipv6_address,
        IpAddr::from_str("2605:9880:400::2").unwrap()
    );
    assert_eq!(hosts["validator-00"].ipv6_cidr, 48);
    assert_eq!(
        hosts["validator-00"].ipv6_gateway,
        IpAddr::from_str("2605:9880:400::1").unwrap()
    );
    assert_eq!(
        hosts["validator-00"].validator_keys,
        Some(ValidatorKeys {
            key_file: PathBuf::from("validator_key.json"),
            node_key_file: PathBuf::from("node_key.json")
        })
    );

    assert_eq!(hosts["validator-01"].validator_keys, None);
    Ok(())
}
