// ██╗    ██╗ ██████╗ ██████╗ ██████╗ ███████╗
// ██║    ██║██╔═══██╗██╔══██╗██╔══██╗██╔════╝
// ██║ █╗ ██║██║   ██║██████╔╝██║  ██║███████╗
// ██║███╗██║██║   ██║██╔══██╗██║  ██║╚════██║
// ╚███╔███╔╝╚██████╔╝██║  ██║██████╔╝███████║
//  ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: words

//! The tree shell's words (decisions §33 decision 18): each is a fence word, read as its own
//! Unix-like line (`ls -r Root.ShowData`), never as Lum.
//!
//! Owns:          the tree shell's fence words (one struct per word on its fence line), their Unix flags, Exit.
//! Doesn't own:   the rows (rows.rs), what each word prints (read, tree, search, count, diff), the walk (the kernel's navigate).
//! Entry points:  Cd, Pwd, Ls, Cat, Head, Tail, Stat, Tree, Find, Grep, Du, Wc, Diff, exit, the flags.
//! Decisions:     §33 18, §34 25.7.

use lumacore_syntax::FenceLine;

use lumacore_objects::claims::Claims;
use lumacore_objects::commands::flags::FlagDef;
use lumacore_objects::commands::typed::FenceCommand;
use lumacore_objects::errors::reply_error;
use lumacore_objects::navigation::Navigation;
use lumacore_objects::objects::out;
use lumacore_objects::text::render_success;
use lumacore_objects::{AppState, DispatchResult, ReplContext, Reply};

use crate::count::{cmd_du, cmd_wc};
use crate::diff::cmd_diff;
use crate::read::{cmd_cat, cmd_head, cmd_stat, cmd_tail};
use crate::search::{cmd_find, cmd_grep};
use crate::tree::cmd_tree;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

// The fence words' Unix flags (decisions §33 decision 18): `-x` for one letter, `--name` else.

/// `tree -d 2`: how deep to draw.
pub const TREE_DEPTH: FlagDef = FlagDef::new("d", "How deep to draw (`tree -d 2`)").value("levels");
/// `head -n 5`, `tail -n 5`.
pub const LINES: FlagDef = FlagDef::new("n", "How many entries (default 10)").value("count");
/// `find "Front" --type Group`.
pub const FIND_TYPE: FlagDef = FlagDef::new("type", "Only nodes of this type").value("type");
/// `diff a b --all`.
pub const DIFF_ALL: FlagDef = FlagDef::new("all", "Show the fields that are the same too");

/// One tree-shell word: its handler on the fence line (`fenced::<Cat>`).
macro_rules! fence_word {
    ($ty:ident, $word:literal, $paths:literal, |$s:ident, $l:ident| $body:expr) => {
        pub struct $ty;
        impl FenceCommand for $ty {
            const WORD: &'static str = $word;
            const PATHS: usize = $paths;
            fn run($s: &mut AppState, $l: &FenceLine) -> DispatchResult {
                $body
            }
        }
    };
}

fence_word!(Cd, "cd", 1, |s, l| out(
    lumacore_objects::commands::navigate::cmd_cd(s, l)
));
fence_word!(Pwd, "pwd", 0, |s, _l| out(s.path_string()));
fence_word!(Ls, "ls", 1, |s, l| out(
    lumacore_objects::commands::navigate::cmd_ls(s, l)
));
fence_word!(Cat, "cat", 1, |s, l| out(cmd_cat(s, l)));
fence_word!(Head, "head", 1, |s, l| out(cmd_head(s, l)));
fence_word!(Tail, "tail", 1, |s, l| out(cmd_tail(s, l)));
fence_word!(Stat, "stat", 1, |s, l| out(cmd_stat(s, l)));
fence_word!(Tree, "tree", 1, |s, l| out(cmd_tree(s, l)));
fence_word!(Find, "find", 1, |s, l| out(cmd_find(s, l)));
fence_word!(Grep, "grep", 2, |s, l| out(cmd_grep(s, l)));
fence_word!(Du, "du", 1, |s, l| out(cmd_du(s, l)));
fence_word!(Wc, "wc", 1, |s, l| out(cmd_wc(s, l)));
fence_word!(Diff, "diff", 2, |s, l| out(cmd_diff(s, l)));

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// Exit — unclaim the current node and reset context to root. Does NOT change
/// navigation_path (tree position stays the same).
pub(crate) fn exit(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    let Some(obj) = state.claimable_at_path() else {
        return DispatchResult::Output(reply_error(
            "Nothing to exit. Use 'shutdown' to quit LumaCore.",
            "E-CMD-009",
        ));
    };
    if state.is_claimed_by_me(&obj) {
        let _ = state.unclaim(&obj);
        state.context = ReplContext::Fixture;
        DispatchResult::Output(Reply::ok(render_success(&format!("{obj} released."))))
    } else {
        DispatchResult::Output(reply_error(
            "Nothing to exit — you don't have a claim here.",
            "E-CMD-009",
        ))
    }
}
