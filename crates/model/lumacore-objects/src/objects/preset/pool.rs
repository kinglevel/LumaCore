// ██████╗  ██████╗  ██████╗ ██╗
// ██╔══██╗██╔═══██╗██╔═══██╗██║
// ██████╔╝██║   ██║██║   ██║██║
// ██╔═══╝ ██║   ██║██║   ██║██║
// ██║     ╚██████╔╝╚██████╔╝███████╗
// ╚═╝      ╚═════╝  ╚═════╝ ╚══════╝
// FILE: pool

//! The Presets window's cells (decisions 0288): the DataPool's one preset pool, narrowed to the
//! presets that touch the window's feature group (the programmer's on the encoder bar, or the
//! pinned one; All = every one) and carry every lit tag. A cell names how it fits the user's
//! selection (all / some / none), its mode letter, the feature groups it holds (each with its
//! colour), a preview drawn from its values, its uses (the fixtures using it in a programmer and
//! the cues that link it, decisions 0293) and whether it is in the user's own.
//!
//! Owns:          PRESET_POOL, WINDOW, the cells, tag_list.
//! Doesn't own:   the pool window (`lumacore-views::pools`, the WebUI), the presets (`preset`).
//! Entry points:  PRESET_POOL, WINDOW, filter_of, tag_list, groups_of.
//! Decisions:     docs/decisions/0288-one-preset-pool-follows-the-programmer, §22.

use std::collections::{BTreeSet, HashMap};

use lumacore_tree::{Preset, PresetMode, ShowData};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::pools::{PoolAction, PoolCell, PoolContext, PoolDef};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The window type's name.
pub const WINDOW: &str = "presets";

/// Tap = the window's action (At by default; decisions 0288).
pub static PRESET_POOL: PoolDef = PoolDef {
    window: WINDOW,
    actions: &[
        PoolAction { name: "At", command: "At {obj}", release: "" },
        PoolAction { name: "SelFix At", command: "SelectFixtures {obj}; At {obj}", release: "" },
        PoolAction { name: "SelFix", command: "SelectFixtures {obj}", release: "" },
        PoolAction { name: "Call", command: "Call {obj}", release: "" },
        PoolAction { name: "None", command: "", release: "" },
    ],
    cells: preset_cells,
    detail: Some(preset_detail),
    head: Some(preset_head),
};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

fn preset_cells(ctx: &PoolContext) -> Vec<PoolCell> {
    let Some(dp) = ctx.show.datapools.get(&ctx.datapool) else { return Vec::new() };
    let selected: Vec<Uuid> = ctx.show.programmer_of(ctx.user).map(|p| p.selected_fixtures.clone()).unwrap_or_default();
    let usage = usage(ctx.show, ctx.user);
    // The cues that link each preset (decisions 0293) count as uses too.
    let cues = lumacore_tree::PresetUses::build(ctx.show);
    let feature = ctx.filter.feature.as_deref().filter(|f| !f.is_empty() && !f.eq_ignore_ascii_case("all"));
    dp.presets
        .iter()
        .filter_map(|(&no, p)| {
            let groups = groups_of(ctx.show, p);
            if feature.is_some_and(|f| !groups.iter().any(|(g, _)| g.eq_ignore_ascii_case(f))) {
                return None;
            }
            if !ctx.filter.tags.iter().all(|t| p.meta.annotation.tags.contains(t)) {
                return None;
            }
            let mut cell = PoolCell::of(no, &p.meta, ctx);
            cell.applicability = fit(ctx.show, p, &selected);
            let (used, mine) = usage.get(&p.meta.identity.guid).copied().unwrap_or((0, false));
            let in_cues = cues.count(p.meta.identity.guid);
            cell.selected = mine;
            cell.badges = vec![letter(&p.mode).to_string()];
            cell.extra = Some(json!({
                "mode": letter(&p.mode),
                "groups": groups.iter().map(|(name, color)| json!({ "name": name, "color": color })).collect::<Vec<_>>(),
                "preview": preview(ctx.show, p, &groups),
                "use": used + in_cues,
                "fixtures": used,
                "cues": in_cues,
                "mine": mine,
                "tags": p.meta.annotation.tags.iter().filter_map(|t| tag_name(ctx.show, *t)).collect::<Vec<_>>(),
            }));
            Some(cell)
        })
        .collect()
}

/// Beside the cells: the tags the pool's presets carry (the chip row), every number in use and
/// the ones the filter hid (Fixed slots draws those as gaps; Packed skips them for new ones).
fn preset_head(ctx: &PoolContext) -> Value {
    let Some(dp) = ctx.show.datapools.get(&ctx.datapool) else { return json!({ "tags": [], "used": [], "hidden": [] }) };
    let shown: BTreeSet<u32> = preset_cells(ctx).iter().map(|c| c.no).collect();
    let mut carried: Vec<Uuid> = Vec::new();
    for p in dp.presets.values() {
        for t in &p.meta.annotation.tags {
            if !carried.contains(t) {
                carried.push(*t);
            }
        }
    }
    let tags: Vec<Value> = carried.iter().filter_map(|t| tag_name(ctx.show, *t).map(|n| json!({ "guid": t.to_string(), "name": n }))).collect();
    json!({
        "tags": tags,
        "used": dp.presets.keys().collect::<Vec<_>>(),
        "hidden": dp.presets.keys().filter(|n| !shown.contains(n)).collect::<Vec<_>>(),
        "feature": ctx.filter.feature,
    })
}

/// One preset's values for its editor (`Edit Preset n`): a row per fixture (Universal: one row
/// "every fixture"), a column per attribute.
fn preset_detail(ctx: &PoolContext, no: u32) -> Option<Value> {
    let p = ctx.show.datapools.get(&ctx.datapool)?.presets.get(&no)?;
    let attrs = p.attributes();
    let mut rows: Vec<(Option<Uuid>, Vec<Value>)> = Vec::new();
    for e in &p.values {
        let i = attrs.iter().position(|a| *a == e.attribute).expect("listed");
        let row = match rows.iter_mut().position(|(f, _)| *f == e.fixture_id) {
            Some(r) => r,
            None => {
                rows.push((e.fixture_id, vec![Value::Null; attrs.len()]));
                rows.len() - 1
            }
        };
        rows[row].1[i] = json!(readout(&e.value));
    }
    let rows: Vec<Value> = rows
        .into_iter()
        .map(|(f, cells)| {
            let label = match f {
                Some(g) => ctx.show.fixture_id_for_guid(g).map_or_else(|| "missing fixture".to_string(), |id| format!("Fixture {id}")),
                None => "every fixture".to_string(),
            };
            json!({ "fixture": label, "values": cells })
        })
        .collect();
    Some(json!({ "no": no, "name": p.meta.identity.name, "mode": letter(&p.mode), "attributes": attrs, "rows": rows }))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The `pools` feed's `{feature, tags}` as a filter: a pinned feature group is its GUID, a
/// following window sends the encoder bar's group name; the tags are the window's setting.
pub fn filter_of(show: &ShowData, v: &Value) -> crate::pools::PoolFilter {
    let feature = v["feature"].as_str().filter(|f| !f.is_empty()).map(|f| match f.parse::<Uuid>() {
        Ok(g) => show.feature_groups.values().find(|x| x.meta.identity.guid == g).and_then(|x| x.meta.identity.name.clone()).unwrap_or_default(),
        Err(_) => f.to_string(),
    });
    crate::pools::PoolFilter { feature, tags: v["tags"].as_str().map(tag_list).unwrap_or_default() }
}

/// A window's `Tags` setting: tag GUIDs, comma separated.
pub fn tag_list(s: &str) -> Vec<Uuid> {
    s.split(',').filter_map(|t| t.trim().parse().ok()).collect()
}

fn tag_name(show: &ShowData, guid: Uuid) -> Option<String> {
    lumacore_tree::tags::tag::all(&show.tags).into_iter().find(|(_, t)| t.meta.identity.guid == guid).map(|(_, t)| t.name().to_string())
}

fn letter(mode: &PresetMode) -> &'static str {
    match mode {
        PresetMode::Selective => "S",
        PresetMode::Global => "G",
        PresetMode::Universal => "U",
    }
}

/// The show's feature groups a preset's attributes belong to, in the show's order, with their
/// colours (`#rrggbb`, or null).
pub fn groups_of(show: &ShowData, p: &Preset) -> Vec<(String, Value)> {
    let ids: BTreeSet<Uuid> = p.attributes().iter().filter_map(|a| show.attribute_defs.get(*a).and_then(|d| d.feature_group)).collect();
    show.feature_groups
        .values()
        .filter(|g| ids.contains(&g.meta.identity.guid))
        .map(|g| {
            let color = g.meta.annotation.paint.map_or(Value::Null, |[r, gr, b, _]| json!(format!("#{r:02x}{gr:02x}{b:02x}")));
            (g.meta.name().unwrap_or_default().to_string(), color)
        })
        .collect()
}

/// How the preset fits the selected fixtures: all, some, none (None: nothing selected).
fn fit(show: &ShowData, p: &Preset, selected: &[Uuid]) -> Option<&'static str> {
    if selected.is_empty() {
        return None;
    }
    let own: BTreeSet<Uuid> = p.fixtures().collect();
    let universal: BTreeSet<String> = p.values.iter().filter(|e| e.fixture_id.is_none()).map(|e| e.attribute.to_ascii_lowercase()).collect();
    let patch = &show.devices.dmx.patch;
    let mut by_type: HashMap<(Uuid, Uuid, Uuid), bool> = HashMap::new();
    let hits = selected
        .iter()
        .filter(|g| {
            own.contains(g)
                || (!universal.is_empty()
                    && patch.fixtures.values().find(|f| f.meta.identity.guid == **g).is_some_and(|f| {
                        *by_type.entry((f.mvr.fixture_type, f.mvr.revision, f.mvr.mode)).or_insert_with(|| {
                            patch
                                .fixture_types
                                .values()
                                .find(|t| t.meta.identity.guid == f.mvr.fixture_type)
                                .and_then(|t| t.mode_of(f.mvr.revision, f.mvr.mode))
                                .is_some_and(|(_, m)| m.channel_functions.iter().any(|cf| universal.contains(&cf.attribute_name.to_ascii_lowercase())))
                        })
                    }))
        })
        .count();
    Some(match hits {
        0 => "none",
        n if n == selected.len() => "all",
        _ => "some",
    })
}

/// Per preset: the fixtures whose values in any programmer came from it, and whether `user`'s
/// own programmer holds one.
fn usage(show: &ShowData, user: &str) -> HashMap<Uuid, (usize, bool)> {
    let mut fixtures: HashMap<Uuid, BTreeSet<Uuid>> = HashMap::new();
    let mut mine: BTreeSet<Uuid> = BTreeSet::new();
    for u in show.users.values() {
        let own = u.login.eq_ignore_ascii_case(user);
        for ((f, _), pv) in &u.programmer.values {
            if let Some(p) = pv.value.preset {
                fixtures.entry(p).or_default().insert(*f);
                if own {
                    mine.insert(p);
                }
            }
        }
    }
    fixtures.into_iter().map(|(p, fs)| (p, (fs.len(), mine.contains(&p)))).collect()
}

/// What the cell draws of the values (the first fixture's): a colour from RGB, a dimmer level,
/// a pan / tilt dot, else each other feature group's short name.
fn preview(show: &ShowData, p: &Preset, groups: &[(String, Value)]) -> Vec<Value> {
    let first = |names: &[&str]| -> Option<f64> {
        p.values.iter().find(|e| names.iter().any(|n| e.attribute.eq_ignore_ascii_case(n))).and_then(|e| normalized(show, &e.attribute, &e.value))
    };
    let mut out = Vec::new();
    let rgb = [first(&["ColorRGB_R", "ColorAdd_R"]), first(&["ColorRGB_G", "ColorAdd_G"]), first(&["ColorRGB_B", "ColorAdd_B"])];
    let has_colour = rgb.iter().any(Option::is_some);
    if has_colour {
        let [r, g, b] = rgb.map(|c| (c.unwrap_or(0.0).clamp(0.0, 1.0) * 255.0).round() as u8);
        out.push(json!({ "kind": "color", "rgb": format!("#{r:02x}{g:02x}{b:02x}") }));
    }
    if let Some(d) = first(&["Dimmer"]) {
        out.push(json!({ "kind": "level", "value": d }));
    }
    let (pan, tilt) = (first(&["Pan"]), first(&["Tilt"]));
    if pan.is_some() || tilt.is_some() {
        out.push(json!({ "kind": "xy", "x": pan.unwrap_or(0.5), "y": tilt.unwrap_or(0.5) }));
    }
    let (has_level, has_xy) = (out.iter().any(|v| v["kind"] == "level"), out.iter().any(|v| v["kind"] == "xy"));
    let drawn = |g: &str| {
        (has_colour && g.eq_ignore_ascii_case("color"))
            || (has_level && g.eq_ignore_ascii_case("dimmer"))
            || (has_xy && g.eq_ignore_ascii_case("position"))
    };
    for (g, _) in groups {
        if !drawn(g) {
            out.push(json!({ "kind": "text", "text": g.chars().take(4).collect::<String>().to_uppercase() }));
        }
    }
    out
}

/// A value's first step as 0–1 of its attribute's range (a physical value by the attribute's
/// definition when it has one: Pan ±270°, Tilt ±135°).
fn normalized(_show: &ShowData, attr: &str, v: &lumacore_base::Value) -> Option<f64> {
    use lumacore_base::value::Absolute;
    match v.steps.first()?.absolute? {
        Absolute::Normalized(p) => Some(p / 100.0),
        Absolute::Physical(x) if attr.eq_ignore_ascii_case("pan") => Some((x + 270.0) / 540.0),
        Absolute::Physical(x) if attr.eq_ignore_ascii_case("tilt") => Some((x + 135.0) / 270.0),
        Absolute::Physical(x) => Some(x / 100.0),
    }
}

/// A value as the editor shows it.
fn readout(v: &lumacore_base::Value) -> String {
    use lumacore_base::value::Absolute;
    match v.steps.first().and_then(|s| s.absolute) {
        Some(Absolute::Normalized(p)) => format!("{}%", (p * 10.0).round() / 10.0),
        Some(Absolute::Physical(x)) => format!("{}", (x * 100.0).round() / 100.0),
        None if v.release => "release".to_string(),
        None => "–".to_string(),
    }
}
