// ███████╗███╗   ██╗ ██████╗ ██╗███╗   ██╗███████╗███████╗
// ██╔════╝████╗  ██║██╔════╝ ██║████╗  ██║██╔════╝██╔════╝
// █████╗  ██╔██╗ ██║██║  ███╗██║██╔██╗ ██║█████╗  ███████╗
// ██╔══╝  ██║╚██╗██║██║   ██║██║██║╚██╗██║██╔══╝  ╚════██║
// ███████╗██║ ╚████║╚██████╔╝██║██║ ╚████║███████╗███████║
// ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
// FILE: engines

//! The engine registry — see the crate's README.md (decisions §26, §34 20.4, 24.2).

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "perfetto")]
pub mod perfetto;
pub mod registry;
pub mod render;

pub use registry::{Engine, EngineContext, EngineRegistry, PublishedShow, http_ui_engine_state};
