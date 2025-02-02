//! Command to kneard

/// Command to kneard
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(clap::Subcommand, PartialEq, Debug, Clone)]
pub enum Command {
    /// Setup restart in maintenance window, and wait for complete
    Restart(RestartArgs),

    /// Show the status of maintenance restart
    MaintenanceStatus,

    /// Show the current voted validator
    ActiveValidator,

    /// Check the status of rpc service
    CheckRpc(CheckRpcArgs),

    /// Show system info
    SystemInfo(SystemInfoArgs),
}

/// Arguments for restart command
#[derive(clap::Args, PartialEq, Debug, Clone)]
pub struct RestartArgs {
    /// Specify the minimum length in blocks for the maintenance shutdown, if not provided,
    /// neard will try to shutdown in the longest maintenance window in the current epoch
    pub minimum_length: Option<u64>,

    /// Specify the block height to shutdown at, and will not check on it in maintenance window or
    /// not.
    #[arg(long)]
    pub schedule_at: Option<u64>,

    /// Cancel the maintenance restart setting
    #[arg(long)]
    pub cancel: bool,

    /// Cli will wait for restart
    #[clap(long)]
    pub wait: bool,
}

/// Arguments for check rpc command
#[derive(clap::Args, PartialEq, Debug, Clone)]
pub struct CheckRpcArgs {
    /// Cli will keep blocking when rpc service up
    #[clap(long)]
    pub watch: bool,
}

/// Arguments for system info command
#[derive(clap::Args, PartialEq, Debug, Clone)]
pub struct SystemInfoArgs {
    /// Cli will show system info with inline format
    #[clap(long)]
    pub inline: bool,
}
