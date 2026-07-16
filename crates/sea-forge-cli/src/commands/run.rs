use crate::pipeline::{run_intent, RunOptions};
use sea_forge_core::types::{SettlementStatus, Verdict};
use std::path::PathBuf;
pub fn execute(
    intent: Option<String>,
    plan: Option<PathBuf>,
    policy: PathBuf,
    root: PathBuf,
    timeout_secs: u64,
    entity: String,
    process: String,
) -> Result<u8, sea_forge_core::ForgeError> {
    if let Some(plan) = plan {
        let outcome = crate::plan_pipeline::run_plan(crate::plan_pipeline::PlanRunOptions {
            plan,
            policy,
            root,
            timeout_secs,
            entity,
            process,
            intent_summary: None,
            origin_evidence_refs: vec![],
        })?;
        println!("case_id={}", outcome.case_id);
        println!("case_state={}", outcome.state);
        return Ok(outcome.exit_code);
    }
    let intent = intent
        .ok_or_else(|| sea_forge_core::ForgeError::Input("intent or plan is required".into()))?;
    let outcome = run_intent(RunOptions {
        intent,
        policy,
        root,
        timeout_secs,
        entity,
        process,
    })?;
    println!("run_id={}", outcome.run_id);
    for d in &outcome.decisions {
        println!("authority={}:{}", d.decision_id, verdict_label(&d.verdict));
    }
    println!(
        "execution={}",
        outcome
            .execution
            .as_ref()
            .map_or("not_run", |execution| execution_label(&execution.status))
    );
    println!(
        "settlement={}",
        settlement_label(&outcome.settlement.status)
    );
    println!("run_dir={}", outcome.run_dir.display());
    Ok(match outcome.settlement.status {
        SettlementStatus::Accepted => 0,
        SettlementStatus::Rejected => 3,
        SettlementStatus::Escalated => 4,
    })
}

fn verdict_label(verdict: &Verdict) -> &'static str {
    match verdict {
        Verdict::Allow => "allow",
        Verdict::Deny => "deny",
        Verdict::Escalate => "escalate",
    }
}

fn settlement_label(status: &SettlementStatus) -> &'static str {
    match status {
        SettlementStatus::Accepted => "accepted",
        SettlementStatus::Rejected => "rejected",
        SettlementStatus::Escalated => "escalated",
    }
}

fn execution_label(status: &sea_forge_core::types::ExecutionStatus) -> &'static str {
    use sea_forge_core::types::ExecutionStatus;
    match status {
        ExecutionStatus::Completed => "completed",
        ExecutionStatus::SpawnFailed => "spawn_failed",
        ExecutionStatus::TimedOut => "timed_out",
        ExecutionStatus::SandboxViolation => "sandbox_violation",
    }
}
