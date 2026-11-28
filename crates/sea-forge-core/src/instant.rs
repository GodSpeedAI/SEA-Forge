//! Comparing RFC 3339 stamps by INSTANT, never by text.
//!
//! Two spellings of one instant (`Z` vs `+00:00`, `.5` vs `.500`, `+05:30` vs `Z`) differ as text, and
//! text order is not time order across spellings. Every decision that turns on "is this stamp at or before
//! that one" goes through here. A stamp that does not parse proves nothing, so each helper names which way it
//! fails; for expiry and ordering of authority the answer is always closed (expired, refused).
use chrono::{DateTime, Utc};

/// The instant a stamp names, or `None` when it is not RFC 3339.
pub fn parse(stamp: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(stamp)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

/// Whether a window ending at `expires_at` has closed as of `now`: `expires_at <= now`.
/// Fails closed: an `expires_at` or `now` that does not parse counts as closed.
pub fn is_expired(expires_at: &str, now: &str) -> bool {
    match (parse(expires_at), parse(now)) {
        (Some(expires), Some(now)) => expires <= now,
        _ => true,
    }
}

/// Whether `later` is strictly after `earlier`. `None` when either stamp does not parse.
pub fn is_after(later: &str, earlier: &str) -> Option<bool> {
    Some(parse(later)? > parse(earlier)?)
}

/// Whether `at` is at or before `limit`. `None` when either stamp does not parse.
pub fn is_at_or_before(at: &str, limit: &str) -> Option<bool> {
    Some(parse(at)? <= parse(limit)?)
}
