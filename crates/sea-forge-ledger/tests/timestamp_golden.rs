//! Golden pins for every timestamp the ledger writes, taken on chrono before the move to the `time` crate.
//! Only public functions are used and no test names a date-time type, so the swap leaves them unedited.
//!
//! Pinned: `committed_at` (entries), `created_at` (stream and global checkpoints) and `signed_at`
//! (witness receipts) are `to_rfc3339()` of a UTC instant, that is `+00:00` with chrono's automatic
//! 0/3/6/9-digit fraction, never `Z`; they are signed, so they must also verify from the bytes on disk;
//! and the ULID prefix is the wall clock in milliseconds.
use sea_forge_ledger::signing::load_or_create_signing_key;
use sea_forge_ledger::types::{ulid, LedgerStream};
use sea_forge_ledger::LedgerManager;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

/// Days from 1970-01-01 (Howard Hinnant's algorithm), so the tests need no time crate.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Nanoseconds since the epoch of exactly `YYYY-MM-DDTHH:MM:SS[.fff|.ffffff|.fffffffff]+00:00`; `None` for
/// any other shape (in particular `Z`, one or two fraction digits, or another offset).
fn epoch_ns_plus_zero(stamp: &str) -> Option<i128> {
    let body = stamp.strip_suffix("+00:00")?;
    let (whole, frac) = match body.split_once('.') {
        None => (body, None),
        Some((w, f)) if matches!(f.len(), 3 | 6 | 9) && f.bytes().all(|b| b.is_ascii_digit()) => {
            (w, Some(f))
        }
        _ => return None,
    };
    let b = whole.as_bytes();
    if whole.len() != 19
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let n = |r: std::ops::Range<usize>| whole.get(r)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, s) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s;
    let frac_ns: i128 = frac.map_or(0, |f| format!("{f:0<9}").parse().unwrap());
    Some(secs as i128 * 1_000_000_000 + frac_ns)
}

fn now_ns() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128
}

fn assert_recent(stamp: &str, before: i128, after: i128) -> i128 {
    let ns = epoch_ns_plus_zero(stamp).unwrap_or_else(|| panic!("not the pinned shape: {stamp}"));
    assert!(
        ns >= before - 1_000_000_000 && ns <= after + 1_000_000_000,
        "{stamp}"
    );
    ns
}

#[test]
fn entry_committed_at_is_plus_zero_with_an_automatic_fraction_and_is_monotone() {
    let tmp = tempdir().unwrap();
    let stream = LedgerStream::open(tmp.path(), "stamps", "writer").unwrap();
    let before = now_ns();
    let mut digits = std::collections::BTreeSet::new();
    for i in 0..300 {
        let committed = stream
            .commit_typed(
                "note",
                vec![format!("s{i}")],
                &serde_json::json!({"i": i}),
                vec![],
            )
            .unwrap();
        assert_eq!(committed.entry_ulid().len(), 26);
    }
    let after = now_ns();
    let entries = stream.read_entries().unwrap();
    let mut previous = 0;
    for e in &entries {
        previous = previous.max(0);
        let ns = assert_recent(&e.committed_at, before, after);
        assert!(
            ns >= previous,
            "committed_at never goes backwards within a stream"
        );
        previous = ns;
        digits.insert(
            e.committed_at
                .split_once('.')
                .map_or(0, |(_, f)| f.len() - "+00:00".len()),
        );
    }
    // Whatever the clock resolution, only 0/3/6/9 fraction digits ever appear.
    assert!(
        digits.iter().all(|d| matches!(d, 0 | 3 | 6 | 9)),
        "{digits:?}"
    );
    // The stored line is the same text the reader returned (no re-rendering on read).
    let raw = std::fs::read_to_string(stream.entries_path()).unwrap();
    for e in &entries {
        assert!(
            raw.contains(&format!("\"committed_at\":\"{}\"", e.committed_at)),
            "{}",
            e.committed_at
        );
    }
}

#[test]
fn checkpoint_created_at_has_the_pinned_shape_and_still_verifies_from_disk() {
    let tmp = tempdir().unwrap();
    let key_dir = tmp.path().join("keys");
    let key = load_or_create_signing_key(&key_dir, "k").unwrap();
    let stream = LedgerStream::open(tmp.path(), "cp", "writer").unwrap();
    stream
        .commit_typed(
            "note",
            vec!["a".into()],
            &serde_json::json!({"a": 1}),
            vec![],
        )
        .unwrap();
    let before = now_ns();
    let cp = stream.create_checkpoint(&key, "k").unwrap();
    let after = now_ns();
    assert_recent(&cp.created_at, before, after);
    // `created_at` is inside the signed bytes: the signature verifies from the text on disk alone.
    stream.verify_checkpoints(&key.verifying_key()).unwrap();
    let raw = std::fs::read_to_string(stream.checkpoints_path()).unwrap();
    assert!(raw.contains(&format!("\"created_at\":\"{}\"", cp.created_at)));
    // A different spelling of the same instant is a different signed message.
    let rewritten = raw.replace(&cp.created_at, &cp.created_at.replace("+00:00", "Z"));
    std::fs::write(stream.checkpoints_path(), rewritten).unwrap();
    assert!(stream.verify_checkpoints(&key.verifying_key()).is_err());
}

#[test]
fn global_checkpoint_and_witness_receipt_stamps_have_the_pinned_shape_and_verify() {
    let tmp = tempdir().unwrap();
    let key_dir = tmp.path().join("keys");
    let mgr = LedgerManager::new(tmp.path()).unwrap();
    let stream = mgr.open_stream("case_a", "operator_01").unwrap();
    stream
        .commit_typed(
            "note",
            vec!["a".into()],
            &serde_json::json!({"a": 1}),
            vec![],
        )
        .unwrap();
    let key = load_or_create_signing_key(&key_dir, "integrity_01").unwrap();
    let witness = load_or_create_signing_key(&key_dir, "witness_alpha").unwrap();

    let before = now_ns();
    let gcp = mgr.create_global_checkpoint(&key, "integrity_01").unwrap();
    let receipt = mgr
        .witness_global_checkpoint(&gcp, "witness_alpha", &witness)
        .unwrap();
    let after = now_ns();
    assert_recent(&gcp.created_at, before, after);
    assert_recent(&receipt.signed_at, before, after);
    mgr.verify_global_checkpoints(&key.verifying_key()).unwrap();
    mgr.verify_witness_receipts(
        &gcp.global_checkpoint_hash,
        &[("witness_alpha".into(), witness.verifying_key())],
        "operator_01",
        1,
    )
    .unwrap();
}

#[test]
fn ulid_prefix_is_the_wall_clock_in_milliseconds() {
    const ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let before_ms = (now_ns() / 1_000_000) as u64;
    let id = ulid().unwrap();
    let after_ms = (now_ns() / 1_000_000) as u64;
    assert_eq!(id.len(), 26);
    assert!(id.chars().all(|c| ALPHABET.contains(c)), "{id}");
    let ms = id[..10]
        .chars()
        .fold(0_u64, |acc, c| acc * 32 + ALPHABET.find(c).unwrap() as u64);
    assert!(
        ms >= before_ms.saturating_sub(5) && ms <= after_ms + 5,
        "{id}: {ms} vs {before_ms}..{after_ms}"
    );
    // Within a millisecond the ids stay strictly increasing.
    let mut last = ulid().unwrap();
    for _ in 0..200 {
        let next = ulid().unwrap();
        assert!(next > last, "{last} !< {next}");
        last = next;
    }
}
