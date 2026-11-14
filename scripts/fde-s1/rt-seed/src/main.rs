//! usage: rt-seed <ledger.db> <repo_id> <record_id>...
//! Appends one `evidence.recorded` row per record id (the same rows the
//! RealityTrace CLI test `sxr-cli/tests/compare.rs::setup` appends).
use serde_json::json;
use sxr_ledger::ledger::{Ledger, LedgerConfig, PendingEntry};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut l = Ledger::open(std::path::Path::new(&a[1]), &a[2], LedgerConfig::default()).unwrap();
    for r in &a[3..] {
        l.append(&PendingEntry {
            record_kind: "evidence.recorded".into(),
            idempotency_key: Some(format!("ev-{r}")),
            subject_refs: vec![],
            payload: json!({"evidence": [{"evidence_id": format!("evidence:recorded:{r}")}]}).to_string(),
            entry_ulid: None,
            committed_at: None,
        })
        .unwrap();
    }
}
