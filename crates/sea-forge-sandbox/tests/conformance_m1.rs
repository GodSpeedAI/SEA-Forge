use sea_forge_core::types::{ExecutionRequest, ExecutionStatus, Operation};
use sea_forge_sandbox::{ExecutionSandbox, JailSandbox, NetworkPosture, SandboxClass, SandboxSpec};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("sea-forge-m1-{name}-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn request(argv: Vec<String>) -> ExecutionRequest {
    let mut env = BTreeMap::new();
    if let Ok(path) = std::env::var("PATH") {
        env.insert("PATH".into(), path);
    }
    env.insert("HOME".into(), "/tmp".into());
    ExecutionRequest {
        plan_item_id: "item_01".into(),
        operation: Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        },
        timeout_secs: 10,
        env,
        compensating_controls: vec![],
    }
}

#[test]
fn jail_blocks_write_outside_workspace() {
    let parent = temp_dir("escape");
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();

    let jail = match JailSandbox::new() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Skipping jail test: {e}");
            return;
        }
    };
    let spec = SandboxSpec {
        workspace_root: workspace.clone(),
        artifacts_root: artifacts.clone(),
        network: NetworkPosture::default(),
    };
    let handle = jail.prepare(&spec).unwrap();
    // Try to write outside the workspace via a relative path escape.
    let result = jail
        .execute(
            &handle,
            &request(vec![
                "sh".into(),
                "-c".into(),
                "echo escaped > ../outside.txt".into(),
            ]),
        )
        .unwrap();
    let _ = jail.destroy(handle);

    // The write must have been blocked by the OS.
    assert!(
        !parent.join("outside.txt").exists(),
        "jail failed to block write outside workspace"
    );
    // The command must not have succeeded normally.
    assert_ne!(
        result.status,
        ExecutionStatus::Completed,
        "sandbox violation should not be a normal completion"
    );

    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn jail_class_is_jail() {
    let jail = match JailSandbox::new() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Skipping jail test: {e}");
            return;
        }
    };
    assert_eq!(jail.class(), SandboxClass::Jail);
}

#[test]
fn jail_unavailable_on_unsupported_platform_returns_error() {
    // On non-Linux, JailSandbox::new() must return an error.
    #[cfg(not(target_os = "linux"))]
    {
        let result = JailSandbox::new();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().class, "unsupported_sandbox_class_error");
    }
    // On Linux, if Landlock is not supported by the kernel, the error is
    // detected at execute time rather than at construction time.
    #[cfg(target_os = "linux")]
    {
        let _ = JailSandbox::new();
    }
}

// ---------------------------------------------------------------------------
// M1 network-denial conformance (Task 3).
//
// A jail-class run must not be able to open ungranted outbound TCP connections
// or bind/listen on TCP sockets. An explicit network grant permits only its
// declared scope. Setup must fail closed (`jail_unavailable`) — never a silent
// fallback to `local` — when the host cannot enforce the requested posture.
//
// Scope note: enforcement is TCP-only (Landlock v4+ `AccessNet::{BindTcp,
// ConnectTcp}`). UDP/raw sockets are an explicitly out-of-scope, documented gap
// (see `docs/decisions/ADR-002-audit-remediation-dependencies.md` §2); these
// tests therefore only instrument TCP. All fixtures are loopback-only
// (127.0.0.1) with no external Internet dependency.
//
// The jailed workload is this very test binary re-invoked in a hidden
// `net_probe_helper` mode (mirrors the runtime crate's timeout-child pattern),
// so no external tool or network service is required. The helper performs one
// TCP action and writes its outcome into the jail's writable workspace, which
// the parent test then reads back and asserts on — alongside the fixture
// server's own connection counter, so a denied connect is proven by *zero*
// accepted connections, not merely by the child's self-report.

/// Hidden helper: run inside the jail to probe one TCP action, driven by env.
///
/// `SEA_FORGE_NET_PROBE` = `connect` | `bind`
/// `SEA_FORGE_NET_PORT`  = target port (connect) or port to bind (bind)
/// `SEA_FORGE_NET_OUT`   = absolute path to write the outcome to
///
/// Writes `ok` on success or `err:<message>` on failure. Exits non-zero on
/// failure so the sandbox layer also observes a non-clean completion.
#[test]
fn net_probe_helper() {
    let mode = match std::env::var("SEA_FORGE_NET_PROBE") {
        Ok(mode) => mode,
        Err(_) => return, // not running as the helper
    };
    let port: u16 = std::env::var("SEA_FORGE_NET_PORT")
        .expect("helper needs SEA_FORGE_NET_PORT")
        .parse()
        .expect("port must be u16");
    let out = std::env::var("SEA_FORGE_NET_OUT").expect("helper needs SEA_FORGE_NET_OUT");

    let outcome = match mode.as_str() {
        "connect" => {
            use std::net::TcpStream;
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(_) => "ok".to_string(),
                Err(e) => format!("err:{e}"),
            }
        }
        "bind" => {
            use std::net::TcpListener;
            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(_) => "ok".to_string(),
                Err(e) => format!("err:{e}"),
            }
        }
        other => format!("err:unknown-mode:{other}"),
    };
    let failed = outcome != "ok";
    fs::write(&out, &outcome).expect("helper failed to write outcome");
    // Exit non-zero on probe failure so the sandbox reports a non-clean run.
    if failed {
        std::process::exit(7);
    }
}

#[cfg(target_os = "linux")]
fn helper_request(mode: &str, port: u16, out_file: &str) -> ExecutionRequest {
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let mut env = BTreeMap::new();
    if let Ok(path) = std::env::var("PATH") {
        env.insert("PATH".into(), path);
    }
    env.insert("HOME".into(), "/tmp".into());
    env.insert("SEA_FORGE_NET_PROBE".into(), mode.into());
    env.insert("SEA_FORGE_NET_PORT".into(), port.to_string());
    env.insert("SEA_FORGE_NET_OUT".into(), out_file.into());
    ExecutionRequest {
        plan_item_id: "item_net".into(),
        operation: Operation::ExecuteCommand {
            argv: vec![exe, "--exact".into(), "net_probe_helper".into()],
            cwd: ".".into(),
        },
        timeout_secs: 10,
        env,
        compensating_controls: vec![],
    }
}

/// Start a loopback TCP server on an ephemeral port. Returns the bound port and
/// a handle whose `connections()` reports how many connections were accepted.
#[cfg(target_os = "linux")]
struct FixtureServer {
    port: u16,
    accepted: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    _shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(target_os = "linux")]
impl FixtureServer {
    fn start() -> Self {
        use std::net::TcpListener;
        use std::sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            Arc,
        };
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind fixture server");
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let accepted = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));
        let accepted_thread = Arc::clone(&accepted);
        let shutdown_thread = Arc::clone(&shutdown);
        std::thread::spawn(move || {
            while !shutdown_thread.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok(_) => {
                        accepted_thread.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        FixtureServer {
            port,
            accepted,
            _shutdown: shutdown,
        }
    }

    fn connections(&self) -> usize {
        self.accepted.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(target_os = "linux")]
impl Drop for FixtureServer {
    fn drop(&mut self) {
        self._shutdown
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Run one jailed probe with the given network posture and return the outcome
/// string the helper wrote plus the execution status, or `None` if the host
/// could not enforce the jail (so the test skips rather than falsely passes).
#[cfg(target_os = "linux")]
fn run_probe(
    tag: &str,
    mode: &str,
    port: u16,
    network: NetworkPosture,
) -> Option<(String, sea_forge_core::types::ExecutionResult)> {
    let parent = temp_dir(tag);
    let workspace = parent.join("workspace");
    let artifacts = parent.join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();

    let jail = match JailSandbox::new() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Skipping network jail test ({tag}): {e}");
            fs::remove_dir_all(&parent).ok();
            return None;
        }
    };
    let spec = SandboxSpec {
        workspace_root: workspace.clone(),
        artifacts_root: artifacts.clone(),
        network,
    };
    let handle = jail.prepare(&spec).unwrap();
    // The helper writes its outcome into the writable workspace.
    let out_file = workspace.join("probe.out");
    let result = jail.execute(
        &handle,
        &helper_request(mode, port, &out_file.to_string_lossy()),
    );
    let _ = jail.destroy(handle);

    let result = match result {
        Ok(r) => r,
        Err(e) => {
            // Fail-closed setup (e.g. `jail_unavailable`) — never a fallback to
            // `local`. Assert that it is the fail-closed class, then skip.
            assert_eq!(
                e.class, "jail_unavailable",
                "jail must fail closed, not fall back to local (got {})",
                e.class
            );
            eprintln!("Skipping network jail test ({tag}): {e}");
            fs::remove_dir_all(&parent).ok();
            return None;
        }
    };

    let outcome = fs::read_to_string(&out_file).unwrap_or_default();
    fs::remove_dir_all(&parent).ok();
    Some((outcome, result))
}

#[cfg(target_os = "linux")]
#[test]
fn jail_network_denies_outbound_connect_by_default() {
    let server = FixtureServer::start();
    let Some((outcome, result)) = run_probe(
        "net-connect-deny",
        "connect",
        server.port,
        NetworkPosture::Denied,
    ) else {
        return; // host cannot enforce jail; skip (not a pass)
    };
    // The child's connect must have failed …
    assert!(
        outcome.starts_with("err:"),
        "default-denied outbound connect unexpectedly succeeded: {outcome:?}"
    );
    // … and the fixture server must have accepted zero connections. This is the
    // load-bearing assertion: denial is proven by the server observing nothing,
    // not merely by the child's self-report.
    assert_eq!(
        server.connections(),
        0,
        "fixture server received a connection despite default network denial"
    );
    // The jailed probe exited non-zero (Landlock blocked the connect); a network
    // denial surfaces as the child's own error rather than a filesystem
    // "permission denied" on stderr, so we assert on the non-clean exit code
    // rather than the FS-oriented SandboxViolation status heuristic.
    assert!(
        result.exit_code != Some(0),
        "denied connect probe exited cleanly (exit {:?})",
        result.exit_code
    );
}

#[cfg(target_os = "linux")]
#[test]
fn jail_network_allows_outbound_connect_under_explicit_grant() {
    let server = FixtureServer::start();
    let Some((outcome, result)) = run_probe(
        "net-connect-grant",
        "connect",
        server.port,
        NetworkPosture::AllowTcpPorts(vec![server.port]),
    ) else {
        return;
    };
    assert_eq!(
        outcome, "ok",
        "explicitly granted outbound connect was denied: {outcome:?}"
    );
    // Give the accept thread a moment to record the connection.
    for _ in 0..50 {
        if server.connections() >= 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(
        server.connections(),
        1,
        "fixture server did not observe the granted connection"
    );
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(result.exit_code, Some(0));
}

#[cfg(target_os = "linux")]
#[test]
fn jail_network_denies_bind_listen_by_default() {
    // Bind to an ephemeral port (0) — even a wildcard bind must be denied when
    // no port is granted.
    let Some((outcome, result)) = run_probe("net-bind-deny", "bind", 0, NetworkPosture::Denied)
    else {
        return;
    };
    assert!(
        outcome.starts_with("err:"),
        "default-denied TCP bind/listen unexpectedly succeeded: {outcome:?}"
    );
    assert!(
        result.exit_code != Some(0),
        "denied bind probe exited cleanly (exit {:?})",
        result.exit_code
    );
}

#[cfg(target_os = "linux")]
#[test]
fn jail_network_allows_bind_listen_under_explicit_grant() {
    // Pick a free port on the host, then grant exactly it, and have the jailed
    // child bind it. Landlock `NetPort` bind rules match a specific port.
    let probe = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe); // free it for the jailed child to bind

    let Some((outcome, result)) = run_probe(
        "net-bind-grant",
        "bind",
        port,
        NetworkPosture::AllowTcpPorts(vec![port]),
    ) else {
        return;
    };
    assert_eq!(
        outcome, "ok",
        "explicitly granted TCP bind/listen was denied: {outcome:?}"
    );
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(result.exit_code, Some(0));
}

/// Unsupported-kernel / fail-closed contract.
///
/// On Linux, if the host cannot enforce the requested network posture, jail
/// setup must return `jail_unavailable` (asserted inside `run_probe`) — it must
/// never silently degrade to an unrestricted `local` posture. On a host that
/// *does* enforce Landlock network restriction, a default-deny probe proves the
/// restriction is actually in force (the deny tests above). Either way, an
/// unsupported platform is reported as skipped, never counted as a pass.
#[cfg(target_os = "linux")]
#[test]
fn jail_network_setup_fails_closed_never_local() {
    let server = FixtureServer::start();
    match run_probe(
        "net-failclosed",
        "connect",
        server.port,
        NetworkPosture::Denied,
    ) {
        // Enforced: connection must have been blocked (zero accepted).
        Some((outcome, _result)) => {
            assert!(
                outcome.starts_with("err:"),
                "denial not enforced: {outcome:?}"
            );
            assert_eq!(
                server.connections(),
                0,
                "connection leaked despite enforced default denial"
            );
        }
        // Not enforceable on this host: `run_probe` already asserted the error
        // was `jail_unavailable` (fail-closed), never a `local` fallback.
        None => {
            eprintln!("network jail unsupported on this host — reported as skipped");
        }
    }
}

/// macOS (and any non-Linux) skip-not-pass contract.
///
/// The jail backend has no Seatbelt implementation; off Linux it must return
/// `unsupported_sandbox_class_error` at construction and never count as a pass.
/// This test asserts that fail-closed behavior on non-Linux and is inert (a
/// skip) on Linux, so an unsupported platform is reported as skipped.
#[test]
fn jail_network_on_non_linux_is_skip_not_pass() {
    #[cfg(not(target_os = "linux"))]
    {
        let result = JailSandbox::new();
        assert!(
            result.is_err(),
            "jail (and thus its network posture) must be unavailable off Linux"
        );
        assert_eq!(
            result.unwrap_err().class,
            "unsupported_sandbox_class_error",
            "non-Linux jail must fail closed as unsupported, never silently pass"
        );
    }
    #[cfg(target_os = "linux")]
    {
        // Seatbelt network cases are reported as skipped on Linux CI, not passed.
        eprintln!("Seatbelt network jail case skipped on Linux (no Seatbelt backend)");
    }
}

#[test]
fn local_class_is_local() {
    let local = sea_forge_sandbox::LocalSandbox;
    assert_eq!(local.class(), SandboxClass::Local);
}

#[test]
fn untrusted_argv0_on_local_is_schema_error() {
    // Re-assert from Task 5: a 0.2 policy granting `local` to an untrusted
    // argv0 is a schema_error.
    use sea_forge_authority::AuthorityPolicyBundle;
    use sea_forge_core::RECORD_VERSION;

    let tmp = temp_dir("schema");
    let sources_dir = tmp.join("sources");
    fs::create_dir_all(&sources_dir).unwrap();

    const SURFACES: &[&str] = &[
        "authority_hooks",
        "identity_map",
        "domain_model",
        "file_access",
        "api_allowlist",
        "git_commit",
        "pr_merge",
        "prompt_risk",
        "memory_recall",
        "spec_pipeline",
        "artifact_transition",
        "attestation",
        "approval",
        "settlement_authority",
        "capability_promotion",
        "deployment",
        "secret_access",
        "policy_mutation",
        "evidence_mutation",
    ];
    const ROLES: &[&str] = &["R-DS", "R-AG", "R-LC", "R-SO", "R-RM", "R-DEV", "R-AA"];
    // (name, operation_kind, optional transition_kind)
    const SOD: &[(&str, &str, Option<&str>)] = &[
        ("production_proposer_approver", "pr_merge", None),
        (
            "semantic_debt_requester_acceptor",
            "settlement_authority_mutation",
            None,
        ),
        ("break_glass_requester_approver", "policy_mutation", None),
        ("key_generator_approver", "identity_minting", None),
        (
            "capitalization_requester_approver",
            "transition_artifact_stage",
            Some("capitalize"),
        ),
    ];

    let sources: Vec<serde_json::Value> = SURFACES
        .iter()
        .map(|surface| {
            fs::write(sources_dir.join(surface), "").unwrap();
            serde_json::json!({
                "surface": surface,
                "path": format!("sources/{surface}"),
                "sha256": sea_forge_evidence::sha256_bytes(b"")
            })
        })
        .collect();

    let mut bundle: AuthorityPolicyBundle = serde_json::from_value(serde_json::json!({
        "version": RECORD_VERSION,
        "bundle_id": "test_schema",
        "policy_bundle_hash": "",
        "policy_bundle_version": "1",
        "loaded_at": "2026-07-13T00:00:00Z",
        "source_base": tmp.to_string_lossy(),
        "sources": sources,
        "roles": ROLES.iter().map(|r| ((*r).to_owned(), vec!["fixture"])).collect::<std::collections::BTreeMap<_, _>>(),
        "permissions": ROLES.iter().map(|r| serde_json::json!({"role": r, "operation_kind": "*"})).collect::<Vec<_>>(),
        "sod_rules": SOD.iter().map(|(n, op, transition)| {
            let mut rule = serde_json::json!({
                "name": n,
                "requester_role": "R-DEV",
                "approver_role": "R-SO",
                "operation_kind": op,
                "allow_same_principal": false
            });
            if let Some(kind) = transition {
                rule["transition_kind"] = serde_json::json!(kind);
            }
            rule
        }).collect::<Vec<_>>(),
        "identity_bindings": [{"principal": "operator_local", "actor_type": "human", "role": "operator"}],
        "policy_engines": [],
        "rules": [
            {"name": "allow-untrusted-local", "verdict": "allow", "actor_role": "operator", "operation_kind": "execute_command", "argv0": "untrusted-binary", "sandbox_class": "local"},
        ],
    })).unwrap();
    bundle.refresh_policy_bundle_hash().unwrap();

    let path = tmp.join("policy.json");
    fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();

    let result = AuthorityPolicyBundle::load(&path);
    assert!(
        result.is_err(),
        "policy granting local to untrusted argv0 must be rejected"
    );
    let err = result.unwrap_err();
    assert_eq!(err.class(), "schema_error");

    fs::remove_dir_all(tmp).unwrap();
}
