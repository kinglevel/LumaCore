// ██████╗  ██████╗  ██████╗ ██╗
// ██╔══██╗██╔═══██╗██╔═══██╗██║
// ██████╔╝██║   ██║██║   ██║██║
// ██╔═══╝ ██║   ██║██║   ██║██║
// ██║     ╚██████╔╝╚██████╔╝███████╗
// ╚═╝      ╚═════╝  ╚═════╝ ╚══════╝
// FILE: pool

//! The pool as data: its name and version, its groups in the file's order, each item's id text
//! and grid cell. An item's id is a fixture number (`840`) or a sub-fixture's dotted id
//! (`849.1`, `769.2.1`); `fid` tells them apart — a fixture is a fixture, LumaCore has no
//! sub-fixtures (decisions §40 3).
//!
//! Owns:          PoolXml, PoolXmlGroup, PoolXmlItem.
//! Doesn't own:   the read (read.rs), what a group becomes (the objects crate).
//! Entry points:  PoolXmlItem::fid.
//! Decisions:     §40 13.

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One `<Pool>` of the export (a DataPool as the desk writes it): what this crate reads of it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolXml {
    /// The root's `DataVersion`, the desk's version (`2.2.5.2`).
    pub data_version: String,
    /// `Pool@Name` (`Default`, `Event`, `Busk1`).
    pub name: String,
    /// `Groups` › every `Group`, in the file's order.
    pub groups: Vec<PoolXmlGroup>,
    /// What else the pool holds, counted by element name (`Sequences`, `Macros`, `Layouts`…),
    /// for the file list and the reply; never read into data here.
    pub other: Vec<(String, usize)>,
}

/// One `<Group>`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolXmlGroup {
    pub name: String,
    /// `Group@Guid` as the desk writes it (`01 74 C4 …`), kept as text.
    pub guid: String,
    /// `SelectionData` › every `Item`, in order.
    pub items: Vec<PoolXmlItem>,
}

/// One `<Item IDType="0" ID="840" X="87" Y="98" Z="0"/>`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolXmlItem {
    /// `IDType`: 0 = a fixture id.
    pub id_type: u32,
    /// `ID` as written: `840`, or a sub-fixture's `849.1`.
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl PoolXmlItem {
    /// The fixture number when the item is a whole fixture; None for a sub-fixture (`849.1`).
    pub fn fid(&self) -> Option<u32> {
        if self.id.contains('.') {
            return None;
        }
        self.id.trim().parse().ok()
    }
}

impl PoolXmlGroup {
    /// Whether every item is a sub-fixture: nothing LumaCore's group can hold.
    pub fn sub_fixtures_only(&self) -> bool {
        !self.items.is_empty() && self.items.iter().all(|i| i.fid().is_none())
    }
}
