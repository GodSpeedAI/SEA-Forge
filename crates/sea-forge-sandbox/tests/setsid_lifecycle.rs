//! F08 adversarial lifecycle tests: a workload that `setsid(2)`s a detached
//! descendant must not leave it running past the run's completion or timeout
//! boundary, and must not let it write into the authorized workspace after
//! the run has settled.
//!
//! A process group cannot reach a detached session, so on the pre-F08
//! mechanics (own pgid + `/bin/kill -KILL -- -<pid>`) the descendant survives
//! and its delayed write lands after the run is already recorded as
//! Completed. `group_kill_alone_leaves_a_setsid_descendant_running` pins that
//! hazard with the old mechanics; the containment-gated scenarios assert the
//! fixed behavior (cgroup-v2 leaf per spawn: pre-exec join, `cgroup.kill`,
//! verified-empty teardown).
//!
//! Containment-gated scenarios block honestly when no delegated cgroup-v2
//! root is available (`BLOCKED:` tally; a `SEA_FORGE_REQUIRE_CGROUP=1` lane
//! fails instead of skipping) — a blocked scenario can never count as a pass.
#![cfg(target_os = "linux")]

use sea_forge_core::types::{ExecutionRequest, ExecutionStatus, LifecycleContainment, Operation};
use sea_forge_sandbox::{
    cgroup, ExecutionSandbox, JailSandbox, LocalSandbox, NetworkPosture, SandboxSpec,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Mutex, MutexGuard,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

// ---------------------------------------------------------------------------
// F15 blocked/require harness (per-binary copy: test files are separate
// binaries, each needs its own tally).
// ---------------------------------------------------------------------------

/// Scenarios that could not run because the cgroup prerequisite is
/// unavailable. Tallied separately from the jail tally so a required
/// `SEA_FORGE_REQUIRE_CGROUP` lane is not failed by an unrelated jail gap
/// (and vice versa).
static CGROUP_BLOCKS: AtomicUsize = AtomicUsize::new(0);
static JAIL_BLOCKS: AtomicUsize = AtomicUsize::new(0);

/// Record an unavailable prerequisite for `what`. Under `require_env` this
/// fails the scenario; otherwise the scenario is tallied and labeled
/// `BLOCKED:` so it cannot be mistaken for a tested pass.
fn blocked_or_require(require_env: &str, what: &str, why: &dyn std::fmt::Display) {
    if std::env::var(require_env).as_deref() == Ok("1") {
        panic!("{require_env}=1 but {what} is unavailable, not a pass: {why}");
    }
    match require_env {
        cgroup::REQUIRE_ENV => {
            CGROUP_BLOCKS.fetch_add(1, Ordering::Relaxed);
        }
        "SEA_FORGE_REQUIRE_LANDLOCK_NET" => {
            JAIL_BLOCKS.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
    eprintln!("BLOCKED: {what} scenario could not run: {why}");
}

#[test]
fn no_blocked_scenarios_in_required_mode() {
    let cgroup_blocks = CGROUP_BLOCKS.load(Ordering::SeqCst);
    let jail_blocks = JAIL_BLOCKS.load(Ordering::SeqCst);
    let require_cgroup = std::env::var(cgroup::REQUIRE_ENV).as_deref() == Ok("1");
    let require_jail = std::env::var("SEA_FORGE_REQUIRE_LANDLOCK_NET").as_deref() == Ok("1");
    assert!(
        !(require_cgroup && cgroup_blocks > 0),
        "required lane ({0}=1) finished with {cgroup_blocks} BLOCKED cgroup scenario(s); \
         required lanes cannot skip enforcement scenarios",
        cgroup::REQUIRE_ENV
    );
    assert!(
        !(require_jail && jail_blocks > 0),
        "required lane (SEA_FORGE_REQUIRE_LANDLOCK_NET=1) finished with {jail_blocks} BLOCKED \
         jail scenario(s); required lanes cannot skip enforcement scenarios"
    );
}

// ---------------------------------------------------------------------------
// Environment serialization: `SEA_FORGE_CGROUP_ROOT` /
// `SEA_FORGE_REQUIRE_CGROUP` are process-global, and tests of one binary run
// in parallel threads. Every scenario that reads or sets them holds this
// lock for its whole body; forced-unavailable scenarios restore the prior
// values on drop so a lane's real configuration survives.
// ---------------------------------------------------------------------------

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner())
}

struct EnvRestore(Vec<(&'static str, Option<String>)>);

impl Drop for EnvRestore {
    fn drop(&mut self) {
        for (name, value) in &self.0 {
            match value {
                Some(v) => std::env::set_var(name, v),
                None => std::env::remove_var(name),
            }
        }
    }
}

fn force_env(name: &'static str, value: &str) -> EnvRestore {
    let prior = std::env::var(name).ok();
    std::env::set_var(name, value);
    EnvRestore(vec![(name, prior)])
}

// ---------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let nonce = format!(
        "{nonce}-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    );
    let path = std::env::temp_dir().join(format!("sea-forge-setsid-{name}-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn spec(workspace: PathBuf, artifacts: PathBuf) -> SandboxSpec {
    SandboxSpec {
        workspace_root: workspace,
        artifacts_root: artifacts,
        network: NetworkPosture::default(),
    }
}

fn request(argv: Vec<String>, timeout_secs: u64) -> ExecutionRequest {
    ExecutionRequest {
        plan_item_id: "item_01".into(),
        operation: Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        },
        timeout_secs,
        env: BTreeMap::new(),
        compensating_controls: vec![],
    }
}

/// The adversarial payload: the run's child backgrounds a `setsid` detached
/// descendant that records its pid (identity independent of any process
/// group), sleeps past the run's boundary, then writes `late.txt` into the
/// authorized workspace. The child waits until the descendant has recorded
/// itself before exiting, so cancellation always happens after the marker
/// exists and while the descendant is still mid-sleep.
fn setsid_payload(
    workspace: &Path,
    descendant_delay_secs: u32,
    child_sleep_secs: u32,
) -> Vec<String> {
    let ws = workspace.display();
    vec![
        "/bin/sh".into(),
        "-c".into(),
        format!(
            "/usr/bin/setsid /bin/sh -c 'echo $$ > {ws}/detached.pid; \
             sleep {descendant_delay_secs}; echo late > {ws}/late.txt' & \
             i=0; while [ ! -f {ws}/detached.pid ] && [ $i -lt 100 ]; do i=$((i+1)); sleep 0.05; done; \
             sleep {child_sleep_secs}"
        ),
    ]
}

/// Read the state letter of `/proc/<pid>` (`None` when reaped).
fn process_state(pid: i32) -> Option<char> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // comm may contain spaces and parentheses; state follows the last ')'.
    let rest = stat.rsplit_once(')').map(|(_, rest)| rest)?;
    rest.split_whitespace()
        .next()
        .map(|s| s.as_bytes()[0] as char)
}

fn assert_descendant_alive(pid: i32) {
    match process_state(pid) {
        Some(state) if state != 'Z' => {}
        other => panic!(
            "the detached descendant {pid} should be provably alive before the kill is \
             asserted (state {other:?})"
        ),
    }
}

fn assert_descendant_dead(pid: i32, deadline: Duration) {
    let start = Instant::now();
    loop {
        match process_state(pid) {
            // Reaped or zombie: the descendant no longer executes anything.
            None => return,
            Some('Z') => return,
            Some(state) => {
                if start.elapsed() >= deadline {
                    panic!(
                        "the detached descendant {pid} is still {state:?} {deadline:?} after \
                         the run ended — it outlived the run"
                    );
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn wait_for_file(path: &Path, deadline: Duration) -> bool {
    let start = Instant::now();
    while !path.exists() {
        if start.elapsed() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    true
}

/// No leaf of this test process may remain under the delegated root: a
/// remaining one means teardown was not verified.
fn assert_no_leaves(root: &Path) {
    let prefix = format!("sea-forge-{}-", std::process::id());
    let remaining: Vec<PathBuf> = fs::read_dir(root)
        .unwrap()
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(&prefix))
        })
        .map(|entry| entry.path())
        .collect();
    assert!(
        remaining.is_empty(),
        "lifecycle leaves of this process remain under {}: {remaining:?}",
        root.display()
    );
}

/// Resolve the delegated root or block honestly. `None` means the scenario
/// could not run (blocked, or failed the required lane).
fn require_active_root(what: &str) -> Option<PathBuf> {
    match cgroup::containment() {
        cgroup::CGroupContainment::Active { root } => Some(root),
        cgroup::CGroupContainment::Unavailable { reason } => {
            blocked_or_require(cgroup::REQUIRE_ENV, what, &reason);
            None
        }
    }
}

/// A deterministic uncontainable root for the forced-unavailable scenarios:
/// a plain directory is not a cgroup-v2 mount, so containment reports
/// Unavailable even on a host with a delegated cgroup. The caller decides
/// the `SEA_FORGE_REQUIRE_CGROUP` value the scenario is run under.
struct ForcedUnavailable {
    _dir: PathBuf,
    _restore: EnvRestore,
}

fn force_unavailable_root() -> ForcedUnavailable {
    let dir = temp_dir("uncontainable");
    fs::create_dir_all(dir.join("not-a-cgroup")).unwrap();
    ForcedUnavailable {
        _restore: force_env(
            cgroup::CGROUP_ROOT_ENV,
            &dir.join("not-a-cgroup").display().to_string(),
        ),
        _dir: dir,
    }
}

/// Run the delayed-write adversarial payload to completion and assert the
/// descendant died inside the run and never wrote.
fn assert_contained_completion(
    what: &str,
    sandbox: &dyn ExecutionSandbox,
    workspace: PathBuf,
    artifacts: PathBuf,
    child_sleep_secs: u32,
) {
    let root = match cgroup::containment() {
        cgroup::CGroupContainment::Active { root } => root,
        cgroup::CGroupContainment::Unavailable { reason } => {
            blocked_or_require(cgroup::REQUIRE_ENV, what, &reason);
            return;
        }
    };
    let started = Instant::now();
    let handle = sandbox
        .prepare(&spec(workspace.clone(), artifacts))
        .unwrap();
    let result = sandbox
        .execute(
            &handle,
            &request(setsid_payload(&workspace, 3, child_sleep_secs), 15),
        )
        .unwrap_or_else(|e| panic!("{what}: execute failed: {e}"));
    assert_eq!(
        result.status,
        ExecutionStatus::Completed,
        "{what}: the run must complete; cleanup failures must be loud, not silently swallowed"
    );
    // C1: the run lived in a cgroup leaf and the teardown's verified-empty
    // kill completed (a failure would have returned Err instead), so the
    // result must claim the leaf — never the weaker process group.
    assert_eq!(
        result.lifecycle_containment,
        LifecycleContainment::CgroupLeaf {
            kill_verified: true
        },
        "{what}: a contained run must claim the verified cgroup-leaf kill"
    );
    let marker = workspace.join("detached.pid");
    assert!(
        wait_for_file(&marker, Duration::from_secs(2)),
        "{what}: the detached descendant must have recorded its pid before the run completed"
    );
    // The marker is the pre-cancellation identity record: the payload's
    // child only exits after the descendant has written it, so the run's
    // teardown boundary always happens after the record exists. (The
    // descendant's liveness up to that boundary is asserted by the hazard
    // scenario; here containment has already ended it by the time execute
    // returns.)
    let pid: i32 = fs::read_to_string(&marker).unwrap().trim().parse().unwrap();
    // It must be dead — the cgroup leaf reaches it across the setsid.
    assert_descendant_dead(pid, Duration::from_secs(5));
    // Delayed-write race: the descendant planned to write after the run's
    // completion boundary. With containment that write is impossible; wait
    // past the planned write moment to prove it never lands.
    let write_deadline = started + Duration::from_secs(3 + 2);
    while Instant::now() < write_deadline {
        assert!(
            !workspace.join("late.txt").exists(),
            "{what}: the detached descendant wrote late.txt after the run completed — it \
             outlived the run"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!workspace.join("late.txt").exists());
    // Verified teardown: no leaf of this process remains.
    assert_no_leaves(&root);
    sandbox.destroy(handle).unwrap();
    assert_no_leaves(&root);
}

// ---------------------------------------------------------------------------
// Containment-gated adversarial scenarios (Active cgroup required).
// ---------------------------------------------------------------------------

#[test]
fn setsid_descendant_does_not_outlive_completed_local_run() {
    let _env = env_lock();
    if require_active_root("local completion setsid lifecycle").is_none() {
        return;
    }
    let parent = temp_dir("local-complete");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    assert_contained_completion(
        "local completion setsid lifecycle",
        &LocalSandbox,
        workspace,
        artifacts,
        0,
    );
    fs::remove_dir_all(parent).ok();
}

#[test]
fn setsid_descendant_does_not_outlive_timed_out_local_run() {
    let _env = env_lock();
    let Some(root) = require_active_root("local timeout setsid lifecycle") else {
        return;
    };
    let parent = temp_dir("local-timeout");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let started = Instant::now();
    let sandbox = LocalSandbox;
    let handle = sandbox
        .prepare(&spec(workspace.clone(), artifacts))
        .unwrap();
    // The child outlives the 1s timeout; its detached descendant would write
    // late.txt at +3s — after the timeout boundary.
    let result = sandbox
        .execute(&handle, &request(setsid_payload(&workspace, 3, 30), 1))
        .unwrap();
    assert_eq!(result.status, ExecutionStatus::TimedOut);
    // C1: the leaf reached the setsid descendant on the timeout path too.
    assert_eq!(
        result.lifecycle_containment,
        LifecycleContainment::CgroupLeaf {
            kill_verified: true
        },
        "a contained timeout run must claim the verified cgroup-leaf kill"
    );
    let marker = workspace.join("detached.pid");
    assert!(
        wait_for_file(&marker, Duration::from_secs(2)),
        "the detached descendant must have recorded its pid before the timeout fired"
    );
    let pid: i32 = fs::read_to_string(&marker).unwrap().trim().parse().unwrap();
    assert_descendant_dead(pid, Duration::from_secs(5));
    let write_deadline = started + Duration::from_secs(3 + 2);
    while Instant::now() < write_deadline {
        assert!(
            !workspace.join("late.txt").exists(),
            "the detached descendant wrote late.txt after the run timed out — it outlived the run"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!workspace.join("late.txt").exists());
    assert_no_leaves(&root);
    sandbox.destroy(handle).unwrap();
    assert_no_leaves(&root);
    fs::remove_dir_all(parent).ok();
}

#[test]
fn setsid_descendant_does_not_outlive_completed_jail_run() {
    let _env = env_lock();
    if require_active_root("jail completion setsid lifecycle").is_none() {
        return;
    }
    let Ok(jail) = JailSandbox::new() else {
        blocked_or_require(
            "SEA_FORGE_REQUIRE_LANDLOCK_NET",
            "jail completion setsid lifecycle",
            &"JailSandbox unavailable on this host",
        );
        return;
    };
    let parent = temp_dir("jail-complete");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    assert_contained_completion(
        "jail completion setsid lifecycle",
        &jail,
        workspace,
        artifacts,
        0,
    );
    fs::remove_dir_all(parent).ok();
}

#[test]
fn setsid_descendant_does_not_outlive_timed_out_jail_run() {
    let _env = env_lock();
    let Some(root) = require_active_root("jail timeout setsid lifecycle") else {
        return;
    };
    let Ok(jail) = JailSandbox::new() else {
        blocked_or_require(
            "SEA_FORGE_REQUIRE_LANDLOCK_NET",
            "jail timeout setsid lifecycle",
            &"JailSandbox unavailable on this host",
        );
        return;
    };
    let parent = temp_dir("jail-timeout");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let started = Instant::now();
    let handle = jail.prepare(&spec(workspace.clone(), artifacts)).unwrap();
    let result = jail
        .execute(&handle, &request(setsid_payload(&workspace, 3, 30), 1))
        .unwrap();
    assert_eq!(result.status, ExecutionStatus::TimedOut);
    // C1: the leaf reached the setsid descendant on the timeout path too.
    assert_eq!(
        result.lifecycle_containment,
        LifecycleContainment::CgroupLeaf {
            kill_verified: true
        },
        "a contained timeout run must claim the verified cgroup-leaf kill"
    );
    let marker = workspace.join("detached.pid");
    assert!(
        wait_for_file(&marker, Duration::from_secs(2)),
        "the detached descendant must have recorded its pid before the timeout fired"
    );
    let pid: i32 = fs::read_to_string(&marker).unwrap().trim().parse().unwrap();
    assert_descendant_dead(pid, Duration::from_secs(5));
    let write_deadline = started + Duration::from_secs(3 + 2);
    while Instant::now() < write_deadline {
        assert!(
            !workspace.join("late.txt").exists(),
            "the detached descendant wrote late.txt after the run timed out — it outlived the run"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!workspace.join("late.txt").exists());
    assert_no_leaves(&root);
    jail.destroy(handle).unwrap();
    assert_no_leaves(&root);
    fs::remove_dir_all(parent).ok();
}

/// Verified teardown of `destroy`: a leaked leaf (spawn-failure window or an
/// earlier teardown error) must be found, required empty, and removed.
#[test]
fn destroy_sweeps_a_leaked_leaf() {
    let _env = env_lock();
    let Some(root) = require_active_root("destroy sweeps a leaked leaf") else {
        return;
    };
    let leaked = root.join(format!(
        "sea-forge-{}-{}",
        std::process::id(),
        990_042 // outside the runtime's sequence space
    ));
    fs::create_dir(&leaked).unwrap();
    let parent = temp_dir("sweep");
    let sandbox = LocalSandbox;
    let handle = sandbox
        .prepare(&spec(parent.join("workspace"), parent.join("artifacts")))
        .unwrap();
    sandbox.destroy(handle).unwrap();
    assert!(
        !leaked.exists(),
        "destroy must remove this process's leaked lifecycle leaf {}",
        leaked.display()
    );
    fs::remove_dir_all(parent).ok();
}

// ---------------------------------------------------------------------------
// Forced-unavailable scenarios (deterministic on every host).
// ---------------------------------------------------------------------------

/// Fail-closed contract: with `SEA_FORGE_REQUIRE_CGROUP=1` and containment
/// unavailable, the spawn is refused with a typed `cgroup_unavailable` error
/// — never a process-group-only run masquerading as contained.
#[test]
fn required_lane_refuses_to_spawn_without_containment() {
    let _env = env_lock();
    let forced = force_unavailable_root();
    let _require = force_env(cgroup::REQUIRE_ENV, "1");
    let parent = temp_dir("require-refusal");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let sandbox = LocalSandbox;
    let handle = sandbox.prepare(&spec(workspace, artifacts)).unwrap();
    let err = sandbox
        .execute(&handle, &request(vec!["/bin/true".into()], 5))
        .expect_err("required lane must refuse to run without cgroup containment");
    assert_eq!(
        err.class, "cgroup_unavailable",
        "the typed degrade must be `cgroup_unavailable`, got: {err}"
    );
    drop(forced);
    fs::remove_dir_all(parent).ok();
}

/// Non-required contract: an unavailable cgroup degrades loudly to the
/// process-group kill and the run itself still completes.
#[test]
fn unavailable_containment_degrades_and_the_run_completes() {
    let _env = env_lock();
    let forced = force_unavailable_root();
    let _require = force_env(cgroup::REQUIRE_ENV, "0");
    let parent = temp_dir("degrade");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let sandbox = LocalSandbox;
    let handle = sandbox.prepare(&spec(workspace, artifacts)).unwrap();
    let result = sandbox
        .execute(&handle, &request(vec!["/bin/echo".into(), "hi".into()], 5))
        .expect("a non-required run must proceed without containment");
    assert_eq!(result.status, ExecutionStatus::Completed);
    // C1: the capability claim fails closed — an unavailable cgroup reports
    // ProcessGroup, never CgroupLeaf (the typed degrade was surfaced on
    // stderr separately).
    assert_eq!(
        result.lifecycle_containment,
        LifecycleContainment::ProcessGroup,
        "an uncontained run must claim the process group, never the cgroup leaf"
    );
    drop(forced);
    fs::remove_dir_all(parent).ok();
}

// ---------------------------------------------------------------------------
// The hazard itself, with the pre-F08 mechanics (runs on every host).
// ---------------------------------------------------------------------------

/// Documents the F08 finding with HEAD's own mechanics — own process group
/// plus `/bin/kill -KILL -- -<pid>`: a setsid descendant is in a different
/// session, survives the group kill, and completes its workspace write after
/// the run's completion boundary. This is the behavior the cgroup leaf
/// removes; if this ever stops reproducing, the payload or the kernel
/// session semantics changed.
#[test]
fn group_kill_alone_leaves_a_setsid_descendant_running() {
    let parent = temp_dir("hazard");
    let workspace = parent.join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    let ws = workspace.display();
    let mut command = std::process::Command::new("/bin/sh");
    command.arg("-c").arg(format!(
        "/usr/bin/setsid /bin/sh -c 'echo $$ > {ws}/detached.pid; sleep 2; echo late > {ws}/late.txt' & \
         i=0; while [ ! -f {ws}/detached.pid ] && [ $i -lt 100 ]; do i=$((i+1)); sleep 0.05; done"
    ));
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn().unwrap();
    use wait_timeout::ChildExt;
    let status = child
        .wait_timeout(Duration::from_secs(10))
        .unwrap()
        .expect("the child exits on its own");
    assert!(status.success());
    let pid: i32 = fs::read_to_string(workspace.join("detached.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_descendant_alive(pid);
    // The old teardown: group kill only.
    let group = format!("-{}", child.id());
    let _ = std::process::Command::new("/bin/kill")
        .args(["-KILL", "--", &group])
        .output();
    child.wait().unwrap();
    // The descendant survives and its write lands after the boundary.
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline && !workspace.join("late.txt").exists() {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        workspace.join("late.txt").exists(),
        "expected the F08 hazard to reproduce: the setsid descendant {pid} should survive \
         the group kill and write late.txt after the run completed"
    );
    // Do not leak the survivor.
    let _ = std::process::Command::new("/bin/kill")
        .args(["-KILL", &pid.to_string()])
        .output();
    fs::remove_dir_all(parent).ok();
}
