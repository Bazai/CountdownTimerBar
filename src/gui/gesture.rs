//! Pointer gesture on a timer circle: click and knob drag.
//!
//! Pure logic, no GPUI: the view feeds it window-space Y coordinates. There
//! is deliberately no "drag out to remove" (see docs/adr/0001).

pub const DRAG_THRESHOLD: f32 = 4.;
pub const PX_PER_STEP: f32 = 6.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Pressed,
    Knob,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Click,
    Commit(i32),
    Cancel,
}

#[derive(Clone, Copy, Debug)]
pub struct Gesture {
    start_y: f32,
    phase: Phase,
    steps: i32,
}

impl Gesture {
    pub fn new(start_y: f32) -> Self {
        Self {
            start_y,
            phase: Phase::Pressed,
            steps: 0,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn steps(&self) -> i32 {
        self.steps
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "pointer travel in px divided by the step size is a small number"
    )]
    pub fn moved(&mut self, y: f32) {
        let up = self.start_y - y;
        if self.phase == Phase::Pressed && up.abs() >= DRAG_THRESHOLD {
            self.phase = Phase::Knob;
        }
        if self.phase == Phase::Knob {
            self.steps = (up / PX_PER_STEP).trunc() as i32;
        }
    }

    pub fn released(mut self, y: f32) -> Outcome {
        self.moved(y);
        match self.phase {
            Phase::Pressed => Outcome::Click,
            Phase::Knob if self.steps == 0 => Outcome::Cancel,
            Phase::Knob => Outcome::Commit(self.steps),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Circle,
    EditExisting,
    EditNew,
    Plus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Nothing,
    PressTimer,
    /// Set the value to this number; the view clamps it.
    SetValue(i64),
    BeginNew,
}

/// Maps a gesture outcome to an action. `base` is the value the knob started
/// from (the typed number for the edit sources, 0 for "+").
pub fn resolve(source: Source, outcome: Outcome, base: i64) -> Action {
    match (source, outcome) {
        (Source::Circle, Outcome::Click) => Action::PressTimer,
        // "+": a click opens the field, a knob drag creates the preset right
        // away (value from 0).
        (Source::Plus, Outcome::Click) => Action::BeginNew,
        (_, Outcome::Click | Outcome::Cancel) => Action::Nothing,
        (_, Outcome::Commit(steps)) => Action::SetValue(base + i64::from(steps)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_without_movement_is_a_click() {
        assert_eq!(Gesture::new(100.).released(100.), Outcome::Click);
    }

    #[test]
    fn small_wobble_below_threshold_is_still_a_click() {
        let mut g = Gesture::new(100.);
        g.moved(97.1);
        g.moved(103.9);
        assert_eq!(g.phase(), Phase::Pressed);
        assert_eq!(g.released(103.9), Outcome::Click);
    }

    #[test]
    fn exactly_the_threshold_enters_knob_mode() {
        let mut g = Gesture::new(100.);
        g.moved(96.);
        assert_eq!(g.phase(), Phase::Knob);
        assert_eq!(g.steps(), 0);
        assert_eq!(g.released(96.), Outcome::Cancel);
    }

    #[test]
    fn one_step_per_six_pixels_up_is_positive() {
        let mut g = Gesture::new(100.);
        g.moved(94.);
        assert_eq!(g.steps(), 1);
        g.moved(58.);
        assert_eq!(g.steps(), 7);
        assert_eq!(g.released(58.), Outcome::Commit(7));
    }

    #[test]
    fn dragging_down_decreases_and_truncates_toward_zero() {
        let mut g = Gesture::new(100.);
        g.moved(113.);
        assert_eq!(g.steps(), -2);
        g.moved(105.);
        assert_eq!(g.steps(), 0);
        assert_eq!(g.phase(), Phase::Knob);
    }

    #[test]
    fn knob_has_no_vertical_limit() {
        let mut g = Gesture::new(100.);
        g.moved(-500.);
        assert_eq!(g.released(-500.), Outcome::Commit(100));
    }

    #[test]
    fn circle_click_presses_the_timer_and_other_outcomes_edit_the_preset() {
        use Action::*;
        assert_eq!(resolve(Source::Circle, Outcome::Click, 15), PressTimer);
        assert_eq!(
            resolve(Source::Circle, Outcome::Commit(5), 15),
            SetValue(20)
        );
        assert_eq!(
            resolve(Source::Circle, Outcome::Commit(-3), 15),
            SetValue(12)
        );
        assert_eq!(resolve(Source::Circle, Outcome::Cancel, 15), Nothing);
    }

    #[test]
    fn edit_field_click_never_starts_the_timer() {
        for source in [Source::EditExisting, Source::EditNew] {
            assert_eq!(resolve(source, Outcome::Click, 15), Action::Nothing);
            assert_eq!(resolve(source, Outcome::Cancel, 15), Action::Nothing);
        }
    }

    #[test]
    fn knob_in_the_field_changes_the_typed_number() {
        assert_eq!(
            resolve(Source::EditExisting, Outcome::Commit(4), 30),
            Action::SetValue(34)
        );
        assert_eq!(
            resolve(Source::EditNew, Outcome::Commit(2), 0),
            Action::SetValue(2)
        );
    }

    #[test]
    fn plus_click_opens_the_field_and_drag_creates_a_preset() {
        assert_eq!(resolve(Source::Plus, Outcome::Click, 0), Action::BeginNew);
        assert_eq!(
            resolve(Source::Plus, Outcome::Commit(7), 0),
            Action::SetValue(7)
        );
        assert_eq!(resolve(Source::Plus, Outcome::Cancel, 0), Action::Nothing);
    }
}
