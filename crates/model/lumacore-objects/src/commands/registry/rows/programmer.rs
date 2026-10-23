// ██████╗ ██████╗  ██████╗  ██████╗ ██████╗  █████╗ ███╗   ███╗███╗   ███╗███████╗██████╗
// ██╔══██╗██╔══██╗██╔═══██╗██╔════╝ ██╔══██╗██╔══██╗████╗ ████║████╗ ████║██╔════╝██╔══██╗
// ██████╔╝██████╔╝██║   ██║██║  ███╗██████╔╝███████║██╔████╔██║██╔████╔██║█████╗  ██████╔╝
// ██╔═══╝ ██╔══██╗██║   ██║██║   ██║██╔══██╗██╔══██║██║╚██╔╝██║██║╚██╔╝██║██╔══╝  ██╔══██╗
// ██║     ██║  ██║╚██████╔╝╚██████╔╝██║  ██║██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║███████╗██║  ██║
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝
// FILE: programmer

//! The command rows of the programmer: At, Select, On / Off, Full / Zero, Grid, Default, Reset,
//! Flash (decisions §34 25.7, 25.13: each crate of 34g4 takes its file).
//!
//! Owns:          the programmer family's command rows (`ROWS`): each word's declaration and handler.
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
        run: |s, a| out(control::cmd_at(s, a)),
        decl: CommandDecl {
            verb: "at",
            aliases: &["a"],
            doc: Doc {
                display: "At",
                category: HelpCategory::Programmer,
                syntax: "At <value | Full | Zero>",
                description: "Set the dimmer value of the current fixture selection in the programmer. \
                          Accepts a percentage (0–100), the keyword Full (100%), or Zero (0%). \
                          Requires fixtures to be selected first.",
                examples: &[
                    "At 50      -- set dimmer to 50%",
                    "At Full    -- set dimmer to 100%",
                    "At Zero    -- set dimmer to 0%",
                ],
                related: &["fixture", "full", "zero", "attribute"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            args: Args::Value,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_full(s, a)),
        decl: CommandDecl {
            verb: "full",
            doc: Doc {
                display: "Full",
                category: HelpCategory::Programmer,
                syntax: "Full",
                description: "Set all selected fixtures to full dimmer (100%). \
                          Shorthand for `At 100`.",
                examples: &[
                    "Full              -- set selection to 100%",
                    "Fixture 1 Full    -- select fixture 1 and set to full",
                ],
                related: &["at", "zero", "on"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_zero(s, a)),
        decl: CommandDecl {
            verb: "zero",
            doc: Doc {
                display: "Zero",
                category: HelpCategory::Programmer,
                syntax: "Zero",
                description: "Set all selected fixtures to zero (0% dimmer). \
                          Shorthand for `At 0`.",
                examples: &[
                    "Zero              -- set selection to 0%",
                    "Fixture 1 Zero    -- select fixture 1 and set to zero",
                ],
                related: &["at", "full", "off"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::knock_in(s, a)),
        decl: CommandDecl {
            verb: "on",
            doc: Doc {
                display: "On",
                category: HelpCategory::Programmer,
                syntax: "On [<object>]",
                description: "Switch on. On a mode object it switches the mode on (`On Blind`). \
                          On fixtures it knocks in: activates the programmer values of the selection \
                          or of the fixtures named (`On Fixture 2`, `On Group 3`). Bare `On` acts on \
                          the selection.",
                examples: &[
                    "On                -- activate the selection's programmer values",
                    "On Fixture 2 + 4  -- activate those fixtures' values",
                    "On Blind          -- switch Blind on",
                ],
                related: &["off", "toggle", "blind", "highlight"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            object_verb: Some(Verb::On),
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::knock_out(s, a)),
        decl: CommandDecl {
            verb: "off",
            aliases: &["knockout"],
            doc: Doc {
                display: "Off",
                category: HelpCategory::Programmer,
                syntax: "Off [<object>]",
                description: "Switch off. On a mode object it switches the mode off (`Off Blind`). \
                          On fixtures it knocks out: removes their values from the programmer and \
                          deselects them (`Off Fixture 2`, `Off Group 3`, `Off Selection`). Bare `Off` \
                          acts on the selection.",
                examples: &[
                    "Off                -- knock the selection out of the programmer",
                    "Off Fixture 2 + 4  -- knock those fixtures out",
                    "Off Highlight      -- switch Highlight off",
                ],
                related: &["on", "toggle", "clear"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            object_verb: Some(Verb::Off),
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::commands::selections::cmd_select(s, a)),
        decl: CommandDecl {
            verb: "select",
            doc: Doc {
                display: "Select",
                category: HelpCategory::Programmer,
                syntax: "Select DataPool|Sequence|Timecode <n | \"name\"> | Select <tree path>",
                description: "Make an object your default of its kind (Select): the selected \
                          DataPool is where pool objects resolve, the selected Sequence is what a \
                          bare Go, Off Sequence, Goto Cue and Store Cue act on, and the selected \
                          Timecode. One of each per user, kept with the show. Fixtures are \
                          selected with `Fixture <n>` or a group, not with Select.",
                examples: &[
                    "Select Sequence 3                       -- bare Go now runs Sequence 3",
                    "Select DataPool 2                       -- Group 1 means datapool 2's",
                    "Select Root.ShowData.DataPools.2        -- the same, by tree path",
                ],
                related: &["deselect", "go", "datapool", "sequence"],
                ..Doc::BASE
            },
            undo: SELECTION,
            scope: Scope::Master,
            object_verb: Some(Verb::Select),
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| {
            if a.is_empty() {
                return DispatchResult::Output(crate::Reply::error(
                    "SelectFixtures needs what to select: SelectFixtures Group 3",
                    "E-SYN-002",
                ));
            }
            crate::objects::call(s, &crate::objects::FIXTURE, a)
        },
        decl: CommandDecl {
            verb: "selectfixtures",
            doc: Doc {
                display: "SelectFixtures",
                category: HelpCategory::Programmer,
                syntax: "SelectFixtures Fixture|Group|Preset <ids> | SelectFixtures <ids>",
                description: "Select fixtures (decisions §33 26): the default verb of \
                          Fixture, Group and Preset — `Group 3` is `SelectFixtures Group 3`. A plain \
                          select adds until a value is set, then replaces; `+` always adds. Bare ids \
                          are fixtures. Objects you make your selected one (DataPool, Sequence, \
                          Timecode, Window) use Select.",
                examples: &[
                    "SelectFixtures Group 3        -- the same as Group 3",
                    "SelectFixtures Fixture 1 Thru 4",
                ],
                related: &["select", "fixture", "group"],
                ..Doc::BASE
            },
            undo: SELECTION,
            family: Some(CommandFamily::Selection),
            scope: Scope::Master,
            object_verb: Some(Verb::SelectFixtures),
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::commands::selections::cmd_deselect(s, a)),
        decl: CommandDecl {
            verb: "deselect",
            doc: Doc {
                display: "Deselect",
                category: HelpCategory::Programmer,
                syntax: "Deselect [DataPool | Sequence | Timecode]",
                description: "Clear your selected DataPool, Sequence or Timecode (bare: all three). \
                          Without a selected DataPool pool objects resolve in the first one; without \
                          a selected Sequence a bare Go says so.",
                examples: &["Deselect Sequence", "Deselect"],
                related: &["select"],
                ..Doc::BASE
            },
            undo: SELECTION,
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| crate::commands::attributes::default_attributes(s, a),
        decl: CommandDecl {
            verb: "default",
            aliases: &["def"],
            doc: Doc {
                display: "Default",
                category: HelpCategory::Programmer,
                syntax: "Default Attribute /New | Default UserProfile Attribute /New",
                description: "Back to the standard (the Default keyword; decisions §30 \"12f-a\"): \
                          Default Attribute /New keeps where the resolver placed the attributes a patch \
                          brought (the first dialog's \"Follow standard attribute grouping\").",
                examples: &[
                    "Default Attribute /New               -- the show: keep the standard grouping",
                ],
                related: &["attribute", "featuregroup", "unclaim"],
                ..Doc::BASE
            },
            undo: tracked(OopsCategory::Patch),
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            args: Args::Words,
            rights: Rights::Setup,
            reach: Reach::Show,
            flags: crate::commands::attributes::ATTRIBUTE_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_please(s, a)),
        decl: CommandDecl {
            verb: "meow",
            doc: Doc {
                display: "Meow",
                category: HelpCategory::Programmer,
                syntax: "Meow",
                description: "Execute/confirm. With fixtures selected: toggles active values \
                          (knock-in/deactivate). With nothing selected: meow.",
                examples: &["Meow    -- activate/deactivate programmer values or meow"],
                related: &["reset", "resetactive"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_clear(s, a)),
        decl: CommandDecl {
            verb: "reset",
            doc: Doc {
                display: "Reset",
                category: HelpCategory::Programmer,
                syntax: "Reset | Reset Self | Reset Window [screen.]n | Reset UserProfile [<n>|\"name\"] Window /Type <type>",
                description: "Four-stage programmer reset (industry-standard order). \
                          1st: clears selection. \
                          2nd: deactivates values (still visible, won't store). \
                          3rd: empties programmer completely. \
                          4th: clears terminal buffer. \
                          On an object it puts its defaults back: `Reset Self` / `Reset Window 1.4` \
                          (every setting and the window bar; label, colour and position stay), \
                          `Reset UserProfile \"x\" Window /Type <type>` (every window of that type in \
                          the profile's screens and views; asks when more than one).",
                examples: &[
                    "Reset                -- step through reset stages",
                    "Reset Self           -- this window's settings and bar back to its type's defaults",
                    "Reset UserProfile \"Bob\" Window /Type DMX -- every DMX window of a profile (asks)",
                ],
                related: &["resetall", "resetselection", "resetactive"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            object_verb: Some(Verb::Reset),
            rights: Rights::Playback,
            reach: Reach::Show,
            flags: &[flags::TYPE],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_clear_all(s, a)),
        decl: CommandDecl {
            verb: "resetall",
            doc: Doc {
                display: "ResetAll",
                category: HelpCategory::Programmer,
                syntax: "ResetAll",
                description: "Reset programmer completely in one step. \
                          Clears selection, all values, and returns to Fixture context.",
                examples: &["ResetAll    -- reset everything immediately"],
                related: &["reset", "resetselection", "resetactive"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_reset_selection(s, a)),
        decl: CommandDecl {
            verb: "resetselection",
            doc: Doc {
                display: "ResetSelection",
                category: HelpCategory::Programmer,
                syntax: "ResetSelection",
                description: "Clear fixture selection only. Values are untouched.",
                examples: &["ResetSelection    -- deselect all fixtures"],
                related: &["reset", "resetactive"],
                ..Doc::BASE
            },
            undo: SELECTION,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_reset_active(s, a)),
        decl: CommandDecl {
            verb: "resetactive",
            doc: Doc {
                display: "ResetActive",
                category: HelpCategory::Programmer,
                syntax: "ResetActive",
                description: "Deactivate all active values. Values remain visible in the \
                          programmer but are dimmed and will not be included in Store.",
                examples: &["ResetActive    -- deactivate values without deleting them"],
                related: &["reset", "resetselection"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::commands::groups::cmd_plus(s, a)),
        decl: CommandDecl {
            verb: "plus",
            aliases: &["+"],
            doc: Doc {
                display: "Plus",
                category: HelpCategory::Programmer,
                syntax: "+ <selection>",
                description: "Add to the selection — always, even after a value was set \
                          (a plain `Group 3` then replaces, decisions §22). \
                          Fixtures and groups land in the grid at the grid cursor.",
                examples: &["+ Group 3", "+ Fixture 4 Thru 6", "+ 7"],
                related: &["minus", "group", "fixture", "grid"],
                ..Doc::BASE
            },
            undo: SELECTION,
            family: Some(CommandFamily::Selection),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::commands::groups::cmd_minus(s, a)),
        decl: CommandDecl {
            verb: "minus",
            aliases: &["-"],
            doc: Doc {
                display: "Minus",
                category: HelpCategory::Programmer,
                syntax: "- <selection>",
                description: "Take fixtures out of the selection (their programmer \
                          values stay; `Off` knocks them out).",
                examples: &["- Fixture 2", "- Group 4", "- 3 Thru 5"],
                related: &["plus", "group", "fixture"],
                ..Doc::BASE
            },
            undo: SELECTION,
            family: Some(CommandFamily::Selection),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::commands::groups::cmd_grid(s, a)),
        decl: CommandDecl {
            verb: "grid",
            doc: Doc {
                display: "Grid",
                category: HelpCategory::Programmer,
                syntax: "Grid [<x> [<y> [<z>]]] | Grid <x>/<y>[/<z>] | Grid [DataPool <d>] Group <n> \"<Tool>\" [\"<Arg>\"]",
                description: "Show or move the grid cursor: where the next selection \
                          lands in the selection grid (decisions §22). \
                          After a selection the cursor moves by the group's / your \
                          Move Grid Cursor (None, AppendX, NewLine). An empty \
                          selection puts it home (0/0). With a group, a tool on its \
                          grid (the Selection Grid tools, §30 \"12e\"): Linearize \
                          Number / LeftToRight / TopToBottom, RemoveGaps, RemoveOffset, \
                          Align Left / Center / Right, Transpose, Rotate Right / Left, \
                          Flip X / Y, Multiply n, Divide n — one Oops step each.",
                examples: &[
                    "Grid             -- where is the cursor",
                    "Grid 0/2",
                    "Grid 4 1",
                    "Grid Group 3 \"Linearize\" \"LeftToRight\"",
                    "Grid Group 3 \"RemoveGaps\"",
                    "Grid Group 3 \"Multiply\" 2",
                ],
                related: &["group", "plus", "fixture"],
                ..Doc::BASE
            },
            undo: SELECTION,
            family: Some(CommandFamily::Selection),
            scope: Scope::Master,
            args: Args::Value,
            rights: Rights::Playback,
            // A tool on a group changes the show: the group's own level.
            arg_rights: Some(|a| match a.first() {
                Some(w) if crate::objects::find(w).is_some() => Rights::Program,
                _ => Rights::Playback,
            }),
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_next(s, a)),
        decl: CommandDecl {
            verb: "next",
            aliases: &["n"],
            doc: Doc {
                display: "Next",
                category: HelpCategory::Programmer,
                syntax: "Next",
                description: "Walk forward through the current selection. \
                          With no selection, selects the first fixture. \
                          With a single fixture, jumps to the next by ID. \
                          With multiple fixtures, enters walk mode and steps through \
                          the selection order one fixture at a time. Wraps around.",
                examples: &[
                    "Next           -- advance to next fixture in selection",
                    "Fixture 1 Thru 10 ; Next  -- walk into selection",
                ],
                related: &["previous", "select", "fixture"],
                ..Doc::BASE
            },
            undo: SELECTION,
            family: Some(CommandFamily::Selection),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_previous(s, a)),
        decl: CommandDecl {
            verb: "previous",
            aliases: &["prev"],
            doc: Doc {
                display: "Previous",
                category: HelpCategory::Programmer,
                syntax: "Previous",
                description: "Walk backward through the current selection. \
                          With no selection, selects the last fixture. \
                          With a single fixture, jumps to the previous by ID. \
                          With multiple fixtures, enters walk mode and steps backward \
                          through the selection order. Wraps around.",
                examples: &[
                    "Previous       -- step to previous fixture in selection",
                    "Prev           -- shorthand",
                ],
                related: &["next", "select", "fixture"],
                ..Doc::BASE
            },
            undo: SELECTION,
            family: Some(CommandFamily::Selection),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "flash",
            hold: true,
            doc: Doc {
                display: "Flash",
                category: HelpCategory::Programmer,
                syntax: "Flash [Executor <id>]",
                description: "Momentarily activate an executor at full. Releases when Flash key is released.",
                examples: &["Flash Executor 1"],
                related: &["go", "executor"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "if",
            doc: Doc {
                display: "If",
                category: HelpCategory::Programmer,
                syntax: "<command> If <selection>",
                description: "Limit a command to a selection: `At 50 If Group 3`. Not built yet (planned).",
                examples: &["At 50 If Group 3"],
                related: &["at", "group"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Selection),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::commands::groups::GROUP,
    &crate::objects::poolxml::POOL_XML,
    &crate::objects::PRESET,
    &crate::commands::attributes::ATTRIBUTE,
    &crate::commands::attributes::FEATURE_GROUP,
    &crate::commands::attributes::ATTRIBUTE_RULE,
    &crate::commands::attributes::ATTRIBUTE_TEMPLATE,
    &crate::objects::system::PROGRAMMER,
    &crate::objects::system::SELECTION,
    &crate::objects::modes::BLIND,
    &crate::objects::modes::HIGHLIGHT,
    &crate::objects::modes::FREEZE,
    &crate::objects::modes::SOLO,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[
    Hook::PatchKept(crate::commands::attributes::after_patch_kept),
    // `Call Preset n` is `At Preset n` (decisions 0288, §33).
    Hook::Rewrite(crate::objects::preset::apply::call_is_at),
];
