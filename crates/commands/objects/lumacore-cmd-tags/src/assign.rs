//  █████╗ ███████╗███████╗██╗ ██████╗ ███╗   ██╗
// ██╔══██╗██╔════╝██╔════╝██║██╔════╝ ████╗  ██║
// ███████║███████╗███████╗██║██║  ███╗██╔██╗ ██║
// ██╔══██║╚════██║╚════██║██║██║   ██║██║╚██╗██║
// ██║  ██║███████║███████║██║╚██████╔╝██║ ╚████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: assign

//! `Assign Tag … At <objects>` / `Unassign Tag … At <objects>` (decisions §12 "Tags hardened" 5,
//! lum.md): a tag link added to or taken from any show object's NodeMeta — a tag included (that
//! tags the tag; nesting is Move). One Oops step for the line.
//!
//! Owns:          AssignTag, UnassignTag.
//! Doesn't own:   the links' edit (links.rs), the resolver (lumacore-objects).
//! Entry points:  AssignTag, UnassignTag.
//! Decisions:     §12 "Tags hardened".

use lumacore_objects::errors::render_code;
use lumacore_objects::objects::out;
use lumacore_objects::{AppState, DispatchResult};
use lumacore_syntax::ast::Value;
use uuid::Uuid;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Assign Tag <path> At <objects>`: tag them.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Assign", object = "Tag")]
pub struct AssignTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "destination")]
    pub to: Value,
}

impl lumacore_objects::commands::typed::Command for AssignTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(retag(state, &self.target, &self.to, true).unwrap_or_else(|e| e))
    }
}

/// `Unassign Tag <path> At <objects>`: untag them.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Unassign", object = "Tag")]
pub struct UnassignTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "destination")]
    pub to: Value,
}

impl lumacore_objects::commands::typed::Command for UnassignTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(retag(state, &self.target, &self.to, false).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn retag(state: &mut AppState, target: &lumacore_syntax::ast::Target, to: &Value, add: bool) -> Result<String, String> {
    let tags = crate::tag::targeted(state, target)?;
    let Value::Target(objects) = to else {
        return Err(render_code("Assign Tag … At names the objects to tag: At Fixture 1 Thru 12, At Sequence 3.", "E-CMD-008"));
    };
    let items = lumacore_objects::resolve::resolve(state, objects, &lumacore_objects::objects::FIXTURE)?;
    let mut guids: Vec<Uuid> = Vec::new();
    let mut kinds: Vec<&str> = Vec::new();
    for (op, item) in &items {
        let found: Vec<Uuid> = item.members.iter().filter_map(|m| m.guid).collect();
        if found.len() < item.members.len() {
            return Err(render_code(format!("A {} can't carry tags yet.", item.object.keyword), "E-CMD-010"));
        }
        match op {
            Some(lumacore_syntax::ast::SetOp::Minus) => guids.retain(|g| !found.contains(g)),
            _ => {
                for g in found {
                    if !guids.contains(&g) {
                        guids.push(g);
                    }
                }
                kinds.push(item.object.keyword);
            }
        }
    }
    if guids.is_empty() {
        return Err(render_code("Nothing to tag.", "E-CMD-009"));
    }
    let tag_guids: Vec<Uuid> = tags.iter().map(|(_, g)| *g).collect();
    let names: Vec<String> = tags.iter().map(|(p, _)| crate::tag::label(state, p)).collect();
    kinds.dedup();
    let what = format!("{} {}{}", guids.len(), kinds.join(" / ").to_lowercase(), if guids.len() == 1 { "" } else { "s" });
    // Nothing to change (a tap on a cell that carries the armed tag, decisions 0297): say so,
    // and the line files no Oops step.
    if let Some(reply) = unchanged(state, &guids, &tag_guids, add) {
        return Ok(format!("{what} {reply} {}", names.join(", ")));
    }
    let (add_list, remove_list): (&[Uuid], &[Uuid]) = if add { (&tag_guids, &[]) } else { (&[], &tag_guids) };
    let missing = crate::links::retag(state, &guids, add_list, remove_list)?;
    if !missing.is_empty() {
        return Err(render_code(format!("{} of them can't carry tags yet (no NodeMeta).", missing.len()), "E-CMD-010"));
    }
    Ok(if add { format!("Tagged {what} with {}", names.join(", ")) } else { format!("Untagged {what} from {}", names.join(", ")) })
}

/// "already carries" / "doesn't carry" when the line would change nothing: every object already
/// carries every tag (Assign), or none carries any (Unassign).
fn unchanged(state: &AppState, guids: &[Uuid], tags: &[Uuid], add: bool) -> Option<&'static str> {
    let walk = state.root.show_data.walk_identity();
    let carriers: Vec<Vec<Uuid>> = tags.iter().map(|t| lumacore_tree::tags::tag::members(&walk, &[*t]).iter().map(|m| m.guid).collect()).collect();
    let one = guids.len() == 1;
    if add && carriers.iter().all(|c| guids.iter().all(|g| c.contains(g))) {
        return Some(if one { "already carries" } else { "already carry" });
    }
    if !add && carriers.iter().all(|c| guids.iter().all(|g| !c.contains(g))) {
        return Some(if one { "doesn't carry" } else { "don't carry" });
    }
    None
}

