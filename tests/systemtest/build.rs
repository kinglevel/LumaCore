use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct CommandManifest {
    registry_hash: u64,
    commands_hash: u64,
    demo_hash: u64,
    tracked_files: Vec<String>,
}

fn hash_file(path: &Path) -> u64 {
    let bytes = fs::read(path).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn main() {
    // The repo root (decisions §34 23.6; cargo sets it for build scripts from .cargo/config.toml).
    let repo_root = PathBuf::from(std::env::var("LUMACORE_ROOT").expect("LUMACORE_ROOT (.cargo/config.toml)"));
    let tracked_files = [
        repo_root.join("crates/model/lumacore-objects/src/commands/registry.rs"),
        repo_root.join("crates/model/lumacore-objects/src/commands/mod.rs"),
        // The demo's sources (the file itself is built, decisions §40 15).
        repo_root.join("library/mvr/ti2026.mvr"),
        repo_root.join("library/poolxml/ti2026-groups.xml"),
        repo_root.join("library/demo/manual.lum"),
    ];

    for path in &tracked_files {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let manifest = CommandManifest {
        registry_hash: hash_file(&tracked_files[0]),
        commands_hash: hash_file(&tracked_files[1]),
        demo_hash: tracked_files[2..].iter().map(|p| hash_file(p)).fold(0, |a, h| a ^ h.rotate_left(7)),
        tracked_files: tracked_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(|dir| repo_root.join(dir))
        .unwrap_or_else(|_| repo_root.join("target"));
    let output_dir = target_dir.join("systemtest/current_build");
    fs::create_dir_all(&output_dir).expect("create systemtest build output dir");
    fs::write(output_dir.join("command_manifest.json"), manifest_json).expect("write manifest");
}
