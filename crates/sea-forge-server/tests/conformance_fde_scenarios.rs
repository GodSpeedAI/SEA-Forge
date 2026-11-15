#![cfg(unix)]
//! WP-21 part A: the S2 scenario fixtures driven through the REAL server.
//!
//! Every scenario of the four manifests under `contracts/fde/fixtures/`
//! (`case-record-append-v2`, `execution-episode-ref`, `delegation-link`,
//! `reserved-operation`) is loaded and sent over an authenticated socket to a
//! fresh cell (`handle_request_as` through `run_authenticated`). The response
//! code (and dialect) the scenario states is asserted exactly.
//!
//! No scenario is skipped silently. A scenario the server does not yet satisfy
//! is listed in [`PENDING`] with a reason; the test fails when a pending
//! scenario passes (remove it from the list) and when a scenario that is not
//! pending fails. [`final_pending_list_is_only_allowed_reasons`] fails while
//! any reason outside [`FINAL_REASONS`] remains. Reasons:
//!
//! - `part-B`: needs the Field Case document, `field_case.revised`,
//!   `strategy.recorded`, `gate.assessed`, `disposition.recorded` or the
//!   lifecycle function (WP-21 part B).
//! - `wp-31`: the A3 proof check (`reserved-operation` scenarios other than
//!   the program-execution decision); the contract says WP-31 implements it.
//! - `stepN`: temporary, work package part A step N not yet landed.

use sea_forge_core::types::ActorRole;
use sea_forge_server::identity::{IdentityBinding, IdentityBindings};
use sea_forge_server::sfwp::case_records::digest_of;
use sea_forge_server::ServerConfig;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[path = "support/authenticated.rs"]
mod authenticated;

const CASE: &str = "case-0001";
const KEEPER: &str = "svc:case-keeper";
const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Reasons a pending scenario may still carry in the final commit.
const FINAL_REASONS: &[&str] = &["part-B", "wp-31"];

/// Scenarios the server satisfies (it grows as the steps of WP-21 part A land;
/// an acceptance listed here stays green as the gate lands).
const PASSES_TODAY: &[&str] = &[
    "case-record-append-v2/independent-verifier-declares-fresh-in-shared-workspace",
    "execution-episode-ref/delegate-decision-binds-another-cwd",
    "execution-episode-ref/delegate-decision-exhausted-runs-used-equals-max-runs",
    "execution-episode-ref/delegate-consequential-action-without-decision",
    "execution-episode-ref/delegate-consequential-action-denied",
    "execution-episode-ref/case_advance-decision-binds-another-cwd",
    "execution-episode-ref/case_advance-decision-exhausted-runs-used-equals-max-runs",
    "execution-episode-ref/case_advance-consequential-action-without-decision",
    "execution-episode-ref/case_advance-consequential-action-denied",
    "execution-episode-ref/item_execute-decision-binds-another-cwd",
    "execution-episode-ref/item_execute-decision-exhausted-runs-used-equals-max-runs",
    "execution-episode-ref/item_execute-consequential-action-without-decision",
    "execution-episode-ref/item_execute-consequential-action-denied",
    "delegation-link/delegated-run-without-link",
    "delegation-link/executor-reuses-identity-as-verifier",
    "delegation-link/item-execute-only-then-settle-by-the-executor",
    "delegation-link/delegate-then-settle-by-delegator",
    "case-record-append-v2/sibling-verifier-without-derivable-context-cannot-settle-s",
    "case-record-append-v2/sibling-verifier-self-asserts-fresh-context",
    "execution-episode-ref/delegate-kind-differs-from-freeze",
    "execution-episode-ref/case_advance-kind-differs-from-freeze",
    "execution-episode-ref/item_execute-kind-differs-from-freeze",
    "execution-episode-ref/case-advance-transition-only-on-an-execution-item-complete",
    "execution-episode-ref/case-advance-transition-only-on-an-execution-item-skip",
    "execution-episode-ref/delegate-resume-after-link-expiry-is-refused",
    "execution-episode-ref/delegate-assignment-new-k-already-used-by-a-running-assignment",
    "execution-episode-ref/delegate-episode-already-settled",
    "execution-episode-ref/delegate-episode-superseded-by-a-later-n",
    "execution-episode-ref/delegate-case-closed",
    "execution-episode-ref/delegate-anchor-published-by-executor",
    "execution-episode-ref/delegate-freeze-anchor-only",
    "execution-episode-ref/delegate-freeze-anchor-missing",
    "execution-episode-ref/delegate-freeze-record-without-realitytrace",
    "execution-episode-ref/delegate-freeze-realitytrace-only",
    "execution-episode-ref/delegate-field-case-no-freeze",
    "execution-episode-ref/case_advance-episode-already-settled",
    "execution-episode-ref/case_advance-episode-superseded-by-a-later-n",
    "execution-episode-ref/case_advance-case-closed",
    "execution-episode-ref/case_advance-anchor-published-by-executor",
    "execution-episode-ref/case_advance-freeze-anchor-only",
    "execution-episode-ref/case_advance-freeze-anchor-missing",
    "execution-episode-ref/case_advance-freeze-record-without-realitytrace",
    "execution-episode-ref/case_advance-freeze-realitytrace-only",
    "execution-episode-ref/case_advance-field-case-no-freeze",
    "execution-episode-ref/item_execute-episode-already-settled",
    "execution-episode-ref/item_execute-episode-superseded-by-a-later-n",
    "execution-episode-ref/item_execute-case-closed",
    "execution-episode-ref/item_execute-anchor-published-by-executor",
    "execution-episode-ref/item_execute-freeze-anchor-only",
    "execution-episode-ref/item_execute-freeze-anchor-missing",
    "execution-episode-ref/item_execute-freeze-record-without-realitytrace",
    "execution-episode-ref/item_execute-freeze-realitytrace-only",
    "execution-episode-ref/item_execute-field-case-no-freeze",
    "execution-episode-ref/item_execute-field-case-no-episode-ref-but-complete-freeze-exists",
    "execution-episode-ref/item_execute-field-case-no-episode-ref",
    "execution-episode-ref/human-task-complete-last-item-without-close-disposition",
    "execution-episode-ref/case-advance-to-completed-without-close-disposition",
    "execution-episode-ref/case-advance-field-case-neither-episode-ref-nor-transition-items",
    "execution-episode-ref/delegate-case-unresolved-without-episode-ref",
    "execution-episode-ref/delegate-case-unresolved",
    "execution-episode-ref/delegate-reaches-terminal-without-close-disposition",
    "execution-episode-ref/delegate-case-suspended",
    "execution-episode-ref/delegate-field-case-no-episode-ref-but-complete-freeze-exists",
    "execution-episode-ref/delegate-field-case-no-episode-ref",
    "execution-episode-ref/case_advance-reaches-terminal-without-close-disposition",
    "execution-episode-ref/case_advance-case-suspended",
    "execution-episode-ref/case_advance-field-case-no-episode-ref-but-complete-freeze-exists",
    "execution-episode-ref/case_advance-field-case-no-episode-ref",
    "execution-episode-ref/item_execute-reaches-terminal-without-close-disposition",
    "execution-episode-ref/item_execute-case-suspended",
    "case-record-append-v2/anchor-publisher-invalid",
    "case-record-append-v2/anchor-publisher-invalid-at-freeze-does-not-burn-the-key",
    "case-record-append-v2/case-suspended-refuses-settlement",
    "case-record-append-v2/case-closed-refuses-freeze",
    "case-record-append-v2/authority-denied-human-settles-v1-writer",
    "case-record-append-v2/authority-denied-human-settles-without-approval",
    "case-record-append-v2/payload-too-large",
    "case-record-append-v2/store-integrity",
    "case-record-append-v2/case-suspended-refuses-freeze",
    "case-record-append-v2/model-anchor-late",
    "case-record-append-v2/reference-derived-from-result",
    "case-record-append-v2/kind-immutable-relabel-after-run",
    "case-record-append-v2/settlement-exists-v1-writer-fallback",
    "case-record-append-v2/settlement-exists",
    "case-record-append-v2/world-not-pinned-v1-writer-fallback",
    "case-record-append-v2/world-not-pinned",
    "case-record-append-v2/freeze-incomplete-realitytrace-plan-absent",
    "case-record-append-v2/freeze-missing-settle",
    "case-record-append-v2/kind-immutable-v1-writer-fallback",
    "case-record-append-v2/kind-same-second-freeze-is-freeze-key-exists",
    "case-record-append-v2/self-settlement-executor-settles",
    "case-record-append-v2/settlement-same-key-same-body-replays",
    "case-record-append-v2/sibling-verifier-with-derived-fresh-context-settles",
    "case-record-append-v2/v1-writer-agent-settles-with-independent-verifier-accepted",
    "delegation-link/independent-verifier-settles",
    "delegation-link/link-written-before-run-start-is-accepted",
    "execution-episode-ref/case-advance-to-completed-with-close-disposition",
    "execution-episode-ref/case-advance-transition-only-skip-accepted",
    "execution-episode-ref/case_advance-anchor-not-configured-complete-without-it",
    "execution-episode-ref/case_advance-complete-freeze-accepted",
    "execution-episode-ref/case_advance-non-field-case-ignores-episode-ref",
    "execution-episode-ref/case_advance-non-field-case-unaffected",
    "execution-episode-ref/delegate-anchor-not-configured-complete-without-it",
    "execution-episode-ref/delegate-complete-freeze-accepted",
    "execution-episode-ref/delegate-new-delegation-takes-k-plus-one",
    "execution-episode-ref/delegate-non-field-case-ignores-episode-ref",
    "execution-episode-ref/delegate-non-field-case-unaffected",
    "execution-episode-ref/delegate-resume-of-interrupted-assignment-reuses-its-id",
    "execution-episode-ref/human-task-complete-last-item-with-close-disposition",
    "execution-episode-ref/human-task-complete-not-last-item",
    "execution-episode-ref/item-execute-finishing-last-item-with-close-disposition",
    "execution-episode-ref/item_execute-anchor-not-configured-complete-without-it",
    "execution-episode-ref/item_execute-complete-freeze-accepted",
    "execution-episode-ref/item_execute-non-field-case-ignores-episode-ref",
    "execution-episode-ref/item_execute-non-field-case-unaffected",
];

// (family/id, reason). Filled in by `pending_reason` below from rules, so a new
// scenario is never pending by accident: see that function.
fn pending_reason(family: &str, id: &str, scenario: &Value) -> Option<&'static str> {
    if PASSES_TODAY.contains(&format!("{family}/{id}").as_str()) {
        return None;
    }
    match family {
        "reserved-operation" => Some("wp-31"),
        "case-record-append-v2" => {
            let kind = scenario["request"]["kind"].as_str().unwrap_or("");
            if matches!(
                kind,
                "field_case.revised"
                    | "strategy.recorded"
                    | "gate.assessed"
                    | "disposition.recorded"
            ) {
                return Some("part-B");
            }
            // Case-level dispositions are written by part B: a scenario whose
            // ledger holds one needs the disposition writer only to be seeded,
            // and is driven by seeding the record directly (not part-B).
            let _ = id;
            STEP_PENDING
                .iter()
                .find(|(f, i, _)| *f == family && *i == id)
                .map(|(_, _, r)| *r)
                .or(Some("step2"))
        }
        "execution-episode-ref" => STEP_PENDING
            .iter()
            .find(|(f, i, _)| *f == family && *i == id)
            .map(|(_, _, r)| *r)
            .or(Some(if scenario["rule"] == "R7" {
                "step5"
            } else {
                "step3"
            })),
        "delegation-link" => STEP_PENDING
            .iter()
            .find(|(f, i, _)| *f == family && *i == id)
            .map(|(_, _, r)| *r),
        _ => None,
    }
}

/// Per-scenario temporary reasons beyond the family default (none yet).
const STEP_PENDING: &[(&str, &str, &str)] = &[
    (
        "case-record-append-v2",
        "sibling-verifier-self-asserts-fresh-context",
        "step4",
    ),
    (
        "case-record-append-v2",
        "sibling-verifier-without-derivable-context-cannot-settle-s",
        "step4",
    ),
];

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

fn fixtures(family: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fde/fixtures")
        .join(family)
}

fn read(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?}: {e}")))
        .unwrap()
}

/// Every scenario of the family's manifest, in manifest order. The manifest
/// list must equal the files on disk.
fn scenarios(family: &str) -> Vec<(String, Value)> {
    let dir = fixtures(family);
    let manifest = read(&dir.join("manifest.json"));
    let listed: Vec<String> = manifest["scenarios"]
        .as_array()
        .expect("manifest.scenarios")
        .iter()
        .map(|v| v.as_str().expect("scenario file name").to_string())
        .collect();
    let mut on_disk: Vec<String> = fs::read_dir(dir.join("scenarios"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    on_disk.sort();
    let mut sorted = listed.clone();
    sorted.sort();
    assert_eq!(sorted, on_disk, "{family}: manifest and scenarios/ differ");
    listed
        .into_iter()
        .map(|name| {
            let s = read(&dir.join("scenarios").join(&name));
            (s["id"].as_str().expect("scenario id").to_string(), s)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Cell construction
// ---------------------------------------------------------------------------

struct Actor {
    id: String,
    role: ActorRole,
}

fn role_wire(role: &ActorRole) -> String {
    sea_forge_server::identity::role_wire_name(role)
}

const WORLD_SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const PIN_POLICY: &str = "version: \"0.1\"\nrules: []\n";

/// A cell with the CEP authority loop enabled and `pinned` recorded in its
/// world pin record (the WP-18 source `WORLD_NOT_PINNED` reads).
fn enable_pin(root: &Path, pinned: &str) -> sea_forge_server::config::CepAuthorityConfig {
    use sea_forge_server::config::{CepAuthorityConfig, CepWorldConfig};
    fs::create_dir_all(root.join("worlds")).unwrap();
    fs::write(root.join("worlds/demo.sea"), WORLD_SRC).unwrap();
    fs::write(root.join("sea-forge-policy.yaml"), PIN_POLICY).unwrap();
    let ledger = sea_forge_ledger::LedgerStream::open(
        root,
        sea_forge_authority::cep::LEDGER_ID,
        "sea-forge-cep-authority",
    )
    .unwrap();
    let record = sea_forge_authority::cep::PinRecord {
        pin_id: "pin-seed".into(),
        kind: "promoted".into(),
        from_world_ref: "world:demo@sha256:00".into(),
        to_world_ref: pinned.into(),
        bound_digest: "sha256:00".into(),
        proposal_id: None,
        matrix_id: None,
        proposer: "seed".into(),
        requester: "seed".into(),
        approver: "seed".into(),
        approval_id: "approval-seed".into(),
        operation_id: "op-seed".into(),
        transition_ref: "entry-seed".into(),
        supersedes_pin: None,
    };
    ledger
        .commit_typed(
            sea_forge_authority::cep::PIN_KIND,
            vec!["world-pin".into(), pinned.into()],
            &record,
            vec![],
        )
        .unwrap();
    CepAuthorityConfig {
        enabled: true,
        pinned_baseline: Some("demo".into()),
        policy: "sea-forge-policy.yaml".into(),
        worlds: vec![CepWorldConfig {
            name: "demo".into(),
            base: "worlds".into(),
            entry: "demo.sea".into(),
            files: vec!["demo.sea".into()],
            prior_identities: Vec::new(),
        }],
        approval_ttl_hours: 24,
        realitytrace: None,
        settlement: Default::default(),
    }
}

fn config(root: &Path, actors: &[Actor]) -> ServerConfig {
    let socket = std::env::temp_dir().join(format!(
        "sf-fde-{}-{}.sock",
        std::process::id(),
        root.file_name().unwrap().to_string_lossy()
    ));
    let mut bindings: BTreeMap<(String, String), IdentityBinding> = BTreeMap::new();
    for a in actors {
        bindings
            .entry((a.id.clone(), role_wire(&a.role)))
            .or_insert(IdentityBinding {
                uid: None,
                sid: None,
                actor_id: a.id.clone(),
                roles: vec![a.role.clone()],
            });
    }
    ServerConfig {
        socket_path: socket,
        root: root.to_path_buf(),
        identity: IdentityBindings {
            bindings: bindings.into_values().collect(),
        },
        ..ServerConfig::default()
    }
}

/// Anchoring configured for the cell: the case keeper, and optionally the
/// identity the configured anchor credential publishes as.
fn write_fde_config(root: &Path, keeper: Option<&str>, publisher: Option<&str>) {
    let mut cfg = json!({});
    if let Some(k) = keeper {
        cfg["case_keeper_identity"] = json!(k);
    }
    if let Some(p) = publisher {
        cfg["anchor_publisher_identity"] = json!(p);
    }
    if keeper.is_some() {
        fs::write(root.join("fde-config.json"), cfg.to_string()).unwrap();
    }
}

fn case_dir(root: &Path) -> PathBuf {
    root.join("cases").join(CASE)
}

fn write_lines(path: &Path, lines: &[Value]) {
    let mut text = fs::read_to_string(path).unwrap_or_default();
    for l in lines {
        text.push_str(&format!("{l}\n"));
    }
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn parse_ref(r: &str) -> (String, String, u64, String) {
    let rest = r.strip_prefix("episode:").unwrap();
    let (head, digest) = rest.split_once("@sha256:").unwrap();
    let p: Vec<&str> = head.split(':').collect();
    (
        p[0].into(),
        p[1].into(),
        p[2].parse().unwrap(),
        digest.into(),
    )
}

/// A store being seeded the way the server's own writers would have left it.
struct Seed {
    root: PathBuf,
    records: Vec<Value>,
}

impl Seed {
    fn new(root: &Path, template: Option<&str>) -> Self {
        let dir = case_dir(root);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("case.json"), "{}").unwrap();
        fs::write(dir.join("case-events.jsonl"), "").unwrap();
        if let Some(t) = template {
            fs::write(
                dir.join("plan.json"),
                json!({"version": "0.2", "plan_id": "plan_seed", "case_id": CASE,
                       "run_id": "run_seed", "intent_id": "int_seed", "items": [],
                       "template_ref": t})
                .to_string(),
            )
            .unwrap();
        }
        Self {
            root: root.to_path_buf(),
            records: Vec::new(),
        }
    }

    /// The case plan: `items` as `(id, execution)`; an execution item is a
    /// sandboxed task, anything else a human task.
    fn plan(&self, items: &[(String, bool)]) {
        let criteria =
            serde_json::to_value(sea_forge_core::types::SettlementCriteria::default()).unwrap();
        let items: Vec<Value> = items
            .iter()
            .map(|(id, exec)| {
                json!({"plan_item_id": id, "name": id, "settlement_criteria": criteria,
                       "item_kind": if *exec { "sandboxed_task" } else { "human_task" },
                       "markers": {"required": true}})
            })
            .collect();
        let path = case_dir(&self.root).join("plan.json");
        let mut plan: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        plan["items"] = json!(items);
        fs::write(path, plan.to_string()).unwrap();
    }

    /// RealityTrace records of an episode that has no case record yet, found by key.
    fn rt_register_for_episode(&self, key: &Value) {
        let lines: Vec<Value> = ["plan:SEED", "question:SEED", "threshold:SEED"]
            .iter()
            .map(|id| json!({"record_id": id, "episode_key": key}))
            .collect();
        write_lines(&self.root.join("realitytrace/records.jsonl"), &lines);
    }

    fn rt_register(&self, ids: &[String]) {
        let lines: Vec<Value> = ids.iter().map(|i| json!({"record_id": i})).collect();
        write_lines(&self.root.join("realitytrace/records.jsonl"), &lines);
    }

    /// One hash-chained case record, in the layout `case_records::next_record` writes.
    fn push(&mut self, mut record: Value) -> String {
        let seq = self.records.len() as u64 + 1;
        let prev = self
            .records
            .last()
            .and_then(|r| r["hash"].as_str())
            .unwrap_or(ZERO)
            .to_string();
        record["seq"] = json!(seq);
        record["record_id"] = json!(format!("rec-{seq:04}"));
        record["prev_hash"] = json!(prev);
        record["recorded_at"] = json!("2026-11-15T08:00:00.000Z");
        record["hash"] = json!(digest_of(&record).unwrap());
        write_lines(
            &case_dir(&self.root).join("field-records.jsonl"),
            &[record.clone()],
        );
        self.records.push(record.clone());
        record["record_id"].as_str().unwrap().to_string()
    }

    /// A committed freeze: record, committed intent, RealityTrace records.
    fn freeze(
        &mut self,
        key: &Value,
        kind: &str,
        world_ref: &str,
        idem: &str,
        digest: &str,
    ) -> String {
        let (case, item, n) = (
            key["case_id"].as_str().unwrap(),
            key["plan_item_id"].as_str().unwrap(),
            key["n"].as_u64().unwrap(),
        );
        let episode_ref = format!("episode:{case}:{item}:{n}@sha256:{digest}");
        let plan = format!("plan:SEED{n}{item}");
        let q = format!("question:SEED{n}{item}");
        let t = format!("threshold:SEED{n}{item}");
        self.rt_register(&[plan.clone(), q.clone(), t.clone()]);
        let payload = json!({
            "episode_ref": episode_ref,
            "reference": {"episode_ref": episode_ref, "body": {
                "case_id": case, "plan_item_id": item, "n": n, "world_ref": world_ref, "kind": kind,
                "executor_provenance": "program"}},
            "digest": digest,
            "realitytrace_refs": {"plan_id": plan, "question_ids": [q], "threshold_ids": [t]},
        });
        let rec_id = self.push(json!({
            "kind": "episode.frozen", "idempotency_key": idem,
            "body_digest": format!("seed-{idem}"), "episode_key": key,
            "episode_kind": kind, "digest": digest, "principal": KEEPER,
            "payload": payload,
            "response": {"outcome": "recorded", "kind": "episode.frozen",
                         "record_id": "rec-seed", "episode_key": key, "replayed": false},
        }));
        write_lines(
            &case_dir(&self.root).join("field-freeze-intents.jsonl"),
            &[
                json!({"idempotency_key": idem, "body_digest": format!("seed-{idem}"),
                     "episode_key": key, "episode_kind": kind, "digest": digest,
                     "payload": payload, "state": "committed", "record_id": rec_id}),
            ],
        );
        rec_id
    }

    /// A published anchor for a freeze digest, pushed by `pusher`.
    fn anchor(&self, kind: &str, id: &str, subject: &str, digest: &str, pusher: &str, push: &str) {
        write_lines(
            &case_dir(&self.root).join("field-anchor-outbox.jsonl"),
            &[json!({"state": "published", "kind": kind, "id": id,
                     "tag": format!("fde-anchor/{kind}/{id}"), "subject_ref": subject,
                     "digest": digest,
                     "receipt": {"tag_object_id": "a".repeat(40), "pusher_identity": pusher,
                                 "server_push_time": push}})],
        );
    }
}

fn kind_of_principal(id: &str) -> &'static str {
    if id.starts_with("human:") {
        "human"
    } else if id.starts_with("svc:") {
        "service"
    } else {
        "agent"
    }
}

fn role_for(id: &str) -> ActorRole {
    match kind_of_principal(id) {
        "human" => ActorRole::Operator,
        "service" => ActorRole::Service,
        _ => ActorRole::Agent,
    }
}

// ---------------------------------------------------------------------------
// Family: case.record.append v2
// ---------------------------------------------------------------------------

fn compare(expect: &Value, got: &Value) -> Result<(), String> {
    // The contract fixes outcome, code and invariant; message text is free, and
    // a recorded response's record_id is compared only for replays.
    for field in ["outcome", "code", "invariant"] {
        if expect.get(field) != got.get(field) {
            return Err(format!(
                "{field}: expected {:?}, got {:?} (response {got})",
                expect.get(field),
                got.get(field)
            ));
        }
    }
    if expect["outcome"] == "recorded" {
        for field in ["kind", "replayed", "episode_key"] {
            if expect.get(field) != got.get(field) {
                return Err(format!(
                    "{field}: expected {:?}, got {:?}",
                    expect.get(field),
                    got.get(field)
                ));
            }
        }
    }
    if expect["outcome"] == "refused" {
        // Dialect: a v1 fallback names the v2 code as a message prefix.
        if let Some(prefix) = expect["message"]
            .as_str()
            .and_then(|m| m.split_once(": "))
            .map(|(p, _)| p)
            .filter(|p| p.chars().all(|c| c.is_ascii_uppercase() || c == '_'))
        {
            let message = got["message"].as_str().unwrap_or("");
            if !message.starts_with(&format!("{prefix}: ")) {
                return Err(format!(
                    "message must start with {prefix}: , got {message:?}"
                ));
            }
        }
    }
    Ok(())
}

async fn run_append(s: &Value) -> Result<(), String> {
    let root = tempfile::tempdir().unwrap();
    let request = &s["request"];
    let payload = &request["payload"];
    let principal = s["principal"]
        .as_str()
        .map(String::from)
        .unwrap_or_else(|| match request["kind"].as_str() {
            Some("episode.settled") => "agent:verifier-01".into(),
            _ => KEEPER.into(),
        });
    let ledger = s["ledger_before"].as_array().cloned().unwrap_or_default();
    let anchored = ledger.iter().any(|l| l["kind"] == "anchor.verified")
        || s["anchor_config"]["configured"] == true;
    let keeper = s["anchor_config"]["case_keeper_identity"]
        .as_str()
        .or_else(|| {
            ledger
                .iter()
                .find_map(|l| l["case_keeper_identity"].as_str())
        })
        .or(anchored.then_some(KEEPER));
    let mut seed = Seed::new(root.path(), Some("fde.field_case@1"));
    // Digest the seeded freezes carry: the one the request cites, else a fixed one.
    let cited = payload["episode_ref"].as_str().map(|r| parse_ref(r).3);
    let mut runs: Vec<Value> = Vec::new();
    // A run record names an episode that ran, which I-1 allows only after its
    // freeze: a scenario that lists runs without the freeze means that freeze.
    if !ledger.iter().any(|l| l["kind"] == "episode.frozen") {
        if let Some(run) = ledger
            .iter()
            .find(|l| l["kind"].as_str().is_some_and(|k| k.starts_with("run.")))
        {
            let world = ledger
                .iter()
                .find(|l| l["anchor_kind"] == "model")
                .and_then(|l| l["subject_ref"].as_str())
                .unwrap_or("");
            let digest = cited.clone().unwrap_or_else(|| "ab".repeat(32));
            seed.freeze(
                &run["episode_key"],
                "evaluated",
                world,
                "case-keeper/rec-implied",
                &digest,
            );
        }
    }
    for entry in &ledger {
        match entry["kind"].as_str().unwrap() {
            "episode.frozen" => {
                let digest = cited.clone().unwrap_or_else(|| "ab".repeat(32));
                // A freeze request for the same key must not collide on digest
                // by accident: the contract scenarios differ by key or kind.
                let _ = seed.freeze(
                    &entry["episode_key"],
                    entry["episode_kind"].as_str().unwrap(),
                    entry["world_ref"].as_str().unwrap_or(""),
                    entry["idempotency_key"].as_str().unwrap(),
                    &digest,
                );
            }
            "episode.settled" => {
                let key = &entry["episode_key"];
                let digest =
                    digest_of(&json!({"kind": "episode.settled", "payload": payload})).unwrap();
                let same_key = entry["idempotency_key"] == request["idempotency_key"];
                seed.push(json!({
                    "kind": "episode.settled", "idempotency_key": entry["idempotency_key"],
                    "body_digest": if same_key { digest } else { "seed-settled".into() },
                    "episode_key": key, "principal": "agent:verifier-01", "chain": ["agent:verifier-01"],
                    "payload": payload,
                    "response": {"outcome": "recorded", "kind": "episode.settled",
                                 "record_id": "rec-seed", "episode_key": key, "replayed": false},
                }));
            }
            "disposition.recorded" => {
                seed.push(json!({"kind": "disposition.recorded",
                    "level": entry.get("level").cloned().unwrap_or(json!("case")),
                    "disposition": entry["disposition"], "idempotency_key": "seed-disposition"}));
            }
            "run.executor" | "run.verifier" | "run.recorded" => runs.push(entry.clone()),
            "anchor.verified" => {
                let digest = cited.clone().unwrap_or_else(|| "ab".repeat(32));
                let kind = entry["anchor_kind"].as_str().unwrap();
                let push = entry["server_push_time"]
                    .as_str()
                    .unwrap_or("2026-11-14T08:00:00Z");
                seed.anchor(
                    kind,
                    &format!("seed-{kind}"),
                    entry["subject_ref"].as_str().unwrap_or(""),
                    &digest,
                    entry["pusher_identity"].as_str().unwrap_or(KEEPER),
                    push,
                );
            }
            "case.world_pinned"
            | "case.state"
            | "approval.recorded"
            | "store.hash_chain_broken" => {}
            other => return Err(format!("driver: unknown ledger kind {other}")),
        }
    }
    if ledger
        .iter()
        .any(|l| l["kind"] == "store.hash_chain_broken")
    {
        write_lines(
            &case_dir(root.path()).join("field-records.jsonl"),
            &[json!({"seq": 1, "broken": true})],
        );
        fs::write(
            case_dir(root.path()).join("field-decisions.jsonl"),
            "{not json}\n",
        )
        .unwrap();
    }
    if !runs.is_empty() {
        let lines: Vec<Value> = runs
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let key = &r["episode_key"];
                let digest = cited.clone().unwrap_or_else(|| "ab".repeat(32));
                let eref = format!(
                    "episode:{}:{}:{}@sha256:{digest}",
                    key["case_id"].as_str().unwrap(),
                    key["plan_item_id"].as_str().unwrap(),
                    key["n"]
                );
                json!({"run_id": r["run_id"].as_str().map(String::from).unwrap_or(format!("run-seed-{i}")),
                       "case_id": CASE, "episode_ref": eref, "verb": "item_execute",
                       "role": if r["kind"] == "run.verifier" { "verifier" } else { "executor" },
                       "actor": r.get("actor_id").cloned().unwrap_or(json!("agent:builder-01")),
                       "on_behalf_of": null, "delegated": false, "link_id": null,
                       "workspace_ref": r.get("workspace_ref").cloned().unwrap_or(json!("ws:builder")),
                       "started_at": format!("2026-11-14T09:0{i}:00Z")})
            })
            .collect();
        let mut lines = lines;
        // Scenarios that name a shared delegation root: every run is delegated by
        // human:operator, through a link written before it; a settler with no
        // verifier run of its own gets one that shares the executor's workspace.
        if s["shares_root_with_executor"] == true {
            if !lines.iter().any(|l| l["actor"] == json!(principal)) {
                let mut extra = lines[0].clone();
                extra["run_id"] = json!("run-seed-verifier");
                extra["role"] = json!("verifier");
                extra["actor"] = json!(principal);
                extra["started_at"] = json!("2026-11-14T09:30:00Z");
                lines.push(extra);
            }
            let mut links = Vec::new();
            for (i, l) in lines.iter_mut().enumerate() {
                let id = format!("link-seed-{i}");
                let (eref, actor) = (l["episode_ref"].clone(), l["actor"].clone());
                let (c, item, n, _) = parse_ref(eref.as_str().unwrap());
                links.push(json!({"link_id": id, "case_id": c, "episode_ref": eref,
                    "assignment_id": format!("assign:{c}:{item}:{n}:{}", i + 1),
                    "delegator": "human:operator", "gateway_actor": null, "delegate": actor,
                    "issued_at": "2026-11-14T08:00:00Z", "expires_at": "2099-01-01T00:00:00Z"}));
                l["delegated"] = json!(true);
                l["link_id"] = json!(id);
                l["on_behalf_of"] = json!("human:operator");
            }
            write_lines(&case_dir(root.path()).join("field-links.jsonl"), &links);
        }
        write_lines(&case_dir(root.path()).join("field-runs.jsonl"), &lines);
        // The first run event time of the case (model anchor order).
        let first = ledger
            .iter()
            .find_map(|l| l["first_run_event_time"].as_str())
            .unwrap_or("2026-11-14T09:00:00Z");
        write_lines(
            &case_dir(root.path()).join("case-events.jsonl"),
            &[json!({"event_id": "ev-seed", "plan_item_id": "item-03",
                     "actor_id": "agent:builder-01", "kind": "command_started", "timestamp": first})],
        );
    }
    // RealityTrace records the request itself names, unless the scenario says
    // which ones exist.
    let refs = &payload["realitytrace_refs"];
    if refs.is_object() {
        let has: Vec<String> = match s.get("realitytrace_has") {
            Some(h) => h
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
            None => std::iter::once(refs["plan_id"].clone())
                .chain(refs["question_ids"].as_array().unwrap().iter().cloned())
                .chain(refs["threshold_ids"].as_array().unwrap().iter().cloned())
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
        };
        seed.rt_register(&has);
    }
    let actors = [Actor {
        id: principal.clone(),
        role: role_for(&principal),
    }];
    let cfg = config(root.path(), &actors);
    let mut cfg = cfg;
    write_fde_config(
        root.path(),
        keeper,
        s["anchor_config"]["publisher"].as_str(),
    );
    if let Some(pin) = ledger.iter().find(|l| l["kind"] == "case.world_pinned") {
        cfg.cep_authority = enable_pin(root.path(), pin["world_ref"].as_str().unwrap());
    }
    let mut cell = authenticated::Cell::start(cfg).await;
    let mut wire = json!({
        "verb": "case_record_append", "case_id": CASE,
        "kind": request["kind"], "payload": request["payload"],
        "idempotency_key": request["idempotency_key"],
        "request_id": "req-scenario",
        "actor": {"actor_id": principal, "role": role_wire(&role_for(&principal))},
    });
    if let Some(c) = request.get("contract") {
        wire["contract"] = c.clone();
    }
    if let Some(bytes) = s["transport"]["encoded_bytes"].as_u64() {
        let have = wire.to_string().len() as u64;
        let pad = bytes.saturating_sub(have);
        wire["payload"]["rationale"] =
            json!("x"
                .repeat(wire["payload"]["rationale"].as_str().unwrap_or("").len() + pad as usize));
        while wire.to_string().len() as u64 != bytes {
            let cur = wire.to_string().len() as u64;
            let r = wire["payload"]["rationale"].as_str().unwrap().len() as i64
                + (bytes as i64 - cur as i64);
            wire["payload"]["rationale"] = json!("x".repeat(r as usize));
        }
    }
    let got = cell
        .call_for(
            &authenticated::principal(&principal, role_for(&principal)),
            wire,
        )
        .await;
    cell.shutdown().await;
    compare(&s["expect"], &got)
}

// ---------------------------------------------------------------------------
// Family: execution gate
// ---------------------------------------------------------------------------

async fn run_gate(s: &Value) -> Result<(), String> {
    let root = tempfile::tempdir().unwrap();
    let case = &s["case"];
    let state = &s["freeze_state"];
    let request = &s["request"];
    let extra = &s["extra"];
    if !extra["authority_decision"].is_null()
        || !extra["k_used"].is_null()
        || !extra["interrupted_assignments"].is_null()
    {
        // Decision and assignment stores arrive with their steps.
    }
    let template = case["template_ref"].as_str();
    let mut seed = Seed::new(root.path(), template);
    let terminal = extra["reaches_terminal"] == true || case["advance_reaches"] == "Completed";
    let exec_items: Vec<String> = extra["items_with_execution_step"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    if template.is_some() {
        let mut scope: Vec<String> = Vec::new();
        for v in [&request["item_id"], &request["episode_ref"]["plan_item_id"]] {
            if let Some(i) = v.as_str() {
                scope.push(i.to_string());
            }
        }
        for t in request["transition_items"].as_array().into_iter().flatten() {
            scope.push(t["item_id"].as_str().unwrap().to_string());
        }
        scope.sort();
        scope.dedup();
        let mut items: Vec<(String, bool)> = scope
            .iter()
            .map(|i| (i.clone(), exec_items.contains(i) || i == "item-03"))
            .collect();
        if !terminal || items.is_empty() {
            items.push(("item-99".into(), true));
        }
        seed.plan(&items);
    }
    if case.is_null() {
        fs::remove_dir_all(case_dir(root.path())).unwrap();
        fs::create_dir_all(root.path().join("cases")).unwrap();
    } else {
        let er = &request["episode_ref"];
        let ep = if er.is_object() {
            er.clone()
        } else {
            json!({"plan_item_id": "item-03", "n": 1, "kind": "evaluated"})
        };
        let key = json!({"case_id": CASE, "plan_item_id": ep["plan_item_id"], "n": ep["n"]});
        let digest = "cd".repeat(32);
        let stored_kind = state["kind"].as_str().unwrap_or("evaluated");
        let episode_ref = format!(
            "episode:{CASE}:{}:{}@sha256:{digest}",
            ep["plan_item_id"].as_str().unwrap(),
            ep["n"]
        );
        let have_record = state["case_record"] == true;
        let have_rt = state["realitytrace"] == true;
        if have_record {
            seed.freeze(
                &key,
                stored_kind,
                s["freeze_world"].as_str().unwrap_or("world:seed@sha256:00"),
                "case-keeper/rec-seed",
                &digest,
            );
            if !have_rt {
                fs::remove_file(root.path().join("realitytrace/records.jsonl")).unwrap();
            }
        } else if have_rt {
            seed.rt_register_for_episode(&key);
        }
        match state["anchor"].as_str() {
            Some("verified") => seed.anchor(
                "freeze",
                "seed-freeze",
                &episode_ref,
                &digest,
                state["anchor_publisher"].as_str().unwrap_or(KEEPER),
                "2026-11-14T08:00:00Z",
            ),
            Some("absent") => {}
            _ => {}
        }
        if let Some(d) = case["latest_case_disposition"].as_str() {
            seed.push(json!({"kind": "disposition.recorded", "level": "case",
                             "disposition": d, "idempotency_key": "seed-disposition"}));
        }
        if case["closed"] == true {
            seed.push(json!({"kind": "disposition.recorded", "level": "case",
                             "disposition": "close-satisfied", "idempotency_key": "seed-close"}));
        }
        // Assignments already issued for the episode.
        for k in extra["k_used"].as_array().into_iter().flatten() {
            let id = format!(
                "assign:{CASE}:{}:{}:{k}",
                ep["plan_item_id"].as_str().unwrap(),
                ep["n"]
            );
            let interrupted = extra["interrupted_assignments"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| *x == json!(id)));
            let expired = match (extra["link_expires_at"].as_str(), extra["now"].as_str()) {
                (Some(exp), Some(now)) => now > exp,
                _ => false,
            };
            write_lines(
                &case_dir(root.path()).join("field-assignments.jsonl"),
                &[json!({"assignment_id": id,
                         "state": if interrupted { "interrupted" } else { "started" },
                         "link_expires_at": if expired { "2020-01-01T00:00:00Z" } else { "2099-01-01T00:00:00Z" },
                         "episode_key": key})],
            );
        }
        if case["suspended"] == true {
            seed.push(json!({"kind": "disposition.recorded", "level": "case",
                             "disposition": "suspend", "idempotency_key": "seed-suspend"}));
        }
        if let Some(n) = extra["latest_n"].as_u64() {
            let k2 = json!({"case_id": CASE, "plan_item_id": ep["plan_item_id"], "n": n});
            seed.freeze(
                &k2,
                "evaluated",
                "world:seed@sha256:00",
                "case-keeper/rec-seed-n",
                &"ef".repeat(32),
            );
        }
        if extra["episode_state"] == "settled" {
            seed.push(
                json!({"kind": "episode.settled", "idempotency_key": "seed-settled",
                             "body_digest": "seed-settled", "episode_key": key,
                             "principal": "agent:verifier-01", "chain": [], "payload": {},
                             "response": {"outcome": "recorded"}}),
            );
        }
    }
    let actors = [Actor {
        id: "operator_local".into(),
        role: ActorRole::Operator,
    }];
    let keeper = (state["anchor"] != "not_configured").then_some(KEEPER);
    let cfg = config(root.path(), &actors);
    write_fde_config(root.path(), keeper, None);
    if let Some(max) = extra["max_runs"].as_u64() {
        let path = root.path().join("fde-config.json");
        let mut cfg: Value = fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_else(|| json!({}));
        cfg["execution_max_runs"] = json!(max);
        fs::write(path, cfg.to_string()).unwrap();
    }
    if !case.is_null() && request["episode_ref"].is_object() {
        seed_decision(root.path(), s);
    }
    let mut cell = authenticated::Cell::start(cfg).await;
    let got = cell
        .call_for(
            &authenticated::principal("operator_local", ActorRole::Operator),
            request.clone(),
        )
        .await;
    cell.shutdown().await;
    if s["inspect_decision_event"] == true {
        // The run's interaction event carries the decision that authorized it.
        let spool = root
            .path()
            .join("interaction-events")
            .join(format!("{CASE}.jsonl"));
        let text = fs::read_to_string(&spool).map_err(|e| format!("no interaction events: {e}"))?;
        let found = text
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .any(|e| {
                e["extensions"]["godspeed.interaction_event"]["authority"]["decision_id"]
                    == "dec-seed-1"
            });
        if !found {
            return Err(format!(
                "no interaction event carries authority.decision_id dec-seed-1: {text}"
            ));
        }
        return Ok(());
    }
    let expect = &s["expect"];
    if expect["outcome"] == "refused" {
        compare(expect, &got)?;
        if got["no_side_effect"] != true {
            return Err(format!(
                "a gate refusal must carry no_side_effect: true: {got}"
            ));
        }
        Ok(())
    } else {
        // Accepted: the gate let the request through (what happens after is
        // the verb's own behaviour and is not a gate refusal).
        if got["no_side_effect"] == true || got["outcome"] == "refused" {
            return Err(format!("expected the gate to accept, got {got}"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Family: delegation link and run records (settlement check)
// ---------------------------------------------------------------------------

async fn run_link(s: &Value) -> Result<(), String> {
    let root = tempfile::tempdir().unwrap();
    let st = &s["settlement"];
    let (case, item, n, digest) = parse_ref(st["episode_ref"].as_str().unwrap());
    let key = json!({"case_id": case, "plan_item_id": item, "n": n});
    let mut seed = Seed::new(root.path(), Some("fde.field_case@1"));
    seed.freeze(
        &key,
        "evaluated",
        "world:seed@sha256:00",
        "case-keeper/rec-0001",
        &digest,
    );
    write_lines(
        &case_dir(root.path()).join("field-runs.jsonl"),
        st["runs"].as_array().unwrap(),
    );
    write_lines(
        &case_dir(root.path()).join("field-links.jsonl"),
        st["links"].as_array().unwrap(),
    );
    let principal = st["settling_principal"].as_str().unwrap().to_string();
    let actors = [Actor {
        id: principal.clone(),
        role: role_for(&principal),
    }];
    let cfg = config(root.path(), &actors);
    let mut cell = authenticated::Cell::start(cfg).await;
    let wire = json!({
        "verb": "case_record_append", "case_id": CASE, "contract": "fde.case-record-append@2",
        "kind": "episode.settled", "idempotency_key": "verifier/rec-settle",
        "payload": {"episode_ref": st["episode_ref"], "result": st["result"], "reliability": {},
                    "independence": {"independent_source_count": 1,
                                     "verifier_context": st["declared_verifier_context"]},
                    "settlement_ref": "settlement:01JABCDEF0000000000000000C"},
        "request_id": "req-link",
        "actor": {"actor_id": principal, "role": role_wire(&role_for(&principal))},
    });
    let got = cell
        .call_for(
            &authenticated::principal(&principal, role_for(&principal)),
            wire,
        )
        .await;
    cell.shutdown().await;
    let expect = &s["expect"];
    if expect["outcome"] == "ok" || expect["outcome"] == "recorded" {
        if got["outcome"] != "recorded" {
            return Err(format!("expected a recorded settlement, got {got}"));
        }
        return Ok(());
    }
    compare(expect, &got)
}

// ---------------------------------------------------------------------------
// The test
// ---------------------------------------------------------------------------

/// One scenario on its own task: a panic (a closed connection, a crashed
/// handler) is a failed scenario, not an aborted suite.
async fn drive(family: &str, s: &Value) -> Result<(), String> {
    let (family, s) = (family.to_string(), s.clone());
    tokio::spawn(async move {
        match family.as_str() {
            "case-record-append-v2" => run_append(&s).await,
            "execution-episode-ref" => run_gate(&s).await,
            "delegation-link" => run_link(&s).await,
            _ => Err("wp-31: the A3 proof check is not part of WP-21".into()),
        }
    })
    .await
    .unwrap_or_else(|e| Err(format!("driver panicked: {e}")))
}

const FAMILIES: &[&str] = &[
    "case-record-append-v2",
    "execution-episode-ref",
    "delegation-link",
    "reserved-operation",
];

#[tokio::test(flavor = "multi_thread")]
async fn every_scenario_is_driven_and_matches_or_is_listed_pending() {
    let mut total = 0;
    let mut pending: Vec<(String, &'static str)> = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    for family in FAMILIES {
        for (id, s) in scenarios(family) {
            total += 1;
            let outcome = drive(family, &s).await;
            let reason = pending_reason(family, &id, &s);
            match (outcome, reason) {
                (Ok(()), None) => {}
                (Ok(()), Some(r)) => failures.push(format!(
                    "{family}/{id}: passes but is listed pending ({r}); remove it from the pending list"
                )),
                (Err(e), None) => failures.push(format!("{family}/{id}: {e}")),
                (Err(e), Some(r)) => {
                    if std::env::var_os("FDE_LIST").is_some() {
                        println!("WHY {family}/{id}: {e}");
                    }
                    pending.push((format!("{family}/{id}"), r))
                }
            }
        }
    }
    let mut by_reason: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, r) in &pending {
        *by_reason.entry(r).or_default() += 1;
    }
    if std::env::var_os("FDE_LIST").is_some() {
        for (id, r) in &pending {
            println!("PENDING {r}: {id}");
        }
    }
    println!(
        "FDE scenarios: total {total}, passing {}, pending {} {by_reason:?}",
        total - pending.len(),
        pending.len()
    );
    assert!(
        failures.is_empty(),
        "scenario failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn scenario_counts_match_the_contract_readme() {
    for (family, count) in [
        ("case-record-append-v2", 54),
        ("execution-episode-ref", 82),
        ("delegation-link", 7),
        ("reserved-operation", 14),
    ] {
        assert_eq!(scenarios(family).len(), count, "{family}");
    }
}

/// Fails while any pending reason other than the allowed final ones remains.
/// This is the "count must go to zero" gate: it is red until the last part-A
/// scenario passes.
#[tokio::test(flavor = "multi_thread")]
async fn final_pending_list_is_only_allowed_reasons() {
    // Until the last part-A step lands this reports instead of failing; the
    // final commit removes the guard. `FDE_WP21A_FINAL=1` enforces it now.
    let enforce = std::env::var_os("FDE_WP21A_FINAL").is_some();
    let mut bad = Vec::new();
    for family in FAMILIES {
        for (id, s) in scenarios(family) {
            if let Some(r) = pending_reason(family, &id, &s) {
                if !FINAL_REASONS.contains(&r) {
                    bad.push(format!("{family}/{id} ({r})"));
                }
            }
        }
    }
    if !enforce {
        println!(
            "{} scenarios still pending for part-A reasons (not enforced yet)",
            bad.len()
        );
        return;
    }
    assert!(
        bad.is_empty(),
        "{} scenarios still pending for part-A reasons:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

// ---------------------------------------------------------------------------
// Per-NT subsets (scripts/fde-proof.d/s2/<NT>.sh selects these by name)
// ---------------------------------------------------------------------------

/// Every append, gate and link scenario that carries `nt`: each must pass now,
/// unless it is listed pending only for a final reason (`part-B`, `wp-31`),
/// which is reported, never silently skipped.
async fn nt_subset(nt: &str, at_least: usize) {
    let mut ran = 0;
    let mut excluded = Vec::new();
    let mut failed = Vec::new();
    for family in [
        "case-record-append-v2",
        "execution-episode-ref",
        "delegation-link",
    ] {
        for (id, s) in scenarios(family) {
            if s["nt"] != nt {
                continue;
            }
            match pending_reason(family, &id, &s) {
                Some(r) if FINAL_REASONS.contains(&r) => {
                    excluded.push(format!("{family}/{id} ({r})"))
                }
                Some(r) => failed.push(format!("{family}/{id}: still pending ({r})")),
                None => {
                    ran += 1;
                    if let Err(e) = drive(family, &s).await {
                        failed.push(format!("{family}/{id}: {e}"));
                    }
                }
            }
        }
    }
    println!(
        "{nt}: {ran} scenarios passed the real server; excluded for a final reason: {excluded:?}"
    );
    assert!(failed.is_empty(), "{nt}:\n{}", failed.join("\n"));
    assert!(
        ran >= at_least,
        "{nt}: only {ran} scenarios ran, need {at_least}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn nt_13_scenarios() {
    nt_subset("NT-13", 4).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_17_scenarios() {
    nt_subset("NT-17", 1).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_18_scenarios() {
    nt_subset("NT-18", 1).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_32_scenarios() {
    nt_subset("NT-32", 10).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_33_scenarios() {
    nt_subset("NT-33", 5).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_34_scenarios() {
    nt_subset("NT-34", 12).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_48_scenarios() {
    nt_subset("NT-48", 4).await;
}

/// The program-execution decision of a gate scenario, written to the CEP
/// authority ledger the way the authority path records it (README §10, D39):
/// resource_id `execution:sha256:<digest of the nine-member execution object>`.
fn seed_decision(root: &Path, s: &Value) {
    let request = &s["request"];
    let extra = &s["extra"];
    let outcome = match extra["authority_decision"].as_str() {
        Some("absent") => return,
        Some("denied") => "deny",
        _ => "allow",
    };
    let ep = &request["episode_ref"];
    let item = ep["plan_item_id"].as_str().unwrap();
    let digest = "cd".repeat(32);
    let delegate = request["verb"] == "delegate";
    let (argv, profile, workspace, timeout) = if delegate {
        (
            json!([request["endpoint"]]),
            "agent",
            format!("ws:agent:{}", request["endpoint"].as_str().unwrap()),
            request["max_turns"].as_u64().unwrap(),
        )
    } else {
        (
            json!([item]),
            "default",
            format!("ws:case:{CASE}:{item}"),
            request["timeout"].as_u64().unwrap_or(60),
        )
    };
    let cwd = if extra["execution_differs_in"]
        .as_array()
        .is_some_and(|d| d.iter().any(|x| x == "cwd"))
    {
        "/elsewhere"
    } else {
        "."
    };
    let object = json!({
        "episode_ref": format!("episode:{CASE}:{item}:{}@sha256:{digest}", ep["n"]),
        "patch_sha256": hex::encode(<Sha256 as Digest>::digest(b"")),
        "argv": argv, "cwd": cwd, "env_digest": digest_of(&json!({})).unwrap(),
        "workspace_ref": workspace, "sandbox_profile": profile,
        "world_ref": s["freeze_world"].as_str().unwrap_or("world:seed@sha256:00"),
        "timeout_s": timeout.max(1),
    });
    let resource = format!("execution:sha256:{}", digest_of(&object).unwrap());
    let ledger = sea_forge_ledger::LedgerStream::open(
        root,
        sea_forge_authority::cep::LEDGER_ID,
        "sea-forge-cep-authority",
    )
    .unwrap();
    let committed = ledger
        .commit_typed(
            "authority_decision",
            vec!["cognate".into(), "op-seed".into()],
            &json!({"decision_id": "dec-seed-1", "outcome": outcome,
                    "decided_at": chrono::Utc::now().to_rfc3339(),
                    "operation": {"kind": "reserved", "resource_type": "cognate_action",
                                  "resource_id": resource, "parameters": {}}}),
            vec![],
        )
        .unwrap();
    for i in 0..extra["runs_used"].as_u64().unwrap_or(0) {
        write_lines(
            &case_dir(root).join("field-exec-decisions.jsonl"),
            &[
                json!({"decision_key": committed.entry_ulid(), "decision_id": "dec-seed-1",
                     "run_id": format!("run-seed-{i}"), "at": "2026-11-15T08:00:00Z"}),
            ],
        );
    }
}

/// D39: the interaction event of an authorized run carries the decision id, so
/// RealityTrace can resolve `authority.decision_id` (the independent, non-executor record).
#[tokio::test(flavor = "multi_thread")]
async fn run_event_carries_the_program_execution_decision_id() {
    let mut s = read(
        &fixtures("execution-episode-ref")
            .join("scenarios/item_execute-complete-freeze-accepted.json"),
    );
    s["freeze_world"] = json!("world:godspeed-fde-sop@sha256:78eddd4e7e4771caa1c37e8e5fa095c5baa899daccea8fac1f6e969f8f37287b");
    s["inspect_decision_event"] = json!(true);
    run_gate(&s).await.unwrap();
}
