// ██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██╔═══╝ ██╔══██╗██║   ██║██╔═══╝ ╚════██║
// ██║     ██║  ██║╚██████╔╝██║     ███████║
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝
// FILE: props

//! A tag's own properties (decisions §12 "Tags hardened" 9): its Action (what Run action does
//! in the Tags pool window), its Priority (which painted tag colours an object, decisions 0294)
//! and its NodeMeta — Name, Note, Paint — on EditSetting's Meta tab. A tag's number is a path that
//! moves, so a row is the tag's GUID (the resolver's member).
//!
//! Owns:          PROPS (the Tag object's properties).
//! Doesn't own:   the one Set handler and EditSetting (lumacore-objects), the tree (lumacore-tags).
//! Entry points:  PROPS.
//! Decisions:     §12 "Tags hardened", §33 58, docs/decisions/0294-object-paint-palette-free-theme-links.

use lumacore_objects::AppState;
use lumacore_objects::errors::render_code;
use lumacore_objects::props::{Access, PropKind, Property, Props};
use lumacore_tags::tag::{self as tree, Tag, TagAction, TagPath};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The Tag object's properties, keyed by the tag's GUID.
pub static PROPS: Props = Props { lists: &[&TAG_PROPS], label: Some(label), ..Props::BASE };

static TAG_PROPS: [Property; 5] = [
    Property {
        name: "Action",
        kind: PropKind::Choice(&["Default", "Select"]),
        section: "Tag",
        doc: "what Run action does in the Tags window: the window's default, or Select (its fixtures)",
        default: Some("Default"),
        access: Access::Keyed { get: get_action, set: set_action },
        ..Property::BASE
    },
    Property {
        name: "Name",
        section: "Meta",
        doc: "display name, unique among its siblings",
        access: Access::Keyed { get: get_name, set: set_name },
        ..Property::BASE
    },
    Property {
        name: "Note",
        section: "Meta",
        doc: "operator note",
        access: Access::Keyed { get: get_note, set: set_note },
        ..Property::BASE
    },
    Property {
        name: "Priority",
        kind: PropKind::Int { min: -1000, max: 1000 },
        section: "Tag",
        doc: "an object without a Paint of its own takes the colour of its painted tag with the highest Priority (equal: the first in the tag tree)",
        default: Some("0"),
        access: Access::Keyed { get: get_priority, set: set_priority },
        ..Property::BASE
    },
    Property {
        name: "Paint",
        kind: PropKind::Paint,
        section: "Meta",
        doc: "its colour, and the colour of what it tags that has no Paint of its own: #rrggbb, a palette colour, Theme \"<slot>\", or none (its parent's)",
        access: Access::Keyed { get: get_paint, set: set_paint },
        ..Property::BASE
    },
];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn get_action(state: &AppState, key: &str) -> String {
    at(state, key).map(|(_, t)| match t.action {
        TagAction::WindowDefault => "Default",
        TagAction::SelectFixtures => "Select",
    })
    .unwrap_or_default()
    .to_string()
}

fn set_action(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let action = if value.eq_ignore_ascii_case("Select") { TagAction::SelectFixtures } else { TagAction::WindowDefault };
    edit(state, key, |t| t.action = action)?;
    Ok(format!("{}: Action {value}", label(state, key)))
}

fn get_name(state: &AppState, key: &str) -> String {
    at(state, key).map(|(_, t)| t.name().to_string()).unwrap_or_default()
}

fn set_name(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let (path, guid) = at(state, key).map(|(p, t)| (p, t.meta.identity.guid)).ok_or_else(gone)?;
    let name = Some(value.trim()).filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("none")).map(str::to_string);
    if let Some(n) = &name {
        lumacore_objects::resolve::check_name(n)?;
    }
    crate::label::rename(state, &path, guid, name)?;
    Ok(format!("{}: Name", label(state, key)))
}

fn get_note(state: &AppState, key: &str) -> String {
    at(state, key).and_then(|(_, t)| t.meta.annotation.note.clone()).unwrap_or_default()
}

fn set_note(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let note = Some(value.trim()).filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("none")).map(str::to_string);
    edit(state, key, |t| t.meta.annotation.note = note)?;
    Ok(format!("{}: Note", label(state, key)))
}

fn get_paint(state: &AppState, key: &str) -> String {
    at(state, key).and_then(|(_, t)| t.meta.annotation.paint.as_ref().map(|p| p.word())).unwrap_or_default()
}

/// The one Paint parser (decisions 0294): a colour, a palette word, a theme slot link or none.
fn set_paint(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let paint = lumacore_tree::paint::Paint::parse(value).map_err(|e| render_code(format!("{} \"Paint\": {e}.", label(state, key)), "E-SET-003"))?;
    let shown = paint.as_ref().map_or_else(|| "none".to_string(), |p| p.word());
    edit(state, key, |t| t.meta.annotation.paint = paint)?;
    Ok(format!("{}: Paint {shown}", label(state, key)))
}

fn get_priority(state: &AppState, key: &str) -> String {
    at(state, key).map(|(_, t)| t.priority.to_string()).unwrap_or_default()
}

fn set_priority(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let n: i32 = value.trim().parse().map_err(|_| render_code(format!("Priority is a whole number, not {value}."), "E-CMD-008"))?;
    edit(state, key, |t| t.priority = n)?;
    Ok(format!("{}: Priority {n}", label(state, key)))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The tag a row names (its GUID) and where it is now.
fn at<'a>(state: &'a AppState, key: &str) -> Option<(TagPath, &'a Tag)> {
    let guid = key.parse().ok()?;
    let tags = &state.root.show_data.tags;
    let path = tree::path_of(tags, guid)?;
    tree::find(tags, &path).map(|t| (path, t))
}

fn edit(state: &mut AppState, key: &str, f: impl FnOnce(&mut Tag)) -> Result<(), String> {
    let path = at(state, key).map(|(p, _)| p).ok_or_else(gone)?;
    tree::find_mut(&mut state.root.show_data.tags, &path).map(f).ok_or_else(gone)
}

fn gone() -> String {
    render_code("No such Tag.", "E-TAG-001")
}

/// `Tag 1.1 "Lx1"`: how replies and the settings title name a row.
fn label(state: &AppState, key: &str) -> String {
    at(state, key).map(|(p, _)| crate::tag::label(state, &p)).unwrap_or_else(|| "Tag".into())
}
