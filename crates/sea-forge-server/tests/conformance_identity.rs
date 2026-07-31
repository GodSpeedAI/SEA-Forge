//! Governed actor identity over the wire (SF-005, decision U-07).
//!
//! The unit tests in `server::identity` prove the resolver's logic. These prove
//! the *gate is installed*: that a protected verb arriving on a real socket
//! without a verified actor is refused before it reaches a handler, and that an
//! inspect verb on the same connection is not.
//!
//! Those are different claims. A resolver that works perfectly and is never
//! called would satisfy the first set and none of these.

use sea_forge_server::identity::{IdentityBinding, IdentityBindings};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

/// The actor block a well-behaved client sends.
fn actor() -> Value {
    json!({"actor_id": "operator_local", "role": "operator"})
}

/// Two actors on this machine's uid.
///
/// Both bind to the same uid because a test can only ever present one — the
/// kernel fills in `SO_PEERCRED` and nothing here can forge a second. That is
/// exactly the configuration separation of duty has to survive: two identities
/// one human can legitimately hold, where the *actor*, not the uid, is what
/// keeps a submitter from approving their own work.
fn two_actors() -> IdentityBindings {
    let uid = sea_forge_server::identity::current_uid().expect("uid must be readable");
    IdentityBindings {
        bindings: ["operator_a", "operator_b"]
            .into_iter()
            .map(|actor_id| IdentityBinding {
                uid,
                actor_id: actor_id.into(),
                roles: vec![sea_forge_core::types::ActorRole::Operator],
            })
            .collect(),
    }
}

/// A policy that escalates `execute_command` — the only way to get a real
/// `approval_request` into a case ledger, since an approval exists only because
/// authority asked a human a question.
///
/// `argv0: sea-forge` is required: `AuthorityPolicyBundle::validate` refuses a
/// local rule for `execute_command` with any other argv0.
fn escalating_policy(dir: &Path) -> PathBuf {
    let path = dir.join("escalate.yaml");
    fs::write(
        &path,
        "version: \"0.1\"\nrules:\n  - name: escalate-command\n    verdict: escalate\n    \
         actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n",
    )
    .unwrap();
    path
}

/// A plan whose single item escalates. Mirrors `conformance_case_episode`'s
/// fixture; the symlink is what satisfies both argv0 checks at once (the policy
/// matches on `file_name`, and `untrusted_executable` requires the canonical
/// path to equal `current_exe`).
fn escalating_plan(dir: &Path) -> PathBuf {
    let link = dir.join("sea-forge");
    std::os::unix::fs::symlink(std::env::current_exe().expect("current_exe"), &link).unwrap();
    let plan = json!({
        "version": "0.2",
        "plan_id": "plan_sod",
        "case_id": "case_placeholder",
        "run_id": "run_placeholder",
        "intent_id": "int_sod",
        "items": [{
            "plan_item_id": "task",
            "name": "sandboxed",
            "operations": [{
                "kind": "execute_command",
                "argv": [link.to_string_lossy(), "--exact", "__no_test_matches_this_name__"],
                "cwd": "."
            }],
            "entry_criteria": [], "exit_criteria": [],
            "settlement_criteria": {"required_artifacts": [], "required_declarations": []},
            "item_kind": "sandboxed_task",
            "markers": {"required": true},
            "max_instances": 1,
            "depends_on": []
        }]
    });
    let path = dir.join("plan.json");
    fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    path
}

async fn boot(identity: IdentityBindings) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    // Short, because a deep temp path overflows `sun_path` (see the cell
    // contract) and the failure would look like an identity problem.
    let socket = std::env::temp_dir().join(format!(
        "sf-id-{}-{}.sock",
        std::process::id(),
        root.path().file_name().unwrap().to_string_lossy()
    ));
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        identity,
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

/// A protected verb with no actor block. The refusal has to be typed, and it
/// has to say plainly that nothing happened — an operator should not have to
/// infer "no side effect" from the absence of other fields.
#[tokio::test]
async fn a_protected_verb_without_an_actor_is_refused() {
    let (root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit", "plan": "/nonexistent/plan.json",
                     "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(response["error_class"], "identity_required", "{response}");
    assert_eq!(response["no_side_effect"], true);
    // Nothing ran, so the cell has no cases at all — not an empty case, none.
    assert!(
        !root.path().join("cases").exists(),
        "a refused submit created {}/cases",
        root.path().display()
    );
}

/// The compatibility half of U-07 answer 4. An old client that never learned
/// about the actor block keeps working for everything that only reads.
#[tokio::test]
async fn inspect_verbs_still_work_without_an_actor() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    for verb in ["run_list", "case_list", "agent_list", "asset_list"] {
        let response = client.call(json!({"verb": verb})).await;
        assert_ne!(
            response["error_class"], "identity_required",
            "inspect verb {verb} demanded an actor: {response}"
        );
    }
}

/// An unconfigured cell must not fall back to deriving an actor from whoever
/// connects. That fallback would be the same fabricated identity SF-005 exists
/// to remove, relocated into the server.
#[tokio::test]
async fn an_unconfigured_cell_refuses_protected_work_even_with_a_valid_actor() {
    let (root, socket) = boot(IdentityBindings::default()).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit", "actor": actor(),
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(
        response["error_class"], "identity_unconfigured",
        "{response}"
    );
    assert!(!root.path().join("cases").exists());
}

/// Claiming an actor this cell has never heard of must fail, even though the
/// uid is the one the cell is configured for. The binding is (uid, actor_id),
/// not either half alone.
#[tokio::test]
async fn an_unbound_actor_id_is_refused_on_a_configured_cell() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "someone_else", "role": "operator"},
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(response["error_class"], "identity_not_bound", "{response}");
}

/// A role the actor does not hold is refused. `local_operator` grants only
/// `operator`, so a claim to act as a security officer must not pass.
#[tokio::test]
async fn a_role_the_actor_does_not_hold_is_refused() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_local", "role": "R-SO"},
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(
        response["error_class"], "identity_role_not_held",
        "{response}"
    );
}

/// `identity.get` is how a client learns which actor to claim. Before it, the
/// desktop router fabricated one (`mockGuardContext`); the whole point is that
/// the answer comes from the cell rather than from the client's imagination.
#[tokio::test]
async fn identity_get_reports_the_actors_this_connection_may_claim() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let view = client.call(json!({"verb": "identity_get"})).await;

    assert_eq!(view["configured"], true, "{view}");
    assert_eq!(view["available"][0]["actor_id"], "operator_local");
    assert_eq!(view["available"][0]["roles"][0], "operator");
    assert!(
        view["uid"].is_u64(),
        "the peer uid must be reported: {view}"
    );
    assert!(
        view.get("refusal").is_none(),
        "a resolvable connection must not carry a refusal: {view}"
    );
}

/// An unconfigured cell must say so through the *same* vocabulary a refused
/// protected verb uses, so a client can explain the block before attempting
/// work rather than discovering it from a denial.
#[tokio::test]
async fn identity_get_reports_an_unconfigured_cell_in_the_refusal_vocabulary() {
    let (_root, socket) = boot(IdentityBindings::default()).await;
    let mut client = Client::connect(&socket).await;

    let view = client.call(json!({"verb": "identity_get"})).await;

    assert_eq!(view["configured"], false, "{view}");
    assert_eq!(view["refusal"]["error_class"], "identity_unconfigured");
    assert_eq!(
        view["available"].as_array().map(Vec::len),
        Some(0),
        "nothing is claimable on an unconfigured cell: {view}"
    );
}

/// Identity must be *used*, not merely checked. `entity` is what reaches the
/// authority engine and lands in the ledger as the acting principal, so a
/// request that verifies one actor and attributes its work to another would
/// authenticate `a` and record `b` — and could then approve its own work as
/// `a`. That is the separation-of-duty hole, one layer up.
#[tokio::test]
async fn a_request_cannot_verify_one_actor_and_attribute_its_work_to_another() {
    let (_root, socket) = boot(two_actors()).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_b", "process": "test", "timeout": 30}))
        .await;

    assert_eq!(
        response["error_class"], "identity_entity_mismatch",
        "{response}"
    );
    assert_eq!(response["no_side_effect"], true);
}

/// The two-actor journey SF-005 exists for, end to end over a real socket:
/// `operator_a` submits work that escalates, then cannot resolve the approval
/// it created — but `operator_b` can.
///
/// Both halves matter. A gate that refuses everyone would pass the first
/// assertion and make approvals unusable, so the test proves the refusal is
/// specific to the submitter rather than general.
#[tokio::test]
async fn a_submitter_cannot_approve_their_own_work_but_another_actor_can() {
    let (root, socket) = boot(two_actors()).await;
    let plan = escalating_plan(root.path());
    let policy = escalating_policy(root.path());
    let mut client = Client::connect(&socket).await;

    let submitted = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     "plan": plan, "policy": policy,
                     "entity": "operator_a", "process": "test", "timeout": 60}))
        .await;
    let case_id = submitted["case_id"]
        .as_str()
        .unwrap_or_else(|| panic!("submit did not return a case_id: {submitted}"))
        .to_owned();

    let inbox = client.call(json!({"verb": "approval_list"})).await;
    let approval_id = inbox["approvals"][0]["approval_id"]
        .as_str()
        .unwrap_or_else(|| panic!("an escalated episode must open an approval: {inbox}"))
        .to_owned();

    let decide = |actor_id: &str| {
        json!({"verb": "approval_decide",
               "actor": {"actor_id": actor_id, "role": "operator"},
               "case_id": case_id, "approval_id": approval_id, "decision": "approve"})
    };

    let by_submitter = client.call(decide("operator_a")).await;
    assert_eq!(
        by_submitter["error_class"], "separation_of_duty",
        "the submitter resolved their own approval: {by_submitter}"
    );
    assert_eq!(by_submitter["no_side_effect"], true);

    let by_other = client.call(decide("operator_b")).await;
    let class = by_other["error_class"].as_str().unwrap_or_default();
    assert_ne!(
        class, "separation_of_duty",
        "a second actor must be able to approve: {by_other}"
    );

    // Getting past separation of duty is not the same as the approval being
    // resolvable, and live driving found that it was not: the escalation
    // recorded no criteria binding, so `approve` refused every approval this
    // dispatcher opened and the case parked forever.
    //
    // This test cannot carry that assertion to completion. `decide` shells out
    // through `run_cli`, which resolves a `sea-forge` binary beside
    // `current_exe()` — inside a test harness that is the test binary, so the
    // approve subcommand never runs here regardless of correctness. What is
    // assertable is that the *specific* defect is gone; the end-to-end
    // resolution is proven live by `docs/execution/journey/drive_identity.py`,
    // and the record binding by the test below.
    let message = by_other["error"].as_str().unwrap_or_default();
    assert!(
        !message.contains("criteria reference is missing")
            && !message.contains("criteria record is missing"),
        "the escalation opened an approval with no usable criteria binding: {by_other}"
    );
}

/// `ask` is protected, so it carries the identity block — and it also has a
/// field naming who is asking. Those collided: SF-005 claimed `actor` for
/// `{actor_id, role}` while `Request::Ask` already used `actor` for a plain
/// string, so the gate could not parse the block and refused every `ask` that
/// arrived on a real socket.
///
/// The whole kernel suite stayed green through it, because every `ask` test
/// calls `handle_request` directly — below the gate. This one goes over the
/// wire on purpose; that is the only thing that would have caught it.
#[tokio::test]
async fn ask_is_reachable_over_a_socket_with_an_identity_block() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "ask", "actor": actor(),
                     "kind": "ask_capability", "subject": "delegation",
                     "purpose": "conformance"}))
        .await;

    let class = response["error_class"].as_str().unwrap_or_default();
    assert!(
        !class.starts_with("identity_"),
        "ask was refused by the identity gate it satisfies: {response}"
    );
}

/// An implemented method that the catalog does not list is unreachable: no
/// client can discover it. `thoth.ask` was in exactly that state.
#[tokio::test]
async fn the_catalog_advertises_every_method_a_client_can_call() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let hello = client
        .call(json!({"verb": "system_hello", "protocol_version": "1"}))
        .await;
    let methods: Vec<&str> = hello["implemented_methods"]
        .as_array()
        .expect("hello must list methods")
        .iter()
        .filter_map(|m| m.as_str())
        .collect();

    for method in ["identity.get", "thoth.ask"] {
        assert!(
            methods.contains(&method),
            "{method} is implemented but not advertised: {methods:?}"
        );
    }
}

/// The binding that makes an approval resolvable at all: a committed
/// `settlement_criteria` record, and an approval whose three criteria fields
/// point at it. `approve` cross-checks all of them, so any one being absent
/// makes the escalation a dead end.
#[tokio::test]
async fn an_escalated_approval_is_bound_to_committed_criteria() {
    let (root, socket) = boot(two_actors()).await;
    let plan = escalating_plan(root.path());
    let policy = escalating_policy(root.path());
    let mut client = Client::connect(&socket).await;

    let submitted = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     "plan": plan, "policy": policy,
                     "entity": "operator_a", "process": "test", "timeout": 60}))
        .await;
    let case_id = submitted["case_id"].as_str().expect("a case").to_owned();

    let entries =
        sea_forge_ledger::LedgerStream::open(root.path(), format!("case-{case_id}"), "test")
            .unwrap()
            .read_entries()
            .unwrap();

    let criteria: Vec<_> = entries
        .iter()
        .filter(|entry| entry.record_kind == "settlement_criteria")
        .collect();
    assert_eq!(
        criteria.len(),
        1,
        "an escalation must commit exactly one criteria record"
    );

    let approval = entries
        .iter()
        .find(|entry| entry.record_kind == "approval_request")
        .expect("an escalation must open an approval");

    assert_eq!(
        approval.payload["criteria_ref"], criteria[0].payload["criteria_id"],
        "the approval must name the criteria it gates"
    );
    assert_eq!(
        approval.payload["criteria_sha256"], criteria[0].payload["criteria_sha256"],
        "the approval must pin the criteria content hash"
    );
    assert_eq!(
        approval.payload["criteria_record_hash"], criteria[0].payload["criteria_record_hash"],
        "the approval must pin the criteria record hash"
    );
}

/// The refusal must survive a reconnect. Identity binds per request (U-07
/// answer 1), so a submitter who drops and redials is a *new connection* with
/// the same actor — and the comparison is against the ledger, not the session,
/// precisely so the new connection changes nothing.
#[tokio::test]
async fn reconnecting_does_not_launder_a_self_approval() {
    let (root, socket) = boot(two_actors()).await;
    let plan = escalating_plan(root.path());
    let policy = escalating_policy(root.path());

    let mut first = Client::connect(&socket).await;
    let submitted = first
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     "plan": plan, "policy": policy,
                     "entity": "operator_a", "process": "test", "timeout": 60}))
        .await;
    let case_id = submitted["case_id"].as_str().unwrap().to_owned();
    let inbox = first.call(json!({"verb": "approval_list"})).await;
    let approval_id = inbox["approvals"][0]["approval_id"].as_str().unwrap();

    drop(first);
    let mut second = Client::connect(&socket).await;
    let response = second
        .call(json!({"verb": "approval_decide",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     "case_id": case_id, "approval_id": approval_id, "decision": "approve"}))
        .await;

    assert_eq!(
        response["error_class"], "separation_of_duty",
        "a fresh connection laundered a self-approval: {response}"
    );
}

/// A bound actor gets past the gate. The submit then fails on its own merits
/// (the plan path does not exist), which is the point: the *identity* check no
/// longer refused it.
#[tokio::test]
async fn a_bound_actor_passes_the_gate_and_is_judged_on_the_request_itself() {
    let (_root, socket) = boot(IdentityBindings::local_operator("operator_local")).await;
    let mut client = Client::connect(&socket).await;

    let response = client
        .call(json!({"verb": "submit", "actor": actor(),
                     "plan": "/nonexistent/plan.json", "policy": "/nonexistent/policy.yaml",
                     "entity": "operator_local", "process": "test", "timeout": 30}))
        .await;

    let class = response["error_class"].as_str().unwrap_or_default();
    assert!(
        !class.starts_with("identity_"),
        "a bound actor was still refused on identity: {response}"
    );
}
