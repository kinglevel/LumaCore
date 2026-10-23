//! Claim listing and target parsing.

use crate::AppState;
use crate::claims::Claims;

/// Show currently claimed objects.
pub fn list_claims(state: &AppState) -> String {
    use nu_ansi_term::{Color, Style};

    let mut lines: Vec<String> = Vec::new();

    lines.push(format!(
        "  {} {} {}",
        Color::Green.paint("*"),
        "Programmer",
        Style::new().dimmed().paint("(you)")
    ));

    if let Some(programmer) = state.current_programmer() {
        let selected = programmer.selected_fixtures.len();
        if selected > 0 {
            let label = if selected == 1 {
                "1 Fixture".to_string()
            } else {
                format!("{selected} Fixtures")
            };
            lines.push(format!(
                "  {} {} {}",
                Color::Green.paint("*"),
                label,
                Style::new().dimmed().paint("(selected)")
            ));
        }
    }

    for (obj, owner, editor) in state.claims_in_order() {
        let how = if editor { ", editor" } else { "" };
        if owner == state.effective_user_name() {
            lines.push(format!(
                "  {} {} {}",
                Color::Green.paint("*"),
                obj,
                Style::new().dimmed().paint(format!("(you{how})"))
            ));
        } else {
            lines.push(format!(
                "  {} {} {}",
                Color::Yellow.paint("*"),
                obj,
                Style::new().dimmed().paint(format!("({owner}{how})"))
            ));
        }
    }
    for ask in &state.root.runtime.claim_asks {
        lines.push(format!(
            "  {} {} {}",
            Color::Red.paint("?"),
            ask.label,
            Style::new()
                .dimmed()
                .paint(format!("({} asks {} for it)", ask.from, ask.owner))
        ));
    }

    if lines.is_empty() {
        return Style::new()
            .dimmed()
            .paint("No objects claimed.")
            .to_string();
    }

    format!(
        "{}\n{}",
        Style::new().bold().paint("Claimed objects:"),
        lines.join("\n")
    )
}

pub fn parse_claim_targets(args: &[&str]) -> Vec<String> {
    let mut targets = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let mut object = capitalize(args[i]);
        if let Some(next) = args.get(i + 1) {
            let trimmed = next.trim();
            let numeric_like = trimmed
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '*' | '/'));
            let quoted = (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''));
            if numeric_like || quoted {
                object.push(' ');
                object.push_str(trimmed);
                i += 1;
            }
        }
        targets.push(object);
        i += 1;
    }

    targets
}

/// Capitalize first letter of a string.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
    }
}
