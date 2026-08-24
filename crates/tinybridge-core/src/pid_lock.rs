//! PID-file/lock registry for detecting orphaned processes after a crash or
//! force-kill.
//!
//! The daemon (`tinybridge-daemon/src/daemon.rs`) and the per-VM host
//! process (`tinybridge-vmhost/src/socket_server.rs`) used to blindly
//! `remove_file` a stale Unix control socket on startup with no check for
//! whether the process that created it is still alive. If a previous
//! instance was force-killed (SIGKILL can't run cleanup code) mid-flight,
//! its live socket file could be yanked out from under it by a next
//! instance racing to start -- or worse, a genuinely still-running
//! instance's socket removed while it's actively serving connections.
//!
//! This writes a `<socket_path>.pid` file alongside the socket containing
//! the owning process's PID, and on startup checks whether that PID is
//! actually still alive (not just "the file exists") before deciding
//! whether cleanup is safe.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Holds the lock for a control socket's lifetime. Dropping it removes the
/// PID file (best-effort) so a clean shutdown doesn't leave a lock file
/// behind for the next start to have to reason about.
pub struct PidLock {
    pid_path: PathBuf,
}

impl PidLock {
    /// Acquire the lock for `socket_path` (writes `<socket_path>.pid`).
    ///
    /// If a PID file already exists and names a process that's still
    /// alive, returns an error rather than proceeding -- there is a real,
    /// running instance already using this socket, and blindly deleting
    /// its socket file out from under it would break that instance. If the
    /// PID file is missing, corrupt, or names a process that's no longer
    /// running, it's treated as an orphan from a previous crash/force-kill
    /// and cleaned up automatically (logged, not silent).
    pub fn acquire(socket_path: &Path) -> Result<Self> {
        let pid_path = pid_file_path(socket_path);

        if let Some(existing_pid) = read_pid_file(&pid_path) {
            if is_process_alive(existing_pid) {
                bail!(
                    "another instance is already running (pid {existing_pid}, socket {}); \
                     refusing to remove its control socket",
                    socket_path.display()
                );
            }
            tracing::warn!(
                pid = existing_pid,
                socket = %socket_path.display(),
                "found an orphaned lock file from a process that is no longer running \
                 (likely a previous crash or force-kill); cleaning up"
            );
        }

        if socket_path.exists() {
            std::fs::remove_file(socket_path)
                .with_context(|| format!("removing stale socket {}", socket_path.display()))?;
        }

        let pid = std::process::id();
        std::fs::write(&pid_path, pid.to_string())
            .with_context(|| format!("writing pid lock file {}", pid_path.display()))?;

        Ok(PidLock { pid_path })
    }
}

impl Drop for PidLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.pid_path);
    }
}

fn pid_file_path(socket_path: &Path) -> PathBuf {
    let mut path = socket_path.as_os_str().to_owned();
    path.push(".pid");
    PathBuf::from(path)
}

fn read_pid_file(pid_path: &Path) -> Option<u32> {
    std::fs::read_to_string(pid_path).ok()?.trim().parse().ok()
}

/// Check whether `pid` names a live process. Shells out to `ps` rather than
/// an FFI `kill(pid, 0)` call -- this only runs once at startup (not a hot
/// path), and using `ps` avoids adding an `unsafe` libc dependency just for
/// this, while working identically on macOS and Linux.
fn is_process_alive(pid: u32) -> bool {
    Command::new("ps")
        .args(["-p", &pid.to_string()])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;

    #[test]
    fn acquire_writes_a_pid_file_readable_back() {
        let dir = tempdir();
        let socket_path = dir.join("tb-test.sock");

        let lock = PidLock::acquire(&socket_path).unwrap();
        let pid_path = pid_file_path(&socket_path);
        let recorded = read_pid_file(&pid_path).unwrap();

        assert_eq!(recorded, std::process::id());
        drop(lock);
        assert!(!pid_path.exists(), "Drop should remove the lock file");
    }

    #[test]
    fn acquire_removes_a_stale_socket_left_by_a_dead_process() {
        let dir = tempdir();
        let socket_path = dir.join("tb-test.sock");
        let pid_path = pid_file_path(&socket_path);

        // Simulate a crashed prior instance: a socket file and a PID file
        // naming a process that has already exited.
        std::fs::write(&socket_path, b"").unwrap();
        let dead_pid = spawn_and_wait_for_exit();
        std::fs::write(&pid_path, dead_pid.to_string()).unwrap();

        let lock = PidLock::acquire(&socket_path);

        assert!(lock.is_ok(), "should clean up and succeed, not error out");
        assert!(!socket_path.exists(), "stale socket should be removed");
    }

    #[test]
    fn acquire_refuses_when_the_recorded_pid_is_still_alive() {
        let dir = tempdir();
        let socket_path = dir.join("tb-test.sock");
        let pid_path = pid_file_path(&socket_path);

        std::fs::write(&socket_path, b"").unwrap();
        // Our own PID is definitely alive right now.
        std::fs::write(&pid_path, std::process::id().to_string()).unwrap();

        let result = PidLock::acquire(&socket_path);

        assert!(result.is_err());
        assert!(
            socket_path.exists(),
            "must not remove a live instance's socket"
        );
    }

    #[test]
    fn acquire_treats_a_corrupt_pid_file_as_an_orphan() {
        let dir = tempdir();
        let socket_path = dir.join("tb-test.sock");
        let pid_path = pid_file_path(&socket_path);

        std::fs::write(&socket_path, b"").unwrap();
        std::fs::write(&pid_path, "not-a-number").unwrap();

        let result = PidLock::acquire(&socket_path);

        assert!(result.is_ok());
    }

    #[test]
    fn is_process_alive_true_for_self() {
        assert!(is_process_alive(std::process::id()));
    }

    #[test]
    fn is_process_alive_false_for_an_exited_process() {
        let pid = spawn_and_wait_for_exit();
        assert!(!is_process_alive(pid));
    }

    fn spawn_and_wait_for_exit() -> u32 {
        let mut child = Command::new("true")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawning `true` should succeed on macOS/Linux test runners");
        let pid = child.id();
        child.wait().unwrap();
        pid
    }

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tinybridge-pid-lock-test-{}-{}",
            std::process::id(),
            uuid_like()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn uuid_like() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}
