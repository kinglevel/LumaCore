// ██████╗ ██████╗ ██╗███╗   ██╗████████╗
// ██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝
// ██████╔╝██████╔╝██║██╔██╗ ██║   ██║
// ██╔═══╝ ██╔══██╗██║██║╚██╗██║   ██║
// ██║     ██║  ██║██║██║ ╚████║   ██║
// ╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝
// FILE: print

//! What every tree-shell word shares when it prints.
//!
//! Owns:          how the tree shell prints: an argument's dot-path, coloured JSON, short values, dates.
//! Doesn't own:   what is printed (the words' files).
//! Entry points:  vfs_resolve_dot_path, colorize_json, format_json_short, chrono_lite.
//! Decisions:     §33 18.

use lumacore_objects::AppState;
use lumacore_objects::navigation::Navigation;
use lumacore_objects::vfs;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The dot path a word's argument names from where the shell stands: the one parse of cd and
/// ls (`Navigation::resolve_target`, decisions §31), so a name that isn't there is refused
/// (E-SET-004) instead of read as another node (`cat 1` in ShowData, decisions 0291).
pub(crate) fn vfs_resolve_dot_path(state: &AppState, arg: Option<&str>) -> Result<String, String> {
    let target = arg.unwrap_or(".");
    state
        .resolve_target(target)
        .map(|path| vfs::dot_path_from_segments(&path))
        .map_err(|e| lumacore_objects::errors::render_code(format!("cannot access '{target}': No such node ({e})"), "E-SET-004"))
}

/// Pretty-print JSON with ANSI colors.
pub(crate) fn colorize_json(json_str: &str) -> String {
    use nu_ansi_term::{Color, Style};
    let mut out = String::new();
    let dim = Style::new().dimmed();

    for line in json_str.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('"') {
            // Key-value line — colorize key
            if let Some(colon_pos) = trimmed.find(':') {
                let indent = &line[..line.len() - trimmed.len()];
                let key = &trimmed[..colon_pos];
                let value = &trimmed[colon_pos + 1..].trim();
                out.push_str(&format!(
                    "{}{}: {}\n",
                    indent,
                    Color::Cyan.paint(key),
                    colorize_json_value(value)
                ));
            } else {
                out.push_str(line);
                out.push('\n');
            }
        } else {
            out.push_str(&format!("{}\n", dim.paint(line)));
        }
    }
    out.trim_end().to_string()
}

fn colorize_json_value(value: &str) -> String {
    use nu_ansi_term::Color;
    let v = value.trim_end_matches(',');
    let trailing = if value.ends_with(',') { "," } else { "" };
    if v.starts_with('"') {
        format!("{}{}", Color::Green.paint(v), trailing)
    } else if v == "true" || v == "false" {
        format!("{}{}", Color::Yellow.paint(v), trailing)
    } else if v == "null" {
        format!("{}{}", Color::Red.paint(v), trailing)
    } else if v.parse::<f64>().is_ok() {
        format!("{}{}", Color::White.bold().paint(v), trailing)
    } else {
        value.to_string()
    }
}

pub(crate) fn format_json_short(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Array(a) => format!("[{} items]", a.len()),
        serde_json::Value::Object(m) => format!("{{{} fields}}", m.len()),
    }
}

/// Lightweight timestamp formatting from Unix epoch seconds.
pub(crate) fn chrono_lite(epoch_secs: u64) -> String {
    // Simple: days since epoch → year/month/day + time
    const SECS_PER_DAY: u64 = 86400;
    const SECS_PER_HOUR: u64 = 3600;
    const SECS_PER_MIN: u64 = 60;

    let days = epoch_secs / SECS_PER_DAY;
    let time_of_day = epoch_secs % SECS_PER_DAY;
    let hours = time_of_day / SECS_PER_HOUR;
    let minutes = (time_of_day % SECS_PER_HOUR) / SECS_PER_MIN;

    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm)
    let z = days as i64 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}")
}
