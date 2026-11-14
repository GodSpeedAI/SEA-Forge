#![cfg(unix)]
//! WP-18 over a real Unix socket: `authority_promote`, `authority_repin` and
//! `authority_pin` carry a world promotion over the existing approval-gated
//! `semantic_change` transition; the pin is a durable record; evidence settled
//! under a world stays bound to it after promotion and after re-pin (NT-07); a
//! subsequent episode freezes against the new `world_ref` (AT-13, SEA-Forge half).
//!
//! The pin/decision semantics (NT-05 four codes, NT-08, NT-21, proposer is not
//! the approver) are in `sea-forge-authority/tests/world_pin.rs`; this file is
//! the verbs, the identity gate and the case-record store.

use sea_forge_core::types::ActorRole;
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use sea_forge_server::config::{CepAuthorityConfig, CepWorldConfig};
use sea_forge_server::identity::{IdentityBinding, IdentityBindings};
use sea_forge_server::sfwp::case_records::{append, digest_of, AppendContext, FileIndex};
use sea_forge_server::ServerConfig;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

#[path = "support/authenticated.rs"]
mod authenticated;

const BASE: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const CHANGED: &str =
    "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\nentity \"Pump\" { key id: uuid }\n";
const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: moves\n    verdict: allow\n    actor_role: service\n    operation_kind: world_transition\n  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

fn world_ref_of(src: &str) -> String {
    let mut reg = WorldRegistry::new();
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: hex::encode(Sha256::digest(src.as_bytes())),
            content: src.into(),
        }],
    };
    reg.register_source_set("demo", &set).unwrap().to_string()
}

fn config() -> (tempfile::TempDir, ServerConfig) {
    let root = tempfile::tempdir().unwrap();
    for (dir, src) in [("worlds", BASE), ("worlds-changed", CHANGED)] {
        std::fs::create_dir_all(root.path().join(dir)).unwrap();
        std::fs::write(root.path().join(dir).join("demo.sea"), src).unwrap();
    }
    std::fs::write(root.path().join("sea-forge-policy.yaml"), POLICY).unwrap();
    let cases = root.path().join("cases/case-0001");
    std::fs::create_dir_all(&cases).unwrap();
    std::fs::write(cases.join("case.json"), "{}").unwrap();
    std::fs::write(cases.join("case-events.jsonl"), "").unwrap();
    let socket = std::env::temp_dir().join(format!(
        "sf-pin-{}-{}.sock",
        std::process::id(),
        root.path().file_name().unwrap().to_string_lossy()
    ));
    let uid = sea_forge_server::identity::current_uid().expect("uid");
    let world = |base: &str| CepWorldConfig {
        name: "demo".into(),
        base: base.into(),
        entry: "demo.sea".into(),
        files: vec!["demo.sea".into()],
        prior_identities: Vec::new(),
    };
    let config = ServerConfig {
        socket_path: socket,
        root: root.path().to_path_buf(),
        identity: IdentityBindings {
            bindings: vec![
                IdentityBinding {
                    uid: Some(uid),
                    sid: None,
                    actor_id: "cognate-service".into(),
                    roles: vec![ActorRole::Service],
                },
                IdentityBinding {
                    uid: None,
                    sid: None,
                    actor_id: "op-alice".into(),
                    roles: vec![ActorRole::Operator],
                },
            ],
        },
        cep_authority: CepAuthorityConfig {
            enabled: true,
            pinned_baseline: Some("demo".into()),
            policy: "sea-forge-policy.yaml".into(),
            worlds: vec![world("worlds"), world("worlds-changed")],
            approval_ttl_hours: 24,
            realitytrace: None,
            settlement: Default::default(),
        },
        ..ServerConfig::default()
    };
    (root, config)
}

/// A plain connection: no capability, so no verified identity.
struct Client {
    writer: OwnedWriteHalf,
    reader: BufReader<OwnedReadHalf>,
}

impl Client {
    async fn connect(socket: &Path) -> Self {
        let (reader, writer) = UnixStream::connect(socket).await.unwrap().into_split();
        Self {
            writer,
            reader: BufReader::new(reader),
        }
    }
    async fn call(&mut self, value: Value) -> Value {
        self.writer
            .write_all(format!("{value}\n").as_bytes())
            .await
            .unwrap();
        self.writer.flush().await.unwrap();
        let mut line = String::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.reader.read_line(&mut line),
        )
        .await
        .expect("read timed out")
        .unwrap();
        serde_json::from_str(line.trim()).unwrap()
    }
}

fn jcs(v: &Value) -> String {
    sea_forge_server::sfwp::case_records::jcs(v).unwrap()
}
fn hex_of(v: &Value) -> String {
    hex::encode(Sha256::digest(jcs(v).as_bytes()))
}

fn proposal(old: &str, cand: &str) -> Value {
    let mut p = json!({
        "contract": "revision-proposal.v1", "proposer": "cognate-service", "scope": "case:case-0001",
        "old_world_ref": old, "candidate_world_ref": cand,
        "held_out_rule": {"rule_id": "rule:s1-heldout", "sha256": format!("sha256:{}", "3".repeat(64))},
        "diagnosis": {"episodes": ["episode:case-0001:item-03:1@sha256:11"], "residual_ids": ["residual:a"], "occurrence_ids": ["sea-forge:a1"]},
        "held_out": {"episodes": ["episode:case-0001:item-09:1@sha256:22"], "residual_ids": ["residual:b"], "occurrence_ids": ["sea-forge:b1"]},
        "intended_changes": [{"element": "flow-fresh-context-verification", "change": "add_declaration", "supersedes": ["undeclared:x"]}],
        "semantic_diff_sha256": format!("sha256:{}", "4".repeat(64)),
    });
    p["proposal_id"] = json!(format!("proposal:{}", hex_of(&p)));
    p
}

fn matrix(p: &Value) -> Value {
    let id = p["proposal_id"].as_str().unwrap();
    let rows: Vec<Value> = ["item-03", "item-09"]
        .iter()
        .map(|item| {
            json!({
                "element": format!("episode:case-0001:{item}:1|structural|undeclared:x"),
                "episode_ref": format!("episode:case-0001:{item}:1"), "comparison": "structural",
                "old_expectation": "undeclared:x", "new_expectation": "flow-fresh-context-verification",
                "old_class": "observed_undeclared", "new_class": "confirmed",
                "result": "pass", "code": null, "reason": "declared", "intended": true,
                "escalated": false, "reported": "gain"})
        })
        .collect();
    let mut m = json!({
        "contract": "regression-matrix.v1", "proposal_id": id,
        "proposal_digest": format!("sha256:{}", &id["proposal:".len()..]),
        "old_world_ref": p["old_world_ref"], "candidate_world_ref": p["candidate_world_ref"],
        "old_comparisons": ["comparison:o"], "new_comparisons": ["comparison:n"],
        "rows": rows,
        "summary": {"pass": 2, "fail": 0, "escalated": 0, "gains": 2},
        "verdict": "pass", "failure_codes": [], "refusal_code": null, "escalated": false,
    });
    m["matrix_id"] = json!(format!("matrix:{}", hex_of(&m)));
    m
}

fn transition_request(
    source: &str,
    target: &str,
    op: &str,
    claims: Value,
    approval: Option<&str>,
) -> Value {
    let mut ext = json!({"operation_id": op, "correlation_id": "corr-1", "operation_kind": "world_transition",
        "operation_name": "transition", "resource_id": target, "claims": claims});
    if let Some(a) = approval {
        ext["approval_id"] = json!(a);
    }
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-04T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": source},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid",
        "lineage_refs": approval.map(|_| vec![format!("env-authority_request-{}", op.trim_end_matches("-b"))]).unwrap_or_default(),
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-04T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "capability_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": target,
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": ext,
            "godspeed.world_transition": {"target_world_ref": target, "transition_kind": "semantic_change",
                "reason": "adopt the reviewed model", "compatibility": "compatible"}
        }
    })
}

fn as_actor(mut verb: Value, actor_id: &str, role: &str) -> Value {
    verb["actor"] = json!({"actor_id": actor_id, "role": role});
    verb
}

fn authority_request(envelope: Value, op: &str) -> Value {
    as_actor(
        json!({"verb": "authority_request", "envelope": envelope, "request_id": op}),
        "cognate-service",
        "service",
    )
}

// ------------------------------------------------- case records under a world

fn contracts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fde/fixtures/case-record-append/valid")
}

fn read(name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(contracts().join(name)).unwrap()).unwrap()
}

/// The fixture freeze, re-pointed at `world`, `item`, with its digests recomputed.
fn frozen_under(world: &str, item: &str) -> Value {
    let mut req = read("request-episode-frozen.json");
    let body = &mut req["payload"]["reference"]["body"];
    body["world_ref"] = json!(world);
    body["plan_item_id"] = json!(item);
    body["n"] = json!(1);
    let digest = digest_of(body).unwrap();
    let episode_ref = format!("episode:case-0001:{item}:1@sha256:{digest}");
    req["payload"]["episode_ref"] = json!(episode_ref);
    req["payload"]["reference"]["episode_ref"] = json!(episode_ref);
    req["payload"]["digest"] = json!(digest);
    req["idempotency_key"] = json!(format!("case-keeper/{item}"));
    req
}

fn settled_for(freeze: &Value) -> Value {
    let mut req = read("request-episode-settled.json");
    req["payload"]["episode_ref"] = freeze["payload"]["episode_ref"].clone();
    req
}

fn rt_register(root: &Path, request: &Value) {
    let dir = root.join("realitytrace");
    std::fs::create_dir_all(&dir).unwrap();
    let refs = &request["payload"]["realitytrace_refs"];
    let Some(plan) = refs["plan_id"].as_str() else {
        return; // a settlement carries no RealityTrace refs
    };
    let mut ids = vec![plan.to_string()];
    for f in ["question_ids", "threshold_ids"] {
        ids.extend(
            refs[f]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string()),
        );
    }
    let mut text = std::fs::read_to_string(dir.join("records.jsonl")).unwrap_or_default();
    for id in ids {
        text.push_str(&format!("{}\n", json!({"record_id": id})));
    }
    std::fs::write(dir.join("records.jsonl"), text).unwrap();
}

fn write(root: &Path, principal: &str, request: &Value) -> Value {
    rt_register(root, request);
    let idx = FileIndex(root.to_path_buf());
    let chain = vec![principal.to_string()];
    append(
        &AppendContext {
            root,
            case_id: "case-0001",
            principal,
            role: "operator",
            chain: &chain,
            rt: &idx,
            anchor: None,
        },
        request,
    )
}

/// `(kind, world_ref the record was made under)` of every case record, in order.
fn record_worlds(root: &Path) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(root.join("cases/case-0001/field-records.jsonl"))
        .unwrap_or_default();
    let records: Vec<Value> = text
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    // A settled record names its episode; the world is that episode's frozen world.
    let frozen: std::collections::BTreeMap<String, String> = records
        .iter()
        .filter(|r| r["kind"] == "episode.frozen")
        .map(|r| {
            (
                r["payload"]["episode_ref"].as_str().unwrap().to_string(),
                r["payload"]["reference"]["body"]["world_ref"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    records
        .iter()
        .map(|r| {
            let ep = r["payload"]["episode_ref"].as_str().unwrap();
            (
                format!(
                    "{}:{}",
                    r["kind"].as_str().unwrap(),
                    ep.split(':').nth(2).unwrap()
                ),
                frozen[ep].clone(),
            )
        })
        .collect()
}

#[tokio::test]
async fn wp18_promote_and_re_pin_over_the_socket_keep_settled_evidence_with_its_world_and_a_new_episode_freezes_under_the_new_one(
) {
    let (root, config) = config();
    let mut cell = authenticated::Cell::start(config).await;
    let service = authenticated::principal("cognate-service", ActorRole::Service);
    let operator = authenticated::principal("op-alice", ActorRole::Operator);
    let (v1, v2) = (world_ref_of(BASE), world_ref_of(CHANGED));

    // Before: the baseline world is pinned, nothing is recorded.
    let pin = cell
        .call_for(&service, json!({"verb": "authority_pin"}))
        .await;
    assert_eq!(pin["current"], json!(v1), "{pin}");
    assert_eq!(pin["history"], json!([]));

    // Episode 1 freezes and settles under v1 (the S1 control episode).
    let ep1 = frozen_under(&v1, "item-03");
    assert_eq!(
        write(root.path(), "case-keeper", &ep1)["outcome"],
        "recorded"
    );
    assert_eq!(
        write(root.path(), "verifier", &settled_for(&ep1))["outcome"],
        "recorded"
    );
    assert_eq!(
        record_worlds(root.path()),
        vec![
            ("episode.frozen:item-03".into(), v1.clone()),
            ("episode.settled:item-03".into(), v1.clone())
        ]
    );

    // The proposal, the matrix, the escalation bound to the proposal digest, the approval.
    let p = proposal(&v1, &v2);
    let digest = format!(
        "sha256:{}",
        &p["proposal_id"].as_str().unwrap()["proposal:".len()..]
    );
    // The approver decides on the proposal digest AND the matrix and rule.
    let claims = json!({"proposal_digest": digest, "matrix_id": matrix(&p)["matrix_id"],
                        "held_out_rule_sha256": p["held_out_rule"]["sha256"]});
    let esc = cell
        .call_for(
            &service,
            authority_request(
                transition_request(&v1, &v2, "op-p1", claims.clone(), None),
                "op-p1",
            ),
        )
        .await;
    let approval = esc["envelope"]["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .expect("an escalation")
        .to_string();

    // Not approved yet: unknown is never approved.
    let second = transition_request(&v1, &v2, "op-p1-b", claims.clone(), Some(&approval));
    let promote = |env: &Value, id: &str| {
        as_actor(
            json!({"verb": "authority_promote", "proposal": p, "matrix": matrix(&p), "envelope": env, "request_id": id}),
            "cognate-service",
            "service",
        )
    };
    let early = cell.call_for(&service, promote(&second, "pr-1")).await;
    assert_eq!(
        (early["error_class"].as_str(), early["code"].as_str()),
        (Some("cep_promotion_refused"), Some("AUTHORITY_UNAVAILABLE")),
        "{early}"
    );
    let pin = cell
        .call_for(&service, json!({"verb": "authority_pin"}))
        .await;
    assert_eq!(pin["current"], json!(v1));
    assert_eq!(
        pin["attempts"][0]["code"], "AUTHORITY_UNAVAILABLE",
        "the refused attempt is retained"
    );

    // The approver is another identity (the requester cannot resolve its own request).
    let self_approve = cell
        .call_for(&service, as_actor(json!({"verb": "authority_approval", "approval_id": approval, "resolution": "approved", "request_id": "ar-self"}), "cognate-service", "service"))
        .await;
    assert!(self_approve["error_class"].is_string(), "{self_approve}");
    let resolved = cell
        .call_for(&operator, as_actor(json!({"verb": "authority_approval", "approval_id": approval, "resolution": "approved", "request_id": "ar-ok"}), "op-alice", "operator"))
        .await;
    assert_eq!(resolved["state"], "approved", "{resolved}");

    // A failing regression matrix blocks, with a valid approval in hand (NT-08).
    let mut broken = matrix(&p);
    broken["rows"][0]["result"] = json!("fail");
    broken["rows"][0]["code"] = json!("REGRESSION_FAILED");
    broken["verdict"] = json!("fail");
    broken["refusal_code"] = json!("REGRESSION_FAILED");
    broken["failure_codes"] = json!(["REGRESSION_FAILED"]);
    broken.as_object_mut().unwrap().remove("matrix_id");
    broken["matrix_id"] = json!(format!("matrix:{}", hex_of(&broken)));
    let blocked = cell
        .call_for(&service, as_actor(json!({"verb": "authority_promote", "proposal": p, "matrix": broken, "envelope": second, "request_id": "pr-0"}), "cognate-service", "service"))
        .await;
    assert_eq!(blocked["code"], "REGRESSION_FAILED", "{blocked}");

    // Promotion: one allowed semantic_change transition and one pin record.
    let promoted = cell.call_for(&service, promote(&second, "pr-2")).await;
    assert_eq!(promoted["ok"], true, "{promoted}");
    assert_eq!(promoted["pin"]["kind"], "promoted");
    assert_eq!(promoted["pin"]["approver"], "op-alice");
    assert_eq!(promoted["pin"]["proposer"], "cognate-service");
    assert_eq!(promoted["pin"]["to_world_ref"], json!(v2));
    let pin = cell
        .call_for(&service, json!({"verb": "authority_pin"}))
        .await;
    assert_eq!(pin["current"], json!(v2), "{pin}");
    let transitions = cell
        .call_for(&service, json!({"verb": "authority_transitions"}))
        .await;
    assert_eq!(transitions["transitions"].as_array().unwrap().len(), 1);
    assert_eq!(
        transitions["transitions"][0]["ledger_entry"],
        promoted["pin"]["transition_ref"]
    );

    // The subsequent episode freezes against the new world_ref; episode 1 is untouched (NT-07, after promotion).
    let ep2 = frozen_under(&v2, "item-04");
    assert_eq!(
        write(root.path(), "case-keeper", &ep2)["outcome"],
        "recorded"
    );
    let worlds = record_worlds(root.path());
    assert_eq!(worlds[0], ("episode.frozen:item-03".into(), v1.clone()));
    assert_eq!(
        worlds[1],
        ("episode.settled:item-03".into(), v1.clone()),
        "settled evidence still bound to the old world_ref"
    );
    assert_eq!(worlds[2], ("episode.frozen:item-04".into(), v2.clone()));

    // Re-pin back through the same gated transition (the recovery).
    let reason = "the revision regressed on the held-out episode";
    let rd = sea_forge_authority::cep::repin_digest(
        &v2,
        &v1,
        reason,
        promoted["pin"]["pin_id"].as_str().unwrap(),
    );
    let esc = cell
        .call_for(
            &service,
            authority_request(
                transition_request(&v2, &v1, "op-r1", json!({"repin_digest": rd}), None),
                "op-r1",
            ),
        )
        .await;
    let approval = esc["envelope"]["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    cell.call_for(&operator, as_actor(json!({"verb": "authority_approval", "approval_id": approval, "resolution": "approved", "request_id": "ar-2"}), "op-alice", "operator")).await;
    let env = transition_request(
        &v2,
        &v1,
        "op-r1-b",
        json!({"repin_digest": rd}),
        Some(&approval),
    );
    let repinned = cell
        .call_for(&service, as_actor(json!({"verb": "authority_repin", "to_world_ref": v1, "reason": reason, "envelope": env, "request_id": "rp-1"}), "cognate-service", "service"))
        .await;
    assert_eq!(repinned["ok"], true, "{repinned}");
    assert_eq!(repinned["pin"]["kind"], "repinned");
    assert_eq!(repinned["pin"]["supersedes_pin"], promoted["pin"]["pin_id"]);
    let pin = cell
        .call_for(&service, json!({"verb": "authority_pin"}))
        .await;
    assert_eq!(pin["current"], json!(v1));
    assert_eq!(pin["history"].as_array().unwrap().len(), 2);
    // NT-07 after re-pin: nothing was retargeted.
    assert_eq!(
        record_worlds(root.path()),
        worlds,
        "after re-pin the records are bound to the worlds they were made under"
    );

    // The identity gate: no verified identity, no promotion.
    let mut anon = Client::connect(&cell.socket).await;
    let refused = anon.call(json!({"verb": "authority_promote", "proposal": p, "matrix": matrix(&p), "envelope": second, "request_id": "pr-anon"})).await;
    assert!(
        refused["error_class"].is_string() && refused["ok"] != json!(true),
        "{refused}"
    );

    sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test-observer")
        .unwrap()
        .verify()
        .unwrap();
    cell.verify_authentication();
    cell.shutdown().await;
}

// ------------------------------------------------------- review fixes (F3, F8)

#[test]
fn f3_the_vendored_regression_matrix_schema_matches_its_digest_and_the_source() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/rt");
    let manifest = std::fs::read_to_string(dir.join("DIGESTS")).unwrap();
    let text = std::fs::read_to_string(dir.join("regression-matrix.v1.json")).unwrap();
    let want = manifest
        .lines()
        .find_map(|l| {
            l.split_once("  ")
                .filter(|(_, n)| *n == "regression-matrix.v1.json")
        })
        .map(|(h, _)| h.to_string())
        .expect("listed in DIGESTS");
    assert_eq!(want, hex::encode(sha2::Sha256::digest(text.as_bytes())));
    // When the RealityTrace worktree is present the copy must equal it.
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fde-sxr-integration/contracts/regression-matrix.v1.json");
    if let Ok(origin) = std::fs::read_to_string(source) {
        assert_eq!(
            origin, text,
            "the vendored schema drifted from RealityTrace's"
        );
    }
}

#[tokio::test]
async fn f3_a_matrix_that_is_not_a_regression_matrix_document_is_refused_before_the_service() {
    let (_root, config) = config();
    let mut cell = authenticated::Cell::start(config).await;
    let service = authenticated::principal("cognate-service", ActorRole::Service);
    let (v1, v2) = (world_ref_of(BASE), world_ref_of(CHANGED));
    let p = proposal(&v1, &v2);
    let mut bad = matrix(&p);
    bad["rows"] = json!([{"element": "e"}]);
    let out = cell
        .call_for(
            &service,
            as_actor(
                json!({"verb": "authority_promote", "proposal": p, "matrix": bad,
                       "envelope": {}, "request_id": "pr-bad"}),
                "cognate-service",
                "service",
            ),
        )
        .await;
    assert_eq!(out["code"], "MATRIX_INVALID", "{out}");
}

#[tokio::test]
async fn f8_two_worlds_without_a_named_baseline_cannot_move_the_pin() {
    let (_root, mut config) = config();
    config.cep_authority.pinned_baseline = None;
    let mut cell = authenticated::Cell::start(config).await;
    let service = authenticated::principal("cognate-service", ActorRole::Service);
    let (v1, v2) = (world_ref_of(BASE), world_ref_of(CHANGED));
    let p = proposal(&v1, &v2);
    let out = cell
        .call_for(
            &service,
            as_actor(
                json!({"verb": "authority_promote", "proposal": p, "matrix": matrix(&p),
                       "envelope": {}, "request_id": "pr-amb"}),
                "cognate-service",
                "service",
            ),
        )
        .await;
    assert_eq!(out["error_class"], "cep_pin_baseline_ambiguous", "{out}");
    let pin = cell
        .call_for(&service, json!({"verb": "authority_pin"}))
        .await;
    assert_eq!(pin["current"], json!(null), "no baseline is guessed: {pin}");
}
