// ██████╗  ██████╗ ██╗    ██╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: rows

//! The command rows of key bindings: Bind, Unbind, Shortcuts (decisions §34 25.7, 25.13: each crate
//! of 34g4 takes its file).
//!
//! Owns:          the keys family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use lumacore_objects::commands::registry::*;
use lumacore_objects::objects::{ObjectDef, out};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The keys' commands with their handlers; `register` adds them to the catalog.
pub static ROWS: &[CommandDef] = &[
    CommandDef {
        run: crate::shortcuts::shortcuts,
        decl: CommandDecl {
            verb: "shortcuts",
            aliases: &["sc"],
            doc: Doc {
                display: "Shortcuts",
                category: HelpCategory::Programmer,
                syntax: "Shortcuts [Keyboard|Programmer|Navigator|On|Off]",
                description: "Cycle or set shortcut mode. Keyboard (KBD) = normal typing, \
                          Programmer (PRG) = single-key programmer shortcuts (A = At, T = Thru, \
                          etc.), Navigator (NAV) = tree browser with arrow-key navigation. The \
                          short names work as values too. On = Programmer, Off = Keyboard. \
                          Vanilla key: F10.",
                examples: &[
                    "Shortcuts             -- cycle Keyboard → Programmer → Navigator",
                    "Shortcuts Programmer  -- set Programmer mode (also Shortcuts PRG)",
                    "Shortcuts Keyboard    -- set Keyboard mode (also Shortcuts KBD)",
                    "Shortcuts Navigator   -- set Navigator mode (also Shortcuts NAV)",
                ],
                related: &["at", "full"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Settings),
            scope: Scope::Client,
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::bind::bind(s, a)),
        decl: CommandDecl {
            verb: "bind",
            doc: Doc {
                display: "Bind",
                category: HelpCategory::System,
                syntax: "Bind <key> `<command line>` [`<line>` …] | <UiKey> [/Shortcuts KBD|PRG|NAV] [/Description \"…\"]",
                description: "Bind a keyboard key for your profile (decisions §17: key → UIKey → \
                          command lines), in every mode or in one shortcut mode (0295). A UIKey id \
                          binds that key; command lines go into your UIKey `user.<key>` \
                          (`user.prg.<key>` in PRG) — several are a press cycle, each press the \
                          next. Unknown commands are refused; takes effect at once. A letter types \
                          in KBD: bind it with /Shortcuts \"PRG\" or \"NAV\". Hold Meow (Set \
                          UserProfile \"MeowKey\", default Shift) to run the key's Meow lines. A \
                          mode's binding wins over an every-mode one; yours over LumaCore's.",
                examples: &[
                    "Bind \"F5\" `List Fixture` /Description \"Quick list\"   -- F5 → user.f5",
                    "Bind \"F6\" \"blind\"                      -- F6 presses the Blind key (a UiKey)",
                    "Bind \"E\" `Edit` `EditSetting` /Shortcuts \"PRG\"   -- E cycles Edit › EditSetting in PRG",
                ],
                related: &["unbind", "list keys"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: KEYS,
            family: Some(CommandFamily::Settings),
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[crate::bind::SHORTCUTS, crate::bind::DESCRIPTION],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::bind::unbind(s, a)),
        decl: CommandDecl {
            verb: "unbind",
            doc: Doc {
                display: "Unbind",
                category: HelpCategory::System,
                syntax: "Unbind <key> [/Shortcuts KBD|PRG|NAV]",
                description: "Remove your binding of a key (LumaCore's comes back); with none of \
                          yours there, turn LumaCore's default off in that mode on your profile \
                          (Unbind again brings it back). Core keys stay.",
                examples: &[
                    "Unbind \"F5\"                     -- remove your F5",
                    "Unbind \"E\" /Shortcuts \"PRG\"     -- turn LumaCore's PRG E off on your profile",
                ],
                related: &["bind"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: KEYS,
            family: Some(CommandFamily::Settings),
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[crate::bind::SHORTCUTS],
            ..CommandDecl::BASE
        },
    },
];

/// The keys' objects with their behaviour; `register` adds them to the catalog.
pub static OBJECTS: &[&ObjectDef] = &[&crate::ui_key::UI_KEY, &crate::key_bindings::KEY_BINDINGS];

/// What the keys add to how a line runs: a show that replaces the last has its key lines
/// checked (decisions §17 "re-checked on load").
pub static HOOKS: &[Hook] = &[Hook::ShowReplaced(crate::lines::check_loaded)];
