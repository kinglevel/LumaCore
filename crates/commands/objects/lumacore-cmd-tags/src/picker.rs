// ██████╗ ██╗ ██████╗██╗  ██╗███████╗██████╗
// ██╔══██╗██║██╔════╝██║ ██╔╝██╔════╝██╔══██╗
// ██████╔╝██║██║     █████╔╝ █████╗  ██████╔╝
// ██╔═══╝ ██║██║     ██╔═██╗ ██╔══╝  ██╔══██╗
// ██║     ██║╚██████╗██║  ██╗███████╗██║  ██║
// ╚═╝     ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
// FILE: picker

//! `Tag <objects>` (decisions 0297): a Tag with no path and objects after it opens the tag picker
//! on them — `Tag Group 3`, `Tag Sequence 1 Thru 4`, `Tag Fixture 101`; any object with a NodeMeta.
//! The page and the app open `<luma-tag-picker>` (the `tag_picker` event, fed by the `tags` feed's
//! `on`); each tick there is one `Assign` / `Unassign Tag <path> At <objects>` line. A terminal
//! gets the tags they carry and those lines to type.
//!
//! Owns:          the picker form of the Tag object (`open`).
//! Doesn't own:   the picker (kit/tagpicker.js), the Assign lines (assign.rs), the resolver.
//! Entry points:  open.
//! Decisions:     docs/decisions/0297-tag-squares-and-picker, docs/decisions/0207-tags-pool-and-links.

use lumacore_objects::AppState;
use lumacore_objects::errors::render_code;
use lumacore_syntax::ast::{Item, Target, Value};
use lumacore_tree::tags::tag as tree;
use uuid::Uuid;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Tag <objects>`: the tag picker on those objects. `target` is the Tag the line starts with: a
/// path there names a tag, and naming objects after it is the Assign line's job.
pub(crate) fn open(state: &mut AppState, target: &Target, on: &Value) -> Result<String, String> {
    let bare = matches!(target.items.as_slice(), [(None, Item::Path(segments))] if segments.len() == 1 && segments[0].ids.is_none());
    if !bare {
        return Err(render_code("A tag and objects: to tag them, Assign Tag 1 At Group 3; for the tag picker, Tag Group 3.", "E-CMD-006"));
    }
    let Value::Target(objects) = on else {
        return Err(render_code("Tag what? Tag <objects> opens the tag picker: Tag Group 3, Tag Sequence 1 Thru 4.", "E-CMD-006"));
    };
    let items = lumacore_objects::resolve::resolve(state, objects, &lumacore_objects::objects::FIXTURE)?;
    let mut guids: Vec<Uuid> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for (op, item) in &items {
        let found: Vec<Uuid> = item.members.iter().filter_map(|m| m.guid).collect();
        if found.len() < item.members.len() {
            return Err(render_code(format!("A {} can't carry tags yet.", item.object.keyword), "E-CMD-010"));
        }
        match op {
            Some(lumacore_syntax::ast::SetOp::Minus) => guids.retain(|g| !found.contains(g)),
            _ => {
                for m in &item.members {
                    if let Some(g) = m.guid.filter(|g| !guids.contains(g)) {
                        guids.push(g);
                        names.push(m.name.clone().unwrap_or_default());
                    }
                }
            }
        }
    }
    if guids.is_empty() {
        return Err(render_code("Nothing to tag.", "E-CMD-009"));
    }
    let words = lumacore_syntax::target_text(objects, lumacore_objects::lum::vocabulary());
    let name = match names.as_slice() {
        [one] => one.clone(),
        _ => String::new(),
    };
    state.ui_requests.push(lumacore_wire::UiEvent::TagPicker {
        object: words.clone(),
        name,
        guids: guids.iter().map(Uuid::to_string).collect(),
    });
    Ok(format!("Tags of {words}: {} — tick them in the tag picker, or type Assign Tag <path> At {words}", carried(state, &guids)))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// "Tag 2 "FOH", Tag 901 "Wash"": the tags every one of `guids` carries, or "none".
fn carried(state: &AppState, guids: &[Uuid]) -> String {
    let walk = state.root.show_data.walk_identity();
    let tags = &state.root.show_data.tags;
    let all: Vec<String> = tree::all(tags)
        .into_iter()
        .filter(|(_, t)| {
            let members = tree::members(&walk, &[t.meta.identity.guid]);
            guids.iter().all(|g| members.iter().any(|m| m.guid == *g))
        })
        .map(|(p, _)| crate::tag::label_in(tags, &p))
        .collect();
    if all.is_empty() { "none".into() } else { all.join(", ") }
}
