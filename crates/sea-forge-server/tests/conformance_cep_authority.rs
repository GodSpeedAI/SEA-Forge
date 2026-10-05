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

const SRC: &str = "@namespace \"t\"\nentity \"Tank\" { key id: uuid }\n";
const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: allow-actions\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_action\n  - name: gate-capabilities\n    verdict: allow\n    actor_role: service\n    operation_kind: cognate_capability\n    requires_approval: true\n";

fn world_ref() -> String {
    world_ref_of(SRC)
}

fn world_ref_of(src: &str) -> String {
    let mut reg = WorldRegistry::new();
    let set = SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![SourceFile {
            uri: "demo.sea".into(),
            sha256: format!("{:x}", Sha256::digest(src.as_bytes())),
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
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("worlds")).unwrap();
    std::fs::write(root.path().join("worlds/demo.sea"), SRC).unwrap();
    std::fs::write(root.path().join("sea-forge-policy.yaml"), POLICY).unwrap();
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
            bindings: vec![IdentityBinding {
                uid,
                actor_id: "cognate-service".into(),
                roles: vec![ActorRole::Service],
            }],
        },
        cep_authority: CepAuthorityConfig {
            enabled,
            policy: "sea-forge-policy.yaml".into(),
            worlds: vec![CepWorldConfig {
                name: "demo".into(),
                base: "worlds".into(),
                entry: "demo.sea".into(),
                files: vec!["demo.sea".into()],
            }],
        },
        ..ServerConfig::default()
    };
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
