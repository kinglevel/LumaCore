//! Key binding data model for LumaCore.
//!
//! Three-layer priority system (highest to lowest):
//!   1. **Debug binds** — active only in debug mode, override everything
//!   2. **User binds**  — per-user overrides stored in UserProfile (ShowData)
//!   3. **Vanilla binds** — factory defaults in Root (outside ShowData)
//!
//! Key names are cross-platform strings:
//!   "Ctrl-R", "Ctrl-Shift-S", "Alt-F", "F5", "F12"
//!
//! A binding names a UiKey; the UiKey holds the command lines (one per
//! gesture, plus Meow lines) — decisions §17.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single key binding: a key combo names a UiKey (decisions §17 — keyboard
/// shortcut → UiKey → command lines). The command lines live in the UiKey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Human-readable key name: "Ctrl-R", "F5", "Alt-S", etc.
    pub key: String,
    /// The UiKey it presses (`help`, `blind`, `user.f5`).
    #[serde(default)]
    pub ui_key: String,
    /// Short description shown in help/list.
    #[serde(default)]
    pub description: String,
}

/// Collection of key bindings. BTreeMap keyed by normalized key name for fast lookup.
pub type KeyBindingMap = BTreeMap<String, KeyBinding>;

/// Normalize a key name to canonical form for matching.
///
/// "ctrl-r" -> "Ctrl-R", "CTRL-SHIFT-S" -> "Ctrl-Shift-S", "f5" -> "F5"
pub fn normalize_key(key: &str) -> String {
    key.split('-')
        .map(|part| {
            let lower = part.to_lowercase();
            match lower.as_str() {
                "ctrl" => "Ctrl".to_string(),
                "alt" => "Alt".to_string(),
                "shift" => "Shift".to_string(),
                other => {
                    let mut c = other.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

/// The factory bindings: (key, UiKey id, command line, description). Each
/// UiKey is a locked core key (`lumacore_tree::builtin_ui_keys`).
pub const VANILLA: &[(&str, &str, &str, &str)] = &[
    // Screen (terminal conventions)
    ("Ctrl-L", "cls", "cls", "Clear screen"),
    ("Ctrl-H", "help", "help", "Show help"),
    // Core keys F9–F12 (decisions §30 "12a3"; F1–F8 stay free
    // for executors and user keys). Meow lines: `ui_keys::CORE_MEOW`.
    ("F9", "desklock", "DeskLock", "DeskLock — Meow: normal theme"),
    ("Pause", "desklock", "DeskLock", "DeskLock — Meow: normal theme"),
    ("F10", "shortcuts", "shortcuts", "Shortcut mode KBD/PRG/NAV — Meow: view bar"),
    // Print Screen / F11 take screenshots (decisions §29 "11d").
    ("F11", "screenshot", "Screenshot", "Screenshot of my screens — Meow: Shutdown menu"),
    ("PrintScreen", "screenshot", "Screenshot", "Screenshot of my screens"),
    ("F12", "sidebars", "Sidebars", "Sidebars — Meow: full screen"),
];

/// Keys `Bind` can never take, with any modifier (decisions §30 "12a3":
/// "Bind may add keys, never take these").
pub const CORE_KEYS: &[&str] = &["F9", "F10", "F11", "F12", "Pause", "PrintScreen"];

/// Whether a normalized key name (`Alt-F9`) is a core key.
pub fn is_core_key(name: &str) -> bool {
    let base = name.rsplit('-').next().unwrap_or(name);
    CORE_KEYS.iter().any(|k| k.eq_ignore_ascii_case(base))
}

/// Build the vanilla (system default) key bindings.
///
/// These are the factory defaults. They live in Root outside ShowData
/// so they don't pollute the show file.
pub fn vanilla_keybindings() -> KeyBindingMap {
    VANILLA
        .iter()
        .map(|&(key, ui_key, _, desc)| {
            let key = normalize_key(key);
            (
                key.clone(),
                KeyBinding {
                    key,
                    ui_key: ui_key.to_string(),
                    description: desc.to_string(),
                },
            )
        })
        .collect()
}

/// Build the debug-only key bindings.
///
/// These override everything when debug mode is active.
/// Kept separate so they don't leak into production workflows.
pub fn debug_keybindings() -> KeyBindingMap {
    KeyBindingMap::new()
}

/// Resolve effective bindings with three-layer priority.
///
/// Priority (highest first): debug (if active) > user > vanilla.
pub fn resolve_bindings(
    vanilla: &KeyBindingMap,
    user: &KeyBindingMap,
    debug: &KeyBindingMap,
    debug_active: bool,
) -> KeyBindingMap {
    let mut merged = vanilla.clone();

    // Layer 2: user overrides vanilla — never a core key.
    for (key, binding) in user {
        if !is_core_key(key) {
            merged.insert(key.clone(), binding.clone());
        }
    }

    // Layer 1: debug overrides everything (only when active)
    if debug_active {
        for (key, binding) in debug {
            merged.insert(key.clone(), binding.clone());
        }
    }

    merged
}

/// Identify which layer a key binding comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingSource {
    Vanilla,
    User,
    Debug,
}

/// Resolve bindings and return each with its source layer.
pub fn resolve_bindings_with_source(
    vanilla: &KeyBindingMap,
    user: &KeyBindingMap,
    debug: &KeyBindingMap,
    debug_active: bool,
) -> BTreeMap<String, (KeyBinding, BindingSource)> {
    let mut merged = BTreeMap::new();

    // Layer 3: vanilla (base)
    for (key, binding) in vanilla {
        merged.insert(key.clone(), (binding.clone(), BindingSource::Vanilla));
    }

    // Layer 2: user overrides — never a core key.
    for (key, binding) in user {
        if !is_core_key(key) {
            merged.insert(key.clone(), (binding.clone(), BindingSource::User));
        }
    }

    // Layer 1: debug overrides (only when active)
    if debug_active {
        for (key, binding) in debug {
            merged.insert(key.clone(), (binding.clone(), BindingSource::Debug));
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_key_cases() {
        assert_eq!(normalize_key("ctrl-r"), "Ctrl-R");
        assert_eq!(normalize_key("CTRL-SHIFT-S"), "Ctrl-Shift-S");
        assert_eq!(normalize_key("f5"), "F5");
        assert_eq!(normalize_key("alt-f"), "Alt-F");
        assert_eq!(normalize_key("Ctrl-R"), "Ctrl-R");
    }

    #[test]
    fn vanilla_has_defaults() {
        let vanilla = vanilla_keybindings();
        // Core keys F9–F12 (decisions §30 "12a3"); F1–F8 stay free.
        assert_eq!(vanilla["F9"].ui_key, "desklock");
        assert_eq!(vanilla["Pause"].ui_key, "desklock");
        assert_eq!(vanilla["F12"].ui_key, "sidebars");
        assert!(!vanilla.contains_key("F1"));
    }

    #[test]
    fn debug_bindings_empty() {
        let debug = debug_keybindings();
        assert!(debug.is_empty());
    }

    #[test]
    fn user_overrides_vanilla() {
        let vanilla = vanilla_keybindings();
        let mut user = KeyBindingMap::new();
        user.insert(
            "F1".to_string(),
            KeyBinding {
                key: "F1".to_string(),
                ui_key: "user.f1".to_string(),
                description: "Custom F1".to_string(),
            },
        );
        let debug = debug_keybindings();
        let resolved = resolve_bindings(&vanilla, &user, &debug, false);
        assert_eq!(resolved["F1"].ui_key, "user.f1");
    }

    #[test]
    fn debug_overrides_user() {
        let vanilla = vanilla_keybindings();
        let mut user = KeyBindingMap::new();
        user.insert(
            "Ctrl-R".to_string(),
            KeyBinding {
                key: "Ctrl-R".to_string(),
                ui_key: "user.rotate".to_string(),
                description: "Rotate fixture".to_string(),
            },
        );
        let debug = debug_keybindings();

        // Debug off: user wins
        let resolved = resolve_bindings(&vanilla, &user, &debug, false);
        assert_eq!(resolved["Ctrl-R"].ui_key, "user.rotate");

        // Debug on with empty debug bindings: user still wins
        let resolved = resolve_bindings(&vanilla, &user, &debug, true);
        assert_eq!(resolved["Ctrl-R"].ui_key, "user.rotate");
    }

    #[test]
    fn source_tracking() {
        let vanilla = vanilla_keybindings();
        let mut user = KeyBindingMap::new();
        for key in ["F1", "F9"] {
            user.insert(
                key.to_string(),
                KeyBinding {
                    key: key.to_string(),
                    ui_key: "user.custom".to_string(),
                    description: "".to_string(),
                },
            );
        }
        let debug = debug_keybindings();

        let resolved = resolve_bindings_with_source(&vanilla, &user, &debug, true);
        assert_eq!(resolved["F1"].1, BindingSource::User);
        assert_eq!(resolved["F11"].1, BindingSource::Vanilla);
        assert_eq!(resolved["F9"].1, BindingSource::Vanilla, "a core key stays");
    }

    #[test]
    fn every_vanilla_binding_names_a_ui_key() {
        for (key, ui_key, cmd, _) in VANILLA {
            assert!(!ui_key.is_empty() && !cmd.is_empty(), "{key}");
        }
    }
}
