// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗███████╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║██╔════╝
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║███████╗
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║╚════██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝███████║
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
// FILE: menus

//! The station's menus: Network (its interfaces, its NIC for each network) and LumaSessions (the
//! stations of the session, the known ones, this station).
//!
//! Owns:          the Network and LumaSessions menus with their grids.
//! Doesn't own:   the grid builders (the kernel's menu_defs), the JSON (lumacore-views).
//! Entry points:  MENUS.
//! Decisions:     §30 "12d", §34 25.17.

use lumacore_objects::menu_defs::*;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

static INTERFACES: Grid = Grid {
    key: "network.interfaces",
    object: Some(&crate::stations::INTERFACE.decl),
    object_of: None,
    row_object: None,
    id: "name",
    row_key: "name",
    feed: "network",
    rows: "interfaces",
    cards: false,
    new_line: None,
    hardware: Some("Interfaces come from the hardware"),
    left_filter: None,
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(col("name", "Interface", "name", 140, 1))),
        ro("address", "Address", 160, 2),
        toned("link_tone", ro("link", "Link", 120, 2)),
        set(
            "LumaSession",
            section(
                "Services",
                col("lumasession", "LumaSession", "lumasession", 120, 1),
            ),
        ),
        set(
            "Shell",
            section("Services", col("shell", "Shell", "shell", 100, 1)),
        ),
        set(
            "HTTP",
            section("Services", col("http", "HTTP", "http", 100, 1)),
        ),
    ],
};

/// Menu → Drives (decisions §38): this station's drives on the `station` feed's `drives` rows.
static DRIVES: Grid = Grid {
    key: "drives.list",
    object: Some(&crate::drives::DRIVE.decl),
    object_of: None,
    row_object: None,
    id: "id",
    row_key: "id",
    feed: "station",
    rows: "drives",
    cards: false,
    new_line: None,
    hardware: Some("Drives come from the hardware: put a stick in, or Add folder…"),
    left_filter: None,
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(ro("id", "Drive", 130, 1))),
        set("Label", col("label", "Label", "label", 160, 1)),
        set("Kind", col("kind", "Kind", "kind", 90, 1)),
        toned("tone", ro("state", "State", 70, 1)),
        toned("library_tone", set("Library", col("library", "Library", "library", 90, 1))),
        ro("free", "Free", 90, 2),
        ro("total", "Total", 90, 3),
        ro("path", "Path", 260, 2),
    ],
};

static NETWORKS: Grid = Grid {
    key: "network.networks",
    object: Some(&crate::network::NETWORK.decl),
    object_of: None,
    row_object: None,
    id: "id",
    row_key: "guid",
    feed: "network",
    rows: "networks",
    cards: false,
    new_line: Some("New network…"),
    hardware: None,
    left_filter: None,
    store: Some("Store Network {value}"),
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(set(
            "Name",
            col("name", "Network (show)", "name", 160, 1),
        ))),
        set(
            "Interface",
            col("interface", "Interface (this station)", "interface", 170, 1),
        ),
        toned("tone", ro("state", "State", 180, 1)),
        ro("lines", "Lines on it", 200, 2),
        set("Note", col("note", "Note", "note", 160, 3)),
    ],
};

static STATIONS: Grid = Grid {
    key: "lumasessions.stations",
    object: Some(&crate::stations::STATION.decl),
    object_of: None,
    row_object: None,
    id: "name",
    row_key: "guid",
    feed: "lumasessions",
    rows: "members",
    cards: true,
    new_line: Some("New station row…"),
    hardware: None,
    left_filter: None,
    store: Some("Store Station {value}"),
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(toned(
            "tone",
            set("Name", col("name", "Station", "name", 150, 1)),
        ))),
        ro("role", "Role", 100, 1),
        set(
            "Priority",
            section("Station", col("priority", "Priority", "priority", 110, 2)),
        ),
        set(
            "Location",
            section("Station", col("location", "Location", "location", 150, 2)),
        ),
        // One column per show network: the station's NIC for it (its row).
        Col {
            expand: "networks",
            ..section("Wiring", col("nic", "→ NIC", "nic", 110, 2))
        },
        unit("ms", ro("rtt", "Round trip", 100, 2)),
        ro("users", "Users", 140, 3),
    ],
};

static KNOWN: Grid = Grid {
    key: "lumasessions.known",
    object: Some(&crate::stations::STATION.decl),
    object_of: None,
    row_object: None,
    id: "name",
    row_key: "guid",
    feed: "lumasessions",
    rows: "known",
    cards: false,
    new_line: None,
    hardware: None,
    left_filter: None,
    // A station is known once it joined; New here makes a station table row.
    store: Some("Store Station {value}"),
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("name", "Station", 150, 1)),
        ro("address", "Address", 140, 1),
        toned("tone", ro("state", "State", 120, 1)),
        ro("priority", "Priority", 100, 2),
        ro("seen_text", "First joined", 160, 2),
    ],
};

/// The station's menus (decisions §34 25.17): Network (its interfaces and its NIC for each
/// network) and LumaSessions (the stations of the session, the known ones, this station).
pub static MENUS: &[MenuDef] = &[
    MenuDef {
        name: "Drives",
        label: "Drives",
        icon: "hard-drive",
        aliases: &["drive"],
        kind: MenuKind::Setup,
        home: "This station · never in the show",
        doc: "This station's drives: the local library, the volumes it finds, the folders you add — each with the library tree LumaCore/library, made on the first write",
        power: None,
        left: None,
        area: None,
        tabs: &[Tab {
            name: "Drives",
            label: "Drives",
            sub: "",
            home: "",
            phone: &[],
            content: Content::Grid(&DRIVES),
            actions: &[
                act("init", "Initialise", "folder-plus", "", "Set Drive {rows} \"Library\" Yes", SETUP, "selection"),
                act("eject", "Eject", "", "", "Eject Drive {rows}", SETUP, "selection"),
                act("rename", "Rename…", "", "", "Set Drive {rows} \"Label\" {value}", SETUP, "selection"),
                act("add", "Add folder…", "", "", "Store Drive /Path {value}", SETUP, ""),
                act("remove", "Remove", "", "", "Delete Drive {rows}", SETUP, "selection"),
                act("info", "Info", "info", "info", "Info Drive", GUEST, ""),
                SETTINGS_TAB,
            ],
            bound: &[],
        }],
    },
    MenuDef {
        name: "Network",
        label: "Network",
        icon: "network",
        aliases: &[],
        kind: MenuKind::Setup,
        home: "Wiring: this station's row in the show · identity: this station",
        doc: "This station's interfaces and services, and its interface for each of the show's networks (its row of the station table)",
        power: Some(Power {
            label: "Network",
            feed: "network",
            path: "online",
            invert: false,
            off_values: &[],
            on: Look {
                word: "ONLINE",
                press: "go offline (asks)",
                line: "Set Station \"Online\" Off",
            },
            off: Look {
                word: "OFFLINE",
                press: "go online",
                line: "Set Station \"Online\" On",
            },
            failing: Look {
                word: "ONLINE",
                press: "no interface serves",
                line: "Set Station \"Online\" Off",
            },
            rights: SETUP,
        }),
        left: None,
        area: None,
        tabs: &[
            Tab {
                name: "Interfaces",
                label: "Interfaces",
                sub: "",
                home: "",
                phone: &[],
                content: Content::Grid(&INTERFACES),
                actions: &[
                    act(
                        "all",
                        "All services",
                        "",
                        "",
                        "Set Interface {rows} \"LumaSession\" Yes; Set Interface {rows} \"Shell\" Yes; Set Interface {rows} \"HTTP\" Yes",
                        SETUP,
                        "selection",
                    ),
                    act("info", "Info", "info", "info", "Info Interface", GUEST, ""),
                    act(
                        "networks",
                        "Networks…",
                        "",
                        "",
                        "Menu \"Network\" /Tab Networks",
                        GUEST,
                        "",
                    ),
                    SETTINGS_TAB,
                ],
                bound: &[
                    bound("Station", "Name", "Station name"),
                    bound("Station", "Location", "Location"),
                    bound("Station", "Priority", "Priority"),
                    bound("Station", "MulticastBase", "Multicast base"),
                ],
            },
            Tab {
                name: "Networks",
                label: "Networks",
                sub: "in the show",
                home: "",
                phone: &[],
                content: Content::Grid(&NETWORKS),
                actions: &[
                    act(
                        "lines",
                        "DMX lines…",
                        "",
                        "",
                        "Menu \"DMXProtocols\"",
                        GUEST,
                        "",
                    ),
                    act("info", "Info", "info", "info", "Info Network", GUEST, ""),
                    SETTINGS_TAB,
                ],
                bound: &[],
            },
        ],
    },
    MenuDef {
        name: "LumaSessions",
        label: "LumaSessions",
        icon: "waypoints",
        aliases: &["lumasession", "sessions"],
        kind: MenuKind::Setup,
        home: "Identity: each station · wiring: in the show",
        doc: "The stations of this LumaSession with their wiring rows, every station that ever joined, and this station",
        power: Some(Power {
            label: "LumaSession",
            feed: "lumasessions",
            path: "station.status",
            invert: false,
            off_values: &["Standalone"],
            on: Look {
                word: "JOINED",
                press: "leave (asks)",
                line: "Leave LumaSession",
            },
            off: Look {
                word: "STANDALONE",
                press: "join",
                line: "Join LumaSession \"{station.lumasession}\"",
            },
            failing: Look {
                word: "JOINED",
                press: "the master is lost",
                line: "Leave LumaSession",
            },
            rights: SETUP,
        }),
        left: None,
        area: None,
        tabs: &[
            Tab {
                name: "Stations",
                label: "Stations",
                sub: "",
                home: "",
                phone: &["invite", "hand"],
                content: Content::Grid(&STATIONS),
                actions: &[
                    act(
                        "invite",
                        "Invite…",
                        "",
                        "",
                        "Invite Station {value}",
                        SETUP,
                        "",
                    ),
                    act(
                        "hand",
                        "Hand master…",
                        "",
                        "",
                        "Claim LumaSession Station {rows}",
                        SETUP,
                        "one",
                    ),
                    act(
                        "pause",
                        "Pause",
                        "pause",
                        "",
                        "Pause LumaSession",
                        SETUP,
                        "",
                    ),
                    act(
                        "info",
                        "Info",
                        "info",
                        "info",
                        "Info Station {rows}",
                        GUEST,
                        "one",
                    ),
                    act(
                        "dismiss",
                        "Dismiss",
                        "",
                        "",
                        "Dismiss Station {rows}",
                        SETUP,
                        "one",
                    ),
                    SETTINGS_TAB,
                ],
                bound: &[],
            },
            Tab {
                name: "Known",
                label: "Known",
                sub: "ever joined",
                home: "show",
                phone: &["invite"],
                content: Content::Grid(&KNOWN),
                actions: &[act(
                    "invite",
                    "Invite",
                    "",
                    "",
                    "Invite Station {rows}",
                    SETUP,
                    "one",
                )],
                bound: &[],
            },
            Tab {
                name: "ThisStation",
                label: "This station",
                sub: "",
                home: "station",
                phone: &[],
                content: Content::Settings("Station"),
                actions: &[],
                bound: &[],
            },
        ],
    },
];
