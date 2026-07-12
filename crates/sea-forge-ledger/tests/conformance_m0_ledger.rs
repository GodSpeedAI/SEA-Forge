use sea_forge_ledger::signing::{load_or_create_signing_key, SigningKey};
use sea_forge_ledger::types::{verify_proof, LedgerStream, MerkleProof, MmrState};
use sha2::Digest;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

fn append_mixed_records(
    stream: &LedgerStream,
    count: usize,
) -> Vec<sea_forge_ledger::types::LedgerEntry> {
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let kind = if i % 3 == 0 {
            "case_event"
        } else if i % 3 == 1 {
            "authority_decision"
        } else {
            "trace_event"
        };
        let payload = serde_json::json!({
            "index": i,
            "note": "mixed record",
            "nested": { "value": i % 7 }
        });
        let entry = stream
            .append(
                kind,
                vec![format!("subject_{i}")],
                payload,
                vec![format!("auth_{i}")],
            )
            .unwrap();
        entries.push(entry);
    }
    entries
}

fn prove_and_verify(
    stream: &LedgerStream,
    entry: &sea_forge_ledger::types::LedgerEntry,
) -> (MerkleProof, String) {
    let proof = stream.prove_entry(&entry.entry_ulid).unwrap();
    let mmr = stream.load_mmr().unwrap();

    // Find the peak containing the entry and verify the proof against it.
    let mut start = 0u64;
    let mut peak_root = None;
    let sizes = peak_sizes(mmr.leaf_count);
    for size in &sizes {
        if entry.mmr_leaf_index >= start && entry.mmr_leaf_index < start + size {
            let leaves: Vec<String> = stream
                .read_entries()
                .unwrap()
                .into_iter()
                .skip(start as usize)
                .take(*size as usize)
                .map(|e| e.entry_hash)
                .collect();
            let tree = build_tree(&leaves);
            peak_root = tree.last().and_then(|level| level.first()).cloned();
            break;
        }
        start += size;
    }
    let peak_root = peak_root.expect("peak containing entry");
    assert!(verify_proof(&entry.entry_hash, &peak_root, &proof));
    assert!(mmr.peaks.contains(&peak_root));
    (proof, peak_root)
}

fn peak_sizes(leaf_count: u64) -> Vec<u64> {
    let mut sizes = Vec::new();
    let mut remaining = leaf_count;
    let mut size = 1u64 << (63 - remaining.leading_zeros());
    while remaining > 0 {
        if size <= remaining {
            sizes.push(size);
            remaining -= size;
        }
        size >>= 1;
    }
    sizes
}

fn build_tree(leaves: &[String]) -> Vec<Vec<String>> {
    let mut levels = vec![leaves.to_vec()];
    while levels.last().unwrap().len() > 1 {
        let level = levels.last().unwrap();
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks(2) {
            let mut hasher = sha2::Sha256::new();
            hasher.update(b"sea-forge/mmr-node/v1\0");
            hasher.update(pair[0].as_bytes());
            hasher.update(pair[1].as_bytes());
            next.push(format!("sha256:{:x}", hasher.finalize()));
        }
        levels.push(next);
    }
    levels
}

#[test]
fn m0_append_1000_mixed_records_and_verify_integrity() {
    let tmp = tempdir().unwrap();

    let case_a = LedgerStream::open(tmp.path(), "case_a", "writer_01").unwrap();
    let case_b = LedgerStream::open(tmp.path(), "case_b", "writer_02").unwrap();
    let global = LedgerStream::open(tmp.path(), "global", "writer_03").unwrap();

    let entries_a = append_mixed_records(&case_a, 500);
    let entries_b = append_mixed_records(&case_b, 300);
    let entries_g = append_mixed_records(&global, 200);

    case_a.verify().unwrap();
    case_b.verify().unwrap();
    global.verify().unwrap();

    for (name, entries) in [
        ("case_a", &entries_a),
        ("case_b", &entries_b),
        ("global", &entries_g),
    ] {
        let mut ulids = HashSet::new();
        let mut prev_hash: Option<String> = None;
        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(entry.append_ordinal, i as u64, "{name} ordinal gap at {i}");
            assert_eq!(
                entry.previous_entry_hash.as_ref(),
                prev_hash.as_ref(),
                "{name} predecessor link at {i}"
            );
            assert!(
                ulids.insert(entry.entry_ulid.clone()),
                "{name} duplicate ULID: {}",
                entry.entry_ulid
            );
            assert_eq!(entry.entry_ulid.len(), 26, "{name} ULID length at {i}");
            prev_hash = Some(entry.entry_hash.clone());
        }
    }

    // Inclusion proofs for first, middle, and last entries across streams.
    let first = &entries_a[0];
    let middle = &entries_a[entries_a.len() / 2];
    let last = entries_a.last().unwrap();
    prove_and_verify(&case_a, first);
    prove_and_verify(&case_a, middle);
    prove_and_verify(&case_a, last);
}

#[test]
fn m0_alter_one_byte_fails_verification() {
    let tmp = tempdir().unwrap();
    let stream = LedgerStream::open(tmp.path(), "case_tamper", "writer_01").unwrap();
    stream
        .append(
            "case_event",
            vec!["case_abc".into()],
            serde_json::json!({"state": "active"}),
            vec![],
        )
        .unwrap();
    stream
        .append(
            "case_event",
            vec!["case_abc".into()],
            serde_json::json!({"state": "pending"}),
            vec![],
        )
        .unwrap();

    let path = stream.entries_path();
    let mut content = fs::read_to_string(&path).unwrap();
    content = content.replace("pending", "PENDING");
    fs::write(&path, content).unwrap();

    let result = stream.verify();
    assert!(
        result.is_err(),
        "expected verification failure after one-byte-equivalent alteration"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("ledger_integrity_error"),
        "expected typed integrity error: {err}"
    );
}

#[test]
fn m0_create_and_verify_signed_checkpoint() {
    let tmp = tempdir().unwrap();
    let key_dir = tmp.path().join("keys");
    let stream = LedgerStream::open(tmp.path(), "case_checkpoint", "writer_01").unwrap();

    append_mixed_records(&stream, 10);

    let signing_key = load_or_create_signing_key(&key_dir, "integrity_01").unwrap();
    let checkpoint = stream
        .create_checkpoint(&signing_key, "integrity_01")
        .unwrap();

    assert_eq!(checkpoint.ledger_id, "case_checkpoint");
    assert_eq!(checkpoint.first_ordinal, 0);
    assert_eq!(checkpoint.last_ordinal, 9);
    assert_eq!(checkpoint.entry_count, 10);
    assert!(checkpoint.mmr_root.starts_with("sha256:"));
    assert_eq!(checkpoint.signing_algorithm, "ed25519");
    assert_eq!(checkpoint.signing_key_id, "integrity_01");
    assert!(checkpoint.signature.starts_with("ed25519:"));
    assert!(checkpoint.checkpoint_hash.starts_with("sha256:"));

    let verifying_key = signing_key.verifying_key();
    stream.verify_checkpoints(&verifying_key).unwrap();
}

#[test]
fn m0_checkpoint_signature_verifies_under_public_key() {
    let tmp = tempdir().unwrap();
    let key_dir = tmp.path().join("keys");
    let stream = LedgerStream::open(tmp.path(), "case_sign", "writer_01").unwrap();

    append_mixed_records(&stream, 5);

    let signing_key = load_or_create_signing_key(&key_dir, "integrity_01").unwrap();
    let checkpoint = stream
        .create_checkpoint(&signing_key, "integrity_01")
        .unwrap();

    let wrong_key = SigningKey::generate(&mut rand::thread_rng());
    let result = stream.verify_checkpoints(&wrong_key.verifying_key());
    assert!(
        result.is_err(),
        "checkpoint must not verify with unrelated key"
    );

    let verifying_key = signing_key.verifying_key();
    stream.verify_checkpoints(&verifying_key).unwrap();

    // Corrupting the checkpoint hash breaks verification.
    let mut corrupted = checkpoint.clone();
    corrupted.checkpoint_hash = corrupted.checkpoint_hash.replace("a", "b");
    let path = stream.checkpoints_path();
    let mut content = String::new();
    content.push_str(&serde_json::to_string(&corrupted).unwrap());
    content.push('\n');
    fs::write(&path, content).unwrap();
    let result = stream.verify_checkpoints(&verifying_key);
    assert!(result.is_err(), "checkpoint with corrupted hash must fail");
}

#[test]
fn m0_truncate_middle_entry_fails_verification() {
    let tmp = tempdir().unwrap();
    let stream = LedgerStream::open(tmp.path(), "case_truncate", "writer_01").unwrap();
    let entries = append_mixed_records(&stream, 10);

    let path = stream.entries_path();
    let lines: Vec<String> = fs::read_to_string(&path)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    // Remove the middle entry (index 5) from the file.
    let mut truncated = lines.clone();
    truncated.remove(5);
    fs::write(&path, truncated.join("\n")).unwrap();

    let result = stream.verify();
    assert!(result.is_err(), "truncation should break verification");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("ledger_integrity_error"), "typed error: {err}");

    // Restore and verify reordering fails.
    fs::write(&path, lines.join("\n")).unwrap();
    stream.verify().unwrap();

    let mut reordered = lines.clone();
    reordered.swap(4, 6);
    fs::write(&path, reordered.join("\n")).unwrap();
    let result = stream.verify();
    assert!(result.is_err(), "reordering should break verification");

    // Restore and verify duplication fails.
    fs::write(&path, lines.join("\n")).unwrap();
    stream.verify().unwrap();

    let entry5 = &entries[5];
    let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
    let mut bytes = serde_json::to_vec(entry5).unwrap();
    bytes.push(b'\n');
    file.write_all(&bytes).unwrap();
    drop(file);
    let result = stream.verify();
    assert!(result.is_err(), "duplication should break verification");
}

#[test]
fn m0_global_stream_checkpoint_chain() {
    let tmp = tempdir().unwrap();
    let key_dir = tmp.path().join("keys");
    let global = LedgerStream::open(tmp.path(), "global", "writer_01").unwrap();

    append_mixed_records(&global, 50);
    let signing_key = load_or_create_signing_key(&key_dir, "integrity_global").unwrap();
    let cp1 = global
        .create_checkpoint(&signing_key, "integrity_global")
        .unwrap();

    append_mixed_records(&global, 50);
    let cp2 = global
        .create_checkpoint(&signing_key, "integrity_global")
        .unwrap();

    assert_eq!(cp2.first_ordinal, cp1.last_ordinal + 1);
    assert_eq!(cp2.previous_checkpoint_hash, Some(cp1.checkpoint_hash));

    global
        .verify_checkpoints(&signing_key.verifying_key())
        .unwrap();
}

#[test]
fn m0_mmr_root_matches_recomputed_root() {
    let tmp = tempdir().unwrap();
    let stream = LedgerStream::open(tmp.path(), "case_root", "writer_01").unwrap();
    let entries = append_mixed_records(&stream, 100);

    let mut mmr = MmrState::empty();
    for entry in &entries {
        mmr = mmr.append(&entry.entry_hash).unwrap();
    }
    let stored = stream.load_mmr().unwrap();
    assert_eq!(stored, mmr);
    assert_eq!(stored.root().unwrap(), mmr.root().unwrap());
}
