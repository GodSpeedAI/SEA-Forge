use std::{fs::File, io::Read, path::Path};

use chrono::Utc;

use sea_forge_core::{errors::ForgeError, ids, types::*, RECORD_VERSION};
use sea_forge_sandbox::safe_existing;

pub mod declaration;

pub use declaration::{
    append_declaration, append_declaration_ledgered, append_declaration_ledgered_once,
    compute_declaration_hash, load_declarations, CommandSweSeedTransport, LocalSettlementAuthority,
    SettlementAuthority, SweSeedResponse, SweSeedSettlementAuthority, SweSeedTransport,
};

pub fn settle(
    claim: &SettlementClaim,
    workspace: &Path,
    run_dir: &Path,
) -> Result<SettlementEvent, ForgeError> {
    let mut basis = Vec::new();
    let (status, review_required) = if claim.authority_verdicts.contains(&Verdict::Escalate) {
        basis.push("authority_escalate".into());
        (SettlementStatus::Escalated, true)
    } else if claim.authority_verdicts.contains(&Verdict::Deny) {
        basis.push("authority_deny".into());
        (SettlementStatus::Rejected, false)
    } else {
        basis.push("authority_allow".into());
        match &claim.execution {
            None if claim.write_only => {
                // F-10: a write-only item has no process result to settle
                // against. Accept on the materialized artifacts and record an
                // honest `write_only` basis — never a fabricated process result.
                basis.push("write_only".into());
                let mut accepted = true;
                for path in &claim.criteria.required_artifacts {
                    let present =
                        safe_existing(workspace, path).is_ok_and(|candidate| candidate.is_file());
                    basis.push(format!(
                        "required_artifact_{}:{path}",
                        if present { "present" } else { "missing" }
                    ));
                    accepted &= present;
                }
                (
                    if accepted {
                        SettlementStatus::Accepted
                    } else {
                        SettlementStatus::Rejected
                    },
                    false,
                )
            }
            None => (SettlementStatus::Rejected, false),
            Some(execution) => match execution.status {
                ExecutionStatus::SpawnFailed => {
                    basis.push("spawn_failed".into());
                    (SettlementStatus::Rejected, false)
                }
                ExecutionStatus::TimedOut => {
                    basis.push("timed_out".into());
                    (SettlementStatus::Rejected, false)
                }
                ExecutionStatus::SandboxViolation => {
                    basis.push("jail_violation".into());
                    (SettlementStatus::Rejected, false)
                }
                // F-20: the stderr heuristic cannot prove the jail denied
                // anything — the child may have failed on its own permission
                // error. Rejected either way, but the durable basis says
                // suspected, never definite.
                ExecutionStatus::SuspectedSandboxViolation => {
                    basis.push("suspected_jail_violation".into());
                    (SettlementStatus::Rejected, false)
                }
                ExecutionStatus::Completed => {
                    let mut accepted = true;
                    if claim.criteria.require_exit_zero {
                        let zero = execution.exit_code == Some(0);
                        basis.push(if zero { "exit_zero" } else { "exit_nonzero" }.into());
                        accepted &= zero;
                    }
                    for path in &claim.criteria.required_artifacts {
                        let present = safe_existing(workspace, path)
                            .is_ok_and(|candidate| candidate.is_file());
                        basis.push(format!(
                            "required_artifact_{}:{path}",
                            if present { "present" } else { "missing" }
                        ));
                        accepted &= present;
                    }
                    if let Some(needle) = &claim.criteria.stdout_must_contain {
                        let stdout = safe_existing(run_dir, &execution.stdout_path)?;
                        let matches = file_contains(&stdout, needle)?;
                        basis.push(
                            if matches {
                                "stdout_match"
                            } else {
                                "stdout_mismatch"
                            }
                            .into(),
                        );
                        accepted &= matches;
                    }
                    // Evaluator scores recorded as evidence (§10.6 — never standing).
                    for (name, score) in &claim.evaluator_scores {
                        basis.push(format!("evaluator_score:{name}={score}"));
                    }
                    // Batch settlement (§7.6).
                    if let Some(batch) = &claim.batch {
                        basis.push(format!("batch_pass_ratio:{}", batch.pass_ratio));
                        basis.push(format!("batch_threshold:{}", batch.min_pass_ratio));
                        basis.push(format!("batch_passed:{}/{}", batch.passed, batch.total));
                        if batch.pass_ratio < batch.min_pass_ratio {
                            basis.push("batch_below_threshold".into());
                            accepted = false;
                        }
                        // Write failing records to quarantine (never silently dropped).
                        if !batch.failures.is_empty() {
                            let q_dir = run_dir.join("quarantine");
                            std::fs::create_dir_all(&q_dir)
                                .map_err(|e| ForgeError::io("create quarantine dir", e))?;
                            // The plan-item id is interpolated into a filesystem path;
                            // validate it as a single safe segment (F-17) so a
                            // traversal-shaped id cannot write outside `run_dir`.
                            if !sea_forge_core::path::valid_id_segment(&claim.plan_item_id, 128) {
                                return Err(ForgeError::Input(format!(
                                    "unsafe plan item id: {}",
                                    claim.plan_item_id
                                )));
                            }
                            let q_path = q_dir.join(format!("{}.jsonl", claim.plan_item_id));
                            let mut buf = Vec::new();
                            for failure in &batch.failures {
                                let line = serde_json::to_vec(failure)?;
                                buf.extend(line);
                                buf.push(b'\n');
                            }
                            std::fs::write(&q_path, buf)
                                .map_err(|e| ForgeError::io("write quarantine", e))?;
                        }
                    }
                    (
                        if accepted {
                            SettlementStatus::Accepted
                        } else {
                            SettlementStatus::Rejected
                        },
                        false,
                    )
                }
            },
        }
    };
    if claim.criteria_ref.is_none() {
        basis.push("legacy_unattributed_criteria".into());
    }
    Ok(SettlementEvent {
        version: RECORD_VERSION.into(),
        // Every other settlement site in the workspace mints an id here
        // (`case-runner:183,218`, `cli/plan_pipeline.rs:601`, `server/lib.rs:364`).
        // This one returned the literal `set_01`, so every settlement `settle`
        // produced shared one id: two settlements in a cell were
        // indistinguishable and a `settlement_ref` pointing at `set_01` named
        // all of them at once.
        settlement_id: ids::random_id("set")?,
        run_id: claim.run_id.clone(),
        status,
        basis,
        review_required,
        settled_at: Utc::now().to_rfc3339(),
        criteria_ref: claim.criteria_ref.clone(),
    })
}

/// Returns `None` when no agent-output criterion was declared.
pub fn evaluate_agent_output(output: &str, required: Option<&str>) -> Option<bool> {
    required.map(|needle| output.contains(needle))
}

/// Minimal JSON Schema subset validator (M13 T15, spec §7.3
/// `response_schema`). No JSON Schema crate is approved for this workspace
/// (ADR-002/ADR-003 approve no such dependency), so per the plan's redesign
/// trigger this validates a narrow, explicitly-scoped subset instead of
/// pretending arbitrary JSON Schema is enforced: `type`, `enum`,
/// `properties` (object member schemas), `required` (object member
/// presence), and `items` (array element schema, applied uniformly). Any
/// other keyword is not enforced. `output` is treated as untrusted JSON
/// text; malformed JSON never validates.
pub fn validate_response_schema(schema: &serde_json::Value, output: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(output) {
        Ok(value) => schema_matches(schema, &value),
        Err(_) => false,
    }
}

fn schema_matches(schema: &serde_json::Value, value: &serde_json::Value) -> bool {
    let Some(schema_obj) = schema.as_object() else {
        return true;
    };
    if let Some(enum_value) = schema_obj.get("enum") {
        let Some(allowed) = enum_value.as_array() else {
            return false;
        };
        if !allowed.contains(value) {
            return false;
        }
    }
    if let Some(expected_type) = schema_obj.get("type").and_then(|v| v.as_str()) {
        if !json_type_matches(expected_type, value) {
            return false;
        }
    }
    if let Some(properties_value) = schema_obj.get("properties") {
        let Some(properties) = properties_value.as_object() else {
            return false;
        };
        let Some(object) = value.as_object() else {
            return false;
        };
        for (key, sub_schema) in properties {
            if let Some(field) = object.get(key) {
                if !schema_matches(sub_schema, field) {
                    return false;
                }
            }
        }
    }
    if let Some(required_value) = schema_obj.get("required") {
        let Some(required) = required_value.as_array() else {
            return false;
        };
        let Some(object) = value.as_object() else {
            return false;
        };
        if !required.iter().all(|name| match name.as_str() {
            Some(name) => object.contains_key(name),
            None => false,
        }) {
            return false;
        }
    }
    if let Some(items_schema) = schema_obj.get("items") {
        let Some(array) = value.as_array() else {
            return false;
        };
        if !array.iter().all(|item| schema_matches(items_schema, item)) {
            return false;
        }
    }
    true
}

fn json_type_matches(expected: &str, value: &serde_json::Value) -> bool {
    match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        // Unrecognized type keyword: not enforced rather than falsely rejected.
        _ => true,
    }
}

#[cfg(test)]
mod response_schema_tests {
    use super::*;

    #[test]
    fn valid_object_satisfies_type_properties_and_required() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["status"],
            "properties": {"status": {"type": "string", "enum": ["accepted"]}}
        });
        assert!(validate_response_schema(
            &schema,
            r#"{"status":"accepted"}"#
        ));
    }

    #[test]
    fn wrong_enum_value_fails() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"status": {"enum": ["accepted"]}}
        });
        assert!(!validate_response_schema(&schema, r#"{"status":"other"}"#));
    }

    #[test]
    fn missing_required_field_fails() {
        let schema = serde_json::json!({"type": "object", "required": ["status"]});
        assert!(!validate_response_schema(&schema, r#"{"other":1}"#));
    }

    #[test]
    fn malformed_json_output_fails() {
        let schema = serde_json::json!({"type": "object"});
        assert!(!validate_response_schema(&schema, "not json"));
    }

    #[test]
    fn array_items_validated_uniformly() {
        let schema = serde_json::json!({"type": "array", "items": {"type": "integer"}});
        assert!(validate_response_schema(&schema, "[1,2,3]"));
        assert!(!validate_response_schema(&schema, "[1,\"x\",3]"));
    }

    #[test]
    fn malformed_enum_shape_fails_closed() {
        let schema = serde_json::json!({"enum": "accepted"});
        assert!(!validate_response_schema(&schema, r#""accepted""#));
    }

    #[test]
    fn malformed_properties_shape_fails_closed() {
        let schema = serde_json::json!({"properties": ["status"]});
        assert!(!validate_response_schema(
            &schema,
            r#"{"status":"accepted"}"#
        ));
    }

    #[test]
    fn malformed_required_shape_fails_closed() {
        let schema = serde_json::json!({"required": "status"});
        assert!(!validate_response_schema(
            &schema,
            r#"{"status":"accepted"}"#
        ));
    }

    #[test]
    fn required_entries_that_are_not_strings_fail_closed() {
        let schema = serde_json::json!({"required": [42]});
        assert!(!validate_response_schema(
            &schema,
            r#"{"status":"accepted"}"#
        ));
    }
}

#[cfg(test)]
mod agent_output_tests {
    use super::*;

    #[test]
    fn literal_agent_output_mismatch_rejects_narrated_success() {
        assert_eq!(
            evaluate_agent_output("task completed successfully", Some("artifact: accepted")),
            Some(false)
        );
    }
}

/// Evaluate a batch of records against a min_pass_ratio threshold (§7.6).
/// Pure: scores are pre-computed; this function only tallies and quarantines.
pub fn evaluate_batch(
    records: &[serde_json::Value],
    scores: &[f64],
    min_pass_ratio: f64,
) -> BatchEvaluationResult {
    let total = records.len();
    let mut passed = 0usize;
    let mut failures = Vec::new();
    for (i, record) in records.iter().enumerate() {
        let score = scores.get(i).copied().unwrap_or(0.0);
        if score >= 0.5 {
            passed += 1;
        } else {
            failures.push(BatchFailure {
                record: record.clone(),
                score,
                evidence_ref: format!("record_{i}"),
            });
        }
    }
    let pass_ratio = if total == 0 {
        1.0
    } else {
        passed as f64 / total as f64
    };
    BatchEvaluationResult {
        total,
        passed,
        pass_ratio,
        min_pass_ratio,
        failures,
    }
}

fn file_contains(path: &Path, needle: &str) -> Result<bool, ForgeError> {
    if needle.is_empty() {
        return Ok(true);
    }
    let mut file =
        File::open(path).map_err(|error| ForgeError::io("open captured stdout", error))?;
    let needle = needle.as_bytes();
    let mut chunk = [0_u8; 8192];
    let mut overlap = Vec::new();
    loop {
        let count = file
            .read(&mut chunk)
            .map_err(|error| ForgeError::io("read captured stdout", error))?;
        if count == 0 {
            return Ok(false);
        }
        let mut window = Vec::with_capacity(overlap.len() + count);
        window.extend_from_slice(&overlap);
        window.extend_from_slice(&chunk[..count]);
        if window
            .windows(needle.len())
            .any(|candidate| candidate == needle)
        {
            return Ok(true);
        }
        let keep = needle.len().saturating_sub(1).min(window.len());
        overlap.clear();
        overlap.extend_from_slice(&window[window.len() - keep..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn escalation_wins_over_denial() {
        let claim = SettlementClaim {
            run_id: "run".into(),
            plan_item_id: "item_01".into(),
            criteria_ref: None,
            criteria: SettlementCriteria {
                require_exit_zero: true,
                ..Default::default()
            },
            execution: None,
            authority_verdicts: vec![Verdict::Deny, Verdict::Escalate],
            evaluator_scores: BTreeMap::new(),
            batch: None,
            write_only: false,
        };
        assert_eq!(
            settle(&claim, Path::new("."), Path::new("."))
                .unwrap()
                .status,
            SettlementStatus::Escalated
        );
    }

    #[test]
    fn two_settlements_are_distinguishable() {
        let claim = SettlementClaim {
            run_id: "run".into(),
            plan_item_id: "item_01".into(),
            criteria_ref: None,
            criteria: SettlementCriteria::default(),
            execution: None,
            authority_verdicts: vec![Verdict::Deny],
            evaluator_scores: BTreeMap::new(),
            batch: None,
            write_only: false,
        };
        let first = settle(&claim, Path::new("."), Path::new(".")).unwrap();
        let second = settle(&claim, Path::new("."), Path::new(".")).unwrap();
        assert_ne!(first.settlement_id, second.settlement_id);
        assert!(first.settlement_id.starts_with("set_"));
    }

    #[test]
    fn zero_exit_without_required_artifact_is_rejected() {
        let root =
            std::env::temp_dir().join(format!("sea-forge-false-success-{}", std::process::id()));
        let workspace = root.join("workspace");
        let artifacts = root.join("artifacts");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&artifacts).unwrap();
        std::fs::write(artifacts.join("stdout.txt"), "sea-forge: model valid").unwrap();
        let execution = ExecutionResult {
            status: ExecutionStatus::Completed,
            exit_code: Some(0),
            stdout_path: "artifacts/stdout.txt".into(),
            stderr_path: "artifacts/stderr.txt".into(),
            started_at: "start".into(),
            finished_at: "finish".into(),
        };
        let claim = SettlementClaim {
            run_id: "run".into(),
            plan_item_id: "item_01".into(),
            criteria_ref: None,
            criteria: SettlementCriteria {
                require_exit_zero: true,
                required_artifacts: vec!["model.sea".into()],
                stdout_must_contain: Some("model valid".into()),
                ..Default::default()
            },
            execution: Some(execution),
            authority_verdicts: vec![Verdict::Allow],
            evaluator_scores: BTreeMap::new(),
            batch: None,
            write_only: false,
        };
        let event = settle(&claim, &workspace, &root).unwrap();
        assert_eq!(event.status, SettlementStatus::Rejected);
        assert!(event
            .basis
            .contains(&"required_artifact_missing:model.sea".into()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
