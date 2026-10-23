// ██╗     ██╗██████╗
// ██║     ██║██╔══██╗
// ██║     ██║██████╔╝
// ██║     ██║██╔══██╗
// ███████╗██║██████╔╝
// ╚══════╝╚═╝╚═════╝
// FILE: lib

//! A console's pool export as XML (a root › `Pool`) as a format (decisions §40 13): the pool's
//! `Groups` read from the XML, each group with its items — a fixture's id (`840`, or a sub-fixture's `849.1`) and its
//! grid cell. Read streaming: the Default pool of a big show is 310 MB of presets and cues this
//! crate never looks at. What a group means for the show — one LumaCore group per exported group,
//! members by fixture id — is the objects crate's (`Import PoolXml "file" Groups`). Presets,
//! sequences, cues and macros follow when their LumaCore objects exist.
//!
//! Owns:          read (the streaming walk), pool (the data), error.
//! Doesn't own:   where the file comes from (the caller's door), what a group means.
//! Entry points:  read, read_from, PoolXml.
//! Decisions:     §40 13 (2026-10-20).

pub mod error;
pub mod pool;
pub mod read;

pub use error::PoolXmlError;
pub use pool::{PoolXmlGroup, PoolXmlItem, PoolXml};
pub use read::{read, read_from};
