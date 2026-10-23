// ███████╗██╗  ██╗████████╗██████╗  █████╗  ██████╗████████╗
// ██╔════╝╚██╗██╔╝╚══██╔══╝██╔══██╗██╔══██╗██╔════╝╚══██╔══╝
// █████╗   ╚███╔╝    ██║   ██████╔╝███████║██║        ██║
// ██╔══╝   ██╔██╗    ██║   ██╔══██╗██╔══██║██║        ██║
// ███████╗██╔╝ ██╗   ██║   ██║  ██║██║  ██║╚██████╗   ██║
// ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═╝
// FILE: extract

//! `Extract Sequence <m> [Cue <n>]` (decisions 0293): every preset link in the sequence, or in
//! one cue, becomes the preset's value as it is now.
//!
//! Owns:          ExtractSequence.
//! Doesn't own:   the write (`objects::preset::links::extract_cues`), `Extract Preset`.
//! Entry points:  ExtractSequence.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

use crate::errors::render_code;
use crate::objects::preset::links;
use crate::text::render_success;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Extract", object = "Sequence")]
pub struct ExtractSequence {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
}

impl crate::commands::typed::Command for ExtractSequence {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let mut run = || -> Result<String, String> {
            let scope = links::cue_scope(state, &lumacore_syntax::ast::Value::Target(self.target.clone()))?;
            let place = crate::objects::preset::recast::scope_text(state, scope);
            match links::extract_cues(&mut state.root.show_data, None, scope) {
                0 => Err(render_code(format!("Extract {place}: no preset links there."), "E-CMD-009")),
                n => Ok(render_success(&format!("Extract {place}: {n} value{} plain now.", if n == 1 { "" } else { "s" }))),
            }
        };
        DispatchResult::Output(run().unwrap_or_else(|e| e).into())
    }
}
