//! Patch objects: Fixture, FixtureType, Universe, DataPool. (Network,
//! Output and Input are in `protocols.rs`.)

use super::{ObjectDef, Verb, filter, out, patch_claimed};
use crate::AppState;
use crate::commands::{edit, info, list, show, universe};

/// Fixture and Universe on the object door (decisions §35).
mod door;
/// The Fixture's MVR ids and its place (decisions §34 "34g4 patch hardened").
pub(crate) mod fixture;
/// The MVR tree's words: MvrLayer, MvrGroup, MvrClass, MvrPosition (§34 "34g4 part 3c hardened").
pub mod devices_meta;
/// A universe in the Devices draft: its Delete and Move questions (§11 "Issue 11 hardening").
mod universe_ask;
/// `Info DataPool <n>`: its pools and their counts (decisions 0291).
mod dp_info;
mod dp_props;
/// The Patch setup dialog's menu row (decisions §34 "34g4 patch hardened" 9).
pub mod menu;
/// `DMX`: a universe's slots, the object (decisions §33 48, round 3 19).
pub(crate) mod dmx;
/// `Identify`: fixtures or addresses show themselves on the rig (§34 round 3).
pub mod identify;
pub mod mvr;
pub(crate) use fixture::by_second_id as second_ids;
use door::{FIXTURE_DOOR, UNIVERSE_DOOR, fixture_door, universe_door};
use crate::DispatchResult;

// ── Fixture ──────────────────────────────────────────────────────────────────

/// The numbers a typed target names, through the one resolver (decisions §33 57).
fn numbers(state: &AppState, target: &lumacore_syntax::ast::Target, obj: &'static ObjectDef) -> Result<Vec<u32>, String> {
    Ok(crate::resolve::resolve_one(state, target, obj)?.members.iter().map(|m| m.no).collect())
}

/// `Store Universe [<n>] ["name"]` (the Devices claim): a new universe at the slot.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Universe")]
pub struct StoreUniverse {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
    #[lum(value)]
    pub name: Option<String>,
}

impl crate::commands::typed::Command for StoreUniverse {
    fn run(self, state: &mut AppState) -> DispatchResult {
        use crate::resolve::Slot;
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        let next = |s: &AppState| (1..).find(|n| !s.root.show_data.devices.dmx.patch.universes.contains_key(n)).unwrap_or(1);
        out(match crate::resolve::slot(state, self.target.as_ref(), &UNIVERSE) {
            Ok((Slot::Number(n), _)) => universe::create_universe(state, n, self.name),
            Ok((Slot::NewName(name), _)) => universe::create_universe(state, next(state), Some(name)),
            Ok((Slot::Next, _)) => universe::create_universe(state, next(state), self.name),
            Ok((Slot::Key(k), _)) => crate::errors::render_code(format!("'{k}' isn't a Universe number."), "E-CMD-008"),
            Err(e) => e,
        })
    }
}

/// `Delete Universe <target>`: those without fixtures go (the Devices claim).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "Universe")]
pub struct DeleteUniverse {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
}

impl crate::commands::typed::Command for DeleteUniverse {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        out(numbers(state, &self.target, &UNIVERSE).map_or_else(|e| e, |ids| universe::delete_universes(state, &ids)))
    }
}

fn fixture_edit(state: &mut AppState, args: &[&str]) -> DispatchResult {
    edit::edit_with(state, args, edit::edit_fixture)
}
fn fixture_dump(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(show::dump_fixture(state, args))
}

/// Collect fixture IDs and names for completion after `Fixture`.
fn collect_fixture_completions(state: &AppState, word_lower: &str) -> Vec<String> {
    use lumacore_tree::TreeNode;
    let mut candidates = Vec::new();
    // By number (decisions §40 5): a fixture without one completes by its name only.
    for fixture in state.root.show_data.devices.dmx.patch.fixtures.values() {
        let id_str = fixture.mvr.fid.map(|n| n.to_string()).unwrap_or_default();
        if !id_str.is_empty() && id_str.starts_with(word_lower) {
            candidates.push(id_str);
        }
        if let Some(name) = fixture.name()
            && name.to_lowercase().starts_with(word_lower)
        {
            candidates.push(format!("\"{name}\""));
        }
    }
    candidates
}

/// Fixture ids / names, and `Thru` for ranges (`Fixture 1 Thru 5`).
fn fixture_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let mut out = collect_fixture_completions(state, prefix);
    if "thru".starts_with(prefix) {
        out.push("Thru".to_string());
    }
    out
}

/// What follows a fixture owner first: its addresses (`Fixture 3 DMX 2`, round 3 19).
static FIXTURE_CHILDREN: [&crate::objects::ObjectDecl; 2] = [&dmx::DMX.decl, &fixture::multipatch::MULTIPATCH.decl];

pub static FIXTURE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Fixture",
        lum: crate::objects::LumShape::ids(1).children(&FIXTURE_CHILDREN).id_kinds(fixture::ID_KINDS),
        default_verb: crate::objects::Verb::SelectFixtures,
        flags: { use fixture::store::*; &[FIXTURE_TYPE, MODE, REVISION, EVERY, KEEP_SPACING, SWAP, crate::commands::flags::DRIVE.on(&[Verb::Store]), crate::commands::flags::KEEP_CHILDREN.on(&[Verb::Delete])] },
        aliases: &["fixtures", "f"],
        doc: crate::commands::registry::Doc {
            display: "Fixture",
            category: crate::commands::help::HelpCategory::Fixture,
            also_applies_to: &[],
            syntax: "Fixture [<sel>] [At <value>]",
            description: "A patched lighting fixture in the show. \
                          Use as a selector to target fixtures for commands. \
                          Without `At`, selects the fixture(s). \
                          With `At <value>`, also sets their dimmer. \
                          Watchable: `watch fixture` shows the fixture list live.",
            examples: &[
                "Fixture 1          -- select fixture 1",
                "Fixture 1 At 50    -- select and set dimmer to 50%",
                "Fixture 1 Thru 4 At Full  -- select 1–4 and set to full",
                "watch fixture      -- live fixture list",
            ],
            related: &["at", "select", "group", "list"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: Some(crate::commands::registry::CommandFamily::Selection),
        call_undo: crate::commands::registry::PROGRAMMER,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::fixture,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::SelectFixtures, fixture::call),
        (Verb::At, fixture::call),
        (Verb::Store, crate::commands::typed::typed::<fixture::store::StoreFixture>),
        (Verb::Delete, crate::commands::typed::typed::<fixture::DeleteFixture>),
        (Verb::Assign, crate::commands::typed::typed::<devices_meta::AssignFixture>),
        (Verb::Unassign, crate::commands::typed::typed::<devices_meta::UnassignFixture>),
        (Verb::Edit, fixture_edit),
        (Verb::Move, crate::commands::typed::typed::<devices_meta::MoveFixture>),
        (Verb::Insert, crate::commands::typed::typed::<fixture_door::Insert>),
        (Verb::Compact, crate::commands::typed::typed::<fixture_door::Compact>),
        (Verb::Cut, crate::commands::typed::typed::<fixture_door::Cut>),
        (Verb::Paste, crate::commands::typed::typed::<fixture_door::Paste>),
        (Verb::List, fixture::info::fixture_list),
        (Verb::Info, fixture::info::fixture_info),
        (Verb::Dump, fixture_dump),
        // `On Fixture 2 + 4`: the selection grammar takes the bare numbers.
        (Verb::On, |s, a| {
            out(crate::commands::control::knock_in(s, a))
        }),
        (Verb::Off, |s, a| {
            out(crate::commands::control::knock_out(s, a))
        }),
    ],
    // A node's question names what is under it (decisions §40 "40d hardened" 3).
    delete_ask: Some(fixture::delete_ask),
    complete: Some(fixture_complete),
    props: Some(&crate::props::Props {
        meta: crate::props::MetaLevel::Node,
        lists: &[&fixture::LUMA_PROPS, &FIXTURE_ADDRESS, &fixture::MVR_PROPS, &fixture::PLACE_PROPS],
        apply: Some(fixture_write),
        scoped: &[&crate::commands::groups::MEMBER_PROPS],
        // The Live Patch columns (decisions §34 "34g4 patch hardened" 8).
        live: &["Name", "Note", "Address", "PosX", "PosY", "PosZ", "RotX", "RotY", "RotZ", "MasterReact"],
        ..crate::props::Props::BASE
    }),
    tree: Some(super::TreeDef {
        door: Some(&FIXTURE_DOOR),
        datapool: None,
        name: None,
        node: crate::tree_path::Node::Fixtures,
        children: fixtures_children,
        object: fixture_view,
    }),
};

// ── FixtureType ──────────────────────────────────────────────────────────────

fn fixture_type_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(list::fixture_types(state, args))
}
fn fixture_type_label(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(edit::label_or_ask(state, "FixtureType", args, edit::label_fixture_type, |s, n| {
        s.root.show_data.devices.dmx.patch.fixture_types.get(&n).and_then(|t| t.meta.identity.name.clone())
    }, |s| s.root.show_data.devices.dmx.patch.fixture_types.keys().copied().collect()))
}
fn fixture_type_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(info::info_fixture_type(state, args.first().copied()))
}
fn fixture_type_dump(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(show::dump_fixture_type(state, args))
}
fn fixture_type_complete(state: &AppState, prefix: &str) -> Vec<String> {
    filter(
        state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .fixture_types
            .keys()
            .map(u32::to_string),
        prefix,
    )
}

pub static FIXTURE_TYPE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "FixtureType",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        flags: &[crate::commands::flags::DRIVE.on(&[Verb::Import])],
        aliases: &["fixturetypes", "type", "types", "ft"],
        doc: crate::commands::registry::Doc::new(
            "FixtureType",
            "An imported fixture type (GDTF)",
        ),
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: |v| if v == Verb::Import { crate::commands::registry::Reach::Drive } else { crate::commands::registry::Reach::Show },
        rights: crate::rights::setup,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::List, fixture_type_list),
        (Verb::Info, fixture_type_info),
        (Verb::Dump, fixture_type_dump),
        (Verb::Label, fixture_type_label),
        (Verb::Import, crate::commands::typed::typed::<crate::commands::import::ImportFixtureType>),
    ],
    delete_ask: None,
    complete: Some(fixture_type_complete),
    props: Some(&crate::props::Props { meta: crate::props::MetaLevel::Node, apply: Some(fixture_type_write), ..crate::props::Props::BASE }),
    tree: Some(super::TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: crate::tree_path::Node::FixtureTypes,
        children: fixture_types_children,
        object: fixture_type_view,
    }),
};

/// `Set Fixture <n> "Address" <u.a>`: re-patch a fixture (Live Patch, no
/// claim — decisions §11).
fn set_address(state: &mut AppState, args: &[&str]) -> Option<String> {
    let [word, id, prop, addr] = args else {
        return None;
    };
    if !super::find(word).is_some_and(|o| std::ptr::eq(o, &FIXTURE))
        || !prop.eq_ignore_ascii_case("address")
    {
        return None;
    }
    let usage = || {
        crate::errors::render_code(
            "Usage: Set Fixture <n> \"Address\" <universe>.<address>",
            "E-CMD-006",
        )
    };
    let Ok(fixture_id) = id.parse::<u32>() else {
        return Some(usage());
    };
    let Some((u, a)) = addr
        .split_once('.')
        .and_then(|(u, a)| Some((u.parse::<u32>().ok()?, a.parse::<u16>().ok()?)))
    else {
        return Some(usage());
    };
    Some(
        match lumacore_patch::PatchEngine::readdress_fixture(
            &mut state.root.show_data,
            fixture_id,
            u,
            a,
        ) {
            Ok(()) => crate::text::render_success(&format!(
                "Fixture {fixture_id} now at {u}.{a:03}"
            )),
            Err(e) => crate::errors::render(&e),
        },
    )
}

// ── Universe ─────────────────────────────────────────────────────────────────

fn universe_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(list::universes(state, args))
}
fn universe_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(info::info_universe(state, args.first().copied()))
}
fn universe_dump(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(show::dump_universe(state, args))
}
fn universe_complete(state: &AppState, prefix: &str) -> Vec<String> {
    filter(
        state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .universes
            .keys()
            .map(u32::to_string),
        prefix,
    )
}

/// A universe's name/note and how DMX input merges into it (Live Patch).
/// A fixture's DMX address (Live Patch, decisions §11): `Set Fixture 3 "Address" 1.001`.
static FIXTURE_ADDRESS: [props::Property; 1] = [props::Property {
    name: "Address",
    kind: props::PropKind::Declared,
    rights: crate::commands::registry::Rights::Program,
    section: "Patch",
    doc: "its first DMX address, universe.address (re-patch, Live Patch)",
    access: props::Access::Keyed {
        get: |s, k| {
            let patch = &s.root.show_data.devices.dmx.patch;
            let Some(b) = k.parse::<u32>().ok().and_then(|n| patch.fixture(n)).and_then(|f| f.mvr.breaks.first()) else { return String::new() };
            let u = patch.universes.iter().find(|(_, u)| u.meta.identity.guid == b.universe_id).map_or(0, |(n, _)| *n);
            format!("{u}.{:03}", b.start_address)
        },
        set: |s, k, v| props::from_reply(set_address(s, &["Fixture", k, "Address", v]).unwrap_or_default()),
    },
    ..props::Property::BASE
}];

const UNIVERSE_PROPS: [props::Property; 1] = [
    props::Property {
        name: "MergeMode",
        access: crate::props::Access::Field("merge_mode"),
        kind: props::PropKind::Choice(&lumacore_protocols::MergeMode::LABELS),
        rights: crate::commands::registry::Rights::Setup,
        doc: "how DMX input merges in: Off, Prio (input priority of the line), HTP, LowTP",
        ..crate::props::Property::BASE
    },
];

/// The protocol universe a run starts at (decisions §33 22, §34 "34f7c hardened" 3): sACN's
/// universe, Art-Net's Port-Address.
pub const ROUTE_UNIVERSE: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("Universe", "The protocol universe the run starts at (sACN n, Art-Net Net.SubNet.Universe)")
        .value("address")
        .on(&[Verb::Assign]);

/// The protocol line a destination names (`Sacn 1`, `ArtNet 2`), through the resolver: its
/// protocol (the object's tree name) and number. A line that doesn't exist yet is named by the
/// number typed (Assign makes it).
pub fn line_of(state: &AppState, to: &lumacore_syntax::ast::Target) -> Result<(&'static str, u32), String> {
    let usage = || crate::errors::render_code("Name the line: At Sacn <n> | At ArtNet <n>.", "E-CMD-006");
    let lines = |n: crate::tree_path::Node| crate::objects::all().filter(move |o| o.tree.as_ref().is_some_and(|t| t.node == n));
    let Some(first) = lines(crate::tree_path::Node::DmxLines).next() else { return Err(usage()) };
    let mut items = crate::resolve::resolve(state, to, first)?.into_iter();
    let (Some((None, r)), None) = (items.next(), items.next()) else { return Err(usage()) };
    // A line is known by its tree: a protocol's (decisions §34 "34f7c hardened" 5).
    let Some(protocol) = r.object.tree.as_ref().filter(|t| matches!(t.node, crate::tree_path::Node::DmxLines | crate::tree_path::Node::DataLines)).and_then(|t| t.name) else {
        return Err(usage());
    };
    match r.members.as_slice() {
        [one] => Ok((protocol, one.no)),
        [] => {
            // Not there yet: the one number as typed.
            use lumacore_syntax::ast::{Id, Item, Range};
            let typed = match to.items.as_slice() {
                [(None, Item::Path(segs))] => match segs.last().and_then(|seg| seg.ids.as_ref()).map(|ids| ids.ranges.as_slice()) {
                    Some([(None, Range::One(Id::Number(n)))]) => n.node.parse::<u32>().ok(),
                    _ => None,
                },
                _ => None,
            };
            typed.map(|n| (protocol, n)).ok_or_else(|| crate::errors::render_code(format!("No such {protocol}."), "E-CMD-007"))
        }
        _ => Err(crate::errors::render_code("Name one line.", "E-CMD-008")),
    }
}

fn target_of(v: &lumacore_syntax::ast::Value) -> Option<&lumacore_syntax::ast::Target> {
    match v {
        lumacore_syntax::ast::Value::Target(t) => Some(t),
        _ => None,
    }
}

/// `/Universe <address>` as typed.
pub fn wire_text(v: &Option<lumacore_syntax::ast::Value>) -> Result<Option<String>, String> {
    match v {
        None => Ok(None),
        Some(lumacore_syntax::ast::Value::Number(n)) => Ok(Some(n.node.clone())),
        Some(_) => Err(crate::errors::render_code("/Universe takes a number (Art-Net Net.SubNet.Universe).", "E-CMD-008")),
    }
}

/// `Assign Universe <target> At Sacn <n> [/Universe <address>]`: the line sends them
/// (decisions §34 "34f7c hardened" 3; lines are live, no claim, §11).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Assign", object = "Universe")]
pub struct AssignUniverse {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "destination")]
    pub to: lumacore_syntax::ast::Value,
    #[lum(option = "Universe")]
    pub wire: Option<lumacore_syntax::ast::Value>,
}

impl crate::commands::typed::Command for AssignUniverse {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let ids = numbers(state, &self.target, &UNIVERSE);
        let to = target_of(&self.to).ok_or_else(|| crate::errors::render_code("Assign Universe … At Sacn <n>.", "E-CMD-006"));
        match (ids, to.and_then(|t| line_of(state, t)), wire_text(&self.wire)) {
            (Ok(ids), Ok((protocol, line)), Ok(wire)) => {
                universe::assign_run(state, protocol, line, &ids, lumacore_protocols::Direction::Out, wire.as_deref())
            }
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => out(e),
        }
    }
}

/// `Unassign Universe <target> [At Sacn <n>]` (no `At`: every line carrying them).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Unassign", object = "Universe")]
pub struct UnassignUniverse {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "destination")]
    pub to: Option<lumacore_syntax::ast::Value>,
}

impl crate::commands::typed::Command for UnassignUniverse {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let reply = (|| {
            let ids = numbers(state, &self.target, &UNIVERSE)?;
            let line = match &self.to {
                None => None,
                Some(v) => {
                    let to = target_of(v).ok_or_else(|| crate::errors::render_code("Unassign Universe … [At Sacn <n>].", "E-CMD-006"))?;
                    Some(line_of(state, to)?)
                }
            };
            Ok(universe::unassign_universes(state, &ids, line))
        })();
        out(reply.unwrap_or_else(|e| e))
    }
}

pub static UNIVERSE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Universe",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        flags: &[ROUTE_UNIVERSE],
        aliases: &["universes", "dmxuniverse"],
        doc: crate::commands::registry::Doc {
            display: "Universe",
            category: crate::commands::help::HelpCategory::Universe,
            also_applies_to: &[],
            syntax: "Universe [<id> [\"name\"]]",
            description: "A DMX universe in the show (512 channels). \
                          A DMX universe: `Store Universe <n> [\"name\"]` creates one (requires the Devices claim). \
                          Routes: Assign … At Output / At Input. MergeMode sets how DMX input \
                          merges in (Off, Prio, HTP, LowTP). \
                          Watchable: `watch universe` shows the universe list live.",
            examples: &[
                "universe           -- navigate to Universe context",
                "Store Universe 1 \"Main\"  -- create with a name (needs claim)",
                "Set Universe 3 \"MergeMode\" HTP",
                "watch universe     -- live universe list",
            ],
            related: &["assign", "unassign", "output", "fixture"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: Some(crate::commands::registry::CommandFamily::Selection),
        call_undo: crate::commands::registry::PATCH,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::setup,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Store, crate::commands::typed::typed::<StoreUniverse>),
        (Verb::Delete, crate::commands::typed::typed::<DeleteUniverse>),
        (Verb::Move, crate::commands::typed::typed::<universe_ask::MoveUniverse>),
        (Verb::Insert, crate::commands::typed::typed::<universe_door::Insert>),
        (Verb::Compact, crate::commands::typed::typed::<universe_door::Compact>),
        (Verb::Cut, crate::commands::typed::typed::<universe_door::Cut>),
        (Verb::Paste, crate::commands::typed::typed::<universe_door::Paste>),
        (Verb::Assign, crate::commands::typed::typed::<AssignUniverse>),
        (Verb::Unassign, crate::commands::typed::typed::<UnassignUniverse>),
        (Verb::List, universe_list),
        (Verb::Info, universe_info),
        (Verb::Dump, universe_dump),
    ],
    delete_ask: Some(universe_ask::delete_ask),
    complete: Some(universe_complete),
    // A universe's name and note are live like a fixture's (the Live Patch).
    props: Some(&crate::props::Props { meta: crate::props::MetaLevel::Node, lists: &[&UNIVERSE_PROPS], apply: Some(universe_write), live: &["Name", "Note"], ..crate::props::Props::BASE }),
    tree: Some(super::TreeDef {
        door: Some(&UNIVERSE_DOOR),
        datapool: None,
        name: None,
        node: crate::tree_path::Node::Universes,
        children: universes_children,
        object: universe_view,
    }),
};

// ── DataPool ─────────────────────────────────────────────────────────────────

/// `DataPool 2` = `Select DataPool 2`; `DataPool 2 Group 3` = Group 3's default verb in
/// DataPool 2 (decisions §33 6, 49).
fn datapool_select(state: &mut AppState, args: &[&str]) -> DispatchResult {
    if let Some(result) = in_datapool(state, Verb::Call, args) {
        return result;
    }
    let line: Vec<&str> = std::iter::once("DataPool").chain(args.iter().copied()).collect();
    out(crate::commands::selections::cmd_select(state, &line))
}

fn datapool_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    if let Some(result) = in_datapool(state, Verb::List, args) {
        return result;
    }
    out(list::tree(state, crate::tree_path::Node::DataPools, args))
}

/// `<verb> DataPool <n|"name"> Group …` (owner first, CLAUDE.md "Nested
/// objects"; R1-55): the verb of an object that lives in a DataPool (its
/// `TreeDef::datapool`: Group, Sequence …, and a sequence's Cue) runs in that
/// DataPool instead of the user's selected one — a pool window or sequence
/// sheet pinned to a DataPool sends this form. None when the line doesn't
/// name such an object after the DataPool.
fn in_datapool(state: &mut AppState, verb: Verb, args: &[&str]) -> Option<DispatchResult> {
    let [owner, word, rest @ ..] = args else {
        return None;
    };
    let lives_in_pool = |o: &&ObjectDef| {
        o.matches(word) && (o.tree.as_ref().is_some_and(|t| t.datapool.is_some()) || std::ptr::eq(*o, &super::playback::CUE))
    };
    let object: &ObjectDef = super::all().find(lives_in_pool)?;
    let owner = owner.trim_matches('"');
    let no = owner.parse::<u32>().ok().filter(|n| state.root.show_data.datapools.contains_key(n)).or_else(|| {
        state
            .root
            .show_data
            .datapools
            .iter()
            .find(|(_, d)| d.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(owner)))
            .map(|(n, _)| *n)
    });
    let Some(no) = no else {
        return Some(out(crate::errors::render_code(format!("No DataPool {owner}."), "E-CMD-007")));
    };
    // `Verb::Call` here is the bare form (`DataPool 2 Group 3`): the child's default verb.
    let handler = match verb {
        Verb::Call => None,
        v => match object.handler(v) {
            Some(h) => Some(h),
            None => {
                return Some(out(crate::errors::render_code(
                    format!("{} doesn't take {verb:?}.", object.keyword),
                    "E-CMD-004",
                )));
            }
        },
    };
    let previous = state.datapool_override.replace(no);
    let asked_before = state.asking.is_some();
    let result = match handler {
        Some(h) => h(state, rest),
        None => super::call(state, object, rest),
    };
    state.datapool_override = previous;
    // A question's answer runs later, without the override: its lines name
    // the DataPool again (`Store DataPool 2 Group 3 /Overwrite`) — whether
    // the handler returned it or (a text handler) left it in `asking`.
    if !asked_before && let Some(ask) = state.asking.take() {
        state.asking = Some(ask.map_lines(|line| owner_first(line, no)));
    }
    Some(match result {
        DispatchResult::Ask(ask) => DispatchResult::Ask(Box::new((*ask).map_lines(|line| owner_first(line, no)))),
        other => other,
    })
}

/// `line` with `DataPool <no>` put before a Group its first statement names
/// right after the verb (or first): `Store Group 3 /o` → `Store DataPool 2 Group 3 /o`.
fn owner_first(line: &str, no: u32) -> String {
    let (head, tail) = line.split_once(';').map_or((line, None), |(h, t)| (h, Some(t)));
    let words: Vec<&str> = head.split_whitespace().collect();
    let group = |w: &&str| crate::commands::groups::GROUP.matches(w);
    let at = match words.as_slice() {
        [first, ..] if group(first) => Some(0),
        [_, second, ..] if group(second) => Some(1),
        _ => None,
    };
    let Some(at) = at else {
        return line.to_string();
    };
    let mut out: Vec<String> = words.iter().map(|w| w.to_string()).collect();
    out.insert(at, format!("DataPool {no}"));
    let mut out = out.join(" ");
    if let Some(tail) = tail {
        out.push(';');
        out.push_str(tail);
    }
    out
}

/// A DataPool verb that only works owner first (`Delete DataPool 2 Group 3`).
fn datapool_owner(state: &mut AppState, verb: Verb, args: &[&str]) -> DispatchResult {
    in_datapool(state, verb, args).unwrap_or_else(|| {
        out(crate::errors::render_code(
            format!("Name a pool object: {verb:?} DataPool <n> Group <m>"),
            "E-CMD-006",
        ))
    })
}
fn datapool_complete(state: &AppState, prefix: &str) -> Vec<String> {
    filter(
        state.root.show_data.datapools.keys().map(u32::to_string),
        prefix,
    )
}

pub static DATAPOOL: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "DataPool",
        lum: crate::objects::LumShape::ids(1).children(&[&crate::commands::groups::GROUP.decl, &crate::objects::playback::SEQUENCE.decl, &crate::objects::preset::PRESET.decl, &crate::objects::presets::TIMING_PRESET.decl, &crate::objects::presets::CURVE_PRESET.decl]),
        default_verb: crate::objects::Verb::Select,
        flags: &[],
        aliases: &["datapools", "pool", "pools"],
        doc: crate::commands::registry::Doc {
            display: "DataPool",
            category: crate::commands::help::HelpCategory::DataPool,
            also_applies_to: &[],
            syntax: "DataPool <id | name> | <verb> DataPool <id | name> Group <n> …",
            description: "A datapool: groups, presets, sequences, macros and other reusable show content. `Select DataPool <n>` makes it the one your pool objects resolve in (decisions §29); owner first, a line acts in another one without selecting it (`Delete DataPool 2 Group 3`) — what a pool window pinned to a DataPool sends.",
            examples: &["Select DataPool 2", "List DataPool", "Store DataPool 2 Group 3", "DataPool \"Busk\" Group 1"],
            related: &["group", "preset", "macro"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: Some(crate::commands::registry::CommandFamily::Selection),
        call_undo: crate::commands::registry::PROGRAMMER_SHOWDATA,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::content,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::List, datapool_list),
        (Verb::Info, crate::commands::typed::typed::<dp_info::InfoDataPool>),
        // `DataPool 2 Group 3` calls the group in DataPool 2; bare
        // `DataPool 2` lists, as before.
        (Verb::Select, datapool_select),
        (Verb::Store, |s, a| datapool_owner(s, Verb::Store, a)),
        (Verb::Delete, |s, a| datapool_owner(s, Verb::Delete, a)),
        (Verb::Edit, |s, a| datapool_owner(s, Verb::Edit, a)),
        (Verb::Copy, |s, a| datapool_owner(s, Verb::Copy, a)),
        (Verb::Update, |s, a| datapool_owner(s, Verb::Update, a)),
        (Verb::Move, |s, a| datapool_owner(s, Verb::Move, a)),
    ],
    delete_ask: None,
    complete: Some(datapool_complete),
    props: Some(&dp_props::DATAPOOL_PROPS),
    tree: None,
};

// ── Tree collections ─────────────────────────────────────────────────────────

use crate::vfs::{DynChild, NodeView, btree_to_children, object_view};
use lumacore_tree::Root;

/// The numbered fixtures, by number (decisions §40 5: a fixture without a number is not a
/// `Fixture n`; the resolver reaches it as `Fixture Media n` or by name).
fn fixtures_children(root: &Root) -> Vec<DynChild> {
    use lumacore_tree::TreeNode;
    root.show_data
        .devices
        .dmx
        .patch
        .by_number()
        .into_iter()
        .map(|(id, obj)| DynChild {
            display_name: obj.name().map(|n| format!("{} \"{}\"", id, n)).unwrap_or_else(|| format!("{}", id)),
            id,
            node_type: obj.node_type(),
            is_leaf: true,
            child_count: 0,
            name: obj.name().map(str::to_string),
            guid: Some(obj.guid()),
        })
        .collect()
}
fn fixture_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.devices.dmx.patch.fixture(id).map(|t| object_view(id, t))
}
fn fixture_types_children(root: &Root) -> Vec<DynChild> {
    btree_to_children(&root.show_data.devices.dmx.patch.fixture_types)
}
fn fixture_type_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data
        .devices
        .dmx
        .patch
        .fixture_types
        .get(&id)
        .map(|t| object_view(id, t))
}
fn universes_children(root: &Root) -> Vec<DynChild> {
    btree_to_children(&root.show_data.devices.dmx.patch.universes)
}
fn universe_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data
        .devices
        .dmx
        .patch
        .universes
        .get(&id)
        .map(|t| object_view(id, t))
}

// ── Writable properties (`Set Fixture 3 "Name" "Spot"`) ──────────────────────

use crate::props;

type Edit<'a> = &'a mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>;

fn fixture_write(root: &mut Root, _dp: Option<u32>, id: u32, edit: Edit) -> Result<(), String> {
    let fixture = root.show_data.devices.dmx.patch.fixture_mut(id).ok_or("no such fixture")?;
    props::edit_json(fixture, edit)
}

/// A fixture type's meta, written by the registry (`Set FixtureType 3 "Note" "…"`).
fn fixture_type_write(root: &mut Root, _dp: Option<u32>, id: u32, edit: Edit) -> Result<(), String> {
    let ft = root.show_data.devices.dmx.patch.fixture_types.get_mut(&id).ok_or("no such fixture type")?;
    props::edit_json(std::sync::Arc::make_mut(ft), edit)
}

fn universe_write(root: &mut Root, _dp: Option<u32>, id: u32, edit: Edit) -> Result<(), String> {
    let universe = root
        .show_data
        .devices
        .dmx
        .patch
        .universes
        .get_mut(&id)
        .ok_or("no such universe")?;
    // Slot occupancy isn't serialized: keep it across the round trip.
    let occupied = universe.occupied.clone();
    props::edit_json(universe, edit)?;
    universe.occupied = occupied;
    Ok(())
}
