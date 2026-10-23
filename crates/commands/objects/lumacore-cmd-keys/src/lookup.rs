// ██╗      ██████╗  ██████╗ ██╗  ██╗██╗   ██╗██████╗
// ██║     ██╔═══██╗██╔═══██╗██║ ██╔╝██║   ██║██╔══██╗
// ██║     ██║   ██║██║   ██║█████╔╝ ██║   ██║██████╔╝
// ██║     ██║   ██║██║   ██║██╔═██╗ ██║   ██║██╔═══╝
// ███████╗╚██████╔╝╚██████╔╝██║  ██╗╚██████╔╝██║
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝
// FILE: lookup

//! Finding keys (decisions §17): core keys, plugin buttons and the profile's own, in that order;
//! a binding's layers debug > profile > vanilla.
//!
//! Owns:          which keys this user has, a key by id, this user's bindings, a gesture's line.
//! Doesn't own:   the key data (lumacore-tree ui_keys, keybindings), what a key runs (the terminal, the page).
//! Entry points:  all_keys, find, bindings, bindings_in, command_for.
//! Decisions:     §17.

use lumacore_tree::keybindings::Layer;
use lumacore_tree::resolve_bindings;
use lumacore_tree::ui_keys::{Gesture, UiKey, resolve};

use lumacore_objects::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Every key this user can use: core keys, plugin buttons, the profile's keys.
pub fn all_keys(state: &AppState) -> Vec<UiKey> {
    let mut keys: Vec<UiKey> = state.root.resources.ui_keys.values().cloned().collect();
    for dp in state.root.show_data.datapools.values() {
        for p in dp.plugins.values() {
            keys.extend(p.buttons.iter().cloned());
        }
    }
    if let Some(p) = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
    {
        keys.extend(p.content.ui_keys.values().cloned());
    }
    keys
}

/// The key `id` as this user sees it (core wins over plugin over profile).
pub fn find(state: &AppState, id: &str) -> Option<UiKey> {
    let plugins = state
        .root
        .show_data
        .datapools
        .values()
        .flat_map(|dp| dp.plugins.values())
        .flat_map(|p| p.buttons.iter());
    let profile = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| &p.content.ui_keys);
    resolve(id, &state.root.resources.ui_keys, plugins, profile).cloned()
}

/// The bindings this user's keys have in the screen's shortcut mode: debug > profile > vanilla,
/// a mode's own over its every-mode ones (§17 layers, decisions 0295).
pub fn bindings(state: &AppState) -> lumacore_tree::KeyBindingMap {
    bindings_in(state, state.shortcut_mode.layer())
}

/// The bindings in one layer (`Layer::All` = only the every-mode ones).
pub fn bindings_in(state: &AppState, layer: Layer) -> lumacore_tree::KeyBindingMap {
    let empty = lumacore_tree::KeyBindingMap::new();
    let user = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map_or(&empty, |p| &p.content.keybindings);
    resolve_bindings(
        &state.root.default_keybindings,
        user,
        &state.root.debug_keybindings,
        state.debug_mode,
        layer,
    )
}

/// The command line a gesture of key `id` runs (Meow held or not). A
/// DoublePress without its own line counts as a second press.
pub fn command_for(key: &UiKey, gesture: Gesture, meow: bool) -> (Gesture, String) {
    match key.command_for(gesture, meow) {
        "" if gesture == Gesture::Double => (
            Gesture::Press,
            key.command_for(Gesture::Press, meow).to_string(),
        ),
        line => (gesture, line.to_string()),
    }
}
