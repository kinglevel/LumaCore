// ██████╗ ███████╗██╗   ██╗██╗███████╗██╗    ██╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██║    ██║
// ██████╔╝█████╗  ██║   ██║██║█████╗  ██║ █╗ ██║
// ██╔══██╗██╔══╝  ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║
// ██║  ██║███████╗ ╚████╔╝ ██║███████╗╚███╔███╔╝
// ╚═╝  ╚═╝╚══════╝  ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝
// FILE: review

//! The review's rule (decisions §34 "Import MVR hardened" 3, "34g4 patch hardened" 10–11): the
//! file's fixtures against the draft — matched by UUID, then FID, then name (or one key only),
//! within a scope —, each field compared with the live value, the preset saying whether a change
//! is taken, offered or hidden; new fixtures, lost fixtures, matches outside the scope, clashes
//! of address or number. Pure: devices and the file in, the review out.
//!
//! Owns:          build, Scope, the compare rules (position within 1 mm, rotation within 0.5°).
//! Doesn't own:   reading the file, the settings' source, writing (Apply).
//! Entry points:  build.
//! Decisions:     §34 "Import MVR hardened" 3.

use std::collections::{BTreeMap, BTreeSet};

use lumacore_fixtures::fixture_mvr::Matrix;
use lumacore_mvr::{MvrFile, MvrFixture};
use lumacore_tree::{DevicesData, MvrMatch, MvrRotation, MvrTake};
use uuid::Uuid;

use super::Settings;
use crate::mvr_review::{ImportReview, ReviewCell, ReviewRow, RowStatus};
// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES


/// What the import works on (decision 11): an MVR layer or group (the file's and the draft's, by
/// UUID), a class, or a tag of the draft's fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Node(Uuid),
    Class(Uuid),
    Tag(Uuid),
}

/// What the draft knows, indexed once.
struct Draft<'a> {
    devices: &'a DevicesData,
    by_guid: BTreeMap<Uuid, u32>,
    by_name: BTreeMap<String, Vec<u32>>,
    universe_no: BTreeMap<Uuid, u32>,
    /// The draft's MVR nodes: each node's parent (a layer has none).
    parent_of: BTreeMap<Uuid, Option<Uuid>>,
    node_name: BTreeMap<Uuid, String>,
    /// Every draft fixture's slots: (universe, start, end) per break, by number.
    runs: BTreeMap<u32, Vec<(u32, u16, u16)>>,
}
// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC


/// The review of `file` against `devices`. `gdtf_hashes`: the SHA-256 of each GDTF the zip holds,
/// by spec; `file_name` / `drive` / `scope_text` are what the line said.
pub fn build(devices: &DevicesData, file: &MvrFile, settings: &Settings, scope: Option<&Scope>, gdtf_hashes: &BTreeMap<String, String>, file_name: &str, drive: Option<&str>, scope_text: Option<&str>) -> ImportReview {
    let draft = Draft::index(devices);
    // The draft's fixtures without a number (folders), by uuid: a file fixture that is one of
    // them is Same, never made again (decisions §40 7).
    let folders: BTreeSet<Uuid> = devices.dmx.patch.fixtures.values().filter(|f| f.mvr.fid.is_none()).map(|f| f.meta.identity.guid).collect();
    let file_in_scope = file_scope(file, &draft, scope);
    let draft_in_scope = draft_scope(&draft, scope);
    let mut rows = Vec::new();
    let mut matched_draft: BTreeSet<u32> = BTreeSet::new();
    // 1. Every file fixture in scope: matched or new.
    for f in &file.scene.fixtures {
        if !file_in_scope.contains(&f.uuid) {
            continue;
        }
        let (matched, key) = draft.matching(f, settings);
        if let Some(no) = matched {
            matched_draft.insert(no);
        }
        let mut row = row_of(&draft, file, f, settings, matched, key, draft_in_scope.as_ref(), gdtf_hashes);
        if folders.contains(&f.uuid) {
            row.status = RowStatus::Same;
            row.matched = Some(f.uuid);
            row.match_key = "UUID".into();
            row.taken = false;
        }
        rows.push(row);
    }
    // 2. New rows: a number another new row or an unmatched draft fixture holds is a clash. A
    // folder with no id at all (an exporter may let a tree node go without a CID) gets the next free
    // second id of the preset's type, shown as it will be (decisions §40 9, 10).
    let mut taken_numbers: BTreeSet<u32> = BTreeSet::new();
    let folder_type = if settings.id_type == lumacore_fixtures::IdType::None { lumacore_fixtures::IdType::Generic } else { settings.id_type };
    let mut next_cid = devices
        .dmx
        .patch
        .fixtures
        .values()
        .filter(|f| f.mvr.id_type == folder_type)
        .filter_map(|f| f.mvr.cid)
        .chain(rows.iter().filter(|r| r.id_type == folder_type.name()).filter_map(|r| r.cid))
        .max()
        .map_or(1, |n| n + 1);
    let is_folder = |uuid: Uuid| file.scene.fixtures.iter().any(|f| f.parent == uuid);
    for r in rows.iter_mut() {
        if r.status != RowStatus::New {
            continue;
        }
        if r.no == 0 && r.cid.is_none() && is_folder(r.uuid) {
            r.cid = Some(next_cid);
            r.id_type = folder_type.name().to_string();
            r.id = format!("{} {next_cid}", folder_type.name());
            next_cid += 1;
        }
        let by_draft = draft.devices.dmx.patch.has_fid(r.no) && !matched_draft.contains(&r.no);
        if r.no == 0 && r.cid.is_none() {
            r.clash = Some("no fixture number: the file has none (FidOffset gives one)".into());
        } else if r.no == 0 {
            // A folder (decisions §40 7): no number, its second id is the handle; one per type.
            let ty = lumacore_fixtures::IdType::from_name(&r.id_type).unwrap_or_default();
            if let (Some(cid), true) = (r.cid, ty != lumacore_fixtures::IdType::None)
                && draft.devices.dmx.patch.fixture_by_id(ty, cid).is_some_and(|(_, f)| f.meta.identity.guid != r.uuid)
            {
                r.clash = Some(format!("{} exists and the file doesn't match it", r.id));
            }
        } else if by_draft {
            r.clash = Some(format!("Fixture {} exists and the file doesn't match it", r.no));
        } else if !taken_numbers.insert(r.no) {
            r.clash = Some(format!("Fixture {} twice in the file", r.no));
        }
        if r.clash.is_some() {
            r.status = RowStatus::Clash;
            r.taken = false;
        }
    }
    // 3. Lost: draft fixtures in the MVR tree, in scope, no file fixture matched.
    for fx in draft.devices.dmx.patch.fixtures.values() {
        // A draft fixture without a number (a folder) is never a Lost row (decisions §40 7).
        let Some(no) = fx.mvr.fid else { continue };
        if matched_draft.contains(&no) || fx.mvr.parent.is_none() {
            continue;
        }
        if let Some(s) = &draft_in_scope
            && !s.contains(&no)
        {
            continue;
        }
        rows.push(ReviewRow {
            no,
            id: String::new(),
            id_type: String::new(),
            cid: None,
            parent_fixture: None,
            order: fx.mvr.order,
            uuid: fx.meta.identity.guid,
            status: RowStatus::Lost,
            matched: Some(fx.meta.identity.guid),
            match_key: String::new(),
            name: fx.meta.identity.name.clone().unwrap_or_default(),
            gdtf_spec: String::new(),
            gdtf_mode: String::new(),
            gdtf_embedded: false,
            type_known: true,
            group: fx.mvr.parent,
            group_name: fx.mvr.parent.and_then(|p| draft.node_name.get(&p)).cloned().unwrap_or_default(),
            layer: Uuid::nil(),
            layer_name: String::new(),
            group_missing: false,
            cells: Vec::new(),
            taken: settings.lost == lumacore_tree::MvrLost::Delete,
            clash: None,
        });
    }
    rows.sort_by_key(|r| (r.no, r.uuid));
    ImportReview {
        file: file_name.to_string(),
        drive: drive.map(str::to_string),
        scope: scope_text.map(str::to_string),
        settings: settings.for_review(),
        rows,
        answers: Default::default(),
    }
}

/// The file fixtures the scope takes: all without one; under the node (its chain) for a node;
/// of the class; for a tag, the ones matching a tagged draft fixture plus their group mates.
fn file_scope(file: &MvrFile, draft: &Draft, scope: Option<&Scope>) -> BTreeSet<Uuid> {
    let all: BTreeSet<Uuid> = file.scene.fixtures.iter().map(|f| f.uuid).collect();
    let Some(scope) = scope else { return all };
    // The file's tree: a node's parent.
    let mut parent: BTreeMap<Uuid, Option<Uuid>> = BTreeMap::new();
    for l in &file.scene.layers {
        parent.insert(l.uuid, None);
    }
    for g in &file.scene.groups {
        parent.insert(g.uuid, Some(g.parent));
    }
    for f in &file.scene.fixtures {
        parent.insert(f.uuid, Some(f.parent));
    }
    let under = |mut at: Uuid, node: Uuid| {
        let mut hops = 0;
        while hops < 64 {
            if at == node {
                return true;
            }
            match parent.get(&at) {
                Some(Some(p)) => at = *p,
                _ => return false,
            }
            hops += 1;
        }
        false
    };
    match scope {
        Scope::Node(n) => file.scene.fixtures.iter().filter(|f| under(f.uuid, *n)).map(|f| f.uuid).collect(),
        Scope::Class(c) => file.scene.fixtures.iter().filter(|f| f.classing == Some(*c)).map(|f| f.uuid).collect(),
        Scope::Tag(t) => {
            let tagged: Vec<&lumacore_fixtures::Fixture> = draft.devices.dmx.patch.fixtures.values().filter(|f| f.meta.annotation.tags.contains(t)).collect();
            let mut out = BTreeSet::new();
            let mut groups = BTreeSet::new();
            for f in &file.scene.fixtures {
                let hit = tagged.iter().any(|d| d.meta.identity.guid == f.uuid || f.fixture_id_numeric.is_some_and(|n| draft.by_guid.get(&d.meta.identity.guid) == Some(&n)) || d.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(&f.name)));
                if hit {
                    out.insert(f.uuid);
                    groups.insert(f.parent);
                }
            }
            for f in &file.scene.fixtures {
                if groups.contains(&f.parent) {
                    out.insert(f.uuid);
                }
            }
            out
        }
    }
}

/// The draft fixtures the scope takes (None = all).
fn draft_scope(draft: &Draft, scope: Option<&Scope>) -> Option<BTreeSet<u32>> {
    let scope = scope?;
    let under = |mut at: Option<Uuid>, node: Uuid| {
        let mut hops = 0;
        while let Some(n) = at.filter(|_| hops < 64) {
            if n == node {
                return true;
            }
            at = draft.parent_of.get(&n).copied().flatten();
            hops += 1;
        }
        false
    };
    Some(
        draft
            .devices
            .dmx
            .patch
            .fixtures
            .iter()
            .filter(|(_, f)| match scope {
                Scope::Node(n) => under(f.mvr.parent, *n),
                Scope::Class(c) => f.mvr.class == Some(*c),
                Scope::Tag(t) => f.meta.annotation.tags.contains(t),
            })
            .filter_map(|(_, f)| f.mvr.fid)
            .collect(),
    )
}

impl<'a> Draft<'a> {
    fn index(devices: &'a DevicesData) -> Self {
        let patch = &devices.dmx.patch;
        let mvr = &devices.metadata.mvr;
        let mut by_name: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for f in patch.fixtures.values() {
            if let (Some(n), Some(no)) = (&f.meta.identity.name, f.mvr.fid) {
                by_name.entry(n.to_lowercase()).or_default().push(no);
            }
        }
        let universe_no: BTreeMap<Uuid, u32> = patch.universes.iter().map(|(n, u)| (u.meta.identity.guid, *n)).collect();
        let mut parent_of = BTreeMap::new();
        let mut node_name = BTreeMap::new();
        for l in mvr.layers.values() {
            parent_of.insert(l.meta.identity.guid, None);
            node_name.insert(l.meta.identity.guid, l.meta.identity.name.clone().unwrap_or_default());
        }
        for g in mvr.groups.values() {
            parent_of.insert(g.meta.identity.guid, Some(g.parent));
            node_name.insert(g.meta.identity.guid, g.meta.identity.name.clone().unwrap_or_default());
        }
        let runs = patch
            .fixtures
            .values()
            .filter_map(|f| {
                let r = f.mvr.breaks.iter().filter_map(|b| universe_no.get(&b.universe_id).map(|u| (*u, b.start_address, b.end_address()))).collect();
                f.mvr.fid.map(|no| (no, r))
            })
            .collect();
        Self {
            devices,
            by_guid: patch.fixtures.values().filter_map(|f| f.mvr.fid.map(|n| (f.meta.identity.guid, n))).collect(),
            by_name,
            universe_no,
            parent_of,
            node_name,
            runs,
        }
    }

    /// The draft fixture a file fixture matches, and by which key.
    fn matching(&self, f: &MvrFixture, s: &Settings) -> (Option<u32>, &'static str) {
        let by_uuid = || self.by_guid.get(&f.uuid).copied();
        let by_fid = || f.fixture_id_numeric.and_then(|n| u32::try_from(n as i64 + s.fid_offset).ok()).filter(|n| self.devices.dmx.patch.has_fid(n));
        let by_name = || self.by_name.get(&f.name.to_lowercase()).filter(|v| v.len() == 1 && !f.name.trim().is_empty()).map(|v| v[0]);
        match s.match_key {
            MvrMatch::Uuid => (by_uuid(), "UUID"),
            MvrMatch::Fid => (by_fid(), "FID"),
            MvrMatch::FixtureName => (by_name(), "Name"),
            MvrMatch::Auto => {
                if let Some(n) = by_uuid() {
                    (Some(n), "UUID")
                } else if let Some(n) = by_fid() {
                    (Some(n), "FID")
                } else if let Some(n) = by_name() {
                    (Some(n), "Name")
                } else {
                    (None, "")
                }
            }
        }
    }

    /// Whose run (universe, start..=end) overlaps, other than `except`.
    fn overlapping(&self, universe: u32, start: u16, end: u16, except: Option<u32>) -> Option<u32> {
        self.runs.iter().find(|(no, runs)| Some(**no) != except && runs.iter().any(|(u, s, e)| *u == universe && *s <= end && start <= *e)).map(|(no, _)| *no)
    }
}

/// A matched or new file fixture's row.
#[allow(clippy::too_many_arguments)]
fn row_of(draft: &Draft, file: &MvrFile, f: &MvrFixture, s: &Settings, matched: Option<u32>, key: &'static str, draft_scope: Option<&BTreeSet<u32>>, gdtf_hashes: &BTreeMap<String, String>) -> ReviewRow {
    let patch = &draft.devices.dmx.patch;
    let live = matched.and_then(|n| patch.fixture(n));
    let mut cells = Vec::new();
    // The file's values, offsets applied.
    let file_matrix = f.matrix.map(|m| Matrix { u: m[0], v: m[1], w: m[2], o: [m[3][0] + s.offset_mm[0] as f64, m[3][1] + s.offset_mm[1] as f64, m[3][2] + s.offset_mm[2] as f64] });
    let file_address = f.addresses.first().map(|a| (u32::try_from(a.universe as i64 + s.universe_offset).unwrap_or(0), a.channel));
    let spec_embedded = file.has_gdtf(&f.gdtf_spec);
    let hash = gdtf_hashes.get(&f.gdtf_spec);
    let known_type = patch.fixture_types.iter().find(|(_, t)| hash.is_some_and(|h| t.revisions.iter().any(|r| &r.file_hash == h)));
    let footprint = known_type
        .and_then(|(_, t)| t.revisions.iter().find(|r| hash.is_some_and(|h| &r.file_hash == h)).and_then(|r| r.modes.iter().find(|m| m.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(&f.gdtf_mode)))).map(|m| m.dmx_footprint))
        .or_else(|| live.and_then(|l| l.primary_break().map(|b| b.channel_count)))
        .unwrap_or(1);
    let live_name = live.map(|l| l.meta.identity.name.clone().unwrap_or_default());
    // Names.
    let mut cell = |field: &str, group: &str, file_v: String, live_v: Option<String>, mode: MvrTake| {
        let differs = live_v.as_ref() != Some(&file_v) && !(live_v.is_none() && file_v.is_empty());
        cells.push(ReviewCell { field: field.into(), group: group.into(), file: file_v, live: live_v, differs, mode, taken: differs && mode == MvrTake::Take, clash: None });
    };
    cell("name", "Names", f.name.clone(), live_name.clone(), s.names);
    // Addresses.
    let live_address = live.and_then(|l| l.primary_break()).and_then(|b| draft.universe_no.get(&b.universe_id).map(|u| format!("{u}.{:03}", b.start_address)));
    let file_address_text = file_address.map(|(u, c)| format!("{u}.{c:03}")).unwrap_or_default();
    cell("address", "Addresses", file_address_text, live_address, s.addresses);
    // Ids: the number (FID + offset), the unit number, the custom id.
    let file_no = f.fixture_id_numeric.and_then(|n| u32::try_from(n as i64 + s.fid_offset).ok());
    cell("unit", "Ids", f.unit_number.map(|n| n.to_string()).unwrap_or_default(), live.map(|l| l.mvr.unit_number.to_string()).filter(|u| u != "0"), s.ids);
    cell("custom_id", "Ids", f.custom_id.filter(|c| *c != 0).map(|n| n.to_string()).unwrap_or_default(), live.and_then(|l| l.mvr.cid).map(|c| c.to_string()), s.ids);
    // Position and rotation (metres and degrees as the grid shows them).
    let pos_text = |m: &Matrix| { let [x, y, z] = m.position_m(); format!("{:.3} {:.3} {:.3}", x, y, z) };
    let rot_text = |m: &Matrix| { let [x, y, z] = m.rotation_deg(); format!("{:.1} {:.1} {:.1}", x, y, z) };
    if let Some(fm) = &file_matrix {
        let lm = live.map(|l| &l.mvr.matrix);
        let pos_same = lm.is_some_and(|l| (0..3).all(|i| (l.o[i] - fm.o[i]).abs() < 1.0));
        let p_file = pos_text(fm);
        let p_live = lm.map(pos_text);
        let differs = !pos_same && lm.is_some() || lm.is_none();
        cells.push(ReviewCell { field: "pos".into(), group: "Position".into(), file: p_file, live: p_live, differs, mode: s.position, taken: differs && s.position == MvrTake::Take, clash: None });
        if s.rotation == MvrRotation::Take {
            let fr = fm.rotation_deg();
            let rot_same = lm.is_some_and(|l| { let lr = l.rotation_deg(); (0..3).all(|i| (lr[i] - fr[i]).abs() < 0.5) });
            let differs = !rot_same && lm.is_some() || lm.is_none();
            cells.push(ReviewCell { field: "rot".into(), group: "Position".into(), file: rot_text(fm), live: lm.map(rot_text), differs, mode: s.position, taken: differs && s.position == MvrTake::Take, clash: None });
        }
    }
    // The type: the file's spec and mode against the live type's revision and mode.
    let live_type = live.and_then(|l| patch.fixture_types.values().find(|t| t.meta.identity.guid == l.mvr.fixture_type).map(|t| (t, l)));
    let live_type_text = live_type.map(|(t, l)| {
        let mode = t.mode_of(l.mvr.revision, l.mvr.mode).map(|(_, m)| m.meta.identity.name.clone().unwrap_or_default()).unwrap_or_default();
        format!("{} · {}", t.meta.identity.name.clone().unwrap_or_default(), mode)
    });
    let same_type = live_type.is_some_and(|(t, l)| hash.is_some_and(|h| t.revisions.iter().any(|r| r.meta.identity.guid == l.mvr.revision && &r.file_hash == h)) && t.mode_of(l.mvr.revision, l.mvr.mode).is_some_and(|(_, m)| m.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(&f.gdtf_mode))));
    let type_file = format!("{} · {}", f.gdtf_spec.trim_end_matches(".gdtf"), f.gdtf_mode);
    let differs = live.is_some() && !same_type || live.is_none();
    cells.push(ReviewCell { field: "type".into(), group: "Revisions".into(), file: type_file, live: live_type_text, differs, mode: s.revisions, taken: differs && s.revisions == MvrTake::Take, clash: None });
    // The address clash: the file's run on someone else's slots — only where the address would
    // change (a draft fixture's own overlap is the draft's business, parked as it is).
    let mut clash = None;
    let address_changes = cells.iter().any(|c| c.field == "address" && c.differs);
    if let Some((u, c)) = file_address
        && u > 0
        && address_changes
        && let Some(other) = draft.overlapping(u, c, c.saturating_add(footprint).saturating_sub(1), matched)
    {
        let who = format!("{u}.{c:03} is Fixture {other}'s");
        if let Some(a) = cells.iter_mut().find(|x| x.field == "address") {
            a.clash = Some(who.clone());
            a.taken = false;
        }
        clash = Some(who);
    }
    let outside = matched.is_some() && draft_scope.is_some_and(|s| !matched.is_some_and(|n| s.contains(&n)));
    let any_differs = cells.iter().any(|c| c.differs && c.mode != MvrTake::None);
    let status = if clash.is_some() {
        RowStatus::Clash
    } else if live.is_none() {
        RowStatus::New
    } else if outside {
        RowStatus::Outside
    } else if any_differs {
        RowStatus::Changed
    } else {
        RowStatus::Same
    };
    // Hidden groups (None) show nothing; a new row's cells are all the file's.
    cells.retain(|c| c.mode != MvrTake::None || live.is_none());
    let taken = match status {
        RowStatus::New => s.new_fixtures == MvrTake::Take,
        RowStatus::Changed => cells.iter().any(|c| c.taken),
        _ => false,
    };
    // A parent that is a fixture of the file (MVR's `ChildList`, decisions §40 3): the row nests
    // under it, and no group is asked for.
    let parent_fixture = file.scene.fixtures.iter().any(|p| p.uuid == f.parent).then_some(f.parent);
    let group = Some(f.parent).filter(|p| *p != f.layer && parent_fixture.is_none());
    let group_missing = live.is_none() && group.is_some_and(|g| !draft.parent_of.contains_key(&g));
    // Its second id: the file's type, else the preset's for an untyped CustomId (decisions §40 9).
    let cid = f.custom_id.filter(|c| *c != 0);
    let id_type = cid.map(|_| f.custom_id_type.map(lumacore_fixtures::IdType::from_number).filter(|t| *t != lumacore_fixtures::IdType::None).unwrap_or(s.id_type)).unwrap_or_default();
    let id = match (cid, id_type) {
        (Some(c), t) if t != lumacore_fixtures::IdType::None => format!("{} {c}", t.name()),
        _ => String::new(),
    };
    let order = file.scene.fixtures.iter().filter(|x| x.parent == f.parent).position(|x| x.uuid == f.uuid).map_or(0, |i| (i as u32 + 1) * 10);
    ReviewRow {
        no: matched.or(file_no).unwrap_or(0),
        id,
        id_type: if id_type == lumacore_fixtures::IdType::None { String::new() } else { id_type.name().to_string() },
        cid,
        parent_fixture,
        order,
        uuid: f.uuid,
        status,
        matched: live.map(|l| l.meta.identity.guid),
        match_key: key.to_string(),
        name: f.name.clone(),
        gdtf_spec: f.gdtf_spec.clone(),
        gdtf_mode: f.gdtf_mode.clone(),
        gdtf_embedded: spec_embedded,
        type_known: known_type.is_some(),
        group,
        group_name: group.map(|g| file.scene.groups.iter().find(|x| x.uuid == g).map(|x| x.name.clone()).unwrap_or_default()).unwrap_or_default(),
        layer: f.layer,
        layer_name: file.scene.layers.iter().find(|l| l.uuid == f.layer).map(|l| l.name.clone()).unwrap_or_default(),
        group_missing,
        cells,
        taken,
        clash,
    }
}
