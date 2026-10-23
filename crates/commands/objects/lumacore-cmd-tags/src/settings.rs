// ███████╗███████╗████████╗████████╗██╗███╗   ██╗ ██████╗ ███████╗
// ██╔════╝██╔════╝╚══██╔══╝╚══██╔══╝██║████╗  ██║██╔════╝ ██╔════╝
// ███████╗█████╗     ██║      ██║   ██║██╔██╗ ██║██║  ███╗███████╗
// ╚════██║██╔══╝     ██║      ██║   ██║██║╚██╗██║██║   ██║╚════██║
// ███████║███████╗   ██║      ██║   ██║██║ ╚████║╚██████╔╝███████║
// ╚══════╝╚══════╝   ╚═╝      ╚═╝   ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
// FILE: settings

//! The Tags tab of every show object's EditSetting (decisions §12 "Tags hardened" 8): the tag tree
//! as a picker, ticked where the object carries the tag; a tick is `Assign Tag … At <object>`, an
//! untick `Unassign …`, "New tag here" `Store Tag …`. A `SettingsTab` row, so the kernel never names
//! tags.
//!
//! Owns:          TABS, the Tags field.
//! Doesn't own:   EditSetting and its other tabs (lumacore-objects), the kit's tree (kit/tree.js).
//! Entry points:  TABS.
//! Decisions:     §12 "Tags hardened", §30 "12b2" (typed settings).

use lumacore_objects::AppState;
use lumacore_objects::objects::SettingsTab;
use lumacore_tags::tag as tree;
use lumacore_wire::{FieldInput, SettingField, TagNode};
use uuid::Uuid;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The Tags tab, for the catalog.
pub static TABS: &[SettingsTab] = &[SettingsTab { fields }];

fn fields(state: &AppState, object: Uuid, target: &str) -> Vec<SettingField> {
    let walk = state.root.show_data.walk_identity();
    let carried: Vec<Uuid> = walk.links.iter().filter(|l| l.kind == "Tag" && l.from == Some(object)).map(|l| l.target).collect();
    let tags = &state.root.show_data.tags;
    let nodes: Vec<TagNode> = tree::all(tags)
        .into_iter()
        .map(|(path, t)| TagNode {
            path: path.to_string(),
            name: t.meta.identity.name.clone().unwrap_or_default(),
            colour: t.meta.annotation.paint.map(|c| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])).unwrap_or_default(),
            state: if carried.contains(&t.meta.identity.guid) { "on" } else { "off" }.to_string(),
            count: tree::members(&walk, &[t.meta.identity.guid]).len(),
        })
        .collect();
    let names: Vec<String> = tree::all(tags).into_iter().filter(|(_, t)| carried.contains(&t.meta.identity.guid)).map(|(p, t)| format!("{p} {}", t.name())).collect();
    vec![SettingField {
        name: "Tags".to_string(),
        key: String::new(),
        changed: false,
        reset: String::new(),
        section: "Tags".to_string(),
        input: FieldInput::Tags {
            nodes,
            assign: format!("Assign Tag {{value}} At {target}"),
            unassign: format!("Unassign Tag {{value}} At {target}"),
            store: "Store Tag {value} Tag {name}".to_string(),
            store_top: "Store Tag {name}".to_string(),
        },
        hint: "Tick a tag to give it to this object; untick to take it off.".to_string(),
        value: if names.is_empty() { "none".to_string() } else { names.join(", ") },
        line: String::new(),
        pick: None,
        home: "show".to_string(),
        unit: String::new(),
        rights: "Program".to_string(),
        readonly: String::new(),
    }]
}
