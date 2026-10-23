// ██╗     ██╗███████╗████████╗
// ██║     ██║██╔════╝╚══██╔══╝
// ██║     ██║███████╗   ██║
// ██║     ██║╚════██║   ██║
// ███████╗██║███████║   ██║
// ╚══════╝╚═╝╚══════╝   ╚═╝
// FILE: list

//! `List Tag` (the tree, with what carries each tag) and `Info Tag` (one tag: its place, children,
//! action and carriers; `/R` counts those of the tags under it too).
//!
//! Owns:          ListTag, InfoTag.
//! Doesn't own:   the tree (lumacore-tags).
//! Entry points:  ListTag, InfoTag.
//! Decisions:     §12 "Tags hardened".

use lumacore_objects::objects::out;
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tags::tag as tree;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `List Tag`: every tag, indented by level, with direct / deep carrier counts.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "List", object = "Tag")]
pub struct ListTag {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
}

impl lumacore_objects::commands::typed::Command for ListTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(list(state, self.target.as_ref()).unwrap_or_else(|e| e))
    }
}

/// `Info Tag <path> [/R]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Info", object = "Tag")]
pub struct InfoTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(option = "R")]
    pub recursive: bool,
}

impl lumacore_objects::commands::typed::Command for InfoTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(info(state, self).unwrap_or_else(|e| e))
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Every tag, or the subtrees of the tags `target` names (`List Tag 1`).
fn list(state: &AppState, target: Option<&lumacore_syntax::ast::Target>) -> Result<String, String> {
    let tags = &state.root.show_data.tags;
    let roots: Option<Vec<lumacore_tags::TagPath>> = match target {
        Some(t) => Some(crate::tag::targeted(state, t)?.into_iter().map(|(p, _)| p).collect()),
        None => None,
    };
    let all: Vec<_> = tree::all(tags).into_iter().filter(|(p, _)| roots.as_ref().is_none_or(|r| r.iter().any(|root| root.contains(p)))).collect();
    if all.is_empty() {
        return Ok("No tags. Store Tag \"All\" makes the first.".into());
    }
    let walk = state.root.show_data.walk_identity();
    let mut lines = vec!["Tags".to_string()];
    for (path, tag) in all {
        let direct = tree::members(&walk, &[tag.meta.identity.guid]).len();
        let deep = tree::members(&walk, &tree::subtree(tag)).len();
        let indent = "  ".repeat(path.0.len().saturating_sub(1));
        let name = tag.meta.identity.name.as_deref().map(|n| format!(" \"{n}\"")).unwrap_or_default();
        lines.push(format!("  {indent}{path}{name}  · {direct} direct · {deep} with /R"));
    }
    Ok(lines.join("\n"))
}

fn info(state: &AppState, cmd: InfoTag) -> Result<String, String> {
    let found = crate::tag::targeted(state, &cmd.target)?;
    let tags = &state.root.show_data.tags;
    let mut out = Vec::new();
    for (path, guid) in found {
        let Some(tag) = tree::find(tags, &path) else { continue };
        let reach = crate::tag::reach(state, &[guid], cmd.recursive);
        let carriers = crate::tag::carriers(state, &reach);
        let kids: Vec<String> = tag.children.iter().map(|(n, c)| format!("{} \"{}\"", path.child(*n), c.name())).collect();
        out.push(crate::tag::label(state, &path));
        out.push(format!("  guid {guid}"));
        out.push(format!("  action {:?}", tag.action));
        out.push(format!("  child tags {}", if kids.is_empty() { "none".to_string() } else { kids.join(", ") }));
        out.push(format!("  carried by {}{}", crate::tag::kinds_text(&carriers), if cmd.recursive { " (with the tags under it)" } else { "" }));
    }
    Ok(out.join("\n"))
}
