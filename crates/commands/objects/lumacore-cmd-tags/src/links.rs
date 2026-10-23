// ██╗     ██╗███╗   ██╗██╗  ██╗███████╗
// ██║     ██║████╗  ██║██║ ██╔╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╔╝ ███████╗
// ██║     ██║██║╚██╗██║██╔═██╗ ╚════██║
// ███████╗██║██║ ╚████║██║  ██╗███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝
// FILE: links

//! The tag links on any show object (decisions §12 "Tags hardened" 1): `NodeMeta.tags` wherever
//! the object sits. There is no mutable walk over the show, so the show goes through its JSON: every
//! `meta` with a matching `guid` gets its `tags` changed, then the show is read back and its
//! unsaved pools rebuilt. Generic and exhaustive; fast enough for a line (roadmap: a typed walk).
//!
//! Owns:          retag, strip.
//! Doesn't own:   what a link means (lumacore-tags), the change path (dispatch fingerprints it).
//! Entry points:  retag, strip.
//! Decisions:     §12 "Tags hardened", §21 "6d" (links are GUIDs).

use lumacore_objects::AppState;
use lumacore_objects::errors::render_code;
use serde_json::{Map, Value};
use uuid::Uuid;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Add `add` to and take `remove` from the tags of each object in `objects`; returns the objects
/// not found (no NodeMeta with that GUID).
pub(crate) fn retag(state: &mut AppState, objects: &[Uuid], add: &[Uuid], remove: &[Uuid]) -> Result<Vec<Uuid>, String> {
    let wanted: Vec<String> = objects.iter().map(Uuid::to_string).collect();
    let add: Vec<String> = add.iter().map(Uuid::to_string).collect();
    let remove: Vec<String> = remove.iter().map(Uuid::to_string).collect();
    let mut seen: Vec<String> = Vec::new();
    rewrite(state, |meta| {
        let Some(guid) = meta.get("guid").and_then(Value::as_str).map(str::to_string) else { return false };
        if !wanted.contains(&guid) {
            return false;
        }
        seen.push(guid);
        let tags = meta.entry("tags").or_insert_with(|| Value::Array(Vec::new()));
        let Some(list) = tags.as_array_mut() else { return false };
        let before = list.clone();
        list.retain(|t| t.as_str().is_none_or(|s| !remove.iter().any(|r| r == s)));
        for a in &add {
            if !list.iter().any(|t| t.as_str() == Some(a)) {
                list.push(Value::String(a.clone()));
            }
        }
        *list != before
    })?;
    Ok(objects.iter().filter(|g| !seen.contains(&g.to_string())).copied().collect())
}

/// Take every link to `gone` off every object; returns how many objects changed.
pub(crate) fn strip(state: &mut AppState, gone: &[Uuid]) -> Result<usize, String> {
    if gone.is_empty() {
        return Ok(0);
    }
    let gone: Vec<String> = gone.iter().map(Uuid::to_string).collect();
    rewrite(state, |meta| {
        let Some(list) = meta.get_mut("tags").and_then(Value::as_array_mut) else { return false };
        let n = list.len();
        list.retain(|t| t.as_str().is_none_or(|s| !gone.iter().any(|g| g == s)));
        list.len() != n
    })
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Run `f` on every NodeMeta of the show (a JSON object under a `meta` key that has a `guid`);
/// when any changed, read the show back. Returns how many changed.
fn rewrite(state: &mut AppState, mut f: impl FnMut(&mut Map<String, Value>) -> bool) -> Result<usize, String> {
    let fail = |e: serde_json::Error| render_code(format!("The show's tags couldn't be changed: {e}"), "E-INTERNAL-001");
    let mut show = serde_json::to_value(&state.root.show_data).map_err(fail)?;
    let mut changed = 0;
    visit(&mut show, &mut |meta| {
        if f(meta) {
            changed += 1;
        }
    });
    if changed == 0 {
        return Ok(0);
    }
    let mut back: lumacore_tree::ShowData = serde_json::from_value(show).map_err(fail)?;
    back.rebuild_id_pools();
    back.rebuild_occupancy();
    state.root.show_data = back;
    Ok(changed)
}

fn visit(v: &mut Value, f: &mut impl FnMut(&mut Map<String, Value>)) {
    match v {
        Value::Object(map) => {
            if let Some(Value::Object(meta)) = map.get_mut("meta")
                && meta.contains_key("guid")
            {
                f(meta);
            }
            for (k, child) in map.iter_mut() {
                if k != "meta" {
                    visit(child, f);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|i| visit(i, f)),
        _ => {}
    }
}
