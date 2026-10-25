//! Frozen behavioral acceptance probes. RED is an unmet contract, never a stub.
//! Reuses conformance_identity and convergence_t05/t06 public entry points.
#![cfg(unix)]

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, AuthorityAction, ExecutionRequest, ExecutionStatus, Operation,
    SettlementStatus, Verdict,
};
use sea_forge_ledger::LedgerStream;
use sea_forge_server::governed_execution_boundary::{emit_authorized_invocation, InvocationLedger};
use sea_forge_server::governed_settlement_return::{
    emit_operational_settlement, SettlementOptionalFields,
};
use sea_forge_server::identity::{IdentityBinding, IdentityBindings};
use sea_forge_server::{run, ServerConfig};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn report(mut value: Value) {
    value["suite"] = json!("SEA-Forge-security-v1");
    println!("SECURITY_V1 {value}");
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
            .expect("INFRASTRUCTURE-BLOCKED: response timed out")
            .unwrap();
        serde_json::from_str(line.trim()).expect("INFRASTRUCTURE-BLOCKED: invalid response")
    }
}

#[tokio::test]
async fn security_v1_a_sec01_same_uid_cannot_claim_another_approver() {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("kernel.sock");
    let uid = sea_forge_server::identity::current_uid().unwrap();
    let config = ServerConfig {
        socket_path: socket.clone(),
        root: root.path().to_path_buf(),
        identity: IdentityBindings {
            // One uid, one actor (SF-005 follow-up): the same OS user may act
            // directly only as operator_a. operator_b has no OS principal and is
            // reachable only through the gateway, so a connection from this uid
            // claiming operator_b must be refused as not bound.
            bindings: vec![
                IdentityBinding {
                    uid: Some(uid),
                    actor_id: "operator_a".into(),
                    roles: vec![ActorRole::Operator],
                },
                IdentityBinding {
                    uid: None,
                    actor_id: "operator_b".into(),
                    roles: vec![ActorRole::Operator],
                },
            ],
        },
        ..ServerConfig::default()
    };
    let server = tokio::spawn(async move {
        let _ = run(config).await;
    });
    for _ in 0..500 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        socket.exists(),
        "INFRASTRUCTURE-BLOCKED: real server socket absent"
    );
    let executable = root.path().join("sea-forge");
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &executable).unwrap();
    fs::write(root.path().join("plan.json"), serde_json::to_vec(&json!({
        "version":"0.2", "plan_id":"plan_security_v1", "case_id":"case_placeholder",
        "run_id":"run_placeholder", "intent_id":"int_security_v1",
        "items":[{"plan_item_id":"task", "name":"sandboxed", "operations":[{
            "kind":"execute_command", "argv":[executable, "--exact", "__no_test_matches_this_name__"], "cwd":"."
        }], "entry_criteria":[], "exit_criteria":[],
        "settlement_criteria":{"required_artifacts":[],"required_declarations":[]},
        "item_kind":"sandboxed_task", "markers":{"required":true}, "max_instances":1,"depends_on":[]}]
    })).unwrap()).unwrap();
    fs::write(root.path().join("policy.yaml"), "version: \"0.1\"\nrules:\n  - name: escalate-command\n    verdict: escalate\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n  - name: allow-resolution\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n").unwrap();
    let mut client = Client::connect(&socket).await;
    let submitted = client
        .call(json!({"verb":"submit",
        "actor":{"actor_id":"operator_a","role":"operator"},
        "plan":"plan.json","policy":"policy.yaml","entity":"operator_a",
        "process":"security-v1","timeout":60,"request_id":"sec01-submit"}))
        .await;
    let case_id = submitted["case_id"]
        .as_str()
        .expect("INFRASTRUCTURE-BLOCKED: valid submit control failed");
    let inbox = client.call(json!({"verb":"approval_list"})).await;
    let approval_id = inbox["approvals"][0]["approval_id"]
        .as_str()
        .expect("INFRASTRUCTURE-BLOCKED: pending approval absent");
    let journal = root.path().join("approvals.jsonl");
    let before = fs::read(&journal).unwrap();
    let pending: Value = serde_json::from_str(
        std::str::from_utf8(&before)
            .unwrap()
            .lines()
            .last()
            .unwrap(),
    )
    .unwrap();
    let decide = |actor: Value, id: &str| {
        json!({"verb":"approval_decide", "actor":actor,
        "case_id":case_id,"approval_id":approval_id,"decision":"approve","request_id":id})
    };
    let self_approval = client
        .call(decide(
            json!({"actor_id":"operator_a","role":"operator"}),
            "sec01-self",
        ))
        .await;
    let stranger = client
        .call(decide(
            json!({"actor_id":"unbound","role":"operator"}),
            "sec01-unbound",
        ))
        .await;
    let role_forgery = client
        .call(decide(
            json!({"actor_id":"operator_a","role":"R-SO"}),
            "sec01-role",
        ))
        .await;
    let missing = client
        .call(json!({"verb":"approval_decide","case_id":case_id,
        "approval_id":approval_id,"decision":"approve","request_id":"sec01-missing"}))
        .await;
    let controls_unchanged = fs::read(&journal).unwrap() == before;
    // Same client, connection, peer UID and workload. Only attacker-selected actor changes.
    let attack = client
        .call(decide(
            json!({"actor_id":"operator_b","role":"operator"}),
            "sec01-impersonate",
        ))
        .await;
    let after = fs::read(&journal).unwrap();
    let unchanged = before == after;
    let refused = attack.get("error_class").is_some() && attack["no_side_effect"] == true;
    let controls = controls_unchanged
        && self_approval["error_class"] == "separation_of_duty"
        && stranger["error_class"] == "identity_not_bound"
        && role_forgery.get("error_class").is_some()
        && missing.get("error_class").is_some();
    report(
        json!({"case_id":"SEC-01.same_uid_approval", "classification":if unchanged && refused && controls {"PASS"}else{"SECURITY-RED"},
        "trusted_identity":"operator_a workload on original socket", "claimed_identity":"operator_b",
        "run_id":pending["run_id"],"kernel_case_id":case_id,"approval_id":approval_id,
        "authority_decision_id":pending["decision_id"],"action_id":"sec01-impersonate", "uid":uid,"attempted":true,
        "decision":attack,"observed_effect":{"journal_unchanged":unchanged,"before_sha256":sha(&before),"after_sha256":sha(&after),
        "journal_records":String::from_utf8_lossy(&after)},"positive_control":controls,
        "control_responses":{"self":self_approval,"unbound":stranger,"role":role_forgery,"missing":missing},
        "cleanup":"server aborted and joined; temporary cell removed before assertion",
        "scope_limit":"legitimate submit control; distinct per-workload authentication and independent B positive control still required"}),
    );
    drop(client);
    server.abort();
    let _ = server.await;
    drop(root);
    assert!(
        controls,
        "SECURITY-RED: safe submit/negative identity controls failed"
    );
    assert!(
        unchanged && refused,
        "SECURITY-RED SEC-01: same-UID actor substitution changed protected approval state"
    );
}

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: bounded-helper\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";
const WORLD: &str =
    "world:sf@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

struct ExecutionFixture {
    _root: tempfile::TempDir,
    workspace: PathBuf,
    artifacts: PathBuf,
    executable: PathBuf,
}

fn fixture() -> ExecutionFixture {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    let artifacts = root.path().join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let executable = root.path().join("sea-forge");
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &executable).unwrap();
    ExecutionFixture {
        _root: root,
        workspace,
        artifacts,
        executable,
    }
}

// Re-entered only through the real runtime, under the evaluated argv/env grant.
#[test]
fn security_v1_resource_helper() {
    if std::env::var("SECURITY_V1_HELPER").as_deref() != Ok("1") {
        return;
    }
    let attempt = fs::write("protected-marker", b"authorized-synthetic-effect");
    let observation = json!({"attempted":true,"succeeded":attempt.is_ok(),
        "os_error":attempt.as_ref().err().and_then(|e|e.raw_os_error()),
        "inherited_credential_present":std::env::var_os("SECURITY_V1_PARENT_CREDENTIAL").is_some()});
    fs::write(
        "resource-observation.json",
        serde_json::to_vec(&observation).unwrap(),
    )
    .unwrap();
    let shell = std::process::Command::new("/bin/sh")
        .args(["-c", "printf bounded-shell-ok"])
        .output()
        .unwrap();
    assert!(shell.status.success());
    fs::write("safe-shell-output", shell.stdout).unwrap();
}

struct RunEvidence {
    decision: sea_forge_core::types::AuthorityDecision,
    ledger: LedgerStream,
    observed: Value,
    invocation_id: String,
    event_id: String,
    registry: InvocationLedger,
}

fn run_helper(h: &ExecutionFixture, case: &str) -> RunEvidence {
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let argv = vec![
        h.executable.to_string_lossy().into_owned(),
        "--exact".into(),
        "security_v1_resource_helper".into(),
    ];
    let action = AuthorityAction::ExecuteCommand {
        argv: argv.clone(),
        cwd: ".".into(),
    };
    let env = BTreeMap::from([
        ("PATH".into(), "/usr/bin:/bin".into()),
        ("HOME".into(), h.workspace.to_string_lossy().into_owned()),
        ("SECURITY_V1_HELPER".into(), "1".into()),
    ]);
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: case,
            case_id: case,
            plan_item_id: "security-item",
            sequence: 1,
            action: &action,
            workspace_root: &h.workspace,
            evidence_refs: vec![],
            artifacts_root: Some(&h.artifacts),
            timeout_secs: Some(10),
            env_keys: env.keys().cloned().collect(),
            opa_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(
        decision.verdict,
        Verdict::Allow,
        "INFRASTRUCTURE-BLOCKED: control grant not allowed"
    );
    let ledger = LedgerStream::open(h._root.path(), case, "security-observer").unwrap();
    let committed = ledger
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let model_hash = sha(b"security-v1-model");
    let invocation = emit_authorized_invocation(
        &decision,
        case,
        &model_hash,
        &model_hash,
        "12345678-1234-4234-8234-123456789abc",
        None,
        WORLD,
    )
    .unwrap();
    let mut registry = InvocationLedger::default();
    registry
        .admit_authorized_invocation(&invocation.envelope, &model_hash)
        .unwrap();
    ledger
        .commit_typed(
            "security_authorized_invocation",
            vec![],
            &invocation.envelope,
            vec![committed.entry_ulid().to_string()],
        )
        .unwrap();
    let grant = engine.grant(&decision, &committed, &action, None).unwrap();
    let request = ExecutionRequest {
        plan_item_id: "security-item".into(),
        operation: Operation::ExecuteCommand {
            argv,
            cwd: ".".into(),
        },
        timeout_secs: 10,
        env,
        compensating_controls: vec![],
    };
    let executed = sea_forge_runtime::execute(grant, &request, case, &h.workspace, &h.artifacts)
        .expect("INFRASTRUCTURE-BLOCKED: governed runtime failed");
    assert_eq!(
        executed.status,
        ExecutionStatus::Completed,
        "INFRASTRUCTURE-BLOCKED: helper failed"
    );
    let observed: Value =
        serde_json::from_slice(&fs::read(h.workspace.join("resource-observation.json")).unwrap())
            .unwrap();
    assert_eq!(
        fs::read(h.workspace.join("safe-shell-output")).unwrap(),
        b"bounded-shell-ok"
    );
    ledger
        .commit_typed(
            "security_resource_observation",
            vec![],
            &observed,
            vec![committed.entry_ulid().to_string()],
        )
        .unwrap();
    RunEvidence {
        decision,
        ledger,
        observed,
        invocation_id: invocation.invocation_id,
        event_id: invocation.event_id,
        registry,
    }
}

#[test]
fn security_v1_a_sec12_resource_rejection_cannot_settle_as_success() {
    let model_hash = sha(b"security-v1-model");
    let mut observations = vec![];
    let mut rejected_attack = false;
    for reject in [false, true] {
        let h = fixture();
        if reject {
            fs::write(
                h.workspace.join("protected-marker"),
                b"original-synthetic-record",
            )
            .unwrap();
            fs::set_permissions(
                h.workspace.join("protected-marker"),
                fs::Permissions::from_mode(0o400),
            )
            .unwrap();
        }
        let case = if reject {
            "case-security-rejection"
        } else {
            "case-security-control"
        };
        let before = fs::read(h.workspace.join("protected-marker")).ok();
        let RunEvidence {
            decision,
            ledger,
            observed,
            invocation_id,
            event_id,
            mut registry,
        } = run_helper(&h, case);
        let after = fs::read(h.workspace.join("protected-marker")).unwrap();
        let actual_effect = fs::read(h.workspace.join("protected-marker")).unwrap()
            == b"authorized-synthetic-effect";
        if reject && (observed["succeeded"] != false || observed["os_error"] != 13) {
            report(
                json!({"case_id":"SEC-12.false_settlement","classification":"INFRASTRUCTURE-BLOCKED",
                "reason":"fixture filesystem did not reject write (for example privileged root execution)","attempted":true}),
            );
            panic!("INFRASTRUCTURE-BLOCKED: fixture permission refusal absent");
        }
        if reject {
            assert_eq!(
                before.as_ref().unwrap(),
                &after,
                "protected resource changed after denied write"
            );
        }
        let observation = json!({"schema_version":"v1","event_id":if reject{"99999999-8888-4777-8666-555555555555"}else{"99999999-8888-4777-8666-555555555556"},
            "source_agent":"execution-environment","event_type":"ExecutionObservation","occurred_at":"2026-10-16T12:00:00+00:00",
            "idempotency_key":sha(case.as_bytes()),"payload":{"domain_model_hash":model_hash,
            "namespace":"agentic_capability_loop","work_request_id":case,"invocation_id":invocation_id,
            "execution_status":"completed","exit_code":0,"observed_effects":[{"kind":"file_write",
                "ref":h.workspace.join("protected-marker"),"sha256":sha(b"authorized-synthetic-effect"),
                "effect":"protected marker exists with authorized-synthetic-effect"}]},
            "provenance":{"origin":"execution-environment","chain":[format!("caused_by:{event_id}")]}});
        let outcome = registry
            .settle_execution_observation(&observation, &model_hash)
            .unwrap();
        let returned = emit_operational_settlement(
            &outcome,
            &["protected marker exists with authorized-synthetic-effect".into()],
            &model_hash,
            &model_hash,
            &event_id,
            observation["event_id"].as_str().unwrap(),
            SettlementOptionalFields::default(),
        )
        .unwrap();
        let attempt_ref = ledger.commit_typed("security_independent_outcome", vec![case.into()],
            &json!({"authority_decision_id":decision.decision_id,"run_id":case,"invocation_id":invocation_id,
                "attempted":true,"native_result":observed,"before_sha256":before.as_ref().map(|bytes|sha(bytes)),"after_sha256":sha(&after)}),vec![]).unwrap();
        let observation_ref = ledger
            .commit_typed(
                "security_submitted_observation",
                vec![case.into()],
                &observation,
                vec![attempt_ref.entry_ulid().to_string()],
            )
            .unwrap();
        ledger
            .commit_typed(
                "security_returned_settlement",
                vec![case.into()],
                &returned.envelope,
                vec![observation_ref.entry_ulid().to_string()],
            )
            .unwrap();
        ledger.verify().expect("observer ledger chain must verify");
        let entries = ledger.read_entries().unwrap();
        for kind in [
            "authority_decision",
            "security_authorized_invocation",
            "security_resource_observation",
            "security_independent_outcome",
            "security_submitted_observation",
            "security_returned_settlement",
        ] {
            assert!(
                entries.iter().any(|entry| entry.record_kind == kind),
                "missing evidence record {kind}"
            );
        }
        let decision_record = entries
            .iter()
            .find(|entry| entry.record_kind == "authority_decision")
            .unwrap();
        assert_eq!(decision_record.payload["decision_id"], decision.decision_id);
        assert_eq!(decision_record.payload["run_id"], case);
        let submitted_record = entries
            .iter()
            .find(|entry| entry.record_kind == "security_submitted_observation")
            .unwrap();
        assert_eq!(
            submitted_record.payload["payload"]["invocation_id"],
            invocation_id
        );
        assert_eq!(
            returned.envelope["payload"]["authority_decision_id"],
            decision.decision_id
        );
        let ledger_bytes = fs::read(ledger.entries_path()).unwrap();
        if !reject {
            assert!(
                actual_effect && returned.evaluation.status == SettlementStatus::Accepted,
                "positive settlement control failed"
            );
        } else {
            rejected_attack = returned.evaluation.status != SettlementStatus::Accepted;
        }
        observations.push(json!({"action_id":case,"trusted_identity":"test operator binding (local default)",
            "claimed_identity":"execution-environment","authority_decision_id":decision.decision_id,
            "attempted":true,"independent_resource_observation":observed,"actual_effect":actual_effect,
            "before_sha256":before.as_ref().map(|bytes|sha(bytes)),"after_sha256":sha(&after),
            "submitted_observation":observation,"returned_settlement":returned.envelope,
            "ledger_sha256":sha(&ledger_bytes),"ledger_verified":true,"ledger_records":entries}));
    }
    report(
        json!({"case_id":"SEC-12.false_settlement","classification":if rejected_attack{"PASS"}else{"SECURITY-RED"},
        "trusted_identity":"independent filesystem observer","claimed_identity":"execution-environment",
        "run_id":"case-security-rejection","action_id":"forged-success-after-EACCES","decision":"real Allow + supplied observation",
        "attempted":true,"positive_control":true,"observations":observations,"cleanup":"temporary fixtures removed",
        "scope_limit":"public operational settlement API; trusted observer authentication on external ingress requires supplementary coverage"}),
    );
    assert!(
        rejected_attack,
        "SECURITY-RED SEC-12: accepted caller-claimed success despite observed resource refusal"
    );
}

#[test]
fn security_v1_b_sec06_inherited_credential_is_absent_and_bounded_shell_works() {
    let h = fixture();
    // Unique synthetic variable, never enumerating or printing the host environment.
    std::env::set_var(
        "SECURITY_V1_PARENT_CREDENTIAL",
        "synthetic-owner-credential",
    );
    struct CredentialCleanup;
    impl Drop for CredentialCleanup {
        fn drop(&mut self) {
            std::env::remove_var("SECURITY_V1_PARENT_CREDENTIAL");
        }
    }
    let cleanup = CredentialCleanup;
    assert!(std::env::var_os("SECURITY_V1_PARENT_CREDENTIAL").is_some());
    let RunEvidence {
        ledger, observed, ..
    } = run_helper(&h, "case-security-environment");
    drop(cleanup);
    let absent = observed["inherited_credential_present"] == false;
    ledger.verify().unwrap();
    report(
        json!({"case_id":"SEC-06.child_environment","classification":if absent{"PASS"}else{"SECURITY-RED"},
        "run_id":"case-security-environment","trusted_identity":"operator_local","claimed_identity":"operator_local",
        "action_id":"inherited-credential-read","attempted":true,"decision":"real authority grant for bounded helper",
        "observed_effect":observed,"positive_control":true,"cleanup":"synthetic variable removed; temporary fixture removed",
        "scope_limit":"environment inheritance only; host file/DB/egress bypasses remain obligations"}),
    );
    assert!(
        absent,
        "SECURITY-RED SEC-06: unrestricted synthetic parent credential inherited"
    );
}
