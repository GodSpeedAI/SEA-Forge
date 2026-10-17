//! SEC-05/SEC-02/SEC-08 against a real PostgreSQL cluster and a real
//! isolated workload. PostgreSQL's own refusals (SQLSTATE 42501), its
//! `pg_stat_activity`, and a launcher-side digest of the protected table are
//! the observations; no database behavior is simulated.
//!
//! Needs `SEA_FORGE_SECURITY_PG_INSTALL` (a PostgreSQL install with `bin/`
//! and `lib/`, e.g. pg0's `~/.pg0/installation/18.1.0`) and workload
//! isolation; otherwise NOT-EXECUTABLE (set `SEA_FORGE_REQUIRE_POSTGRES=1`
//! to fail instead).
#![cfg(target_os = "linux")]

use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_ledger::LedgerStream;
use sea_forge_sandbox::workload::{
    grant_exclusive_access, isolated_command, WorkloadConfinement, WorkloadIdentity,
};
use sea_forge_sandbox::NetworkPosture;
use sea_forge_server::postgres::PostgresBroker;

#[path = "support/pg_cluster.rs"]
mod pg_cluster;
use pg_cluster::{digest, prerequisites, start_cluster};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: audit-reader-session\n    verdict: allow\n    actor_role: operator\n    operation_kind: database_session\n    boundary_constraints:\n      db_resource: [app-db]\n      db_scope: [audit-read]\n";

fn workload_dirs(base: &Path, name: &str, identity: &WorkloadIdentity) -> WorkloadConfinement {
    let root = base.join(name);
    let confinement = WorkloadConfinement {
        workspace_root: root.join("workspace"),
        artifacts_root: root.join("artifacts"),
        network: NetworkPosture::Denied,
    };
    fs::create_dir_all(&confinement.workspace_root).unwrap();
    fs::create_dir_all(&confinement.artifacts_root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o711)).unwrap();
    grant_exclusive_access(&confinement.workspace_root, identity).unwrap();
    grant_exclusive_access(&confinement.artifacts_root, identity).unwrap();
    confinement
}

fn wait_for(path: &Path, marker: &str) -> String {
    let start = Instant::now();
    loop {
        if let Ok(text) = fs::read_to_string(path) {
            if text.lines().any(|l| l == marker) {
                return text;
            }
        }
        assert!(
            start.elapsed() < Duration::from_secs(40),
            "timed out waiting for {marker}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn verdicts(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn spawn_workload(
    identity: &WorkloadIdentity,
    helper: &Path,
    dirs: &WorkloadConfinement,
    install: &Path,
    conninfo: &str,
    script: &str,
) -> Child {
    isolated_command(
        identity,
        helper,
        dirs,
        &["/bin/sh".into(), "-c".into(), script.into()],
    )
    .unwrap()
    .env(
        "PATH",
        format!("{}:/usr/bin:/bin", install.join("bin").display()),
    )
    .env("LD_LIBRARY_PATH", install.join("lib"))
    .env("PGCONN", conninfo)
    .stdin(Stdio::null())
    .spawn()
    .unwrap()
}

#[test]
fn workload_bound_session_enforces_native_privileges_and_revokes_live_backends() {
    let Some((install, pool, helper, _state)) = prerequisites() else {
        return;
    };
    let cluster = start_cluster(&install);
    let broker = PostgresBroker::new(cluster.config.clone()).unwrap();
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "audit-agent".into(),
        role: ActorRole::Operator,
    };
    let ledger = LedgerStream::open(&cluster.base, "run-pg", "control-plane").unwrap();
    let identity = pool.acquire().unwrap();
    let other = pool.acquire().unwrap();
    let dirs = workload_dirs(&cluster.base, "agent", &identity);
    let before = digest(&broker);

    // Scope escalation: an unauthorized scope issues nothing at all.
    let refused = broker
        .issue(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-pg",
            &dirs.workspace_root,
            "audit-write",
            identity.host_uid,
        )
        .unwrap();
    assert!(refused.is_none());
    assert_eq!(
        broker
            .admin_sql("SELECT count(*) FROM pg_roles WHERE rolname LIKE 'sfw_%'")
            .unwrap(),
        "0"
    );

    let session = broker
        .issue(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-pg",
            &dirs.workspace_root,
            "audit-read",
            identity.host_uid,
        )
        .unwrap()
        .expect("authorized session");
    // A second workload's credential, never given to the first.
    let foreign = broker
        .issue(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-pg",
            &dirs.workspace_root,
            "audit-read",
            other.host_uid,
        )
        .unwrap()
        .unwrap();
    let ws = dirs.workspace_root.display();
    let admin_conn = format!(
        "host={} port={} dbname=postgres user=sea_admin",
        cluster.config.socket_dir.display(),
        cluster.config.port
    );
    let foreign_conn = format!(
        "host={} port={} dbname=postgres user={}",
        cluster.config.socket_dir.display(),
        cluster.config.port,
        foreign.role
    );
    let script = format!(
        r#"q() {{ PGAPPNAME=agent psql "$PGCONN" -X -At -v ON_ERROR_STOP=1 -v VERBOSITY=sqlstate -c "$1" 2>&1; }}
r() {{ n=$1; shift; out=$("$@" 2>&1); case "$out" in *42501*) echo "$n:42501";; *FATAL*|*failed*|*denied*|*"No such"*|*"no password"*) echo "$n:refused";; *) echo "$n:ALLOWED:$out";; esac; }}
{{
echo "read:$(q 'SELECT count(*) FROM audit_log')"
r delete q 'DELETE FROM audit_log'
r drop q 'DROP TABLE audit_log'
r truncate q 'TRUNCATE audit_log'
r superuser q 'ALTER ROLE CURRENT_USER SUPERUSER'
r set_role q 'SET ROLE audit_reader'
r self_grant q 'GRANT audit_writer TO CURRENT_USER'
r create_role q 'CREATE ROLE intruder LOGIN'
r admin_login psql "{admin_conn}" -w -X -At -c 'SELECT 1'
r foreign_login env PGPASSWORD=guess psql "{foreign_conn}" -w -X -At -c 'SELECT 1'
r data_dir cat {data}/PG_VERSION
PGAPPNAME=held psql "$PGCONN" -X -At -c 'SELECT pg_sleep(30)' > {ws}/held.out 2>&1; echo "held_rc:$?"
r reconnect q 'SELECT 1'
echo done
}} > {ws}/results &
sleep 0.2; echo started > {ws}/marker; wait"#,
        data = cluster.data.display(),
    );
    let mut child = spawn_workload(
        &identity,
        &helper,
        &dirs,
        &install,
        &session.conninfo(),
        &script,
    );
    wait_for(&dirs.workspace_root.join("marker"), "started");

    // Attribution: the live backend belongs to this workload's native role.
    let start = Instant::now();
    let backends = loop {
        let rows = broker.active_backends(&session.role).unwrap();
        if rows.iter().any(|(_, app)| app == "held") {
            break rows;
        }
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "held query never appeared"
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    assert!(backends.iter().all(|(_, app)| app == "held"));

    // Revocation reaches the established session, not only new logins.
    let revoked = broker
        .revoke(&session, &ledger, "run-pg", "operator revocation")
        .unwrap();
    assert!(revoked["terminated_backends"].as_u64().unwrap() >= 1);
    let text = wait_for(&dirs.workspace_root.join("results"), "done");
    child.wait().unwrap();
    let rows = verdicts(&text);
    assert_eq!(rows["read"], "3", "{text}");
    for name in [
        "delete",
        "drop",
        "truncate",
        "superuser",
        "set_role",
        "self_grant",
        "create_role",
    ] {
        assert_eq!(rows[name], "42501", "{name}: {text}");
    }
    for name in ["admin_login", "foreign_login", "data_dir", "reconnect"] {
        assert_eq!(rows[name], "refused", "{name}: {text}");
    }
    assert_ne!(rows["held_rc"], "0", "{text}");
    let held = fs::read_to_string(dirs.workspace_root.join("held.out")).unwrap();
    assert!(
        held.contains("terminating connection due to administrator command"),
        "{held}"
    );
    assert_eq!(digest(&broker), before, "protected rows changed");
    assert!(broker.active_backends(&session.role).unwrap().is_empty());

    // Evidence carries attribution but never the credential.
    let entries = ledger.read_entries().unwrap();
    let kinds: Vec<&str> = entries.iter().map(|e| e.record_kind.as_str()).collect();
    for kind in [
        "database_session_refused",
        "database_session_issued",
        "database_session_revoked",
    ] {
        assert!(kinds.contains(&kind), "missing {kind}: {kinds:?}");
    }
    let raw = fs::read_to_string(ledger.entries_path()).unwrap();
    assert!(!raw.contains(session.conninfo().rsplit('=').next().unwrap()));
    assert!(!format!("{session:?}").contains("password"));
    broker
        .revoke(&foreign, &ledger, "run-pg", "cleanup")
        .unwrap();
}

#[test]
fn expired_credential_cannot_open_new_sessions() {
    let Some((install, pool, helper, _state)) = prerequisites() else {
        return;
    };
    let mut cluster = start_cluster(&install);
    cluster.config.session_ttl_secs = 2;
    let broker = PostgresBroker::new(cluster.config.clone()).unwrap();
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "audit-agent".into(),
        role: ActorRole::Operator,
    };
    let ledger = LedgerStream::open(&cluster.base, "run-pg-ttl", "control-plane").unwrap();
    let identity = pool.acquire().unwrap();
    let dirs = workload_dirs(&cluster.base, "agent", &identity);
    let session = broker
        .issue(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-pg-ttl",
            &dirs.workspace_root,
            "audit-read",
            identity.host_uid,
        )
        .unwrap()
        .unwrap();
    let ws = dirs.workspace_root.display();
    let script = format!(
        r#"{{ psql "$PGCONN" -X -At -c 'SELECT 1' >/dev/null 2>&1 && echo "fresh:ok" || echo "fresh:refused"
sleep 4
psql "$PGCONN" -X -At -c 'SELECT 1' >/dev/null 2>&1 && echo "expired:ok" || echo "expired:refused"
echo done; }} > {ws}/results"#
    );
    let mut child = spawn_workload(
        &identity,
        &helper,
        &dirs,
        &install,
        &session.conninfo(),
        &script,
    );
    let text = wait_for(&dirs.workspace_root.join("results"), "done");
    child.wait().unwrap();
    let rows = verdicts(&text);
    assert_eq!(rows["fresh"], "ok", "{text}");
    assert_eq!(rows["expired"], "refused", "{text}");
    broker
        .revoke(&session, &ledger, "run-pg-ttl", "cleanup")
        .unwrap();
}
