//! Golden pin for trace event timestamps: chrono `to_rfc3339()` of a UTC instant, `+00:00` with an
//! automatic 0/3/6/9-digit fraction, serialized as the `timestamp` field of each JSONL line.
use sea_forge_core::types::TraceKind;
use sea_forge_trace::{append_internal_error, JsonlTraceRecorder};
use std::time::{SystemTime, UNIX_EPOCH};

fn civil_days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

fn plus_zero_seconds(stamp: &str) -> i64 {
    let body = stamp
        .strip_suffix("+00:00")
        .unwrap_or_else(|| panic!("{stamp}"));
    let (whole, frac) = body.split_once('.').map_or((body, ""), |(w, f)| (w, f));
    assert!(
        matches!(frac.len(), 0 | 3 | 6 | 9) && frac.bytes().all(|b| b.is_ascii_digit()),
        "{stamp}"
    );
    assert_eq!(whole.len(), 19, "{stamp}");
    let n = |r: std::ops::Range<usize>| whole[r].parse::<i64>().unwrap();
    civil_days(n(0..4), n(5..7), n(8..10)) * 86_400 + n(11..13) * 3600 + n(14..16) * 60 + n(17..19)
}

#[test]
fn trace_event_timestamps_are_plus_zero_with_an_automatic_fraction() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace.jsonl");
    let mut recorder = JsonlTraceRecorder::create(&path, "run_x", "actor").unwrap();
    recorder
        .append(TraceKind::InternalError, None, serde_json::json!({"n": 1}))
        .unwrap();
    drop(recorder);
    append_internal_error(&path, "run_x", "actor", "boom");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let text = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(!lines.is_empty(), "{text}");
    for line in &lines {
        let stamp = line["timestamp"].as_str().unwrap();
        assert!((plus_zero_seconds(stamp) - now).abs() <= 5, "{stamp}");
    }
}
