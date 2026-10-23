// ██████╗ ██████╗ ███████╗███████╗███████╗████████╗    ██╗     ██╗███╗   ██╗██╗  ██╗███████╗
// ██╔══██╗██╔══██╗██╔════╝██╔════╝██╔════╝╚══██╔══╝    ██║     ██║████╗  ██║██║ ██╔╝██╔════╝
// ██████╔╝██████╔╝█████╗  ███████╗█████╗     ██║       ██║     ██║██╔██╗ ██║█████╔╝ ███████╗
// ██╔═══╝ ██╔══██╗██╔══╝  ╚════██║██╔══╝     ██║       ██║     ██║██║╚██╗██║██╔═██╗ ╚════██║
// ██║     ██║  ██║███████╗███████║███████╗   ██║       ███████╗██║██║ ╚████║██║  ██╗███████║
// ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝   ╚═╝       ╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝
// FILE: preset_links

//! Presets linked into cues (decisions 0293): a cue value that came from a preset holds only the
//! preset's GUID (`CueData::Link`) and plays the preset as it is now — the fixture's own entry,
//! else the preset's universal one; a preset without that value gives none. This file looks the
//! links up: `Preset::value_for`, the show's `PresetValues`, the plan compiler's per-compile
//! `PresetIndex`, and the preset → cue back-references (`PresetUses`) behind "N cues follow",
//! the Update ask, Recast, the delete ask and a preset's use count.
//!
//! Owns:          Preset::value_for, ShowData::preset_by_guid / preset_number, PresetIndex,
//!                PresetUses, CueUse.
//! Doesn't own:   the link itself (lumacore-playback `CueData`), the words (Update, Recast,
//!                Extract in lumacore-objects), keeping the uses current (AppState).
//! Entry points:  PresetIndex::new, PresetUses::build, ShowData::preset_by_guid.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use lumacore_base::Value;
use lumacore_playback::{CueData, PresetValues};
use uuid::Uuid;

use crate::datapool::Preset;
use crate::tree::ShowData;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The plan compiler's preset lookup for one compile: every preset by GUID, and each linked
/// preset's entries keyed by (fixture, attribute) the first time a link asks for it — a link is
/// one hash lookup, never a scan of the preset.
pub struct PresetIndex<'a> {
    by_guid: HashMap<Uuid, &'a Preset>,
    entries: RefCell<HashMap<Uuid, Entries<'a>>>,
}

/// One preset's values: (fixture, attribute) → value, and its universal values by attribute.
#[derive(Default)]
struct Entries<'a> {
    own: HashMap<(Uuid, &'a str), &'a Value>,
    universal: HashMap<&'a str, &'a Value>,
}

/// One cue that links to a preset: where it is and how many of its values do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueUse {
    /// The sequence's DataPool, number and GUID.
    pub datapool: u32,
    pub sequence: u32,
    pub sequence_guid: Uuid,
    /// The cue's key (`Cue 1.5` = 1500; CueZero = 0).
    pub cue: u32,
    /// The cue's name (empty when it has none).
    pub name: String,
    /// How many of its values link to the preset.
    pub values: usize,
    /// The attributes it links (each once, in stored order).
    pub attributes: Vec<String>,
}

/// Which cues link to each preset (decisions 0293): built from the show, kept by the state
/// (rebuilt when the show changed since).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PresetUses {
    by_preset: BTreeMap<Uuid, Vec<CueUse>>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Preset {
    /// The value this preset gives `fixture`'s `attribute`: the fixture's own entry, else the
    /// universal one (no fixture); None when it holds neither.
    pub fn value_for(&self, fixture: Uuid, attribute: &str) -> Option<&Value> {
        let mut universal = None;
        for e in &self.values {
            if !e.attribute.eq_ignore_ascii_case(attribute) {
                continue;
            }
            match e.fixture_id {
                Some(f) if f == fixture => return Some(&e.value),
                None if universal.is_none() => universal = Some(&e.value),
                _ => {}
            }
        }
        universal
    }
}

impl ShowData {
    /// The preset with this GUID, in whichever DataPool it is: (datapool, number, preset).
    pub fn preset_by_guid(&self, guid: Uuid) -> Option<(u32, u32, &Preset)> {
        self.datapools
            .iter()
            .find_map(|(&dp, d)| d.presets.iter().find(|(_, p)| p.meta.identity.guid == guid).map(|(&n, p)| (dp, n, p)))
    }

    /// A preset's number in its pool (a link's readout: `P3`).
    pub fn preset_number(&self, guid: Uuid) -> Option<u32> {
        self.preset_by_guid(guid).map(|(_, n, _)| n)
    }
}

/// The show resolves a link by looking the preset up (commands and views; the compiler uses
/// [`PresetIndex`]).
impl PresetValues for ShowData {
    fn preset_value(&self, preset: Uuid, fixture: Uuid, attribute: &str) -> Option<&Value> {
        self.preset_by_guid(preset)?.2.value_for(fixture, attribute)
    }
}

impl<'a> PresetIndex<'a> {
    pub fn new(show: &'a ShowData) -> Self {
        let by_guid = show.datapools.values().flat_map(|d| d.presets.values()).map(|p| (p.meta.identity.guid, p)).collect();
        Self { by_guid, entries: RefCell::new(HashMap::new()) }
    }

    /// The value a link plays (the fixture's own entry, else the universal one).
    pub fn value(&self, preset: Uuid, fixture: Uuid, attribute: &str) -> Option<&'a Value> {
        let p = *self.by_guid.get(&preset)?;
        let mut entries = self.entries.borrow_mut();
        let e = entries.entry(preset).or_insert_with(|| {
            let mut e = Entries::default();
            for v in &p.values {
                match v.fixture_id {
                    Some(f) => {
                        e.own.entry((f, v.attribute.as_str())).or_insert(&v.value);
                    }
                    None => {
                        e.universal.entry(v.attribute.as_str()).or_insert(&v.value);
                    }
                }
            }
            e
        });
        e.own.get(&(fixture, attribute)).or_else(|| e.universal.get(attribute)).copied()
    }

    /// What a cue value plays: its own steps, or the linked preset's value.
    pub fn resolve(&self, data: &'a CueData, fixture: Uuid, attribute: &str) -> Option<&'a Value> {
        match data {
            CueData::Value(v) => Some(v),
            CueData::Link(g) => self.value(*g, fixture, attribute),
        }
    }
}

impl PresetValues for PresetIndex<'_> {
    fn preset_value(&self, preset: Uuid, fixture: Uuid, attribute: &str) -> Option<&Value> {
        self.value(preset, fixture, attribute)
    }
}

impl PresetUses {
    /// Every cue link of the show, by preset (cues in DataPool, sequence and cue order).
    pub fn build(show: &ShowData) -> Self {
        let mut by_preset: BTreeMap<Uuid, Vec<CueUse>> = BTreeMap::new();
        for (dp, no, seq) in show.sequences() {
            let cues = std::iter::once((&lumacore_playback::CUE_ZERO, &seq.cue_zero)).chain(seq.cues.iter());
            for (&key, cue) in cues {
                for v in cue.values() {
                    let Some(g) = v.value.link() else { continue };
                    let uses = by_preset.entry(g).or_default();
                    let at = match uses.iter().position(|u| u.sequence_guid == seq.meta.identity.guid && u.cue == key) {
                        Some(i) => i,
                        None => {
                            uses.push(CueUse {
                                datapool: dp,
                                sequence: no,
                                sequence_guid: seq.meta.identity.guid,
                                cue: key,
                                name: cue.meta.name().unwrap_or_default().to_string(),
                                values: 0,
                                attributes: Vec::new(),
                            });
                            uses.len() - 1
                        }
                    };
                    let u = &mut uses[at];
                    u.values += 1;
                    if !u.attributes.contains(&v.attribute) {
                        u.attributes.push(v.attribute.clone());
                    }
                }
            }
        }
        Self { by_preset }
    }

    /// The cues that link to `preset` (none: empty).
    pub fn cues(&self, preset: Uuid) -> &[CueUse] {
        self.by_preset.get(&preset).map_or(&[], Vec::as_slice)
    }

    /// How many cues link to `preset`.
    pub fn count(&self, preset: Uuid) -> usize {
        self.cues(preset).len()
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datapool::{PresetEntry, PresetMode};
    use lumacore_base::NodeMeta;

    fn preset(values: Vec<(Option<Uuid>, &str, f64)>) -> Preset {
        Preset {
            meta: NodeMeta::new(None),
            mode: PresetMode::Selective,
            values: values.into_iter().map(|(f, a, v)| PresetEntry { fixture_id: f, attribute: a.into(), value: Value::normalized(v) }).collect(),
        }
    }

    #[test]
    fn a_link_reads_the_fixtures_own_value_else_the_universal_one() {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let p = preset(vec![(None, "Dimmer", 40.0), (Some(a), "Dimmer", 70.0)]);
        assert_eq!(p.value_for(a, "Dimmer").and_then(Value::percent), Some(70.0));
        assert_eq!(p.value_for(b, "Dimmer").and_then(Value::percent), Some(40.0));
        assert!(p.value_for(a, "Pan").is_none(), "a preset without the attribute gives no value");
    }
}
