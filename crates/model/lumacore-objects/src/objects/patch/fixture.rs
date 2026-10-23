// ███████╗██╗██╗  ██╗████████╗██╗   ██╗██████╗ ███████╗
// ██╔════╝██║╚██╗██╔╝╚══██╔══╝██║   ██║██╔══██╗██╔════╝
// █████╗  ██║ ╚███╔╝    ██║   ██║   ██║██████╔╝█████╗
// ██╔══╝  ██║ ██╔██╗    ██║   ██║   ██║██╔══██╗██╔══╝
// ██║     ██║██╔╝ ██╗   ██║   ╚██████╔╝██║  ██║███████╗
// ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: fixture

//! The Fixture's declared properties for its two parts (decisions §34 "34g4 patch hardened" 3–5): the ids
//! MVR carries and the place in the world. Each reads and writes its one home in `Fixture.Mvr`;
//! the operator names them flat (`Set Fixture 101 "PosX" 2.5`).
//!
//! Owns:          LUMA_PROPS (MasterReact), MVR_PROPS (Function, FixtureID, UnitNumber, IDType, CID),
//!                PLACE_PROPS (PosX…RotZ).
//! Doesn't own:   the matrix maths (`lumacore_fixtures::fixture_mvr`), Address (`objects::patch`).
//! Entry points:  LUMA_PROPS, MVR_PROPS, PLACE_PROPS.
//! Decisions:     §34 "34g4 patch hardened" 3, 4, 5.

use crate::AppState;
use crate::commands::registry::Rights;
use crate::props::{Access, PropKind, Property};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Where fixtures land on `At DMX …` (decisions §34 round 3 19).
pub(crate) mod address;
/// `Store Fixture …` (decisions §34 round 3 1, 19).
pub(crate) mod store;
/// `Fixture 201 Multipatch 1` (decisions §34 round 3 19).
pub(crate) mod multipatch;
pub(crate) mod info;

/// MVR's ID types a second id is typed with (`Fixture Houselights 4`, decisions §34 round 3 19):
/// all but None and Fixture (type 1 is the fixture number itself).
pub const ID_KINDS: &[&str] = &["Channel", "Generic", "Houselights", "NonDim", "Media", "Fog", "Effect", "Pyro", "Marker"];

/// What only Luma has (`Fixture.Luma`).
pub const LUMA_PROPS: [Property; 1] = [
    Property {
        name: "MasterReact",
        access: Access::Field("luma.master_react"),
        kind: PropKind::Choice(lumacore_fixtures::MasterReact::NAMES),
        rights: Rights::Program,
        doc: "which masters act on it: Grand (group + grand), Group (Additive group masters skip the grand master), None (no master, no Solo)",
        ..Property::BASE
    },
];

/// The ids MVR carries besides the number (which is the pool's key): Setup, like the patch.
pub const MVR_PROPS: [Property; 6] = [
    Property {
        name: "Function",
        access: Access::Field("mvr.function"),
        rights: Rights::Setup,
        section: "Patch",
        doc: "what it is for in the drawing (MVR Function)",
        ..Property::BASE
    },
    Property {
        name: "FixtureID",
        access: Access::Field("mvr.fixture_id"),
        rights: Rights::Setup,
        section: "Patch",
        doc: "the drawing's text id (MVR FixtureID); the number you select is the fixture's own",
        ..Property::BASE
    },
    Property {
        name: "UnitNumber",
        access: Access::Field("mvr.unit_number"),
        kind: PropKind::Int { min: 0, max: u32::MAX as i64 },
        rights: Rights::Setup,
        section: "Patch",
        doc: "the unit number in the drawing (MVR UnitNumber)",
        ..Property::BASE
    },
    Property {
        name: "FID",
        access: Access::Keyed { get: fid, set: set_fid },
        kind: PropKind::Int { min: 0, max: u32::MAX as i64 },
        rights: Rights::Setup,
        section: "Patch",
        doc: "its number, `Fixture 849` (MVR FixtureIDNumeric); 0 = none (a folder); unique in the show (decisions §40 7)",
        ..Property::BASE
    },
    Property {
        name: "IDType",
        access: Access::Field("mvr.id_type"),
        kind: PropKind::Choice(lumacore_fixtures::IdType::NAMES),
        rights: Rights::Setup,
        section: "Patch",
        doc: "the kind of its second id, from MVR's list (CustomIdType; Generic is MVR's reserved 3, the default of a CID); None = no second id",
        ..Property::BASE
    },
    Property {
        name: "CID",
        access: Access::Keyed { get: cid, set: set_cid },
        kind: PropKind::Int { min: 0, max: u32::MAX as i64 },
        rights: Rights::Setup,
        section: "Patch",
        doc: "its second id, a number in its IDType (MVR CustomId), `Fixture Media 7003`; 0 = none; unique within its IDType",
        ..Property::BASE
    },
];

/// Where it is, read from its one world matrix and written into it (point 4): metres and
/// degrees, decimals and relative input (`+ 0.5`).
pub static PLACE_PROPS: [Property; 6] = [
    place("PosX", "m", "where it is along x, in metres", get_pos_x, set_pos_x),
    place("PosY", "m", "where it is along y, in metres", get_pos_y, set_pos_y),
    place("PosZ", "m", "its height, in metres", get_pos_z, set_pos_z),
    place("RotX", "°", "its turn about x, in degrees (applied first)", get_rot_x, set_rot_x),
    place("RotY", "°", "its turn about y, in degrees", get_rot_y, set_rot_y),
    place("RotZ", "°", "its turn about z, in degrees (applied last)", get_rot_z, set_rot_z),
];

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Fixture <sel> [At <value>]` (the bare word and its At): a selection, a value on it.
pub(super) fn call(state: &mut AppState, args: &[&str]) -> crate::DispatchResult {
    if args.is_empty() {
        return crate::objects::out(crate::commands::list::fixtures(state, args));
    }
    // `Fixture 201 Multipatch 1 [At …]`: a copy selects its fixture (it has no values of its own).
    if let Some(words) = multipatch::fixtures_only(args) {
        return call(state, &words);
    }
    // `Fixture Houselights 4 [At …]`: by second id through the one resolver, then by number.
    if args.iter().any(|a| ID_KINDS.iter().any(|k| k.eq_ignore_ascii_case(a))) {
        let at = args.iter().position(|a| a.eq_ignore_ascii_case("at")).unwrap_or(args.len());
        let mut words = vec!["Fixture"];
        words.extend_from_slice(&args[..at]);
        // A plain selection goes by GUID: a folder has no number to retype (decisions §40 7, 11).
        if at == args.len() {
            return crate::objects::out(match crate::resolve::fixture_guid_words(state, &words) {
                Ok(guids) => select_guids(state, &guids),
                Err(e) => e,
            });
        }
        return match crate::resolve::fixture_words(state, &words) {
            Ok(nos) => {
                let nos: Vec<String> = nos.iter().map(u32::to_string).collect();
                let mut plain: Vec<&str> = nos.iter().flat_map(|n| ["+", n.as_str()]).skip(1).collect();
                plain.extend_from_slice(&args[at..]);
                crate::objects::out(crate::commands::control::cmd_fixture_at(state, &plain))
            }
            Err(e) => crate::objects::out(e),
        };
    }
    crate::objects::out(crate::commands::control::cmd_fixture_at(state, args))
}

/// A plain selection by GUID (`Fixture Media 7003`): the claim check, then the programmer.
fn select_guids(state: &mut AppState, guids: &[uuid::Uuid]) -> String {
    if let Err(e) = crate::programmers::ensure_unclaimed_guids(state, guids) {
        return e;
    }
    match state.select_members(guids, &[], lumacore_tree::programmer::SelectMode::Plain, true, lumacore_tree::GridCursorMove::Linked) {
        Ok(n) => crate::text::render_success(&format!("Selected {n} fixture{}", if n == 1 { "" } else { "s" })),
        Err(e) => crate::errors::render_code(e, "E-CMD-011"),
    }
}

/// `Delete Fixture <target> [/KeepChildren]`: they go (the patch claim; dispatch asked once
/// first). A fixture that holds fixtures takes them along — or, `/KeepChildren`, they move up to
/// its parent, their order kept (decisions §40 "40d hardened" 3).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "Fixture")]
pub struct DeleteFixture {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(option = "KeepChildren")]
    pub keep_children: bool,
}

impl crate::commands::typed::Command for DeleteFixture {
    fn run(self, state: &mut AppState) -> crate::DispatchResult {
        if let Err(e) = crate::objects::patch_claimed(state) {
            return e;
        }
        if let Some((fixtures, copies)) = multipatch::split(&self.target) {
            return crate::objects::out(multipatch::delete(state, &fixtures, copies.as_ref()).unwrap_or_else(|e| e));
        }
        let guids = match crate::resolve::fixture_guids(state, &self.target) {
            Ok(g) => g,
            Err(e) => return crate::objects::out(e),
        };
        crate::objects::out(delete_guids(state, &guids, self.keep_children))
    }
}

/// The one Delete question for fixtures: a node names what is under it, with both ways out.
pub fn delete_ask(state: &AppState, args: &[&str]) -> crate::objects::DeleteAsk {
    use crate::objects::DeleteAsk;
    use super::devices_meta::nodes;
    let words: Vec<&str> = args.iter().copied().filter(|w| !w.starts_with('/')).collect();
    let keep = args.iter().any(|w| crate::commands::flags::is(w, &crate::commands::flags::KEEP_CHILDREN));
    if multipatch::split(&match crate::resolve::target_of(&std::iter::once("Fixture").chain(words.iter().copied()).collect::<Vec<_>>().join(" ")) { Ok(t) => t, Err(e) => return DeleteAsk::Refuse(e) }).is_some() {
        return DeleteAsk::Plain;
    }
    let guids = match crate::resolve::fixture_guid_words(state, &std::iter::once("Fixture").chain(words.iter().copied()).collect::<Vec<_>>()) {
        Ok(g) => g,
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let devices = &state.root.show_data.devices;
    let folders: usize = guids.iter().map(|g| nodes::folders_under(devices, *g)).sum();
    let fixtures: usize = guids.iter().map(|g| nodes::fixtures_under(devices, *g).len()).sum();
    if folders + fixtures == 0 {
        return DeleteAsk::Plain;
    }
    let what = match guids.as_slice() {
        [g] => nodes::node_label(devices, *g),
        _ => format!("{} fixtures", guids.len()),
    };
    let s = |n: usize| if n == 1 { "" } else { "s" };
    let under = match (folders, fixtures) {
        (0, f) => format!("{f} fixture{} under it", s(f)),
        (d, 0) => format!("{d} folder{} under it", s(d)),
        (d, f) => format!("{d} folder{} and {f} fixture{} under it", s(d), s(f)),
    };
    let line = format!("Delete Fixture {}", words.join(" "));
    let ask = crate::dialogs::Ask::new("Delete", format!("Delete {what} — {under}?"));
    let ask = if keep {
        ask.button("Delete, keep them", lumacore_wire::ButtonKind::Danger, format!("{line} /KeepChildren /NoConfirmation"))
    } else {
        let up = guids
            .first()
            .and_then(|g| nodes::parent_of(devices, *g))
            .map(|p| nodes::node_label(devices, p))
            .unwrap_or_else(|| "the top".to_string());
        ask.button("Delete all", lumacore_wire::ButtonKind::Danger, format!("{line} /NoConfirmation"))
            .button(&format!("Keep them: move up to {up}"), lumacore_wire::ButtonKind::Primary, format!("{line} /KeepChildren /NoConfirmation"))
    };
    DeleteAsk::Ask(Box::new(ask.cancel("Cancel").flags("/NoConfirmation")))
}

/// Delete fixtures by GUID: a node's subtree with it, or lifted to its parent.
fn delete_guids(state: &mut AppState, guids: &[uuid::Uuid], keep_children: bool) -> String {
    use super::devices_meta::nodes;
    let mut lines: Vec<String> = Vec::new();
    for g in guids {
        let devices = &state.root.show_data.devices;
        let Some((_, f)) = devices.dmx.patch.fixture_by_guid(*g) else { continue };
        let label = nodes::node_label(devices, *g);
        let under = nodes::nodes_under(devices, *g);
        if keep_children {
            let up = f.mvr.parent;
            let patch = &mut state.root.show_data.devices.dmx.patch;
            let mut order = patch.next_order(up);
            let direct: Vec<uuid::Uuid> = under.iter().copied().filter(|c| patch.fixture_by_guid(*c).is_some_and(|(_, x)| x.mvr.parent == Some(*g))).collect();
            for c in direct {
                if let Some((_, x)) = patch.fixture_by_guid_mut(c) {
                    x.mvr.parent = up;
                    x.mvr.order = order;
                    order += 10;
                }
            }
        } else {
            for c in under.iter().rev() {
                remove_guid(state, *c);
            }
        }
        remove_guid(state, *g);
        let went = if keep_children { String::new() } else { format!(" with {} under it", under.len()).replace(" with 0 under it", "") };
        lines.push(crate::text::render_success(&format!("Deleted {label}{went}.")));
    }
    if lines.is_empty() { crate::errors::render_code("No such Fixture.", "E-CMD-007") } else { lines.join("\n") }
}

/// One fixture out, numbered or not: its addresses released, its number back to the pool.
fn remove_guid(state: &mut AppState, g: uuid::Uuid) {
    let show = &mut state.root.show_data;
    match show.devices.dmx.patch.fixture_by_guid(g).and_then(|(_, f)| f.mvr.fid) {
        Some(no) => {
            let _ = lumacore_patch::PatchEngine::unpatch_fixture(show, no);
        }
        None => {
            if let Some((slot, _)) = show.devices.dmx.patch.fixture_by_guid(g) {
                show.devices.dmx.patch.remove_fixture(slot);
            }
        }
    }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The fixtures `kind` + `ids` name by their second ids (`Fixture Houselights 1 Thru 4`, decisions
/// §34 round 3 19), in typed order: (number, GUID, name). A range walks the second ids that exist.
pub(crate) fn by_second_id(state: &AppState, kind: &str, ids: &lumacore_syntax::ast::Ids) -> Result<Vec<(u32, uuid::Uuid, Option<String>)>, String> {
    use lumacore_syntax::ast::{Id, Range, SetOp};
    let ty = lumacore_fixtures::IdType::from_name(kind).ok_or_else(|| crate::errors::render_code(format!("{kind} isn't an ID type."), "E-CMD-008"))?;
    // (cid, the number a line names it by — 0 for a folder —, GUID, name).
    let mut all: Vec<(u32, u32, uuid::Uuid, Option<String>)> = Vec::new();
    for f in state.root.show_data.devices.dmx.patch.fixtures.values() {
        for (_, cid) in f.second_ids().filter(|(t, _)| *t == ty) {
            all.push((cid, f.mvr.fid.unwrap_or(0), f.meta.identity.guid, f.meta.identity.name.clone()));
        }
    }
    all.sort_by_key(|(cid, no, ..)| (*cid, *no));
    let number = |id: &Option<Id>, edge: u32| match id {
        Some(Id::Number(n)) => n.node.parse::<u32>().map_err(|_| crate::errors::render_code(format!("{kind} {} isn't a number.", n.node), "E-CMD-008")),
        Some(Id::Name(n)) => Err(crate::errors::render_code(format!("{kind} takes numbers, not \"{}\".", n.node), "E-CMD-008")),
        None => Ok(edge),
    };
    let mut out: Vec<(u32, uuid::Uuid, Option<String>)> = Vec::new();
    for (op, range) in &ids.ranges {
        let (from, to) = match range {
            Range::One(id) => {
                let n = number(&Some(id.clone()), 0)?;
                (n, n)
            }
            Range::Thru { from, to, .. } => (number(from, 0)?, number(to, u32::MAX)?),
        };
        let these = all.iter().filter(|(cid, ..)| (from..=to).contains(cid)).map(|(_, no, g, n)| (*no, *g, n.clone()));
        match op.as_ref().map(|o| o.node) {
            Some(SetOp::Minus) => {
                let gone: Vec<u32> = these.map(|(no, ..)| no).collect();
                out.retain(|(no, ..)| !gone.contains(no));
            }
            _ => {
                for m in these {
                    if !out.iter().any(|(no, ..)| *no == m.0) {
                        out.push(m);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// `"FID"`: the number, 0 = none.
fn fid(state: &AppState, key: &str) -> String {
    fixture_of(state, key).and_then(|f| f.mvr.fid).unwrap_or(0).to_string()
}

/// A new number: free in the show, or 0 to take the number away (a folder). The line keeps
/// naming the fixture by the number it had.
fn set_fid(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let n: u32 = value.trim().parse().map_err(|_| crate::errors::render_code(format!("'{value}' isn't a fixture number."), "E-CMD-008"))?;
    let guid = fixture_of(state, key).map(|f| f.meta.identity.guid).ok_or_else(|| crate::errors::render_code(format!("No Fixture {key}."), "E-CMD-007"))?;
    let patch = &mut state.root.show_data.devices.dmx.patch;
    if n != 0 && patch.fixture_by_fid(n).is_some_and(|(_, f)| f.meta.identity.guid != guid) {
        return Err(crate::errors::render_code(format!("Fixture {n} exists: a number is one fixture's."), "E-CMD-010"));
    }
    let (_, f) = patch.fixture_by_guid_mut(guid).expect("found above");
    let was = f.mvr.fid;
    f.mvr.fid = (n != 0).then_some(n);
    if let Some(w) = was {
        patch.fixture_id_pool.release(w);
    }
    if n != 0 {
        patch.fixture_id_pool.reserve(n);
    }
    Ok(format!("Fixture {key} FID = {n}"))
}

/// `"CID"`: the second id's number, 0 = none.
fn cid(state: &AppState, key: &str) -> String {
    fixture_of(state, key).and_then(|f| f.mvr.cid).unwrap_or(0).to_string()
}

/// A new second id: free within its IDType (Generic when the fixture has none yet, decisions
/// §40 10), or 0 to take it away.
fn set_cid(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let n: u32 = value.trim().parse().map_err(|_| crate::errors::render_code(format!("'{value}' isn't a number."), "E-CMD-008"))?;
    let (guid, ty) = fixture_of(state, key).map(|f| (f.meta.identity.guid, f.mvr.id_type)).ok_or_else(|| crate::errors::render_code(format!("No Fixture {key}."), "E-CMD-007"))?;
    let ty = if ty == lumacore_fixtures::IdType::None { lumacore_fixtures::IdType::Generic } else { ty };
    let patch = &mut state.root.show_data.devices.dmx.patch;
    if n != 0 && patch.fixture_by_id(ty, n).is_some_and(|(_, f)| f.meta.identity.guid != guid) {
        return Err(crate::errors::render_code(format!("Fixture {} {n} exists: a second id is one fixture's within its type.", ty.name()), "E-CMD-010"));
    }
    let (_, f) = patch.fixture_by_guid_mut(guid).expect("found above");
    f.mvr.cid = (n != 0).then_some(n);
    f.mvr.id_type = if n == 0 { f.mvr.id_type } else { ty };
    Ok(format!("Fixture {key} CID = {} {n}", ty.name()))
}

/// The fixture a property key names: its number as text (the resolver's row), else its GUID.
fn fixture_of<'a>(state: &'a AppState, key: &str) -> Option<&'a lumacore_fixtures::Fixture> {
    let patch = &state.root.show_data.devices.dmx.patch;
    match key.parse::<u32>() {
        Ok(n) => patch.fixture(n),
        Err(_) => key.parse::<uuid::Uuid>().ok().and_then(|g| patch.fixture_by_guid(g)).map(|(_, f)| f),
    }
}

const fn place(
    name: &'static str,
    unit: &'static str,
    doc: &'static str,
    get: fn(&AppState, &str) -> String,
    set: fn(&mut AppState, &str, &str) -> Result<String, String>,
) -> Property {
    Property {
        name,
        kind: PropKind::Declared,
        rights: Rights::Program,
        section: "Position",
        unit,
        doc,
        access: Access::Keyed { get, set },
        ..Property::BASE
    }
}

/// Position (0–2) or rotation (3–5) on one axis.
#[derive(Clone, Copy)]
enum Axis {
    Pos(usize),
    Rot(usize),
}

macro_rules! axis_fns {
    ($($get:ident, $set:ident = $axis:expr;)*) => {$(
        fn $get(s: &AppState, k: &str) -> String {
            read(s, k, $axis)
        }
        fn $set(s: &mut AppState, k: &str, v: &str) -> Result<String, String> {
            write(s, k, $axis, v)
        }
    )*};
}

axis_fns! {
    get_pos_x, set_pos_x = Axis::Pos(0);
    get_pos_y, set_pos_y = Axis::Pos(1);
    get_pos_z, set_pos_z = Axis::Pos(2);
    get_rot_x, set_rot_x = Axis::Rot(0);
    get_rot_y, set_rot_y = Axis::Rot(1);
    get_rot_z, set_rot_z = Axis::Rot(2);
}

fn matrix(s: &AppState, k: &str) -> Option<lumacore_fixtures::Matrix> {
    let n = k.parse::<u32>().ok()?;
    s.root.show_data.devices.dmx.patch.fixture(n).map(|f| f.mvr.matrix)
}

fn value(m: &lumacore_fixtures::Matrix, axis: Axis) -> f64 {
    match axis {
        Axis::Pos(i) => m.position_m()[i],
        Axis::Rot(i) => m.rotation_deg()[i],
    }
}

fn read(s: &AppState, k: &str, axis: Axis) -> String {
    matrix(s, k).map_or_else(String::new, |m| lumacore_fixtures::fixture_mvr::decimal(value(&m, axis)))
}

fn write(s: &mut AppState, k: &str, axis: Axis, raw: &str) -> Result<String, String> {
    use crate::errors::render_code;
    let Some(m) = matrix(s, k) else {
        return Err(render_code(format!("No Fixture {k}."), "E-CMD-007"));
    };
    let now = value(&m, axis);
    let next = match crate::props::relative_f64(raw, now) {
        Some(r) => r.map_err(|e| render_code(format!("Fixture {k}: {e}"), "E-CMD-008"))?,
        None => raw.trim().parse::<f64>().ok().filter(|v| v.is_finite()).ok_or_else(|| {
            render_code(format!("Fixture {k} takes a number here — not '{raw}'."), "E-CMD-008")
        })?,
    };
    let Some(f) = k.parse::<u32>().ok().and_then(|n| s.root.show_data.devices.dmx.patch.fixture_mut(n)) else {
        return Err(render_code(format!("No Fixture {k}."), "E-CMD-007"));
    };
    match axis {
        Axis::Pos(i) => f.mvr.matrix.set_position_m(i, next),
        Axis::Rot(i) => f.mvr.matrix.set_rotation_deg(i, next),
    }
    Ok(String::new())
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
