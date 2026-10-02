use super::preset::Preset;

/// Time source, injectable for tests. `SystemClock` uses `Instant`, which is
/// monotonic and, on macOS, keeps counting across sleep.
pub trait Clock {
    fn now(&self) -> std::time::Instant;
}

impl Clock for std::rc::Rc<dyn Clock> {
    fn now(&self) -> std::time::Instant {
        (**self).now()
    }
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Started(Preset),
    Paused(Preset),
    Resumed(Preset),
    Stopped,
    Finished(Preset),
}

#[derive(Clone, Copy)]
enum State {
    Idle,
    Running {
        deadline: std::time::Instant,
        preset: Preset,
    },
    Paused {
        remaining: std::time::Duration,
        preset: Preset,
    },
}

/// A finished timer is `Idle` again; the `Finished` event is its only trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Running,
    Paused,
}

fn ceil_seconds(remaining: std::time::Duration) -> u32 {
    let seconds = remaining.as_secs() + u64::from(remaining.subsec_millis() > 0);
    u32::try_from(seconds).unwrap_or(u32::MAX)
}

/// Remaining time is always `deadline - now`, never a decremented counter, so
/// sleep or missed ticks cannot make it drift.
pub struct Countdown<C: Clock> {
    clock: C,
    state: State,
}

impl<C: Clock> Countdown<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            state: State::Idle,
        }
    }

    /// Starts `preset` with a fresh deadline, replacing whatever ran.
    pub fn start(&mut self, preset: Preset) -> Event {
        let deadline =
            self.clock.now() + std::time::Duration::from_secs(u64::from(preset.duration.as_secs()));
        self.state = State::Running { deadline, preset };
        Event::Started(preset)
    }

    pub fn stop(&mut self) -> Option<Event> {
        if let State::Idle = self.state {
            None
        } else {
            self.state = State::Idle;
            Some(Event::Stopped)
        }
    }

    /// Tap on a preset: starts it, or pauses / resumes it if it is the active one.
    pub fn press(&mut self, preset: Preset) -> Option<Event> {
        match self.active_preset() {
            Some(active) if active.id == preset.id => {
                if self.phase() == Phase::Paused {
                    self.resume()
                } else {
                    self.pause()
                }
            }
            _ => Some(self.start(preset)),
        }
    }

    pub fn pause(&mut self) -> Option<Event> {
        match self.state {
            State::Running { deadline, preset } => {
                let remaining = deadline.saturating_duration_since(self.clock.now());
                self.state = State::Paused { remaining, preset };
                Some(Event::Paused(preset))
            }
            _ => None,
        }
    }

    pub fn resume(&mut self) -> Option<Event> {
        match self.state {
            State::Paused { remaining, preset } => {
                self.state = State::Running {
                    deadline: self.clock.now() + remaining,
                    preset,
                };
                Some(Event::Resumed(preset))
            }
            _ => None,
        }
    }

    /// Fires `Finished` once when the deadline passes, then returns to `Idle`.
    pub fn tick(&mut self) -> Option<Event> {
        match self.state {
            State::Running { deadline, preset } => {
                if self.clock.now() >= deadline {
                    self.state = State::Idle;
                    Some(Event::Finished(preset))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Whole seconds left, rounded up so a preset reads its full length until a
    /// whole second has elapsed.
    pub fn remaining(&self) -> u32 {
        match self.state {
            State::Running { deadline, .. } => {
                ceil_seconds(deadline.saturating_duration_since(self.clock.now()))
            }
            State::Paused { remaining, .. } => ceil_seconds(remaining),
            State::Idle => 0,
        }
    }

    /// Share of the active timer left, `1.0` down to `0.0`; `0.0` when idle.
    #[expect(
        clippy::cast_precision_loss,
        reason = "durations are far below 2^24 seconds, so f32 holds them exactly"
    )]
    pub fn remaining_fraction(&self) -> f32 {
        let (remaining, preset) = match self.state {
            State::Running { deadline, preset } => {
                (deadline.saturating_duration_since(self.clock.now()), preset)
            }
            State::Paused { remaining, preset } => (remaining, preset),
            State::Idle => return 0.0,
        };
        let total = preset.duration.as_secs() as f32;
        (remaining.as_secs_f32() / total).clamp(0.0, 1.0)
    }

    pub fn phase(&self) -> Phase {
        match self.state {
            State::Idle => Phase::Idle,
            State::Running { .. } => Phase::Running,
            State::Paused { .. } => Phase::Paused,
        }
    }

    pub fn active_preset(&self) -> Option<Preset> {
        match self.state {
            State::Running { preset, .. } | State::Paused { preset, .. } => Some(preset),
            State::Idle => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::duration::DurationSeconds;
    use super::super::preset::{Preset, PresetId, PresetKind};
    use super::*;

    #[derive(Clone)]
    struct FakeClock {
        now: std::rc::Rc<std::cell::Cell<std::time::Instant>>,
    }

    impl Clock for FakeClock {
        fn now(&self) -> std::time::Instant {
            self.now.get()
        }
    }

    impl FakeClock {
        fn new() -> Self {
            Self {
                now: std::rc::Rc::new(std::cell::Cell::new(std::time::Instant::now())),
            }
        }
        fn advance(&self, by: std::time::Duration) {
            self.now.set(self.now.get() + by);
        }
    }

    /// Each distinct `seconds` is a distinct preset slot (id = seconds).
    fn focus_preset(seconds: u32) -> Preset {
        Preset {
            kind: PresetKind::Focus,
            duration: DurationSeconds::new(seconds).unwrap(),
            id: PresetId::new(u64::from(seconds)),
        }
    }

    #[test]
    fn start_sets_remaining_and_running() {
        let mut cd = Countdown::new(FakeClock::new());
        let event = cd.start(focus_preset(10));
        assert_eq!(event, Event::Started(focus_preset(10)));
        assert_eq!(cd.remaining(), 10);
        assert_eq!(cd.phase(), Phase::Running);
        assert_eq!(cd.active_preset(), Some(focus_preset(10)));
    }

    #[test]
    fn remaining_rounds_up_instead_of_truncating_just_after_start() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(900));
        clock.advance(std::time::Duration::from_millis(10));
        assert_eq!(cd.remaining(), 900);
    }

    #[test]
    fn tick_before_deadline_counts_down_and_keeps_running() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(4));
        assert_eq!(cd.tick(), None);
        assert_eq!(cd.remaining(), 6);
        assert_eq!(cd.phase(), Phase::Running);
    }

    #[test]
    fn tick_at_deadline_fires_finished_exactly_once() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(10));
        assert_eq!(cd.tick(), Some(Event::Finished(focus_preset(10))));
        assert_eq!(cd.remaining(), 0);
        assert_ne!(cd.phase(), Phase::Running);
        assert_eq!(cd.tick(), None);
    }

    #[test]
    fn large_clock_jump_past_deadline_fires_finished_once_without_panicking() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(600));
        assert_eq!(cd.tick(), Some(Event::Finished(focus_preset(10))));
        assert_eq!(cd.remaining(), 0);
    }

    #[test]
    fn remaining_fraction_runs_from_one_down_to_a_quarter() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        assert!((cd.remaining_fraction() - 1.).abs() < 1e-6);
        clock.advance(std::time::Duration::from_secs(5));
        assert!((cd.remaining_fraction() - 0.5).abs() < 1e-6);
        clock.advance(std::time::Duration::from_millis(2500));
        assert!((cd.remaining_fraction() - 0.25).abs() < 1e-6);
    }

    #[test]
    fn remaining_fraction_is_zero_when_idle() {
        let cd = Countdown::new(FakeClock::new());
        assert!(cd.remaining_fraction().abs() < 1e-6);
    }

    #[test]
    fn remaining_fraction_is_clamped_past_the_deadline() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(60));
        assert!(cd.remaining_fraction().abs() < 1e-6);
    }

    #[test]
    fn remaining_fraction_is_frozen_while_paused_and_resumes_from_there() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(4));
        cd.pause();
        clock.advance(std::time::Duration::from_secs(100));
        assert!((cd.remaining_fraction() - 0.6).abs() < 1e-6);
        cd.resume();
        clock.advance(std::time::Duration::from_secs(3));
        assert!((cd.remaining_fraction() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn remaining_fraction_is_zero_after_stop_and_after_finish() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        cd.stop();
        assert!(cd.remaining_fraction().abs() < 1e-6);
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(10));
        cd.tick();
        assert!(cd.remaining_fraction().abs() < 1e-6);
    }

    #[test]
    fn pause_emits_paused_and_keeps_the_preset() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(4));
        assert_eq!(cd.pause(), Some(Event::Paused(focus_preset(10))));
        assert_eq!(cd.phase(), Phase::Paused);
        assert_eq!(cd.active_preset(), Some(focus_preset(10)));
    }

    #[test]
    fn pause_freezes_the_remaining_time() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(4));
        cd.pause();
        clock.advance(std::time::Duration::from_secs(600));
        assert_eq!(cd.remaining(), 6);
    }

    #[test]
    fn a_paused_countdown_never_finishes() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        cd.pause();
        clock.advance(std::time::Duration::from_secs(600));
        assert_eq!(cd.tick(), None);
    }

    #[test]
    fn resume_continues_from_the_frozen_remaining() {
        let clock = FakeClock::new();
        let mut cd = Countdown::new(clock.clone());
        cd.start(focus_preset(10));
        clock.advance(std::time::Duration::from_secs(4));
        cd.pause();
        clock.advance(std::time::Duration::from_secs(100));
        assert_eq!(cd.resume(), Some(Event::Resumed(focus_preset(10))));
        assert_eq!(cd.phase(), Phase::Running);
        assert_eq!(cd.remaining(), 6);
        clock.advance(std::time::Duration::from_secs(6));
        assert_eq!(cd.tick(), Some(Event::Finished(focus_preset(10))));
    }

    #[test]
    fn pause_and_resume_are_noops_in_the_wrong_state() {
        let mut cd = Countdown::new(FakeClock::new());
        assert_eq!(cd.pause(), None);
        assert_eq!(cd.resume(), None);
        cd.start(focus_preset(10));
        assert_eq!(cd.resume(), None);
        cd.pause();
        assert_eq!(cd.pause(), None);
    }

    #[test]
    fn press_starts_pauses_and_resumes_the_same_preset() {
        let mut cd = Countdown::new(FakeClock::new());
        assert_eq!(
            cd.press(focus_preset(10)),
            Some(Event::Started(focus_preset(10)))
        );
        assert_eq!(
            cd.press(focus_preset(10)),
            Some(Event::Paused(focus_preset(10)))
        );
        assert_eq!(
            cd.press(focus_preset(10)),
            Some(Event::Resumed(focus_preset(10)))
        );
        assert_eq!(cd.phase(), Phase::Running);
    }

    #[test]
    fn a_twin_with_the_same_duration_is_a_different_timer() {
        let twin = Preset {
            id: PresetId::new(999),
            ..focus_preset(10)
        };
        let mut cd = Countdown::new(FakeClock::new());
        cd.press(focus_preset(10));
        assert_eq!(cd.press(twin), Some(Event::Started(twin)));
        assert_eq!(cd.active_preset(), Some(twin));
        assert_eq!(cd.phase(), Phase::Running);
    }

    #[test]
    fn press_on_another_preset_replaces_a_running_or_paused_one() {
        let mut cd = Countdown::new(FakeClock::new());
        cd.press(focus_preset(10));
        assert_eq!(
            cd.press(focus_preset(20)),
            Some(Event::Started(focus_preset(20)))
        );
        cd.pause();
        assert_eq!(
            cd.press(focus_preset(30)),
            Some(Event::Started(focus_preset(30)))
        );
        assert_eq!(cd.phase(), Phase::Running);
        assert_eq!(cd.active_preset(), Some(focus_preset(30)));
    }

    #[test]
    fn stop_from_paused_goes_idle() {
        let mut cd = Countdown::new(FakeClock::new());
        cd.start(focus_preset(10));
        cd.pause();
        assert_eq!(cd.stop(), Some(Event::Stopped));
        assert_ne!(cd.phase(), Phase::Paused);
        assert_eq!(cd.remaining(), 0);
        assert_eq!(cd.active_preset(), None);
    }

    #[test]
    fn starting_another_preset_while_paused_replaces_it() {
        let mut cd = Countdown::new(FakeClock::new());
        cd.start(focus_preset(10));
        cd.pause();
        cd.start(focus_preset(20));
        assert_eq!(cd.phase(), Phase::Running);
        assert_eq!(cd.remaining(), 20);
    }

    #[test]
    fn stop_resets_to_idle() {
        let mut cd = Countdown::new(FakeClock::new());
        cd.start(focus_preset(10));
        assert_eq!(cd.stop(), Some(Event::Stopped));
        assert_eq!(cd.remaining(), 0);
        assert_ne!(cd.phase(), Phase::Running);
        assert_eq!(cd.active_preset(), None);
    }

    #[test]
    fn stop_when_already_idle_returns_none() {
        let mut cd = Countdown::new(FakeClock::new());
        assert_eq!(cd.stop(), None);
    }

    #[test]
    fn starting_a_new_preset_while_running_replaces_the_old_one() {
        let mut cd = Countdown::new(FakeClock::new());
        cd.start(focus_preset(10));
        let event = cd.start(focus_preset(20));
        assert_eq!(event, Event::Started(focus_preset(20)));
        assert_eq!(cd.remaining(), 20);
        assert_eq!(cd.active_preset(), Some(focus_preset(20)));
    }
}
