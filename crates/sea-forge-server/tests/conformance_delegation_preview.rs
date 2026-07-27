//! Delegation job-contract conformance tests (Task 10, ADR-003 additive):
//! `delegation.preview`.
//!
//! The two things these tests exist to hold down:
//!
//! 1. **The preview agrees with execution.** Its preconditions come from
//!    `delegation::check_preconditions` and its authority action from
//!    `delegation::action_for_delegation`; if either grows a rule the preview
//!    does not see, an operator would be told a delegation is runnable that the
//!    kernel refuses. The precondition tests below drive the *same* inputs
//!    through both and assert they reach the same verdict.
//! 2. **The preview decides nothing.** No case, run, ledger entry, or authority
//!    decision may exist afterwards, and no field may report a verdict.

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind, TranscriptRetentionMode};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

async fn boot_with(agent: AgentConfig) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("sfwp.sock");
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        agent,
        ..ServerConfig::default()
    };
    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..200 {
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
        let stream = UnixStream::connect(socket).await.unwrap();
        let (reader, writer) = stream.into_split();
        Self {
            writer,
            reader: BufReader::new(reader),
        }
    }

    async fn call(&mut self, value: Value) -> Value {
        let line = format!("{value}\n");
        self.writer.write_all(line.as_bytes()).await.unwrap();
        self.writer.flush().await.unwrap();
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(30), self.reader.read_line(&mut line))
            .await
            .expect("read timed out")
            .unwrap();
        assert!(n > 0, "connection closed unexpectedly");
        serde_json::from_str(line.trim()).unwrap()
    }
}

fn endpoint(id: &str) -> AgentEndpointConfig {
    AgentEndpointConfig {
        id: id.into(),
        kind: ProviderKind::OpenAiCompatible,
        base_url: Some("https://api.example.com/v1".into()),
        argv: Vec::new(),
        env: Vec::new(),
        credential_ref: None,
        default_model: Some("m1".into()),
        allow_loopback_test: false,
        max_request_bytes: 64,
        max_response_bytes: 2048,
        timeout_secs: 45,
        status: None,
        transcript_retention: None,
    }
}

fn agent_with(endpoints: Vec<AgentEndpointConfig>) -> AgentConfig {
    AgentConfig {
        endpoints,
        ..AgentConfig::default()
    }
}

/// A well-formed preview request against `local`.
fn preview_request() -> Value {
    json!({
        "verb": "delegation_preview",
        "endpoint": "local",
        "instruction": "summarize the failing test",
        "max_turns": 4,
    })
}

fn blocking(body: &Value) -> Vec<String> {
    body["blocking_reasons"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|reason| reason.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn write_json(path: &Path, value: &Value) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

/// A probe run exactly as `agent_probe::probe` records it — the evidence
/// `asset.list` folds into endpoint standing.
fn write_probe_run(root: &Path, run_id: &str, endpoint_ref: &str, status: &str, basis: Value) {
    let dir = root.join("runs").join(run_id);
    write_json(
        &dir.join("plan.json"),
        &json!({
            "version": "0.2",
            "plan_id": format!("plan-{run_id}"),
            "case_id": format!("case-{run_id}"),
            "run_id": run_id,
            "intent_id": format!("int-{run_id}"),
            "items": [{
                "plan_item_id": "item_agent_probe",
                "name": "agent_probe",
                "operations": [{
                    "kind": "agent_probe",
                    "endpoint_ref": endpoint_ref,
                    "model": "m1",
                    "prompt_sha256": format!("sha256:{}", "a".repeat(64)),
                }],
                "entry_criteria": [],
                "exit_criteria": [],
                "settlement_criteria": {},
                "item_kind": "sandboxed_task",
                "max_instances": 1,
                "depends_on": [],
            }],
        }),
    );
    write_json(
        &dir.join("settlement.json"),
        &json!({
            "version": "0.2",
            "settlement_id": format!("set-{run_id}"),
            "run_id": run_id,
            "status": status,
            "basis": basis,
            "review_required": false,
            "settled_at": "2026-07-27T00:00:00+00:00",
        }),
    );
}

#[tokio::test]
async fn preview_projects_the_contract_a_delegation_would_run_under() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let body = client.call(preview_request()).await;

    assert_eq!(body["endpoint"], "local");
    assert_eq!(body["eligible"], true, "{body}");
    // No probe has been recorded, so the endpoint is declared and nothing more.
    // `eligible` must not be read as `demonstrated`: they answer different
    // questions, and a preview that conflated them would let an operator
    // believe an unproven endpoint had been proven.
    assert_eq!(body["standing"], "declared");

    let contract = &body["contract"];
    assert_eq!(contract["provider_kind"], "openai_compatible");
    assert_eq!(contract["model"]["value"], "m1");
    assert_eq!(contract["model"]["source"], "endpoint_default");
    assert_eq!(contract["max_turns"], 4);
    assert_eq!(contract["max_request_bytes"], 64);
    assert_eq!(contract["max_response_bytes"], 2048);
    assert_eq!(contract["timeout_secs"], 45);
    assert_eq!(contract["required_authority"], "agent_task");
    assert!(
        contract["endpoint_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"),
        "{contract}"
    );
    assert!(
        contract["contract_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"),
        "{contract}"
    );
}

/// Every path under the cell root, for before/after comparison.
fn tree(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            out.push(
                path.strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .into_owned(),
            );
            if path.is_dir() {
                walk(&path, base, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// The invariant the whole method hangs on: an inspect method writes nothing.
/// If a preview ever committed an intent, plan, or authority decision, it would
/// have taken the side effect the operator was still deciding about.
///
/// Compared as a full before/after tree. An earlier version of this test named
/// the directories it expected to stay empty and got one of the names wrong
/// (`ledger`, which the kernel never creates — the real path is `ledgers`), so
/// that third of the assertion could never have failed. A tree diff cannot be
/// fooled by a name the author did not think of.
#[tokio::test]
async fn preview_creates_no_run_case_or_ledger_entry() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;

    // Read once first so any lazily-created scaffolding already exists and the
    // comparison isolates the preview itself.
    let _ = client.call(preview_request()).await;
    let before = tree(root.path());
    let body = client.call(preview_request()).await;
    let after = tree(root.path());

    assert_eq!(body["eligible"], true, "{body}");
    assert_eq!(before, after, "delegation.preview must not touch the cell");
}

/// Naming the gate is not passing it. The preview reports which authority the
/// delegation will need and carries no verdict field at all — a verdict with no
/// ledger entry behind it would be an unrecorded grant.
#[tokio::test]
async fn preview_names_the_required_authority_without_deciding_it() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let body = client.call(preview_request()).await;

    assert_eq!(body["contract"]["required_authority"], "agent_task");
    for forbidden in ["verdict", "decision", "authority_decision", "granted"] {
        assert!(
            body.get(forbidden).is_none() && body["contract"].get(forbidden).is_none(),
            "preview must not report `{forbidden}`: {body}"
        );
    }
}

#[tokio::test]
async fn an_unconfigured_endpoint_has_no_standing_and_no_contract() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let body = client
        .call(json!({
            "verb": "delegation_preview",
            "endpoint": "nowhere",
            "instruction": "hello",
            "max_turns": 1,
        }))
        .await;

    assert_eq!(body["eligible"], false);
    // Absence reported, not inferred: an endpoint that does not exist has no
    // standing word, rather than being demoted to `declared` as though it had
    // been configured but never probed.
    assert!(body.get("standing").is_none(), "{body}");
    assert!(body.get("contract").is_none(), "{body}");
    assert!(
        blocking(&body).iter().any(|r| r.contains("not configured")),
        "{body}"
    );
}

/// An operator repairing a request wants every problem at once. Execution stops
/// at the first; the preview must not.
#[tokio::test]
async fn every_unmet_precondition_is_reported_not_only_the_first() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let body = client
        .call(json!({
            "verb": "delegation_preview",
            "endpoint": "local",
            "instruction": "",
            "max_turns": 0,
        }))
        .await;

    let reasons = blocking(&body);
    assert_eq!(body["eligible"], false, "{body}");
    assert!(
        reasons
            .iter()
            .any(|r| r.contains("instruction must be non-empty")),
        "{reasons:?}"
    );
    assert!(
        reasons.iter().any(|r| r.contains("max_turns must be >= 1")),
        "{reasons:?}"
    );
    // The contract is still shown: the endpoint resolved, and the caps the
    // operator has to work within are exactly what they need to see next.
    assert_eq!(body["contract"]["max_request_bytes"], 64);
}

/// The cap is the endpoint's, and the preview must quote the real one — a
/// preview that guessed a limit would send the operator to a refusal.
#[tokio::test]
async fn an_over_long_instruction_is_blocked_against_the_endpoints_own_cap() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let body = client
        .call(json!({
            "verb": "delegation_preview",
            "endpoint": "local",
            "instruction": "x".repeat(65),
            "max_turns": 1,
        }))
        .await;

    assert_eq!(body["eligible"], false, "{body}");
    assert!(
        blocking(&body)
            .iter()
            .any(|r| r.contains("max_request_bytes (64)")),
        "{body}"
    );
    assert_eq!(body["contract"]["instruction_bytes"], 65);
}

/// The instruction is hashed, never echoed. A projection that repeated the
/// packet would become a second place it lives.
#[tokio::test]
async fn the_instruction_is_reported_by_hash_and_size_only() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let body = client.call(preview_request()).await;

    let contract = &body["contract"];
    assert_eq!(contract["instruction_bytes"], 26);
    assert!(contract["instruction_sha256"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(
        !body.to_string().contains("summarize the failing test"),
        "the instruction text must not be echoed back: {body}"
    );
}

/// Provenance is the point of `ResolvedValue`. A model the caller asked for and
/// one the endpoint supplied are different facts even when they are the same
/// string, and an operator signing off needs to tell them apart.
#[tokio::test]
async fn a_requested_model_is_distinguishable_from_an_endpoint_default() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;

    let defaulted = client.call(preview_request()).await;
    assert_eq!(defaulted["contract"]["model"]["source"], "endpoint_default");
    assert_eq!(defaulted["contract"]["model"]["value"], "m1");

    let asked = client
        .call(json!({
            "verb": "delegation_preview",
            "endpoint": "local",
            "instruction": "go",
            "max_turns": 1,
            "model": "m1",
        }))
        .await;
    assert_eq!(asked["contract"]["model"]["source"], "requested");
    assert_eq!(asked["contract"]["model"]["value"], "m1");
}

/// Retention is the field most likely to be read as "what will be kept", so the
/// preview must resolve it through the same chain `delegate` does. This test is
/// what fails if the two ever diverge again: `delegate` used to pin every
/// socket delegation to `summarized`, ignoring the endpoint entirely.
#[tokio::test]
async fn transcript_retention_is_resolved_through_the_endpoint_then_the_cell() {
    let mut retaining = endpoint("local");
    retaining.transcript_retention = Some(TranscriptRetentionMode::Full);
    let mut agent = agent_with(vec![retaining, endpoint("other")]);
    agent.transcript_retention = TranscriptRetentionMode::Summarized;
    let (_root, socket) = boot_with(agent).await;
    let mut client = Client::connect(&socket).await;

    let from_endpoint = client.call(preview_request()).await;
    assert_eq!(
        from_endpoint["contract"]["transcript_retention"],
        json!({"value": "full", "source": "endpoint_default"}),
        "{from_endpoint}"
    );

    let from_cell = client
        .call(json!({
            "verb": "delegation_preview",
            "endpoint": "other",
            "instruction": "go",
            "max_turns": 1,
        }))
        .await;
    assert_eq!(
        from_cell["contract"]["transcript_retention"],
        json!({"value": "summarized", "source": "cell_default"}),
        "{from_cell}"
    );
}

/// The preview reads standing from the catalog rather than deriving its own, so
/// a rejected probe blocks selection here for the same reason and with the same
/// words `/assets` shows. Two views disagreeing about usability would make one
/// of them a liar.
#[tokio::test]
async fn a_rejected_probe_blocks_the_preview_the_way_it_blocks_the_catalog() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    write_probe_run(
        root.path(),
        "run-1",
        "local",
        "rejected",
        json!(["endpoint returned 503"]),
    );
    let mut client = Client::connect(&socket).await;

    let preview = client.call(preview_request()).await;
    assert_eq!(preview["eligible"], false, "{preview}");
    assert_eq!(preview["standing"], "probed");
    assert!(
        blocking(&preview)
            .iter()
            .any(|r| r.contains("endpoint returned 503")),
        "{preview}"
    );

    let catalog = client.call(json!({"verb": "asset_list"})).await;
    let row = catalog["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["asset_id"] == "agent_endpoint:local")
        .expect("endpoint is in the catalog");
    assert_eq!(row["standing"], preview["standing"]);
    assert_eq!(
        json!([row["blocking_reason"].clone()]),
        json!(blocking(&preview))
    );
}

/// An accepted probe is evidence, and the preview carries the same run
/// reference the catalog does so the claim stays resolvable.
#[tokio::test]
async fn an_accepted_probe_carries_its_evidence_into_the_preview() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    write_probe_run(
        root.path(),
        "run-7",
        "local",
        "accepted",
        json!(["probe ok"]),
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(preview_request()).await;

    assert_eq!(body["standing"], "demonstrated");
    assert_eq!(body["eligible"], true, "{body}");
    assert_eq!(body["evidence_refs"], json!(["run:run-7"]));
}

/// The digest is what tells an operator whether the contract they inspected is
/// the one that would run. If it did not move with the endpoint descriptor,
/// it would certify a contract that had already changed.
#[tokio::test]
async fn the_contract_digest_moves_when_the_delegation_would_differ() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let base = client.call(preview_request()).await;
    let base_digest = base["contract"]["contract_digest"]
        .as_str()
        .unwrap()
        .to_string();

    let more_turns = client
        .call(json!({
            "verb": "delegation_preview",
            "endpoint": "local",
            "instruction": "summarize the failing test",
            "max_turns": 5,
        }))
        .await;
    assert_ne!(
        more_turns["contract"]["contract_digest"].as_str().unwrap(),
        base_digest
    );

    // A different endpoint descriptor is a different contract even at identical
    // request values, because the authority action carries the descriptor hash.
    let mut wider = endpoint("local");
    wider.timeout_secs = 90;
    let (_other_root, other_socket) = boot_with(agent_with(vec![wider])).await;
    let mut other = Client::connect(&other_socket).await;
    let retimed = other.call(preview_request()).await;
    assert_ne!(
        retimed["contract"]["contract_digest"].as_str().unwrap(),
        base_digest
    );
}

/// A cell with no token cap must say so by omission, not by reporting `0` —
/// a zero budget and an absent one are opposite instructions to the runner.
#[tokio::test]
async fn an_absent_token_budget_is_omitted_rather_than_zeroed() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;

    let unbounded = client.call(preview_request()).await;
    assert!(
        unbounded["contract"].get("token_budget").is_none(),
        "{unbounded}"
    );

    let bounded = client
        .call(json!({
            "verb": "delegation_preview",
            "endpoint": "local",
            "instruction": "go",
            "max_turns": 1,
            "token_budget": 5000,
        }))
        .await;
    assert_eq!(bounded["contract"]["token_budget"], 5000);
}

#[tokio::test]
async fn delegation_preview_is_advertised_as_an_inspect_method() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    assert!(
        hello["implemented_methods"]
            .as_array()
            .unwrap()
            .iter()
            .any(|method| method == "delegation.preview"),
        "{hello}"
    );

    let describe = client.call(json!({"verb": "system_describe"})).await;
    let descriptor = describe["methods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["method"] == "delegation.preview")
        .expect("delegation.preview is described");
    assert_eq!(descriptor["class"], "inspect");
}
