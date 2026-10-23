// ████████╗██╗██████╗ ██╗   ██╗
// ╚══██╔══╝██║██╔══██╗╚██╗ ██╔╝
//    ██║   ██║██║  ██║ ╚████╔╝
//    ██║   ██║██║  ██║  ╚██╔╝
//    ██║   ██║██████╔╝   ██║
//    ╚═╝   ╚═╝╚═════╝    ╚═╝
// FILE: tidy
//! The source rules of the whole workspace, one file per area — see README.md and docs/meta/tidy.md.

#[cfg(test)]
mod docs;
#[cfg(test)]
mod doors;
#[cfg(test)]
mod files;
#[cfg(test)]
mod frontend;
#[cfg(test)]
mod station;
#[cfg(test)]
mod release;
#[cfg(test)]
mod render;
#[cfg(test)]
mod rulebook;
#[cfg(test)]
mod structure;
#[cfg(test)]
mod words;

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// The rules read the catalog: the objects and below, the HTTP engine's with `http` — installed
/// before the first rule (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(|r| {
        lumacore_objects::register(r);
        lumacore_cmd_treeshell::register(r);
        lumacore_cmd_keys::register(r);
        lumacore_cmd_tags::register(r);
        lumacore_cmd_station::register(r);
        lumacore_cmd_devices::register(r);
        lumacore_cmd_protocols::register(r);
        lumacore_cmd_lumasession::register(r);
        #[cfg(feature = "http")]
        lumacore_http::register(r);
    });
}
