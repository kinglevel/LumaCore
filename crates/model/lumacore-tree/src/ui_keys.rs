//! UI keys: any button of the interface with a command line per gesture —
//! a `VirtualKey` + `VKValue` (decisions §17). Gestures: Press, Release,
//! Hold, ReleaseAfterHold, DoublePress; with the **Meow** modifier held a key
//! runs its Meow command lines instead (falling back to the plain ones).
//! Keyboard bindings only name a UiKey (`KeyBinding::ui_key`).
//!
//! Three kinds, resolved by id:
//! - **core** — built in, published read-only at `Root.Resources.UIKeys`,
//!   fully locked (users can't change or break them);
//! - **plugin** — declared in a plugin's manifest (`plugin.toml`), stored with
//!   the plugin in the show (`DataPool.Plugins`): global, locked;
//!   id `plugin.<plugin-id>.<button-id>`;
//! - **user** — on a UserProfile (`UserProfile.ui_keys`): everyone on that
//!   profile sees, uses and edits them; id `user.<name>`.
//!
//! UI engines only report a gesture (and Meow) for a key id; the server runs
//! the resolved command. Actions are commands, never UI code.

use std::collections::BTreeMap;

use lumacore_base::NodeMeta;
use lumacore_registry::KeyDecl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UiKeyOwner {
    #[default]
    Core,
    Plugin,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gesture {
    Press,
    Release,
    /// Held past the profile's hold time.
    Hold,
    /// Released after a Hold fired.
    ReleaseAfterHold,
    Double,
}

impl Gesture {
    pub const ALL: [Gesture; 5] = [
        Self::Press,
        Self::Release,
        Self::Hold,
        Self::ReleaseAfterHold,
        Self::Double,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().replace(['_', '-'], "").as_str() {
            "press" => Some(Self::Press),
            "release" => Some(Self::Release),
            "hold" => Some(Self::Hold),
            "releaseafterhold" => Some(Self::ReleaseAfterHold),
            "double" | "doublepress" => Some(Self::Double),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Press => "Press",
            Self::Release => "Release",
            Self::Hold => "Hold",
            Self::ReleaseAfterHold => "ReleaseAfterHold",
            Self::Double => "DoublePress",
        }
    }
}

/// A command line per gesture (empty = nothing).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyCommands {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub press: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub release: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hold: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub release_after_hold: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub double_press: String,
}

impl KeyCommands {
    pub fn get(&self, gesture: Gesture) -> &str {
        match gesture {
            Gesture::Press => &self.press,
            Gesture::Release => &self.release,
            Gesture::Hold => &self.hold,
            Gesture::ReleaseAfterHold => &self.release_after_hold,
            Gesture::Double => &self.double_press,
        }
    }

    pub fn get_mut(&mut self, gesture: Gesture) -> &mut String {
        match gesture {
            Gesture::Press => &mut self.press,
            Gesture::Release => &mut self.release,
            Gesture::Hold => &mut self.hold,
            Gesture::ReleaseAfterHold => &mut self.release_after_hold,
            Gesture::Double => &mut self.double_press,
        }
    }

    pub fn is_empty(&self) -> bool {
        Gesture::ALL.iter().all(|g| self.get(*g).is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiKey {
    pub meta: NodeMeta,
    pub id: String,
    /// A glyph, or a file of the plugin (`icons/strobe.svg`).
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub tooltip: String,
    #[serde(default)]
    pub owner: UiKeyOwner,
    /// Commands per gesture; empty = nothing.
    #[serde(default)]
    pub press: String,
    #[serde(default)]
    pub double_press: String,
    #[serde(default)]
    pub hold: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub release: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub release_after_hold: String,
    /// What the key runs while Meow is held.
    #[serde(default, skip_serializing_if = "KeyCommands::is_empty")]
    pub meow: KeyCommands,
    /// The double press stands in for the press: the UI undoes what the
    /// first press opened (Menu closes again before Quick Save).
    #[serde(default)]
    pub double_replaces_press: bool,
    /// Live state for the button's LED (key LEDs): a tree path such as
    /// `Runtime.Users.{User}.Modes.Blind` — lit while that mode is on — or a
    /// read of the window the key acts on (decisions §30 "12c"):
    /// `Self.AutoScroll` / `{Self}.AutoScroll`, `!Self.X`, `Self.View=input`
    /// (the station resolves it per window: the `windows` feed's `keys`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub state: String,
    /// Can't be changed by users (core and plugin keys).
    #[serde(default)]
    pub locked: bool,
    /// A press cycle (decisions 0295): the presses after the first, each the next row. The next
    /// press replaces the row the key just typed while it is still the last on the line.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cycle: Vec<String>,
    /// The presses (1 = the first) that type their words onto the command line; the others run,
    /// as every key's press always did.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<u8>,
    /// The Meow press's cycle and the Meow presses that type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meow_cycle: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meow_types: Vec<u8>,
    /// It acts only on an empty command line; with words there it types its character (a mode
    /// key, Space = Go in PRG).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub empty_only: bool,
}

/// One press of a key's press cycle (decisions 0295).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PressRow {
    pub line: String,
    /// It types its words onto the command line instead of running.
    pub types: bool,
}

impl UiKey {
    /// A catalog key as the show tree's locked core key.
    pub fn from_decl(d: &KeyDecl) -> Self {
        Self {
            press: d.press.clone(),
            double_press: d.double_press.clone(),
            hold: d.hold.clone(),
            release: d.release.clone(),
            release_after_hold: d.release_after_hold.clone(),
            meow: KeyCommands { press: d.meow_press.clone(), release_after_hold: d.meow_release_after_hold.clone(), ..KeyCommands::default() },
            double_replaces_press: d.double_replaces_press,
            state: d.state.clone(),
            cycle: d.cycle.clone(),
            types: d.types.clone(),
            meow_cycle: d.meow_cycle.clone(),
            meow_types: d.meow_types.clone(),
            empty_only: d.empty_only,
            ..Self::core(&d.id, &d.label, &d.icon, &d.tooltip)
        }
    }

    fn core(id: &str, label: &str, icon: &str, tooltip: &str) -> Self {
        Self {
            meta: NodeMeta::with_guid(lumacore_base::Guid::named("UiKey", id).0, Some(label.to_string()).filter(|l| !l.is_empty())),
            id: id.into(),
            icon: icon.into(),
            tooltip: tooltip.into(),
            owner: UiKeyOwner::Core,
            press: String::new(),
            double_press: String::new(),
            hold: String::new(),
            release: String::new(),
            release_after_hold: String::new(),
            meow: KeyCommands::default(),
            double_replaces_press: false,
            state: String::new(),
            locked: true,
            cycle: Vec::new(),
            types: Vec::new(),
            meow_cycle: Vec::new(),
            meow_types: Vec::new(),
            empty_only: false,
        }
    }

    /// The command for `gesture` (empty = none).
    pub fn command(&self, gesture: Gesture) -> &str {
        match gesture {
            Gesture::Press => &self.press,
            Gesture::Release => &self.release,
            Gesture::Hold => &self.hold,
            Gesture::ReleaseAfterHold => &self.release_after_hold,
            Gesture::Double => &self.double_press,
        }
    }

    pub fn command_mut(&mut self, gesture: Gesture) -> &mut String {
        match gesture {
            Gesture::Press => &mut self.press,
            Gesture::Release => &mut self.release,
            Gesture::Hold => &mut self.hold,
            Gesture::ReleaseAfterHold => &mut self.release_after_hold,
            Gesture::Double => &mut self.double_press,
        }
    }

    /// The command for `gesture` with Meow held or not: the Meow line when the
    /// key has one, else the plain line.
    pub fn command_for(&self, gesture: Gesture, meow: bool) -> &str {
        match self.meow.get(gesture) {
            line if meow && !line.is_empty() => line,
            _ => self.command(gesture),
        }
    }

    /// The press cycle with Meow held or not: the rows in order. With Meow held and no Meow press,
    /// the plain rows.
    pub fn press_rows(&self, meow: bool) -> Vec<PressRow> {
        let (first, more, types) = if meow && !self.meow.press.is_empty() {
            (&self.meow.press, &self.meow_cycle, &self.meow_types)
        } else {
            (&self.press, &self.cycle, &self.types)
        };
        std::iter::once(first)
            .chain(more)
            .enumerate()
            .filter(|(i, l)| *i == 0 || !l.is_empty())
            .map(|(i, l)| PressRow { line: l.clone(), types: types.contains(&(i as u8 + 1)) })
            .filter(|r| !r.line.is_empty())
            .collect()
    }

    /// Whether the key has more than one press (a cycle), with Meow held or not.
    pub fn cycles(&self, meow: bool) -> bool {
        self.press_rows(meow).len() > 1
    }

    /// Every (gesture, meow, command line) the key has.
    pub fn lines(&self) -> impl Iterator<Item = (Gesture, bool, &str)> {
        Gesture::ALL.into_iter().flat_map(move |g| {
            [(g, false, self.command(g)), (g, true, self.meow.get(g))]
                .into_iter()
                .filter(|(_, _, line)| !line.trim().is_empty())
        })
    }

    /// A user key made by `Bind` for a command line (`user.<key>`).
    pub fn user(id: &str, label: &str, press: &str) -> Self {
        Self {
            press: press.into(),
            owner: UiKeyOwner::User,
            locked: false,
            ..Self::core(id, label, "", "")
        }
    }
}

/// Double-press window and hold threshold (per user profile).
pub const DEFAULT_DOUBLE_CLICK_MS: u32 = 300;
pub const DEFAULT_HOLD_MS: u32 = 500;

/// The built-in keys — this crate's rows in the catalog (decisions §34 21), in their order.
pub(crate) fn core_keys() -> Vec<lumacore_registry::KeyDecl> {
    let keys = vec![
        // The usual Menu key: Press `Menu MenuSelector`, DoublePress `SaveShow`.
        // `Toggle Menu` (decisions §33 38): `Menu` alone sets your default object (39).
        KeyDecl { press: "Toggle Menu".into(), double_press: "SaveShow".into(), double_replaces_press: true, ..core("menu", "Menu", "⚙", "Menu — double-click: Quick Save") },
        KeyDecl { press: "Menu Shutdown".into(), ..core("power", "Power", "⏻", "Shutdown menu") },
        // The Oops key (decisions §30 "12f"): a tap undoes one step, a hold opens the Oops window.
        KeyDecl { press: "Oops".into(), hold: "Menu \"Oops\"".into(), ..core("oops", "Oops", "↶", "Oops — tap: undo one step · hold: the Oops window") },
        mode("blind", "Blind", "◑"),
        mode("highlight", "Highlight", "◐"),
        mode("freeze", "Freeze", "❄"),
        mode("solo", "Solo", "◎"),
    ];
    keys.into_iter().chain(command_keys()).chain(layer_keys()).collect()
}

/// The shortcut layers' own keys (decisions 0295; `keybindings::PRG_KEYS`): each row a press of
/// the cycle — `!…` runs at once, the others type onto the command line.
fn layer_keys() -> impl Iterator<Item = KeyDecl> {
    fn rows(list: &[&str]) -> (String, Vec<String>, Vec<u8>) {
        let mut lines = list.iter().map(|r| r.strip_prefix('!').unwrap_or(r).to_string());
        let first = lines.next().unwrap_or_default();
        let types = list.iter().enumerate().filter(|(_, r)| !r.starts_with('!')).map(|(i, _)| i as u8 + 1).collect();
        (first, lines.collect(), types)
    }
    lumacore_base::keybindings::PRG_KEYS.iter().map(|k| {
        let (press, cycle, types) = rows(k.press);
        let (meow_press, meow_cycle, meow_types) = rows(k.meow);
        KeyDecl {
            press,
            cycle,
            types,
            meow_press,
            meow_cycle,
            meow_types,
            hold: k.hold.into(),
            double_press: k.double.into(),
            empty_only: k.empty_only,
            ..core(k.id, k.label, "", &format!("{} {} — {}", k.layer.label(), k.key, k.label))
        }
    })
}

/// The built-in keys, numbered as published at `Root.Resources.UIKeys`: the catalog's keys as
/// locked core `UiKey`s.
pub fn builtin_ui_keys() -> BTreeMap<u32, UiKey> {
    lumacore_registry::get().keys().iter().enumerate().map(|(i, d)| (i as u32 + 1, UiKey::from_decl(d))).collect()
}

/// A core key's row with its text and nothing bound.
fn core(id: &str, label: &str, icon: &str, tooltip: &str) -> KeyDecl {
    KeyDecl { id: id.into(), label: label.into(), icon: icon.into(), tooltip: tooltip.into(), ..KeyDecl::default() }
}

/// A built-in mode key: toggles the mode, lit while it is on (§16, §17).
fn mode(id: &str, label: &str, icon: &str) -> KeyDecl {
    KeyDecl {
        press: label.into(),
        state: format!("Runtime.Users.{{User}}.Modes.{label}"),
        ..core(id, label, icon, &format!("{label} — toggle"))
    }
}

/// The core keys' Meow lines (decisions §30 "12a3"): (UiKey id, Meow press,
/// Meow release after hold). Normal theme: a tap toggles, a hold is "while
/// held" (the release after the hold toggles it back).
pub const CORE_MEOW: &[(&str, &str, &str)] = &[
    ("desklock", "Toggle NormalTheme", "Toggle NormalTheme"),
    ("shortcuts", "ViewBar", ""),
    ("screenshot", "Menu Shutdown", ""),
    ("sidebars", "FullScreen", ""),
];

/// Core keys for the factory keyboard bindings (`vanilla_keybindings`): a
/// binding only names a UiKey, so every default binding has one here.
fn command_keys() -> impl Iterator<Item = KeyDecl> {
    let mut seen = std::collections::BTreeSet::new();
    lumacore_base::keybindings::VANILLA
        .iter()
        // Several keys can press the same UiKey (F1 and Ctrl-H → help).
        .filter(move |(_, id, _, _)| seen.insert(*id))
        .map(|&(_, id, cmd, desc)| {
            let mut key = KeyDecl { press: cmd.into(), ..core(id, desc, "", desc) };
            if let Some((_, press, after_hold)) = CORE_MEOW.iter().find(|(k, _, _)| *k == id) {
                key.meow_press = (*press).into();
                key.meow_release_after_hold = (*after_hold).into();
            }
            if id == "desklock" {
                key.state = "Device.Locked".into();
            }
            key
        })
}

/// Find a key by id among core keys, plugin buttons and a profile's keys.
pub fn resolve<'a>(
    id: &str,
    core: &'a BTreeMap<u32, UiKey>,
    plugins: impl IntoIterator<Item = &'a UiKey>,
    profile: Option<&'a BTreeMap<u32, UiKey>>,
) -> Option<&'a UiKey> {
    let eq = |k: &&UiKey| k.id.eq_ignore_ascii_case(id);
    core.values()
        .find(eq)
        .or_else(|| plugins.into_iter().find(eq))
        .or_else(|| profile.and_then(|p| p.values().find(eq)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_double_press_is_quick_save_and_locked() {
        let core = builtin_ui_keys();
        let menu = resolve("Menu", &core, [], None).unwrap();
        assert_eq!(menu.command(Gesture::Double), "SaveShow");
        assert_eq!(menu.command(Gesture::Press), "Toggle Menu");
        assert!(menu.locked && menu.double_replaces_press);
        assert_eq!(Gesture::parse("DoublePress"), Some(Gesture::Double));
        assert_eq!(
            Gesture::parse("release_after_hold"),
            Some(Gesture::ReleaseAfterHold)
        );
    }

    #[test]
    fn meow_lines_win_and_fall_back_to_plain_ones() {
        let mut key = UiKey::user("user.f5", "F5", "List Fixture");
        assert_eq!(key.command_for(Gesture::Press, true), "List Fixture");
        key.meow.press = "List Group".into();
        assert_eq!(key.command_for(Gesture::Press, true), "List Group");
        assert_eq!(key.command_for(Gesture::Press, false), "List Fixture");
        assert_eq!(key.lines().count(), 2);
    }

    #[test]
    fn two_profiles_binding_one_key_get_two_guids() {
        let mut a = crate::programmer::UserProfile::new(Some("A".into()));
        let mut b = crate::programmer::UserProfile::new(Some("B".into()));
        a.bind_command("F5", "List Fixture", "");
        b.bind_command("F5", "List Group", "");
        let guid = |p: &crate::programmer::UserProfile| p.content.ui_keys.values().next().unwrap().meta.identity.guid;
        assert_ne!(guid(&a), guid(&b));
        // Binding again keeps the key (and its GUID).
        let before = guid(&a);
        a.bind_command("F5", "List Cue", "");
        assert_eq!(guid(&a), before);
    }

    #[test]
    fn mode_keys_toggle_and_light_from_the_user_modes() {
        let core = builtin_ui_keys();
        let blind = resolve("blind", &core, [], None).unwrap();
        assert_eq!(blind.press, "Blind");
        assert_eq!(blind.state, "Runtime.Users.{User}.Modes.Blind");
        let ids: std::collections::BTreeSet<_> = core.values().map(|k| k.id.as_str()).collect();
        assert_eq!(ids.len(), core.len(), "core key ids are unique");
    }

    #[test]
    fn core_wins_over_plugin_and_profile_keys() {
        let core = builtin_ui_keys();
        let fake = UiKey {
            owner: UiKeyOwner::User,
            locked: false,
            double_press: "Oops".into(),
            ..core[&1].clone()
        };
        let mut profile = BTreeMap::new();
        profile.insert(1, fake.clone());
        let k = resolve("menu", &core, [&fake], Some(&profile)).unwrap();
        assert_eq!(k.owner, UiKeyOwner::Core);
        let user = UiKey {
            id: "user.strobe".into(),
            ..fake
        };
        profile.insert(2, user);
        assert_eq!(
            resolve("user.strobe", &core, [], Some(&profile))
                .unwrap()
                .owner,
            UiKeyOwner::User
        );
    }
}
