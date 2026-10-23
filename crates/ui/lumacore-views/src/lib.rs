// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// FILE: views

//! What every UI shows — see the crate's README.md (decisions §34 20.7).

pub mod attribute_board;
pub mod context;
pub mod detail;
pub mod devices;
pub mod encoder;
pub mod insert;
pub mod keys;
pub mod inspector;
pub mod menu_rows;
pub mod menus;
pub mod paint;
pub mod patch;
pub mod pools;
pub mod rules;
pub mod sequence;
pub mod tags;
pub mod tree_window;
pub mod view_bar;
pub mod window_state;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog (none of its own), after the crates it builds on
/// (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_objects::register(r);
    r.enter(env!("CARGO_PKG_NAME"));
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
