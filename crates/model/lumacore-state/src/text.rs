//! Rich formatted output with ANSI colors and tabled tables.
//!
//! Color scheme:
//! - Green: success messages and values
//! - Red: errors
//! - Cyan: DMX values and addresses
//! - Yellow: warnings and DIN SPEC notices
//! - Bold: table headers

use nu_ansi_term::{Color, Style};
use tabled::settings::Style as TableStyle;
use tabled::{Table, Tabled};

use lumacore_gdtf::GdtfWarning;
use lumacore_patch::FixtureListEntry;

/// A row in the fixture list table.
#[derive(Tabled)]
pub struct FixtureRow {
    #[tabled(rename = "ID")]
    pub id: u32,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Type")]
    pub fixture_type: String,
    #[tabled(rename = "Universe")]
    pub universe: String,
    #[tabled(rename = "Address")]
    pub address: String,
    #[tabled(rename = "Mode")]
    pub mode: String,
    #[tabled(rename = "Footprint")]
    pub footprint: String,
}

/// A row in the fixture type library table.
#[derive(Tabled)]
pub struct FixtureTypeRow {
    #[tabled(rename = "ID")]
    pub id: u32,
    #[tabled(rename = "Manufacturer")]
    pub manufacturer: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Modes")]
    pub modes: String,
    #[tabled(rename = "Channels")]
    pub channels: String,
    #[tabled(rename = "File")]
    pub file: String,
}

/// A row in the universe status table.
#[derive(Tabled)]
pub struct UniverseRow {
    #[tabled(rename = "ID")]
    pub id: u32,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Fixtures")]
    pub fixtures: u32,
    #[tabled(rename = "Usage")]
    pub usage: String,
    #[tabled(rename = "Output")]
    pub sacn: String,
}

/// A row for a DMX channel in the channel map display.
#[derive(Tabled)]
pub struct ChannelRow {
    #[tabled(rename = "Offset")]
    pub offset: String,
    #[tabled(rename = "Attribute")]
    pub attribute: String,
    #[tabled(rename = "Type")]
    pub attr_type: String,
    #[tabled(rename = "Bytes")]
    pub bytes: u8,
    #[tabled(rename = "Virtual")]
    pub is_virtual: String,
}

/// A row of a protocol's line list (`List Sacn`).
#[derive(Tabled)]
pub struct LineRow {
    #[tabled(rename = "ID")]
    pub id: u32,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Network")]
    pub network: String,
    #[tabled(rename = "Mode")]
    pub mode: String,
    #[tabled(rename = "Universes")]
    pub universes: String,
    /// How it reaches the network (multicast, unicast, ArtPoll) or whom it runs as (Data lines).
    #[tabled(rename = "Via")]
    pub via: String,
    #[tabled(rename = "Settings")]
    pub settings: String,
    #[tabled(rename = "State")]
    pub status: String,
}

/// Render a list of patched fixtures as a colored table.
pub fn render_fixture_list(entries: &[FixtureListEntry]) -> String {
    render_fixture_list_with_selection(entries, &std::collections::HashSet::new())
}

/// Selection display state for a fixture in the list.
///
/// - `None`: not selected
/// - `Selected`: in the current selection (solid yellow)
/// - `Pool`: in the walk pool but not the currently walked fixture (dark yellow)
/// - `Walked`: the currently walked fixture in walk mode (bright yellow highlight)
#[derive(Clone, Copy, PartialEq)]
pub enum FixtureSelState {
    None,
    Selected,
    Pool,
    Walked,
}

pub fn render_fixture_list_with_selection(
    entries: &[FixtureListEntry],
    selected: &std::collections::HashSet<u32>,
) -> String {
    render_fixture_list_with_walk(entries, selected, &std::collections::HashSet::new(), None)
}

/// Render fixture list with full walk-mode awareness.
///
/// - `selected`: currently active selection (drives values)
/// - `pool`: the full walk pool (when Next/Previous is active)
/// - `walked_id`: the single fixture currently being walked (bright highlight)
pub fn render_fixture_list_with_walk(
    entries: &[FixtureListEntry],
    selected: &std::collections::HashSet<u32>,
    pool: &std::collections::HashSet<u32>,
    walked_id: Option<u32>,
) -> String {
    if entries.is_empty() {
        return format!("{}", Color::Yellow.paint("No fixtures patched."));
    }

    // Dark yellow (RGB 180, 140, 0) for pool fixtures
    let dark_yellow = Color::Rgb(180, 140, 0);
    // Bright yellow for the walked fixture — use standard Yellow (already bright)
    let bright_yellow = Color::Yellow;

    let rows: Vec<FixtureRow> = entries
        .iter()
        .map(|e| {
            let sel_state = if walked_id == Some(e.fixture_id) {
                FixtureSelState::Walked
            } else if !pool.is_empty() && pool.contains(&e.fixture_id) {
                FixtureSelState::Pool
            } else if selected.contains(&e.fixture_id) {
                FixtureSelState::Selected
            } else {
                FixtureSelState::None
            };

            let color = match sel_state {
                FixtureSelState::Walked => bright_yellow,
                FixtureSelState::Pool => dark_yellow,
                FixtureSelState::Selected => bright_yellow,
                FixtureSelState::None => Color::Default,
            };

            let paint = |s: String| -> String {
                match sel_state {
                    FixtureSelState::None => s,
                    FixtureSelState::Walked => Style::new().bold().fg(color).paint(s).to_string(),
                    _ => color.paint(s).to_string(),
                }
            };
            let paint_cyan = |s: String| -> String {
                match sel_state {
                    FixtureSelState::None => Color::Cyan.paint(s).to_string(),
                    FixtureSelState::Walked => Style::new().bold().fg(color).paint(s).to_string(),
                    _ => color.paint(s).to_string(),
                }
            };

            FixtureRow {
                id: e.fixture_id,
                name: paint(
                    e.name
                        .clone()
                        .unwrap_or_else(|| format!("Fixture {}", e.fixture_id)),
                ),
                fixture_type: paint(format!(
                    "{} {}",
                    e.manufacturer,
                    e.fixture_type_name.as_deref().unwrap_or("?")
                )),
                universe: paint_cyan(e.universe_id.to_string()),
                address: paint_cyan(format!("{:03}", e.start_address)),
                mode: paint(e.mode_name.clone().unwrap_or_else(|| "—".to_string())),
                footprint: paint(e.footprint.to_string()),
            }
        })
        .collect();

    let table = Table::new(rows).with(TableStyle::modern()).to_string();

    let pool_count = if !pool.is_empty() {
        entries
            .iter()
            .filter(|e| pool.contains(&e.fixture_id))
            .count()
    } else {
        0
    };
    let sel_count = entries
        .iter()
        .filter(|e| selected.contains(&e.fixture_id))
        .count();

    let summary = if walked_id.is_some() && pool_count > 0 {
        format!(
            "{} fixture(s) total, walking {} in pool",
            entries.len(),
            bright_yellow.paint(pool_count.to_string())
        )
    } else if sel_count > 0 {
        format!(
            "{} fixture(s) total, {} selected",
            entries.len(),
            Color::Yellow.paint(sel_count.to_string())
        )
    } else {
        format!("{} fixture(s) total", entries.len())
    };

    format!(
        "{}\n{}\n{}",
        Style::new().bold().paint("Patched Fixtures:"),
        table,
        Color::Green.paint(summary)
    )
}

/// Render channel map sorted by DMX offset.
pub fn render_channel_map_offset(
    mode_name: &str,
    channels: &[lumacore_fixtures::ChannelFunction],
) -> String {
    if channels.is_empty() {
        return format!("{}", Color::Yellow.paint("No channels in this mode."));
    }

    let rows: Vec<ChannelRow> = channels
        .iter()
        .map(|cf| ChannelRow {
            offset: if cf.is_virtual {
                Color::Yellow.paint("virt").to_string()
            } else {
                Color::Cyan
                    .paint(format!("{:03}", cf.dmx_offset + 1))
                    .to_string()
            },
            attribute: cf.attribute_name.clone(),
            attr_type: format!("{:?}", cf.attribute_type),
            bytes: cf.byte_count,
            is_virtual: if cf.is_virtual {
                Color::Yellow.paint("yes").to_string()
            } else {
                "no".to_string()
            },
        })
        .collect();

    let table = Table::new(rows).with(TableStyle::modern()).to_string();

    format!(
        "{} {}\n{}",
        Style::new().bold().paint("Mode:"),
        Color::Green.paint(mode_name),
        table
    )
}

/// Render channel map grouped by attribute type.
pub fn render_channel_map_grouped(
    mode_name: &str,
    groups: &std::collections::BTreeMap<String, Vec<lumacore_fixtures::ChannelFunction>>,
) -> String {
    if groups.is_empty() {
        return format!("{}", Color::Yellow.paint("No channels in this mode."));
    }

    let mut output = format!(
        "{} {} (grouped by attribute type)\n",
        Style::new().bold().paint("Mode:"),
        Color::Green.paint(mode_name)
    );

    for (group_name, channels) in groups {
        output.push_str(&format!(
            "\n  {}\n",
            Color::Yellow.paint(format!("--- {group_name} ---"))
        ));
        for cf in channels {
            let offset_str = if cf.is_virtual {
                Color::Yellow.paint("virt").to_string()
            } else {
                Color::Cyan
                    .paint(format!("{:03}", cf.dmx_offset + 1))
                    .to_string()
            };
            output.push_str(&format!(
                "    [{offset_str}] {} ({} byte{}){}\n",
                cf.attribute_name,
                cf.byte_count,
                if cf.byte_count == 1 { "" } else { "s" },
                if cf.is_virtual { " [virtual]" } else { "" }
            ));
        }
    }

    output
}

/// Render universe status table.
pub fn render_universe_status(rows: Vec<UniverseRow>) -> String {
    if rows.is_empty() {
        return format!("{}", Color::Yellow.paint("No universes created."));
    }
    let count = rows.len();
    let table = Table::new(rows).with(TableStyle::modern()).to_string();

    format!(
        "{}\n{}\n{}",
        Style::new().bold().paint("Universes:"),
        table,
        Color::Green.paint(format!("{count} universe(s) total"))
    )
}

/// Render output line list.
/// `List Sacn`: one protocol's lines as a table.
pub fn render_line_list(protocol: &str, rows: Vec<LineRow>) -> String {
    if rows.is_empty() {
        return format!("{}", Color::Yellow.paint(format!("No {protocol} lines. Store {protocol} 1 /Network \"Rig\"")));
    }
    let count = rows.len();
    let table = Table::new(rows).with(TableStyle::modern()).to_string();

    format!(
        "{}\n{}\n{}",
        Style::new().bold().paint(format!("{protocol} Lines:")),
        table,
        Color::Green.paint(format!("{count} {protocol} line(s) total"))
    )
}

// ─── PropertySheet ───────────────────────────────────────────────────────
// Compact key/value display for info views and the future tree browser.
//
// Usage:
//   PropertySheet::new("Universe 1")
//       .field("Name", "Main Stage")
//       .field_colored("sACN", &Color::Green.paint("active").to_string())
//       .separator()
//       .field("Usage", "128/512 (25%)")
//       .bar("Map", 25, 40)     // 25% filled, 40-char wide
//       .section("Fixtures")
//       .line("  [001-024] ID:1 Spot")
//       .build()

/// Builder for compact key-value property displays.
pub struct PropertySheet {
    title: String,
    lines: Vec<String>,
    key_width: usize,
}

impl PropertySheet {
    /// Create a new property sheet with a bold title.
    pub fn new(title: &str) -> Self {
        Self {
            title: Style::new().bold().paint(title).to_string(),
            lines: Vec::new(),
            key_width: 12,
        }
    }

    /// Set the key column width (default 12).
    pub fn key_width(mut self, w: usize) -> Self {
        self.key_width = w;
        self
    }

    /// Add a key-value field.
    pub fn field(mut self, key: &str, value: &str) -> Self {
        let pad = if key.len() < self.key_width {
            self.key_width - key.len()
        } else {
            1
        };
        self.lines.push(format!(
            "  {}{}{}",
            Style::new().dimmed().paint(key),
            " ".repeat(pad),
            value
        ));
        self
    }

    /// Add a key-value field where value is already ANSI-colored.
    pub fn field_colored(self, key: &str, value: &str) -> Self {
        self.field(key, value)
    }

    /// Add a visual bar (e.g. for usage).
    pub fn bar(mut self, key: &str, pct: u32, width: usize) -> Self {
        let filled = ((pct as usize) * width) / 100;
        let empty = width.saturating_sub(filled);
        let bar = format!("[{}{}] {}%", "█".repeat(filled), "░".repeat(empty), pct);
        let pad = if key.len() < self.key_width {
            self.key_width - key.len()
        } else {
            1
        };
        self.lines.push(format!(
            "  {}{}{}",
            Style::new().dimmed().paint(key),
            " ".repeat(pad),
            bar
        ));
        self
    }

    /// Add a blank separator line.
    pub fn separator(mut self) -> Self {
        self.lines.push(String::new());
        self
    }

    /// Add a sub-section header (bold, indented).
    pub fn section(mut self, name: &str) -> Self {
        self.lines
            .push(format!("  {}", Style::new().bold().paint(name)));
        self
    }

    /// Add a raw line (already formatted).
    pub fn line(mut self, text: &str) -> Self {
        self.lines.push(text.to_string());
        self
    }

    /// Build the final string.
    pub fn build(self) -> String {
        let mut out = self.title;
        out.push('\n');
        for l in &self.lines {
            out.push_str(l);
            out.push('\n');
        }
        out
    }
}

/// Plain text of terminal output: ANSI escape sequences (`ESC [ … final`)
/// removed. The one implementation (logs, sysmon, HTTP replies, journal).
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Render an error message in red with an optional suggestion.
pub fn render_error(err: &str) -> String {
    format!("{} {}", Color::Red.bold().paint("Error:"), err)
}

/// Render a notice that isn't an error (session events, a pending forward)
/// in yellow — errors go through `errors::render_code` with their code.
pub fn render_warning(msg: &str) -> String {
    format!("{} {}", Color::Yellow.bold().paint("Note:"), msg)
}

/// Render a success message in green.
pub fn render_success(msg: &str) -> String {
    format!("{}", Color::Green.paint(msg))
}

/// Render DIN SPEC 15800 warnings in yellow after import.
pub fn render_din_warnings(warnings: &[GdtfWarning]) -> String {
    if warnings.is_empty() {
        return String::new();
    }

    let mut output = format!(
        "{}\n",
        Color::Yellow
            .bold()
            .paint(format!("{} DIN SPEC 15800 warning(s):", warnings.len()))
    );

    for w in warnings {
        output.push_str(&format!("  {} {}\n", Color::Yellow.paint("!"), w));
    }

    output
}
