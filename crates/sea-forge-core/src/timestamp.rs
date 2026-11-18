//! UTC timestamps used by the durable records and their projections.

use time::{OffsetDateTime, UtcOffset};

/// Return the current time in the existing RFC 3339 record format.
pub fn now_rfc3339() -> String {
    format_utc(OffsetDateTime::now_utc())
}

/// Return the current UTC time with exactly three fractional digits and a Z suffix.
pub fn now_rfc3339_millis_z() -> String {
    format_utc_millis_z(OffsetDateTime::now_utc())
}

fn format_utc_millis_z(timestamp: OffsetDateTime) -> String {
    let timestamp = timestamp.to_offset(UtcOffset::UTC);
    format!(
        "{}T{:02}:{:02}:{:02}.{:03}Z",
        timestamp.date(),
        timestamp.hour(),
        timestamp.minute(),
        timestamp.second(),
        timestamp.millisecond(),
    )
}

fn format_utc(timestamp: OffsetDateTime) -> String {
    let timestamp = timestamp.to_offset(UtcOffset::UTC);
    let mut text = format!(
        "{}T{:02}:{:02}:{:02}",
        timestamp.date(),
        timestamp.hour(),
        timestamp.minute(),
        timestamp.second()
    );
    let nanos = timestamp.nanosecond();
    if nanos != 0 {
        // Existing record hashes depend on millisecond, microsecond, or
        // nanosecond precision, rather than trimming arbitrary trailing zeros.
        use std::fmt::Write as _;
        if nanos.is_multiple_of(1_000_000) {
            write!(&mut text, ".{:03}", nanos / 1_000_000).unwrap();
        } else if nanos.is_multiple_of(1_000) {
            write!(&mut text, ".{:06}", nanos / 1_000).unwrap();
        } else {
            write!(&mut text, ".{nanos:09}").unwrap();
        }
    }
    text.push_str("+00:00");
    text
}

#[cfg(test)]
mod tests {
    use super::{format_utc, format_utc_millis_z};
    use time::macros::datetime;

    #[test]
    fn repair_timestamp_bytes_keep_fixed_milliseconds_and_z() {
        assert_eq!(
            format_utc_millis_z(datetime!(2026-11-18 08:00:00.123_456_7 -05:00)),
            "2026-11-18T13:00:00.123Z",
        );
        assert_eq!(
            format_utc_millis_z(datetime!(2026-11-18 08:00:00 UTC)),
            "2026-11-18T08:00:00.000Z",
        );
    }

    #[test]
    fn timestamp_bytes_match_historical_record_precision() {
        for (timestamp, expected) in [
            (
                datetime!(2026-11-18 08:00:00 UTC),
                "2026-11-18T08:00:00+00:00",
            ),
            (
                datetime!(2026-11-18 08:00:00.1 UTC),
                "2026-11-18T08:00:00.100+00:00",
            ),
            (
                datetime!(2026-11-18 08:00:00.123_4 UTC),
                "2026-11-18T08:00:00.123400+00:00",
            ),
            (
                datetime!(2026-11-18 08:00:00.123_456_7 -05:00),
                "2026-11-18T13:00:00.123456700+00:00",
            ),
        ] {
            assert_eq!(format_utc(timestamp), expected);
        }
    }
}
