// ██╗    ██╗██╗███╗   ██╗██████╗  ██████╗ ██╗    ██╗███████╗
// ██║    ██║██║████╗  ██║██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██║ █╗ ██║██║██╔██╗ ██║██║  ██║██║   ██║██║ █╗ ██║███████╗
// ██║███╗██║██║██║╚██╗██║██║  ██║██║   ██║██║███╗██║╚════██║
// ╚███╔███╔╝██║██║ ╚████║██████╔╝╚██████╔╝╚███╔███╔╝███████║
//  ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═════╝  ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: windows

//! The command rows of windows: ReloadUI, Screenshot (decisions §34 25.7, 25.13: each crate of 34g4
//! takes its file).
//!
//! Owns:          the windows family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use crate::commands::registry::*;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This family's commands with their handlers; `commands::register` adds them to the catalog.
pub(crate) static ROWS: &[CommandDef] = &[
    CommandDef {
        run: |s, a| out(windows::cmd_reload_ui(s, a)),
        decl: CommandDecl {
            verb: "reloadui",
            aliases: &["ru"],
            doc: Doc {
                display: "ReloadUI",
                category: HelpCategory::System,
                syntax: "ReloadUI [/Screen \"a,b\"] [/All]",
                description: "Refresh the user interface (ReloadUI). Reloads the screen the command \
                          came from — from a terminal-only session, all of your screens. \
                          /Screen picks screens, /All reloads every UI client.",
                examples: &[
                    "ReloadUI              -- reload this screen",
                    "ReloadUI /Screen 2    -- reload screen 2",
                    "ReloadUI /All         -- reload every connected UI",
                ],
                related: &["window", "view"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            rights: Rights::Guest,
            reach: Reach::Show,
            flags: &[flags::ALL, flags::SCREEN],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::commands::screenshot::cmd_screenshot(s, a)),
        decl: CommandDecl {
            verb: "screenshot",
            doc: Doc {
                display: "Screenshot",
                category: HelpCategory::System,
                syntax: "Screenshot [Screen <n>] [User <name>]",
                description: "Take screenshots (Print Screen / F11): every UI client showing your open \
                          screens on this station renders its screen as seen and uploads it, with a green \
                          flash. Files go to library/screenshots/<station>/YYYYMMDD_hhmmss_<user>_screen<n>.png; \
                          each saved file is a System Monitor line and a notice. `Screen <n>` takes one \
                          screen, `User <name>` another user's. F11 and Print Screen run it.",
                examples: &[
                    "Screenshot              -- all my screens",
                    "Screenshot Screen 2     -- my screen 2",
                    "Screenshot User \"LD\"   -- the LD's screens",
                ],
                related: &["reloadui", "screen"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Station,
            rights: Rights::Playback,
            reach: Reach::Drive,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::commands::attributes::ENCODER_BAR,
    &crate::commands::windows::WINDOW,
    &crate::commands::window_presets::WINDOW_PRESET,
    &crate::objects::system::WINDOW_TYPE,
    &crate::commands::views::VIEW,
    &crate::commands::views::VIEW_BUTTON,
    &crate::commands::screens::SCREEN,
    &crate::devices::SIDEBARS,
    &crate::devices::VIEW_BAR,
    &crate::devices::FULL_SCREEN,
    &crate::commands::menus::MENU,
    &crate::commands::menus::GRID_COLUMNS,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[
    Hook::Changed(crate::commands::views::follow_touched_profiles),
    Hook::SetUntracked(&crate::commands::menus::GRID_COLUMNS),
    Hook::DeleteNeverAsks(&crate::commands::views::VIEW_BUTTON),
    // A tap on a view button is the line of what it holds; Assign … At ViewButton takes any
    // object (decisions 0287).
    Hook::Rewrite(crate::commands::views::buttons::tap_line),
    Hook::Intercept(crate::commands::views::buttons::assign_intercept),
];
