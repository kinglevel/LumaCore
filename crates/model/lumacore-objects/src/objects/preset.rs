// ██████╗ ██████╗ ███████╗███████╗███████╗████████╗
// ██╔══██╗██╔══██╗██╔════╝██╔════╝██╔════╝╚══██╔══╝
// ██████╔╝██████╔╝█████╗  ███████╗█████╗     ██║
// ██╔═══╝ ██╔══██╗██╔══╝  ╚════██║██╔══╝     ██║
// ██║     ██║  ██║███████╗███████║███████╗   ██║
// ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝   ╚═╝
// FILE: preset

//! The one preset pool (decisions 0288): `Preset <n>` in the DataPool, whatever feature groups
//! its values touch. `Store Preset [n]` keeps every active programmer value; `At Preset n` puts
//! its values on the selection (`Call Preset n` is the same line); `Preset n` selects its
//! fixtures (`SelectFixtures`). Tags are its NodeMeta's (`Assign Tag … At Preset n`); the Presets
//! window filters the pool by the programmer's feature group and those tags.
//!
//! Owns:          PRESET, its door and tree row, List.
//! Doesn't own:   storing (`store`), applying and selecting (`apply`), the pool window's cells
//!                (`pool`), the programmer (`programmers`).
//! Entry points:  PRESET, PRESET_DOOR.
//! Decisions:     docs/decisions/0288-one-preset-pool-follows-the-programmer, §33 (SelectFixtures, At).

use std::collections::BTreeMap;

use lumacore_tree::{Datapool, Preset, Root};

use super::{Doc, ObjectDef, TreeDef, Verb, json_view, out};
use crate::AppState;
use crate::DispatchResult;
use crate::commands::help::HelpCategory;
use crate::commands::registry::{AliasDepth, CommandFamily};
use crate::errors::render_code;
use crate::vfs::{DynChild, NodeView};

pub mod apply;
pub mod pool;
pub mod store;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// `Store Preset … /Merge` (the Presets level on an existing preset, `rights::presets`),
/// `/Overwrite`, `/Remove`.
const PRESET_FLAGS: &[crate::commands::flags::FlagDef] = &[
    crate::commands::flags::MERGE.on(&[Verb::Store]),
    crate::commands::flags::OVERWRITE.on(&[Verb::Store, Verb::Copy, Verb::Paste]),
    crate::commands::flags::REMOVE.on(&[Verb::Store]),
    crate::commands::flags::BY.on(&[Verb::Compact]),
];

// The object door (decisions §35): Copy / Cut / Paste / Move / Insert / Compact / Delete.
crate::door_verbs!(preset_door, "Preset", super::PRESET);

pub static PRESET_DOOR: crate::pools::door::BTreePool<Preset> = crate::pools::door::BTreePool {
    object: "Preset",
    in_datapool: true,
    max: crate::pools::MAX_NO,
    map: door_map,
    map_ref: door_map_ref,
    name: |p| p.meta.name().map(str::to_string),
    identify: |p, guid| p.meta.identity.guid = guid,
    rules: crate::pools::door::PoolRules { can_delete: |_, _| Ok(()), deleted: |_, _| {}, changed: |_| {}, merge: None, orders: &[] },
};

pub static PRESET: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Preset",
        lum: crate::objects::LumShape::ids(1),
        default_verb: Verb::SelectFixtures,
        flags: PRESET_FLAGS,
        aliases: &["presets"],
        doc: Doc {
            display: "Preset",
            category: HelpCategory::DataPool,
            also_applies_to: &[],
            syntax: "Preset <n> | Store Preset [<n>] [\"name\"] [/Merge|/Overwrite|/Remove] | At Preset <n> | Call Preset <n>",
            description: "The DataPool's one preset pool (decisions 0288). Store keeps every active value of \
                          your programmer (Selective: per fixture; Universal when every fixture holds the \
                          same value); At puts its values on the selected fixtures, Call is the same; a bare \
                          Preset selects its fixtures. Tag presets to sort them; the Presets window shows the \
                          ones that touch the feature group you are on.",
            examples: &[
                "Store Preset 3 \"Deep red\"   -- your active values as preset 3",
                "Store Preset                 -- the next free number",
                "At Preset 3                  -- its values on your selection",
                "Preset 3                     -- select its fixtures",
                "Assign Tag \"Song 3\" At Preset 3",
            ],
            related: &["store", "at", "tag"],
            alias_depth: AliasDepth::Base,
            hidden: false,
        },
        family: Some(CommandFamily::Selection),
        call_undo: crate::commands::registry::PROGRAMMER_SHOWDATA,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::presets,
        planned: &[],
    },
    pool: Some(&pool::PRESET_POOL),
    verbs: &[
        (Verb::List, list),
        (Verb::Store, crate::commands::typed::typed::<store::StorePreset>),
        (Verb::SelectFixtures, crate::commands::typed::typed::<apply::SelectPreset>),
        (Verb::Delete, crate::commands::typed::typed::<preset_door::Delete>),
        (Verb::Copy, crate::commands::typed::typed::<preset_door::Copy>),
        (Verb::Cut, crate::commands::typed::typed::<preset_door::Cut>),
        (Verb::Paste, crate::commands::typed::typed::<preset_door::Paste>),
        (Verb::Move, crate::commands::typed::typed::<preset_door::Move>),
        (Verb::Insert, crate::commands::typed::typed::<preset_door::Insert>),
        (Verb::Compact, crate::commands::typed::typed::<preset_door::Compact>),
    ],
    delete_ask: Some(|s, a| crate::commands::door::delete_ask(s, &PRESET, a)),
    complete: None,
    props: Some(&PRESET_PROPS),
    tree: Some(TreeDef {
        door: Some(&PRESET_DOOR),
        datapool: Some(super::DatapoolTree { children: presets_in, object: preset_one }),
        name: None,
        node: crate::tree_path::Node::Presets,
        children: |root| first_dp(root).map_or_else(Vec::new, |dp| presets_in(root, dp)),
        object: |root, id| preset_one(root, first_dp(root)?, id),
    }),
};

/// What only a preset does in the one context menu (decisions 0288): its editor (the host opens
/// it), select its fixtures, store into it.
pub static CONTEXT: &[crate::objects::ContextDef] = &[crate::objects::ContextDef {
    object: &PRESET.decl,
    items: &[
        crate::objects::ContextItem { label: "Edit values…", icon: "pencil", verb: Verb::Info, line: "", live: true, value: "preseteditor" },
        crate::objects::ContextItem { label: "Select its fixtures", icon: "crosshair", verb: Verb::SelectFixtures, line: "SelectFixtures {object} {rows}", live: true, value: "" },
        crate::objects::ContextItem { label: "Store (merge)", icon: "arrow-down", verb: Verb::Store, line: "Store {object} {rows} /Merge", live: true, value: "" },
    ],
}];

static PRESET_PROPS: crate::props::Props =
    crate::props::Props { meta: crate::props::MetaLevel::Node, apply: Some(apply_json), ..crate::props::Props::BASE };

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `List Preset` (the pool) / `List Preset <n>` (one preset as stored).
fn list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    if let [no] = words.as_slice() {
        let found = no.parse::<u32>().ok().and_then(|n| in_pool(state)?.get(&n));
        return out(match found {
            Some(p) => serde_json::to_string_pretty(p).unwrap_or_default(),
            None => render_code(format!("No Preset {no} in this DataPool."), "E-CMD-007"),
        });
    }
    out(crate::commands::list::tree(state, crate::tree_path::Node::Presets, args))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The acting user's DataPool's presets.
pub fn in_pool(state: &AppState) -> Option<&BTreeMap<u32, Preset>> {
    state.root.show_data.datapools.get(&state.selected_datapool()).map(|d| &d.presets)
}

fn door_map(state: &mut AppState, dp: Option<u32>) -> Option<&mut BTreeMap<u32, Preset>> {
    let dp = dp?;
    Some(&mut state.root.show_data.datapools.entry(dp).or_insert_with(|| Datapool::new(Some("Default".into()))).presets)
}

fn door_map_ref(state: &AppState, dp: Option<u32>) -> Option<&BTreeMap<u32, Preset>> {
    state.root.show_data.datapools.get(&dp?).map(|d| &d.presets)
}

fn first_dp(root: &Root) -> Option<u32> {
    root.show_data.datapools.keys().next().copied()
}

fn presets_in(root: &Root, dp: u32) -> Vec<DynChild> {
    let Some(d) = root.show_data.datapools.get(&dp) else { return Vec::new() };
    d.presets
        .iter()
        .map(|(&id, p)| {
            let name = p.meta.name().unwrap_or_default();
            DynChild {
                display_name: if name.is_empty() { id.to_string() } else { format!("{id} \"{name}\"") },
                id,
                node_type: "Preset",
                is_leaf: true,
                child_count: 0,
                name: Some(name.to_string()).filter(|n| !n.is_empty()),
                guid: Some(p.meta.identity.guid),
            }
        })
        .collect()
}

fn preset_one(root: &Root, dp: u32, id: u32) -> Option<NodeView> {
    root.show_data.datapools.get(&dp)?.presets.get(&id).map(|p| json_view("Preset", id, p))
}

/// A preset's JSON written by the registry (`"Name"`): the one in its DataPool.
fn apply_json(root: &mut Root, dp: Option<u32>, id: u32, edit: &mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>) -> Result<(), String> {
    let dp = dp.or_else(|| first_dp(root)).ok_or("no DataPool")?;
    let p = root.show_data.datapools.get_mut(&dp).and_then(|d| d.presets.get_mut(&id)).ok_or_else(|| format!("no Preset {id}"))?;
    crate::props::edit_json(p, edit)
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
