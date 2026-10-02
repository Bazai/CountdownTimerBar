use super::duration::DurationSeconds;
use super::preset_list::{PresetSpec, MAX_PRESETS};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PresetKind {
    Focus,
    Rest,
}

/// Stable identity of one preset slot. Two presets with the same kind and
/// duration are still different timers, so the running timer, the highlight
/// and the "cannot edit the active one" rule all go by id, never by value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PresetId(u64);

impl PresetId {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Preset {
    pub kind: PresetKind,
    pub duration: DurationSeconds,
    pub id: PresetId,
}

/// Source of fresh [`PresetId`]s; ids are unique across both columns.
#[derive(Debug, Default)]
pub struct IdGen(u64);

impl IdGen {
    pub fn next_id(&mut self) -> PresetId {
        self.0 += 1;
        PresetId(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListFull;

/// One column of presets. Specs and ids stay in lockstep and the size limit is
/// enforced here.
#[derive(Debug, Clone, Default)]
pub struct PresetList {
    specs: Vec<PresetSpec>,
    ids: Vec<PresetId>,
}

impl PresetList {
    /// A list of `specs` with fresh ids; entries past [`MAX_PRESETS`] are dropped.
    pub fn new(specs: impl IntoIterator<Item = PresetSpec>, ids: &mut IdGen) -> Self {
        let specs: Vec<_> = specs.into_iter().take(MAX_PRESETS).collect();
        let ids = specs.iter().map(|_| ids.next_id()).collect();
        Self { specs, ids }
    }

    pub fn specs(&self) -> &[PresetSpec] {
        &self.specs
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.specs.len() >= MAX_PRESETS
    }

    pub fn preset(&self, kind: PresetKind, index: usize) -> Option<Preset> {
        Some(Preset {
            kind,
            duration: self.specs.get(index)?.to_duration(),
            id: *self.ids.get(index)?,
        })
    }

    pub fn index_of(&self, id: PresetId) -> Option<usize> {
        self.ids.iter().position(|&slot| slot == id)
    }

    /// Replaces slot `index` in place, keeping its id; `false` if there is none.
    pub fn replace_at(&mut self, index: usize, spec: PresetSpec) -> bool {
        self.specs.get_mut(index).map(|slot| *slot = spec).is_some()
    }

    /// Removes slot `index`; `false` if there is none.
    pub fn remove_at(&mut self, index: usize) -> bool {
        if index < self.specs.len() {
            self.specs.remove(index);
            self.ids.remove(index);
            true
        } else {
            false
        }
    }

    /// # Errors
    ///
    /// [`ListFull`] when the list already has [`MAX_PRESETS`] entries.
    pub fn push(&mut self, spec: PresetSpec, ids: &mut IdGen) -> Result<(), ListFull> {
        if self.is_full() {
            return Err(ListFull);
        }
        self.specs.push(spec);
        self.ids.push(ids.next_id());
        Ok(())
    }

    /// Replaces the whole list (text input). Ids carry over to the entries
    /// that stayed, so the running timer keeps its identity.
    pub fn replace_all(&mut self, specs: Vec<PresetSpec>, ids: &mut IdGen) {
        self.ids = reconcile_ids(&self.specs, &self.ids, &specs, ids);
        self.specs = specs;
    }

    pub fn durations(&self) -> Vec<DurationSeconds> {
        self.specs.iter().map(|spec| spec.to_duration()).collect()
    }
}

/// Ids for a replaced list: an entry keeps the id of the old entry with the same
/// value, preferring the same position; anything else gets a fresh id.
fn reconcile_ids(
    old_specs: &[PresetSpec],
    old_ids: &[PresetId],
    new_specs: &[PresetSpec],
    ids: &mut IdGen,
) -> Vec<PresetId> {
    let mut taken = vec![false; old_specs.len()];
    let mut carried: Vec<Option<PresetId>> = vec![None; new_specs.len()];
    for (i, spec) in new_specs.iter().enumerate() {
        if old_specs.get(i) == Some(spec) {
            taken[i] = true;
            carried[i] = Some(old_ids[i]);
        }
    }
    for (i, spec) in new_specs.iter().enumerate() {
        if carried[i].is_some() {
            continue;
        }
        let found = (0..old_specs.len()).find(|&j| !taken[j] && old_specs[j] == *spec);
        if let Some(j) = found {
            taken[j] = true;
            carried[i] = Some(old_ids[j]);
        }
    }
    carried
        .into_iter()
        .map(|id| id.unwrap_or_else(|| ids.next_id()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::preset_list::{PresetSpec, Unit};

    fn m(value: u32) -> PresetSpec {
        PresetSpec::new(value, Unit::Minutes).unwrap()
    }

    fn id(raw: u64) -> PresetId {
        PresetId::new(raw)
    }

    fn gen_at(last: u64) -> IdGen {
        IdGen(last)
    }

    fn preset(kind: PresetKind, seconds: u32, raw_id: u64) -> Preset {
        Preset {
            kind,
            duration: DurationSeconds::new(seconds).unwrap(),
            id: id(raw_id),
        }
    }

    #[test]
    fn same_kind_and_duration_with_different_ids_are_different_presets() {
        assert_ne!(
            preset(PresetKind::Focus, 300, 1),
            preset(PresetKind::Focus, 300, 2)
        );
        assert_eq!(
            preset(PresetKind::Focus, 300, 1),
            preset(PresetKind::Focus, 300, 1)
        );
    }

    #[test]
    fn presets_with_different_kinds_are_not_equal() {
        assert_ne!(
            preset(PresetKind::Focus, 300, 1),
            preset(PresetKind::Rest, 300, 1)
        );
    }

    #[test]
    fn unchanged_list_keeps_every_id() {
        let mut next = gen_at(10);
        let ids = reconcile_ids(
            &[m(5), m(5), m(7)],
            &[id(1), id(2), id(3)],
            &[m(5), m(5), m(7)],
            &mut next,
        );
        assert_eq!(ids, vec![id(1), id(2), id(3)]);
        assert_eq!(next.next_id(), id(11));
    }

    #[test]
    fn duplicates_keep_their_own_ids_when_a_neighbour_is_appended() {
        let mut next = gen_at(10);
        let ids = reconcile_ids(
            &[m(5), m(5)],
            &[id(1), id(2)],
            &[m(5), m(5), m(9)],
            &mut next,
        );
        assert_eq!(ids, vec![id(1), id(2), id(11)]);
    }

    #[test]
    fn a_value_that_moved_keeps_its_id() {
        let mut next = gen_at(10);
        let ids = reconcile_ids(&[m(5), m(7)], &[id(1), id(2)], &[m(7), m(5)], &mut next);
        assert_eq!(ids, vec![id(2), id(1)]);
    }

    #[test]
    fn changed_or_new_values_get_fresh_ids() {
        let mut next = gen_at(10);
        let ids = reconcile_ids(&[m(5), m(7)], &[id(1), id(2)], &[m(5), m(8)], &mut next);
        assert_eq!(ids, vec![id(1), id(11)]);
        assert_eq!(next.next_id(), id(12));
    }

    #[test]
    fn removed_items_free_their_ids_and_shifted_items_keep_theirs() {
        let mut next = gen_at(10);
        let ids = reconcile_ids(
            &[m(5), m(6), m(7)],
            &[id(1), id(2), id(3)],
            &[m(6), m(7)],
            &mut next,
        );
        assert_eq!(ids, vec![id(2), id(3)]);
    }

    #[test]
    fn empty_lists_reconcile_to_empty() {
        let mut next = gen_at(0);
        assert!(reconcile_ids(&[], &[], &[], &mut next).is_empty());
        assert_eq!(reconcile_ids(&[m(1)], &[id(1)], &[], &mut next), vec![]);
    }

    fn list(values: &[u32], ids: &mut IdGen) -> PresetList {
        PresetList::new(values.iter().map(|&value| m(value)), ids)
    }

    #[test]
    fn push_stops_at_the_column_limit() {
        let mut ids = IdGen::default();
        let mut list = list(&[1, 2, 3, 4, 5, 6, 7], &mut ids);
        assert_eq!(list.push(m(8), &mut ids), Ok(()));
        assert_eq!(list.push(m(9), &mut ids), Err(ListFull));
        assert_eq!(list.len(), MAX_PRESETS);
    }

    #[test]
    fn replace_at_keeps_the_slot_identity() {
        let mut ids = IdGen::default();
        let mut list = list(&[5, 7], &mut ids);
        let before = list.preset(PresetKind::Focus, 1).unwrap().id;
        assert!(list.replace_at(1, m(9)));
        assert_eq!(list.preset(PresetKind::Focus, 1).unwrap().id, before);
    }

    #[test]
    fn remove_at_shifts_later_slots_but_not_their_ids() {
        let mut ids = IdGen::default();
        let mut list = list(&[5, 6, 7], &mut ids);
        let third = list.preset(PresetKind::Focus, 2).unwrap().id;
        assert!(list.remove_at(0));
        assert_eq!(list.index_of(third), Some(1));
    }

    #[test]
    fn out_of_range_edits_report_failure() {
        let mut ids = IdGen::default();
        let mut list = list(&[5], &mut ids);
        assert!(!list.replace_at(3, m(9)));
        assert!(!list.remove_at(3));
    }

    #[test]
    fn new_drops_entries_past_the_limit() {
        let mut ids = IdGen::default();
        let list = list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], &mut ids);
        assert_eq!(list.len(), MAX_PRESETS);
    }

    #[test]
    fn replace_all_keeps_the_ids_of_unchanged_entries() {
        let mut ids = IdGen::default();
        let mut list = list(&[5, 7], &mut ids);
        let first = list.preset(PresetKind::Focus, 0).unwrap().id;
        list.replace_all(vec![m(5), m(8)], &mut ids);
        assert_eq!(list.index_of(first), Some(0));
    }
}
