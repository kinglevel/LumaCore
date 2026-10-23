// ██╗      █████╗ ██████╗ ███████╗██╗
// ██║     ██╔══██╗██╔══██╗██╔════╝██║
// ██║     ███████║██████╔╝█████╗  ██║
// ██║     ██╔══██║██╔══██╗██╔══╝  ██║
// ███████╗██║  ██║██████╔╝███████╗███████╗
// ╚══════╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝
// FILE: label

//! `Label Tag` (decisions §12 "Tags hardened" 3): a new name, unique among the tag's siblings.
//!
//! Owns:          LabelTag.
//! Doesn't own:   the tree (lumacore-tags).
//! Entry points:  LabelTag.
//! Decisions:     §12 "Tags hardened".

use lumacore_objects::errors::{render, render_code};
use lumacore_objects::objects::out;
use lumacore_objects::resolve::check_name;
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tags::tag as tree;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Label Tag <path> "name"` (several tags get the name with 2, 3, … after it).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Label", object = "Tag")]
pub struct LabelTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(value)]
    pub name: Option<String>,
}

impl lumacore_objects::commands::typed::Command for LabelTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(label(state, self).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn label(state: &mut AppState, cmd: LabelTag) -> Result<String, String> {
    let targets = crate::tag::targeted(state, &cmd.target)?;
    let Some(base) = cmd.name else {
        return Err(render_code("Label Tag takes the new name: Label Tag 1.1 \"Lx1\".", "E-CMD-006"));
    };
    check_name(&base)?;
    let many = targets.len() > 1;
    let mut done = Vec::new();
    for (i, (path, guid)) in targets.iter().enumerate() {
        let name = if many { format!("{base} {}", i + 1) } else { base.clone() };
        rename(state, path, *guid, Some(name.clone()))?;
        done.push(format!("{path} \"{name}\""));
    }
    Ok(format!("Labelled Tag {}", done.join(", ")))
}

/// Give the tag at `path` a new name (None clears it), refused when a sibling has it.
pub(crate) fn rename(state: &mut AppState, path: &lumacore_tags::TagPath, guid: uuid::Uuid, name: Option<String>) -> Result<(), String> {
    let tags = &mut state.root.show_data.tags;
    if let Some(name) = &name {
        let siblings = tree::level(tags, path.parent().as_ref()).map(|l| l.values().any(|t| t.meta.identity.guid != guid && t.name().eq_ignore_ascii_case(name)));
        if siblings == Some(true) {
            let parent = path.parent().map(|p| format!("tag {p}")).unwrap_or_else(|| "the top".into());
            return Err(render(&lumacore_tags::TagError::NameTaken { name: name.clone(), parent }));
        }
    }
    if let Some(t) = tree::find_mut(tags, path) {
        t.meta.identity.name = name;
    }
    Ok(())
}
