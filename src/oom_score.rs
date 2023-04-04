//! Module to modify oom scores on Linux

use std::fs::OpenOptions;
use std::io;
use std::io::Write;

/// The default score for processes on Linux
pub const DEFAULT_OOM_SCORE: u32 = 200;
/// The score that kneard uses
pub const KUUTAMOD_OOM_SCORE: u32 = 100;

/// Adjust the process specific oom score. The lower the oom score the less likely a process get killed.
pub fn adjust_oom_score(score: u32) -> io::Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .open("/proc/self/oom_score_adj")?;
    f.write_all(format!("{score}").as_bytes())?;
    Ok(())
}
