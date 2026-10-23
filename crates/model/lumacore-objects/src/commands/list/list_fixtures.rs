//! Fixture-related list functions.

use lumacore_patch::PatchEngine;
use nu_ansi_term::{Color, Style};
use tabled::Table;
use tabled::settings::Style as TableStyle;

use crate::AppState;
use crate::text::FixtureTypeRow;

/// List all patched fixtures.
pub fn list_fixtures(state: &AppState) -> String {
    let entries = PatchEngine::list_fixtures(&state.root.show_data);
    list_entries(state, entries)
}

/// The list narrowed to `nos` (decisions §40 "40d hardened": `List Fixture Media 1`).
pub fn list_fixtures_of(state: &AppState, nos: &[u32]) -> String {
    let entries: Vec<_> = PatchEngine::list_fixtures(&state.root.show_data).into_iter().filter(|e| nos.contains(&e.fixture_id)).collect();
    list_entries(state, entries)
}

fn list_entries(state: &AppState, entries: Vec<lumacore_patch::FixtureListEntry>) -> String {

    // In verbose/debug mode, append programmer values inline for each fixture.
    if state.effective_verbosity().is_verbose() {
        return list_fixtures_verbose(state, &entries);
    }

    // Collect selected fixture IDs (as u32) and walk state from the current programmer
    let (selected, pool, walked_id) = match state.current_programmer() {
        Some(p) => {
            let sel: std::collections::HashSet<u32> = p
                .selected_fixtures
                .iter()
                .filter_map(|guid| state.root.show_data.fixture_id_for_guid(*guid))
                .collect();
            let pool_set: std::collections::HashSet<u32> = p
                .selection_pool
                .as_ref()
                .map(|v| {
                    v.iter()
                        .filter_map(|guid| state.root.show_data.fixture_id_for_guid(*guid))
                        .collect()
                })
                .unwrap_or_default();
            let walked = if p.is_walking() {
                p.selected_fixtures
                    .first()
                    .and_then(|guid| state.root.show_data.fixture_id_for_guid(*guid))
            } else {
                None
            };
            (sel, pool_set, walked)
        }
        None => (
            std::collections::HashSet::new(),
            std::collections::HashSet::new(),
            None,
        ),
    };

    crate::text::render_fixture_list_with_walk(&entries, &selected, &pool, walked_id)
}

/// Verbose fixture list: shows all patched fixtures with inline programmer values.
///
/// Called when `/v` or `/d` flag is present, or user profile verbosity is Verbose/Debug.
fn list_fixtures_verbose(state: &AppState, entries: &[lumacore_patch::FixtureListEntry]) -> String {
    if entries.is_empty() {
        return format!("{}", Color::Yellow.paint("No fixtures patched."));
    }

    // Get programmer active values for the current user
    let programmer = state.current_programmer();

    let selected: std::collections::HashSet<u32> = programmer
        .map(|p| {
            p.selected_fixtures
                .iter()
                .filter_map(|guid| state.root.show_data.fixture_id_for_guid(*guid))
                .collect()
        })
        .unwrap_or_default();

    // Walk state for coloring
    let pool_set: std::collections::HashSet<u32> = programmer
        .and_then(|p| p.selection_pool.as_ref())
        .map(|v| {
            v.iter()
                .filter_map(|guid| state.root.show_data.fixture_id_for_guid(*guid))
                .collect()
        })
        .unwrap_or_default();
    let walked_id: Option<u32> = programmer.filter(|p| p.is_walking()).and_then(|p| {
        p.selected_fixtures
            .first()
            .and_then(|guid| state.root.show_data.fixture_id_for_guid(*guid))
    });
    let dark_yellow = Color::Rgb(180, 140, 0);

    let mut lines = Vec::new();
    lines.push(
        Style::new()
            .bold()
            .paint("Patched Fixtures (verbose):")
            .to_string(),
    );
    lines.push(String::new());

    for e in entries {
        let name = e
            .name
            .clone()
            .unwrap_or_else(|| format!("Fixture {}", e.fixture_id));
        let ft_label = format!(
            "{} {}",
            e.manufacturer,
            e.fixture_type_name.as_deref().unwrap_or("?")
        );
        let addr = format!("{}.{:03}", e.universe_id, e.start_address);

        let is_walked = walked_id == Some(e.fixture_id);
        let is_pool = !pool_set.is_empty() && pool_set.contains(&e.fixture_id);
        let is_sel = selected.contains(&e.fixture_id);

        let sel_marker = if is_walked {
            Color::Yellow.bold().paint("►").to_string()
        } else if is_pool {
            dark_yellow.paint("●").to_string()
        } else if is_sel {
            Color::Green.paint("►").to_string()
        } else {
            Style::new().dimmed().paint(" ").to_string()
        };

        // Dimmer value from programmer (active or inactive), or "—"
        let fixture_guid = state.root.show_data.fixture_guid(e.fixture_id);
        let dimmer_str = programmer
            .and_then(|p| fixture_guid.and_then(|g| p.values.get(&(g, "Dimmer".to_string()))))
            .map(|pv| {
                let readout = pv.value.readout();
                if pv.active {
                    Color::Cyan.paint(format!("Dimmer {readout}")).to_string()
                } else {
                    Style::new()
                        .dimmed()
                        .paint(format!("Dimmer {readout} [inactive]"))
                        .to_string()
                }
            })
            .unwrap_or_else(|| Style::new().dimmed().paint("—").to_string());

        // Any other programmer attributes (active shown normal, inactive dimmed)
        let active_attrs: Vec<String> = if let Some(prog) = programmer {
            prog.values
                .iter()
                .filter(|((fid, attr), _)| {
                    fixture_guid.map(|g| *fid == g).unwrap_or(false) && attr != "Dimmer"
                })
                .map(|((_, attr), pv)| {
                    let readout = pv.value.readout();
                    if pv.active {
                        format!("{attr} {readout}")
                    } else {
                        format!("{attr} {readout} [inactive]")
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let attrs_str = if active_attrs.is_empty() {
            String::new()
        } else {
            format!("  {}", Style::new().dimmed().paint(active_attrs.join(", ")))
        };

        lines.push(format!(
            "{} {} {}  {}  {}  {}{}",
            sel_marker,
            Color::Cyan.paint(format!("{:>3}", e.fixture_id)),
            Style::new().bold().paint(&name),
            Style::new().dimmed().paint(ft_label),
            Style::new().dimmed().paint(addr),
            dimmer_str,
            attrs_str,
        ));
    }

    lines.push(String::new());
    lines.push(
        Color::Green
            .paint(format!("{} fixture(s) total", entries.len()))
            .to_string(),
    );
    lines.join("\n")
}

/// Show detail/properties for a specific fixture (like info but as list).
pub fn list_fixture_detail(state: &AppState, id: u32) -> String {
    use crate::commands::info;
    info::info_fixture(state, Some(&id.to_string()))
}

/// Show detail/properties for a specific fixture type.
pub fn list_fixture_type_detail(state: &AppState, id: u32) -> String {
    use crate::commands::info;
    info::info_fixture_type(state, Some(&id.to_string()))
}

pub fn list_fixture_types(state: &AppState) -> String {
    if state.root.show_data.devices.dmx.patch.fixture_types.is_empty() {
        return format!(
            "{}",
            Color::Yellow
                .paint("No GDTF fixture types imported. Use 'Import FixtureType \"file.gdtf\"'")
        );
    }

    let rows: Vec<FixtureTypeRow> = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture_types
        .iter()
        .map(|(id, ft)| {
            let name = ft.meta.name().unwrap_or("<unnamed>").to_string();
            let manufacturer = ft.manufacturer.clone();
            let display_id = *id;

            let (mode_count, total_channels) = ft
                .revisions
                .first()
                .map(|rev| {
                    let modes = rev.modes.len();
                    let channels: u16 =
                        rev.modes.iter().map(|m| m.dmx_footprint).max().unwrap_or(0);
                    (modes, channels)
                })
                .unwrap_or((0, 0));

            let revisions_str = format!("{} rev(s), {} mode(s)", ft.revisions.len(), mode_count);

            let file = state
                .gdtf_sources
                .get(id)
                .map(|s| s.file_name.clone())
                .unwrap_or_else(|| "<unknown>".to_string());

            FixtureTypeRow {
                id: display_id,
                manufacturer,
                name,
                modes: revisions_str,
                channels: format!("up to {total_channels} ch"),
                file,
            }
        })
        .collect();

    let table = Table::new(rows).with(TableStyle::modern()).to_string();

    format!(
        "{}\n{}\n{}",
        Style::new().bold().paint("GDTF Fixture Types:"),
        table,
        Color::Green.paint(format!(
            "{} type(s) total",
            state.root.show_data.devices.dmx.patch.fixture_types.len()
        ))
    )
}

pub fn list_gdtf_files(state: &AppState) -> String {
    use tabled::Tabled;

    #[derive(Tabled)]
    struct GdtfRow {
        #[tabled(rename = "File")]
        file: String,
        #[tabled(rename = "Status")]
        status: String,
    }

    // Scan library/gdtf/ on disk
    let library_dir = std::path::Path::new("library/gdtf");
    let mut disk_files: Vec<String> = Vec::new();
    if library_dir.is_dir()
        && let Ok(entries) = lumacore_luma::drive::read_dir(library_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if crate::commands::import::is_gdtf(&path)
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                disk_files.push(name.to_string());
            }
        }
    }
    disk_files.sort();

    // Collect imported file names for status matching
    let imported_names: std::collections::HashSet<String> = state
        .gdtf_sources
        .values()
        .map(|s| s.file_name.clone())
        .collect();

    if disk_files.is_empty() && imported_names.is_empty() {
        return format!(
            "{}\n{}",
            Color::Yellow.paint("No GDTF files found in library/GDTF/"),
            Style::new()
                .dimmed()
                .paint("Place .gdtf files in library/GDTF/ or use 'Import FixtureType \"file.gdtf\"'"),
        );
    }

    // Build rows: disk files first, then any imported-only files not on disk
    let mut rows: Vec<GdtfRow> = Vec::new();

    for file in &disk_files {
        let status = if imported_names.contains(file) {
            Color::Green.paint("imported").to_string()
        } else {
            Style::new().dimmed().paint("on disk").to_string()
        };
        rows.push(GdtfRow {
            file: file.clone(),
            status,
        });
    }

    // Add imported files not found on disk
    let mut imported_only: Vec<&String> = imported_names
        .iter()
        .filter(|n| !disk_files.contains(n))
        .collect();
    imported_only.sort();
    for file in imported_only {
        rows.push(GdtfRow {
            file: file.clone(),
            status: Color::Cyan.paint("imported (external)").to_string(),
        });
    }

    let total = rows.len();
    let table = Table::new(rows).with(TableStyle::modern()).to_string();

    format!(
        "{}\n{}\n{}",
        Style::new().bold().paint("Library: GDTF"),
        table,
        Color::Green.paint(format!("{total} file(s)"))
    )
}
