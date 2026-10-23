// ███╗   ███╗███████╗██████╗ ██╗ █████╗
// ████╗ ████║██╔════╝██╔══██╗██║██╔══██╗
// ██╔████╔██║█████╗  ██║  ██║██║███████║
// ██║╚██╔╝██║██╔══╝  ██║  ██║██║██╔══██║
// ██║ ╚═╝ ██║███████╗██████╔╝██║██║  ██║
// ╚═╝     ╚═╝╚══════╝╚═════╝ ╚═╝╚═╝  ╚═╝
// FILE: media

//! The station's media (decisions §30 "12a2 / 12a3 hardening", §8).
//!
//! - `library/media/` holds files to import (`Import Image "stage.jpg"`; a
//!   browser upload lands there too).
//! - `library/media/.store/<sha256>.<ext>` is the content-addressed store: one
//!   copy of every embedded picture this station has seen (a show load
//!   unpacks into it, a save packs from it, LumaSession members fetch into it
//!   by hash, `GET /api/media/<sha256>` serves from it).
//!
//! Everything goes through `crate::drive`.
//!
//! Owns:          the media folder and its content-addressed store: import, hashes, store paths.
//! Doesn't own:   media objects and their commands (objects), assets on asset drives (§8).
//! Entry points:  store, stored, sha, read_from_store, keep_in_store.
//! Decisions:     §8, §30 "12a2", §34 25.10.

use lumacore_tree::MediaFile;
use lumacore_tree::media::{IMAGE_EXTENSIONS, MAX_FILE_BYTES, MAX_SHOW_BYTES};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Files to import.
pub const LIBRARY: &str = "library/media";

/// The store, by SHA-256.
pub const STORE: &str = "library/media/.store";

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The import folder (tests keep theirs out of the repo).
pub fn library_dir() -> String {
    if cfg!(test) {
        std::env::temp_dir().join("lumacore-test-library-media").to_string_lossy().into_owned()
    } else {
        LIBRARY.to_string()
    }
}

fn store_dir() -> String {
    if cfg!(test) {
        std::env::temp_dir().join("lumacore-test-media").to_string_lossy().into_owned()
    } else {
        STORE.to_string()
    }
}

/// The SHA-256 of `bytes` (lowercase hex).
pub fn sha256(bytes: &[u8]) -> String {
    lumacore_gdtf::file_hash(bytes)
}

pub fn store_path(sha: &str, ext: &str) -> String {
    format!("{}/{sha}.{ext}", store_dir())
}

/// A hash is 64 hex digits (never a path).
pub fn is_hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Keep `bytes` in the store; returns the SHA-256.
pub fn store(bytes: &[u8], ext: &str) -> Result<String, String> {
    let sha = sha256(bytes);
    keep_in_store(&store_dir(), &store_path(&sha, ext), &sha, bytes)?;
    Ok(sha)
}

/// Put `bytes` (whose SHA-256 is `sha`) at `path` in a content-addressed
/// store unless a whole copy is already there; a store file whose bytes no
/// longer match its name is written again (R1-15).
pub fn keep_in_store(dir: &str, path: &str, sha: &str, bytes: &[u8]) -> Result<(), String> {
    let whole = crate::drive::read(path).is_ok_and(|have| sha256(&have).eq_ignore_ascii_case(sha));
    if !whole {
        crate::drive::create_dir_all(dir).map_err(|e| format!("Cannot create {dir}: {e}"))?;
        crate::drive::write_atomic(path, bytes).map_err(|e| format!("Cannot write {path}: {e}"))?;
    }
    Ok(())
}

/// Read a store file and check it against its name: a damaged one (cut
/// short, changed) is removed, the System Monitor warns and the caller sees
/// it as missing — it never reaches a show file under a new hash (R1-15).
pub fn read_from_store(path: &str, sha: &str) -> Option<Vec<u8>> {
    let bytes = crate::drive::read(path).ok()?;
    if sha256(&bytes).eq_ignore_ascii_case(sha) {
        return Some(bytes);
    }
    let _ = crate::drive::remove_file(path);
    lumacore_sysmon::sysmon_note(
        "Media",
        format!("[WARN] store file {path} was damaged (its bytes don't match its hash); removed — the show lists it as missing"),
    );
    None
}

/// A file from the store.
pub fn stored(sha: &str, ext: &str) -> Option<Vec<u8>> {
    if !is_hash(sha) {
        return None;
    }
    read_from_store(&store_path(sha, ext), sha)
}

/// A file from the store by hash alone (any extension it was stored with).
pub fn stored_any(sha: &str) -> Option<(Vec<u8>, String)> {
    if !is_hash(sha) {
        return None;
    }
    IMAGE_EXTENSIONS
        .iter()
        .find_map(|ext| stored(sha, ext).map(|b| (b, (*ext).to_string())))
}

/// Whether the store has the file.
pub fn has(sha: &str, ext: &str) -> bool {
    is_hash(sha) && crate::drive::metadata(store_path(sha, ext)).is_ok()
}

/// Every picture a show embeds: the show's Images and each profile's Icons
/// and Images.
pub fn files_of(show: &lumacore_tree::ShowData) -> Vec<&MediaFile> {
    show.images
        .values()
        .chain(
            show.user_profiles
                .values()
                .flat_map(|p| p.content.icons.values().chain(p.content.images.values())),
        )
        .collect()
}

/// The pictures of the show this station's store doesn't have (a member
/// fetches them from its master).
pub fn missing(show: &lumacore_tree::ShowData) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = files_of(show)
        .into_iter()
        .filter(|f| !has(&f.sha256, &f.ext))
        .map(|f| (f.sha256.clone(), f.ext.clone()))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Bytes the show embeds: every distinct picture and GDTF file.
pub fn embedded_bytes(show: &lumacore_tree::ShowData) -> u64 {
    let mut seen = std::collections::BTreeSet::new();
    let pictures: u64 = files_of(show)
        .into_iter()
        .filter(|f| seen.insert(f.sha256.clone()))
        .map(|f| f.size)
        .sum();
    let gdtf: u64 = show
        .devices
        .dmx
        .patch
        .fixture_types
        .values()
        .flat_map(|ft| &ft.revisions)
        .filter(|r| !r.file_hash.is_empty() && seen.insert(r.file_hash.clone()))
        .filter_map(|r| crate::show_file::stored_gdtf(&r.file_hash))
        .map(|b| b.len() as u64)
        .sum();
    pictures + gdtf
}

/// Read `file` from `library/media/` and make its manifest.
/// Refuses what isn't a picture, a file over 16 MB, and a show that would
/// embed more than 256 MB (E-MEDIA-…).
pub fn import_file(
    show: &lumacore_tree::ShowData,
    file: &str,
    name: Option<&str>,
) -> Result<MediaFile, String> {
    import_file_in(show, &library_dir(), file, name)
}

/// `import_file` from `dir` (a drive's media folder, decisions §38).
pub fn import_file_in(
    show: &lumacore_tree::ShowData,
    dir: &str,
    file: &str,
    name: Option<&str>,
) -> Result<MediaFile, String> {
    use lumacore_base::with_code;
    // A file in library/media/ (sub-folders allowed; issue 07).
    let path = crate::drive::library_file(dir, file).map_err(|e| with_code(e, "E-MEDIA-001"))?;
    let file = file.trim_matches('"');
    let ext = std::path::Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();
    let ext = if ext == "jpeg" { "jpg".to_string() } else { ext };
    if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Err(with_code(
            format!("'{file}' is not a picture ({}).", IMAGE_EXTENSIONS.join(", ")),
            "E-MEDIA-002",
        ));
    }
    let bytes = crate::drive::read(&path)
        .map_err(|e| with_code(format!("Cannot read {}: {e}", path.display()), "E-MEDIA-001"))?;
    let size = bytes.len() as u64;
    if size > MAX_FILE_BYTES {
        return Err(with_code(
            format!(
                "{file} is {} MB; an embedded file may be 16 MB at most. Large media belongs on an asset drive (AssetRoot).",
                size / (1024 * 1024)
            ),
            "E-MEDIA-003",
        ));
    }
    let sha = sha256(&bytes);
    let already = files_of(show).iter().any(|f| f.sha256 == sha);
    if !already && embedded_bytes(show) + size > MAX_SHOW_BYTES {
        return Err(with_code(
            "The show would embed more than 256 MB. Large media belongs on an asset drive (AssetRoot).",
            "E-MEDIA-004",
        ));
    }
    let (width, height) = lumacore_tree::media::image_size(&bytes, &ext).ok_or_else(|| {
        with_code(format!("{file} is not a readable {ext} picture."), "E-MEDIA-002")
    })?;
    store(&bytes, &ext)?;
    let stem = std::path::Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file);
    Ok(MediaFile {
        meta: lumacore_base::NodeMeta::new(Some(name.unwrap_or(stem).to_string())),
        sha256: sha,
        ext,
        size,
        width,
        height,
        source: file.to_string(),
    })
}

/// Put an upload into `library/media/` under a safe name; returns the name.
pub fn save_upload(name: &str, bytes: &[u8]) -> Result<String, String> {
    let base = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload");
    let safe: String = base
        .chars()
        .map(|c| if c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') { c } else { '_' })
        .collect();
    let safe = safe.trim().trim_start_matches('.').to_string();
    if safe.is_empty() {
        return Err("No file name".into());
    }
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(lumacore_base::with_code(
            "An embedded file may be 16 MB at most.",
            "E-MEDIA-003",
        ));
    }
    let dir = library_dir();
    crate::drive::create_dir_all(&dir).map_err(|e| e.to_string())?;
    crate::drive::write_atomic(format!("{dir}/{safe}"), bytes).map_err(|e| e.to_string())?;
    Ok(safe)
}
