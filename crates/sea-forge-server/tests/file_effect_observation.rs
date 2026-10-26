#![cfg(unix)]
//! Implementation tests for the resource witness; v1 remains immutable.
use sea_forge_core::types::SettlementStatus;
use sea_forge_server::governed_execution_boundary::InvocationLedger;
use sea_forge_server::governed_settlement_return::evaluate_operational_settlement;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn evaluate(root: Option<&std::path::Path>, effect: Value) -> SettlementStatus {
    let model = digest(b"file-observer-model");
    let parent = "12345678-1234-4234-8234-123456789abc";
    let mut constraints = json!({});
    if let Some(root) = root {
        constraints["workspace_root"] = json!(root);
    }
    let invocation = json!({
        "schema_version":"v1", "event_type":"AuthorizedInvocation",
        "source_agent":"sea-forge", "event_id":parent,
        "payload":{"domain_model_hash":model, "namespace":"agentic_capability_loop",
            "world_ref":format!("world:sf@sha256:{}", "b".repeat(64)),
            "work_request_id":"work-file", "invocation_id":"inv-file",
            "authority_decision_id":"auth-file", "operation":"execute_command",
            "resource":"sea-forge", "execution_constraints":constraints},
        "provenance":{"origin":"sea-forge", "chain":["caused_by:governed-file"]}
    });
    let mut ledger = InvocationLedger::default();
    ledger
        .admit_authorized_invocation(&invocation, &model)
        .unwrap();
    let observation = json!({
        "schema_version":"v1", "event_type":"ExecutionObservation",
        "source_agent":"execution-environment",
        "event_id":"99999999-8888-4777-8666-555555555555",
        "payload":{"domain_model_hash":model, "namespace":"agentic_capability_loop",
            "work_request_id":"work-file", "invocation_id":"inv-file",
            "execution_status":"completed", "observed_effects":[effect]},
        "provenance":{"origin":"execution-environment", "chain":[format!("caused_by:{parent}")]}
    });
    let outcome = ledger
        .settle_execution_observation(&observation, &model)
        .unwrap();
    evaluate_operational_settlement(&outcome, &["approved file exists".into()])
        .unwrap()
        .status
}

#[test]
fn file_claims_require_scoped_resource_evidence_even_with_forged_witnesses() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&workspace).unwrap();
    std::fs::write(workspace.join("result"), b"actual contents").unwrap();
    std::fs::write(temp.path().join("outside"), b"actual contents").unwrap();
    let effect = |reference: Value, expected: &str| {
        json!({
            "kind":"file_write", "ref":reference, "sha256":expected,
            "effect":"approved file exists",
            "resource_observation":{"status":"verified", "sha256":expected}
        })
    };
    let correct = digest(b"actual contents");
    // Unsupported observers must fail closed, even for matching real bytes.
    let matching_status = if cfg!(unix) {
        SettlementStatus::Accepted
    } else {
        SettlementStatus::Rejected
    };
    assert_eq!(
        evaluate(Some(&workspace), effect(json!("result"), &correct)),
        matching_status
    );
    assert_eq!(
        evaluate(
            Some(&workspace),
            effect(json!(workspace.join("result")), &correct)
        ),
        matching_status
    );
    for reference in [
        json!("result"),
        json!("missing"),
        json!("../outside"),
        json!(temp.path().join("outside")),
    ] {
        assert_eq!(
            evaluate(
                Some(&workspace),
                effect(reference, &digest(b"claimed contents"))
            ),
            SettlementStatus::Rejected
        );
    }
    assert_eq!(
        evaluate(None, effect(json!(workspace.join("result")), &correct)),
        SettlementStatus::Rejected
    );
    for reference in [json!("../outside"), json!(temp.path().join("outside"))] {
        let claim = effect(reference, &correct);
        assert_eq!(
            evaluate(Some(&workspace), claim.clone()),
            SettlementStatus::Rejected
        );
        assert_eq!(
            evaluate(Some(&workspace), json!({"details":claim})),
            SettlementStatus::Rejected
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(temp.path().join("outside"), workspace.join("link")).unwrap();
        assert_eq!(
            evaluate(Some(&workspace), effect(json!("link"), &correct)),
            SettlementStatus::Rejected
        );
    }
}
