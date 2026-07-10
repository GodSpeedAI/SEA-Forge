use std::{fs, path::Path};

use chrono::Utc;

use crate::{types::*, RECORD_VERSION};

pub fn settle(claim: &SettlementClaim, workspace: &Path, run_dir: &Path) -> SettlementEvent {
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
                ExecutionStatus::Completed => {
                    let mut accepted = true;
                    if claim.criteria.require_exit_zero {
                        let zero = execution.exit_code == Some(0);
                        basis.push(if zero { "exit_zero" } else { "exit_nonzero" }.into());
                        accepted &= zero;
                    }
                    for path in &claim.criteria.required_artifacts {
                        let present = workspace.join(path).is_file();
                        basis.push(format!(
                            "required_artifact_{}:{path}",
                            if present { "present" } else { "missing" }
                        ));
                        accepted &= present;
                    }
                    if let Some(needle) = &claim.criteria.stdout_must_contain {
                        let stdout = fs::read_to_string(run_dir.join(&execution.stdout_path))
                            .unwrap_or_default();
                        let matches = stdout.contains(needle);
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
    SettlementEvent {
        version: RECORD_VERSION.into(),
        settlement_id: "set_01".into(),
        run_id: claim.run_id.clone(),
        status,
        basis,
        review_required,
        settled_at: Utc::now().to_rfc3339(),
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
            criteria: SettlementCriteria {
                require_exit_zero: true,
                required_artifacts: vec![],
                stdout_must_contain: None,
            },
            execution: None,
            authority_verdicts: vec![Verdict::Deny, Verdict::Escalate],
        };
        assert_eq!(
            settle(&claim, Path::new("."), Path::new(".")).status,
            SettlementStatus::Escalated
        );
    }
}
