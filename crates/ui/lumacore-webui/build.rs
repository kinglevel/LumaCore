//! Embeds every file under `web/` (decisions §36 1–2): writes `$OUT_DIR/files.rs`, the list of
//! `(url path, include_bytes!, MIME type)`, so a new kit file ships without touching code.
//! Dot files are skipped.

use std::fmt::Write;
use std::path::Path;

fn mime(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("") {
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "txt" | "md" | "LICENSE" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut entries: Vec<_> = std::fs::read_dir(dir).expect("read web/").flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            walk(&path, root, out);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
            let rel = path.strip_prefix(root).expect("under web/");
            let url = format!("/{}", rel.to_string_lossy().replace('\\', "/"));
            out.push((url, path.display().to_string()));
        }
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    let mut out = String::from("&[\n");
    for (url, file) in &files {
        let _ = writeln!(out, "    ({url:?}, include_bytes!({file:?}), {:?}),", mime(url));
    }
    out.push(']');
    let dest = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("files.rs");
    std::fs::write(dest, out).expect("write files.rs");
}
