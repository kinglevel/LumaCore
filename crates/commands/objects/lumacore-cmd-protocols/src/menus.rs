// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗███████╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║██╔════╝
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║███████╗
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║╚════██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝███████║
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
// FILE: menus

//! The DMX Protocols menu (decisions §30 "12d", §34 "34f7c hardened"): every protocol's lines,
//! the universes they carry, the nodes found.
//!
//! Owns:          the DMX Protocols menu: its lines, routing and discovery grids.
//! Doesn't own:   the grid builders (the kernel's menu_defs), the JSON (lumacore-views).
//! Entry points:  MENUS.
//! Decisions:     §30 "12d".

use lumacore_objects::menu_defs::*;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

static DMX_LINES: Grid = Grid {
    key: "dmx.lines",
    object: None,
    // Each protocol's object is its own crate's, above this one: the columns are what every DMX
    // line shares (decisions §34 "34f7c hardened" 7); a row names its own object.
    object_of: Some(crate::lines::any_dmx_line),
    row_object: Some("object"),
    id: "line",
    row_key: "guid",
    feed: "network",
    rows: "lines",
    cards: false,
    new_line: Some("New line…"),
    hardware: None,
    left_filter: Some("network"),
    store: Some("Store {value} /Network \"{left}\""),
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(set("Name", col("name", "Name", "name", 150, 1)))),
        ro("protocol", "Protocol", 100, 1),
        set("Mode", col("mode", "Mode", "mode", 90, 2)),
        ro("universes", "Universes", 140, 1),
        set(
            "Rate",
            section("Timing", unit("Hz", col("rate", "Rate", "rate_hz", 90, 1))),
        ),
        set(
            "Send",
            section("Timing", col("send", "Send", "send", 100, 1)),
        ),
        set(
            "Sync",
            section("Timing", col("sync", "Sync", "sync", 100, 2)),
        ),
        set(
            "PacketGap",
            section(
                "Timing",
                unit("µs", col("packet_gap", "Packet gap", "packet_gap", 100, 3)),
            ),
        ),
        set(
            "Target",
            section("Network", col("target", "Target", "target", 140, 3)),
        ),
        set("Enabled", col("enabled", "Enabled", "enabled", 90, 2)),
        toned("tone", ro("state", "State", 180, 1)),
    ],
};

static DMX_ROUTING: Grid = Grid {
    key: "dmx.routing",
    object: Some(&lumacore_objects::objects::UNIVERSE.decl),
    object_of: None,
    row_object: None,
    id: "universe",
    row_key: "key",
    feed: "network",
    rows: "routes",
    cards: false,
    new_line: None,
    hardware: Some("Lines carry universes by Assign (Assign Universe 1 Thru 4 At Sacn 1 /Universe 1)"),
    left_filter: Some("network"),
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("universe", "Universe", 90, 1)),
        toned("tone", ro("line", "Line", 120, 1)),
        ro("line_name", "Line name", 140, 2),
        ro("line_universe", "Line universe", 120, 1),
        set(
            "MergeMode",
            col("merge_mode", "Merge", "merge_mode", 110, 2),
        ),
    ],
};

static DMX_DISCOVERY: Grid = Grid {
    key: "dmx.discovery",
    object: None,
    object_of: None,
    row_object: None,
    id: "ip",
    row_key: "ip",
    feed: "network",
    rows: "nodes",
    cards: false,
    new_line: None,
    hardware: Some("Nodes are found on the network"),
    left_filter: Some("network"),
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("name", "Node", 160, 1)),
        ro("address", "Address", 130, 1),
        ro("protocol", "Kind", 120, 1),
        ro("universes", "Universes", 140, 2),
        ro("seen", "Last seen", 90, 2),
    ],
};

/// The DMX Protocols menu (decisions §30 "12d"): the lines, their routing, discovery.
pub static MENUS: &[MenuDef] = &[MenuDef {
    name: "DMXProtocols",
    label: "DMX Protocols",
    icon: "radio-tower",
    aliases: &["dmx", "protocols"],
    kind: MenuKind::Setup,
    home: "Lines: in the show · NIC map: this station's row",
    doc: "The show's lines (each protocol of this build) per network, their routing and the nodes found on the networks",
    power: Some(Power {
        label: "DMX output",
        feed: "network",
        path: "output_held",
        invert: true,
        off_values: &[],
        on: Look {
            word: "SENDING",
            press: "hold output (asks)",
            line: "Set Station \"HoldOutput\" On",
        },
        off: Look {
            word: "OUTPUT HELD",
            press: "send",
            line: "Set Station \"HoldOutput\" Off",
        },
        failing: Look {
            word: "SENDING",
            press: "a line can't send",
            line: "Set Station \"HoldOutput\" On",
        },
        rights: SETUP,
    }),
    left: Some(Left {
        feed: "network",
        rows: "networks",
        key: "name",
        sub: "counts",
        stripe: "flow",
        empty: "No networks yet: Menu \"Network\" /Tab Networks → New network.",
    }),
    area: None,
    tabs: &[
        Tab {
            name: "Lines",
            label: "Lines",
            sub: "",
            home: "show",
            phone: &[],
            content: Content::Grid(&DMX_LINES),
            actions: &[
                act(
                    "assign",
                    "Assign…",
                    "link",
                    "",
                    "Assign Universe {value} At {object} {rows}",
                    SETUP,
                    "one",
                ),
                act(
                    "info",
                    "Info",
                    "info",
                    "info",
                    "Info {object} {rows}",
                    GUEST,
                    "one",
                ),
                act(
                    "map",
                    "Map network…",
                    "",
                    "",
                    "Menu \"Network\" /Tab Networks",
                    GUEST,
                    "",
                ),
                SETTINGS_TAB,
            ],
            bound: &[],
        },
        Tab {
            name: "Routing",
            label: "Routing",
            sub: "",
            home: "show",
            phone: &[],
            content: Content::Grid(&DMX_ROUTING),
            actions: &[
                act(
                    "assign",
                    "Assign…",
                    "link",
                    "",
                    "Assign Universe {rows} At {value}",
                    SETUP,
                    "selection",
                ),
                act(
                    "unassign",
                    "Unassign…",
                    "",
                    "",
                    "Unassign Universe {rows} At {value}",
                    SETUP,
                    "selection",
                ),
            ],
            bound: &[],
        },
        Tab {
            name: "Discovery",
            label: "Discovery",
            sub: "",
            home: "station",
            phone: &[],
            content: Content::Grid(&DMX_DISCOVERY),
            actions: &[],
            bound: &[],
        },
    ],
}];
