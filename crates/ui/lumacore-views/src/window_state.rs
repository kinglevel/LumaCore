// ██╗    ██╗██╗███╗   ██╗██████╗  ██████╗ ██╗    ██╗    ███████╗████████╗ █████╗ ████████╗███████╗
// ██║    ██║██║████╗  ██║██╔══██╗██╔═══██╗██║    ██║    ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██╔════╝
// ██║ █╗ ██║██║██╔██╗ ██║██║  ██║██║   ██║██║ █╗ ██║    ███████╗   ██║   ███████║   ██║   █████╗
// ██║███╗██║██║██║╚██╗██║██║  ██║██║   ██║██║███╗██║    ╚════██║   ██║   ██╔══██║   ██║   ██╔══╝
// ╚███╔███╔╝██║██║ ╚████║██████╔╝╚██████╔╝╚███╔███╔╝    ███████║   ██║   ██║  ██║   ██║   ███████╗
//  ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═════╝  ╚═════╝  ╚══╝╚══╝     ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: window state

//! The live state of windows (decisions §30 "12c": one state rule through
//! `Self.<Property>`): each window's title, bar item states, chips and key
//! LEDs, resolved on the station and pushed to clients on the `windows` feed
//! of `/ws/feeds` (a State feed: one batch on sub, then changes).
//!
//! The rules live in `lumacore_tree::window_types` (`WindowType::view`,
//! `eval_state`); this module gives them the station's reads: link names
//! from the show, `Self.Errors` from the System Monitor. Macros will read
//! `Self.<Property>` through the same resolver later.
//!
//! Owns:          `StationSelf` (the station's reads for `Self.<Property>`), `window_view`, `screen_view` (the `windows` feed).
//! Doesn't own:   window types and their rules (`lumacore_tree::window_types`), window settings (`lumacore_objects::commands::windows`).
//! Entry points:  StationSelf, window_view, screen_view.
//! Decisions:     §30 "12c", §34 20.7.

use std::collections::BTreeMap;

use lumacore_tree::ShowData;
use lumacore_tree::runtime::{Screen, Window};
use lumacore_tree::window_types::{SelfContext, WindowType, WindowTypeView, eval_state, same_key};
use serde_json::{Map, Value, json};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The station's reads for `Self.<Property>` of one user's windows.
pub struct StationSelf<'a> {
    pub show: &'a ShowData,
    pub user: &'a str,
}

impl SelfContext for StationSelf<'_> {
    fn link(&self, kind: &str, value: &Value) -> Option<(u32, String)> {
        // tidy: not-keywords (the link kinds a window setting names)
        match kind {
            "DataPool" => {
                let no = lumacore_objects::pools::resolve_datapool(self.show, self.user, value)?;
                let dp = self.show.datapools.get(&no)?;
                Some((no, dp.meta.identity.name.clone().unwrap_or_default()))
            }
            "Sequence" => {
                let (_, no, seq) = crate::sequence::resolve(self.show, self.user, value)?;
                Some((no, seq.meta.identity.name.clone().unwrap_or_else(|| format!("Sequence {no}"))))
            }
            "FeatureGroup" => {
                if value.as_str() == Some("all") {
                    return Some((0, "All".into()));
                }
                let g: uuid::Uuid = value.as_str()?.parse().ok()?;
                let (no, fg) = self.show.feature_groups.iter().find(|(_, f)| f.meta.identity.guid == g)?;
                Some((*no, fg.meta.identity.name.clone().unwrap_or_default()))
            }
            "UserProfile" => {
                let no = lumacore_objects::pools::resolve_profile(self.show, self.user, value)?;
                let p = self.show.user_profiles.get(&no)?;
                Some((no, p.meta.identity.name.clone().unwrap_or_default()))
            }
            "Universe" => {
                let no = value.as_u64().or_else(|| value.as_str()?.parse().ok())? as u32;
                let name = self
                    .show
                    .devices
                    .dmx
                    .patch
                    .universes
                    .get(&no)
                    .and_then(|u| u.meta.identity.name.clone())
                    .unwrap_or_default();
                Some((no, name))
            }
            _ => None,
        }
    }

    fn computed(&self, ty: &WindowType, settings: &Map<String, Value>, name: &str) -> Option<Value> {
        // A System Monitor's errors since its Clear mark (`Since`).
        if ty.name == "sysmon" && same_key(name, "Errors") {
            let since = settings.get("since").and_then(Value::as_u64).unwrap_or(0);
            return Some(json!(lumacore_sysmon::errors_since(since)));
        }
        None
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The UiKeys whose LED reads `Self` (`state: "Self.AutoScroll"`): the
/// user's profile's keys and the plugins' (decisions §17, §30 "12c").
fn self_keys<'a>(show: &'a ShowData, user: &str) -> Vec<&'a lumacore_tree::UiKey> {
    let reads_self = |k: &&lumacore_tree::UiKey| {
        let s = k.state.trim_start_matches('!');
        s.starts_with("Self.") || s.starts_with("{Self}.")
    };
    let mut keys: Vec<&lumacore_tree::UiKey> = show
        .datapools
        .values()
        .flat_map(|dp| dp.plugins.values().flat_map(|p| p.buttons.iter()))
        .filter(reads_self)
        .collect();
    if let Some(p) = show.profile_for_user(user) {
        keys.extend(p.content.ui_keys.values().filter(reads_self));
    }
    keys
}

/// One window as its clients see it, or None for a type the station
/// doesn't know (and the empty placeholder).
pub fn window_view(
    show: &ShowData,
    user: &str,
    types: &BTreeMap<u32, WindowType>,
    win: &Window,
    keys: &[&lumacore_tree::UiKey],
) -> Option<Value> {
    let ty = types.values().find(|t| t.name == win.kind && !t.hidden)?;
    let ctx = StationSelf { show, user };
    let view = ty.view(&win.settings, &ctx);
    let settings = ty.resolve_settings(&win.settings);
    let key_states: BTreeMap<&str, bool> = keys
        .iter()
        .map(|k| (k.id.as_str(), eval_state(ty, &settings, &k.state, &ctx)))
        .collect();
    let mut v = serde_json::to_value(&view).ok()?;
    v["keys"] = json!(key_states);
    v["preset"] = json!(win.preset.map(|g| g.to_string()));
    // The pick lists of the links pinned in the bar (`DataPool: Selected ▾`):
    // the bar draws them with the kit's pick list; the choice is a `Set` line.
    let picks: Map<String, Value> = ty
        .settings
        .iter()
        .filter(|d| view.items.iter().any(|i| same_key(i, &d.key)))
        .filter_map(|d| match &d.kind {
            lumacore_tree::window_types::SettingKind::Link { kind, follow, by_guid } => {
                let current = settings.get(&d.key).cloned().unwrap_or(Value::Null);
                let pick = lumacore_objects::commands::windows::link_pick(show, kind, follow.clone(), *by_guid, &current);
                Some((d.key.clone(), serde_json::to_value(pick).ok()?))
            }
            _ => None,
        })
        .collect();
    v["picks"] = Value::Object(picks);
    Some(v)
}

/// A live screen's windows (`windows` feed data): `{screen, maximized,
/// windows: {no: view}}` — `maximized` the window held full screen on the
/// screen (decisions §30 "12c2"), null when none.
pub fn screen_view(show: &ShowData, user: &str, screen: &Screen) -> Value {
    let keys = self_keys(show, user);
    let types = lumacore_registry::get().window_types();
    let windows: Map<String, Value> = screen
        .windows
        .iter()
        .filter_map(|(no, w)| Some((no.to_string(), window_view(show, user, types, w, &keys)?)))
        .collect();
    json!({ "screen": screen.no, "maximized": screen.maximized, "windows": windows })
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
