// ████████╗███████╗███████╗████████╗    ██╗    ██╗ ██████╗ ██████╗ ██████╗ ███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝    ██║    ██║██╔═══██╗██╔══██╗██╔══██╗██╔════╝
//    ██║   █████╗  ███████╗   ██║       ██║ █╗ ██║██║   ██║██████╔╝██║  ██║███████╗
//    ██║   ██╔══╝  ╚════██║   ██║       ██║███╗██║██║   ██║██╔══██╗██║  ██║╚════██║
//    ██║   ███████╗███████║   ██║       ╚███╔███╔╝╚██████╔╝██║  ██║██████╔╝███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝        ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: test words

//! The kernel's stand-in words (decisions §34 25.16): rows only the kernel's own test binary
//! registers, so its machinery tests — dialogs, the rights gate, flags, `;` — never depend on
//! which command crate a real word lives in (VS Code's command service tests register `foo` /
//! `test` the same way). Never in the product's catalog.
//!
//!
//! Owns:          TEST_ROWS: TestAsk (asks, Admin), TestEcho, TestInternal (no user runs it).
//! Doesn't own:   the real words (each command crate), the dispatch they exercise.
//! Entry points:  TEST_ROWS (the kernel's test_catalog).
//! Decisions:     §34 25.16.

use crate::commands::registry::*;
use crate::objects::out;
use crate::{AppState, DispatchResult};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `TestAsk [/NoConfirmation]`: asks first (the standard dialog), the answer toggles debug mode.
fn test_ask(state: &mut AppState, args: &[&str]) -> DispatchResult {
    if crate::commands::flags::has(args, &crate::commands::flags::NO_CONFIRMATION) {
        state.debug_mode = !state.debug_mode;
        return out(format!("debug mode {}", if state.debug_mode { "on" } else { "off" }));
    }
    DispatchResult::Ask(Box::new(
        crate::dialogs::Ask::new("TestAsk", "Toggle debug mode?")
            .button("Toggle", lumacore_wire::ButtonKind::Primary, "TestAsk /NoConfirmation")
            .cancel("Cancel"),
    ))
}

/// The stand-in rows, added after the kernel's own.
pub(crate) static TEST_ROWS: &[CommandDef] = &[
    CommandDef {
        run: test_ask,
        decl: CommandDecl {
            verb: "testask",
            doc: Doc {
                display: "TestAsk",
                category: HelpCategory::System,
                syntax: "TestAsk [/NoConfirmation]",
                description: "A test word: asks, then toggles debug mode (Admin).",
                hidden: true,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Station,
            rights: Rights::Admin,
            class: CommandClass::DangerousRuntime,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, a| out(a.join(" ").trim_matches('"').to_string()),
        decl: CommandDecl {
            verb: "testecho",
            doc: Doc {
                display: "TestEcho",
                category: HelpCategory::System,
                syntax: "TestEcho text",
                description: "A test word: replies with its text.",
                hidden: true,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::Words,
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| out("internal".to_string()),
        decl: CommandDecl {
            verb: "testinternal",
            doc: Doc {
                display: "TestInternal",
                category: HelpCategory::System,
                syntax: "TestInternal",
                description: "A test word only the station itself may run (Rights::Internal).",
                hidden: true,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Station,
            rights: Rights::Internal,
            class: CommandClass::DangerousRuntime,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];
