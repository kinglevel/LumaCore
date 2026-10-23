// ███████╗██████╗ ██████╗  ██████╗ ██████╗
// ██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
// █████╗  ██████╔╝██████╔╝██║   ██║██████╔╝
// ██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗
// ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║
// ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝
// FILE: error

//! What reading an MVR can refuse: not a zip, no scene file in it, scene XML that doesn't parse,
//! a fixture without a UUID. The message is the whole story; the objects crate gives it a code.
//!
//! Owns:          MvrError.
//! Doesn't own:   the error catalog (the objects crate renders E-codes).
//! Entry points:  MvrError.
//! Decisions:     §34 "34g4 patch hardened" 10.


// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Why an MVR couldn't be read.
#[derive(Debug, thiserror::Error)]
pub enum MvrError {
    #[error("not an MVR (not a zip): {0}")]
    NotAZip(String),
    #[error("no GeneralSceneDescription.xml in the MVR")]
    NoScene,
    #[error("the scene XML doesn't parse: {0}")]
    BadXml(String),
    #[error("{0} '{1}' has no valid uuid")]
    BadUuid(&'static str, String),
}
