//! Playback objects: Sequence, Cue, Executor, Master, Effect, Macro, Timecode,
//! EffectEngine. Sequence, Cue and the grand master run on the playback
//! engine (`commands/playback.rs`); the others list their tree node and
//! publish their other verbs as planned (decisions §29) until their part of
//! the engine exists.

use super::{Doc, ObjectDef, Undo, Verb, out};
use crate::AppState;
use crate::commands::help::HelpCategory;
use crate::commands::playback as pb;
use crate::commands::registry::{AliasDepth, PROGRAMMER_SHOWDATA};
use crate::commands::list;
use crate::commands::registry::CommandFamily;
use crate::DispatchResult;

/// Generates a playback object whose engine isn't built: List shows its tree
/// node `$path`; Info, and Call / Store when it has them, are planned.
macro_rules! playback_object {
    (@pool) => { None };
    (@pool $pool:expr) => { Some($pool) };
    ($static:ident, $label:literal, $node:ident, [$($alias:literal),*], $desc:literal, default: $default:ident,
     call: $call:literal, store: $store:literal, tree: $tree:expr) => {
        playback_object!($static, $label, $node, [$($alias),*], $desc, default: $default, call: $call, store: $store, tree: $tree,
            extra: (Doc::new($label, $desc), None, Undo::ReadOnly));
    };
    ($static:ident, $label:literal, $node:ident, [$($alias:literal),*], $desc:literal, default: $default:ident,
     call: $call:literal, store: $store:literal, tree: $tree:expr,
     extra: ($doc:expr, $family:expr, $undo:expr) $(, pool: $pool:expr)?) => {
        pub static $static: ObjectDef = {
            fn list_h(state: &mut AppState, args: &[&str]) -> DispatchResult {
                out(list::tree(state, crate::tree_path::Node::$node, args))
            }
            const VERBS: &[(Verb, super::Handler)] = &[(Verb::List, list_h)];
            const PLANNED: &[Verb] = match ($call, $store) {
                (true, true) => &[Verb::$default, Verb::Store, Verb::Info],
                (true, false) => &[Verb::$default, Verb::Info],
                (false, true) => &[Verb::Store, Verb::Info],
                (false, false) => &[Verb::Info],
            };
            ObjectDef {
                decl: crate::objects::ObjectDecl {
                    keyword: $label,
                    lum: crate::objects::LumShape::ids(1),
                    default_verb: crate::objects::Verb::$default,
                    flags: &[],
                    aliases: &[$($alias),*],
                    doc: $doc,
                    family: $family,
                    call_undo: $undo,
                    call_class: crate::commands::registry::CommandClass::Commit,
                    scope: crate::objects::on_master,
                    reach: crate::objects::show_reach,
                    rights: crate::rights::content,
                    planned: PLANNED,
                },
                pool: playback_object!(@pool $($pool)?),
                verbs: VERBS,
                delete_ask: None,
                complete: None,
                props: None,
                tree: $tree,
            }
        };
    };
}

// ── Tree collections (Root.ShowData.Playback) ───────────────────────────────

use super::{TreeDef, json_view};
use crate::vfs::{self, DynChild, NodeView};
use lumacore_tree::Root;

// Sequences live in the DataPools (decisions §30 "12f"): the plain functions serve the first.
fn sequences(root: &Root) -> Vec<DynChild> {
    root.show_data.datapools.keys().next().map_or_else(Vec::new, |&dp| sequences_in(root, dp))
}
fn sequence_view(root: &Root, id: u32) -> Option<NodeView> {
    sequence_in(root, *root.show_data.datapools.keys().next()?, id)
}
fn sequences_in(root: &Root, dp: u32) -> Vec<DynChild> {
    root.show_data.datapools.get(&dp).map_or_else(Vec::new, |d| vfs::playback_seq_children(&d.sequences))
}
fn sequence_in(root: &Root, dp: u32, id: u32) -> Option<NodeView> {
    root.show_data.datapools.get(&dp)?.sequences.get(&id).map(|o| json_view("Sequence", id, o))
}
fn executors(root: &Root) -> Vec<DynChild> {
    vfs::playback_exec_children(&root.show_data.playback.executors)
}
fn executor_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data
        .playback
        .executors
        .get(&id)
        .map(|o| json_view("Executor", id, o))
}
fn effects(root: &Root) -> Vec<DynChild> {
    vfs::playback_effect_children(&root.show_data.playback.effects)
}
fn effect_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data
        .playback
        .effects
        .get(&id)
        .map(|o| json_view("Effect", id, o))
}

fn seq_target(args: &[&str]) -> Result<pb::Target, DispatchResult> {
    pb::parse_target(args, &SEQUENCE).map_err(out)
}
fn cue_target(args: &[&str]) -> Result<pb::Target, DispatchResult> {
    pb::parse_target(args, &CUE).map_err(out)
}
macro_rules! with_target {
    ($parse:ident, $args:expr, |$t:ident| $body:expr) => {
        match $parse($args) {
            Ok($t) => out($body),
            Err(e) => e,
        }
    };
}

/// Opening a sheet stays on this station; everything else runs on the master.
fn seq_scope(verb: Verb, args: &[&str]) -> crate::commands::registry::Scope {
    match verb {
        Verb::Edit => crate::commands::registry::Scope::Client,
        _ => crate::objects::on_master(verb, args),
    }
}

/// The sequence a typed target names (none = your selected one), with its DataPool, through the
/// one resolver; `f` runs with that DataPool as the line's.
fn on_sequence(state: &mut AppState, target: Option<&lumacore_syntax::ast::Target>, f: fn(&mut AppState, pb::Target) -> String) -> DispatchResult {
    use crate::errors::render_code;
    let (sequence, datapool) = match target {
        None => (None, None),
        Some(t) => match crate::resolve::resolve_one(state, t, &SEQUENCE) {
            Ok(r) if !std::ptr::eq(r.object, &SEQUENCE) => return out(render_code(format!("Name a sequence, not {}.", r.object.keyword), "E-CMD-008")),
            Ok(r) => match r.members.as_slice() {
                [one] => (Some(one.no), r.datapool),
                [] if !r.named => (None, r.datapool),
                [] => return out(render_code("That sequence doesn't exist (List Sequence).", "E-PLAY-001")),
                _ => return out(render_code("Name one sequence.", "E-CMD-008")),
            },
            Err(e) => return out(e),
        },
    };
    let pool = datapool.or(state.datapool_override);
    let before = std::mem::replace(&mut state.datapool_override, pool);
    let text = f(state, pb::Target { sequence, cue: None });
    state.datapool_override = before;
    out(text)
}

macro_rules! sequence_verb {
    ($name:ident, $verb:literal, $run:expr) => {
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = $verb, object = "Sequence")]
        pub struct $name {
            #[lum(target)]
            pub target: Option<lumacore_syntax::ast::Target>,
        }

        impl crate::commands::typed::Command for $name {
            fn run(self, state: &mut AppState) -> DispatchResult {
                on_sequence(state, self.target.as_ref(), $run)
            }
        }
    };
}

sequence_verb!(OnSequence, "On", pb::on);
sequence_verb!(OffSequence, "Off", pb::off);
sequence_verb!(ToggleSequence, "Toggle", pb::toggle);
sequence_verb!(EditSequence, "Edit", |s, t| match t.sequence {
    None => crate::errors::render_code("Usage: Edit Sequence <n>", "E-CMD-006"),
    Some(no) => {
        let dp = s.selected_datapool();
        let Some(seq) = s.root.show_data.datapools.get(&dp).and_then(|d| d.sequences.get(&no)) else {
            return crate::errors::render_code(format!("Sequence {no} doesn't exist."), "E-CMD-007");
        };
        let name = seq.meta.identity.name.clone().unwrap_or_default();
        let guid = seq.meta.identity.guid.to_string();
        s.ui_requests.push(lumacore_wire::UiEvent::SequenceSheet { sequence: guid });
        crate::text::render_success(format!("Sequence {no} {name}").trim_end())
    }
});

fn seq_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    if args.is_empty() {
        return out(format!(
            "{}\n{}",
            list::tree(state, crate::tree_path::Node::Sequences, args),
            pb::list_running(state)
        ));
    }
    with_target!(seq_target, args, |t| pb::list_cues(state, t))
}

/// The Sequence pool (decisions §30 "12f": executor style, round 7): tap = the window's action;
/// Top holds an echo (`Top On` at the press, `Top Off` at the release, §32).
static SEQUENCE_POOL: crate::pools::PoolDef = crate::pools::PoolDef {
    window: "sequences",
    actions: &[
        crate::pools::PoolAction { name: "Select", command: "Select {obj}", release: "" },
        crate::pools::PoolAction { name: "Go", command: "Go {obj}", release: "" },
        crate::pools::PoolAction { name: "Toggle", command: "Toggle {obj}", release: "" },
        crate::pools::PoolAction { name: "Off", command: "Off {obj}", release: "" },
        crate::pools::PoolAction { name: "Top", command: "Top On {obj}", release: "Top Off {obj}" },
        crate::pools::PoolAction { name: "Edit", command: "Edit {obj}", release: "" },
        crate::pools::PoolAction { name: "None", command: "", release: "" },
    ],
    cells: sequence_cells,
    detail: None,
    head: None,
};

/// Every sequence of the pool as an executor-style cell: the cue before, the active cue and the
/// next; the active cue gold while it runs, framed when off; the blue mark while it runs; the
/// user's selected sequence framed gold.
fn sequence_cells(ctx: &crate::pools::PoolContext) -> Vec<crate::pools::PoolCell> {
    use crate::pools::{PoolCell, PoolCue};
    let Some(dp) = ctx.show.datapools.get(&ctx.datapool) else { return Vec::new() };
    let selected = ctx.show.user(ctx.user).and_then(|u| u.selected.sequence);
    dp.sequences
        .iter()
        .map(|(&no, seq)| {
            let mut cell = PoolCell::of(no, &seq.meta, ctx);
            let run = ctx.show.playbacks.live.sequences.get(&seq.meta.identity.guid);
            cell.running = run.is_some();
            cell.selected = selected == Some(seq.meta.identity.guid);
            let keys: Vec<u32> = seq.cues.keys().copied().collect();
            let at = run.and_then(|r| keys.iter().position(|k| *k == r.current_cue)).unwrap_or(0);
            if !keys.is_empty() {
                cell.cues = [at.checked_sub(1), Some(at), Some(at + 1)]
                    .into_iter()
                    .map(|i| {
                        let text = i
                            .and_then(|i| keys.get(i).map(|k| (i, k)))
                            .map(|(_, k)| format!("{} {}", lumacore_tree::cue_label(*k), seq.cues[k].meta.name().unwrap_or_default()))
                            .unwrap_or_default();
                        let state = match (i == Some(at), cell.running) {
                            (true, true) => "on",
                            (true, false) => "off",
                            _ => "",
                        };
                        PoolCue { text, state }
                    })
                    .collect();
            }
            cell
        })
        .collect()
}

// The object door (decisions §35): Copy / Cut / Paste / Move / Insert / Compact / Delete.
crate::door_verbs!(sequence_door, "Sequence", super::SEQUENCE);

type SequenceArc = std::sync::Arc<lumacore_tree::SequenceObject>;

fn sequence_map(state: &mut AppState, dp: Option<u32>) -> Option<&mut std::collections::BTreeMap<u32, SequenceArc>> {
    let dp = dp?;
    Some(&mut state.root.show_data.datapools.entry(dp).or_insert_with(|| lumacore_tree::Datapool::new(Some("Default".into()))).sequences)
}

fn sequence_map_ref(state: &AppState, dp: Option<u32>) -> Option<&std::collections::BTreeMap<u32, SequenceArc>> {
    state.root.show_data.datapools.get(&dp?).map(|d| &d.sequences)
}

/// Sequences on the door: a copy renews its cues' GUIDs (their `Walk`); a deleted one stops —
/// its running state and echoes go with it.
pub static SEQUENCE_DOOR: crate::pools::door::BTreePool<SequenceArc> = crate::pools::door::BTreePool {
    object: "Sequence",
    in_datapool: true,

    max: crate::pools::MAX_NO,
    map: sequence_map,
    map_ref: sequence_map_ref,
    name: |s| s.meta.identity.name.clone(),
    identify: |s, guid| std::sync::Arc::make_mut(s).meta.identity.guid = guid,
    rules: crate::pools::door::PoolRules {
        can_delete: |_, _| Ok(()),
        deleted: |state, s| {
            let playbacks = &mut state.root.show_data.playbacks;
            playbacks.live.sequences.remove(&s.meta.identity.guid);
            playbacks.echoes.sequences.remove(&s.meta.identity.guid);
        },
        changed: |_| {},
        merge: None,
        orders: &[],
    },
};

pub static SEQUENCE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Sequence",
        lum: crate::objects::LumShape::ids(1).children(&[&crate::objects::playback::CUE.decl]),
        default_verb: crate::objects::Verb::Select,
        flags: crate::commands::flags::STORE_CUE,
        aliases: &["sequences", "seq"],
        doc: Doc {
            display: "Sequence",
            category: HelpCategory::Playback,
            also_applies_to: &[],
            syntax: "Sequence <id> | On|Off|Toggle Sequence <id> | Store Sequence <id> Cue <n>",
            description: "A cue list. `Sequence 1` lists its cues (▶ = current). On starts it, Off \
                          stops it (its values release), Go/Go-/Goto step through it.",
            examples: &[
                "Sequence 1",
                "Go Sequence 1",
                "Off Sequence 1",
                "Store Sequence 2 Cue 1",
            ],
            related: &["go", "goto", "cue"],
            alias_depth: AliasDepth::Base,
            hidden: false,
        },
        family: Some(CommandFamily::Playback),
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: seq_scope,
        reach: crate::objects::show_reach,
        rights: crate::rights::content,
        planned: &[],
    },
    pool: Some(&SEQUENCE_POOL),
    verbs: &[
        (Verb::List, seq_list),
        (Verb::Info, seq_list),
        (Verb::Store, crate::commands::typed::typed::<pb::StoreSequence>),
        (Verb::On, crate::commands::typed::typed::<OnSequence>),
        (Verb::Off, crate::commands::typed::typed::<OffSequence>),
        (Verb::Toggle, crate::commands::typed::typed::<ToggleSequence>),
        (Verb::Edit, crate::commands::typed::typed::<EditSequence>),
        (Verb::Delete, crate::commands::typed::typed::<sequence_door::Delete>),
        (Verb::Copy, crate::commands::typed::typed::<sequence_door::Copy>),
        (Verb::Cut, crate::commands::typed::typed::<sequence_door::Cut>),
        (Verb::Paste, crate::commands::typed::typed::<sequence_door::Paste>),
        (Verb::Move, crate::commands::typed::typed::<sequence_door::Move>),
        (Verb::Insert, crate::commands::typed::typed::<sequence_door::Insert>),
        (Verb::Compact, crate::commands::typed::typed::<sequence_door::Compact>),
    ],
    delete_ask: Some(|s, a| crate::commands::door::delete_ask(s, &SEQUENCE, a)),
    complete: None,
    props: Some(&crate::props::Props {
        meta: crate::props::MetaLevel::Node,
        lists: &[&pb::SEQUENCE_PROPS],
        apply: Some(pb::sequence_write),
        within: Some(pb::sequence_within),
        ..crate::props::Props::BASE
    }),
    tree: Some(TreeDef {
        door: Some(&SEQUENCE_DOOR),
        datapool: Some(super::DatapoolTree { children: sequences_in, object: sequence_in }),
        name: None,
        node: crate::tree_path::Node::Sequences,
        children: sequences,
        object: sequence_view,
    }),
};
playback_object!(EXECUTOR, "Executor", Executors, ["executors", "exec"], "A playback executor", default: Select, call: true, store: true,
    tree: Some(TreeDef { datapool: None, door: None, name: None, node: crate::tree_path::Node::Executors, children: executors, object: executor_view }),
    extra: (Doc {
        display: "Executor",
        category: HelpCategory::Playback,
        also_applies_to: &[],
        syntax: "Executor <id> [Assign Sequence <id>]",
        description: "Select or configure an executor. Assign sequences to executors for playback.",
        examples: &["Executor 1", "Executor 1 Assign Sequence 1"],
        related: &["sequence", "assign", "flash"],
        alias_depth: AliasDepth::Base,
        hidden: false,
    }, Some(CommandFamily::Playback), PROGRAMMER_SHOWDATA));
playback_object!(MACRO, "Macro", Macros, ["macros"], "A macro", default: Go, call: true, store: true, tree: None,
    extra: (Doc {
        display: "Macro",
        category: HelpCategory::DataPool,
        also_applies_to: &[],
        syntax: "Macro <id> [Store | Edit | Delete]",
        description: "Execute, store, or edit a macro. Macros are stored Lum command sequences.",
        examples: &["Macro 1", "Store Macro 1", "Edit Macro 1"],
        related: &["store", "edit", "delete"],
        alias_depth: AliasDepth::Base,
        hidden: false,
    }, Some(CommandFamily::Selection), PROGRAMMER_SHOWDATA), pool: &MACRO_POOL);

/// The Macros pool (decisions 0287): one cell per macro of the DataPool, its line count as a
/// badge; tap = the window's action (Go is the macro's default verb).
static MACRO_POOL: crate::pools::PoolDef = crate::pools::PoolDef {
    window: "macros",
    actions: &[
        crate::pools::PoolAction { name: "Go", command: "Go {obj}", release: "" },
        crate::pools::PoolAction { name: "Edit", command: "Edit {obj}", release: "" },
        crate::pools::PoolAction { name: "None", command: "", release: "" },
    ],
    cells: macro_cells,
    detail: None,
    head: None,
};

fn macro_cells(ctx: &crate::pools::PoolContext) -> Vec<crate::pools::PoolCell> {
    let Some(dp) = ctx.show.datapools.get(&ctx.datapool) else { return Vec::new() };
    dp.macros
        .iter()
        .map(|(&no, m)| {
            let mut cell = crate::pools::PoolCell::of(no, &m.meta, ctx);
            cell.badges.push(format!("{} line{}", m.lines.len(), if m.lines.len() == 1 { "" } else { "s" }));
            cell
        })
        .collect()
}
playback_object!(EFFECT, "Effect", Effects, ["effects"], "An effect", default: Info, call: false, store: true,
    tree: Some(TreeDef { datapool: None, door: None, name: None, node: crate::tree_path::Node::Effects, children: effects, object: effect_view }));
pub static MASTER: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Master",
        lum: crate::objects::LumShape::ids(2).children(&[&crate::commands::groups::GROUP.decl]),
        default_verb: crate::objects::Verb::Info,
        flags: &[],
        aliases: &["masters"],
        doc: Doc {
            display: "Master",
            category: HelpCategory::Playback,
            also_applies_to: &[],
            syntax: "Master 2.1|2.2|2.4|Highlight|Solo [At <0-100> | At + 10 …] | Master Highlight|Solo Mode Toggle|AlwaysOn|Disabled | On|Off|Toggle Master 2.1",
            description: "Masters scale the output after the merge (the master groups, decisions §30 \"12f\"). \
                          Master 2.1 is the grand master: its level scales every intensity; switched \
                          Off it is a blackout (the level is kept). Master 2.2 / 2.4 cap everybody's \
                          Highlight / Solo; Master \"Highlight\" and Master \"Solo\" are yours (how strongly \
                          your Highlight and Solo act: Toggle, AlwaysOn or Disabled). Levels take \
                          relative input (At + 10, At * 2). Masters are never undone. Groups 1, 3, 4, \
                          5 and 2.3 / 2.5 are planned.",
            examples: &[
                "Master 2.1 At 80",
                "Master 2.1 At - 10",
                "Off Master 2.1   -- blackout",
                "On Master 2.1",
                "Master 2.4 At 50  -- Solo for everybody at half",
                "Master \"Highlight\" At 50",
                "Set Master \"Solo\" \"Mode\" AlwaysOn",
                "Master Group 3 At 50 Fade 2",
            ],
            related: &["park", "highlight", "solo"],
            alias_depth: AliasDepth::Base,
            hidden: false,
        },
        family: Some(CommandFamily::Playback),
        // Masters are never undone (decisions §29, §30 "12f"): replicated and journaled only.
        call_undo: crate::commands::registry::Undo::Untracked,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::playback,
        planned: &[],
    },
    pool: None,
    verbs: &[
        // `Master 2.1` = Info; `Master 2.1 At 80` = its level (decisions §33 41–43).
        (Verb::Info, |s, a| {
            if a.iter().any(|w| w.eq_ignore_ascii_case("at")) {
                return out(crate::errors::render_code("Info only reads: Master 2.1 At 80 sets the level.", "E-SYN-003"));
            }
            out(pb::master_call(s, a))
        }),
        (Verb::At, crate::commands::typed::typed::<pb::MasterAt>),
        (Verb::List, |s, a| {
            out(list::tree(s, crate::tree_path::Node::LiveMasters, a))
        }),
        (Verb::On, crate::commands::typed::typed::<pb::OnMaster>),
        (Verb::Off, crate::commands::typed::typed::<pb::OffMaster>),
        (Verb::Toggle, crate::commands::typed::typed::<pb::ToggleMaster>),
    ],
    delete_ask: None,
    complete: None,
    props: Some(&pb::MASTER_PROPS),
    tree: None,
};
playback_object!(TIMECODE, "Timecode", Timecodes, ["timecodes"], "A timecode show", default: Select, call: false, store: false, tree: None);
playback_object!(EFFECT_ENGINE, "EffectEngine", EffectEngines, ["effectengines"], "An effect engine", default: Info, call: false, store: false, tree: None);

// Cue: cues live in sequences (`Cue 3 Sequence 2`; no sequence = the selected one).

pub static CUE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Cue",
        // A feature group's timing in the cue is that group under it (§33 87).
        lum: crate::objects::LumShape::decimal().children(&[&crate::commands::attributes::FEATURE_GROUP.decl]),
        default_verb: crate::objects::Verb::Goto,
        flags: crate::commands::flags::STORE_CUE,
        aliases: &["cues"],
        doc: Doc {
            display: "Cue",
            category: HelpCategory::Playback,
            also_applies_to: &[],
            syntax: "Cue <n> [Sequence <m>] | Store Cue <n> [Sequence <m>] | List Cue [Sequence <m>] | Set Cue <n> [Sequence <m>] [FeatureGroup <g>] Fade|OutFade|Delay|OutDelay <s> | Set Cue <n> FeatureGroup <g> Path Linear|SCurve|EaseIn|EaseOut|Snap",
            description: "A cue of a sequence (numbers may have decimals: Cue 1.5). `Cue 3` goes to \
                          it (Goto). Store Cue records the active programmer values; one equal to \
                          what tracks into the cue is kept as a blocked value. `At Release` stores a release. \
                          Without `Sequence <m>`, your selected sequence. Set gives the cue's \
                          timing: a dimmer going up and every other attribute use Fade/Delay, a \
                          dimmer going down OutFade/OutDelay; snap attributes (GDTF) jump after \
                          the delay. With `FeatureGroup <g>` the time is that group's own (`Cue` = \
                          back to the cue's), and its Path shapes its fades.",
            examples: &[
                "Store Cue 1",
                "Cue 3",
                "Store Sequence 4 Cue 2",
                "Set Cue 2 \"Fade\" 3",
                "Set Cue 2 FeatureGroup \"Position\" \"Path\" SCurve",
                "List Cue Sequence 4",
            ],
            related: &["sequence", "go", "goto", "store"],
            alias_depth: AliasDepth::Base,
            hidden: false,
        },
        family: Some(CommandFamily::Playback),
        call_undo: Undo::Untracked,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::content,
        planned: &[Verb::Delete, Verb::Move],
    },
    pool: None,
    verbs: &[
        (Verb::Store, crate::commands::typed::typed::<pb::StoreCue>),
        (Verb::List, |s, a| {
            with_target!(cue_target, a, |t| pb::list_cues(s, t))
        }),
        (Verb::Info, |s, a| {
            with_target!(cue_target, a, |t| pb::list_cues(s, t))
        }),
    ],
    delete_ask: None,
    complete: None,
    props: Some(&crate::props::Props {
        lists: &[&pb::CUE_TIMING],
        keyed: Some(crate::props::Keyed {
            rows: pb::cue_rows,
            row: pb::cue_row,
        }),
        label: Some(|_, k| format!("Cue {k}")),
        within: Some(pb::cue_within),
        ..crate::props::Props::BASE
    }),
    tree: None,
};
