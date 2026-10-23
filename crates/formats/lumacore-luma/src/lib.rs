// ██╗     ██╗   ██╗███╗   ███╗ █████╗
// ██║     ██║   ██║████╗ ████║██╔══██╗
// ██║     ██║   ██║██╔████╔██║███████║
// ██║     ██║   ██║██║╚██╔╝██║██╔══██║
// ███████╗╚██████╔╝██║ ╚═╝ ██║██║  ██║
// ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝
// FILE: luma

//! The `.luma` show file and the station's disk — see the crate's README.md (decisions §30
//! "12a2", "12d part 5 built", §34 25.10).

pub mod background;
pub mod backups;
pub mod drive;
pub mod errors;
pub mod media;
pub mod show_file;
pub mod station_config;

// ██╗███╗   ██╗██╗████████╗
// ██║████╗  ██║██║╚══██╔══╝
// ██║██╔██╗ ██║██║   ██║
// ██║██║╚██╗██║██║   ██║
// ██║██║ ╚████║██║   ██║
// ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝
// PART: INIT

/// The build's git commit, handed down once by the station (decisions §34 "Build times hardened"
/// 1: the stamp lives at the top of the crate graph, so a commit recompiles only the station).
static GIT_COMMIT: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Set the build's git commit (`lumacore_station::install_catalog`, from its `build.rs`); the first
/// call wins.
pub fn set_git_commit(stamp: &str) {
    let _ = GIT_COMMIT.set(stamp.to_string());
}

/// This build: version, profile, target, git commit (the station's stamp; `unknown` in a test or
/// tool that never went through the station).
pub fn build_info() -> lumacore_tree::BuildInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let profile = if cfg!(debug_assertions) {
        "debug".to_string()
    } else {
        "release".to_string()
    };
    let target_os = std::env::consts::OS.to_string();
    let target_arch = std::env::consts::ARCH.to_string();
    // The station's `build.rs` (`-dirty` = uncommitted changes).
    let git_commit = GIT_COMMIT.get().cloned().unwrap_or_else(|| "unknown".to_string());
    let build_id = format!("{version}/{profile}/{target_os}-{target_arch}/{git_commit}");
    lumacore_tree::BuildInfo {
        version,
        profile,
        target_os,
        target_arch,
        git_commit,
        build_id,
    }
}

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog: the show file's and the media store's error codes, after the
/// crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_tree::register(r);
    lumacore_gdtf::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(errors::ERRORS);
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
