//! Liveness evidence for the external SWE_SEED command (issue #86).
//!
//! The command is spawned as its own process group, so its group id equals the leader's pid. Right after the spawn
//! the caller records the group id, the leader's start time and the boot id on the claimed attempt. An operator
//! resolving that attempt as `NotDeclared` is refused while any process of the recorded group still exists on the
//! same boot, because that process may still append its declaration to the external authority.
//!
//! Everything here is `std` only: Linux reads `/proc`; other Unix hosts run `ps`. No `unsafe`.
//!
//! The check fails closed. Only a missing `/proc` entry (ENOENT/ESRCH) means a process is gone; any other read
//! error, an unparseable line or a failing `ps` is `Unknown`. Zombies (state `Z`/`X`) have exited and do not count.
//!
//! Bounds: a process that left the group (`setsid`/`setpgid`), a resolver in a different pid namespace on the same
//! boot, and the instant between `fork` and the ledger write are not seen. Only an `Unknown` result (or a malformed
//! launch record) can be overridden, by the operator's explicit, audited `--attest-command-gone`. A group that is
//! positively alive is always refused; the operator must wait for it to exit or end it.

use serde::{Deserialize, Serialize};

/// What is recorded about one launched command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandLaunch {
    /// The process group id, which is the leader's pid.
    pub pgid: u32,
    /// The leader's start time as the platform reports it, when it could be read.
    pub leader_starttime: Option<String>,
    /// An identifier of the current boot, when it could be read.
    pub boot_id: Option<String>,
}

/// Whether the recorded process group still has members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupLiveness {
    Alive,
    Gone,
    /// The check could not run; the caller must fail closed.
    Unknown(String),
}

impl CommandLaunch {
    /// Capture the identity of the group led by `pgid`, right after the spawn.
    pub fn capture(pgid: u32) -> Self {
        Self {
            pgid,
            leader_starttime: platform::process_starttime(pgid),
            boot_id: platform::boot_id(),
        }
    }

    /// Whether any live process of the recorded group exists now.
    pub fn liveness(&self) -> GroupLiveness {
        // On Linux the boot id is exact. Elsewhere it derives from the wall clock, which can step, so a
        // difference proves nothing there and the group itself decides.
        if cfg!(target_os = "linux") {
            if let (Some(recorded), Some(current)) = (&self.boot_id, platform::boot_id()) {
                if *recorded != current {
                    return GroupLiveness::Gone; // every process of an earlier boot is gone
                }
            }
        }
        let members = match platform::group_members(self.pgid) {
            Ok(members) => members,
            Err(reason) => return GroupLiveness::Unknown(reason),
        };
        if members.is_empty() {
            return GroupLiveness::Gone;
        }
        // The leader still exists under its pid. A different start time means the pid was reused (Linux: the
        // kernel never reuses a pid still in use as a group id, so our group is gone) or, off Linux, that the
        // clock stepped. Those cannot be told apart there, so fail closed.
        if members.contains(&self.pgid) {
            if let Some(recorded) = &self.leader_starttime {
                match platform::process_starttime(self.pgid) {
                    Some(current) if current == *recorded => {}
                    Some(_) if cfg!(target_os = "linux") => return GroupLiveness::Gone,
                    Some(_) => {
                        return GroupLiveness::Unknown(
                            "a member of the group has a different start time than recorded (pid reuse or a clock step)".into(),
                        )
                    }
                    None => {
                        return GroupLiveness::Unknown("cannot read the group leader's start time".into())
                    }
                }
            }
        }
        GroupLiveness::Alive
    }
}

/// Parse `/proc/<pid>/stat`: `pid (comm) state ppid pgrp ... starttime ...`. `comm` may contain spaces and
/// parentheses, so the fields start after the last `)`. Returns `(state, pgrp, starttime)`.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_proc_stat(text: &str) -> Result<(char, u32, String), String> {
    let rest = text
        .rfind(')')
        .map(|i| &text[i + 1..])
        .ok_or("a /proc stat line has no `)`")?;
    let fields: Vec<&str> = rest.split_whitespace().collect();
    let state = fields
        .first()
        .and_then(|f| f.chars().next())
        .ok_or("a /proc stat line has no state")?;
    let pgrp = fields
        .get(2)
        .and_then(|f| f.parse::<u32>().ok())
        .ok_or("a /proc stat line has no numeric process group")?;
    let starttime = fields
        .get(19)
        .ok_or("a /proc stat line has no start time")?
        .to_string();
    Ok((state, pgrp, starttime))
}

/// A process that has exited but not been reaped (`Z`), or is being torn down (`X`), runs no code.
#[cfg_attr(not(unix), allow(dead_code))]
fn has_exited(state: char) -> bool {
    matches!(state, 'Z' | 'X' | 'x')
}

/// Parse `ps -A -o pid=,pgid=,stat=` and return the live members of `pgid`. Any unparseable line is an error.
#[cfg_attr(any(target_os = "linux", not(unix)), allow(dead_code))]
fn parse_ps_members(text: &str, pgid: u32) -> Result<Vec<u32>, String> {
    let mut members = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split_whitespace();
        let (Some(pid), Some(group), Some(stat)) = (
            parts.next().and_then(|p| p.parse::<u32>().ok()),
            parts.next().and_then(|p| p.parse::<u32>().ok()),
            parts.next().and_then(|p| p.chars().next()),
        ) else {
            return Err(format!("unparseable ps line: {line:?}"));
        };
        if group == pgid && !has_exited(stat) {
            members.push(pid);
        }
    }
    Ok(members)
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{has_exited, parse_proc_stat};
    use std::path::Path;

    pub fn boot_id() -> Option<String> {
        std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .ok()
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
    }

    /// Only these mean the process is gone; every other error is unknown.
    fn is_gone(error: &std::io::Error) -> bool {
        error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(3)
    }

    pub fn process_starttime(pid: u32) -> Option<String> {
        let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        parse_proc_stat(&text)
            .ok()
            .map(|(_, _, starttime)| starttime)
    }

    pub fn group_members(pgid: u32) -> Result<Vec<u32>, String> {
        group_members_in(Path::new("/proc"), pgid)
    }

    pub(super) fn group_members_in(proc_root: &Path, pgid: u32) -> Result<Vec<u32>, String> {
        // With `hidepid` set, other users' processes are invisible, so absence proves nothing.
        if !proc_root.join("1/stat").exists() {
            return Err("/proc does not show other users' processes (hidepid?)".into());
        }
        let dir =
            std::fs::read_dir(proc_root).map_err(|error| format!("cannot read /proc: {error}"))?;
        let mut members = Vec::new();
        for entry in dir {
            let entry = entry.map_err(|error| format!("cannot list /proc: {error}"))?;
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|n| n.parse::<u32>().ok())
            else {
                continue;
            };
            let text = match std::fs::read_to_string(entry.path().join("stat")) {
                Ok(text) => text,
                Err(error) if is_gone(&error) => continue, // exited between the listing and the read
                Err(error) => return Err(format!("cannot read /proc/{pid}/stat: {error}")),
            };
            let (state, group, _) =
                parse_proc_stat(&text).map_err(|error| format!("/proc/{pid}/stat: {error}"))?;
            if group == pgid && !has_exited(state) {
                members.push(pid);
            }
        }
        Ok(members)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn proc_with(entries: &[(&str, &str)]) -> tempfile::TempDir {
            let dir = tempfile::tempdir().unwrap();
            for (pid, stat) in entries
                .iter()
                .chain(&[("1", "1 (init) S 0 1 1 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 1 0")])
            {
                std::fs::create_dir_all(dir.path().join(pid)).unwrap();
                std::fs::write(dir.path().join(pid).join("stat"), stat).unwrap();
            }
            dir
        }

        #[test]
        fn a_zombie_member_is_not_alive_but_a_running_one_is() {
            let dir = proc_with(&[
                (
                    "200",
                    "200 (cmd) Z 1 200 200 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 5 0",
                ),
                (
                    "201",
                    "201 (cmd) S 200 200 200 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 6 0",
                ),
            ]);
            assert_eq!(group_members_in(dir.path(), 200).unwrap(), vec![201]);
            let only_zombie = proc_with(&[(
                "300",
                "300 (cmd) X 1 300 300 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 5 0",
            )]);
            assert!(group_members_in(only_zombie.path(), 300)
                .unwrap()
                .is_empty());
        }

        #[test]
        fn an_unparseable_stat_line_is_unknown_not_a_non_member() {
            for bad in [
                "garbage without a paren",
                "5 (c) S 1",
                "5 (c) S 1 2 notanumber 0",
            ] {
                let dir = proc_with(&[("400", bad)]);
                assert!(group_members_in(dir.path(), 400).is_err(), "{bad:?}");
            }
        }

        #[test]
        fn a_read_error_other_than_not_found_is_unknown() {
            let dir = proc_with(&[]);
            // `stat` is a directory: reading it fails with EISDIR, which does not mean the process is gone.
            std::fs::create_dir_all(dir.path().join("500/stat")).unwrap();
            assert!(group_members_in(dir.path(), 500).is_err());
            // A pid directory with no `stat` is NotFound: that process is gone.
            std::fs::create_dir_all(dir.path().join("501")).unwrap();
            std::fs::remove_dir(dir.path().join("500/stat")).unwrap();
            std::fs::remove_dir(dir.path().join("500")).unwrap();
            assert!(group_members_in(dir.path(), 501).unwrap().is_empty());
        }

        #[test]
        fn a_comm_with_parentheses_does_not_shift_the_fields() {
            let line = "77 (a) b (c) S 1 77 77 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 4242 0";
            assert_eq!(
                parse_proc_stat(line).unwrap(),
                ('S', 77, "4242".to_string())
            );
        }
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
mod platform {
    use super::parse_ps_members;
    use std::process::Command;

    fn run(program: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(program).args(args).env_clear().output().ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn boot_id() -> Option<String> {
        run("/usr/sbin/sysctl", &["-n", "kern.boottime"]).filter(|text| !text.is_empty())
    }

    pub fn process_starttime(pid: u32) -> Option<String> {
        run("/bin/ps", &["-o", "lstart=", "-p", &pid.to_string()]).filter(|text| !text.is_empty())
    }

    pub fn group_members(pgid: u32) -> Result<Vec<u32>, String> {
        let listing = Command::new("/bin/ps")
            .args(["-A", "-o", "pid=,pgid=,stat="])
            .env_clear()
            .output()
            .map_err(|error| format!("cannot run ps: {error}"))?;
        if !listing.status.success() {
            return Err("ps exited with an error".into());
        }
        parse_ps_members(&String::from_utf8_lossy(&listing.stdout), pgid)
    }
}

#[cfg(not(unix))]
mod platform {
    // Windows runs the command in a kill-on-close Job Object, so a server crash ends it.
    pub fn boot_id() -> Option<String> {
        None
    }
    pub fn process_starttime(_pid: u32) -> Option<String> {
        None
    }
    pub fn group_members(_pgid: u32) -> Result<Vec<u32>, String> {
        Ok(Vec::new())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn an_unused_group_is_gone() {
        let launch = CommandLaunch {
            pgid: u32::MAX - 1,
            leader_starttime: None,
            boot_id: None,
        };
        assert_eq!(launch.liveness(), GroupLiveness::Gone);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_different_boot_id_proves_the_group_is_gone() {
        let launch = CommandLaunch {
            pgid: 1,
            leader_starttime: None,
            boot_id: Some("not-this-boot".into()),
        };
        assert_eq!(launch.liveness(), GroupLiveness::Gone);
    }

    #[test]
    fn ps_listings_skip_zombies_and_reject_unparseable_lines() {
        let listing = "  10   10 Ss\n  11   10 Z\n  12   10 S+\n  13   99 S\n";
        assert_eq!(parse_ps_members(listing, 10).unwrap(), vec![10, 12]);
        assert!(parse_ps_members("10 10\n", 10).is_err());
        assert!(parse_ps_members("ten 10 S\n", 10).is_err());
    }
}
