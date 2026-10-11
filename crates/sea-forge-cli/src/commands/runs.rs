use sea_forge_core::{
    errors::ForgeError,
    types::{Case, SettlementEvent},
};
use std::{fs, path::Path};

pub fn list(root: &Path, unsettled: bool) -> Result<u8, ForgeError> {
    let cases_dir = root.join("cases");
    if !cases_dir.exists() {
        return Ok(0);
    }

    let mut cases = Vec::new();
    for entry in fs::read_dir(&cases_dir).map_err(|error| ForgeError::io("read cases", error))? {
        let path = entry
            .map_err(|error| ForgeError::io("read case entry", error))?
            .path()
            .join("case.json");
        if !path.is_file() {
            continue;
        }
        let case: Case = serde_json::from_slice(
            &fs::read(&path).map_err(|error| ForgeError::io("read case.json", error))?,
        )
        .map_err(|error| ForgeError::Serialization(format!("parse {}: {error}", path.display())))?;
        for run_id in &case.run_ids {
            let settlement_path = cases_dir
                .join(&case.case_id)
                .join("runs")
                .join(run_id)
                .join("settlement.json");
            let settled = fs::read(&settlement_path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<SettlementEvent>(&bytes).ok())
                .is_some_and(|settlement| settlement.run_id == *run_id);
            if !unsettled || !settled {
                cases.push((run_id.clone(), case.case_id.clone(), case.state.clone()));
            }
        }
    }
    cases.sort_by(|left, right| (&left.0, &left.1).cmp(&(&right.0, &right.1)));
    for (run_id, case_id, state) in cases {
        println!(
            "{}\t{}\t{}",
            run_id,
            case_id,
            serde_json::to_string(&state)
                .unwrap_or_default()
                .trim_matches('"')
        );
    }
    Ok(0)
}
