// ██╗   ██╗██╗███████╗██╗    ██╗
// ██║   ██║██║██╔════╝██║    ██║
// ██║   ██║██║█████╗  ██║ █╗ ██║
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝
// FILE: view

//! How an object shows in the tree (decisions §31 "Tree paths are typed"): a collection's rows
//! (`DynChild`) and one node read out (`NodeView`) — what a `TreeDef` returns.
//!
//! Owns:          DynChild, NodeView.
//! Doesn't own:   building them from the show (each object's tree functions, `vfs`).
//! Entry points:  DynChild, NodeView.
//! Decisions:     §31, §33 57, §34 15.

use serde_json::Value as JsonValue;
use std::path::PathBuf;
use std::time::SystemTime;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A dynamic child entry — an actual object inside a collection.
#[derive(Debug, Clone)]
pub struct DynChild {
    /// Display name (user-given name or fallback like "Fixture 1").
    pub display_name: String,
    /// The BTreeMap key (user-facing ID).
    pub id: u32,
    /// Type tag from TreeNode::node_type().
    pub node_type: &'static str,
    /// Whether this is a leaf (no further children).
    pub is_leaf: bool,
    /// Number of sub-children (0 for most objects).
    pub child_count: usize,
    /// The object's stored name (not the display form) — what `crate::resolve` matches a quoted
    /// name against (decisions §33 57).
    pub name: Option<String>,
    /// The object's GUID, where it has one.
    pub guid: Option<uuid::Uuid>,
}

/// A resolved view of a node — either structural (directory) or a concrete object.
#[derive(Debug, Clone)]
pub enum NodeView {
    /// A structural directory node (e.g. "ShowData", "Patch").
    Structural { name: String, children: Vec<String> },
    /// A concrete data object, serialized to JSON.
    Object {
        json: JsonValue,
        node_type: &'static str,
        id: u32,
        guid: String,
        name: Option<String>,
    },
    /// A real filesystem entry (file or directory) under Root.Drives.
    FileInfo {
        path: PathBuf,
        name: String,
        size: u64,
        is_dir: bool,
        modified: Option<SystemTime>,
        extension: Option<String>,
    },
}

