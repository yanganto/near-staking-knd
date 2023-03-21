//! Consul client implementation

#![deny(missing_docs)]
//! kneard executable

use anyhow::bail;
use anyhow::Result;
use kneard::commands::spawn_control_server;
use kneard::prometheus::spawn_prometheus_exporter;
use kneard::settings::parse_settings;
use kneard::supervisor::run_supervisor;
use log::warn;
use std::sync::Arc;
use tokio::sync::mpsc;

/// The kneard program entry point
#[tokio::main]
pub async fn main() -> Result<()> {
    let settings = Arc::new(parse_settings()?);

    if let Err(e) = kneard::log_fmt::init(&settings.node_id) {
        bail!("Failed to setup logger: {:?}", e);
    };

    let (tx, rx) = mpsc::channel(1);

    tokio::select!(
        res = run_supervisor(&settings, rx) => {
            if let Err(e) = res {
                warn!("supervisor failed: {}", e);
                return Err(e);
            }
            res
        }
        res = spawn_prometheus_exporter(&settings.exporter_address) => {
            if let Err(e) = res {
                warn!("prometheus exporter failed: {}", e);
                return Err(e);
            }
            res
        }
        res = spawn_control_server(&settings, tx) => {
            if let Err(e) = res {
                warn!("control socket server failed: {}", e);
                return Err(e);
            }
            res
        }
    )
}
