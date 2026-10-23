// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests
//! Tests of `catalog`: freeze checks the tables, the lookups read them.
//!
//! Owns:          the unit tests of catalog.rs
//! Doesn't own:   adding rows (registry/tests.rs)
//! Entry points:  the #[test] fns
//! Decisions:     §26, §34 21

use lumacore_base::ErrorInfo;

use crate::{FeedDecl, FeedKind, FlagDef, FreezeError, KeyDecl, Registry, Status, ToolDecl, WindowType};

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

const A: &[ErrorInfo] = &[ErrorInfo { code: "E-TEST-001", title: "One", explain: "The first." }];
const B: &[ErrorInfo] = &[ErrorInfo { code: "E-TEST-002", title: "Two", explain: "The second." }];
const TWICE: &[ErrorInfo] = &[ErrorInfo { code: "E-TEST-001", title: "Again", explain: "A copy." }];

#[test]
fn freeze_finds_codes_across_catalogs() {
    let mut r = Registry::new();
    r.errors(A).errors(B);
    let catalog = r.freeze().unwrap();
    assert_eq!(catalog.error("e-test-2").map(|i| i.title), Some("Two"));
    assert_eq!(catalog.errors().count(), 2);
}

#[test]
fn freeze_refuses_a_code_two_catalogs_declare() {
    let mut r = Registry::new();
    r.errors(A).errors(TWICE);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a duplicate code froze") };
    assert!(wrong[0].contains("duplicate code 'E-TEST-001'"), "{wrong:?}");
}

const FLAGS: &[FlagDef] = &[FlagDef::new("NoOops", "No step").alias(&["NU"]), FlagDef::new("Verbose", "Say more")];
const CLASH: &[FlagDef] = &[FlagDef::new("nu", "Clashes with /NU")];

#[test]
fn global_flags_are_found_by_name_or_short_form() {
    let mut r = Registry::new();
    r.global_flags(FLAGS);
    let catalog = r.freeze().unwrap();
    assert_eq!(catalog.global_flag("nu").map(|f| f.name), Some("NoOops"));
    assert_eq!(catalog.global_flags().count(), 2);
}

#[test]
fn freeze_refuses_two_global_flags_of_one_name() {
    let mut r = Registry::new();
    r.global_flags(FLAGS).global_flags(CLASH);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a clash froze") };
    assert!(wrong[0].contains("/nu"), "{wrong:?}");
}

const FEED: FeedDecl = FeedDecl { name: "health", kind: FeedKind::State, open: false, params: &[], period_ms: 250, doc: "Health." };
const FEEDS: &[FeedDecl] = &[FEED];

#[test]
fn freeze_refuses_a_feed_declared_twice() {
    let mut r = Registry::new();
    r.feeds(FEEDS);
    assert_eq!(r.freeze().unwrap().feed("health").map(|f| f.period_ms), Some(250));
    let mut r = Registry::new();
    r.feeds(FEEDS).feeds(FEEDS);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a feed declared twice froze") };
    assert!(wrong[0].contains("feed health"), "{wrong:?}");
}

#[test]
fn freeze_refuses_a_window_type_number_or_name_twice() {
    let mut r = Registry::new();
    r.window_types([WindowType::new(1, "terminal", "Terminal", "Common", "terminal", "A shell.")]);
    assert_eq!(r.freeze().unwrap().window_type("terminal").map(|t| t.no), Some(1));
    let mut r = Registry::new();
    r.window_types([
        WindowType::new(1, "terminal", "Terminal", "Common", "terminal", "A shell."),
        WindowType::new(1, "groups", "Groups", "Pools", "grid", "Groups."),
    ]);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a taken number froze") };
    assert!(wrong[0].contains("number 1 is taken"), "{wrong:?}");
}

const TOOLS: &[ToolDecl] = &[ToolDecl { id: "Flip", label: "Flip", icon: "flip", group: "Transform", ask: None, status: Status::Ready }];

#[test]
fn freeze_refuses_a_tool_outside_the_toolbar_groups() {
    let mut r = Registry::new();
    r.tools(&["Transform"], TOOLS);
    assert_eq!(r.freeze().unwrap().tool("flip").map(|t| t.label), Some("Flip"));
    let mut r = Registry::new();
    r.tools(&["Order"], TOOLS);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a tool outside the groups froze") };
    assert!(wrong[0].contains("isn't a toolbar group"), "{wrong:?}");
}

#[test]
fn freeze_refuses_a_ui_key_twice() {
    let key = || KeyDecl { id: "menu".into(), label: "Menu".into(), ..KeyDecl::default() };
    let mut r = Registry::new();
    r.keys([key(), key()]);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a key twice froze") };
    assert!(wrong[0].contains("UI key menu"), "{wrong:?}");
}

static GO: crate::command::CommandDecl = crate::command::CommandDecl { verb: "go", aliases: &["g"], ..crate::command::CommandDecl::BASE };
static GOTO: crate::command::CommandDecl = crate::command::CommandDecl { verb: "goto", aliases: &["G"], ..crate::command::CommandDecl::BASE };

#[test]
fn freeze_refuses_two_commands_of_one_word() {
    let mut r = Registry::new();
    r.commands([&GO]);
    assert_eq!(r.freeze().unwrap().command("G").map(|c| c.verb), Some("go"));
    let mut r = Registry::new();
    r.commands([&GO, &GOTO]);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("one word for two commands froze") };
    assert!(wrong[0].contains("command word G"), "{wrong:?}");
}

static GROUP: crate::object::ObjectDecl = crate::object::ObjectDecl {
    keyword: "Group",
    lum: crate::object::LumShape::ONE,
    default_verb: crate::Verb::Call,
    aliases: &["go"],
    doc: crate::command::Doc::BASE,
    family: None,
    call_undo: crate::command::Undo::ReadOnly,
    call_class: crate::command::CommandClass::Query,
    scope: |_, _| crate::command::Scope::Client,
    reach: |_| crate::command::Reach::Show,
    rights: |_, _| crate::command::Rights::Guest,
    flags: &[],
    planned: &[],
};

#[test]
fn freeze_refuses_a_word_that_is_a_command_and_an_object() {
    let mut r = Registry::new();
    r.objects([&GROUP]);
    assert_eq!(r.freeze().unwrap().object("\"group\"").map(|o| o.keyword), Some("Group"));
    let mut r = Registry::new();
    r.commands([&GO]).objects([&GROUP]);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a word of two meanings froze") };
    assert!(wrong[0].contains("word go"), "{wrong:?}");
}

static GRID: crate::menu::Grid = crate::menu::Grid {
    key: "groups",
    object: Some(&GROUP),
    object_of: None,
    row_object: None,
    id: "no",
    row_key: "guid",
    feed: "",
    rows: "",
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
    columns: &[],
};
static TABS: [crate::menu::Tab; 1] = [crate::menu::Tab {
    name: "Groups",
    label: "Groups",
    sub: "",
    home: "",
    phone: &[],
    content: crate::menu::Content::Grid(&GRID),
    actions: &[],
    bound: &[],
}];
static MENUS: [crate::menu::MenuDef; 1] = [crate::menu::MenuDef {
    name: "Groups",
    label: "Groups",
    icon: "",
    aliases: &[],
    kind: crate::menu::MenuKind::Setup,
    home: "",
    doc: "",
    power: None,
    left: None,
    area: None,
    tabs: &TABS,
}];

#[test]
fn freeze_refuses_a_grid_whose_object_isnt_in_the_catalog() {
    let mut r = Registry::new();
    r.objects([&GROUP]).menus(&MENUS);
    assert_eq!(r.freeze().unwrap().menu("groups").map(|m| m.tabs.len()), Some(1));
    let mut r = Registry::new();
    r.menus(&MENUS);
    let Err(FreezeError(wrong)) = r.freeze() else { panic!("a grid of a missing object froze") };
    assert!(wrong[0].contains("isn't in the catalog"), "{wrong:?}");
}
