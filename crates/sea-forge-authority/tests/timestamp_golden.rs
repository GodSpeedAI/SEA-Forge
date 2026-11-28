#![cfg(unix)]
//! Golden pins for every timestamp this crate serializes or parses, taken on chrono before the move to
//! the `time` crate. They go through public functions only, and the single place that names a chrono type
//! is `service_ttl` below, so a type swap re-points one line and no assertion changes.
//!
//! What is pinned:
//! * the exact normalized bytes of an obligation window (`+00:00`, chrono's automatic 0/3/6/9-digit
//!   fraction) for fixed instants with 0, 1 ms, 1 us, 1 ns and 999999999 ns fractions, and for the other
//!   fractions and offsets chrono accepts;
//! * which RFC 3339 spellings parse and which are refused (window, discrimination and verification runs);
//! * the shape of every `now`-derived stamp (`created_at`, `committed_at`, `expires_at`, `claimed_at`),
//!   including exact arithmetic between stamps taken from one instant.

use sea_forge_authority::cep::{
    argv_digest, CepAuthorityError, CepAuthorityService, ObligationSource, SettlementCriteria,
    SourceRow,
};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const LEDGER: &str = "rt-ledger-1";
const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: a\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n  - name: e\n    verdict: allow\n    actor_role: service\n    operation_kind: evidence_mutation\n  - name: s\n    verdict: allow\n    actor_role: service\n    operation_kind: settlement_declaration\n  - name: c\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_capability\n    requires_approval: true\n  - name: ap\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

/// The one place a chrono type is named: how long an approval stays open.
fn service_ttl(secs: i64) -> chrono::Duration {
    chrono::Duration::seconds(secs)
}

// --- a tiny civil-time toolkit on std, so the tests need no time crate at all -----------------------

/// Days from 1970-01-01 for a proleptic Gregorian date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Nanoseconds since the epoch of a `YYYY-MM-DDTHH:MM:SS[.fraction](Z|+HH:MM|-HH:MM)` stamp, strictly the
/// shapes SEA itself writes (no lowercase, no space). `None` when the text is not one of them.
fn epoch_ns(stamp: &str) -> Option<i128> {
    let b = stamp.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let n = |r: std::ops::Range<usize>| stamp.get(r)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, s) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let mut idx = 19;
    let mut frac_ns: i128 = 0;
    if b[idx] == b'.' {
        let start = idx + 1;
        idx = start;
        while idx < b.len() && b[idx].is_ascii_digit() {
            idx += 1;
        }
        let digits = &stamp[start..idx];
        if digits.is_empty() || digits.len() > 9 {
            return None;
        }
        frac_ns = format!("{digits:0<9}").parse().ok()?;
    }
    let offset_s: i64 = match &stamp[idx..] {
        "Z" => 0,
        tz if tz.len() == 6 && (tz.starts_with('+') || tz.starts_with('-')) && &tz[3..4] == ":" => {
            let sign = if tz.starts_with('-') { -1 } else { 1 };
            sign * (tz[1..3].parse::<i64>().ok()? * 3600 + tz[4..6].parse::<i64>().ok()? * 60)
        }
        _ => return None,
    };
    let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s - offset_s;
    Some(secs as i128 * 1_000_000_000 + frac_ns)
}

/// `YYYY-MM-DDTHH:MM:SS.fffffffff` + the given suffix, for an instant, shifted by `offset_min`.
fn stamp_at(ns: i128, offset_min: i64, fraction_digits: usize, suffix: &str) -> String {
    let local = ns + offset_min as i128 * 60 * 1_000_000_000;
    let secs = local.div_euclid(1_000_000_000) as i64;
    let frac = local.rem_euclid(1_000_000_000);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    let rem = secs.rem_euclid(86_400);
    let mut out = format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    );
    if fraction_digits > 0 {
        out.push_str(&format!(".{:09}", frac)[..=fraction_digits]);
    }
    out.push_str(suffix);
    out
}

fn now_ns() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128
}

/// `^YYYY-MM-DDTHH:MM:SS(.fff|.ffffff|.fffffffff)?+00:00$`: chrono `to_rfc3339()` on a UTC instant.
fn is_auto_plus_zero(stamp: &str) -> bool {
    let Some(rest) = stamp.strip_suffix("+00:00") else {
        return false;
    };
    epoch_ns(&format!("{rest}Z")).is_some()
        && match rest.split_once('.') {
            None => rest.len() == 19,
            Some((_, f)) => matches!(f.len(), 3 | 6 | 9),
        }
}

/// `^YYYY-MM-DDTHH:MM:SSZ$`: chrono `format("%Y-%m-%dT%H:%M:%SZ")`.
fn is_secs_z(stamp: &str) -> bool {
    stamp.len() == 20 && stamp.ends_with('Z') && epoch_ns(stamp).is_some()
}

// --- fixture -----------------------------------------------------------------------------------------

struct Cell {
    root: tempfile::TempDir,
    worlds: WorldRegistry,
    world: String,
}

fn cell() -> Cell {
    let mut worlds = WorldRegistry::new();
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: hex::encode(Sha256::digest(SRC.as_bytes())),
            content: SRC.into(),
        }],
    };
    let world = worlds
        .register_source_set("demo", &set)
        .unwrap()
        .to_string();
    Cell {
        root: tempfile::tempdir().unwrap(),
        worlds,
        world,
    }
}

fn with<T>(c: &Cell, ttl_secs: i64, f: impl FnOnce(&CepAuthorityService<'_>) -> T) -> T {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &c.worlds,
        root: c.root.path(),
        approval_ttl: service_ttl(ttl_secs),
        criteria: SettlementCriteria::default(),
    };
    f(&service)
}

fn svc() -> Actor {
    Actor {
        actor_id: "cognate-service".into(),
        role: ActorRole::Service,
    }
}

struct FakeRt {
    rows: RefCell<Vec<SourceRow>>,
}

impl ObligationSource for FakeRt {
    fn ledger_rows(&self, ledger_id: &str) -> Result<Vec<SourceRow>, String> {
        Ok(self
            .rows
            .borrow()
            .iter()
            .filter(|r| r.ledger_id == ledger_id)
            .cloned()
            .collect())
    }
}

fn argv() -> Vec<Value> {
    vec![
        json!("/pinned/verifier"),
        json!("--intent"),
        json!("11111111-1111-4111-8111-111111111111"),
    ]
}

fn request(world: &str, op: &str, claims: Option<Value>, kind: &str) -> Value {
    let mut ext = json!({"operation_id": op, "correlation_id": "corr-1",
        "operation_kind": kind, "operation_name": "run.start", "resource_id": "agent.echo"});
    if let Some(c) = claims {
        ext["claims"] = c;
    }
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-23T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": world},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": [],
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-23T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": format!("tr-{op}"), "transformation_type": "capability_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": "horizon.write",
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": ext
        }
    })
}

fn obligation_claims(window: &str, grace: i64) -> Value {
    json!({
        "modality": "obligation",
        "expectation_digest": format!("sha256:{}", "e".repeat(64)),
        "threshold_id": "thr-1",
        "threshold_ledger": LEDGER,
        "threshold_position": 3,
        "window_closes_at": window,
        "close_grace_s": grace,
        "verifier_argv_digest": argv_digest(&argv()),
    })
}

fn execution_packet(world: &str, op: &str) -> Value {
    json!({
        "envelope_id": format!("env-evidence_packet-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "evidence_packet",
        "created_at": "2026-10-23T12:00:01Z", "created_by": "cognate-realitytrace-adapter",
        "scope": {"evaluation_context": "godspeed-evidence_packet", "world_ref": world},
        "boundary_record": {"scope": "execution evidence", "included_sections": ["evidence", "questions"],
            "excluded_sections": [], "known_omissions": ["only the execution trace"], "unknowns": [],
            "redactions": [], "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-ev-{op}")], "validation_status": "valid",
        "lineage_refs": [format!("env-execution_trace-{op}")],
        "provenance": [{"provenance_id": format!("prov-ev-{op}"), "source_system_refs": ["realitytrace"],
            "producer_refs": ["cognate-realitytrace-adapter"], "production_method": "evidence_packaging", "created_at": "2026-10-23T12:00:01Z"}],
        "questions": [{"question_id": "q-exec", "question_form": "Did the operation execute?"}],
        "evidence": [{"evidence_id": format!("ev-{op}"), "evidence_type": "execution_trace", "question_ref": "q-exec",
            "target_entity_ref": "concept:runtime_execution", "direction": "supports", "reliability": "medium"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.evidence_packet", "profile_version": "1.0.0"},
            "godspeed.evidence_packet": {"operation_id": op,
                "authority_decision_ref": format!("env-authority_decision-{op}"),
                "execution_trace_ref": format!("env-execution_trace-{op}")}
        }
    })
}

fn threshold_row() -> SourceRow {
    SourceRow {
        ledger_id: LEDGER.into(),
        ordinal: 3,
        record_kind: "threshold.declared".into(),
        payload: json!({"states": [{"state_id": "thr-1", "state_type": "settlement_threshold"}]}),
    }
}

/// A run whose `started_at` / `completed_at` are the literal strings given.
fn obs_row(ordinal: u64, id: &str, started: &str, completed: Option<&str>, exit: i64) -> SourceRow {
    let mut content = json!({
        "verifier_id": "command@1",
        "invocation": {"argv": argv(), "launcher": [], "timeout_s": 30},
        "started_at": started,
        "exit_code": exit,
        "crashed": false,
    });
    if let Some(done) = completed {
        content["completed_at"] = json!(done);
    }
    SourceRow {
        ledger_id: LEDGER.into(),
        ordinal,
        record_kind: "observation.recorded".into(),
        payload: json!({"observations": [{"observation_id": id}], "representations": [{"inline_content": content}]}),
    }
}

/// The discrimination baseline: completed well before any execution evidence.
fn baseline(completed: &str) -> SourceRow {
    obs_row(5, "obs-d", "2019-12-31T23:00:00Z", Some(completed), 1)
}

fn ledger_entries(c: &Cell, id: &str) -> Vec<sea_forge_ledger::LedgerEntry> {
    sea_forge_ledger::LedgerStream::open(c.root.path(), id, "test")
        .unwrap()
        .read_entries()
        .unwrap()
}

fn authority_entries(c: &Cell) -> Vec<sea_forge_ledger::LedgerEntry> {
    ledger_entries(c, sea_forge_authority::cep::LEDGER_ID)
}

/// Allow an obligation with `window` as its literal `window_closes_at` and submit execution evidence.
/// `Err` carries the refusal when the allow decision or evidence is refused.
fn open_obligation(c: &Cell, op: &str, window: &str, grace: i64) -> Result<(), CepAuthorityError> {
    with(c, 3600, |s| {
        s.decide(
            &request(
                &c.world,
                op,
                Some(obligation_claims(window, grace)),
                "action",
            ),
            &svc(),
        )
        .map(|_| ())
    })?;
    with(c, 3600, |s| {
        s.submit_evidence(&execution_packet(&c.world, op), &svc())
            .map(|_| ())
    })
}

fn rt(rows: Vec<SourceRow>) -> FakeRt {
    FakeRt {
        rows: RefCell::new(rows),
    }
}

/// The closure outcome for one literal `window_closes_at`: the normalized stamp stored in the closure
/// record, or the refusal text.
fn closure_window(window: &str) -> Result<String, String> {
    let c = cell();
    open_obligation(&c, "op-w", window, 0).map_err(|e| e.to_string())?;
    let source = rt(vec![threshold_row(), baseline("2019-12-31T23:00:01Z")]);
    with(&c, 3600, |s| {
        s.record_closure("op-w", "obs-d", &["obs-d".to_string()], &svc(), &source)
    })
    .map_err(|e| e.to_string())?;
    let row = authority_entries(&c)
        .into_iter()
        .find(|e| e.record_kind == sea_forge_authority::cep::CLOSURE_KIND)
        .expect("a closure record");
    Ok(row.payload["window_closes_at"]
        .as_str()
        .unwrap()
        .to_string())
}

// --- tests -------------------------------------------------------------------------------------------

#[test]
fn obligation_window_fractions_normalize_to_plus_zero_with_an_automatic_fraction() {
    // Fixed past instants (the window must have closed for a closure to be recorded).
    for (input, expected) in [
        ("2020-01-01T00:00:00Z", "2020-01-01T00:00:00+00:00"),
        ("2020-01-01T00:00:00.001Z", "2020-01-01T00:00:00.001+00:00"),
        (
            "2020-01-01T00:00:00.000001Z",
            "2020-01-01T00:00:00.000001+00:00",
        ),
        (
            "2020-01-01T00:00:00.000000001Z",
            "2020-01-01T00:00:00.000000001+00:00",
        ),
        (
            "2020-01-01T00:00:00.999999999Z",
            "2020-01-01T00:00:00.999999999+00:00",
        ),
        // Any fraction chrono accepts is rendered with the smallest of 0, 3, 6 or 9 digits that is exact.
        ("2020-01-01T00:00:00.0Z", "2020-01-01T00:00:00+00:00"),
        ("2020-01-01T00:00:00.5Z", "2020-01-01T00:00:00.500+00:00"),
        ("2020-01-01T00:00:00.50Z", "2020-01-01T00:00:00.500+00:00"),
        (
            "2020-01-01T00:00:00.1234Z",
            "2020-01-01T00:00:00.123400+00:00",
        ),
        (
            "2020-01-01T00:00:00.1234567Z",
            "2020-01-01T00:00:00.123456700+00:00",
        ),
        ("2020-01-01T00:00:00.000Z", "2020-01-01T00:00:00+00:00"),
        ("2020-01-01T00:00:00.120Z", "2020-01-01T00:00:00.120+00:00"),
        // Spelling of the zone does not change the instant or the bytes.
        ("2020-01-01T00:00:00+00:00", "2020-01-01T00:00:00+00:00"),
        ("2020-01-01T00:00:00-00:00", "2020-01-01T00:00:00+00:00"),
        ("2020-01-01T05:30:00+05:30", "2020-01-01T00:00:00+00:00"),
        ("2019-12-31T16:00:00-08:00", "2020-01-01T00:00:00+00:00"),
        (
            "2020-01-01T05:30:00.000000001+05:30",
            "2020-01-01T00:00:00.000000001+00:00",
        ),
        // Leap day and a year boundary.
        (
            "2020-02-29T23:59:59.999999999Z",
            "2020-02-29T23:59:59.999999999+00:00",
        ),
        ("2020-12-31T23:59:59Z", "2020-12-31T23:59:59+00:00"),
    ] {
        assert_eq!(closure_window(input), Ok(expected.to_string()), "{input}");
    }
}

#[test]
fn obligation_window_parse_acceptance_is_pinned() {
    // Everything chrono's `parse_from_rfc3339` refuses today stays refused; everything it accepts parses
    // to the same instant. (`Ok` carries the normalized bytes, `Err` only that it was refused.)
    for (input, expected) in [
        ("2020-01-01T00:00:00z", Some("2020-01-01T00:00:00+00:00")),
        ("2020-01-01t00:00:00Z", Some("2020-01-01T00:00:00+00:00")),
        ("2020-01-01 00:00:00Z", Some("2020-01-01T00:00:00+00:00")),
        // A leap second is accepted and rendered as second 60, not folded into the previous second.
        ("2016-12-31T23:59:60Z", Some("2016-12-31T23:59:60+00:00")),
        (
            "2020-01-01T00:00:00.1234567890Z",
            Some("2020-01-01T00:00:00.123456789+00:00"),
        ),
        ("2020-01-01T00:00:00", None),
        ("2020-01-01T00:00Z", None),
        ("2020-01-01Z", None),
        ("2020-01-01", None),
        ("2020-01-01T00:00:00+0000", None),
        ("2020-01-01T00:00:00+00", None),
        ("2020-13-01T00:00:00Z", None),
        ("2020-02-30T00:00:00Z", None),
        ("2020-01-01T24:00:00Z", None),
        ("2020-01-01T00:60:00Z", None),
        (" 2020-01-01T00:00:00Z", None),
        ("2020-01-01T00:00:00Z ", None),
        ("2020-01-01T00:00:00.Z", None),
        ("", None),
        ("yesterday", None),
    ] {
        let got = closure_window(input);
        match expected {
            Some(bytes) => assert_eq!(got, Ok(bytes.to_string()), "{input:?}"),
            None => assert!(got.is_err(), "{input:?} must stay refused, got {got:?}"),
        }
    }
}

#[test]
fn discrimination_completion_parse_acceptance_is_pinned() {
    // `completed_at` of the baseline run is parsed by the same RFC 3339 reader; a refused spelling means
    // "no recorded completion", which refuses the closure.
    for (completed, accepted) in [
        ("2019-12-31T23:00:01Z", true),
        ("2019-12-31T23:00:01+00:00", true),
        ("2019-12-31T23:00:01.999999999Z", true),
        ("2020-01-01T00:30:01+01:00", true),
        ("2019-12-31T23:00:01z", true),
        ("2019-12-31 23:00:01Z", true),
        ("2019-12-31T23:00:01", false),
        ("2019-12-31T23:00:01+0000", false),
        ("23:00:01", false),
        ("", false),
    ] {
        let c = cell();
        open_obligation(&c, "op-d", "2020-01-01T00:00:00Z", 0).unwrap();
        let source = rt(vec![threshold_row(), baseline(completed)]);
        let got = with(&c, 3600, |s| {
            s.record_closure("op-d", "obs-d", &["obs-d".to_string()], &svc(), &source)
        });
        assert_eq!(got.is_ok(), accepted, "{completed:?}: {got:?}");
    }
}

#[test]
fn a_late_closure_settlement_packet_carries_the_pinned_window_floor_and_created_at() {
    let c = cell();
    open_obligation(&c, "op-late", "2020-01-01T00:00:00.123456789Z", 0).unwrap();
    let source = rt(vec![threshold_row(), baseline("2019-12-31T23:00:01Z")]);
    let closure = with(&c, 3600, |s| {
        s.record_closure("op-late", "obs-d", &["obs-d".to_string()], &svc(), &source)
    })
    .unwrap();
    assert!(!closure.on_time, "closed years after the deadline");
    let settled = with(&c, 3600, |s| {
        s.settle_obligation("op-late", &svc(), &source)
    })
    .unwrap();
    let packet = &settled.envelope;
    let basis = &packet["settlements"][0]["threshold_basis"];
    assert_eq!(
        basis["window_closes_at"],
        "2020-01-01T00:00:00.123456789+00:00"
    );
    // The execution floor is the evidence row's own `committed_at`, re-rendered: byte-identical.
    let evidence = authority_entries(&c)
        .into_iter()
        .find(|e| e.record_kind == "evidence_packet")
        .unwrap();
    assert!(
        is_auto_plus_zero(&evidence.committed_at),
        "{}",
        evidence.committed_at
    );
    assert_eq!(basis["execution_floor"], json!(evidence.committed_at));
    // The closure basis names the closure row's committed_at verbatim.
    let closure_row = authority_entries(&c)
        .into_iter()
        .find(|e| e.entry_ulid == basis["closure"]["entry"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        basis["closure"]["committed_at"],
        json!(closure_row.committed_at)
    );
    // The packet's own created_at is whole seconds with a literal Z.
    let created = packet["created_at"].as_str().unwrap();
    assert!(is_secs_z(created), "{created}");
    assert!(is_secs_z(
        packet["provenance"][0]["created_at"].as_str().unwrap()
    ));
}

#[test]
fn verification_run_times_parse_with_the_pinned_acceptance_set() {
    // A real short window: runs must start after the execution floor and complete by the close.
    let c = cell();
    let t0 = now_ns();
    let closes = t0 + 2_500_000_000;
    open_obligation(&c, "op-runs", &stamp_at(closes, 0, 3, "Z"), 30).unwrap();
    let floor = now_ns() + 50_000_000;
    let start = floor + 10_000_000;
    let end = floor + 20_000_000;
    // (started_at, completed_at, expected classification). The same instant in many spellings must be
    // admissible; spellings chrono refuses are Undefined.
    let cases: Vec<(String, String, &str)> = vec![
        (
            stamp_at(start, 0, 9, "Z"),
            stamp_at(end, 0, 9, "Z"),
            "Known0",
        ),
        (
            stamp_at(start, 0, 9, "+00:00"),
            stamp_at(end, 0, 9, "+00:00"),
            "Known0",
        ),
        (
            stamp_at(start, 330, 9, "+05:30"),
            stamp_at(end, 330, 9, "+05:30"),
            "Known0",
        ),
        (
            stamp_at(start, -480, 6, "-08:00"),
            stamp_at(end, -480, 6, "-08:00"),
            "Known0",
        ),
        (
            stamp_at(start, 0, 9, "z"),
            stamp_at(end, 0, 9, "z"),
            "Known0",
        ),
        (
            stamp_at(start, 0, 9, "Z").replace('T', "t"),
            stamp_at(end, 0, 9, "Z"),
            "Known0",
        ),
        (
            stamp_at(start, 0, 9, "Z").replace('T', " "),
            stamp_at(end, 0, 9, "Z"),
            "Known0",
        ),
        (
            stamp_at(start, 0, 9, ""),
            stamp_at(end, 0, 9, "Z"),
            "Undefined",
        ),
        (
            stamp_at(start, 0, 9, "Z"),
            stamp_at(end, 0, 9, ""),
            "Undefined",
        ),
        (
            "not a time".to_string(),
            stamp_at(end, 0, 9, "Z"),
            "Undefined",
        ),
    ];
    let mut rows = vec![
        threshold_row(),
        baseline(&stamp_at(t0 - 4_000_000_000, 0, 9, "Z")),
    ];
    let mut ids = Vec::new();
    for (i, (started, completed, _)) in cases.iter().enumerate() {
        let id = format!("obs-v{i}");
        rows.push(obs_row(7 + i as u64, &id, started, Some(completed), 0));
        ids.push(id);
    }
    let source = rt(rows);
    // Wait for the window, then record the closure and settle.
    let wait = (closes - now_ns()).max(0) as u64 + 80_000_000;
    std::thread::sleep(std::time::Duration::from_nanos(wait));
    let mut closed_ids = vec![];
    closed_ids.extend(ids.iter().cloned());
    with(&c, 3600, |s| {
        s.record_closure("op-runs", "obs-d", &closed_ids, &svc(), &source)
    })
    .unwrap();
    let settled = with(&c, 3600, |s| {
        s.settle_obligation("op-runs", &svc(), &source)
    })
    .unwrap();
    let results = settled.envelope["settlements"][0]["threshold_basis"]["results"].clone();
    let want: Vec<&str> = cases.iter().map(|(_, _, r)| *r).collect();
    assert_eq!(results, json!(want), "{}", settled.envelope);
}

#[test]
fn approval_expiry_is_the_escalation_commit_instant_plus_the_ttl_to_the_nanosecond() {
    for ttl in [1_i64, 60, 3600, 86_400] {
        let c = cell();
        let before = now_ns();
        let decided = with(&c, ttl, |s| {
            s.decide(&request(&c.world, "op-cap", None, "capability"), &svc())
                .unwrap()
        });
        let after = now_ns();
        // The envelope's issued expiry: whole seconds, literal Z, floor of (now + ttl).
        let ext = &decided.envelope["extensions"]["godspeed.authority_decision"];
        let issued = ext["approval_expires_at"]
            .as_str()
            .expect("an issued expiry");
        assert!(is_secs_z(issued), "{issued}");
        let issued_ns = epoch_ns(issued).unwrap();
        let ttl_ns = ttl as i128 * 1_000_000_000;
        assert!(
            issued_ns > before + ttl_ns - 1_000_000_000 && issued_ns <= after + ttl_ns,
            "{issued} vs now+{ttl}s"
        );
        // The standing: the escalation row's committed_at plus the ttl, re-rendered as `+00:00`.
        let approvals = with(&c, ttl, |s| s.list_approvals()).unwrap();
        assert_eq!(approvals.len(), 1);
        let expires = approvals[0].to_json()["expires_at"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(is_auto_plus_zero(&expires), "{expires}");
        let escalation = authority_entries(&c)
            .into_iter()
            .find(|e| e.idempotency_key.as_deref() == Some("cep:op-cap"))
            .unwrap();
        assert!(
            is_auto_plus_zero(&escalation.committed_at),
            "{}",
            escalation.committed_at
        );
        let committed_ns = epoch_ns(&escalation.committed_at.replace("+00:00", "Z")).unwrap();
        let expires_ns = epoch_ns(&expires.replace("+00:00", "Z")).unwrap();
        assert_eq!(
            expires_ns - committed_ns,
            ttl_ns,
            "{} -> {expires}",
            escalation.committed_at
        );
    }
}

#[test]
fn every_now_derived_stamp_in_the_authority_ledger_has_the_pinned_shape() {
    let c = cell();
    let before = now_ns();
    with(&c, 3600, |s| {
        s.decide(
            &request(
                &c.world,
                "op-shape",
                Some(obligation_claims("2099-01-01T00:00:00Z", 0)),
                "action",
            ),
            &svc(),
        )
        .unwrap();
        s.decide(&request(&c.world, "op-cap2", None, "capability"), &svc())
            .unwrap();
        s.submit_evidence(&execution_packet(&c.world, "op-shape"), &svc())
            .unwrap();
    });
    let after = now_ns();
    let entries = authority_entries(&c);
    assert!(entries.len() >= 3);
    let mut previous = 0_i128;
    for e in &entries {
        assert!(is_auto_plus_zero(&e.committed_at), "{}", e.committed_at);
        let ns = epoch_ns(&e.committed_at.replace("+00:00", "Z")).unwrap();
        assert!(
            ns >= before - 1_000_000_000 && ns <= after + 1_000_000_000,
            "{}",
            e.committed_at
        );
        assert!(ns >= previous, "committed_at is monotone within one stream");
        previous = ns;
    }
    // The decision envelope's own `created_at` is whole seconds with a Z.
    let decision = with(&c, 3600, |s| {
        s.decide(&request(&c.world, "op-env", None, "action"), &svc())
            .unwrap()
    });
    let created = decision.envelope["created_at"].as_str().unwrap();
    assert!(is_secs_z(created), "{created}");
}

#[test]
fn engine_decision_decided_at_has_the_pinned_shape() {
    use sea_forge_authority::AuthorityEvaluation;
    use sea_forge_core::types::AuthorityAction;
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = svc();
    let action = AuthorityAction::Reserved {
        resource_type: "settlement_declaration".into(),
        resource_id: "op-x".into(),
        parameters: json!({"operation_id": "op-x"}),
    };
    let root = tempfile::tempdir().unwrap();
    let before = now_ns();
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
            run_id: "op-x",
            case_id: "cognate",
            plan_item_id: "op-x",
            sequence: 1,
            action: &action,
            workspace_root: root.path(),
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            opa_candidate: None,
            environment: None,
        })
        .unwrap();
    let after = now_ns();
    assert!(
        is_auto_plus_zero(&decision.decided_at),
        "{}",
        decision.decided_at
    );
    let ns = epoch_ns(&decision.decided_at.replace("+00:00", "Z")).unwrap();
    assert!(ns >= before - 1_000_000_000 && ns <= after + 1_000_000_000);
    // It is serialized as that same text.
    let json = serde_json::to_value(&decision).unwrap();
    assert_eq!(json["decided_at"], json!(decision.decided_at));
}
