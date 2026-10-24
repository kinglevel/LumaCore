//  █████╗ ██████╗ ███╗   ███╗███████╗██████╗
// ██╔══██╗██╔══██╗████╗ ████║██╔════╝██╔══██╗
// ███████║██████╔╝██╔████╔██║█████╗  ██║  ██║
// ██╔══██║██╔══██╗██║╚██╔╝██║██╔══╝  ██║  ██║
// ██║  ██║██║  ██║██║ ╚═╝ ██║███████╗██████╔╝
// ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝╚═════╝
// FILE: armed

//! The armed tag (decisions 0297): a command line that reads `Assign Tag <path> At` or `Unassign
//! Tag <path> At` with nothing after it waits for objects. A tap on a pool cell finishes it and the
//! line comes back armed; the pools mark what carries the tag. The terminal asks here after each
//! key and tap, and sends the `armed_tag` event when the answer changes.
//!
//! Owns:          `armed` (is this line armed, and with what), `Armed`.
//! Doesn't own:   the tap rules (lumacore-terminal), the Assign lines (assign.rs).
//! Entry points:  armed, Armed.
//! Decisions:     docs/decisions/0297-tag-squares-and-picker.

use lumacore_objects::AppState;
use lumacore_objects::commands::registry::{self, Word};
use lumacore_objects::objects::Verb;
use lumacore_tree::tags::tag::{self as tree, TagPath};

use crate::tag::TAG;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// An armed line's tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armed {
    /// `Assign` (else `Unassign`).
    pub assign: bool,
    /// The tag's path as typed (`1.2`).
    pub path: String,
    /// Its colour for the current user (`#rrggbb`), "" when it has none.
    pub colour: String,
    /// The objects that carry it.
    pub carriers: Vec<uuid::Uuid>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The armed tag of `line`: `Assign` / `Unassign Tag <path> At` naming a tag that exists, or None.
pub fn armed(state: &AppState, line: &str) -> Option<Armed> {
    let (assign, path) = arming(line)?;
    let tags = &state.root.show_data.tags;
    let tag = tree::find(tags, &path)?;
    let guid = tag.meta.identity.guid;
    let colour = lumacore_objects::paint::Painter::new(&state.root.show_data, state.effective_user_name()).tag(guid).unwrap_or_default();
    let carriers = tree::members(&state.root.show_data.walk_identity(), &[guid]).iter().map(|m| m.guid).collect();
    Some(Armed { assign, path: path.to_string(), colour, carriers })
}

/// The words alone: `(assign, path)` when `line` is a verb Assign / Unassign, the Tag object, a
/// path and At, with nothing after it.
pub fn arming(line: &str) -> Option<(bool, TagPath)> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let [verb, object, path, at] = words.as_slice() else { return None };
    let assign = match registry::resolve(verb) {
        Ok(Word::Command(c)) if c.object_verb == Some(Verb::Assign) => true,
        Ok(Word::Command(c)) if c.object_verb == Some(Verb::Unassign) => false,
        _ => return None,
    };
    let is_tag = matches!(registry::resolve(object), Ok(Word::Object(o)) if std::ptr::eq(o, &TAG));
    (is_tag && at.eq_ignore_ascii_case("at")).then_some(())?;
    Some((assign, TagPath::parse(path).ok()?))
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
