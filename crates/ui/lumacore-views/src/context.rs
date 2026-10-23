//  ██████╗ ██████╗ ███╗   ██╗████████╗███████╗██╗  ██╗████████╗
// ██╔════╝██╔═══██╗████╗  ██║╚══██╔══╝██╔════╝╚██╗██╔╝╚══██╔══╝
// ██║     ██║   ██║██╔██╗ ██║   ██║   █████╗   ╚███╔╝    ██║
// ██║     ██║   ██║██║╚██╗██║   ██║   ██╔══╝   ██╔██╗    ██║
// ╚██████╗╚██████╔╝██║ ╚████║   ██║   ███████╗██╔╝ ██╗   ██║
//  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝
// FILE: context

//! The one context menu of every list, grid and pool (decisions §34 round 3, 18, the approved
//! `context-menu.html`; `GET /api/object-menus`): per object, from the registry only — the
//! common actions its verbs take (Edit…, Renumber…, Label, Copy, Move…, Delete), a Set part with
//! every declared property (its editor by kind, its level, live or not), then its own lines (a
//! `ContextDef` row, else Info). A window adds nothing of its own; the kit's `<luma-context>`
//! draws it on the selection and fills `{object}` / `{rows}`.
//!
//! Owns:          object_menus_json.
//! Doesn't own:   the objects' verbs and properties (`lumacore_objects`), the drawing (kit
//!                `context.js`), the rows' values (the grid's or pool's feed).
//! Entry points:  object_menus_json.
//! Decisions:     §34 round 3, 18.

use lumacore_objects::objects::{ContextDef, ObjectDef, Verb};
use lumacore_objects::props::Access;
use serde_json::{Map, Value, json};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Every object's menu, by keyword (built once: the registry doesn't change at run time).
pub fn object_menus_json() -> Value {
    let own: Vec<&ContextDef> = lumacore_registry::get().rows::<ContextDef>().collect();
    let mut out = Map::new();
    for obj in lumacore_objects::objects::all() {
        let items: Vec<&ContextDef> = own.iter().copied().filter(|c| std::ptr::eq(c.object, &obj.decl)).collect();
        let menu = menu_of(obj, &items);
        if menu["common"].as_array().is_some_and(|a| !a.is_empty()) || menu["set"].as_array().is_some_and(|a| !a.is_empty()) {
            out.insert(obj.keyword.to_string(), menu);
        }
    }
    Value::Object(out)
}

/// One object's menu: `{object, plural, common: [{id, label, icon, line, rights, claim, value?}],
/// set: [{name, input, rights, live, section, unit, meta}], own: [{label, icon, line, rights,
/// live, value}]}`; `value` names the value dialog's kind for `{value}` (`number`, `address`).
/// `claim`: it changes more than live columns, so a claim area's Live greys it.
fn menu_of(obj: &'static ObjectDef, own: &[&ContextDef]) -> Value {
    let rights = |v: Verb| (obj.rights)(v, &[]).label();
    let on_door = obj.tree.as_ref().is_some_and(|t| t.door.is_some());
    let mut common = Vec::new();
    // Edit… is the edit card (every field in tabs), the host's to open.
    if obj.props.is_some_and(|p| p.readonly.is_none()) {
        common.push(json!({ "id": "edit", "label": "Edit…", "icon": "pencil", "line": null, "rights": rights(Verb::Set), "claim": false }));
    }
    // A name is a live column where the object has one (a fixture's Name).
    let name_live = obj.props.is_some_and(|p| p.is_live("Name"));
    // Label names every object with a "Name" property (the Label command), or its own verb.
    let label = obj.supports(Verb::Label) || obj.props.is_some_and(|p| p.get("Name").is_some());
    // An action the object's verbs take (`take`: its own verb, or a command that reaches it).
    let item = |id: &str, label: &str, icon: &str, verb: Verb, line: &str, claim: bool, take: bool| {
        take.then(|| json!({ "id": id, "label": label, "icon": icon, "line": line, "rights": rights(verb), "claim": claim }))
    };
    common.extend(
        [
            item("renumber", "Renumber…", "hash", Verb::Move, "Move {object} {rows} At {value}", true, obj.supports(Verb::Move)).map(|mut v| {
                // The value dialog's number kind: preview, free numbers, Skip taken / Swap.
                v["value"] = json!("number");
                v
            }),
            item("label", "Label…", "tag", Verb::Label, "Label {object} {rows} {value}", !name_live, label),
            // On the object door the station keeps the clipboard (decisions §35 9).
            item(
                "copy",
                "Copy",
                "copy",
                Verb::Copy,
                if on_door { "Copy {object} {rows}" } else { "Copy {object} {rows} At {value}" },
                true,
                obj.supports(Verb::Copy),
            ),
            item("move", "Move…", "move", Verb::Move, "Move {object} {rows} At {value}", true, obj.supports(Verb::Move)),
            // Into a node of the MVR tree (decisions §40 "40d hardened" 3): the host picks it.
            item("move_into", "Move into…", "folder-input", Verb::Assign, "Assign {object} {rows} At {value}", true, std::ptr::eq(obj, &lumacore_objects::objects::FIXTURE)).map(|mut v| {
                v["value"] = json!("node");
                v
            }),
            item("delete", "Delete", "trash-2", Verb::Delete, "Delete {object} {rows}", true, obj.supports(Verb::Delete)),
        ]
        .into_iter()
        .flatten(),
    );
    let set: Vec<Value> = match obj.props {
        Some(p) if p.readonly.is_none() => p
            .all()
            .map(|s| {
                json!({
                    "name": s.name,
                    "input": lumacore_objects::commands::edit_setting::input(s),
                    "rights": lumacore_objects::props::rights_of(obj, s).label(),
                    "live": p.is_live(s.name),
                    "section": s.section,
                    "unit": s.unit,
                    // The levels every show object has (Name, Note, Paint …): a pool's menu lists
                    // them after the object's own.
                    "meta": matches!(s.access, Access::Field(f) if f.starts_with("meta.")),
                })
            })
            .collect(),
        _ => Vec::new(),
    };
    let mut own: Vec<Value> = own
        .iter()
        .flat_map(|c| c.items.iter())
        .map(|i| json!({ "label": i.label, "icon": i.icon, "line": i.line, "rights": rights(i.verb), "live": i.live, "value": i.value }))
        .collect();
    if own.is_empty() && obj.supports(Verb::Info) {
        own.push(json!({ "label": "Info", "icon": "info", "line": "Info {object} {rows}", "rights": rights(Verb::Info), "live": true }));
    }
    json!({ "object": obj.keyword, "plural": format!("{}s", obj.keyword), "common": common, "set": set, "own": own })
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
