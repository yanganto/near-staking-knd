//! Command to kuutamod

/// Command to kuutamod
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(clap::Subcommand, PartialEq, Debug, Clone)]
pub enum Command {
    /// Initiate maintenance shutdown
    MaintenanceShutdown(MaintenanceShutdownArgs),
    /// Show the current voted validator
    ActiveValidator,
}

/// Arguments for maintenance shutdonw command
#[derive(clap::Args, PartialEq, Debug, Clone)]
pub struct MaintenanceShutdownArgs {
    /// Specify the minimum length in blockheight for the maintenance shutdown
    pub minimum_length: Option<u64>,

    /// Specify the block height to shutdown at, and will not check on it in maintenance window or
    /// not.
    #[arg(long)]
    pub shutdown_at: Option<u64>,

    /// The host to do maintenance_shutdown
    #[clap(long, default_value = "")]
    pub host: String,
}
