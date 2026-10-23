// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝
// FILE: menu

//! The Patch setup dialog's row (decisions §34 "34g4 patch hardened" 9; the locked mockup
//! `2026-10-18-34g4`, part 3a): Devices › DMX › Patch with the Patch, Fixture types and Universes
//! grids on the `patch` feed — the draft for the holder of the Devices draft, the live devices for
//! everyone else. The tree sidebar, kind rail and Exit come with the kit (3b), the MVR tree's
//! actions with 3c.
//!
//! Owns:          MENUS (the Patch menu row), its grids, CONTEXT (the patch objects' own part of
//!                the one context menu, §34 round 3, 18).
//! Doesn't own:   the rows (`lumacore_views::patch`), the Fixture's properties (`objects::patch`).
//! Entry points:  MENUS, CONTEXT.
//! Decisions:     §34 "34g4 patch hardened" 9, 12.

use crate::menu_defs::*;
use crate::commands::registry::Rights;
use crate::objects::{ContextDef, ContextItem, Verb};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

static FIXTURES: Grid = Grid {
    key: "patch.fixtures",
    object: Some(&crate::objects::FIXTURE.decl),
    object_of: None,
    row_object: None,
    id: "no",
    row_key: "guid",
    feed: "patch",
    rows: "fixtures",
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
        frozen(draft(ro("no", "FID", 56, 1))),
        frozen(per_row(set("Name", col("name", "Name", "name", 160, 1)))),
        draft(ro("fixture_type", "Fixture type", 160, 1)),
        draft(ro("mode", "Mode", 120, 1)),
        // Red where it overlaps another fixture or runs past 512, "no universe" without one.
        toned("address_tone", set("Address", col("address", "Address", "address", 110, 1))),
        set("PosX", section("Position", unit("m", col("pos_x", "X", "pos_x", 70, 2)))),
        set("PosY", section("Position", unit("m", col("pos_y", "Y", "pos_y", 70, 2)))),
        set("PosZ", section("Position", unit("m", col("pos_z", "Z", "pos_z", 70, 2)))),
        set("RotX", section("Position", unit("°", col("rot_x", "Rot X", "rot_x", 70, 3)))),
        set("RotY", section("Position", unit("°", col("rot_y", "Rot Y", "rot_y", 70, 3)))),
        set("RotZ", section("Position", unit("°", col("rot_z", "Rot Z", "rot_z", 70, 3)))),
        set("MasterReact", col("master_react", "Master react", "master_react", 100, 2)),
        ro("tags", "Tags", 140, 2),
        draft(ro("group", "MVR node", 120, 2)),
        set("Note", col("note", "Note", "note", 160, 2)),
    ],
};

static UNIVERSES: Grid = Grid {
    key: "patch.universes",
    object: Some(&crate::objects::UNIVERSE.decl),
    object_of: None,
    row_object: None,
    id: "no",
    row_key: "guid",
    feed: "patch",
    rows: "universes",
    cards: false,
    new_line: None,
    hardware: None,
    left_filter: None,
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: true,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 56, 1)),
        frozen(per_row(set("Name", col("name", "Name", "name", 160, 1)))),
        set("MergeMode", col("merge_mode", "Merge", "merge_mode", 110, 1)),
        ro("fixtures", "Fixtures", 90, 1),
    ],
};

static FIXTURE_TYPES: Grid = Grid {
    key: "patch.fixture_types",
    object: Some(&crate::objects::FIXTURE_TYPE.decl),
    object_of: None,
    row_object: None,
    id: "no",
    row_key: "guid",
    feed: "patch",
    rows: "fixture_types",
    cards: false,
    new_line: None,
    hardware: Some("Fixture types arrive by import (Import FixtureType \"file.gdtf\")"),
    left_filter: None,
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 56, 1)),
        frozen(per_row(set("Name", col("name", "Name", "name", 180, 1)))),
        ro("manufacturer", "Manufacturer", 140, 1),
        ro("revisions", "Revisions", 90, 2),
        ro("modes", "Modes", 80, 2),
        ro("fixtures", "Fixtures", 90, 1),
    ],
};

/// The Patch tab's function block (decisions §34 round 3, 11, 18; `patch-grid.html`): an address
/// is a live column, so Patch to… and Next free address work in Live.
static FIXTURE_ACTIONS: &[Action] = &[
    // The two-step dialog (`insert-fixtures.html` v6) writes the Store line; New opens it too.
    claimed(valued("insert", short("Insert…", act("insert", "Insert fixtures…", "plus", "", "Store {object}", Rights::Setup, "")))),
    valued("address", act("patch_to", "Patch to…", "plug", "", "Assign {object} {rows} At DMX {value}", Rights::Setup, "selection")),
    // The DMX map zoomed in on the selection (§34 round 3, the ways into Edit Patch): opening
    // runs no line.
    valued("editpatch", act("edit_patch", "Edit Patch…", "grid-3x3", "", "", Rights::Guest, "selection")),
    // `packed`: the Offset cell's spare slots join the line as `/Every`.
    valued("packed", short("Next free", act("next_free", "Next free address", "plug", "", "Assign {object} {rows} At DMX", Rights::Setup, "selection"))),
    // Each fixture's next copy at an address (round 3, the left-out items: a bare Multipatch).
    claimed(valued("copies", act("multipatch", "Multipatch…", "copy", "", "Store {object} {rows} Multipatch At DMX {value}", Rights::Setup, "selection"))),
    act("info", "Info", "info", "info", "Info {object} {rows}", Rights::Guest, "selection"),
    // Import MVR (decisions §34 "Import MVR hardened" 1): an own action of value `import` marks the
    // slab's Import cell as the page's — the two-step dialog (File · Options), then the review.
    claimed(valued("import", short("Import…", act("mvr", "Import MVR…", "upload", "", "", Rights::Setup, "")))),
    // The review's own cells (decision 3): shown in the grid's review mode, the rest hidden.
    reviewing(valued("takecell", short("Take", act("take_cell", "Take cell", "check", "", "Import Mvr {rows} /Take {value}", Rights::Setup, "selection")))),
    reviewing(valued("offercell", short("Offer", act("offer_cell", "Offer cell", "minus", "", "Import Mvr {rows} /Offer {value}", Rights::Setup, "selection")))),
    reviewing(short("Apply", act("apply_review", "Apply ticked", "check", "load", "Import Mvr {rows} /Apply", Rights::Setup, "selection"))),
    reviewing(short("Cancel", act("cancel_review", "Cancel import", "x", "alert", "Import Mvr /Cancel", Rights::Setup, ""))),
    // View cells: the grid's order and the spare slots the DMX actions leave (the user's grid
    // state, `Set GridColumns`; no show change).
    valued("sort", act("sort", "Sort", "arrow-down-up", "", "", Rights::Guest, "")),
    valued("offset", act("offset", "Offset", "", "", "", Rights::Guest, "")),
];

/// Devices › DMX › Patch: the area the dialog edits (the locked round 2).
static DEVICES: Area = Area {
    trail: &["Devices", "DMX"],
    kinds: &[("DMX", "Patch")],
    kinds_label: "Devices",
    later: "+ more device kinds later",
    feed: "patch",
    tree: &[
        // Nodes only — layers, groups, folder fixtures (decisions §40 "40d hardened" 1–2); Deep
        // is the default: a stage's fixtures sit in its folders.
        TreeMode { id: "tree", label: "MVR tree", rows: "tree", filter: "parent", list: false, nested: true, none: "No MVR node", deep: true, read_only: false },
        TreeMode { id: "class", label: "Class", rows: "classes", filter: "class", list: false, nested: false, none: "No class", deep: false, read_only: true },
        TreeMode { id: "tag", label: "Tag", rows: "tag_nodes", filter: "tag_guids", list: true, nested: true, none: "No tag", deep: true, read_only: true },
    ],
    claim: "Claim Root.ShowData.Devices",
    exit: "Unclaim Root.ShowData.Devices",
    close: "Unclaim Root.ShowData.Devices; Menu \"Patch\"",
    bars: [
        ("Live patch", "Name, address, X / Y / Z, master react and note change at once, as your own Oops step. Hold Claim to change the rest."),
        ("Your draft", "Every column edits the draft; the rig keeps the live patch until you leave with Save. Gold dot = your edit · violet = a live edit that reached your draft."),
        ("{holder} holds the draft", "Live columns still change at once, and {holder}'s draft follows — a cell {holder} changed is refused, naming them."),
    ],
    adds: &[("+ Layer", "Store MvrLayer {value}"), ("+ Group", "Store MvrGroup {value}")],
    unit: "fixtures",
};

pub static MENUS: &[MenuDef] = &[MenuDef {
    name: "Patch",
    label: "Patch",
    icon: "plug",
    aliases: &["devices"],
    kind: MenuKind::Setup,
    home: "Devices › DMX › Patch: in the show",
    doc: "The patch (Devices › DMX › Patch): fixtures, fixture types and universes. Live columns change at once; Claim Root.ShowData.Devices (or enter patch) edits a draft of every device kind",
    power: None,
    left: None,
    area: Some(&DEVICES),
    tabs: &[
        Tab {
            name: "Patch",
            label: "Patch",
            sub: "fixtures",
            home: "show",
            phone: &["patch_to", "edit_patch", "insert", "next_free", "multipatch"],
            content: Content::Grid(&FIXTURES),
            actions: FIXTURE_ACTIONS,
            bound: &[],
        },
        Tab {
            name: "FixtureTypes",
            label: "Fixture types",
            sub: "GDTF",
            home: "show",
            phone: &[],
            content: Content::Grid(&FIXTURE_TYPES),
            actions: &[],
            bound: &[],
        },
        Tab {
            name: "Universes",
            label: "Universes",
            sub: "512 slots each",
            home: "show",
            phone: &[],
            content: Content::Grid(&UNIVERSES),
            actions: &[],
            bound: &[],
        },
    ],
}];

/// What only a fixture and a universe do, in the one context menu (decisions §34 round 3, 18, the
/// approved `context-menu.html`): an address is a live column, so the DMX lines run in Live too.
pub static CONTEXT: &[ContextDef] = &[
    ContextDef {
        object: &crate::objects::FIXTURE.decl,
        items: &[
            ContextItem { label: "Patch to…", icon: "plug", verb: Verb::Assign, line: "Assign {object} {rows} At DMX {value}", live: true, value: "address" },
            ContextItem { label: "Edit Patch…", icon: "grid-3x3", verb: Verb::Info, line: "", live: true, value: "editpatch" },
            ContextItem { label: "Next free address", icon: "plug", verb: Verb::Assign, line: "Assign {object} {rows} At DMX", live: true, value: "" },
            ContextItem { label: "Swap addresses…", icon: "arrow-left-right", verb: Verb::Assign, line: "Assign {object} {rows} At Fixture {value} DMX /Swap", live: true, value: "" },
            ContextItem { label: "Unpatch", icon: "unplug", verb: Verb::Unassign, line: "Unassign {object} {rows} At DMX", live: true, value: "" },
            ContextItem { label: "Info", icon: "info", verb: Verb::Info, line: "Info {object} {rows}", live: true, value: "" },
        ],
    },
    ContextDef {
        object: &crate::objects::UNIVERSE.decl,
        items: &[ContextItem { label: "Info", icon: "info", verb: Verb::Info, line: "Info {object} {rows}", live: true, value: "" }],
    },
];
