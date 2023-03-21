//! Methods to start and stop neard

use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use log::warn;
use nix::errno::Errno;
use nix::sys::signal::{kill, SIGCHLD, SIGTERM};
use nix::sys::signal::{sigprocmask, SigmaskHow, SIGKILL};
use nix::sys::signalfd::SigSet;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use prometheus::{register_int_counter, IntCounter};
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Mutex;
use std::{io, ptr};
use tokio::process::Child;
use tokio::process::Command;
use tokio::time::Duration;

use crate::oom_score;

/// How much time we give neard to exit. We give it some time to sync rocksdb to disk.
const NEARD_STOP_TIMEOUT: Duration = Duration::from_secs(60);

lazy_static! {
    static ref NEARD_RESTARTS: IntCounter = register_int_counter!(
        "kneard_neard_restarts",
        "How often neard has been restarted"
    )
    .unwrap();
}

lazy_static! {
    static ref NEARD_PID: Mutex<Option<Pid>> = Mutex::new(None);
}

/// Get Pid of latest started neard instance
pub fn get_neard_pid() -> Result<Option<Pid>> {
    match NEARD_PID.lock() {
        Ok(v) => Ok(*v),
        Err(e) => {
            bail!("Cannot get pid lock: {}", e)
        }
    }
}

fn set_neard_pid(p: Option<Pid>) {
    match NEARD_PID.lock() {
        Ok(mut lock) => {
            *lock = p;
        }
        Err(e) => {
            warn!("Cannot take near pid lock: {}", e)
        }
    }
}

fn reset_oom_score() -> io::Result<()> {
    let r = oom_score::adjust_oom_score(oom_score::DEFAULT_OOM_SCORE);
    if let Err(ref e) = r {
        warn!("Failed to reset oom score: {}", e);
    }
    r
}

/// Starts a neard daemon for the given home
pub fn run_neard(neard_home: &Path, boot_nodes: &Option<String>) -> Result<Child> {
    let mut args = vec![
        OsStr::new("--home"),
        neard_home.as_os_str(),
        OsStr::new("run"),
    ];
    if let Some(v) = boot_nodes {
        args.push(OsStr::new("--boot-nodes"));
        args.push(OsStr::new(v.as_str()));
    };
    let proc = unsafe {
        Command::new("neard")
            .args(args)
            .pre_exec(reset_oom_score)
            .spawn()
            .with_context(|| {
                format!(
                    "failed to spawn `neard --home {} run`",
                    neard_home.display()
                )
            })
    };
    if let Ok(ref p) = proc {
        set_neard_pid(p.id().map(|id| Pid::from_raw(id as i32)));
    }
    proc
}

/// Stops a process by first sending SIGTERM and than after `NEARD_STOP_TIMEOUT`
pub fn graceful_stop_neard(process: &mut Child) -> Result<()> {
    let pid = match process.id() {
        None => {
            // pid is empty, process have been already stopped and reapped!
            return Ok(());
        }
        Some(pid) => Pid::from_raw(pid as i32),
    };

    NEARD_RESTARTS.inc();
    set_neard_pid(None);

    kill(pid, SIGTERM).context("SIGTERM failed")?;
    let mut mask = SigSet::empty();
    let mut old_mask = SigSet::empty();
    mask.add(SIGCHLD);
    mask.thread_block().unwrap();
    sigprocmask(SigmaskHow::SIG_BLOCK, Some(&mask), Some(&mut old_mask))
        .context("sigprocmask failed")?;

    let timeout = nix::libc::timespec {
        tv_sec: NEARD_STOP_TIMEOUT.as_secs() as i64,
        tv_nsec: 0,
    };

    // wait for neard to finish or for timeout
    let res = unsafe {
        loop {
            let r = nix::libc::sigtimedwait(mask.as_ref(), ptr::null_mut(), &timeout);
            if r != nix::libc::EAGAIN {
                break r;
            }
        }
    };
    // reset sigprocmask to old value
    if let Err(e) = sigprocmask(SigmaskHow::SIG_SETMASK, Some(&old_mask), None) {
        warn!("Failed to restore old signal mask: {}", e);
    }

    if res < 0 {
        warn!("sigtimedwait failed: {}", Errno::from_i32(-res));
    } else if res == nix::libc::SIGCHLD {
        // We got a signal that a child was finished,
        // Should be the neard one...
        // Let's check
        let ret = waitpid(pid, Some(WaitPidFlag::WNOHANG));
        match ret {
            Ok(WaitStatus::Exited(_, _)) => return Ok(()),
            Ok(WaitStatus::Signaled(_, _, _)) => return Ok(()),
            Ok(WaitStatus::StillAlive) => {
                warn!("neard process is still alive also we got SIGCHLD!");
            }
            Ok(WaitStatus::Stopped(_, _)) => {
                warn!("neard process was stopped instead of terminated!");
            }
            Ok(WaitStatus::Continued(_)) => {
                warn!("neard process was continued instead of terminated!");
            }
            Ok(WaitStatus::PtraceSyscall(_)) => {
                warn!("neard process was ptraced instead of terminated!");
            }
            Ok(WaitStatus::PtraceEvent(_, _, _)) => {
                warn!("neard process was ptraced instead of terminated!");
            }
            Err(e) => {
                warn!("Failed to collect exit status of neard: {}", e);
                return Ok(());
            }
        }
        warn!("Neard still not terminated. Send SIGKILL to neard!");
    } else {
        warn!("Termination timeout reached. Send SIGKILL to neard!");
    }

    kill(pid, SIGKILL).context("SIGKILL failed")?;
    let _ = waitpid(pid, None);
    Ok(())
}
