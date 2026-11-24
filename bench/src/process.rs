//! One process run and measured as the kernel accounts for it: the wall time from before it was
//! started to when it was reaped, and its peak resident memory (`ru_maxrss` of `wait4`), which
//! counts the process and any child it waited for. A process past its time limit is killed.
//!
//! `ru_maxrss` is in kilobytes on Linux and in bytes on macOS; both are given here in KB. The
//! bench is run and its results taken on Linux.

use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

/// How a run ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    /// It exited with this code.
    Exited(i32),
    /// A signal ended it.
    Signalled(i32),
    /// It ran past its time limit and was killed.
    TimedOut,
}

/// What one run cost.
#[derive(Clone, Copy, Debug)]
pub struct Run {
    pub wall: Duration,
    pub peak_rss_kb: u64,
    pub ending: Ending,
}

impl Run {
    /// Whether it exited with 0.
    pub fn succeeded(&self) -> bool {
        self.ending == Ending::Exited(0)
    }

    /// What went wrong, in a few words; empty when nothing did.
    pub fn failure(&self, limit: Duration) -> String {
        match self.ending {
            Ending::Exited(0) => String::new(),
            Ending::Exited(code) => format!("exited with {code}"),
            Ending::Signalled(signal) => format!("ended by signal {signal}"),
            Ending::TimedOut => format!("killed after {} s", limit.as_secs()),
        }
    }
}

/// Runs `command`, its standard output and error both into `log`, and waits for it at most
/// `limit`.
pub fn measure(command: &mut Command, log: &Path, limit: Duration) -> std::io::Result<Run> {
    let out = File::create(log)?;
    let err = out.try_clone()?;
    command.stdin(Stdio::null()).stdout(out).stderr(err);
    let start = Instant::now();
    let child = command.spawn()?;
    let pid = libc::pid_t::try_from(child.id()).map_err(std::io::Error::other)?;
    let (done, until) = mpsc::channel::<()>();
    let watchdog = thread::spawn(move || match until.recv_timeout(limit) {
        Err(RecvTimeoutError::Timeout) => {
            // SAFETY: `kill` takes a pid and a signal and touches no memory of ours; the child
            // has not been reaped yet, so the pid is still its own.
            unsafe { libc::kill(pid, libc::SIGKILL) };
            true
        }
        Ok(()) | Err(RecvTimeoutError::Disconnected) => false,
    });
    let (status, usage) = reap(pid)?;
    let wall = start.elapsed();
    let _ = done.send(());
    let timed_out = watchdog.join().unwrap_or(false);
    drop(child);
    let ending = if timed_out {
        Ending::TimedOut
    } else if libc::WIFEXITED(status) {
        Ending::Exited(libc::WEXITSTATUS(status))
    } else {
        Ending::Signalled(libc::WTERMSIG(status))
    };
    Ok(Run {
        wall,
        peak_rss_kb: kilobytes(usage.ru_maxrss),
        ending,
    })
}

/// Waits for `pid` to end: its status and what it used.
fn reap(pid: libc::pid_t) -> std::io::Result<(i32, libc::rusage)> {
    let mut status = 0;
    // SAFETY: an all-zero `rusage` is a valid value of the plain C struct.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: both pointers are to locals that outlive the call.
        let reaped = unsafe { libc::wait4(pid, &mut status, 0, &mut usage) };
        if reaped == pid {
            return Ok((status, usage));
        }
        let err = std::io::Error::last_os_error();
        if err.kind() != std::io::ErrorKind::Interrupted {
            return Err(err);
        }
    }
}

#[cfg(target_os = "macos")]
fn kilobytes(maxrss: libc::c_long) -> u64 {
    u64::try_from(maxrss).unwrap_or(0) / 1024
}

#[cfg(not(target_os = "macos"))]
fn kilobytes(maxrss: libc::c_long) -> u64 {
    u64::try_from(maxrss).unwrap_or(0)
}

/// The standard output of a short command, trimmed; `None` when it cannot be run or fails.
pub fn output(program: &str, args: &[&str], dir: &Path) -> Option<String> {
    let out = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}
