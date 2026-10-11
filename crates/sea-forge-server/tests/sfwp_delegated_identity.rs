//! Delegated actor identity for the gateway principal (plan
//! casework-live-wiring-production T02, resolves GAP-B; operator decision D-2).
//!
//! What these hold down, over the real Unix-socket server on a real cell:
//!
//! 1. **The gateway acts, the end user is recorded.** A request carrying
//!    `on_behalf_of` is attributed to the end user in durable kernel truth
//!    (`case-events.jsonl` `actor_id`), while the delegation audit ledger names
//!    both principals. Two end users behind one gateway stay distinct.
//! 2. **A non-gateway uid cannot delegate.** A connection whose uid is not the
//!    configured gateway uid is refused with `identity_delegation_refused`,
//!    `no_side_effect: true`, and no case directory created.
//! 3. **An unconfigured cell delegates nothing.** With no `gateway:` section,
//!    every `on_behalf_of` is refused — the fail-closed default.
//! 4. **Separation of duty compares end users, never the gateway.** Item
//!    proposer A and executor B both arrive through the same gateway; A's
//!    attempt to resolve the approval its own proposal gated is refused as
//!    `sod_violation`. If the server attributed to the gateway, there would be
//!    no A/B distinction left to refuse. The positive is held down too: B
//!    resolving A's delegated proposal is allowed, both principals durable —
//!    the SoD fields name end users, the delegation audit names the pair.
//! 5. **The allowlist bounds delegation, not the binding table.** A bound
//!    actor the operator never allowlisted is as unreachable through
//!    `on_behalf_of` as an unknown one, and a uid bound as a plain operator
//!    (a trusted direct principal) is refused just the same — with the cell's
//!    ledgers and approvals byte-identical around every refusal.
//!
//! Which uid the suite runs as (the prereg's known confound): every connection
//! from this test process presents *its own* uid, so the "gateway" cases bind
//! that uid, and the "not the gateway" case binds a different uid to the same
//! gateway actor. The end-user bindings deliberately sit at a uid that is
//! neither of those — which is the point: delegation resolves the actor's
//! standing from its binding, not from who opened the socket.

use sea_forge_core::types::{TraceEvent, TraceKind};
use sea_forge_server::{identity, run, ServerConfig};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

/// The invoking uid — the uid every connection in this suite presents.
fn test_uid() -> u32 {
    identity::current_uid().expect("invoking uid must be readable")
}

/// A uid that is certainly not this process's: a gateway actor bound to it can
/// never speak for our connections.
fn other_uid() -> u32 {
    let uid = test_uid();
    if uid == 4242 {
        4243
    } else {
        4242
    }
}

// ---------------------------------------------------------------------------
// Cell seeds
// ---------------------------------------------------------------------------

/// A cell whose gateway is *this* process (so delegation is reachable), with
/// two end-user actors bound at a uid no connection presents. `operator_c`
/// is bound — it holds a role — but is deliberately outside the
/// `delegable_actors` allowlist: being bound establishes an actor's
/// standing; only the allowlist lets the gateway speak for them.
fn seed_gateway_cell(root: &Path) {
    let uid = test_uid();
    let users = other_uid();
    fs::write(
        root.join("server.yaml"),
        format!(
            "# T02 delegated-identity cell: the gateway principal is uid {uid};\n# the end users are bound at uid {users}, which no connection presents.\nidentity:\n  bindings:\n    - uid: {uid}\n      actor_id: gateway\n      roles: [\"service\"]\n    - uid: {users}\n      actor_id: operator_a\n      roles: [\"operator\"]\n    - uid: {users}\n      actor_id: operator_b\n      roles: [\"operator\"]\n    - uid: {users}\n      actor_id: operator_c\n      roles: [\"operator\"]\n    - uid: {users}\n      actor_id: security_officer\n      roles: [\"R-SO\"]\ngateway:\n  uid: {uid}\n  actor: gateway\n  delegable_actors: [operator_a, operator_b, security_officer]\n"
        ),
    )
    .expect("write server.yaml");
}

/// A cell whose gateway sits at a uid no connection here presents, while
/// *this* process's uid is bound as a plain operator — a fully legitimate
/// direct principal that must still not be able to speak for anyone else.
fn seed_operator_bound_cell(root: &Path) {
    let uid = test_uid();
    let gateway_uid = other_uid();
    fs::write(
        root.join("server.yaml"),
        format!(
            "identity:\n  bindings:\n    - uid: {uid}\n      actor_id: operator_local\n      roles: [\"operator\"]\n    - uid: {gateway_uid}\n      actor_id: gateway\n      roles: [\"service\"]\n    - uid: {gateway_uid}\n      actor_id: operator_b\n      roles: [\"operator\"]\ngateway:\n  uid: {gateway_uid}\n  actor: gateway\n  delegable_actors: [operator_b]\n"
        ),
    )
    .expect("write server.yaml");
}

/// The same rules, but with the gateway bound to a uid no connection here
/// presents: delegation is configured, and this peer is not its principal.
fn seed_foreign_gateway_cell(root: &Path) {
    let uid = other_uid();
    fs::write(
        root.join("server.yaml"),
        format!(
            "identity:\n  bindings:\n    - uid: {uid}\n      actor_id: gateway\n      roles: [\"service\"]\n    - uid: 3101\n      actor_id: operator_local\n      roles: [\"operator\"]\ngateway:\n  uid: {uid}\n  actor: gateway\n  delegable_actors: [operator_local]\n"
        ),
    )
    .expect("write server.yaml");
}

/// The T03 seed: one operator binding, and no `gateway:` section at all.
fn seed_ungoverned_cell(root: &Path) {
    let uid = test_uid();
    fs::write(
        root.join("server.yaml"),
        format!(
            "identity:\n  bindings:\n    - uid: {uid}\n      actor_id: operator_local\n      roles: [\"operator\"]\n"
        ),
    )
    .expect("write server.yaml");
}

async fn boot_with(seed: fn(&Path)) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    // Short, because a deep temp path overflows `sun_path` (cell contract).
    let socket = std::env::temp_dir().join(format!(
        "sf-t02-{}-{}.sock",
        std::process::id(),
        root.path().file_name().unwrap().to_string_lossy()
    ));
    let mut config = ServerConfig::load(&root.path().join("server.yaml"))
        .expect("seeded server.yaml must load as a real ServerConfig");
    config.root = root.path().to_path_buf();
    config.socket_path = socket.clone();

    tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..500 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(socket.exists(), "server socket must appear");
    (root, socket)
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

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

fn line_nonce() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Attach the D-2 wire shape to a request body: the gateway's own claim plus
/// the end user it speaks for. Splice rather than template, so each test body
/// stays readable as the verb it exercises.
fn with_delegation(mut body: Value, actor: &str, role: &str) -> Value {
    let object = body.as_object_mut().expect("request bodies are objects");
    object.insert(
        "actor".into(),
        json!({"actor_id": "gateway", "role": "service"}),
    );
    object.insert(
        "on_behalf_of".into(),
        json!({"actor_id": actor, "role": role}),
    );
    body
}

/// A parked case: `e2e-sentry-chain`'s first item is manual-activation, so the
/// commit both proves the delegated write and leaves a case to mutate.
fn seeded_case_commit(actor: &str) -> Value {
    json!({
        "verb": "case_commit",
        "template_ref": "e2e-sentry-chain@0.1.0",
        "params": {
            "dataset_name": "orders-q3",
            "dataset_label": "t02",
            "max_rows": "25",
            "out_dir": "work"
        },
        "policy": "authority/active-policy.json",
        "entity": actor,
        "process": "sfwp_t02_tests",
        "request_id": format!("req-t02-commit-{actor}-{}", line_nonce()),
    })
}

fn case_events(root: &Path, case_id: &str) -> Vec<TraceEvent> {
    let path = root.join("cases").join(case_id).join("case-events.jsonl");
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// The case ledger's entries: the durable record of who wrote the case
/// (`writer_identity_ref` is the acting principal, set at open).
fn case_ledger_entries(root: &Path, case_id: &str) -> Vec<sea_forge_ledger::types::LedgerEntry> {
    let ledger =
        sea_forge_ledger::LedgerStream::open(root, format!("case-{case_id}"), "sfwp_t02_tests")
            .expect("open case ledger");
    ledger.read_entries().expect("read case ledger")
}

/// Every delegation audit record written so far, in append order.
fn delegation_audit(root: &Path) -> Vec<Value> {
    let ledger = sea_forge_ledger::LedgerStream::open(root, "delegation-audit", "sfwp_t02_tests")
        .expect("open delegation audit ledger");
    ledger
        .read_entries()
        .expect("read delegation audit ledger")
        .into_iter()
        .map(|entry| entry.payload)
        .collect()
}

/// How many cases exist (`cases/` is created on the first commit).
fn case_count(root: &Path) -> usize {
    fs::read_dir(root.join("cases"))
        .map(|entries| entries.count())
        .unwrap_or(0)
}

/// A deterministic content digest of everything a refusal must leave
/// untouched: `approvals.jsonl` and every ledger stream file under
/// `<root>/ledgers`. Sorted relative paths feed the hash before their
/// contents, so equal digests prove byte-identical files, not merely equal
/// content — a rename or an extra empty file changes the digest.
///
/// Printed by the teeth tests (before/after the refused request) so the
/// recorded transcripts carry the no-side-effect proof, not only the
/// assertion of it.
fn cell_digest(root: &Path) -> String {
    use sha2::{Digest, Sha256};
    fn walk(dir: &Path, paths: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, paths);
            } else {
                paths.push(path);
            }
        }
    }
    let mut paths: Vec<PathBuf> = vec![root.join("approvals.jsonl")];
    walk(&root.join("ledgers"), &mut paths);
    paths.retain(|path| path.is_file());
    paths.sort();
    let mut hasher = Sha256::new();
    for path in &paths {
        hasher.update(
            path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .as_bytes(),
        );
        hasher.update(fs::read(path).unwrap_or_default());
    }
    format!("{:x}", hasher.finalize())
}

/// Everything a refused protected request must leave untouched, printed for
/// the evidence transcript: the digest above, the case count, and whether a
/// delegation-audit stream exists at all.
fn print_no_side_effect(root: &Path, label: &str, digest: &str) {
    println!(
        "[tooth] {label}: ledgers/approvals digest={digest} cases={} delegation_audit={}",
        case_count(root),
        root.join("ledgers")
            .join("delegation-audit")
            .join("entries.jsonl")
            .exists()
    );
}

/// The cell must carry the E2E templates and policy the commit path reads.
fn install_e2e_fixtures(root: &Path) {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cells/e2e");
    let templates = root.join("templates");
    fs::create_dir_all(&templates).expect("create templates dir");
    for entry in fs::read_dir(fixtures.join("templates"))
        .expect("fixtures templates dir must exist")
        .flatten()
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            fs::copy(entry.path(), templates.join(entry.file_name())).expect("copy template");
        }
    }
    let authority = root.join("authority");
    fs::create_dir_all(&authority).expect("create authority dir");
    fs::copy(
        fixtures.join("policy.yaml"),
        authority.join("active-policy.json"),
    )
    .expect("install E2E policy");
}

// ---------------------------------------------------------------------------
// Claim 1: the gateway acts, the end user is recorded
// ---------------------------------------------------------------------------

/// Two end users behind one gateway are attributed distinctly: same
/// connection, same gateway claim, different `on_behalf_of`.
#[tokio::test]
async fn two_end_users_behind_one_gateway_are_attributed_distinctly() {
    let (root, socket) = boot_with(seed_gateway_cell).await;
    install_e2e_fixtures(root.path());
    let mut client = Client::connect(&socket).await;

    let mut case_ids = Vec::new();
    for actor in ["operator_a", "operator_b"] {
        // `entity` names the *end user*, not the gateway: the identity gate
        // checks it against the effective actor, which is what keeps a gateway
        // from writing work under someone else's name.
        let response = client
            .call(with_delegation(
                seeded_case_commit(actor),
                actor,
                "operator",
            ))
            .await;
        assert!(response.get("error").is_none(), "{actor}: {response}");
        case_ids.push(response["case_id"].as_str().expect("case_id").to_owned());
    }

    // Durable kernel truth: who wrote each case. `CaseCreated` is a
    // case-*engine* event (the kernel hardcodes its actor), so attribution is
    // asserted on the carriers that carry the acting principal: the case
    // ledger's own writer identity, and the intent the case records.
    let mut seen = Vec::new();
    for (actor, case_id) in ["operator_a", "operator_b"].into_iter().zip(&case_ids) {
        let entries = case_ledger_entries(root.path(), case_id);
        assert!(!entries.is_empty(), "case {case_id} must have a ledger");
        for entry in &entries {
            assert_eq!(
                entry.writer_identity_ref, actor,
                "case {case_id} record {} must be written as {actor}",
                entry.record_kind
            );
            let rendered = serde_json::to_string(&entry.payload).unwrap();
            assert!(
                !rendered.contains("\"actor_id\":\"gateway\"")
                    && !rendered.contains("\"principal\":\"gateway\""),
                "case {case_id} {} must not name the gateway: {rendered}",
                entry.record_kind
            );
        }

        let case_json: Value = serde_json::from_str(
            &fs::read_to_string(root.path().join("cases").join(case_id).join("case.json"))
                .expect("case.json must exist"),
        )
        .unwrap();
        assert_eq!(case_json["intent"]["actor_id"], actor, "case {case_id}");
        seen.push(case_json["intent"]["actor_id"].as_str().unwrap().to_owned());
    }
    assert_eq!(
        seen,
        vec!["operator_a", "operator_b"],
        "distinct attribution"
    );

    // Both principals, durably, once per admitted delegated request.
    let audit = delegation_audit(root.path());
    assert_eq!(
        audit.len(),
        2,
        "one audit record per delegated request: {audit:?}"
    );
    for (record, actor) in audit.iter().zip(["operator_a", "operator_b"]) {
        assert_eq!(record["effective_actor_id"], actor, "{record}");
        assert_eq!(record["effective_role"], "operator", "{record}");
        assert_eq!(record["gateway_actor_id"], "gateway", "{record}");
        assert_eq!(record["gateway_uid"], test_uid(), "{record}");
        assert_eq!(record["verb"], "case_commit", "{record}");
        assert!(record["request_id"].is_string(), "{record}");
    }
}

/// The delegation machinery does not widen the gateway's standing: a role the
/// end user does not hold is refused, and nothing is written.
#[tokio::test]
async fn delegation_may_not_widen_the_end_users_roles() {
    let (root, socket) = boot_with(seed_gateway_cell).await;
    install_e2e_fixtures(root.path());
    let mut client = Client::connect(&socket).await;

    let before = case_count(root.path());
    let digest_before = cell_digest(root.path());
    println!("[tooth] scenario: the gateway itself sends on_behalf_of claiming role `operator` for `security_officer`, who is allowlisted but holds only R-SO");
    print_no_side_effect(root.path(), "before", &digest_before);
    // `security_officer` holds only R-SO; claiming operator for them is a
    // widening attempt, refused as a delegation failure rather than as an
    // unbound actor.
    let refused = client
        .call(with_delegation(
            seeded_case_commit("security_officer"),
            "security_officer",
            "operator",
        ))
        .await;

    println!("[tooth] refusal (verbatim): {refused}");
    let digest_after = cell_digest(root.path());
    print_no_side_effect(root.path(), "after", &digest_after);
    assert_eq!(
        refused["error_class"], "identity_delegation_refused",
        "{refused}"
    );
    assert_eq!(refused["no_side_effect"], true, "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("exceeds the roles bound to `security_officer`"),
        "{refused}"
    );
    assert_eq!(case_count(root.path()), before, "a refusal writes no case");
    assert_eq!(
        digest_after, digest_before,
        "a refusal must leave ledgers and approvals byte-identical"
    );
    assert!(
        !root
            .path()
            .join("ledgers")
            .join("delegation-audit")
            .exists(),
        "a refused delegation writes no audit record"
    );
}

// ---------------------------------------------------------------------------
// Teeth: a non-gateway uid cannot delegate; an unconfigured cell delegates
// nothing
// ---------------------------------------------------------------------------

/// Tooth 1 (prereg): `on_behalf_of` from a uid that is not the configured
/// gateway uid is refused, with no case and no audit record.
#[tokio::test]
async fn a_non_gateway_uid_cannot_send_on_behalf_of() {
    let (root, socket) = boot_with(seed_foreign_gateway_cell).await;
    install_e2e_fixtures(root.path());
    let mut client = Client::connect(&socket).await;

    let before = case_count(root.path());
    let digest_before = cell_digest(root.path());
    println!("[tooth] scenario: uid {} (this connection, bound as no actor in this cell) sends on_behalf_of for the delegable actor `operator_local`", test_uid());
    print_no_side_effect(root.path(), "before", &digest_before);
    // The delegation *rules* are configured and `operator_local` is delegable:
    // the only thing wrong is that this connection is not the gateway.
    let refused = client
        .call(with_delegation(
            seeded_case_commit("operator_local"),
            "operator_local",
            "operator",
        ))
        .await;

    println!("[tooth] refusal (verbatim): {refused}");
    let digest_after = cell_digest(root.path());
    print_no_side_effect(root.path(), "after", &digest_after);
    assert_eq!(
        refused["error_class"], "identity_delegation_refused",
        "{refused}"
    );
    assert_eq!(refused["no_side_effect"], true, "{refused}");
    assert_eq!(case_count(root.path()), before, "a refusal writes no case");
    assert_eq!(
        digest_after, digest_before,
        "a refusal must leave ledgers and approvals byte-identical"
    );
    assert!(
        !root
            .path()
            .join("ledgers")
            .join("delegation-audit")
            .exists(),
        "a refused delegation writes no audit record"
    );
}

/// Tooth 1, precise form (prereg): the sender is not some anonymous uid — it
/// is bound as a plain `operator`, a principal the cell fully trusts for its
/// own direct work. Delegation is a property of the *gateway* binding, never
/// of being bound at all, and the refusal leaves the cell byte-identical.
#[tokio::test]
async fn an_operator_bound_uid_cannot_send_on_behalf_of() {
    let (root, socket) = boot_with(seed_operator_bound_cell).await;
    install_e2e_fixtures(root.path());
    let mut client = Client::connect(&socket).await;

    let before = case_count(root.path());
    let digest_before = cell_digest(root.path());
    println!("[tooth] scenario: uid {} (bound in this cell as `operator_local`/operator, NOT the gateway) sends on_behalf_of for the delegable actor `operator_b`", test_uid());
    print_no_side_effect(root.path(), "before", &digest_before);
    // The sender claims itself — an actor its uid legitimately holds — and
    // attaches `on_behalf_of` for `operator_b`, which the (elsewhere-bound)
    // gateway may legitimately speak for. Only the sender is wrong.
    let mut request = seeded_case_commit("operator_b");
    let object = request.as_object_mut().expect("request body");
    object.insert(
        "actor".into(),
        json!({"actor_id": "operator_local", "role": "operator"}),
    );
    object.insert(
        "on_behalf_of".into(),
        json!({"actor_id": "operator_b", "role": "operator"}),
    );
    let refused = client.call(request).await;

    println!("[tooth] refusal (verbatim): {refused}");
    let digest_after = cell_digest(root.path());
    print_no_side_effect(root.path(), "after", &digest_after);
    assert_eq!(
        refused["error_class"], "identity_delegation_refused",
        "{refused}"
    );
    assert_eq!(refused["no_side_effect"], true, "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("is not the configured gateway uid"),
        "{refused}"
    );
    assert_eq!(case_count(root.path()), before, "a refusal writes no case");
    assert_eq!(
        digest_after, digest_before,
        "a refusal must leave ledgers and approvals byte-identical"
    );
    assert!(
        !root
            .path()
            .join("ledgers")
            .join("delegation-audit")
            .exists(),
        "a refused delegation writes no audit record"
    );
}

/// Tooth 2 (prereg): with no `gateway:` section, every `on_behalf_of` is
/// refused — the fail-closed default for a cell that configures nothing.
#[tokio::test]
async fn an_ungoverned_cell_refuses_every_delegation() {
    let (root, socket) = boot_with(seed_ungoverned_cell).await;
    install_e2e_fixtures(root.path());
    let mut client = Client::connect(&socket).await;

    let before = case_count(root.path());
    let digest_before = cell_digest(root.path());
    println!("[tooth] scenario: server.yaml has no `gateway:` section at all; this connection sends on_behalf_of for `operator_local`");
    print_no_side_effect(root.path(), "before", &digest_before);
    let refused = client
        .call(with_delegation(
            seeded_case_commit("operator_local"),
            "operator_local",
            "operator",
        ))
        .await;

    println!("[tooth] refusal (verbatim): {refused}");
    let digest_after = cell_digest(root.path());
    print_no_side_effect(root.path(), "after", &digest_after);
    assert_eq!(
        refused["error_class"], "identity_delegation_refused",
        "{refused}"
    );
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("configures no `gateway` principal"),
        "{refused}"
    );
    assert_eq!(case_count(root.path()), before, "a refusal writes no case");
    assert_eq!(
        digest_after, digest_before,
        "a refusal must leave ledgers and approvals byte-identical"
    );

    // ... while the same connection's *direct* claim still works: refusing
    // delegation must not disable the actor the cell does bind.
    let mut direct = seeded_case_commit("operator_local");
    direct.as_object_mut().unwrap().insert(
        "actor".into(),
        json!({"actor_id": "operator_local", "role": "operator"}),
    );
    let committed = client.call(direct).await;
    assert!(committed.get("error").is_none(), "{committed}");
    assert_eq!(
        case_count(root.path()),
        before + 1,
        "the direct path still works"
    );
}

/// `identity.get` reports the effective actor for a delegated inspect, rather
/// than leaving the caller to infer it (D-2).
#[tokio::test]
async fn identity_get_reports_the_effective_actor() {
    let (_root, socket) = boot_with(seed_gateway_cell).await;
    let mut client = Client::connect(&socket).await;

    let view = client
        .call(with_delegation(
            json!({"verb": "identity_get"}),
            "operator_a",
            "operator",
        ))
        .await;
    assert_eq!(view["effective_actor"]["actor_id"], "operator_a", "{view}");
    assert_eq!(view["effective_actor"]["role"], "operator", "{view}");
    assert_eq!(
        view["effective_actor"]["delegated_by"]["actor_id"], "gateway",
        "{view}"
    );
    assert_eq!(
        view["effective_actor"]["delegated_by"]["uid"],
        test_uid(),
        "{view}"
    );

    // A widening claim reports the refusal rather than an effective actor:
    // the inspect shows what *would* happen, it does not grant it.
    let refused = client
        .call(with_delegation(
            json!({"verb": "identity_get"}),
            "security_officer",
            "operator",
        ))
        .await;
    assert!(
        refused["effective_actor"].is_null(),
        "no effective actor for a refused delegation: {refused}"
    );
    assert_eq!(
        refused["refusal"]["error_class"], "identity_delegation_refused",
        "{refused}"
    );

    // ... and a plain inspect is unchanged: the connection's own claimable set.
    let plain = client.call(json!({"verb": "identity_get"})).await;
    assert!(plain["effective_actor"].is_null(), "{plain}");
    assert!(
        plain["available"]
            .as_array()
            .unwrap()
            .iter()
            .any(|actor| actor["actor_id"] == "gateway"),
        "{plain}"
    );
}

// ---------------------------------------------------------------------------
// Claim 4: separation of duty compares end users, never the gateway
// ---------------------------------------------------------------------------

const ESCALATE_RULES: &str = "  - name: escalate-command\n    verdict: escalate\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";
const ADD_ITEM_RULE: &str = "  - name: allow-propose\n    verdict: allow\n    actor_role: operator\n    operation_kind: discretionary_task_add\n";
const APPROVAL_RESOLUTION_RULE: &str = "  - name: allow-approval-resolution\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n";

/// Arguments that make the libtest harness exit 0 without running anything.
const EXIT_ZERO: &[&str] = &["--exact", "__no_test_matches_this_name__"];

fn write_policy(root: &Path, name: &str, rules: &str) -> String {
    fs::write(
        root.join(name),
        format!("version: \"0.1\"\nrules:\n{rules}"),
    )
    .unwrap();
    name.to_string()
}

/// An absolute path named `sea-forge` that resolves to *this* test binary —
/// the intersection of the two argv0 constraints (policy match on the file
/// name, `untrusted_executable` on the canonical path) that makes a real
/// sandboxed episode executable in a test.
fn trusted_self_executable(dir: &Path) -> PathBuf {
    let link = dir.join("sea-forge");
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(std::env::current_exe().expect("current_exe"), &link)
        .expect("link the test binary as sea-forge");
    link
}

fn execute_operation(dir: &Path) -> Value {
    let mut argv = vec![trusted_self_executable(dir).to_string_lossy().into_owned()];
    argv.extend(EXIT_ZERO.iter().map(|arg| (*arg).to_owned()));
    json!({"kind": "execute_command", "argv": argv, "cwd": "."})
}

/// A plan whose single SandboxedTask carries `manual_activation`: submit
/// enables it and parks, which is the state `item.execute` exists to drive.
fn manual_sandbox_plan(dir: &Path) -> Value {
    json!({
        "version": "0.2",
        "plan_id": "plan_t02_sod",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_t02_sod",
        "items": [{
            "plan_item_id": "task",
            "name": "sandboxed",
            "operations": [execute_operation(dir)],
            "entry_criteria": [], "exit_criteria": [],
            "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
            "item_kind": "sandboxed_task",
            "markers": {"required": true, "manual_activation": true},
            "max_instances": 1,
            "depends_on": []
        }],
        "template_ref": null,
        "job_contract_ref": null
    })
}

/// The discretionary item `operator_a` proposes in the SoD test.
fn discretionary_item(dir: &Path) -> Value {
    json!({
        "plan_item_id": "disc",
        "name": "discretionary",
        "operations": [execute_operation(dir)],
        "entry_criteria": [], "exit_criteria": [],
        "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
        "item_kind": "sandboxed_task",
        "markers": {"required": false},
        "max_instances": 1,
        "depends_on": []
    })
}

/// operator_a proposes a discretionary sandboxed item; operator_b executes it
/// under an escalating policy (so the episode opens an approval whose requester
/// is operator_b); operator_a then tries to resolve that approval and is
/// refused as an SoD violation, with `approvals.jsonl` unchanged.
///
/// Every request in this test arrives through the *same* gateway principal. If
/// the server attributed requests to the gateway, there would be no proposer
/// and no executor to tell apart — and nothing to refuse.
#[tokio::test]
async fn separation_of_duty_holds_between_end_users_behind_one_gateway() {
    let (root, socket) = boot_with(seed_gateway_cell).await;
    let escalate = write_policy(
        root.path(),
        "escalate.yaml",
        &format!("{ESCALATE_RULES}{ADD_ITEM_RULE}"),
    );
    let plan_path = root.path().join("plan-sod.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&manual_sandbox_plan(root.path())).unwrap(),
    )
    .unwrap();
    let mut proposer = Client::connect(&socket).await;

    let submitted = proposer
        .call(with_delegation(
            json!({
                "verb": "submit",
                "plan": "plan-sod.json",
                "policy": escalate,
                "entity": "operator_a",
                "process": "sfwp_t02_tests",
                "timeout": 60,
                "request_id": format!("req-t02-sod-submit-{}", line_nonce()),
            }),
            "operator_a",
            "operator",
        ))
        .await;
    let case_id = submitted["case_id"]
        .as_str()
        .unwrap_or_else(|| panic!("submit did not return a case_id: {submitted}"))
        .to_owned();

    let proposed = proposer
        .call(with_delegation(
            json!({
                "verb": "case_add_item",
                "case_id": case_id,
                "item": discretionary_item(root.path()),
                "policy": escalate,
                "request_id": format!("req-t02-sod-add-{}", line_nonce()),
            }),
            "operator_a",
            "operator",
        ))
        .await;
    assert_eq!(proposed["ok"], true, "{proposed}");
    assert_eq!(proposed["proposed_by"], "operator_a", "{proposed}");

    // operator_b executes it; the escalate verdict opens an approval whose
    // requester is operator_b (the episode's entity).
    let mut executor = Client::connect(&socket).await;
    let executed = executor
        .call(with_delegation(
            json!({
                "verb": "item_execute",
                "case_id": case_id,
                "item_id": "disc",
                "policy": escalate,
                "timeout": 60,
                "request_id": format!("req-t02-sod-exec-{}", line_nonce()),
            }),
            "operator_b",
            "operator",
        ))
        .await;
    assert_eq!(executed["ok"], true, "{executed}");
    assert_eq!(
        executed["episodes"][0]["settlement_status"], "escalated",
        "{executed}"
    );

    // The episode's authority evaluation is attributed to the *end user* that
    // ran it, durably: the case ledger's `authority_decision` carries both the
    // acting actor and the identity binding the authority layer resolved for
    // it. If the server had attributed the request to the gateway, this record
    // would name the gateway instead of operator_b.
    let entries = case_ledger_entries(root.path(), &case_id);
    let decisions: Vec<&sea_forge_ledger::types::LedgerEntry> = entries
        .iter()
        .filter(|entry| entry.record_kind == "authority_decision")
        .collect();
    assert!(
        !decisions.is_empty(),
        "the escalation must commit an authority decision: {entries:?}"
    );
    let executor_decision = decisions
        .iter()
        .find(|entry| {
            entry
                .payload
                .pointer("/action_request/actor/actor_id")
                .and_then(Value::as_str)
                == Some("operator_b")
        })
        .unwrap_or_else(|| panic!("no authority decision from operator_b: {decisions:?}"));
    assert_eq!(
        executor_decision
            .payload
            .pointer("/identity_binding/principal")
            .and_then(Value::as_str),
        Some("operator_b"),
        "the binding must resolve the end user: {executor_decision:?}"
    );
    let rendered = serde_json::to_string(&entries).unwrap();
    assert!(
        !rendered.contains("\"actor_id\":\"gateway\"")
            && !rendered.contains("\"principal\":\"gateway\""),
        "the gateway must appear in no actor or principal field: {rendered}"
    );

    // The trace agrees for the mutation path: `PlanMutated` names the proposer.
    let events = case_events(root.path(), &case_id);
    assert!(
        events.iter().any(|event| {
            event.kind == TraceKind::PlanMutated && event.actor_id == "operator_a"
        }),
        "PlanMutated must name operator_a: {events:?}"
    );
    assert!(
        events.iter().all(|event| event.actor_id != "gateway"),
        "no trace may name the gateway as the actor: {events:?}"
    );

    let listed = executor
        .call(json!({"verb": "approval_list", "case_id": case_id}))
        .await;
    let approval_id = listed["approvals"][0]["approval_id"]
        .as_str()
        .unwrap_or_else(|| panic!("no pending approval: {listed}"))
        .to_owned();

    let approvals_path = root.path().join("approvals.jsonl");
    let before = fs::read(&approvals_path).unwrap();

    // The proposer tries to resolve its own item's approval, still behind the
    // same gateway principal.
    let refused = proposer
        .call(with_delegation(
            json!({
                "verb": "approval_decide",
                "case_id": case_id,
                "approval_id": approval_id,
                "decision": "approve",
                "note": "trust me",
                "request_id": format!("req-t02-sod-decide-{}", line_nonce()),
            }),
            "operator_a",
            "operator",
        ))
        .await;
    println!("[tooth] scenario: end user A (via gateway) proposes the item; A then tries to resolve the approval gating it -> sod_violation (SoD compares the END USER's proposed_by, never the gateway)");
    println!("[tooth] refusal (verbatim): {refused}");
    assert_eq!(refused["error_class"], "sod_violation", "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("cannot resolve an approval"),
        "{refused}"
    );
    assert_eq!(
        fs::read(&approvals_path).unwrap(),
        before,
        "a refused resolution must leave approvals.jsonl unchanged"
    );
}

// ---------------------------------------------------------------------------
// Falsifier hunt: the effective actor must always be allowlisted, and the
// positive SoD path (A proposes, B approves) must be ALLOWED with both
// principals ledgered
// ---------------------------------------------------------------------------

/// The falsifier's first form: an accepted request whose effective actor is
/// not in the allowlist. `operator_c` is bound and holds a role — only the
/// operator never allowlisted it for the gateway — so the gateway naming it
/// is refused, exactly like an unknown actor would be.
#[tokio::test]
async fn the_gateway_cannot_speak_for_a_non_allowlisted_actor() {
    let (root, socket) = boot_with(seed_gateway_cell).await;
    install_e2e_fixtures(root.path());
    let mut client = Client::connect(&socket).await;

    let before = case_count(root.path());
    let digest_before = cell_digest(root.path());
    println!("[tooth] scenario: the gateway itself (correctly authenticated) sends on_behalf_of for `operator_c`, who is bound but NOT in gateway.delegable_actors");
    print_no_side_effect(root.path(), "before", &digest_before);
    let refused = client
        .call(with_delegation(
            seeded_case_commit("operator_c"),
            "operator_c",
            "operator",
        ))
        .await;

    println!("[tooth] refusal (verbatim): {refused}");
    let digest_after = cell_digest(root.path());
    print_no_side_effect(root.path(), "after", &digest_after);
    assert_eq!(
        refused["error_class"], "identity_delegation_refused",
        "{refused}"
    );
    assert_eq!(refused["no_side_effect"], true, "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap_or_default()
            .contains("not in this cell's gateway-delegable allowlist"),
        "{refused}"
    );
    assert_eq!(case_count(root.path()), before, "a refusal writes no case");
    assert_eq!(
        digest_after, digest_before,
        "a refusal must leave ledgers and approvals byte-identical"
    );
    assert!(
        !root
            .path()
            .join("ledgers")
            .join("delegation-audit")
            .exists(),
        "a refused delegation writes no audit record"
    );
}

/// The SoD positive the refusal test leaves unproven: A proposes the work
/// (and runs it, so the escalation names A as requester), and B — a
/// different end user, through the SAME gateway principal — resolves the
/// approval. Allowed, attributed to B, both principals durable: the kernel
/// records name only end users (proposed_by A, requester A, resolved_by B —
/// the fields SoD compares), and the delegation-audit record names the
/// gateway that spoke for each of them.
#[tokio::test]
async fn user_b_may_approve_user_as_delegated_proposal_with_both_principals_ledgered() {
    let (root, socket) = boot_with(seed_gateway_cell).await;
    install_e2e_fixtures(root.path());
    // One policy serves both halves: the escalating episode (submit/add/
    // execute carry it explicitly) and the resolution (approval.decide
    // authorizes against <root>/authority/active-policy.json).
    let policy = write_policy(
        root.path(),
        "authority/active-policy.json",
        &format!("{ESCALATE_RULES}{ADD_ITEM_RULE}{APPROVAL_RESOLUTION_RULE}"),
    );
    let plan_path = root.path().join("plan-sod-pos.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&manual_sandbox_plan(root.path())).unwrap(),
    )
    .unwrap();
    let mut user_a = Client::connect(&socket).await;

    let submitted = user_a
        .call(with_delegation(
            json!({
                "verb": "submit",
                "plan": "plan-sod-pos.json",
                "policy": policy,
                "entity": "operator_a",
                "process": "sfwp_t02_tests",
                "timeout": 60,
                "request_id": format!("req-t02-pos-submit-{}", line_nonce()),
            }),
            "operator_a",
            "operator",
        ))
        .await;
    let case_id = submitted["case_id"]
        .as_str()
        .unwrap_or_else(|| panic!("submit did not return a case_id: {submitted}"))
        .to_owned();

    // A proposes the discretionary item; the response and the durable plan
    // both name A as the proposer (an SoD field).
    let proposed = user_a
        .call(with_delegation(
            json!({
                "verb": "case_add_item",
                "case_id": case_id,
                "item": discretionary_item(root.path()),
                "policy": policy,
                "request_id": format!("req-t02-pos-add-{}", line_nonce()),
            }),
            "operator_a",
            "operator",
        ))
        .await;
    assert_eq!(proposed["ok"], true, "{proposed}");
    assert_eq!(proposed["proposed_by"], "operator_a", "{proposed}");

    // A also executes its own proposal; the escalate verdict opens the
    // approval with A as the requester (the other SoD field).
    let executed = user_a
        .call(with_delegation(
            json!({
                "verb": "item_execute",
                "case_id": case_id,
                "item_id": "disc",
                "policy": policy,
                "timeout": 60,
                "request_id": format!("req-t02-pos-exec-{}", line_nonce()),
            }),
            "operator_a",
            "operator",
        ))
        .await;
    assert_eq!(executed["ok"], true, "{executed}");
    assert_eq!(
        executed["episodes"][0]["settlement_status"], "escalated",
        "{executed}"
    );

    let listed = user_a
        .call(json!({"verb": "approval_list", "case_id": case_id}))
        .await;
    let approval_id = listed["approvals"][0]["approval_id"]
        .as_str()
        .unwrap_or_else(|| panic!("no pending approval: {listed}"))
        .to_owned();

    // B resolves it through the same gateway principal. SoD compares end
    // users: B is neither the proposer nor the requester, so this is the
    // lawful path, and the response names B as the resolver.
    let mut user_b = Client::connect(&socket).await;
    let resolved = user_b
        .call(with_delegation(
            json!({
                "verb": "approval_decide",
                "case_id": case_id,
                "approval_id": approval_id,
                "decision": "approve",
                "note": "distinct end user, same gateway",
                "request_id": format!("req-t02-pos-decide-{}", line_nonce()),
            }),
            "operator_b",
            "operator",
        ))
        .await;
    assert_eq!(resolved["ok"], true, "{resolved}");
    assert!(
        resolved["output"]
            .as_str()
            .unwrap_or_default()
            .contains("resolved_by=operator_b"),
        "{resolved}"
    );

    // --- Durable proof, printed for the evidence transcript -----------------
    // (1) The SoD fields on the kernel's own records name END USERS: the
    //     plan's proposed_by, the escalation decision's requester, and the
    //     approval resolution's resolved_by.
    let plan_json: Value = serde_json::from_str(
        &fs::read_to_string(root.path().join("cases").join(&case_id).join("plan.json"))
            .expect("plan.json must exist"),
    )
    .unwrap();
    let disc = plan_json["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["plan_item_id"] == "disc")
        .expect("disc item in plan");
    assert_eq!(disc["proposed_by"], "operator_a", "{disc}");

    let entries = case_ledger_entries(root.path(), &case_id);
    let escalation = entries
        .iter()
        .find(|entry| {
            entry.record_kind == "authority_decision"
                && entry.payload["plan_item_id"] == "disc"
                && entry.payload["verdict"] == "escalate"
        })
        .expect("the disc escalation decision");
    let requester = escalation.payload["action_request"]["actor"]["actor_id"]
        .as_str()
        .unwrap_or_default();

    let resolution = entries
        .iter()
        .rfind(|entry| entry.record_kind == "approval_resolution")
        .expect("the approval resolution record");

    // (2) The dual-principal record: the delegation audit names the gateway
    //     AND the end user it spoke for on the resolving request.
    let audit = delegation_audit(root.path());
    let deciding = audit
        .iter()
        .find(|record| record["verb"] == "approval_decide")
        .unwrap_or_else(|| panic!("no approval_decide audit record: {audit:?}"));

    println!("[proof] case {case_id} approval {approval_id}: delegated mutation, both principals durable");
    println!(
        "[proof] SoD fields (end users only): proposed_by={} requester={} resolved_by={}",
        disc["proposed_by"], requester, resolution.payload["resolved_by"]
    );
    println!(
        "[proof] approval_resolution ledger record: {}",
        serde_json::to_string(&resolution.payload).unwrap()
    );
    println!("[proof] delegation-audit record for the resolving request: {deciding}");

    assert_eq!(disc["proposed_by"], "operator_a");
    assert_eq!(requester, "operator_a", "the requester is the end user");
    assert_eq!(
        resolution.payload["resolved_by"], "operator_b",
        "the resolver is the end user"
    );
    assert_eq!(
        resolution.payload["status"], "approved",
        "{}",
        resolution.payload
    );

    // The gateway never enters a SoD-comparable field: no case record or
    // approval names it as an actor or principal.
    let rendered = serde_json::to_string(&entries).unwrap();
    assert!(
        !rendered.contains("\"actor_id\":\"gateway\"")
            && !rendered.contains("\"principal\":\"gateway\"")
            && !rendered.contains("\"resolved_by\":\"gateway\""),
        "the gateway must appear in no SoD field: {rendered}"
    );
    {
        let path = "approvals.jsonl";
        let approvals = fs::read_to_string(root.path().join(path)).unwrap();
        assert!(
            !approvals.contains("\"resolved_by\":\"gateway\""),
            "{path} must not name the gateway: {approvals}"
        );
    }

    // Both principals, durably, on the resolving request.
    assert_eq!(deciding["gateway_actor_id"], "gateway", "{deciding}");
    assert_eq!(deciding["gateway_uid"], test_uid(), "{deciding}");
    assert_eq!(deciding["effective_actor_id"], "operator_b", "{deciding}");
    assert_eq!(deciding["effective_role"], "operator", "{deciding}");
    assert_eq!(deciding["verb"], "approval_decide", "{deciding}");
    // ... and the earlier delegated requests from A are audited as A.
    assert!(
        audit
            .iter()
            .any(|record| record["effective_actor_id"] == "operator_a"),
        "{audit:?}"
    );
}
