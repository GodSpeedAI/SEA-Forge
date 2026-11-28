//! Golden pin for the timestamp embedded in generated identifiers (chrono `%Y%m%dT%H%M%SZ`, UTC, whole
//! seconds, literal Z). Public functions only; no date-time type is named.
use sea_forge_core::ids::{case_id, run_id, snapshot_id};
use std::time::{SystemTime, UNIX_EPOCH};

fn civil_days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

/// Epoch seconds of the `YYYYMMDDTHHMMSSZ` segment of `<prefix>_<segment>_<hex>`.
fn embedded_seconds(id: &str, prefix: &str) -> i64 {
    let rest = id
        .strip_prefix(&format!("{prefix}_"))
        .unwrap_or_else(|| panic!("{id}"));
    let (stamp, hex) = rest.split_once('_').unwrap_or_else(|| panic!("{id}"));
    assert_eq!(stamp.len(), 16, "{id}");
    assert!(stamp.ends_with('Z') && stamp.as_bytes()[8] == b'T', "{id}");
    assert!(
        !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit()),
        "{id}"
    );
    let n = |r: std::ops::Range<usize>| stamp[r].parse::<i64>().unwrap();
    civil_days(n(0..4), n(4..6), n(6..8)) * 86_400 + n(9..11) * 3600 + n(11..13) * 60 + n(13..15)
}

#[test]
fn generated_ids_embed_the_utc_second_with_a_literal_z() {
    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    };
    for (id, prefix) in [
        (run_id().unwrap(), "run"),
        (case_id().unwrap(), "case"),
        (snapshot_id().unwrap(), "smsnap"),
    ] {
        let before = now();
        let seconds = embedded_seconds(&id, prefix);
        assert!((seconds - before).abs() <= 2, "{id}");
    }
    assert!(sea_forge_core::ids::valid_run_id(&run_id().unwrap()));
}
