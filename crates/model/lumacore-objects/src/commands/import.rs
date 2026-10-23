//! Import of GDTF files: `Import FixtureType "file.gdtf"` (decisions §34 round 3 2; a file of
//! library/gdtf/, sub-folders allowed): a new type, or a new revision of the type with the same
//! GDTF FixtureTypeID — that changes no fixture, the reply offers the line that would. The bare
//! `Import "x"` / `Import library` / `Import GDTF` went (round 3 2).

use nu_ansi_term::{Color, Style};
use std::path::Path;

use lumacore_gdtf::ChannelMap;
use lumacore_syntax::ast::{Id, Item, Range, Target};

use crate::errors::render_code;
use crate::text::{
    render_channel_map_grouped, render_channel_map_offset, render_din_warnings, render_success,
};
use lumacore_fixtures::fixture_type::FixtureType;

use crate::{AppState, DispatchResult, FixtureTypeSource};

/// Where `Import FixtureType "file.gdtf"` reads from.
pub(crate) const GDTF_DIR: &str = "library/gdtf";

/// Every `.gdtf` file of library/gdtf/ (sub-folders too) as a line names it, relative and sorted:
/// the library file index's GDTF kind (decisions §34 round 3 3; `/FixtureType <Tab>`, the picker).
pub fn gdtf_files() -> Vec<String> {
    gdtf_files_in(GDTF_DIR)
}

/// Every `.gdtf` under `dir`, relative (`Robe/Spiider.gdtf`): a drive's gdtf folder too.
pub fn gdtf_files_in(dir: &str) -> Vec<String> {
    fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) {
        let Ok(entries) = lumacore_luma::drive::read_dir(dir) else { return };
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            let here = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if path.is_dir() && !name.starts_with('.') {
                walk(&path, &here, out);
            } else if is_gdtf(&path) {
                out.push(here);
            }
        }
    }
    let mut out = Vec::new();
    walk(Path::new(dir), "", &mut out);
    out.sort_by_key(|n| n.to_lowercase());
    out
}

/// `Import <object> …` without one: the bare file form went (round 3 2), Import names its kind.
pub fn handle_import(_: &mut AppState, args: &[&str]) -> String {
    let file = args.iter().find(|a| a.starts_with('"')).copied().unwrap_or("\"file.gdtf\"");
    render_code(
        format!("Import names what it brings in: Import FixtureType {file} | Import Image … | Import UserProfile … | Import Show …"),
        "E-CMD-006",
    )
}

/// `Import FixtureType "file.gdtf"` (the Devices claim: the type joins the patch).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Import", object = "FixtureType")]
pub struct ImportFixtureType {
    #[lum(target)]
    pub target: Option<Target>,
    /// `/Drive "usb1"`: the file on that drive's library (decisions §38).
    #[lum(option = "Drive")]
    pub drive: Option<lumacore_syntax::ast::Value>,
}

impl crate::commands::typed::Command for ImportFixtureType {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = crate::objects::patch_claimed(state) {
            return e;
        }
        let drive = self.drive.as_ref().and_then(|v| match v {
            lumacore_syntax::ast::Value::Text(t) => Some(t.node.clone()),
            _ => None,
        });
        let Some(target) = self.target.as_ref() else {
            // The station's "which file?" ask (decisions §38 5): the files element on gdtf/.
            state.asking = Some(
                crate::dialogs::Ask::new("Import FixtureType", "A GDTF file of the library becomes a fixture type of the show.")
                    .file("file", "File", "gdtf")
                    .button("Import", lumacore_wire::ButtonKind::Primary, "Import FixtureType {file}")
                    .cancel("Cancel")
                    .flags("the file: Import FixtureType \"file.gdtf\" [/Drive \"usb1\"]"),
            );
            return crate::objects::out(render_code("Import FixtureType \"file.gdtf\": a file of library/gdtf/.", "E-CMD-006"));
        };
        let file = match target.items.as_slice() {
            [(None, Item::Path(segs))] => segs.last().and_then(|s| s.ids.as_ref()).and_then(|ids| match ids.ranges.as_slice() {
                [(None, Range::One(Id::Name(n)))] => Some(n.node.clone()),
                _ => None,
            }),
            _ => None,
        };
        let Some(file) = file else {
            return crate::objects::out(render_code("Import FixtureType \"file.gdtf\": a file of library/gdtf/.", "E-CMD-006"));
        };
        let folder = match state.library_folder(drive.as_deref(), "gdtf", false) {
            Ok(f) => f,
            Err(e) => return crate::objects::out(e),
        };
        let path = match lumacore_luma::drive::library_file(&folder, &file) {
            Ok(path) => path,
            // A quoted path that leaves library/gdtf/ is a bad value (issue 07).
            Err(e) => return crate::objects::out(render_code(e, "E-CMD-008")),
        };
        crate::objects::out(import_gdtf_file(state, &path))
    }
}

/// What `insert_gdtf_bytes` did with a file.
pub enum Inserted {
    /// The show has this very file as a revision of that type.
    SameFile { no: u32, revision: usize },
    /// A new revision of the type the show has (the same GDTF FixtureTypeID).
    NewRevision { no: u32, revision: usize },
    /// A new fixture type.
    NewType { no: u32, fixture_type: FixtureType, warnings: Vec<lumacore_gdtf::GdtfWarning> },
}

impl Inserted {
    /// The type's number and the revision's index the file is now.
    pub fn at(&self) -> (u32, usize) {
        match self {
            Self::SameFile { no, revision } | Self::NewRevision { no, revision } => (*no, *revision),
            Self::NewType { no, .. } => (*no, 0),
        }
    }
}

/// The one door for a GDTF's bytes into the show (an upload, a library file, a file an MVR
/// embeds): capped, stored by hash, the same file found as a revision, the same FixtureTypeID a
/// new revision of that type, else a new type at the next number (`file_name` is remembered for
/// `List FixtureType`).
pub fn insert_gdtf_bytes(state: &mut AppState, bytes: &[u8], file_name: &str) -> Result<Inserted, String> {
    // Embedded files are capped (decisions §30 "12a2").
    if bytes.len() as u64 > lumacore_tree::media::MAX_FILE_BYTES {
        return Err(render_code(
            format!("{file_name} is {} MB; an embedded file may be 16 MB at most.", bytes.len() / (1024 * 1024)),
            "E-MEDIA-003",
        ));
    }
    let hash = lumacore_gdtf::file_hash(bytes);
    let patch = &state.root.show_data.devices.dmx.patch;
    if let Some((no, i)) = patch.fixture_types.iter().find_map(|(no, ft)| ft.revisions.iter().position(|r| r.file_hash == hash).map(|i| (*no, i))) {
        return Ok(Inserted::SameFile { no, revision: i });
    }
    if lumacore_luma::media::embedded_bytes(&state.root.show_data) + bytes.len() as u64 > lumacore_tree::media::MAX_SHOW_BYTES {
        return Err(render_code("The show would embed more than 256 MB.", "E-MEDIA-004"));
    }
    let (fixture_type, warnings) = lumacore_gdtf::import_gdtf_bytes(bytes)
        .and_then(|result| {
            lumacore_luma::show_file::store_gdtf(bytes).map_err(lumacore_gdtf::GdtfError::InvalidGdtf)?;
            Ok(result)
        })
        .map_err(|e| render_code(format!("Import failed: {e}"), "E-CMD-008"))?;
    // The same GDTF fixture type again (its FixtureTypeID): a new revision of the type the show
    // has, never a second type (decisions §30 "12a2").
    if let Some(id) = fixture_type.gdtf_fixture_type_id
        && let Some((&no, existing)) = state.root.show_data.devices.dmx.patch.fixture_types.iter_mut().find(|(_, ft)| ft.gdtf_fixture_type_id == Some(id)).map(|(no, ft)| (no, std::sync::Arc::make_mut(ft)))
    {
        existing.revisions.push(fixture_type.revisions[0].clone());
        state.show_dirty = true;
        return Ok(Inserted::NewRevision { no, revision: existing.revisions.len() - 1 });
    }
    let patch = &mut state.root.show_data.devices.dmx.patch;
    let no = patch.fixture_types.keys().max().copied().unwrap_or(0) + 1;
    patch.fixture_types.insert(no, std::sync::Arc::new(fixture_type.clone()));
    state.gdtf_sources.insert(no, FixtureTypeSource { file_name: file_name.to_string() });
    Ok(Inserted::NewType { no, fixture_type, warnings })
}

/// Import the GDTF file at `path` (a typed line has already made it a file in
/// library/gdtf/; the demo builder passes its own).
pub fn import_gdtf_file(state: &mut AppState, path: &Path) -> String {
    let path_str = path.display().to_string();

    // Through the drive door; the file itself is kept in the station's store
    // and travels in the show (GDTF native, decisions §29).
    let bytes = match lumacore_luma::drive::read(path) {
        Ok(b) => b,
        Err(e) => return render_code(format!("Import failed: {e}"), "E-CMD-012"),
    };
    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path_str.to_string());
    let inserted = match insert_gdtf_bytes(state, &bytes, &file_name) {
        Ok(i) => i,
        Err(e) => return e,
    };
    let type_name = |state: &AppState, no: u32| state.root.show_data.devices.dmx.patch.fixture_types.get(&no).and_then(|t| t.meta.name().map(str::to_string)).unwrap_or_else(|| "<unnamed>".into());
    match inserted {
        Inserted::SameFile { no, revision } => render_success(&format!("FixtureType {no} \"{}\" already has this file (revision {}).", type_name(state, no), revision + 1)),
        Inserted::NewRevision { no, revision } => {
            let rev_no = revision + 1;
            let type_guid = state.root.show_data.devices.dmx.patch.fixture_types[&no].meta.identity.guid;
            // It changes no fixture: the reply offers the line that would (decisions round 3 5).
            let users: Vec<u32> = state.root.show_data.devices.dmx.patch.fixtures.values().filter(|f| f.mvr.fixture_type == type_guid).filter_map(|f| f.mvr.fid).collect();
            let offer = if users.is_empty() {
                String::new()
            } else {
                format!(" Fixtures keep theirs; to change them: Set Fixture {} \"Revision\" {rev_no}", crate::commands::door::numbers_text(&users))
            };
            render_success(&format!("FixtureType {no} \"{}\": new revision {rev_no} (same GDTF FixtureTypeID).{offer}", type_name(state, no)))
        }
        Inserted::NewType { no: next_id, fixture_type, warnings } => {
            let name = fixture_type.meta.name().unwrap_or("<unnamed>").to_string();
            let manufacturer = fixture_type.manufacturer.clone();

            // Count modes and channels for display
            let mode_summary: Vec<String> = fixture_type
                .revisions
                .iter()
                .flat_map(|rev| {
                    rev.modes.iter().map(|mode| {
                        let mode_name = mode.meta.name().unwrap_or("<unnamed>").to_string();
                        format!("  Mode: {} ({} channels)", mode_name, mode.dmx_footprint)
                    })
                })
                .collect();

            let mut output = String::new();

            // Header
            output.push_str(&format!(
                "{} {}\n{} {}\n{} {}\n{}\n",
                Style::new().bold().paint("Imported:"),
                Color::Green.paint(&name),
                Style::new().bold().paint("Manufacturer:"),
                manufacturer,
                Style::new().bold().paint("ID:"),
                Color::Cyan.paint(next_id.to_string()),
                Style::new().bold().paint("Modes:")
            ));

            for ms in &mode_summary {
                output.push_str(&format!("{ms}\n"));
            }

            // Render channel maps for all modes in the first revision
            if let Some(rev) = fixture_type.revisions.first() {
                for mode in &rev.modes {
                    let mode_name = mode.meta.name().unwrap_or("<unnamed>").to_string();

                    output.push('\n');
                    output.push_str(&format!(
                        "{}\n",
                        Style::new()
                            .bold()
                            .paint("--- Channel Map (by DMX offset) ---")
                    ));

                    let channels_by_offset = ChannelMap::by_dmx_offset(mode);
                    // Convert refs to owned for renderer
                    let owned: Vec<lumacore_fixtures::ChannelFunction> =
                        channels_by_offset.iter().map(|cf| (*cf).clone()).collect();
                    output.push_str(&render_channel_map_offset(&mode_name, &owned));

                    output.push('\n');
                    output.push_str(&format!(
                        "{}\n",
                        Style::new()
                            .bold()
                            .paint("--- Channel Map (grouped by type) ---")
                    ));

                    let grouped = ChannelMap::grouped_by_type(mode);
                    // Convert to owned
                    let owned_groups: std::collections::BTreeMap<
                        String,
                        Vec<lumacore_fixtures::ChannelFunction>,
                    > = grouped
                        .iter()
                        .map(|(k, v)| (k.clone(), v.iter().map(|cf| (*cf).clone()).collect()))
                        .collect();
                    output.push_str(&render_channel_map_grouped(&mode_name, &owned_groups));
                }
            }

            // DIN SPEC warnings
            let warnings_text = render_din_warnings(&warnings);
            if !warnings_text.is_empty() {
                output.push('\n');
                output.push_str(&warnings_text);
            }

            output.push_str(&render_success(&format!(
                "Import complete. Use 'Patch \"{name}\" At U.AAA' to patch a fixture."
            )));

            output
        }

    }
}

/// A GDTF file by its extension, any case (`Strobe.GDTF` too; R1-54).
pub fn is_gdtf(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("gdtf"))
}

#[cfg(test)]
mod tests {
    use super::is_gdtf;
    use std::path::Path;

    #[test]
    fn gdtf_files_are_found_whatever_the_case_of_the_extension() {
        assert!(is_gdtf(Path::new("library/gdtf/Strobe.GDTF")));
        assert!(is_gdtf(Path::new("AuraXB.gdtf")));
        assert!(is_gdtf(Path::new("x.Gdtf")));
        assert!(!is_gdtf(Path::new("x.gdtf.zip")));
        assert!(!is_gdtf(Path::new("gdtf")));
    }
}
