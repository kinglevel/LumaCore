// ███████╗██╗    ██╗███████╗███████╗██████╗
// ██╔════╝██║    ██║██╔════╝██╔════╝██╔══██╗
// ███████╗██║ █╗ ██║█████╗  █████╗  ██████╔╝
// ╚════██║██║███╗██║██╔══╝  ██╔══╝  ██╔═══╝
// ███████║╚███╔███╔╝███████╗███████╗██║
// ╚══════╝ ╚══╝╚══╝ ╚══════╝╚══════╝╚═╝
// FILE: sweep

//! The command sweep and the fuzz entry points — see the crate's README.md (decisions §25
//! "Step 10 hardening", §34 20.10, 24.2).

pub mod fuzzing;
pub mod lines;
pub mod run;
pub mod vocab;
pub mod world;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog (none of its own), after the crates it builds on
/// (decisions §34 21). The overnight run installs the station's whole catalog instead; this one
/// is what the property tests and the fuzz targets run with.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_lumasession::register(r);
    lumacore_suggest::register(r);
    lumacore_cmd_treeshell::register(r);
    lumacore_cmd_keys::register(r);
    lumacore_cmd_tags::register(r);
    lumacore_cmd_station::register(r);
    lumacore_cmd_devices::register(r);
    lumacore_cmd_protocols::register(r);
    lumacore_cmd_osc::register(r);
    lumacore_cmd_sacn::register(r);
    lumacore_cmd_artnet::register(r);
    r.enter(env!("CARGO_PKG_NAME"));
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The sweep never installs the station's catalog (decisions §34 24.1: the binary does); running
/// without one is a mistake of the caller.
pub(crate) fn require_catalog() -> Result<(), String> {
    if lumacore_registry::installed() {
        Ok(())
    } else {
        Err("no catalog: install it before the sweep runs (lumacore_station::install_catalog)".into())
    }
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

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
