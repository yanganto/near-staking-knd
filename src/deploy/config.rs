use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use format_serde_error::SerdeError;
use log::{info, warn};
use nix::libc::STDIN_FILENO;
use nix::sys::termios;
use regex::Regex;
use reqwest::Client;
use serde::Serialize;
use serde_derive::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, Read};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use toml;
use toml_example::TomlExample;
use url::Url;

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
/// This struct detects such cashes in the kneard.toml file and converting it to 2607:5300:203:5cdf:: with a warning message, providing a more user-friendly experience.
type IpV6String = String;

trait AsIpAddr {
    /// Handle ipv6 subnet identifier and normalize to a valide ip address and a mask.
    fn normalize(&self) -> Result<(IpAddr, Option<u8>)>;
}

impl AsIpAddr for IpV6String {
    fn normalize(&self) -> Result<(IpAddr, Option<u8>)> {
        if let Some(idx) = self.find('/') {
            let mask = self
                .get(idx + 1..self.len())
                .map(|i| i.parse::<u8>())
                .with_context(|| {
                    format!("ipv6_address contains invalid subnet identifier: {self}")
                })?
                .ok();

            match self.get(0..idx) {
                Some(addr_str) if mask.is_some() => {
                    if let Ok(addr) = addr_str.parse::<IpAddr>() {
                        warn!("{self:} contains a ipv6 subnet identifier... will use {addr:} for ipv6_address and {:} for ipv6_cidr", mask.unwrap_or_default());
                        Ok((addr, mask))
                    } else {
                        Err(anyhow!("ipv6_address is not invalid"))
                    }
                }
                _ => Err(anyhow!("ipv6_address is not invalid")),
            }
        } else {
            Ok((self.parse::<IpAddr>()?, None))
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

/// Neard keys
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct NearKeyFile {
    /// near account
    pub account_id: String,
    /// near public key
    pub public_key: String,
    // Credential files generated which near cli works with have private_key
    // rather than secret_key field.  To make it possible to read those from
    // neard add private_key as an alias to this field so either will work.
    #[serde(alias = "private_key")]
    /// near secret key
    pub secret_key: String,
}

#[derive(Debug, Default, Deserialize, TomlExample)]
struct HostConfig {
    /// Ipv4 address of the node
    #[serde(default)]
    #[toml_example(default = "111.11.11.11")]
    ipv4_address: Option<IpAddr>,
    /// Ipv4 gateway of the node
    #[serde(default)]
    #[toml_example(default = "111.11.11.254")]
    ipv4_gateway: Option<IpAddr>,
    #[serde(default)]
    /// Ipv4 CIDR of the node
    #[toml_example(default = 24)]
    ipv4_cidr: Option<u8>,
    /// Nixos module will deploy to the node
    #[serde(default)]
    #[toml_example(default = "single-node-validator-testnet")]
    nixos_module: Option<String>,
    /// Extra nixos module will deploy to the node
    #[serde(default)]
    #[toml_example(default = [ ])]
    extra_nixos_modules: Vec<String>,

    /// Mac address of the node
    #[serde(default)]
    pub mac_address: Option<String>,
    /// Network interface of the node
    #[serde(default)]
    pub interface: Option<String>,
    /// Ipv6 address of the node
    #[serde(default)]
    ipv6_address: Option<IpV6String>,
    /// Ipv6 gateway of the node
    #[serde(default)]
    ipv6_gateway: Option<IpAddr>,
    /// Ipv6 cidr of the node
    #[serde(default)]
    ipv6_cidr: Option<u8>,

    /// The ssh public keys of the user
    /// After installation the user could login as root with the corresponding ssh private key
    #[serde(default)]
    #[toml_example(default = [ "ssh-ed25519 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/xxxxxxxx/xxxxxxxxxxxxxxxxxxxxxxxxxxxx", ])]
    public_ssh_keys: Vec<String>,

    /// Admin user for install,
    /// Please use `ubuntu` when you use OVH to install at first time,
    /// Ubuntu did not allow `root` login
    #[serde(default)]
    #[toml_example(default = "ubuntu")]
    install_ssh_user: Option<String>,

    /// Setup ssh host name
    #[serde(default)]
    ssh_hostname: Option<String>,

    /// Validator key will use in the validator node
    #[serde(default)]
    #[toml_example(default = "validator_key.json")]
    validator_key_file: Option<PathBuf>,
    /// Validator node key will use in the node
    #[serde(default)]
    #[toml_example(default = "node_key.json")]
    validator_node_key_file: Option<PathBuf>,

    /// Disk configure on the node
    #[serde(default)]
    #[toml_example(default = [ "/dev/vdb", ])]
    pub disks: Option<Vec<PathBuf>>,

    /// load configure from encrypt app file
    #[serde(default)]
    #[toml_example(default = "hello.pool.devnet.zip")]
    encrypted_kuutamo_app_file: Option<PathBuf>,

    /// Token file for monitoring, default is "kuutamo-monitoring.token"
    /// Provide this if you have a different file
    #[serde(default)]
    #[toml_example(default = "kuutamo-monitoring.token")]
    kuutamo_monitoring_token_file: Option<PathBuf>,
    /// Self monitoring server
    /// The url should implements [Prometheus's Remote Write API] (https://prometheus.io/docs/prometheus/latest/configuration/configuration/#remote_write).
    #[serde(default)]
    #[toml_example(default = "https://my.monitoring.server/api/v1/push")]
    self_monitoring_url: Option<Url>,
    /// The http basic auth username to access self monitoring server
    #[serde(default)]
    self_monitoring_username: Option<String>,
    /// The http basic auth password to access self monitoring server
    #[serde(default)]
    self_monitoring_password: Option<String>,
}

/// Near validator keys
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct ValidatorKeys {
    /// Near validator key
    pub validator_key: NearKeyFile,
    /// Near validator node key
    pub validator_node_key: NearKeyFile,
}

/// Telegraf monitor
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TelegrafOutputConfig {
    /// url for monitor
    pub url: Url,
    /// username for monitor
    pub username: String,
    /// password for monitor
    pub password: String,
}

/// Kuutamo monitor
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub struct KmonitorConfig {
    /// self host url for monitoring, None for kuutamo monitoring
    pub url: Option<Url>,
    /// username for kuutamo monitor
    pub username: String,
    /// password for kuutamo monitor
    pub password: String,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Global configuration affecting all hosts
#[derive(Debug, Default, Deserialize, TomlExample)]
pub struct GlobalConfig {
    /// Flake url to use as a blue print to build up
    #[serde(default)]
    #[toml_example(default = "github:kuutamolabs/near-staking-knd")]
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
    /// interface to use
    pub interface: String,

    /// Public ipv4 address of the host
    pub ipv4_address: IpAddr,
    /// Cidr of the public ipv4 address
    pub ipv4_cidr: u8,
    /// Public ipv4 gateway ip address
    pub ipv4_gateway: IpAddr,

    /// Public ipv6 address of the host
    pub ipv6_address: Option<IpAddr>,
    /// Cidr of the public ipv6 address
    pub ipv6_cidr: Option<u8>,
    /// Public ipv6 gateway address of the host
    pub ipv6_gateway: Option<IpAddr>,

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

    /// Setup telegraf output auth for kuutamo monitor server
    #[serde(skip_serializing)]
    pub kmonitor_config: Option<KmonitorConfig>,

    /// Has monitoring server or not
    pub telegraf_has_monitoring: bool,

    /// Hash for monitoring config
    pub telegraf_config_hash: String,
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
        if let Some(KmonitorConfig {
            url,
            username,
            password,
        }) = &self.kmonitor_config
        {
            secret_files.push((
                PathBuf::from("/var/lib/secrets/telegraf"),
                format!("MONITORING_URL={}\nMONITORING_USERNAME={username}\nMONITORING_PASSWORD={password}", url.as_ref().map(|u|u.to_string()).unwrap_or("https://mimir.monitoring-00-cluster.kuutamo.computer/api/v1/push".to_string()))
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

async fn validate_host(
    name: &str,
    host: &HostConfig,
    default: &HostConfig,
    working_directory: Option<&Path>,
    load_keys: bool,
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

    let maybe_interface = host.interface.as_deref().or(default.interface.as_deref());
    if mac_address.is_some() && maybe_interface.is_some() {
        bail!(
            "Either mac_address or interface (not both) must be provided for host.{}",
            name
        );
    }
    let default_interface = "eth0";

    let interface = maybe_interface.unwrap_or(default_interface).to_string();
    let interface_regex = Regex::new(r"^[0-9a-z]*$").unwrap();
    if !interface_regex.is_match(&interface) {
        bail!(
            "interface match a valid format: {} (valid example value: enp1s0f0)",
            &interface
        );
    }

    let ipv4_address = host
        .ipv4_address
        .with_context(|| format!("no ipv4_address provided for host.{name}"))?;
    let ipv4_cidr = host
        .ipv4_cidr
        .or(default.ipv4_cidr)
        .with_context(|| format!("no ipv4_cidr provided for hosts.{name}"))?;

    if !ipv4_address.is_ipv4() {
        format!("ipv4_address provided for hosts.{name} is not an ipv4 address: {ipv4_address}");
    }

    // FIXME: this is currently an unstable feature
    //if ipv4_address.is_global() {
    //    warn!("ipv4_address provided for hosts.{} is not a public ipv4 address: {}. This might not work with near mainnet", name, ipv4_address);
    //}

    if !(0..32_u8).contains(&ipv4_cidr) {
        bail!("ipv4_cidr for hosts.{name} is not between 0 and 32: {ipv4_cidr}")
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
        .with_context(|| format!("no ipv4_gateway provided for hosts.{name}"))?;

    let ipv6_cidr = host.ipv6_cidr.or(default.ipv6_cidr);

    let ipv6_gateway = host.ipv6_gateway.or(default.ipv6_gateway);

    let (ipv6_address, mask) = if let Some(ipv6_address) = host.ipv6_address.as_ref() {
        let (ipv6_address, mask) = ipv6_address
            .normalize()
            .with_context(|| format!("ipv6_address provided for host.{name:} is not valid"))?;
        if !ipv6_address.is_ipv6() {
            bail!("value provided in ipv6_address for hosts.{name} is not an ipv6 address: {ipv6_address}");
        }

        if let Some(ipv6_cidr) = ipv6_cidr {
            if !(0..128_u8).contains(&ipv6_cidr) {
                bail!("ipv6_cidr for hosts.{name} is not between 0 and 128: {ipv6_cidr}")
            }
        } else if mask.is_none() {
            bail!("no ipv6_cidr provided for hosts.{name}");
        }

        if ipv6_gateway.is_none() {
            bail!("no ipv6_gateway provided for hosts.{name}")
        }

        // FIXME: this is currently an unstable feature
        //if ipv6_address.is_global() {
        //    warn!("ipv6_address provided for hosts.{} is not a public ipv6 address: {}. This might not work with near mainnet", name, ipv6_address);
        //}

        (Some(ipv6_address), mask)
    } else {
        warn!("No ipv6_address provided");
        (None, None)
    };

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
        bail!("no public_ssh_keys provided for hosts.{name}");
    }

    let default_disks = vec![PathBuf::from("/dev/nvme0n1"), PathBuf::from("/dev/nvme1n1")];
    let disks = host
        .disks
        .as_ref()
        .or(default.disks.as_ref())
        .unwrap_or(&default_disks)
        .to_vec();

    if disks.is_empty() {
        bail!("no disks specified for hosts.{name}");
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

    let validator_keys = match (
        load_keys,
        validator_key_file,
        validator_node_key_file,
        &host.encrypted_kuutamo_app_file,
    ) {
        (true, key_file, node_key_file, Some(encrypted_kuutamo_app_file)) => {
            if let Some(ref key_file) = key_file {
                warn!(
                    "using {}, and ignore {}",
                    encrypted_kuutamo_app_file
                        .to_str()
                        .unwrap_or("encrypted_kuutamo_app_file"),
                    key_file.to_str().unwrap_or("validator_key_file")
                );
            }
            if let Some(ref node_key_file) = node_key_file {
                warn!(
                    "using {}, and ignore {}",
                    encrypted_kuutamo_app_file
                        .to_str()
                        .unwrap_or("encrypted_kuutamo_app_file"),
                    node_key_file.to_str().unwrap_or("validator_node_key_file")
                );
            }
            Some(decrypt_and_unzip_file(
                encrypted_kuutamo_app_file,
                ask_password_for(encrypted_kuutamo_app_file)?,
            )?)
        }
        (true, Some(validator_key_file), Some(validator_node_key_file), _) => {
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
        }
        (true, None, Some(_), _) => {
            bail!("hosts.{name} has a validator_node_key_file but not a validator_key_file")
        }
        (true, Some(_), None, _) => {
            bail!("hosts.{name} has a validator_key_file but not a validator_node_key_file")
        }
        (true, None, None, None) => {
            bail!("There is neither a validator_node_key_file nor validator_key_file provided, please check your configuration.")
        }
        _ => None,
    };

    let token_auth = if load_keys {
        fs::read_to_string(
            host.kuutamo_monitoring_token_file
                .as_ref()
                .unwrap_or(&PathBuf::from("kuutamo-monitoring.token")),
        )
        .ok()
        .map(|s| s.trim().into())
        .and_then(|t| decode_token(t).ok())
    } else {
        None
    };

    let kmonitor_config = match (
        &host.self_monitoring_url,
        &host.self_monitoring_username,
        &host.self_monitoring_password,
        token_auth,
    ) {
        (url, Some(username), Some(password), _) if url.is_some() => Some(KmonitorConfig {
            url: url.clone(),
            username: username.to_string(),
            password: password.to_string(),
        }),
        (url, _, _, Some((user_id, password))) if url.is_some() => Some(KmonitorConfig {
            url: url.clone(),
            username: user_id,
            password,
        }),
        (None, _, _, Some((user_id, password))) => {
            try_verify_kuutamo_monitoring_config(
                host.nixos_module.clone().or(default.nixos_module.clone()),
                user_id,
                password,
            )
            .await
        }
        _ => {
            eprintln!("auth information for monitoring is insufficient, will not set up monitoring when deploying");
            None
        }
    };

    let telegraf_has_monitoring = kmonitor_config.is_some();
    let telegraf_config_hash = calculate_hash(&kmonitor_config).to_string();

    Ok(Host {
        name,
        nixos_module,
        extra_nixos_modules,
        install_ssh_user,
        ssh_hostname,
        mac_address,
        interface,
        ipv4_address,
        ipv4_cidr,
        ipv4_gateway,
        ipv6_address,
        ipv6_cidr: mask.or(ipv6_cidr),
        ipv6_gateway,
        validator_keys,
        public_ssh_keys,
        disks,
        kmonitor_config,
        telegraf_has_monitoring,
        telegraf_config_hash,
    })
}

/// Try to access kuutamo monitoring, if auth is invalid the config will drop
async fn try_verify_kuutamo_monitoring_config(
    nixos_module: Option<String>,
    user_id: String,
    password: String,
) -> Option<KmonitorConfig> {
    if let Some(nixos_module) = nixos_module {
        let client = Client::new();
        let username = if nixos_module.ends_with("mainnet") {
            format!("near-mainnet-{}", user_id)
        } else {
            format!("near-testnet-{}", user_id)
        };
        if let Ok(r) = client
            .get("https://mimir.monitoring-00-cluster.kuutamo.computer")
            .basic_auth(&username, Some(&password))
            .send()
            .await
        {
            if r.status() == reqwest::StatusCode::UNAUTHORIZED {
                eprintln!("token for kuutamo monitoring.token is invalid, please check, else the monitor will not work after deploy");
                return None;
            }
        } else {
            eprintln!("Could not validate kuutamo-monitoring.token (network issue)");
        }
        Some(KmonitorConfig {
            url: None,
            username,
            password,
        })
    } else {
        None
    }
}

fn ask_password_for(file: &Path) -> Result<String> {
    let file_name = file
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    println!("Please enter the password for {file_name}");

    let disable_terminal_echo = DisableTerminalEcho::new();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    drop(disable_terminal_echo);

    Ok(line.trim_end().to_string())
}

fn decrypt_and_unzip_file(file: &PathBuf, password: String) -> Result<ValidatorKeys> {
    let mut key_file_content = String::new();
    let mut node_key_file_content = String::new();

    let file_name = file
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let mut archive = fs::File::open(file)
        .map(zip::ZipArchive::new)
        .with_context(|| format!("{file_name:} could not treat as zip archive"))??;

    if let Ok(Ok(mut zip)) = archive
        .by_name_decrypt("validator_key.json", password.as_bytes())
        .with_context(|| format!("password for {file_name:} is incorrect"))
    {
        zip.read_to_string(&mut key_file_content)?;
    }
    if let Ok(Ok(mut zip)) = archive
        .by_name_decrypt("node_key.json", password.as_bytes())
        .with_context(|| format!("password for {file_name:} is incorrect"))
    {
        zip.read_to_string(&mut node_key_file_content)?;
    }

    Ok(ValidatorKeys {
        validator_key: serde_json::from_str(&key_file_content)
            .map_err(|e| SerdeError::new(key_file_content, e))
            .with_context(|| format!("validator key file at '{file_name:}' is not valid"))?,
        validator_node_key: serde_json::from_str(&node_key_file_content)
            .map_err(|e| SerdeError::new(node_key_file_content, e))
            .with_context(|| format!("validator node key file at '{file_name:}' is not valid"))?,
    })
}

fn read_validator_keys(
    validator_key_file: PathBuf,
    validator_node_key_file: PathBuf,
) -> Result<ValidatorKeys> {
    let validator_key = fs::read_to_string(&validator_key_file).with_context(|| {
        format!(
            "cannot read validator key file: '{}'",
            validator_key_file.display()
        )
    })?;

    let validator_node_key = if validator_node_key_file.exists() {
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
                    "validator node key file at '{}' is not valid",
                    validator_node_key_file.display()
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
pub async fn parse_config(
    content: &str,
    working_directory: Option<&Path>,
    load_keys: bool,
) -> Result<Config> {
    let mut config: ConfigFile = toml::from_str(content)?;
    let mut hosts = HashMap::new();
    for (name, host) in config.hosts.iter_mut() {
        hosts.insert(
            name.clone(),
            validate_host(
                name,
                host,
                &config.host_defaults,
                working_directory,
                load_keys,
            )
            .await?,
        );
    }

    let global = validate_global(&config.global)?;
    Ok(Config { hosts, global })
}

/// Load and validate configuration from path
/// The key will not provide without load_keys flag
pub async fn load_configuration(config: &Path, load_keys: bool) -> Result<Config> {
    let content = fs::read_to_string(config).context("Cannot read file")?;
    let working_directory = config.parent();
    parse_config(&content, working_directory, load_keys).await
}

fn decode_token(s: String) -> Result<(String, String)> {
    let binding =
        general_purpose::STANDARD_NO_PAD.decode(s.trim_matches(|c| c == '=' || c == '\n'))?;
    let decode_str = std::str::from_utf8(&binding)?;
    decode_str
        .split_once(':')
        .map(|(u, p)| (u.trim().to_string(), p.trim().to_string()))
        .ok_or(anyhow!("token should be `username: password` pair"))
}

/// Generate kneard.toml example
pub fn generate_example() -> Result<String> {
    let global_example = GlobalConfig::toml_example();
    let host_example = HostConfig::toml_example();
    Ok(format!(
        "[global]\n{global_example}\n[hosts]\n[hosts.example]\n{host_example}"
    ))
}

#[tokio::test]
pub async fn test_parse_config() -> Result<()> {
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

    let config = parse_config(config_str, None, true).await?;
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
        IpAddr::from_str("2605:9880:400::2").ok()
    );
    assert_eq!(hosts["validator-00"].ipv6_cidr, Some(48));
    assert_eq!(
        hosts["validator-00"].ipv6_gateway,
        IpAddr::from_str("2605:9880:400::1").ok()
    );
    let k = hosts["validator-00"].validator_keys.as_ref().unwrap();
    assert_eq!(k.validator_key, NearKeyFile {
        account_id: String::from("kneard"),
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
    let config = parse_config(config_str, None, true).await?;
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
fn test_valid_ip_string_for_ipv6() {
    let ip: IpV6String = "2607:5300:203:5cdf::".into();
    assert_eq!(ip.normalize().unwrap().1, None);

    let subnet_identifire: IpV6String = "2607:5300:203:5cdf::/64".into();
    assert_eq!(
        subnet_identifire.normalize().unwrap().0,
        ip.normalize().unwrap().0
    );
    assert_eq!(subnet_identifire.normalize().unwrap().1, Some(64));
}

#[test]
fn test_invalid_string_for_ipv6() {
    let mut invalid_str: IpV6String = "2607:5300:203:7cdf::/".into();
    assert!(invalid_str.normalize().is_err());

    invalid_str = "/2607:5300:203:7cdf::".into();
    assert!(invalid_str.normalize().is_err());
}

#[tokio::test]
async fn test_validate_host() {
    let mut config = HostConfig {
        ipv4_address: Some("192.168.0.1".parse::<IpAddr>().unwrap()),
        ipv4_cidr: Some(0),
        ipv4_gateway: Some("192.168.255.255".parse::<IpAddr>().unwrap()),
        ipv6_address: None,
        ipv6_gateway: None,
        ipv6_cidr: None,
        public_ssh_keys: vec!["".to_string()],
        ..Default::default()
    };
    assert_eq!(
        validate_host("ipv4-only", &config, &HostConfig::default(), None, true)
            .await
            .unwrap(),
        Host {
            name: "ipv4-only".to_string(),
            nixos_module: "single-node-validator-mainnet".to_string(),
            extra_nixos_modules: Vec::new(),
            mac_address: None,
            interface: "eth0".to_string(),
            ipv4_address: "192.168.0.1".parse::<IpAddr>().unwrap(),
            ipv4_cidr: 0,
            ipv4_gateway: "192.168.255.255".parse::<IpAddr>().unwrap(),
            ipv6_address: None,
            ipv6_cidr: None,
            ipv6_gateway: None,
            install_ssh_user: "root".to_string(),
            ssh_hostname: "192.168.0.1".to_string(),
            public_ssh_keys: vec!["".to_string()],
            disks: vec!["/dev/nvme0n1".into(), "/dev/nvme1n1".into()],
            validator_keys: None,
            kmonitor_config: None,
            telegraf_has_monitoring: false,
            telegraf_config_hash: "13646096770106105413".to_string(),
        }
    );

    // If `ipv6_address` is provied, the `ipv6_gateway` and `ipv6_cidr` should be provided too,
    // else the error will raise
    config.ipv6_address = Some("2607:5300:203:6cdf::".into());
    assert!(
        validate_host("ipv4-only", &config, &HostConfig::default(), None, true)
            .await
            .is_err()
    );

    config.ipv6_gateway = Some(
        "2607:5300:0203:6cff:00ff:00ff:00ff:00ff"
            .parse::<IpAddr>()
            .unwrap(),
    );
    assert!(
        validate_host("ipv4-only", &config, &HostConfig::default(), None, true)
            .await
            .is_err()
    );

    // The `ipv6_cidr` could be provided by subnet in address field
    config.ipv6_address = Some("2607:5300:203:6cdf::/64".into());
    assert!(
        validate_host("ipv4-only", &config, &HostConfig::default(), None, true)
            .await
            .is_ok()
    );
}

#[test]
pub fn test_decrypt_and_unzip_file() {
    let keys = decrypt_and_unzip_file(
        &PathBuf::from("tests/assets/hello.pool.devnet.zip"),
        "1234".into(),
    )
    .unwrap();

    assert_eq!(keys.validator_key, NearKeyFile {
        account_id: String::from("hello.pool.devnet"),
        public_key: String::from("ed25519:4TfQz1x8LVn332JokpkDU5sUroSFwqhpP9ezVyXqPGst"),
        secret_key: String::from("ed25519:5A8GbPvqKbU5iSszR8Q6oZnXa9WuNz6futH3ZJ5WMHnV8nyssieo9NpdubPtmC1ftgM9a9q7RLMwD3wjktvWihaa"),
    });

    assert_eq!(keys.validator_node_key, NearKeyFile {
        account_id: String::from("node"),
        public_key: String::from("ed25519:8Qn4z2ir1akQuS1nZB5RjyLt4fiw7rH626pk5Qv3pvtv"),
        secret_key: String::from("ed25519:4UzKpf3gP8FtvpED8BE3YWUhc5JVNnwptdBZHCe5ZKLm9jXtec2JQ3TNzFFPLUFshcr8yyDBTPN2tej3CgjNctEi"),
    });
}

#[test]
fn test_token_decode() {
    let token = "MjoyRE5HTWx6YWxoQVQzN0M1NVgwSG53WFFkZlpjZG5CZXhXVFJobEloRVFvVTluRW8=";
    assert_eq!(
        decode_token(token.to_string()).unwrap(),
        (
            "2".to_string(),
            "2DNGMlzalhAT37C55X0HnwXQdfZcdnBexWTRhlIhEQoU9nEo".to_string()
        )
    )
}
