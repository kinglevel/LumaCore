// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// FILE: registry
//! The builder (decisions §34 21): every crate's `register(&mut Registry)` first calls the
//! `register` of the crates it depends on, then adds its own rows. [`Registry::enter`] makes a
//! crate count once however many crates above include it, so an assembler calls only the top
//! crates and a crate's tests get it and everything below.
//!
//! Owns:          Registry (the rows while they are added), enter, the add_* calls
//! Doesn't own:   the checks and lookups (catalog), the process-wide catalog (global)
//! Entry points:  Registry::new, enter, errors, global_flags, feeds, window_types, tools,
//!                keys, commands, objects, menus, rows, check, freeze
//! Decisions:     §34 21

use lumacore_base::ErrorInfo;

use crate::command::CommandDecl;
use crate::feed::FeedDecl;
use crate::flag::FlagDef;
use crate::key::KeyDecl;
use crate::menu::MenuDef;
use crate::object::ObjectDecl;
use crate::tool::ToolDecl;
use crate::window_type::WindowType;

use crate::catalog::{Catalog, FreezeError};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The rows while crates add them; [`Registry::freeze`] checks them and makes the [`Catalog`].
#[derive(Default)]
pub struct Registry {
    /// The crates that registered, in order (each once).
    pub(crate) crates: Vec<&'static str>,
    /// Every error catalog, one per crate or area, in registration order.
    pub(crate) errors: Vec<&'static [ErrorInfo]>,
    /// The flags valid on every line.
    pub(crate) global_flags: Vec<&'static FlagDef>,
    /// The live feeds a page can subscribe.
    pub(crate) feeds: Vec<&'static FeedDecl>,
    /// The window types (built rows: owned data, frozen with the rest).
    pub(crate) window_types: Vec<WindowType>,
    /// The group grid's toolbar groups, in order, and its tools.
    pub(crate) tool_groups: Vec<&'static str>,
    pub(crate) tools: Vec<&'static ToolDecl>,
    /// The built-in UI keys, in order.
    pub(crate) keys: Vec<KeyDecl>,
    /// The commands (their handlers are bound in dispatch).
    pub(crate) commands: Vec<&'static CommandDecl>,
    /// The objects (their behaviour sits beside each row, in lumacore-state's `ObjectDef`).
    pub(crate) objects: Vec<&'static ObjectDecl>,
    /// The menus (their JSON and rows are built above, in the views).
    pub(crate) menus: Vec<&'static MenuDef>,
    /// Rows of types declared by the crates above (decisions §34 22.7): each an
    /// `&'static [T]`, read back by its type.
    pub(crate) rows: Vec<&'static (dyn std::any::Any + Send + Sync)>,
    /// The checks those crates ask `freeze()` to run on the whole catalog.
    pub(crate) checks: Vec<fn(&Catalog) -> Vec<String>>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether `krate` registers now: true the first time, false when it already did (a crate
    /// included by two crates above it).
    pub fn enter(&mut self, krate: &'static str) -> bool {
        if self.crates.contains(&krate) {
            return false;
        }
        self.crates.push(krate);
        true
    }

    /// An error catalog (`E-AREA-NNN` codes, decisions §26).
    pub fn errors(&mut self, catalog: &'static [ErrorInfo]) -> &mut Self {
        self.errors.push(catalog);
        self
    }

    /// Flags valid on every line (`/NoConfirmation`, decisions §31); a word's own flag of the
    /// same name wins.
    pub fn global_flags(&mut self, flags: &'static [FlagDef]) -> &mut Self {
        self.global_flags.extend(flags);
        self
    }

    /// Live feeds on `/ws/feeds` (decisions §30 "12d").
    pub fn feeds(&mut self, feeds: &'static [FeedDecl]) -> &mut Self {
        self.feeds.extend(feeds);
        self
    }

    /// Window types (decisions §30 "12c"): what a window shows and how it is built.
    pub fn window_types(&mut self, types: impl IntoIterator<Item = WindowType>) -> &mut Self {
        self.window_types.extend(types);
        self
    }

    /// The group grid's toolbar groups (in order) and its tools (decisions §30 "12e").
    pub fn tools(&mut self, groups: &'static [&'static str], tools: &'static [ToolDecl]) -> &mut Self {
        self.tool_groups.extend(groups);
        self.tools.extend(tools);
        self
    }

    /// Built-in UI keys, numbered in the order they are added (`Root.Resources.UIKeys`).
    pub fn keys(&mut self, keys: impl IntoIterator<Item = KeyDecl>) -> &mut Self {
        self.keys.extend(keys);
        self
    }

    /// Commands (decisions §15): their words, help, undo, scope, rights, reach, flags.
    pub fn commands(&mut self, commands: impl IntoIterator<Item = &'static CommandDecl>) -> &mut Self {
        self.commands.extend(commands);
        self
    }

    /// Objects (decisions §15): their words, Lum shape, help, undo, scope, rights, reach, flags.
    pub fn objects(&mut self, objects: impl IntoIterator<Item = &'static ObjectDecl>) -> &mut Self {
        self.objects.extend(objects);
        self
    }

    /// Menus (decisions §30 "12d"): what `Menu "<name>"` opens.
    pub fn menus(&mut self, menus: &'static [MenuDef]) -> &mut Self {
        self.menus.extend(menus);
        self
    }

    /// Rows of a type a crate above declares (decisions §34 22.7: a protocol's
    /// `ProtocolDef`): the catalog holds them and [`Catalog::rows`] reads them back by type, so
    /// a row's behaviour can sit beside it in the crate that defines the type.
    pub fn rows<T: std::any::Any + Sync>(&mut self, rows: &'static [T]) -> &mut Self {
        let slot: &'static &'static [T] = Box::leak(Box::new(rows));
        self.rows.push(slot);
        self
    }

    /// A check `freeze()` runs on the whole catalog, for rows only the crate above knows
    /// (names unique across them, …); each returned line is one problem.
    pub fn check(&mut self, check: fn(&Catalog) -> Vec<String>) -> &mut Self {
        if !self.checks.contains(&check) {
            self.checks.push(check);
        }
        self
    }

    /// The checked, read-only catalog — or every problem found.
    pub fn freeze(self) -> Result<Catalog, FreezeError> {
        Catalog::from_rows(self)
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
