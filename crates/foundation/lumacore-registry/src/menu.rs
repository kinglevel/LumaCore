// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝
// FILE: menu
//! A menu as the catalog holds it (decisions §30 "12d hardening": a `MenuDef` registry → JSON,
//! the 12c model): its name, how it is drawn, its tabs (left tabs from a feed), what each tab shows
//! (a grid, settings cells, cards), the grids' columns (their kind, range and level read from
//! the object's declared properties, never repeated here), the slab's actions, bound cells and
//! the power toggle. Pure data: the objects crate declares the menus, the views build their JSON
//! and rows (decisions §34 21).
//!
//! Owns:          MenuDef and its parts (MenuKind, Left, Power, Look, Tab, Content, Grid, Detail,
//!                DetailGroup, Part, PartContent, Col, Own, Action, Bound)
//! Doesn't own:   the table of menus (lumacore-objects), their JSON (the views)
//! Entry points:  MenuDef, Tab, Content, Grid, Col
//! Decisions:     §30 "12d", §34 21

use crate::command::Rights;
use crate::object::ObjectDecl;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How the UI draws a menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum MenuKind {
    /// The kit's setup frame (`<luma-setup>`).
    Setup,
    /// Drawn by the page itself (the ⚙ Menu tiles, Shutdown).
    Page,
}

pub struct MenuDef {
    /// The name `Menu "<name>"` takes and `OpenMenu.menu` carries.
    pub name: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    /// Other names (lowercase).
    pub aliases: &'static [&'static str],
    pub kind: MenuKind,
    /// The title row's home mark.
    pub home: &'static str,
    pub doc: &'static str,
    pub power: Option<Power>,
    /// Left tabs that come from a feed (DMX Protocols: the show's networks);
    /// the menu's own tabs are then the strip on top (Lines · Routing ·
    /// Discovery). None = the tabs are the left tabs.
    pub left: Option<Left>,
    /// The claim area it edits (Devices › DMX › Patch, decisions §34 "34g4 patch hardened" 9):
    /// the crumb, the kinds, the tree sidebar, the claim chip, Exit and the live columns.
    pub area: Option<&'static Area>,
    pub tabs: &'static [Tab],
}

/// A menu over a claim area with a draft (the Patch dialog): what the frame adds to the plain
/// setup menu. Every field it reads is in `feed`.
pub struct Area {
    /// The crumb before the menu's label (`Devices`, `DMX`).
    pub trail: &'static [&'static str],
    /// The rail of the area's kinds: (label, the menu it opens); this menu is the one on.
    pub kinds: &'static [(&'static str, &'static str)],
    /// The rail's label and the slot for kinds that come later.
    pub kinds_label: &'static str,
    pub later: &'static str,
    /// The feed with `mode` (`draft` for the holder, else `live`), `holder` and the tree's rows.
    pub feed: &'static str,
    /// The sidebar's groupings (the first is the default).
    pub tree: &'static [TreeMode],
    /// The claim button (decisions §34 "34g4 patch entry hardened"): a hold on Live runs `claim`,
    /// a press on Claimed runs `exit` (Save / Discard asked when changed); the ✕ while claimed
    /// runs `close` (the same question, then the dialog closes: the answer runs the rest).
    pub claim: &'static str,
    pub exit: &'static str,
    pub close: &'static str,
    /// The coloured bar under the strip (decisions §34 round 3, 10): what Live, your draft and
    /// someone else's draft let you do — (title, text) each; `{holder}` names the holder.
    pub bars: [(&'static str, &'static str); 3],
    /// The tree's add bar (`+ Layer`, `+ Group`): (label, line); `{value}` is the name asked;
    /// greyed in Live.
    pub adds: &'static [(&'static str, &'static str)],
    /// What the tree's counts count (`fixtures`).
    pub unit: &'static str,
}

/// One grouping of the sidebar: its nodes are `rows` of the feed (`guid`, `name`, `parent`,
/// `count`); a grid row is under a node when its `filter` field is that node (or one under it,
/// `nested`); `list` = the field is a list of GUIDs.
pub struct TreeMode {
    pub id: &'static str,
    pub label: &'static str,
    pub rows: &'static str,
    pub filter: &'static str,
    pub list: bool,
    pub nested: bool,
    /// The grid rows with none (`No MVR group`).
    pub none: &'static str,
    /// Direct / Deep (a tag's own members or the tags under it too).
    pub deep: bool,
    /// Its nodes come from the area's drawing, never made here (classes).
    pub read_only: bool,
}

/// Left tabs from a feed's rows (decisions §30 "12d": DMX Protocols' left =
/// the show's networks, a stripe for what is really sent).
pub struct Left {
    pub feed: &'static str,
    pub rows: &'static str,
    /// The row field a tab is named by (and grids filter on).
    pub key: &'static str,
    /// The row field under the name (`4 out · 1 in`).
    pub sub: &'static str,
    /// The row field with the stripe: `off`, `out`, `in`, `both`.
    pub stripe: &'static str,
    /// Shown when the feed has no rows.
    pub empty: &'static str,
}

/// The big power toggle (decisions §30 "12d": look A).
pub struct Power {
    pub label: &'static str,
    /// Where its state is: a feed and a field of it.
    pub feed: &'static str,
    pub path: &'static str,
    /// On is when the field is false (`output_held`).
    pub invert: bool,
    /// A word field: on unless it is one of these (`station.status`
    /// Standalone); empty = the field is a bool.
    pub off_values: &'static [&'static str],
    pub on: Look,
    pub off: Look,
    pub failing: Look,
    pub rights: Rights,
}

pub struct Look {
    pub word: &'static str,
    pub press: &'static str,
    /// The line a press runs; `{a.b}` names a field of the power's feed
    /// (`Join LumaSession "{station.lumasession}"`).
    pub line: &'static str,
}

pub struct Tab {
    pub name: &'static str,
    pub label: &'static str,
    pub sub: &'static str,
    /// `station`, `show` or "" (mixed: the cells say).
    pub home: &'static str,
    /// Action ids a phone shows first in the slab. (`home` above: `station`, `show` or `user`,
    /// decisions §39 — the cells whose home differs show it gold — or the menu's own words.)
    pub phone: &'static [&'static str],
    pub content: Content,
    pub actions: &'static [Action],
    /// Up to 4 setting cells in the function block.
    pub bound: &'static [Bound],
}

pub enum Content {
    Grid(&'static Grid),
    /// A grid with the drives rail (decisions §38 5): its feed takes the drive the rail picked.
    Drives(&'static Grid),
    /// Settings cells of `EditSetting <target>`.
    Settings(&'static str),
    /// Drawn by the page (a note says what comes).
    Custom(&'static str),
    Empty(&'static str),
}

pub struct Grid {
    pub key: &'static str,
    /// The object the lines name; None = rows that aren't objects (found
    /// nodes): no line edits them.
    pub object: Option<&'static ObjectDecl>,
    /// The object found at run time instead, when the rows' objects are declared by crates
    /// above the menu's (DMX lines: each protocol's own crate, decisions §34 "34f7c hardened"
    /// 9); its declared props give the columns.
    pub object_of: Option<fn() -> Option<&'static ObjectDecl>>,
    /// A row field naming the row's own object word, when rows mix
    /// objects (DMX lines: Sacn and ArtNet).
    pub row_object: Option<&'static str>,
    /// The row field naming the row in a line (`no`, `name`).
    pub id: &'static str,
    /// The row field the selection keys on (a GUID, or the name of a
    /// hardware row).
    pub row_key: &'static str,
    pub feed: &'static str,
    pub rows: &'static str,
    pub cards: bool,
    pub new_line: Option<&'static str>,
    /// Rows come from the hardware: no new-object line; New / Copy / Cut /
    /// Paste / Delete / Import greyed with this reason.
    pub hardware: Option<&'static str>,
    /// The row field matched against the selected left tab (`network`); a
    /// row whose field is `*` shows under every tab.
    pub left_filter: Option<&'static str>,
    /// The New cell's line (default `Store {object}`); `{left}` = the
    /// selected left tab, `{value}` asks.
    pub store: Option<&'static str>,
    /// A row field that marks a locked row (UserProfile Default) and why:
    /// with one selected, Cut / Delete / Edit and the tab's row actions are
    /// greyed with the reason.
    pub locked: Option<(&'static str, &'static str)>,
    /// A feed field naming the rows' owner: lines name it after the object
    /// word (`Delete Version "demo" "20261013-140203"`; History shows any
    /// show's saves).
    pub owner: Option<&'static str>,
    /// A second view of the same rows (History: List | Matrix — a column
    /// per object kind); empty = none.
    pub matrix: &'static [Col],
    /// A second view: the DMX map (Universes: List | Map, decisions §34 round 3, 12) — the
    /// feed's `runs` and `conflicts` drawn by `<luma-dmxmap>`.
    pub dmx_map: bool,
    /// New runs `store` with this level although the object takes no
    /// Store (Library: `NewShow`, Admin).
    pub new_rights: Option<Rights>,
    /// A row opens its detail (›, decisions §30 "Users & Profiles follows
    /// storage"): the object's parts where they are stored.
    pub detail: Option<&'static Detail>,
    pub columns: &'static [Col],
}

impl Grid {
    /// The object the lines name: the declared one, else the one found at run time.
    pub fn object_decl(&self) -> Option<&'static ObjectDecl> {
        self.object.or_else(|| self.object_of.and_then(|f| f()))
    }
}

/// A row's detail: `‹ <grid> › <name>`, its groups (tabs; one group = no
/// group level) and their parts. `{no}` / `{name}` in the templates are
/// the row's.
pub struct Detail {
    /// The words a part's lines start with (`UserProfile {no}`).
    pub owner: &'static str,
    /// Where it is stored (`ShowData.UserProfiles.{no}`).
    pub path: &'static str,
    /// The settings fields of the object (`EditSetting UserProfile {no}`).
    pub edit: &'static str,
    /// The `detail` feed's kind (`profile`, `user`).
    pub feed: &'static str,
    pub groups: &'static [DetailGroup],
}

pub struct DetailGroup {
    pub name: &'static str,
    pub label: &'static str,
    /// Below the object's path (`Settings`); "" = the object itself.
    pub path: &'static str,
    /// What a phone's tile says it holds.
    pub sub: &'static str,
    pub parts: &'static [Part],
}

pub struct Part {
    pub name: &'static str,
    pub label: &'static str,
    /// Below the group's path (`Preferences`).
    pub path: &'static str,
    pub content: PartContent,
    /// Buttons above the content: (label, line template).
    pub actions: &'static [(&'static str, &'static str)],
}

pub enum PartContent {
    /// The object's settings fields of one section ("" = all).
    Settings(&'static str),
    /// Drawn by the page: `encoder_bars`, `attributes`.
    Custom(&'static str),
    /// A read-only list from the `detail` feed: (rows field, row key, columns).
    List(&'static str, &'static str, &'static [Col]),
}

/// A grid column. `prop` names the object's declared property (its kind,
/// range and level come from there); `own` gives the kind of a property an
/// object sets in its own handler (User Rights) or of a read-only column.
pub struct Col {
    pub id: &'static str,
    pub label: &'static str,
    pub field: &'static str,
    pub prop: Option<&'static str>,
    pub own: Option<Own>,
    pub width: u32,
    /// 1 Condensed, 2 Full, 3 Extended; a phone folds the highest first.
    pub priority: u8,
    pub frozen: bool,
    pub section: &'static str,
    pub unit: &'static str,
    /// A value of one row only (Name): read-only with several rows.
    pub per_row: bool,
    /// The row field holding the cell's dot (`ok`, `warn`, `alert`, `off`).
    pub tone: &'static str,
    /// `networks`: the page makes one column per row of the feed's
    /// `networks` (`Network "Main" Interface` on a station's row).
    pub expand: &'static str,
    /// The row field that says what a selected cell holds (the History
    /// matrix: the objects that changed in that save).
    pub detail: &'static str,
    /// It changes only in a claim area's draft (Fixture type, Mode, MVR group, the FID): Live
    /// locks it with the reason, header and cells (decisions §34 round 3, 10).
    pub draft: bool,
}

/// A kind declared by the column (no declared prop to read it from).
#[derive(Clone, Copy)]

pub enum Own {
    Text,
    Bool(&'static str, &'static str),
    List(&'static [&'static str]),
}

/// A History matrix column: the save's `k_<kind>` cell, its tone and the
/// objects behind it. All priority 1: a phone scrolls them sideways beside
/// the frozen Saved column.
pub struct Action {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    /// `""`, `load` (gold primary), `save`, `info`, `history`, `neutral`,
    /// `ok`, `warn`, `alert`.
    pub tone: &'static str,
    pub line: &'static str,
    pub rights: Rights,
    /// `selection` (one or more rows), `one` (exactly one) or "".
    pub needs: &'static str,
    /// The row field naming whose row it is: on your own row the action
    /// needs no level (your own password).
    pub mine: &'static str,
    /// It changes more than live columns: a claim area's Live greys it, saying why (§34 round 3,
    /// 18: Insert fixtures…, Multipatch…).
    pub claim: bool,
    /// The value dialog kind its `{value}` opens (`address`, `copies` — new copies at an address —,
    /// `number`), or a view cell the host draws (`sort`, `offset`; `packed`: the Offset joins the
    /// line, `insert`: the Insert fixtures dialog); "" asks plainly.
    pub value: &'static str,
    /// Its label in a narrow slab cell (a dialog under about 1000 px, the phone's bar; decisions
    /// §37 3): "Insert…" for "Insert fixtures…"; "" wraps the label to two lines instead.
    pub short: &'static str,
    /// When the cell shows: "" always, `review` only while a claim area's grid is in its review
    /// mode (an MVR import, decisions §34 "Import MVR hardened" 3) — the others hide then.
    pub when: &'static str,
}

pub struct Bound {
    pub object: &'static str,
    pub prop: &'static str,
    pub label: &'static str,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl MenuDef {
    /// The tab `name` names (its name or label, any case).
    pub fn tab(&self, name: &str) -> Option<&'static Tab> {
        let name = name.trim_matches('"');
        self.tabs
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(name) || t.label.eq_ignore_ascii_case(name) || t.label.replace(' ', "").eq_ignore_ascii_case(name))
    }

    /// The grid keys of its tabs.
    pub fn grid_keys(&self) -> impl Iterator<Item = &'static str> {
        self.tabs.iter().filter_map(|t| match t.content {
            Content::Grid(g) => Some(g.key),
            _ => None,
        })
    }
}
