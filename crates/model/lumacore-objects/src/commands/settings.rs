//! User setting commands: set (Bind / Unbind live in `terminal::keys`).

use crate::AppState;

/// Show current user settings.
pub fn handle_list_settings(state: &AppState) -> String {
    use nu_ansi_term::{Color, Style};

    let profile = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name());

    let mut lines = Vec::new();
    lines.push(Style::new().bold().paint("User Settings").to_string());

    if let Some(p) = profile {
        let on = Color::Green.paint("ON");
        let off = Style::new().dimmed().paint("OFF");

        lines.push(format!(
            "  descriptions   {}",
            if p.settings.show_descriptions { &on } else { &off }
        ));
        lines.push(format!(
            "  clear_buffer   {}{}",
            if p.settings.clear_buffer { &on } else { &off },
            if !state.watches().is_empty() && !p.settings.clear_buffer {
                format!(
                    "  {}",
                    Style::new().dimmed().paint("(forced ON by active watches)")
                )
            } else {
                String::new()
            }
        ));
        lines.push(format!(
            "  verbosity      {}  {}",
            Color::Cyan.paint(p.settings.verbosity.display()),
            Style::new()
                .dimmed()
                .paint("(override per-command: /v or /d)"),
        ));
        if let Some(prefs) = state
            .root
            .show_data
            .profile_for_user(state.effective_user_name())
            .map(|p| &p.settings.preferences)
        {
            lines.push(format!(
                "  doubleclicktime {} ms   holdtime {} ms   meowkey {}   suggestions {}",
                prefs.double_click_ms, prefs.hold_ms, prefs.meow_key, prefs.suggestions.label()
            ));
        }
    } else {
        lines.push(format!(
            "  {}",
            Style::new().dimmed().paint("No user profile found.")
        ));
    }

    if !state.watches().is_empty() {
        lines.push(String::new());
        lines.push(Style::new().bold().paint("Active Watches").to_string());
        for (i, w) in state.watches().iter().enumerate() {
            lines.push(format!(
                "  {}  {} {}",
                Color::Cyan.paint(format!("[{}]", i + 1)),
                w.command,
                Style::new()
                    .dimmed()
                    .paint(format!("({}ms)", w.interval_ms)),
            ));
        }
    }

    lines.join("\n")
}

