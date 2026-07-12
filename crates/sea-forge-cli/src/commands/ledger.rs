use crate::LedgerAction;
use sea_forge_core::errors::ForgeError;
use sea_forge_ledger::types::LedgerStream;
use std::path::Path;

pub fn execute(action: LedgerAction, root: &Path) -> Result<u8, ForgeError> {
    match action {
        LedgerAction::Verify { ledger_id } => {
            let stream = LedgerStream::open(root, &ledger_id, "cli_verify")?;
            match stream.verify() {
                Ok(()) => {
                    println!("ledger {ledger_id}: verified");
                    Ok(0)
                }
                Err(e) => {
                    println!("ledger {ledger_id}: VERIFICATION FAILED: {e}");
                    Ok(1)
                }
            }
        }
        LedgerAction::Prove {
            ledger_id,
            entry_ulid,
        } => {
            let stream = LedgerStream::open(root, &ledger_id, "cli_prove")?;
            let proof = stream.prove_entry(&entry_ulid)?;
            let mmr = stream.load_mmr()?;
            let entries = stream.read_entries()?;
            let entry = entries
                .iter()
                .find(|e| e.entry_ulid == entry_ulid)
                .ok_or_else(|| ForgeError::Input(format!("entry {entry_ulid} not found")))?;
            let proof_json = serde_json::json!({
                "entry_ulid": entry_ulid,
                "entry_hash": entry.entry_hash,
                "leaf_index": proof.leaf_index,
                "proof_hashes": proof.proof_hashes,
                "mmr_peaks": mmr.peaks,
            });
            println!("{}", serde_json::to_string_pretty(&proof_json)?);
            Ok(0)
        }
    }
}
