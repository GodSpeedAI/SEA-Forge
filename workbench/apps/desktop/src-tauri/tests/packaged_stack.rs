//! The packaged stack, proven against the artifact that actually ships.
//!
//! `tests/bridge.rs` boots the server *in process*, as a library, which proves
//! the transport but not the product: a library call cannot fail to be
//! installed, cannot fail to be found beside the application, and cannot be
//! left running by a previous window. This file boots the **staged sidecar
//! binary** — byte-for-byte the file `bundle.externalBin` copies into the
//! `.deb` — through the **real `CellSupervisor`**, and then talks to it over a
//! real Unix socket with the host's own client.
//!
//! What that covers, which nothing else did:
//!   * a clean cell with nothing listening ends up serving SFWP, with no
//!     operator step in between (decision U-06);
//!   * a second window adopts the first one's kernel instead of starting a
//!     rival or refusing to open;
//!   * closing a window stops only a kernel it started;
//!   * records survive the kernel being stopped and started again;
//!   * a cell that cannot start says why, instead of hanging or looking like a
//!     connection failure.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use app_lib::socket::SocketHandle;
use app_lib::supervisor::{is_listening, Cell, CellSupervisor, Supervision};
use serde_json::{json, Value};
use tokio::sync::mpsc;

/// The staged sidecar: `binaries/sea-forge-server-<target-triple>`.
///
/// Tauri's build script refuses to build this crate when that file is absent,
/// so if this test is running at all, the binary is there. Resolved by prefix
/// rather than by reconstructing the triple, which `std::env::consts` does not
/// expose.
fn sidecar() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    let entry = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| {
            panic!(
                "cannot read {}: {e} — run `just workbench-sidecar`",
                dir.display()
            )
        })
        .filter_map(Result::ok)
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("sea-forge-server-")
        })
        .unwrap_or_else(|| {
            panic!(
                "no staged sidecar in {} — run `just workbench-sidecar`",
                dir.display()
            )
        });
    entry.path()
}

/// A cell root short enough for a Unix socket path. The enforced budget is 95
/// bytes, and `TempDir` under a deep `TMPDIR` can exceed it on its own — which
/// is a real failure this repo has hit before, so it is worth not re-hitting in
/// the test harness.
fn scratch_cell() -> (tempfile::TempDir, Cell) {
    let dir = tempfile::TempDir::with_prefix_in("sf-pkg-", "/tmp").unwrap();
    let cell = Cell {
        root: dir.path().to_path_buf(),
        socket: dir.path().join("server.sock"),
    };
    (dir, cell)
}

/// Bind the current uid so protected verbs are answerable. A cell that
/// configures no identity refuses every protected verb (SF-005), which is
/// correct but would make this test prove only that refusals work.
fn write_identity(cell: &Cell) {
    let uid = users_uid();
    std::fs::create_dir_all(&cell.root).unwrap();
    std::fs::write(
        cell.root.join("server.yaml"),
        format!(
            "identity:\n  bindings:\n    - uid: {uid}\n      actor_id: operator_a\n      roles: [operator]\n"
        ),
    )
    .unwrap();
}

fn users_uid() -> u32 {
    // No `libc` dependency in this crate; `id -u` is available anywhere this
    // test can run, and the value only has to agree with what SO_PEERCRED will
    // report for this same process.
    let out = std::process::Command::new("id").arg("-u").output().unwrap();
    String::from_utf8_lossy(&out.stdout).trim().parse().unwrap()
}

/// Talk to a cell the way the Tauri commands do: through `SocketHandle::call`.
async fn call(cell: &Cell, request: Value) -> Value {
    let (tx, _rx) = mpsc::unbounded_channel();
    let handle = SocketHandle::new(cell.socket.clone(), tx);
    handle.call(request).await.expect("call failed")
}

fn wait_until_gone(socket: &std::path::Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if !is_listening(socket) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

/// The headline claim of U-06: install the product, open it, and there is a
/// kernel — no second binary to find, no service to start, no source tree.
#[tokio::test]
async fn a_cold_cell_is_started_by_the_workbench_and_serves_sfwp() {
    let (_dir, cell) = scratch_cell();
    write_identity(&cell);
    assert!(!is_listening(&cell.socket), "the cell must start cold");

    let supervisor = CellSupervisor::start_with(cell.clone(), Ok(sidecar()));
    match supervisor.supervision() {
        Supervision::Supervised { pid } => assert!(*pid > 0),
        other => panic!("expected a supervised cell, got {other:?}"),
    }

    let hello = call(
        &cell,
        json!({"verb": "system_hello", "protocol_version": "1"}),
    )
    .await;
    let methods = hello["implemented_methods"]
        .as_array()
        .expect("hello must advertise a method catalog");
    assert!(
        methods.iter().any(|m| m == "identity.get"),
        "the catalog the packaged kernel serves is missing identity.get: {hello}"
    );

    // The socket the sidecar published must be owner-only. This is the
    // property that keeps another local user off a governed kernel, and it is
    // the packaged path's version of it — not the library path's.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&cell.socket)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "socket must be 0600, got {:o}", mode);
    }

    supervisor.shutdown();
}

/// Opening a second window must not start a rival kernel over the first one's
/// records, and must not refuse to open either.
#[tokio::test]
async fn a_second_window_adopts_the_running_cell() {
    let (_dir, cell) = scratch_cell();
    write_identity(&cell);

    let first = CellSupervisor::start_with(cell.clone(), Ok(sidecar()));
    assert!(matches!(
        first.supervision(),
        Supervision::Supervised { .. }
    ));

    let second = CellSupervisor::start_with(cell.clone(), Ok(sidecar()));
    assert_eq!(
        second.supervision(),
        &Supervision::Adopted,
        "a live cell must be adopted, not restarted"
    );

    // Closing the adopting window must leave the kernel it did not start alone.
    // Getting this wrong would mean quitting one window kills another window's
    // in-flight work.
    second.shutdown();
    assert!(
        is_listening(&cell.socket),
        "shutting down an adopting window stopped a kernel it did not start"
    );

    let hello = call(
        &cell,
        json!({"verb": "system_hello", "protocol_version": "1"}),
    )
    .await;
    assert_eq!(hello["protocol_version"], "1");

    first.shutdown();
    assert!(
        wait_until_gone(&cell.socket),
        "the window that started the kernel must stop it"
    );
}

/// Records are the product. They must outlive the process that wrote them,
/// and the cell must come back up on the same root.
#[tokio::test]
async fn records_survive_the_kernel_being_stopped_and_started_again() {
    let (_dir, cell) = scratch_cell();
    write_identity(&cell);

    let first = CellSupervisor::start_with(cell.clone(), Ok(sidecar()));
    assert!(matches!(
        first.supervision(),
        Supervision::Supervised { .. }
    ));

    // Commit something real. A policy with no rules denies by default, which
    // settles the run as rejected without needing an agent endpoint or a
    // runnable command — and a denial is still a fully recorded outcome
    // (authority, evidence, trace, settlement), which is exactly what has to
    // survive. `readiness.get` would not do: it is an inspect verb and
    // correctly writes nothing.
    let policy = cell.root.join("deny.yaml");
    std::fs::write(&policy, "version: \"0.1\"\nrules: []\n").unwrap();
    let plan = cell.root.join("plan.json");
    std::fs::write(
        &plan,
        json!({
            "version": "0.2",
            "plan_id": "plan_restart",
            "case_id": "case_placeholder",
            "run_id": "run_placeholder",
            "intent_id": "int_restart",
            "items": [{
                "plan_item_id": "task",
                "name": "work the cell will refuse",
                "operations": [{"kind": "execute_command", "argv": ["/bin/true"], "cwd": "."}],
                "entry_criteria": [], "exit_criteria": [],
                "settlement_criteria": {"require_exit_zero": true},
                "item_kind": "sandboxed_task",
                "markers": {"required": true},
                "max_instances": 1, "depends_on": [],
            }],
        })
        .to_string(),
    )
    .unwrap();

    let submitted = call(
        &cell,
        json!({
            "verb": "submit",
            "actor": {"actor_id": "operator_a", "role": "operator"},
            "plan": plan, "policy": policy,
            "entity": "operator_a", "process": "restart-proof", "timeout": 60,
        }),
    )
    .await;
    assert!(
        submitted.get("error_class").is_none(),
        "submit was refused: {submitted}"
    );

    let listed = call(&cell, json!({"verb": "run_list"})).await;
    let runs = listed["runs"].as_array().cloned().unwrap_or_default();
    assert_eq!(runs.len(), 1, "expected exactly one run, got {listed}");
    let run_id = runs[0]["run_id"]
        .as_str()
        .or_else(|| runs[0].as_str())
        .expect("a run must be identifiable")
        .to_owned();

    let before = call(&cell, json!({"verb": "run_get", "run_id": run_id})).await;
    assert!(
        before.get("error_class").is_none(),
        "run.get failed: {before}"
    );

    first.shutdown();
    assert!(wait_until_gone(&cell.socket));

    // Restart on the same root. A stale socket file from the abrupt stop must
    // not block the rebind — the server stages and renames, so it replaces it.
    let second = CellSupervisor::start_with(cell.clone(), Ok(sidecar()));
    match second.supervision() {
        Supervision::Supervised { .. } => {}
        other => panic!("the cell did not come back up: {other:?}"),
    }

    // The record must still resolve, and say the same thing. A restart that
    // loses a settled run, or reports it differently, has lost the evidence the
    // product exists to keep.
    let after = call(&cell, json!({"verb": "run_get", "run_id": run_id})).await;
    assert!(
        after.get("error_class").is_none(),
        "the run stopped resolving after a restart: {after}"
    );
    assert_eq!(
        after, before,
        "the same run reported differently before and after a restart"
    );

    let relisted = call(&cell, json!({"verb": "run_list"})).await;
    assert_eq!(
        relisted["runs"].as_array().map(Vec::len),
        Some(1),
        "the run went missing from the listing after a restart: {relisted}"
    );

    second.shutdown();
}

/// A cell that cannot start must say why. The failure mode this guards against
/// is the worst one for an installed application: a window that opens onto
/// nothing and reports only that some call failed.
#[tokio::test]
async fn a_cell_that_cannot_start_reports_the_reason_and_the_remedy() {
    // A socket path past the enforced 95-byte budget. The server refuses this
    // before it creates anything, and its refusal names both remedies.
    let dir = tempfile::TempDir::with_prefix_in("sf-pkg-", "/tmp").unwrap();
    let deep = dir.path().join("a".repeat(120));
    std::fs::create_dir_all(&deep).unwrap();
    let cell = Cell {
        root: deep.clone(),
        socket: deep.join("server.sock"),
    };

    let started = Instant::now();
    let supervisor = CellSupervisor::start_with(cell, Ok(sidecar()));
    match supervisor.supervision() {
        Supervision::Unavailable {
            error_class,
            message,
        } => {
            assert_eq!(error_class, "server_start_refused");
            assert!(
                message.contains("SEA_FORGE_ROOT") || message.contains("SEA_FORGE_SOCKET"),
                "the refusal must name a remedy, got: {message}"
            );
        }
        other => panic!("expected an explained refusal, got {other:?}"),
    }
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "a refusing server must be noticed, not waited out"
    );
}
