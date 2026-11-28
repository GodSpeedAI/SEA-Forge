#![cfg(target_os = "linux")]
//! Golden pin for the PostgreSQL session expiry stamp (chrono `to_rfc3339_opts(Secs, true)`): whole seconds
//! with a literal `Z`, written into `CREATE ROLE ... VALID UNTIL` and `ALTER ROLE ... VALID UNTIL`, kept on
//! the session as `expires_at`, and accepted by PostgreSQL. Runs against a real cluster.
//!
//! Needs `SEA_FORGE_SECURITY_PG_INSTALL` (e.g. `~/.pg0/installation/18.1.0`). When
//! `SEA_FORGE_REQUIRE_POSTGRES` is set the test FAILS instead of skipping if it is missing.
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_ledger::LedgerStream;
use sea_forge_server::postgres::PostgresBroker;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "support/pg_cluster.rs"]
mod pg_cluster;
use pg_cluster::start_cluster;

/// The PostgreSQL install, or a skip that becomes a FAILURE when `SEA_FORGE_REQUIRE_POSTGRES` is set. This
/// test only mints and renews roles, so it needs no workload identity or isolation (unlike the broker suite).
fn install() -> Option<std::path::PathBuf> {
    match std::env::var_os("SEA_FORGE_SECURITY_PG_INSTALL") {
        Some(path) => Some(path.into()),
        None => {
            assert!(
                std::env::var_os("SEA_FORGE_REQUIRE_POSTGRES").is_none(),
                "PostgreSQL qualification required but SEA_FORGE_SECURITY_PG_INSTALL is unset"
            );
            eprintln!("NOT-EXECUTABLE: SEA_FORGE_SECURITY_PG_INSTALL unset");
            None
        }
    }
}

/// Any uid: the role name embeds it, nothing is launched as it.
const WORKLOAD_UID: u32 = 4242;

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: audit-reader-session\n    verdict: allow\n    actor_role: operator\n    operation_kind: database_session\n    boundary_constraints:\n      db_resource: [app-db]\n      db_scope: [audit-read]\n";

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

/// Epoch seconds of exactly `YYYY-MM-DDTHH:MM:SSZ`; `None` for any other shape.
fn secs_z(stamp: &str) -> Option<i64> {
    let b = stamp.as_bytes();
    if stamp.len() != 20
        || b[19] != b'Z'
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let n = |r: std::ops::Range<usize>| stamp.get(r)?.parse::<i64>().ok();
    Some(
        days_from_civil(n(0..4)?, n(5..7)?, n(8..10)?) * 86_400
            + n(11..13)? * 3600
            + n(14..16)? * 60
            + n(17..19)?,
    )
}

#[test]
fn session_expiry_is_whole_seconds_with_a_literal_z_and_postgres_accepts_it() {
    let Some(install) = install() else {
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
    let ledger = LedgerStream::open(&cluster.base, "run-ts", "control-plane").unwrap();
    let workspace = cluster.base.join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    };
    let ttl = cluster.config.session_ttl_secs as i64;

    let before = now();
    let mut session = broker
        .issue(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-ts",
            &workspace,
            "audit-read",
            WORKLOAD_UID,
        )
        .unwrap()
        .expect("authorized session");
    let after = now();
    let issued = secs_z(&session.expires_at)
        .unwrap_or_else(|| panic!("not whole seconds with Z: {}", session.expires_at));
    assert!(
        issued >= before + ttl - 1 && issued <= after + ttl + 1,
        "{} vs ttl {ttl}",
        session.expires_at
    );

    // PostgreSQL parsed the very same text into the role's expiry.
    let valid_until = |role: &str| {
        broker
            .admin_sql(&format!(
                "SELECT to_char(rolvaliduntil AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') FROM pg_roles WHERE rolname = '{role}'"
            ))
            .unwrap()
    };
    assert_eq!(valid_until(&session.role), session.expires_at);

    // Renewal writes the next stamp in the same shape and PostgreSQL moves VALID UNTIL to it.
    std::thread::sleep(std::time::Duration::from_millis(1100));
    broker
        .renew(
            &engine,
            &bundle,
            &actor,
            &ledger,
            "run-ts",
            &workspace,
            &mut session,
            WORKLOAD_UID,
        )
        .unwrap();
    let renewed = secs_z(&session.expires_at).unwrap_or_else(|| panic!("{}", session.expires_at));
    assert!(renewed > issued, "{renewed} !> {issued}");
    assert_eq!(valid_until(&session.role), session.expires_at);
}
