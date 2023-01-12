use anyhow::{bail, Context, Result};
use std::process::ExitStatus;

/// Human-friendly error messages for failed programs
pub fn status_to_pretty_err<E>(
    res: std::result::Result<ExitStatus, E>,
    command: &str,
    args: &[&str],
) -> Result<()>
where
    E: Send + 'static,
    E: Sync,
    E: std::error::Error,
{
    let status = res.with_context(|| format!("failed to start this command: {}", command))?;
    if status.success() {
        return Ok(());
    }
    match status.code() {
        Some(code) => bail!(
            "command {} failed ({} {}) with exit code: {}",
            command,
            command,
            args.join(" "),
            code
        ),
        None => bail!(
            "command {} ({} {}) was terminated by a signal",
            command,
            command,
            args.join(" ")
        ),
    }
}
