#![cfg(unix)]
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
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: audit-reader-session\n    verdict: allow\n    actor_role: operator\n    operation_kind: database_session\n    boundary_constraints:\n      db_resource: [app-db]\n      db_scope: [audit-read]\n";

fn workload_dirs(base: &Path, name: &str, identity: &WorkloadIdentity) -> WorkloadConfinement {
    let root = base.join(name);
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
    // The client installation is the only runtime tree it may read.
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
    let (mut command, lifeline) = isolated_command(
        identity,
        helper,
        &dirs,
        &["/bin/sh".into(), "-c".into(), script.into()],
    )
    .unwrap();
    // Session lifetime, not workload lifetime, is under test here.
    std::mem::forget(lifeline);
    command
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

const ELEVATION_POLICY: &str = "version: \"0.1\"\nidentity_bindings:\n  - principal: audit-agent\n    actor_type: human\n    role: operator\n  - principal: security_officer\n    actor_type: human\n    role: R-SO\n  - principal: developer\n    actor_type: human\n    role: R-DEV\nsod_rules:\n  - name: elevation-needs-security-officer\n    requester_role: operator\n    approver_role: R-SO\n    operation_kind: database_session\nrules:\n  - name: audit-writer-elevation\n    verdict: allow\n    actor_role: operator\n    operation_kind: database_session\n    requires_approval: true\n    boundary_constraints:\n      db_resource: [app-db]\n      db_scope: [audit-write]\n";

fn criteria_record() -> sea_forge_core::types::SettlementCriteriaRecord {
    use sea_forge_core::types::*;
    use sea_forge_ledger::types::hash_canonical;
    let criteria = SettlementCriteria {
        require_exit_zero: true,
        ..Default::default()
    };
    let mut record = SettlementCriteriaRecord {
        version: "0.2".into(),
        criteria_id: "crit_jit_elevation".into(),
        criteria_sha256: hash_canonical(&criteria).unwrap(),
        criteria,
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::ImplementationDefined,
            reference: "jit-elevation-test".into(),
            sha256: "sha256:origin".into(),
            role: OriginRole::AcceptanceSource,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        derivation: CriteriaDerivation {
            method: DerivationMethod::ImplementationDefined,
            actor_ref: "audit-agent".into(),
            producer_ref: None,
            rationale: "JIT elevation fixture".into(),
        },
        declared_at: "2026-10-17T00:00:00Z".into(),
        criteria_record_hash: String::new(),
    };
    record.criteria_record_hash = hash_canonical(&record).unwrap();
    record
}

fn approval(
    decision: &sea_forge_core::types::AuthorityDecision,
    criteria: &sea_forge_core::types::SettlementCriteriaRecord,
    id: &str,
    resolver: &str,
    window: chrono::Duration,
) -> sea_forge_core::types::ApprovalRequest {
    let now = chrono::Utc::now();
    sea_forge_core::types::ApprovalRequest {
        version: "0.2".into(),
        approval_id: id.into(),
        run_id: decision.run_id.clone(),
        case_id: decision.audit_record.case_id.clone().unwrap(),
        decision_id: decision.decision_id.clone(),
        plan_item_id: decision.plan_item_id.clone(),
        criteria_ref: Some(criteria.criteria_id.clone()),
        criteria_sha256: Some(criteria.criteria_sha256.clone()),
        criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
        job_contract_ref: None,
        requested_at: now.to_rfc3339(),
        expires_at: (now + window).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        status: sea_forge_core::types::ApprovalStatus::Approved,
        resolved_by: Some(resolver.into()),
        resolved_at: Some(now.to_rfc3339()),
        note: None,
    }
}

fn sfw_roles(broker: &PostgresBroker) -> String {
    broker
        .admin_sql("SELECT count(*) FROM pg_roles WHERE rolname LIKE 'sfw_%'")
        .unwrap()
}

#[test]
fn jit_elevation_needs_independent_one_use_approval_and_ends_at_the_resource() {
    let Some((install, pool, helper, _state)) = prerequisites() else {
        return;
    };
    let mut cluster = start_cluster(&install);
    cluster.config.session_ttl_secs = 600;
    let broker = PostgresBroker::new(cluster.config.clone()).unwrap();
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(ELEVATION_POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let requester = Actor {
        actor_id: "audit-agent".into(),
        role: ActorRole::Operator,
    };
    let identity = pool.acquire().unwrap();
    let dirs = workload_dirs(&cluster.base, "agent", &identity);
    let ledger = LedgerStream::open(&cluster.base, "case-jit", "control-plane").unwrap();
    let action = broker.session_action("audit-write", identity.host_uid);
    let decision = engine
        .evaluate(sea_forge_authority::AuthorityEvaluation {
            actor: &requester,
            binding: bundle.resolve_identity("audit-agent", ActorRole::Operator),
            run_id: "run-jit",
            case_id: "case-jit",
            plan_item_id: "database-session",
            sequence: 1,
            action: &action,
            workspace_root: &dirs.workspace_root,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            opa_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, sea_forge_core::types::Verdict::Escalate);
    // Without approval the plain issuance path grants nothing.
    assert!(broker
        .issue(
            &engine,
            &bundle,
            &requester,
            &ledger,
            "run-jit",
            &dirs.workspace_root,
            "audit-write",
            identity.host_uid
        )
        .unwrap()
        .is_none());
    let decision_ref = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let criteria = criteria_record();
    let criteria_ref = ledger
        .commit_typed("settlement_criteria", vec![], &criteria, vec![])
        .unwrap();
    let issue = |approval: &sea_forge_core::types::ApprovalRequest, scope: &str, uid: u32| {
        let approval_ref = ledger
            .commit_typed("approval_resolution", vec![], approval, vec![])
            .unwrap();
        broker.issue_with_approval(
            &engine,
            &decision,
            &decision_ref,
            approval,
            &approval_ref,
            &criteria,
            &criteria_ref,
            &ledger,
            &dirs.workspace_root,
            scope,
            uid,
        )
    };
    let window = chrono::Duration::seconds(4);
    let refused = |result: Result<_, Box<dyn std::error::Error>>| match result {
        Ok(_) => panic!("elevation unexpectedly issued"),
        Err(error) => error.to_string(),
    };
    // Self-approval and an approver outside the required role are refused.
    assert!(refused(issue(
        &approval(&decision, &criteria, "apr-self", "audit-agent", window),
        "audit-write",
        identity.host_uid,
    ))
    .contains("does not match the escalated decision"));
    assert!(refused(issue(
        &approval(&decision, &criteria, "apr-dev", "developer", window),
        "audit-write",
        identity.host_uid,
    ))
    .contains("SOD approver roles"));
    let good = approval(&decision, &criteria, "apr-so", "security_officer", window);
    // The approval cannot be redirected to another scope or workload.
    assert!(refused(issue(&good, "audit-read", identity.host_uid))
        .contains("does not grant this action"));
    assert!(refused(issue(&good, "audit-write", identity.host_uid + 1))
        .contains("does not grant this action"));
    assert_eq!(
        sfw_roles(&broker),
        "0",
        "a refused elevation created a role"
    );
    let session = issue(&good, "audit-write", identity.host_uid).unwrap();
    // One use: the consumed approval cannot mint a second session.
    assert!(refused(issue(&good, "audit-write", identity.host_uid)).contains("already granted"));
    assert_eq!(sfw_roles(&broker), "1");
    let reaper = broker.expiry_reaper(
        &session,
        cluster.base.clone(),
        "case-jit".into(),
        "run-jit".into(),
    );

    let ws = dirs.workspace_root.display();
    let script = format!(
        r#"{{ psql "$PGCONN" -w -X -At -c "INSERT INTO audit_log VALUES (4,'approved-jit')" >/dev/null 2>&1 && echo "in_window:ok" || echo "in_window:refused"
PGAPPNAME=held psql "$PGCONN" -w -X -At -c 'SELECT pg_sleep(30)' > {ws}/held.out 2>&1; echo "held_rc:$?"
psql "$PGCONN" -w -X -At -c "INSERT INTO audit_log VALUES (5,'after-expiry')" >/dev/null 2>&1 && echo "after_expiry:ok" || echo "after_expiry:refused"
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
    let start = Instant::now();
    while !broker
        .active_backends(&session.role)
        .unwrap()
        .iter()
        .any(|(_, a)| a == "held")
    {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "held query never started"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    reaper.join();
    let text = wait_for(&dirs.workspace_root.join("results"), "done");
    child.wait().unwrap();
    let rows = verdicts(&text);
    assert_eq!(rows["in_window"], "ok", "{text}");
    assert_ne!(
        rows["held_rc"], "0",
        "live elevated session survived expiry: {text}"
    );
    assert_eq!(rows["after_expiry"], "refused", "{text}");
    assert_eq!(
        broker
            .admin_sql("SELECT string_agg(id::text, ',' ORDER BY id) FROM audit_log")
            .unwrap(),
        "1,2,3,4"
    );
    assert_eq!(sfw_roles(&broker), "0");
    let kinds: Vec<String> = ledger
        .read_entries()
        .unwrap()
        .into_iter()
        .map(|e| e.record_kind)
        .collect();
    assert_eq!(
        kinds
            .iter()
            .filter(|k| *k == "database_elevation_issued")
            .count(),
        1
    );
    assert!(kinds.iter().any(|k| k == "database_session_revoked"));
}

#[test]
fn renewal_redecides_and_a_refused_renewal_ends_the_live_session() {
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
    let ledger = LedgerStream::open(&cluster.base, "run-renew", "control-plane").unwrap();
    let identity = pool.acquire().unwrap();
    let dirs = workload_dirs(&cluster.base, "agent", &identity);
    let mut session = broker
        .issue(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-renew",
            &dirs.workspace_root,
            "audit-read",
            identity.host_uid,
        )
        .unwrap()
        .unwrap();
    let valid_until = |role: &str| {
        broker
            .admin_sql(&format!(
                "SELECT rolvaliduntil FROM pg_roles WHERE rolname = '{role}'"
            ))
            .unwrap()
    };
    let before = valid_until(&session.role);
    std::thread::sleep(Duration::from_millis(1100));
    broker
        .renew(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-renew",
            &dirs.workspace_root,
            &mut session,
            identity.host_uid,
        )
        .unwrap();
    assert_ne!(
        valid_until(&session.role),
        before,
        "renewal did not move VALID UNTIL"
    );

    let ws = dirs.workspace_root.display();
    let script = format!(
        r#"{{ PGAPPNAME=held psql "$PGCONN" -w -X -At -c 'SELECT pg_sleep(30)' > {ws}/held.out 2>&1; echo "held_rc:$?"; echo done; }} > {ws}/results"#
    );
    let mut child = spawn_workload(
        &identity,
        &helper,
        &dirs,
        &install,
        &session.conninfo(),
        &script,
    );
    let start = Instant::now();
    while broker.active_backends(&session.role).unwrap().is_empty() {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "held query never started"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    // Policy withdraws the scope: renewal is refused and revokes live access.
    let withdrawn: AuthorityPolicyBundle =
        sea_forge_core::yaml::from_str("version: \"0.1\"\nrules: []\n").unwrap();
    let withdrawn_engine = PolicyAuthorityEngine::new(withdrawn.clone()).unwrap();
    assert!(broker
        .renew(
            &withdrawn_engine,
            &withdrawn,
            &actor,
            &ledger,
            "run-renew",
            &dirs.workspace_root,
            &mut session,
            identity.host_uid
        )
        .is_err());
    let text = wait_for(&dirs.workspace_root.join("results"), "done");
    child.wait().unwrap();
    assert_ne!(verdicts(&text)["held_rc"], "0", "{text}");
    assert_eq!(sfw_roles(&broker), "0");
    // A revoked session cannot be renewed back into existence.
    assert!(broker
        .renew(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-renew",
            &dirs.workspace_root,
            &mut session,
            identity.host_uid
        )
        .is_err());
    let kinds: Vec<String> = ledger
        .read_entries()
        .unwrap()
        .into_iter()
        .map(|e| e.record_kind)
        .collect();
    assert!(kinds.iter().any(|k| k == "database_session_renewed"));
    assert!(kinds.iter().any(|k| k == "database_session_revoked"));
}

/// ADV-B07: with the launcher (and so the expiry reaper) gone, `VALID
/// UNTIL` alone left a live backend running past expiry. The next issuance
/// on the cluster now ends every expired, unreaped session first.
#[test]
fn expired_sessions_without_a_reaper_end_at_the_next_issuance() {
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
    let ledger = LedgerStream::open(&cluster.base, "run-pg-orphan", "control-plane").unwrap();
    let identity = pool.acquire().unwrap();
    let dirs = workload_dirs(&cluster.base, "agent", &identity);
    let issue = || {
        broker
            .issue(
                &engine,
                &bundle,
                &actor,
                &ledger,
                "run-pg-orphan",
                &dirs.workspace_root,
                "audit-read",
                identity.host_uid,
            )
            .unwrap()
            .unwrap()
    };
    // No expiry reaper is started: the launcher that would own it is dead.
    let orphaned = issue();
    let mut child = spawn_workload(
        &identity,
        &helper,
        &dirs,
        &install,
        &orphaned.conninfo(),
        "psql \"$PGCONN\" -X -At -c 'SELECT pg_sleep(60)' >/dev/null 2>&1",
    );
    let start = std::time::Instant::now();
    while broker.active_backends(&orphaned.role).unwrap().is_empty() {
        assert!(
            start.elapsed() < Duration::from_secs(15),
            "backend never connected"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_secs(3));
    // Control: past VALID UNTIL, the database alone has not ended it.
    assert!(
        !broker.active_backends(&orphaned.role).unwrap().is_empty(),
        "expired backend already gone without reconciliation"
    );
    // Positive control: the next issuance works, and reconciles first.
    let next = issue();
    assert!(broker.active_backends(&orphaned.role).unwrap().is_empty());
    assert_eq!(
        broker
            .admin_sql(&format!(
                "SELECT count(*) FROM pg_roles WHERE rolname = '{}'",
                orphaned.role
            ))
            .unwrap(),
        "0"
    );
    let reasons: Vec<String> = ledger
        .read_entries()
        .unwrap()
        .into_iter()
        .filter(|e| e.record_kind == "database_session_revoked")
        .map(|e| e.payload["reason"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(reasons, ["expired_unreaped"]);
    child.wait().unwrap();
    broker
        .revoke(&next, &ledger, "run-pg-orphan", "cleanup")
        .unwrap();
}
