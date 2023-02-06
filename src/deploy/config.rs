use anyhow::{anyhow, bail, Context, Result};
use format_serde_error::SerdeError;
use log::{info, warn};
use nix::libc::STDIN_FILENO;
use nix::sys::termios;
use regex::Regex;
use serde::Serialize;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Read};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use toml;

use super::command::status_to_pretty_err;
use super::secrets::Secrets;
use super::NixosFlake;

struct DisableTerminalEcho {
    flags: Option<termios::Termios>,
}

impl DisableTerminalEcho {
    fn new() -> Self {
        let old_flags = match termios::tcgetattr(STDIN_FILENO) {
            Ok(flags) => flags,
            Err(_) => {
                // Not a terminal, just make this a NOOP
                return DisableTerminalEcho { flags: None };
            }
        };
        let mut new_flags = old_flags.clone();
        new_flags.local_flags &= !termios::LocalFlags::ECHO;
        match termios::tcsetattr(STDIN_FILENO, termios::SetArg::TCSANOW, &new_flags) {
            Ok(_) => DisableTerminalEcho {
                flags: Some(old_flags),
            },
            Err(_) => DisableTerminalEcho { flags: None },
        }
    }
}

impl Drop for DisableTerminalEcho {
    fn drop(&mut self) {
        if let Some(ref flags) = self.flags {
            let _ = termios::tcsetattr(STDIN_FILENO, termios::SetArg::TCSANOW, flags);
        }
    }
}

/// IpV6String allows prefix only address format and normal ipv6 address
///
/// Some providers include the subnet in their address shown in the webinterface i.e. 2607:5300:203:5cdf::/64
/// This format is rejected by IpAddr in Rust and we store subnets in a different configuration option.
/// This struct detects such cashes in the kuutamo.toml file and converting it to 2607:5300:203:5cdf:: with a warning message, providing a more user-friendly experience.
type IpV6String = String;

trait AsIpAddr {
    fn is_ipv6(&self) -> bool;
    fn to_ipv6(&self) -> Result<IpAddr>;
}

impl AsIpAddr for IpV6String {
    fn is_ipv6(&self) -> bool {
        if let Some(idx) = self.find('/') {
            self.get(idx + 1..self.len())
                .map(|i| i.parse::<usize>().is_ok())
                .unwrap_or(false)
                && self
                    .get(0..idx)
                    .map(|addr| addr.parse::<IpAddr>().is_ok())
                    .unwrap_or(false)
        } else {
            self.parse::<IpAddr>()
                .map(|ip| ip.is_ipv6())
                .unwrap_or(false)
        }
    }

    /// Handle ipv6 segament to a valide ip address
    fn to_ipv6(&self) -> Result<IpAddr> {
        if let Some(idx) = self.find('/') {
            let _ = self
                .get(idx + 1..self.len())
                .map(|i| i.parse::<usize>().is_ok())
                .with_context(|| "no ipv6_address should '/' a unsign intager")?;

            match self.get(0..idx) {
                Some(addr_str) => {
                    if let Ok(addr) = addr_str.parse::<IpAddr>() {
                        warn!("{self:} is a ipv6 segament will use {addr:} for ipv6 address");
                        Ok(addr)
                    } else {
                        Err(anyhow!("ipv6_address invalid"))
                    }
                }
                _ => Err(anyhow!("ipv6_address invalid")),
            }
        } else {
            Ok(self.parse::<IpAddr>()?)
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    global: GlobalConfig,

    #[serde(default)]
    host_defaults: HostConfig,
    #[serde(default)]
    hosts: HashMap<String, HostConfig>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct NearKeyFile {
    pub account_id: String,
    pub public_key: String,
    // Credential files generated which near cli works with have private_key
    // rather than secret_key field.  To make it possible to read those from
    // neard add private_key as an alias to this field so either will work.
    #[serde(alias = "private_key")]
    pub secret_key: String,
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
    ipv6_address: Option<IpV6String>,
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
    pub validator_key: NearKeyFile,
    // Near validator node key
    pub validator_node_key: NearKeyFile,
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
    #[serde(skip_serializing)]
    pub validator_keys: Option<ValidatorKeys>,
}

impl Host {
    /// Returns prepared secrets directory for host
    pub fn secrets(&self) -> Result<Secrets> {
        let mut secret_files = vec![];
        if let Some(keys) = &self.validator_keys {
            secret_files.push((
                PathBuf::from("/var/lib/secrets/validator_key.json"),
                serde_json::to_string_pretty(&keys.validator_key)
                    .context("failed to convert validator key to json")?,
            ));
            secret_files.push((
                PathBuf::from("/var/lib/secrets/node_key.json"),
                serde_json::to_string_pretty(&keys.validator_node_key)
                    .context("failed to convert validator node to json")?,
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

fn validate_host(
    name: &str,
    host: &HostConfig,
    default: &HostConfig,
    working_directory: Option<&Path>,
) -> Result<Host> {
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
        .as_ref()
        .with_context(|| format!("no ipv6_address provided for host.{}", name))?;

    if !ipv6_address.is_ipv6() {
        format!(
            "ipv6_address provided for hosts.{} is not an ipv6 address: {}",
            name,
            host.ipv6_address.as_ref().expect("already check is some")
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
            let validator_key_path = if validator_key_file.is_absolute() {
                validator_key_file
            } else {
                working_directory
                    .unwrap_or_else(|| Path::new("."))
                    .join(validator_key_file)
            };
            let validator_node_key_path = if validator_node_key_file.is_absolute() {
                validator_node_key_file
            } else {
                working_directory
                    .unwrap_or_else(|| Path::new("."))
                    .join(validator_node_key_file)
            };
            Some(read_validator_keys(
                validator_key_path,
                validator_node_key_path,
            )?)
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
        ipv6_address: ipv6_address.to_ipv6()?,
        ipv6_cidr,
        ipv6_gateway,
        validator_keys,
        public_ssh_keys,
        disks,
    })
}

fn ask_password_for(file: &Path) -> Result<String> {
    let file_name = file
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    println!("Please give your password for {}", file_name);

    let disable_terminal_echo = DisableTerminalEcho::new();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    drop(disable_terminal_echo);

    Ok(line.trim_end().to_string())
}

fn decrypt_and_unzip_file(file: &PathBuf, password: String) -> Result<String> {
    let mut content = String::new();
    let file_name = file
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let mut archive = fs::File::open(file)
        .map(zip::ZipArchive::new)
        .with_context(|| format!("{file_name:} could not treat as zip archive"))??;
    if let Ok(Ok(mut zip)) = archive
        .by_index_decrypt(0, password.as_bytes())
        .with_context(|| format!("password for {file_name:} is incorrect"))
    {
        zip.read_to_string(&mut content)?;
    }
    Ok(content)
}

fn read_validator_keys(
    validator_key_file: PathBuf,
    validator_node_key_file: PathBuf,
) -> Result<ValidatorKeys> {
    let validator_key = if let Some("zip") = validator_key_file
        .extension()
        .map(|s| s.to_str().unwrap_or_default())
    {
        let password = ask_password_for(&validator_key_file)?;
        decrypt_and_unzip_file(&validator_key_file, password)?
    } else {
        fs::read_to_string(&validator_key_file).with_context(|| {
            format!(
                "cannot read validator key file: '{}'",
                validator_key_file.display()
            )
        })?
    };

    let validator_node_key = if let Some("zip") = validator_node_key_file
        .extension()
        .map(|s| s.to_str().unwrap_or_default())
    {
        let password = ask_password_for(&validator_node_key_file)?;
        decrypt_and_unzip_file(&validator_node_key_file, password)?
    } else if validator_node_key_file.exists() {
        fs::read_to_string(&validator_node_key_file).with_context(|| {
            format!(
                "cannot read validator node key file: '{}'",
                validator_key_file.display()
            )
        })?
    } else {
        info!(
            "{} does not exist yet, generate it...",
            validator_node_key_file.display()
        );
        let tmp_dir = TempDir::new()?;
        let args = &[
            "--home",
            tmp_dir
                .path()
                .to_str()
                .context("cannot convert path to string")?,
            "init",
        ];
        let status = Command::new("neard").args(args).status();
        status_to_pretty_err(status, "neard", args)
            .context("cannot generate validator node key")?;
        let tmp_node_key_path = tmp_dir.path().join("node_key.json");
        let content = fs::read_to_string(&tmp_node_key_path).with_context(|| {
            format!(
                "cannot read generated node_key.json: {}",
                tmp_node_key_path.display()
            )
        })?;
        fs::write(&validator_node_key_file, &content).with_context(|| {
            format!(
                "failed to write validator node key to {}",
                validator_node_key_file.display()
            )
        })?;
        content
    };

    Ok(ValidatorKeys {
        validator_key: serde_json::from_str(&validator_key)
            .map_err(|e| SerdeError::new(validator_key.to_string(), e))
            .with_context(|| {
                format!(
                    "validator key file at '{}' is not valid",
                    validator_key_file.display()
                )
            })?,
        validator_node_key: serde_json::from_str(&validator_node_key)
            .map_err(|e| SerdeError::new(validator_node_key.to_string(), e))
            .with_context(|| {
                format!(
                    "validator key file at '{}' is not valid",
                    validator_key_file.display()
                )
            })?,
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
pub fn parse_config(content: &str, working_directory: Option<&Path>) -> Result<Config> {
    let config: ConfigFile = toml::from_str(content)?;
    let hosts = config
        .hosts
        .iter()
        .map(|(name, host)| {
            Ok((
                name.clone(),
                validate_host(name, host, &config.host_defaults, working_directory)?,
            ))
        })
        .collect::<Result<_>>()?;

    let global = validate_global(&config.global)?;
    Ok(Config { hosts, global })
}

/// Load configuration from path
pub fn load_configuration(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path).context("Cannot read file")?;
    let working_directory = path.parent();
    parse_config(&content, working_directory)
}

#[test]
pub fn test_parse_config() -> Result<()> {
    use std::str::FromStr;

    let validator_key = include_str!("../../nix/modules/tests/validator_key.json");
    let node_key = include_str!("../../nix/modules/tests/node_key.json");

    fs::write("validator_key.json", validator_key).unwrap();
    fs::write("node_key.json", node_key).unwrap();

    let config_str = r#"
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
"#;

    let config = parse_config(config_str, None)?;
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
    let k = hosts["validator-00"].validator_keys.as_ref().unwrap();
    assert_eq!(k.validator_key, NearKeyFile {
        account_id: String::from("kuutamod0"),
        public_key: String::from("ed25519:3XGPceVrDHPaysJ2LV2iftYjnRVAJm31GkJCnG4cGLp1"),
        secret_key: String::from("ed25519:22eQKH8uYsesa8qy5g4yCwmpr6hmy2srmUnC155EbV6vxSAkeMioZucdcGxnDQ1HHPtTRGpFGexUtPdKGEMV5BE1"),
    });
    let test_public_key = String::from("ed25519:CFWNpHyt3L8erkD9fjeqo1fs46H9x57EkiQ3V2YRoRbw");
    assert_eq!(k.validator_node_key, NearKeyFile {
        account_id: String::from("node"),
        public_key: test_public_key.clone(),
        secret_key: String::from("ed25519:2n3WTvm538TizGD2QFxotr3aNYbWgmoF13sb5Tx6ZC7mtsDHaPsH6dnH4n5m8pjistqbF6BY1k9bsq9mC9ZsbAy"),
    });

    assert_eq!(hosts["validator-01"].validator_keys, None);

    // we delete the node_key.json, `parse-config` will generate it for us:
    fs::remove_file("node_key.json").unwrap();
    let config = parse_config(config_str, None)?;
    let validator_node_key = &config.hosts["validator-00"]
        .validator_keys
        .as_ref()
        .unwrap()
        .validator_node_key;
    assert_eq!(validator_node_key.account_id, "node");
    assert_ne!(validator_node_key.public_key, test_public_key);

    Ok(())
}

#[test]
pub fn test_decrypt_and_unzip_file() {
    let validator_key_file = PathBuf::from("tests/assets/validator_key.zip");
    assert_eq!(
        decrypt_and_unzip_file(&validator_key_file, "1234".into()).unwrap(),
        r#"{
  "account_id": "test.pool.devnet",
  "public_key": "ed25519:9E2PD1zw7YE2oyaNwSEXj54GcayiZiMQX5bfgzCzqpHk",
  "private_key": "ed25519:5a8tzPJxDjEZjsrJmYqwRweQhmeHh3BTmy9aWUhyAkJ3DpVgPnDiA21GfGR7SKLcj2Z9LW7ZcYZ75JNCDa6EvsMG"
}"#
    );
}

#[test]
fn test_valid_ip_string_for_ipv6() {
    let ip: IpV6String = "2607:5300:203:5cdf::".into();
    assert!(ip.is_ipv6());

    let prefix_only_segament: IpV6String = "2607:5300:203:5cdf::/64".into();
    assert!(prefix_only_segament.is_ipv6());
    assert_eq!(
        prefix_only_segament.to_ipv6().unwrap(),
        ip.to_ipv6().unwrap()
    );

    let ip_v4: IpV6String = "192.168.0.1".into();
    assert!(!ip_v4.is_ipv6());
}

#[test]
fn test_invalid_string_for_ipv6() {
    let mut invalid_str: IpV6String = "2607:5300:203:7cdf::/".into();
    assert!(!invalid_str.is_ipv6());

    invalid_str = "/2607:5300:203:7cdf::".into();
    assert!(!invalid_str.is_ipv6());
}
