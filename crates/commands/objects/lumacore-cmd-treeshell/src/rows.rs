// ██████╗  ██████╗ ██╗    ██╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: rows

//! The command rows of the tree shell: cd, ls, pwd, cat, tree, find, grep, cp, mv, rm, …, Enter and
//! Exit (decisions §34 25.7, 25.13), and its hook (`mv` / `cp` / `rm` written out as Lum).
//!
//! Owns:          the tree shell's command rows (`ROWS`): each word's declaration and handler; HOOKS.
//! Doesn't own:   the handlers' work (words.rs, door.rs), the catalog's checks (the kernel's registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use lumacore_objects::commands::registry::*;
use lumacore_objects::objects::{ObjectDef, out};

use crate::words::TREE_DEPTH;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The tree shell's commands with their handlers; `register` adds them to the catalog.
pub static ROWS: &[CommandDef] = &[
    CommandDef {
        run: crate::words::exit,
        decl: CommandDecl {
            verb: "exit",
            aliases: &["q"],
            doc: Doc {
                display: "Exit",
                category: HelpCategory::ProgrammerNav,
                syntax: "Exit",
                description: "Release the claim on the current node and reset scope to default. \
                          Does NOT change your tree position (use `cd ..` for that). \
                          Use this when you're done editing in a claimed scope like Patch.",
                examples: &["exit    -- release Patch claim (stay at /ShowData/Patch)"],
                related: &["enter", "cd", "shutdown"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(lumacore_objects::commands::navigate::cmd_enter(s, a)),
        decl: CommandDecl {
            verb: "enter",
            doc: Doc {
                display: "Enter",
                category: HelpCategory::ProgrammerNav,
                syntax: "Enter [<path> | .. | /]",
                description: "Navigate the object tree, change scope, and auto-claim. \
                          Works like Linux `cd` — changes your position in the tree. \
                          The prompt scope updates to show where you are (e.g. Patch>, Library>). \
                          Claimable nodes (Patch, ShowData, etc.) are automatically claimed on entry. \
                          `enter ..` goes up one level and unclaims. `enter /` goes to root. \
                          Supports dot-paths: `enter showdata.patch` or `enter root.showdata.patch`.",
                examples: &[
                    "enter              -- show current path and scope",
                    "enter patch        -- navigate into Patch (relative)",
                    "enter showdata.patch  -- navigate to Patch via dot-path",
                    "enter library      -- navigate to Library",
                    "enter ..           -- go up one level",
                    "enter /            -- go to root",
                ],
                related: &["exit", "cd", "ls", "pwd"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Navigation),
            scope: Scope::Client,
            args: Args::TreeNav { up: false },
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Cd>,
        decl: CommandDecl {
            verb: "cd",
            fence: true,
            doc: Doc {
                display: "cd",
                category: HelpCategory::TreeNav,
                syntax: "cd <target | .. | />",
                description: "Navigate the object tree without claiming or changing scope. \
                          Use `enter` to navigate + claim + set context. \
                          Supports dot-paths and relative names.",
                examples: &[
                    "cd showdata    -- navigate to ShowData (no claim)",
                    "cd ..          -- go up one level",
                    "cd /           -- go to root",
                ],
                related: &["enter", "exit", "pwd"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Navigation),
            scope: Scope::Client,
            args: Args::TreeNav { up: true },
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::TreeShell,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Pwd>,
        decl: CommandDecl {
            verb: "pwd",
            fence: true,
            doc: Doc {
                display: "pwd",
                category: HelpCategory::TreeNav,
                syntax: "pwd",
                description: "Print your current tree position.",
                examples: &["pwd    -- show current path (e.g. /ShowData/Patch)"],
                related: &["enter", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Navigation),
            scope: Scope::Client,
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::TreeShell,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Ls>,
        decl: CommandDecl {
            verb: "ls",
            fence: true,
            doc: Doc {
                display: "ls",
                category: HelpCategory::TreeNav,
                syntax: "ls [<path>] [-r]",
                description: "List children of the current position in the object tree. \
                          Use `enter` to navigate into a node, `ls` to see what is here. \
                          With a dot-path argument, lists children of that path without navigating.",
                examples: &[
                    "ls              -- list children at current path",
                    "ls showdata     -- list children of ShowData",
                    "ls showdata.patch  -- list children of Patch",
                ],
                related: &["enter", "pwd", "list"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Navigation),
            scope: Scope::Client,
            args: Args::TreeFromRoot,
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::TreeShell,
            flags: &[lumacore_objects::commands::navigate::LS_RECURSIVE],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Cat>,
        decl: CommandDecl {
            verb: "cat",
            fence: true,
            doc: Doc {
                display: "cat",
                category: HelpCategory::TreeNav,
                also_applies_to: &[HelpCategory::Inspection],
                syntax: "cat [<path>]",
                description: "Display the contents of a node as pretty-printed JSON. \
                          Without a path, shows the current node. \
                          Works on object nodes (fixtures, universes, etc.) and structural directories.",
                examples: &[
                    "cat                    -- show current node",
                    "cat ShowData.UserProfiles.1  -- show Admin profile",
                ],
                related: &["stat", "head", "tail", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Head>,
        decl: CommandDecl {
            verb: "head",
            fence: true,
            doc: Doc {
                display: "head",
                category: HelpCategory::TreeNav,
                syntax: "head [<path>] [-n N]",
                description: "Show the first N entries of a collection (default 10). \
                          For objects, shows the first N fields.",
                examples: &[
                    "head ShowData.Devices.Dmx.Patch.Fixtures  -- first 10 fixtures",
                    "head -n 5 ShowData.UserProfiles       -- first 5 user profiles",
                ],
                related: &["tail", "cat", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            flags: &[crate::words::LINES],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Tail>,
        decl: CommandDecl {
            verb: "tail",
            fence: true,
            doc: Doc {
                display: "tail",
                category: HelpCategory::TreeNav,
                syntax: "tail [<path>] [-n N]",
                description: "Show the last N entries of a collection (default 10). \
                          For objects, shows the last N fields.",
                examples: &[
                    "tail ShowData.Devices.Dmx.Patch.Fixtures  -- last 10 fixtures",
                    "tail -n 3 ShowData.UserProfiles       -- last 3 user profiles",
                ],
                related: &["head", "cat", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            flags: &[crate::words::LINES],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Stat>,
        decl: CommandDecl {
            verb: "stat",
            fence: true,
            doc: Doc {
                display: "stat",
                category: HelpCategory::TreeNav,
                also_applies_to: &[HelpCategory::Inspection],
                syntax: "stat [<path>]",
                description: "Show metadata property sheet for a node: \
                          Type, ID, GUID, Name, Path, Children count, Size.",
                examples: &[
                    "stat                   -- current node metadata",
                    "stat ShowData.UserProfiles.1  -- metadata of Admin profile",
                ],
                related: &["cat", "info", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Tree>,
        decl: CommandDecl {
            verb: "tree",
            fence: true,
            doc: Doc {
                display: "tree",
                category: HelpCategory::TreeNav,
                syntax: "tree [<path>] [-d N]",
                description: "Recursive tree listing with object counts at leaves. \
                          Always recurses (unlike ls, which needs -r). \
                          tree -d N limits the depth.",
                examples: &[
                    "tree                   -- full tree from cwd",
                    "tree ShowData -d 2     -- 2 levels deep",
                    "tree ShowData.Devices.Dmx.Patch  -- patch subtree",
                ],
                related: &["ls", "du", "find"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            flags: &[TREE_DEPTH],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Find>,
        decl: CommandDecl {
            verb: "find",
            fence: true,
            doc: Doc {
                display: "find",
                category: HelpCategory::TreeNav,
                also_applies_to: &[HelpCategory::Inspection],
                syntax: "find <pattern> [--type TYPE]",
                description: "Search node names by substring or glob pattern (case-insensitive). \
                          Walks the full tree from the current position. \
                          Use --type to filter by object type (Fixture, Universe, etc.).",
                examples: &[
                    "find \"Admin\"           -- nodes with 'Admin' in name",
                    "find \"Main*\"           -- nodes matching glob",
                ],
                related: &["grep", "tree", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 2,
                options: &["--type"],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            flags: &[crate::words::FIND_TYPE],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Grep>,
        decl: CommandDecl {
            verb: "grep",
            fence: true,
            doc: Doc {
                display: "grep",
                category: HelpCategory::TreeNav,
                also_applies_to: &[HelpCategory::Inspection],
                syntax: "grep <pattern> [<path>]",
                description: "Search object values (not just names). \
                          Serializes each object to JSON and searches within all fields. \
                          Useful for finding which output uses an IP, which fixture has a setting, etc.",
                examples: &[
                    "grep \"192.168\"         -- find objects with this IP",
                    "grep \"Admin\" ShowData.UserProfiles  -- profiles with 'Admin' in any field",
                ],
                related: &["find", "cat"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 2,
                options: &["--type"],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Du>,
        decl: CommandDecl {
            verb: "du",
            fence: true,
            doc: Doc {
                display: "du",
                category: HelpCategory::TreeNav,
                syntax: "du [<path>]",
                description: "Recursive object count (\"disk usage\"). \
                          Shows how many objects of each type exist below the given path.",
                examples: &[
                    "du                     -- count all objects from cwd",
                    "du ShowData            -- count across all show data",
                ],
                related: &["wc", "tree", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Wc>,
        decl: CommandDecl {
            verb: "wc",
            fence: true,
            doc: Doc {
                display: "wc",
                category: HelpCategory::TreeNav,
                syntax: "wc [<path>]",
                description: "Count direct children of a node (non-recursive). \
                          Includes both structural subdirectories and dynamic objects.",
                examples: &[
                    "wc                     -- children at cwd",
                    "wc Fixtures            -- how many fixtures",
                ],
                related: &["du", "ls"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::door::Mv>,
        decl: CommandDecl {
            verb: "mv",
            fence: true,
            doc: Doc {
                display: "mv",
                category: HelpCategory::TreeNav,
                syntax: "mv <path> <to> [--overwrite | --keep]",
                description: "Move: on the show tree the Lum line it means (`mv 3 7` in a DataPool's Groups = `Move DataPool 1 Group 3 At 7`: swaps onto a taken number); under Root.Drives a file or folder of this station (a taken name asks Overwrite / Keep both). Between the two: Import / Export (decisions §35 19).",
                examples: &[
                    "mv 3 7",
                    "mv Root.ShowData.DataPools.1.Groups.3 Root.ShowData.DataPools.2.Groups.8",
                ],
                related: &["move", "copy", "delete"],
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            scope: Scope::Station,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Setup,
            reach: Reach::TreeShell,
            flags: &[crate::door::OVERWRITE, crate::door::KEEP],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::door::Cp>,
        decl: CommandDecl {
            verb: "cp",
            fence: true,
            doc: Doc {
                display: "cp",
                category: HelpCategory::TreeNav,
                syntax: "cp <path> <to> [--merge | --overwrite | --keep]",
                description: "Copy: on the show tree the Lum `Copy` it means (new GUIDs; onto one that exists it asks); under Root.Drives a file of this station (decisions §35 19).",
                examples: &["cp 3 10", "cp DataPools.1.Groups.3 DataPools.2.Groups.8"],
                related: &["move", "copy", "delete"],
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            scope: Scope::Station,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Setup,
            reach: Reach::TreeShell,
            flags: &[
                crate::door::MERGE,
                crate::door::OVERWRITE,
                crate::door::KEEP,
            ],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::door::Rm>,
        decl: CommandDecl {
            verb: "rm",
            fence: true,
            doc: Doc {
                display: "rm",
                category: HelpCategory::TreeNav,
                syntax: "rm <path> [-n]",
                description: "Remove: on the show tree the Lum `Delete` (its one ask names what uses it); under Root.Drives the file goes to that drive's .trash for 30 days and the reply says how to put it back, never a real delete (decisions §35 19). -n: no question.",
                examples: &["rm 3", "rm Root.Drives.Volumes.USB.old.luma -n"],
                related: &["move", "copy", "delete"],
                ..Doc::BASE
            },
            undo: STORE_MAIN,
            scope: Scope::Station,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Setup,
            reach: Reach::TreeShell,
            flags: &[crate::door::NO_ASK],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::typed::fenced::<crate::words::Diff>,
        decl: CommandDecl {
            verb: "diff",
            fence: true,
            doc: Doc {
                display: "diff",
                category: HelpCategory::TreeNav,
                also_applies_to: &[HelpCategory::Inspection],
                syntax: "diff <path1> <path2> [--all]",
                description: "Compare two objects field by field. \
                          Shows fields that differ. Use --all to also show matching fields.",
                examples: &[
                    "diff Root.ShowData.UserProfiles.1 Root.ShowData.UserProfiles.2  -- compare two user profiles",
                ],
                related: &["cat", "stat"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            args: Args::TreeItems {
                from: 1,
                options: &[],
            },
            rights: Rights::Guest,
            reach: Reach::TreeShell,
            flags: &[crate::words::DIFF_ALL],
            ..CommandDecl::BASE
        },
    },
];

/// The tree shell's objects (none).
pub static OBJECTS: &[&ObjectDef] = &[];

/// What the tree shell adds to how a line runs: `mv` / `cp` / `rm` on the show tree are the Lum
/// line they mean (§35 19).
pub static HOOKS: &[Hook] = &[Hook::Rewrite(crate::door::write_out)];
