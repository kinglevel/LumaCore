// ██████╗ ███████╗████████╗ █████╗ ██╗██╗     ███████╗
// ██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██║██║     ██╔════╝
// ██║  ██║█████╗     ██║   ███████║██║██║     ███████╗
// ██║  ██║██╔══╝     ██║   ██╔══██║██║██║     ╚════██║
// ██████╔╝███████╗   ██║   ██║  ██║██║███████╗███████║
// ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚══════╝╚══════╝
// FILE: details

//! A fixture file's own facts for the Insert fixtures dialog's Details (decisions §34 round 3,
//! `insert-fixtures.html`): its names, description, the type's id, the GDTF version and every
//! revision's text, date and uploader — read from `description.xml`, nothing derived.
//!
//! Owns:          FileDetails, file_details, thumbnail.
//! Doesn't own:   the fixture type it becomes (importer), the file on disk (the caller's).
//! Entry points:  file_details, thumbnail.
//! Decisions:     §34 round 3 (Insert fixtures v6).

use std::io::Cursor;

use gdtf::GdtfFile;
use serde::Serialize;

use crate::errors::GdtfError;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a fixture file says about itself.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FileDetails {
    pub short_name: String,
    pub long_name: String,
    pub manufacturer: String,
    pub description: String,
    /// The type's own id (`FixtureTypeID`), empty when nil.
    pub fixture_type_id: String,
    pub gdtf_version: String,
    /// Oldest first, as the file lists them.
    pub revisions: Vec<FileRevision>,
    /// The thumbnail's name inside the archive, when it has one.
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FileRevision {
    pub text: String,
    pub date: String,
    pub user_id: i32,
}

// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

/// The facts of the file in `bytes`.
pub fn file_details(bytes: &[u8]) -> Result<FileDetails, GdtfError> {
    let file = GdtfFile::new(Cursor::new(bytes.to_vec())).map_err(|e| GdtfError::InvalidGdtf(e.to_string()))?;
    let ft = file.description.fixture_types.first().ok_or_else(|| GdtfError::InvalidGdtf("GDTF file contains no fixture types".to_string()))?;
    let id = uuid::Uuid::from_bytes(*ft.fixture_type_id.as_bytes());
    Ok(FileDetails {
        short_name: ft.short_name.clone(),
        long_name: ft.long_name.clone(),
        manufacturer: ft.manufacturer.clone(),
        description: ft.description.clone(),
        fixture_type_id: if id.is_nil() { String::new() } else { id.to_string() },
        gdtf_version: format!("{}.{}", file.description.data_version.major, file.description.data_version.minor),
        revisions: ft.revisions.iter().map(|r| FileRevision { text: r.text.clone(), date: r.date.clone(), user_id: r.user_id }).collect(),
        thumbnail: ft.thumbnail.clone(),
    })
}

/// The file's thumbnail (its `Thumbnail` name beside `description.xml`): the PNG, else the SVG,
/// with its media type.
pub fn thumbnail(bytes: &[u8]) -> Option<(Vec<u8>, &'static str)> {
    let name = file_details(bytes).ok()?.thumbnail.filter(|n| !n.is_empty())?;
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    for (ext, mime) in [("png", "image/png"), ("svg", "image/svg+xml")] {
        if let Ok(mut f) = zip.by_name(&format!("{name}.{ext}")) {
            let mut out = Vec::new();
            std::io::Read::read_to_end(&mut f, &mut out).ok()?;
            return Some((out, mime));
        }
    }
    None
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
