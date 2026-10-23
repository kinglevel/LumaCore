// ██████╗ ███████╗██╗     ███████╗████████╗███████╗
// ██╔══██╗██╔════╝██║     ██╔════╝╚══██╔══╝██╔════╝
// ██║  ██║█████╗  ██║     █████╗     ██║   █████╗
// ██║  ██║██╔══╝  ██║     ██╔══╝     ██║   ██╔══╝
// ██████╔╝███████╗███████╗███████╗   ██║   ███████╗
// ╚═════╝ ╚══════╝╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: delete

//! `Delete Tag` (decisions §12 "Tags hardened" 5): one question naming the child tags and what
//! carries the tag — With children / Keep children (they move up one level) / Cancel. Objects are
//! never deleted, only their links to the tags that go. One Oops step.
//!
//! Owns:          DeleteTag, ask (the one Delete question).
//! Doesn't own:   the tree (lumacore-tags), the links' edit (links.rs).
//! Entry points:  DeleteTag, ask.
//! Decisions:     §12 "Tags hardened", §30 "Merge of bugfixes" (one Delete question).

use lumacore_objects::dialogs::Ask;
use lumacore_objects::errors::render_code;
use lumacore_objects::objects::{DeleteAsk, out};
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tags::TagPath;
use lumacore_tags::tag as tree;
use lumacore_wire::dialog::ButtonKind;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Delete Tag <path> [/WithChildren | /KeepChildren]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "Tag")]
pub struct DeleteTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(option = "WithChildren")]
    pub with_children: bool,
    #[lum(option = "KeepChildren")]
    pub keep_children: bool,
}

impl lumacore_objects::commands::typed::Command for DeleteTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(delete(state, self).unwrap_or_else(|e| e))
    }
}

/// The one Delete question (dispatch asks it before the line runs; `/NoConfirmation` skips it).
pub fn ask(state: &AppState, args: &[&str]) -> DeleteAsk {
    let words: Vec<&str> = args.iter().copied().filter(|w| !w.starts_with('/')).collect();
    let flagged = |f: &str| args.iter().any(|w| w.trim_start_matches('/').eq_ignore_ascii_case(f));
    let target = match lumacore_objects::resolve::target_of(&std::iter::once("Tag").chain(words.iter().copied()).collect::<Vec<_>>().join(" ")) {
        Ok(t) => t,
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let found = match crate::tag::targeted(state, &target) {
        Ok(f) => f,
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let tags = &state.root.show_data.tags;
    let paths: Vec<String> = found.iter().map(|(p, _)| p.to_string()).collect();
    let names: Vec<String> = found.iter().map(|(p, _)| crate::tag::label(state, p)).collect();
    let kids: Vec<String> = found
        .iter()
        .filter_map(|(p, _)| tree::find(tags, p))
        .flat_map(|t| t.children.values().map(|c| c.name().to_string()).collect::<Vec<_>>())
        .collect();
    let guids: Vec<uuid::Uuid> = found.iter().map(|(_, g)| *g).collect();
    let carriers = crate::tag::carriers(state, &guids);
    let line = format!("Delete Tag {}", paths.join(" + "));
    let mut text = format!("Delete {}?", names.join(", "));
    if !kids.is_empty() {
        text.push_str(&format!("\n{} child tag{}: {}", kids.len(), if kids.len() == 1 { "" } else { "s" }, kids.join(", ")));
    }
    text.push_str(&format!("\n{} lose{} the tag. No object is deleted, only links.", crate::tag::kinds_text(&carriers), if carriers.len() == 1 { "s" } else { "" }));
    let ask = Ask::new("Delete", text);
    let ask = if kids.is_empty() || flagged("WithChildren") || flagged("KeepChildren") {
        let flag = if flagged("KeepChildren") { " /KeepChildren" } else if flagged("WithChildren") { " /WithChildren" } else { "" };
        ask.button("Delete", ButtonKind::Danger, format!("{line}{flag} /NoConfirmation"))
    } else {
        ask.button("With children", ButtonKind::Danger, format!("{line} /WithChildren /NoConfirmation"))
            .button("Keep children", ButtonKind::Primary, format!("{line} /KeepChildren /NoConfirmation"))
    };
    DeleteAsk::Ask(Box::new(ask.cancel("Cancel").flags("/NoConfirmation")))
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn delete(state: &mut AppState, cmd: DeleteTag) -> Result<String, String> {
    if cmd.with_children && cmd.keep_children {
        return Err(render_code("Delete Tag takes /WithChildren or /KeepChildren, not both.", "E-CMD-006"));
    }
    let found = crate::tag::targeted(state, &cmd.target)?;
    let mut gone: Vec<uuid::Uuid> = Vec::new();
    let mut lifted: Vec<String> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for (_, guid) in &found {
        // Paths move when a tag before it lifts its children: find it again by GUID.
        let tags = &mut state.root.show_data.tags;
        let Some(path) = tree::path_of(tags, *guid) else { continue };
        let has_children = tree::find(tags, &path).is_some_and(|t| !t.children.is_empty());
        if has_children && !cmd.with_children && !cmd.keep_children {
            return Err(render_code(
                format!("Tag {path} has child tags: Delete Tag {path} /WithChildren, or /KeepChildren to move them up one level."),
                "E-CMD-006",
            ));
        }
        if cmd.keep_children {
            lifted.extend(tree::lift_children(tags, &path).iter().map(TagPath::to_string));
        }
        names.push(crate::tag::label_in(tags, &path));
        if let Some(t) = tree::remove(tags, &path) {
            gone.extend(tree::subtree(&t));
        }
    }
    let lost = crate::links::strip(state, &gone)?;
    let mut text = format!("Deleted {}", names.join(", "));
    if lost > 0 {
        text.push_str(&format!(" · {lost} object{} lost the tag", if lost == 1 { "" } else { "s" }));
    }
    if !lifted.is_empty() {
        text.push_str(&format!(" · child tags now {}", lifted.join(", ")));
    }
    Ok(text)
}
