//! UTC timestamps used by the durable records and their projections.

use time::{OffsetDateTime, UtcOffset};

/// Return the current time in the existing RFC 3339 record format.
pub fn now_rfc3339() -> String {
    format_utc(OffsetDateTime::now_utc())
}

/// Format a Unix second in the historical Chrono UTC record format.
///
/// Chrono supports years -262143 through 262142. Return `None` outside
/// that range even though `time` with `large-dates` can represent more.
pub fn rfc3339_from_unix_seconds(seconds: i64) -> Option<String> {
    let timestamp = OffsetDateTime::from_unix_timestamp(seconds).ok()?;
    if !(-262_143..=262_142).contains(&timestamp.year()) {
        return None;
    }
    Some(format_utc(timestamp))
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
    fn unix_seconds_match_chrono_year_boundaries_and_format() {
        use time::{Date, Month};

        let seconds = |year, month, day, hour, minute, second| {
            Date::from_calendar_date(year, month, day)
                .unwrap()
                .with_hms(hour, minute, second)
                .unwrap()
                .assume_utc()
                .unix_timestamp()
        };
        for (epoch, expected) in [
            (
                seconds(-262_143, Month::January, 1, 0, 0, 0),
                "-262143-01-01T00:00:00+00:00",
            ),
            (
                seconds(-1, Month::December, 31, 23, 59, 59),
                "-0001-12-31T23:59:59+00:00",
            ),
            (
                seconds(0, Month::January, 1, 0, 0, 0),
                "0000-01-01T00:00:00+00:00",
            ),
            (-1, "1969-12-31T23:59:59+00:00"),
            (0, "1970-01-01T00:00:00+00:00"),
            (
                seconds(9999, Month::December, 31, 23, 59, 59),
                "9999-12-31T23:59:59+00:00",
            ),
            (
                seconds(10000, Month::January, 1, 0, 0, 0),
                "+10000-01-01T00:00:00+00:00",
            ),
            (
                seconds(262_142, Month::December, 31, 23, 59, 59),
                "+262142-12-31T23:59:59+00:00",
            ),
        ] {
            assert_eq!(
                super::rfc3339_from_unix_seconds(epoch).as_deref(),
                Some(expected)
            );
        }
        for epoch in [
            seconds(-262_143, Month::January, 1, 0, 0, 0) - 1,
            seconds(262_142, Month::December, 31, 23, 59, 59) + 1,
            i64::MIN,
            i64::MAX,
        ] {
            assert!(super::rfc3339_from_unix_seconds(epoch).is_none(), "{epoch}");
        }
    }

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
