//! The one resolver (decisions §33 57 — strict): every target becomes objects here, and only here.
//!
//! The parser gives a [`Target`] (items joined by `+` / `-`, each a keyword path, a `Root.` tree
//! path or bare ids); [`resolve`] turns each item into its object kind, its owner and its members
//! — number, GUID, stored name — from what the registries declare (`ObjectDef::lum` for the
//! owner-first path, `TreeDef` for the members). [`resolve_one`] is the same for a verb that takes
//! one kind, [`fixtures`] the fixture view (`Group 1 + Fixture 7 - Group 2`).
//!
//! **The one range rule:** `3`, `1 Thru 4`, `1 Thru` (to the last), `Thru 4` (from the first),
//! joined by `+` / `-`; a range keeps its typed order (`5 Thru 2` = 5, 4, 3, 2); numbers that
//! don't exist are skipped; a quoted name matches the stored name ignoring case (decision 24).
//! Nothing at all is one refusal: E-CMD-007.
//!
//! Handlers that still get raw words use the bridges [`words`] / [`fixture_words`], which parse
//! the words as Lum first; no handler or module resolves ids or names any other way (the tidy
//! ratchet `old_resolvers_only_go_down` counts the old ones until part 5 removes them).

use lumacore_syntax::ast::{Body, Id, Ids, Item, Range, SetOp, Target};
use lumacore_syntax::{Origin, parse};
use uuid::Uuid;

use crate::AppState;
use crate::errors::render_code;
use crate::objects::ObjectDef;
use crate::vfs::DynChild;

/// One object a target names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub no: u32,
    pub guid: Option<Uuid>,
    pub name: Option<String>,
}

/// One item of a target, resolved.
#[derive(Clone)]
pub struct Resolved {
    pub object: &'static ObjectDef,
    /// The datapool its members live in, for objects that live in one (`DataPool 2 Group 3`,
    /// else your selected one).
    pub datapool: Option<u32>,
    /// Whether ids were written (`Group 3`) or only the kind (`Group`, `Root.ShowData.…Groups`).
    pub named: bool,
    /// In typed order, no duplicates.
    pub members: Vec<Member>,
}

impl std::fmt::Debug for Resolved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolved")
            .field("object", &self.object.keyword)
            .field("datapool", &self.datapool)
            .field("named", &self.named)
            .field("members", &self.members)
            .finish()
    }
}

impl Resolved {
    pub fn numbers(&self) -> Vec<u32> {
        self.members.iter().map(|m| m.no).collect()
    }
}

/// Every item of `target`, with the operator joining it to the one before. Bare ids name
/// `bare` (the verb's default object, else the user's — decisions §33 21, 25).
pub fn resolve(state: &AppState, target: &Target, bare: &'static ObjectDef) -> Result<Vec<(Option<SetOp>, Resolved)>, String> {
    target.items.iter().map(|(op, item)| Ok((op.as_ref().map(|o| o.node), item_of(state, item, bare)?))).collect()
}

/// A target of one kind (`Delete Group 1 Thru 4 - 2`), its items joined into one member list.
pub fn resolve_one(state: &AppState, target: &Target, bare: &'static ObjectDef) -> Result<Resolved, String> {
    let mut items = resolve(state, target, bare)?.into_iter();
    let (_, mut first) = items.next().ok_or_else(|| render_code("Nothing named.", "E-SYN-002"))?;
    for (op, item) in items {
        if !std::ptr::eq(item.object, first.object) {
            return Err(render_code(
                format!("One kind at a time here: {} and {}.", first.object.keyword, item.object.keyword),
                "E-CMD-008",
            ));
        }
        first.named |= item.named;
        join(&mut first.members, item.members, op.unwrap_or(SetOp::Plus), |m| (m.no, m.name.clone()));
    }
    if first.named && first.members.is_empty() {
        return Err(nothing(first.object));
    }
    Ok(first)
}

/// The fixtures a target names, in order (decisions §22, §33 26): `Fixture 1 Thru 4`,
/// `Group 3` (its members in their order), bare ids (fixtures), joined by `+` / `-`.
pub fn fixtures(state: &AppState, target: &Target) -> Result<Vec<u32>, String> {
    let fixture = &crate::objects::FIXTURE;
    let mut out: Vec<u32> = Vec::new();
    for (op, item) in resolve(state, target, fixture)? {
        let ids = if std::ptr::eq(item.object, fixture) {
            item.numbers()
        } else if std::ptr::eq(item.object, &crate::commands::groups::GROUP) {
            group_fixtures(state, &item)
        } else {
            return Err(render_code(format!("{} doesn't name fixtures.", item.object.keyword), "E-CMD-008"));
        };
        join(&mut out, ids, op.unwrap_or(SetOp::Plus), |n| *n);
    }
    if out.is_empty() {
        return Err(render_code("No matching fixtures.", "E-CMD-007"));
    }
    Ok(out)
}

/// The fixtures `target` names, by GUID — a fixture without a number (a folder, decisions §40 7)
/// is in here where `fixtures` has no number for it.
pub fn fixture_guids(state: &AppState, target: &Target) -> Result<Vec<Uuid>, String> {
    let fixture = &crate::objects::FIXTURE;
    let mut out: Vec<Uuid> = Vec::new();
    for (op, item) in resolve(state, target, fixture)? {
        let guids: Vec<Uuid> = if std::ptr::eq(item.object, fixture) {
            item.members.iter().filter_map(|m| m.guid).collect()
        } else if std::ptr::eq(item.object, &crate::commands::groups::GROUP) {
            group_fixtures(state, &item).into_iter().filter_map(|n| state.root.show_data.fixture_guid(n)).collect()
        } else {
            return Err(render_code(format!("{} doesn't name fixtures.", item.object.keyword), "E-CMD-008"));
        };
        join(&mut out, guids, op.unwrap_or(SetOp::Plus), |g| *g);
    }
    if out.is_empty() {
        return Err(render_code("No matching fixtures.", "E-CMD-007"));
    }
    Ok(out)
}

/// `fixture_guids` for raw words (`["Fixture", "Media", "7003"]`).
pub fn fixture_guid_words(state: &AppState, words: &[&str]) -> Result<Vec<Uuid>, String> {
    fixture_guids(state, &target_of(&words.join(" "))?)
}

/// Bridge for a handler that still gets raw words: `words` after `object`'s keyword
/// (`["2", "Thru", "5"]` for `Group`), parsed as Lum and resolved as one kind.
pub fn words(state: &AppState, object: &'static ObjectDef, words: &[&str]) -> Result<Resolved, String> {
    let line: Vec<&str> = std::iter::once(object.keyword).chain(words.iter().copied()).collect();
    resolve_one(state, &target_of(&line.join(" "))?, object)
}

/// Bridge: fixtures named by raw selection words (`["2", "Thru", "5"]`, `["Group", "3"]`).
pub fn fixture_words(state: &AppState, words: &[&str]) -> Result<Vec<u32>, String> {
    fixtures(state, &target_of(&words.join(" "))?)
}

/// The target of a line fragment (`Group 1 + 3`, `2 Thru 5`), parsed with the registries' words.
pub fn target_of(src: &str) -> Result<Target, String> {
    let line = parse(src, crate::lum::vocabulary(), Origin::Typed);
    if let Some(d) = line.diags.first() {
        return Err(render_code(&d.message, d.code));
    }
    match line.statements.first().map(|s| &s.body) {
        Some(Body::Lum(lum)) if lum.verb.is_none() && lum.slots.is_empty() => {
            lum.target.clone().ok_or_else(|| render_code("Nothing named.", "E-SYN-002"))
        }
        _ => Err(render_code(format!("'{src}' isn't a target."), "E-SYN-003")),
    }
}

// ── items ──

fn item_of(state: &AppState, item: &Item, bare: &'static ObjectDef) -> Result<Resolved, String> {
    match item {
        Item::Ids(ids) => members(state, bare, None, Some(ids)),
        Item::Path(segments) => {
            let vocab = crate::lum::vocabulary();
            let object_of = |seg: &lumacore_syntax::ast::Segment| crate::objects::find(&vocab.word(seg.object.node).name);
            let (last, owners) = segments.split_last().expect("a path has a segment");
            if let Some(object) = object_of(last).filter(|o| o.lum.reading == crate::objects::IdReading::Path) {
                return path_item(state, object, owners, last);
            }
            // `Fixture Houselights 4`: by second id (decisions §34 round 3 19).
            if let (Some(kind), Some(ids)) = (&last.kind, &last.ids) {
                let members = crate::objects::patch::second_ids(state, &kind.node, ids)?;
                let members = members.into_iter().map(|(no, guid, name)| Member { no, guid: Some(guid), name }).collect();
                return Ok(Resolved { object: &crate::objects::FIXTURE, datapool: None, named: true, members });
            }
            let datapool = owner_datapool(state, owners)?;
            let object = object_of(last).ok_or_else(|| render_code("Unknown object.", "E-CMD-007"))?;
            members(state, object, datapool, last.ids.as_ref())
        }
        Item::Tree(parts) => tree_item(state, &parts.node.join(".")),
        Item::SelfRef(_) => Err(render_code("Self names the window the line belongs to, not a list of objects.", "E-CMD-008")),
    }
}

/// The DataPool the owners in front of an object name (`DataPool 2 Group 3`); None = yours.
fn owner_datapool(state: &AppState, owners: &[lumacore_syntax::ast::Segment]) -> Result<Option<u32>, String> {
    let vocab = crate::lum::vocabulary();
    let mut datapool = None;
    for owner in owners {
        let owner_obj = crate::objects::find(&vocab.word(owner.object.node).name).ok_or_else(|| render_code("Unknown owner.", "E-CMD-007"))?;
        if !std::ptr::eq(owner_obj, &crate::objects::DATAPOOL) {
            // Owners other than DataPool resolve where their area moves to the resolver (§33
            // part 3d); until then their handlers read them.
            return Err(render_code(format!("{} as an owner isn't resolved here yet.", owner_obj.keyword), "E-CMD-010"));
        }
        let pool = members(state, owner_obj, None, owner.ids.as_ref())?;
        datapool = match pool.members.as_slice() {
            [one] => Some(one.no),
            [] if !pool.named => None,
            _ => return Err(render_code("Name one DataPool as the owner.", "E-CMD-008")),
        };
    }
    Ok(datapool)
}

/// The owners a target named that scope their children (a Sequence's cues, a UserProfile's
/// Theme, a Station's wiring rows — `Props::within`), outer first, by their row keys.
pub type Owners = Vec<(&'static ObjectDef, String)>;

/// `resolve_one` for a target whose owners scope it (`Sequence 4 Cue 2`, `UserProfile "Bob"
/// Theme`, decisions §33 3d): each scoping owner is resolved to one row and its scope entered
/// while the next is resolved; the rows come back with the owners, to run them in the same scopes
/// again (`with_owners`). A DataPool owner works as in `resolve`.
pub fn resolve_owned(state: &mut AppState, target: &Target, bare: &'static ObjectDef) -> Result<(Resolved, Owners), String> {
    let [(None, Item::Path(segments))] = target.items.as_slice() else {
        return Ok((resolve_one(state, target, bare)?, Vec::new()));
    };
    let vocab = crate::lum::vocabulary();
    let object_of = |seg: &lumacore_syntax::ast::Segment| crate::objects::find(&vocab.word(seg.object.node).name);
    let (last, owners_segs) = segments.split_last().expect("a path has a segment");
    if let Some(object) = object_of(last).filter(|o| o.lum.reading == crate::objects::IdReading::Path) {
        return Ok((path_item(state, object, owners_segs, last)?, Vec::new()));
    }
    let mut datapool = None;
    let mut owners: Owners = Vec::new();
    let mut guards: Vec<Box<dyn std::any::Any>> = Vec::new();
    for owner in owners_segs {
        let owner_obj = object_of(owner).ok_or_else(|| render_code("Unknown owner.", "E-CMD-007"))?;
        if std::ptr::eq(owner_obj, &crate::objects::DATAPOOL) {
            datapool = owner_datapool(state, std::slice::from_ref(owner))?;
            continue;
        }
        let Some(props) = owner_obj.props.filter(|p| p.within.is_some()) else {
            return Err(render_code(format!("{} doesn't own a {}.", owner_obj.keyword, object_of(last).map_or("", |o| o.keyword)), "E-CMD-008"));
        };
        let rows = members(state, owner_obj, datapool, owner.ids.as_ref())?;
        let keyed = owner_obj.tree.is_none() && props.keyed.is_some();
        let key = match (rows.named, rows.members.as_slice(), props.implicit) {
            (false, _, Some(implicit)) => implicit(state),
            (true, [one], _) => if keyed { one.name.clone().unwrap_or_default() } else { one.no.to_string() },
            (true, [], _) => return Err(nothing(owner_obj)),
            _ => return Err(render_code(format!("Name one {} as the owner.", owner_obj.keyword), "E-CMD-008")),
        };
        guards.push((props.within.expect("filtered"))(state, &key)?);
        owners.push((owner_obj, key));
    }
    let object = object_of(last).ok_or_else(|| render_code("Unknown object.", "E-CMD-007"))?;
    let resolved = members(state, object, datapool, last.ids.as_ref());
    drop(guards);
    Ok((resolved?, owners))
}

/// Run `f` inside the owners' scopes again (the rows `resolve_owned` found).
pub fn with_owners<R>(state: &mut AppState, owners: &Owners, f: impl FnOnce(&mut AppState) -> R) -> Result<R, String> {
    let mut guards: Vec<Box<dyn std::any::Any>> = Vec::new();
    for (obj, key) in owners {
        if let Some(within) = obj.props.and_then(|p| p.within) {
            guards.push(within(state, key)?);
        }
    }
    let out = f(state);
    drop(guards);
    Ok(out)
}

/// Whether a stored name fits a quoted id (decisions §35 17): ignoring case, and a `*` in
/// the id stands for any characters (`"Mac*"`, `"backt*blue"`); without one it is the name.
pub fn name_fits(pattern: &str, name: &str) -> bool {
    let (p, n) = (pattern.to_lowercase(), name.to_lowercase());
    if !p.contains('*') {
        return p == n;
    }
    let parts: Vec<&str> = p.split('*').collect();
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !n.starts_with(first) || n.len() < first.len() + last.len() || !n.ends_with(last) {
        return false;
    }
    let mut rest = &n[first.len()..n.len() - last.len()];
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    true
}

/// A name may not hold `*`: it is the pattern sign (decisions §35 17).
pub fn check_name(name: &str) -> Result<(), String> {
    if name.contains('*') {
        return Err(render_code(format!("\"{name}\": a name can't hold * — it stands for any characters in a target."), "E-CMD-008"));
    }
    Ok(())
}

/// The one place a Store writes to (decisions §33 3d): a slot may not exist yet.
#[derive(Debug, Clone, PartialEq)]
pub enum Slot {
    /// `Store Group 7`, or a name an existing one has (`Store Group "Front"`).
    Number(u32),
    /// A number in parts as typed, for an object whose ids have them (`Store Cue 2.5`).
    Key(String),
    /// `Store Group "Front"` when none is called that: a new one with this name.
    NewName(String),
    /// `Store Group`: the next free one.
    Next,
}

/// The slot a Store's target names — one id (a number, or a name: an existing one's number,
/// else a new name) or none — and the DataPool its owners name. A range is refused here.
pub fn slot(state: &AppState, target: Option<&Target>, object: &'static ObjectDef) -> Result<(Slot, Option<u32>), String> {
    let Some(target) = target else { return Ok((Slot::Next, None)) };
    let one = || render_code(format!("Store one {} at a time: a number or a name.", object.keyword), "E-CMD-008");
    let (ids, datapool) = match target.items.as_slice() {
        [(None, Item::Ids(ids))] => (Some(ids), None),
        [(None, Item::Path(segments))] => {
            let (last, owners) = segments.split_last().expect("a path has a segment");
            let vocab = crate::lum::vocabulary();
            if !crate::objects::find(&vocab.word(last.object.node).name).is_some_and(|o| std::ptr::eq(o, object)) {
                return Err(render_code(format!("Name a {}.", object.keyword), "E-CMD-008"));
            }
            (last.ids.as_ref(), owner_datapool(state, owners)?)
        }
        _ => return Err(one()),
    };
    let Some(ids) = ids else { return Ok((Slot::Next, datapool)) };
    let [(None, Range::One(id))] = ids.ranges.as_slice() else { return Err(one()) };
    Ok((
        match id {
            Id::Number(n) => Slot::Number(n.node.parse().map_err(|_| render_code(format!("'{}' isn't a {} number.", n.node, object.keyword), "E-CMD-008"))?),
            Id::Name(n) => name_slot(object, &members(state, object, datapool, None)?, &n.node)?,
        },
        datapool,
    ))
}

/// `slot` under scoping owners (`Store Sequence 4 Cue 2.5`, decisions §33 3d): each owner is
/// resolved to one row as in `resolve_owned`; the slot is the last segment's one id — a number
/// (in parts, as typed, for objects whose ids have them), a name, or none.
pub fn slot_owned(state: &mut AppState, target: Option<&Target>, object: &'static ObjectDef) -> Result<(Slot, Option<u32>, Owners), String> {
    let Some(target) = target else { return Ok((Slot::Next, None, Vec::new())) };
    let one = || render_code(format!("Store one {} at a time: a number or a name.", object.keyword), "E-CMD-008");
    let segments = match target.items.as_slice() {
        [(None, Item::Path(segments))] => segments,
        [(None, Item::Ids(_))] => {
            let (slot, dp) = slot(state, Some(target), object)?;
            return Ok((slot, dp, Vec::new()));
        }
        _ => return Err(one()),
    };
    let (last, owners_segs) = segments.split_last().expect("a path has a segment");
    let vocab = crate::lum::vocabulary();
    if !crate::objects::find(&vocab.word(last.object.node).name).is_some_and(|o| std::ptr::eq(o, object)) {
        return Err(render_code(format!("Name a {}.", object.keyword), "E-CMD-008"));
    }
    // The owners: resolved by the same walk as a target (the slot itself may not exist yet).
    let owners_only = Target { items: vec![(None, Item::Path(owners_segs.to_vec()))] };
    let (datapool, owners) = if owners_segs.is_empty() {
        (None, Vec::new())
    } else {
        let mut datapool = None;
        let mut owners = Vec::new();
        for owner in owners_segs {
            let owner_obj = crate::objects::find(&vocab.word(owner.object.node).name).ok_or_else(|| render_code("Unknown owner.", "E-CMD-007"))?;
            if std::ptr::eq(owner_obj, &crate::objects::DATAPOOL) {
                datapool = owner_datapool(state, std::slice::from_ref(owner))?;
                continue;
            }
            let _ = &owners_only;
            let props = owner_obj.props.filter(|p| p.within.is_some()).ok_or_else(|| render_code(format!("{} doesn't own a {}.", owner_obj.keyword, object.keyword), "E-CMD-008"))?;
            let rows = members(state, owner_obj, datapool, owner.ids.as_ref())?;
            let keyed = owner_obj.tree.is_none() && props.keyed.is_some();
            let key = match (rows.named, rows.members.as_slice(), props.implicit) {
                (false, _, Some(implicit)) => implicit(state),
                (true, [one], _) => if keyed { one.name.clone().unwrap_or_default() } else { one.no.to_string() },
                // A Store may make its owner too (`Store Sequence 9 Cue 1`): one number names it.
                (true, [], _) => match owner.ids.as_ref().map(|i| i.ranges.as_slice()) {
                    Some([(None, Range::One(Id::Number(n)))]) if n.node.parse::<u32>().is_ok() => n.node.clone(),
                    _ => return Err(nothing(owner_obj)),
                },
                _ => return Err(render_code(format!("Name one {} as the owner.", owner_obj.keyword), "E-CMD-008")),
            };
            owners.push((owner_obj, key));
        }
        (datapool, owners)
    };
    let Some(ids) = last.ids.as_ref() else { return Ok((Slot::Next, datapool, owners)) };
    let [(None, Range::One(id))] = ids.ranges.as_slice() else { return Err(one()) };
    let slot = match id {
        Id::Number(n) if object.lum.id_parts > 1 => Slot::Key(n.node.clone()),
        Id::Number(n) => Slot::Number(n.node.parse().map_err(|_| render_code(format!("'{}' isn't a {} number.", n.node, object.keyword), "E-CMD-008"))?),
        Id::Name(n) => {
            let found = with_owners(state, &owners, |s| members(s, object, datapool, None))??;
            name_slot(object, &found, &n.node)?
        }
    };
    Ok((slot, datapool, owners))
}

/// A quoted Store id: an existing row of that name, a new name — or, with `*`, the one row it fits
/// (a pattern never becomes a name; several rows are refused for now).
fn name_slot(object: &ObjectDef, found: &Resolved, id: &str) -> Result<Slot, String> {
    let fits: Vec<&Member> = found.members.iter().filter(|m| m.name.as_deref().is_some_and(|x| name_fits(id, x))).collect();
    match (fits.as_slice(), id.contains('*')) {
        ([m], _) => Ok(Slot::Number(m.no)),
        ([], false) => Ok(Slot::NewName(id.to_string())),
        ([], true) => Err(render_code(format!("No {} fits \"{id}\" (not found).", object.keyword), "E-CMD-007")),
        (many, _) => Err(render_code(format!("\"{id}\" fits {} {}s — Store names one.", many.len(), object.keyword), "E-CMD-008")),
    }
}

/// `Root.ShowData.DataPools.2.Groups.3` — the typed tree path (decisions §31, §33 49).
fn tree_item(state: &AppState, path: &str) -> Result<Resolved, String> {
    let parsed = crate::tree_path::TreePath::parse_dot(path).ok_or_else(|| render_code(format!("No node {path}."), "E-CMD-007"))?;
    let node = parsed.node();
    let object = crate::objects::all()
        .find(|o| o.tree.as_ref().is_some_and(|t| t.node == node))
        .ok_or_else(|| render_code(format!("{path} isn't a collection of objects."), "E-CMD-008"))?;
    let all = members(state, object, parsed.datapool(), None)?;
    Ok(match parsed.entry() {
        None => all,
        Some(no) => {
            let members: Vec<Member> = all.members.into_iter().filter(|m| m.no == no).collect();
            if members.is_empty() {
                return Err(nothing(object));
            }
            Resolved { named: true, members, ..all }
        }
    })
}

/// The members of `object` (in `datapool`, else the user's selected one where it lives in one)
/// that `ids` name; all of them, unnamed, when there are no ids.
fn members(state: &AppState, object: &'static ObjectDef, datapool: Option<u32>, ids: Option<&Ids>) -> Result<Resolved, String> {
    // A tree of itself (Tag 1.2.1): its ids read as paths (decisions §12 "Tags hardened").
    if object.lum.reading == crate::objects::IdReading::Path {
        return path_members(state, object, None, ids).map(|(r, _)| r);
    }
    // Rows an object declares by key (an interface, a station, the show — decisions §33 58).
    if let Some(props) = object.props
        && (object.tree.is_none() || props.keyed_beside_tree)
        && let Some(keyed) = props.keyed.as_ref()
    {
        return keyed_members(state, object, keyed, ids);
    }
    // DataPools are numbered nodes, not a collection: the tree's own list of them.
    let (children, datapool) = if std::ptr::eq(object, &crate::objects::DATAPOOL) {
        (crate::vfs::datapool_children(&state.root), None)
    } else {
        let tree = object
            .tree
            .as_ref()
            .ok_or_else(|| render_code(format!("{} isn't resolved by number yet.", object.keyword), "E-CMD-010"))?;
        let datapool = tree.datapool.as_ref().map(|_| datapool.unwrap_or_else(|| state.selected_datapool()));
        (tree.children_in(&state.root, datapool), datapool)
    };
    let all = || children.iter().map(member).collect::<Vec<_>>();
    let Some(ids) = ids else {
        return Ok(Resolved { object, datapool, named: false, members: all() });
    };
    let mut out: Vec<Member> = Vec::new();
    for (op, range) in &ids.ranges {
        let named = range_members(object, &children, range)?;
        join(&mut out, named, op.as_ref().map_or(SetOp::Plus, |o| o.node), |m| m.no);
    }
    Ok(Resolved { object, datapool, named: true, members: out })
}

/// Keyed rows: a name or number is the row its declaration finds (`Interface "en0"`,
/// `Station 2` = the station table's row 2); a range runs over the numbered keys.
fn keyed_members(state: &AppState, object: &'static ObjectDef, keyed: &crate::props::Keyed, ids: Option<&Ids>) -> Result<Resolved, String> {
    let member = |key: String| Member { no: key.parse().unwrap_or(0), guid: None, name: Some(key) };
    let Some(ids) = ids else {
        return Ok(Resolved { object, datapool: None, named: false, members: (keyed.rows)(state).into_iter().map(member).collect() });
    };
    let mut out: Vec<Member> = Vec::new();
    for (op, range) in &ids.ranges {
        let named: Vec<Member> = match range {
            Range::One(Id::Name(n) | Id::Number(n)) => (keyed.row)(state, &n.node).map(member).into_iter().collect(),
            Range::Thru { from, to, .. } => {
                let number = |id: &Option<Id>| match id {
                    Some(Id::Number(n)) => n.node.parse::<u32>().map(Some).map_err(|_| render_code(format!("{} isn't a number.", n.node), "E-CMD-008")),
                    Some(Id::Name(n)) => Err(render_code(format!("A range takes numbers, not \"{}\".", n.node), "E-CMD-008")),
                    None => Ok(None),
                };
                let (lo, hi) = (number(from)?, number(to)?);
                // The numbers the declaration knows (`Station 1 Thru 2` = the station table's rows
                // 1–2), each found as typed; an open end runs to the first / last numbered key.
                let numbered: Vec<u32> = (keyed.rows)(state).iter().filter_map(|k| k.parse().ok()).collect();
                let lo_n = lo.or_else(|| numbered.iter().min().copied()).unwrap_or(1);
                let hi_n = hi.or_else(|| numbered.iter().max().copied()).unwrap_or(lo_n);
                let (a, b) = (lo_n.min(hi_n), lo_n.max(hi_n));
                let mut inside: Vec<Member> = (a..=b).filter_map(|n| (keyed.row)(state, &n.to_string())).map(member).collect();
                inside.dedup_by(|x, y| x.name == y.name);
                if lo_n > hi_n {
                    inside.reverse();
                }
                inside
            }
        };
        join(&mut out, named, op.as_ref().map_or(SetOp::Plus, |o| o.node), |m| (m.no, m.name.clone()));
    }
    Ok(Resolved { object, datapool: None, named: true, members: out })
}

// ── paths (an object that nests itself: Tag 1.2.1, decisions §12 "Tags hardened") ──

/// `Tag "Lx1" Tag "Right"`: each owner names one node of the same object, the next is looked for
/// under it; the last segment's ids are under the last owner.
fn path_item(state: &AppState, object: &'static ObjectDef, owners: &[lumacore_syntax::ast::Segment], last: &lumacore_syntax::ast::Segment) -> Result<Resolved, String> {
    let vocab = crate::lum::vocabulary();
    let mut parent: Option<Vec<u32>> = None;
    for owner in owners {
        let same = crate::objects::find(&vocab.word(owner.object.node).name).is_some_and(|o| std::ptr::eq(o, object));
        if !same {
            return Err(render_code(format!("A {} is owned only by another {} (owner first).", object.keyword, object.keyword), "E-SYN-004"));
        }
        let (found, paths) = path_members(state, object, parent.as_deref(), owner.ids.as_ref())?;
        match paths.as_slice() {
            [one] => parent = Some(one.clone()),
            [] if found.named => return Err(nothing(object)),
            _ => return Err(render_code(format!("Name one {} as the owner.", object.keyword), "E-CMD-008")),
        }
    }
    path_members(state, object, parent.as_deref(), last.ids.as_ref()).map(|(r, _)| r)
}

/// The nodes `ids` name under `parent` (the top level when None), with their paths. A number is a
/// path below `parent` (`1.2` = its first child's second child); a name is looked for among
/// `parent`'s children, or anywhere when there is no parent; `Thru` walks one parent's children.
fn path_members(
    state: &AppState,
    object: &'static ObjectDef,
    parent: Option<&[u32]>,
    ids: Option<&Ids>,
) -> Result<(Resolved, Vec<Vec<u32>>), String> {
    let def = lumacore_registry::get()
        .rows::<crate::objects::PathDef>()
        .find(|d| std::ptr::eq(d.object, &object.decl))
        .ok_or_else(|| render_code(format!("{} isn't resolved by path yet.", object.keyword), "E-CMD-010"))?;
    let nodes = (def.nodes)(&state.root);
    let base: Vec<u32> = parent.map(<[u32]>::to_vec).unwrap_or_default();
    let under = |p: &Vec<u32>| p.len() == base.len() + 1 && p.starts_with(&base);
    let to_member = |(_, c): &(Vec<u32>, DynChild)| member(c);
    let Some(ids) = ids else {
        let level: Vec<&(Vec<u32>, DynChild)> = nodes.iter().filter(|(p, _)| under(p)).collect();
        let members = level.iter().map(|n| to_member(n)).collect();
        let paths = level.iter().map(|(p, _)| p.clone()).collect();
        return Ok((Resolved { object, datapool: None, named: false, members }, paths));
    };
    let path = |text: &str| -> Result<Vec<u32>, String> {
        let mut out = base.clone();
        for part in text.split('.') {
            let ok = !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) && !(part.len() > 1 && part.starts_with('0'));
            let n: u32 = if ok { part.parse().unwrap_or(0) } else { 0 };
            if n == 0 {
                return Err(render_code(format!("\"{text}\" is not a {} number: one whole number per level ({} 1.2.1).", object.keyword, object.keyword), "E-TAG-007"));
            }
            out.push(n);
        }
        Ok(out)
    };
    let mut out: Vec<(Vec<u32>, Member)> = Vec::new();
    for (op, range) in &ids.ranges {
        let named: Vec<(Vec<u32>, Member)> = match range {
            Range::One(Id::Number(n)) => {
                let p = path(&n.node)?;
                nodes.iter().filter(|(q, _)| *q == p).map(|n| (n.0.clone(), to_member(n))).collect()
            }
            Range::One(Id::Name(name)) => {
                let fits: Vec<&(Vec<u32>, DynChild)> = nodes
                    .iter()
                    .filter(|(p, c)| (parent.is_none() || under(p)) && c.name.as_deref().is_some_and(|n| name_fits(&name.node, n)))
                    .collect();
                if fits.len() > 1 && !name.node.contains('*') {
                    let paths: Vec<String> = fits.iter().map(|(p, _)| p.iter().map(u32::to_string).collect::<Vec<_>>().join(".")).collect();
                    return Err(render_code(
                        format!(
                            "\"{}\" names {} {}s ({}): give its number, or its owner first ({} \"…\" {} \"{}\").",
                            name.node,
                            fits.len(),
                            object.keyword,
                            paths.join(", "),
                            object.keyword,
                            object.keyword,
                            name.node
                        ),
                        "E-TAG-006",
                    ));
                }
                fits.into_iter().map(|n| (n.0.clone(), to_member(n))).collect()
            }
            Range::Thru { from, to, .. } => {
                let end = |id: &Option<Id>| -> Result<Option<Vec<u32>>, String> {
                    match id {
                        Some(Id::Number(n)) => path(&n.node).map(Some),
                        Some(Id::Name(n)) => Err(render_code(format!("A range takes numbers, not \"{}\".", n.node), "E-CMD-008")),
                        None => Ok(None),
                    }
                };
                let (a, b) = (end(from)?, end(to)?);
                let level_of = |p: &Vec<u32>| p[..p.len() - 1].to_vec();
                let parent_of = match (&a, &b) {
                    (Some(a), Some(b)) if level_of(a) != level_of(b) => {
                        let show = |p: &Vec<u32>| p.iter().map(u32::to_string).collect::<Vec<_>>().join(".");
                        return Err(render_code(
                            format!("Thru walks the children of one {}: {} and {} have different parents (join them with +).", object.keyword, show(a), show(b)),
                            "E-TAG-005",
                        ));
                    }
                    (Some(a), _) => level_of(a),
                    (None, Some(b)) => level_of(b),
                    (None, None) => base.clone(),
                };
                let siblings: Vec<&(Vec<u32>, DynChild)> =
                    nodes.iter().filter(|(p, _)| p.len() == parent_of.len() + 1 && p.starts_with(&parent_of)).collect();
                let last_of = |p: &Vec<u32>| *p.last().unwrap_or(&0);
                let lo = a.as_ref().map(last_of).or_else(|| siblings.first().map(|(p, _)| last_of(p))).unwrap_or(0);
                let hi = b.as_ref().map(last_of).or_else(|| siblings.last().map(|(p, _)| last_of(p))).unwrap_or(0);
                let (x, y) = (lo.min(hi), lo.max(hi));
                let mut inside: Vec<(Vec<u32>, Member)> =
                    siblings.into_iter().filter(|(p, _)| (x..=y).contains(&last_of(p))).map(|n| (n.0.clone(), to_member(n))).collect();
                if lo > hi {
                    inside.reverse();
                }
                inside
            }
        };
        join(&mut out, named, op.as_ref().map_or(SetOp::Plus, |o| o.node), |(p, _)| p.clone());
    }
    let paths = out.iter().map(|(p, _)| p.clone()).collect();
    Ok((Resolved { object, datapool: None, named: true, members: out.into_iter().map(|(_, m)| m).collect() }, paths))
}

/// Where `Store` puts a new node of an object that reads a path (decisions §12 "Tags hardened").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSlot {
    /// `Store Tag 1.1.3`: this path (its parent must exist, the number be free).
    At(Vec<u32>),
    /// `Store Tag 1.1 Tag "Centre"`: a new name under `parent` (empty = the top), at its next free number.
    Named { parent: Vec<u32>, name: String },
    /// `Store Tag` / `Store Tag 1.1 Tag`: the next free number under `parent`.
    Next(Vec<u32>),
}

/// The slot a Store target names for an object that reads a path: owners first (each one existing
/// node), then a number (a path below the last owner), a new name, or nothing.
pub fn path_slot(state: &AppState, object: &'static ObjectDef, target: Option<&Target>) -> Result<PathSlot, String> {
    let Some(target) = target else { return Ok(PathSlot::Next(Vec::new())) };
    let one = || render_code(format!("Store names one place: Store {} 1.2, Store {} \"Name\".", object.keyword, object.keyword), "E-CMD-008");
    let (owners, ids): (&[lumacore_syntax::ast::Segment], Option<&Ids>) = match target.items.as_slice() {
        [(None, Item::Path(segments))] => {
            let (last, owners) = segments.split_last().expect("a path has a segment");
            (owners, last.ids.as_ref())
        }
        [(None, Item::Ids(ids))] => (&[], Some(ids)),
        _ => return Err(one()),
    };
    let vocab = crate::lum::vocabulary();
    let mut parent: Vec<u32> = Vec::new();
    for owner in owners {
        if !crate::objects::find(&vocab.word(owner.object.node).name).is_some_and(|o| std::ptr::eq(o, object)) {
            return Err(render_code(format!("A {} is owned only by another {} (owner first).", object.keyword, object.keyword), "E-SYN-004"));
        }
        let (_, paths) = path_members(state, object, (!parent.is_empty()).then_some(parent.as_slice()), owner.ids.as_ref())?;
        let [p] = paths.as_slice() else { return Err(render_code(format!("Name one {} as the owner.", object.keyword), "E-CMD-008")) };
        parent = p.clone();
    }
    let Some(ids) = ids else { return Ok(PathSlot::Next(parent)) };
    let [(None, Range::One(id))] = ids.ranges.as_slice() else { return Err(one()) };
    match id {
        Id::Name(n) => Ok(PathSlot::Named { parent, name: n.node.clone() }),
        Id::Number(n) => {
            let mut path = parent;
            for part in n.node.split('.') {
                let ok = !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) && !(part.len() > 1 && part.starts_with('0'));
                match part.parse::<u32>() {
                    Ok(v) if ok && v > 0 => path.push(v),
                    _ => return Err(render_code(format!("\"{}\" is not a {} number: one whole number per level ({} 1.2.1).", n.node, object.keyword, object.keyword), "E-TAG-007")),
                }
            }
            Ok(PathSlot::At(path))
        }
    }
}

fn member(c: &DynChild) -> Member {
    Member { no: c.id, guid: c.guid, name: c.name.clone() }
}

/// One range of the one range rule, against the existing members (sorted by number).
fn range_members(object: &ObjectDef, children: &[DynChild], range: &Range) -> Result<Vec<Member>, String> {
    let number = |id: &Id| -> Result<u32, String> {
        match id {
            Id::Number(n) => n.node.parse::<u32>().map_err(|_| {
                render_code(format!("{} {} isn't a number of it.", object.keyword, n.node), "E-CMD-008")
            }),
            Id::Name(n) => Err(render_code(format!("A range takes numbers, not \"{}\".", n.node), "E-CMD-008")),
        }
    };
    Ok(match range {
        Range::One(Id::Name(name)) => children
            .iter()
            .filter(|c| c.name.as_deref().is_some_and(|n| name_fits(&name.node, n)))
            .map(member)
            .collect(),
        Range::One(id) => {
            let no = number(id)?;
            children.iter().filter(|c| c.id == no).map(member).collect()
        }
        Range::Thru { from, to, .. } => {
            let lo = match from {
                Some(id) => number(id)?,
                None => children.first().map_or(0, |c| c.id),
            };
            let hi = match to {
                Some(id) => number(id)?,
                None => children.last().map_or(0, |c| c.id),
            };
            let (a, b) = (lo.min(hi), lo.max(hi));
            let mut inside: Vec<Member> = children.iter().filter(|c| (a..=b).contains(&c.id)).map(member).collect();
            if lo > hi {
                inside.reverse();
            }
            inside
        }
    })
}

/// `+` adds what isn't there yet (keeping the first place), `-` takes out.
fn join<T, K: PartialEq>(into: &mut Vec<T>, items: Vec<T>, op: SetOp, key: impl Fn(&T) -> K) {
    match op {
        SetOp::Plus => {
            for item in items {
                if !into.iter().any(|x| key(x) == key(&item)) {
                    into.push(item);
                }
            }
        }
        SetOp::Minus => into.retain(|x| !items.iter().any(|i| key(i) == key(x))),
    }
}

fn nothing(object: &ObjectDef) -> String {
    render_code(format!("No such {}.", object.keyword), "E-CMD-007")
}

/// A group's fixtures in its member order, by GUID.
fn group_fixtures(state: &AppState, item: &Resolved) -> Vec<u32> {
    let Some(pool) = item.datapool.and_then(|dp| state.root.show_data.datapools.get(&dp)) else {
        return Vec::new();
    };
    let by_guid: std::collections::HashMap<Uuid, u32> =
        state.root.show_data.devices.dmx.patch.fixtures.values().filter_map(|f| f.mvr.fid.map(|no| (f.meta.identity.guid, no))).collect();
    let mut out = Vec::new();
    for m in &item.members {
        if let Some(group) = pool.groups.get(&m.no) {
            for guid in group.fixtures() {
                if let Some(&no) = by_guid.get(&guid)
                    && !out.contains(&no)
                {
                    out.push(no);
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
