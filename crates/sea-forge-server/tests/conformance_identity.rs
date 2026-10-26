#![cfg(unix)]
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

/// Two actors, one of them with a direct OS principal.
///
/// One uid maps to exactly one actor (SF-005 follow-up), so `operator_a` is
/// this machine's uid and `operator_b` has no uid: it can act only through an
/// authenticated workload capability or the gateway, never over this test's
/// own socket. Separation of duty between the two is therefore enforced by the
/// actor, with each side proving its identity through its own channel.
fn two_actors() -> IdentityBindings {
    let uid = sea_forge_server::identity::current_uid().expect("uid must be readable");
    IdentityBindings {
        bindings: vec![
            IdentityBinding {
                uid: Some(uid),
                sid: None,
                actor_id: "operator_a".into(),
                roles: vec![sea_forge_core::types::ActorRole::Operator],
            },
            IdentityBinding {
                uid: None,
                sid: None,
                actor_id: "operator_b".into(),
                roles: vec![sea_forge_core::types::ActorRole::Operator],
            },
        ],
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
    // Fixtures land at `<root>/plan.json` and `<root>/escalate.yaml`, the
    // cell-relative spellings the submit below references.
    escalating_plan(root.path());
    escalating_policy(root.path());
    let mut client = Client::connect(&socket).await;

    let submitted = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     // F-16: plan/policy references are cell-relative spellings.
                     "plan": "plan.json", "policy": "escalate.yaml",
                     "entity": "operator_a", "process": "test", "timeout": 60, "request_id": "req-identity-submit-1"}))
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
               "case_id": case_id, "approval_id": approval_id, "decision": "approve",
               "request_id": format!("req-identity-decide-{actor_id}")})
    };

    let by_submitter = client.call(decide("operator_a")).await;
    assert_eq!(
        by_submitter["error_class"], "separation_of_duty",
        "the submitter resolved their own approval: {by_submitter}"
    );
    assert_eq!(by_submitter["no_side_effect"], true);

    // Premise changed: operator_b has no OS principal, so a direct claim for it
    // over this uid's socket is refused as not bound. That is the invariant now
    // (one uid, one actor); the positive case, operator_b approving through its
    // own authenticated connection, is proven by the authenticated test below.
    let by_other = client.call(decide("operator_b")).await;
    assert_eq!(
        by_other["error_class"], "identity_not_bound",
        "a uid-less actor must not act directly over another uid's socket: {by_other}"
    );
    assert_eq!(by_other["no_side_effect"], true);

    // Getting past separation of duty is not the same as the approval being
    // resolvable, and live driving found that it was not: the escalation
    // recorded no criteria binding, so `approve` refused every approval this
    // dispatcher opened and the case parked forever.
    //
    // This test still cannot carry that assertion to completion: `escalate.yaml`
    // authorizes no `approval_resolution`, so an in-process resolution (T04
    // moved `decide` off spawning the CLI onto the shared case-ops library)
    // is refused by policy here. The end-to-end resolution with a resolvable
    // policy is proven by the dedicated wiring test below, and the record
    // binding by the test after this one.
    let message = by_other["error"].as_str().unwrap_or_default();
    assert!(
        !message.contains("criteria reference is missing")
            && !message.contains("criteria record is missing"),
        "the escalation opened an approval with no usable criteria binding: {by_other}"
    );
}

/// The in-process approval wiring (T04 step 3): `approval.decide` resolves
/// through the shared case-ops library rather than spawning the CLI, so the
/// resolution must land in the approvals journal attributed to the actor the
/// identity gate verified. The journal, not the response, is the claim under
/// test: a wiring that silently substituted another actor (or a server-side
/// default) would pass the gate and write the wrong name here.
#[tokio::test]
async fn approval_decide_resolves_as_the_verified_actor_and_refuses_unbound_ones() {
    // A second actor claim on a same-UID socket was not authentication. The
    // trusted launcher now issues independent, request-bounded capabilities
    // through the production authority and private-connection APIs.
    use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
    use sea_forge_core::types::{Actor, ActorRole};
    use sea_forge_ledger::LedgerStream;
    use sea_forge_server::workload_connection::{authentication_action, AuthenticatedConnection};

    #[allow(clippy::too_many_arguments)]
    async fn authenticated_client(
        sender: &tokio::sync::mpsc::Sender<AuthenticatedConnection>,
        engine: &PolicyAuthorityEngine,
        bundle: &AuthorityPolicyBundle,
        ledger: &LedgerStream,
        root: &Path,
        actor_id: &str,
        request: &Value,
        sequence: usize,
    ) -> Client {
        let actor = Actor {
            actor_id: actor_id.into(),
            role: ActorRole::Operator,
        };
        let issuer = Actor {
            actor_id: "trusted-identity-issuer".into(),
            role: ActorRole::SecurityOfficer,
        };
        let principal = format!("workload-{actor_id}");
        let action = authentication_action(&principal, &actor, request);
        let decision = engine
            .evaluate(AuthorityEvaluation {
                actor: &issuer,
                binding: bundle.resolve_identity(&issuer.actor_id, issuer.role.clone()),
                run_id: "identity-issuance",
                case_id: "identity-issuance",
                plan_item_id: "private-connection",
                sequence,
                action: &action,
                workspace_root: root,
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: Some(60),
                env_keys: Default::default(),
                opa_candidate: None,
                environment: None,
            })
            .unwrap();
        assert_eq!(decision.verdict, sea_forge_core::types::Verdict::Allow);
        let committed = ledger
            .commit_typed("authority_decision", vec![], &decision, vec![])
            .unwrap();
        let grant = engine.grant(&decision, &committed, &action, None).unwrap();
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let connection = AuthenticatedConnection::new(
            server_stream,
            grant,
            &principal,
            actor,
            request,
            "identity-issuance",
            "private-connection",
            root,
        )
        .unwrap();
        sender
            .send(connection)
            .await
            .unwrap_or_else(|_| panic!("trusted issuer disconnected"));
        let (reader, writer) = client_stream.into_split();
        Client {
            reader: BufReader::new(reader),
            writer,
        }
    }

    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("kernel.sock");
    let config = ServerConfig {
        root: root.path().into(),
        socket_path: socket.clone(),
        identity: two_actors(),
        ..ServerConfig::default()
    };
    let (issuer_sender, issuer_receiver) = tokio::sync::mpsc::channel(8);
    let server = tokio::spawn(async move {
        sea_forge_server::run_authenticated(config, issuer_receiver)
            .await
            .unwrap();
    });
    for _ in 0..500 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(socket.exists());
    let issuer_bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(
        "version: \"0.1\"\nrules:\n  - name: trusted-private-connection-issuer\n    verdict: allow\n    actor_role: R-SO\n    operation_kind: identity_minting\n"
    ).unwrap();
    let issuer_engine = PolicyAuthorityEngine::new(issuer_bundle.clone()).unwrap();
    let issuer_ledger =
        LedgerStream::open(root.path(), "identity-issuance", "trusted-identity-issuer").unwrap();
    escalating_plan(root.path());
    // Escalate the item, then allow the resolution of what the escalation
    // opened: without the second rule the approval can never be resolved
    // ("approval resolution is not authorized by policy").
    fs::write(
        root.path().join("resolvable.yaml"),
        "version: \"0.1\"\nrules:\n  - name: escalate-command\n    verdict: escalate\n    \
         actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n  \
         - name: allow-approval-resolution\n    verdict: allow\n    actor_role: operator\n    \
         operation_kind: approval_resolution\n",
    )
    .unwrap();
    let submit_request = json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     "plan": "plan.json", "policy": "resolvable.yaml",
                     "entity": "operator_a", "process": "test", "timeout": 60,
                     "request_id": "req-identity-submit-resolvable"});
    let mut client = authenticated_client(
        &issuer_sender,
        &issuer_engine,
        &issuer_bundle,
        &issuer_ledger,
        root.path(),
        "operator_a",
        &submit_request,
        1,
    )
    .await;
    let mut changed_request = submit_request.clone();
    changed_request["request_id"] = json!("outside-private-grant");
    let changed_refused = client.call(changed_request).await;
    assert_eq!(
        changed_refused["error_class"],
        "identity_authentication_refused"
    );
    let canonical = submit_request.to_string();
    let ambiguous = format!(
        "{},\"opaque\":1,\"opaque\":2}}\n",
        &canonical[..canonical.len() - 1]
    );
    client.writer.write_all(ambiguous.as_bytes()).await.unwrap();
    let mut response_line = String::new();
    client.reader.read_line(&mut response_line).await.unwrap();
    let ambiguous_refused: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(
        ambiguous_refused["error_class"],
        "identity_authentication_refused"
    );
    assert!(!root.path().join("approvals.jsonl").exists());
    let submitted = client.call(submit_request).await;
    let case_id = submitted["case_id"]
        .as_str()
        .unwrap_or_else(|| panic!("submit did not return a case_id: {submitted}"))
        .to_owned();

    let inbox = client.call(json!({"verb": "approval_list"})).await;
    let approval_id = inbox["approvals"][0]["approval_id"]
        .as_str()
        .unwrap_or_else(|| panic!("an escalated episode must open an approval: {inbox}"))
        .to_owned();

    // The escalation itself journals the pending request, so the refusal's
    // durable proof is an unchanged journal, not an absent one.
    let journal_before = fs::read_to_string(root.path().join("approvals.jsonl"))
        .expect("the escalation must journal its pending approval request");

    // Negative: an actor this cell has never bound is refused at the gate,
    // and the refusal writes nothing.
    let by_stranger = client
        .call(json!({"verb": "approval_decide",
                     "actor": {"actor_id": "mallory", "role": "operator"},
                     "case_id": case_id, "approval_id": approval_id, "decision": "approve",
                     "request_id": "req-identity-decide-mallory"}))
        .await;
    assert_eq!(
        by_stranger["error_class"], "identity_not_bound",
        "{by_stranger}"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("approvals.jsonl")).unwrap(),
        journal_before,
        "a refused decision must not write the approvals journal"
    );

    // Positive: operator_b — verified by the gate — resolves in-process, and
    // the journal names operator_b, not the submitter and not any default.
    let approve_request = json!({"verb": "approval_decide",
                     "actor": {"actor_id": "operator_b", "role": "operator"},
                     "case_id": case_id, "approval_id": approval_id, "decision": "approve",
                     "request_id": "req-identity-decide-operator-b"});
    let switched = client.call(approve_request.clone()).await;
    assert_eq!(switched["error_class"], "identity_authentication_refused");
    let mut unauthenticated = Client::connect(&socket).await;
    let forged = unauthenticated.call(approve_request.clone()).await;
    assert_eq!(forged["error_class"], "identity_authentication_refused");
    let self_request = json!({"verb":"approval_decide", "actor":{"actor_id":"operator_a","role":"operator"},
        "case_id":case_id,"approval_id":approval_id,"decision":"approve","request_id":"authenticated-self-approval"});
    let mut authenticated_submitter = authenticated_client(
        &issuer_sender,
        &issuer_engine,
        &issuer_bundle,
        &issuer_ledger,
        root.path(),
        "operator_a",
        &self_request,
        2,
    )
    .await;
    let self_refused = authenticated_submitter.call(self_request).await;
    assert_eq!(self_refused["error_class"], "separation_of_duty");
    assert_eq!(
        fs::read_to_string(root.path().join("approvals.jsonl")).unwrap(),
        journal_before
    );
    let mut approver = authenticated_client(
        &issuer_sender,
        &issuer_engine,
        &issuer_bundle,
        &issuer_ledger,
        root.path(),
        "operator_b",
        &approve_request,
        3,
    )
    .await;
    let by_other = approver.call(approve_request.clone()).await;
    assert_eq!(by_other["ok"], true, "{by_other}");
    let journal = fs::read_to_string(root.path().join("approvals.jsonl"))
        .expect("the in-process resolution must write the approvals journal");
    let resolved: Value = serde_json::from_str(
        journal
            .lines()
            .last()
            .expect("the journal must carry the resolution"),
    )
    .unwrap();
    assert_eq!(
        resolved["resolved_by"], "operator_b",
        "the resolution must be attributed to the verified actor: {resolved}"
    );
    assert_eq!(resolved["status"], "approved");
    assert_eq!(
        resolved["approval_id"], approval_id,
        "the journal line must name the resolved approval: {resolved}"
    );
    let replay_refused = approver.call(approve_request).await;
    assert_eq!(
        replay_refused["error_class"],
        "identity_authentication_refused"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("approvals.jsonl")).unwrap(),
        journal
    );
    issuer_ledger.verify().unwrap();
    let authentication_ledger =
        LedgerStream::open(root.path(), "workload-authentication", "test-observer").unwrap();
    authentication_ledger.verify().unwrap();
    let authenticated = authentication_ledger.read_entries().unwrap();
    let issuer_decisions = issuer_ledger.read_entries().unwrap();
    assert_eq!(
        authenticated.len(),
        3,
        "submit, self-approval, legitimate approval"
    );
    for entry in &authenticated {
        assert_eq!(entry.record_kind, "workload_request_authenticated");
        let proof = &entry.payload;
        let decision = issuer_decisions
            .iter()
            .find(|decision| {
                decision.entry_ulid == proof["committed_decision"]["entry_ulid"].as_str().unwrap()
            })
            .expect("authentication must name its actual committed issuer decision");
        assert_eq!(entry.authority_refs, vec![decision.entry_ulid.clone()]);
        assert_eq!(
            proof["committed_decision"]["ledger_id"],
            "identity-issuance"
        );
        assert_eq!(
            proof["committed_decision"]["payload_hash"],
            decision.payload_hash
        );
        assert_eq!(
            proof["authentication_action_sha256"],
            decision.payload["action_request"]["evidence"]["payload_hash"]
        );
        assert_eq!(
            proof["actor"],
            decision.payload["operation"]["parameters"]["actor"]
        );
        assert_eq!(
            proof["request_sha256"],
            decision.payload["operation"]["parameters"]["request_sha256"]
        );
    }
    println!(
        "AUTHENTICATED_IDENTITY_V2 {}",
        json!({"legitimate_approval":by_other,"resolved":resolved,
        "same_connection_switch":switched,"fresh_unauthenticated_connection":forged,"self_approval":self_refused,
        "changed_request":changed_refused,"ambiguous_request":ambiguous_refused,"consumed_request":replay_refused,
        "issuer_ledger_verified":true,"authentication_ledger_verified":true,"authenticated_requests":authenticated.len(),
        "scope_limit":"private-channel authentication; hostile same-UID descriptor isolation remains separate"})
    );
    server.abort();
    assert!(server.await.unwrap_err().is_cancelled());
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
    // Fixtures land at `<root>/plan.json` and `<root>/escalate.yaml`, the
    // cell-relative spellings the submit below references.
    escalating_plan(root.path());
    escalating_policy(root.path());
    let mut client = Client::connect(&socket).await;

    let submitted = client
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     // F-16: plan/policy references are cell-relative spellings.
                     "plan": "plan.json", "policy": "escalate.yaml",
                     "entity": "operator_a", "process": "test", "timeout": 60, "request_id": "req-identity-submit-2"}))
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
    // Fixtures land at `<root>/plan.json` and `<root>/escalate.yaml`, the
    // cell-relative spellings the submit below references.
    escalating_plan(root.path());
    escalating_policy(root.path());

    let mut first = Client::connect(&socket).await;
    let submitted = first
        .call(json!({"verb": "submit",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     // F-16: plan/policy references are cell-relative spellings.
                     "plan": "plan.json", "policy": "escalate.yaml",
                     "entity": "operator_a", "process": "test", "timeout": 60, "request_id": "req-identity-submit-3"}))
        .await;
    let case_id = submitted["case_id"].as_str().unwrap().to_owned();
    let inbox = first.call(json!({"verb": "approval_list"})).await;
    let approval_id = inbox["approvals"][0]["approval_id"].as_str().unwrap();

    drop(first);
    let mut second = Client::connect(&socket).await;
    let response = second
        .call(json!({"verb": "approval_decide",
                     "actor": {"actor_id": "operator_a", "role": "operator"},
                     "case_id": case_id, "approval_id": approval_id, "decision": "approve",
                     "request_id": "req-identity-reconnect-decide"}))
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
