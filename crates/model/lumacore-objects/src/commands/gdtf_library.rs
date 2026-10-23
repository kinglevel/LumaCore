//  ██████╗ ██████╗ ████████╗███████╗    ██╗     ██╗██████╗ ██████╗  █████╗ ██████╗ ██╗   ██╗
// ██╔════╝ ██╔══██╗╚══██╔══╝██╔════╝    ██║     ██║██╔══██╗██╔══██╗██╔══██╗██╔══██╗╚██╗ ██╔╝
// ██║  ███╗██║  ██║   ██║   █████╗      ██║     ██║██████╔╝██████╔╝███████║██████╔╝ ╚████╔╝
// ██║   ██║██║  ██║   ██║   ██╔══╝      ██║     ██║██╔══██╗██╔══██╗██╔══██║██╔══██╗  ╚██╔╝
// ╚██████╔╝██████╔╝   ██║   ██║         ███████╗██║██████╔╝██║  ██║██║  ██║██║  ██║   ██║
//  ╚═════╝ ╚═════╝    ╚═╝   ╚═╝         ╚══════╝╚═╝╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝
// FILE: gdtf_library

//! The fixture files in `library/gdtf/`, read (decisions §34 round 3 2–3: the one library file
//! index; the Insert fixtures dialog lists them beside the show's types): each file's fixture
//! type as the importer derives it and its own facts (`lumacore_gdtf::file_details`), cached by
//! name, size and time so a file is parsed once. Read through the disk door; a file that isn't
//! a fixture file keeps its error.
//!
//! Owns:          LibraryFile, library, save_upload, thumbnail.
//! Doesn't own:   the file names (import::gdtf_files), the import (Import FixtureType), the JSON
//!                (`lumacore_views::fixture_types`).
//! Entry points:  library, save_upload, read_bytes, thumbnail.
//! Decisions:     §34 round 3 2, 3 (Insert fixtures v6).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use lumacore_fixtures::fixture_type::FixtureType;

use super::import::{GDTF_DIR, gdtf_files};
use crate::errors::render_code;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One file of `library/gdtf/`.
#[derive(Clone)]
pub struct LibraryFile {
    /// Its name in the folder (`Robe/Spiider.gdtf`).
    pub file: String,
    pub size: u64,
    /// SHA-256 of the file: the same file as a show revision when equal.
    pub sha256: String,
    /// What it becomes, or why it can't.
    pub fixture_type: Result<FixtureType, String>,
    /// Its own facts (`lumacore_gdtf::FileDetails` as JSON), Null when unreadable.
    pub details: serde_json::Value,
}

// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

/// Every file of the library, read (a file unchanged since the last call comes from the cache).
pub fn library() -> Vec<LibraryFile> {
    library_in(GDTF_DIR)
}

/// Every file of the gdtf folder `dir` (a drive's too, decisions §38), read and cached by path.
pub fn library_in(dir: &str) -> Vec<LibraryFile> {
    static CACHE: OnceLock<Mutex<HashMap<String, (u64, Option<SystemTime>, LibraryFile)>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().unwrap_or_else(|p| p.into_inner());
    let files = crate::commands::import::gdtf_files_in(dir);
    let prefix = format!("{dir}/");
    cache.retain(|p, _| !p.starts_with(&prefix) || files.iter().any(|f| *p == format!("{prefix}{f}")));
    files
        .into_iter()
        .filter_map(|file| {
            let path = format!("{dir}/{file}");
            let meta = lumacore_luma::drive::metadata(&path).ok()?;
            let (size, time) = (meta.len(), meta.modified().ok());
            if let Some((s, t, read)) = cache.get(&path)
                && *s == size
                && *t == time
            {
                return Some(read.clone());
            }
            let bytes = lumacore_luma::drive::read(&path).ok()?;
            let read = LibraryFile {
                file: file.clone(),
                size,
                sha256: lumacore_gdtf::file_hash(&bytes),
                fixture_type: lumacore_gdtf::import_gdtf_bytes(&bytes).map(|(t, _)| t).map_err(|e| e.to_string()),
                details: lumacore_gdtf::file_details(&bytes).ok().and_then(|d| serde_json::to_value(d).ok()).unwrap_or_default(),
            };
            cache.insert(path, (size, time, read.clone()));
            Some(read)
        })
        .collect()
}

/// A fixture file's type as the importer derives it (tests and the upload's check).
pub fn read_bytes(bytes: &[u8]) -> Result<FixtureType, String> {
    lumacore_gdtf::import_gdtf_bytes(bytes).map(|(t, _)| t).map_err(|e| e.to_string())
}

/// A fixture file's thumbnail (Insert fixtures' Details, round 3): a library file by name, or a
/// show revision's file by its SHA-256 (the show's store).
pub fn thumbnail(file: Option<&str>, sha256: Option<&str>) -> Option<(Vec<u8>, &'static str)> {
    let bytes = match (file, sha256) {
        (Some(f), _) if gdtf_files().iter().any(|x| x == f) => lumacore_luma::drive::read(format!("{GDTF_DIR}/{f}")).ok()?,
        (_, Some(h)) => lumacore_luma::show_file::stored_gdtf(h)?,
        _ => return None,
    };
    lumacore_gdtf::thumbnail(&bytes)
}

/// Upload from this device (round 3 2): a fixture file into `library/gdtf/` under its own name
/// (letters, digits, `.-_ `), refused unless the importer reads it. The file's name back.
pub fn save_upload(name: &str, bytes: &[u8]) -> Result<String, String> {
    let base = std::path::Path::new(name).file_name().and_then(|n| n.to_str()).unwrap_or("");
    let safe: String = base.chars().map(|c| if c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') { c } else { '_' }).collect();
    let safe = safe.trim().trim_start_matches('.').to_string();
    if !safe.to_lowercase().ends_with(".gdtf") || safe.len() <= ".gdtf".len() {
        return Err(render_code("Upload a fixture file: name.gdtf.", "E-CMD-008"));
    }
    lumacore_gdtf::import_gdtf_bytes(bytes).map_err(|e| render_code(format!("{safe} isn't a fixture file the importer reads: {e}"), "E-CMD-012"))?;
    lumacore_luma::drive::create_dir_all(GDTF_DIR).map_err(|e| render_code(e.to_string(), "E-CMD-012"))?;
    lumacore_luma::drive::write_atomic(format!("{GDTF_DIR}/{safe}"), bytes).map_err(|e| render_code(e.to_string(), "E-CMD-012"))?;
    Ok(safe)
}
