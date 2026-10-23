// ██████╗ ███████╗████████╗ █████╗ ██╗██╗
// ██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██║██║
// ██║  ██║█████╗     ██║   ███████║██║██║
// ██║  ██║██╔══╝     ██║   ██╔══██║██║██║
// ██████╔╝███████╗   ██║   ██║  ██║██║███████╗
// ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚══════╝
// FILE: detail

//! The rows of a Users & Profiles detail (decisions §30 "Users & Profiles
//! follows storage"): what a profile's Content holds and what a user's work
//! is, as the `detail` feed sends them to `<luma-detail>`. Pure: it reads a
//! published show.
//!
//! Owns:          `view` (a profile's Content lists, a user's Work and Usage).
//! Doesn't own:   the detail's shape (`menus`, `<luma-detail>`), the feed (`lumacore-http`).
//! Entry points:  view.
//! Decisions:     §30 "Users & Profiles follows storage", §34 20.7.

use lumacore_tree::ShowData;
use serde_json::{Value, json};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// A profile's Content lists (Keys, Window presets, Images, Icons, Screens)
/// or a user's Work and Usage; None when the object is gone.
pub fn view(show: &ShowData, kind: &str, no: u32) -> Option<Value> {
    match kind {
        "profile" => profile(show, no),
        "user" => user(show, no),
        _ => None,
    }
}

fn profile(show: &ShowData, no: u32) -> Option<Value> {
    let p = show.user_profiles.get(&no)?;
    let c = &p.content;
    let keys: Vec<Value> = c
        .keybindings
        .values()
        .map(|k| json!({ "key": k.key, "ui_key": k.ui_key, "description": k.description }))
        .collect();
    let mut window_presets = Vec::new();
    for (kind, pool) in &c.window_presets {
        for (no, w) in pool {
            window_presets.push(json!({ "guid": w.meta.identity.guid.to_string(), "type": kind, "no": no, "name": w.meta.identity.name }));
        }
    }
    let media = |m: &std::collections::BTreeMap<u32, lumacore_tree::media::MediaFile>| -> Vec<Value> {
        m.iter()
            .map(|(no, f)| json!({ "guid": f.meta.identity.guid.to_string(), "no": no, "name": f.meta.identity.name, "ext": f.ext, "size": f.size, "dims": format!("{} × {}", f.width, f.height) }))
            .collect()
    };
    let mut screens: Vec<Value> = c
        .screen_configs
        .iter()
        .map(|(no, s)| json!({ "guid": s.meta.identity.guid.to_string(), "kind": "ScreenConfig", "no": no, "name": s.meta.identity.name, "screens": s.screens.len() }))
        .collect();
    screens.extend(
        c.views
            .iter()
            .map(|(no, v)| json!({ "guid": v.meta.identity.guid.to_string(), "kind": "View", "no": no, "name": v.meta.identity.name, "screens": v.screens.len() })),
    );
    Some(json!({
        "kind": "profile",
        "no": no,
        "name": p.meta.identity.name.clone().unwrap_or_default(),
        "locked": p.locked,
        "keys": keys,
        "window_presets": window_presets,
        "images": media(&c.images),
        "icons": media(&c.icons),
        "screens": screens,
        "waiting": c.attributes.waiting().len(),
        "encoder_bars": c.attributes.encoder_bars.len(),
    }))
}

fn user(show: &ShowData, no: u32) -> Option<Value> {
    let u = show.users.get(&no)?;
    // The selected objects by their tree path (a walk only when one is set).
    let sel = &u.selected;
    let walked = (sel.datapool.is_some() || sel.sequence.is_some() || sel.timecode.is_some()).then(|| show.walk_identity());
    let name_of = |guid: Option<uuid::Uuid>| {
        guid.and_then(|g| walked.as_ref()?.objects.iter().find(|o| o.guid == g).map(|o| o.path.clone()))
            .unwrap_or_else(|| "—".into())
    };
    let mut usage: Vec<(&String, u32)> = u.usage.entries.iter().map(|(l, e)| (l, e.count)).collect();
    usage.sort_by_key(|u| std::cmp::Reverse(u.1));
    Some(json!({
        "kind": "user",
        "no": no,
        "name": u.login,
        "profile": show.user_profiles.iter().find(|(_, p)| p.meta.identity.guid == u.profile).map(|(n, p)| json!({ "no": n, "name": p.meta.identity.name })),
        "work": [
            { "what": "Programmer", "value": format!("{} values", u.programmer.values.len()) },
            { "what": "Selection", "value": format!("{} fixtures", u.programmer.selected_fixtures.len()) },
            { "what": "DataPool", "value": name_of(u.selected.datapool) },
            { "what": "Sequence", "value": name_of(u.selected.sequence) },
            { "what": "Timecode", "value": name_of(u.selected.timecode) },
        ],
        "usage": usage.iter().take(50).map(|(l, n)| json!({ "line": l, "count": n })).collect::<Vec<_>>(),
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
