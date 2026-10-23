// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// FILE: registry
//! The catalog every crate adds its declarations to (decisions §34 21) — see README.md.

pub mod catalog;
pub mod command;
pub mod feed;
pub mod flag;
pub mod global;
pub mod key;
pub mod menu;
pub mod object;
pub mod registry;
pub mod setting;
pub mod status;
pub mod tool;
pub mod verb;
pub mod window_type;

pub use catalog::{Catalog, FreezeError};
pub use feed::{FeedDecl, FeedKind};
pub use flag::{FlagDef, FlagValue};
pub use key::KeyDecl;
pub use status::Status;
pub use tool::{ToolAsk, ToolDecl};
pub use global::{get, install, install_with, installed};
pub use registry::Registry;
pub use verb::Verb;
pub use window_type::WindowType;
