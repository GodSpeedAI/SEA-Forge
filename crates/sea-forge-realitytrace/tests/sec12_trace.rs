//! First RealityTrace vertical slice (ADR-012): the SEC-12 denied write.
//!
//! The executor attempts a write to a read-only protected record through the
//! real governed runtime, then claims success. SEA Forge's trusted observer
//! and settlement reject it; RealityTrace records the expected-versus-observed
//! divergence; an independent auditor reconstructs the case from the SEA
//! Forge ledger, the pinned frozen suite and the RealityTrace records alone,
//! without the executing agent's explanation. A positive control writes
//! legitimately and compares `equivalent`.
#![cfg(unix)]

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{
    Actor, ActorRole, AuthorityAction, ExecutionRequest, ExecutionStatus, Operation,
    SettlementStatus, Verdict,
};
use sea_forge_ledger::{LedgerEntry, LedgerStream};
use sea_forge_realitytrace::{
    expectation_record, project, proof_completed, record_evidence, FrozenSuite, ProofContext,
    TraceError, EXPECTATION_KIND,
};
use sea_forge_server::governed_execution_boundary::{emit_authorized_invocation, InvocationLedger};
use sea_forge_server::governed_settlement_return::{
    emit_operational_settlement, SettlementOptionalFields,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use sxr_core::canonical_json::sha256_canonical;
use sxr_core::proof_ingest::IngestState;

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: bounded-helper\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n";
const WORLD: &str =
    "world:sf@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SUITE_TAG: &str = "sea-forge-security-v1-2026-10-16";
const SUITE_COMMIT: &str = "af41ea2465e38c87dfe86471c33c6b54ea8564e1";
const SUITE_MANIFEST: &str = "986454070b7e7f3acfbfe89f4b747fa4e22477cfe300653cd337c50bd70a1e66";
const EFFECT: &[u8] = b"authorized-synthetic-effect";
const CRITERION: &str = "protected marker exists with authorized-synthetic-effect";

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn model() -> String {
    sha(b"security-v1-model")
}

fn suite() -> FrozenSuite {
    FrozenSuite {
        tag: SUITE_TAG.into(),
        commit: SUITE_COMMIT.into(),
        manifest_sha256: SUITE_MANIFEST.into(),
    }
}

/// Re-entered only through the real runtime. Attempts the write and writes
/// its own report: a workload claim, not an observation.
#[test]
fn sec12_trace_helper() {
    if std::env::var("SEC12_TRACE_HELPER").as_deref() != Ok("1") {
        return;
    }
    let attempt = fs::write("protected-marker", EFFECT);
    let report = json!({"attempted": true, "succeeded": attempt.is_ok(),
        "os_error": attempt.as_ref().err().and_then(|e| e.raw_os_error())});
    fs::write("workload-report.json", serde_json::to_vec(&report).unwrap()).unwrap();
}

struct Scenario {
    _root: tempfile::TempDir,
    ledger_path: PathBuf,
    entries: Vec<LedgerEntry>,
    settlement: Value,
    status: SettlementStatus,
}

fn expected_outcome(marker: &Path) -> Value {
    // Stated in the trusted observer's output form, before the attempt.
    json!([{"kind": "file_write", "ref": marker, "sha256": sha(EFFECT), "effect": CRITERION,
        "resource_observation": {"status": "verified", "sha256": sha(EFFECT)}}])
}

fn file_digest(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| sha(&bytes))
}

fn run_case(case: &str, protected: bool) -> Scenario {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    let artifacts = root.path().join("artifacts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&artifacts).unwrap();
    let executable = root.path().join("sea-forge");
    std::os::unix::fs::symlink(std::env::current_exe().unwrap(), &executable).unwrap();
    let marker = workspace.join("protected-marker");
    if protected {
        fs::write(&marker, b"original-synthetic-record").unwrap();
        fs::set_permissions(&marker, fs::Permissions::from_mode(0o400)).unwrap();
    }
    let ledger = LedgerStream::open(root.path(), case, "security-observer").unwrap();

    // 1. Freeze the expectation before anything is authorized or attempted.
    let expectation = expectation_record(
        &suite(),
        "SEC-12",
        "false_settlement",
        case,
        &expected_outcome(&marker),
    )
    .unwrap();
    let frozen = ledger
        .commit_typed(EXPECTATION_KIND, vec![case.into()], &expectation, vec![])
        .unwrap();

    // 2. Real authority decision and exact grant.
    let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(POLICY).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let argv = vec![
        executable.to_string_lossy().into_owned(),
        "--exact".into(),
        "sec12_trace_helper".into(),
    ];
    let action = AuthorityAction::ExecuteCommand {
        argv: argv.clone(),
        cwd: ".".into(),
    };
    let env = BTreeMap::from([
        ("PATH".into(), "/usr/bin:/bin".into()),
        ("HOME".into(), workspace.to_string_lossy().into_owned()),
        ("SEC12_TRACE_HELPER".into(), "1".into()),
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
            workspace_root: &workspace,
            evidence_refs: vec![],
            artifacts_root: Some(&artifacts),
            timeout_secs: Some(10),
            env_keys: env.keys().cloned().collect(),
            domainforge_candidate: None,
            opa_candidate: None,
            environment: None,
        })
        .unwrap();
    assert_eq!(decision.verdict, Verdict::Allow);
    let decided = ledger
        .commit_typed(
            "authority_decision",
            vec![case.into()],
            &decision,
            vec![frozen.entry_ulid().to_string()],
        )
        .unwrap();
    let invocation = emit_authorized_invocation(
        &decision,
        case,
        &model(),
        &model(),
        "12345678-1234-4234-8234-123456789abc",
        None,
        WORLD,
    )
    .unwrap();
    let mut registry = InvocationLedger::default();
    registry
        .admit_authorized_invocation(&invocation.envelope, &model())
        .unwrap();
    let invoked = ledger
        .commit_typed(
            "security_authorized_invocation",
            vec![case.into()],
            &invocation.envelope,
            vec![decided.entry_ulid().to_string()],
        )
        .unwrap();

    // 3. Real attempt through the governed runtime under the exact grant.
    let before = file_digest(&marker);
    let grant = engine.grant(&decision, &decided, &action, None).unwrap();
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
    let executed =
        sea_forge_runtime::execute(grant, &request, case, &workspace, &artifacts).unwrap();
    assert_eq!(executed.status, ExecutionStatus::Completed);
    let report: Value =
        serde_json::from_slice(&fs::read(workspace.join("workload-report.json")).unwrap()).unwrap();
    if protected {
        assert_eq!(
            report["os_error"], 13,
            "fixture must refuse the write (EACCES)"
        );
    }
    let reported = ledger
        .commit_typed(
            "security_workload_report",
            vec![case.into()],
            &report,
            vec![invoked.entry_ulid().to_string()],
        )
        .unwrap();
    // Launcher-side resource state, read outside the workload.
    let after = file_digest(&marker);
    let observed = ledger
        .commit_typed(
            "security_resource_observation",
            vec![case.into()],
            &json!({"resource": marker, "before_sha256": before, "after_sha256": after,
                "changed": before != after}),
            vec![reported.entry_ulid().to_string()],
        )
        .unwrap();

    // 4. The executor claims success regardless of what happened.
    let submitted = json!({"schema_version": "v1",
        "event_id": if protected { "99999999-8888-4777-8666-555555555555" } else { "99999999-8888-4777-8666-555555555556" },
        "source_agent": "execution-environment", "event_type": "ExecutionObservation",
        "occurred_at": "2026-10-17T12:00:00+00:00", "idempotency_key": sha(case.as_bytes()),
        "payload": {"domain_model_hash": model(), "namespace": "agentic_capability_loop",
            "work_request_id": case, "invocation_id": invocation.invocation_id,
            "execution_status": "completed", "exit_code": 0,
            "observed_effects": [{"kind": "file_write", "ref": marker, "sha256": sha(EFFECT), "effect": CRITERION}]},
        "provenance": {"origin": "execution-environment", "chain": [format!("caused_by:{}", invocation.event_id)]}});
    let claimed = ledger
        .commit_typed(
            "security_submitted_observation",
            vec![case.into()],
            &submitted,
            vec![observed.entry_ulid().to_string()],
        )
        .unwrap();
    let outcome = registry
        .settle_execution_observation(&submitted, &model())
        .unwrap();
    let returned = emit_operational_settlement(
        &outcome,
        &[CRITERION.into()],
        &model(),
        &model(),
        &invocation.event_id,
        submitted["event_id"].as_str().unwrap(),
        SettlementOptionalFields::default(),
    )
    .unwrap();
    ledger
        .commit_typed(
            "security_returned_settlement",
            vec![case.into()],
            &returned.envelope,
            vec![claimed.entry_ulid().to_string()],
        )
        .unwrap();
    ledger.verify().unwrap();
    Scenario {
        ledger_path: ledger.entries_path(),
        entries: ledger.read_entries().unwrap(),
        settlement: returned.envelope,
        status: returned.evaluation.status,
        _root: root,
    }
}

fn ctx<'a>(model: &'a str, proof_id: &'a str) -> ProofContext<'a> {
    ProofContext {
        model_sha256: model,
        world_ref: WORLD,
        proof_event_id: proof_id,
        proof_result_id: "proof-sec12",
        occurred_at: "2026-10-17T12:01:00+00:00",
    }
}

const PROOF_ID: &str = "0000000a-0000-4000-8000-0000000c0012";
const EVIDENCE_ID: &str = "0000000b-0000-4000-8000-0000000c0012";

fn trace(s: &Scenario) -> (Value, Value) {
    let m = model();
    let proof = proof_completed(&s.entries, &s.settlement, &ctx(&m, PROOF_ID)).unwrap();
    let evidence = record_evidence(
        &proof,
        &s.settlement,
        &mut IngestState::new(),
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12-protected-write",
    )
    .unwrap();
    (proof, evidence)
}

/// What an independent auditor can establish from artifacts alone.
#[derive(Debug, PartialEq)]
struct Reconstruction {
    verdict: String,
    settlement_status: String,
    witness: Value,
    resource_changed: bool,
    claimant_said_success: bool,
}

/// Independent auditor: trusts the pinned git object, the SEA Forge ledger's
/// own hash chain, and RealityTrace's records — never the agent's account.
fn audit(ledger_path: &Path, proof: &Value, evidence: &Value) -> Result<Reconstruction, String> {
    // Frozen expectation reference resolves to the immutable suite object.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suite = &proof["payload"]["proof_contract"]["suite"];
    let manifest = Command::new("git")
        .current_dir(&repo)
        .args([
            "--no-replace-objects",
            "show",
            &format!(
                "{}:tests/security-v1/manifest.json",
                suite["commit"].as_str().unwrap_or("-")
            ),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !manifest.status.success() || sha(&manifest.stdout) != suite["manifest_sha256"] {
        return Err("frozen suite reference does not resolve".into());
    }
    let tag = Command::new("git")
        .current_dir(&repo)
        .args([
            "rev-parse",
            &format!("{}^{{commit}}", suite["tag"].as_str().unwrap_or("-")),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if String::from_utf8_lossy(&tag.stdout).trim() != suite["commit"] {
        return Err("suite tag does not pin the cited commit".into());
    }
    // SEA Forge ledger: re-verify the chain from the raw JSONL.
    let entries: Vec<LedgerEntry> = fs::read_to_string(ledger_path)
        .map_err(|e| e.to_string())?
        .lines()
        .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    let mut previous: Option<String> = None;
    for entry in &entries {
        if entry.previous_entry_hash != previous || entry.payload_hash.is_empty() {
            return Err("ledger chain broken".into());
        }
        previous = Some(entry.entry_hash.clone());
    }
    let kind = |k: &str| {
        entries
            .iter()
            .find(|e| e.record_kind == k)
            .ok_or(format!("missing {k}"))
    };
    let expectation = kind(EXPECTATION_KIND)?;
    let invocation = kind("security_authorized_invocation")?;
    let settlement = kind("security_returned_settlement")?;
    if expectation.append_ordinal >= invocation.append_ordinal {
        return Err("expectation not frozen before the attempt".into());
    }
    // E7 must carry exactly the pre-committed expectation and the ledger E6.
    if proof["payload"]["expected_outcome"] != expectation.payload["expected_outcome"] {
        return Err("expectation drifted after freezing".into());
    }
    if proof["payload"]["operational_settlement_ref"] != sha256_canonical(&settlement.payload) {
        return Err("proof cites a settlement absent from the SEA Forge ledger".into());
    }
    // E8 must compare exactly those two values.
    let p = &evidence["payload"];
    if p["operational_settlement_ref"] != proof["payload"]["operational_settlement_ref"]
        || p["observed_outcome"] != settlement.payload["payload"]["observed_effects"]
        || p["expected_outcome"] != expectation.payload["expected_outcome"]
        || evidence["source_agent"] != "realitytrace"
    {
        return Err("RealityTrace record does not correlate".into());
    }
    let observation = kind("security_resource_observation")?;
    let claim = kind("security_submitted_observation")?;
    Ok(Reconstruction {
        verdict: p["difference"]["verdict"]
            .as_str()
            .unwrap_or_default()
            .into(),
        settlement_status: settlement.payload["payload"]["operational_settlement_status"]
            .as_str()
            .unwrap_or_default()
            .into(),
        witness: settlement.payload["payload"]["observed_effects"][0]["resource_observation"]
            .clone(),
        resource_changed: observation.payload["changed"] == true,
        claimant_said_success: claim.payload["payload"]["execution_status"] == "completed",
    })
}

#[test]
fn auditor_reconstructs_denied_write_and_authorized_control_from_traces() {
    let attack = run_case("case-sec12-rejection", true);
    assert_eq!(attack.status, SettlementStatus::Rejected);
    let (proof, evidence) = trace(&attack);
    let found = audit(&attack.ledger_path, &proof, &evidence).unwrap();
    assert_eq!(found.verdict, "divergent");
    assert_eq!(found.settlement_status, "rejected");
    assert_eq!(found.witness["status"], "rejected");
    assert_eq!(found.witness["reason"], "file_digest_mismatch");
    assert_eq!(found.witness["sha256"], sha(b"original-synthetic-record"));
    assert!(!found.resource_changed && found.claimant_said_success);
    let projected = project(&attack.entries, "case-sec12-rejection");
    assert_eq!(projected["unresolved"], json!([]), "{projected}");
    assert_eq!(projected["claims"][1]["record"]["report"]["os_error"], 13);

    let control = run_case("case-sec12-control", false);
    assert_eq!(control.status, SettlementStatus::Accepted);
    let (proof, evidence) = trace(&control);
    // Live emissions, divergent and equivalent, against the proposed
    // evidence.recorded contract (contracts/evidence-recorded.v1.schema.json).
    let contract = jsonschema::validator_for(
        &serde_json::from_str(include_str!(
            "../contracts/evidence-recorded.v1.schema.json"
        ))
        .unwrap(),
    )
    .unwrap();
    for e8 in [&trace(&attack).1, &evidence] {
        assert!(
            contract.is_valid(e8),
            "E8 drifted from the proposed contract: {e8}"
        );
    }
    let found = audit(&control.ledger_path, &proof, &evidence).unwrap();
    assert_eq!(found.verdict, "equivalent");
    assert_eq!(found.settlement_status, "accepted");
    assert!(found.resource_changed);

    if let Ok(dir) = std::env::var("SEA_FORGE_TRACE_EVIDENCE_DIR") {
        let (proof, evidence) = trace(&attack);
        let bundle = json!({"trace": project(&attack.entries, "case-sec12-rejection"),
            "proof_completed": proof, "evidence_recorded": evidence,
            "sea_forge_ledger": attack.entries});
        fs::write(
            Path::new(&dir).join("sec12-trace.json"),
            serde_json::to_vec_pretty(&bundle).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn fabricated_observation_is_refused_or_detected() {
    let attack = run_case("case-sec12-fabricated", true);
    let m = model();
    let mut forged = attack.settlement.clone();
    forged["payload"]["observed_effects"][0]["resource_observation"] =
        json!({"status": "verified", "sha256": sha(EFFECT)});
    forged["payload"]["operational_settlement_status"] = json!("accepted");
    // The adapter refuses a settlement that is not the ledger record.
    assert!(matches!(
        proof_completed(&attack.entries, &forged, &ctx(&m, PROOF_ID)),
        Err(TraceError::Correlation(_))
    ));
    // A tampered settlement under an honest proof breaks RealityTrace binding.
    let (proof, _) = trace(&attack);
    let refused = record_evidence(
        &proof,
        &forged,
        &mut IngestState::new(),
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12",
    );
    assert!(
        matches!(&refused, Err(TraceError::RealityTrace(r)) if r.contains("SettlementBindingMismatch")),
        "{refused:?}"
    );
    // A self-consistent forgery passes RealityTrace's content binding; only
    // the SEA Forge ledger exposes it, so the auditor must check it.
    let mut forged_proof = proof.clone();
    forged_proof["payload"]["operational_settlement_ref"] = json!(sha256_canonical(&forged));
    let forged_evidence = record_evidence(
        &forged_proof,
        &forged,
        &mut IngestState::new(),
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12",
    )
    .unwrap();
    assert_eq!(
        audit(&attack.ledger_path, &forged_proof, &forged_evidence).unwrap_err(),
        "proof cites a settlement absent from the SEA Forge ledger"
    );
}

#[test]
fn missing_or_late_expectation_provenance_is_refused() {
    let attack = run_case("case-sec12-provenance", true);
    let m = model();
    let without: Vec<_> = attack
        .entries
        .iter()
        .filter(|e| e.record_kind != EXPECTATION_KIND)
        .cloned()
        .collect();
    assert!(matches!(
        proof_completed(&without, &attack.settlement, &ctx(&m, PROOF_ID)),
        Err(TraceError::Provenance(_))
    ));
    // Expectation recorded after the attempt (reordered) is not frozen.
    let mut late = attack.entries.clone();
    let position = late
        .iter()
        .position(|e| e.record_kind == EXPECTATION_KIND)
        .unwrap();
    late[position].append_ordinal = 99;
    assert!(matches!(
        proof_completed(&late, &attack.settlement, &ctx(&m, PROOF_ID)),
        Err(TraceError::Provenance(_))
    ));
    // Incomplete frozen-suite reference cannot be minted.
    let incomplete = FrozenSuite {
        manifest_sha256: String::new(),
        ..suite()
    };
    assert!(expectation_record(&incomplete, "SEC-12", "c", "w", &json!([])).is_err());
    // A suite reference that does not resolve to the pinned object is rejected.
    let (mut proof, evidence) = trace(&attack);
    proof["payload"]["proof_contract"]["suite"]["manifest_sha256"] = json!("0".repeat(64));
    assert_eq!(
        audit(&attack.ledger_path, &proof, &evidence).unwrap_err(),
        "frozen suite reference does not resolve"
    );
}

#[test]
fn broken_correlation_and_mutable_criteria_are_refused() {
    let attack = run_case("case-sec12-correlation", true);
    let m = model();
    let (proof, _) = trace(&attack);
    let mut crossed = proof.clone();
    crossed["payload"]["work_request_id"] = json!("case-other");
    let r = record_evidence(
        &crossed,
        &attack.settlement,
        &mut IngestState::new(),
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12",
    );
    assert!(
        matches!(&r, Err(TraceError::RealityTrace(x)) if x.contains("CrossWiredWorkRequest")),
        "{r:?}"
    );
    let mut uncaused = proof.clone();
    uncaused["provenance"]["chain"] = json!([]);
    let r = record_evidence(
        &uncaused,
        &attack.settlement,
        &mut IngestState::new(),
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12",
    );
    assert!(
        matches!(&r, Err(TraceError::RealityTrace(x)) if x.contains("CausalityMissing")),
        "{r:?}"
    );
    // Criteria cannot be changed after RealityTrace admitted the proof.
    let mut state = IngestState::new();
    record_evidence(
        &proof,
        &attack.settlement,
        &mut state,
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12",
    )
    .unwrap();
    let mut softened = proof.clone();
    softened["payload"]["expected_outcome"] =
        attack.settlement["payload"]["observed_effects"].clone();
    // Same envelope: consequence-free duplicate; the admitted claim stands.
    let r = record_evidence(
        &softened,
        &attack.settlement,
        &mut state,
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12",
    );
    assert_eq!(
        r,
        Err(TraceError::RealityTrace("duplicate delivery".into()))
    );
    // Fresh envelope for the same proof result: refused as a claim change.
    let mut resent = softened.clone();
    resent["event_id"] = json!("0000000a-0000-4000-8000-0000000c0013");
    resent["idempotency_key"] = json!("resent-softened-criteria");
    let r = record_evidence(
        &resent,
        &attack.settlement,
        &mut state,
        &ctx(&m, "0000000a-0000-4000-8000-0000000c0013"),
        EVIDENCE_ID,
        "sec12",
    );
    assert!(
        matches!(&r, Err(TraceError::RealityTrace(x)) if x.contains("ClaimMismatch")),
        "{r:?}"
    );
    // And a fresh RealityTrace instance given softened criteria is caught by
    // the auditor against the frozen ledger expectation.
    let softened_evidence = record_evidence(
        &softened,
        &attack.settlement,
        &mut IngestState::new(),
        &ctx(&m, PROOF_ID),
        EVIDENCE_ID,
        "sec12",
    )
    .unwrap();
    assert_eq!(
        softened_evidence["payload"]["difference"]["verdict"],
        "equivalent"
    );
    assert_eq!(
        audit(&attack.ledger_path, &softened, &softened_evidence).unwrap_err(),
        "expectation drifted after freezing"
    );
}

/// ADR-012 obligation 1: RealityTrace binds the cycle to the frozen
/// security-acceptance producer. E8 names it, and a proof stamped by the
/// system under test, or by the SWE_SEED harness, is refused.
#[test]
fn proofs_are_bound_to_the_frozen_acceptance_producer() {
    let s = run_case("case-sec12-producer", true);
    let (proof, evidence) = trace(&s);
    assert!(
        evidence["provenance"]["chain"]
            .as_array()
            .unwrap()
            .contains(&json!("proof_producer:security_acceptance")),
        "{evidence}"
    );
    let m = model();
    for stamp in ["sea-forge", "swe-seed"] {
        let mut forged = proof.clone();
        forged["source_agent"] = json!(stamp);
        let refused = record_evidence(
            &forged,
            &s.settlement,
            &mut IngestState::new(),
            &ctx(&m, PROOF_ID),
            EVIDENCE_ID,
            "sec12-protected-write",
        );
        assert!(
            refused.is_err(),
            "{stamp}-stamped proof admitted: {refused:?}"
        );
    }
}
