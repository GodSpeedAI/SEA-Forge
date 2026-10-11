//! Asset-catalog conformance tests (Task 9, ADR-003 additive): `asset.list`.
//!
//! Fixtures are raw JSON/YAML on disk rather than constructed structs, so a
//! field this projection reads by the wrong name fails here instead of quietly
//! yielding an empty catalog. The probe fixtures in particular mirror exactly
//! what `crate::agent_probe` writes (`plan.json` carrying
//! `Operation::AgentProbe`, `settlement.json` carrying the verdict) — if that
//! record shape changes, endpoint standing silently falls back to `declared`
//! and these tests are what catches it.

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
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
        base_url: Some("https://api.example.com".into()),
        argv: Vec::new(),
        env: Vec::new(),
        credential_ref: None,
        default_model: Some("m1".into()),
        allow_loopback_test: false,
        max_request_bytes: 1024,
        max_response_bytes: 1024,
        timeout_secs: 30,
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

fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn write_json(path: &Path, value: &Value) {
    write(path, &serde_json::to_string_pretty(value).unwrap());
}

/// A probe run exactly as `agent_probe::probe` records it.
fn write_probe_run(
    root: &Path,
    run_id: &str,
    endpoint_ref: &str,
    status: &str,
    basis: Value,
    settled_at: &str,
) {
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
            "settled_at": settled_at,
        }),
    );
}

fn write_template(root: &Path, name: &str, version: &str) {
    write(
        &root.join("templates").join(format!("{name}@{version}.yaml")),
        &format!(
            "name: {name}\nversion: \"{version}\"\ndescription: a template\nparameters: {{}}\nplan:\n  items: []\n"
        ),
    );
}

fn write_registry(root: &Path, entries: Value) {
    write_json(
        &root.join("extensions").join("registry.json"),
        &json!({
            "version": "0.1",
            "updated_at": "2026-07-26T00:00:00+00:00",
            "extensions": entries,
        }),
    );
}

fn assets(body: &Value) -> Vec<Value> {
    body["assets"]
        .as_array()
        .unwrap_or_else(|| panic!("asset.list returned no assets array: {body}"))
        .clone()
}

fn row<'a>(rows: &'a [Value], asset_id: &str) -> &'a Value {
    rows.iter()
        .find(|row| row["asset_id"] == asset_id)
        .unwrap_or_else(|| panic!("no row {asset_id} in {rows:#?}"))
}

/// An empty cell answers with an empty catalog, never a built-in one.
/// `unknown ≠ unavailable` cuts both ways: inventing entries here would be the
/// same failure as hiding real ones.
#[tokio::test]
async fn empty_cell_returns_an_empty_catalog() {
    let (_root, socket) = boot_with(AgentConfig::default()).await;
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    assert_eq!(assets(&body).len(), 0, "{body}");
}

#[tokio::test]
async fn materialized_templates_are_listed_with_their_identity_digest() {
    let (root, socket) = boot_with(AgentConfig::default()).await;
    write_template(root.path(), "repair", "1.0.0");
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    let template = row(&rows, "template:repair@1.0.0");
    assert_eq!(template["kind"], "plan_template");
    assert_eq!(template["name"], "repair");
    assert_eq!(template["version"], "1.0.0");
    assert_eq!(template["standing"], "materialized");
    assert!(
        template["identity_digest"]
            .as_str()
            .is_some_and(|d| d.starts_with("sha256:")),
        "{template}"
    );
    assert!(template.get("blocking_reason").is_none(), "{template}");
}

/// `case.entry_options` silently skips a template it cannot parse, so a broken
/// file reads there as "no such template". The catalog must not repeat that:
/// the file exists, and an operator wondering why it is not offered needs the
/// reason, not its disappearance (epic 4.8, and "absence reported not inferred").
#[tokio::test]
async fn an_unparseable_template_is_reported_not_dropped() {
    let (root, socket) = boot_with(AgentConfig::default()).await;
    write(
        &root.path().join("templates").join("broken@9.9.9.yaml"),
        "this: [is not: a template\n",
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    let broken = row(&rows, "template:broken@9.9.9");
    assert!(
        broken["blocking_reason"]
            .as_str()
            .is_some_and(|r| r.contains("could not be loaded")),
        "{broken}"
    );
    assert!(
        body["unreadable"]
            .as_array()
            .is_some_and(|list| list.iter().any(|v| v == "template:broken@9.9.9")),
        "unreadable must name the source: {body}"
    );
}

/// The ladder's floor. A configured endpoint with no probe evidence is
/// `declared` — never upgraded on the strength of merely being configured,
/// which is exactly what `AgentEndpointConfig::validate` refuses to let a
/// config assert for itself.
#[tokio::test]
async fn a_configured_endpoint_with_no_evidence_stays_declared() {
    let (_root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    let ep = row(&rows, "agent_endpoint:local");
    assert_eq!(ep["standing"], "declared");
    assert_eq!(ep["kind"], "agent_endpoint");
    assert!(ep.get("evidence_refs").is_none(), "{ep}");
    // Unproven is not blocked: a declared endpoint may lawfully be probed.
    assert!(ep.get("blocking_reason").is_none(), "{ep}");
}

#[tokio::test]
async fn an_accepted_probe_promotes_an_endpoint_to_demonstrated() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    write_probe_run(
        root.path(),
        "run-1",
        "local",
        "accepted",
        json!(["authority_allow", "schema_valid_response"]),
        "2026-07-26T10:00:00+00:00",
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    let ep = row(&rows, "agent_endpoint:local");
    assert_eq!(ep["standing"], "demonstrated");
    assert!(
        ep["evidence_refs"]
            .as_array()
            .is_some_and(|refs| refs.iter().any(|r| r == "run:run-1")),
        "the promoting record must travel with the standing: {ep}"
    );
    assert!(ep.get("blocking_reason").is_none(), "{ep}");
}

/// A rejected probe still happened, so the rung is `probed` — reading it as
/// `declared` would erase the attempt, and `demonstrated` would promote a
/// failure. The rejection blocks selection, and says why.
#[tokio::test]
async fn a_rejected_probe_is_probed_and_blocked_with_its_basis() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    write_probe_run(
        root.path(),
        "run-1",
        "local",
        "rejected",
        json!(["agent_endpoint_unreachable"]),
        "2026-07-26T10:00:00+00:00",
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    let ep = row(&rows, "agent_endpoint:local");
    assert_eq!(ep["standing"], "probed");
    let reason = ep["blocking_reason"].as_str().unwrap_or_default();
    assert!(reason.contains("rejected"), "{ep}");
    assert!(reason.contains("agent_endpoint_unreachable"), "{ep}");
}

/// Standing tracks the *current* evidence, not the best evidence ever seen. An
/// endpoint that demonstrated once and has since started failing must not keep
/// a `demonstrated` badge an operator would delegate against.
#[tokio::test]
async fn the_most_recent_probe_decides_standing() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    write_probe_run(
        root.path(),
        "run-old",
        "local",
        "accepted",
        json!(["schema_valid_response"]),
        "2026-07-26T10:00:00+00:00",
    );
    write_probe_run(
        root.path(),
        "run-new",
        "local",
        "rejected",
        json!(["agent_endpoint_timeout"]),
        "2026-07-26T11:00:00+00:00",
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    let ep = row(&rows, "agent_endpoint:local");
    assert_eq!(ep["standing"], "probed", "{ep}");
    assert!(
        ep["evidence_refs"]
            .as_array()
            .is_some_and(|refs| refs.iter().any(|r| r == "run:run-new")),
        "{ep}"
    );
}

/// A probe of a *different* endpoint must not lend its standing to this one.
#[tokio::test]
async fn probe_evidence_does_not_leak_between_endpoints() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local"), endpoint("other")])).await;
    write_probe_run(
        root.path(),
        "run-1",
        "other",
        "accepted",
        json!(["schema_valid_response"]),
        "2026-07-26T10:00:00+00:00",
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    assert_eq!(row(&rows, "agent_endpoint:local")["standing"], "declared");
    assert_eq!(
        row(&rows, "agent_endpoint:other")["standing"],
        "demonstrated"
    );
}

/// Registration into the extension registry happens only after an allowed
/// authority decision, so it is evidence a probe was authorized and attempted —
/// enough for `probed`, never enough for `demonstrated`.
#[tokio::test]
async fn registry_registration_alone_reaches_probed_but_not_demonstrated() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    write_registry(
        root.path(),
        json!([{
            "extension_id": "agent_endpoint_local",
            "version": "0.1.0",
            "descriptor_sha256": format!("sha256:{}", "c".repeat(64)),
            "trust_level": "first_party",
            "status": "active",
            "authority_ref": "01JABCDEF",
        }]),
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    let ep = row(&rows, "agent_endpoint:local");
    assert_eq!(ep["standing"], "probed", "{ep}");
    assert!(
        ep["evidence_refs"]
            .as_array()
            .is_some_and(|refs| refs.iter().any(|r| r == "01JABCDEF")),
        "{ep}"
    );
    // The endpoint is projected once, as an endpoint — not a second time as the
    // runtime adapter it is registered as. Two rows would mean two standings
    // for one thing on one screen.
    assert!(
        !rows
            .iter()
            .any(|row| row["asset_id"] == "extension:agent_endpoint_local@0.1.0"),
        "{rows:#?}"
    );
}

#[tokio::test]
async fn extension_standing_is_the_registrys_own_word_and_quarantine_blocks() {
    let (root, socket) = boot_with(AgentConfig::default()).await;
    write_registry(
        root.path(),
        json!([
            {
                "extension_id": "acme.linter",
                "version": "2.1.0",
                "descriptor_sha256": format!("sha256:{}", "d".repeat(64)),
                "trust_level": "workspace",
                "status": "active",
                "authority_ref": "01JAUTH1",
            },
            {
                "extension_id": "acme.risky",
                "version": "0.1.0",
                "descriptor_sha256": format!("sha256:{}", "e".repeat(64)),
                "trust_level": "quarantined",
                "status": "active",
                "authority_ref": null,
            },
            {
                "extension_id": "acme.old",
                "version": "0.0.1",
                "descriptor_sha256": format!("sha256:{}", "f".repeat(64)),
                "trust_level": "workspace",
                "status": "superseded",
                "authority_ref": null,
            },
        ]),
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);

    let active = row(&rows, "extension:acme.linter@2.1.0");
    assert_eq!(active["standing"], "active");
    assert!(active.get("blocking_reason").is_none(), "{active}");

    // Quarantined *trust* blocks even though the status says active — the two
    // are separate facts and the stricter one governs.
    let risky = row(&rows, "extension:acme.risky@0.1.0");
    assert_eq!(risky["standing"], "active");
    assert!(
        risky["blocking_reason"]
            .as_str()
            .is_some_and(|r| r.contains("quarantined")),
        "{risky}"
    );

    // Superseded keeps the registry's own word rather than being flattened into
    // a generic "unavailable".
    let old = row(&rows, "extension:acme.old@0.0.1");
    assert_eq!(old["standing"], "superseded");
    assert!(
        old["blocking_reason"]
            .as_str()
            .is_some_and(|r| r.contains("superseded")),
        "{old}"
    );
}

/// Three kinds, three vocabularies, one list. If any future change collapses
/// them into a shared ladder this fails — which is the point.
#[tokio::test]
async fn the_three_kinds_keep_disjoint_standing_vocabularies() {
    let (root, socket) = boot_with(agent_with(vec![endpoint("local")])).await;
    write_template(root.path(), "repair", "1.0.0");
    write_registry(
        root.path(),
        json!([{
            "extension_id": "acme.linter",
            "version": "2.1.0",
            "descriptor_sha256": format!("sha256:{}", "d".repeat(64)),
            "trust_level": "workspace",
            "status": "active",
            "authority_ref": null,
        }]),
    );
    let mut client = Client::connect(&socket).await;
    let body = client.call(json!({"verb": "asset_list"})).await;
    let rows = assets(&body);
    assert_eq!(rows.len(), 3, "{rows:#?}");
    assert_eq!(
        row(&rows, "template:repair@1.0.0")["standing"],
        "materialized"
    );
    assert_eq!(row(&rows, "agent_endpoint:local")["standing"], "declared");
    assert_eq!(
        row(&rows, "extension:acme.linter@2.1.0")["standing"],
        "active"
    );
}

#[tokio::test]
async fn asset_list_is_advertised_as_an_inspect_method() {
    let (_root, socket) = boot_with(AgentConfig::default()).await;
    let mut client = Client::connect(&socket).await;
    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    assert!(
        hello["implemented_methods"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m == "asset.list"),
        "{hello}"
    );
    let describe = client.call(json!({"verb": "system_describe"})).await;
    let entry = describe["methods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["method"] == "asset.list")
        .expect("asset.list in describe");
    assert_eq!(entry["class"], "inspect");
}
