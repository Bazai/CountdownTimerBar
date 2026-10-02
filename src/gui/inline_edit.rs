//! Text buffer behind the in-circle keyboard edit.
//!
//! Digits only; an existing value is pre-filled and replaced by the first
//! digit typed ("fresh"), like a selected field.

use crate::domain::preset_list::{PresetSpec, Unit, MIN_VALUE};

const MAX_DIGITS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditBuffer {
    text: String,
    fresh: bool,
}

impl EditBuffer {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            fresh: false,
        }
    }

    pub fn with_value(value: u32) -> Self {
        Self {
            text: value.to_string(),
            fresh: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn value(&self) -> u32 {
        self.text.parse().unwrap_or(0)
    }

    pub fn set_value(&mut self, value: u32) {
        self.text = if value == 0 {
            String::new()
        } else {
            value.to_string()
        };
        self.fresh = false;
    }

    /// Returns `false` when the key is not a digit or the buffer is full.
    pub fn push_digit(&mut self, ch: char) -> bool {
        if !ch.is_ascii_digit() {
            return false;
        }
        if self.fresh {
            self.text.clear();
            self.fresh = false;
        }
        if self.text.len() >= MAX_DIGITS {
            return false;
        }
        self.text.push(ch);
        true
    }

    pub fn backspace(&mut self) {
        if self.fresh {
            self.text.clear();
            self.fresh = false;
        } else {
            self.text.pop();
        }
    }

    /// The preset to store, or `None` to cancel (empty or zero). Values above
    /// the unit's range are clamped.
    pub fn commit(&self, unit: Unit) -> Option<PresetSpec> {
        let value: u32 = self.text.parse().ok()?;
        if value < MIN_VALUE {
            return None;
        }
        PresetSpec::new(value.min(unit.max_value()), unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_are_appended() {
        let mut buffer = EditBuffer::empty();
        assert!(buffer.push_digit('1'));
        assert!(buffer.push_digit('8'));
        assert_eq!(buffer.text(), "18");
    }

    #[test]
    fn other_keys_are_rejected_and_leave_the_text_alone() {
        let mut buffer = EditBuffer::empty();
        assert!(!buffer.push_digit('a'));
        assert!(!buffer.push_digit('-'));
        assert_eq!(buffer.text(), "");
    }

    #[test]
    fn first_digit_replaces_the_prefilled_value() {
        let mut buffer = EditBuffer::with_value(30);
        assert_eq!(buffer.text(), "30");
        buffer.push_digit('7');
        assert_eq!(buffer.text(), "7");
        buffer.push_digit('5');
        assert_eq!(buffer.text(), "75");
    }

    #[test]
    fn backspace_on_a_fresh_value_clears_it() {
        let mut buffer = EditBuffer::with_value(30);
        buffer.backspace();
        assert_eq!(buffer.text(), "");
        let mut buffer = EditBuffer::empty();
        buffer.push_digit('1');
        buffer.push_digit('2');
        buffer.backspace();
        assert_eq!(buffer.text(), "1");
    }

    #[test]
    fn input_is_capped_at_four_digits() {
        let mut buffer = EditBuffer::empty();
        for ch in "12345".chars() {
            buffer.push_digit(ch);
        }
        assert_eq!(buffer.text(), "1234");
    }

    #[test]
    fn empty_and_zero_commit_as_cancel() {
        assert_eq!(EditBuffer::empty().commit(Unit::Minutes), None);
        let mut zero = EditBuffer::empty();
        zero.push_digit('0');
        zero.push_digit('0');
        assert_eq!(zero.commit(Unit::Seconds), None);
    }

    #[test]
    fn commit_clamps_to_the_unit_range() {
        let mut buffer = EditBuffer::empty();
        for ch in "5000".chars() {
            buffer.push_digit(ch);
        }
        assert_eq!(
            buffer.commit(Unit::Minutes),
            PresetSpec::new(999, Unit::Minutes)
        );
        assert_eq!(
            buffer.commit(Unit::Seconds),
            PresetSpec::new(3599, Unit::Seconds)
        );
    }

    #[test]
    fn set_value_round_trips_through_text_and_value() {
        let mut buffer = EditBuffer::empty();
        assert_eq!(buffer.value(), 0);
        buffer.set_value(42);
        assert_eq!(buffer.text(), "42");
        assert_eq!(buffer.value(), 42);
    }

    #[test]
    fn a_digit_after_set_value_appends() {
        let mut buffer = EditBuffer::empty();
        buffer.set_value(42);
        buffer.push_digit('1');
        assert_eq!(buffer.text(), "421");
    }

    #[test]
    fn setting_zero_clears_the_text() {
        let mut buffer = EditBuffer::with_value(42);
        buffer.set_value(0);
        assert_eq!(buffer.text(), "");
    }

    #[test]
    fn unchanged_prefilled_value_commits_as_itself() {
        let buffer = EditBuffer::with_value(15);
        assert_eq!(
            buffer.commit(Unit::Minutes),
            PresetSpec::new(15, Unit::Minutes)
        );
    }
}
