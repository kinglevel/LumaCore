// ██╗   ██╗██╗    ██╗  ██╗███████╗██╗   ██╗
// ██║   ██║██║    ██║ ██╔╝██╔════╝╚██╗ ██╔╝
// ██║   ██║██║    █████╔╝ █████╗   ╚████╔╝
// ██║   ██║██║    ██╔═██╗ ██╔══╝    ╚██╔╝
// ╚██████╔╝██║    ██║  ██╗███████╗   ██║
//  ╚═════╝ ╚═╝    ╚═╝  ╚═╝╚══════╝   ╚═╝
// FILE: ui key

//! UI keys: a button with a command line per gesture (Press, Release, Hold, ReleaseAfterHold,
//! DoublePress) and Meow lines — core, plugin or the profile's.
//!
//! Owns:          the UIKey object: List / Info, its completion, `Root.Resources.UIKeys`.
//! Doesn't own:   the keys' data (lumacore-tree ui_keys), the checks (lines.rs).
//! Entry points:  UI_KEY.
//! Decisions:     §17, §30 "12c".

use lumacore_tree::ui_keys::Gesture;
use lumacore_tree::{Root, UiKey, UiKeyOwner};

use lumacore_objects::errors::render_code;
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
    props: None,
    tree: Some(TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: lumacore_objects::tree_path::Node::UIKeys,
        children: core_children,
        object: core_view,
    }),
};
