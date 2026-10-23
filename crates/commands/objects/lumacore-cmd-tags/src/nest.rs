// ███╗   ██╗███████╗███████╗████████╗
// ████╗  ██║██╔════╝██╔════╝╚══██╔══╝
// ██╔██╗ ██║█████╗  ███████╗   ██║
// ██║╚██╗██║██╔══╝  ╚════██║   ██║
// ██║ ╚████║███████╗███████║   ██║
// ╚═╝  ╚═══╝╚══════╝╚══════╝   ╚═╝
// FILE: nest

//! `Move Tag` (decisions §12 "Tags hardened", lum.md): `At Tag <path>` puts a tag (with its
//! subtree) under another, at the next free number there — asked first, the one nesting rule
//! (§34 "34g4 part 3c hardened" 3); `At Root` to the top; `At <n>` gives it another number under
//! its own parent. GUIDs stay, so what carries the tag never notices.
//!
//! Owns:          MoveTag.
//! Doesn't own:   the tree's moves (lumacore-tags).
//! Entry points:  MoveTag.
//! Decisions:     §12 "Tags hardened", §34 "34g4 part 3c hardened" 3.

use lumacore_objects::errors::{render, render_code};
use lumacore_objects::objects::out;
use lumacore_objects::{AppState, DispatchResult};
use lumacore_syntax::ast::Value;
use lumacore_tags::tag as tree;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Move Tag <path> At Tag <path> | Root | <n>`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Move", object = "Tag")]
pub struct MoveTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "destination")]
    pub to: Value,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl lumacore_objects::commands::typed::Command for MoveTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        nest(state, self).unwrap_or_else(out)
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Where a Move puts the tags.
enum Dest {
    Under(uuid::Uuid),
    Top,
    Number(u32),
}

fn nest(state: &mut AppState, cmd: MoveTag) -> Result<DispatchResult, String> {
    let found = crate::tag::targeted(state, &cmd.target)?;
    let dest = match &cmd.to {
        Value::Number(n) => Dest::Number(n.node.parse().map_err(|_| render_code(format!("{} isn't a tag number under the same parent.", n.node), "E-CMD-008"))?),
        Value::Target(t) => {
            let items = lumacore_objects::resolve::resolve(state, t, &crate::TAG)?;
            match items.as_slice() {
                [(None, r)] if r.object.keyword == "Root" => Dest::Top,
                [(None, r)] if std::ptr::eq(r.object, &crate::TAG) => {
                    let [m] = r.members.as_slice() else { return Err(render_code("Move Tag … At names one Tag.", "E-CMD-008")) };
                    Dest::Under(m.guid.ok_or_else(|| render_code("No such Tag.", "E-TAG-001"))?)
                }
                _ => return Err(render_code("Move Tag … At Tag <path>, At Root or At <number>.", "E-CMD-008")),
            }
        }
        _ => return Err(render_code("Move Tag … At Tag <path>, At Root or At <number>.", "E-CMD-008")),
    };
    if matches!(dest, Dest::Number(_)) && found.len() > 1 {
        return Err(render_code("Move Tag … At <number> moves one tag.", "E-CMD-008"));
    }
    if let Dest::Under(g) = dest
        && !cmd.confirmed
    {
        // What can't be done is refused before the question (under itself, a name taken there).
        let mut trial = state.root.show_data.tags.clone();
        for (_, guid) in &found {
            let (Some(from), Some(under)) = (tree::path_of(&trial, *guid), tree::path_of(&trial, g)) else { continue };
            tree::move_under(&mut trial, &from, Some(&under)).map_err(|e| render(&e))?;
        }
        return Ok(ask(state, &found, g));
    }
    let mut done = Vec::new();
    for (_, guid) in &found {
        let tags = &mut state.root.show_data.tags;
        let Some(from) = tree::path_of(tags, *guid) else { continue };
        let to = match &dest {
            Dest::Top => tree::move_under(tags, &from, None),
            Dest::Under(g) => {
                let under = tree::path_of(tags, *g).ok_or_else(|| render_code("No such Tag.", "E-TAG-001"))?;
                tree::move_under(tags, &from, Some(&under))
            }
            Dest::Number(n) => tree::renumber(tags, &from, *n),
        }
        .map_err(|e| render(&e))?;
        done.push(format!("{from} → {to}"));
    }
    Ok(out(format!("Moved Tag {}", done.join(", "))))
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// "Tag 1.2 'Right' becomes a child of Tag 2 'Mid' (3 tags, 12 objects)?" — the one nesting ask.
fn ask(state: &AppState, found: &[(lumacore_tags::TagPath, uuid::Uuid)], under: uuid::Uuid) -> DispatchResult {
    let tags = &state.root.show_data.tags;
    let what = match found {
        [(p, _)] => crate::tag::label(state, p),
        _ => format!("{} tags", found.len()),
    };
    let to = tree::path_of(tags, under).map(|p| crate::tag::label(state, &p)).unwrap_or_else(|| "Tag".into());
    let moved: Vec<uuid::Uuid> = found.iter().map(|(_, g)| *g).collect();
    let reach = crate::tag::reach(state, &moved, true);
    let carried = crate::tag::carriers(state, &reach).len();
    let counts = format!("{} tag{}, {carried} object{}", reach.len(), if reach.len() == 1 { "" } else { "s" }, if carried == 1 { "" } else { "s" });
    let paths: Vec<String> = found.iter().map(|(p, _)| p.to_string()).collect();
    let dest = tree::path_of(tags, under).map(|p| p.to_string()).unwrap_or_default();
    lumacore_objects::commands::door::nest_ask(&what, &to, &counts, &format!("Move Tag {} At Tag {dest}", paths.join(" + ")))
}
