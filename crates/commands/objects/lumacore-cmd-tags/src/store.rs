// ███████╗████████╗ ██████╗ ██████╗ ███████╗
// ██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
// ███████╗   ██║   ██║   ██║██████╔╝█████╗
// ╚════██║   ██║   ██║   ██║██╔══██╗██╔══╝
// ███████║   ██║   ╚██████╔╝██║  ██║███████╗
// ╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: store

//! `Store Tag` (decisions §12 "Tags hardened" 5): a new tag at a path, under an owner by name, or
//! at the next free number; its name unique among its siblings.
//!
//! Owns:          StoreTag.
//! Doesn't own:   the slot rule (resolve::path_slot), the tree (lumacore-tags).
//! Entry points:  StoreTag.
//! Decisions:     §12 "Tags hardened".

use lumacore_objects::errors::{render, render_code};
use lumacore_objects::objects::out;
use lumacore_objects::resolve::{PathSlot, check_name, path_slot};
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tags::tag as tree;
use lumacore_tags::{Tag, TagPath};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Store Tag [<path> | <owner> Tag "name"] ["name"]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Tag")]
pub struct StoreTag {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
    #[lum(value)]
    pub name: Option<String>,
}

impl lumacore_objects::commands::typed::Command for StoreTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(store(state, self).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn store(state: &mut AppState, cmd: StoreTag) -> Result<String, String> {
    let slot = path_slot(state, &crate::TAG, cmd.target.as_ref())?;
    if let Some(n) = &cmd.name {
        check_name(n)?;
    }
    let tags = &mut state.root.show_data.tags;
    let (path, name) = match slot {
        PathSlot::At(p) => (TagPath(p), cmd.name),
        PathSlot::Named { parent, name } => {
            if cmd.name.is_some() {
                return Err(render_code("Store Tag takes one name: Store Tag \"Lx1\", or Store Tag 1.2 \"Lx1\".", "E-CMD-006"));
            }
            check_name(&name)?;
            (next(tags, parent)?, Some(name))
        }
        PathSlot::Next(parent) => (next(tags, parent)?, cmd.name),
    };
    tree::insert(tags, &path, Tag::new(name)).map_err(|e| render(&e))?;
    Ok(format!("Stored {}", crate::tag::label(state, &path)))
}

/// The next free number under `parent` (the top when empty).
fn next(tags: &lumacore_tags::TagTree, parent: Vec<u32>) -> Result<TagPath, String> {
    let at = (!parent.is_empty()).then(|| TagPath(parent.clone()));
    let level = tree::level(tags, at.as_ref()).ok_or_else(|| render(&lumacore_tags::TagError::NoSuchTag(TagPath(parent.clone()).to_string())))?;
    let mut p = parent;
    p.push(tree::next_free(level));
    Ok(TagPath(p))
}
