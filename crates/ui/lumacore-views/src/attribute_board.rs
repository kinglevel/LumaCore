//  █████╗ ████████╗████████╗██████╗ ██╗██████╗ ██╗   ██╗████████╗███████╗███████╗
// ██╔══██╗╚══██╔══╝╚══██╔══╝██╔══██╗██║██╔══██╗██║   ██║╚══██╔══╝██╔════╝██╔════╝
// ███████║   ██║      ██║   ██████╔╝██║██████╔╝██║   ██║   ██║   █████╗  ███████╗
// ██╔══██║   ██║      ██║   ██╔══██╗██║██╔══██╗██║   ██║   ██║   ██╔══╝  ╚════██║
// ██║  ██║   ██║      ██║   ██║  ██║██║██████╔╝╚██████╔╝   ██║   ███████╗███████║
// ╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚═╝  ╚═╝╚═╝╚═════╝  ╚═════╝    ╚═╝   ╚══════╝╚══════╝
// FILE: attributes

//! The attribute board's view (decisions §30 "12f-a": board D,
//! `<luma-attr-board>`): one layer — the show's feature groups or a
//! profile's own groups — as the `attributes` feed sends it. Pure: it reads
//! a published show; every gesture on the board is a command line.
//!
//! Owns:          `view` (the `attributes` feed: one layer of the board).
//! Doesn't own:   `BoardTarget`, `owner_words` (`lumacore_objects::attribute_board`), the layers (`lumacore_tree::attribute_resolver`).
//! Entry points:  view.
//! Decisions:     §30 "12f-a", §34 20.7.

use lumacore_objects::attribute_board::{BoardTarget, owner_words};
use lumacore_tree::ShowData;
use lumacore_tree::attribute_resolver::{self, AttributeLayer};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A standard icon's name (a picture of the Images pool has none).
pub(crate) fn icon_word(meta: &lumacore_base::NodeMeta) -> Option<String> {
    match &meta.annotation.icon {
        Some(lumacore_base::IconRef::Builtin(n)) => Some(n.clone()),
        _ => None,
    }
}

pub(crate) fn hex(c: Option<[u8; 4]>) -> Value {
    c.map_or(Value::Null, |[r, g, b, _]| json!(format!("#{r:02x}{g:02x}{b:02x}")))
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The board of `target` as `user` sees it; None when the profile is gone.
pub fn view(show: &ShowData, user: &str, target: BoardTarget) -> Option<Value> {
    let (layer, title, locked): (&dyn AttributeLayer, String, bool) = match target {
        BoardTarget::Show => (show, "Show".into(), false),
        BoardTarget::Profile(no) => {
            let p = show.user_profiles.get(&no)?;
            (&p.content.attributes, p.meta.identity.name.clone().unwrap_or_default(), p.locked)
        }
    };
    let used = attribute_resolver::used_attributes(show);
    // Which patched fixture types use each attribute.
    let mut types: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let ft_of = &show.devices.dmx.patch.fixture_types;
    for f in show.devices.dmx.patch.fixtures.values() {
        let Some(ft) = ft_of.values().find(|t| t.meta.identity.guid == f.mvr.fixture_type) else { continue };
        let Some((_, mode)) = ft.mode_of(f.mvr.revision, f.mvr.mode) else { continue };
        let name = ft.meta.identity.name.clone().unwrap_or_else(|| ft.manufacturer.clone());
        let set = types.entry(name.clone()).or_default();
        for cf in &mode.channel_functions {
            if !cf.attribute_name.is_empty() && set.insert(cf.attribute_name.clone()) {
                let users = by_name.entry(cf.attribute_name.clone()).or_default();
                if !users.contains(&name) {
                    users.push(name.clone());
                }
            }
        }
    }
    let show_group = |name: &str| -> Value {
        show.attribute_defs
            .get(name)
            .and_then(|d| d.feature_group)
            .and_then(|g| show.feature_groups.values().find(|x| x.meta.identity.guid == g))
            .map_or(Value::Null, |g| json!(g.meta.identity.name))
    };
    let mut members: BTreeMap<uuid::Uuid, Vec<(u32, String)>> = BTreeMap::new();
    let mut attrs = Vec::new();
    for name in layer.names() {
        let Some(p) = layer.place_of(&name) else { continue };
        members.entry(p.group).or_default().push((p.position, name.clone()));
        let def = show.attribute_defs.get(&name);
        let (readout, resolution) = match target {
            BoardTarget::Show => (def.map(|d| json!(d.natural_readout)), def.map(|d| json!(d.encoder_resolution))),
            BoardTarget::Profile(no) => {
                let a = show.user_profiles[&no].content.attributes.placed.get(&name);
                (
                    a.and_then(|a| a.readout.clone()).or(def.map(|d| d.natural_readout.clone())).map(|r| json!(r)),
                    a.and_then(|a| a.resolution.clone()).or(def.map(|d| d.encoder_resolution.clone())).map(|r| json!(r)),
                )
            }
        };
        attrs.push(json!({
            "name": name,
            "pretty": def.map(|d| d.pretty.clone()).unwrap_or_default(),
            "new": p.new,
            "used": used.contains_key(&name),
            "how": p.placement.how.word(),
            "why": p.placement.why,
            "file": p.placement.file_feature,
            "main": def.and_then(|d| d.main_attribute.clone()),
            "feature": p.feature,
            "show_group": if matches!(target, BoardTarget::Profile(_)) { show_group(&name) } else { Value::Null },
            "types": by_name.get(&name).cloned().unwrap_or_default(),
            "readout": readout,
            "resolution": resolution,
        }));
    }
    let groups: Vec<Value> = layer
        .groups()
        .iter()
        .map(|(no, g)| {
            let mut m = members.remove(&g.meta.identity.guid).unwrap_or_default();
            m.sort();
            json!({
                "no": no,
                "name": g.meta.identity.name,
                "color": hex(g.meta.annotation.paint),
                "icon": icon_word(&g.meta),
                "members": m.into_iter().map(|(_, n)| n).collect::<Vec<_>>(),
            })
        })
        .collect();
    // A profile's encoder bar layouts (the Encoder bar tab): strip buttons in
    // bar order, each its group and its slots.
    let encoder_bars: Vec<Value> = match target {
        BoardTarget::Show => Vec::new(),
        BoardTarget::Profile(no) => show.user_profiles[&no]
            .content.attributes
            .encoder_bars
            .iter()
            .map(|(id, b)| {
                let buttons: Vec<Value> = b
                    .buttons
                    .iter()
                    .filter_map(|btn| {
                        let (gno, _) = layer.groups().iter().find(|(_, g)| g.meta.identity.guid == btn.group)?;
                        Some(json!({ "group": gno, "slots": btn.slots }))
                    })
                    .collect();
                json!({ "no": id, "name": b.meta.identity.name, "buttons": buttons })
            })
            .collect(),
    };
    let types: Vec<Value> =
        types.into_iter().map(|(name, a)| json!({ "name": name, "attrs": a.into_iter().collect::<Vec<_>>() })).collect();
    Some(json!({
        "target": match target { BoardTarget::Show => json!("show"), BoardTarget::Profile(no) => json!(no) },
        "title": title,
        "owner": owner_words(show, user, target),
        "locked": locked,
        "template": layer.template(),
        "templates": attribute_resolver::standard().template_names(),
        "groups": groups,
        "attrs": attrs,
        "types": types,
        "encoder_bars": encoder_bars,
    }))
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
