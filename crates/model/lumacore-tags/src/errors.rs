// ███████╗██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// █████╗  ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗╚════██║
// ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║███████║
// ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: errors

//! What a tag tree operation refuses, with its code (decisions §12 "Tags hardened").
//!
//! Owns:          TagError, ERRORS (E-TAG-…).
//! Doesn't own:   the general codes (E-CMD-…, lumacore-state).
//! Entry points:  TagError, ERRORS.
//! Decisions:     §12 "Tags hardened", §26 (coded errors).

use lumacore_base::{Coded, ErrorInfo};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Why a tag tree operation was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TagError {
    #[error("Tag {0} doesn't exist")]
    NoSuchTag(String),
    #[error("Tag {0} can't move under itself or one of its own child tags")]
    IntoItself(String),
    #[error("A tag named \"{name}\" is already under {parent}")]
    NameTaken { name: String, parent: String },
    #[error("Tag {0} is already there")]
    SlotTaken(String),
    #[error("Thru walks the child tags of one tag: {from} and {to} have different parents (join them with +)")]
    AcrossParents { from: String, to: String },
    #[error("\"{name}\" names {count} tags ({paths}): give its number or its parent first (Tag \"Lx1\" Tag \"{name}\")")]
    Ambiguous { name: String, count: usize, paths: String },
    #[error("\"{0}\" is not a tag number: write its path, a number per level (Tag 1.2.1)")]
    BadPath(String),
}

impl Coded for TagError {
    fn code(&self) -> &'static str {
        match self {
            Self::NoSuchTag(_) => "E-TAG-001",
            Self::IntoItself(_) => "E-TAG-002",
            Self::NameTaken { .. } => "E-TAG-003",
            Self::SlotTaken(_) => "E-TAG-004",
            Self::AcrossParents { .. } => "E-TAG-005",
            Self::Ambiguous { .. } => "E-TAG-006",
            Self::BadPath(_) => "E-TAG-007",
        }
    }
}

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The tag tree's error catalog (`Info Error <code>`).
pub const ERRORS: &[ErrorInfo] = &[
    ErrorInfo {
        code: "E-TAG-001",
        title: "No such tag",
        explain: "No tag has that number or name. Tags are numbered per parent: Tag 1.2 is the second child of tag 1. List Tag shows the tree.",
    },
    ErrorInfo {
        code: "E-TAG-002",
        title: "A tag can't go under itself",
        explain: "A tag has one parent, and moving it under itself or one of its own child tags would make a loop. Move it under another tag, or to the top.",
    },
    ErrorInfo {
        code: "E-TAG-003",
        title: "Name taken under that parent",
        explain: "Tag names are unique among the children of one tag (two different parents may each have a \"Right\"). Choose another name or another parent.",
    },
    ErrorInfo {
        code: "E-TAG-004",
        title: "Tag number taken",
        explain: "A tag already has that number under that parent. Store it at a free number, or leave the number out to take the next free one.",
    },
    ErrorInfo {
        code: "E-TAG-005",
        title: "Thru across parents",
        explain: "On tags, Thru walks the children of one tag (Tag 1.2 Thru 1.5, Tag 1.2 Thru). Ends under different parents can't be walked: join them with + (Tag 1.2 + 2.1).",
    },
    ErrorInfo {
        code: "E-TAG-006",
        title: "Tag name names several tags",
        explain: "Names are unique only among siblings, so a name can fit tags under different parents. Give the tag's number, or its parent first: Tag \"Lx1\" Tag \"Right\".",
    },
    ErrorInfo {
        code: "E-TAG-007",
        title: "Not a tag number",
        explain: "A tag number is its path, one number per level: Tag 1 (a top tag), Tag 1.2 (its second child), Tag 1.2.1.",
    },
];

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests {
    #[test]
    fn catalog_is_valid() {
        lumacore_base::error_code::check_catalog(super::ERRORS).unwrap();
    }
}
