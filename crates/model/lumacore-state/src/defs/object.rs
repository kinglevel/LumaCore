//  ██████╗ ██████╗      ██╗███████╗ ██████╗████████╗
// ██╔═══██╗██╔══██╗     ██║██╔════╝██╔════╝╚══██╔══╝
// ██║   ██║██████╔╝     ██║█████╗  ██║        ██║
// ██║   ██║██╔══██╗██   ██║██╔══╝  ██║        ██║
// ╚██████╔╝██████╔╝╚█████╔╝███████╗╚██████╗   ██║
//  ╚═════╝ ╚═════╝  ╚════╝ ╚══════╝ ╚═════╝   ╚═╝
// FILE: object

//! What an object does beside its row (decisions §15, §33, §34 21): `ObjectDef` pairs the catalog's
//! `ObjectDecl` with the verbs' handlers, its pool, tree collection, properties, completer and
//! Delete question.
//!
//! Owns:          the object registry's types.
//! Doesn't own:   the lists of objects (each command family's `OBJECTS`, `objects::all`), finding one by word (`objects::find`).
//! Entry points:  ObjectDef, Verb, LumShape, TreeDef, DatapoolTree, DeleteAsk, Handler.
//! Decisions:     §15, §33, §34 15.

use crate::defs::command::{Reach, Scope};
use crate::{AppState, DispatchResult};
/// The verbs, the object rows and their Lum shapes are the catalog's (decisions §34 21).
pub use lumacore_registry::Verb;
pub use lumacore_registry::object::{IdReading, LumShape, ObjectDecl};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A verb's implementation for an object. Gets the arguments after the
/// object keyword (`Store Window 5 "Groups"` → `["5", "\"Groups\""]`).
pub type Handler = fn(&mut AppState, &[&str]) -> DispatchResult;

/// Instance completions for the token after the keyword (window numbers,
/// user names, …), filtered by the typed prefix.
pub type Completer = fn(&AppState, &str) -> Vec<String>;

/// Where an object's instances live in the object tree: the collection node
/// (e.g. `Root.ShowData.Users`), its children and each instance's view.
pub struct TreeDef {
    /// Its node in the object tree (`crate::tree_path`).
    pub node: crate::tree_path::Node,
    /// Its entry's name when `node` is a named node's shape (`Some("Sacn")` under
    /// `DMXProtocols`, decisions §34 "34f7c hardened" 5): one object per name.
    pub name: Option<&'static str>,
    pub children: fn(&lumacore_tree::Root) -> Vec<crate::defs::view::DynChild>,
    pub object: fn(&lumacore_tree::Root, u32) -> Option<crate::defs::view::NodeView>,
    /// A collection inside each datapool (`Root.ShowData.DataPools.<n>.Groups`,
    /// decisions §22, §29): its children and objects in datapool `n`. The
    /// plain `children` / `object` serve the `DataPool` placeholder (the
    /// first datapool).
    pub datapool: Option<DatapoolTree>,
    /// Its pool's door (decisions §35): Store / Delete / Copy / Move / Insert / Compact / Paste
    /// write it only through this. None = not on the door yet (§33 3d moves each area).
    pub door: Option<&'static dyn crate::pools::door::PoolOps>,
}

/// A tree collection's functions inside one datapool.
pub struct DatapoolTree {
    pub children: fn(&lumacore_tree::Root, u32) -> Vec<crate::defs::view::DynChild>,
    pub object: fn(&lumacore_tree::Root, u32, u32) -> Option<crate::defs::view::NodeView>,
}

impl TreeDef {
    /// Its children, in datapool `dp` when it lives in one.
    pub fn children_in(&self, root: &lumacore_tree::Root, dp: Option<u32>) -> Vec<crate::defs::view::DynChild> {
        match (dp, &self.datapool) {
            (Some(dp), Some(d)) => (d.children)(root, dp),
            _ => (self.children)(root),
        }
    }

    /// Object `id`, in datapool `dp` when it lives in one.
    pub fn object_in(&self, root: &lumacore_tree::Root, dp: Option<u32>, id: u32) -> Option<crate::defs::view::NodeView> {
        match (dp, &self.datapool) {
            (Some(dp), Some(d)) => (d.object)(root, dp, id),
            _ => (self.object)(root, id),
        }
    }
}

/// How the one resolver walks an object whose ids read as a path (`IdReading::Path`, decisions §12
/// "Tags hardened"): every node with its path, depth first, a parent before its children. A
/// catalog row the object's own crate adds (`r.rows(..)`), so the kernel never names the object.
pub struct PathDef {
    pub object: &'static ObjectDecl,
    pub nodes: PathNodes,
}

/// Every node of a path object with its path, depth first (`PathDef::nodes`).
pub type PathNodes = fn(&lumacore_tree::Root) -> Vec<(Vec<u32>, crate::defs::view::DynChild)>;

/// An object's own part of the one context menu (decisions §34 round 3, 18): what only it does,
/// below the common actions and the Set part every object gets from its verbs and properties. A
/// catalog row its crate adds (`r.rows(..)`), so neither the views nor a window names the object.
pub struct ContextDef {
    pub object: &'static ObjectDecl,
    pub items: &'static [ContextItem],
}

/// One of an object's own menu lines: `{object}` / `{rows}` name the selection; `{value}` is asked
/// first (a number or name).
pub struct ContextItem {
    pub label: &'static str,
    /// A Lucide name from the kit's sprite.
    pub icon: &'static str,
    /// The verb it runs, for its rights.
    pub verb: Verb,
    pub line: &'static str,
    /// It changes only live columns (an address): it runs in a claim area's Live too.
    pub live: bool,
    /// The value dialog kind its `{value}` opens (decisions §34 round 3, 18): `"address"`, or ""
    /// for the plain ask.
    pub value: &'static str,
}

/// A tab every show object's EditSetting gets from another crate (decisions §12 "Tags hardened"
/// 8: the Tags tab, from lumacore-cmd-tags), so the kernel never names it: its fields for the row
/// with this GUID; `target` is how a line names the row (`Fixture 103`, `DataPool 1 Group 2`).
pub struct SettingsTab {
    pub fields: fn(&AppState, uuid::Uuid, &str) -> Vec<lumacore_wire::SettingField>,
}

/// What dispatch does before a Delete of an object (decisions §30 "Merge of
/// bugfixes": one ask, in dispatch; `/NoConfirmation` skips it).
pub enum DeleteAsk {
    /// The question to show (it names the count and the rows).
    Ask(Box<crate::dialogs::Ask>),
    /// Refused before any question (no such rows, rows that can't go).
    Refuse(String),
    /// The handler asks its own, richer question (what goes with a user), or
    /// says what it takes: dispatch asks nothing and leaves the flags.
    Handler,
    /// The plain question: `Delete <line>?`.
    Plain,
}

/// An object's row and its behaviour, side by side (decisions §34 21): the catalog holds `decl`;
/// the verbs' handlers, its pool, tree collection, properties, completer and Delete question are
/// here. Reads through to the row (`def.keyword`).
pub struct ObjectDef {
    /// The row the catalog holds.
    pub decl: ObjectDecl,
    /// Shown in a pool window (decisions §22 "Unified pool window").
    pub pool: Option<&'static crate::pools::PoolDef>,
    /// Built verbs and their handlers.
    pub verbs: &'static [(Verb, Handler)],
    /// The one Delete question dispatch asks for it (decisions §30 "Merge
    /// of bugfixes"), given the arguments after the object word; None = the
    /// plain `Delete …?`. Row objects use `set_rows::delete_ask`.
    pub delete_ask: Option<fn(&AppState, &[&str]) -> DeleteAsk>,
    pub complete: Option<Completer>,
    /// Its properties (decisions §33 58): the one declaration `Set`, `Info`, `EditSetting`,
    /// the settings cells, the grids' columns, the rights gate, help and the reference read.
    /// None = nothing of it can be set.
    pub props: Option<&'static crate::props::Props>,
    /// Its collection in the object tree, if it has one.
    pub tree: Option<TreeDef>,
}

impl std::ops::Deref for ObjectDef {
    type Target = ObjectDecl;

    fn deref(&self) -> &ObjectDecl {
        &self.decl
    }
}


impl ObjectDef {
    /// An object with only its row: no pool, verbs, Delete question, completer, properties or tree.
    /// The door for a new object (tidy `shared_doors_have_no_copies`): add the rest with the
    /// builders, `ObjectDef::new(decl).verbs(&[…]).tree(…)`, never spelled out field by field.
    pub const fn new(decl: ObjectDecl) -> ObjectDef {
        ObjectDef { decl, pool: None, verbs: &[], delete_ask: None, complete: None, props: None, tree: None }
    }

    pub const fn verbs(self, verbs: &'static [(Verb, Handler)]) -> ObjectDef {
        ObjectDef { verbs, ..self }
    }

    pub const fn tree(self, tree: TreeDef) -> ObjectDef {
        ObjectDef { tree: Some(tree), ..self }
    }

    pub const fn delete_ask(self, ask: fn(&AppState, &[&str]) -> DeleteAsk) -> ObjectDef {
        ObjectDef { delete_ask: Some(ask), ..self }
    }

    pub const fn complete(self, complete: Completer) -> ObjectDef {
        ObjectDef { complete: Some(complete), ..self }
    }

    pub const fn props(self, props: &'static crate::props::Props) -> ObjectDef {
        ObjectDef { props: Some(props), ..self }
    }

    pub const fn pool(self, pool: &'static crate::pools::PoolDef) -> ObjectDef {
        ObjectDef { pool: Some(pool), ..self }
    }

    pub fn handler(&self, verb: Verb) -> Option<Handler> {
        self.verbs.iter().find(|(v, _)| *v == verb).map(|(_, h)| *h)
    }

    pub fn supports(&self, verb: Verb) -> bool {
        self.handler(verb).is_some()
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

// *************************************************************************************************
// Where a line runs
// The scope and reach functions an ObjectDef names.
// *************************************************************************************************

/// [`ObjectDef::scope`] of objects that live in the show or a user's state.
pub fn on_master(_: Verb, _: &[&str]) -> Scope {
    Scope::Master
}

/// [`ObjectDef::scope`] of objects that only touch this UI client (menus).
pub fn on_client(_: Verb, _: &[&str]) -> Scope {
    Scope::Client
}

/// [`ObjectDef::scope`] of objects that belong to this station.
pub fn on_station(_: Verb, _: &[&str]) -> Scope {
    Scope::Station
}

/// [`ObjectDef::reach`] of objects that stay inside the show and the runtime.
pub fn show_reach(_: Verb) -> Reach {
    Reach::Show
}

/// [`ObjectDef::reach`] of objects that are files (show files, the library).
pub fn drive_reach(_: Verb) -> Reach {
    Reach::Drive
}
