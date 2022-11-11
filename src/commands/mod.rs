//! Control socket server

mod active_validator;
mod client;
mod server;

use serde::{Deserialize, Serialize};

pub use client::CommandClient;
pub use server::spawn_control_server;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
struct MaintenanceShutdown {
    /// Specify the minimum length in blockheight for the maintenance shutdown
    minimum_length: Option<u64>,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
struct ApiResponse {
    status: u16,
    message: String,
}
