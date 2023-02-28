//! Control socket server

mod active_validator;
mod client;
pub mod control_commands;
mod server;

use serde::{Deserialize, Serialize};

use clap::Parser;
pub use client::CommandClient;
pub use server::spawn_control_server;

#[derive(Parser, PartialEq, Serialize, Deserialize, Debug, Clone)]
struct MaintenanceShutdown {
    /// Specify the minimum length in blockheight for the maintenance shutdown, if not provided,
    /// neard will try to shutdown in the longest maintenance window in the current epoch
    minimum_length: Option<u64>,

    /// Cancel the maintenance shutdwon setting
    #[arg(long)]
    pub cancel: bool,

    /// Specify the block height to shutdown at, and will not check on it in maintenance window or
    /// not.
    #[arg(long)]
    shutdown_at: Option<u64>,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
struct ApiResponse {
    status: u16,
    message: String,
}
