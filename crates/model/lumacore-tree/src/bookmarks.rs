// ██████╗  ██████╗  ██████╗ ██╗  ██╗███╗   ███╗ █████╗ ██████╗ ██╗  ██╗███████╗
// ██╔══██╗██╔═══██╗██╔═══██╗██║ ██╔╝████╗ ████║██╔══██╗██╔══██╗██║ ██╔╝██╔════╝
// ██████╔╝██║   ██║██║   ██║█████╔╝ ██╔████╔██║███████║██████╔╝█████╔╝ ███████╗
// ██╔══██╗██║   ██║██║   ██║██╔═██╗ ██║╚██╔╝██║██╔══██║██╔══██╗██╔═██╗ ╚════██║
// ██████╔╝╚██████╔╝╚██████╔╝██║  ██╗██║ ╚═╝ ██║██║  ██║██║  ██║██║  ██╗███████║
// ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝
// FILE: bookmarks

//! A user's NAV bookmarks (decisions 0291): `ShowData.Users.{n}.Bookmarks`, a letter → the GUID
//! of the object it marks, so a bookmark still finds its object after it was renumbered or moved.
//! Saved and replicated like Usage, never undone (its own `ShowSubtree::Bookmarks`).
//!
//! Owns:          Bookmarks.
//! Doesn't own:   setting or following one (the TUI's NAV), the GUID → path walk (identity).
//! Entry points:  Bookmarks.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One user's bookmarks, by letter (`a` … `z`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Bookmarks {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub marks: BTreeMap<char, uuid::Uuid>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Bookmarks {
    /// Mark `guid` with `letter` (a mark already on that letter goes).
    pub fn set(&mut self, letter: char, guid: uuid::Uuid) {
        self.marks.insert(letter, guid);
    }

    /// The GUID `letter` marks.
    pub fn get(&self, letter: char) -> Option<uuid::Uuid> {
        self.marks.get(&letter).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.marks.is_empty()
    }
}
