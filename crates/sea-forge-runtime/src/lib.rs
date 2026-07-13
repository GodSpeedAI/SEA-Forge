use chrono::Utc;
use sea_forge_core::{errors::ForgeError, types::*};
use sea_forge_sandbox::safe_join;
use std::os::unix::process::CommandExt;
use std::{
    fs::{self, File},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;
pub fn execute(
    grant: sea_forge_authority::ActionGrant,
    request: &ExecutionRequest,
    run_id: &str,
    workspace: &Path,
    artifacts: &Path,
) -> Result<ExecutionResult, ForgeError> {
    grant.authorize(
        &sea_forge_core::types::AuthorityAction::from(&request.operation),
        run_id,
        &request.plan_item_id,
        workspace,
    )?;
    execute_authorized(request, workspace, artifacts)
}

fn execute_authorized(
    request: &ExecutionRequest,
    workspace: &Path,
    artifacts: &Path,
) -> Result<ExecutionResult, ForgeError> {
    let started_at = Utc::now().to_rfc3339();
    fs::create_dir_all(artifacts).map_err(|e| ForgeError::io("create artifacts", e))?;
    let stdout = File::create(artifacts.join("stdout.txt"))
        .map_err(|e| ForgeError::io("create stdout", e))?;
    let stderr = File::create(artifacts.join("stderr.txt"))
        .map_err(|e| ForgeError::io("create stderr", e))?;
    let (argv, cwd) = match &request.operation {
        Operation::ExecuteCommand { argv, cwd } if !argv.is_empty() => (argv, cwd),
        _ => {
            return Err(ForgeError::Internal(
                "execution request requires non-empty execute_command".into(),
            ))
        }
    };
    let cwd = safe_join(workspace, cwd)?;
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(cwd)
        .env_clear()
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    command.process_group(0);
    for (k, v) in &request.env {
        command.env(k, v);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            return Ok(ExecutionResult {
                status: ExecutionStatus::SpawnFailed,
                exit_code: None,
                stdout_path: "artifacts/stdout.txt".into(),
                stderr_path: "artifacts/stderr.txt".into(),
                started_at,
                finished_at: Utc::now().to_rfc3339(),
            })
        }
    };
    let status = child
        .wait_timeout(Duration::from_secs(request.timeout_secs))
        .map_err(|e| ForgeError::io("wait for child", e))?;
    let (state, code) = if let Some(status) = status {
        (ExecutionStatus::Completed, status.code().map(i64::from))
    } else {
        let group = format!("-{}", child.id());
        let group_kill = Command::new("/bin/kill")
            .args(["-KILL", "--", &group])
            .env_clear()
            .output()
            .map_err(|error| ForgeError::io("terminate timed out process group", error))?;
        if !group_kill.status.success() {
            let group_still_exists = Command::new("/bin/kill")
                .args(["-0", "--", &group])
                .env_clear()
                .output()
                .map_err(|error| ForgeError::io("verify timed out process group", error))?
                .status
                .success();
            if group_still_exists {
                return Err(ForgeError::Internal(
                    "timed out process group survived termination".into(),
                ));
            }
        }
        child
            .wait()
            .map_err(|error| ForgeError::io("reap timed out child", error))?;
        (ExecutionStatus::TimedOut, None)
    };
    Ok(ExecutionResult {
        status: state,
        exit_code: code,
        stdout_path: "artifacts/stdout.txt".into(),
        stderr_path: "artifacts/stderr.txt".into(),
        started_at,
        finished_at: Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::BTreeMap,
        time::{Instant, SystemTime, UNIX_EPOCH},
    };

    fn directories(name: &str) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = std::env::temp_dir().join(format!("sea-forge-runtime-{name}-{nonce}"));
        let workspace = parent.join("workspace");
        let artifacts = parent.join("artifacts");
        fs::create_dir_all(&workspace).unwrap();
        (parent, workspace, artifacts)
    }
    fn request(argv: Vec<String>, timeout_secs: u64) -> ExecutionRequest {
        let mut env = BTreeMap::new();
        env.insert("PATH".into(), std::env::var("PATH").unwrap_or_default());
        env.insert("HOME".into(), "/tmp/sea-forge-home".into());
        ExecutionRequest {
            plan_item_id: "item_01".into(),
            operation: Operation::ExecuteCommand {
                argv,
                cwd: ".".into(),
            },
            timeout_secs,
            env,
        }
    }

    #[test]
    fn child_environment_is_minimal() {
        let (parent, workspace, artifacts) = directories("env");
        let result =
            execute_authorized(&request(vec!["env".into()], 5), &workspace, &artifacts).unwrap();
        assert_eq!(result.status, ExecutionStatus::Completed);
        let output = fs::read_to_string(artifacts.join("stdout.txt")).unwrap();
        let mut names: Vec<_> = output
            .lines()
            .filter_map(|line| line.split_once('=').map(|v| v.0))
            .collect();
        names.sort_unstable();
        assert_eq!(names, ["HOME", "PATH"]);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn timeout_kills_and_reaps_child() {
        let (parent, workspace, artifacts) = directories("timeout");
        let started = Instant::now();
        let result = execute_authorized(
            &request(vec!["sleep".into(), "30".into()], 0),
            &workspace,
            &artifacts,
        )
        .unwrap();
        assert_eq!(result.status, ExecutionStatus::TimedOut);
        assert!(started.elapsed().as_secs() < 10);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn spawn_failure_is_an_execution_result() {
        let (parent, workspace, artifacts) = directories("spawn");
        let result = execute_authorized(
            &request(vec!["sea-forge-command-does-not-exist".into()], 1),
            &workspace,
            &artifacts,
        )
        .unwrap();
        assert_eq!(result.status, ExecutionStatus::SpawnFailed);
        fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn timed_out_child_pid_is_no_longer_alive() {
        let (parent, workspace, artifacts) = directories("reaped");
        let pid_file = parent.join("pid");
        let mut request = request(
            vec![
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                "--exact".into(),
                "tests::timeout_child_helper".into(),
            ],
            1,
        );
        request
            .env
            .insert("SEA_FORGE_TIMEOUT_HELPER".into(), "1".into());
        request.env.insert(
            "SEA_FORGE_TIMEOUT_PID_FILE".into(),
            pid_file.to_string_lossy().into_owned(),
        );
        let result = execute_authorized(&request, &workspace, &artifacts).unwrap();
        assert_eq!(result.status, ExecutionStatus::TimedOut);
        let pids = fs::read_to_string(&pid_file).unwrap();
        for pid in pids.split_whitespace() {
            let mut alive = true;
            for _ in 0..40 {
                alive = Command::new("kill")
                    .args(["-0", pid])
                    .output()
                    .unwrap()
                    .status
                    .success();
                if !alive {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            assert!(!alive, "timed-out process {pid} survived");
        }
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn timeout_child_helper() {
        if std::env::var("SEA_FORGE_TIMEOUT_HELPER").as_deref() != Ok("1") {
            return;
        }
        let path = std::env::var("SEA_FORGE_TIMEOUT_PID_FILE").unwrap();
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        fs::write(path, format!("{} {}", std::process::id(), child.id())).unwrap();
        let _ = child.wait();
    }
}
