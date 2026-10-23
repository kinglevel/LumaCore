// ██╗   ██╗██╗    ██╗  ██╗███████╗██╗   ██╗
// ██║   ██║██║    ██║ ██╔╝██╔════╝╚██╗ ██╔╝
// ██║   ██║██║    █████╔╝ █████╗   ╚████╔╝
// ██║   ██║██║    ██╔═██╗ ██╔══╝    ╚██╔╝
// ╚██████╔╝██║    ██║  ██╗███████╗   ██║
//  ╚═════╝ ╚═╝    ╚═╝  ╚═╝╚══════╝   ╚═╝
// FILE: ui key

//! UI keys: a button with a command line per gesture (Press, Release, Hold, ReleaseAfterHold,
//! DoublePress) and Meow lines — core, plugin or the profile's. A press may be a cycle, and some
//! presses type onto the command line (decisions 0295); a profile's keys are set line by line.
//!
//! Owns:          the UIKey object: List / Info, its properties (the lines), its completion, `Root.Resources.UIKeys`.
//! Doesn't own:   the keys' data (lumacore-tree ui_keys), the checks (lines.rs).
//! Entry points:  UI_KEY.
//! Decisions:     §17, §30 "12c", 0295.

use lumacore_tree::ui_keys::{Gesture, PressRow};
use lumacore_tree::{Root, UiKey, UiKeyOwner};

use lumacore_objects::commands::registry;
use lumacore_objects::errors::render_code;
use lumacore_objects::props::{Access, Home, Keyed, PropKind, Property, Props};
use lumacore_objects::objects::{ObjectDef, TreeDef, Verb, filter, json_view, out};
use lumacore_objects::text::render_success;
use lumacore_objects::vfs::{DynChild, NodeView};
use lumacore_objects::{AppState, DispatchResult};

use crate::lines::key_problems;
use crate::lookup::all_keys;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn owner_label(o: UiKeyOwner) -> &'static str {
    match o {
        UiKeyOwner::Core => "core",
        UiKeyOwner::Plugin => "plugin",
        UiKeyOwner::User => "profile",
    }
}

fn dash(s: &str) -> &str {
    if s.is_empty() { "—" } else { s }
}

/// `gesture: line` for each gesture the key has, Meow lines last.
fn gestures(k: &UiKey) -> String {
    let parts: Vec<String> = k
        .lines()
        .map(|(g, meow, line)| format!("{}{}: {line}", if meow { "Meow+" } else { "" }, g.label()))
        .collect();
    if parts.is_empty() {
        "—".into()
    } else {
        parts.join("  ")
    }
}

fn list_keys(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    let keys = all_keys(state);
    let mut text = String::from("UI keys (gesture: command line; ⚠ = needs attention):\n");
    for k in &keys {
        let problems = key_problems(k);
        text.push_str(&format!(
            "  {:<24} {:<8} {}{}  {}{}\n",
            k.id,
            owner_label(k.owner),
            if k.icon.is_empty() {
                String::new()
            } else {
                format!("{} ", k.icon)
            },
            k.meta.name().unwrap_or_default(),
            gestures(k),
            if problems.is_empty() { "" } else { "  ⚠" },
        ));
    }
    out(text.trim_end().to_string())
}

fn info_key(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let Some(id) = args.first().map(|a| a.trim_matches('"')) else {
        return out(render_code("Usage: Info UIKey \"id\"", "E-CMD-006"));
    };
    let keys = all_keys(state);
    let Some(k) = keys
        .iter()
        .find(|k| k.id.eq_ignore_ascii_case(id) || k.meta.name().unwrap_or_default().eq_ignore_ascii_case(id))
    else {
        return out(render_code(format!("No UI key \"{id}\""), "E-KEY-004"));
    };
    let mut text = format!(
        "UIKey {}  \"{}\" ({}{})",
        k.id,
        k.meta.name().unwrap_or_default(),
        owner_label(k.owner),
        if k.locked { ", locked" } else { "" },
    );
    for g in Gesture::ALL {
        let plain = k.command(g);
        let meow = k.meow.get(g);
        text.push_str(&format!(
            "\n  {:<17} {}",
            format!("{}:", g.label()),
            dash(plain)
        ));
        if g == Gesture::Double && k.double_replaces_press {
            text.push_str("  (replaces the press)");
        }
        if !meow.is_empty() {
            text.push_str(&format!("   Meow: {meow}"));
        }
    }
    if !k.state.is_empty() {
        text.push_str(&format!("\n  LED:              {}", k.state));
    }
    if k.lines()
        .any(|(g, _, _)| !matches!(g, Gesture::Press | Gesture::Double))
    {
        text.push_str(
            "\n  Release / Hold gestures need the app or web UI (a terminal only sees presses).",
        );
    }
    for p in key_problems(k) {
        text.push_str(&format!("\n  ⚠ {p}"));
    }
    if !k.tooltip.is_empty() {
        text.push_str(&format!("\n  {}", k.tooltip));
    }
    out(render_success(&text))
}

fn complete_keys(state: &AppState, prefix: &str) -> Vec<String> {
    filter(all_keys(state).into_iter().map(|k| k.id), prefix)
}

fn core_children(root: &Root) -> Vec<DynChild> {
    root.resources
        .ui_keys
        .iter()
        .map(|(&id, k)| DynChild {
            display_name: format!(
                "{id} \"{}\"  (press: {}, double: {})",
                k.meta.name().unwrap_or_default(),
                dash(&k.press),
                dash(&k.double_press)
            ),
            id,
            node_type: "UIKey",
            is_leaf: true,
            child_count: 0,
            name: k.meta.name().map(str::to_string),
            guid: Some(k.meta.guid()),
        })
        .collect()
}

fn core_view(root: &Root, id: u32) -> Option<NodeView> {
    root.resources
        .ui_keys
        .get(&id)
        .map(|k| json_view("UIKey", id, k))
}

// *************************************************************************************************
// A key's lines as properties (decisions 0295)
// *************************************************************************************************
//
// `Set UIKey "user.prg.e" "Press" `Edit` `EditSetting``: a profile's own key. Press takes one or
// more lines (a press cycle), Types the presses that type onto the command line, the other
// gestures one line each; Meow the same. A LumaCore or plugin key is locked: its cells show its
// lines and say so.

const KEY_SECTION: &str = "Lines";

macro_rules! key_prop {
    ($name:expr, $doc:expr) => {
        Property {
            name: $name,
            kind: PropKind::Text,
            rights: lumacore_objects::commands::registry::Rights::Program,
            home: Home::User,
            section: KEY_SECTION,
            doc: $doc,
            access: Access::Named { get: key_get, set: key_set },
            ..Property::BASE
        }
    };
}

static KEY_PROPS: [Property; 12] = [
    key_prop!("Press", "what a press runs or types: one line, or several in backticks — a press cycle, each press the next"),
    key_prop!("Types", "the presses that type their words onto the command line (1 = the first); the others run"),
    key_prop!("Release", "what the release runs (the app and web; a terminal sees presses only)"),
    key_prop!("Hold", "what a hold past your HoldTime runs (the app and web)"),
    key_prop!("ReleaseAfterHold", "what the release after a hold runs (the app and web)"),
    key_prop!("DoublePress", "what a second press within your DoublePress time runs; none = a second press"),
    key_prop!("MeowPress", "the press with Meow held: one line or a press cycle; none = the plain press"),
    key_prop!("MeowTypes", "the Meow presses that type onto the command line"),
    key_prop!("MeowRelease", "the release with Meow held"),
    key_prop!("MeowHold", "the hold with Meow held"),
    key_prop!("MeowReleaseAfterHold", "the release after a hold with Meow held"),
    key_prop!("MeowDoublePress", "the double press with Meow held"),
];

static UI_KEY_PROPS: Props = Props {
    lists: &[&KEY_PROPS],
    keyed: Some(Keyed {
        rows: |s| all_keys(s).into_iter().map(|k| k.id).collect(),
        row: |s, w| {
            let w = w.trim_matches('"');
            all_keys(s).into_iter().find(|k| k.id.eq_ignore_ascii_case(w)).map(|k| k.id)
        },
    }),
    keyed_beside_tree: true,
    label: Some(|_, k| format!("UIKey \"{k}\"")),
    home: Some(Home::User),
    ..Props::BASE
};

/// A press cycle as a cell shows it and `Set` takes it back: one line bare, several each in
/// backticks.
fn rows_text(rows: &[PressRow]) -> String {
    match rows {
        [one] => one.line.clone(),
        many => many.iter().map(|r| format!("`{}`", r.line)).collect::<Vec<_>>().join(" "),
    }
}

/// Which presses type, as written (`1 2`), or `None`.
fn types_text(rows: &[PressRow]) -> String {
    let typed: Vec<String> = rows.iter().enumerate().filter(|(_, r)| r.types).map(|(i, _)| (i + 1).to_string()).collect();
    if typed.is_empty() { "None".into() } else { typed.join(" ") }
}

fn key_get(state: &AppState, id: &str, prop: &str) -> String {
    let Some(k) = crate::lookup::find(state, id) else { return String::new() };
    let meow = prop.starts_with("Meow");
    match prop.trim_start_matches("Meow") {
        "Press" if meow && k.meow.press.is_empty() => String::new(),
        "Press" => rows_text(&k.press_rows(meow)),
        "Types" if meow && k.meow.press.is_empty() => "None".into(),
        "Types" => types_text(&k.press_rows(meow)),
        g => Gesture::parse(g).map(|g| if meow { k.meow.get(g) } else { k.command(g) }.to_string()).unwrap_or_default(),
    }
}

/// The lines a Press value holds: each in backticks, or one bare line; empty = none.
fn lines_of(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.contains('`') {
        value.split('`').skip(1).step_by(2).map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
    } else if value.is_empty() || value.eq_ignore_ascii_case("none") {
        Vec::new()
    } else {
        vec![value.to_string()]
    }
}

/// `1`, `1 2`, `1 Thru 3`, `None`: the press numbers, each within `count`.
fn presses_of(value: &str, count: usize) -> Result<Vec<u8>, String> {
    let words: Vec<&str> = value.split(|c: char| c.is_whitespace() || c == ',' || c == '+').filter(|w| !w.is_empty()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < words.len() {
        if words[i].eq_ignore_ascii_case("none") {
            i += 1;
            continue;
        }
        let n = |w: &str| w.parse::<u8>().map_err(|_| format!("'{w}' is no press number"));
        let from = n(words[i])?;
        let to = if words.get(i + 1).is_some_and(|w| w.eq_ignore_ascii_case("thru")) {
            i += 2;
            words.get(i).map_or(Ok(count as u8), |w| n(w))?
        } else {
            from
        };
        out.extend(from..=to);
        i += 1;
    }
    if let Some(bad) = out.iter().find(|&&p| p == 0 || p as usize > count) {
        return Err(format!("press {bad} isn't one of the key's {count}"));
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn key_set(state: &mut AppState, id: &str, prop: &str, value: &str) -> Result<String, String> {
    let meow = prop.starts_with("Meow");
    let gesture = prop.trim_start_matches("Meow");
    let Some(mut key) = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .and_then(|p| p.content.ui_keys.values().find(|k| k.id.eq_ignore_ascii_case(id)))
        .cloned()
    else {
        return Err(match crate::lookup::find(state, id) {
            Some(k) => render_code(
                format!("UIKey {} is {}'s and locked — bind your own in its place: Bind <key> `…` /Shortcuts …", k.id, owner_label(k.owner)),
                "E-KEY-007",
            ),
            None => render_code(format!("No UI key \"{id}\" on your profile."), "E-KEY-004"),
        });
    };
    let bad = |e: String| render_code(format!("UIKey {}: {e}.", key.id), "E-KEY-007");
    let shown = match gesture {
        "Press" => {
            let rows: Vec<String> = lines_of(value).iter().map(|l| registry::full_words(l)).collect();
            let types = if meow { &key.meow_types } else { &key.types };
            for (i, row) in rows.iter().enumerate() {
                let typed = types.contains(&(i as u8 + 1));
                let problems = if typed { crate::lines::check_fragment(row) } else { crate::lines::check_line(row) };
                if let Some(Err(e)) = problems.into_iter().find(Result::is_err) {
                    return Err(render_code(format!("UIKey {} press {}: {e}.", key.id, i + 1), "E-KEY-001"));
                }
            }
            let (first, more) = rows.split_first().map_or((String::new(), Vec::new()), |(f, m)| (f.clone(), m.to_vec()));
            if meow {
                key.meow.press = first;
                key.meow_cycle = more;
                key.meow_types.retain(|&p| (p as usize) <= rows.len());
            } else {
                key.press = first;
                key.cycle = more;
                key.types.retain(|&p| (p as usize) <= rows.len());
            }
            rows.iter().map(|r| format!("`{r}`")).collect::<Vec<_>>().join(" › ")
        }
        "Types" => {
            let count = key.press_rows(meow).len();
            let presses = presses_of(value, count).map_err(bad)?;
            // A row that types is a fragment: each of its words one of the language's.
            for (i, row) in key.press_rows(meow).iter().enumerate() {
                if presses.contains(&(i as u8 + 1))
                    && let Some(Err(e)) = crate::lines::check_fragment(&row.line).into_iter().next()
                {
                    return Err(render_code(format!("UIKey {} press {}: {e}.", key.id, i + 1), "E-KEY-001"));
                }
            }
            let text = if presses.is_empty() { "None".to_string() } else { presses.iter().map(u8::to_string).collect::<Vec<_>>().join(" ") };
            if meow { key.meow_types = presses } else { key.types = presses }
            text
        }
        g => {
            let g = Gesture::parse(g).ok_or_else(|| bad(format!("no gesture {g}")))?;
            let line = lines_of(value).into_iter().next().map(|l| registry::full_words(&l)).unwrap_or_default();
            if let Some(Err(e)) = crate::lines::check_line(&line).into_iter().find(Result::is_err) {
                return Err(render_code(format!("UIKey {} {}: {e}.", key.id, g.label()), "E-KEY-001"));
            }
            *if meow { key.meow.get_mut(g) } else { key.command_mut(g) } = line.clone();
            if line.is_empty() { "none".into() } else { format!("`{line}`") }
        }
    };
    let id = key.id.clone();
    let Some(slot) = state
        .current_user_profile_mut()
        .and_then(|p| p.content.ui_keys.values_mut().find(|k| k.id.eq_ignore_ascii_case(&id)))
    else {
        return Err(render_code("No user profile", "E-CMD-011"));
    };
    *slot = key;
    state.show_dirty = true;
    Ok(format!("UIKey {id} {prop} → {shown}"))
}

/// UI keys: List / Info; core keys browsable at `Root.Resources.UIKeys`.
pub static UI_KEY: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "UIKey",
        lum: lumacore_objects::objects::LumShape::ids(1),
        default_verb: lumacore_objects::objects::Verb::Info,
        flags: &[],
        aliases: &["uikeys"],
        doc: lumacore_objects::commands::registry::Doc::new(
            "UIKey",
            "A UI button with a command line per gesture (Press, Release, Hold, ReleaseAfterHold, DoublePress) and Meow lines (core, plugin, profile)",
        ),
        family: None,
        call_undo: lumacore_objects::commands::registry::Undo::ReadOnly,
        call_class: lumacore_objects::commands::registry::CommandClass::Commit,
        scope: lumacore_objects::objects::on_master,
        reach: lumacore_objects::objects::show_reach,
        rights: lumacore_objects::rights::content,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::List, list_keys), (Verb::Info, info_key)],
    delete_ask: None,
    complete: Some(complete_keys),
    props: Some(&UI_KEY_PROPS),
    tree: Some(TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: lumacore_objects::tree_path::Node::UIKeys,
        children: core_children,
        object: core_view,
    }),
};
