// ██╗███╗   ███╗██████╗  ██████╗ ██████╗ ████████╗
// ██║████╗ ████║██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝
// ██║██╔████╔██║██████╔╝██║   ██║██████╔╝   ██║
// ██║██║╚██╔╝██║██╔═══╝ ██║   ██║██╔══██╗   ██║
// ██║██║ ╚═╝ ██║██║     ╚██████╔╝██║  ██║   ██║
// ╚═╝╚═╝     ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝
// FILE: import

//! `Import Show "x" <parts | objects>` — take parts of another show into the
//! loaded one by GUID (decisions §30 "12d hardening", "Load show = one dialog
//! with parts"; instead of load segments and a partial show read). Same GUID = the object
//! is updated where it is; a new GUID is added (at its own number when free,
//! else the next free one); nothing here is removed. One line, one Oops step
//! (the change path files the subtrees that moved).
//!
//! ```text
//! Import Show "Festival" Patch Users              whole parts
//! Import Show "Festival" Sequence 1 + 3 Group 2   single objects (the file's numbers)
//! Import Show "Festival" DataPool 2 Group 1 Preset 4.1 User "LD"
//! Import Show "demo" /Version 20261013-140203 Sequence 2   from a version
//! ```
//!
//! Parts: `Data` (data pools, sequences, effects, timings, images), `Patch`
//! (fixture types, fixtures, universes, FID namespaces), `Protocols`
//! (networks, output and input lines), `Users` (users and profiles),
//! `Stations` (the station table). A user, profile, network or station row
//! whose name is taken here by another object is skipped and named.
//! Broken links are kept and listed (the reference check).
//!
//! Owns:          Import Show: taking parts of another show by GUID as one Oops step, and its parts view.
//! Doesn't own:   the merge of the parts (lumacore-tree), the file's read (lumacore-luma).
//! Entry points:  run, parts_view.
//! Decisions:     §30 "12d hardening", "12d part 5".

use std::collections::{BTreeMap, BTreeSet};

use lumacore_tree::ShowData;
use serde_json::{Map, Value, json};

use lumacore_objects::AppState;
use lumacore_objects::objects::ObjectDef;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A part of a show, as the Load show dialog ticks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    Data,
    Patch,
    Protocols,
    Users,
    Stations,
}

/// Part, its word in a line, its label, its pools (JSON paths in ShowData).
pub const PARTS: &[(Part, &str, &str, &[&[&str]])] = &[
    (
        Part::Data,
        "Data",
        "Show data",
        &[
            &["datapools"],
            &["datapool_priority_orders"],
            &["playback", "sequences"],
            &["playback", "effects"],
            &["playback", "timing_presets"],
            &["playback", "curve_presets"],
            &["images"],
        ],
    ),
    (
        Part::Patch,
        "Patch",
        "Patch",
        &[
            &["devices", "dmx", "patch", "fixture_types"],
            &["devices", "dmx", "patch", "fixtures"],
            &["devices", "dmx", "patch", "universes"],
            &["devices", "metadata"],
        ],
    ),
    (
        Part::Protocols,
        "Protocols",
        "DMX protocols",
        &[
            &["protocols", "networks"],
            &["protocols", "outputs"],
            &["protocols", "inputs"],
        ],
    ),
    (
        Part::Users,
        "Users",
        "Users & profiles",
        &[&["users"], &["user_profiles"]],
    ),
    (
        Part::Stations,
        "Stations",
        "Station table",
        &[&["stations"]],
    ),
];

/// Pools whose objects people find by name: a name taken here by another
/// object is not imported twice.
const NAMED: &[&str] = &["users", "user_profiles", "networks", "stations"];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// A part named in a line (`Data`, `Patch`, …; `ShowData`, `DMXProtocols`
/// too).
pub fn part_of(word: &str) -> Option<Part> {
    let w = word.trim_matches('"');
    PARTS
        .iter()
        .find(|(_, name, label, _)| {
            name.eq_ignore_ascii_case(w) || label.replace([' ', '&'], "").eq_ignore_ascii_case(w)
        })
        .map(|(p, ..)| *p)
        .or_else(|| {
            ["ShowData"]
                .iter()
                .any(|x| x.eq_ignore_ascii_case(w))
                .then_some(Part::Data)
        })
}

fn guid_of(v: &Value) -> Option<&str> {
    v.get("guid")
        .and_then(Value::as_str)
        .or_else(|| v.get("meta")?.get("guid")?.as_str())
}

/// What two objects of one kind are matched by: an identity key (an asset root's name, a
/// user's login) before the display name.
fn name_of(v: &Value) -> Option<&str> {
    v.get("name")
        .or_else(|| v.get("login"))
        .and_then(Value::as_str)
        .or_else(|| v.get("meta")?.get("name")?.as_str())
}

/// Every GUID inside `v` (itself included).
fn guids_in(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            if let Some(g) = guid_of(v) {
                out.push(g.to_string());
            }
            for x in m.values() {
                guids_in(x, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| guids_in(x, out)),
        _ => {}
    }
}

fn has_picked(v: &Value, pick: &BTreeSet<String>) -> bool {
    let mut all = Vec::new();
    guids_in(v, &mut all);
    all.iter().any(|g| pick.contains(g))
}

/// A map whose values are objects with GUIDs (a pool).
fn is_pool(v: &Value) -> bool {
    v.as_object()
        .is_some_and(|m| !m.is_empty() && m.values().all(|x| guid_of(x).is_some()))
}

/// The next free numeric key.
fn free_key(map: &Map<String, Value>, want: &str) -> String {
    if !map.contains_key(want) {
        return want.to_string();
    }
    let next = map
        .keys()
        .filter_map(|k| k.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    next.to_string()
}

/// A copy of `v` with only the picked objects in its pools.
fn only_picked(v: &Value, pick: &BTreeSet<String>) -> Value {
    let mut out = v.clone();
    if let Some(m) = out.as_object_mut() {
        for (_, sub) in m.iter_mut() {
            if is_pool(sub) {
                let kept: Map<String, Value> = sub
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(_, x)| {
                        guid_of(x).is_some_and(|g| pick.contains(g)) || has_picked(x, pick)
                    })
                    .map(|(k, x)| {
                        let whole = guid_of(x).is_some_and(|g| pick.contains(g));
                        (
                            k.clone(),
                            if whole {
                                x.clone()
                            } else {
                                only_picked(x, pick)
                            },
                        )
                    })
                    .collect();
                *sub = Value::Object(kept);
            }
        }
    }
    out
}

#[derive(Default)]
struct Tally {
    updated: usize,
    added: usize,
    skipped: Vec<String>,
}

/// Merge pool `src` into `cur` by GUID (`pick` None = every object).
fn merge(
    cur: &mut Map<String, Value>,
    src: &Map<String, Value>,
    pick: Option<&BTreeSet<String>>,
    named: bool,
    t: &mut Tally,
) {
    for (k, sv) in src {
        let Some(g) = guid_of(sv) else { continue };
        let whole = pick.is_none_or(|p| p.contains(g));
        if !whole && !pick.is_some_and(|p| has_picked(sv, p)) {
            continue;
        }
        let here = cur
            .iter()
            .find(|(_, cv)| guid_of(cv) == Some(g))
            .map(|(ck, _)| ck.clone());
        match here {
            Some(ck) if whole => {
                cur.insert(ck, sv.clone());
                t.updated += 1;
            }
            Some(ck) => {
                // Only some objects inside: merge its pools.
                let Some(cv) = cur.get_mut(&ck).and_then(Value::as_object_mut) else {
                    continue;
                };
                for (sub, subv) in sv.as_object().into_iter().flatten() {
                    if let (true, Some(sm)) = (is_pool(subv), subv.as_object()) {
                        let target = cv
                            .entry(sub.clone())
                            .or_insert_with(|| Value::Object(Map::new()));
                        if let Some(tm) = target.as_object_mut() {
                            merge(tm, sm, pick, false, t);
                        }
                    }
                }
            }
            None => {
                if named
                    && let Some(n) = name_of(sv)
                    && cur
                        .values()
                        .any(|cv| name_of(cv).is_some_and(|x| x.eq_ignore_ascii_case(n)))
                {
                    t.skipped
                        .push(format!("\"{n}\" (another object of that name is here)"));
                    continue;
                }
                let key = free_key(cur, k);
                cur.insert(
                    key,
                    if whole {
                        sv.clone()
                    } else {
                        only_picked(sv, pick.expect("partial needs a pick"))
                    },
                );
                t.added += 1;
            }
        }
    }
}

fn at_mut<'a>(v: &'a mut Value, path: &[&str]) -> Option<&'a mut Map<String, Value>> {
    let mut at = v;
    for p in path {
        at = at.get_mut(*p)?;
    }
    at.as_object_mut()
}

fn at<'a>(v: &'a Value, path: &[&str]) -> Option<&'a Map<String, Value>> {
    let mut at = v;
    for p in path {
        at = at.get(*p)?;
    }
    at.as_object()
}

/// What to take: whole parts and single objects (their GUIDs).
#[derive(Debug, Default, Clone)]
pub struct Take {
    pub parts: BTreeSet<Part>,
    pub objects: BTreeSet<String>,
    /// The objects as named in the line (for the reply).
    pub named: Vec<String>,
}

/// Take `what` from `src` into `show`. Returns the reply's lines.
pub fn apply(show: &mut ShowData, src: &ShowData, what: &Take) -> Result<Vec<String>, String> {
    let mut cur = serde_json::to_value(&*show).map_err(|e| e.to_string())?;
    let from = serde_json::to_value(src).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    for (part, _, label, pools) in PARTS {
        let whole = what.parts.contains(part);
        if !whole && what.objects.is_empty() {
            continue;
        }
        let pick = (!whole).then_some(&what.objects);
        let mut t = Tally::default();
        for path in *pools {
            let Some(sm) = at(&from, path) else { continue };
            let sm = sm.clone();
            let named = NAMED.contains(path.last().unwrap_or(&""));
            if at_mut(&mut cur, path).is_none() {
                // A pool the show lacks: an empty one to merge into.
                let (last, parent) = path.split_last().expect("pool paths have a name");
                if let Some(pm) = at_mut(&mut cur, parent) {
                    pm.insert((*last).to_string(), Value::Object(Map::new()));
                }
            }
            if let Some(cm) = at_mut(&mut cur, path) {
                merge(cm, &sm, pick, named, &mut t);
            }
        }
        if t.updated + t.added + t.skipped.len() == 0 {
            if whole {
                lines.push(format!("{label}: nothing in the file"));
            }
            continue;
        }
        let mut line = format!("{label}: {} updated · {} added", t.updated, t.added);
        if !t.skipped.is_empty() {
            line.push_str(&format!(" · skipped {}", t.skipped.join(", ")));
        }
        lines.push(line);
    }
    let mut next: ShowData = serde_json::from_value(cur)
        .map_err(|e| format!("The imported show doesn't read back: {e}"))?;
    next.after_load();
    *show = next;
    Ok(lines)
}

fn usage(text: String) -> String {
    lumacore_objects::errors::with_code(text, "E-CMD-006")
}

fn missing(text: String) -> String {
    lumacore_objects::errors::with_code(text, "E-CMD-007")
}

// *************************************************************************************************
// Naming single objects in a line
// *************************************************************************************************
/// The objects a line may name, their part, and how to find one in the
/// source show by its number (or name) there.
type Finder = fn(&ShowData, u32, &str) -> Option<String>;

fn dp(s: &ShowData, d: u32) -> Option<&lumacore_tree::Datapool> {
    s.datapools.get(&d)
}

/// The objects Import Show takes one by one: the kernel's here, the rest from their crates'
/// `Hook::ImportByGuid` rows.
fn kinds() -> Vec<(&'static ObjectDef, Finder)> {
    let mut kinds = own_kinds().to_vec();
    kinds.extend(lumacore_objects::commands::hooks::hooks().filter_map(|h| match h {
        lumacore_objects::commands::hooks::Hook::ImportByGuid(o, f) => Some((*o, *f)),
        _ => None,
    }));
    kinds
}

fn own_kinds() -> [(&'static ObjectDef, Finder); 9] {
    use lumacore_objects::objects as o;
    [
        (&lumacore_objects::commands::groups::GROUP, |s, d, id| {
            dp(s, d)?
                .groups
                .get(&id.parse().ok()?)
                .map(|x| x.meta.identity.guid.to_string())
        }),
        (&o::MACRO, |s, d, id| {
            dp(s, d)?
                .macros
                .get(&id.parse().ok()?)
                .map(|x| x.meta.identity.guid.to_string())
        }),
        (&lumacore_objects::objects::PRESET, |s, d, id| {
            dp(s, d)?
                .presets
                .get(&id.parse().ok()?)
                .map(|x| x.meta.identity.guid.to_string())
        }),
        (&o::SEQUENCE, |s, d, id| {
            dp(s, d)?
                .sequences
                .get(&id.parse().ok()?)
                .map(|x| x.meta.identity.guid.to_string())
        }),
        (&o::FIXTURE, |s, _, id| {
            s.devices
                .dmx
                .patch
                .fixture(id.parse::<u32>().ok()?)
                .map(|x| x.meta.identity.guid.to_string())
        }),
        (&o::FIXTURE_TYPE, |s, _, id| {
            s.devices
                .dmx
                .patch
                .fixture_types
                .get(&id.parse().ok()?)
                .map(|x| x.meta.identity.guid.to_string())
        }),
        (&o::UNIVERSE, |s, _, id| {
            s.devices
                .dmx
                .patch
                .universes
                .get(&id.parse().ok()?)
                .map(|x| x.meta.identity.guid.to_string())
        }),
        (&lumacore_objects::commands::users::USER, |s, _, id| {
            by_no_or_name(&s.users, id, |x| (x.meta.identity.guid.to_string(), Some(x.login.clone())))
        }),
        (&o::USER_PROFILE, |s, _, id| {
            by_no_or_name(&s.user_profiles, id, |x| {
                (x.meta.identity.guid.to_string(), x.meta.identity.name.clone())
            })
        }),
    ]
}

/// A pool member's GUID by number or name (Import Show's finders).
pub fn by_no_or_name<T>(
    pool: &BTreeMap<u32, T>,
    id: &str,
    of: impl Fn(&T) -> (String, Option<String>),
) -> Option<String> {
    if let Ok(n) = id.parse::<u32>() {
        return pool.get(&n).map(|x| of(x).0);
    }
    pool.values()
        .map(&of)
        .find(|(_, n)| n.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(id)))
        .map(|(g, _)| g)
}

/// Numbers and names after an object word: `1 + 3`, `1 Thru 4`, `"LD"`,
/// `4.1 + 4.2`. Returns the ids and how many tokens were used.
fn ids(tokens: &[&str]) -> (Vec<String>, usize) {
    let mut out = Vec::new();
    let mut i = 0;
    let mut thru = false;
    while i < tokens.len() {
        let t = tokens[i];
        if t == "+" {
            i += 1;
            continue;
        }
        if t.eq_ignore_ascii_case("thru") {
            thru = true;
            i += 1;
            continue;
        }
        let id = t.trim_matches('"');
        // A name right after the object word (`Network Main`, `User "LD"`).
        let name = (i == 0 || tokens[i - 1] == "+")
            && part_of(t).is_none()
            && lumacore_objects::objects::find(t).is_none();
        let is_id =
            name || t.starts_with('"') || id.chars().all(|c| c.is_ascii_digit() || c == '.');
        if !is_id || id.is_empty() {
            break;
        }
        if thru
            && let (Some(a), Ok(b)) = (
                out.last().and_then(|x: &String| x.parse::<u32>().ok()),
                id.parse::<u32>(),
            )
        {
            for n in a + 1..=b {
                out.push(n.to_string());
            }
        } else {
            out.push(id.to_string());
        }
        thru = false;
        i += 1;
    }
    (out, i)
}

/// What a line names (after the show and its flags), against `src`.
pub fn parse(tokens: &[&str], src: &ShowData) -> Result<Take, String> {
    let mut take = Take::default();
    let mut datapool = 1u32;
    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i];
        if t == "+" || t == ";" {
            i += 1;
            continue;
        }
        if let Some(p) = part_of(t) {
            take.parts.insert(p);
            i += 1;
            continue;
        }
        let Some(obj) = lumacore_objects::objects::find(t) else {
            return Err(usage(format!(
                "'{t}' isn't a part (Data, Patch, Protocols, Users, Stations) or an object Import Show takes"
            )));
        };
        if std::ptr::eq(obj, &lumacore_objects::objects::DATAPOOL) {
            datapool = tokens
                .get(i + 1)
                .and_then(|n| n.parse().ok())
                .ok_or_else(|| usage("DataPool needs its number".into()))?;
            i += 2;
            continue;
        }
        let Some((_, find)) = kinds().into_iter().find(|(o, _)| std::ptr::eq(*o, obj)) else {
            return Err(usage(format!(
                "Import Show doesn't take single {}s; tick its part instead",
                obj.keyword
            )));
        };
        let (list, used) = ids(&tokens[i + 1..]);
        if list.is_empty() {
            return Err(usage(format!("{} needs a number or a name", obj.keyword)));
        }
        for id in list {
            let shown = if id.chars().all(|c| c.is_ascii_digit() || c == '.') {
                id.clone()
            } else {
                format!("\"{id}\"")
            };
            let guid = find(src, datapool, &id)
                .ok_or_else(|| missing(format!("The file has no {} {shown}", obj.keyword)))?;
            take.objects.insert(guid);
            take.named.push(format!("{} {shown}", obj.keyword));
        }
        i += 1 + used;
        // `DataPool 2` names the datapool of the next object only.
        datapool = 1;
    }
    if take.parts.is_empty() && take.objects.is_empty() {
        return Err(usage("Name what to take: parts (Data, Patch, Protocols, Users, Stations) or objects (Sequence 1, Group 2, …)".into()));
    }
    Ok(take)
}

/// `Import Show "x" [/Version <stamp> | /Recovery "<copy>"] <parts | objects>`.
pub fn run(state: &mut AppState, args: &[&str]) -> Result<String, String> {
    use lumacore_luma::backups::{self, Store};
    use lumacore_objects::commands::flags as F;
    let version = F::value(args, &F::VERSION);
    let recovery = F::value(args, &F::RECOVERY);
    let drive = F::value(args, &F::DRIVE);
    let rest = F::positional(args, &[&F::VERSION, &F::RECOVERY, &F::DRIVE]);
    let _dir = crate::words::drive_guard(state, drive.as_deref(), false)?;
    let Some((name, what)) = rest.split_first() else {
        return Err(usage("Usage: Import Show \"name\" <parts | objects> (Data, Patch, Protocols, Users, Stations; Sequence 1, …)".into()));
    };
    let name = name.trim_matches('"').trim_end_matches(".luma").to_string();
    let path = match (&version, &recovery) {
        (Some(id), _) => backups::find(Store::Version, &name, id)
            .map(|c| c.path)
            .ok_or_else(|| missing(format!("\"{name}\" has no version {id}")))?,
        (None, Some(id)) => backups::find(Store::Recovery, &name, id)
            .map(|c| c.path)
            .ok_or_else(|| missing(format!("No recovery copy \"{id}\"")))?,
        (None, None) => lumacore_luma::backups::show_path(&name),
    };
    if lumacore_luma::drive::metadata(&path).is_err() {
        return Err(missing(format!("No show \"{name}\" in the library")));
    }
    let src = lumacore_luma::show_file::read(&path)?;
    let take = parse(what, &src)?;
    if take.parts.contains(&Part::Patch) && state.patch_draft.is_some() {
        return Err(lumacore_objects::errors::with_code(
            "Patch is claimed with a draft: Unclaim Root.ShowData.Devices before importing the patch.",
            "E-CMD-010",
        ));
    }
    let mut lines = apply(&mut state.root.show_data, &src, &take)?;
    let broken = lumacore_objects::references::check(state, "import")
        .broken
        .len();
    let from = match (&version, &recovery) {
        (Some(v), _) => format!("\"{name}\" version {v}"),
        (None, Some(r)) => format!("recovery copy {r}"),
        _ => format!("\"{name}\""),
    };
    let mut head = format!("Imported from {from}");
    if !take.named.is_empty() {
        head.push_str(&format!(": {}", take.named.join(", ")));
    }
    lines.insert(0, head);
    if broken > 0 {
        lines.push(format!(
            "{broken} link(s) point at objects that aren't here (List Reference /Broken)"
        ));
    }
    Ok(lines.join("\n"))
}

/// The Load show dialog's facts and parts of a file (`GET /api/show-parts`):
/// from its manifest only — the counts and, per part, the objects that can
/// be picked (`ref` = how a line names it in that file).
pub fn parts_view(name: &str, version: Option<&str>, recovery: Option<&str>) -> Value {
    use lumacore_luma::backups::{self, Store};
    let path = match (version, recovery) {
        (Some(v), _) => backups::find(Store::Version, name, v).map(|c| c.path),
        (None, Some(r)) => backups::find(Store::Recovery, name, r).map(|c| c.path),
        _ => Some(lumacore_luma::backups::show_path(name)),
    };
    let Some((m, size, mtime)) = path.as_deref().and_then(backups::manifest) else {
        return json!({ "error": format!("No show \"{name}\"") });
    };
    let now = backups::now_ms();
    let at = m["saved_at"].as_u64().unwrap_or(mtime);
    let fmt = m["format"].as_u64().unwrap_or(0);
    let objects = |kinds: &[&str]| -> Vec<Value> {
        let mut out = Vec::new();
        for k in kinds {
            for v in m["index"][*k]
                .as_object()
                .into_iter()
                .flat_map(|o| o.values())
            {
                let label = v[1].as_str().unwrap_or("").to_string();
                // `Group 3 "Wash"` → `Group 3`; `User "LD"` stays.
                let r = match label.find(" \"") {
                    Some(i) if label[..i].contains(' ') => label[..i].to_string(),
                    _ => label.clone(),
                };
                out.push(json!({ "ref": r, "label": label, "kind": k }));
            }
        }
        out.sort_by(|a, b| {
            natural(a["label"].as_str().unwrap_or(""))
                .cmp(&natural(b["label"].as_str().unwrap_or("")))
        });
        out
    };
    let c = &m["counts"];
    let n = |k: &str| c[k].as_u64().unwrap_or(0);
    let parts = json!([
        { "key": "Data", "label": "Show data", "sum": format!("{} groups, {} presets, {} sequences, {} macros, pools", n("Groups"), n("Presets"), n("Sequences"), n("Macros")),
          "objects": objects(&["Groups", "Presets", "Sequences", "Macros"]) },
        { "key": "Patch", "label": "Patch", "sum": format!("{} fixtures, {} fixture types", n("Fixtures"), n("FixtureTypes")),
          "objects": objects(&["FixtureTypes", "Fixtures"]) },
        { "key": "Protocols", "label": "DMX protocols", "sum": format!("{} networks, {} lines", n("Networks"), n("Lines")),
          "objects": objects(&["Networks", "Lines"]) },
        { "key": "Users", "label": "Users & profiles", "sum": format!("{} users, {} profiles", n("Users"), n("Profiles")),
          "objects": objects(&["Users", "Profiles"]) },
        { "key": "Stations", "label": "Station table", "sum": format!("wiring of {} stations", n("Stations")),
          "objects": objects(&["Stations"]) },
    ]);
    let readable = lumacore_luma::show_file::upgrade::readable(fmt);
    let ours = u64::from(lumacore_tree::FORMAT_VERSION);
    json!({
        "show": name, "version": version, "recovery": recovery,
        "facts": {
            "saved": backups::when_text(at, now), "by": m["user"], "station": m["station"],
            "reason": backups::reason_word(m["reason"].as_str().unwrap_or("")), "note": m["note"],
            "size": backups::size_text(size), "format": fmt, "description": m["description"],
        },
        "loads": readable.is_ok(),
        "upgrades": readable.is_ok() && fmt < ours,
        "refused": match readable {
            Ok(()) => Value::Null,
            Err((why, code)) => json!(format!("Saved {} ({code})", why.trim_start_matches("saved "))),
        },
        "parts": parts,
    })
}

/// A sort key that orders `Group 10` after `Group 9`.
fn natural(s: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut num = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else {
            if !num.is_empty() {
                out.push((std::mem::take(&mut text), num.parse().unwrap_or(0)));
                num.clear();
            }
            text.push(ch);
        }
    }
    out.push((text, num.parse().unwrap_or(0)));
    out
}
