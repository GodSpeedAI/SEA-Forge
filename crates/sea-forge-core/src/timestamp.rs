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
pub enum ParseTimestampError {
    /// A date, time or offset component was outside its allowed range.
    OutOfRange,
    /// The input ended before the timestamp was complete.
    Incomplete,
    /// Unexpected characters appeared in the timestamp.
    Invalid,
    /// Extra characters followed a complete timestamp.
    Trailing,
}

impl fmt::Display for ParseTimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Self::OutOfRange => "input is out of range",
            Self::Incomplete => "premature end of input",
            Self::Invalid => "input contains invalid characters",
            Self::Trailing => "trailing input",
        };
        f.write_str(reason)
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
        parse_rfc3339(input)
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

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TimestampVisitor;

        impl serde::de::Visitor<'_> for TimestampVisitor {
            type Value = Timestamp;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an RFC 3339 formatted date and time string")
            }

            fn visit_str<E: serde::de::Error>(self, input: &str) -> Result<Timestamp, E> {
                Timestamp::parse_rfc3339(input)
                    .or_else(|error| parse_wire(input).ok_or(error))
                    .map_err(E::custom)
            }
        }

        deserializer.deserialize_str(TimestampVisitor)
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

fn parse_rfc3339(input: &str) -> Result<Timestamp, ParseTimestampError> {
    use ParseTimestampError::{Incomplete, Invalid};

    let bytes = input.as_bytes();
    if bytes.len() < 19 {
        return Err(Incomplete);
    }
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(Invalid);
    }
    let year = digits(&bytes[0..4]).ok_or(Invalid)? as i32;
    let month = digits(&bytes[5..7]).ok_or(Invalid)? as u8;
    let day = digits(&bytes[8..10]).ok_or(Invalid)? as u8;
    let date = calendar_date(year, month, day)?;
    if !matches!(bytes[10], b'T' | b't' | b' ') || bytes[13] != b':' || bytes[16] != b':' {
        return Err(Invalid);
    }
    let hour = digits(&bytes[11..13]).ok_or(Invalid)? as u8;
    let minute = digits(&bytes[14..16]).ok_or(Invalid)? as u8;
    let second = digits(&bytes[17..19]).ok_or(Invalid)? as u8;
    let (fraction, zone) = fraction_and_zone(&input[19..])?;
    let (local, fraction) = local_time(date, [hour, minute, second], fraction)?;
    let offset = parse_offset(zone, false)?;
    from_local(local, fraction, offset)
}

fn calendar_date(year: i32, month: u8, day: u8) -> Result<Date, ParseTimestampError> {
    use ParseTimestampError::OutOfRange;

    if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return Err(OutOfRange);
    }
    let month = Month::try_from(month).map_err(|_| OutOfRange)?;
    Date::from_calendar_date(year, month, day).map_err(|_| OutOfRange)
}

fn local_time(
    date: Date,
    [hour, minute, second]: [u8; 3],
    fraction: u32,
) -> Result<(time::PrimitiveDateTime, u32), ParseTimestampError> {
    use ParseTimestampError::OutOfRange;

    if second > 60 {
        return Err(OutOfRange);
    }
    let local = date
        .with_hms(hour, minute, second.min(59))
        .map_err(|_| OutOfRange)?;
    Ok((local, fraction + u32::from(second == 60) * 1_000_000_000))
}

fn from_local(
    local: time::PrimitiveDateTime,
    fraction: u32,
    offset: UtcOffset,
) -> Result<Timestamp, ParseTimestampError> {
    Timestamp::from_parts(local.assume_offset(offset).unix_timestamp(), fraction)
        .ok_or(ParseTimestampError::OutOfRange)
}

// Serialized DateTime values used a relaxed RFC 3339 grammar. Callers that
// validate an RFC 3339 input continue to use the strict entry point above.
fn parse_wire(input: &str) -> Option<Timestamp> {
    let input = input.trim_start();
    let sign = usize::from(input.starts_with(['+', '-']));
    let width = if sign == 0 { 4 } else { usize::MAX };
    let (year_digits, rest) = split_digits(&input[sign..], width);
    let year = input[..sign + year_digits.len()].parse().ok()?;
    let (month, rest) = wire_component(rest.trim_start().strip_prefix('-')?)?;
    let (day, rest) = wire_component(rest.trim_start().strip_prefix('-')?)?;
    let date = calendar_date(year, month, day).ok()?;
    let (hour, rest) = wire_component(rest.strip_prefix(['T', 't', ' '])?)?;
    let (minute, rest) = wire_component(rest.trim_start().strip_prefix(':')?)?;
    let (second, rest) = wire_component(rest.trim_start().strip_prefix(':')?)?;
    let (fraction, zone) = fraction_and_zone(rest).ok()?;
    let (local, fraction) = local_time(date, [hour, minute, second], fraction).ok()?;
    from_local(local, fraction, parse_offset(zone, true).ok()?).ok()
}

fn wire_component(input: &str) -> Option<(u8, &str)> {
    let (value, rest) = split_digits(input.trim_start(), 2);
    Some((value.parse().ok()?, rest))
}

fn split_digits(input: &str, width: usize) -> (&str, &str) {
    let len = input
        .bytes()
        .take(width)
        .take_while(u8::is_ascii_digit)
        .count();
    input.split_at(len)
}

// All callers supply slices containing at most nine digits.
fn digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0, |value, digit| {
        digit
            .is_ascii_digit()
            .then(|| value * 10 + u32::from(digit - b'0'))
    })
}

fn fraction_and_zone(input: &str) -> Result<(u32, &str), ParseTimestampError> {
    let Some(input) = input.strip_prefix('.') else {
        return Ok((0, input));
    };
    let (numeral, rest) = split_digits(input, 9);
    if numeral.is_empty() {
        return Err(if input.is_empty() {
            ParseTimestampError::Incomplete
        } else {
            ParseTimestampError::Invalid
        });
    }
    let precision = numeral.len();
    let fraction =
        digits(numeral.as_bytes()).expect("ASCII digits") * 10_u32.pow((9 - precision) as u32);
    Ok((
        fraction,
        rest.trim_start_matches(|c: char| c.is_ascii_digit()),
    ))
}

fn parse_offset(input: &str, relaxed: bool) -> Result<UtcOffset, ParseTimestampError> {
    use ParseTimestampError::{Incomplete, Invalid, OutOfRange, Trailing};

    let input = if relaxed { input.trim() } else { input };
    if relaxed && input.eq_ignore_ascii_case("UTC") {
        return Ok(UtcOffset::UTC);
    }
    if let Some(rest) = input.strip_prefix(['Z', 'z']) {
        return if rest.is_empty() {
            Ok(UtcOffset::UTC)
        } else {
            Err(Trailing)
        };
    }
    let (sign, offset) = match input.chars().next() {
        Some('+') => (1, &input[1..]),
        Some('-') => (-1, &input[1..]),
        Some('−') => (-1, &input['−'.len_utf8()..]),
        Some(_) => return Err(Invalid),
        None => return Err(Incomplete),
    };
    let hour = digits(offset.as_bytes().get(..2).ok_or(Incomplete)?).ok_or(Invalid)? as i8;
    let remainder = &offset[2..];
    let minutes = if relaxed {
        remainder.trim_start_matches(|c: char| c == ':' || c.is_whitespace())
    } else {
        remainder.strip_prefix(':').ok_or(if remainder.is_empty() {
            Incomplete
        } else {
            Invalid
        })?
    };
    let minute = digits(minutes.as_bytes().get(..2).ok_or(Incomplete)?).ok_or(Invalid)? as i8;
    if minute > 59 {
        return Err(OutOfRange);
    }
    if minutes.len() > 2 {
        return Err(Trailing);
    }
    if hour > 23 {
        return Err(OutOfRange);
    }
    UtcOffset::from_hms(sign * hour, sign * minute, 0).map_err(|_| OutOfRange)
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
    fn persisted_timestamps_round_trip_with_legacy_utc_spelling() {
        for (input, expected) in [
            ("2000-1-2 3:4:5 UTC", "2000-01-02T03:04:05Z"),
            ("2017-01-01t00:59:60.25+01:00", "2016-12-31T23:59:60.250Z"),
            ("+10000-01-01T00:00:00Z", "+10000-01-01T00:00:00Z"),
            (
                "1969-12-31T23:59:59.123456789Z",
                "1969-12-31T23:59:59.123456789Z",
            ),
        ] {
            let encoded = serde_json::to_string(input).unwrap();
            let timestamp = serde_json::from_str::<Timestamp>(&encoded).unwrap();
            assert_eq!(serde_json::to_value(timestamp).unwrap(), expected);
        }
        assert!(Timestamp::parse_rfc3339("2000-1-2 3:4:5 UTC").is_err());
        assert_eq!(
            Timestamp::parse_rfc3339("2020-02-30T00:00:00Z")
                .unwrap_err()
                .to_string(),
            "input is out of range"
        );
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
