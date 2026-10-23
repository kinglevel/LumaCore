//! Window types and their settings schemas.
//!
//! A window type describes *what* a window shows (Terminal, Groups pool, …) and
//! which typed settings it has. It is renderer-agnostic: every UI engine
//! (`Root.UIEngines`) decides *how* to draw a type. Published read-only at
//! `Root.Resources.WindowTypes`.
//!
//! Modelled on a `WindowType` registry + per-type `*Settings` classes,
//! but placement is not part of the type (LumaCore uses a split tree, not a grid).
//!
//! **How a window is built ships here** (decisions §30 "12c hardening": two
//! models): besides its settings a type declares its window bar —
//! `actions` (command lines with `Self`), `bar_default` (the ordered item ids
//! a new window shows), `bar_min` (below it the bar hides and the window menu
//! lists the items) — a live `title` with `{Self.<Property>}` reads and
//! `chips` that appear only while their `shown_when` is true. The floor plan
//! (which window, where, its settings) is show / profile data; this is never
//! in a show. Clients get it as JSON (`GET /api/window-types`). Every setting
//! key and action id is a stable string (`data-id="<type>/<id>"`).
//!
//! One state rule: `Self.<Property>` reads a window's setting (by key, any
//! case, `_` ignored: `Self.AutoScroll` = `auto_scroll`), a link's
//! `.Name` / `.No`, or a property the station computes (`Self.Errors`), via
//! [`self_value`], [`eval_state`] and [`render_template`].

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Map, Value, json};

/// The types are the catalog's (decisions §34 21); this module keeps the table and the live view.
pub use lumacore_registry::setting::{FOLLOW_SELECTED, NowSource, SettingDef, SettingKind, list_ids, same_key};
pub use lumacore_registry::window_type::{
    ActionDef, BarMin, ChipDef, WINDOW_BAR, WINDOW_BAR_ITEMS, WINDOW_BAR_SECTION, WindowType, section,
};

/// What a window looks like live (decisions §30 "12c"): the view a type gives its clients.
pub trait WindowTypeView {
    /// The window's view for clients (the `windows` feed): its live title, every bar item's
    /// state, the chips that show and each setting's value as shown.
    fn view(&self, stored: &Map<String, Value>, ctx: &dyn SelfContext) -> WindowView;
}

impl WindowTypeView for WindowType {
    /// The window's view for clients (the `windows` feed, decisions §30
    /// "12c"): its live title, every bar item's state, the chips that show
    /// and each setting's value as shown.
    fn view(&self, stored: &Map<String, Value>, ctx: &dyn SelfContext) -> WindowView {
        let settings = self.resolve_settings(stored);
        let mut states = BTreeMap::new();
        for def in &self.settings {
            if matches!(def.kind, SettingKind::Bool) && def.key != WINDOW_BAR {
                states.insert(def.key.clone(), settings.get(&def.key).and_then(Value::as_bool) == Some(true));
            }
        }
        for a in &self.actions {
            if let Some(expr) = &a.state {
                states.insert(a.id.clone(), eval_state(self, &settings, expr, ctx));
            }
        }
        let chips = self
            .chips
            .iter()
            .filter(|c| eval_state(self, &settings, &c.shown_when, ctx))
            .map(|c| WindowChip {
                id: c.id.clone(),
                label: render_template(self, &settings, &c.label, ctx),
                line: c.line.clone(),
            })
            .collect();
        let values = self
            .settings
            .iter()
            .map(|def| {
                let v = settings.get(&def.key).cloned().unwrap_or(Value::Null);
                let shown = match &def.kind {
                    SettingKind::Link { kind, .. } if v.as_str() != Some(FOLLOW_SELECTED) => ctx
                        .link(kind, &v)
                        .map(|(no, name)| format!("{no} {name}").trim().to_string())
                        .unwrap_or_else(|| def.label_of(&v)),
                    SettingKind::Colour if v.as_str().is_none_or(str::is_empty) => "Theme".to_string(),
                    _ => def.label_of(&v),
                };
                (def.key.clone(), shown)
            })
            .collect();
        WindowView {
            kind: self.name.clone(),
            title: render_template(self, &settings, &self.title, ctx),
            bar: settings.get(WINDOW_BAR).and_then(Value::as_bool).unwrap_or(false),
            items: list_ids(settings.get(WINDOW_BAR_ITEMS).unwrap_or(&Value::Null)),
            states,
            chips,
            values,
        }
    }
}

/// What a window looks like to its clients right now (the `windows` feed).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowView {
    /// Its window type (`dmx`).
    #[serde(rename = "type")]
    pub kind: String,
    /// The live title (empty: the type has none).
    pub title: String,
    /// `WindowBar`.
    pub bar: bool,
    /// `WindowBarItems`, in order (unknown ids already dropped).
    pub items: Vec<String>,
    /// Every Bool setting and every action with a `state`: lit / checked.
    pub states: BTreeMap<String, bool>,
    /// The chips that show now.
    pub chips: Vec<WindowChip>,
    /// Every setting's value as shown (`1 Front`, `Fit`, `Theme`).
    pub values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowChip {
    pub id: String,
    /// Rendered (`× Search “act 2”`).
    pub label: String,
    pub line: String,
}

// ── Self.<Property> (decisions §30 "12c": one state rule) ─────────────────

/// What a `Self.<Property>` read needs beyond the window's own settings.
pub trait SelfContext {
    /// A link setting's target as (number, name): `kind` is the setting's
    /// link kind (`DataPool`, `Universe`), `value` what is stored (a GUID,
    /// `selected`, a number).
    fn link(&self, kind: &str, value: &Value) -> Option<(u32, String)>;
    /// A property the station works out for the window (`Errors` of a
    /// System Monitor since its `Since` mark).
    fn computed(&self, ty: &WindowType, settings: &Map<String, Value>, name: &str) -> Option<Value>;
}

/// A context with nothing beyond the settings (tests, terminals).
pub struct NoContext;

impl SelfContext for NoContext {
    fn link(&self, _: &str, _: &Value) -> Option<(u32, String)> {
        None
    }
    fn computed(&self, _: &WindowType, _: &Map<String, Value>, _: &str) -> Option<Value> {
        None
    }
}

/// `Self.Universe`, `Self.Universe.Name`, `{Self}.AutoScroll`, `AutoScroll`
/// → the value for a window of type `ty` with `settings` (resolved).
pub fn self_value(ty: &WindowType, settings: &Map<String, Value>, path: &str, ctx: &dyn SelfContext) -> Option<Value> {
    let path = path.trim();
    let path = path
        .strip_prefix("{Self}.")
        .or_else(|| path.strip_prefix("Self."))
        .or_else(|| path.strip_prefix("self."))
        .unwrap_or(path);
    let (first, rest) = match path.split_once('.') {
        Some((a, b)) => (a, Some(b)),
        None => (path, None),
    };
    if let Some(def) = ty.setting(first) {
        let v = settings.get(&def.key).cloned().unwrap_or_else(|| def.default.clone());
        return match (rest, &def.kind) {
            (None, _) => Some(v),
            (Some(field), SettingKind::Link { kind, .. }) => {
                let (no, name) = ctx.link(kind, &v)?;
                if same_key(field, "Name") {
                    Some(json!(name))
                } else if same_key(field, "No") {
                    Some(json!(no))
                } else {
                    None
                }
            }
            _ => None,
        };
    }
    ctx.computed(ty, settings, first)
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
        Value::Null => false,
    }
}

/// A state expression: `Self.X` (true when set / on / non-empty),
/// `!Self.X`, `Self.X=value` (a value or its label, any case).
pub fn eval_state(ty: &WindowType, settings: &Map<String, Value>, expr: &str, ctx: &dyn SelfContext) -> bool {
    let expr = expr.trim();
    if let Some(inner) = expr.strip_prefix('!') {
        return !eval_state(ty, settings, inner, ctx);
    }
    if let Some((lhs, rhs)) = expr.split_once('=') {
        let Some(v) = self_value(ty, settings, lhs, ctx) else {
            return false;
        };
        let rhs = rhs.trim().trim_matches('"');
        let label = ty
            .setting(lhs.trim().trim_start_matches("{Self}.").trim_start_matches("Self."))
            .map(|d| d.label_of(&v))
            .unwrap_or_default();
        return text_of(&v).eq_ignore_ascii_case(rhs) || label.eq_ignore_ascii_case(rhs);
    }
    self_value(ty, settings, expr, ctx).is_some_and(|v| truthy(&v))
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => if *b { "Yes" } else { "No" }.into(),
        other => other.to_string(),
    }
}

/// A template with `{Self.<Property>}` reads (`"U {Self.Universe}"`); a
/// read that finds nothing becomes empty; the result is trimmed.
pub fn render_template(ty: &WindowType, settings: &Map<String, Value>, template: &str, ctx: &dyn SelfContext) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            out.push_str(&rest[open..]);
            rest = "";
            break;
        };
        let read = &after[..close];
        // `{Self}` alone is the window itself: no text.
        if !read.eq_ignore_ascii_case("Self")
            && let Some(v) = self_value(ty, settings, read, ctx) {
                out.push_str(&text_of(&v));
            }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Font size: Default (the profile's size, stored 0) or a size in px
/// (decisions §30 "12b2": named choices, never a bare number).
fn font_size() -> SettingDef {
    let mut options = vec![(json!(0), "Default".to_string())];
    options.extend([10, 12, 14, 16, 18, 20, 24, 28, 32].map(|n| (json!(n), n.to_string())));
    SettingDef::new("font_size", "Font size", SettingKind::Enum { options }, json!(0))
}

/// The largest time mark (JavaScript's safe integer: the page reads it).
pub const MAX_SEQ: i64 = 9_007_199_254_740_991;

/// The bar a new pool window shows (WindowBar board: Tap, DataPool, then
/// what overflows into `…`).
pub const POOL_BAR: &[&str] = &["action", "datapool", "columns", "font_size", "show_empty"];

/// The built-in window types, numbered as published.
/// Settings every pool window has (decisions §22 "Unified pool window",
/// after the usual PoolSettings; tabs Pool · Layout · Colours, decisions §30
/// "12b2"); `actions` are the pool's tap actions, the first being the
/// default (they match the object's `PoolDef`).
pub fn pool_settings(actions: &[&str]) -> Vec<SettingDef> {
    let color = |key: &str, label: &str| {
        SettingDef::new(key, label, SettingKind::Colour, json!("")).hint("Theme = the theme's colour")
    };
    let mut out = section(
        "Pool",
        vec![
            SettingDef::new(
                "action",
                "Tap action",
                SettingKind::Enum {
                    options: actions.iter().map(|a| (json!(a), (*a).to_string())).collect(),
                },
                json!(actions.first().copied().unwrap_or("None")),
            ),
            SettingDef::new(
                "datapool",
                "DataPool",
                SettingKind::Link {
                    kind: "DataPool".into(),
                    follow: Some("Selected DataPool".into()),
                    by_guid: true,
                },
                json!(FOLLOW_SELECTED),
            )
            .hint("Selected DataPool follows your Select DataPool"),
            SettingDef::new(
                "first_index",
                "First index",
                SettingKind::Int { min: 1, max: 9999, named: Vec::new() },
                json!(1),
            )
            .hint("the first slot shown; Store without a number uses the first free one from here"),
        ],
    );
    out.extend(section(
        "Layout",
        vec![
            SettingDef::new("slots", "Slots", SettingKind::Int { min: 0, max: 9999, named: Vec::new() }, json!(0))
                .named(&[(0, "Grow")])
                .hint("Grow = more while scrolling"),
            SettingDef::new("columns", "Columns", SettingKind::Int { min: 0, max: 64, named: Vec::new() }, json!(12))
                .named(&[(0, "Fit")])
                .hint("Fit = by button size"),
            SettingDef::new("rows", "Rows", SettingKind::Int { min: 0, max: 64, named: Vec::new() }, json!(12))
                .named(&[(0, "Fit")])
                .hint("Fit = square buttons, scroll"),
            SettingDef::new(
                "size",
                "Button size",
                SettingKind::Enum {
                    options: vec![
                        (json!(56), "Small".into()),
                        (json!(80), "Medium".into()),
                        (json!(110), "Large".into()),
                    ],
                },
                json!(80),
            ),
            font_size(),
            SettingDef::new("show_title", "Show title", SettingKind::Bool, json!(true)),
            SettingDef::new("show_empty", "Show empty slots", SettingKind::Bool, json!(true)),
        ],
    ));
    out.extend(section(
        "Colours",
        vec![
            color("empty_paint", "Empty"),
            color("for_all_paint", "For All"),
            color("for_some_paint", "For Some"),
            color("for_none_paint", "For None"),
        ],
    ));
    out
}

/// The built-in window types — this crate's rows in the catalog; read them from the catalog
/// (`lumacore_registry::get().window_types()`, decisions §34 21).
pub(crate) fn builtin_window_types() -> BTreeMap<u32, WindowType> {
    let types = vec![
        WindowType::new(
            1,
            "terminal",
            "Terminal",
            "Common",
            "terminal",
            "Command line and output.",
        )
        .keywords(&["command", "commandline", "cli", "shell", "console"])
        .settings(section(
            "Display",
            vec![
                font_size(),
                SettingDef::new("cursor_blink", "Cursor blink", SettingKind::Bool, json!(true)),
            ],
        ))
        .bar(false, &[]),
        WindowType::new(
            2,
            "groups",
            "Groups",
            "Pools",
            "layout-grid",
            "Pool of fixture groups.",
        )
        .keywords(&["group", "pool", "selection", "fixtures"])
        .settings(pool_settings(&["Select", "Edit", "None"]))
        .title("{Self.DataPool.Name}")
        .bar_min(260, 160)
        .bar(true, POOL_BAR),
        // The Sequence pool (decisions §30 "12f": executor style; hold = the HoldAction setting).
        WindowType::new(
            8,
            "sequences",
            "Sequences",
            "Pools",
            "list-ordered",
            "Pool of sequences: each cell the cue before, the active cue and the next.",
        )
        .keywords(&["sequence", "sequences", "cue", "cues", "pool", "executor"])
        .settings({
            let mut s = pool_settings(&["Select", "Go", "Toggle", "Off", "Top", "Edit", "None"]);
            // Executor cells hold three cue lines: bigger buttons than a group's.
            for d in s.iter_mut() {
                match d.key.as_str() {
                    "columns" => d.default = json!(8),
                    "rows" => d.default = json!(5),
                    "size" => d.default = json!(110),
                    _ => {}
                }
            }
            s.extend(section("Pool", vec![SettingDef::new(
                "hold_action",
                "Hold action",
                SettingKind::Enum { options: vec![(json!("Edit"), "Edit".into()), (json!("Sheet"), "Sheet".into())] },
                json!("Edit"),
            )
            .hint("Edit opens the sequence; Sheet the object sheet")]));
            s
        })
        .title("{Self.DataPool.Name}")
        .bar_min(260, 160)
        .bar(true, POOL_BAR),
        // The sequence window (decisions §30 "12f" sequence window A): one sequence's cue sheet.
        WindowType::new(
            9,
            "sequence",
            "Sequence",
            "Sheets",
            "list-start",
            "One sequence's cues and their timing — a sheet or a timeline: open a cue for its feature groups' Fade, Delay and Path, with the curves.",
        )
        .keywords(&["sequence", "cue", "cues", "sheet", "timeline", "timing", "fade", "delay", "curve", "path"])
        .settings(section(
            "Display",
            vec![SettingDef::new(
                "sequence",
                "Sequence",
                SettingKind::Link { kind: "Sequence".into(), follow: Some("Selected sequence".into()), by_guid: true },
                json!(FOLLOW_SELECTED),
            )
            .hint("Selected sequence follows your Select Sequence"),
            // Sequence window B (§30 "12f"): the same cues as a horizontal timeline.
            SettingDef::new(
                "layout",
                "Layout",
                SettingKind::Enum { options: vec![(json!("Sheet"), "Sheet".into()), (json!("Timeline"), "Timeline".into())] },
                json!("Sheet"),
            )
            .hint("Sheet = the cue list; Timeline = the cues side by side over the seconds"),
            SettingDef::new("zoom", "Zoom", SettingKind::Int { min: 20, max: 400, named: Vec::new() }, json!(100))
                .hint("the timeline's scale in % (100 = 60 px a second)")],
        ))
        .title("{Self.Sequence.Name}")
        .bar_min(260, 160)
        .bar(true, &["sequence", "layout", "zoom"]),
        // The Tags pool window (decisions §12 "Tags hardened"): an overview in two panes — the tag
        // tree, and the picked tag's members by kind, Direct or Deep.
        WindowType::new(
            10,
            "tags",
            "Tags",
            "Pools",
            "tag",
            "The tag tree and what each tag carries: tap a tag to see its members, Run action selects its fixtures.",
        )
        .keywords(&["tag", "tags", "label", "layer", "tree", "members"])
        .settings(section(
            "Display",
            vec![
                SettingDef::new("tag", "Tag", SettingKind::Link { kind: "Tag".into(), follow: None, by_guid: true }, json!(""))
                    .hint("the tag the members pane shows; a tap on a tag picks it"),
                SettingDef::new(
                    "members",
                    "Members",
                    SettingKind::Enum { options: vec![(json!("Direct"), "Direct".into()), (json!("Deep"), "Deep".into())] },
                    json!("Deep"),
                )
                .hint("Direct = what carries the tag itself; Deep = also what carries the tags under it"),
                SettingDef::new(
                    "action",
                    "Default action",
                    SettingKind::Enum { options: vec![(json!("Select"), "Select fixtures".into()), (json!("None"), "None".into())] },
                    json!("Select"),
                )
                .hint("what Run action does for a tag whose own Action is the window's default"),
                SettingDef::new("counts", "Counts", SettingKind::Bool, json!(true)).hint("each tag's member count in the tree"),
            ],
        ))
        .bar_min(320, 200)
        .bar(true, &["action", "members", "counts"]),
        // The Device Sheet (decisions device-sheet-window, device-grid-and-value-marks; a
        // Fixture Sheet): every device with its folders, each value coloured by where it comes from.
        WindowType::new(
            11,
            "devices",
            "Device Sheet",
            "Sheets",
            "columns-3",
            "Every device and its values: where each comes from (your programmer, a cue, park), its fades and what leaves the station.",
        )
        .keywords(&["device", "devices", "fixture", "fixtures", "sheet", "values", "programmer", "channel", "output"])
        .settings({
            let choice = |words: &[&str]| SettingKind::Enum { options: words.iter().map(|w| (json!(w), (*w).into())).collect() };
            let mut s = section(
                "Display",
                vec![
                    SettingDef::new("mode", "Mode", choice(&["Fixture", "Channel"]), json!("Fixture"))
                        .hint("Fixture = a row per device, a column per attribute; Channel = tiles with the dimmer and the colour"),
                    SettingDef::new("layer", "Layer", choice(&["Absolute", "Fade", "Delay", "Output", "DMX"]), json!("Absolute"))
                        .hint("Absolute = the values; Fade / Delay = their times; Output = after masters; DMX = the byte"),
                    SettingDef::new("merge_cells", "Merge cells", SettingKind::Bool, json!(false))
                        .hint("equal neighbours in one feature group draw once"),
                ],
            );
            s.extend(section(
                "Mask",
                vec![
                    SettingDef::new("programmer_only", "Programmer only", SettingKind::Bool, json!(false))
                        .hint("only devices with values in your programmer"),
                    SettingDef::new("selected_only", "Selected only", SettingKind::Bool, json!(false)).hint("only your selected devices"),
                    SettingDef::new("selection_first", "Selection first", SettingKind::Bool, json!(true))
                        .hint("without folders: your selection on top, in the order you picked it"),
                    SettingDef::new("folders", "Folders", SettingKind::Bool, json!(true)).hint("the patch tree's folders as rows"),
                ],
            ));
            s
        })
        .bar_min(320, 160)
        .bar(true, &["mode", "layer", "programmer_only", "selected_only"]),
        // One window type per pool (decisions 0287): the Macros pool.
        WindowType::new(
            12,
            "macros",
            "Macros",
            "Pools",
            "wand-sparkles",
            "Pool of macros: each cell a macro of the DataPool and its line count.",
        )
        .keywords(&["macro", "macros", "pool", "lines", "script"])
        .settings(pool_settings(&["Go", "Edit", "None"]))
        .title("{Self.DataPool.Name}")
        .bar_min(260, 160)
        .bar(true, POOL_BAR),
        // The Presets window (decisions 0288): the one preset pool, following the programmer's
        // feature group, narrowed by tags.
        WindowType::new(
            14,
            "presets",
            "Presets",
            "Pools",
            "palette",
            "The preset pool: the presets that touch the feature group you are on (or the one pinned), narrowed by tags; each cell shows how it fits your selection and a preview of its values.",
        )
        .keywords(&["preset", "presets", "pool", "palette", "color", "position", "dimmer", "tag"])
        .settings({
            let mut s: Vec<SettingDef> = pool_settings(&["At", "SelFix At", "SelFix", "Call", "None"]);
            // A cell holds a preview and a row of feature-group squares: bigger than a group's.
            for d in s.iter_mut() {
                match d.key.as_str() {
                    "columns" => d.default = json!(8),
                    "rows" => d.default = json!(5),
                    "size" => d.default = json!(110),
                    _ => {}
                }
            }
            s.extend(section(
                "Presets",
                vec![
                    SettingDef::new(
                        "feature_group",
                        "Feature group",
                        SettingKind::Link { kind: "FeatureGroup".into(), follow: Some("Programmer's feature group".into()), by_guid: true },
                        json!(FOLLOW_SELECTED),
                    )
                    .hint("Follows the encoder bar's feature group; pin one, or All for every preset"),
                    SettingDef::new("tags", "Tags", SettingKind::Text { max_len: 2000 }, json!(""))
                        .hint("the tag chips lit on top (tag GUIDs); a preset must carry every one"),
                    SettingDef::new(
                        "layout",
                        "Layout",
                        SettingKind::Enum { options: vec![(json!("Packed"), "Packed".into()), (json!("Fixed"), "Fixed slots".into())] },
                        json!("Packed"),
                    )
                    .hint("Packed closes the cells up; Fixed keeps every preset's slot"),
                ],
            ));
            s
        })
        .title("{Self.DataPool.Name}")
        .bar_min(260, 160)
        .bar(true, &["action", "feature_group", "layout", "columns", "font_size"]),
        // The Views pool (decisions 0287): a profile's views, each cell drawing its windows.
        WindowType::new(
            13,
            "views",
            "Views",
            "Pools",
            "layout-dashboard",
            "Pool of a profile's views: each cell draws the view's windows, its screens and the view buttons it sits on.",
        )
        .keywords(&["view", "views", "pool", "layout", "screen", "viewbutton", "profile"])
        .settings({
            let mut s: Vec<SettingDef> = pool_settings(&["Call", "Store", "None"]).into_iter().filter(|d| d.key != "datapool").collect();
            // A cell draws the view's screens: bigger buttons than a group's.
            for d in s.iter_mut() {
                match d.key.as_str() {
                    "columns" => d.default = json!(6),
                    "rows" => d.default = json!(4),
                    "size" => d.default = json!(140),
                    _ => {}
                }
            }
            s.insert(1, SettingDef::new(
                "profile",
                "Profile",
                SettingKind::Link { kind: "UserProfile".into(), follow: Some("My profile".into()), by_guid: true },
                json!(FOLLOW_SELECTED),
            )
            .hint("My profile = the views of the profile you log in with"));
            s
        })
        .bar_min(260, 160)
        .bar(true, &["action", "profile", "columns", "font_size", "show_empty"]),
        WindowType {
            hidden: true,
            ..WindowType::new(
                3,
                "empty",
                "Empty",
                "Others",
                "plus",
                "Empty pane: pick a window for it.",
            )
        },
        // The System Monitor window (Info and System): the station's log.
        WindowType::new(
            4,
            "sysmon",
            "System Monitor",
            "Others",
            "scroll-text",
            "The station's log: logins, every user's commands, show changes, warnings and errors.",
        )
        .keywords(&[
            "sysmon", "system", "monitor", "log", "errors", "warnings", "info",
        ])
        .settings(section("Display", vec![
            font_size(),
            SettingDef::new(
                "level",
                "Level",
                SettingKind::Enum {
                    options: vec![
                        (json!("info"), "All".into()),
                        (json!("warn"), "Warnings + errors".into()),
                        (json!("error"), "Errors".into()),
                    ],
                },
                json!("info"),
            ),
            SettingDef::new(
                "subsystems",
                "Subsystems",
                SettingKind::Text { max_len: 200 },
                json!(""),
            )
            .hint("comma list, e.g. Session, Cmd — empty = all"),
            SettingDef::new(
                "search",
                "Search",
                SettingKind::Text { max_len: 200 },
                json!(""),
            )
            .hint("show only lines containing this text"),
            SettingDef::new("auto_scroll", "Auto scroll", SettingKind::Bool, json!(true))
                .hint("follow new lines; scrolling up pauses it (red background)"),
            SettingDef::new("wrap", "Wrap lines", SettingKind::Bool, json!(false))
                .hint("long lines wrap instead of running off the right"),
            SettingDef::new(
                "time",
                "Time",
                SettingKind::Enum {
                    options: vec![
                        (json!("session"), "Session".into()),
                        (json!("wall"), "Wall clock".into()),
                    ],
                },
                json!("session"),
            )
            .hint("Session = since the server started; Wall clock = local time"),
            SettingDef::new(
                "since",
                "Cleared at",
                SettingKind::Int { min: 0, max: MAX_SEQ, named: Vec::new() },
                json!(0),
            )
            .named(&[(0, "All")])
            .takes_now(NowSource::WallMs)
            .hint("Clear (Now) hides the lines up to now (their time_ms) in this window only; the station keeps its history"),
        ]))
        .actions(&[
            ("pause", "Pause", "pause", "Set Self \"AutoScroll\" Toggle", Some("!Self.AutoScroll")),
            ("clear", "Clear", "eraser", "Set Self \"Since\" Now", None),
        ])
        .chips(&[
            ("search", "× Search “{Self.Search}”", "Self.Search", "Set Self \"Search\" \"\""),
            ("subsystems", "× {Self.Subsystems}", "Self.Subsystems", "Set Self \"Subsystems\" \"\""),
        ])
        .title("{Self.Errors} errors")
        .bar_min(320, 120)
        .bar(true, &["pause", "wrap", "level", "time", "clear"]),
        // The DMX Sheet window (DMXSheetSettings): every channel of a universe,
        // marked by who won it (decisions §11 "Step 4c hardening").
        WindowType::new(
            5,
            "dmx",
            "DMX",
            "Sheets",
            "grid-3x3",
            "The DMX output, channel by channel: our output, park and merged DMX input (or the raw input).",
        )
        .keywords(&["dmx", "sheet", "universe", "channels", "output", "input", "merge"])
        .settings({
            let mut dmx = section("Display", vec![
            SettingDef::new(
                "universe",
                "Universe",
                SettingKind::Link { kind: "Universe".into(), follow: None, by_guid: false },
                json!(1),
            ),
            SettingDef::new(
                "address",
                "Address",
                SettingKind::Int { min: 1, max: 512, named: Vec::new() },
                json!(1),
            )
            .hint("scroll to this channel"),
            SettingDef::new(
                "view",
                "View",
                SettingKind::Enum {
                    options: vec![
                        (json!("output"), "Output (merged)".into()),
                        (json!("input"), "Input (raw)".into()),
                    ],
                },
                json!("output"),
            ),
            SettingDef::new(
                "readout",
                "Readout",
                SettingKind::Enum {
                    options: vec![
                        (json!("percent"), "Percent".into()),
                        (json!("decimal"), "Decimal".into()),
                        (json!("hex"), "Hex".into()),
                    ],
                },
                json!("percent"),
            ),
            SettingDef::new(
                "address_mode",
                "Address mode",
                SettingKind::Enum {
                    options: vec![
                        (json!("universe"), "Univ.addr".into()),
                        (json!("absolute"), "Absolute".into()),
                    ],
                },
                json!("universe"),
            ),
            SettingDef::new(
                "level_bar",
                "Level bar",
                SettingKind::Enum {
                    options: vec![
                        (json!("off"), "Off".into()),
                        (json!("background"), "Background".into()),
                        (json!("bar"), "Bar".into()),
                    ],
                },
                json!("background"),
            ),
            SettingDef::new(
                "columns",
                "Columns",
                SettingKind::Int { min: 0, max: 64, named: Vec::new() },
                json!(20),
            )
            .named(&[(0, "Fit")])
            .hint("Fit = fit the window"),
            ]);
            dmx.extend(section("Mask", vec![
            SettingDef::new("show_value", "Value", SettingKind::Bool, json!(true)),
            SettingDef::new("show_attribute", "Attribute", SettingKind::Bool, json!(true)),
            SettingDef::new("show_id", "ID", SettingKind::Bool, json!(false))
                .hint("fixture ID of the channel"),
            SettingDef::new(
                "only_selection",
                "Only selection",
                SettingKind::Bool,
                json!(false),
            )
            .hint("follow the universe of your first selected fixture"),
            ]));
            dmx
        })
        .actions(&[
            ("prev", "Universe −", "chevron-left", "Set Self \"Universe\" - 1", None),
            ("next", "Universe +", "chevron-right", "Set Self \"Universe\" + 1", None),
            ("first", "First universe", "rotate-ccw", "Set Self \"Universe\" 1", None),
            ("input", "Input", "arrow-down-to-line", "Set Self \"View\" Toggle", Some("Self.View=input")),
        ])
        .title("U {Self.Universe} {Self.Universe.Name}")
        .bar_min(280, 120)
        .bar(true, &["universe", "prev", "next", "readout"]),
        // System Info "Realtime" (decisions §24 "Step 9 hardening"):
        // the render engine stage by stage on a frame timeline, the lines'
        // send rates, the session clock and every thread.
        WindowType::new(
            6,
            "realtime",
            "Realtime",
            "Others",
            "activity",
            "The render engine frame by frame: each stage's time, headroom, late frames; the output lines' rates; the session clock (offset, path delay, jitter, frequency, PTP) and every thread.",
        )
        .keywords(&[
            "realtime", "health", "render", "frame", "timing", "clock", "ptp", "jitter", "performance",
        ])
        .settings(section(
            "Display",
            vec![
                SettingDef::new(
                    "graph",
                    "Graph",
                    SettingKind::Enum {
                        options: vec![
                            (json!(5), "5 s".into()),
                            (json!(10), "10 s".into()),
                            (json!(30), "30 s".into()),
                        ],
                    },
                    json!(10),
                )
                .hint("how much time each graph shows"),
                SettingDef::new("pause", "Pause", SettingKind::Bool, json!(false))
                    .hint("hold the graphs (frames keep arriving)"),
            ],
        ))
        .bar_min(300, 160)
        .bar(true, &["graph", "pause"]),
        // The UI kit's Showroom (decisions §30): every kit element in every
        // state — the reference for windows and plugins.
        WindowType::new(
            7,
            "showroom",
            "Showroom",
            "Tools",
            "showroom",
            "The UI kit: every element in every state, the theme's colours, sizes and icons.",
        )
        .keywords(&["showroom", "kit", "ui", "elements", "theme", "icons", "design"])
        .bar(false, &[]),
    ];
    types.into_iter().map(|t| (t.no, t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coerce_clamps_and_parses() {
        let types = builtin_window_types();
        let groups = &types[&2];
        assert_eq!(
            groups
                .setting("columns")
                .unwrap()
                .coerce(&json!("99"))
                .unwrap(),
            json!(64)
        );
        assert_eq!(
            groups
                .setting("show_empty")
                .unwrap()
                .coerce(&json!("off"))
                .unwrap(),
            json!(false)
        );
        assert_eq!(
            groups
                .setting("size")
                .unwrap()
                .coerce(&json!("large"))
                .unwrap(),
            json!(110)
        );
        assert!(groups.setting("size").unwrap().coerce(&json!(57)).is_err());
    }

    #[test]
    fn sysmon_settings() {
        let types = builtin_window_types();
        let sysmon = types
            .values()
            .find(|t| t.matches("System Monitor"))
            .unwrap();
        assert_eq!(
            sysmon
                .setting("level")
                .unwrap()
                .coerce(&json!("errors"))
                .unwrap(),
            json!("error")
        );
        assert_eq!(
            sysmon
                .setting("subsystems")
                .unwrap()
                .coerce(&json!("  Session, Cmd "))
                .unwrap(),
            json!("Session, Cmd")
        );
        assert_eq!(
            sysmon.resolve_settings(&serde_json::Map::new())["auto_scroll"],
            json!(true)
        );
    }

    fn by_name(name: &str) -> WindowType {
        builtin_window_types().into_values().find(|t| t.name == name).unwrap()
    }

    struct Ctx;
    impl SelfContext for Ctx {
        fn link(&self, kind: &str, value: &Value) -> Option<(u32, String)> {
            match (kind, value.as_u64()) {
                ("Universe", Some(n)) => Some((n as u32, format!("Uni{n}"))),
                ("DataPool", _) => Some((1, "Default".into())),
                _ => None,
            }
        }
        fn computed(&self, _: &WindowType, settings: &Map<String, Value>, name: &str) -> Option<Value> {
            same_key(name, "Errors").then(|| json!(if settings["since"] == json!(0) { 3 } else { 1 }))
        }
    }

    #[test]
    fn bar_items_are_checked_against_the_type() {
        let dmx = by_name("dmx");
        let items = dmx.setting("WindowBarItems").unwrap();
        assert_eq!(
            dmx.coerce_setting(items, &json!("universe, next ,prev,next"), true).unwrap(),
            json!(["universe", "next", "prev"]),
            "trimmed, repeats dropped"
        );
        assert!(dmx.coerce_setting(items, &json!("universe,bogus"), true).unwrap_err().contains("bogus"));
        let mut stored = Map::new();
        stored.insert(WINDOW_BAR_ITEMS.into(), json!(["bogus", "readout", "FIRST"]));
        assert_eq!(dmx.resolve_settings(&stored)[WINDOW_BAR_ITEMS], json!(["readout", "first"]), "unknown dropped on load");
        assert_eq!(dmx.resolve_settings(&Map::new())[WINDOW_BAR_ITEMS], json!(dmx.bar_default));
        assert!(!dmx.bar_ids().contains(&WINDOW_BAR_ITEMS));
        for ty in builtin_window_types().values().filter(|t| !t.hidden) {
            for id in &ty.bar_default {
                assert!(ty.bar_ids().contains(&id.as_str()), "{}: default item {id}", ty.name);
            }
            let mut ids = ty.bar_ids();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), ty.bar_ids().len(), "{}: one id space", ty.name);
        }
    }

    #[test]
    fn self_reads_title_state_and_chips() {
        let sysmon = by_name("sysmon");
        let mut stored = Map::new();
        let v = sysmon.view(&stored, &Ctx);
        assert_eq!(v.title, "3 errors");
        assert!(!v.states["pause"], "auto scroll on = not paused");
        assert!(v.chips.is_empty());
        stored.insert("auto_scroll".into(), json!(false));
        stored.insert("search".into(), json!("act 2"));
        stored.insert("since".into(), json!(40));
        let v = sysmon.view(&stored, &Ctx);
        assert_eq!(v.title, "1 errors");
        assert!(v.states["pause"]);
        assert_eq!(v.chips[0].label, "× Search “act 2”");
        assert_eq!(v.chips[0].line, "Set Self \"Search\" \"\"");
        assert_eq!(v.values["since"], "40");
        let dmx = by_name("dmx");
        let mut stored = Map::new();
        stored.insert("universe".into(), json!(3));
        let v = dmx.view(&stored, &Ctx);
        assert_eq!(v.title, "U 3 Uni3");
        assert!(!v.states["input"]);
        stored.insert("view".into(), json!("input"));
        assert!(dmx.view(&stored, &Ctx).states["input"]);
        assert_eq!(by_name("groups").view(&Map::new(), &Ctx).title, "Default");
        assert_eq!(render_template(&dmx, &Map::new(), "U {Self.Universe} {Self.Universe.Name}", &NoContext), "U 1");
    }

    #[test]
    fn resolve_fills_defaults_and_drops_unknown() {
        let types = builtin_window_types();
        let mut stored = serde_json::Map::new();
        stored.insert("columns".into(), json!(8));
        stored.insert("bogus".into(), json!(1));
        let resolved = types[&2].resolve_settings(&stored);
        assert_eq!(resolved["columns"], json!(8));
        assert_eq!(resolved["slots"], json!(0));
        assert!(!resolved.contains_key("bogus"));
    }
}
