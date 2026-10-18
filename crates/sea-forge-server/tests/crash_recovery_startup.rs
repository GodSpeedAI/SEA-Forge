//! A server killed between an entry write and its derived `mmr.json` must
//! start again (F-04): startup rebuilds the cache from the entries.
use sea_forge_ledger::types::LedgerStream;
use sea_forge_server::{ServerConfig, ServerState};
use serde_json::json;
use std::path::Path;

fn start(root: &Path) -> Result<ServerState, sea_forge_core::errors::ForgeError> {
    ServerState::new(ServerConfig {
        identity: sea_forge_server::identity::IdentityBindings::local_operator("operator_local"),
        socket_path: root.join("unused.sock"),
        root: root.to_path_buf(),
        max_concurrent_runs: 4,
        ..ServerConfig::default()
    })
}

#[test]
fn startup_repairs_a_ledger_killed_between_entry_and_mmr() {
    let root = tempfile::tempdir().unwrap();
    let ledger = LedgerStream::open(root.path(), "case-crash", "sea-forge-server").unwrap();
    ledger
        .append("note", vec![], json!({"n": 1}), vec![])
        .unwrap();
    let mmr = root.path().join("ledgers/case-crash/mmr.json");
    let before_second = std::fs::read(&mmr).unwrap();
    ledger
        .append("note", vec![], json!({"n": 2}), vec![])
        .unwrap();
    // The kill window: the second entry is durable, its mmr update is not.
    std::fs::write(&mmr, before_second).unwrap();
    assert!(
        ledger.verify().is_err(),
        "the crash state must be detectable"
    );

    start(root.path()).expect("a crashed cell must start again");

    ledger.verify().unwrap();
    assert_eq!(ledger.read_entries().unwrap().len(), 2, "no entry is lost");
}

#[test]
fn startup_quarantines_a_torn_tail_and_keeps_the_valid_entries() {
    let root = tempfile::tempdir().unwrap();
    let ledger = LedgerStream::open(root.path(), "case-torn", "sea-forge-server").unwrap();
    ledger
        .append("note", vec![], json!({"n": 1}), vec![])
        .unwrap();
    let entries = root.path().join("ledgers/case-torn/entries.jsonl");
    let mut torn = std::fs::read_to_string(&entries).unwrap();
    torn.push_str("{\"entry_ulid\":\"half");
    std::fs::write(&entries, torn).unwrap();

    start(root.path()).expect("a torn tail must not brick the cell");

    ledger.verify().unwrap();
    assert_eq!(ledger.read_entries().unwrap().len(), 1);
    let quarantine = root.path().join("ledgers/quarantine");
    assert_eq!(std::fs::read_dir(quarantine).unwrap().count(), 1);
}
