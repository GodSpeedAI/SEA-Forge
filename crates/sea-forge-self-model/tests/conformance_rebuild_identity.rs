//! SUP-09f: self-model rebuild identity is pinned to realization content.
//! A rebuild whose realization bytes differ (different build environment via
//! SOURCE_DATE_EPOCH) must commit cleanly instead of dying on an
//! idempotency-key payload conflict; two identical rebuilds must dedupe to
//! exactly one committed release-realization record.

use sea_forge_ledger::LedgerStream;
use sea_forge_self_model::{
    store::{self, RebuildInputs},
    ExtensionState,
};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Serializes both tests below. SOURCE_DATE_EPOCH is process-global and each
/// rebuild observes it indirectly through `release_realization()`, so tests in
/// this binary running on parallel threads could otherwise flip the variable
/// mid-test and corrupt either test's expectations. Other test binaries are
/// separate processes and cannot race this one; the lock is therefore a
/// sufficient mitigation even though it only covers this file.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn inputs() -> RebuildInputs<'static> {
    RebuildInputs {
        cell_id: "cell_test",
        active_extensions: vec![ExtensionState {
            descriptor_ref: "ext_test".into(),
            status: "active".into(),
        }],
        environments_present: vec!["demo_env@0.1.0".into()],
        probes: vec![],
        sandbox_classes_available: vec!["local".into()],
        created_at: "2026-08-15T00:00:00Z",
        capability_projection_sha256: "sha256:test",
        actor_id: "operator_local",
    }
}

fn temp_root(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sea-forge-rebuild-id-{name}-{nonce}"));
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// Open the same self-model ledger stream `store::rebuild` commits into.
fn open_self_model_stream(root: &Path) -> LedgerStream {
    LedgerStream::open(root, "self-model", "sup_09f_audit").unwrap()
}

/// Number of committed release-realization records in the self-model stream.
fn release_record_count(root: &Path) -> usize {
    open_self_model_stream(root)
        .read_entries()
        .unwrap()
        .iter()
        .filter(|entry| entry.record_kind == "self_model_release_realization")
        .count()
}

/// THE SUP-09f regression: the first rebuild runs under SOURCE_DATE_EPOCH and
/// the second without it, so the realization bytes legitimately differ
/// (`generated_at`). Keyed on the constant release id the second rebuild hit
/// the idempotency-key payload conflict forever; keyed on content hash it
/// commits cleanly beside the first record.
#[test]
fn rebuild_with_a_different_build_epoch_commits_cleanly() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = temp_root("epoch-change");

    // Minimal window: set immediately before, remove immediately after — the
    // variable is process-global for everything calling release_realization().
    std::env::set_var("SOURCE_DATE_EPOCH", "1700000000");
    let first = store::rebuild(&root, &inputs()).unwrap();
    std::env::remove_var("SOURCE_DATE_EPOCH");

    let second = store::rebuild(&root, &inputs()).unwrap_or_else(|e| {
        panic!("rebuild under a different build epoch must commit cleanly, got: {e:?}")
    });

    assert_ne!(
        first.snapshot_id, second.snapshot_id,
        "each rebuild mints its own immutable snapshot"
    );
    // Differing content ⇒ differing key ⇒ both commits exist side by side.
    assert_eq!(
        release_record_count(&root),
        2,
        "two distinct realizations must produce two committed release records"
    );
}

/// Identical rebuilds (same environment, byte-identical realization) share the
/// same content-derived idempotency key and bytes, so the second commit returns
/// the existing record instead of appending.
#[test]
fn identical_rebuilds_dedupe_to_one_release_record() {
    let _guard = ENV_LOCK.lock().unwrap();
    let root = temp_root("dedupe");

    // Keep the window clean: ensure no inherited SOURCE_DATE_EPOCH leaks in so
    // both rebuilds see identical realization bytes.
    std::env::remove_var("SOURCE_DATE_EPOCH");
    store::rebuild(&root, &inputs()).unwrap();
    store::rebuild(&root, &inputs()).unwrap();

    assert_eq!(
        release_record_count(&root),
        1,
        "identical realizations must dedupe to one committed release record"
    );
}
