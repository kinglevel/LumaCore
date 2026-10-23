// ███████╗███████╗██╗     ███████╗ ██████╗████████╗
// ██╔════╝██╔════╝██║     ██╔════╝██╔════╝╚══██╔══╝
// ███████╗█████╗  ██║     █████╗  ██║        ██║
// ╚════██║██╔══╝  ██║     ██╔══╝  ██║        ██║
// ███████║███████╗███████╗███████╗╚██████╗   ██║
// ╚══════╝╚══════╝╚══════╝╚══════╝ ╚═════╝   ╚═╝
// FILE: select

//! A bare `Tag 1.1` (its default verb, decisions §12 "Tags hardened" 4): select the fixtures
//! tagged with it — with `/R` those of every tag under it too — by the one selection rule.
//!
//! Owns:          SelectTag.
//! Doesn't own:   the selection rule (AppState::select_fixtures).
//! Entry points:  SelectTag.
//! Decisions:     §12 "Tags hardened", §22 (the selection rule).

use lumacore_objects::errors::render_code;
use lumacore_objects::objects::out;
use lumacore_objects::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Tag <path> [/R]` = `SelectFixtures Tag <path> [/R]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "SelectFixtures", object = "Tag")]
pub struct SelectTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(option = "R")]
    pub recursive: bool,
}

impl lumacore_objects::commands::typed::Command for SelectTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(select(state, self).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn select(state: &mut AppState, cmd: SelectTag) -> Result<String, String> {
    let tags = crate::tag::targeted(state, &cmd.target)?;
    let guids: Vec<uuid::Uuid> = tags.iter().map(|(_, g)| *g).collect();
    let reach = crate::tag::reach(state, &guids, cmd.recursive);
    let carriers = crate::tag::carriers(state, &reach);
    let fixtures = crate::tag::fixture_numbers(state, &carriers);
    if fixtures.is_empty() {
        let hint = if cmd.recursive { "" } else { " (add /R for the tags under it)" };
        return Err(render_code(format!("No fixtures carry it{hint}."), "E-CMD-009"));
    }
    let n = state.select_fixtures(&fixtures, lumacore_tree::programmer::SelectMode::Plain)?;
    Ok(format!("{} fixture{} from {} · {n} selected", fixtures.len(), if fixtures.len() == 1 { "" } else { "s" }, tags.iter().map(|(p, _)| p.to_string()).collect::<Vec<_>>().join(", ")))
}
