// ██╗  ██╗███████╗██╗   ██╗███████╗
// ██║ ██╔╝██╔════╝╚██╗ ██╔╝██╔════╝
// █████╔╝ █████╗   ╚████╔╝ ███████╗
// ██╔═██╗ ██╔══╝    ╚██╔╝  ╚════██║
// ██║  ██╗███████╗   ██║   ███████║
// ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝
// FILE: keys

//! The key editor's data (decisions 0295), for the `keys` feed: the viewer's keyboard in each
//! shortcut layer (All, KBD, PRG, NAV) — every bound key with where it comes from (core,
//! LumaCore, the profile, turned off), the UiKey it presses and its lines (the press cycle and
//! which presses type, the Meow ones, the other gestures) — and what the command line and NAV do
//! with a key nobody binds. The lines a gesture sends come from here too.
//!
//! Owns:          view (the `keys` feed).
//! Doesn't own:   the bindings and their order (lumacore_base::keybindings), the keys (lumacore_tree::ui_keys).
//! Entry points:  view.
//! Decisions:     docs/decisions/0295-keys-in-shortcut-layers-and-press-cycles.

use lumacore_tree::keybindings::{BindingSource, KeyBindingMap, LINE_KEYS, Layer, NAV_MAP, is_core_key, resolve_bindings_with_source, vanilla_keybindings};
use lumacore_tree::ui_keys::{Gesture, UiKey, UiKeyOwner, builtin_ui_keys, resolve};
use lumacore_tree::ShowData;
use serde_json::{Value, json};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// A UiKey's lines: `press` / `meow` the cycles (`[{line, types}]`), the other gestures by name.
fn lines(k: &UiKey) -> Value {
    let rows = |meow: bool| -> Vec<Value> {
        if meow && k.meow.press.is_empty() {
            return Vec::new();
        }
        k.press_rows(meow).into_iter().map(|r| json!({ "line": r.line, "types": r.types })).collect()
    };
    let mut out = json!({ "press": rows(false), "meow": rows(true) });
    for g in Gesture::ALL.into_iter().filter(|g| *g != Gesture::Press) {
        out[g.label()] = json!(k.command(g));
        out[format!("Meow{}", g.label())] = json!(k.meow.get(g));
    }
    out
}

/// `{profile, name, meow, layers: [{layer, keys: [{key, from, source, ui_key, text, locked,
/// empty_only, lines}]}], lines: {bind, mode, unbind, set}}` — `source`: core, lumacore, profile,
/// off (LumaCore's turned off on the profile), line (the command line's own), nav (NAV's own).
pub fn view(show: &ShowData, user: &str) -> Value {
    let no = show.profile_id_for_user(user);
    let profile = no.and_then(|n| show.user_profiles.get(&n));
    let none = KeyBindingMap::new();
    let mine = profile.map_or(&none, |p| &p.content.keybindings);
    let vanilla = vanilla_keybindings();
    let core = builtin_ui_keys();
    let plugins: Vec<&UiKey> = show.datapools.values().flat_map(|dp| dp.plugins.values()).flat_map(|p| p.buttons.iter()).collect();
    let find = |id: &str| resolve(id, &core, plugins.iter().copied(), profile.map(|p| &p.content.ui_keys));
    let layers: Vec<Value> = [Layer::All, Layer::Keyboard, Layer::Programmer, Layer::Navigator]
        .into_iter()
        .map(|layer| {
            let mut keys: Vec<Value> = resolve_bindings_with_source(&vanilla, mine, &none, false, layer)
                .into_values()
                .map(|(b, source)| {
                    let key = find(&b.ui_key);
                    let source = match source {
                        _ if is_core_key(&b.key) => "core",
                        BindingSource::User if b.ui_key.is_empty() => "off",
                        BindingSource::User => "profile",
                        _ => "lumacore",
                    };
                    json!({
                        "key": b.key,
                        "from": b.layer.label(),
                        "source": source,
                        "ui_key": b.ui_key,
                        "text": key.and_then(|k| k.meta.name().map(str::to_string)).filter(|_| b.description.is_empty()).unwrap_or(b.description.clone()),
                        "locked": key.is_none_or(|k| k.locked || k.owner != UiKeyOwner::User),
                        "empty_only": key.is_some_and(|k| k.empty_only),
                        "lines": key.map_or(Value::Null, lines),
                    })
                })
                .collect();
            let taken = |keys: &[Value], name: &str| keys.iter().any(|k| k["key"] == name);
            // What a key nobody binds does: the command line's own, NAV's own.
            // A mode's own line-editing row wins over the every-mode one for the same key.
            let own = LINE_KEYS
                .iter()
                .filter(|(l, k, _)| *l == layer || (l.is_all() && !LINE_KEYS.iter().any(|(m, k2, _)| *m == layer && k2 == k)))
                .map(|(_, k, t)| (*k, *t, "line"))
                .chain(NAV_MAP.iter().filter(|_| layer == Layer::Navigator).map(|(k, t)| (*k, *t, "nav")))
                .collect::<Vec<_>>();
            for (k, text, source) in own {
                // NAV's own keys come before the command line's in NAV.
                if !taken(&keys, k) && !(source == "line" && layer == Layer::Navigator && NAV_MAP.iter().any(|(n, _)| *n == k)) {
                    keys.push(json!({ "key": k, "from": layer.label(), "source": source, "text": text, "locked": true }));
                }
            }
            // Your own key in this layer is `<own><key in lower case>` (`Bind` makes it).
            json!({ "layer": layer.label(), "own": lumacore_tree::UserProfile::user_key_id(layer, ""), "keys": keys })
        })
        .collect();
    json!({
        "profile": no,
        "name": profile.and_then(|p| p.meta.name()).unwrap_or_default(),
        "meow": profile.map_or("Shift".to_string(), |p| p.settings.preferences.meow_key.clone()),
        "layers": layers,
        // LumaCore's own keys, with any modifier (E-KEY-005).
        "core": lumacore_tree::keybindings::CORE_KEYS,
        "lines": {
            "bind": "Bind \"{key}\" {lines}{mode}",
            "mode": " /Shortcuts \"{layer}\"",
            "unbind": "Unbind \"{key}\"{mode}",
            "set": "Set UIKey \"{id}\" \"{prop}\" {value}",
        },
    })
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
