use crate::{
    errors::ForgeError,
    types::{SemanticEnvelope, SettlementStatus},
};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};
pub fn append(path: &Path, envelope: &SemanticEnvelope) -> Result<(), ForgeError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    serde_json::to_writer(&mut file, envelope)?;
    file.write_all(b"\n")
        .and_then(|_| file.flush())
        .map_err(|e| ForgeError::io("append capability envelope", e))
}
pub struct RecallQuery<'a> {
    pub query: &'a str,
    pub entity: Option<&'a str>,
    pub process: Option<&'a str>,
    pub result: Option<SettlementStatus>,
    pub limit: usize,
}
pub fn recall(
    path: &Path,
    query: RecallQuery<'_>,
) -> Result<(Vec<SemanticEnvelope>, usize), ForgeError> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], 0)),
        Err(e) => return Err(ForgeError::io("open capability memory", e)),
    };
    let mut malformed = 0;
    let mut values: Vec<SemanticEnvelope> = Vec::new();
    for line in BufReader::new(file).lines() {
        match line
            .map_err(|e| ForgeError::io("read capability memory", e))
            .and_then(|l| serde_json::from_str(&l).map_err(ForgeError::from))
        {
            Ok(e) => values.push(e),
            Err(_) => malformed += 1,
        }
    }
    let needle = query.query.to_lowercase();
    values.reverse();
    values.retain(|e| {
        (e.intent.summary.to_lowercase().contains(&needle)
            || e.capability_delta
                .attempted_capability
                .to_lowercase()
                .contains(&needle))
            && query.entity.is_none_or(|v| e.attribution.entity_id == v)
            && query.process.is_none_or(|v| e.attribution.process_id == v)
            && query
                .result
                .as_ref()
                .is_none_or(|v| &e.capability_delta.result == v)
    });
    values.truncate(query.limit);
    Ok((values, malformed))
}
