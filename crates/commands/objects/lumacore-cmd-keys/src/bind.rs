// ██████╗ ██╗███╗   ██╗██████╗
// ██╔══██╗██║████╗  ██║██╔══██╗
// ██████╔╝██║██╔██╗ ██║██║  ██║
// ██╔══██╗██║██║╚██╗██║██║  ██║
// ██████╔╝██║██║ ╚████║██████╔╝
// ╚═════╝ ╚═╝╚═╝  ╚═══╝╚═════╝
// FILE: bind

//! ``Bind <key> `<line>` | <UiKey>``: a command line becomes the user key `user.<key>`; the core keys
//! (F9–F12, Pause, Print Screen) can't be bound (E-KEY-005).
//!
//! Owns:          Bind and Unbind, the key names (`Ctrl-R`, `F5`).
//! Doesn't own:   the profile's bindings (lumacore-tree), the core key list (lumacore-base keybindings).
//! Entry points:  bind, unbind, parse_key_name.
//! Decisions:     §17, §30 "12a3", §33 91.

use lumacore_tree::KeyBinding;

use lumacore_objects::AppState;
use lumacore_objects::commands::registry;

use crate::lines::check_line;
use crate::lookup::find;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// ``Bind <key> `<command line>` | <UiKey id> ["description"]`` — bind a keyboard
/// key for this profile. A command line goes into the user key `user.<key>`.
/// Takes effect at once.
pub fn bind(state: &mut AppState, args: &[&str]) -> String {
    use lumacore_objects::errors::render_code;
    use lumacore_objects::text::render_success;
    let [key, target, rest @ ..] = args else {
        return lumacore_objects::errors::render_code(
            "Usage: Bind <key> `<command line>` | <UiKey> [\"description\"]\n\
             Example: Bind \"F5\" `List Fixture` \"Quick list\"   ·   Bind \"F6\" \"blind\" (a UiKey)",
            "E-CMD-006",
        );
    };
    let key = lumacore_tree::normalize_key(key);
    if lumacore_base::keybindings::is_core_key(&key) {
        return render_code(
            format!(
                "{key} is a core key (F9–F12, Pause, Print Screen) and can't be bound. F1–F8 are free."
            ),
            "E-KEY-005",
        );
    }
    if parse_key_name(&key).is_none() {
        return render_code(
            format!("Unknown key '{key}'. Examples: Ctrl-R, F5, Alt-S, Ctrl-Shift-L"),
            "E-KEY-002",
        );
    }
    // A command line is in backticks (decisions §33 91); anything else names a UiKey.
    let line = lumacore_objects::commands::tokens::line_of(target);
    let target = line.unwrap_or_else(|| target.trim_matches('"'));
    let description = rest.join(" ").trim_matches('"').to_string();
    let existing = if line.is_some() {
        None
    } else {
        find(state, target).map(|k| k.id)
    };
    if line.is_none() && existing.is_none() {
        return render_code(
            format!(
                "No UiKey \"{target}\" — a command line is written in backticks: Bind {key} `{target}`"
            ),
            lumacore_syntax::codes::VALUE_WORD,
        );
    }
    // A command line is stored with the full words (R1-71, Q11): a new
    // registry word can't change what the key does later.
    let full = if existing.is_none() {
        registry::full_words(target)
    } else {
        target.to_string()
    };
    let target = full.as_str();
    let mut warnings = Vec::new();
    if existing.is_none() {
        for problem in check_line(target) {
            match problem {
                Err(e) => {
                    return render_code(format!("Can't bind {key}: {e}."), "E-KEY-001");
                }
                Ok(w) => warnings.push(w),
            }
        }
    }
    let Some(profile) = state.current_user_profile_mut() else {
        return lumacore_objects::errors::render_code("No user profile", "E-CMD-011");
    };
    let id = match &existing {
        Some(id) => {
            profile.bind_ui_key(&key, id, &description);
            id.clone()
        }
        None => profile.bind_command(&key, target, &description),
    };
    state.show_dirty = true;
    let mut text = render_success(&match existing {
        Some(_) => format!("{key} → UIKey {id}"),
        None => format!("{key} → UIKey {id} (\"{target}\")"),
    });
    for w in warnings {
        text.push_str(&format!("\n  ⚠ {w}"));
    }
    text
}

/// `Unbind <key>` — remove this profile's binding (vanilla keys stay).
pub fn unbind(state: &mut AppState, args: &[&str]) -> String {
    let Some(key) = args.first().map(|k| lumacore_tree::normalize_key(k)) else {
        return lumacore_objects::errors::render_code(
            "Usage: Unbind <key>\nExample: Unbind F5",
            "E-CMD-006",
        );
    };
    let removed: Option<KeyBinding> = state
        .current_user_profile_mut()
        .and_then(|p| p.content.keybindings.remove(&key));
    match removed {
        Some(b) => {
            state.show_dirty = true;
            lumacore_objects::text::render_success(&format!("{key} unbound (was {})", b.ui_key))
        }
        None => lumacore_objects::errors::render_code(
            format!("No user binding for '{key}'."),
            "E-KEY-003",
        ),
    }
}

/// Parse a normalized key name ("Ctrl-R", "F5", "Alt-S") into a key code and
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
