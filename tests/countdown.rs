//! The countdown driven through the public API with a controllable clock.

#![expect(
    clippy::expect_used,
    reason = "test helpers state their preconditions with expect"
)]

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use countdown_timer_bar::domain::countdown::{Clock, Countdown, Event, Phase};
use countdown_timer_bar::domain::preset::{IdGen, Preset, PresetKind, PresetList};
use countdown_timer_bar::domain::preset_list::parse_list;

#[derive(Clone)]
struct ManualClock(Rc<Cell<Instant>>);

impl ManualClock {
    fn new() -> Self {
        Self(Rc::new(Cell::new(Instant::now())))
    }

    fn advance(&self, by: Duration) {
        self.0.set(self.0.get() + by);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        self.0.get()
    }
}

fn preset(text: &str, index: usize) -> Preset {
    let specs = parse_list(text).expect("test input is valid");
    PresetList::new(specs, &mut IdGen::default())
        .preset(PresetKind::Focus, index)
        .expect("slot exists")
}

#[test]
fn a_typed_preset_counts_down_and_finishes_once() {
    let clock = ManualClock::new();
    let mut countdown = Countdown::new(clock.clone());
    let preset = preset("90s", 0);

    countdown.start(preset);
    assert_eq!(countdown.remaining(), 90);

    clock.advance(Duration::from_secs(90));
    assert_eq!(countdown.tick(), Some(Event::Finished(preset)));
    assert_eq!(countdown.tick(), None);
}

#[test]
fn a_finished_countdown_is_idle_again() {
    let clock = ManualClock::new();
    let mut countdown = Countdown::new(clock.clone());
    countdown.start(preset("1", 0));

    clock.advance(Duration::from_secs(60));
    countdown.tick();

    assert_eq!(countdown.phase(), Phase::Idle);
}

#[test]
fn pressing_the_running_preset_pauses_and_resumes_it() {
    let clock = ManualClock::new();
    let mut countdown = Countdown::new(clock.clone());
    let preset = preset("10", 0);

    countdown.press(preset);
    clock.advance(Duration::from_secs(60));
    assert_eq!(countdown.press(preset), Some(Event::Paused(preset)));
    clock.advance(Duration::from_secs(3600));
    assert_eq!(countdown.press(preset), Some(Event::Resumed(preset)));

    assert_eq!(countdown.remaining(), 540);
}

#[test]
fn twin_presets_with_the_same_value_are_different_timers() {
    let clock = ManualClock::new();
    let mut countdown = Countdown::new(clock);
    let specs = parse_list("5, 5").expect("valid");
    let list = PresetList::new(specs, &mut IdGen::default());
    let (first, second) = (
        list.preset(PresetKind::Focus, 0).expect("slot 0"),
        list.preset(PresetKind::Focus, 1).expect("slot 1"),
    );

    countdown.press(first);

    assert_eq!(countdown.press(second), Some(Event::Started(second)));
}
