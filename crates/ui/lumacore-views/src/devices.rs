// ██████╗ ███████╗██╗   ██╗██╗ ██████╗███████╗███████╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██╔════╝██╔════╝
// ██║  ██║█████╗  ██║   ██║██║██║     █████╗  ███████╗
// ██║  ██║██╔══╝  ╚██╗ ██╔╝██║██║     ██╔══╝  ╚════██║
// ██████╔╝███████╗ ╚████╔╝ ██║╚██████╗███████╗███████║
// ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝ ╚═════╝╚══════╝╚══════╝
// FILE: devices

//! The Device Sheet's rows and values (the approved mockup `2026-10-22-device-sheet`): every
//! device in scope in tree order with its folders as tree rows, the attributes of the types in
//! scope as columns under their feature groups, and — for the rows a window shows — each value's
//! readout, where it comes from and its marks (the names `kit/marks.js` paints). Built for the
//! `devices` feed from the published show, the plan and the rendered frame, off the render tick.
//! Pure.
//!
//! Owns:          `view`, `Params`, `Layer`.
//! Doesn't own:   the merge (`lumacore_render::merge`), the tree order (`patch::tree_order`),
//!                the marks' paint (`kit/marks.js`), the lines a tap runs (`kit/devicegrid.js`).
//! Entry points:  view, Params, Layer.
//! Decisions:     device-grid-and-value-marks, device-sheet-window.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use lumacore_render::merge::{channels, default_fraction, fraction};
use lumacore_render::{Plan, RenderedFrame, Source};
use lumacore_tree::fixtures::Fixture;
use lumacore_tree::fixtures::attribute_def::{NaturalReadout, PhysicalUnit};
use lumacore_tree::fixtures::fixture_type::{AttributeType, ChannelFunction};
use lumacore_tree::programmer::ProgValue;
use lumacore_tree::{SequenceObject, ShowData, Value};
use serde_json::{Map, json};
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The layer a window shows (decisions device-sheet-window): Output and DMX read the rendered
/// frame, the others the values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layer {
    #[default]
    Absolute,
    Fade,
    Delay,
    Output,
    Dmx,
}

impl Layer {
    pub const ALL: [Layer; 5] = [Layer::Absolute, Layer::Fade, Layer::Delay, Layer::Output, Layer::Dmx];

    /// The word a window's `Layer` setting and the feed use.
    pub fn word(self) -> &'static str {
        match self {
            Layer::Absolute => "Absolute",
            Layer::Fade => "Fade",
            Layer::Delay => "Delay",
            Layer::Output => "Output",
            Layer::Dmx => "DMX",
        }
    }

    /// A word back (any case); anything else is Absolute.
    pub fn parse(word: &str) -> Self {
        Self::ALL.into_iter().find(|l| l.word().eq_ignore_ascii_case(word)).unwrap_or_default()
    }
}

/// What a value cell shows when the value comes from a preset (decisions 0293, the window's
/// `Show` setting): its value, the link — the preset it comes from (`P3 Split`) — or both
/// (`P3 Split · 85`). The preset bar shows whatever it is set to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Show {
    Value,
    Link,
    #[default]
    Both,
}

impl Show {
    pub const ALL: [Show; 3] = [Show::Value, Show::Link, Show::Both];

    pub fn word(self) -> &'static str {
        match self {
            Show::Value => "Value",
            Show::Link => "Link",
            Show::Both => "Both",
        }
    }

    /// A word back (any case); anything else is Both.
    pub fn parse(word: &str) -> Self {
        Self::ALL.into_iter().find(|s| s.word().eq_ignore_ascii_case(word)).unwrap_or_default()
    }
}

/// What a window asks for: its settings and the rows it shows.
#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub layer: Layer,
    /// A linked value's text (decisions 0293).
    pub show: Show,
    /// Only devices with values in the viewer's programmer.
    pub programmer_only: bool,
    /// Only the viewer's selected devices.
    pub selected_only: bool,
    /// Without folders: the selection on top, in the order it was picked.
    pub selection_first: bool,
    /// The patch tree's folders as tree rows.
    pub folders: bool,
    /// The first row the window shows and how many: only these carry cells.
    pub from: usize,
    pub count: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self { layer: Layer::Absolute, show: Show::Both, programmer_only: false, selected_only: false, selection_first: true, folders: true, from: 0, count: 60 }
    }
}

/// The most rows one build gives cells (a window shows far fewer).
pub const MAX_ROWS: usize = 200;

/// A cell's state: the value marks' names (`kit/marks.js`, decisions device-grid-and-value-marks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Default,
    ProgActive,
    ProgInactive,
    New,
    Lower,
    Tracked,
    Blocked,
    Other,
    OtherTracked,
    Park,
    /// A timing layer's value from a cue (the layer's colour as text).
    Cue,
    /// Output and DMX: what leaves the station.
    Out,
}

impl State {
    fn word(self) -> &'static str {
        match self {
            State::Default => "def",
            State::ProgActive => "prgA",
            State::ProgInactive => "prgI",
            State::New => "new",
            State::Lower => "lower",
            State::Tracked => "track",
            State::Blocked => "block",
            State::Other => "other",
            State::OtherTracked => "otherT",
            State::Park => "park",
            State::Cue => "cue",
            State::Out => "out",
        }
    }
}

/// A running sequence as the cells read it: its number, GUID, current cue and what its current
/// and previous cue hold (fixture, attribute → the cue that stored it, its value, its timing).
struct Running<'a> {
    no: u32,
    guid: Uuid,
    sequence: &'a Arc<SequenceObject>,
    cue: u32,
    now: BTreeMap<(Uuid, &'a str), lumacore_tree::tree::Tracked<'a>>,
    before: BTreeMap<(Uuid, &'a str), lumacore_tree::tree::Tracked<'a>>,
}

/// One build's reads.
struct Ctx<'a> {
    show: &'a ShowData,
    prog: HashMap<(Uuid, &'a str), &'a ProgValue>,
    plan: Option<&'a Plan>,
    frame: Option<&'a RenderedFrame>,
    now_ms: u64,
    layer: Layer,
    show_as: Show,
    selected_sequence: Option<Uuid>,
    running: Vec<Running<'a>>,
    /// The plan's first channel slot of each attribute index.
    first_channel: HashMap<u32, usize>,
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The sheet for `user` (decisions device-sheet-window). `plan` / `frame` = the engine's (None:
/// defaults, no output); `now_ms` = the session clock (a fade's progress).
pub fn view(
    show: &ShowData,
    user: &str,
    plan: Option<&Plan>,
    frame: Option<&RenderedFrame>,
    now_ms: u64,
    p: &Params,
) -> serde_json::Value {
    let patch = &show.devices.dmx.patch;
    let programmer = show.programmer_of(user);
    let selected: &[Uuid] = programmer.map_or(&[], |pr| pr.selected_fixtures.as_slice());
    let pick: HashMap<Uuid, usize> = selected.iter().enumerate().map(|(i, g)| (*g, i)).collect();
    // A device in the viewer's programmer: any value active?
    let mut programmed: HashMap<Uuid, bool> = HashMap::new();
    for ((g, _), pv) in programmer.map(|pr| &pr.values).into_iter().flatten() {
        *programmed.entry(*g).or_default() |= pv.active;
    }

    // ----- Rows -----
    let order = crate::patch::tree_order(&show.devices);
    let holds: BTreeSet<Uuid> = patch.fixtures.values().filter_map(|f| f.mvr.parent).collect();
    let by_guid: HashMap<Uuid, &Fixture> = order.iter().map(|f| (f.meta.identity.guid, *f)).collect();
    let folder_of = |f: &Fixture| f.mvr.parent.filter(|g| by_guid.contains_key(g));
    let is_folder = |f: &Fixture| holds.contains(&f.meta.identity.guid);
    let keep = |f: &Fixture| {
        let g = f.meta.identity.guid;
        (!p.programmer_only || programmed.contains_key(&g)) && (!p.selected_only || pick.contains_key(&g))
    };
    // A folder's devices in scope, deep (its cells and its bar read them).
    let mut below: HashMap<Uuid, Vec<&Fixture>> = HashMap::new();
    for f in order.iter().filter(|f| !is_folder(f) && keep(f)) {
        let mut at = folder_of(f);
        let mut hops = 0;
        while let Some(g) = at.filter(|_| hops < 64) {
            below.entry(g).or_default().push(f);
            at = by_guid.get(&g).and_then(|parent| folder_of(parent));
            hops += 1;
        }
    }
    let depth = |f: &Fixture| {
        let (mut d, mut at) = (0, folder_of(f));
        while let Some(g) = at.filter(|_| d < 64) {
            d += 1;
            at = by_guid.get(&g).and_then(|parent| folder_of(parent));
        }
        d
    };
    let mut rows: Vec<(&Fixture, bool, usize)> = if p.folders {
        order
            .iter()
            .filter(|f| if is_folder(f) { below.contains_key(&f.meta.identity.guid) } else { keep(f) })
            .map(|f| (*f, is_folder(f), depth(f)))
            .collect()
    } else {
        order.iter().filter(|f| !is_folder(f) && keep(f)).map(|f| (*f, false, 0)).collect()
    };
    if !p.folders && p.selection_first {
        rows.sort_by_key(|(f, ..)| pick.get(&f.meta.identity.guid).copied().unwrap_or(usize::MAX));
    }

    // ----- Columns: the union of the attributes in scope, by feature group -----
    let mut types: HashMap<usize, Vec<(&str, &ChannelFunction)>> = HashMap::new();
    let mut attrs_of = |f: &Fixture| -> Vec<(&str, &ChannelFunction)> {
        let cfs = channels(show, f);
        types
            .entry(cfs.as_ptr() as usize)
            .or_insert_with(|| {
                let mut seen = BTreeSet::new();
                cfs.iter().filter(|cf| !cf.is_virtual && seen.insert(cf.attribute_name.as_str())).map(|cf| (cf.attribute_name.as_str(), cf)).collect()
            })
            .clone()
    };
    let mut used: BTreeSet<&str> = BTreeSet::new();
    for (f, folder, _) in &rows {
        if !folder {
            used.extend(attrs_of(f).into_iter().map(|(a, _)| a));
        }
    }
    let columns = columns(show, &used);

    // ----- Cells: the rows the window shows -----
    let ctx = context(show, user, plan, frame, now_ms, p.layer, p.show);
    let from = p.from.min(rows.len());
    let to = (from + p.count.min(MAX_ROWS)).min(rows.len());
    let mut cells = Map::new();
    for (f, folder, _) in &rows[from..to] {
        let mut row = Map::new();
        if *folder {
            let kids = below.get(&f.meta.identity.guid).map_or(&[][..], Vec::as_slice);
            for c in &columns {
                row.insert(c.0.to_string(), folder_cell(&ctx, kids, c.0, &mut attrs_of));
            }
        } else {
            let mine = attrs_of(f);
            for c in &columns {
                let v = mine.iter().find(|(a, _)| *a == c.0).map_or(serde_json::Value::Null, |(a, cf)| cell(&ctx, f, a, cf));
                row.insert(c.0.to_string(), v);
            }
            if let Some(mix) = swatch(&ctx, f, &mine) {
                row.insert("#mix".into(), json!(mix));
            }
        }
        cells.insert(f.meta.identity.guid.to_string(), serde_json::Value::Object(row));
    }

    let type_name: HashMap<Uuid, String> =
        patch.fixture_types.values().map(|t| (t.meta.identity.guid, t.meta.identity.name.clone().unwrap_or_default())).collect();
    // The fixture's colour on its name cell, as this viewer sees it (decisions 0294).
    let paint = lumacore_objects::paint::Painter::new(show, user);
    let rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|(f, folder, d)| {
            let g = f.meta.identity.guid;
            let bar = if *folder {
                below.get(&g).is_some_and(|kids| kids.iter().any(|k| programmed.contains_key(&k.meta.identity.guid))).then_some("abs")
            } else {
                programmed.get(&g).map(|active| if *active { "abs" } else { "deact" })
            };
            json!({
                "key": g,
                "kind": if *folder { "folder" } else { "fixture" },
                "id": f.handle(),
                "fid": f.mvr.fid,
                "name": f.meta.identity.name.clone().unwrap_or_default(),
                "paint": paint.of(&f.meta).hex,
                "type": type_name.get(&f.mvr.fixture_type).cloned().unwrap_or_default(),
                "id_type": f.mvr.id_type.name(),
                "cid": f.mvr.cid,
                "depth": d,
                "count": if *folder { below.get(&g).map_or(0, Vec::len) } else { 0 },
                "sel": pick.get(&g),
                "bar": bar,
            })
        })
        .collect();
    json!({
        "layer": p.layer.word(),
        "show": p.show.word(),
        "layers": Layer::ALL.iter().map(|l| l.word()).collect::<Vec<_>>(),
        "total": rows.len(),
        "from": from,
        "rows": rows,
        "columns": columns.iter().map(|c| json!({ "id": c.0, "label": c.1, "group": c.2, "priority": c.3 })).collect::<Vec<_>>(),
        "cells": cells,
        "selected": selected.len(),
    })
}

/// The columns: (attribute, label, feature group, priority — 1 = a group's first, the
/// Condensed set; 2 = the rest; 9 = a sub-attribute, Extended only), in the show's feature
/// group order, each group's attributes by their position; attributes without a group last.
fn columns<'a>(show: &ShowData, used: &BTreeSet<&'a str>) -> Vec<(&'a str, String, String, u8)> {
    let group_no: HashMap<Uuid, (u32, String)> =
        show.feature_groups.iter().map(|(n, g)| (g.meta.identity.guid, (*n, g.meta.identity.name.clone().unwrap_or_default()))).collect();
    let mut pretty_uses: HashMap<&str, usize> = HashMap::new();
    for a in used {
        if let Some(d) = show.attribute_defs.get(*a) {
            *pretty_uses.entry(d.pretty.as_str()).or_default() += 1;
        }
    }
    let mut list: Vec<((u32, u32, &str), &str, String, String, bool)> = used
        .iter()
        .map(|a| {
            let def = show.attribute_defs.get(*a);
            let (no, group) = def.and_then(|d| d.feature_group).and_then(|g| group_no.get(&g).cloned()).unwrap_or((u32::MAX, "Other".into()));
            // The short name (Dim, Pan) when no other column in scope has it; else the name itself.
            let label = def.map(|d| d.pretty.clone()).filter(|s| !s.is_empty() && pretty_uses.get(s.as_str()) == Some(&1)).unwrap_or_else(|| (*a).to_string());
            ((no, def.map_or(u32::MAX, |d| d.position), *a), *a, label, group, def.is_some_and(|d| d.main_attribute.is_some()))
        })
        .collect();
    list.sort_by(|x, y| x.0.cmp(&y.0));
    let mut out = Vec::with_capacity(list.len());
    let mut last = String::new();
    for (_, a, label, group, sub) in list {
        let priority = if sub { 9 } else if group != last { 1 } else { 2 };
        last.clone_from(&group);
        out.push((a, label, group, priority));
    }
    out
}

/// What one build reads, once.
fn context<'a>(
    show: &'a ShowData,
    user: &str,
    plan: Option<&'a Plan>,
    frame: Option<&'a RenderedFrame>,
    now_ms: u64,
    layer: Layer,
    show_as: Show,
) -> Ctx<'a> {
    let prog = show
        .programmer_of(user)
        .map(|pr| pr.values.iter().map(|((g, a), pv)| ((*g, a.as_str()), pv)).collect())
        .unwrap_or_default();
    let running = show
        .playbacks
        .live
        .sequences
        .iter()
        .filter_map(|(guid, st)| {
            let (_, no, sequence) = show.sequence_by_guid(*guid)?;
            let before = sequence.cues.range(..st.current_cue).next_back().map_or(lumacore_tree::tree::CUE_ZERO, |(k, _)| *k);
            Some(Running {
                no,
                guid: *guid,
                sequence,
                cue: st.current_cue,
                now: sequence.tracked(st.current_cue),
                before: if before == st.current_cue { BTreeMap::new() } else { sequence.tracked(before) },
            })
        })
        .collect();
    let mut first_channel = HashMap::new();
    if let Some(plan) = plan {
        for (i, slot) in plan.channel.channels.iter().enumerate() {
            first_channel.entry(slot.attr).or_insert(i);
        }
    }
    Ctx {
        show,
        prog,
        plan,
        frame,
        now_ms,
        layer,
        show_as,
        selected_sequence: show.user(user).and_then(|u| u.selected.sequence),
        running,
        first_channel,
    }
}

/// One device's cell: its readout (`t`), state (`s`), position 0–1 (`n`), the layers holding data
/// (`l`, the bars), a running fade's progress (`f`) and where it comes from (`src`, the hover).
fn cell(ctx: &Ctx, f: &Fixture, attr: &str, cf: &ChannelFunction) -> serde_json::Value {
    let g = f.meta.identity.guid;
    let mut layers: Vec<&str> = Vec::new();
    // The preset the value comes from (decisions 0293): the programmer's link, or the cue's.
    let mut link: Option<Uuid> = None;
    let (state, value, src): (State, Option<&Value>, String) = if let Some(pv) = ctx.prog.get(&(g, attr)) {
        link = pv.value.preset;
        layers.push(if pv.active { "abs" } else { "deact" });
        if pv.timing.fade.is_some() {
            layers.push("fade");
        }
        if pv.timing.delay.is_some() {
            layers.push("delay");
        }
        let state = if pv.active { State::ProgActive } else { State::ProgInactive };
        let src = if pv.active { "your programmer, active" } else { "your programmer, not active: kept, not stored" };
        (state, Some(&pv.value), src.to_string())
    } else {
        match ctx.plan.and_then(|p| p.resolved(g, attr)) {
            Some(r) => match r.source {
                Source::Park => {
                    layers.push("park");
                    (State::Park, Some(&r.value), "parked".to_string())
                }
                Source::Programmer(n) => (State::Other, Some(&r.value), format!("User {n}'s programmer")),
                Source::Sequence(no) => {
                    link = ctx.running.iter().find(|run| run.no == no).and_then(|run| run.now.get(&(g, attr))).and_then(|t| t.value.link());
                    let (state, src) = sequence_state(ctx, no, g, attr, cf, &r.value);
                    (state, Some(&r.value), src)
                }
            },
            None => (State::Default, None, "at its default".to_string()),
        }
    };
    let n = value.and_then(|v| fraction(v, cf)).unwrap_or_else(|| default_fraction(cf));
    let shown = readout(ctx.show, attr, cf, n);
    let preset = link.map(|g| (g, ctx.show.preset_by_guid(g)));
    let (text, src) = match preset {
        Some((_, Some((_, no, p)))) => {
            layers.push("preset");
            let name = p.meta.name().unwrap_or_default();
            let tag = if name.is_empty() { format!("P{no}") } else { format!("P{no} {name}") };
            let text = match ctx.show_as {
                Show::Value => shown.clone(),
                Show::Link => tag.clone(),
                Show::Both => format!("{tag} · {shown}"),
            };
            let named = if name.is_empty() { String::new() } else { format!(" \"{name}\"") };
            (text, format!("{src} · Preset {no}{named} ({shown})"))
        }
        // A link to a preset that is gone: kept and listed (decisions §35).
        Some((g, None)) => {
            layers.push("preset");
            (shown.clone(), format!("{src} · a preset that is gone ({})", &g.to_string()[..8]))
        }
        None => (shown.clone(), src),
    };
    let mut out = json!({ "t": text, "s": state.word(), "n": round3(n), "src": src });
    if let Some((_, Some((_, no, p)))) = preset {
        out["p"] = json!({ "no": no, "name": p.meta.name().unwrap_or_default() });
    }
    if !layers.is_empty() {
        out["l"] = json!(layers);
    }
    if let Some(progress) = fading(ctx, g, attr) {
        out["f"] = json!(round3(progress));
    }
    match ctx.layer {
        Layer::Absolute => out,
        Layer::Fade | Layer::Delay => timing_cell(ctx, g, attr, state, out),
        Layer::Output | Layer::Dmx => output_cell(ctx, g, attr, out),
    }
}

/// A cue value's state: new, lower, tracked or blocked in the current cue of the selected
/// sequence; another sequence's are Other (new or blocked) and OtherTracked.
fn sequence_state(ctx: &Ctx, no: u32, g: Uuid, attr: &str, cf: &ChannelFunction, value: &Value) -> (State, String) {
    let Some(run) = ctx.running.iter().find(|r| r.no == no) else {
        return (State::Other, format!("Sequence {no}"));
    };
    let name = run.sequence.meta.identity.name.clone().unwrap_or_default();
    let cue = lumacore_tree::cue_label(run.cue);
    let now = run.now.get(&(g, attr));
    let stored = now.is_some_and(|t| t.cue == run.cue);
    // A link compares as a link (decisions 0293): the same preset again is blocked.
    let blocked = |prev: &lumacore_tree::tree::Tracked| now.is_some_and(|t| t.value == prev.value);
    let state = match (stored, run.before.get(&(g, attr))) {
        (false, _) => State::Tracked,
        (true, Some(prev)) if blocked(prev) => State::Blocked,
        (true, Some(prev)) if cf.attribute_type == AttributeType::Dimmer
            && prev.value.resolve(g, attr, ctx.show).and_then(|v| fraction(v, cf)).zip(fraction(value, cf)).is_some_and(|(was, now)| now < was) =>
        {
            State::Lower
        }
        (true, _) => State::New,
    };
    let selected = ctx.selected_sequence == Some(run.guid);
    let word = match state {
        State::Tracked => "tracked",
        State::Blocked => "blocked",
        State::Lower => "new, lower",
        _ => "new",
    };
    let state = match (selected, state) {
        (true, s) => s,
        (false, State::Tracked) => State::OtherTracked,
        (false, _) => State::Other,
    };
    let whose = if selected { "" } else { " (not selected)" };
    (state, format!("Sequence {no} \"{name}\"{whose}, cue {cue}: {word}"))
}

/// Fade or Delay: the programmer value's own time, or the time of the current cue's value.
fn timing_cell(ctx: &Ctx, g: Uuid, attr: &str, state: State, mut out: serde_json::Value) -> serde_json::Value {
    let fade = ctx.layer == Layer::Fade;
    let mine = ctx.prog.get(&(g, attr)).and_then(|pv| if fade { pv.timing.fade } else { pv.timing.delay });
    let (seconds, state) = match (mine, state) {
        (Some(s), _) => (Some(s), state),
        (None, State::ProgActive | State::ProgInactive) => (None, state),
        _ => (cue_time(ctx, g, attr, fade), State::Cue),
    };
    out["t"] = json!(seconds.map_or(String::new(), |s| format!("{}s", trim(s))));
    out["s"] = json!(if seconds.is_some() { state.word() } else { State::Default.word() });
    out.as_object_mut().map(|o| o.remove("f"));
    out
}

/// The current cue's own time for this value (its value timing, else its part's), when the cue
/// stored it.
fn cue_time(ctx: &Ctx, g: Uuid, attr: &str, fade: bool) -> Option<f64> {
    ctx.running.iter().find_map(|run| {
        let t = run.now.get(&(g, attr)).filter(|t| t.cue == run.cue)?;
        let part = run.sequence.cue(t.cue)?.parts.get(&t.part)?;
        let own = part.values.get(t.index).and_then(|v| if fade { v.timing.fade } else { v.timing.delay });
        Some(own.unwrap_or(if fade { part.fade_in } else { part.delay_in }))
    })
}

/// Output (after masters, 0–100 of the channel) or DMX (the coarse byte), from the frame.
fn output_cell(ctx: &Ctx, g: Uuid, attr: &str, mut out: serde_json::Value) -> serde_json::Value {
    let byte = |at: usize| -> Option<u8> {
        let plan = ctx.plan?;
        let slot = plan.channel.channels.get(at)?;
        let universe = *plan.channel.universes.get(usize::from(slot.universe))?;
        ctx.frame?.get(universe)?.get(usize::from(slot.at)).copied()
    };
    let at = ctx
        .plan
        .and_then(|p| p.channel.attr_index.get(&(g, attr.to_string())).copied())
        .and_then(|i| ctx.first_channel.get(&i).copied());
    let wide = at.and_then(|at| ctx.plan?.channel.channels.get(at)).is_some_and(|s| s.bytes > 1);
    let (coarse, fine) = match at {
        Some(at) => (byte(at), if wide { byte(at + 1) } else { None }),
        None => (None, None),
    };
    let level = coarse.map(|c| (f64::from(c) * 256.0 + f64::from(fine.unwrap_or(0))) / if wide { 65535.0 } else { 65280.0 });
    out["t"] = json!(match (ctx.layer, coarse, level) {
        (Layer::Dmx, Some(c), _) => c.to_string(),
        (_, _, Some(l)) => format!("{:.0}", (l.min(1.0)) * 100.0),
        _ => String::new(),
    });
    out["s"] = json!(State::Out.word());
    if let Some(l) = level {
        out["n"] = json!(round3(l.min(1.0)));
    }
    out.as_object_mut().map(|o| o.remove("l"));
    out
}

/// A running cue fade's progress 0–1 on this attribute (None: still).
fn fading(ctx: &Ctx, g: Uuid, attr: &str) -> Option<f64> {
    let plan = ctx.plan?;
    let i = *plan.channel.attr_index.get(&(g, attr.to_string()))?;
    let fade = plan.value.channels.get(*ctx.first_channel.get(&i)?)?.fade.as_ref()?;
    if fade.snap || fade.fade_ms == 0 || ctx.now_ms >= fade.end_ms() {
        return None;
    }
    let begin = fade.begin_ms();
    Some(if ctx.now_ms <= begin { 0.0 } else { (ctx.now_ms - begin) as f64 / fade.fade_ms as f64 })
}

/// A folder's cell: the value every device below shares, `·` where they differ; a bar when any
/// device below holds it in your programmer; null when none has the attribute.
fn folder_cell<'a>(
    ctx: &Ctx,
    kids: &[&'a Fixture],
    attr: &str,
    attrs_of: &mut impl FnMut(&'a Fixture) -> Vec<(&'a str, &'a ChannelFunction)>,
) -> serde_json::Value {
    let mut shared: Option<(f64, &ChannelFunction)> = None;
    let mut mixed = false;
    let mut any = false;
    let mut mine = false;
    for k in kids {
        let Some((_, cf)) = attrs_of(k).into_iter().find(|(a, _)| *a == attr) else { continue };
        any = true;
        let g = k.meta.identity.guid;
        mine |= ctx.prog.contains_key(&(g, attr));
        let value = match ctx.prog.get(&(g, attr)) {
            Some(pv) => Some(pv.value.clone()),
            None => ctx.plan.and_then(|p| p.current_value(g, attr)),
        };
        let n = value.as_ref().and_then(|v| fraction(v, cf)).unwrap_or_else(|| default_fraction(cf));
        match shared {
            None => shared = Some((n, cf)),
            Some((first, _)) if (first - n).abs() > 1e-6 => mixed = true,
            Some(_) => {}
        }
        if mixed && mine {
            break;
        }
    }
    if !any {
        return serde_json::Value::Null;
    }
    let mut out = match (mixed, shared) {
        (false, Some((n, cf))) => json!({ "t": readout(ctx.show, attr, cf, n), "s": "def", "n": round3(n), "src": "every device below" }),
        _ => json!({ "t": "·", "s": "mixed", "src": "the devices below differ" }),
    };
    if mine {
        out["l"] = json!(["abs"]);
    }
    out
}

/// The mixed colour of a device's colour attributes (RGB adds, CMY subtracts), when it has them.
fn swatch(ctx: &Ctx, f: &Fixture, attrs: &[(&str, &ChannelFunction)]) -> Option<String> {
    let g = f.meta.identity.guid;
    let level = |attr: &str, cf: &ChannelFunction| {
        let value = match ctx.prog.get(&(g, attr)) {
            Some(pv) => Some(pv.value.clone()),
            None => ctx.plan.and_then(|p| p.current_value(g, attr)),
        };
        value.as_ref().and_then(|v| fraction(v, cf)).unwrap_or_else(|| default_fraction(cf))
    };
    let (mut rgb, mut add, mut sub) = ([0.0_f64; 3], false, [0.0_f64; 3]);
    let mut subtractive = false;
    for (attr, cf) in attrs {
        let i = match (&cf.attribute_type, *attr) {
            (AttributeType::ColorRed, _) => Some((0, true)),
            (AttributeType::ColorGreen, _) => Some((1, true)),
            (AttributeType::ColorBlue, _) => Some((2, true)),
            (_, a) if a.starts_with("ColorSub_C") => Some((0, false)),
            (_, a) if a.starts_with("ColorSub_M") => Some((1, false)),
            (_, a) if a.starts_with("ColorSub_Y") => Some((2, false)),
            _ => None,
        };
        match i {
            Some((i, true)) => {
                rgb[i] = level(attr, cf);
                add = true;
            }
            Some((i, false)) => {
                sub[i] = level(attr, cf);
                subtractive = true;
            }
            None => {}
        }
    }
    if !add && !subtractive {
        return None;
    }
    let c: Vec<u8> = (0..3)
        .map(|i| {
            let v = if add { rgb[i] } else { 1.0 } * if subtractive { 1.0 - sub[i] } else { 1.0 };
            (v.clamp(0.0, 1.0) * 255.0).round() as u8
        })
        .collect();
    Some(format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The Natural readout of a position: the physical value with its unit when the attribute reads
/// physical and the channel has a range, else 0–100.
fn readout(show: &ShowData, attr: &str, cf: &ChannelFunction, n: f64) -> String {
    let def = show.attribute_defs.get(attr);
    match (def.map(|d| &d.natural_readout), cf.physical_range) {
        (Some(NaturalReadout::Physical), Some((lo, hi))) => {
            let v = lo + n * (hi - lo);
            let unit = match def.map(|d| &d.physical_unit) {
                Some(PhysicalUnit::Angle) => "°",
                Some(PhysicalUnit::Temperature) => "K",
                Some(PhysicalUnit::Frequency) => "Hz",
                _ => "",
            };
            format!("{}{unit}", trim(v))
        }
        _ => format!("{:.0}", n * 100.0),
    }
}

fn trim(v: f64) -> String {
    // `+ 0.0` turns a negative zero (`-0°`) into zero.
    let r = (v * 10.0).round() / 10.0 + 0.0;
    if r.fract() == 0.0 { format!("{r:.0}") } else { format!("{r:.1}") }
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests;
