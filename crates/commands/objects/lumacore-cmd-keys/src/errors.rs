// ███████╗██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// █████╗  ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗╚════██║
// ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║███████║
// ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: errors

//! The keys' error catalog (decisions §34 21): `register` adds it.
//!
//! Owns:          the keys' error codes E-KEY-001…005.
//! Doesn't own:   the catalog (lumacore-registry), how codes render (the kernel's errors).
//! Entry points:  ERRORS.
//! Decisions:     §17, §30 "12a3".

use lumacore_base::ErrorInfo;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The keys' error codes (decisions §17, §30 "12a3"); `register` adds them to the catalog.
pub const ERRORS: &[ErrorInfo] = &[
    ErrorInfo {
        code: "E-KEY-001",
        title: "Key command line doesn't parse",
        explain: "A key's command line starts with a word that is neither a command nor an object. Bind refuses it; on show load the key is flagged in the sysmon log (decisions §17).",
    },
    ErrorInfo {
        code: "E-KEY-002",
        title: "Unknown key name",
        explain: "Key names are cross-platform strings: F5, Ctrl-R, Alt-S, Ctrl-Shift-L.",
    },
    ErrorInfo {
        code: "E-KEY-003",
        title: "No such binding",
        explain: "Unbind removes your profile's own binding and LumaCore's comes back; with none of yours it turns LumaCore's default off in that mode (decisions 0295). A key nobody binds there has nothing to remove. `List KeyBindings` shows each binding's source.",
    },
    ErrorInfo {
        code: "E-KEY-004",
        title: "No such UI key",
        explain: "A binding or a UI names a UIKey id that doesn't exist (a plugin was removed, or a user key deleted). `List UIKey` shows the keys you can use.",
    },
    ErrorInfo {
        code: "E-KEY-005",
        title: "Core keys can't be bound",
        explain: "F9–F12, Pause and Print Screen are the core keys (DeskLock, normal theme, shortcut mode, view bar, screenshot, Shutdown menu, sidebars, full screen — decisions §30 \"12a3\"), with or without modifiers. F1–F8 are free for your own keys and executors.",
    },
    ErrorInfo {
        code: "E-KEY-006",
        title: "Letters type in KBD",
        explain: "In the Keyboard mode a letter, digit or sign always types (decisions 0295), so it can't be bound for every mode or for KBD. Bind it where single keys are words: `Bind \"E\" `Edit` /Shortcuts \"PRG\"` (or NAV). F keys and Ctrl / Alt combinations bind in every mode.",
    },
    ErrorInfo {
        code: "E-KEY-007",
        title: "Not a key's line",
        explain: "A UI key's lines (decisions 0295): Press takes one or more lines in backticks (more than one is a press cycle), Types the presses that type onto the command line (`1`, `1 Thru 2`, `\"1 3\"`, `None`), the other gestures one line each. A LumaCore or plugin key is locked: bind your own in its place (`Bind … /Shortcuts`).",
    },
];
