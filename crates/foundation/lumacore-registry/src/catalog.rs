//  ██████╗ █████╗ ████████╗ █████╗ ██╗      ██████╗  ██████╗
// ██╔════╝██╔══██╗╚══██╔══╝██╔══██╗██║     ██╔═══██╗██╔════╝
// ██║     ███████║   ██║   ███████║██║     ██║   ██║██║  ███╗
// ██║     ██╔══██║   ██║   ██╔══██║██║     ██║   ██║██║   ██║
// ╚██████╗██║  ██║   ██║   ██║  ██║███████╗╚██████╔╝╚██████╔╝
//  ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝ ╚═════╝  ╚═════╝
// FILE: catalog
//! The frozen catalog (decisions §34 21): the rows every crate added, checked once by
//! [`Catalog::from_rows`] and read-only afterwards. The lookups every reader uses live here, so
//! no crate keeps a second list.
//!
//! Owns:          Catalog, FreezeError, the freeze checks, the lookups (error, global_flags, feeds, window_types, tools,
//!                keys)
//! Doesn't own:   adding rows (registry), the process-wide catalog (global)
//! Entry points:  Catalog::{crates, errors, error, global_flags, global_flag, feeds, feed,
//!                window_types, window_type, tool_groups, tools, tool, keys,
//!                commands, command, objects, object, menus, menu, rows}
//! Decisions:     §26, §34 21

use lumacore_base::ErrorInfo;
use lumacore_base::error_code::{check_catalog, find};

use crate::command::CommandDecl;
use crate::feed::FeedDecl;
use crate::flag::FlagDef;
use crate::key::KeyDecl;
use crate::menu::MenuDef;
use crate::object::ObjectDecl;
use crate::registry::Registry;
use crate::tool::ToolDecl;
use crate::window_type::WindowType;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Every declaration of the process, checked and read-only.
pub struct Catalog {
    crates: Vec<&'static str>,
    errors: Vec<&'static [ErrorInfo]>,
    global_flags: Vec<&'static FlagDef>,
    feeds: Vec<&'static FeedDecl>,
    window_types: std::collections::BTreeMap<u32, WindowType>,
    tool_groups: Vec<&'static str>,
    tools: Vec<&'static ToolDecl>,
    keys: Vec<KeyDecl>,
    commands: Vec<&'static CommandDecl>,
    objects: Vec<&'static ObjectDecl>,
    menus: Vec<&'static MenuDef>,
    rows: Vec<&'static (dyn std::any::Any + Send + Sync)>,
}

/// What `freeze()` found wrong, one line each.
#[derive(Debug, PartialEq, Eq)]
pub struct FreezeError(pub Vec<String>);

impl std::fmt::Debug for Catalog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Catalog")
            .field("crates", &self.crates)
            .field("commands", &self.commands.len())
            .field("window_types", &self.window_types.len())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for FreezeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the catalog doesn't check (decisions §34 21):\n  {}", self.0.join("\n  "))
    }
}

impl std::error::Error for FreezeError {}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Catalog {
    /// Checks every table (each error catalog well formed, every code unique across them).
    pub(crate) fn from_rows(rows: Registry) -> Result<Self, FreezeError> {
        let mut wrong = Vec::new();
        let mut all = Vec::new();
        for catalog in &rows.errors {
            if let Err(e) = check_catalog(catalog) {
                wrong.push(format!("error catalog: {e}"));
            }
            all.extend_from_slice(catalog);
        }
        if let Err(e) = check_catalog(&all) {
            wrong.push(format!("error codes across catalogs: {e}"));
        }
        let mut names: Vec<String> = Vec::new();
        for flag in &rows.global_flags {
            for name in std::iter::once(flag.name).chain(flag.aliases.iter().copied()) {
                let key = name.to_ascii_lowercase();
                if names.contains(&key) {
                    wrong.push(format!("global flag /{name}: another global flag has that name"));
                }
                names.push(key);
            }
        }
        let mut feeds: Vec<&str> = Vec::new();
        for feed in &rows.feeds {
            if feeds.contains(&feed.name) {
                wrong.push(format!("feed {}: declared twice", feed.name));
            }
            feeds.push(feed.name);
        }
        let mut window_types = std::collections::BTreeMap::new();
        for ty in rows.window_types {
            if window_types.values().any(|t: &WindowType| t.name == ty.name) {
                wrong.push(format!("window type {}: declared twice", ty.name));
            }
            if let Some(old) = window_types.insert(ty.no, ty) {
                wrong.push(format!("window type {}: number {} is taken", old.name, old.no));
            }
        }
        for (i, tool) in rows.tools.iter().enumerate() {
            if rows.tools[..i].iter().any(|t| t.id.eq_ignore_ascii_case(tool.id)) {
                wrong.push(format!("tool {}: declared twice", tool.id));
            }
            if !rows.tool_groups.contains(&tool.group) {
                wrong.push(format!("tool {}: its group {} isn't a toolbar group", tool.id, tool.group));
            }
        }
        for (i, key) in rows.keys.iter().enumerate() {
            if rows.keys[..i].iter().any(|k| k.id == key.id) {
                wrong.push(format!("UI key {}: declared twice", key.id));
            }
        }
        let mut names: Vec<String> = Vec::new();
        for c in &rows.commands {
            for name in c.names() {
                let key = name.to_ascii_lowercase();
                if names.contains(&key) {
                    wrong.push(format!("command word {name}: another command has it (decisions §15: one name, one meaning)"));
                }
                names.push(key);
            }
        }
        for o in &rows.objects {
            for name in std::iter::once(o.keyword).chain(o.aliases.iter().copied()) {
                let key = name.to_ascii_lowercase();
                if names.contains(&key) {
                    wrong.push(format!("word {name}: a command or another object has it (decisions §15: one name, one meaning)"));
                }
                names.push(key);
            }
            for child in o.lum.children {
                if !rows.objects.iter().any(|r| std::ptr::eq(*r, *child)) {
                    wrong.push(format!("object {}: its child {} isn't in the catalog", o.keyword, child.keyword));
                }
            }
        }
        for (i, m) in rows.menus.iter().enumerate() {
            if rows.menus[..i].iter().any(|o| o.name.eq_ignore_ascii_case(m.name)) {
                wrong.push(format!("menu {}: declared twice", m.name));
            }
            for tab in m.tabs {
                if let crate::menu::Content::Grid(g) | crate::menu::Content::Drives(g) = tab.content
                    && let Some(obj) = g.object
                    && !rows.objects.iter().any(|r| std::ptr::eq(*r, obj))
                {
                    wrong.push(format!("menu {} tab {}: its grid's object {} isn't in the catalog", m.name, tab.name, obj.keyword));
                }
            }
        }
        let catalog = Self {
            crates: rows.crates,
            errors: rows.errors,
            global_flags: rows.global_flags,
            feeds: rows.feeds,
            window_types,
            tool_groups: rows.tool_groups,
            tools: rows.tools,
            keys: rows.keys,
            commands: rows.commands,
            objects: rows.objects,
            menus: rows.menus,
            rows: rows.rows,
        };
        for check in rows.checks {
            wrong.extend(check(&catalog));
        }
        if !wrong.is_empty() {
            return Err(FreezeError(wrong));
        }
        Ok(catalog)
    }

    /// The rows of type `T` a crate above added with [`Registry::rows`], in registration order.
    pub fn rows<T: std::any::Any + Sync>(&self) -> impl Iterator<Item = &'static T> + '_ {
        self.rows
            .iter()
            .filter_map(|r| (*r).downcast_ref::<&'static [T]>().copied())
            .flatten()
    }

    /// The crates that registered, in order.
    pub fn crates(&self) -> &[&'static str] {
        &self.crates
    }

    /// Every error code, catalog by catalog (`List Error`).
    pub fn errors(&self) -> impl Iterator<Item = &'static ErrorInfo> + '_ {
        self.errors.iter().flat_map(|c| c.iter())
    }

    /// The flags valid on every line.
    pub fn global_flags(&self) -> impl Iterator<Item = &'static FlagDef> + '_ {
        self.global_flags.iter().copied()
    }

    /// The global flag `word` names (no slash, any case, a short form too).
    pub fn global_flag(&self, word: &str) -> Option<&'static FlagDef> {
        self.global_flags().find(|f| f.named(word))
    }

    /// Every live feed, in registration order (the `hello`).
    pub fn feeds(&self) -> impl Iterator<Item = &'static FeedDecl> + '_ {
        self.feeds.iter().copied()
    }

    /// The feed of that name.
    pub fn feed(&self, name: &str) -> Option<&'static FeedDecl> {
        self.feeds().find(|f| f.name == name)
    }

    /// Every window type by number (`Root.Resources.WindowTypes`, `/api/window-types`).
    pub fn window_types(&self) -> &std::collections::BTreeMap<u32, WindowType> {
        &self.window_types
    }

    /// The window type of that name (`terminal`).
    pub fn window_type(&self, name: &str) -> Option<&WindowType> {
        self.window_types.values().find(|t| t.name == name)
    }

    /// The group grid's toolbar groups, in order.
    pub fn tool_groups(&self) -> &[&'static str] {
        &self.tool_groups
    }

    /// Every tool, in toolbar order.
    pub fn tools(&self) -> impl Iterator<Item = &'static ToolDecl> + '_ {
        self.tools.iter().copied()
    }

    /// The tool of that id (any case).
    pub fn tool(&self, id: &str) -> Option<&'static ToolDecl> {
        self.tools().find(|t| t.id.eq_ignore_ascii_case(id))
    }

    /// The built-in UI keys, in their order (numbered from 1).
    pub fn keys(&self) -> &[KeyDecl] {
        &self.keys
    }

    /// Every command, in registration order.
    pub fn commands(&self) -> impl Iterator<Item = &'static CommandDecl> + '_ {
        self.commands.iter().copied()
    }

    /// The command a word names (its verb or an alias, any case).
    pub fn command(&self, word: &str) -> Option<&'static CommandDecl> {
        self.commands().find(|c| c.names().any(|n| n.eq_ignore_ascii_case(word)))
    }

    /// Every object, in registration order.
    pub fn objects(&self) -> impl Iterator<Item = &'static ObjectDecl> + '_ {
        self.objects.iter().copied()
    }

    /// The object a word names (its keyword or an alias, any case; quotes ignored).
    pub fn object(&self, word: &str) -> Option<&'static ObjectDecl> {
        self.objects().find(|o| o.matches(word))
    }

    /// Every menu, in registration order.
    pub fn menus(&self) -> impl Iterator<Item = &'static MenuDef> + '_ {
        self.menus.iter().copied()
    }

    /// The menu a name names: its name, label or an alias (any case; quotes ignored).
    pub fn menu(&self, name: &str) -> Option<&'static MenuDef> {
        let name = name.trim_matches('"');
        self.menus()
            .find(|m| m.name.eq_ignore_ascii_case(name) || m.label.eq_ignore_ascii_case(name) || m.aliases.iter().any(|a| a.eq_ignore_ascii_case(name)))
    }

    /// The entry for a code (`e-patch-2` finds `E-PATCH-002`).
    pub fn error(&self, code: &str) -> Option<&'static ErrorInfo> {
        find(&self.errors, code)
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
