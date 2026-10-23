// ███╗   ███╗██╗   ██╗██████╗     ██╗     ██╗██████╗ ██████╗  █████╗ ██████╗ ██╗   ██╗
// ████╗ ████║██║   ██║██╔══██╗    ██║     ██║██╔══██╗██╔══██╗██╔══██╗██╔══██╗╚██╗ ██╔╝
// ██╔████╔██║██║   ██║██████╔╝    ██║     ██║██████╔╝██████╔╝███████║██████╔╝ ╚████╔╝
// ██║╚██╔╝██║╚██╗ ██╔╝██╔══██╗    ██║     ██║██╔══██╗██╔══██╗██╔══██║██╔══██╗  ╚██╔╝
// ██║ ╚═╝ ██║ ╚████╔╝ ██║  ██║    ███████╗██║██████╔╝██║  ██║██║  ██║██║  ██║   ██║
// ╚═╝     ╚═╝  ╚═══╝  ╚═╝  ╚═╝    ╚══════╝╚═╝╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝
// FILE: mvr_library

//! The MVR files of a library's `mvr/` folder, their heads read (decisions §38 5: the `files`
//! feed's MVR columns — scene, layers, fixtures, the GDTFs embedded or missing) and cached by path,
//! size and time so a file is parsed once; only the scene entry of the zip is read, never the
//! whole file. Read through the disk door. A file that isn't an MVR keeps its error.
//!
//! Owns:          heads_in, mvr_files_in.
//! Doesn't own:   the format (`lumacore-mvr`), the import (34g4 part 4), the JSON (the server's files feed).
//! Entry points:  heads_in.
//! Decisions:     §38 5, §34 "Import MVR hardened" 1.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use lumacore_mvr::MvrHead;
// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO


/// Every `.mvr` under `dir`, relative (`2026/Rig v3.mvr`).
pub fn mvr_files_in(dir: &str) -> Vec<String> {
    fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) {
        let Ok(entries) = lumacore_luma::drive::read_dir(dir) else { return };
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            let here = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if path.is_dir() && !name.starts_with('.') {
                walk(&path, &here, out);
            } else if name.to_lowercase().ends_with(".mvr") {
                out.push(here);
            }
        }
    }
    let mut out = Vec::new();
    walk(Path::new(dir), "", &mut out);
    out.sort();
    out
}

/// The head of every MVR of `dir`, by relative file name (a file unchanged since the last call
/// comes from the cache).
pub fn heads_in(dir: &str) -> Vec<(String, Result<MvrHead, String>)> {
    static CACHE: OnceLock<Mutex<HashMap<String, (u64, Option<SystemTime>, Result<MvrHead, String>)>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().unwrap_or_else(|p| p.into_inner());
    let files = mvr_files_in(dir);
    let prefix = format!("{dir}/");
    cache.retain(|p, _| !p.starts_with(&prefix) || files.iter().any(|f| *p == format!("{prefix}{f}")));
    files
        .into_iter()
        .filter_map(|file| {
            let path = format!("{dir}/{file}");
            let meta = lumacore_luma::drive::metadata(&path).ok()?;
            let (size, time) = (meta.len(), meta.modified().ok());
            if let Some((s, t, head)) = cache.get(&path)
                && *s == size
                && *t == time
            {
                return Some((file, head.clone()));
            }
            let head = lumacore_luma::drive::open(&path)
                .map_err(|e| e.to_string())
                .and_then(|f| lumacore_mvr::head_from(std::io::BufReader::new(f)).map_err(|e| e.to_string()));
            cache.insert(path, (size, time, head.clone()));
            Some((file, head))
        })
        .collect()
}
