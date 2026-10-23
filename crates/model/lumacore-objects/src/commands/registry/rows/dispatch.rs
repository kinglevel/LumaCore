// ██████╗ ██╗███████╗██████╗  █████╗ ████████╗ ██████╗██╗  ██╗
// ██╔══██╗██║██╔════╝██╔══██╗██╔══██╗╚══██╔══╝██╔════╝██║  ██║
// ██║  ██║██║███████╗██████╔╝███████║   ██║   ██║     ███████║
// ██║  ██║██║╚════██║██╔═══╝ ██╔══██║   ██║   ██║     ██╔══██║
// ██████╔╝██║███████║██║     ██║  ██║   ██║   ╚██████╗██║  ██║
// ╚═════╝ ╚═╝╚══════╝╚═╝     ╚═╝  ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝
// FILE: dispatch

//! The command rows of the verbs every object shares: Set, Label, Store, the door, Edit, Oops,
//! Claim, Clear, Sort / Split, Enter / Exit (decisions §34 25.7, 25.13: the kernel's; the reading
//! verbs are `inspection`).
//!
//! Owns:          the dispatch family's command rows (`ROWS`): each word's declaration and handler.
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
        run: |_, _| {
            out(crate::errors::render_code(
                "Toggle what? e.g. `Toggle Blind`.".to_string(),
                "E-CMD-004",
            ))
        },
        decl: CommandDecl {
            verb: "toggle",
            doc: Doc {
                display: "Toggle",
                category: HelpCategory::Programmer,
                syntax: "Toggle <object>",
                description: "On or Off, depending on the object's state: `Toggle Freeze`. \
                          The bare mode word does the same (`Freeze`).",
                examples: &["Toggle Blind   -- Blind on if off, off if on"],
                related: &["on", "off"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            object_verb: Some(Verb::Toggle),
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| {
            DispatchResult::Output(crate::Reply::error(
                "Call needs what to bring back: Call View 3",
                "E-SYN-002",
            ))
        },
        decl: CommandDecl {
            verb: "call",
            doc: Doc {
                display: "Call",
                category: HelpCategory::System,
                syntax: "Call View|ViewButton|WindowPreset <ids>",
                description: "Bring a stored thing back here (decisions §33 35): a view onto \
                          your screen, a window preset into its window. It is the default verb of \
                          View, ViewButton and WindowPreset — `View 3` is `Call View 3`.",
                examples: &["Call View 3", "Call WindowPreset 2"],
                related: &["view", "viewbutton", "windowpreset"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Master,
            object_verb: Some(Verb::Call),
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::store_default,
        decl: CommandDecl {
            verb: "store",
            aliases: &["s"],
            doc: Doc {
                display: "Store",
                category: HelpCategory::Group,
                syntax: "Store [Cue] [<n>] [/Merge|/Overwrite|/Remove|/CreateSecondCue] | Store Group <id> [\"name\"] | Store Sequence [<n>] | Store Universe <id> | Store Preset ...",
                description: "Store the current programmer state. Cue is Store's default object: \
                          `Store` alone stores the next cue of your selected sequence, `Store 2` is \
                          `Store Cue 2`; over an existing cue it asks Merge / Overwrite / Remove / \
                          Create Second Cue. \
                          `Store Group` creates or updates a fixture group. \
                          `Store Universe` (requires the Devices claim) creates a DMX universe. \
                          `Store Preset` stores attribute values as a preset (planned).",
                examples: &[
                    "Store                    -- the next cue of the selected sequence",
                    "Store 2 /Merge           -- Store Cue 2, merged into it",
                    "Store Group 1            -- store current selection as group 1",
                    "Store Group 2 \"Wash\"      -- store with name",
                    "Store Universe 1         -- create universe 1 (requires claim)",
                ],
                related: &["delete", "copy", "group"],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Store),
            rights: Rights::Program,
            reach: Reach::Show,
            // `Store 2 /Merge` (Cue is Store's default object): the store ask's answers.
            flags: flags::STORE_CUE,
            default_object: Some("Cue"),
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::delete,
        decl: CommandDecl {
            verb: "delete",
            aliases: &["d"],
            doc: Doc {
                display: "Delete",
                category: HelpCategory::Group,
                syntax: "Delete <object> <id> [/NoConfirmation]",
                description: "Remove an object. The station asks first; /NoConfirmation (/nc) \
                          skips the question. Group deletion does not require the Devices claim. \
                          Fixture and universe deletion requires the Devices claim.",
                examples: &[
                    "Delete Group 1      -- delete group 1",
                    "Delete Fixture 5    -- delete fixture 5 (needs claim)",
                    "Delete Universe 1   -- delete universe 1 (needs claim)",
                ],
                related: &["store", "unpatch"],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Delete),
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| run::objects_usage(Verb::Copy),
        decl: CommandDecl {
            verb: "copy",
            aliases: &["co"],
            doc: Doc {
                display: "Copy",
                category: HelpCategory::Group,
                syntax: "Copy Group <src> [At] <dst>",
                description: "Duplicate a group to a new ID. \
                          If the destination ID already exists, it is replaced.",
                examples: &["Copy Group 1 At 2    -- copy group 1 to group 2"],
                related: &["store", "update"],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Copy),
            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[flags::MERGE, flags::OVERWRITE],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::door::CutBare>,
        decl: CommandDecl {
            verb: "cut",
            doc: Doc {
                display: "Cut",
                category: HelpCategory::Group,
                syntax: "Cut <object> <ids>",
                description: "Put objects on your clipboard to be moved: `Paste` puts them where its target \
                          says, the same objects (GUIDs kept). The clipboard holds the objects themselves, \
                          so Paste takes them as they are then (decisions §35 9, 14).",
                examples: &["Cut Group 1", "Cut Group 1 Thru 4"],
                related: &["paste", "copy", "move"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Cut),
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::door::PasteBare>,
        decl: CommandDecl {
            verb: "paste",
            doc: Doc {
                display: "Paste",
                category: HelpCategory::Group,
                syntax: "Paste [<object> <id>] [/Merge | /Overwrite]",
                description: "Finish a bare `Copy` (new objects) or a `Cut` (the same objects moved): the \
                          target is where they go, their spacing kept, onto a taken number the Copy asks \
                          and the Cut swaps. `Paste 10` takes your default object. Bare, a copy goes to the \
                          first free numbers and a cut asks where (decisions §35 13).",
                examples: &["Paste Group 10", "Paste DataPool 2 Group 10", "Paste"],
                related: &["copy", "cut", "move"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Paste),
            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[flags::MERGE, flags::OVERWRITE],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::door::InsertBare>,
        decl: CommandDecl {
            verb: "insert",
            doc: Doc {
                display: "Insert",
                category: HelpCategory::Group,
                syntax: "Insert <object> <ids> At <n>",
                description: "A move with shift: the objects land at `At` and the ones in the way move up \
                          to the first free number, in order; GUIDs kept (decisions §35 15). `Move` swaps \
                          instead.",
                examples: &["Insert Group 5 At 9"],
                related: &["move", "compact"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Insert),
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::door::CompactBare>,
        decl: CommandDecl {
            verb: "compact",
            doc: Doc {
                display: "Compact",
                category: HelpCategory::Group,
                syntax: "Compact <object> [<ids>] [/By Name | <order>]",
                description: "Renumber a pool one after another, holes closed: the whole pool from 1, or the \
                          rows named from their first; `/By Name` (or an order the object declares) instead \
                          of the numbers' order. GUIDs kept, one Oops step (decisions §35 16).",
                examples: &["Compact Group", "Compact Group 10 Thru 50", "Compact Group /By Name"],
                related: &["move", "insert", "sort"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Compact),
            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[flags::BY],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| run::objects_usage(Verb::Move),
        decl: CommandDecl {
            verb: "move",
            doc: Doc {
                display: "Move",
                category: HelpCategory::Fixture,
                also_applies_to: &[HelpCategory::Group, HelpCategory::Universe],
                syntax: "Move <type> <id> [Thru <end>] At <new_id>",
                description: "Renumber objects by changing their user-facing ID. \
                          Internal cross-references (groups, programmer, presets) \
                          stay stable via GUID — only the integer ID changes.",
                examples: &[
                    "Move Fixture 1 At 10         -- renumber fixture 1 to 10",
                    "Move Fixture 1 Thru 4 At 10  -- renumber fixtures 1-4 to 10-13",
                    "Move Group 1 At 5            -- renumber group 1 to 5",
                ],
                related: &["store", "copy", "delete"],
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            scope: Scope::Master,
            object_verb: Some(Verb::Move),
            // Renumbering (decisions §34 round 3 19): `/Skip` jumps over taken numbers, `/By
            // <column>` numbers them in a column's order.
            flags: &[flags::SKIP, flags::BY],
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| run::objects_usage(Verb::Update),
        decl: CommandDecl {
            verb: "update",
            aliases: &["u"],
            doc: Doc {
                display: "Update",
                category: HelpCategory::Group,
                syntax: "Update Group <id>",
                description: "Merge the current programmer selection into an existing group. \
                          Existing group members are retained; new fixtures are added.",
                examples: &["Update Group 1    -- add current selection to group 1"],
                related: &["store", "copy"],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: STORE,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            object_verb: Some(Verb::Update),
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| run::objects_usage(Verb::Edit),
        decl: CommandDecl {
            verb: "edit",
            aliases: &["e"],
            doc: Doc {
                display: "Edit",
                category: HelpCategory::Group,
                syntax: "Edit Group <id> | Edit Fixture <id>",
                description: "Enter edit mode for a specific object. \
                          Shows an inline property sheet and enables sub-commands (Name, Label, Add, Remove). \
                          Exit edit mode with `Exit`.",
                examples: &[
                    "Edit Group 1      -- edit group 1",
                    "Edit Fixture 3    -- edit fixture 3",
                ],
                related: &["store", "delete"],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: EDIT,
            family: Some(CommandFamily::Edit),
            scope: Scope::Master,
            object_verb: Some(Verb::Edit),
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(oops::cmd_oops(s, a)),
        decl: CommandDecl {
            verb: "oops",
            aliases: &["hiss", "unmeow"],
            doc: Doc {
                display: "Oops",
                category: HelpCategory::Undo,
                syntax: "Oops [<n>] [/NoConfirmation]",
                description: "Undo your newest step (or the newest n). Each user has one Oops list, \
                          newest first, of every kind of step: programmer, selection, show, patch, \
                          UI (windows, views, screens, watches), users, settings, keys and station \
                          settings (put back on the station where they changed). The category is a \
                          label: `List Undo /<category>`. Running playback and the Blind, Highlight, \
                          Freeze and Solo modes are never undone. Loading a show clears the list. \
                          Your profile's OopsConfirmation (Always / Main / Never) asks first; \
                          `/NoConfirmation` skips it. `/NoOops` on any command files no step; \
                          `Set UserProfile \"CreateOops\" No` (or \"OopsViews\", \"OopsProgrammer\", \"OopsSelection\") stops filing \
                          those steps.",
                examples: &[
                    "Oops      -- undo your newest step",
                    "Oops 3    -- undo the newest 3 steps",
                    "List Undo /Patch -- the patch steps in your list",
                    "Store Group 5 /NoOops -- no Oops step for this",
                ],
                related: &["redo"],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<oops::Redo>,
        decl: CommandDecl {
            verb: "redo",
            doc: Doc {
                display: "Redo",
                category: HelpCategory::Undo,
                syntax: "Redo [<n>]",
                description: "Redo the last undone step, or the newest n (after Oops; a new step \
                          clears what can be redone).",
                examples: &["Redo      -- redo last undone command", "Redo 3    -- the three newest undone steps"],
                related: &["oops"],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::StoreUpdate),
            scope: Scope::Master,
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::attributes::Sort>,
        decl: CommandDecl {
            verb: "sort",
            doc: Doc {
                display: "Sort",
                category: HelpCategory::System,
                syntax: "Sort <object> /By <key>",
                description: "Reorder an object's members once, by a key the object declares (decisions \
                          §33 88): a feature group's attributes by Name, Family or MainFirst. A one-shot \
                          change, one Oops step.",
                examples: &["Sort FeatureGroup \"Beam\" /By Family", "Sort UserProfile FeatureGroup \"Position\" /By MainFirst"],
                related: &["split", "assign"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: PATCH,
            family: Some(CommandFamily::Edit),
            scope: Scope::Master,
            // The object's level decides (a show feature group: Setup — decisions §30 "12f-a").
            object_verb: Some(Verb::Sort),

            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[flags::BY],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::attributes::Split>,
        decl: CommandDecl {
            verb: "split",
            doc: Doc {
                display: "Split",
                category: HelpCategory::System,
                syntax: "Split <object> /By <key>",
                description: "Split an object's members into groups of their own, by a key the object \
                          declares (decisions §33 88): a feature group into one per attribute Family. \
                          Asks first; one Oops step.",
                examples: &["Split FeatureGroup \"Effects\" /By Family"],
                related: &["sort", "assign"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: PATCH,
            family: Some(CommandFamily::Edit),
            scope: Scope::Master,
            object_verb: Some(Verb::Split),

            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[flags::BY],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::claim,
        decl: CommandDecl {
            verb: "claim",
            doc: Doc {
                display: "Claim",
                category: HelpCategory::System,
                syntax: "Claim [<object> ...] [/Force | /Keep [/Remember]]",
                description: "Claim an object for exclusive editing. \
                          Bare `claim` claims the current scoped object when that scope is claimable. \
                          `claim Root.ShowData.Devices` claims the Patch for editing fixtures and universes. \
                          Multiple object handles may be claimed in one sentence. \
                          An object another user holds asks them (10 s): `Claim <object> /Keep` is the owner's No, \
                          with `/Remember` that user's later claims are refused at once until you log out. \
                          An Admin's `/Force` takes it without asking.",
                examples: &[
                    "claim          -- claim the current scoped object",
                    "claim Root.ShowData.Devices    -- claim Root.ShowData.Devices for editing",
                    "claim Timecode 1 Group 1    -- claim multiple objects",
                ],
                related: &["unclaim", "claims", "patch"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::System),
            scope: Scope::Master,
            object_verb: Some(Verb::Claim),
            rights: Rights::Program,
            arg_rights: Some(claim_rights),
            reach: Reach::Show,
            flags: &[flags::FORCE, flags::KEEP, flags::REMEMBER],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::unclaim,
        decl: CommandDecl {
            verb: "unclaim",
            doc: Doc {
                display: "Unclaim",
                category: HelpCategory::System,
                syntax: "Unclaim [<object>] [/Keep | /Discard] [/Remember]",
                description: "Release claimed objects. Bare `unclaim` releases all normal claims held by the active user. `Programmer` remains permanently claimed. Admins can unclaim any user's specific object. Claiming Patch edits a draft (the rig keeps the live patch); leaving it with changes asks Keep / Discard / Cancel — `/Keep` or `/Discard` answer without asking. Unclaiming an object someone asked for hands it to them; with `/Remember` that user's later claims of your objects are handed over at once until you log out.",
                examples: &[
                    "unclaim          -- release all of your normal claims",
                    "unclaim Patch    -- release Patch claim",
                ],
                related: &["claim"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::System),
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[flags::KEEP, DISCARD, DISCARD_CHANGES, PATCH_DEFAULT, PATCH_EDIT, PATCH_MINE, flags::REMEMBER],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::set::route,
        decl: CommandDecl {
            verb: "set",
            doc: Doc {
                display: "Set",
                category: HelpCategory::System,
                syntax: "Set <setting> <value> | Set <object> <id> <property> <value> | Set <tree path> <property> <value>",
                description: "Modify a user profile setting. \
                          `set clearbuffer on/off` — clear screen before each output. \
                          `set descriptions on/off` — show/hide command descriptions in help. \
                          `set doubleclicktime <ms>` / `set holdtime <ms>` — UI key double-press \
                          window and hold threshold for this profile (300 / 500 ms). \
                          `set notifications <category> <level> none|card|popup|default` and \
                          `set notificationmode timed|permanent` — how messages reach this profile \
                          (important and critical events always reach everyone). \
                          `set createoops|oopsviews|oopsprogrammer|oopsselection yes|no` and \
                          `set oopsconfirmation always|main|never` — this profile's Oops settings \
                          `set suggestions auto|tabonly|off` — when the command line's suggestion \
                          list opens: after a short pause while typing, only on Tab, or never. \
                          `set <object> <id> <property> <value>` (or a tree path) changes any \
                          property the object declares writable — `set fixture 3` lists them; \
                          others are read-only and Set says why.",
                examples: &[
                    "set clearbuffer on     -- clear screen each command",
                    "set descriptions off   -- compact help mode",
                    "set doubleclicktime 400 -- slower double-click on UI keys",
                    "set notifications network warn popup -- network warnings as pop-ups",
                    "set oopsconfirmation main -- ask before oopsing a store, move or delete",
                    "set suggestions tabonly -- the suggestion list opens only on Tab",
                    "set fixture 3 name \"Spot SR\" -- rename a fixture",
                    "set output 1 priority 120 -- sACN priority of output 1",
                ],
                related: &["list settings"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: USER_SETTINGS,
            family: Some(CommandFamily::Settings),
            scope: Scope::Master,
            object_verb: Some(Verb::Set),
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::clear_terminal,
        decl: CommandDecl {
            verb: "clear",
            aliases: &["cls", "clearterm"],
            doc: Doc {
                display: "Clear",
                category: HelpCategory::System,
                syntax: "Clear | Clear Programmer User <n> [Thru <m>]",
                description: "Clear the terminal output buffer. With `Programmer User …` an Admin clears \
                          other users' programmers (they are told; Oops undoes it).",
                examples: &[
                    "clear    -- wipe the terminal output area",
                    "cls      -- alias for clear",
                    "Clear Programmer User 2 Thru 5",
                ],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            scope: Scope::Master,
            rights: Rights::Guest,
            arg_rights: Some(clear_rights),
            class: CommandClass::Session,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::label::Label>,
        decl: CommandDecl {
            verb: "label",
            aliases: &["name", "rename"],
            doc: Doc {
                display: "Label",
                category: HelpCategory::Group,
                also_applies_to: &[HelpCategory::Fixture, HelpCategory::Universe],
                syntax: "Label <target> [\"name\"] [/Clear]",
                description: "Name objects: every object with a \"Name\" property. Several rows number on from                       the name (a trailing number counts up, keeping the spacing); an empty name or                       /Clear clears it; one row without a name asks.",
                examples: &[
                    "Label Fixture 1 \"Spot\"",
                    "Label Universe 1 Thru 5 \"Uni 1\"",
                    "Label Group 3 \"\"",
                ],
                alias_depth: AliasDepth::ProgLang,
                ..Doc::BASE
            },
            undo: STORE,
            family: Some(CommandFamily::Edit),
            scope: Scope::Master,
            object_verb: Some(Verb::Label),
            rights: Rights::Program,
            reach: Reach::Show,
            flags: &[flags::CLEAR],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::edit_only,
        decl: CommandDecl {
            verb: "add",
            doc: Doc {
                display: "",
                category: HelpCategory::Group,
                syntax: "Add <ids>",
                description: "Add fixtures to group (edit mode).",
                alias_depth: AliasDepth::ProgLang,
                hidden: true,
                ..Doc::BASE
            },
            undo: PROGRAMMER_SHOWDATA,
            family: Some(CommandFamily::Edit),
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::edit_only,
        decl: CommandDecl {
            verb: "remove",
            doc: Doc {
                display: "",
                category: HelpCategory::Group,
                syntax: "Remove <ids>",
                description: "Remove fixtures from group (edit mode).",
                alias_depth: AliasDepth::ProgLang,
                hidden: true,
                ..Doc::BASE
            },
            undo: main(OopsCategory::Show),
            family: Some(CommandFamily::Edit),
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    // Assign / Unassign are every object's (a universe on a line, an attribute to a feature group).
    CommandDef {
        run: |_, _| run::objects_usage(Verb::Assign),
        decl: CommandDecl {
            verb: "assign",
            doc: Doc {
                display: "Assign",
                category: HelpCategory::System,
                syntax: "Assign Universe <n> [Thru <m>] At Output|Input <line> [/Universe <u>]",
                description: "Route universes to (or from) a line as network universes u, u+1, … \
                          (decisions §33 22; no /Universe = from the first universe's own number). \
                          Art-Net takes a Port-Address or Net.SubNet.Universe. Nothing is sent until the \
                          line's network is mapped on the station. Live Patch: no claim.",
                examples: &[
                    "Assign Universe 1 Thru 6 At Output 1 /Universe 101",
                    "Assign Universe 2 At Output 3 /Universe 0.1.2",
                    "Assign Universe 5 At Input 1 /Universe 7",
                ],
                related: &["unassign", "output", "network", "universe"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: PATCH,
            family: Some(CommandFamily::Assignment),
            scope: Scope::Master,
            object_verb: Some(Verb::Assign),
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| run::objects_usage(Verb::Unassign),
        decl: CommandDecl {
            verb: "unassign",
            doc: Doc {
                display: "Unassign",
                category: HelpCategory::System,
                syntax: "Unassign Universe <n> [Thru <m>] [At Output <line>]",
                description: "Remove universes' routes (to one output line, or all).",
                examples: &[
                    "Unassign Universe 1 At Output 2",
                    "Unassign Universe 1 Thru 6",
                ],
                related: &["assign", "output"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: PATCH,
            family: Some(CommandFamily::Assignment),
            scope: Scope::Master,
            object_verb: Some(Verb::Unassign),
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::self_ref::SELF,
    &crate::objects::system::SHOW_DATA,
    &crate::objects::system::REFERENCE,
    &crate::objects::system::CLAIMS,
    &crate::objects::system::UNDO,
    &crate::objects::system::ERROR,
    &crate::objects::system::GLOBAL_VAR,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[];
