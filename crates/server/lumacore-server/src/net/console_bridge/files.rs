// ███████╗██╗██╗     ███████╗███████╗
// ██╔════╝██║██║     ██╔════╝██╔════╝
// █████╗  ██║██║     █████╗  ███████╗
// ██╔══╝  ██║██║     ██╔══╝  ╚════██║
// ██║     ██║███████╗███████╗███████║
// ╚═╝     ╚═╝╚══════╝╚══════╝╚══════╝
// FILE: files

//! The `files` feed and This device's upload on the station (decisions §38 5): a drive's
//! library folder by kind, and a file from the browser into one.
//!
//! Owns:          files (the rows), save_upload (the write), the kinds' file filters, a drive's library root from the station view.
//! Doesn't own:   the feed (lumacore-http), the drive table (lumacore-state), the first write's rule (`AppState::ensure_library`).
//! Entry points:  files, save_upload.
//! Decisions:     §38 5, 6.

use serde_json::{Value, json};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The `files` feed: `station` is the published station view (its `drives` rows).
pub(super) fn files(station: &Value, drive: &str, folder: &str, kind: &str) -> Value {
    let Some(root) = drive_library_root(station, drive) else {
        return json!({ "drive": drive, "folder": folder, "kind": kind, "files": [], "error": format!("Drive \"{drive}\" is unknown or out") });
    };
    let dir = if folder.is_empty() {
        root.clone()
    } else {
        match lumacore_luma::drive::library_file(&root.to_string_lossy(), folder) {
            Ok(p) => p,
            Err(e) => return json!({ "drive": drive, "folder": folder, "kind": kind, "files": [], "error": e }),
        }
    };
    // A GDTF's own facts, read once per file and cached (the gdtf library reader on this folder).
    let gdtf = (kind == "gdtf").then(|| lumacore_objects::commands::gdtf_library::library_in(&root.join("gdtf").to_string_lossy()));
    // An MVR's head, read once per file and cached: scene, layers, fixtures, the GDTFs it holds.
    let mvr = (kind == "mvr").then(|| lumacore_objects::commands::mvr_library::heads_in(&root.join("mvr").to_string_lossy()));
    let rel_folder = folder.strip_prefix(kind).map(|f| f.trim_start_matches('/')).unwrap_or(folder);
    let mut rows: Vec<Value> = lumacore_luma::drive::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                return None;
            }
            let meta = e.metadata().ok()?;
            let is_dir = meta.is_dir();
            if !is_dir && !file_of_kind(kind, &name) {
                return None;
            }
            let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_millis() as u64);
            let mut row = json!({ "name": name, "file": name, "dir": is_dir, "size": meta.len(), "modified": modified });
            if kind == "shows" && !is_dir {
                // A show's facts from its manifest (decisions §30 "12d part 5").
                row["name"] = Value::String(name.trim_end_matches(".luma").to_string());
                if let Some((m, _, _)) = lumacore_luma::backups::manifest(&e.path()) {
                    row["format"] = m["format"].clone();
                    row["saved_by"] = m["user"].clone();
                    row["station"] = m["station"].clone();
                    row["saved_at"] = m["saved_at"].clone();
                    row["note"] = m["note"].clone();
                }
            }
            if let (Some(lib), false) = (&gdtf, is_dir) {
                let rel = if rel_folder.is_empty() { name.clone() } else { format!("{rel_folder}/{name}") };
                if let Some(f) = lib.iter().find(|f| f.file == rel) {
                    match &f.fixture_type {
                        Ok(t) => {
                            let modes: Vec<String> = t.revisions.last().map(|r| r.modes.iter().map(|m| format!("{} · {} ch", m.meta.identity.name.clone().unwrap_or_default(), m.dmx_footprint)).collect()).unwrap_or_default();
                            row["manufacturer"] = Value::String(t.manufacturer.clone());
                            row["type"] = Value::String(t.meta.identity.name.clone().unwrap_or_default());
                            row["modes"] = json!(modes.len());
                            row["modes_text"] = Value::String(modes.join(", "));
                            row["revision"] = f.details["revisions"].as_array().and_then(|r| r.last()).map(|r| r["text"].clone()).unwrap_or_default();
                        }
                        Err(e) => row["error"] = Value::String(e.clone()),
                    }
                }
            }
            if let (Some(heads), false) = (&mvr, is_dir) {
                let rel = if rel_folder.is_empty() { name.clone() } else { format!("{rel_folder}/{name}") };
                if let Some((_, head)) = heads.iter().find(|(f, _)| *f == rel) {
                    match head {
                        Ok(h) => {
                            row["scene"] = Value::String(format!("{} {}", h.provider, h.provider_version).trim().to_string());
                            row["layers"] = json!(h.layers);
                            row["groups"] = json!(h.groups);
                            row["fixtures"] = json!(h.fixtures);
                            row["gdtf_embedded"] = json!(h.gdtf_embedded);
                            row["gdtf_missing"] = json!(h.gdtf_missing.len());
                            row["gdtf_text"] = Value::String(match (h.gdtf_embedded, h.gdtf_missing.len()) {
                                (0, 0) => "none".to_string(),
                                (e, 0) => format!("{e} embedded"),
                                (0, m) => format!("{m} missing"),
                                (e, m) => format!("{e} embedded · {m} missing"),
                            });
                        }
                        Err(e) => row["error"] = Value::String(e.clone()),
                    }
                }
            }
            Some(row)
        })
        .collect();
    rows.sort_by(|a, b| b["dir"].as_bool().cmp(&a["dir"].as_bool()).then_with(|| a["name"].as_str().unwrap_or("").to_lowercase().cmp(&b["name"].as_str().unwrap_or("").to_lowercase())));
    json!({ "drive": drive, "folder": folder, "kind": kind, "path": dir.to_string_lossy(), "files": rows })
}

/// This device's upload: Setup and up; `root()` asks the state thread for the drive's library
/// (made on this first write); the file lands in `folder` under its own name.
pub(super) fn save_upload(
    rights: lumacore_tree::UserRights,
    root: impl FnOnce() -> Result<Result<std::path::PathBuf, String>, String>,
    user: &str,
    drive: &str,
    folder: &str,
    name: &str,
    bytes: &[u8],
) -> Result<String, String> {
    if rights < lumacore_tree::UserRights::Setup {
        return Err(lumacore_objects::errors::render_code(format!("Uploading a file needs Setup — you are {}.", rights.label()), "E-RIGHTS-001"));
    }
    let safe = std::path::Path::new(name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .filter(|n| !n.is_empty() && !n.starts_with('.'))
        .ok_or_else(|| lumacore_objects::errors::render_code("Upload: a file name is needed", "E-CMD-006"))?;
    let root = root()??;
    let dir = if folder.is_empty() { root } else { lumacore_luma::drive::library_file(&root.to_string_lossy(), folder).map_err(|e| lumacore_objects::errors::render_code(e, "E-CMD-008"))? };
    lumacore_luma::drive::create_dir_all(&dir).map_err(|e| lumacore_objects::errors::render_code(e.to_string(), "E-CMD-012"))?;
    let path = dir.join(&safe);
    lumacore_luma::drive::write_atomic(&path, bytes).map_err(|e| lumacore_objects::errors::render_code(format!("Cannot write {}: {e}", path.display()), "E-CMD-012"))?;
    log::info!("{user} uploaded {} onto {drive}", path.display());
    Ok(if folder.is_empty() { safe } else { format!("{folder}/{safe}") })
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// A drive's library root from the published station view (decisions §38): the local library
/// is `library/`, a volume's `<mount>/LumaCore/library`; None when unknown or Out.
fn drive_library_root(station: &Value, id: &str) -> Option<std::path::PathBuf> {
    let row = station["drives"].as_array()?.iter().find(|d| d["id"].as_str().is_some_and(|i| i.eq_ignore_ascii_case(id)))?;
    if row["state"] != "In" {
        return None;
    }
    if row["kind"] == "Internal" {
        return Some(std::path::PathBuf::from("library"));
    }
    Some(lumacore_drives::library::library_root(std::path::Path::new(row["path"].as_str()?)))
}

/// The shows folder of a drive other than the local library, for the rest of a build (the
/// `shows` feed and the Load show dialog's parts on a drive).
pub(super) fn drive_shows_guard(station: &Value, drive: Option<&str>) -> Option<lumacore_luma::backups::DirGuard> {
    let d = drive.filter(|d| !d.eq_ignore_ascii_case(lumacore_objects::drives::LOCAL))?;
    let root = drive_library_root(station, d)?;
    Some(lumacore_luma::backups::DirGuard::enter(root.join("shows")))
}

/// A drive's gdtf folder for the fixture-type lookups (None = the local library's).
pub(super) fn gdtf_dir(station: &Value, drive: Option<&str>) -> Option<String> {
    let d = drive.filter(|d| !d.eq_ignore_ascii_case(lumacore_objects::drives::LOCAL))?;
    Some(drive_library_root(station, d)?.join("gdtf").to_string_lossy().into_owned())
}

/// Which files a kind lists (folders always).
fn file_of_kind(kind: &str, name: &str) -> bool {
    let lower = name.to_lowercase();
    match kind {
        "shows" => lower.ends_with(".luma"),
        "gdtf" => lower.ends_with(".gdtf"),
        "mvr" => lower.ends_with(".mvr"),
        "media" => [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg"].iter().any(|e| lower.ends_with(e)),
        "userprofiles" => lower.ends_with(".lumaprofile"),
        _ => true,
    }
}
