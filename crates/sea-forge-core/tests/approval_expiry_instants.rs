//! `check_expiry` compares INSTANTS, not text. A pending approval is expired when its `expires_at` is at or
//! before `now`, whatever spelling either stamp uses, and an `expires_at` that does not parse is expired
//! (fail closed).
use sea_forge_core::approvals::{append, check_expiry};
use sea_forge_core::types::{ApprovalRequest, ApprovalStatus};
use sea_forge_core::RECORD_VERSION;

fn pending(id: &str, expires_at: &str) -> ApprovalRequest {
    ApprovalRequest {
        version: RECORD_VERSION.into(),
        approval_id: id.into(),
        run_id: "run-1".into(),
        case_id: "case-1".into(),
        decision_id: "dec-1".into(),
        plan_item_id: "item-1".into(),
        criteria_ref: None,
        criteria_sha256: None,
        criteria_record_hash: None,
        job_contract_ref: None,
        requested_at: "2026-01-01T00:00:00Z".into(),
        expires_at: expires_at.into(),
        status: ApprovalStatus::Pending,
        resolved_by: None,
        resolved_at: None,
        note: None,
    }
}

fn expired(expires_at: &str, now: &str) -> Option<bool> {
    let root = tempfile::tempdir().unwrap();
    append(root.path(), &pending("ap", expires_at)).unwrap();
    check_expiry(root.path(), "case-1", "ap", now).unwrap()
}

#[test]
fn the_same_instant_in_different_spellings_is_expired() {
    // expires_at <= now holds at equality, however each side is written.
    for (expires, now) in [
        ("2026-06-01T00:00:00Z", "2026-06-01T00:00:00+00:00"),
        ("2026-06-01T00:00:00+00:00", "2026-06-01T00:00:00Z"),
        ("2026-06-01T05:30:00+05:30", "2026-06-01T00:00:00Z"),
        ("2026-06-01T00:00:00Z", "2026-06-01T05:30:00+05:30"),
        ("2026-06-01T00:00:00.000Z", "2026-06-01T00:00:00+00:00"),
        (
            "2026-05-31T16:00:00-08:00",
            "2026-06-01T00:00:00.000000000+00:00",
        ),
    ] {
        assert_eq!(expired(expires, now), Some(true), "{expires} vs {now}");
    }
}

#[test]
fn fractions_of_different_lengths_straddling_now_compare_by_instant() {
    // Earlier by a fraction, though the text sorts later ('Z' > digit).
    assert_eq!(
        expired(
            "2026-06-01T00:00:00.123Z",
            "2026-06-01T00:00:00.123456+00:00"
        ),
        Some(true)
    );
    // Later by a fraction.
    assert_eq!(
        expired(
            "2026-06-01T00:00:00.123457Z",
            "2026-06-01T00:00:00.123456+00:00"
        ),
        Some(false)
    );
    assert_eq!(
        expired("2026-06-01T00:00:00.5+00:00", "2026-06-01T00:00:00.25Z"),
        Some(false)
    );
    assert_eq!(
        expired("2026-06-01T00:00:00.25+00:00", "2026-06-01T00:00:00.5Z"),
        Some(true)
    );
    // One nanosecond either side.
    assert_eq!(
        expired(
            "2026-06-01T00:00:00.000000001Z",
            "2026-06-01T00:00:00+00:00"
        ),
        Some(false)
    );
    assert_eq!(
        expired(
            "2026-06-01T00:00:00Z",
            "2026-06-01T00:00:00.000000001+00:00"
        ),
        Some(true)
    );
}

#[test]
fn a_stamp_that_does_not_parse_counts_as_expired() {
    for bad in [
        "zzz",
        "ABC",
        "",
        "2026-06-01",
        "2026-13-01T00:00:00Z",
        "2026-06-01T00:00:00",
        "tomorrow",
    ] {
        assert_eq!(expired(bad, "2026-06-01T00:00:00Z"), Some(true), "{bad:?}");
    }
    // An unparsable `now` cannot prove the window is open either.
    assert_eq!(expired("2099-01-01T00:00:00Z", "not a time"), Some(true));
}

#[test]
fn a_clearly_open_window_stays_open() {
    assert_eq!(
        expired("2099-01-01T00:00:00Z", "2026-06-01T00:00:00+00:00"),
        Some(false)
    );
    assert_eq!(
        expired("2099-01-01T05:30:00+05:30", "2026-06-01T00:00:00Z"),
        Some(false)
    );
}
