#![cfg(unix)]
//! SEC-02/SEC-08/SEC-09 live-connection lifecycle against a real PostgreSQL
//! cluster and a real isolated workload that resists: every connection
//! first disables the per-session timeouts a role can carry
//! (`statement_timeout`, `idle_session_timeout`,
//! `idle_in_transaction_session_timeout`, `transaction_timeout`), which
//! PostgreSQL lets any session `SET`. Enforcement must therefore come from
//! the trusted server, not from the session's own settings.
//!
//! Needs `SEA_FORGE_SECURITY_PG_INSTALL` and workload isolation; otherwise
//! NOT-EXECUTABLE (set `SEA_FORGE_REQUIRE_POSTGRES=1` to fail instead).
#![cfg(target_os = "linux")]

use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_ledger::LedgerStream;
use sea_forge_sandbox::workload::{
    grant_exclusive_access, isolated_command, WorkloadConfinement, WorkloadIdentity,
};
use sea_forge_sandbox::NetworkPosture;
use sea_forge_server::exposure::ExposureRegistry;
use sea_forge_server::postgres::{DatabaseSession, EpisodeSessions, LiveSessions, PostgresBroker};

#[path = "support/pg_cluster.rs"]
mod pg_cluster;
use pg_cluster::{prerequisites, start_cluster};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: audit-reader-session\n    verdict: allow\n    actor_role: operator\n    operation_kind: database_session\n    boundary_constraints:\n      db_resource: [app-db]\n      db_scope: [audit-read]\n";

/// Disables every timeout a session can carry, then runs `sql`.
const RESIST: &str = "SET statement_timeout = 0; SET idle_session_timeout = 0; \
     SET idle_in_transaction_session_timeout = 0; SET transaction_timeout = 0;";

fn workload_dirs(base: &Path, identity: &WorkloadIdentity) -> WorkloadConfinement {
    let root = base.join("agent");
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        read_paths: Vec::new(),
        socket_paths: Vec::new(),
        network: NetworkPosture::Denied,
    };
    fs::create_dir_all(&confinement.workspace_root).unwrap();
    fs::create_dir_all(&confinement.artifacts_root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    grant_exclusive_access(&confinement.workspace_root, identity).unwrap();
    grant_exclusive_access(&confinement.artifacts_root, identity).unwrap();
    confinement
}

fn spawn_workload(
    identity: &WorkloadIdentity,
    helper: &Path,
    dirs: &WorkloadConfinement,
    install: &Path,
    conninfo: &str,
    script: &str,
) -> Child {
    // SEC-06: the workload may connect to the PostgreSQL socket directory
    // its conninfo names, and to no other pathname socket.
    let socket_dir = conninfo
        .split_whitespace()
        .find_map(|pair| pair.strip_prefix("host="))
        .filter(|host| host.starts_with('/'))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/nonexistent-socket-dir"));
    let dirs = WorkloadConfinement {
        read_paths: vec![install.canonicalize().unwrap()],
        socket_paths: vec![socket_dir],
        ..dirs.clone()
    };
    let (mut command, lifeline, mut ack) = isolated_command(
        identity,
        helper,
        &dirs,
        &["/bin/sh".into(), "-c".into(), script.into()],
    )
    .unwrap();
    // The launcher is treated as gone: nothing it holds may matter.
    std::mem::forget(lifeline);
    let child = command
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", install.join("bin").display()),
        )
        .env("LD_LIBRARY_PATH", install.join("lib"))
        .env("PGCONN", conninfo)
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    // Review pass 2 (R4): consume the trusted setup acknowledgement; the
    // helper fails its launch when its launcher never reads the ack byte.
    sea_forge_sandbox::workload::wait_for_isolation_ack(
        &mut ack,
        sea_forge_sandbox::workload::ISOLATION_ACK_TIMEOUT,
    )
    .expect("isolated launch acknowledged its setup");
    child
}

fn wait_until(what: &str, limit: Duration, mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < limit, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn read_marker(path: &Path, marker: &str) -> BTreeMap<String, String> {
    let mut rows = BTreeMap::new();
    wait_until(marker, Duration::from_secs(40), || {
        let text = fs::read_to_string(path).unwrap_or_default();
        rows = text
            .lines()
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        text.lines().any(|l| l == marker)
    });
    rows
}

fn role_exists(broker: &PostgresBroker, role: &str) -> bool {
    broker
        .admin_sql(&format!(
            "SELECT count(*) FROM pg_roles WHERE rolname = '{role}'"
        ))
        .unwrap()
        != "0"
}

struct Fixture {
    cluster: pg_cluster::Cluster,
    install: std::path::PathBuf,
    helper: std::path::PathBuf,
    identity: WorkloadIdentity,
    dirs: WorkloadConfinement,
    broker: PostgresBroker,
    ledger: LedgerStream,
    _state: tempfile::TempDir,
    _pool: sea_forge_sandbox::workload::WorkloadIdentityPool,
}

fn fixture(run: &str, ttl: u64, permitted: &[&str]) -> Option<Fixture> {
    let (install, pool, helper, state) = prerequisites()?;
    let mut cluster = start_cluster(&install);
    cluster.config.session_ttl_secs = ttl;
    cluster.config.permitted_exposure = BTreeMap::from([(
        "audit-read".to_string(),
        permitted.iter().map(|l| l.to_string()).collect(),
    )]);
    let broker = PostgresBroker::new(cluster.config.clone()).unwrap();
    let ledger = LedgerStream::open(&cluster.base, run, "sea-forge-control-plane").unwrap();
    let identity = pool.acquire().unwrap();
    let dirs = workload_dirs(&cluster.base, &identity);
    Some(Fixture {
        cluster,
        install,
        helper,
        identity,
        dirs,
        broker,
        ledger,
        _state: state,
        _pool: pool,
    })
}

fn issue(f: &Fixture, run: &str) -> DatabaseSession {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "audit-agent".into(),
        role: ActorRole::Operator,
    };
    f.broker
        .issue(
            &engine,
            &bundle,
            &actor,
            &f.ledger,
            run,
            &f.dirs.workspace_root,
            "audit-read",
            f.identity.host_uid,
        )
        .unwrap()
        .unwrap()
}

/// Busy, idle, and idle-in-transaction connections that disable their own
/// timeouts; once all end, a reconnection attempt is reported.
fn resisting_script(results: &Path) -> String {
    format!(
        r#"# Keeps the connection open, idle between trivial queries; psql
# exits once it finds the connection gone, and the next write fails.
hold() {{ while echo 'SELECT 1;'; do sleep 0.5; done 2>/dev/null; }}
psql "$PGCONN" -X -At -c "{RESIST}" -c 'SELECT pg_sleep(120)' >/dev/null 2>&1 &
( echo "{RESIST}"; hold ) | psql "$PGCONN" -X -At >/dev/null 2>&1 &
( echo "{RESIST}"; echo 'BEGIN; SELECT 1;'; hold ) | psql "$PGCONN" -X -At >/dev/null 2>&1 &
wait
{{ psql "$PGCONN" -X -At -c 'SELECT 1' >/dev/null 2>&1 && echo "reconnect:ok" || echo "reconnect:refused"
echo done; }} > {results}"#,
        results = results.display()
    )
}

/// ADV-B07 follow-up: expiry is enforced on live connections without any
/// later issuance, without the issuing launcher, and against a session
/// that disabled its own timeouts.
#[test]
fn supervisor_ends_resisting_connections_at_expiry_without_issuance_or_launcher() {
    let run = "run-pg-supervise";
    let Some(f) = fixture(run, 3, &[]) else {
        return;
    };
    // No expiry reaper: the launcher that would own it is gone.
    let issued_at = Instant::now();
    let session = issue(&f, run);
    let results = f.dirs.workspace_root.join("results");
    let mut child = spawn_workload(
        &f.identity,
        &f.helper,
        &f.dirs,
        &f.install,
        &session.conninfo(),
        &resisting_script(&results),
    );
    wait_until("three backends", Duration::from_secs(20), || {
        f.broker.active_backends(&session.role).unwrap().len() == 3
    });
    let states = f
        .broker
        .admin_sql(&format!(
            "SELECT string_agg(state, ',' ORDER BY state) FROM pg_stat_activity \
             WHERE usename = '{}'",
            session.role
        ))
        .unwrap();
    assert_eq!(states, "active,idle,idle in transaction");
    // TTL 3 s; 5 s after issuance is past VALID UNTIL and past any
    // timeout bound to the session lifetime.
    wait_until("expiry", Duration::from_secs(10), || {
        issued_at.elapsed() > Duration::from_secs(5)
    });
    // Control: past VALID UNTIL, nothing inside the database ended them.
    assert_eq!(
        f.broker.active_backends(&session.role).unwrap().len(),
        3,
        "resisting backends ended without the supervisor"
    );

    let _supervisor = f
        .broker
        .supervise(f.cluster.base.clone(), Duration::from_millis(500));
    wait_until("supervisor revocation", Duration::from_secs(10), || {
        !role_exists(&f.broker, &session.role)
    });
    assert!(f.broker.active_backends(&session.role).unwrap().is_empty());
    let rows = read_marker(&results, "done");
    assert_eq!(rows["reconnect"], "refused");
    child.wait().unwrap();

    let supervised = LedgerStream::open(
        &f.cluster.base,
        "dbsupervisor-app-db",
        "sea-forge-control-plane",
    )
    .unwrap();
    let revoked: Vec<_> = supervised
        .read_entries()
        .unwrap()
        .into_iter()
        .filter(|e| e.record_kind == "database_session_revoked")
        .collect();
    assert_eq!(revoked.len(), 1);
    assert_eq!(revoked[0].payload["native_role"], session.role.as_str());
    assert_eq!(revoked[0].payload["reason"], "expired_unreaped");
    assert_eq!(revoked[0].payload["terminated_backends"], 3);
    assert_eq!(revoked[0].payload["remaining_backends"], 0);
}

/// SEC-09: an exposure transition revokes already-issued credentials on
/// their live connections before the gate returns, across episodes: the
/// workload's own session, and another episode's session in a context
/// derived from the exposed one. A label the scope permits, and a
/// transition in an unrelated context, revoke nothing.
#[test]
fn exposure_transition_revokes_issued_sessions_across_episodes_before_gate_returns() {
    let run = "run-pg-exposure";
    let Some(f) = fixture(run, 600, &["confidential"]) else {
        return;
    };
    let registry = ExposureRegistry::open(&f.cluster.base, "sea-forge-control-plane").unwrap();
    registry.derive("case:c1", "case:c1-child").unwrap();
    let live = LiveSessions::default();
    let push = |episode: &EpisodeSessions, context: &str, session: DatabaseSession| {
        let reaper = f.broker.expiry_reaper(
            &session,
            f.cluster.base.clone(),
            run.to_string(),
            run.to_string(),
        );
        live.push(
            episode.owner(),
            context,
            f.cluster.base.clone(),
            run.to_string(),
            PostgresBroker::new(f.cluster.config.clone()).unwrap(),
            session,
            reaper,
        )
        .unwrap();
    };

    // Episode A: a resisting workload with three live connections.
    let session = issue(&f, run);
    let role = session.role.clone();
    let results = f.dirs.workspace_root.join("results");
    let mut child = spawn_workload(
        &f.identity,
        &f.helper,
        &f.dirs,
        &f.install,
        &session.conninfo(),
        &resisting_script(&results),
    );
    wait_until("three backends", Duration::from_secs(20), || {
        f.broker.active_backends(&role).unwrap().len() == 3
    });
    let a = live.episode(run);
    push(&a, "case:c1", session);
    // Episode B (delegated, derived context) and C (unrelated case).
    let b = live.episode("run-child");
    let b_session = issue(&f, run);
    let b_role = b_session.role.clone();
    push(&b, "case:c1-child", b_session);
    let c = live.episode("run-other");
    let c_session = issue(&f, run);
    let c_role = c_session.role.clone();
    push(&c, "case:c9", c_session);
    let gate = live.gate(f.cluster.base.clone());

    // Positive control: a permitted label keeps every session and connection.
    gate("case:c1", "confidential").unwrap();
    registry
        .record("case:c1", "confidential", "test", "e1")
        .unwrap();
    assert_eq!(f.broker.active_backends(&role).unwrap().len(), 3);
    for r in [&role, &b_role, &c_role] {
        assert!(role_exists(&f.broker, r), "{r}");
    }

    // A label outside the scope's permitted exposure: A and B revoked and
    // verified by the time the gate returns, with no polling; C untouched.
    gate("case:c1", "restricted").unwrap();
    assert!(f.broker.active_backends(&role).unwrap().is_empty());
    assert!(!role_exists(&f.broker, &role));
    assert!(!role_exists(&f.broker, &b_role));
    assert!(role_exists(&f.broker, &c_role));
    let rows = read_marker(&results, "done");
    assert_eq!(rows["reconnect"], "refused");
    child.wait().unwrap();

    // Teardown is per episode and tolerates already-revoked sessions; a
    // dropped episode guard revokes what it still holds.
    live.revoke_all(a.owner(), "episode_ended").unwrap();
    drop((a, b));
    assert!(role_exists(&f.broker, &c_role));
    drop(c);
    assert!(!role_exists(&f.broker, &c_role));
    let reasons: Vec<String> = f
        .ledger
        .read_entries()
        .unwrap()
        .into_iter()
        .filter(|e| e.record_kind == "database_session_revoked")
        .map(|e| e.payload["reason"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        reasons,
        [
            "exposure_not_permitted",
            "exposure_not_permitted",
            "episode_aborted"
        ]
    );
}
