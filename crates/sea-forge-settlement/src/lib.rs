use std::{fs::File, io::Read, path::Path};

use chrono::Utc;

use sea_forge_core::{errors::ForgeError, types::*, RECORD_VERSION};
use sea_forge_sandbox::safe_existing;

pub mod declaration;

pub use declaration::{
    append_declaration, compute_declaration_hash, load_declarations, LocalSettlementAuthority,
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
        settlement_id: "set_01".into(),
        run_id: claim.run_id.clone(),
        status,
        basis,
        review_required,
        settled_at: Utc::now().to_rfc3339(),
        criteria_ref: claim.criteria_ref.clone(),
    })
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
    #[test]
    fn escalation_wins_over_denial() {
        let claim = SettlementClaim {
            run_id: "run".into(),
            plan_item_id: "item_01".into(),
            criteria_ref: None,
            criteria: SettlementCriteria {
                require_exit_zero: true,
                required_artifacts: vec![],
                stdout_must_contain: None,
                require_approval: false,
            },
            execution: None,
            authority_verdicts: vec![Verdict::Deny, Verdict::Escalate],
        };
        assert_eq!(
            settle(&claim, Path::new("."), Path::new("."))
                .unwrap()
                .status,
            SettlementStatus::Escalated
        );
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
                require_approval: false,
            },
            execution: Some(execution),
            authority_verdicts: vec![Verdict::Allow],
        };
        let event = settle(&claim, &workspace, &root).unwrap();
        assert_eq!(event.status, SettlementStatus::Rejected);
        assert!(event
            .basis
            .contains(&"required_artifact_missing:model.sea".into()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
