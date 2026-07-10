use sea_forge_core::{run_intent, types::SettlementStatus, RunOptions};
use std::path::PathBuf;
pub fn execute(
    intent: String,
    policy: PathBuf,
    root: PathBuf,
    timeout_secs: u64,
    entity: String,
    process: String,
) -> Result<u8, sea_forge_core::ForgeError> {
    let outcome = run_intent(RunOptions {
        intent,
        policy,
        root,
        timeout_secs,
        entity,
        process,
        executable: Some(
            std::env::current_exe()
                .map_err(|e| sea_forge_core::ForgeError::io("resolve executable", e))?,
        ),
    })?;
    println!("run_id={}", outcome.run_id);
    for d in &outcome.decisions {
        println!(
            "authority={}:{}",
            d.decision_id,
            serde_json::to_value(&d.verdict)?
                .as_str()
                .expect("verdict serializes as a string")
        );
    }
    println!(
        "execution={}",
        outcome
            .execution
            .as_ref()
            .map_or("not_run".into(), |e| format!("{:?}", e.status)
                .to_lowercase())
    );
    println!(
        "settlement={}",
        serde_json::to_value(&outcome.settlement.status)?
            .as_str()
            .expect("settlement status serializes as a string")
    );
    println!("run_dir={}", outcome.run_dir.display());
    Ok(match outcome.settlement.status {
        SettlementStatus::Accepted => 0,
        SettlementStatus::Rejected => 3,
        SettlementStatus::Escalated => 4,
    })
}
