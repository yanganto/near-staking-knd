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
struct ScheduleRestartOperation {
    /// Specify the minimum length in blocks of the maintenance window, if not provided,
    /// neard will try to restart in the longest maintenance window in the current epoch
    minimum_length: Option<u64>,

    /// Cancel the schedule
    #[arg(long)]
    pub cancel: bool,

    /// Specify the block height to schedule at, and will not check on it in maintenance window or
    /// not.
    #[arg(long)]
    schedule_at: Option<u64>,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
struct ApiResponse {
    status: u16,
    message: String,
}
