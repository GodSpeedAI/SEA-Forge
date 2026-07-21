use std::fs;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn server_bin() -> PathBuf {
    let cli = env!("CARGO_BIN_EXE_sea-forge");
    Path::new(cli).with_file_name("sea-forge-server")
}

fn wait_for_socket(path: &Path, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if path.exists() && UnixStream::connect(path).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

/// Task 1: the public serial plan facade remains complete after its lifecycle
/// primitives move into the shared case runner.
#[test]
fn serial_plan_facade_completes_through_case_runner() {
    let root = tempfile::tempdir().unwrap();
    let root_path = root.path().canonicalize().unwrap();
    let policy = root_path.join("policy.yaml");
    let plan = root_path.join("plan.json");

    fs::write(
        &policy,
        "version: \"0.1\"\n\
         rules:\n\
         \x20 - name: allow-write\n\
         \x20   verdict: allow\n\
         \x20   actor_role: operator\n\
         \x20   operation_kind: write_file\n\
         \x20   path_prefix: \"\"\n",
    )
    .unwrap();
    fs::write(
        &plan,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "0.2",
            "plan_id": "plan_01",
            "case_id": "case_proposal",
            "run_id": "run_proposal",
            "intent_id": "int_proposal",
            "items": [{
                "plan_item_id": "write",
                "name": "sandboxed_task",
                "operations": [{"kind": "write_file", "path": "output.txt", "content_hint": "done"}],
                "entry_criteria": [],
                "exit_criteria": [],
                "settlement_criteria": {},
                "item_kind": "sandboxed_task",
                "sandbox_class": "local",
                "parent_stage": null,
                "markers": {"required": true, "repetition": false, "manual_activation": false},
                "max_instances": 1,
                "depends_on": []
            }],
            "template_ref": null,
            "job_contract_ref": null
        }))
        .unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--root",
            root_path.to_str().unwrap(),
            "--policy",
            policy.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "sea-forge run --plan failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("case_state=completed"));
}

/// T13.1: the plan pipeline routes an `agent_task` item to the running
/// sea-forge-server, which executes the delegation and settles. The CLI
/// emits `SettlementRecorded` and `ItemCompleted` as for any other item.
#[test]
fn t13_1_agent_task_routed_through_server_and_settles() {
    let server_bin = server_bin();
    if !server_bin.exists() {
        eprintln!(
            "skipping: server binary not found at {}",
            server_bin.display()
        );
        return;
    }

    let root = tempfile::tempdir().unwrap();
    let root_path = root.path().canonicalize().unwrap();

    // Stub HTTP endpoint returning a valid OpenAI-compatible response.
    let stub_port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let _ = listener.set_nonblocking(true);
        // ponytail: accept one connection in a background thread.
        std::thread::spawn(move || {
            let _ = listener.set_nonblocking(false);
            if let Ok((mut stream, _)) = listener.accept() {
                use std::io::{Read, Write};
                let mut req = vec![0_u8; 4096];
                let _ = stream.read(&mut req);
                let body =
                    br#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":1}}"#;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(body);
            }
        });
        port
    };

    // server.yaml
    let server_yaml = format!(
        "socket_path: {root}/server.sock\n\
         root: {root}\n\
         max_concurrent_runs: 2\n\
         agent:\n\
         \x20 endpoints:\n\
         \x20   - id: local-test\n\
         \x20     kind: open_ai_compatible\n\
         \x20     base_url: http://127.0.0.1:{port}/\n\
         \x20     default_model: test-model\n\
         \x20     allow_loopback_test: true\n",
        root = root_path.display(),
        port = stub_port,
    );
    fs::write(root_path.join("server.yaml"), server_yaml).unwrap();

    // policy.yaml
    fs::write(
        root_path.join("policy.yaml"),
        "version: \"0.1\"\n\
         policy_surfaces:\n\
         \x20 external_api:\n\
         \x20   mode: deny-by-default\n\
         \x20   allow_hosts: [127.0.0.1]\n\
         rules:\n\
         \x20 - name: allow-agent-task\n\
         \x20   verdict: allow\n\
         \x20   actor_role: operator\n\
         \x20   operation_kind: agent_task\n\
         \x20 - name: allow-write\n\
         \x20   verdict: allow\n\
         \x20   actor_role: operator\n\
         \x20   operation_kind: write_file\n\
         \x20   path_prefix: \"\"\n",
    )
    .unwrap();

    // plan.json — single agent_task item.
    let plan = root_path.join("plan.json");
    fs::write(
        &plan,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "0.2",
            "plan_id": "plan_01",
            "case_id": "case_proposal",
            "run_id": "run_proposal",
            "intent_id": "int_proposal",
            "items": [{
                "plan_item_id": "agent",
                "name": "agent_task",
                "operations": [{
                    "kind": "agent_task",
                    "endpoint_ref": "local-test",
                    "instruction": "say ok",
                    "max_turns": 1
                }],
                "entry_criteria": [],
                "exit_criteria": [],
                "settlement_criteria": {},
                "item_kind": "agent_task",
                "sandbox_class": null,
                "parent_stage": null,
                "markers": {"required": true, "repetition": false, "manual_activation": false},
                "max_instances": 1,
                "depends_on": []
            }, {
                "plan_item_id": "downstream",
                "name": "sandboxed_task",
                "operations": [{"kind": "write_file", "path": "output.txt", "content_hint": "done"}],
                "entry_criteria": [{
                    "on": {"source": "agent", "event": "settlement_status"},
                    "if": {"kind": "settlement_status", "status": "accepted"}
                }],
                "exit_criteria": [],
                "settlement_criteria": {},
                "item_kind": "sandboxed_task",
                "sandbox_class": "local",
                "parent_stage": null,
                "markers": {"required": true, "repetition": false, "manual_activation": false},
                "max_instances": 1,
                "depends_on": []
            }],
            "template_ref": null,
            "job_contract_ref": null
        }))
        .unwrap(),
    )
    .unwrap();

    // Start server.
    let mut server_child = Command::new(&server_bin)
        .env("SEA_FORGE_ROOT", &root_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start sea-forge-server");

    let socket = root_path.join("server.sock");
    let connected = wait_for_socket(&socket, Duration::from_secs(5));
    if !connected {
        let _ = server_child.kill();
        let output = server_child.wait_with_output().unwrap();
        panic!(
            "server socket did not appear\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Run CLI.
    let mut cli_child = Command::new(env!("CARGO_BIN_EXE_sea-forge"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "--root",
            root_path.to_str().unwrap(),
            "--policy",
            root_path.join("policy.yaml").to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run sea-forge");
    let deadline = Instant::now() + Duration::from_secs(20);
    while cli_child.try_wait().unwrap().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    if cli_child.try_wait().unwrap().is_none() {
        let _ = cli_child.kill();
        let output = cli_child.wait_with_output().unwrap();
        let _ = server_child.kill();
        let server_output = server_child.wait_with_output().unwrap();
        panic!(
            "sea-forge run --plan timed out\ncli stderr: {}\nserver stderr: {}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&server_output.stderr)
        );
    }
    let output = cli_child.wait_with_output().unwrap();

    // Cleanup.
    let _ = server_child.kill();
    let _ = server_child.wait();

    assert_eq!(
        output.status.code(),
        Some(0),
        "sea-forge run --plan failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
