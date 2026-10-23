//! Miscellaneous list functions: users, keybindings, watches, selection, library.

use lumacore_tree::TreeNode;
use nu_ansi_term::{Color, Style};

use crate::AppState;
use crate::errors::render_code;
use crate::text::PropertySheet;

pub fn list_users(state: &AppState) -> String {
    if state.root.show_data.user_profiles.is_empty() {
        return format!("{}", Color::Yellow.paint("No user profiles."));
    }

    let bold = |s: &str| Style::new().bold().paint(s).to_string();
    let cyan = |s: &str| Color::Cyan.paint(s).to_string();
    let dim = |s: &str| Style::new().dimmed().paint(s).to_string();
    let green = |s: &str| Color::Green.paint(s).to_string();

    let mut lines: Vec<String> = Vec::new();
    lines.push(bold("User Profiles"));

    let count = state.root.show_data.user_profiles.len();
    for (i, (id, profile)) in state.root.show_data.user_profiles.iter().enumerate() {
        let is_last = i == count - 1;
        let c = if is_last { "└" } else { "├" };
        let name = profile.name().unwrap_or("<unnamed>");
        // Rights live on the users (decisions §30 "12c2"): who works here.
        let sd = &state.root.show_data;
        let role = if profile.locked {
            "template (locked)".to_string()
        } else {
            let users: Vec<&str> = sd
                .users
                .values()
                .filter(|u| u.profile == profile.meta.identity.guid)
                .map(|u| u.login.as_str())
                .collect();
            if users.is_empty() { "no users".to_string() } else { users.join(", ") }
        };
        let binds = profile.content.keybindings.len();
        let active = if sd.profile_id_for_user(state.effective_user_name()) == Some(*id) {
            green(" (active)")
        } else {
            String::new()
        };
        let binds_str = if binds > 0 {
            format!("  {}", dim(&format!("{binds} keybind(s)")))
        } else {
            String::new()
        };
        lines.push(format!(
            "{}── {}  {}  {}{}{}",
            c,
            cyan(&format!("{id}")),
            name,
            dim(&role),
            binds_str,
            active,
        ));
    }

    lines.join("\n")
}

/// List active watches.
pub fn list_watches(state: &AppState) -> String {
    if state.watches().is_empty() {
        return render_code("No active watches.", "E-CMD-009");
    }

    let mut lines = Vec::new();
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

    lines.push(String::new());
    lines.push(format!(
        "  {}",
        Style::new()
            .dimmed()
            .paint("unwatch [index|name|all] to stop")
    ));

    lines.join("\n")
}

pub fn list_selection(state: &AppState) -> String {
    let prog = match state.current_programmer() {
        Some(p) => p,
        None => return render_code("No programmer available.", "E-CMD-011"),
    };

    if prog.selected_fixtures.is_empty() {
        return Style::new()
            .dimmed()
            .paint("No fixtures selected.")
            .to_string();
    }

    let fixtures = &state.root.show_data.devices.dmx.patch;
    let mut lines = Vec::new();
    lines.push(format!(
        "{} ({})",
        Style::new().bold().paint("Selection"),
        Color::Cyan.paint(format!(
            "{} fixture{}",
            prog.selected_fixtures.len(),
            if prog.selected_fixtures.len() == 1 {
                ""
            } else {
                "s"
            }
        )),
    ));

    for &guid in &prog.selected_fixtures {
        let int_id = state.root.show_data.fixture_id_for_guid(guid);
        let name = int_id
            .and_then(|id| fixtures.fixture(id))
            .and_then(|f| f.name())
            .unwrap_or("?");
        let id_display = int_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "?".to_string());
        lines.push(format!(
            "  {} {}",
            Color::Cyan.paint(format!("{:>4}", id_display)),
            name,
        ));
    }

    lines.join("\n")
}

/// List all library contents (summary of each library type).
pub fn list_library_all(_state: &AppState) -> String {
    let gdtf_count = super::count_library_gdtf();

    let count_or_soon = |n: usize| -> String {
        if n > 0 {
            Color::Green.paint(format!("{n} files")).to_string()
        } else {
            Style::new().dimmed().paint("empty").to_string()
        }
    };

    PropertySheet::new("Library")
        .field("GDTF/", &count_or_soon(gdtf_count))
        .field(
            "shows/",
            &Style::new().dimmed().paint("coming soon").to_string(),
        )
        .field(
            "presets/",
            &Style::new().dimmed().paint("coming soon").to_string(),
        )
        .field(
            "datapools/",
            &Style::new().dimmed().paint("coming soon").to_string(),
        )
        .field(
            "mvr/",
            &Style::new().dimmed().paint("coming soon").to_string(),
        )
        .separator()
        .line(&format!(
            "  {}",
            Style::new()
                .dimmed()
                .paint("Use 'list library GDTF' to browse a specific library.")
        ))
        .build()
}
