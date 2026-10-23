// ██████╗  █████╗ ██╗███╗   ██╗████████╗     ██████╗ ██████╗      ██╗███████╗ ██████╗████████╗
// ██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝    ██╔═══██╗██╔══██╗     ██║██╔════╝██╔════╝╚══██╔══╝
// ██████╔╝███████║██║██╔██╗ ██║   ██║       ██║   ██║██████╔╝     ██║█████╗  ██║        ██║
// ██╔═══╝ ██╔══██║██║██║╚██╗██║   ██║       ██║   ██║██╔══██╗██   ██║██╔══╝  ██║        ██║
// ██║     ██║  ██║██║██║ ╚████║   ██║       ╚██████╔╝██████╔╝╚█████╔╝███████╗╚██████╗   ██║
// ╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝        ╚═════╝ ╚═════╝  ╚════╝ ╚══════╝ ╚═════╝   ╚═╝
// FILE: paint_object

//! `Paint <target>` (decisions 0296): the paint picker on the objects the line names — `Paint Group
//! 1`, `Paint Sequence 1`, `Paint DataPool 2 Group 1 Thru 3`; any object with a `"Paint"`. The page
//! and the app open `<luma-paint>` (the `paint_picker` event) and its answer is one `Set <target>
//! "Paint" <value>` line; a terminal gets the Paint now and that line to type.
//!
//! Owns:          the Paint verb.
//! Doesn't own:   the picker (kit/paint.js), what a Paint is (lumacore_base::Paint), the Set line.
//! Entry points:  PaintObject.
//! Decisions:     docs/decisions/0296-paint-verb-and-theme-slots, docs/decisions/0294-object-paint-palette-free-theme-links.

use lumacore_syntax::ast::Target;

use crate::commands::set;
use crate::errors::render_code;
use crate::objects::out;
use crate::props::PropKind;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Paint <target>`: open the paint picker on those objects.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Paint")]
pub struct PaintObject {
    #[lum(target)]
    pub target: Option<Target>,
}

impl crate::commands::typed::Command for PaintObject {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let Some(target) = self.target else {
            return out(render_code("Paint what? Paint <object> <ids>: Paint Group 1, Paint Sequence 1", "E-CMD-006"));
        };
        let resolved = match crate::resolve::resolve_one(state, &target, crate::objects::default_object(state)) {
            Ok(r) => r,
            Err(e) => return out(e),
        };
        let obj = resolved.object;
        let paint = obj.props.and_then(|p| p.all().find(|p| matches!(p.kind, PropKind::Paint)).map(|prop| (p, prop)));
        let Some((props, prop)) = paint else {
            return out(render_code(format!("{} has no Paint.", obj.keyword), "E-SET-001"));
        };
        let rows = match set::rows_of(state, obj, props, &resolved, prop) {
            Ok(r) if !r.is_empty() => r,
            Ok(_) => return out(render_code(format!("No such {}.", obj.keyword), "E-CMD-007")),
            Err(e) => return out(e),
        };
        let words = lumacore_syntax::target_text(&target, crate::lum::vocabulary());
        let value = set::current(state, obj, &rows[0], prop).unwrap_or_default();
        let name = match resolved.members.as_slice() {
            [one] => one.name.clone().unwrap_or_default(),
            _ => String::new(),
        };
        let line = format!("Set {words} \"{}\" {{value}}", prop.name);
        state.ui_requests.push(lumacore_wire::UiEvent::PaintPicker { object: words.clone(), name, line: line.clone(), value: value.clone() });
        let shown = if value.is_empty() { "none (its tags', then its pool's colour)".to_string() } else { value };
        out(crate::text::render_success(&format!("Paint {words}: {shown} — {}", line.replace("{value}", "<value>"))))
    }
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
