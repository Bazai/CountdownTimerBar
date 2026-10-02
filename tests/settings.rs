//! Settings persistence against an in-memory store.

use std::collections::HashMap;

use countdown_timer_bar::domain::preset::{IdGen, PresetKind, PresetList};
use countdown_timer_bar::domain::preset_list::{format_list, parse_list, PresetSpec};
use countdown_timer_bar::domain::settings::{
    load_settings, save_presets, save_sound_on, KeyValueStore,
};

#[derive(Default)]
struct MemoryStore {
    arrays: HashMap<String, Vec<String>>,
    flags: HashMap<String, bool>,
}

impl KeyValueStore for MemoryStore {
    fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
        self.arrays.get(key).cloned()
    }

    fn get_bool(&self, key: &str) -> bool {
        self.flags.get(key).copied().unwrap_or(false)
    }

    fn set_string_array(&mut self, key: &str, values: &[String]) {
        self.arrays.insert(key.to_owned(), values.to_vec());
    }

    fn set_bool(&mut self, key: &str, value: bool) {
        self.flags.insert(key.to_owned(), value);
    }
}

#[test]
fn an_empty_store_gives_the_default_columns() {
    let settings = load_settings(&MemoryStore::default());
    let focus = settings.preset_specs(PresetKind::Focus);
    assert_eq!(format_list(&focus), "15, 30, 35");
}

#[test]
fn a_saved_rest_column_is_loaded_back() {
    let mut store = MemoryStore::default();
    let list = PresetList::new(
        parse_list("13s, 1, 10").expect("valid"),
        &mut IdGen::default(),
    );
    save_presets(&mut store, PresetKind::Rest, &list.durations());

    let rest = load_settings(&store).preset_specs(PresetKind::Rest);

    assert_eq!(format_list(&rest), "13s, 1, 10");
}

#[test]
fn the_sound_flag_survives_a_round_trip() {
    let mut store = MemoryStore::default();
    save_sound_on(&mut store, true);
    assert!(load_settings(&store).sound_on);
}

#[test]
fn an_emptied_column_stays_empty() {
    let mut store = MemoryStore::default();
    save_presets(&mut store, PresetKind::Focus, &[]);

    let focus: Vec<PresetSpec> = load_settings(&store).preset_specs(PresetKind::Focus);

    assert!(focus.is_empty());
}
