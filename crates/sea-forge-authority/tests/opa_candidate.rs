//! OPA candidate: a real `opa` 1.21.1 evaluation over the DomainForge Rego
//! bundle, and stub-binary tests proving every evaluation failure fails closed.
//!
//! Tests marked `requires opa 1.21.1 on PATH` need the pinned binary and run
//! with `--include-ignored`. Everything else runs in a plain `cargo test`.

use sea_forge_authority::{
    AuthorityEvaluation, AuthorityPolicyBundle, OpaBinding, OpaCandidate, PolicyAuthorityEngine,
    PolicySource, OPA_VERSION,
};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{
    Actor, ActorRole, AuthorityAction, AuthorityDecision, NormalizedDisposition, Verdict,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};
use tempfile::TempDir;

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/cells/repair-agent/rego"
);
const MODEL_HASH: &str = "sha256:7ed5d9b70ec8a6b74815caa97d0735c3233ed555a739dc943cf9248b3c36631f";
const RESULT_KEY: &str = "godspeed.cells.repair::dependency_files_immutable";
const TIMEOUT: Duration = Duration::from_secs(60);

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A private copy of the fixture bundle at `<tmp>/rego`.
fn staged_bundle() -> TempDir {
    let tmp = TempDir::new().unwrap();
    copy_dir(Path::new(FIXTURES), &tmp.path().join("rego"));
    tmp
}

/// A policy whose only engine is `opa`, optionally with one local allow rule
/// for `write_file`. Without the rule, the local authority denies by default.
fn policy_yaml(local_allow: bool, opa_required: bool) -> String {
    let rules = if local_allow {
        "rules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n"
    } else {
        "rules: []\n"
    };
    format!(
        "version: \"0.1\"\nidentity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\npolicy_engines:\n  - engine: opa\n    fail_mode: closed\n    required: {opa_required}\n{rules}"
    )
}

/// The policy with its `rego_provenance` source pointing at `<base>/rego`,
/// hashed from the provenance file as it is right now.
fn bundle_for(base: &Path, local_allow: bool, opa_required: bool) -> AuthorityPolicyBundle {
    let mut bundle: AuthorityPolicyBundle =
        sea_forge_core::yaml::from_str(&policy_yaml(local_allow, opa_required)).unwrap();
    let digest = sha256_hex(&fs::read(base.join("rego/provenance.json")).unwrap());
    bundle.sources = vec![PolicySource {
        surface: "rego_provenance".into(),
        path: "rego/provenance.json".into(),
        sha256: digest,
    }];
    bundle.source_base = Some(base.to_path_buf());
    bundle
}

fn real_opa() -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|dir| dir.join("opa"))
                .find(|candidate| candidate.is_file())
        })
        .expect("requires opa 1.21.1 on PATH")
}

fn binding(cli: PathBuf) -> OpaBinding {
    OpaBinding {
        cli,
        timeout: TIMEOUT,
    }
}

fn fixture(name: &str) -> Value {
    let path = Path::new(FIXTURES)
        .join("fixtures")
        .join(format!("{name}.json"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn class_of(error: ForgeError) -> &'static str {
    match error {
        ForgeError::Config { class, .. } => class,
        other => panic!("expected a config error, got {other:?}"),
    }
}

/// Runs the policy engine with a single write_file action and the given OPA
/// candidate (or none).
fn decide(bundle: &AuthorityPolicyBundle, candidate: Option<&OpaCandidate>) -> AuthorityDecision {
    let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let binding = bundle.resolve_identity(&actor.actor_id, actor.role.clone());
    let action = AuthorityAction::WriteFile {
        path: "notes.txt".into(),
        content_hint: "fixed".into(),
    };
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: "run_opa",
            case_id: "case_opa",
            plan_item_id: "item_opa",
            sequence: 1,
            action: &action,
            workspace_root: Path::new("/workspace"),
            evidence_refs: vec!["intent:write_file".into()],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            opa_candidate: candidate,
            environment: None,
        })
        .unwrap()
}

/// The OPA entry in the decision's candidate verdicts: its disposition and
/// evidence refs.
fn opa_record(decision: &AuthorityDecision) -> (NormalizedDisposition, Vec<String>) {
    let record = decision
        .candidate_verdicts
        .iter()
        .find(|record| record.engine == "opa")
        .expect("decision records an opa candidate");
    (record.disposition.clone(), record.evidence_refs.clone())
}

fn disposition_name(disposition: &NormalizedDisposition) -> &'static str {
    match disposition {
        NormalizedDisposition::Allow => "allow",
        NormalizedDisposition::Deny => "deny",
        NormalizedDisposition::Escalate => "escalate",
        _ => "other",
    }
}

/// Writes a fake `opa` into `dir`. Every invocation first drops a `spawned`
/// marker there. `version` answers `opa version`; any other call runs
/// `eval_body`.
fn stub(dir: &Path, version: &str, eval_body: &str) -> PathBuf {
    let path = dir.join("opa");
    let script = format!(
        "#!/bin/sh\n: > '{}'\ncase \"$1\" in\n  version) echo 'Version: {version}' ;;\n  *) {eval_body} ;;\nesac\n",
        dir.join("spawned").display()
    );
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// A shell statement that prints `text` verbatim.
fn printf(text: &str) -> String {
    format!("printf '%s' '{}'", text.replace('\'', "'\\''"))
}

fn decision_value(outcome: &str, results: Value, input_errors: Value) -> Value {
    json!({
        "contract": "domainforge.rego.decision/v1",
        "outcome": outcome,
        "results": results,
        "reasons": [],
        "input_errors": input_errors,
        "model": {"hash": MODEL_HASH, "projection": "domainforge-rego/v1"}
    })
}

/// `opa eval --format json` output wrapping one decision value.
fn eval_output(value: &Value) -> String {
    json!({"result": [{"expressions": [{
        "value": value,
        "text": "data.domainforge.policy.decision",
        "location": {"row": 1, "col": 1}
    }]}]})
    .to_string()
}

fn good_allow() -> Value {
    decision_value("allow", json!({RESULT_KEY: "true"}), json!([]))
}

fn valid_input() -> Value {
    json!({"facts": {}})
}

/// Asserts that the candidate, and the whole engine, fail closed with `failure`.
fn assert_fails_closed(bundle: &AuthorityPolicyBundle, cli: PathBuf, failure: &str) {
    let candidate =
        OpaCandidate::evaluate(bundle, &binding(cli), &valid_input()).expect("candidate");
    assert_eq!(candidate.trace().failure.as_deref(), Some(failure));
    let decision = decide(bundle, Some(&candidate));
    assert_eq!(decision.verdict, Verdict::Deny, "{failure}");
    let (disposition, refs) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "deny", "{failure}");
    assert!(refs.contains(&format!("opa-failure:{failure}")), "{refs:?}");
    assert!(!refs.is_empty());
}

// ---------------------------------------------------------------------------
// A. Real opa 1.21.1
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_matches_each_fixture_outcome() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = binding(real_opa());
    for name in ["model", "empty", "incomplete", "malformed"] {
        let fixture = fixture(name);
        let expected = &fixture["expected"];
        let want = expected["outcome"].as_str().unwrap();
        let candidate = OpaCandidate::evaluate(&bundle, &cli, &fixture["input"]).unwrap();
        let decision = decide(&bundle, Some(&candidate));
        let (disposition, refs) = opa_record(&decision);
        assert_eq!(disposition_name(&disposition), want, "fixture {name}");
        let trace = candidate.trace();
        if name == "malformed" {
            // A non-object `facts` is refused before opa is spawned.
            assert_eq!(trace.failure.as_deref(), Some("opa_input_invalid"));
            assert_eq!(decision.verdict, Verdict::Deny);
            continue;
        }
        assert_eq!(trace.failure, None, "fixture {name}");
        assert_eq!(trace.opa_version, "1.21.1");
        assert_eq!(trace.model_hash, MODEL_HASH);
        assert_eq!(trace.projection, "domainforge-rego/v1");
        assert!(trace.provenance_sha256.starts_with("sha256:"));
        assert!(trace.input_sha256.starts_with("sha256:"));
        assert_eq!(trace.outcome.as_deref(), Some(want), "fixture {name}");
        assert_eq!(trace.results, expected["results"], "fixture {name}");
        assert_eq!(trace.reasons, expected["reasons"], "fixture {name}");
        assert!(trace.decision_sha256.is_some());
        assert!(refs.contains(&format!("rego-bundle:{MODEL_HASH}")));
        assert!(refs.contains(&"opa:1.21.1".to_string()));
        assert!(refs.iter().any(|r| r.starts_with("rego-input:sha256:")));
        assert!(refs.iter().any(|r| r.starts_with("rego-decision:sha256:")));
    }
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_allow_fixture_allows_through_engine() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let candidate =
        OpaCandidate::evaluate(&bundle, &binding(real_opa()), &fixture("model")["input"]).unwrap();
    let decision = decide(&bundle, Some(&candidate));
    assert_eq!(decision.verdict, Verdict::Allow);
    let (disposition, refs) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "allow");
    assert!(refs.iter().any(|r| r.starts_with("rego-bundle:sha256:")));
    assert!(refs.contains(&"opa:1.21.1".to_string()));
    assert!(refs.iter().any(|r| r.starts_with("rego-input:sha256:")));
    assert!(refs.iter().any(|r| r.starts_with("rego-decision:sha256:")));
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_deny_fixture_denies_through_engine() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    // The malformed fixture is the only one that denies; it is refused before
    // opa is spawned, and the deny must still reach the final verdict.
    let candidate = OpaCandidate::evaluate(
        &bundle,
        &binding(real_opa()),
        &fixture("malformed")["input"],
    )
    .unwrap();
    let decision = decide(&bundle, Some(&candidate));
    assert_eq!(decision.verdict, Verdict::Deny);
    let (disposition, refs) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "deny");
    assert!(refs.iter().any(|r| r.starts_with("rego-bundle:sha256:")));
    assert!(refs.contains(&"opa-failure:opa_input_invalid".to_string()));
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_allow_cannot_override_local_deny() {
    let staged = staged_bundle();
    // No local allow rule: the local authority denies by default.
    let bundle = bundle_for(staged.path(), false, true);
    let candidate =
        OpaCandidate::evaluate(&bundle, &binding(real_opa()), &fixture("model")["input"]).unwrap();
    let decision = decide(&bundle, Some(&candidate));
    let (disposition, _) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "allow");
    assert_eq!(decision.verdict, Verdict::Deny);
}

// ---------------------------------------------------------------------------
// A2. Real opa 1.21.1 over the gate fixture, which reaches deny and escalate.
// ---------------------------------------------------------------------------

const GATE_FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/policies/gate");

/// A private copy of the gate bundle at `<tmp>/rego`.
fn staged_gate() -> TempDir {
    let tmp = TempDir::new().unwrap();
    copy_dir(
        &Path::new(GATE_FIXTURES).join("rego"),
        &tmp.path().join("rego"),
    );
    tmp
}

fn gate_input(name: &str) -> Value {
    let path = Path::new(GATE_FIXTURES)
        .join("inputs")
        .join(format!("{name}.json"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// Evaluates the gate bundle with real opa. The local rule allows the action,
/// so the final verdict is decided by OPA's outcome.
fn gate_run(staged: &TempDir, input: &Value) -> (OpaCandidate, AuthorityDecision) {
    let bundle = bundle_for(staged.path(), true, true);
    let candidate = OpaCandidate::evaluate(&bundle, &binding(real_opa()), input).unwrap();
    let decision = decide(&bundle, Some(&candidate));
    (candidate, decision)
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_denies_a_violated_policy() {
    let staged = staged_gate();
    let (candidate, decision) = gate_run(&staged, &gate_input("deny"));
    let trace = candidate.trace();
    assert_eq!(trace.failure, None);
    assert_eq!(trace.outcome.as_deref(), Some("deny"));
    let (disposition, refs) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "deny");
    assert!(refs.contains(&"opa:1.21.1".to_string()));
    assert_eq!(decision.verdict, Verdict::Deny);
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_escalates_an_unknown() {
    let staged = staged_gate();
    let (candidate, decision) = gate_run(&staged, &gate_input("escalate"));
    let trace = candidate.trace();
    assert_eq!(trace.failure, None);
    assert_eq!(trace.outcome.as_deref(), Some("escalate"));
    let (disposition, refs) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "escalate");
    assert!(refs.contains(&"opa:1.21.1".to_string()));
    assert_eq!(decision.verdict, Verdict::Escalate);
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_allows_a_satisfied_policy() {
    let staged = staged_gate();
    let (candidate, decision) = gate_run(&staged, &gate_input("allow"));
    let trace = candidate.trace();
    assert_eq!(trace.failure, None);
    assert_eq!(trace.outcome.as_deref(), Some("allow"));
    let (disposition, refs) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "allow");
    assert!(refs.contains(&"opa:1.21.1".to_string()));
    assert_eq!(decision.verdict, Verdict::Allow);
}

// ---------------------------------------------------------------------------
// Bundle integrity and configuration (no opa is spawned: these return Err).
// ---------------------------------------------------------------------------

/// Builds the bundle, applies `edit` to the staged copy, then evaluates with a
/// binary that must never run.
fn evaluate_after(edit: impl FnOnce(&Path)) -> ForgeError {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    edit(&staged.path().join("rego"));
    OpaCandidate::evaluate(
        &bundle,
        &binding(PathBuf::from("/nonexistent/opa")),
        &valid_input(),
    )
    .expect_err("tampered bundle must not evaluate")
}

#[test]
fn edited_rego_is_tampered() {
    let error = evaluate_after(|rego| {
        let path = rego.join("bundle/domainforge/policy/decision.rego");
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\n# tampered\n");
        fs::write(path, text).unwrap();
    });
    assert_eq!(class_of(error), "opa_bundle_tampered");
}

#[test]
fn extra_file_under_bundle_is_tampered() {
    let error = evaluate_after(|rego| {
        fs::write(rego.join("bundle/extra.rego"), "package extra\n").unwrap();
    });
    assert_eq!(class_of(error), "opa_bundle_tampered");
}

#[test]
fn deleted_data_json_is_tampered() {
    let error = evaluate_after(|rego| {
        fs::remove_file(rego.join("bundle/data.json")).unwrap();
    });
    assert_eq!(class_of(error), "opa_bundle_tampered");
}

#[test]
fn symlink_under_bundle_is_tampered() {
    let error = evaluate_after(|rego| {
        std::os::unix::fs::symlink(
            rego.join("bundle/data.json"),
            rego.join("bundle/linked.json"),
        )
        .unwrap();
    });
    assert_eq!(class_of(error), "opa_bundle_tampered");
}

#[test]
fn edited_provenance_is_source_hash_mismatch() {
    let error = evaluate_after(|rego| {
        let path = rego.join("provenance.json");
        let mut text = fs::read_to_string(&path).unwrap();
        text.push('\n');
        fs::write(path, text).unwrap();
    });
    assert_eq!(class_of(error), "source_hash_mismatch");
}

#[test]
fn configuration_errors_are_errors_not_candidates() {
    let staged = staged_bundle();
    let cli = binding(PathBuf::from("/nonexistent/opa"));

    let mut no_engine = bundle_for(staged.path(), true, true);
    no_engine.policy_engines.clear();
    let error = OpaCandidate::evaluate(&no_engine, &cli, &valid_input()).unwrap_err();
    assert_eq!(class_of(error), "opa_engine_not_configured");

    let mut no_source = bundle_for(staged.path(), true, true);
    no_source.sources.clear();
    let error = OpaCandidate::evaluate(&no_source, &cli, &valid_input()).unwrap_err();
    assert_eq!(class_of(error), "opa_bundle_missing");

    let bundle = bundle_for(staged.path(), true, true);
    let relative = binding(PathBuf::from("opa"));
    let error = OpaCandidate::evaluate(&bundle, &relative, &valid_input()).unwrap_err();
    assert_eq!(class_of(error), "opa_binding_invalid");
}

// ---------------------------------------------------------------------------
// B. Stub opa: every failure fails closed (no real opa needed).
// ---------------------------------------------------------------------------

#[test]
fn stub_valid_allow_reaches_allow() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = stub(
        staged.path(),
        OPA_VERSION,
        &printf(&eval_output(&good_allow())),
    );
    let candidate = OpaCandidate::evaluate(&bundle, &binding(cli), &valid_input()).unwrap();
    assert_eq!(candidate.trace().failure, None);
    assert_eq!(decide(&bundle, Some(&candidate)).verdict, Verdict::Allow);
}

/// An allow candidate evaluated over one policy's Rego bundle is refused by an
/// engine whose policy names a different bundle, required or not.
#[test]
fn candidate_from_another_bundle_is_denied() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = stub(
        staged.path(),
        OPA_VERSION,
        &printf(&eval_output(&good_allow())),
    );
    let candidate = OpaCandidate::evaluate(&bundle, &binding(cli), &valid_input()).unwrap();
    assert_eq!(candidate.trace().failure, None);
    assert_eq!(decide(&bundle, Some(&candidate)).verdict, Verdict::Allow);

    let other = staged_gate();
    for required in [true, false] {
        let foreign = bundle_for(other.path(), true, required);
        assert_ne!(foreign.sources[0].sha256, bundle.sources[0].sha256);
        let decision = decide(&foreign, Some(&candidate));
        assert_eq!(decision.verdict, Verdict::Deny, "required={required}");
        let (disposition, _) = opa_record(&decision);
        assert_eq!(disposition_name(&disposition), "deny");
    }

    let mut unsourced = bundle_for(staged.path(), true, true);
    unsourced.sources.clear();
    assert_eq!(decide(&unsourced, Some(&candidate)).verdict, Verdict::Deny);
}

#[test]
fn stub_escalate_reaches_escalate() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let value = decision_value("escalate", json!({RESULT_KEY: "unknown"}), json!([]));
    let cli = stub(staged.path(), OPA_VERSION, &printf(&eval_output(&value)));
    let candidate = OpaCandidate::evaluate(&bundle, &binding(cli), &valid_input()).unwrap();
    let decision = decide(&bundle, Some(&candidate));
    assert_eq!(decision.verdict, Verdict::Escalate);
    let (disposition, _) = opa_record(&decision);
    assert_eq!(disposition_name(&disposition), "escalate");
}

#[test]
fn stub_wrong_version_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = stub(staged.path(), "9.9.9", &printf(&eval_output(&good_allow())));
    assert_fails_closed(&bundle, cli, "opa_version_mismatch");
}

#[test]
fn stub_nonzero_exit_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = stub(staged.path(), OPA_VERSION, "exit 3");
    assert_fails_closed(&bundle, cli, "opa_exit_nonzero");
}

#[test]
fn stub_garbage_stdout_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = stub(staged.path(), OPA_VERSION, &printf("not json"));
    assert_fails_closed(&bundle, cli, "opa_output_invalid");
}

#[test]
fn stub_empty_result_set_fails_closed() {
    for body in [r#"{"result":[]}"#, "{}"] {
        let staged = staged_bundle();
        let bundle = bundle_for(staged.path(), true, true);
        let cli = stub(staged.path(), OPA_VERSION, &printf(body));
        assert_fails_closed(&bundle, cli, "opa_decision_undefined");
    }
}

#[test]
fn stub_wrong_contract_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let mut value = good_allow();
    value["contract"] = json!("domainforge.rego.decision/v0");
    let cli = stub(staged.path(), OPA_VERSION, &printf(&eval_output(&value)));
    assert_fails_closed(&bundle, cli, "opa_contract_mismatch");
}

#[test]
fn stub_wrong_model_hash_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let mut value = good_allow();
    value["model"]["hash"] = json!("sha256:someone-else");
    let cli = stub(staged.path(), OPA_VERSION, &printf(&eval_output(&value)));
    assert_fails_closed(&bundle, cli, "opa_model_mismatch");
}

#[test]
fn stub_bad_outcome_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let mut value = good_allow();
    value["outcome"] = json!("maybe");
    let cli = stub(staged.path(), OPA_VERSION, &printf(&eval_output(&value)));
    assert_fails_closed(&bundle, cli, "opa_decision_invalid");
}

#[test]
fn stub_allow_with_unknown_result_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let value = decision_value("allow", json!({RESULT_KEY: "unknown"}), json!([]));
    let cli = stub(staged.path(), OPA_VERSION, &printf(&eval_output(&value)));
    assert_fails_closed(&bundle, cli, "opa_decision_invalid");
}

#[test]
fn stub_allow_with_input_errors_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let value = decision_value(
        "allow",
        json!({RESULT_KEY: "true"}),
        json!(["facts: expected object"]),
    );
    let cli = stub(staged.path(), OPA_VERSION, &printf(&eval_output(&value)));
    assert_fails_closed(&bundle, cli, "opa_decision_invalid");
}

#[test]
fn stub_timeout_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = stub(staged.path(), OPA_VERSION, "/bin/sleep 5");
    let candidate = OpaCandidate::evaluate(
        &bundle,
        &OpaBinding {
            cli,
            timeout: Duration::from_millis(300),
        },
        &valid_input(),
    )
    .unwrap();
    assert_eq!(candidate.trace().failure.as_deref(), Some("opa_timeout"));
    let decision = decide(&bundle, Some(&candidate));
    assert_eq!(decision.verdict, Verdict::Deny);
}

#[test]
fn stub_non_object_input_is_refused_before_spawn() {
    for input in [
        json!("text"),
        json!([1, 2]),
        json!({"facts": null}),
        json!({"other": 1}),
    ] {
        let staged = staged_bundle();
        let bundle = bundle_for(staged.path(), true, true);
        let cli = stub(
            staged.path(),
            OPA_VERSION,
            &printf(&eval_output(&good_allow())),
        );
        let candidate = OpaCandidate::evaluate(&bundle, &binding(cli), &input).unwrap();
        assert_eq!(
            candidate.trace().failure.as_deref(),
            Some("opa_input_invalid"),
            "{input}"
        );
        assert!(!staged.path().join("spawned").exists(), "{input}");
        let decision = decide(&bundle, Some(&candidate));
        assert_eq!(decision.verdict, Verdict::Deny, "{input}");
    }
}

#[test]
fn stub_missing_binary_fails_closed() {
    let staged = staged_bundle();
    let bundle = bundle_for(staged.path(), true, true);
    let cli = PathBuf::from("/nonexistent/dir/opa");
    assert_fails_closed(&bundle, cli, "opa_unavailable");
}

#[test]
fn opa_not_configured_or_unavailable_never_allows() {
    let staged = staged_bundle();
    // required: true, no candidate -> Deny.
    let required = bundle_for(staged.path(), true, true);
    let decision = decide(&required, None);
    assert_eq!(decision.verdict, Verdict::Deny);
    let (disposition, refs) = match decision
        .candidate_verdicts
        .iter()
        .find(|r| r.engine == "opa")
    {
        Some(record) => (record.disposition.clone(), record.evidence_refs.clone()),
        None => panic!("missing opa record"),
    };
    assert_eq!(disposition_name(&disposition), "deny");
    assert!(!refs.is_empty());

    // required: false, no candidate -> Escalate, never Allow.
    let optional = bundle_for(staged.path(), true, false);
    let decision = decide(&optional, None);
    assert_eq!(decision.verdict, Verdict::Escalate);
}
