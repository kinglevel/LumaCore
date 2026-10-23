//! Menus and UI keys.
//!
//! ```text
//! Menu ["name"] [/Tab x]     open a menu (the same line again closes it)
//! List Menu / Info Menu "x"  the menus of the registry (crate::menu_defs)
//! Set GridColumns "dmx.lines" "Columns"|"Widths"|"Sort"|"ColumnSet"|"NewLine"|"Offset" … · Reset GridColumns "dmx.lines"
//! List UIKey                 every UI key: core, plugin and this profile's
//! Info UIKey "Menu"          one key: owner, every gesture, Meow lines, LED
//! ```
//!
//! UI keys (virtual keys) are buttons with a command line per gesture —
//! model in `lumacore_tree::ui_keys`, bindings and checks in `terminal::keys`. Core keys are published at
//! `Root.Resources.UIKeys` and locked.

use crate::commands::flags as F;

use crate::AppState;
use crate::objects::{ObjectDef, Verb, filter, out};
use crate::errors::render_code;
use crate::text::render_success;
use crate::DispatchResult;
use crate::menu_defs::{self, Content, MenuDef, MenuKind};
use lumacore_wire::UiEvent;

/// `Menu "x" /Tab Networks` (`/Tab "This station"`, `/Tab=Show`; decisions §31).
const TAB: F::FlagDef = F::FlagDef::new("Tab", "The tab it opens on").rest("tab");
/// `/Open "<row> [<group> [<part>]]"`: drill into a row's detail (decisions §30 "Users &
/// Profiles follows storage").
const OPEN: F::FlagDef = F::FlagDef::new("Open", "Drill into a row's detail").value("row group part");

fn menu_names() -> String {
    lumacore_registry::get().menus().map(|m| m.name).collect::<Vec<_>>().join(", ")
}

fn tab_names(m: &MenuDef) -> String {
    m.tabs.iter().map(|t| t.name).collect::<Vec<_>>().join(", ")
}

/// `Menu ["name"] [/Tab x]` (`Menu MenuSelector`, decisions §30 "12d"):
/// open a menu on this UI client. Without `/Tab` it opens at the profile's
/// last tab and the same line again closes it; `/Tab x` opens (or switches
/// to) that tab and keeps it as the profile's last (never an Oops step; the
/// locked Default profile keeps nothing).
pub fn menu_call(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let name = args
        .first()
        .filter(|a| !F::is_flag(a))
        .map_or("Menu".to_string(), |a| a.trim_matches('"').to_string());
    let Some(menu) = menu_defs::find(&name) else {
        return out(render_code(format!("No menu \"{name}\". Menus: {}", menu_names()), "E-CMD-007"));
    };
    let asked = F::has(args, &TAB);
    let given = match F::rest_value(args, &TAB) {
        Some(t) => match menu.tab(&t) {
            Some(tab) => Some(tab.name),
            None => {
                return out(render_code(
                    format!("{} has no tab \"{t}\". Tabs: {}", menu.name, if menu.tabs.is_empty() { "none".into() } else { tab_names(menu) }),
                    "E-CMD-007",
                ));
            }
        },
        None if asked => return out(render_code(format!("Usage: Menu \"{}\" /Tab <tab> ({})", menu.name, tab_names(menu)), "E-CMD-006")),
        None => None,
    };
    if !state.has_ui() {
        return out(render_code("Menu opens a menu in the UI (web / app window).", "E-CMD-010"));
    }
    // The last tab rides on the profile (Default is locked: it keeps none).
    if let Some(tab) = given
        && let Some(profile) = state.current_user_profile_mut()
        && !profile.locked
        && profile.settings.ui.menu_tabs.get(menu.name).map(String::as_str) != Some(tab)
    {
        profile.settings.ui.menu_tabs.insert(menu.name.to_string(), tab.to_string());
        state.show_dirty = true;
    }
    let profile = state.root.show_data.profile_for_user(state.effective_user_name());
    let tab = given.map(str::to_string).or_else(|| {
        profile
            .and_then(|p| p.settings.ui.menu_tabs.get(menu.name))
            .filter(|t| menu.tab(t).is_some())
            .cloned()
    });
    let grids: serde_json::Map<String, serde_json::Value> = profile
        .map(|p| {
            menu.grid_keys()
                .filter_map(|k| p.settings.ui.grids.get(k).map(|g| (k.to_string(), serde_json::to_value(g).unwrap_or_default())))
                .collect()
        })
        .unwrap_or_default();
    let open = F::value(args, &OPEN).unwrap_or_default();
    state.ui_requests.push(UiEvent::OpenMenu {
        menu: menu.name.to_string(),
        tab,
        toggle: given.is_none() && open.is_empty(),
        open,
        grids: if grids.is_empty() { String::new() } else { serde_json::Value::Object(grids).to_string() },
    });
    out(String::new())
}

fn content_word(c: &Content) -> String {
    match c {
        Content::Grid(g) => format!("grid {}{}", g.key, g.object_decl().map_or(String::new(), |o| format!(" ({})", o.keyword))),
        Content::Drives(g) => format!("grid {} on the drives", g.key),
        Content::Settings(t) => format!("settings of {t}"),
        Content::Custom(_) => "custom".into(),
        Content::Empty(_) => "to come".into(),
    }
}

fn menu_list(_: &mut AppState, _: &[&str]) -> DispatchResult {
    let mut text = String::from("Menus (Menu \"name\" [/Tab x]):\n");
    for m in lumacore_registry::get().menus() {
        text.push_str(&format!(
            "  {:<14} {:<18} {:<6} {}\n",
            m.name,
            m.label,
            if m.kind == MenuKind::Setup { "setup" } else { "page" },
            if m.tabs.is_empty() { "—".to_string() } else { tab_names(m) }
        ));
    }
    out(text.trim_end().to_string())
}

fn menu_info(_: &mut AppState, args: &[&str]) -> DispatchResult {
    let Some(name) = args.first() else {
        return out(render_code("Usage: Info Menu \"name\"", "E-CMD-006"));
    };
    let Some(m) = menu_defs::find(name) else {
        return out(render_code(format!("No menu {name}. Menus: {}", menu_names()), "E-CMD-007"));
    };
    let mut text = format!("Menu \"{}\"  {} — {}", m.name, m.label, m.doc);
    if !m.aliases.is_empty() {
        text.push_str(&format!("\n  Also:   {}", m.aliases.join(", ")));
    }
    if !m.home.is_empty() {
        text.push_str(&format!("\n  Lives:  {}", m.home));
    }
    if let Some(p) = &m.power {
        text.push_str(&format!("\n  Power:  {} — on: {} ({}), off: {} ({}); needs {}", p.label, p.on.word, p.on.line, p.off.word, p.off.line, p.rights.label()));
    }
    for t in m.tabs {
        text.push_str(&format!("\n  Tab {:<12} {}", t.name, content_word(&t.content)));
        for a in t.actions {
            text.push_str(&format!("\n      {:<14} {}", a.label, a.line));
        }
    }
    out(render_success(&text))
}

fn complete_menus(_: &AppState, prefix: &str) -> Vec<String> {
    filter(lumacore_registry::get().menus().map(|m| m.name.to_string()), prefix)
}

/// Menus (decisions §30 "12d"): `Menu "x"` opens, `List Menu`, `Info Menu "x"`.
pub static MENU: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Menu",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Toggle,
        flags: &[TAB, OPEN],
        aliases: &["menus"],
        doc: crate::commands::registry::Doc {
            display: "Menu",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Menu [\"name\"] [/Tab <tab>] [/Open \"<row> [<group> [<part>]]\"] | List Menu | Info Menu \"name\"",
            description: "Open a menu in this UI: the ⚙ menu (bare), Settings, Network, \
                          DMXProtocols, LumaSessions, Backup, Users or Shutdown. The same line again \
                          closes it (never a second copy); /Tab opens a tab, which your profile keeps \
                          as the menu's last. The ⚙ button runs `Menu`; double-clicking it is a Quick \
                          Save (UI key `menu`, see List UIKey).",
            examples: &[
                "menu                       -- the ⚙ menu",
                "menu \"Settings\"            -- Settings, at your last tab",
                "menu \"Network\" /tab networks",
                "list menu                  -- every menu and its tabs",
                "info menu \"DMXProtocols\"",
            ],
            related: &["saveshow", "list uikey", "gridcolumns"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: crate::commands::registry::Undo::Untracked,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_client,
        reach: crate::objects::show_reach,
        rights: crate::rights::anyone,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::Toggle, menu_call), (Verb::List, menu_list), (Verb::Info, menu_info)],
    delete_ask: None,
    complete: Some(complete_menus),
    props: None,
    tree: None,
};

// ── GridColumns: a grid's columns on your profile ───────────────────────────

/// The grid `word` names (its key, any case).
fn grid_of(word: &str) -> Option<(&'static str, &'static menu_defs::Grid)> {
    let word = word.trim_matches('"');
    lumacore_registry::get().menus().flat_map(|m| m.tabs).find_map(|t| match t.content {
        Content::Grid(g) if g.key.eq_ignore_ascii_case(word) => Some((g.key, g)),
        _ => None,
    })
}

fn column_ids(g: &menu_defs::Grid, raw: &str) -> Result<Vec<String>, String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|id| {
            g.columns
                .iter()
                .find(|c| c.id.eq_ignore_ascii_case(id))
                .map(|c| c.id.to_string())
                .ok_or_else(|| format!("no column '{id}' (columns: {})", g.columns.iter().map(|c| c.id).collect::<Vec<_>>().join(", ")))
        })
        .collect()
}

// tidy: not-keywords (the column sets' words, as Set GridColumns takes them)
const COLUMN_SETS: &[&str] = &["Condensed", "Full", "Extended"];
/// A grid's column state on your profile, one property of it (`Set GridColumns "<grid>"
/// "Columns" "no,name,rate"`; empty = the factory state, which `Reset` writes — §33 61).
fn grid_write(state: &mut AppState, key: &str, prop: &str, raw: &str) -> Result<String, String> {
    let Some((key, g)) = grid_of(key) else {
        return Err(render_code(format!("No grid {key} (List GridColumns)."), "E-CMD-007"));
    };
    let raw = raw.trim().trim_matches('"').to_string();
    let factory = raw.is_empty();
    let mut state_now = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| p.settings.ui.grids.get(key).cloned().unwrap_or_default())
        .unwrap_or_default();
    let bad = |e: String| Err(render_code(format!("GridColumns {key} \"{prop}\": {e}"), "E-SET-003"));
    let p = prop.to_ascii_lowercase();
    // tidy: not-keywords (GridColumns' own properties)
    let shown = match p.as_str() {
        "columns" => {
            state_now.columns = if factory {
                Vec::new()
            } else {
                match column_ids(g, &raw) {
                    Ok(c) => c,
                    Err(e) => return bad(e),
                }
            };
            format!("columns {}", if factory { "Default".into() } else { state_now.columns.join(",") })
        }
        "widths" => {
            state_now.widths.clear();
            if !factory {
                for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    let Some((id, px)) = part.split_once('=') else {
                        return bad(format!("'{part}' is not column=px"));
                    };
                    let id = match column_ids(g, id) {
                        Ok(mut c) if c.len() == 1 => c.remove(0),
                        Ok(_) => return bad(format!("'{part}' is not column=px")),
                        Err(e) => return bad(e),
                    };
                    match px.trim().trim_end_matches("px").parse::<u32>() {
                        Ok(px) if (24..=2000).contains(&px) => {
                            state_now.widths.insert(id, px);
                        }
                        _ => return bad(format!("'{px}' is not a width 24–2000")),
                    }
                }
            }
            format!("widths {}", if factory { "Default".into() } else { raw.clone() })
        }
        "sort" => {
            if factory || raw.eq_ignore_ascii_case("none") {
                state_now.sort = None;
                "sort none".to_string()
            } else {
                let mut words = raw.split_whitespace();
                let col = match column_ids(g, words.next().unwrap_or("")) {
                    Ok(mut c) if c.len() == 1 => c.remove(0),
                    Ok(_) => return bad("name a column".into()),
                    Err(e) => return bad(e),
                };
                let dir = match words.next().map(str::to_ascii_lowercase).as_deref() {
                    None | Some("asc") => "asc",
                    Some("desc") => "desc",
                    Some(other) => return bad(format!("'{other}' is not Asc or Desc")),
                };
                state_now.sort = Some(lumacore_tree::GridSort { col: col.clone(), dir: dir.into() });
                format!("sort {col} {dir}")
            }
        }
        "columnset" => {
            if factory {
                state_now.set = None;
            } else {
                match COLUMN_SETS.iter().find(|s| s.eq_ignore_ascii_case(&raw)) {
                    // Full is the factory set.
                    Some(s) if s.eq_ignore_ascii_case("Full") => state_now.set = None,
                    Some(s) => state_now.set = Some((*s).to_string()),
                    None => return bad(format!("'{raw}' is not one of {}", COLUMN_SETS.join(", "))),
                }
            }
            format!("column set {}", state_now.set.as_deref().unwrap_or("Full"))
        }
        "offset" => {
            // Checked by the property's kind (a number 0–512); 0 = packed.
            state_now.offset = if factory { None } else { raw.parse::<u32>().ok().filter(|n| *n > 0) };
            format!("offset {}", state_now.offset.unwrap_or(0))
        }
        "newline" => {
            // Checked by the property's kind (On / Off); empty = the factory On.
            state_now.new_line = (raw == "Off").then_some(false);
            format!("new-object line {}", if state_now.new_line == Some(false) { "Off" } else { "On" })
        }
        _ => return Err(render_code(format!("GridColumns has no property \"{prop}\"."), "E-SET-001")),
    };
    let Some(profile) = state.current_user_profile_mut() else {
        return Err(render_code("No user profile", "E-CMD-011"));
    };
    if profile.locked {
        return Err(render_code(crate::commands::profiles::DEFAULT_LOCKED, "E-USER-002"));
    }
    if state_now.is_factory() {
        profile.settings.ui.grids.remove(key);
    } else {
        profile.settings.ui.grids.insert(key.to_string(), state_now);
    }
    state.show_dirty = true;
    Ok(format!("Grid {key}: {shown} (your profile)"))
}

/// A grid's state now on your profile.
fn grid_now(state: &AppState, key: &str) -> lumacore_tree::GridState {
    state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .and_then(|p| p.settings.ui.grids.get(key).cloned())
        .unwrap_or_default()
}

macro_rules! grid_prop {
    ($name:literal, $kind:expr, $default:literal, $doc:literal, $get:expr) => {
        crate::props::Property {
            name: $name,
            kind: $kind,
            rights: crate::commands::registry::Rights::Guest,
            section: "Grid",
            doc: $doc,
            default: Some($default),
            access: crate::props::Access::Keyed { get: $get, set: |s, k, v| grid_write(s, k, $name, v) },
            ..crate::props::Property::BASE
        }
    };
}

static GRID_PROPS: [crate::props::Property; 6] = [
    grid_prop!("Columns", crate::props::PropKind::Text, "", "the columns shown, in order (ids, comma-separated)", |s, k| grid_now(s, k).columns.join(",")),
    grid_prop!("Widths", crate::props::PropKind::Text, "", "column widths, column=px (24–2000)", |s, k| grid_now(s, k).widths.iter().map(|(c, w)| format!("{c}={w}")).collect::<Vec<_>>().join(",")),
    grid_prop!("Sort", crate::props::PropKind::Text, "", "the sort column and Asc / Desc (\"rate Desc\"), none = unsorted", |s, k| grid_now(s, k).sort.map(|x| format!("{} {}", x.col, x.dir)).unwrap_or_default()),
    grid_prop!("ColumnSet", crate::props::PropKind::Choice(COLUMN_SETS), "Full", "Condensed / Full / Extended", |s, k| grid_now(s, k).set.unwrap_or_else(|| "Full".into())),
    grid_prop!("NewLine", crate::props::PropKind::OnOff, "On", "the grid's new-object line", |s, k| if grid_now(s, k).new_line == Some(false) { "Off" } else { "On" }.to_string()),
    grid_prop!("Offset", crate::props::PropKind::Int { min: 0, max: 512 }, "0", "spare DMX slots after each fixture when its DMX actions pack a selection (the patch), 0 = packed", |s, k| grid_now(s, k).offset.unwrap_or(0).to_string()),
];

/// A grid's columns on your profile (decisions §30 "12d"): never an Oops step.
static GRID_SETTINGS: crate::props::Props = crate::props::Props {
    lists: &[&GRID_PROPS],
    keyed: Some(crate::props::Keyed {
        rows: |_| lumacore_registry::get().menus().flat_map(|m| m.tabs).filter_map(|t| match t.content {
            Content::Grid(g) => Some(g.key.to_string()),
            _ => None,
        }).collect(),
        row: |_, w| grid_of(w).map(|(k, _)| k.to_string()),
    }),
    label: Some(|_, k| format!("GridColumns \"{k}\"")),
    ..crate::props::Props::BASE
};

fn grid_state_text(g: &lumacore_tree::GridState) -> String {
    let mut parts = Vec::new();
    if !g.columns.is_empty() {
        parts.push(format!("columns {}", g.columns.join(",")));
    }
    if !g.widths.is_empty() {
        parts.push(format!("widths {}", g.widths.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(",")));
    }
    if let Some(s) = &g.sort {
        parts.push(format!("sort {} {}", s.col, s.dir));
    }
    if let Some(s) = &g.set {
        parts.push(format!("set {s}"));
    }
    if g.new_line == Some(false) {
        parts.push("new line off".into());
    }
    if parts.is_empty() { "factory".into() } else { parts.join(" · ") }
}

fn grid_list(state: &mut AppState, _: &[&str]) -> DispatchResult {
    let profile = state.root.show_data.profile_for_user(state.effective_user_name());
    let mut text = String::from("Grids (Set GridColumns \"<grid>\" \"Columns\"|\"Widths\"|\"Sort\"|\"ColumnSet\"|\"NewLine\"|\"Offset\" · Reset GridColumns \"<grid>\"):\n");
    for key in menu_defs::grid_keys() {
        let mine = profile.and_then(|p| p.settings.ui.grids.get(key)).map_or("factory".into(), grid_state_text);
        text.push_str(&format!("  {key:<24} {mine}\n"));
    }
    out(text.trim_end().to_string())
}

fn grid_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let Some((key, g)) = args.first().and_then(|a| grid_of(a)) else {
        return out(render_code("Usage: Info GridColumns \"<grid>\" (List GridColumns)", "E-CMD-006"));
    };
    let mine = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .and_then(|p| p.settings.ui.grids.get(key))
        .map_or("factory".into(), grid_state_text);
    let mut text = format!("Grid {key}{}  — yours: {mine}", g.object_decl().map_or(String::new(), |o| format!(" ({})", o.keyword)));
    for c in g.columns {
        text.push_str(&format!(
            "\n  {:<14} {:<22} {}{}",
            c.id,
            c.label,
            c.prop.map_or("read-only".to_string(), |p| format!("Set {p}")),
            if c.frozen { " · frozen" } else { "" }
        ));
    }
    out(render_success(&text))
}

fn complete_grids(_: &AppState, prefix: &str) -> Vec<String> {
    filter(menu_defs::grid_keys().map(|k| format!("\"{k}\"")), prefix)
}

/// A grid's columns on your profile (decisions §30 "12d"): order, widths,
/// sort, column set and the new-object line, per grid; never an Oops step.
pub static GRID_COLUMNS: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "GridColumns",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::List,
        flags: &[],
        aliases: &[],
        doc: crate::commands::registry::Doc {
            display: "GridColumns",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Set GridColumns \"<grid>\" \"Columns\" \"a,b\" | \"Widths\" \"a=120\" | \"Sort\" \"a Desc\" | \"ColumnSet\" Condensed|Full|Extended | \"NewLine\" On|Off · Reset GridColumns \"<grid>\" [\"<Property>\"]",
            description: "Your profile's columns of a grid (the setup menus' grids, decisions §30 \"12d\"): \
                          which columns show and in what order, their widths, the sort, the column set and \
                          the new-object line. The grid's header menu sends these lines; they are never an \
                          Oops step. List GridColumns names every grid.",
            examples: &[
                "set gridcolumns \"dmx.lines\" columns \"no,name,rate,send\"",
                "set gridcolumns \"dmx.lines\" sort \"rate desc\"",
                "set gridcolumns \"dmx.lines\" default",
                "list gridcolumns",
            ],
            related: &["menu"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::anyone,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::List, grid_list), (Verb::Info, grid_info)],
    delete_ask: None,
    complete: Some(complete_grids),
    props: Some(&GRID_SETTINGS),
    tree: None,
};
