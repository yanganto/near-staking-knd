use crate::near_client::NeardClient;
use crate::neard_process::NeardProcess;
use anyhow::{bail, Result};
use log::info;
use near_primitives::types::BlockHeight;
use near_primitives::views::StatusResponse;
use nix::unistd::Pid;
use std::path::PathBuf;

/// The default minimum window length for maintenance we need, else we will ignore the small windows
static MINIMUN_MAINTENANCE: u64 = 60;

/// Book maintenance shutdown and return the block height the shutdown neard will shutdown or None
/// If there is no window in current epoch for it will raise error, such that we can base on error
/// and retry it on next epoch
pub(crate) async fn execute(
    near_rpc_port: u16,
    pid: Option<Pid>,
    dyn_config_path: &PathBuf,
    minimum_length: Option<u64>,
) -> Result<Option<BlockHeight>> {
    let neard_client = NeardClient::new(&format!("http://127.0.0.1:{}", near_rpc_port))?;
    match (neard_client.status().await, pid) {
        (Err(_), _) => {
            bail!("Neard node did not open RPC, fail to fetch maintenance window")
        }
        (
            Ok(StatusResponse {
                validator_account_id: Some(validator_account_id),
                ..
            }),
            Some(p),
        ) => {
            let windows = neard_client
                .maintenance_windows(validator_account_id)
                .await?;
            let mut largest_window_length = 0;
            let mut largest_window_start = None;
            for window in windows.0.iter() {
                let new_window_length = window.1 - window.0;
                if new_window_length > largest_window_length {
                    largest_window_length = new_window_length;
                    largest_window_start = Some(window.0)
                }
            }

            let expect_shutdown_at = largest_window_start.map(|b| b + 2); // shutdown on final block
            if let Some(expect_shutdown_at) = expect_shutdown_at {
                if largest_window_length >= minimum_length.unwrap_or(MINIMUN_MAINTENANCE) {
                    NeardProcess::update_dynamic_config(p, dyn_config_path, expect_shutdown_at)
                        .await?;
                } else {
                    bail!("Current maintenance windows in current epoch are too small , please wait for next");
                }
            } else {
                // TODO need to disscuss
                bail!("Neard have no maintenance window in current epoch, please wait");
            }

            Ok(expect_shutdown_at)
        }
        (
            Ok(StatusResponse {
                validator_account_id: None,
                ..
            }),
            Some(p),
        ) => {
            NeardProcess::restart(p).await?;
            info!("Neard is not a validator. No need wait maintenance window, neard restarting");
            Ok(None)
        }
        (Ok(_), None) => {
            info!("Neard non-existing, may restarting by kuutamod, please wait");
            Ok(None)
        }
    }
}
