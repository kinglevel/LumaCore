// ██████╗ ██████╗     ██╗███╗   ██╗███████╗ ██████╗
// ██╔══██╗██╔══██╗    ██║████╗  ██║██╔════╝██╔═══██╗
// ██║  ██║██████╔╝    ██║██╔██╗ ██║█████╗  ██║   ██║
// ██║  ██║██╔═══╝     ██║██║╚██╗██║██╔══╝  ██║   ██║
// ██████╔╝██║         ██║██║ ╚████║██║     ╚██████╔╝
// ╚═════╝ ╚═╝         ╚═╝╚═╝  ╚═══╝╚═╝      ╚═════╝
// FILE: dp_info

//! `Info DataPool <n>` (decisions 0291): a datapool's identity and each of its pools with its
//! count — the rows `ls` and NAV show below it (`tree_rows`).
//!
//! Owns:          InfoDataPool.
//! Doesn't own:   the rows (tree_rows), the DataPool object (patch.rs).
//! Entry points:  InfoDataPool.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.

use lumacore_syntax::ast::Target;

use crate::objects::out;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Info DataPool <n | "name">`: one datapool per line.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Info", object = "DataPool")]
pub struct InfoDataPool {
    #[lum(target)]
    pub target: Target,
}

impl crate::commands::typed::Command for InfoDataPool {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let resolved = match crate::resolve::resolve_one(state, &self.target, &super::DATAPOOL) {
            Ok(r) => r,
            Err(e) => return out(e),
        };
        let [member] = resolved.members.as_slice() else {
            return out(crate::errors::render_code("Usage: Info DataPool <n | \"name\"> (one datapool)", "E-CMD-006"));
        };
        let Some(dp) = state.root.show_data.datapools.get(&member.no) else {
            return out(crate::errors::render_code(format!("DataPool {} is gone.", member.no), "E-CMD-007"));
        };
        let path = format!("{}.{}", crate::tree_path::Node::DataPools.path(), member.no);
        let title = match dp.meta.name() {
            Some(name) => format!("DataPool {} \"{name}\"", member.no),
            None => format!("DataPool {}", member.no),
        };
        let mut sheet = crate::text::PropertySheet::new(&title)
            .field("GUID", &dp.meta.guid().to_string())
            .field("Path", &path)
            .separator()
            .section("Pools");
        for row in crate::tree_rows::node_rows(state, &path) {
            sheet = sheet.field(&row.name, if row.meta.is_empty() { "—" } else { &row.meta });
        }
        out(sheet.build())
    }
}
