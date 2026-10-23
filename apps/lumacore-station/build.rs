//! Stamps the build with its git commit (`LUMACORE_GIT_SHA`): shown in build
//! info and written into saved shows (decisions §20). `-dirty` = built with
//! uncommitted changes; `unknown` outside a git checkout. It lives in the
//! station, the top of the crate graph (decisions §34 "Build times hardened" 1:
//! ripgrep, rust-analyzer), so a commit recompiles only the station;
//! `install_catalog` hands the stamp to `lumacore-luma`.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    let sha = git(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|s| !s.is_empty());
    let stamp = if dirty { format!("{sha}-dirty") } else { sha };
    println!("cargo:rustc-env=LUMACORE_GIT_SHA={stamp}");
    // Re-stamp on a commit or checkout — not on staging (`.git/index`), which
    // would recompile the station on every `git add` (decisions §23 "Step 8").
    if let Some(dir) = git(&["rev-parse", "--git-dir"]) {
        println!("cargo:rerun-if-changed={dir}/HEAD");
        println!("cargo:rerun-if-changed={dir}/refs");
    }
    println!("cargo:rerun-if-changed=src");
}
