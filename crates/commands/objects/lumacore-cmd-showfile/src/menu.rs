// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝
// FILE: menu

//! The Backup menu (decisions §30 "12d part 5"): three grids and the fixed show actions.
//!
//! Owns:          the Backup menu: the library, the show's history, the recovery copies; their slab actions.
//! Doesn't own:   the grid builders (the kernel's menu_defs), the JSON (lumacore-views menus).
//! Entry points:  MENUS.
//! Decisions:     §30 "12d part 5".

use lumacore_objects::menu_defs::*;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

static LIBRARY: Grid = Grid {
    key: "backup.library",
    object: Some(&crate::show::SHOW.decl),
    object_of: None,
    row_object: None,
    id: "name",
    row_key: "name",
    feed: "shows",
    rows: "shows",
    cards: false,
    new_line: Some("New show…"),
    hardware: None,
    left_filter: None,
    store: Some("NewShow {value}"),
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: Some(ADMIN),
    detail: None,
    columns: &[
        frozen(per_row(col("name", "Show", "name", 200, 1))),
        ro("saved", "Last saved", 150, 1),
        ro("size_text", "Size", 90, 2),
        toned("format_tone", ro("format_text", "Format", 150, 1)),
        ro("by", "Saved by", 110, 2),
        ro("station", "Station", 110, 3),
        ro("versions", "Versions", 90, 2),
    ],
};

static HISTORY: Grid = Grid {
    key: "backup.history",
    object: Some(&crate::copies::VERSION.decl),
    object_of: None,
    row_object: None,
    id: "vid",
    row_key: "id",
    feed: "shows",
    rows: "history",
    cards: false,
    new_line: None,
    hardware: None,
    left_filter: None,
    store: None,
    locked: Some((
        "current",
        "The current file is the show: save again to keep it as a version",
    )),
    owner: Some("history_of"),
    matrix: &[
        frozen(toned("reason_tone", ro("saved", "Saved", 150, 1))),
        kind_col(
            "m_fixtures",
            "Fixtures",
            "k_Fixtures",
            "t_Fixtures",
            "d_Fixtures",
            1,
        ),
        kind_col("m_patch", "Patch", "k_Patch", "t_Patch", "d_Patch", 1),
        kind_col("m_groups", "Groups", "k_Groups", "t_Groups", "d_Groups", 1),
        kind_col(
            "m_presets",
            "Presets",
            "k_Presets",
            "t_Presets",
            "d_Presets",
            1,
        ),
        kind_col(
            "m_sequences",
            "Sequences",
            "k_Sequences",
            "t_Sequences",
            "d_Sequences",
            1,
        ),
        kind_col("m_cues", "Cues", "k_Cues", "t_Cues", "d_Cues", 1),
        kind_col("m_macros", "Macros", "k_Macros", "t_Macros", "d_Macros", 1),
        kind_col("m_lines", "Lines", "k_Lines", "t_Lines", "d_Lines", 1),
        kind_col("m_users", "Users", "k_Users", "t_Users", "d_Users", 1),
        kind_col(
            "m_profiles",
            "Profiles",
            "k_Profiles",
            "t_Profiles",
            "d_Profiles",
            1,
        ),
    ],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(toned("reason_tone", ro("saved", "Saved", 150, 1))),
        ro("by", "By", 100, 1),
        ro("station", "Station", 110, 2),
        toned("reason_tone", ro("reason", "Reason", 120, 1)),
        own(Own::Text, set("Note", col("note", "Note", "note", 170, 2))),
        ro("changes", "Changes", 240, 1),
        toned("kept_tone", ro("kept", "Kept", 100, 1)),
        own(
            Own::Bool("Yes", "No"),
            set("Pinned", col("pinned", "Pinned", "pinned", 90, 3)),
        ),
    ],
};

static RECOVERY: Grid = Grid {
    key: "backup.recovery",
    object: Some(&crate::copies::RECOVERY.decl),
    object_of: None,
    row_object: None,
    id: "id",
    row_key: "id",
    feed: "shows",
    rows: "recovery",
    cards: false,
    new_line: None,
    hardware: None,
    left_filter: None,
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("time", "Time", 150, 1)),
        ro("show", "Show", 140, 1),
        toned("reason_tone", ro("reason", "Reason", 140, 1)),
        ro("by", "Station · user", 150, 2),
        ro("size_text", "Size", 90, 3),
        toned("kept_tone", ro("kept", "Kept", 110, 1)),
        own(Own::Text, set("Note", col("note", "Note", "note", 150, 3))),
        own(
            Own::Bool("Yes", "No"),
            set("Pinned", col("pinned", "Pinned", "pinned", 90, 2)),
        ),
    ],
};

/// The show actions, fixed and the same on Library and Show · History
/// (decisions §30 "Show actions are fixed": one icon and colour each).
const LOAD_SHOW: Action = short("Load…", act("load", "Load show…", "folder-open", "load", "", SETUP, "one"));
const SAVE_SHOW: Action = short("Save", act("save", "Save show", "save", "save", "SaveShow{drive}", PRESETS, ""));
const SAVE_AS: Action = act(
    "saveas",
    "Save as…",
    "save-plus",
    "save",
    "SaveShow {value}{drive}",
    PROGRAM,
    "",
);
const NEW_SHOW: Action = short("New…", act("new", "New show…", "file-plus-2", "neutral", "NewShow {value}", ADMIN, ""));

static LIBRARY_ACTIONS: &[Action] = &[
    LOAD_SHOW,
    SAVE_SHOW,
    SAVE_AS,
    NEW_SHOW,
    act(
        "info",
        "Info",
        "info",
        "info",
        "Info Show {rows}",
        GUEST,
        "one",
    ),
    // The page shows the selected show's history (`shows` feed `history`),
    // then runs `Menu "Backup" /Tab History`.
    act(
        "history",
        "History…",
        "history",
        "history",
        "",
        GUEST,
        "one",
    ),
];

static HISTORY_ACTIONS: &[Action] = &[
    act(
        "load",
        "Load show…",
        "folder-open",
        "load",
        "Menu \"Backup\" /Tab Library",
        GUEST,
        "",
    ),
    SAVE_SHOW,
    SAVE_AS,
    NEW_SHOW,
    act(
        "open",
        "Open version",
        "",
        "",
        "LoadShow {owner} /Version {rows}",
        SETUP,
        "one",
    ),
    act("parts", "Take parts…", "", "", "", SETUP, "one"),
];

static RECOVERY_ACTIONS: &[Action] = &[
    act(
        "open",
        "Open copy",
        "folder-open",
        "",
        "LoadShow /Recovery {rows}",
        SETUP,
        "one",
    ),
    act("parts", "Take parts…", "", "", "", SETUP, "one"),
    act(
        "pin",
        "Pin",
        "",
        "",
        "Set Recovery {rows} \"Pinned\" Yes",
        PROGRAM,
        "selection",
    ),
    act(
        "clean",
        "Clean up now",
        "",
        "",
        "Delete Recovery /Cleanup",
        SETUP,
        "",
    ),
    act(
        "rules",
        "Cleanup rules",
        "settings",
        "",
        "Menu \"Settings\" /Tab Station",
        GUEST,
        "",
    ),
];

/// The Backup menu (decisions §30 "12d part 5"): the library, the show's history, the recovery
/// copies; `register` adds it to the catalog.
pub static MENUS: &[MenuDef] = &[MenuDef {
    name: "Backup",
    label: "Backup",
    icon: "save",
    aliases: &[],
    kind: MenuKind::Setup,
    home: "library/shows on this station",
    doc: "The library of shows, the loaded show's history (one row per save: List | Matrix) and this station's recovery copies",
    power: None,
    left: None,
    area: None,
    tabs: &[
        Tab {
            name: "Library",
            label: "Library",
            sub: "shows on this station",
            home: "station",
            phone: &["load", "save", "new"],
            content: Content::Drives(&LIBRARY),
            actions: LIBRARY_ACTIONS,
            bound: &[],
        },
        Tab {
            name: "History",
            label: "Show · History",
            sub: "one row per save",
            home: "station",
            phone: &["load", "save", "new"],
            content: Content::Grid(&HISTORY),
            actions: HISTORY_ACTIONS,
            bound: &[],
        },
        Tab {
            name: "Recovery",
            label: "Recovery",
            sub: "copies on this station",
            home: "station",
            phone: &["open", "clean"],
            content: Content::Grid(&RECOVERY),
            actions: RECOVERY_ACTIONS,
            bound: &[],
        },
    ],
}];
