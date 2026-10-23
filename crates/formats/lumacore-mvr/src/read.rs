// ██████╗ ███████╗ █████╗ ██████╗
// ██╔══██╗██╔════╝██╔══██╗██╔══██╗
// ██████╔╝█████╗  ███████║██║  ██║
// ██╔══██╗██╔══╝  ██╔══██║██║  ██║
// ██║  ██║███████╗██║  ██║██████╔╝
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝
// FILE: read

//! The zip walk: the entries listed, `GeneralSceneDescription.xml` read (at the root, any case),
//! the scene parsed. `read_from` takes any reader so a big file on a drive is never read whole for
//! its head; `read` takes bytes (an upload, a test).
//!
//! Owns:          read, read_from, head, head_from, SCENE_FILE.
//! Doesn't own:   where the file comes from (the caller's door), what the scene means.
//! Entry points:  read, read_from, head, head_from.
//! Decisions:     §34 "34g4 patch hardened" 10.

use std::io::{Cursor, Read, Seek};

use crate::scene::{MvrFile, MvrHead, MvrScene};
use crate::{MvrError, xml};


// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

/// The scene file every MVR has at its root.
pub const SCENE_FILE: &str = "GeneralSceneDescription.xml";

/// The file from bytes.
pub fn read(bytes: &[u8]) -> Result<MvrFile, MvrError> {
    read_from(Cursor::new(bytes))
}

/// The file from any seekable reader (a file on a drive: only the scene entry is read).
pub fn read_from<R: Read + Seek>(reader: R) -> Result<MvrFile, MvrError> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| MvrError::NotAZip(e.to_string()))?;
    let entries: Vec<String> = zip.file_names().map(str::to_string).collect();
    let scene_name = entries
        .iter()
        .find(|e| e.eq_ignore_ascii_case(SCENE_FILE))
        .cloned()
        .ok_or(MvrError::NoScene)?;
    let mut text = Vec::new();
    zip.by_name(&scene_name)
        .map_err(|e| MvrError::NotAZip(e.to_string()))?
        .read_to_end(&mut text)
        .map_err(|e| MvrError::BadXml(e.to_string()))?;
    let root = xml::parse(&text)?;
    Ok(MvrFile { scene: MvrScene::from_xml(&root)?, entries })
}

/// The head of a file from bytes.
pub fn head(bytes: &[u8]) -> Result<MvrHead, MvrError> {
    Ok(read(bytes)?.head())
}

/// The head of a file from a reader.
pub fn head_from<R: Read + Seek>(reader: R) -> Result<MvrHead, MvrError> {
    Ok(read_from(reader)?.head())
}

/// The bytes of the GDTF entry a fixture names (`GDTFSpec`, by file name in any folder, any
/// case), or None when the zip lacks it.
pub fn gdtf_entry<R: Read + Seek>(reader: R, spec: &str) -> Result<Option<Vec<u8>>, MvrError> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| MvrError::NotAZip(e.to_string()))?;
    let want = spec.trim().to_lowercase();
    let want = want.rsplit('/').next().unwrap_or(&want).to_string();
    let want = if want.ends_with(".gdtf") { want } else { format!("{want}.gdtf") };
    let name = zip.file_names().find(|e| e.to_lowercase().rsplit('/').next() == Some(want.as_str())).map(str::to_string);
    let Some(name) = name else { return Ok(None) };
    let mut bytes = Vec::new();
    zip.by_name(&name)
        .map_err(|e| MvrError::NotAZip(e.to_string()))?
        .read_to_end(&mut bytes)
        .map_err(|e| MvrError::NotAZip(e.to_string()))?;
    Ok(Some(bytes))
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
