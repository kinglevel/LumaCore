//! `Label <target> ["name"] [/Clear]` (decisions §33 3d): one typed handler for every object
//! that declares a `"Name"` property. The target goes through the one resolver; each row is
//! written through the property registry (`set::write`: rights, the profile rule, the change
//! path). Several rows number on from the name (`Label Fixture 1 Thru 4 "Spot 1"` → Spot 1…4,
//! keeping the spacing of the numbers); one row without a name asks for it.

use lumacore_syntax::ast::Target;

use crate::commands::{edit, set};
use crate::errors::render_code;
use crate::text::render_success;
use crate::{AppState, DispatchResult, Reply};

#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Label")]
pub struct Label {
    #[lum(target)]
    pub target: Target,
    #[lum(value)]
    pub name: Option<String>,
    #[lum(option = "Clear")]
    pub clear: bool,
}

/// Whether `Label` on `obj` is this handler's (it declares a `"Name"` and has no Label of its own).
pub fn takes(obj: &crate::objects::ObjectDef) -> bool {
    !obj.supports(crate::objects::Verb::Label) && obj.props.is_some_and(|p| p.get("Name").is_some())
}

impl crate::commands::typed::Command for Label {
    fn run(self, state: &mut AppState) -> DispatchResult {
        DispatchResult::Output(label(state, &self).unwrap_or_else(Reply::from))
    }
}

fn label(state: &mut AppState, l: &Label) -> Result<Reply, String> {
    let resolved = crate::resolve::resolve_one(state, &l.target, crate::objects::default_object(state))?;
    let obj = resolved.object;
    let (Some(props), true) = (obj.props, takes(obj)) else {
        return Err(render_code(format!("{} has no name to label.", obj.keyword), "E-CMD-015"));
    };
    let prop = props.get("Name").expect("takes() checked it");
    if !resolved.named {
        return Err(render_code(format!("Label which {}? Label {} <n> \"name\"", obj.keyword, obj.keyword), "E-CMD-006"));
    }
    let rows = set::rows_of(state, obj, props, &resolved, prop)?;
    if rows.is_empty() {
        return Err(render_code(format!("{} not found: none of those exist.", obj.keyword), "E-CMD-007"));
    }
    // Another DataPool than yours is named owner first in the ask's line.
    let owner = resolved
        .datapool
        .filter(|&dp| obj.tree.as_ref().is_some_and(|t| t.datapool.is_some()) && dp != state.selected_datapool())
        .map(|dp| format!("DataPool {dp} "))
        .unwrap_or_default();
    let name = match (&l.name, l.clear) {
        (_, true) => String::new(),
        (Some(n), false) => n.clone(),
        // One row and no name: the station asks (an empty answer clears).
        (None, false) => {
            let [row] = rows.as_slice().iter().filter(|_| one_id(&l.target)).collect::<Vec<_>>()[..] else {
                return Err(render_code(format!("Label {} … \"name\" | /Clear — several rows need the name.", obj.keyword), "E-CMD-006"));
            };
            let line = format!("Label {owner}{} {}", obj.keyword, row.key);
            let current = set::current(state, obj, row, prop).unwrap_or_default();
            state.asking = Some(
                crate::dialogs::Ask::new(line.clone(), "A new name. Empty clears the label.")
                    .input("name", "Name", current, lumacore_wire::InputKind::Text)
                    .button("Please", lumacore_wire::ButtonKind::Please, format!("{line} \"{{name}}\""))
                    .or_when_empty(format!("{line} /Clear"))
                    .cancel("Cancel")
                    .flags(format!("the name ({line} \"name\"), or /Clear")),
            );
            return Err(render_code(format!("Usage: {line} \"name\" | /Clear"), "E-CMD-006"));
        }
    };
    crate::resolve::check_name(&name)?;
    let sanitized = edit::sanitize_label(&name);
    let first = rows[0].no;
    let mut outs: Vec<Reply> = Vec::new();
    for row in &rows {
        let value = match (sanitized.is_empty(), rows.len()) {
            (true, _) => String::new(),
            (false, 1) => sanitized.clone(),
            // Number on by the row's distance from the first (the spacing is kept).
            (false, _) => edit::enumerate_name(&sanitized, row.no.saturating_sub(first) as usize),
        };
        outs.push(match set::write(state, obj, props, row, prop, &value) {
            Ok(_) if value.is_empty() => render_success(&format!("{} label cleared.", row.label)).into(),
            Ok(_) => render_success(&format!("{} labeled \"{value}\".", row.label)).into(),
            Err(e) => e.into(),
        });
    }
    if sanitized.len() != name.len() {
        outs.push(render_success("Note: Some restricted characters were removed from the label.").into());
    }
    Ok(Reply::joined(outs))
}

/// The target names one id, no range (`Label Fixture 1 Thru 5` with only 1 there still needs a
/// name or /Clear).
fn one_id(target: &Target) -> bool {
    use lumacore_syntax::ast::{Item, Range};
    match target.items.as_slice() {
        [(None, Item::Path(segments))] => segments.last().and_then(|s| s.ids.as_ref()).is_some_and(|ids| matches!(ids.ranges.as_slice(), [(None, Range::One(_))])),
        [(None, Item::Ids(ids))] => matches!(ids.ranges.as_slice(), [(None, Range::One(_))]),
        _ => false,
    }
}
