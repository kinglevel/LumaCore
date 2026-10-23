// ██╗     ██╗██████╗
// ██║     ██║██╔══██╗
// ██║     ██║██████╔╝
// ██║     ██║██╔══██╗
// ███████╗██║██████╔╝
// ╚══════╝╚═╝╚═════╝
// FILE: lib

//! lumacore-mvr: the MVR file (GDTF's My Virtual Rig, 1.6) as a format — a zip with
//! `GeneralSceneDescription.xml` and the GDTF files it names. `read` gives the scene as data:
//! the layers, the groups nested in them, every fixture with its place in the tree, its matrix,
//! its GDTF spec and mode, its addresses and ids, the classes and positions, and which named
//! GDTFs the zip holds or lacks. `head` is the few facts a file list shows. Nothing here knows
//! the show: mapping the scene onto `Devices` (the match, the review, Apply) is the objects
//! crate's (34g4 part 4); export and MVR-xchange come later (parts 5, 6).
//!
//! Owns:          the zip walk, the scene XML as data, the address and matrix text rules, the head.
//! Doesn't own:   the show's MVR data (`lumacore-fixtures` fixture_mvr, devices_meta), the import, the store.
//! Entry points:  read, read_from, head, head_from, gdtf_entry, MvrFile, MvrScene, MvrHead.
//! Decisions:     §34 "34g4 patch hardened" 10–12, "Import MVR hardened".

pub mod error;
pub mod read;
pub mod scene;
pub mod xml;

pub use error::MvrError;
pub use read::{gdtf_entry, head, head_from, read, read_from, SCENE_FILE};
pub use scene::{MvrAddress, MvrFile, MvrFixture, MvrGroup, MvrHead, MvrLayer, MvrNamed, MvrScene};
