use std::num::NonZeroU32;

/// A positive duration measured in seconds. Zero is unrepresentable.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DurationSeconds(NonZeroU32);

impl DurationSeconds {
    pub fn new(seconds: u32) -> Option<Self> {
        NonZeroU32::new(seconds).map(Self)
    }

    pub const fn from_non_zero(seconds: NonZeroU32) -> Self {
        Self(seconds)
    }

    pub fn as_secs(self) -> u32 {
        self.0.get()
    }
}

/// Format a duration in clock format: `mm:ss`, or `h:mm:ss` from one hour up.
///
/// ```
/// use countdown_timer_bar::domain::duration::format_clock;
///
/// assert_eq!(format_clock(65), "01:05");
/// assert_eq!(format_clock(3600), "1:00:00");
/// ```
pub fn format_clock(remaining_seconds: u32) -> String {
    let hours = remaining_seconds / 3600;
    let minutes = remaining_seconds % 3600 / 60;
    let seconds = remaining_seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

pub fn uses_hours(duration_seconds: u32) -> bool {
    duration_seconds >= 3600
}

/// Clock text in a format chosen up front, so the clock keeps its width at the
/// one-hour mark: `h:mm:ss` when `with_hours`, otherwise `mm:ss`.
///
/// ```
/// use countdown_timer_bar::domain::duration::format_clock_fixed;
///
/// assert_eq!(format_clock_fixed(3599, true), "0:59:59");
/// assert_eq!(format_clock_fixed(3599, false), "59:59");
/// ```
pub fn format_clock_fixed(remaining_seconds: u32, with_hours: bool) -> String {
    if with_hours {
        let hours = remaining_seconds / 3600;
        let minutes = remaining_seconds % 3600 / 60;
        let seconds = remaining_seconds % 60;
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format_clock(remaining_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_clock_pads_to_mm_ss() {
        assert_eq!(format_clock(0), "00:00");
        assert_eq!(format_clock(65), "01:05");
        assert_eq!(format_clock(3599), "59:59");
        assert_eq!(format_clock(3600), "1:00:00");
        assert_eq!(format_clock(3600 * 16 + 61), "16:01:01");
    }

    #[test]
    fn hours_are_used_from_exactly_one_hour() {
        assert!(!uses_hours(3599));
        assert!(uses_hours(3600));
        assert!(uses_hours(7200));
    }

    #[test]
    fn fixed_format_keeps_hours_below_one_hour() {
        assert_eq!(format_clock_fixed(3600, true), "1:00:00");
        assert_eq!(format_clock_fixed(3599, true), "0:59:59");
        assert_eq!(format_clock_fixed(61, true), "0:01:01");
        assert_eq!(format_clock_fixed(0, true), "0:00:00");
    }

    #[test]
    fn fixed_format_without_hours_is_mm_ss() {
        assert_eq!(format_clock_fixed(0, false), "00:00");
        assert_eq!(format_clock_fixed(65, false), "01:05");
        assert_eq!(format_clock_fixed(3599, false), "59:59");
    }
}
