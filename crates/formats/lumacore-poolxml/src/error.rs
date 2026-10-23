// ███████╗██████╗ ██████╗  ██████╗ ██████╗
// ██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
// █████╗  ██████╔╝██████╔╝██║   ██║██████╔╝
// ██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗
// ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║
// ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝
// FILE: error

//! What reading an export can refuse: XML that doesn't parse, a file that isn't a pool export
//! (no root element, no `Pool` under it). The message is the whole story; the objects crate
//! gives it a code.
//!
//! Owns:          PoolXmlError.
//! Doesn't own:   the error catalog (the objects crate renders E-codes).
//! Entry points:  PoolXmlError.
//! Decisions:     §40 13.

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Why an export couldn't be read.
#[derive(Debug, thiserror::Error)]
pub enum PoolXmlError {
    #[error("the XML doesn't parse: {0}")]
    BadXml(String),
    #[error("not a pool export (no root element)")]
    NotAnExport,
    #[error("not a pool export (no Pool under the root)")]
    NoPool,
    #[error("can't read: {0}")]
    Io(String),
}
