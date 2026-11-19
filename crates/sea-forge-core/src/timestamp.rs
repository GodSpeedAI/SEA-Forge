//! UTC timestamps used by the durable records and their projections.

use std::fmt::{self, Write as _};
use std::ops::{Add, Sub};

use time::{Date, Duration, Month, OffsetDateTime, UtcOffset};

const MIN_YEAR: i32 = -262_143;
const MAX_YEAR: i32 = 262_142;
const NANOS_PER_SECOND: i128 = 1_000_000_000;
const SECONDS_PER_DAY: i64 = 86_400;

/// A UTC instant parsed from or written to SEA Forge records.
///
/// The Unix second anchors a leap second to the preceding ordinary second.
/// The fractional field stores the additional second so that an instant
/// during a leap second sorts between that ordinary second and the next.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp {
    unix_second: i64,
    fraction: u32,
}

/// The timestamp did not match the RFC 3339 syntax accepted by stored records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseTimestampError;

impl fmt::Display for ParseTimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid RFC 3339 timestamp")
    }
}

impl std::error::Error for ParseTimestampError {}

impl Timestamp {
    /// Read the system clock in UTC.
    pub fn now_utc() -> Self {
        Self::from_datetime(OffsetDateTime::now_utc())
    }

    /// Parse an RFC 3339 timestamp and normalize its offset to UTC.
    ///
    /// This accepts lowercase separators, a space in place of T, a Unicode
    /// minus in the offset, and leap seconds, as the previous parser did.
    pub fn parse_rfc3339(input: &str) -> Result<Self, ParseTimestampError> {
        parse_rfc3339(input).ok_or(ParseTimestampError)
    }

    /// Format in UTC using the existing variable precision and +00:00 suffix.
    pub fn to_rfc3339(self) -> String {
        self.format(Precision::Variable, "+00:00")
    }

    /// Format in UTC at whole-second precision with a Z suffix.
    pub fn to_rfc3339_secs_z(self) -> String {
        self.format(Precision::Seconds, "Z")
    }

    /// Format in UTC with exactly three fractional digits and a Z suffix.
    pub fn to_rfc3339_millis_z(self) -> String {
        self.format(Precision::Millis, "Z")
    }

    /// Return Unix epoch nanoseconds, counting leap fractions from their anchor.
    pub fn unix_timestamp_nanos(self) -> i128 {
        i128::from(self.unix_second) * NANOS_PER_SECOND + i128::from(self.fraction)
    }

    /// Remove fractional precision while preserving the second on the clock.
    pub fn truncate_to_seconds(self) -> Self {
        Self {
            fraction: (self.fraction / 1_000_000_000) * 1_000_000_000,
            ..self
        }
    }

    /// Compute the signed difference using the historical leap-second rules.
    pub fn signed_duration_since(self, earlier: Self) -> Duration {
        // The calendar day difference has always assumed exactly 86400
        // seconds. Account for a leap second only in the time-of-day delta.
        let local_second = self.unix_second.rem_euclid(SECONDS_PER_DAY);
        let earlier_local_second = earlier.unix_second.rem_euclid(SECONDS_PER_DAY);
        let leap_adjustment = if local_second > earlier_local_second && earlier.is_leap() {
            1
        } else if local_second < earlier_local_second && self.is_leap() {
            -1
        } else {
            0
        };
        Duration::seconds(self.unix_second - earlier.unix_second + leap_adjustment)
            + Duration::nanoseconds(i64::from(self.fraction) - i64::from(earlier.fraction))
    }

    /// Add a duration, returning None if the result exceeds the record calendar.
    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        self.checked_shift(duration.whole_nanoseconds())
    }

    fn checked_shift(self, nanos: i128) -> Option<Self> {
        let mut fraction = i128::from(self.fraction) + nanos;
        if self.is_leap() {
            if (NANOS_PER_SECOND..2 * NANOS_PER_SECOND).contains(&fraction) {
                return Some(Self {
                    fraction: fraction as u32,
                    ..self
                });
            }
            if fraction >= 2 * NANOS_PER_SECOND {
                fraction -= NANOS_PER_SECOND;
            }
        }
        let shift = i64::try_from(fraction.div_euclid(NANOS_PER_SECOND)).ok()?;
        let unix_second = self.unix_second.checked_add(shift)?;
        Self::from_parts(unix_second, fraction.rem_euclid(NANOS_PER_SECOND) as u32)
    }

    fn from_parts(unix_second: i64, fraction: u32) -> Option<Self> {
        let year = OffsetDateTime::from_unix_timestamp(unix_second)
            .ok()?
            .year();
        (MIN_YEAR..=MAX_YEAR).contains(&year).then_some(Self {
            unix_second,
            fraction,
        })
    }

    fn from_datetime(value: OffsetDateTime) -> Self {
        Self {
            unix_second: value.unix_timestamp(),
            fraction: value.nanosecond(),
        }
    }

    fn is_leap(self) -> bool {
        self.fraction >= 1_000_000_000
    }

    fn format(self, precision: Precision, suffix: &str) -> String {
        let time =
            OffsetDateTime::from_unix_timestamp(self.unix_second).expect("validated UTC timestamp");
        let mut output = format!(
            "{}T{:02}:{:02}:{:02}",
            time.date(),
            time.hour(),
            time.minute(),
            time.second() + u8::from(self.is_leap())
        );
        let nanos = self.fraction % 1_000_000_000;
        match precision {
            Precision::Seconds => {}
            Precision::Millis => write!(output, ".{:03}", nanos / 1_000_000).unwrap(),
            Precision::Variable if nanos == 0 => {}
            Precision::Variable if nanos.is_multiple_of(1_000_000) => {
                write!(output, ".{:03}", nanos / 1_000_000).unwrap();
            }
            Precision::Variable if nanos.is_multiple_of(1_000) => {
                write!(output, ".{:06}", nanos / 1_000).unwrap();
            }
            Precision::Variable => write!(output, ".{nanos:09}").unwrap(),
        }
        output.push_str(suffix);
        output
    }
}

#[derive(Clone, Copy)]
enum Precision {
    Seconds,
    Millis,
    Variable,
}

impl serde::Serialize for Timestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.format(Precision::Variable, "Z"))
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        self.checked_add(duration)
            .expect("timestamp addition out of range")
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self {
        self.checked_shift(-duration.whole_nanoseconds())
            .expect("timestamp subtraction out of range")
    }
}

impl Sub for Timestamp {
    type Output = Duration;

    fn sub(self, earlier: Self) -> Duration {
        self.signed_duration_since(earlier)
    }
}

fn parse_rfc3339(input: &str) -> Option<Timestamp> {
    let b = input.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || !matches!(b[10], b'T' | b't' | b' ')
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let year = digits(&b[0..4])? as i32;
    let month = Month::try_from(digits(&b[5..7])? as u8).ok()?;
    let day = digits(&b[8..10])? as u8;
    let hour = digits(&b[11..13])? as u8;
    let minute = digits(&b[14..16])? as u8;
    let second = digits(&b[17..19])? as u8;
    let (fraction, zone) = fraction_and_zone(&input[19..])?;
    let offset = parse_offset(zone)?;
    from_local(year, month, day, [hour, minute, second], fraction, offset)
}

fn from_local(
    year: i32,
    month: Month,
    day: u8,
    [hour, minute, second]: [u8; 3],
    fraction: u32,
    offset: UtcOffset,
) -> Option<Timestamp> {
    if second > 60 || !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return None;
    }
    let local = Date::from_calendar_date(year, month, day)
        .ok()?
        .with_hms(hour, minute, second.min(59))
        .ok()?;
    Timestamp::from_parts(
        local.assume_offset(offset).unix_timestamp(),
        fraction + u32::from(second == 60) * 1_000_000_000,
    )
}

// All callers supply slices containing at most nine digits.
fn digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0, |value, digit| {
        digit
            .is_ascii_digit()
            .then(|| value * 10 + u32::from(digit - b'0'))
    })
}

fn fraction_and_zone(input: &str) -> Option<(u32, &str)> {
    let Some(input) = input.strip_prefix('.') else {
        return Some((0, input));
    };
    let len = input.bytes().take_while(u8::is_ascii_digit).count();
    if len == 0 {
        return None;
    }
    let precision = len.min(9);
    let fraction = digits(&input.as_bytes()[..precision])? * 10_u32.pow((9 - precision) as u32);
    Some((fraction, &input[len..]))
}

fn parse_offset(input: &str) -> Option<UtcOffset> {
    if matches!(input, "Z" | "z") {
        return Some(UtcOffset::UTC);
    }
    let (sign, offset) = if let Some(offset) = input.strip_prefix('+') {
        (1, offset)
    } else {
        (-1, input.strip_prefix(['-', '−'])?)
    };
    let b = offset.as_bytes();
    if b.len() != 5 || b[2] != b':' {
        return None;
    }
    let hour = digits(&b[..2])? as i8;
    let minute = digits(&b[3..])? as i8;
    if hour > 23 {
        return None;
    }
    UtcOffset::from_hms(sign * hour, sign * minute, 0).ok()
}

/// Return the current time in the existing RFC 3339 record format.
pub fn now_rfc3339() -> String {
    Timestamp::now_utc().to_rfc3339()
}

/// Format a Unix second in the historical UTC record calendar.
pub fn rfc3339_from_unix_seconds(seconds: i64) -> Option<String> {
    Some(Timestamp::from_parts(seconds, 0)?.to_rfc3339())
}

/// Return the current UTC time with exactly three fractional digits and a Z suffix.
pub fn now_rfc3339_millis_z() -> String {
    Timestamp::now_utc().to_rfc3339_millis_z()
}

#[cfg(test)]
mod tests {
    use super::Timestamp;
    use time::macros::datetime;

    #[test]
    fn rfc3339_acceptance_and_order_match_existing_records() {
        use super::Timestamp;

        let parse = |text| Timestamp::parse_rfc3339(text).unwrap();
        let leap = parse("2016-12-31t22:59:60.1234567899−01:00");
        assert_eq!(leap.to_rfc3339(), "2016-12-31T23:59:60.123456789+00:00");
        assert_eq!(leap.to_rfc3339_millis_z(), "2016-12-31T23:59:60.123Z");
        assert_eq!(
            serde_json::to_string(&leap).unwrap(),
            r#""2016-12-31T23:59:60.123456789Z""#
        );
        assert!(parse("2016-12-31T23:59:59.999999999Z") < leap);
        assert!(leap < parse("2017-01-01T00:00:00Z"));
        assert_eq!(leap, parse("2017-01-01 00:59:60.123456789+01:00"));
        assert_eq!(
            parse("2000-01-01T12:03:60Z").to_rfc3339_secs_z(),
            "2000-01-01T12:03:60Z"
        );
        for input in [
            "2000-02-30T00:00:00Z",
            "2000-01-01T00:00:00.+01:00",
            "2000-01-01T00:00:00+24:00",
            "2000-01-01T00:00:00+00:60",
            "2000-01-01T00:00:00+0000",
            "2000-01-01T00:00:00Z ",
            "2000-01-01T00:00:61Z",
            "2000-01-01T00:00:0é",
        ] {
            assert!(Timestamp::parse_rfc3339(input).is_err(), "{input}");
        }
    }

    #[test]
    fn signed_deadline_arithmetic_matches_existing_leap_second_behavior() {
        use super::Timestamp;
        use time::Duration;

        let parse = |text| Timestamp::parse_rfc3339(text).unwrap();
        let leap = parse("2016-12-31T23:59:60.250Z");
        let previous = parse("2016-12-31T23:59:59.700Z");
        let midnight = parse("2017-01-01T00:00:00Z");
        assert_eq!(
            leap.truncate_to_seconds().to_rfc3339_secs_z(),
            "2016-12-31T23:59:60Z"
        );
        assert_eq!(leap - previous, Duration::milliseconds(550));
        assert_eq!(previous - leap, Duration::milliseconds(-550));
        // Chrono counts the date delta and time-of-day delta separately.
        assert_eq!(midnight - leap, Duration::milliseconds(-250));
        assert_eq!(leap + Duration::milliseconds(750), midnight);
        assert_eq!(leap - Duration::milliseconds(550), previous);
        assert_eq!(
            leap + Duration::seconds(1),
            midnight + Duration::milliseconds(250)
        );
        assert!(leap.checked_add(Duration::MAX).is_none());
    }

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
            Timestamp::from_datetime(datetime!(2026-11-18 08:00:00.123_456_7 -05:00))
                .to_rfc3339_millis_z(),
            "2026-11-18T13:00:00.123Z",
        );
        assert_eq!(
            Timestamp::from_datetime(datetime!(2026-11-18 08:00:00 UTC)).to_rfc3339_millis_z(),
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
            assert_eq!(Timestamp::from_datetime(timestamp).to_rfc3339(), expected);
        }
    }
}
