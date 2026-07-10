use sea_forge_core::errors::ForgeError;
use std::{fs, path::Path};
pub fn execute(root: &Path, run_id: &str) -> Result<u8, ForgeError> {
    let run = root.join("runs").join(run_id);
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
