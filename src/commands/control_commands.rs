//! Command to kneard

/// Command to kneard
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(clap::Subcommand, PartialEq, Debug, Clone)]
pub enum Command {
    /// Initiate maintenance shutdown, and wait for complete
    MaintenanceShutdown(MaintenanceOperationArgs),

    /// Initiate maintenance restart
    MaintenanceRestart(MaintenanceOperationArgs),

    /// Show the status of maintenance shutdown / restart
    MaintenanceStatus,

    /// Show the current voted validator
    ActiveValidator,
}

/// Arguments for maintenance shutdonw command
#[derive(clap::Args, PartialEq, Debug, Clone)]
pub struct MaintenanceOperationArgs {
    /// Specify the minimum length in blocks for the maintenance shutdown, if not provided,
    /// neard will try to shutdown in the longest maintenance window in the current epoch
    pub minimum_length: Option<u64>,

    /// Specify the block height to shutdown at, and will not check on it in maintenance window or
    /// not.
    #[arg(long)]
    pub shutdown_at: Option<u64>,

    /// Cancel the maintenance shutdwon setting
    #[arg(long)]
    pub cancel: bool,

    /// The host to do maintenance_shutdown
    #[clap(long, default_value = "")]
    pub host: String,

    /// Cli will wait for shutdown/restart
    #[clap(long)]
    pub wait: bool,
}
