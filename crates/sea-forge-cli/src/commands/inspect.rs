use sea_forge_core::{errors::ForgeError, types::AuthorityAction};
use std::{fs, path::Path};

fn find_run_dir(root: &Path, run_id: &str) -> Result<std::path::PathBuf, ForgeError> {
    let legacy = root.join("runs").join(run_id);
    if legacy.is_dir() {
        return Ok(legacy);
    }
    for case_id in sea_forge_case_runner::case_ops::case_ids(root)? {
        let case_dir = sea_forge_case_runner::case_ops::case_snapshot_dir(root, &case_id)?;
        let candidate = case_dir.join("runs").join(run_id);
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Err(ForgeError::Input(format!("run {run_id} not found")))
}

pub fn execute(
    root: &Path,
    policy: Option<&Path>,
    actor_id: &str,
    run_id: &str,
) -> Result<u8, ForgeError> {
    if !sea_forge_core::ids::valid_run_id(run_id) {
        return Err(ForgeError::Input("invalid run_id".into()));
    }
    let policy = super::mediated::policy_path(root, policy);
    super::mediated::authorize_read(
        root,
        &policy,
        actor_id,
        &AuthorityAction::Reserved {
            resource_type: "inspect_run".into(),
            resource_id: run_id.into(),
            parameters: serde_json::json!({}),
        },
    )?;
    let _verified_ledger = super::mediated::assurance(root, &policy, actor_id)?;
    println!("== assurance ==\nlegacy_digest_only");
    let run = find_run_dir(root, run_id)?;
    for name in [
        "plan.json",
        "authority.json",
        "trace.jsonl",
        "evidence.jsonl",
        "settlement.json",
        "semantic-envelope.json",
    ] {
        let path = run.join(name);
        let content = fs::read_to_string(&path)
            .map_err(|e| ForgeError::io(format!("read {}", path.display()), e))?;
        println!("== {name} ==\n{content}");
    }
    Ok(0)
}
