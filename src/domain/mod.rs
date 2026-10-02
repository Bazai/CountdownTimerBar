//! The pure model of the app: no UI and no platform code.
//!
//! - [`countdown`]: the countdown state machine over an injectable [`countdown::Clock`].
//! - [`duration`]: positive durations and clock text.
//! - [`preset`]: preset kinds, stable ids and the reconciliation of ids when a list is replaced.
//! - [`preset_list`]: editable presets (number plus unit) and their text format.
//! - [`settings`]: the persisted settings over a [`settings::KeyValueStore`].

pub mod countdown;
pub mod duration;
pub mod preset;
pub mod preset_list;
pub mod settings;
