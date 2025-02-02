//! For sending messages to the supervisor

use anyhow::Result;
use near_primitives::types::BlockHeight;
use tokio::sync::mpsc;

/// Response from the supervisor
pub struct ScheduleRestartOperationResponse {
    /// When to shutdown.
    /// Error if shutdown could not be scheduled.
    /// None if neard process is not a validator and an intermediate shutdown was requested
    pub shutdown_at_blockheight: Result<Option<BlockHeight>>,
}

/// Request to send to the supervisor
pub enum Request {
    /// Schedule maintenance shutdown for
    ///     * a maintainace window of given size,
    ///     * book a block height to shutdown at,
    ///     * cancel previous schedule
    ///     + Channel where the supervisor will respond to once the request is finished
    ScheduleRestartOperation(
        Option<u64>,
        Option<u64>,
        bool,
        mpsc::Sender<ScheduleRestartOperationResponse>,
    ),
}
