// ██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██╔═══╝ ██╔══██╗██║   ██║██╔═══╝ ╚════██║
// ██║     ██║  ██║╚██████╔╝██║     ███████║
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝
// FILE: props

//! A tag's own properties (decisions §12 "Tags hardened" 9): its Action (what Run action does
//! in the Tags pool window) and its NodeMeta — Name, Note, Paint — on EditSetting's Meta tab. A
//! tag's number is a path that moves, so a row is the tag's GUID (the resolver's member).
//!
//! Owns:          PROPS (the Tag object's properties).
//! Doesn't own:   the one Set handler and EditSetting (lumacore-objects), the tree (lumacore-tags).
//! Entry points:  PROPS.
//! Decisions:     §12 "Tags hardened", §33 58.

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

static TAG_PROPS: [Property; 4] = [
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
        name: "Paint",
        kind: PropKind::Rgba,
        section: "Meta",
        doc: "its colour in the tree",
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
    at(state, key).and_then(|(_, t)| t.meta.annotation.paint).map(|[r, g, b, _]| format!("#{r:02x}{g:02x}{b:02x}")).unwrap_or_default()
}

fn set_paint(state: &mut AppState, key: &str, value: &str) -> Result<String, String> {
    let text = value.trim().trim_start_matches('#');
    let colour = match text.to_ascii_lowercase().as_str() {
        "" | "none" => None,
        hex if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) => {
            let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            Some([byte(0), byte(2), byte(4), 255])
        }
        _ => return Err(render_code(format!("Paint takes a colour #rrggbb (or none), not {value}."), "E-CMD-008")),
    };
    edit(state, key, |t| t.meta.annotation.paint = colour)?;
    Ok(format!("{}: Paint {}", label(state, key), if colour.is_some() { value.trim() } else { "none" }))
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
