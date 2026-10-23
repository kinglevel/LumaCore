// ██████╗ ██╗███╗   ██╗██████╗
// ██╔══██╗██║████╗  ██║██╔══██╗
// ██████╔╝██║██╔██╗ ██║██║  ██║
// ██╔══██╗██║██║╚██╗██║██║  ██║
// ██████╔╝██║██║ ╚████║██████╔╝
// ╚═════╝ ╚═╝╚═╝  ╚═══╝╚═════╝
// FILE: bind

//! ``Bind <key> `<line>` … | <UiKey> [/Shortcuts <mode>]``: command lines become the user key
//! `user.<key>` (`user.prg.<key>` in a mode; several lines are a press cycle); the core keys
//! (F9–F12, Pause, Print Screen) can't be bound (E-KEY-005); a letter types in KBD (E-KEY-006).
//!
//! Owns:          Bind and Unbind, their options, the key names (`Ctrl-R`, `F5`, `Space`).
//! Doesn't own:   the profile's bindings (lumacore-tree), the core key list (lumacore-base keybindings).
//! Entry points:  bind, unbind, parse_key_name, typing_key, SHORTCUTS, DESCRIPTION.
//! Decisions:     §17, §30 "12a3", §33 91, 0295.

use lumacore_tree::keybindings::{Layer, slot};

use lumacore_objects::AppState;
use lumacore_objects::commands::flags::{self, FlagDef};
use lumacore_objects::commands::registry;
use lumacore_objects::errors::render_code;
use lumacore_objects::text::render_success;

use crate::lines::check_line;
use crate::lookup::find;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// `/Shortcuts "PRG"`: the mode a binding works in (decisions 0295); absent = every mode.
pub const SHORTCUTS: FlagDef = FlagDef::new("Shortcuts", "The shortcut mode the binding works in (KBD, PRG, NAV); absent = every mode")
    .value("mode")
    .choices(&["All", "KBD", "PRG", "NAV"]);
/// `/Description "…"`: the key's text in lists and the editor.
pub const DESCRIPTION: FlagDef = FlagDef::new("Description", "The key's text in lists and the key editor").rest("text");

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The words of a Bind / Unbind line without its options, and the mode `/Shortcuts` names.
fn split(args: &[&str]) -> Result<(Vec<String>, Layer, Option<String>), String> {
    let mut words = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i];
        i += 1;
        if flags::is(a, &DESCRIPTION) {
            while i < args.len() && !flags::is_flag(args[i]) {
                i += 1;
            }
        } else if flags::is(a, &SHORTCUTS) {
            if a.contains('=') {
                continue;
            }
            i += usize::from(args.get(i).is_some_and(|n| !flags::is_flag(n)));
        } else {
            words.push(a.to_string());
        }
    }
    let layer = match flags::value(args, &SHORTCUTS) {
        None => Layer::All,
        Some(m) => Layer::parse(&m).ok_or_else(|| {
            render_code(format!("/Shortcuts takes All, KBD, PRG or NAV — not '{m}'."), lumacore_syntax::codes::VALUE_WORD)
        })?,
    };
    Ok((words, layer, flags::rest_value(args, &DESCRIPTION)))
}

/// Whether a key types a character in the Keyboard mode: a letter, digit or sign (Shift or not)
/// or Space — no Ctrl or Alt (decisions 0295).
pub fn typing_key(key: &str) -> bool {
    let Some((code, mods)) = parse_key_name(key) else { return false };
    mods & 6 == 0 && matches!(code, lumacore_wire::RemoteShellCode::Char(_))
}

/// The key a line names, checked: a key name, never a core key.
fn checked_key(word: &str, layer: Layer) -> Result<String, String> {
    let key = lumacore_tree::normalize_key(word.trim_matches('"'));
    if lumacore_base::keybindings::is_core_key(&key) {
        return Err(render_code(
            format!("{key} is a core key (F9–F12, Pause, Print Screen) and can't be bound. F1–F8 are free."),
            "E-KEY-005",
        ));
    }
    if parse_key_name(&key).is_none() {
        return Err(render_code(format!("Unknown key '{key}'. Examples: Ctrl-R, F5, Alt-S, Ctrl-Shift-L, Space"), "E-KEY-002"));
    }
    if typing_key(&key) && matches!(layer, Layer::All | Layer::Keyboard) {
        return Err(render_code(
            format!("{key} types in KBD. Bind it in a mode where keys are words: Bind \"{key}\" `…` /Shortcuts \"PRG\" (or \"NAV\")."),
            "E-KEY-006",
        ));
    }
    Ok(key)
}

/// ``Bind <key> `<line>` [`<line>` …] | <UiKey id> [/Shortcuts <mode>] [/Description "…"]`` —
/// bind a keyboard key for this profile, in every mode or one. Lines go into the user key
/// `user.<key>` (several are a press cycle, each press running the next); a UiKey id binds the
/// key to it. Takes effect at once.
pub fn bind(state: &mut AppState, args: &[&str]) -> String {
    let (words, layer, description) = match split(args) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let [key, rest @ ..] = words.as_slice() else {
        return usage();
    };
    let key = match checked_key(key, layer) {
        Ok(k) => k,
        Err(e) => return e,
    };
    // Lines are in backticks (decisions §33 91); a quoted word after them is the old description.
    let lines: Vec<String> = rest.iter().filter_map(|w| lumacore_objects::commands::tokens::line_of(w).map(str::to_string)).collect();
    let others: Vec<&String> = rest.iter().filter(|w| lumacore_objects::commands::tokens::line_of(w).is_none()).collect();
    let description = description.unwrap_or_else(|| match (lines.is_empty(), others.as_slice()) {
        (false, [d]) | (true, [_, d]) => d.trim_matches('"').to_string(),
        _ => String::new(),
    });
    if lines.is_empty() {
        let Some(target) = others.first().map(|t| t.trim_matches('"').to_string()) else {
            return usage();
        };
        let Some(id) = find(state, &target).map(|k| k.id) else {
            return render_code(
                format!("No UiKey \"{target}\" — a command line is written in backticks: Bind \"{key}\" `{target}`"),
                lumacore_syntax::codes::VALUE_WORD,
            );
        };
        let Some(profile) = state.current_user_profile_mut() else {
            return render_code("No user profile", "E-CMD-011");
        };
        profile.bind_ui_key_in(layer, &key, &id, &description);
        state.show_dirty = true;
        return render_success(&format!("{key}{} → UIKey {id}", in_mode(layer)));
    }
    // A command line is stored with the full words (R1-71, Q11): a new registry word can't
    // change what the key does later.
    let rows: Vec<String> = lines.iter().map(|l| registry::full_words(l)).collect();
    let mut warnings = Vec::new();
    for row in &rows {
        for problem in check_line(row) {
            match problem {
                Err(e) => return render_code(format!("Can't bind {key}: {e}."), "E-KEY-001"),
                Ok(w) => warnings.push(w),
            }
        }
    }
    let Some(profile) = state.current_user_profile_mut() else {
        return render_code("No user profile", "E-CMD-011");
    };
    let id = profile.bind_rows(layer, &key, &rows, &[], &description);
    state.show_dirty = true;
    let mut text = render_success(&format!(
        "{key}{} → UIKey {id} ({})",
        in_mode(layer),
        rows.iter().map(|r| format!("`{r}`")).collect::<Vec<_>>().join(" › ")
    ));
    for w in warnings {
        text.push_str(&format!("\n  ⚠ {w}"));
    }
    text
}

fn usage() -> String {
    render_code(
        "Usage: Bind <key> `<command line>` [`<line>` …] | <UiKey> [/Shortcuts KBD|PRG|NAV] [/Description \"…\"]\n\
         Example: Bind \"F5\" `List Fixture`   ·   Bind \"E\" `Edit` `EditSetting` /Shortcuts \"PRG\"   ·   Bind \"F6\" \"blind\" (a UiKey)",
        "E-CMD-006",
    )
}

fn in_mode(layer: Layer) -> String {
    match layer {
        Layer::All => String::new(),
        l => format!(" in {}", l.label()),
    }
}

/// `Unbind <key> [/Shortcuts <mode>]` — remove this profile's binding (LumaCore's comes back);
/// with none of yours there, turn LumaCore's default off in that mode on this profile.
pub fn unbind(state: &mut AppState, args: &[&str]) -> String {
    let (words, layer, _) = match split(args) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let Some(word) = words.first() else {
        return render_code("Usage: Unbind <key> [/Shortcuts KBD|PRG|NAV]\nExample: Unbind \"F5\"   ·   Unbind \"E\" /Shortcuts \"PRG\"", "E-CMD-006");
    };
    let key = lumacore_tree::normalize_key(word.trim_matches('"'));
    if lumacore_base::keybindings::is_core_key(&key) {
        return render_code(format!("{key} is a core key and stays."), "E-KEY-005");
    }
    let at = slot(layer, &key);
    let factory = state.root.default_keybindings.get(&at).map(|b| b.ui_key.clone());
    let Some(profile) = state.current_user_profile_mut() else {
        return render_code("No user profile", "E-CMD-011");
    };
    let mode = in_mode(layer);
    let reply = match (profile.content.keybindings.remove(&at), factory) {
        (Some(b), Some(f)) if b.ui_key.is_empty() => format!("LumaCore's {key}{mode} is back ({f})"),
        (Some(b), Some(f)) => format!("{key}{mode} unbound (was {}) — LumaCore's {f} is back", b.ui_key),
        (Some(b), None) => format!("{key}{mode} unbound (was {})", b.ui_key),
        (None, Some(f)) => {
            profile.bind_ui_key_in(layer, &key, "", "");
            format!("LumaCore's {key}{mode} ({f}) is off on this profile — Unbind again brings it back")
        }
        (None, None) => return render_code(format!("No binding for '{key}'{mode} to remove."), "E-KEY-003"),
    };
    state.show_dirty = true;
    render_success(&reply)
}

/// Parse a normalized key name ("Ctrl-R", "F5", "Alt-S", "Space") into a key code and
/// modifier bits (SHIFT 1, ALT 2, CONTROL 4 — the wire's and crossterm's).
/// None for names that aren't keys.
pub fn parse_key_name(name: &str) -> Option<(lumacore_wire::RemoteShellCode, u8)> {
    use lumacore_wire::RemoteShellCode as Code;
    let parts: Vec<&str> = name.split('-').collect();
    let mut modifiers = 0u8;
    let mut i = 0;
    loop {
        match *parts.get(i)? {
            "Ctrl" => modifiers |= 4,
            "Alt" => modifiers |= 2,
            "Shift" => modifiers |= 1,
            _ => break,
        }
        i += 1;
    }
    let key_part = parts[i..].join("-");
    let code = if key_part.chars().count() == 1 {
        Code::Char(key_part.chars().next()?.to_ascii_lowercase())
    } else {
        match key_part.as_str() {
            "Space" => Code::Char(' '),
            "Tab" => Code::Tab,
            "Enter" => Code::Enter,
            "Esc" => Code::Esc,
            "Backspace" => Code::Backspace,
            "Delete" | "Del" => Code::Delete,
            "Home" => Code::Home,
            "End" => Code::End,
            "PageUp" => Code::PageUp,
            "PageDown" => Code::PageDown,
            "Up" => Code::Up,
            "Down" => Code::Down,
            "Left" => Code::Left,
            "Right" => Code::Right,
            "Insert" | "Ins" => Code::Insert,
            "PrintScreen" | "Print" => Code::PrintScreen,
            "Pause" => Code::Pause,
            f => match f.strip_prefix('F')?.parse::<u8>() {
                Ok(n @ 1..=12) => Code::F(n),
                _ => return None,
            },
        }
    };
    Some((code, modifiers))
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
