//! The menu registry (decisions §30 "12d hardening": "Shape: a `MenuDef`
//! registry → JSON", the 12c model): every menu the UI opens — its tabs,
//! what each tab shows (a `<luma-grid>`, settings cells, cards), the grid
//! columns (their kind, range and level read from the object's declared
//! props, never repeated here), the function block (`<luma-slab>`: the
//! standard edit actions on the selection, the tab's own actions, bound
//! cells) and the power toggle. `Menu "<name>"` opens one (names from here),
//! `List Menu` / `Info Menu "x"` publish them and `GET /api/menu-types`
//! sends them to the kit's `<luma-setup>`. Nothing about a menu's shape is
//! stored in a show; a profile keeps only its last tab and its grids'
//! columns (`ProfileUi`).

use crate::commands::registry::Rights;

/// The menus' rows are the catalog's (decisions §34 21); the table is here, the JSON in
/// `lumacore_views::menus`.
pub use lumacore_registry::menu::{
    Action, Area, Bound, Col, Content, Detail, DetailGroup, Grid, Left, Look, MenuDef, MenuKind, Own, Part, PartContent, Power, Tab,
    TreeMode,
};

pub const fn col(id: &'static str, label: &'static str, field: &'static str, width: u32, priority: u8) -> Col {
    Col { id, label, field, prop: None, own: None, width, priority, frozen: false, section: "", unit: "", per_row: false, tone: "", expand: "", detail: "", draft: false }
}

pub const fn ro(id: &'static str, label: &'static str, width: u32, priority: u8) -> Col {
    col(id, label, id, width, priority)
}

pub const fn frozen(c: Col) -> Col {
    Col { frozen: true, ..c }
}

pub const fn set(prop: &'static str, c: Col) -> Col {
    Col { prop: Some(prop), ..c }
}

pub const fn section(s: &'static str, c: Col) -> Col {
    Col { section: s, ..c }
}

pub const fn unit(u: &'static str, c: Col) -> Col {
    Col { unit: u, ..c }
}

pub const fn own(o: Own, c: Col) -> Col {
    Col { own: Some(o), ..c }
}

pub const fn per_row(c: Col) -> Col {
    Col { per_row: true, ..c }
}

/// Changes only in a claim area's draft (§34 round 3, 10): locked in Live.
pub const fn draft(c: Col) -> Col {
    Col { draft: true, ..c }
}

pub const fn toned(tone: &'static str, c: Col) -> Col {
    Col { tone, ..c }
}

pub const fn kind_col(id: &'static str, label: &'static str, field: &'static str, tone: &'static str, detail: &'static str, priority: u8) -> Col {
    Col { tone, detail, ..col(id, label, field, 92, priority) }
}

pub const fn act(id: &'static str, label: &'static str, icon: &'static str, tone: &'static str, line: &'static str, rights: Rights, needs: &'static str) -> Action {
    Action { id, label, icon, tone, line, rights, needs, mine: "", claim: false, value: "", short: "", when: "" }
}

/// Shown only while the claim area's grid is the MVR import review (decisions §34 "Import MVR
/// hardened" 3).
pub const fn reviewing(a: Action) -> Action {
    Action { when: "review", ..a }
}

/// Its label in a narrow slab cell (decisions §37 3).
pub const fn short(label: &'static str, a: Action) -> Action {
    Action { short: label, ..a }
}

/// Needs the claim area's draft: greyed in Live (§34 round 3, 18).
pub const fn claimed(a: Action) -> Action {
    Action { claim: true, ..a }
}

/// Its `{value}` opens the value dialog of this kind (`address`, `number`).
pub const fn valued(kind: &'static str, a: Action) -> Action {
    Action { value: kind, ..a }
}

pub const fn mine(field: &'static str, a: Action) -> Action {
    Action { mine: field, ..a }
}

pub const fn bound(object: &'static str, prop: &'static str, label: &'static str) -> Bound {
    Bound { object, prop, label }
}

// ── The menus ────────────────────────────────────────────────────────────────

pub const GUEST: Rights = Rights::Guest;
pub const SETUP: Rights = Rights::Setup;
pub const PROGRAM: Rights = Rights::Program;
pub const PRESETS: Rights = Rights::Presets;
pub const ADMIN: Rights = Rights::Admin;

static ASSET_ROOTS: Grid = Grid {
    key: "settings.assets",
    object: Some(&crate::commands::media::ASSET_ROOT.decl),
    object_of: None,
    row_object: None,
    id: "no",
    row_key: "guid",
    feed: "network",
    rows: "asset_roots",
    cards: false,
    new_line: Some("New asset drive…"),
    hardware: None,
    left_filter: None,
    store: Some("Store AssetRoot {value}"),
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: None,
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(col("name", "Asset drive (show)", "name", 170, 1))),
        set("Folder", col("folder", "Folder here (this station)", "folder", 260, 1)),
        toned("tone", ro("state", "State", 120, 1)),
    ],
};

/// The rights ladder's words (`Set User "x" "Rights" …`).
const LEVELS: &[&str] = &["Guest", "Playback", "Presets", "Program", "Setup", "Admin"];

const fn part(name: &'static str, label: &'static str, path: &'static str, content: PartContent) -> Part {
    Part { name, label, path, content, actions: &[] }
}

static PROFILE_DETAIL: Detail = Detail {
    owner: "UserProfile {no}",
    path: "ShowData.UserProfiles.{no}",
    edit: "EditSetting UserProfile {no}",
    feed: "profile",
    groups: &[
        DetailGroup {
            name: "Settings",
            label: "Settings",
            path: "Settings",
            sub: "Look · Paint slots · Lock screen · Input & keys · Oops · Command line · Update · MVR import",
            parts: &[
                part("Look", "Look", "Theme", PartContent::Settings("Look")),
                part("PaintSlots", "Paint slots", "Theme.Paint", PartContent::Settings("Paint slots")),
                part("LockScreen", "Lock screen", "LockScreen", PartContent::Settings("Lock screen")),
                part("Input", "Input & keys", "Preferences", PartContent::Settings("Input & keys")),
                part("Oops", "Oops", "Preferences.Oops", PartContent::Settings("Oops")),
                part("CommandLine", "Command line", "Preferences", PartContent::Settings("Command line")),
                part("Update", "Update", "Preferences.Update", PartContent::Settings("Update")),
                part("MvrImport", "MVR import", "Preferences.Mvr", PartContent::Settings("MVR import")),
            ],
        },
        DetailGroup {
            name: "Content",
            label: "Content",
            path: "Content",
            sub: "Attributes · Encoder bars · Keys · Window presets · Images · Icons · Screens",
            parts: &[
                Part {
                    name: "Attributes",
                    label: "Attributes",
                    path: "Attributes",
                    content: PartContent::Settings("Attributes"),
                    actions: &[("Open the attribute board", "Edit {owner} Attribute"), ("New attributes", "Edit {owner} Attribute /New"), ("Rules", "List {owner} AttributeRule")],
                },
                part("EncoderBars", "Encoder bars", "Attributes.EncoderBars", PartContent::Custom("encoder_bars")),
                Part {
                    name: "Keys",
                    label: "Keys",
                    path: "Keybindings",
                    content: PartContent::List("keys", "key", &[frozen(ro("key", "Key", 120, 1)), ro("ui_key", "UI key", 160, 1), ro("description", "Description", 220, 2)]),
                    // Bind edits your own profile's keys (decisions 0295).
                    actions: &[("Open the key editor (your keys)", "Edit KeyBindings")],
                },
                part("WindowPresets", "Window presets", "WindowPresets", PartContent::List("window_presets", "guid", &[frozen(ro("no", "No", 48, 1)), ro("name", "Name", 160, 1), ro("type", "Window type", 140, 1)])),
                part("Images", "Images", "Images", PartContent::List("images", "guid", &[frozen(ro("no", "No", 48, 1)), ro("name", "Name", 160, 1), ro("dims", "Size", 110, 2), ro("ext", "Type", 70, 2)])),
                part("Icons", "Icons", "Icons", PartContent::List("icons", "guid", &[frozen(ro("no", "No", 48, 1)), ro("name", "Name", 160, 1), ro("ext", "Type", 70, 2)])),
                part("Screens", "Screens", "ScreenConfigs", PartContent::List("screens", "guid", &[frozen(ro("no", "No", 48, 1)), ro("name", "Name", 160, 1), ro("kind", "Kind", 120, 1), ro("screens", "Screens", 80, 2)])),
            ],
        },
    ],
};

static USER_DETAIL: Detail = Detail {
    owner: "User \"{name}\"",
    path: "ShowData.Users.{no}",
    edit: "EditSetting User \"{name}\"",
    feed: "user",
    groups: &[DetailGroup {
        name: "User",
        label: "User",
        path: "",
        sub: "",
        parts: &[
            part("Account", "Account", "", PartContent::Settings("")),
            Part {
                name: "Profile",
                label: "Profile",
                path: "Profile",
                content: PartContent::Settings("Profile"),
                actions: &[("Open profile", "Menu \"Users\" /Tab Profiles /Open \"{profile}\"")],
            },
            Part {
                name: "Work",
                label: "Work",
                path: "Programmer",
                content: PartContent::List("work", "what", &[frozen(ro("what", "What", 120, 1)), ro("value", "Now", 260, 1)]),
                actions: &[("Clear programmer", "Clear Programmer {owner}")],
            },
            part("Usage", "Usage", "Usage", PartContent::List("usage", "line", &[frozen(ro("line", "Line", 280, 1)), ro("count", "Used", 70, 1)])),
        ],
    }],
};

static USERS: Grid = Grid {
    key: "users.users",
    object: Some(&crate::commands::users::USER.decl),
    object_of: None,
    row_object: None,
    id: "name",
    row_key: "guid",
    feed: "station",
    rows: "users",
    cards: false,
    new_line: Some("New user…"),
    hardware: None,
    left_filter: None,
    store: None,
    locked: None,
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: Some(&USER_DETAIL),
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(own(Own::Text, set("Name", col("name", "Name", "name", 140, 1))))),
        own(Own::List(LEVELS), set("Rights", col("rights", "Rights", "rights", 110, 1))),
        own(Own::List(&[]), set("Profile", col("profile", "Profile", "profile", 130, 2))),
        ro("password", "Password", 100, 2),
        own(Own::Bool("Yes", "No"), set("RemoteLogin", col("remote_login", "Remote login", "remote_login", 120, 2))),
        ro("online", "Logged in", 150, 1),
    ],
};

static PROFILES: Grid = Grid {
    key: "users.profiles",
    object: Some(&crate::objects::USER_PROFILE.decl),
    object_of: None,
    row_object: None,
    id: "no",
    row_key: "guid",
    feed: "station",
    rows: "profiles",
    cards: false,
    new_line: Some("New profile…"),
    hardware: None,
    left_filter: None,
    store: None,
    locked: Some(("locked", "UserProfile Default is the locked template: nobody changes, renames or deletes it")),
    owner: None,
    matrix: &[],
    dmx_map: false,
    new_rights: None,
    detail: Some(&PROFILE_DETAIL),
    columns: &[
        frozen(ro("no", "No", 48, 1)),
        frozen(per_row(set("Name", col("name", "Name", "name", 140, 1)))),
        ro("used_by_text", "Used by", 160, 1),
        ro("screens", "Screens", 90, 2),
        ro("views", "Views", 90, 2),
        ro("locked", "Locked", 90, 2),
    ],
};

pub const SETTINGS_TAB: Action = short("Settings", act("settings", "Settings tab", "settings", "", "Menu \"Settings\" /Tab Station", GUEST, ""));

/// Every menu, in the order `List Menu` shows them.
/// Every menu — this crate's rows in the catalog; read them from the catalog.
pub(crate) static MENUS: &[MenuDef] = &[
    MenuDef {
        name: "Menu",
        label: "Menu",
        icon: "menu",
        aliases: &["main"],
        kind: MenuKind::Page,
        home: "",
        doc: "The ⚙ menu: tiles for every menu, window and setting (double press = Quick Save)",
        power: None,
        left: None,
        area: None,
        tabs: &[],
    },
    MenuDef {
        name: "Shutdown",
        label: "Shutdown",
        icon: "power",
        aliases: &["power"],
        kind: MenuKind::Page,
        home: "",
        doc: "Shutdown, Restart, Lock, Log out",
        power: None,
        left: None,
        area: None,
        tabs: &[],
    },
    MenuDef {
        name: "Oops",
        label: "Oops",
        icon: "undo-2",
        aliases: &["undo"],
        kind: MenuKind::Page,
        home: "",
        doc: "Your Oops list (decisions §30 \"12f\"): newest at the bottom; tap a step to select it and every newer one, then Oops n steps; undone steps dimmed below, Redo n; the chips are your profile's Oops filters. Hold the Oops key opens it",
        power: None,
        left: None,
        area: None,
        tabs: &[],
    },
    MenuDef {
        name: "Settings",
        label: "Settings",
        icon: "settings",
        aliases: &[],
        kind: MenuKind::Setup,
        home: "",
        doc: "This station's settings (never in the show), the show's (every station), the asset drives and the build",
        power: None,
        left: None,
        area: None,
        tabs: &[
            Tab {
                name: "Station",
                label: "Station",
                sub: "never in the show",
                home: "station",
                phone: &[],
                content: Content::Settings("Station"),
                actions: &[],
                bound: &[],
            },
            Tab {
                name: "Show",
                label: "Show",
                sub: "in the show",
                home: "show",
                phone: &[],
                content: Content::Settings("Show"),
                actions: &[],
                bound: &[],
            },
            Tab {
                name: "Assets",
                label: "Assets",
                sub: "drives",
                home: "",
                phone: &[],
                content: Content::Grid(&ASSET_ROOTS),
                actions: &[act("info", "Info", "info", "info", "List AssetRoot", GUEST, "")],
                bound: &[],
            },
            Tab {
                name: "About",
                label: "About",
                sub: "build, paths",
                home: "station",
                phone: &[],
                content: Content::Settings("Server"),
                actions: &[],
                bound: &[],
            },
        ],
    },
    MenuDef {
        name: "Users",
        label: "Users & Profiles",
        icon: "users",
        aliases: &["people", "userprofiles"],
        kind: MenuKind::Setup,
        home: "In the show · every station",
        doc: "Users (name, rights level, profile, password, remote login) and their profiles",
        power: None,
        left: None,
        area: None,
        tabs: &[
            Tab {
                name: "Users",
                label: "Users",
                sub: "",
                home: "show",
                phone: &["password"],
                content: Content::Grid(&USERS),
                actions: &[
                    // Your own password is yours; another user's is Admin's.
                    mine("name", act("password", "Password…", "lock", "", "Set User {rows} \"Password\" Ask", ADMIN, "one")),
                    act("rename", "Rename…", "pencil", "", "Set User {rows} \"Name\" Ask", ADMIN, "one"),
                    act("rights", "Rights…", "", "", "EditSetting User {rows}", ADMIN, "one"),
                    act("profile", "Profile…", "", "", "Menu \"Users\" /Tab Profiles", GUEST, ""),
                    act("settings", "Settings…", "settings", "", "EditSetting User {rows}", GUEST, "one"),
                ],
                bound: &[],
            },
            Tab {
                name: "Profiles",
                label: "Profiles",
                sub: "",
                home: "show",
                phone: &["settings"],
                content: Content::Grid(&PROFILES),
                actions: &[
                    act("rename", "Rename…", "pencil", "", "Label UserProfile {rows}", ADMIN, "one"),
                    act("settings", "Settings…", "settings", "", "EditSetting UserProfile {rows}", GUEST, "one"),
                ],
                bound: &[],
            },
        ],
    },
];

/// The menu `name` names (its name or an alias, any case, quotes ignored).
pub fn find(name: &str) -> Option<&'static MenuDef> {
    lumacore_registry::get().menu(name)
}


/// Every grid key of every menu.
pub fn grid_keys() -> impl Iterator<Item = &'static str> {
    lumacore_registry::get().menus().flat_map(|m| m.grid_keys())
}
