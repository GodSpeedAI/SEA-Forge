//! Stage 6 over a real Unix socket: the `authority_request` verb returns CEP
//! `authority_decision` envelopes from the real policy engine, commits them to
//! the cell's append-only ledger, and fails closed.

use sea_forge_core::types::ActorRole;
use sea_forge_domainforge::{SeaSourceSet, SourceFile, WorldRegistry};
use sea_forge_server::config::{CepAuthorityConfig, CepWorldConfig};
use sea_forge_server::identity::{IdentityBinding, IdentityBindings};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

#[path = "support/authenticated.rs"]
mod authenticated;

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: allow-actions\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n  - name: gate-capabilities\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_capability\n    requires_approval: true\n";

/// Same meaning as `SRC`, different bytes (a comment-only edit): a new world, the same semantic closure.
const EDITED: &str = "// reviewed\n@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
/// A different model.
const CHANGED: &str =
    "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\nentity \"Pump\" { key id: uuid }\n";
const TRANSITION_POLICY: &str = "version: \"0.1\"\nrules:\n  - name: moves\n    verdict: allow\n    actor_role: service\n    operation_kind: world_transition\n  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

fn world_ref() -> String {
    world_ref_of(SRC)
}

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

fn request(world: &str, op: &str, kind: &str, name: &str) -> Value {
    json!({
        "envelope_id": format!("env-authority_request-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
        "created_at": "2026-10-04T12:00:00Z", "created_by": "cognate",
        "scope": {"evaluation_context": "cognate-authority_request", "world_ref": world},
        "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [],
            "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": [format!("prov-{op}")], "validation_status": "valid", "lineage_refs": [],
        "provenance": [{"provenance_id": format!("prov-{op}"), "source_system_refs": ["cognate"],
            "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-04T12:00:00Z"}],
        "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}],
        "transformations": [{"transformation_id": "tr-1", "transformation_type": "capability_invocation"}],
        "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": name,
            "availability_status": "available", "integrity_status": "verifiable"}],
        "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"},
            "godspeed.authority_request": {"operation_id": op, "correlation_id": "corr-1",
                "operation_kind": kind, "operation_name": name, "resource_id": name}
        }
    })
}

async fn boot(enabled: bool) -> (tempfile::TempDir, PathBuf) {
    boot_with(enabled, POLICY).await
}

fn fixture(enabled: bool, policy: &str) -> (tempfile::TempDir, ServerConfig) {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("worlds")).unwrap();
    std::fs::write(root.path().join("worlds/demo.sea"), SRC).unwrap();
    // Same logical file name in other directories: a world's identity includes its logical URIs.
    for (dir, src) in [("worlds-edited", EDITED), ("worlds-changed", CHANGED)] {
        std::fs::create_dir_all(root.path().join(dir)).unwrap();
        std::fs::write(root.path().join(dir).join("demo.sea"), src).unwrap();
    }
    std::fs::write(root.path().join("sea-forge-policy.yaml"), policy).unwrap();
    let socket = std::env::temp_dir().join(format!(
        "sf-cep-{}-{}.sock",
        std::process::id(),
        root.path().file_name().unwrap().to_string_lossy()
    ));
    let uid = sea_forge_server::identity::current_uid().expect("uid");
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        identity: IdentityBindings {
            bindings: vec![
                IdentityBinding {
                    uid: Some(uid),
                    actor_id: "cognate-service".into(),
                    roles: vec![ActorRole::Service],
                },
                // Migrated multi-actor controls establish these identities
                // with separate authority-issued private capabilities.
                IdentityBinding {
                    uid: None,
                    actor_id: "op-alice".into(),
                    roles: vec![ActorRole::Operator],
                },
            ],
        },
        cep_authority: CepAuthorityConfig {
            enabled,
            policy: "sea-forge-policy.yaml".into(),
            worlds: vec![
                CepWorldConfig {
                    name: "demo".into(),
                    base: "worlds".into(),
                    entry: "demo.sea".into(),
                    files: vec!["demo.sea".into()],
                    prior_identities: Vec::new(),
                },
                CepWorldConfig {
                    name: "demo".into(),
                    base: "worlds-edited".into(),
                    entry: "demo.sea".into(),
                    files: vec!["demo.sea".into()],
                    prior_identities: Vec::new(),
                },
                CepWorldConfig {
                    name: "demo".into(),
                    base: "worlds-changed".into(),
                    entry: "demo.sea".into(),
                    files: vec!["demo.sea".into()],
                    prior_identities: Vec::new(),
                },
            ],
            approval_ttl_hours: 24,
            settlement: Default::default(),
        },
        ..ServerConfig::default()
    };
    (root, config)
}

async fn boot_with(enabled: bool, policy: &str) -> (tempfile::TempDir, PathBuf) {
    let (root, config) = fixture(enabled, policy);
    let socket = config.socket_path.clone();
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..500 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    (root, socket)
}

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
        tokio::time::timeout(Duration::from_secs(30), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        serde_json::from_str(line.trim()).unwrap()
    }
}

fn verb(envelope: Value, op: &str) -> Value {
    json!({"verb": "authority_request", "envelope": envelope, "request_id": op,
           "actor": {"actor_id": "cognate-service", "role": "service"}})
}

fn decision(response: &Value) -> &str {
    response["envelope"]["extensions"]["godspeed.authority_decision"]["decision"]
        .as_str()
        .unwrap_or("<none>")
}

#[tokio::test]
async fn an_actor_is_required() {
    let (_root, socket) = boot(true).await;
    let mut c = Client::connect(&socket).await;
    let req = json!({"verb": "authority_request", "envelope": request(&world_ref(), "op-1", "action", "run.start"), "request_id": "op-1"});
    let response = c.call(req).await;
    assert_eq!(response["error_class"], "identity_required", "{response}");
}

#[tokio::test]
async fn a_disabled_cell_refuses_every_request() {
    let (_root, socket) = boot(false).await;
    let mut c = Client::connect(&socket).await;
    let response = c
        .call(verb(
            request(&world_ref(), "op-1", "action", "run.start"),
            "op-1",
        ))
        .await;
    assert_eq!(
        response["error_class"], "cep_authority_disabled",
        "{response}"
    );
}

#[tokio::test]
async fn policy_decides_and_the_decision_is_a_cep_envelope() {
    let (_root, socket) = boot(true).await;
    let mut c = Client::connect(&socket).await;
    let world = world_ref();

    let allow = c
        .call(verb(
            request(&world, "op-allow", "action", "run.start"),
            "op-allow",
        ))
        .await;
    assert_eq!(decision(&allow), "allow", "{allow}");
    assert_eq!(allow["envelope"]["envelope_kind"], "authority_decision");
    assert_eq!(allow["envelope"]["scope"]["world_ref"], world);
    assert_eq!(
        allow["envelope"]["lineage_refs"],
        json!(["env-authority_request-op-allow"])
    );
    assert!(allow["ledger_entry"]
        .as_str()
        .is_some_and(|s| !s.is_empty()));

    let gated = c
        .call(verb(
            request(&world, "op-cap", "capability", "tool.ping"),
            "op-cap",
        ))
        .await;
    assert_eq!(decision(&gated), "escalate", "{gated}");
}

#[tokio::test]
async fn the_same_operation_id_replays_the_recorded_decision() {
    let (_root, socket) = boot(true).await;
    let mut c = Client::connect(&socket).await;
    let req = verb(
        request(&world_ref(), "op-replay", "action", "run.start"),
        "op-replay",
    );
    let first = c.call(req.clone()).await;
    let second = c.call(req).await;
    assert_eq!(
        first["ledger_entry"], second["ledger_entry"],
        "{first} / {second}"
    );
    assert_eq!(first["envelope"], second["envelope"]);
}

#[tokio::test]
async fn an_unregistered_world_is_refused() {
    let (_root, socket) = boot(true).await;
    let mut c = Client::connect(&socket).await;
    let ghost = format!("world:demo@sha256:{}", "d".repeat(64));
    let response = c
        .call(verb(
            request(&ghost, "op-ghost", "action", "run.start"),
            "op-ghost",
        ))
        .await;
    assert_eq!(response["error_class"], "cep_world_refused", "{response}");
    assert!(response.get("envelope").is_none());
}

#[tokio::test]
async fn editing_a_world_file_never_serves_a_stale_cached_world() {
    let (root, socket) = boot(true).await;
    let mut c = Client::connect(&socket).await;
    let old = world_ref();
    let first = c
        .call(verb(
            request(&old, "op-before", "action", "run.start"),
            "op-before",
        ))
        .await;
    assert_eq!(decision(&first), "allow", "{first}");

    let edited = format!("// edited\n{SRC}");
    std::fs::write(root.path().join("worlds/demo.sea"), &edited).unwrap();

    // The world the request names no longer exists in this cell...
    let stale = c
        .call(verb(
            request(&old, "op-stale", "action", "run.start"),
            "op-stale",
        ))
        .await;
    assert_eq!(stale["error_class"], "cep_world_refused", "{stale}");
    // ...and the edited file is a new, valid world.
    let fresh = c
        .call(verb(
            request(&world_ref_of(&edited), "op-fresh", "action", "run.start"),
            "op-fresh",
        ))
        .await;
    assert_eq!(decision(&fresh), "allow", "{fresh}");
}

const GATED_POLICY: &str = "version: \"0.1\"\nrules:\n  - name: gated\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n    requires_approval: true\n  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

fn as_actor(mut verb: Value, actor_id: &str, role: &str) -> Value {
    verb["actor"] = json!({"actor_id": actor_id, "role": role});
    verb
}

#[tokio::test]
async fn an_escalation_is_approved_by_someone_else_and_used_once_over_the_socket() {
    let (root, config) = fixture(true, GATED_POLICY);
    let mut cell = authenticated::Cell::start(config).await;
    let socket = cell.socket.clone();
    let service = authenticated::principal("cognate-service", ActorRole::Service);
    let operator = authenticated::principal("op-alice", ActorRole::Operator);
    let mut c = Client::connect(&socket).await;
    let world = world_ref();

    let initial = verb(request(&world, "op-e1", "action", "run.start"), "op-e1");
    let mut requester = cell.connect_for(&service, &initial).await;
    let escalated = requester.call(initial).await;
    assert_eq!(decision(&escalated), "escalate", "{escalated}");
    let approval = escalated["envelope"]["extensions"]["godspeed.authority_decision"]
        ["approval_id"]
        .as_str()
        .expect("approval id")
        .to_string();

    let listed = c.call(json!({"verb": "authority_approvals"})).await;
    assert_eq!(listed["approvals"][0]["state"], "pending", "{listed}");
    assert_eq!(listed["approvals"][0]["approval_id"], approval);

    let revalidation = |op: &str| {
        let mut r = request(&world, op, "action", "run.start");
        r["extensions"]["godspeed.authority_request"]["approval_id"] = json!(approval);
        r["lineage_refs"] = json!(["env-authority_request-op-e1"]);
        verb(r, op)
    };

    // Not approved yet.
    let early = cell.call_for(&service, revalidation("op-e2")).await;
    assert_eq!(early["error_class"], "cep_approval_refused", "{early}");

    // The requester's own service identity may not resolve it.
    let self_approve = cell
        .call_for(&service, as_actor(
            json!({"verb": "authority_approval", "approval_id": approval, "resolution": "approved", "request_id": "r-self"}),
            "cognate-service",
            "service",
        ))
        .await;
    assert_eq!(
        self_approve["error_class"], "cep_approval_refused",
        "{self_approve}"
    );

    // A different, authorized identity can.
    let resolve = as_actor(
        json!({"verb": "authority_approval", "approval_id": approval, "resolution": "approved", "request_id": "r-ok"}),
        "op-alice",
        "operator",
    );
    let journal_path = root.path().join("ledgers/cognate-authority/entries.jsonl");
    let journal_before = std::fs::read(&journal_path).unwrap();
    let switched = requester.call(resolve.clone()).await;
    assert_eq!(switched["error_class"], "identity_authentication_refused");
    let mut forged = Client::connect(&socket).await;
    let impersonated = forged.call(resolve.clone()).await;
    assert_eq!(
        impersonated["error_class"],
        "identity_authentication_refused"
    );
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal_before);
    let pending = c.call(json!({"verb":"authority_approvals"})).await;
    assert_eq!(pending["approvals"][0]["state"], "pending");
    let resolved = cell.call_for(&operator, resolve.clone()).await;
    assert_eq!(resolved["ok"], true, "{resolved}");
    assert_eq!(resolved["resolved_by"], "op-alice");
    let approved_journal = std::fs::read(&journal_path).unwrap();
    // The same resolution request id replays rather than resolving twice.
    assert_eq!(cell.call_for(&operator, resolve).await, resolved);
    assert_eq!(std::fs::read(&journal_path).unwrap(), approved_journal);

    let allowed = cell.call_for(&service, revalidation("op-e3")).await;
    assert_eq!(decision(&allowed), "allow", "{allowed}");
    assert_eq!(
        allowed["envelope"]["extensions"]["godspeed.authority_decision"]["approval_used"],
        true
    );

    let again = cell.call_for(&service, revalidation("op-e4")).await;
    assert_eq!(again["error_class"], "cep_approval_refused", "{again}");
    assert!(again["error"].as_str().unwrap().contains("used"));
    sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test-observer")
        .unwrap()
        .verify()
        .unwrap();
    cell.verify_authentication();
    println!(
        "MIGRATED_CEP_APPROVAL {}",
        json!({"resolved_by":resolved["resolved_by"],"same_connection_switch":switched["error_class"],"public_impersonation":impersonated["error_class"],"journal_verified":true,"replay_journal_unchanged":true})
    );
    cell.shutdown().await;
}

#[tokio::test]
async fn resolution_needs_a_verified_actor_and_a_valid_resolution() {
    let (root, config) = fixture(true, GATED_POLICY);
    let mut cell = authenticated::Cell::start(config).await;
    let operator = authenticated::principal("op-alice", ActorRole::Operator);
    let journal = root.path().join("ledgers/cognate-authority/entries.jsonl");
    assert_eq!(
        std::fs::read(&journal).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    let no_actor_request = json!({"verb": "authority_approval", "approval_id": "apr-x", "resolution": "approved", "request_id": "r1"});
    let no_actor = cell.call_for(&operator, no_actor_request.clone()).await;
    assert_eq!(no_actor["error_class"], "identity_required", "{no_actor}");
    let mut public = Client::connect(&cell.socket).await;
    let unauthenticated = public.call(no_actor_request).await;
    assert_eq!(
        unauthenticated["error_class"],
        "identity_authentication_refused"
    );
    let bad_request = as_actor(
        json!({"verb": "authority_approval", "approval_id": "apr-x", "resolution": "maybe", "request_id": "r2"}),
        "op-alice",
        "operator",
    );
    let forged = public.call(bad_request.clone()).await;
    assert_eq!(forged["error_class"], "identity_authentication_refused");
    assert_eq!(
        std::fs::read(&journal).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    let bad = cell.call_for(&operator, bad_request).await;
    assert_eq!(bad["error_class"], "cep_request_invalid", "{bad}");
    cell.verify_authentication();
    cell.shutdown().await;
}

const EVIDENCE_POLICY: &str = "version: \"0.1\"\nrules:\n  - name: a\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n  - name: e\n    verdict: allow\n    actor_role: service\n    operation_kind: evidence_mutation\n  - name: s\n    verdict: allow\n    actor_role: service\n    operation_kind: settlement_declaration\n";

fn evidence_packet(world: &str, op: &str, direction: &str) -> Value {
    json!({
        "envelope_id": format!("env-evidence_packet-{op}"),
        "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "evidence_packet",
        "created_at": "2026-10-04T12:05:00Z", "created_by": "cognate-realitytrace-adapter",
        "scope": {"evaluation_context": "godspeed-evidence_packet", "world_ref": world},
        "boundary_record": {"scope": "s", "included_sections": ["evidence"], "excluded_sections": [],
            "known_omissions": ["only RealityTrace evidence"], "unknowns": [], "redactions": [],
            "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
        "completeness_status": "partial", "omission_status": "marked",
        "provenance_refs": ["prov-e"], "validation_status": "valid",
        "lineage_refs": [format!("env-execution_trace-{op}")],
        "provenance": [{"provenance_id": "prov-e", "source_system_refs": ["realitytrace"],
            "producer_refs": ["cognate-realitytrace-adapter"], "production_method": "evidence_packaging", "created_at": "2026-10-04T12:05:00Z"}],
        "questions": [{"question_id": "q-1", "question_form": "Did the operation reach the declared state?"}],
        "evidence": [{"evidence_id": format!("ev-{op}"), "evidence_type": "test_result", "question_ref": "q-1",
            "target_entity_ref": "concept:runtime_execution", "direction": direction, "reliability": "high"}],
        "extensions": {
            "cep.profile": {"profile_id": "godspeed.evidence_packet", "profile_version": "1.0.0"},
            "godspeed.evidence_packet": {"operation_id": op,
                "authority_decision_ref": format!("env-authority_decision-{op}"),
                "execution_trace_ref": format!("env-execution_trace-{op}")}
        }
    })
}

#[tokio::test]
async fn evidence_then_settlement_over_the_socket_and_completion_alone_settles_nothing() {
    let (_root, socket) = boot_with(true, EVIDENCE_POLICY).await;
    let mut c = Client::connect(&socket).await;
    let world = world_ref();

    // Execution without evidence: allowed, but there is nothing to settle.
    let allowed = c
        .call(verb(
            request(&world, "op-s1", "action", "run.start"),
            "op-s1",
        ))
        .await;
    assert_eq!(decision(&allowed), "allow", "{allowed}");
    let none = c
        .call(
            json!({"verb": "authority_settle", "operation_id": "op-s1", "request_id": "settle-0",
                     "actor": {"actor_id": "cognate-service", "role": "service"}}),
        )
        .await;
    assert_eq!(none["error_class"], "cep_evidence_refused", "{none}");

    // An actor is required for both new verbs.
    let anon = c
        .call(json!({"verb": "authority_evidence", "envelope": evidence_packet(&world, "op-s1", "supports"), "request_id": "ev-anon"}))
        .await;
    assert_eq!(anon["error_class"], "identity_required", "{anon}");

    // Evidence for an operation nobody allowed is refused.
    let ghost = c
        .call(as_actor(
            json!({"verb": "authority_evidence", "envelope": evidence_packet(&world, "op-ghost", "supports"), "request_id": "ev-ghost"}),
            "cognate-service",
            "service",
        ))
        .await;
    assert_eq!(ghost["error_class"], "cep_evidence_refused", "{ghost}");

    let ev = as_actor(
        json!({"verb": "authority_evidence", "envelope": evidence_packet(&world, "op-s1", "supports"), "request_id": "ev-1"}),
        "cognate-service",
        "service",
    );
    let acked = c.call(ev.clone()).await;
    assert_eq!(acked["ok"], true, "{acked}");
    // The same request id replays rather than recording twice.
    assert_eq!(c.call(ev).await, acked);

    let settle = as_actor(
        json!({"verb": "authority_settle", "operation_id": "op-s1", "request_id": "settle-1"}),
        "cognate-service",
        "service",
    );
    let settled = c.call(settle.clone()).await;
    assert_eq!(settled["status"], "settled", "{settled}");
    assert_eq!(settled["final"], true);
    assert_eq!(settled["envelope"]["envelope_kind"], "settlement_packet");
    assert_eq!(c.call(settle).await, settled);

    // Contradicting evidence rejects.
    c.call(verb(
        request(&world, "op-s2", "action", "run.start"),
        "op-s2",
    ))
    .await;
    let bad = as_actor(
        json!({"verb": "authority_evidence", "envelope": evidence_packet(&world, "op-s2", "contradicts"), "request_id": "ev-2"}),
        "cognate-service",
        "service",
    );
    assert_eq!(c.call(bad).await["ok"], true);
    let rejected = c
        .call(as_actor(
            json!({"verb": "authority_settle", "operation_id": "op-s2", "request_id": "settle-2"}),
            "cognate-service",
            "service",
        ))
        .await;
    assert_eq!(rejected["status"], "rejected", "{rejected}");
}

fn transition(source: &str, target: &str, op: &str, kind: &str) -> Value {
    let mut r = request(source, op, "world_transition", "transition");
    r["extensions"]["godspeed.authority_request"]["resource_id"] = json!(target);
    r["extensions"]["godspeed.world_transition"] = json!({
        "target_world_ref": target, "transition_kind": kind,
        "reason": "adopt the reviewed model", "compatibility": "compatible"
    });
    r
}

#[tokio::test]
async fn stage10_a_world_transition_is_decided_approved_and_listed_over_the_socket() {
    let (root, config) = fixture(true, TRANSITION_POLICY);
    let mut cell = authenticated::Cell::start(config).await;
    let socket = cell.socket.clone();
    let service = authenticated::principal("cognate-service", ActorRole::Service);
    let operator = authenticated::principal("op-alice", ActorRole::Operator);
    let mut c = Client::connect(&socket).await;
    let (base, edited, changed) = (world_ref(), world_ref_of(EDITED), world_ref_of(CHANGED));

    // Nothing has moved yet.
    let none = c.call(json!({"verb": "authority_transitions"})).await;
    assert_eq!(none["transitions"], json!([]), "{none}");

    // A meaning-preserving move follows policy.
    let first_request = verb(
        transition(&base, &edited, "op-t1", "source_edit_only"),
        "op-t1",
    );
    let mut requester = cell.connect_for(&service, &first_request).await;
    let moved = requester.call(first_request).await;
    assert_eq!(decision(&moved), "allow", "{moved}");

    // A move across a semantic change is floored to an escalation although policy says allow.
    let esc = cell
        .call_for(
            &service,
            verb(
                transition(&base, &changed, "op-t2", "semantic_change"),
                "op-t2",
            ),
        )
        .await;
    assert_eq!(decision(&esc), "escalate", "{esc}");
    let approval = esc["envelope"]["extensions"]["godspeed.authority_decision"]["approval_id"]
        .as_str()
        .expect("approval id")
        .to_string();
    assert_eq!(
        c.call(json!({"verb": "authority_transitions"})).await["transitions"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "an escalation is not a move"
    );

    // The requester cannot approve; an operator can; the revalidated request is allowed.
    let denied = cell
        .call_for(&service, as_actor(
            json!({"verb": "authority_approval", "approval_id": approval, "resolution": "approved", "request_id": "r-self"}),
            "cognate-service",
            "service",
        ))
        .await;
    assert!(denied["error_class"].is_string(), "{denied}");
    let resolution = as_actor(
        json!({"verb": "authority_approval", "approval_id": approval, "resolution": "approved", "request_id": "r-ok"}),
        "op-alice",
        "operator",
    );
    let journal_path = root.path().join("ledgers/cognate-authority/entries.jsonl");
    let before = std::fs::read(&journal_path).unwrap();
    let switched = requester.call(resolution.clone()).await;
    assert_eq!(switched["error_class"], "identity_authentication_refused");
    let mut forged = Client::connect(&socket).await;
    let impersonated = forged.call(resolution.clone()).await;
    assert_eq!(
        impersonated["error_class"],
        "identity_authentication_refused"
    );
    assert_eq!(std::fs::read(&journal_path).unwrap(), before);
    let resolved = cell.call_for(&operator, resolution).await;
    assert_eq!(resolved["state"], "approved", "{resolved}");
    let mut again = transition(&base, &changed, "op-t2b", "semantic_change");
    again["extensions"]["godspeed.authority_request"]["approval_id"] = json!(approval);
    again["lineage_refs"] = json!(["env-authority_request-op-t2"]);
    let allowed = cell.call_for(&service, verb(again, "op-t2b")).await;
    assert_eq!(decision(&allowed), "allow", "{allowed}");

    let listed = c.call(json!({"verb": "authority_transitions"})).await;
    let all = listed["transitions"].as_array().unwrap();
    assert_eq!(all.len(), 2, "{listed}");
    assert_eq!(all[0]["record"]["semantic_closure_equal"], true);
    assert_eq!(all[0]["approval_id"], Value::Null);
    assert_eq!(all[1]["record"]["semantic_closure_equal"], false);
    assert_eq!(all[1]["approval_id"], json!(approval));
    assert_eq!(all[1]["record"]["target_world_ref"], json!(changed));

    // A world SEA-Forge never registered fails closed with a stable class.
    let unknown = format!("world:demo@sha256:{}", "e".repeat(64));
    let r = cell
        .call_for(
            &service,
            verb(transition(&base, &unknown, "op-t3", "migration"), "op-t3"),
        )
        .await;
    assert_eq!(r["error_class"], "cep_transition_refused", "{r}");
    sea_forge_ledger::LedgerStream::open(root.path(), "cognate-authority", "test-observer")
        .unwrap()
        .verify()
        .unwrap();
    cell.verify_authentication();
    println!(
        "MIGRATED_CEP_TRANSITION {}",
        json!({"approved_state":resolved["state"],"source_only_allowed":decision(&moved),"semantic_change_allowed":decision(&allowed),"transitions":all.len(),"public_impersonation":impersonated["error_class"],"journal_verified":true})
    );
    cell.shutdown().await;
}

// --- compiler_upgrade lineage: a prior identity is host config, a transition source, never an action site ----

/// The real DomainForge 0.21.0 identity for `SRC` (the authority crate's fixture), as a prior identity.
const PRIOR_0_21_0: &str = include_str!(
    "../../sea-forge-authority/tests/fixtures/identities/demo-tank.domainforge-0.21.0.json"
);
const PRIOR_PATH: &str = "identities/demo.domainforge-0.21.0.json";
const OLD_REF: &str =
    "world:demo@sha256:c2d3978a06cad4f44b8f50ef4a9e570c4800f1ba6b7e4ac76a0128337e0f3439";
const NEW_REF: &str =
    "world:demo@sha256:a71ff6d4d9941a937a2a2b7fd5ed1c1eabb2242b4e6919440927ca1772cbb6ca";

/// The cell with one current world (`SRC`, DomainForge 0.21.1) and `prior_doc` written as its prior identity.
fn upgrade_fixture(prior_doc: &str) -> (tempfile::TempDir, ServerConfig) {
    let (root, mut config) = fixture(true, TRANSITION_POLICY);
    std::fs::create_dir_all(root.path().join("identities")).unwrap();
    std::fs::write(root.path().join(PRIOR_PATH), prior_doc).unwrap();
    config.cep_authority.worlds.truncate(1);
    config.cep_authority.worlds[0].prior_identities = vec![PRIOR_PATH.into()];
    (root, config)
}

#[tokio::test]
async fn a_prior_identity_boots_resolves_and_its_compiler_upgrade_is_allowed_and_listed_over_the_socket(
) {
    let (_root, config) = upgrade_fixture(PRIOR_0_21_0);
    let mut cell = authenticated::Cell::start(config).await;
    let socket = cell.socket.clone();
    let service = authenticated::principal("cognate-service", ActorRole::Service);
    let mut c = Client::connect(&socket).await;
    assert_eq!(
        world_ref(),
        NEW_REF,
        "the current world is the 0.21.1 build"
    );

    // The prior ref resolves: it is the source of the transition.
    let up = verb(
        transition(OLD_REF, NEW_REF, "op-up", "compiler_upgrade"),
        "op-up",
    );
    let mut requester = cell.connect_for(&service, &up).await;
    let moved = requester.call(up).await;
    assert_eq!(decision(&moved), "allow", "{moved}");

    let listed = c.call(json!({"verb": "authority_transitions"})).await;
    let transitions = listed["transitions"].as_array().expect("transitions");
    assert_eq!(transitions.len(), 1, "{listed}");
    assert!(listed.to_string().contains("compiler_upgrade"), "{listed}");
    assert!(listed.to_string().contains(OLD_REF), "{listed}");
}

#[tokio::test]
async fn a_governed_action_in_a_prior_identity_is_refused_over_the_socket() {
    let (_root, config) = upgrade_fixture(PRIOR_0_21_0);
    let mut cell = authenticated::Cell::start(config).await;
    let service = authenticated::principal("cognate-service", ActorRole::Service);
    let act = verb(request(OLD_REF, "op-act", "action", "author"), "op-act");
    let refused = cell.call_for(&service, act).await;
    assert_eq!(refused["error_class"], "cep_world_refused", "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or("")
            .contains("prior identity"),
        "{refused}"
    );
}

#[tokio::test]
async fn a_malformed_prior_identity_fails_closed_with_a_clear_error() {
    let (_root, config) = upgrade_fixture("{\"producer\": \"domainforge-core\", \"surprise\": 1}");
    let socket = config.socket_path.clone();
    let cell = authenticated::Cell::start(config).await;
    let mut c = Client::connect(&socket).await;
    let refused = c.call(json!({"verb": "authority_transitions"})).await;
    assert_eq!(refused["error_class"], "cep_world_config", "{refused}");
    let message = refused["error"].as_str().unwrap_or("");
    assert!(message.contains("prior identity"), "{refused}");
    assert!(message.contains(PRIOR_PATH), "{refused}");
    drop(cell);
}
