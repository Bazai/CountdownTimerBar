use super::duration::DurationSeconds;
use super::preset::PresetKind;
use super::preset_list::PresetSpec;

const DEFAULT_FOCUS_SECONDS: [u32; 3] = [900, 1800, 2100];
const DEFAULT_REST_SECONDS: [u32; 3] = [60, 600, 1200];

pub const KEY_FOCUS_TIMERS: &str = "focusTimers";
pub const KEY_REST_TIMERS: &str = "restTimers";
pub const KEY_SOUND_ON: &str = "soundOn";

/// Storage backend, so settings are testable without `NSUserDefaults`.
pub trait KeyValueStore {
    fn get_string_array(&self, key: &str) -> Option<Vec<String>>;
    fn get_bool(&self, key: &str) -> bool;
    fn set_string_array(&mut self, key: &str, values: &[String]);
    fn set_bool(&mut self, key: &str, value: bool);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub focus: Vec<DurationSeconds>,
    pub rest: Vec<DurationSeconds>,
    pub sound_on: bool,
}

impl Settings {
    pub fn defaults() -> Self {
        Self {
            focus: seconds_list(&DEFAULT_FOCUS_SECONDS),
            rest: seconds_list(&DEFAULT_REST_SECONDS),
            sound_on: false,
        }
    }

    pub fn presets(&self, kind: PresetKind) -> &[DurationSeconds] {
        match kind {
            PresetKind::Focus => &self.focus,
            PresetKind::Rest => &self.rest,
        }
    }

    /// The column as editable specs; the unit is recovered from stored seconds.
    pub fn preset_specs(&self, kind: PresetKind) -> Vec<PresetSpec> {
        self.presets(kind)
            .iter()
            .copied()
            .map(PresetSpec::from_duration)
            .collect()
    }
}

fn seconds_list(values: &[u32]) -> Vec<DurationSeconds> {
    values
        .iter()
        .filter_map(|&v| DurationSeconds::new(v))
        .collect()
}

/// Reads a column: unparsable entries are dropped, a missing key or an
/// all-invalid array falls back to `default`, and a stored empty array stays
/// empty.
fn load_preset_list(
    store: &(impl KeyValueStore + ?Sized),
    key: &str,
    default: Vec<DurationSeconds>,
) -> Vec<DurationSeconds> {
    let Some(raw) = store.get_string_array(key) else {
        return default;
    };
    if raw.is_empty() {
        return Vec::new();
    }

    let values: Vec<_> = raw
        .iter()
        .filter_map(|s| s.parse::<u32>().ok())
        .filter_map(DurationSeconds::new)
        .collect();

    if values.is_empty() {
        default
    } else {
        values
    }
}

/// Loads settings, falling back to the defaults per column.
pub fn load_settings(store: &(impl KeyValueStore + ?Sized)) -> Settings {
    let defaults = Settings::defaults();
    Settings {
        focus: load_preset_list(store, KEY_FOCUS_TIMERS, defaults.focus),
        rest: load_preset_list(store, KEY_REST_TIMERS, defaults.rest),
        sound_on: store.get_bool(KEY_SOUND_ON),
    }
}

pub fn save_presets(
    store: &mut (impl KeyValueStore + ?Sized),
    kind: PresetKind,
    values: &[DurationSeconds],
) {
    let key = match kind {
        PresetKind::Focus => KEY_FOCUS_TIMERS,
        PresetKind::Rest => KEY_REST_TIMERS,
    };
    let strings: Vec<String> = values.iter().map(|d| d.as_secs().to_string()).collect();
    store.set_string_array(key, &strings);
}

pub fn save_sound_on(store: &mut (impl KeyValueStore + ?Sized), value: bool) {
    store.set_bool(KEY_SOUND_ON, value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FakeStore {
        strings: HashMap<String, Vec<String>>,
        bools: HashMap<String, bool>,
    }

    impl FakeStore {
        fn empty() -> Self {
            Self {
                strings: HashMap::new(),
                bools: HashMap::new(),
            }
        }
    }

    impl KeyValueStore for FakeStore {
        fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
            self.strings.get(key).cloned()
        }

        fn get_bool(&self, key: &str) -> bool {
            self.bools.get(key).copied().unwrap_or(false)
        }

        fn set_string_array(&mut self, key: &str, values: &[String]) {
            self.strings.insert(key.to_string(), values.to_vec());
        }

        fn set_bool(&mut self, key: &str, value: bool) {
            self.bools.insert(key.to_string(), value);
        }
    }

    #[test]
    fn defaults_match_spec_initial_values() {
        let settings = Settings::defaults();
        assert_eq!(
            settings
                .focus
                .iter()
                .map(|d| d.as_secs())
                .collect::<Vec<_>>(),
            vec![900, 1800, 2100]
        );
        assert_eq!(
            settings
                .rest
                .iter()
                .map(|d| d.as_secs())
                .collect::<Vec<_>>(),
            vec![60, 600, 1200]
        );
        assert!(!settings.sound_on);
    }

    #[test]
    fn load_settings_falls_back_to_defaults_when_store_is_empty() {
        let store = FakeStore::empty();
        let settings = load_settings(&store);
        assert_eq!(settings.focus, Settings::defaults().focus);
    }

    #[test]
    fn load_settings_reads_existing_string_array_keys() {
        let mut store = FakeStore::empty();
        store.set_string_array(KEY_FOCUS_TIMERS, &["120".into(), "240".into()]);
        let settings = load_settings(&store);
        assert_eq!(
            settings
                .focus
                .iter()
                .map(|d| d.as_secs())
                .collect::<Vec<_>>(),
            vec![120, 240]
        );
    }

    #[test]
    fn a_preset_list_round_trips_through_the_store() {
        use super::super::preset::{IdGen, PresetList};
        use super::super::preset_list::{format_list, parse_list};
        let specs = parse_list("13s, 1, 10, 2").unwrap();
        let list = PresetList::new(specs, &mut IdGen::default());
        let mut store = FakeStore::empty();
        save_presets(&mut store, PresetKind::Rest, &list.durations());
        assert_eq!(
            store.get_string_array(KEY_REST_TIMERS),
            Some(vec!["13".into(), "60".into(), "600".into(), "120".into()])
        );
        let reloaded = load_settings(&store);
        assert_eq!(
            format_list(&reloaded.preset_specs(PresetKind::Rest)),
            "13s, 1, 10, 2"
        );
    }

    #[test]
    fn presets_returns_the_matching_list_for_each_kind() {
        let settings = Settings::defaults();
        assert_eq!(
            settings.presets(PresetKind::Focus),
            settings.focus.as_slice()
        );
        assert_eq!(settings.presets(PresetKind::Rest), settings.rest.as_slice());
    }

    #[test]
    fn save_presets_then_load_settings_round_trips_rest_list() {
        let mut store = FakeStore::empty();
        let values = vec![
            DurationSeconds::new(90).unwrap(),
            DurationSeconds::new(180).unwrap(),
        ];
        save_presets(&mut store, PresetKind::Rest, &values);
        let settings = load_settings(&store);
        assert_eq!(settings.rest, values);
        assert_eq!(settings.focus, Settings::defaults().focus);
    }

    #[test]
    fn save_sound_on_then_load_settings_round_trips_the_flag() {
        let mut store = FakeStore::empty();
        save_sound_on(&mut store, true);
        let settings = load_settings(&store);
        assert!(settings.sound_on);
    }

    #[test]
    fn load_settings_falls_back_when_stored_list_is_all_invalid() {
        let mut store = FakeStore::empty();
        store.set_string_array(KEY_FOCUS_TIMERS, &["abc".into(), "0".into()]);
        let settings = load_settings(&store);
        assert_eq!(settings.focus, Settings::defaults().focus);
    }

    #[test]
    fn load_settings_preserves_a_genuinely_empty_stored_list_instead_of_defaulting() {
        let mut store = FakeStore::empty();
        store.set_string_array(KEY_FOCUS_TIMERS, &[]);
        let settings = load_settings(&store);
        assert_eq!(settings.focus, Vec::new());
    }
}
