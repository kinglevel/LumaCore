// ██████╗  ██████╗  ██████╗ ██╗     ██╗  ██╗███╗   ███╗██╗
// ██╔══██╗██╔═══██╗██╔═══██╗██║     ╚██╗██╔╝████╗ ████║██║
// ██████╔╝██║   ██║██║   ██║██║      ╚███╔╝ ██╔████╔██║██║
// ██╔═══╝ ██║   ██║██║   ██║██║      ██╔██╗ ██║╚██╔╝██║██║
// ██║     ╚██████╔╝╚██████╔╝███████╗██╔╝ ██╗██║ ╚═╝ ██║███████╗
// ╚═╝      ╚═════╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝
// FILE: poolxml

//! `Import PoolXml "file" Groups [/Drive "usb1"] [/DataPool n]` (decisions §40 13): a console's
//! pool export as XML, read by `lumacore-poolxml`, one LumaCore group per exported group in your
//! selected DataPool (`/DataPool` picks another), members by fixture number with their grid cells, the
//! file's order kept; a group whose items are all sub-fixtures is reported and skipped (a fixture
//! is a fixture, §40 3). Presets, sequences, cues and macros follow when their objects exist.
//!
//! Owns:          the PoolXml object, Import PoolXml.
//! Doesn't own:   the reader (`lumacore-poolxml`), the groups themselves (`commands::groups`).
//! Entry points:  POOL_XML, ImportPoolXml.
//! Decisions:     §40 13 (2026-10-20).

use lumacore_syntax::ast::{Id, Item, Range, Target, Value};
use lumacore_tree::{GridCell, Group, GroupMember};

use crate::commands::flags as F;
use crate::errors::render_code;
use crate::objects::{ObjectDef, Verb, out};
use crate::{AppState, DispatchResult};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

pub const DATA_POOL: F::FlagDef = F::FlagDef::new("DataPool", "The DataPool the groups go into (yours when none)").value("n");

pub static POOL_XML: ObjectDef = ObjectDef::new(crate::objects::ObjectDecl {
    keyword: "PoolXml",
    lum: crate::objects::LumShape::ids(1),
    default_verb: Verb::Info,
    flags: &[F::DRIVE.on(&[Verb::Import]), DATA_POOL.on(&[Verb::Import])],
    aliases: &[],
    doc: crate::commands::registry::Doc::new("PoolXml", "A console's pool export as XML: Import PoolXml \"file\" Groups makes its groups in a DataPool"),
    family: None,
    call_undo: crate::commands::registry::Undo::ReadOnly,
    call_class: crate::commands::registry::CommandClass::Commit,
    scope: crate::objects::on_master,
    reach: |v| if v == Verb::Import { crate::commands::registry::Reach::Drive } else { crate::commands::registry::Reach::Show },
    rights: crate::rights::content,
    planned: &[],
})
.verbs(&[(Verb::Import, crate::commands::typed::typed::<ImportPoolXml>)]);

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Import PoolXml "file" Groups [/Drive "usb1"] [/DataPool n]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Import", object = "PoolXml")]
pub struct ImportPoolXml {
    #[lum(target)]
    pub target: Option<Target>,
    /// What to take of the pool: `Groups` (the rest when their objects exist).
    #[lum(value)]
    pub what: Option<Value>,
    #[lum(option = "Drive")]
    pub drive: Option<Value>,
    #[lum(option = "DataPool")]
    pub datapool: Option<Value>,
}

impl crate::commands::typed::Command for ImportPoolXml {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(import(state, &self).unwrap_or_else(|e| e))
    }
}

fn import(state: &mut AppState, cmd: &ImportPoolXml) -> Result<String, String> {
    let usage = || render_code("Import PoolXml \"file.xml\" Groups [/Drive \"usb1\"] [/DataPool n]: a pool export (XML) of library/poolxml/.", "E-CMD-006");
    let Some(file) = cmd.target.as_ref().and_then(file_name) else { return Err(usage()) };
    let what = cmd.what.as_ref().map(word).unwrap_or_default();
    if !what.eq_ignore_ascii_case("groups") {
        return Err(render_code(format!("Import PoolXml takes Groups for now, not {}: presets, sequences and macros come with their objects (decisions §40 13).", if what.is_empty() { "nothing".to_string() } else { what }), "E-CMD-008"));
    }
    let drive = cmd.drive.as_ref().map(word);
    let folder = state.library_folder(drive.as_deref(), "poolxml", false)?;
    let path = lumacore_luma::drive::library_file(&folder, &file).map_err(|e| render_code(e, "E-CMD-008"))?;
    let open = lumacore_luma::drive::open(&path).map(std::io::BufReader::new).map_err(|e| render_code(format!("No export \"{file}\" in {folder}/: {e}"), "E-CMD-012"))?;
    let pool = lumacore_poolxml::read_from(open).map_err(|e| render_code(format!("\"{file}\": {e}"), "E-CMD-008"))?;
    let dp = match &cmd.datapool {
        None => state.selected_datapool(),
        Some(v) => {
            let n: u32 = word(v).parse().map_err(|_| render_code("/DataPool takes a DataPool number.", "E-CMD-008"))?;
            if !state.root.show_data.datapools.contains_key(&n) {
                return Err(render_code(format!("No DataPool {n}."), "E-CMD-007"));
            }
            n
        }
    };
    let by_fid: std::collections::BTreeMap<u32, uuid::Uuid> = state.root.show_data.devices.dmx.patch.by_number().iter().map(|(n, f)| (*n, f.meta.identity.guid)).collect();
    let mut made: Vec<u32> = Vec::new();
    let mut subs_only: Vec<String> = Vec::new();
    let (mut sub_items, mut unknown) = (0usize, 0usize);
    let datapools = &mut state.root.show_data.datapools;
    let pool_data = datapools.entry(dp).or_insert_with(|| lumacore_tree::Datapool::new(Some("Default".to_string())));
    for g in &pool.groups {
        if g.sub_fixtures_only() {
            subs_only.push(g.name.clone());
            continue;
        }
        let mut members: Vec<GroupMember> = Vec::new();
        for item in &g.items {
            let Some(fid) = item.fid() else {
                sub_items += 1;
                continue;
            };
            let Some(guid) = by_fid.get(&fid) else {
                unknown += 1;
                continue;
            };
            if members.iter().any(|m| m.fixture == *guid) {
                continue;
            }
            members.push(GroupMember { fixture: *guid, cell: GridCell { x: item.x, y: item.y, z: item.z, ..GridCell::default() } });
        }
        let no = crate::pools::door::next_free(&pool_data.groups.keys().copied().collect::<Vec<_>>());
        let mut group = Group::new(Some(g.name.clone()).filter(|n| !n.is_empty()));
        group.members = members;
        pool_data.groups.insert(no, group);
        made.push(no);
    }
    state.show_generation += 1;
    let pool_name = pool_data.meta.name().unwrap_or("").to_string();
    let mut parts = vec![format!(
        "Imported {} group{} ({}) into DataPool {dp} \"{pool_name}\" from pool \"{}\" (version {})",
        made.len(),
        if made.len() == 1 { "" } else { "s" },
        crate::commands::door::numbers_text(&made),
        pool.name,
        pool.data_version
    )];
    if !subs_only.is_empty() {
        parts.push(format!("skipped {} of sub-fixtures only: {}", subs_only.len(), subs_only.join(", ")));
    }
    if sub_items > 0 || unknown > 0 {
        parts.push(format!("left out {sub_items} sub-fixture item{} and {unknown} fixture{} the patch lacks", if sub_items == 1 { "" } else { "s" }, if unknown == 1 { "" } else { "s" }));
    }
    let rest: Vec<String> = pool.other.iter().filter(|(_, n)| *n > 0).map(|(k, n)| format!("{n} {}", k.to_lowercase())).collect();
    if !rest.is_empty() {
        parts.push(format!("in the file, not read yet: {}", rest.join(", ")));
    }
    Ok(crate::text::render_success(&format!("{}.", parts.join("; "))))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The quoted file name the target carries (`Import PoolXml "default.xml" …`).
fn file_name(target: &Target) -> Option<String> {
    let one = |ids: &lumacore_syntax::ast::Ids| match ids.ranges.as_slice() {
        [(None, Range::One(Id::Name(n)))] => Some(n.node.clone()),
        _ => None,
    };
    match target.items.as_slice() {
        [(None, Item::Path(segs))] => segs.last().and_then(|s| s.ids.as_ref()).and_then(one),
        [(None, Item::Ids(ids))] => one(ids),
        _ => None,
    }
}

/// A line value as a word (the MVR import's reading).
fn word(v: &Value) -> String {
    crate::objects::patch::mvr::word(v)
}

#[cfg(test)]
mod tests;
