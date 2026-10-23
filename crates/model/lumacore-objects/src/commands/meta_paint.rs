// ███╗   ███╗███████╗████████╗ █████╗     ██████╗  █████╗ ██╗███╗   ██╗████████╗
// ████╗ ████║██╔════╝╚══██╔══╝██╔══██╗    ██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝
// ██╔████╔██║█████╗     ██║   ███████║    ██████╔╝███████║██║██╔██╗ ██║   ██║
// ██║╚██╔╝██║██╔══╝     ██║   ██╔══██║    ██╔═══╝ ██╔══██║██║██║╚██╗██║   ██║
// ██║ ╚═╝ ██║███████╗   ██║   ██║  ██║    ██║     ██║  ██║██║██║ ╚████║   ██║
// ╚═╝     ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝    ╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝
// FILE: meta_paint

//! The Paint of an object whose rows are keyed rather than tree members (a cue of a sequence, a
//! macro of a DataPool, a profile's view; decisions 0294: every show object with a NodeMeta can
//! be painted). The declaration is the one Paint property (`keyed_paint!`), read and written
//! through the object's own meta accessors and the one Paint parser.
//!
//! Owns:          keyed_paint!, get, set.
//! Doesn't own:   the Paint value (lumacore_base::paint), MetaLevel::Node's Paint of tree objects
//!                (lumacore_state::props).
//! Entry points:  keyed_paint!.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links, 0006.

use lumacore_base::{NodeMeta, Paint};

use crate::AppState;
use crate::errors::render_code;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The Paint property of a keyed object: `$meta(state, key)` reads a row's NodeMeta,
/// `$meta_mut(state, key)` writes it, `$label` names the row in replies.
#[macro_export]
macro_rules! keyed_paint {
    ($meta:path, $meta_mut:path, $label:literal) => {
        $crate::props::Property {
            name: "Paint",
            kind: $crate::props::PropKind::Paint,
            rights: $crate::commands::registry::Rights::Guest,
            section: "Meta",
            doc: "its colour on screen: #rrggbb, a palette colour, Theme \"<slot>\" (the viewer's theme), or none = its tags', then the pool's",
            access: $crate::props::Access::Keyed {
                get: |s, k| $crate::commands::meta_paint::get($meta(s, k)),
                set: |s, k, v| $crate::commands::meta_paint::set(s, k, v, $label, $meta_mut),
            },
            ..$crate::props::Property::BASE
        }
    };
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// A row's Paint as the line writes it (empty = none).
pub fn get(meta: Option<&NodeMeta>) -> String {
    meta.and_then(|m| m.annotation.paint.as_ref()).map(Paint::word).unwrap_or_default()
}

/// Write a row's Paint through the one parser.
pub fn set(state: &mut AppState, key: &str, value: &str, label: &str, meta_mut: for<'a> fn(&'a mut AppState, &str) -> Option<&'a mut NodeMeta>) -> Result<String, String> {
    let paint = Paint::parse(value).map_err(|e| render_code(format!("{label} {key} \"Paint\": {e}."), "E-SET-003"))?;
    let shown = paint.as_ref().map_or_else(|| "none".to_string(), Paint::word);
    let meta = meta_mut(state, key).ok_or_else(|| render_code(format!("No {label} {key}."), "E-CMD-007"))?;
    meta.annotation.paint = paint;
    state.show_dirty = true;
    Ok(format!("{label} {key} Paint = {shown}"))
}
