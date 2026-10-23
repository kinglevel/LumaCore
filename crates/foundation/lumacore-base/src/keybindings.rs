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
//!
//! A binding belongs to a shortcut layer (decisions 0295): every mode (`All`), or only while the
//! screen is in KBD, PRG or NAV. Within a layer the order is the same; a mode's binding wins over
//! an All binding on the same key.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Which shortcut mode a binding works in (decisions 0295): every mode, or one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum Layer {
    #[default]
    All,
    Keyboard,
    Programmer,
    Navigator,
}

impl Layer {
    /// The three modes, in the F10 order.
    pub const MODES: [Layer; 3] = [Layer::Keyboard, Layer::Programmer, Layer::Navigator];

    /// How it is written (`/Shortcuts "PRG"`) and shown.
    pub fn label(self) -> &'static str {
        match self {
            Layer::All => "All",
            Layer::Keyboard => "KBD",
            Layer::Programmer => "PRG",
            Layer::Navigator => "NAV",
        }
    }

    /// A mode by its short or long name, any case (`PRG`, `Programmer`); `All` = every mode.
    pub fn parse(word: &str) -> Option<Self> {
        match word.trim_matches('"').to_ascii_lowercase().as_str() {
            "all" => Some(Layer::All),
            "kbd" | "keyboard" => Some(Layer::Keyboard),
            "prg" | "programmer" => Some(Layer::Programmer),
            "nav" | "navigator" => Some(Layer::Navigator),
            _ => None,
        }
    }

    pub fn is_all(&self) -> bool {
        *self == Layer::All
    }
}

/// A single key binding: a key combo names a UiKey (decisions §17 — keyboard
/// shortcut → UiKey → command lines). The command lines live in the UiKey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Human-readable key name: "Ctrl-R", "F5", "Alt-S", etc.
    pub key: String,
    /// The UiKey it presses (`help`, `blind`, `user.f5`); empty = the key is turned off in its
    /// layer on this profile (a LumaCore default it doesn't want).
    #[serde(default)]
    pub ui_key: String,
    /// Short description shown in help/list.
    #[serde(default)]
    pub description: String,
    /// The shortcut mode it works in; All = every mode.
    #[serde(default, skip_serializing_if = "Layer::is_all")]
    pub layer: Layer,
}

/// Collection of key bindings, keyed by [`slot`] (the layer and the normalized key name).
pub type KeyBindingMap = BTreeMap<String, KeyBinding>;

/// A binding's place in a [`KeyBindingMap`]: the key name for All, else `"PRG S"` (a key name has
/// no space).
pub fn slot(layer: Layer, key: &str) -> String {
    match layer {
        Layer::All => key.to_string(),
        l => format!("{} {key}", l.label()),
    }
}

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

/// A LumaCore default key of one shortcut layer (decisions 0295): the key, the UiKey it presses
/// and that key's lines. A press row types its words onto the command line; a row written `!…`
/// runs at once. Several rows are a press cycle: the next press replaces the row the key just
/// typed while it is still the last thing on the line.
#[derive(Debug, Clone, Copy)]
pub struct LayerKey {
    pub layer: Layer,
    pub key: &'static str,
    /// The UiKey (`prg.e`).
    pub id: &'static str,
    pub label: &'static str,
    pub press: &'static [&'static str],
    /// With Meow held (the same rows).
    pub meow: &'static [&'static str],
    pub hold: &'static str,
    pub double: &'static str,
    /// It acts only on an empty command line; with words there it types its character (a mode
    /// key, Space = Go).
    pub empty_only: bool,
}

impl LayerKey {
    const fn prg(key: &'static str, id: &'static str, label: &'static str, press: &'static [&'static str]) -> Self {
        Self { layer: Layer::Programmer, key, id, label, press, meow: &[], hold: "", double: "", empty_only: false }
    }

    const fn meow(self, meow: &'static [&'static str]) -> Self {
        Self { meow, ..self }
    }

    const fn hold(self, hold: &'static str) -> Self {
        Self { hold, ..self }
    }

    const fn double(self, double: &'static str) -> Self {
        Self { double, ..self }
    }

    const fn empty_only(self) -> Self {
        Self { empty_only: true, ..self }
    }
}

/// The Programmer's keys (decisions 0295; the user's map of 2026-10-23, `docs/research/keys`): a
/// letter is a word, a second press of the same key cycles its word in place.
pub const PRG_KEYS: &[LayerKey] = &[
    // Objects
    LayerKey::prg("F", "prg.f", "Fixture", &["Fixture", "Selection"]).meow(&["FixtureType"]),
    LayerKey::prg("G", "prg.g", "Group", &["Group"]).meow(&["Tag"]),
    LayerKey::prg("P", "prg.p", "Preset", &["Preset", "Attribute"]).meow(&["FeatureGroup", "DataPool"]),
    LayerKey::prg("Alt-C", "prg.alt-c", "Cue", &["Cue"]).meow(&["Programmer"]),
    LayerKey::prg("Alt-S", "prg.alt-s", "Sequence", &["Sequence"]),
    // Verbs
    LayerKey::prg("A", "prg.a", "At", &["At", "At 100"]),
    LayerKey::prg("Alt-A", "prg.alt-a", "Assign", &["Assign", "Label", "Paint"]).meow(&["Set"]),
    LayerKey::prg("E", "prg.e", "Edit", &["Edit", "EditSetting"]),
    LayerKey::prg("S", "prg.s", "Store", &["Store"]).meow(&["Store /Merge"]),
    LayerKey::prg("U", "prg.u", "Update", &["Update"]).meow(&["Recast"]),
    LayerKey::prg("T", "prg.t", "Thru", &["Thru"]),
    LayerKey::prg("I", "prg.i", "If", &["If", "IfStomp"]),
    LayerKey::prg("O", "prg.o", "Off", &["Off"]),
    LayerKey::prg("L", "prg.l", "List", &["List"]),
    LayerKey::prg("Ctrl-C", "prg.ctrl-c", "Copy", &["Copy", "Paste", "Insert"]).meow(&["Cut"]),
    LayerKey::prg("Ctrl-M", "prg.ctrl-m", "Move", &["Move"]),
    LayerKey::prg("Ctrl-D", "prg.ctrl-d", "Delete", &["Delete"]),
    LayerKey::prg("Ctrl-S", "prg.ctrl-s", "Select", &["Select", "Deselect"]),
    LayerKey::prg("Ctrl-O", "prg.ctrl-o", "On", &["On", "Call"]),
    LayerKey::prg("Alt-T", "prg.alt-t", "Stomp", &["Stomp", "Unstomp"]),
    LayerKey::prg("Alt-G", "prg.alt-g", "Goto", &["Goto"]),
    LayerKey::prg("Ctrl-Alt-F", "prg.ctrl-alt-f", "SelFix", &["SelectFixtures", "Shuffle"]),
    LayerKey::prg("Ctrl-P", "prg.ctrl-p", "Pause", &["Pause", "Park"]),
    LayerKey::prg("Ctrl-G", "prg.ctrl-g", "Go", &["Go", "Unpark"]),
    // Values: the second press runs at once
    LayerKey::prg(".", "prg.dot", ".", &[".", "!Zero"]),
    LayerKey::prg("+", "prg.plus", "+", &["+", "!At + 10"]).meow(&["!Next"]),
    LayerKey::prg("-", "prg.minus", "-", &["-", "!At - 10"]).meow(&["!Previous"]),
    LayerKey::prg("Ctrl-F", "prg.ctrl-f", "Full", &["!Full"]),
    // Clear: each press one level further, held everything
    LayerKey::prg("Delete", "prg.delete", "Clear", &["!ResetSelection", "!ResetActive", "!ResetAll"]).hold("ResetAll"),
    // Modes and playback
    LayerKey::prg("H", "prg.h", "Highlight", &["!Toggle Highlight"]).empty_only(),
    LayerKey::prg("B", "prg.b", "Blind", &["!Toggle Blind"]).empty_only(),
    LayerKey::prg("Alt-F", "prg.alt-f", "Freeze", &["!Toggle Freeze"]),
    LayerKey::prg("Ctrl-Alt-S", "prg.ctrl-alt-s", "Solo", &["!Toggle Solo"]),
    LayerKey::prg("Space", "prg.space", "Go", &["!Go"]).empty_only(),
    LayerKey::prg("Alt-Space", "prg.alt-space", "Pause", &["!Pause"]),
    LayerKey::prg("Ctrl-Z", "prg.ctrl-z", "Oops", &["!Oops"]).meow(&["!Redo"]),
    LayerKey::prg("Ctrl-Alt-H", "prg.ctrl-alt-h", "Help", &["!Help"]).double("quicksheet"),
];

/// What the command line itself does with a key (decisions 0295; the terminal's and the page's
/// line editing, no UiKey): shown by the key editor so every key says what it does, never bound.
/// A mode's row is that mode's own meaning.
pub const LINE_KEYS: &[(Layer, &str, &str)] = &[
    (Layer::All, "Enter", "Run the line"),
    (Layer::All, "Esc", "Clear"),
    (Layer::All, "Tab", "Complete"),
    (Layer::All, "Backspace", "Erase"),
    (Layer::All, "Up", "History"),
    (Layer::All, "Down", "History"),
    (Layer::All, "Left", "Cursor"),
    (Layer::All, "Right", "Cursor · ghost"),
    (Layer::All, "PageUp", "Scroll"),
    (Layer::All, "PageDown", "Scroll"),
    (Layer::All, "Home", "Line start"),
    (Layer::All, "End", "Line end"),
    (Layer::All, "Ctrl-Z", "Undo typing"),
    (Layer::All, "Ctrl-Y", "Redo typing"),
    (Layer::All, "Ctrl-C", "Clear line"),
    (Layer::All, "Alt-Left", "cd back"),
    (Layer::All, "Alt-Right", "cd forward"),
    (Layer::Programmer, "Backspace", "Reset (empty line)"),
    (Layer::Programmer, "Up", "Previous (empty line)"),
    (Layer::Programmer, "Down", "Next (empty line)"),
];

/// NAV's own keys, one per key (decisions 0291; the `?` overlay's list in the terminal): shown by
/// the key editor; a profile's NAV binding wins over them. Not lines yet, so not bindable rows.
pub const NAV_MAP: &[(&str, &str)] = &[
    ("Up", "Move"),
    ("Down", "Move"),
    ("PageUp", "Page"),
    ("PageDown", "Page"),
    ("Home", "First"),
    ("End", "Last"),
    ("Right", "Open"),
    ("Left", "Close · parent"),
    ("Enter", "Open · Set line"),
    ("Tab", "Tree ⇄ inspector"),
    ("Shift-Tab", "Tree ⇄ inspector"),
    ("0", "Slot"),
    ("1", "Slot"),
    ("2", "Slot"),
    ("3", "Slot"),
    ("4", "Slot"),
    ("5", "Slot"),
    ("6", "Slot"),
    ("7", "Slot"),
    ("8", "Slot"),
    ("9", "Slot"),
    ("/", "Filter"),
    ("N", "Next hit"),
    ("Shift-N", "Previous hit"),
    ("Space", "Mark"),
    ("Y", "Copy…"),
    ("R", "Reset line"),
    ("V", "Changed only"),
    ("M", "Bookmark…"),
    ("'", "Jump…"),
    ("P", "Pin"),
    ("C", "cd here"),
    ("Delete", "Delete line"),
    ("Shift-L", "Log"),
    ("?", "Help"),
    ("Esc", "Leave NAV"),
    ("Backspace", "cd .. (held: cd /)"),
];

/// Build the vanilla (system default) key bindings: every mode's ([`VANILLA`]) and each mode's
/// own ([`PRG_KEYS`]), by [`slot`].
///
/// These are the factory defaults. They live in Root outside ShowData
/// so they don't pollute the show file.
pub fn vanilla_keybindings() -> KeyBindingMap {
    let all = VANILLA.iter().map(|&(key, ui_key, _, desc)| (Layer::All, key, ui_key, desc));
    let layers = PRG_KEYS.iter().map(|k| (k.layer, k.key, k.id, k.label));
    all.chain(layers)
        .map(|(layer, key, ui_key, desc)| {
            let key = normalize_key(key);
            (
                slot(layer, &key),
                KeyBinding {
                    key,
                    ui_key: ui_key.to_string(),
                    description: desc.to_string(),
                    layer,
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

/// The bindings a screen in `layer` uses, by key name (`S`, `Ctrl-R`): debug (if active) >
/// profile > vanilla, and within each a mode's own binding over its All one (decisions 0295).
/// A profile binding never takes a core key; one with no UiKey turns the key off.
pub fn resolve_bindings(
    vanilla: &KeyBindingMap,
    user: &KeyBindingMap,
    debug: &KeyBindingMap,
    debug_active: bool,
    layer: Layer,
) -> KeyBindingMap {
    resolve_bindings_with_source(vanilla, user, debug, debug_active, layer)
        .into_iter()
        .filter(|(_, (b, _))| !b.ui_key.is_empty())
        .map(|(k, (b, _))| (k, b))
        .collect()
}

/// Identify which layer a key binding comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingSource {
    Vanilla,
    User,
    Debug,
}

/// [`resolve_bindings`] with each binding's source, keeping a profile's turned-off keys (no
/// UiKey) so a list can show them.
pub fn resolve_bindings_with_source(
    vanilla: &KeyBindingMap,
    user: &KeyBindingMap,
    debug: &KeyBindingMap,
    debug_active: bool,
    layer: Layer,
) -> BTreeMap<String, (KeyBinding, BindingSource)> {
    let mut merged: BTreeMap<String, (KeyBinding, BindingSource)> = BTreeMap::new();
    // All first, then the mode's own: a mode's binding wins over an All one on the same key.
    let applies = |b: &KeyBinding, pass: Layer| b.layer == pass;
    let passes = if layer.is_all() { vec![Layer::All] } else { vec![Layer::All, layer] };
    let put = |map: &KeyBindingMap, source: BindingSource, merged: &mut BTreeMap<String, (KeyBinding, BindingSource)>| {
        for pass in &passes {
            for b in map.values().filter(|b| applies(b, *pass)) {
                // A profile never takes a core key (decisions §30 "12a3").
                if source != BindingSource::User || !is_core_key(&b.key) {
                    merged.insert(b.key.clone(), (b.clone(), source));
                }
            }
        }
    };
    put(vanilla, BindingSource::Vanilla, &mut merged);
    put(user, BindingSource::User, &mut merged);
    if debug_active {
        put(debug, BindingSource::Debug, &mut merged);
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
                layer: Layer::All,
            },
        );
        let debug = debug_keybindings();
        let resolved = resolve_bindings(&vanilla, &user, &debug, false, Layer::All);
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
                layer: Layer::All,
            },
        );
        let debug = debug_keybindings();

        // Debug off: user wins
        let resolved = resolve_bindings(&vanilla, &user, &debug, false, Layer::All);
        assert_eq!(resolved["Ctrl-R"].ui_key, "user.rotate");

        // Debug on with empty debug bindings: user still wins
        let resolved = resolve_bindings(&vanilla, &user, &debug, true, Layer::All);
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
                    layer: Layer::All,
                },
            );
        }
        let debug = debug_keybindings();

        let resolved = resolve_bindings_with_source(&vanilla, &user, &debug, true, Layer::All);
        assert_eq!(resolved["F1"].1, BindingSource::User);
        assert_eq!(resolved["F11"].1, BindingSource::Vanilla);
        assert_eq!(resolved["F9"].1, BindingSource::Vanilla, "a core key stays");
    }

    fn binding(layer: Layer, key: &str, ui_key: &str) -> (String, KeyBinding) {
        (slot(layer, key), KeyBinding { key: key.into(), ui_key: ui_key.into(), description: String::new(), layer })
    }

    #[test]
    fn a_mode_has_its_own_keys_and_every_modes_ones() {
        let vanilla = vanilla_keybindings();
        let none = KeyBindingMap::new();
        let prg = resolve_bindings(&vanilla, &none, &none, false, Layer::Programmer);
        assert_eq!(prg["E"].ui_key, "prg.e");
        assert_eq!(prg["F10"].ui_key, "shortcuts");
        let kbd = resolve_bindings(&vanilla, &none, &none, false, Layer::Keyboard);
        assert!(!kbd.contains_key("E"), "letters type in KBD");
        assert!(!resolve_bindings(&vanilla, &none, &none, false, Layer::All).contains_key("E"));
    }

    #[test]
    fn a_mode_binding_wins_over_all_and_an_empty_one_turns_a_key_off() {
        let vanilla = vanilla_keybindings();
        let user: KeyBindingMap = [
            binding(Layer::All, "F7", "user.f7"),
            binding(Layer::Programmer, "F7", "user.prg.f7"),
            binding(Layer::All, "Ctrl-C", "user.ctrl-c"),
            binding(Layer::Programmer, "E", ""),
            binding(Layer::Programmer, "F9", "user.prg.f9"),
        ]
        .into_iter()
        .collect();
        let none = KeyBindingMap::new();
        let prg = resolve_bindings(&vanilla, &user, &none, false, Layer::Programmer);
        assert_eq!(prg["F7"].ui_key, "user.prg.f7");
        // The profile's every-mode binding beats LumaCore's PRG default.
        assert_eq!(prg["Ctrl-C"].ui_key, "user.ctrl-c");
        assert!(!prg.contains_key("E"), "turned off on this profile");
        assert_eq!(prg["F9"].ui_key, "desklock", "a core key stays");
        assert_eq!(resolve_bindings(&vanilla, &user, &none, false, Layer::Keyboard)["F7"].ui_key, "user.f7");
        let listed = resolve_bindings_with_source(&vanilla, &user, &none, false, Layer::Programmer);
        assert_eq!(listed["E"].1, BindingSource::User);
    }

    #[test]
    fn layers_read_their_short_and_long_names() {
        assert_eq!(Layer::parse("prg"), Some(Layer::Programmer));
        assert_eq!(Layer::parse("\"Navigator\""), Some(Layer::Navigator));
        assert_eq!(Layer::parse("x"), None);
        assert_eq!(slot(Layer::Programmer, "S"), "PRG S");
        assert_eq!(slot(Layer::All, "S"), "S");
    }

    #[test]
    fn every_vanilla_binding_names_a_ui_key() {
        for (key, ui_key, cmd, _) in VANILLA {
            assert!(!ui_key.is_empty() && !cmd.is_empty(), "{key}");
        }
    }
}
