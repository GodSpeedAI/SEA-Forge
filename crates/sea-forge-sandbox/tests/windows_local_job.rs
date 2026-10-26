//! Windows Local sandbox job-object tests.
//!
//! `LocalSandbox` on Windows starts each workload suspended inside a
//! kill-on-close Job Object (`sea_forge_os::spawn_in_job`) and terminates the
//! job when the timeout fires. Each test checks host-side effects: a marker
//! file a grandchild would write after the timeout, and the process table.
//!
//! Scope: this backend has NO filesystem or network containment. These tests
//! cover process-tree lifetime and basic run/env behavior only.
#![cfg(windows)]

use sea_forge_core::types::{ExecutionRequest, ExecutionResult, ExecutionStatus, Operation};
use sea_forge_sandbox::{ExecutionSandbox, LocalSandbox, NetworkPosture, SandboxSpec};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Per-test temp tree: `workspace/`, `artifacts/`, and `outside/` (not granted).
struct Dirs {
    root: PathBuf,
    workspace: PathBuf,
    artifacts: PathBuf,
    outside: PathBuf,
}

impl Dirs {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("sf-winjob-{tag}-{}-{nanos}", std::process::id()));
        let dirs = Dirs {
            workspace: root.join("workspace"),
            artifacts: root.join("artifacts"),
            outside: root.join("outside"),
            root,
        };
        for dir in [&dirs.workspace, &dirs.artifacts, &dirs.outside] {
            fs::create_dir_all(dir).expect("create fixture dir");
        }
        dirs
    }
}

impl Drop for Dirs {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Timeout for the workload. Long enough that PowerShell startup on a slow CI
/// runner (about 2-3 s) finishes spawning before the kill. The grandchild must
/// outlive the kill, so its marker delay is much longer than the timeout.
const TIMEOUT_SECS: u64 = 15;
/// `ping -n N` waits about N-1 seconds, so the grandchild writes its marker
/// about 35 s after spawning, well after the kill.
const GRANDCHILD_PINGS: u32 = 36;
/// Wait after the kill. Must exceed the grandchild's marker time (about
/// 5 s of spawn lag plus 35 s) so a surviving grandchild would be caught.
const POST_KILL_WAIT_SECS: u64 = 40;

fn system_root() -> String {
    std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into())
}

fn cmd_exe() -> String {
    format!(r"{}\System32\cmd.exe", system_root())
}

fn powershell_exe() -> String {
    format!(
        r"{}\System32\WindowsPowerShell\v1.0\powershell.exe",
        system_root()
    )
}

fn taskkill_exe() -> String {
    format!(r"{}\System32\taskkill.exe", system_root())
}

/// Environment passed as `req.env` for the containment tests. PowerShell needs
/// the standard Windows profile variables to start, so pass a full snapshot of
/// the host environment. The env contract is tested separately in
/// `child_sees_only_req_env`.
fn host_env() -> BTreeMap<String, String> {
    std::env::vars().collect()
}

/// Minimal environment for the env test. Only these names reach the child.
fn minimal_env() -> BTreeMap<String, String> {
    let root = system_root();
    let temp = std::env::temp_dir().display().to_string();
    BTreeMap::from([
        ("PATH".to_string(), format!(r"{root}\System32;{root}")),
        ("SYSTEMROOT".to_string(), root),
        ("TEMP".to_string(), temp.clone()),
        ("TMP".to_string(), temp),
    ])
}

/// PID written by the parent script after it spawned the grandchild.
fn spawned_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Child stdout and stderr, for assertion messages.
fn child_output(dirs: &Dirs) -> String {
    let read = |name: &str| {
        fs::read_to_string(dirs.artifacts.join(name)).unwrap_or_else(|e| format!("<{e}>"))
    };
    format!(
        "stdout.txt: {:?}\nstderr.txt: {:?}",
        read("stdout.txt"),
        read("stderr.txt")
    )
}

fn powershell_argv(script: &str) -> Vec<String> {
    vec![
        powershell_exe(),
        "-NoProfile".into(),
        "-NonInteractive".into(),
        "-Command".into(),
        script.into(),
    ]
}

/// PowerShell single-quoted literal (doubles embedded single quotes).
fn ps_single(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Alphanumeric token, unique per run, used to find the test's processes.
fn unique_token(tag: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    format!("sfjob{tag}{}x{nanos}", std::process::id())
}

/// cmd.exe arguments for a grandchild that waits about 5 s, then writes
/// `marker`. The token sits in the command line so the process can be found.
fn grandchild_args(token: &str, marker: &Path) -> String {
    format!(
        "/d /c ping -n {GRANDCHILD_PINGS} 127.0.0.1 >nul & rem {token} & echo done > \"{}\"",
        marker.display()
    )
}

fn run(dirs: &Dirs, argv: Vec<String>, timeout_secs: u64) -> ExecutionResult {
    let spec = SandboxSpec {
        workspace_root: dirs.workspace.clone(),
        artifacts_root: dirs.artifacts.clone(),
        network: NetworkPosture::Denied,
    };
    let sandbox = LocalSandbox;
    let handle = sandbox.prepare(&spec).expect("prepare");
    let req = ExecutionRequest {
        plan_item_id: "windows-local-job".into(),
        operation: Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        },
        timeout_secs,
        env: host_env(),
        compensating_controls: vec![],
    };
    sandbox.execute(&handle, &req).expect("execute")
}

/// PIDs of live processes whose command line contains `token`. Excludes the
/// query shell itself, whose own command line contains the token.
fn live_pids_with_token(token: &str) -> Vec<u32> {
    let script = format!(
        "Get-CimInstance Win32_Process | Where-Object {{ $_.ProcessId -ne $PID -and $_.CommandLine -like {} }} | ForEach-Object {{ $_.ProcessId }}",
        ps_single(&format!("*{token}*"))
    );
    let out = Command::new(powershell_exe())
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .expect("run process query");
    assert!(
        out.status.success(),
        "process query failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

/// Force-kills each PID and its children. Used to clean up survivors.
fn kill_tree(pids: &[u32]) {
    for pid in pids {
        let _ = Command::new(taskkill_exe())
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output();
    }
}

#[test]
fn process_query_sees_a_live_tagged_process() {
    // Probe for the query helper: without this, the survivor checks below
    // could pass vacuously.
    let token = unique_token("probe");
    let mut child = Command::new(cmd_exe())
        .args([
            "/d",
            "/c",
            &format!("ping -n 30 127.0.0.1 >nul & rem {token}"),
        ])
        .spawn()
        .expect("spawn probe");
    thread::sleep(Duration::from_secs(2));
    let pids = live_pids_with_token(&token);
    let _ = child.kill();
    kill_tree(&pids);
    let _ = child.wait();
    assert!(
        pids.contains(&child.id()),
        "query did not find the live probe process (pid {}); found {pids:?}",
        child.id()
    );
}

#[test]
fn timeout_kills_whole_process_tree() {
    let dirs = Dirs::new("tree");
    let token = unique_token("tree");
    let marker = dirs.outside.join("grandchild-marker.txt");
    let spawned = dirs.outside.join("grandchild-spawned.txt");

    // Parent starts a grandchild with Start-Process (inherits the job), writes
    // a "spawned" marker, then sleeps far longer than the timeout.
    let script = format!(
        "$p = Start-Process -FilePath {cmd} -ArgumentList {args} -PassThru; Set-Content -LiteralPath {sp} -Value $p.Id; Start-Sleep -Seconds 60",
        cmd = ps_single(&cmd_exe()),
        args = ps_single(&grandchild_args(&token, &marker)),
        sp = ps_single(&spawned.display().to_string()),
    );
    let result = run(&dirs, powershell_argv(&script), TIMEOUT_SECS);
    assert_eq!(result.status, ExecutionStatus::TimedOut);
    assert_eq!(result.exit_code, None);
    assert!(
        spawned_pid(&spawned).is_some_and(|pid| pid > 0),
        "grandchild was never spawned before the timeout; the test would be vacuous\n{}",
        child_output(&dirs)
    );

    // The grandchild writes its marker about 35 s after spawning.
    thread::sleep(Duration::from_secs(POST_KILL_WAIT_SECS));
    let survivors = live_pids_with_token(&token);
    kill_tree(&survivors);
    assert!(
        survivors.is_empty(),
        "processes with token {token} survived the job kill: {survivors:?}"
    );
    assert!(
        !marker.exists(),
        "grandchild survived the timeout and wrote {}",
        marker.display()
    );
}

#[test]
fn wmi_created_process_cannot_escape_timeout_kill() {
    // Win32_Process.Create starts the child under WmiPrvSE, outside the job.
    // If this fails, the escape is real: the Local backend does not contain
    // WMI-created children. Do not weaken the assertions.
    let dirs = Dirs::new("wmi");
    let token = unique_token("wmi");
    let marker = dirs.outside.join("wmi-marker.txt");
    let spawned = dirs.outside.join("wmi-spawned.txt");

    let command_line = format!("{} {}", cmd_exe(), grandchild_args(&token, &marker));
    let script = format!(
        "$r = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{{ CommandLine = {cl} }}; Set-Content -LiteralPath {sp} -Value (\"{{0}} {{1}}\" -f $r.ReturnValue, $r.ProcessId); Start-Sleep -Seconds 60",
        cl = ps_single(&command_line),
        sp = ps_single(&spawned.display().to_string()),
    );
    let result = run(&dirs, powershell_argv(&script), TIMEOUT_SECS);
    assert_eq!(result.status, ExecutionStatus::TimedOut);
    // Win32_Process.Create must succeed (ReturnValue 0) with a real PID, or
    // the test would pass without ever creating a process.
    let created = fs::read_to_string(&spawned).unwrap_or_default();
    assert!(
        created.split_whitespace().next() == Some("0")
            && created
                .split_whitespace()
                .nth(1)
                .and_then(|p| p.parse::<u32>().ok())
                .is_some_and(|pid| pid > 0),
        "Win32_Process.Create did not succeed before the timeout ({created:?}); the test would be vacuous\n{}",
        child_output(&dirs)
    );

    thread::sleep(Duration::from_secs(POST_KILL_WAIT_SECS));
    let survivors = live_pids_with_token(&token);
    kill_tree(&survivors);
    assert!(
        survivors.is_empty(),
        "WMI-created process escaped the job; still running with token {token}: {survivors:?}"
    );
    assert!(
        !marker.exists(),
        "WMI-created process escaped the job and wrote {}",
        marker.display()
    );
}

#[test]
fn normal_completion_propagates_exit_code_and_captures_stdout() {
    let dirs = Dirs::new("ok");
    let result = run(
        &dirs,
        vec![
            cmd_exe(),
            "/d".into(),
            "/c".into(),
            "echo sf-win-stdout & exit /b 7".into(),
        ],
        30,
    );
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(result.exit_code, Some(7));
    assert_eq!(result.stdout_path, "artifacts/stdout.txt");
    let stdout = fs::read_to_string(dirs.artifacts.join("stdout.txt")).expect("stdout.txt");
    assert!(
        stdout.contains("sf-win-stdout"),
        "stdout.txt did not capture the child output: {stdout:?}"
    );
}

#[test]
fn child_sees_only_req_env() {
    let dirs = Dirs::new("env");
    // Set a host variable that is NOT in req.env. Only req.env may reach the
    // child, so the child must print it literally. %SF_REQ_PROBE% is in req.env.
    std::env::set_var("SF_WIN_HOST_LEAK_PROBE", "host-leak-value");
    let mut env = minimal_env();
    env.insert("SF_REQ_PROBE".into(), "granted".into());
    let spec = SandboxSpec {
        workspace_root: dirs.workspace.clone(),
        artifacts_root: dirs.artifacts.clone(),
        network: NetworkPosture::Denied,
    };
    let sandbox = LocalSandbox;
    let handle = sandbox.prepare(&spec).expect("prepare");
    let req = ExecutionRequest {
        plan_item_id: "windows-local-job-env".into(),
        operation: Operation::ExecuteCommand {
            argv: vec![
                cmd_exe(),
                "/d".into(),
                "/c".into(),
                "echo LEAK=%SF_WIN_HOST_LEAK_PROBE% GRANT=%SF_REQ_PROBE%".into(),
            ],
            cwd: ".".into(),
        },
        timeout_secs: 30,
        env,
        compensating_controls: vec![],
    };
    let result = sandbox.execute(&handle, &req).expect("execute");
    assert_eq!(result.status, ExecutionStatus::Completed);
    let stdout = fs::read_to_string(dirs.artifacts.join("stdout.txt")).expect("stdout.txt");
    assert!(
        stdout.contains("GRANT=granted"),
        "req.env value did not reach the child: {stdout:?}"
    );
    assert!(
        stdout.contains("LEAK=%SF_WIN_HOST_LEAK_PROBE%"),
        "host env var leaked into the child: {stdout:?}"
    );
}
