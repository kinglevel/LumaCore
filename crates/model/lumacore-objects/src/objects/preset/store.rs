// ███████╗████████╗ ██████╗ ██████╗ ███████╗
// ██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
// ███████╗   ██║   ██║   ██║██████╔╝█████╗
// ╚════██║   ██║   ██║   ██║██╔══██╗██╔══╝
// ███████║   ██║   ╚██████╔╝██║  ██║███████╗
// ╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: store

//! `Store Preset [<n>] ["name"] [/Merge|/Overwrite|/Remove]` (decisions 0288): every active
//! value of the acting user's programmer, whatever its feature group (record 0150: presets stay
//! open). Universal when each attribute holds one value for all its fixtures, else Selective
//! (per fixture). Over an existing preset the values merge (`/Overwrite` replaces them all,
//! `/Remove` takes the programmer's attributes out). From a Presets window, the window's lit tags
//! are assigned to it.
//!
//! Owns:          StorePreset, the mode rule.
//! Doesn't own:   the pool (`preset`), the programmer (`programmers`).
//! Entry points:  StorePreset, mode_of.
//! Decisions:     docs/decisions/0288-one-preset-pool-follows-the-programmer, 0150.

use lumacore_tree::{Preset, PresetEntry, PresetMode};
use uuid::Uuid;

use crate::errors::render_code;
use crate::objects::out;
use crate::resolve::Slot;
use crate::text::render_success;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Preset")]
pub struct StorePreset {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
    #[lum(value)]
    pub name: Option<String>,
    #[lum(option = "Merge")]
    pub merge: bool,
    #[lum(option = "Overwrite")]
    pub overwrite: bool,
    #[lum(option = "Remove")]
    pub remove: bool,
}

impl crate::commands::typed::Command for StorePreset {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(store(state, self).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn store(state: &mut AppState, line: StorePreset) -> Result<String, String> {
    let (slot, datapool) = crate::resolve::slot(state, line.target.as_ref(), &super::PRESET)?;
    let dp = datapool.unwrap_or_else(|| state.selected_datapool());
    let pool = state.root.show_data.datapools.get(&dp).ok_or_else(|| render_code(format!("DataPool {dp} doesn't exist."), "E-CMD-007"))?;
    let next = || crate::pools::door::next_free(&pool.presets.keys().copied().collect::<Vec<_>>());
    let (no, name) = match slot {
        Slot::Number(0) => return Err(render_code("Preset numbers start at 1.", "E-CMD-008")),
        Slot::Number(n) => (n, line.name),
        Slot::NewName(n) => (next(), Some(n)),
        Slot::Next => (next(), line.name),
        Slot::Key(k) => return Err(render_code(format!("'{k}' isn't a preset number: Store Preset 12."), "E-CMD-008")),
    };
    let values = active_values(state);
    if values.is_empty() && !line.remove {
        return Err(render_code("Store Preset: your programmer has no active values.", "E-CMD-009"));
    }
    let tags = window_tags(state);
    // Every fixture's number, for the patch order of the stored values.
    let numbers: std::collections::HashMap<Uuid, u32> = {
        let show = &state.root.show_data;
        let stored = show.datapools.get(&dp).and_then(|d| d.presets.get(&no)).map(|p| p.fixtures().collect::<Vec<_>>()).unwrap_or_default();
        values.iter().map(|(f, ..)| *f).chain(stored).filter_map(|g| Some((g, show.fixture_id_for_guid(g)?))).collect()
    };
    let pool = &mut state.root.show_data.datapools.get_mut(&dp).expect("looked up above").presets;
    let existed = pool.contains_key(&no);
    let preset = pool.entry(no).or_insert_with(|| Preset { meta: lumacore_base::NodeMeta::new(None), mode: PresetMode::Selective, values: Vec::new() });
    if let Some(n) = name.filter(|n| !n.is_empty()) {
        preset.meta.identity.name = Some(n);
    }
    let attrs: Vec<String> = values.iter().map(|(_, a, _)| a.clone()).collect();
    let verb = if line.remove {
        preset.values.retain(|e| !attrs.contains(&e.attribute));
        "Removed from"
    } else {
        // Keep its own entries the new values don't name (Merge), or none (Overwrite).
        let mut all: Vec<(Option<Uuid>, String, lumacore_base::Value)> = if line.overwrite || !existed {
            Vec::new()
        } else {
            preset.values.iter().filter(|e| !attrs.contains(&e.attribute)).map(|e| (e.fixture_id, e.attribute.clone(), e.value.clone())).collect()
        };
        all.extend(values.into_iter().map(|(f, a, v)| (Some(f), a, v)));
        // In patch order: the fixtures by their number, then the attributes as they came.
        all.sort_by_key(|(f, ..)| f.and_then(|g| numbers.get(&g).copied()).unwrap_or(u32::MAX));
        let (mode, entries) = mode_of(all);
        preset.mode = mode;
        preset.values = entries;
        match (existed, line.overwrite) {
            (false, _) => "Stored",
            (true, true) => "Overwrote",
            (true, false) => "Merged into",
        }
    };
    for t in &tags {
        if !preset.meta.annotation.tags.contains(t) {
            preset.meta.annotation.tags.push(*t);
        }
    }
    let shown = preset.meta.name().map(|n| format!(" \"{n}\"")).unwrap_or_default();
    let mode = match preset.mode {
        PresetMode::Universal => "Universal",
        PresetMode::Global => "Global",
        PresetMode::Selective => "Selective",
    };
    let tagged = if tags.is_empty() { String::new() } else { format!(", {} tag{}", tags.len(), if tags.len() == 1 { "" } else { "s" }) };
    Ok(render_success(&format!("{verb} Preset {no}{shown}: {} values, {mode}{tagged}.", preset.values.len())))
}

/// The acting user's active programmer values, the preset link each came from dropped (a preset
/// holds values, not other presets, until embedding comes).
fn active_values(state: &AppState) -> Vec<(Uuid, String, lumacore_base::Value)> {
    let Some(prog) = state.current_programmer() else { return Vec::new() };
    prog.values
        .iter()
        .filter(|(_, pv)| pv.active)
        .map(|((f, a), pv)| {
            let mut v = pv.value.clone();
            v.preset = None;
            (*f, a.clone(), v)
        })
        .collect()
}

/// The mode rule: Universal when every attribute holds one value for all its fixtures (one entry
/// per attribute, no fixture), else Selective (an entry per fixture and attribute).
pub fn mode_of(values: Vec<(Option<Uuid>, String, lumacore_base::Value)>) -> (PresetMode, Vec<PresetEntry>) {
    let mut by_attr: Vec<(String, Vec<&lumacore_base::Value>)> = Vec::new();
    for (_, a, v) in &values {
        match by_attr.iter_mut().find(|(x, _)| x == a) {
            Some((_, vs)) => vs.push(v),
            None => by_attr.push((a.clone(), vec![v])),
        }
    }
    let per_fixture = values.iter().any(|(f, ..)| f.is_some());
    let uniform = by_attr.iter().all(|(_, vs)| vs.windows(2).all(|w| w[0] == w[1]));
    if per_fixture && uniform && values.len() > by_attr.len() {
        let entries = by_attr.into_iter().map(|(a, vs)| PresetEntry { fixture_id: None, attribute: a, value: vs[0].clone() }).collect();
        return (PresetMode::Universal, entries);
    }
    let entries = values.into_iter().map(|(f, a, v)| PresetEntry { fixture_id: f, attribute: a, value: v }).collect();
    (if per_fixture { PresetMode::Selective } else { PresetMode::Universal }, entries)
}

/// The lit tags of the Presets window the line came from (its `Tags` setting: tag GUIDs).
fn window_tags(state: &AppState) -> Vec<Uuid> {
    let Some(crate::self_ref::LineOwner::Window { screen, no }) = state.line_owner else { return Vec::new() };
    let Ok(store) = state.root.runtime.live_screens.read() else { return Vec::new() };
    let Some(w) = store.get(screen).and_then(|s| s.windows.get(&no)) else { return Vec::new() };
    if w.kind != super::pool::WINDOW {
        return Vec::new();
    }
    w.settings.get("tags").and_then(|v| v.as_str()).map(super::pool::tag_list).unwrap_or_default()
}
