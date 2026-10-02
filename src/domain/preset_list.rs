use std::fmt;
use std::num::NonZeroU32;

use super::duration::DurationSeconds;

pub const MAX_PRESETS: usize = 8;
pub const MIN_VALUE: u32 = 1;
pub const MAX_MINUTES: u32 = 999;
pub const MAX_SECONDS: u32 = 3599;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Unit {
    Minutes,
    Seconds,
}

impl Unit {
    pub fn max_value(self) -> u32 {
        match self {
            Self::Minutes => MAX_MINUTES,
            Self::Seconds => MAX_SECONDS,
        }
    }

    pub fn clamp(self, value: i64) -> u32 {
        let clamped = value.clamp(i64::from(MIN_VALUE), i64::from(self.max_value()));
        u32::try_from(clamped).unwrap_or(MIN_VALUE)
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Self::Minutes => "m",
            Self::Seconds => "s",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PresetSpec {
    value: NonZeroU32,
    unit: Unit,
}

const SECONDS_PER_MINUTE: NonZeroU32 = NonZeroU32::MIN.saturating_add(59);

impl PresetSpec {
    /// Returns `None` when `value` is outside `1..=unit.max_value()`.
    pub fn new(value: u32, unit: Unit) -> Option<Self> {
        NonZeroU32::new(value)
            .filter(|value| value.get() <= unit.max_value())
            .map(|value| Self { value, unit })
    }

    fn clamped(value: u32, unit: Unit) -> Self {
        let value = NonZeroU32::new(value.min(unit.max_value())).unwrap_or(NonZeroU32::MIN);
        Self { value, unit }
    }

    pub fn value(self) -> u32 {
        self.value.get()
    }

    pub fn unit(self) -> Unit {
        self.unit
    }

    /// Same number, other unit; the number is clamped to the new unit's range.
    pub fn with_unit(self, unit: Unit) -> Self {
        Self::clamped(self.value.get(), unit)
    }

    pub fn with_value_clamped(self, value: i64) -> Self {
        Self::clamped(self.unit.clamp(value), self.unit)
    }

    pub fn to_duration(self) -> DurationSeconds {
        let seconds = match self.unit {
            Unit::Minutes => self.value.saturating_mul(SECONDS_PER_MINUTE),
            Unit::Seconds => self.value,
        };
        DurationSeconds::from_non_zero(seconds)
    }

    /// Recovers a spec from stored seconds (the unit is not persisted): whole
    /// minutes become minutes, anything else seconds; legacy values are clamped.
    pub fn from_duration(duration: DurationSeconds) -> Self {
        let seconds = duration.as_secs();
        if seconds % 60 == 0 {
            Self::clamped(seconds / 60, Unit::Minutes)
        } else {
            Self::clamped(seconds, Unit::Seconds)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorReason {
    NotADuration,
    OutOfRange(Unit),
    TooMany,
}

/// First problem found in an input string. `token` is empty for `TooMany`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub token: String,
    pub reason: ParseErrorReason,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.reason {
            ParseErrorReason::NotADuration => write!(
                f,
                "“{}” isn’t a duration. Use numbers, add s for seconds: 15, 45s",
                self.token
            ),
            ParseErrorReason::OutOfRange(Unit::Minutes) => write!(
                f,
                "“{}” is out of range. Minutes: {MIN_VALUE}–{MAX_MINUTES}",
                self.token
            ),
            ParseErrorReason::OutOfRange(Unit::Seconds) => write!(
                f,
                "“{}” is out of range. Seconds: {MIN_VALUE}–{MAX_SECONDS}",
                self.token
            ),
            ParseErrorReason::TooMany => write!(f, "At most {MAX_PRESETS} timers per list"),
        }
    }
}

impl std::error::Error for ParseError {}

fn parse_token(token: &str) -> Result<PresetSpec, ParseError> {
    let error = |reason| ParseError {
        token: token.to_owned(),
        reason,
    };
    let (digits, unit) = if let Some(rest) = token.strip_suffix('s') {
        (rest, Unit::Seconds)
    } else if let Some(rest) = token.strip_suffix('m') {
        (rest, Unit::Minutes)
    } else {
        (token, Unit::Minutes)
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(ParseErrorReason::NotADuration));
    }
    // Only digits reach here, so a parse failure means u32 overflow: out of range.
    let value = digits.parse::<u32>().unwrap_or(u32::MAX);
    PresetSpec::new(value, unit).ok_or_else(|| error(ParseErrorReason::OutOfRange(unit)))
}

/// Parses `"13s, 1, 10, 2"`. `N` and `Nm` are minutes, `Ns` seconds;
/// whitespace and empty tokens are ignored. Fails on the first bad token,
/// then on more than [`MAX_PRESETS`] entries.
///
/// ```
/// use countdown_timer_bar::domain::preset_list::{format_list, parse_list};
///
/// let specs = parse_list("13s, 1, 10, 2")?;
/// assert_eq!(specs.len(), 4);
/// assert_eq!(format_list(&specs), "13s, 1, 10, 2");
/// assert!(parse_list("1, soon").is_err());
/// # Ok::<(), countdown_timer_bar::domain::preset_list::ParseError>(())
/// ```
///
/// # Errors
///
/// Returns a [`ParseError`] naming the first bad token, or the overflow.
pub fn parse_list(input: &str) -> Result<Vec<PresetSpec>, ParseError> {
    let specs = input
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(parse_token)
        .collect::<Result<Vec<_>, _>>()?;
    if specs.len() > MAX_PRESETS {
        return Err(ParseError {
            token: String::new(),
            reason: ParseErrorReason::TooMany,
        });
    }
    Ok(specs)
}

/// Inverse of [`parse_list`]: minutes without suffix, seconds with `s`.
///
/// ```
/// use countdown_timer_bar::domain::preset_list::{format_list, PresetSpec, Unit};
///
/// let specs = [
///     PresetSpec::new(90, Unit::Seconds).unwrap(),
///     PresetSpec::new(15, Unit::Minutes).unwrap(),
/// ];
/// assert_eq!(format_list(&specs), "90s, 15");
/// ```
pub fn format_list(specs: &[PresetSpec]) -> String {
    specs
        .iter()
        .map(|spec| match spec.unit {
            Unit::Minutes => spec.value.to_string(),
            Unit::Seconds => format!("{}s", spec.value),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(value: u32) -> PresetSpec {
        PresetSpec::new(value, Unit::Minutes).unwrap()
    }

    fn s(value: u32) -> PresetSpec {
        PresetSpec::new(value, Unit::Seconds).unwrap()
    }

    #[test]
    fn round_trips_the_mock_example() {
        let parsed = parse_list("13s, 1, 10, 2").unwrap();
        assert_eq!(parsed, vec![s(13), m(1), m(10), m(2)]);
        assert_eq!(format_list(&parsed), "13s, 1, 10, 2");
    }

    #[test]
    fn accepts_optional_m_suffix_and_ignores_whitespace() {
        assert_eq!(
            parse_list("  15m ,30 ,  45s ").unwrap(),
            vec![m(15), m(30), s(45)]
        );
    }

    #[test]
    fn empty_input_and_empty_tokens_are_fine() {
        assert_eq!(parse_list("").unwrap(), vec![]);
        assert_eq!(parse_list("  ").unwrap(), vec![]);
        assert_eq!(parse_list("1,,2,").unwrap(), vec![m(1), m(2)]);
        assert_eq!(format_list(&[]), "");
    }

    #[test]
    fn reports_the_first_invalid_token() {
        let error = parse_list("15, 30, 35фыва, abc").unwrap_err();
        assert_eq!(error.token, "35фыва");
        assert_eq!(error.reason, ParseErrorReason::NotADuration);
        assert_eq!(
            error.to_string(),
            "“35фыва” isn’t a duration. Use numbers, add s for seconds: 15, 45s"
        );
    }

    #[test]
    fn rejects_malformed_tokens() {
        for token in ["s", "m", "-5", "+5", "1.5", "1 5", "5ms", "5S", "٣"] {
            assert_eq!(
                parse_list(token).unwrap_err().reason,
                ParseErrorReason::NotADuration,
                "{token}"
            );
        }
    }

    #[test]
    fn enforces_minute_limits() {
        assert_eq!(parse_list("1").unwrap(), vec![m(1)]);
        assert_eq!(parse_list("999").unwrap(), vec![m(999)]);
        for token in ["0", "0m", "1000", "4294967296"] {
            assert_eq!(
                parse_list(token).unwrap_err().reason,
                ParseErrorReason::OutOfRange(Unit::Minutes),
                "{token}"
            );
        }
    }

    #[test]
    fn enforces_second_limits() {
        assert_eq!(parse_list("1s").unwrap(), vec![s(1)]);
        assert_eq!(parse_list("3599s").unwrap(), vec![s(3599)]);
        for token in ["0s", "3600s", "99999999999s"] {
            assert_eq!(
                parse_list(token).unwrap_err().reason,
                ParseErrorReason::OutOfRange(Unit::Seconds),
                "{token}"
            );
        }
    }

    #[test]
    fn allows_eight_presets_but_not_nine() {
        assert_eq!(parse_list("1,2,3,4,5,6,7,8").unwrap().len(), MAX_PRESETS);
        let error = parse_list("1,2,3,4,5,6,7,8,9").unwrap_err();
        assert_eq!(error.reason, ParseErrorReason::TooMany);
    }

    #[test]
    fn invalid_token_wins_over_too_many() {
        let error = parse_list("1,2,3,4,5,6,7,8,9,x").unwrap_err();
        assert_eq!(error.reason, ParseErrorReason::NotADuration);
    }

    #[test]
    fn spec_constructor_checks_range() {
        assert!(PresetSpec::new(0, Unit::Minutes).is_none());
        assert!(PresetSpec::new(1000, Unit::Minutes).is_none());
        assert!(PresetSpec::new(3600, Unit::Seconds).is_none());
        assert!(PresetSpec::new(3599, Unit::Seconds).is_some());
    }

    #[test]
    fn converts_to_duration() {
        assert_eq!(m(15).to_duration().as_secs(), 900);
        assert_eq!(s(13).to_duration().as_secs(), 13);
        assert_eq!(m(999).to_duration().as_secs(), 59_940);
    }

    #[test]
    fn recovers_unit_from_stored_seconds() {
        let d = |secs| DurationSeconds::new(secs).unwrap();
        assert_eq!(PresetSpec::from_duration(d(900)), m(15));
        assert_eq!(PresetSpec::from_duration(d(60)), m(1));
        assert_eq!(PresetSpec::from_duration(d(13)), s(13));
        assert_eq!(PresetSpec::from_duration(d(90)), s(90));
    }

    #[test]
    fn legacy_out_of_range_seconds_are_clamped_when_recovered() {
        let d = |secs| DurationSeconds::new(secs).unwrap();
        assert_eq!(PresetSpec::from_duration(d(60_000_000)), m(999));
        assert_eq!(PresetSpec::from_duration(d(7201)), s(3599));
    }

    #[test]
    fn unit_toggle_keeps_the_number_and_clamps() {
        assert_eq!(m(30).with_unit(Unit::Seconds), s(30));
        assert_eq!(m(500).with_unit(Unit::Seconds), s(500));
        assert_eq!(m(999).with_unit(Unit::Seconds), s(999));
        assert_eq!(s(45).with_unit(Unit::Minutes), m(45));
    }

    #[test]
    fn knob_value_is_clamped_to_the_unit_range() {
        assert_eq!(m(5).with_value_clamped(-3), m(1));
        assert_eq!(m(5).with_value_clamped(5000), m(999));
        assert_eq!(s(5).with_value_clamped(5000), s(3599));
        assert_eq!(s(5).with_value_clamped(12), s(12));
    }
}
