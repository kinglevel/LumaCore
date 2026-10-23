//  █████╗ ██████╗ ██████╗ ██╗     ██╗   ██╗
// ██╔══██╗██╔══██╗██╔══██╗██║     ╚██╗ ██╔╝
// ███████║██████╔╝██████╔╝██║      ╚████╔╝
// ██╔══██║██╔═══╝ ██╔═══╝ ██║       ╚██╔╝
// ██║  ██║██║     ██║     ███████╗   ██║
// ╚═╝  ╚═╝╚═╝     ╚═╝     ╚══════╝   ╚═╝
// FILE: apply

//! Using a preset (decisions 0288, §33): `At Preset <n>` puts its values on the selected fixtures
//! (or on the fixtures a line names before At) — a Selective value on its own fixture, a
//! Universal value on every one that has the attribute — each value keeping the preset's GUID;
//! `Call Preset <n>` is the same line (written out as At, a hook); `Preset <n>` =
//! `SelectFixtures Preset <n>` selects the fixtures it fits.
//!
//! Owns:          at, SelectPreset, call_is_at, fixtures_of.
//! Doesn't own:   the programmer's write (`programmers::program`), selecting (`select_members`).
//! Entry points:  at, SelectPreset, call_is_at, fixtures_of.
//! Decisions:     docs/decisions/0288-one-preset-pool-follows-the-programmer, §33 (At, SelectFixtures).

use std::collections::{BTreeSet, HashMap};

use lumacore_tree::Preset;
use uuid::Uuid;

use crate::errors::render_code;
use crate::objects::out;
use crate::text::render_success;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `SelectFixtures Preset <n>` (a bare `Preset <n>`): its fixtures, by the selection rule.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "SelectFixtures", object = "Preset")]
pub struct SelectPreset {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
}

impl crate::commands::typed::Command for SelectPreset {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(select(state, &self.target).unwrap_or_else(|e| e))
    }
}

/// `At Preset <ids>` on the selection, or on `fixtures` (a line's own target before At).
pub fn at(state: &mut AppState, words: &[&str], fixtures: Option<Vec<u32>>) -> String {
    match at_presets(state, words, fixtures) {
        Ok(m) | Err(m) => m,
    }
}

/// `Call Preset <n>` is `At Preset <n>` (§33: a preset is called onto the selection) — written out
/// before rights, Oops and the journal, so it files as the programmer change it is.
pub fn call_is_at(_: &AppState, line: &str) -> Option<Result<String, String>> {
    let mut words = line.split_whitespace();
    let first = words.next()?;
    let call = matches!(
        crate::commands::registry::resolve(first),
        Ok(crate::commands::registry::Word::Command(c)) if c.object_verb == Some(crate::objects::Verb::Call)
    );
    let second = words.next()?;
    (call && super::PRESET.matches(second)).then(|| Ok(format!("At {}", line.trim_start()[first.len()..].trim_start())))
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn at_presets(state: &mut AppState, words: &[&str], fixtures: Option<Vec<u32>>) -> Result<String, String> {
    let picked = crate::resolve::words(state, &super::PRESET, words)?;
    let pool = picked.datapool.unwrap_or_else(|| state.selected_datapool());
    let targets: Vec<Uuid> = match fixtures {
        Some(ids) => state.root.show_data.fixture_guids(&ids),
        None => state.current_programmer().map(|p| p.selected_fixtures.clone()).unwrap_or_default(),
    };
    if targets.is_empty() {
        return Err(render_code("No fixtures selected: select some first, or Preset <n> selects the preset's own.", "E-CMD-009"));
    }
    let mut values = Vec::new();
    let mut names = Vec::new();
    for m in &picked.members {
        let Some(p) = state.root.show_data.datapools.get(&pool).and_then(|d| d.presets.get(&m.no)) else { continue };
        names.push(format!("Preset {}", m.no));
        values.extend(values_for(p, &targets));
    }
    if values.is_empty() {
        return Err(render_code(format!("{} holds nothing for the selected fixtures.", names.join(" + ")), "E-CMD-009"));
    }
    let count = crate::programmers::program(state, values)?;
    Ok(render_success(&format!("{} → {count} values.", names.join(" + "))))
}

/// A preset's values on `targets`: Selective on its own fixtures among them, Universal on all.
fn values_for(p: &Preset, targets: &[Uuid]) -> Vec<(Uuid, String, lumacore_base::Value)> {
    let guid = p.meta.identity.guid;
    let tagged = |v: &lumacore_base::Value| {
        let mut v = v.clone();
        v.preset = Some(guid);
        v
    };
    let mut out = Vec::new();
    for e in &p.values {
        match e.fixture_id {
            Some(f) if targets.contains(&f) => out.push((f, e.attribute.clone(), tagged(&e.value))),
            Some(_) => {}
            None => out.extend(targets.iter().map(|f| (*f, e.attribute.clone(), tagged(&e.value)))),
        }
    }
    out
}

fn select(state: &mut AppState, target: &lumacore_syntax::ast::Target) -> Result<String, String> {
    let picked = crate::resolve::resolve_one(state, target, &super::PRESET)?;
    let pool = picked.datapool.unwrap_or_else(|| state.selected_datapool());
    let mut fixtures: Vec<Uuid> = Vec::new();
    for m in &picked.members {
        let Some(p) = state.root.show_data.datapools.get(&pool).and_then(|d| d.presets.get(&m.no)) else { continue };
        for f in fixtures_of(&state.root.show_data, p) {
            if !fixtures.contains(&f) {
                fixtures.push(f);
            }
        }
    }
    if fixtures.is_empty() {
        return Err(render_code("No fixture here fits that preset.", "E-CMD-009"));
    }
    let n = state.select_members(&fixtures, &[], lumacore_tree::programmer::SelectMode::Plain, false, lumacore_tree::GridCursorMove::default())?;
    Ok(render_success(&format!("{n} fixtures selected.")))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The fixtures a preset fits, in patch order: a Selective preset's own, a Universal one's every
/// fixture whose mode has one of its attributes.
pub fn fixtures_of(show: &lumacore_tree::ShowData, p: &Preset) -> Vec<Uuid> {
    let own: Vec<Uuid> = p.fixtures().collect();
    if p.values.iter().all(|e| e.fixture_id.is_some()) {
        return own;
    }
    let attrs: BTreeSet<String> = p.values.iter().filter(|e| e.fixture_id.is_none()).map(|e| e.attribute.to_ascii_lowercase()).collect();
    let mut by_type: HashMap<(Uuid, Uuid, Uuid), bool> = HashMap::new();
    let patch = &show.devices.dmx.patch;
    let mut out = own;
    for f in patch.fixtures.values() {
        let key = (f.mvr.fixture_type, f.mvr.revision, f.mvr.mode);
        let fits = *by_type.entry(key).or_insert_with(|| {
            patch
                .fixture_types
                .values()
                .find(|t| t.meta.identity.guid == f.mvr.fixture_type)
                .and_then(|t| t.mode_of(f.mvr.revision, f.mvr.mode))
                .is_some_and(|(_, m)| m.channel_functions.iter().any(|cf| attrs.contains(&cf.attribute_name.to_ascii_lowercase())))
        });
        if fits && !out.contains(&f.meta.identity.guid) {
            out.push(f.meta.identity.guid);
        }
    }
    out
}
