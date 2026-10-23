// ██████╗  ██████╗  ██████╗ ████████╗
// ██╔══██╗██╔═══██╗██╔═══██╗╚══██╔══╝
// ██████╔╝██║   ██║██║   ██║   ██║
// ██╔══██╗██║   ██║██║   ██║   ██║
// ██║  ██║╚██████╔╝╚██████╔╝   ██║
// ╚═╝  ╚═╝ ╚═════╝  ╚═════╝    ╚═╝
// FILE: root

//! `Root` (decisions §33 53–54): the top of the object tree as a counted object. Alone it sets
//! `Root>`; its default verb is List — `Root 3` = `List Root 3`. Its ids are tree numbers with
//! any number of parts (`Root 1.5`: the fifth child of the first) or a quoted name (`Root
//! "ShowData"`); a number is the child's No (`tree_path::Node::no`). It is also the top a tree
//! kind moves to (`Move Tag 1.2 At Root`).
//!
//! Owns:          the Root object, List / Info Root.
//! Doesn't own:   the tree (tree_path), its entries (vfs), cd / ls (navigate), Root.Drives (the
//!                tree shell's).
//! Entry points:  ROOT, ListRoot, InfoRoot.
//! Decisions:     §33 53, 54.

use lumacore_syntax::ast::{Id, Item, Range, Target};

use super::{Doc, LumShape, ObjectDecl, ObjectDef, Undo, Verb, out};
use crate::commands::help::HelpCategory;
use crate::commands::registry::CommandClass;
use crate::commands::typed::typed;
use crate::errors::render_code;
use crate::tree_path::{Node, Step, TreePath};
use crate::{AppState, DispatchResult};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// `Root` (decisions §33 53): counted, its ids a path of tree numbers.
pub static ROOT: ObjectDef = ObjectDef::new(ObjectDecl {
    keyword: "Root",
    lum: LumShape::path(),
    default_verb: Verb::List,
    aliases: &[],
    doc: Doc {
        category: HelpCategory::TreeNav,
        syntax: "Root [<n>[.<n>…] | \"name\"] | List Root [<n>] | Info Root [<n>] | Move Tag <path> At Root",
        examples: &[
            "Root                        -- Root> : numbers name its nodes",
            "Root 1                      -- List Root 1: the first node and its children",
            "List Root \"ShowData\"        -- a node by name",
            "Info Root 1.3               -- the third child of the first",
            "Move Tag 1.2 At Root        -- a tag to the top",
        ],
        related: &["cd", "ls", "list"],
        ..Doc::new(
            "Root",
            "The top of the object tree. Every node has a number, its place under its parent (a collection's entries keep their own numbers): Root 1.3 is the third child of the first. Alone it becomes your default object; a number lists that node.",
        )
    },
    family: None,
    call_undo: Undo::ReadOnly,
    call_class: CommandClass::Query,
    scope: super::on_master,
    reach: super::show_reach,
    rights: crate::rights::anyone,
    flags: &[],
    planned: &[],
})
.verbs(&[(Verb::List, typed::<ListRoot>), (Verb::Info, typed::<InfoRoot>)]);

/// How the one resolver walks Root (`IdReading::Path`): the declared tree by number (§33 54); a
/// collection's entries are their own objects' (`Fixture 3`, not `Root …`).
pub static PATHS: &[super::PathDef] = &[super::PathDef { object: &ROOT.decl, nodes }];

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `List Root [<n>]` (and `Root <n>`): a node and its children, each with its number.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "List", object = "Root")]
pub struct ListRoot {
    #[lum(target)]
    pub target: Option<Target>,
}

impl crate::commands::typed::Command for ListRoot {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(paths(self.target.as_ref()).map(|ps| ps.iter().map(|p| list(state, p)).collect::<Vec<_>>().join("\n\n")).unwrap_or_else(|e| e))
    }
}

/// `Info Root [<n>]`: where a node is, its number and what is under it.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Info", object = "Root")]
pub struct InfoRoot {
    #[lum(target)]
    pub target: Option<Target>,
}

impl crate::commands::typed::Command for InfoRoot {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(paths(self.target.as_ref()).map(|ps| ps.iter().map(|p| info(state, p)).collect::<Vec<_>>().join("\n")).unwrap_or_else(|e| e))
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Every declared node under Root with its numbers, depth first (the resolver's `PathDef`).
fn nodes(_: &lumacore_tree::Root) -> Vec<(Vec<u32>, crate::vfs::DynChild)> {
    fn walk(node: Node, path: &mut Vec<u32>, out: &mut Vec<(Vec<u32>, crate::vfs::DynChild)>) {
        for child in node.children() {
            let Some(no) = child.no() else { continue };
            path.push(no);
            let below = child.children().count();
            out.push((
                path.clone(),
                crate::vfs::DynChild {
                    display_name: child.name().to_string(),
                    id: no,
                    node_type: "Root",
                    is_leaf: below == 0,
                    child_count: below,
                    name: Some(child.name().to_string()),
                    guid: None,
                },
            ));
            walk(child, path, out);
            path.pop();
        }
    }
    let mut out = Vec::new();
    walk(Node::Root, &mut Vec::new(), &mut out);
    out
}

/// The nodes a Root target names: none = the top; each id a path of numbers or one name.
fn paths(target: Option<&Target>) -> Result<Vec<TreePath>, String> {
    let Some(target) = target else { return Ok(vec![TreePath::root()]) };
    let mut found = Vec::new();
    for (_, item) in &target.items {
        let Item::Path(segments) = item else {
            return Err(render_code("Root takes tree numbers or a quoted name: Root 1.3, Root \"ShowData\".", "E-CMD-008"));
        };
        let Some(ids) = segments.first().and_then(|s| s.ids.as_ref()) else {
            found.push(TreePath::root());
            continue;
        };
        for (_, range) in &ids.ranges {
            let Range::One(id) = range else {
                return Err(render_code("Root names one node at a time (no Thru): join them with +.", "E-CMD-008"));
            };
            let parts: Vec<&str> = match id {
                Id::Number(n) => n.node.split('.').collect(),
                Id::Name(n) => vec![n.node.as_str()],
            };
            let mut path = TreePath::root();
            for part in parts {
                path = path.join(part).ok_or_else(|| render_code(format!("{path} has no node {part}."), "E-CMD-007"))?;
            }
            if path.steps().contains(&Step::Node(Node::Drives)) {
                let drives = Node::Drives.path();
                return Err(render_code(format!("{drives} is the tree shell's: ls {drives}."), "E-CMD-010"));
            }
            found.push(path);
        }
    }
    Ok(found)
}

/// The declared children below `path` (an entry's are its schema's).
fn declared(path: &TreePath) -> Vec<Node> {
    let node = path.node();
    match path.entry() {
        Some(_) => node.entry_schema().map(|s| s.children().collect()).unwrap_or_default(),
        None => node.children().collect(),
    }
}

/// `path` as numbers: `Root 1.3` (an entry's number is its own).
fn numbers(path: &TreePath) -> String {
    let nos: Vec<String> = path
        .steps()
        .iter()
        .map(|s| match s {
            Step::Node(n) => n.no().map_or_else(|| n.name().to_string(), |no| no.to_string()),
            Step::Entry(e) => e.to_string(),
            Step::Name(name) | Step::Disk(name) => format!("\"{name}\""),
        })
        .collect();
    if nos.is_empty() { "Root".into() } else { format!("Root {}", nos.join(".")) }
}

fn list(state: &AppState, path: &TreePath) -> String {
    let dot = path.to_string();
    let mut lines = vec![format!("{dot} ({})", numbers(path))];
    for child in declared(path) {
        // How much is below it (never read off the disk: Root.Drives is the tree shell's).
        let below = match path.join(child.name()) {
            Some(p) if child != Node::Drives => {
                declared(&p).len() + crate::vfs::dynamic_children_at(&p.to_string(), &state.root).map_or(0, |e| e.len())
            }
            _ => 0,
        };
        let below = if child == Node::Drives { String::new() } else if below == 0 { "  empty".into() } else { format!("  {below}") };
        lines.push(format!("  {:>3}  {}{below}", child.no().unwrap_or(0), child.name()));
    }
    let entries = crate::vfs::dynamic_children_at(&dot, &state.root).unwrap_or_default();
    lines.extend(entries.iter().map(|c| format!("  {:>3}  {}", c.id, c.display_name)));
    if lines.len() == 1 {
        lines.push("  (nothing below)".into());
    }
    lines.join("\n")
}

fn info(state: &AppState, path: &TreePath) -> String {
    let dot = path.to_string();
    let entries = crate::vfs::dynamic_children_at(&dot, &state.root).map_or(0, |e| e.len());
    let under = if path.steps().is_empty() { String::new() } else { format!(" · under {}", path.parent()) };
    format!("{dot} ({}){under} · {} nodes, {entries} entries below", numbers(path), declared(path).len())
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
