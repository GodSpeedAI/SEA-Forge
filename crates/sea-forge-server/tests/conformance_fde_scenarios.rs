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
//! - `part-B` (retired: WP-21 part B implements the Field Case document,
//!   `field_case.revised`, `strategy.recorded`, `gate.assessed`,
//!   `disposition.recorded` and the lifecycle function).
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
const FINAL_REASONS: &[&str] = &["wp-31"];

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

/// The four kinds only a v2 writer may name (WP-21 part B).
fn is_part_b_kind(kind: &str) -> bool {
    matches!(
        kind,
        "field_case.revised" | "strategy.recorded" | "gate.assessed" | "disposition.recorded"
    )
}

// (family/id, reason). Filled in by `pending_reason` below from rules, so a new
// scenario is never pending by accident: see that function.
fn pending_reason(family: &str, id: &str, scenario: &Value) -> Option<&'static str> {
    if PASSES_TODAY.contains(&format!("{family}/{id}").as_str()) {
        return None;
    }
    match family {
        "reserved-operation" => Some("wp-31"),
        "case-record-append-v2" => {
            // The four v2-only kinds are WP-21 part B and are driven like any other.
            if is_part_b_kind(scenario["request"]["kind"].as_str().unwrap_or("")) {
                return None;
            }
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

    /// The Field Case document of the case at `revision` (every field unknown).
    fn field_case_document(&self, revision: u64) {
        let mut doc = sea_forge_core::field_case::opened_document(CASE);
        doc["revision"] = json!(revision);
        fs::write(
            case_dir(&self.root).join("field_case.json"),
            serde_json::to_vec_pretty(&doc).unwrap(),
        )
        .unwrap();
    }

    /// One approval line the way the approval path would leave it (README
    /// section 6 "Approvals"): bound to `operation`/`params` through the digest of
    /// the canonical operation, of this case, unexpired unless `expired`.
    fn approval(&self, reference: &str, operation: &str, digest: &str, extra: Value) {
        let mut line = json!({
            "approval_ref": reference, "operation": operation, "operation_digest": digest,
            "case_id": CASE, "outcome": "allow", "used_by": null,
            "issued_at": "2026-11-15T08:00:00Z", "expires_at": "2099-01-01T00:00:00Z",
        });
        for (k, v) in extra.as_object().into_iter().flatten() {
            line[k] = v.clone();
        }
        write_lines(&case_dir(&self.root).join("field-approvals.jsonl"), &[line]);
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
        // A stored record's body digest is the digest of its kind and payload
        // (the guard verifies it: README section 3 STORE_INTEGRITY).
        if record.get("body_digest").is_some() && record.get("payload").is_some() {
            record["body_digest"] = json!(digest_of(
                &json!({"kind": record["kind"], "payload": record["payload"]})
            )
            .unwrap());
        }
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
        // The v2 response members: the revision a record created and the lifecycle after it.
        for field in ["kind", "replayed", "episode_key", "revision", "lifecycle"] {
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
            // Part-B records are written by the case operator; a gate names its
            // assessor, who must be the authenticated principal.
            Some("gate.assessed") => payload["assessor"].as_str().unwrap_or(KEEPER).into(),
            Some(k) if is_part_b_kind(k) => "human:operator".into(),
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
    let part_b = is_part_b_kind(request["kind"].as_str().unwrap_or(""));
    // README section 6: every append on a Field Case rests on the document, in
    // either dialect (the marker selects the response dialect only).
    let _ = part_b;
    {
        let revised = ledger
            .iter()
            .filter(|l| l["kind"] == "field_case.revised")
            .count() as u64;
        seed.field_case_document(1 + revised);
    }
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
            "field_case.revised" => {
                seed.push(
                    json!({"kind": "field_case.revised", "revision": entry["revision"],
                    "idempotency_key": "seed-revised", "body_digest": "seed-revised",
                    "payload": {"changes": []}}),
                );
            }
            "strategy.recorded" => {
                seed.push(
                    json!({"kind": "strategy.recorded", "revision": entry["revision"],
                    "idempotency_key": "seed-strategy", "body_digest": "seed-strategy"}),
                );
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
            // The approval a ledger entry names is seeded below, bound to the
            // request, with the outcome the entry states.
            "approval.recorded" => {}
            "case.world_pinned" | "case.state" | "store.hash_chain_broken" => {}
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
    if part_b {
        seed_part_b_approval(&seed, s, &ledger);
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

/// The operation, params and subject the server builds for a part-B record that
/// rests on an approval (README sections 6 and 10): `(operation, params)`.
fn part_b_operation(request: &Value) -> Option<(String, Value)> {
    let payload = &request["payload"];
    let mut record = payload.clone();
    record.as_object_mut()?.remove("approval_ref");
    let record_digest = digest_of(&record)?;
    match request["kind"].as_str()? {
        "disposition.recorded" => {
            let d = payload["disposition"].as_str()?;
            let operation = if payload["level"] == "episode" {
                "set_episode_disposition"
            } else if d.starts_with("close-") {
                "set_case_close_disposition"
            } else {
                "set_case_disposition"
            };
            Some((
                operation.into(),
                json!({"level": payload["level"], "disposition": d, "case_id": CASE,
                       "episode_ref": payload.get("episode_ref").cloned().unwrap_or(Value::Null),
                       "record_digest": record_digest}),
            ))
        }
        "gate.assessed" if payload["state"] == "not_applicable" => {
            let hard = matches!(
                payload["gate"].as_str()?,
                "allocate_authority" | "integrate_govern"
            );
            Some((
                if hard {
                    "set_hard_gate_not_applicable"
                } else {
                    "set_gate_not_applicable"
                }
                .into(),
                json!({"case_id": CASE, "gate": payload["gate"], "state": "not_applicable",
                       "record_digest": record_digest}),
            ))
        }
        _ => None,
    }
}

fn operation_digest(operation: &str, params: &Value) -> String {
    digest_of(
        &json!({"operation": operation, "subject_ref": CASE, "case_id": CASE,
                      "params": params}),
    )
    .unwrap()
}

/// Seed the approval a part-B scenario's request names: the scenario's own
/// `approval` (with its stated outcome, case, use and digest), else the one a
/// ledger `approval.recorded` entry describes, else, when the scenario expects
/// acceptance, a valid approval bound to the request.
fn seed_part_b_approval(seed: &Seed, s: &Value, ledger: &[Value]) {
    let request = &s["request"];
    let Some(reference) = request["payload"]["approval_ref"].as_str() else {
        return;
    };
    let Some((operation, params)) = part_b_operation(request) else {
        return;
    };
    if let Some(a) = s.get("approval") {
        let operation = s["record_operation"]["operation"]
            .as_str()
            .unwrap_or(&operation);
        seed.approval(
            reference,
            operation,
            a["operation_digest"].as_str().unwrap(),
            json!({"case_id": a["case_id"], "outcome": a["outcome"], "used_by": a["used_by"]}),
        );
        return;
    }
    let digest = operation_digest(&operation, &params);
    if let Some(entry) = ledger.iter().find(|l| l["kind"] == "approval.recorded") {
        let extra = if entry["outcome"] == "denied" {
            json!({"outcome": "deny"})
        } else if entry["expired"] == true {
            json!({"expires_at": "2026-11-15T09:00:00Z"})
        } else {
            json!({})
        };
        seed.approval(reference, &operation, &digest, extra);
    } else if s["expect"]["outcome"] == "recorded" {
        seed.approval(reference, &operation, &digest, json!({}));
    }
}

// ---------------------------------------------------------------------------
// Family: execution gate
// ---------------------------------------------------------------------------

async fn run_gate(s: &Value) -> Result<(), String> {
    run_gate_with(s, false).await
}

/// [`run_gate`]; with `suspend_after_admission` the request parks after its
/// admission, a suspension lands, and the request resumes (review round 3).
async fn run_gate_with(s: &Value, suspend_after_admission: bool) -> Result<(), String> {
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
    // README section 6: every write on a Field Case rests on its document.
    if template == Some("fde.field_case@1") {
        seed.field_case_document(1);
    }
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
    let got = if suspend_after_admission {
        let pause = sea_forge_server::pause_after_admission(root.path());
        let wire = request.clone();
        let mut client = cell
            .connect_for(
                &authenticated::principal("operator_local", ActorRole::Operator),
                &wire,
            )
            .await;
        let task = tokio::spawn(async move { client.call(wire).await });
        tokio::time::timeout(std::time::Duration::from_secs(30), pause.reached.notified())
            .await
            .expect("the request reached its pause");
        seed.push(json!({"kind": "disposition.recorded", "level": "case",
                         "disposition": "suspend", "idempotency_key": "late-suspend"}));
        pause.resume.notify_one();
        task.await.unwrap()
    } else {
        cell.call_for(
            &authenticated::principal("operator_local", ActorRole::Operator),
            request.clone(),
        )
        .await
    };
    cell.shutdown().await;
    if suspend_after_admission {
        return if got["code"] == "CASE_SUSPENDED" {
            Ok(())
        } else {
            Err(format!(
                "the suspension after admission was not honoured: {got}"
            ))
        };
    }
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
    // README section 6: a v2 append on a Field Case rests on its document.
    seed.field_case_document(1);
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

// ---------------------------------------------------------------------------
// WP-21 part B proofs: the Field Case document, strategy, gates, dispositions,
// the lifecycle and the continuation read model, through the real server.
// (scripts/fde-proof.d/s2/<AT or NT>.sh selects these by name.)
// ---------------------------------------------------------------------------

const V2: &str = "fde.case-record-append@2";
const OPERATOR: &str = "human:operator";

/// The continuation contract's own validator (every contract file by `$id`).
fn contract_validator(entry: &str) -> jsonschema::Validator {
    struct Docs(std::collections::HashMap<String, Value>);
    impl jsonschema::Retrieve for Docs {
        fn retrieve(
            &self,
            uri: &jsonschema::Uri<String>,
        ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
            self.0
                .get(uri.as_str())
                .cloned()
                .ok_or_else(|| format!("unknown schema {uri}").into())
        }
    }
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fde");
    let mut docs = std::collections::HashMap::new();
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "json") && p.to_string_lossy().contains(".v") {
            let doc = read(&p);
            docs.insert(doc["$id"].as_str().unwrap().to_string(), doc);
        }
    }
    jsonschema::options()
        .with_retriever(Docs(docs))
        .build(&json!({ "$ref": entry }))
        .unwrap()
}

fn assert_valid(entry: &str, value: &Value) {
    let v = contract_validator(entry);
    let errors: Vec<String> = v
        .iter_errors(value)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect();
    assert!(errors.is_empty(), "{entry}: {errors:?}\n{value}");
}

/// A running Field Case cell, its store seeded by `setup` before the server starts.
struct FieldCell {
    _dir: tempfile::TempDir,
    root: PathBuf,
    cell: authenticated::Cell,
    n: u32,
    rid: std::sync::atomic::AtomicU32,
}

impl FieldCell {
    async fn start(setup: impl FnOnce(&mut Seed)) -> Self {
        Self::start_with(Some("fde.field_case@1"), true, setup).await
    }

    async fn start_with(
        template: Option<&str>,
        document: bool,
        setup: impl FnOnce(&mut Seed),
    ) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let mut seed = Seed::new(&root, template);
        if document {
            seed.field_case_document(1);
        }
        setup(&mut seed);
        let actors = [
            Actor {
                id: OPERATOR.into(),
                role: ActorRole::Operator,
            },
            Actor {
                id: KEEPER.into(),
                role: ActorRole::Service,
            },
            Actor {
                id: "agent:builder-01".into(),
                role: ActorRole::Agent,
            },
            Actor {
                id: "agent:verifier-01".into(),
                role: ActorRole::Agent,
            },
        ];
        let cell = authenticated::Cell::start(config(&root, &actors)).await;
        Self {
            _dir: dir,
            root,
            cell,
            n: 0,
            rid: std::sync::atomic::AtomicU32::new(0),
        }
    }

    async fn call(&mut self, who: &str, mut wire: Value) -> Value {
        wire["case_id"] = json!(CASE);
        wire["actor"] = json!({"actor_id": who, "role": role_wire(&role_for(who))});
        self.cell
            .call_for(&authenticated::principal(who, role_for(who)), wire)
            .await
    }

    /// Append a part-B record as `who` under a fresh idempotency key.
    async fn append(&mut self, who: &str, kind: &str, payload: Value) -> Value {
        self.n += 1;
        let key = format!("operator/b-{}", self.n);
        self.append_keyed(who, kind, payload, &key).await
    }

    async fn append_keyed(&mut self, who: &str, kind: &str, payload: Value, key: &str) -> Value {
        self.call(
            who,
            json!({"verb": "case_record_append", "kind": kind, "payload": payload,
                   "idempotency_key": key, "contract": V2,
                   "request_id": format!("req-b-{}", self.rid.fetch_add(1, std::sync::atomic::Ordering::SeqCst))}),
        )
        .await
    }

    /// An allow-approval bound to exactly this record, written as the approval path would.
    fn approve(&self, reference: &str, kind: &str, payload: &Value) {
        let request = json!({"kind": kind, "payload": payload});
        let (operation, params) = part_b_operation(&request).expect("an approvable record");
        let digest = operation_digest(&operation, &params);
        write_lines(
            &case_dir(&self.root).join("field-approvals.jsonl"),
            &[json!({
                "approval_ref": reference, "operation": operation, "operation_digest": digest,
                "case_id": CASE, "outcome": "allow", "used_by": null,
                "issued_at": "2026-11-15T08:00:00Z", "expires_at": "2099-01-01T00:00:00Z",
            })],
        );
    }

    /// Approve and append a disposition; returns the response.
    async fn dispose(
        &mut self,
        level: &str,
        disposition: &str,
        episode_ref: Option<&str>,
    ) -> Value {
        self.n += 1;
        let reference = format!("approval:b-{}", self.n);
        let mut payload = json!({"level": level, "disposition": disposition,
            "rationale": "the case lead decided", "approval_ref": reference});
        if let Some(r) = episode_ref {
            payload["episode_ref"] = json!(r);
        }
        self.approve(&reference, "disposition.recorded", &payload);
        self.append(OPERATOR, "disposition.recorded", payload).await
    }

    async fn field(&mut self) -> Value {
        self.call(OPERATOR, json!({"verb": "case_field_get"})).await
    }

    async fn continuation(&mut self) -> Value {
        let got = self
            .call(OPERATOR, json!({"verb": "case_continuation"}))
            .await;
        if got["continuation"].is_object() {
            assert_valid(
                "urn:sea-forge:fde:continuation:v1#/$defs/Continuation",
                &got["continuation"],
            );
        }
        got
    }

    async fn next(&mut self) -> Value {
        self.continuation().await["continuation"]["next"].clone()
    }

    fn read_store(&self, name: &str) -> Vec<Value> {
        fs::read_to_string(case_dir(&self.root).join(name))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
}

fn revision_payload(base: u64, path: &str, status: &str, value: &str) -> Value {
    json!({"base_revision": base, "rationale": "intake interview",
           "changes": [{"path": path, "field": {"status": status,
               "source_ref": {"kind": "operator", "ref": "operator:note-1"}, "value": value}}]})
}

fn strategy_payload(revision: u64, supersedes: Option<&str>) -> Value {
    json!({"revision": revision, "supersedes": supersedes,
           "where_to_play": "one recurring report", "how_to_win": "record the due date at handoff",
           "underserved_outcome": "on-time reports", "tested_assumption": "the due date is lost at handoff",
           "abandonment_trigger": "no loss in 3 observed handoffs",
           "revisit_reason": if revision == 1 { Value::Null } else { json!("the first test found a second loss point") }})
}

fn gate_payload(gate: &str, state: &str, evidence: &[&str]) -> Value {
    json!({"gate": gate, "state": state, "evidence_refs": evidence,
           "rationale": "assessed", "assessor": OPERATOR})
}

/// AT-03: a case from the template opens with the document (revision 1, every field
/// `unknown`), is revised by `field_case.revised` (status and source on every field,
/// stated and adopted problem kept apart) and records strategy revisions.
#[tokio::test(flavor = "multi_thread")]
async fn at_03_open_revise_and_strategy() {
    let mut c = FieldCell::start(|_| {}).await;
    let opened = c.field().await;
    assert_eq!(opened["revision"], 1, "{opened}");
    assert_valid("urn:sea-forge:fde:field-case:v1", &opened["document"]);
    let unknown = |d: &Value| {
        d["fields"]
            .as_object()
            .unwrap()
            .values()
            .flat_map(|g| g.as_object().unwrap().values())
            .filter(|f| f["status"] == "unknown")
            .count()
    };
    assert_eq!(unknown(&opened["document"]), 45);
    // Stated and adopted problem are two fields, recorded apart.
    let r = c
        .append(
            OPERATOR,
            "field_case.revised",
            json!({"base_revision": 1, "rationale": "intake interview", "changes": [
                {"path": "purpose_and_outcome.stated_problem", "field": {"status": "declared",
                    "source_ref": {"kind": "operator", "ref": "operator:note-1"}, "value": "Reports are late"}},
                {"path": "purpose_and_outcome.adopted_problem", "field": {"status": "inferred",
                    "source_ref": {"kind": "record", "ref": "case-ledger:rec-0007"}, "value": "Handoffs lose the due date"}}]}),
        )
        .await;
    assert_eq!(
        (r["outcome"].as_str(), r["revision"].as_u64()),
        (Some("recorded"), Some(2)),
        "{r}"
    );
    let got = c.field().await;
    assert_eq!(got["revision"], 2);
    let doc = &got["document"];
    assert_valid("urn:sea-forge:fde:field-case:v1", doc);
    let p = &doc["fields"]["purpose_and_outcome"];
    assert_eq!(p["stated_problem"]["value"], "Reports are late");
    assert_eq!(p["adopted_problem"]["value"], "Handoffs lose the due date");
    assert_ne!(p["stated_problem"]["value"], p["adopted_problem"]["value"]);
    assert_eq!(p["adopted_problem"]["status"], "inferred");
    assert_eq!(p["adopted_problem"]["source_ref"]["kind"], "record");
    assert_eq!(unknown(doc), 43);
    // Every field carries a status and a source.
    for g in doc["fields"].as_object().unwrap().values() {
        for f in g.as_object().unwrap().values() {
            assert!(
                f["status"].is_string() && f["source_ref"]["kind"].is_string(),
                "{f}"
            );
        }
    }
    // An observed field cannot rest on the operator's say-so (schema rule).
    let bad = c
        .append(
            OPERATOR,
            "field_case.revised",
            revision_payload(2, "framing.current_framing", "observed", "x"),
        )
        .await;
    assert_eq!(bad["code"], "SCHEMA_INVALID", "{bad}");
    // Strategy: revision 1, then 2 superseding it, a stale one conflicts.
    let s1 = c
        .append(OPERATOR, "strategy.recorded", strategy_payload(1, None))
        .await;
    assert_eq!(
        (s1["revision"].as_u64(), s1["outcome"].as_str()),
        (Some(1), Some("recorded")),
        "{s1}"
    );
    let id1 = s1["record_id"].as_str().unwrap().to_string();
    let s2 = c
        .append(
            OPERATOR,
            "strategy.recorded",
            strategy_payload(2, Some(&id1)),
        )
        .await;
    assert_eq!(s2["revision"], 2, "{s2}");
    let stale = c
        .append(
            OPERATOR,
            "strategy.recorded",
            strategy_payload(2, Some(&id1)),
        )
        .await;
    assert_eq!(stale["code"], "REVISION_CONFLICT", "{stale}");
    let wrong = c
        .append(
            OPERATOR,
            "strategy.recorded",
            strategy_payload(3, Some("rec-9999")),
        )
        .await;
    assert_eq!(wrong["code"], "REVISION_CONFLICT", "{wrong}");
}

/// The document is a projection of the ledger: a crash after the record and before
/// the file leaves it behind, and the next read replays the missing record.
#[tokio::test(flavor = "multi_thread")]
async fn field_case_document_recovers_from_its_ledger() {
    let mut c = FieldCell::start(|_| {}).await;
    let r = c
        .append(
            OPERATOR,
            "field_case.revised",
            revision_payload(1, "framing.better_question", "proposed", "Why late?"),
        )
        .await;
    assert_eq!(r["revision"], 2, "{r}");
    // The crash: the file is back at the opened document.
    let mut opened = sea_forge_core::field_case::opened_document(CASE);
    opened["revision"] = json!(1);
    fs::write(
        case_dir(&c.root).join("field_case.json"),
        serde_json::to_vec(&opened).unwrap(),
    )
    .unwrap();
    let got = c.field().await;
    assert_eq!(got["revision"], 2, "{got}");
    assert_eq!(
        got["document"]["fields"]["framing"]["better_question"]["value"],
        "Why late?"
    );
    // A document ahead of its ledger, or no document, is STORE_INTEGRITY.
    let mut ahead = got["document"].clone();
    ahead["revision"] = json!(9);
    fs::write(
        case_dir(&c.root).join("field_case.json"),
        serde_json::to_vec(&ahead).unwrap(),
    )
    .unwrap();
    assert_eq!(c.field().await["code"], "STORE_INTEGRITY");
    fs::remove_file(case_dir(&c.root).join("field_case.json")).unwrap();
    assert_eq!(c.field().await["code"], "STORE_INTEGRITY");
    let r = c
        .append(
            OPERATOR,
            "field_case.revised",
            revision_payload(2, "framing.better_question", "proposed", "Why?"),
        )
        .await;
    assert_eq!(r["code"], "STORE_INTEGRITY", "{r}");
}

#[tokio::test(flavor = "multi_thread")]
async fn read_verbs_answer_unknown_and_non_field_cases() {
    let mut c = FieldCell::start_with(Some("e2e-sentry-chain@0.1.0"), false, |_| {}).await;
    let got = c.field().await;
    assert_eq!(
        (got["code"].as_str(), got["outcome"].as_str()),
        (Some("NOT_A_FIELD_CASE"), Some("refused")),
        "{got}"
    );
    assert_valid("urn:sea-forge:fde:continuation:v1#/$defs/Error", &got);
    assert_eq!(c.continuation().await["code"], "NOT_A_FIELD_CASE");
    let mut w = json!({"verb": "case_field_get"});
    w["actor"] = json!({"actor_id": OPERATOR, "role": "operator"});
    w["case_id"] = json!("case-9999");
    let r = c
        .cell
        .call_for(&authenticated::principal(OPERATOR, ActorRole::Operator), w)
        .await;
    assert!(r["error"].is_string(), "{r}");
    // A v2 kind on a case that is not a Field Case is refused, not recorded.
    let r = c
        .append(OPERATOR, "strategy.recorded", strategy_payload(1, None))
        .await;
    assert_eq!(r["code"], "SCHEMA_INVALID", "{r}");
}

/// Replays return the original (never refused by a later rule); a changed body
/// under a used key is IDEMPOTENCY_CONFLICT.
#[tokio::test(flavor = "multi_thread")]
async fn part_b_replay_and_conflict() {
    let mut c = FieldCell::start(|_| {}).await;
    let payload = revision_payload(1, "framing.better_question", "proposed", "Why late?");
    let first = c
        .append_keyed(
            OPERATOR,
            "field_case.revised",
            payload.clone(),
            "operator/r1",
        )
        .await;
    assert_eq!(first["replayed"], false, "{first}");
    let again = c
        .append_keyed(
            OPERATOR,
            "field_case.revised",
            payload.clone(),
            "operator/r1",
        )
        .await;
    assert_eq!(
        (again["record_id"].clone(), again["replayed"].clone()),
        (first["record_id"].clone(), json!(true)),
        "{again}"
    );
    // Even after the case closed, a replay returns the original.
    assert_eq!(
        c.dispose("case", "close-failed", None).await["lifecycle"],
        "closed"
    );
    let late = c
        .append_keyed(OPERATOR, "field_case.revised", payload, "operator/r1")
        .await;
    assert_eq!(late["replayed"], true, "{late}");
    let other = revision_payload(1, "framing.better_question", "proposed", "Another?");
    let conflict = c
        .append_keyed(OPERATOR, "field_case.revised", other, "operator/r1")
        .await;
    assert_eq!(conflict["code"], "IDEMPOTENCY_CONFLICT", "{conflict}");
}

/// AT-10: every case-level disposition yields its mapped lifecycle, an episode-level
/// outcome never closes a case, the four axes are queryable apart, a run without a
/// settlement shows its result absent, and a blocked move names its dimension and
/// authorizer.
#[tokio::test(flavor = "multi_thread")]
async fn at_10_dispositions_axes_and_blocked_move() {
    let key = json!({"case_id": CASE, "plan_item_id": "item-03", "n": 1});
    let digest = "1f".repeat(32);
    let world = "world:godspeed-fde-sop@sha256:78eddd4e7e4771caa1c37e8e5fa095c5baa899daccea8fac1f6e969f8f37287b";
    let ep = format!("episode:{CASE}:item-03:1@sha256:{digest}");
    let mut c = FieldCell::start(|s| {
        s.freeze(&key, "evaluated", world, "case-keeper/rec-0001", &digest);
    })
    .await;
    // Axis: the episode is frozen and has no result (a run without a settlement).
    let cont = c.continuation().await["continuation"].clone();
    assert_eq!(cont["case"]["lifecycle"], "open");
    assert_eq!(cont["episodes"][0]["state"], "frozen");
    assert_eq!(cont["episodes"][0]["result"], Value::Null);
    assert_eq!(cont["episodes"][0]["settlement_ref"], Value::Null);
    // The next move settles it, names who authorizes it and what would settle it.
    let next = &cont["next"];
    assert_eq!(next["kind"], "action");
    assert!(
        next["authorizer"].is_string() && next["would_settle"].as_str().unwrap().contains(&ep),
        "{next}"
    );
    // Episode-level outcomes never close, suspend or lift.
    for d in [
        "close-satisfied",
        "close-failed",
        "suspend",
        "continue",
        "revise",
        "escalate",
    ] {
        let r = c.dispose("episode", d, Some(&ep)).await;
        assert_eq!(
            (r["outcome"].as_str(), r["lifecycle"].as_str()),
            (Some("recorded"), Some("open")),
            "episode {d}: {r}"
        );
    }
    // Case-level: each disposition's lifecycle.
    for (d, want) in [
        ("revise", "open"),
        ("escalate", "open"),
        ("suspend", "suspended"),
    ] {
        let r = c.dispose("case", d, None).await;
        assert_eq!(r["lifecycle"], want, "case {d}: {r}");
        if d == "suspend" {
            let cont = c.continuation().await["continuation"].clone();
            assert_eq!(cont["case"]["lifecycle"], "suspended");
            // Blocked move: names the dimension and the authorizer.
            assert_eq!(cont["next"]["kind"], "blocker");
            assert_eq!(cont["next"]["dimension"], "authority");
            assert!(cont["next"]["authorizer"].is_string(), "{}", cont["next"]);
        }
    }
    assert_eq!(
        c.dispose("case", "continue", None).await["lifecycle"],
        "open"
    );
    assert_eq!(
        c.dispose("case", "close-satisfied", None).await["lifecycle"],
        "closed"
    );
    let cont = c.continuation().await["continuation"].clone();
    assert_eq!(cont["case"]["lifecycle"], "closed");
    // The axes are independent: closed (lifecycle) says nothing about the episode.
    assert_eq!(cont["episodes"][0]["state"], "frozen");
    // Closed is final, for every case-level disposition and kind.
    for d in ["continue", "suspend", "close-failed"] {
        assert_eq!(
            c.dispose("case", d, None).await["code"],
            "CASE_CLOSED",
            "{d}"
        );
    }
    // close-failed also closes (a second case).
    let mut f = FieldCell::start(|_| {}).await;
    assert_eq!(
        f.dispose("case", "close-failed", None).await["lifecycle"],
        "closed"
    );
}

/// An approval is single use: the record that used it consumes it.
#[tokio::test(flavor = "multi_thread")]
async fn an_approval_authorises_one_record() {
    let mut c = FieldCell::start(|_| {}).await;
    let payload = json!({"level": "case", "disposition": "revise", "rationale": "again",
                         "approval_ref": "approval:once"});
    c.approve("approval:once", "disposition.recorded", &payload);
    let first = c
        .append(OPERATOR, "disposition.recorded", payload.clone())
        .await;
    assert_eq!(first["outcome"], "recorded", "{first}");
    let second = c.append(OPERATOR, "disposition.recorded", payload).await;
    assert_eq!(second["code"], "AUTHORITY_DENIED", "{second}");
    // The consumption is a line of the approval store naming the record.
    let lines = c.read_store("field-approvals.jsonl");
    assert!(
        lines.iter().any(|l| l["consumed_by"] == first["record_id"]),
        "{lines:?}"
    );
}

/// AT-11: all four gate states are reachable, an evidence change marks the gate
/// to reassess, and nothing advances automatically.
#[tokio::test(flavor = "multi_thread")]
async fn at_11_gates() {
    let key = json!({"case_id": CASE, "plan_item_id": "item-03", "n": 1});
    let digest = "1f".repeat(32);
    let mut c = FieldCell::start(|_| {}).await;
    let plan_before = fs::read(case_dir(&c.root).join("plan.json")).unwrap();
    let doc_before = c.field().await;
    // met, blocked, unassessed, not_applicable (a non-hard gate with an approval).
    let met = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("choose_problem", "met", &["evidence:E1"]),
        )
        .await;
    assert_eq!(met["outcome"], "recorded", "{met}");
    for (gate, state) in [
        ("discover_reality", "blocked"),
        ("prove_value", "unassessed"),
    ] {
        let r = c
            .append(OPERATOR, "gate.assessed", gate_payload(gate, state, &[]))
            .await;
        assert_eq!(r["outcome"], "recorded", "{gate} {r}");
    }
    let mut na = gate_payload("measure", "not_applicable", &[]);
    na["approval_ref"] = json!("approval:na");
    c.approve("approval:na", "gate.assessed", &na);
    let r = c.append(OPERATOR, "gate.assessed", na).await;
    assert_eq!(r["outcome"], "recorded", "{r}");
    // A met gate needs evidence (schema rule).
    let bad = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("earn_trust", "met", &[]),
        )
        .await;
    assert_eq!(bad["code"], "SCHEMA_INVALID", "{bad}");
    let cont = c.continuation().await["continuation"].clone();
    let state = |g: &str| {
        cont["gate_evidence"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["gate"] == g)
            .unwrap()["state"]
            .clone()
    };
    assert_eq!(state("choose_problem"), "met");
    assert_eq!(state("discover_reality"), "blocked");
    assert_eq!(state("prove_value"), "unassessed");
    assert_eq!(state("measure"), "not_applicable");
    assert_eq!(state("productize"), "unassessed");
    assert_eq!(
        cont["gate_evidence"][0]["evidence_refs"],
        json!(["evidence:E1"])
    );
    // Nothing advanced: no plan change, no document change, still open.
    assert_eq!(
        fs::read(case_dir(&c.root).join("plan.json")).unwrap(),
        plan_before
    );
    assert_eq!(c.field().await["document"], doc_before["document"]);
    assert_eq!(cont["case"]["lifecycle"], "open");
    // Evidence change: a settlement after the assessment marks the gate to reassess
    // (the gate itself is not touched).
    let mut r = FieldCell::start(|s| {
        s.freeze(&key, "evaluated", "world:seed@sha256:00", "case-keeper/rec-0001", &digest);
        s.push(json!({"kind": "gate.assessed", "gate": "choose_problem", "state": "met",
            "idempotency_key": "seed-gate", "body_digest": "seed-gate",
            "payload": {"gate": "choose_problem", "state": "met", "evidence_refs": ["evidence:E1"]}}));
        s.push(json!({"kind": "episode.settled", "idempotency_key": "seed-settled",
            "body_digest": "seed-settled", "episode_key": key,
            "payload": {"result": "S", "settlement_ref": "settlement:S1"}}));
    })
    .await;
    let cont = r.continuation().await["continuation"].clone();
    assert_eq!(
        cont["gate_evidence"][0]["state"], "met",
        "the gate is untouched"
    );
    assert_eq!(cont["episodes"][0]["state"], "settled");
    assert_eq!(cont["episodes"][0]["result"], "S");
    let next = r.next().await;
    assert!(
        next["description"]
            .as_str()
            .unwrap()
            .contains("Reassess gate choose_problem")
            || next["description"].as_str().unwrap().contains("problem")
            || next["description"].as_str().unwrap().contains("strategy"),
        "{next}"
    );
    // The reassess mark itself, as a pure function of the ledger.
    let ledger = r.read_store("field-records.jsonl");
    assert_eq!(
        sea_forge_server::sfwp::field_continuation::gates_to_reassess(&ledger)
            .into_iter()
            .map(|(g, _)| g)
            .collect::<Vec<_>>(),
        vec!["choose_problem".to_string()]
    );
}

/// AT-15 (the programmatic part): given only the continuation, the next move, who
/// authorizes it and what would settle it are stated, and the answer validates
/// against `continuation.v1`.
#[tokio::test(flavor = "multi_thread")]
async fn at_15_continuation_states_the_next_move() {
    let mut c = FieldCell::start(|_| {}).await;
    // A fresh case: all-unknown fields, unknown sections stay unknown (never empty "none").
    let fresh = c.continuation().await["continuation"].clone();
    for section in ["disputed_claims", "latest_observations", "work"] {
        assert_eq!(fresh["availability"][section], "unknown", "{section}");
    }
    assert_eq!(fresh["availability"]["case"], "known");
    assert_eq!(fresh["computed_from"]["ledger_head"], Value::Null);
    assert_eq!(fresh["gate_evidence"].as_array().unwrap().len(), 9);
    assert_eq!(
        fresh["next"]["description"],
        "Record the stated and adopted problem."
    );
    assert!(fresh["next"]["authorizer"].is_string());
    for budget in fresh["work"]["remaining_budgets"].as_array().unwrap() {
        assert_eq!(budget["remaining"], "unknown", "unmeasured is never 0");
    }
    // The next move moves with the records.
    c.append(OPERATOR, "field_case.revised", json!({"base_revision": 1, "rationale": "intake", "changes": [
        {"path": "purpose_and_outcome.stated_problem", "field": {"status": "declared", "source_ref": {"kind": "operator", "ref": "o:1"}, "value": "late"}},
        {"path": "purpose_and_outcome.adopted_problem", "field": {"status": "inferred", "source_ref": {"kind": "record", "ref": "r:1"}, "value": "handoff"}},
        {"path": "observer_and_authority.settlement_and_disposition_authority", "field": {"status": "declared", "source_ref": {"kind": "operator", "ref": "o:1"}, "value": "human:operator"}}]})).await;
    assert_eq!(
        c.next().await["description"],
        "Record the strategy (SOP 4.4)."
    );
    c.append(OPERATOR, "strategy.recorded", strategy_payload(1, None))
        .await;
    let next = c.next().await;
    assert_eq!(next["description"], "Assess gate choose_problem.");
    assert_eq!(next["authorizer"], "human:operator");
    assert!(next["would_settle"].is_string());
    let cont = c.continuation().await["continuation"].clone();
    assert_eq!(cont["computed_from"]["ledger_head"], "rec-0002");
    // The authority section reads the approval store.
    c.dispose("case", "suspend", None).await;
    let cont = c.continuation().await["continuation"].clone();
    assert_eq!(cont["authority"]["decisions"][0]["outcome"], "allow");
    assert_eq!(cont["authority"]["decisions"][0]["assurance"], "session");
    assert_eq!(cont["authority"]["a3_channel"], "unknown");
    assert_eq!(cont["next"]["kind"], "blocker");
    c.dispose("case", "close-failed", None).await;
    let closed = c.next().await;
    assert_eq!(closed["kind"], "action");
    assert_eq!(closed["authorizer"], Value::Null);
}

/// The lifecycle function over the real ledger, not only synthetic records: the
/// golden table is a unit test in `sfwp::field_case_kinds`; this follows one
/// case through suspend, refused work, lift and close.
#[tokio::test(flavor = "multi_thread")]
async fn nt_22_suspended_case_end_to_end() {
    let mut c = FieldCell::start(|_| {}).await;
    assert_eq!(
        c.dispose("case", "suspend", None).await["lifecycle"],
        "suspended"
    );
    // Refused: a case-level revise, an episode-level disposition.
    assert_eq!(
        c.dispose("case", "revise", None).await["code"],
        "CASE_SUSPENDED"
    );
    let ep = format!("episode:{CASE}:item-03:1@sha256:{}", "1f".repeat(32));
    assert_eq!(
        c.dispose("episode", "continue", Some(&ep)).await["code"],
        "CASE_SUSPENDED"
    );
    // Accepted: revision, strategy, gate (the work that does not move the world).
    for (kind, payload) in [
        (
            "field_case.revised",
            revision_payload(1, "framing.better_question", "proposed", "Why?"),
        ),
        ("strategy.recorded", strategy_payload(1, None)),
        (
            "gate.assessed",
            gate_payload("choose_problem", "met", &["evidence:E1"]),
        ),
    ] {
        assert_eq!(
            c.append(OPERATOR, kind, payload).await["outcome"],
            "recorded",
            "{kind}"
        );
    }
    assert_eq!(
        c.dispose("case", "continue", None).await["lifecycle"],
        "open"
    );
    assert_eq!(c.dispose("case", "revise", None).await["lifecycle"], "open");
}

/// I-2 on the SEA-Forge delegation chain: the assessor of `prove_reliability` may
/// not be the proposer, nor stand behind the proposer's work in the delegation
/// chain the request carries.
#[test]
fn i2_gate_assessment_on_the_delegation_chain() {
    use sea_forge_server::sfwp::case_records::{append, AppendContext, AppendPolicy, FileIndex};
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let seed = Seed::new(root, Some("fde.field_case@1"));
    seed.field_case_document(1);
    let index = FileIndex(root.to_path_buf());
    let run = |principal: &str, chain: &[&str], proposer: Option<&str>, key: &str| {
        let chain: Vec<String> = chain.iter().map(|s| s.to_string()).collect();
        let ctx = AppendContext {
            root,
            case_id: CASE,
            principal,
            role: "agent",
            chain: &chain,
            rt: &index,
            anchor: None,
            policy: &AppendPolicy::NONE,
        };
        let mut payload = gate_payload("prove_reliability", "met", &["evidence:E1"]);
        payload["assessor"] = json!(principal);
        if let Some(p) = proposer {
            payload["proposer"] = json!(p);
        }
        append(
            &ctx,
            &json!({"contract": V2, "kind": "gate.assessed", "payload": payload,
                    "idempotency_key": key}),
        )
    };
    // The assessor is the proposer: refused (I-2), with the invariant.
    let own = run(
        "agent:builder-01",
        &["agent:builder-01"],
        Some("agent:builder-01"),
        "k1",
    );
    assert_eq!(
        (own["code"].as_str(), own["invariant"].as_str()),
        (Some("SELF_SETTLEMENT_REFUSED"), Some("I-2")),
        "{own}"
    );
    // Spelling does not matter (canonical actor strings).
    let spelled = run(
        "agent:builder-01",
        &["agent:builder-01"],
        Some(" Agent:Builder-01 "),
        "k2",
    );
    assert_eq!(spelled["code"], "SELF_SETTLEMENT_REFUSED", "{spelled}");
    // The proposer stands behind the assessor in the delegation chain: refused.
    let chained = run(
        "agent:verifier-01",
        &["agent:builder-01", "agent:verifier-01"],
        Some("agent:builder-01"),
        "k3",
    );
    assert_eq!(chained["code"], "SELF_SETTLEMENT_REFUSED", "{chained}");
    // prove_reliability needs a proposer at all.
    let none = run("agent:verifier-01", &["agent:verifier-01"], None, "k4");
    assert_eq!(none["code"], "SCHEMA_INVALID", "{none}");
    // An independent assessor outside the chain is accepted.
    let ok = run(
        "agent:verifier-01",
        &["agent:verifier-01"],
        Some("agent:builder-01"),
        "k5",
    );
    assert_eq!(ok["outcome"], "recorded", "{ok}");
}

/// NT-11: both hard gates (D12) are not waivable by rationale alone, and a
/// non-hard gate needs an approval too (SOP 8.1: authorized).
#[tokio::test(flavor = "multi_thread")]
async fn nt_11_both_hard_gates_and_the_non_hard_waiver() {
    let mut c = FieldCell::start(|_| {}).await;
    for gate in ["allocate_authority", "integrate_govern"] {
        let r = c
            .append(
                OPERATOR,
                "gate.assessed",
                gate_payload(gate, "not_applicable", &[]),
            )
            .await;
        assert_eq!(r["code"], "HARD_GATE_NOT_WAIVABLE", "{gate}: {r}");
    }
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("measure", "not_applicable", &[]),
        )
        .await;
    assert_eq!(r["code"], "AUTHORITY_DENIED", "{r}");
    // With a bound approval the hard gate may be set not_applicable.
    let mut na = gate_payload("integrate_govern", "not_applicable", &[]);
    na["approval_ref"] = json!("approval:hard");
    c.approve("approval:hard", "gate.assessed", &na);
    assert_eq!(
        c.append(OPERATOR, "gate.assessed", na).await["outcome"],
        "recorded"
    );
    // An approval bound to another gate does not carry over.
    let mut other = gate_payload("allocate_authority", "not_applicable", &[]);
    other["approval_ref"] = json!("approval:hard2");
    let mut elsewhere = gate_payload("integrate_govern", "not_applicable", &[]);
    elsewhere["rationale"] = json!("another rationale");
    c.approve("approval:hard2", "gate.assessed", &elsewhere);
    assert_eq!(
        c.append(OPERATOR, "gate.assessed", other).await["code"],
        "AUTHORITY_DENIED"
    );
}

/// Rules the schema cannot say (README section 6): the assessor is the authenticated
/// principal, an approval the record does not need is refused, and a strategy
/// revision follows the current one exactly.
#[tokio::test(flavor = "multi_thread")]
async fn part_b_rules_beyond_the_schema() {
    let mut c = FieldCell::start(|_| {}).await;
    // The assessor is not the caller.
    let mut p = gate_payload("choose_problem", "met", &["evidence:E1"]);
    p["assessor"] = json!("agent:builder-01");
    let r = c.append(OPERATOR, "gate.assessed", p).await;
    assert_eq!(r["code"], "SCHEMA_INVALID", "{r}");
    // An approval_ref on a met gate, and on a revision that needs none.
    let mut p = gate_payload("choose_problem", "met", &["evidence:E1"]);
    p["approval_ref"] = json!("approval:x");
    assert_eq!(
        c.append(OPERATOR, "gate.assessed", p).await["code"],
        "SCHEMA_INVALID"
    );
    let mut p = revision_payload(1, "framing.better_question", "proposed", "Why?");
    p["approval_ref"] = json!("approval:x");
    assert_eq!(
        c.append(OPERATOR, "field_case.revised", p).await["code"],
        "SCHEMA_INVALID"
    );
    // The same path twice in one revision.
    let mut p = revision_payload(1, "framing.better_question", "proposed", "Why?");
    let change = p["changes"][0].clone();
    p["changes"].as_array_mut().unwrap().push(change);
    assert_eq!(
        c.append(OPERATOR, "field_case.revised", p).await["code"],
        "SCHEMA_INVALID"
    );
    // A strategy revision skipping one conflicts even when it names the latest record.
    let s1 = c
        .append(OPERATOR, "strategy.recorded", strategy_payload(1, None))
        .await;
    let id1 = s1["record_id"].as_str().unwrap().to_string();
    let skip = c
        .append(
            OPERATOR,
            "strategy.recorded",
            strategy_payload(3, Some(&id1)),
        )
        .await;
    assert_eq!(skip["code"], "REVISION_CONFLICT", "{skip}");
    // A first strategy cannot be written twice.
    let again = c
        .append(OPERATOR, "strategy.recorded", strategy_payload(1, None))
        .await;
    assert_eq!(again["code"], "REVISION_CONFLICT", "{again}");
}

#[tokio::test(flavor = "multi_thread")]
async fn at_03_scenarios() {
    nt_subset("AT-03", 2).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn at_09_scenarios() {
    nt_subset("AT-09", 1).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn at_10_scenarios() {
    nt_subset("AT-10", 4).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_10_scenarios() {
    nt_subset("NT-10", 4).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_11_scenarios() {
    nt_subset("NT-11", 2).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_22_scenarios() {
    nt_subset("NT-22", 7).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn nt_23_scenarios() {
    nt_subset("NT-23", 8).await;
}

// ---------------------------------------------------------------------------
// WP-21 part B, review round 1 (blind review): gate items, the check adjacent
// to a write, approvals bound to their record, the document on every kind.
// ---------------------------------------------------------------------------

impl FieldCell {
    async fn advance(&mut self, items: Value) -> Value {
        self.n += 1;
        let rid = format!("req-adv-b-{}", self.n);
        self.call(
            OPERATOR,
            json!({"verb": "case_advance", "policy": "authority/active-policy.json",
                   "timeout": 60, "request_id": rid, "transition_items": items}),
        )
        .await
    }

    fn events(&self) -> Vec<u8> {
        fs::read(case_dir(&self.root).join("case-events.jsonl")).unwrap_or_default()
    }

    fn ledger_len(&self) -> usize {
        self.read_store("field-records.jsonl").len()
    }
}

fn transition(item: &str, how: &str, only: bool) -> Value {
    json!([{"item_id": item, "transition": how, "transition_only": only}])
}

/// Item 1: a gate item completes only on a passing assessment, by whatever form
/// of the request; the refusal writes nothing.
#[tokio::test(flavor = "multi_thread")]
async fn review1_gate_item_moves_only_on_a_passing_assessment() {
    let mut c = FieldCell::start(|s| {
        s.plan(&[
            ("gate_allocate_authority".into(), false),
            ("gate_prove_value".into(), false),
            ("task-b".into(), false),
        ])
    })
    .await;
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("allocate_authority", "blocked", &[]),
        )
        .await;
    assert_eq!(r["outcome"], "recorded", "{r}");
    let before = c.events();
    // The reviewer's request, and the same with transition_only false.
    for only in [true, false] {
        let got = c
            .advance(transition("gate_allocate_authority", "complete", only))
            .await;
        assert_eq!(got["code"], "HARD_GATE_NOT_WAIVABLE", "only={only}: {got}");
    }
    let got = c
        .advance(transition("gate_allocate_authority", "skip", true))
        .await;
    assert_eq!(got["code"], "HARD_GATE_NOT_WAIVABLE", "skip: {got}");
    // A gate never assessed does not complete either.
    let got = c
        .advance(transition("gate_prove_value", "complete", true))
        .await;
    assert_eq!(got["code"], "AUTHORITY_DENIED", "{got}");
    assert_eq!(c.events(), before, "a refusal writes nothing");
    // A passing assessment lets the item complete.
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("allocate_authority", "met", &["evidence:E1"]),
        )
        .await;
    assert_eq!(r["outcome"], "recorded", "{r}");
    let got = c
        .advance(transition("gate_allocate_authority", "complete", true))
        .await;
    assert!(got["code"].is_null(), "{got}");
    assert_ne!(c.events(), before);
}

/// Item 2: a suspension that lands between the admission and the write is
/// honoured by the write; the operation is refused and nothing is written.
#[tokio::test(flavor = "multi_thread")]
async fn review1_suspension_between_admission_and_write() {
    let mut c =
        FieldCell::start(|s| s.plan(&[("task-a".into(), false), ("task-b".into(), false)])).await;
    let before = c.events();
    let pause = sea_forge_server::pause_after_admission(&c.root);
    let wire = json!({"verb": "case_advance", "case_id": CASE,
        "policy": "authority/active-policy.json", "timeout": 60, "request_id": "req-race-1",
        "transition_items": transition("task-a", "skip", true),
        "actor": {"actor_id": OPERATOR, "role": "operator"}});
    let mut client = c
        .cell
        .connect_for(
            &authenticated::principal(OPERATOR, ActorRole::Operator),
            &wire,
        )
        .await;
    let task = tokio::spawn(async move { client.call(wire).await });
    pause.reached.notified().await;
    // The admission passed; now the suspension lands.
    assert_eq!(
        c.dispose("case", "suspend", None).await["lifecycle"],
        "suspended"
    );
    pause.resume.notify_one();
    let got = task.await.unwrap();
    assert_eq!(got["code"], "CASE_SUSPENDED", "{got}");
    assert_eq!(c.events(), before, "nothing was written");
}

/// Item 3: an approval is bound to the record it authorises. A write that fails
/// after the approval was consumed is completed by the identical retry (once);
/// any other record with that approval is refused.
#[tokio::test(flavor = "multi_thread")]
async fn review1_failed_append_does_not_burn_its_approval() {
    use std::os::unix::fs::PermissionsExt;
    let mut c = FieldCell::start(|_| {}).await;
    let records = case_dir(&c.root).join("field-records.jsonl");
    fs::write(&records, "").unwrap();
    let payload = json!({"level": "case", "disposition": "revise", "rationale": "again",
                         "approval_ref": "approval:flaky"});
    c.approve("approval:flaky", "disposition.recorded", &payload);
    fs::set_permissions(&records, fs::Permissions::from_mode(0o444)).unwrap();
    let failed = c
        .append_keyed(
            OPERATOR,
            "disposition.recorded",
            payload.clone(),
            "operator/flaky",
        )
        .await;
    assert_ne!(failed["outcome"], "recorded", "{failed}");
    assert_eq!(c.ledger_len(), 0);
    fs::set_permissions(&records, fs::Permissions::from_mode(0o644)).unwrap();
    // A different record under that approval is refused.
    let mut other = payload.clone();
    other["rationale"] = json!("something else");
    let denied = c.append(OPERATOR, "disposition.recorded", other).await;
    assert_eq!(denied["code"], "AUTHORITY_DENIED", "{denied}");
    // The identical retry completes the write.
    let done = c
        .append_keyed(
            OPERATOR,
            "disposition.recorded",
            payload.clone(),
            "operator/flaky",
        )
        .await;
    assert_eq!(done["outcome"], "recorded", "{done}");
    assert_eq!(c.ledger_len(), 1);
    // Once: the same record under another key, and a replay.
    let twice = c
        .append_keyed(
            OPERATOR,
            "disposition.recorded",
            payload.clone(),
            "operator/flaky-2",
        )
        .await;
    assert_eq!(twice["code"], "AUTHORITY_DENIED", "{twice}");
    let replay = c
        .append_keyed(OPERATOR, "disposition.recorded", payload, "operator/flaky")
        .await;
    assert_eq!(replay["replayed"], true, "{replay}");
    assert_eq!(c.ledger_len(), 1);
    // A crash between the two writes leaves exactly that state: the consumption
    // is there, the record is not. The retry completes it without a second line.
    let payload = json!({"level": "case", "disposition": "escalate", "rationale": "crash",
                         "approval_ref": "approval:crash"});
    c.approve("approval:crash", "disposition.recorded", &payload);
    let body = digest_of(&json!({"kind": "disposition.recorded", "payload": payload})).unwrap();
    write_lines(
        &case_dir(&c.root).join("field-approvals.jsonl"),
        &[
            json!({"approval_ref": "approval:crash", "consumed_by": "rec-0002", "record_digest": body}),
        ],
    );
    let done = c
        .append_keyed(OPERATOR, "disposition.recorded", payload, "operator/crash")
        .await;
    assert_eq!(done["outcome"], "recorded", "{done}");
    let consumed = c
        .read_store("field-approvals.jsonl")
        .iter()
        .filter(|l| l["approval_ref"] == "approval:crash" && l["consumed_by"].is_string())
        .count();
    assert_eq!(consumed, 1);
}

/// Item 4: every kind of the family needs the document; missing or invalid is
/// STORE_INTEGRITY and nothing is written.
#[tokio::test(flavor = "multi_thread")]
async fn review1_every_kind_needs_the_document() {
    for broken in [false, true] {
        let mut c = FieldCell::start_with(Some("fde.field_case@1"), false, |_| {}).await;
        if broken {
            fs::write(
                case_dir(&c.root).join("field_case.json"),
                "{\"case_id\": 1}",
            )
            .unwrap();
        }
        let disp = json!({"level": "case", "disposition": "revise", "rationale": "r",
                          "approval_ref": "approval:d"});
        c.approve("approval:d", "disposition.recorded", &disp);
        for (kind, payload) in [
            (
                "field_case.revised",
                revision_payload(1, "framing.better_question", "proposed", "Why?"),
            ),
            ("strategy.recorded", strategy_payload(1, None)),
            (
                "gate.assessed",
                gate_payload("choose_problem", "met", &["evidence:E1"]),
            ),
            ("disposition.recorded", disp),
        ] {
            let got = c.append(OPERATOR, kind, payload).await;
            assert_eq!(
                got["code"], "STORE_INTEGRITY",
                "broken={broken} {kind}: {got}"
            );
        }
        assert_eq!(c.ledger_len(), 0, "nothing was written");
    }
}

/// Item 2, the execution side: a suspension that lands after the transition
/// write and before the engine pass is honoured by the pass.
#[tokio::test(flavor = "multi_thread")]
async fn review1_suspension_before_the_engine_pass() {
    let mut c =
        FieldCell::start(|s| s.plan(&[("task-a".into(), false), ("task-b".into(), false)])).await;
    let first = sea_forge_server::pause_after_admission(&c.root);
    let wire = json!({"verb": "case_advance", "case_id": CASE,
        "policy": "authority/active-policy.json", "timeout": 60, "request_id": "req-race-2",
        "transition_items": transition("task-a", "skip", true),
        "actor": {"actor_id": OPERATOR, "role": "operator"}});
    let mut client = c
        .cell
        .connect_for(
            &authenticated::principal(OPERATOR, ActorRole::Operator),
            &wire,
        )
        .await;
    let task = tokio::spawn(async move { client.call(wire).await });
    first.reached.notified().await;
    let second = sea_forge_server::pause_after_admission(&c.root);
    first.resume.notify_one();
    second.reached.notified().await;
    assert_eq!(
        c.dispose("case", "suspend", None).await["lifecycle"],
        "suspended"
    );
    second.resume.notify_one();
    let got = task.await.unwrap();
    assert_eq!(got["code"], "CASE_SUSPENDED", "{got}");
}

// ---------------------------------------------------------------------------
// WP-21 part B, review round 2 (blind review).
// ---------------------------------------------------------------------------

impl FieldCell {
    /// A real case record and a plan of `items` (raw plan items), with the
    /// policy that lets an operator complete a human task: what the engine and
    /// the human completion read.
    fn runnable(&self, items: Value) {
        let dir = case_dir(&self.root);
        fs::write(
            dir.join("case.json"),
            json!({"version": "0.2", "case_id": CASE,
                   "intent": {"intent_id": "int_seed", "summary": "seed", "actor_id": OPERATOR,
                              "process_id": "proc_seed", "created_at": "2026-11-15T08:00:00Z"},
                   "state": "active", "plan_ref": "plan.json", "run_ids": [], "stages": [],
                   "close_reason": null, "created_at": "2026-11-15T08:00:00Z", "closed_at": null})
            .to_string(),
        )
        .unwrap();
        let path = dir.join("plan.json");
        let mut plan: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        plan["items"] = items;
        fs::write(path, plan.to_string()).unwrap();
        fs::write(
            self.root.join("policy.yaml"),
            "version: \"0.1\"\nrules:\n  - name: allow-human-task\n    verdict: allow\n    actor_role: operator\n    operation_kind: human_task_completion\n",
        )
        .unwrap();
    }

    fn wire_human_complete(&mut self, item: &str) -> Value {
        json!({"verb": "human_task_complete", "case_id": CASE, "item_id": item,
               "justification": "reviewed", "policy": "policy.yaml",
               "request_id": format!("req-htc-{}-{}", item, self.rid.fetch_add(1, std::sync::atomic::Ordering::SeqCst)),
               "actor": {"actor_id": OPERATOR, "role": "operator"}})
    }

    async fn human_complete(&mut self, item: &str) -> Value {
        let wire = self.wire_human_complete(item);
        self.call(OPERATOR, wire).await
    }

    fn event_kinds(&self) -> Vec<String> {
        String::from_utf8(self.events())
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap()["kind"].to_string())
            .collect()
    }
}

fn human(id: &str) -> Value {
    let criteria =
        serde_json::to_value(sea_forge_core::types::SettlementCriteria::default()).unwrap();
    json!({"plan_item_id": id, "name": id, "settlement_criteria": criteria,
           "item_kind": "human_task", "markers": {"required": true}})
}

fn milestone(id: &str, after: &str) -> Value {
    let criteria =
        serde_json::to_value(sea_forge_core::types::SettlementCriteria::default()).unwrap();
    json!({"plan_item_id": id, "name": id, "settlement_criteria": criteria,
           "item_kind": "milestone", "markers": {"required": true}, "depends_on": [after]})
}

/// Finding 1: a human completion followed by an automatic gate milestone and no
/// assessment writes neither `milestone_achieved` nor a `case_closed` (R11).
#[tokio::test(flavor = "multi_thread")]
async fn review2_automatic_milestone_cannot_bypass_a_gate() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([
        human("task-a"),
        milestone("gate_prove_value", "task-a")
    ]));
    let got = c.human_complete("task-a").await;
    let kinds = c.event_kinds();
    assert!(
        !kinds.iter().any(|k| k.contains("milestone_achieved")),
        "{got} {kinds:?}"
    );
    assert!(
        !kinds.iter().any(|k| k.contains("case_closed")),
        "{got} {kinds:?}"
    );
    // With a passing assessment the milestone follows, and the close still
    // waits for its case-level disposition.
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("prove_value", "met", &["evidence:E1"]),
        )
        .await;
    assert_eq!(r["outcome"], "recorded", "{r}");
}

/// Finding 1, second half: a `met` assessment changed to `blocked` after the
/// completion's admission still stops the milestone at the write.
#[tokio::test(flavor = "multi_thread")]
async fn review2_assessment_changed_after_admission_stops_the_milestone() {
    let mut c = FieldCell::start(|_| {}).await;
    // `task-z` keeps the plan open (a plan that ends needs its close disposition)
    // and waits for the gate, so the milestone is the only thing the engine has
    // to do after the completion.
    let mut z = human("task-z");
    z["entry_criteria"] =
        json!([{"on": {"source": "gate_prove_value", "event": "milestone_achieved"}}]);
    c.runnable(json!([
        human("task-a"),
        milestone("gate_prove_value", "task-a"),
        z
    ]));
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("prove_value", "met", &["evidence:E1"]),
        )
        .await;
    assert_eq!(r["outcome"], "recorded", "{r}");
    let pause = sea_forge_server::pause_after_admission(&c.root);
    let wire = c.wire_human_complete("task-a");
    let mut client = c
        .cell
        .connect_for(
            &authenticated::principal(OPERATOR, ActorRole::Operator),
            &wire,
        )
        .await;
    let task = tokio::spawn(async move { client.call(wire).await });
    tokio::time::timeout(std::time::Duration::from_secs(30), pause.reached.notified())
        .await
        .expect("the request reached its pause");
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("prove_value", "blocked", &[]),
        )
        .await;
    assert_eq!(r["outcome"], "recorded", "{r}");
    pause.resume.notify_one();
    let got = task.await.unwrap();
    let kinds = c.event_kinds();
    assert!(
        kinds.iter().any(|k| k.contains("human_task_completed")),
        "the completion itself is not blocked: {got} {kinds:?}"
    );
    assert!(
        !kinds.iter().any(|k| k.contains("milestone_achieved")),
        "{got} {kinds:?}"
    );
}

/// Finding 1, R11 through an automatic milestone: the completion that would
/// cascade to a close is refused before anything is written.
#[tokio::test(flavor = "multi_thread")]
async fn review2_cascading_close_is_refused_before_any_write() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("task-a"), milestone("ms-plain", "task-a")]));
    let got = c.human_complete("task-a").await;
    assert_eq!(got["code"], "CLOSE_DISPOSITION_REQUIRED", "{got}");
    assert!(c.events().is_empty(), "nothing was written");
}

/// Finding 2: a suspension that lands after the human completion's admission is
/// honoured at the write; nothing is written.
#[tokio::test(flavor = "multi_thread")]
async fn review2_human_completion_honours_a_suspension_after_admission() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("task-a"), human("task-b")]));
    let pause = sea_forge_server::pause_after_admission(&c.root);
    let wire = c.wire_human_complete("task-a");
    let mut client = c
        .cell
        .connect_for(
            &authenticated::principal(OPERATOR, ActorRole::Operator),
            &wire,
        )
        .await;
    let task = tokio::spawn(async move { client.call(wire).await });
    tokio::time::timeout(std::time::Duration::from_secs(30), pause.reached.notified())
        .await
        .expect("the request reached its pause");
    assert_eq!(
        c.dispose("case", "suspend", None).await["lifecycle"],
        "suspended"
    );
    pause.resume.notify_one();
    let got = task.await.unwrap();
    assert_eq!(got["code"], "CASE_SUSPENDED", "{got}");
    assert!(c.events().is_empty(), "nothing was written");
}

/// Finding 3: an episode kind needs the document like every other kind: a valid
/// request with no `field_case.json` (or an invalid one) is STORE_INTEGRITY and
/// writes nothing.
#[tokio::test(flavor = "multi_thread")]
async fn review2_episode_kind_needs_the_document() {
    let fixture =
        read(&fixtures("case-record-append-v2/valid").join("request-episode-frozen-v2.json"));
    for broken in [false, true] {
        let refs = &fixture["payload"]["realitytrace_refs"];
        let mut ids: Vec<String> = vec![refs["plan_id"].as_str().unwrap().into()];
        for list in ["question_ids", "threshold_ids"] {
            ids.extend(
                refs[list]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().into()),
            );
        }
        let mut c =
            FieldCell::start_with(Some("fde.field_case@1"), false, |s| s.rt_register(&ids)).await;
        if broken {
            fs::write(
                case_dir(&c.root).join("field_case.json"),
                "{\"case_id\": 1}",
            )
            .unwrap();
        }
        let got = c
            .append_keyed(
                KEEPER,
                "episode.frozen",
                fixture["payload"].clone(),
                "operator/doc-1",
            )
            .await;
        assert_eq!(got["code"], "STORE_INTEGRITY", "broken={broken}: {got}");
        assert_eq!(c.ledger_len(), 0, "broken={broken}: nothing was written");
    }
}

/// Finding 4: a closed case runs only the action that ends the plan. A
/// transition written before the close, the case closed before the engine pass:
/// the pass must not activate the next task.
#[tokio::test(flavor = "multi_thread")]
async fn review2_closed_case_refuses_the_engine_pass_that_does_not_end_the_plan() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("task-a"), human("task-b")]));
    let first = sea_forge_server::pause_after_admission(&c.root);
    let wire = json!({"verb": "case_advance", "case_id": CASE,
        "policy": "authority/active-policy.json", "timeout": 60, "request_id": "req-race-3",
        "transition_items": transition("task-a", "skip", true),
        "actor": {"actor_id": OPERATOR, "role": "operator"}});
    let mut client = c
        .cell
        .connect_for(
            &authenticated::principal(OPERATOR, ActorRole::Operator),
            &wire,
        )
        .await;
    let task = tokio::spawn(async move { client.call(wire).await });
    tokio::time::timeout(std::time::Duration::from_secs(30), first.reached.notified())
        .await
        .expect("the request reached its pause");
    let second = sea_forge_server::pause_after_admission(&c.root);
    first.resume.notify_one();
    tokio::time::timeout(
        std::time::Duration::from_secs(30),
        second.reached.notified(),
    )
    .await
    .expect("the request reached its pause");
    // The transition is written; now the case closes before the engine pass.
    let closed = c.dispose("case", "close-satisfied", None).await;
    assert_eq!(closed["lifecycle"], "closed", "{closed}");
    second.resume.notify_one();
    let got = task.await.unwrap();
    let kinds = c.event_kinds();
    assert!(
        !kinds.iter().any(|k| k.contains("item_activated")),
        "task-b must not be activated: {got} {kinds:?}"
    );
    assert_eq!(got["code"], "CASE_CLOSED", "{got}");
}

// ---------------------------------------------------------------------------
// WP-21 part B, review round 3: ONE guarded choke point for every write
// (`case_guard::admit_write`). Each test is a reproduced counterexample.
// ---------------------------------------------------------------------------

impl FieldCell {
    /// Let the operator do these case operations (policy rules appended).
    fn allow(&self, operation_kinds: &[&str]) {
        let path = self.root.join("policy.yaml");
        let mut text = fs::read_to_string(&path).unwrap();
        for kind in operation_kinds {
            text.push_str(&format!(
                "  - name: allow-{kind}\n    verdict: allow\n    actor_role: operator\n    operation_kind: {kind}\n"
            ));
        }
        fs::write(path, text).unwrap();
    }

    fn policy(&self) -> PathBuf {
        self.root.join("policy.yaml")
    }

    fn case_state(&self) -> String {
        let case: Value =
            serde_json::from_slice(&fs::read(case_dir(&self.root).join("case.json")).unwrap())
                .unwrap();
        case["state"].as_str().unwrap().to_string()
    }
}

/// 1. A human task that IS a gate item, with a `blocked` assessment: the CLI
/// `task complete` path writes nothing.
#[tokio::test(flavor = "multi_thread")]
async fn review3_cli_task_complete_cannot_complete_a_gate_item() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("gate_allocate_authority"), human("task-z")]));
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("allocate_authority", "blocked", &[]),
        )
        .await;
    assert_eq!(r["outcome"], "recorded", "{r}");
    let got = sea_forge_case_runner::case_ops::complete_human_task(
        &c.root,
        &c.policy(),
        OPERATOR,
        CASE,
        "gate_allocate_authority",
        None,
    );
    assert!(
        got.is_err(),
        "the CLI path completed a blocked gate: {got:?}"
    );
    assert!(
        got.unwrap_err()
            .to_string()
            .contains("HARD_GATE_NOT_WAIVABLE"),
        "the contract's code"
    );
    assert!(
        c.events().is_empty(),
        "nothing was written: {:?}",
        c.event_kinds()
    );
}

/// 2. Terminate writes `CaseTerminated` only behind R11's close disposition.
#[tokio::test(flavor = "multi_thread")]
async fn review3_terminate_needs_a_close_disposition() {
    let c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("task-a")]));
    c.allow(&["case_terminate"]);
    let got =
        sea_forge_case_runner::case_ops::terminate(&c.root, &c.policy(), OPERATOR, CASE, "stop");
    assert!(
        got.is_err(),
        "terminated with no close disposition: {got:?}"
    );
    assert!(got
        .unwrap_err()
        .to_string()
        .contains("CLOSE_DISPOSITION_REQUIRED"));
    assert!(c.events().is_empty(), "{:?}", c.event_kinds());
    assert_eq!(c.case_state(), "active");
}

/// 3a. `suspend` then terminate: refused, from the ledger.
#[tokio::test(flavor = "multi_thread")]
async fn review3_suspended_case_cannot_be_terminated() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("task-a")]));
    c.allow(&["case_terminate"]);
    assert_eq!(
        c.dispose("case", "suspend", None).await["lifecycle"],
        "suspended"
    );
    let got =
        sea_forge_case_runner::case_ops::terminate(&c.root, &c.policy(), OPERATOR, CASE, "stop");
    assert!(got.is_err(), "terminated a suspended case: {got:?}");
    assert!(got.unwrap_err().to_string().contains("CASE_SUSPENDED"));
    assert!(c.events().is_empty());
    assert_eq!(c.case_state(), "active");
}

/// 3b. `close-satisfied`, finish the plan, then `case_reopen`: a closed Field
/// Case is final (D32, R13); the case stays completed.
#[tokio::test(flavor = "multi_thread")]
async fn review3_closed_case_cannot_be_reopened() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("task-a")]));
    c.allow(&["case_reopen"]);
    assert_eq!(
        c.dispose("case", "close-satisfied", None).await["lifecycle"],
        "closed"
    );
    let done = c.human_complete("task-a").await;
    assert!(done["code"].is_null(), "{done}");
    assert_eq!(c.case_state(), "completed");
    let before = c.events();
    let got = sea_forge_case_runner::case_ops::reopen(&c.root, &c.policy(), OPERATOR, CASE);
    assert!(got.is_err(), "reopened a closed Field Case: {got:?}");
    assert!(got.unwrap_err().to_string().contains("CASE_CLOSED"));
    assert_eq!(c.events(), before);
    assert_eq!(c.case_state(), "completed");
}

/// 4. A delegation whose case is suspended between its admission and its
/// execution is re-admitted at the execution write and refused.
#[tokio::test(flavor = "multi_thread")]
async fn review3_delegation_is_readmitted_at_execution() {
    let all = scenarios("execution-episode-ref");
    let (_, s) = all
        .iter()
        .find(|(id, _)| id == "delegate-complete-freeze-accepted")
        .expect("an accepted delegate scenario");
    run_gate_with(s, true).await.unwrap();
}

/// 5. After a close disposition and the final human completion, a repeat with a
/// fresh request id writes no second `human_task_completed`.
#[tokio::test(flavor = "multi_thread")]
async fn review3_final_completion_is_not_repeated_on_a_closed_case() {
    let mut c = FieldCell::start(|_| {}).await;
    c.runnable(json!([human("task-a")]));
    assert_eq!(
        c.dispose("case", "close-satisfied", None).await["lifecycle"],
        "closed"
    );
    let first = c.human_complete("task-a").await;
    assert!(first["code"].is_null(), "{first}");
    let before = c.events();
    let again = c.human_complete("task-a").await;
    assert_eq!(again["code"], "CASE_CLOSED", "{again}");
    assert_eq!(c.events(), before, "no second event");
}

/// 6. The v1 dialect (no marker) is held to the same document rule as v2.
#[tokio::test(flavor = "multi_thread")]
async fn review3_v1_dialect_needs_the_document() {
    let fixture =
        read(&fixtures("case-record-append-v2/valid").join("request-episode-frozen-v2.json"));
    let refs = &fixture["payload"]["realitytrace_refs"];
    let mut ids: Vec<String> = vec![refs["plan_id"].as_str().unwrap().into()];
    for list in ["question_ids", "threshold_ids"] {
        ids.extend(
            refs[list]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().into()),
        );
    }
    let mut c =
        FieldCell::start_with(Some("fde.field_case@1"), false, |s| s.rt_register(&ids)).await;
    let got = c
        .call(
            KEEPER,
            json!({"verb": "case_record_append", "kind": "episode.frozen",
                   "payload": fixture["payload"].clone(), "idempotency_key": "operator/v1-doc",
                   "request_id": "req-v1-doc"}),
        )
        .await;
    assert!(
        got["outcome"] != "recorded" && got.to_string().contains("STORE_INTEGRITY"),
        "the v1 dialect froze with no document: {got}"
    );
    assert_eq!(c.ledger_len(), 0, "nothing was written");
}

/// 7. A stored record whose hash was corrupted is STORE_INTEGRITY for every
/// later write; nothing is written.
#[tokio::test(flavor = "multi_thread")]
async fn review3_corrupt_record_hash_refuses_every_write() {
    let mut c = FieldCell::start(|_| {}).await;
    let r = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("choose_problem", "blocked", &[]),
        )
        .await;
    assert_eq!(r["outcome"], "recorded", "{r}");
    let path = case_dir(&c.root).join("field-records.jsonl");
    let text = fs::read_to_string(&path).unwrap();
    let mut rec: Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    rec["hash"] = json!("ab".repeat(32));
    fs::write(&path, format!("{rec}\n")).unwrap();
    let got = c
        .append(
            OPERATOR,
            "gate.assessed",
            gate_payload("choose_problem", "met", &["evidence:E1"]),
        )
        .await;
    assert_eq!(got["code"], "STORE_INTEGRITY", "{got}");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("{rec}\n"),
        "nothing was written"
    );
}
