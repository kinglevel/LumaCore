//! System, session and listable objects: the things `List` / `Info` / `Dump`
//! show that aren't owned by a feature module (Server, Station, Network,
//! Clock, Settings, Undo, …).

use crate::commands::flags as F;
use super::{Doc, ObjectDef, Undo, Verb, out};
use crate::commands::registry::CommandClass;
use crate::AppState;
use crate::commands::media;
use crate::commands::profiles;
use crate::commands::help::HelpCategory;
use crate::commands::registry::AliasDepth;
use crate::commands::show;
use crate::commands::{info, list};
use crate::commands::registry::CommandFamily;
use crate::DispatchResult;


/// A system object: its verbs and handlers (`List => |s, a| …`), optionally
/// callable (`call: handler`) and with full help (`extra:`).
#[macro_export]
macro_rules! listed_object_inner {
    // `station SERVER, …`: the object belongs to this station (its `Set`
    // runs here, never on the master — decisions §10).
    ($lum:expr, $default:ident, $props:expr; station $($rest:tt)*) => {
        $crate::listed_object_inner!($lum, $default, $props; @scope $crate::objects::on_station, $crate::objects::show_reach; $($rest)*);
    };
    // Files on disk (the library): never swept (decisions §25).
    ($lum:expr, $default:ident, $props:expr; drive station $($rest:tt)*) => {
        $crate::listed_object_inner!($lum, $default, $props; @scope $crate::objects::on_station, $crate::objects::drive_reach; $($rest)*);
    };
    ($lum:expr, $default:ident, $props:expr; @scope $scope:path, $reach:path; $static:ident, $label:literal, [$($alias:literal),*], $desc:literal, [$($verb:ident => $h:expr),+ $(,)?]
     $(, call: $call:path)?) => {
        $crate::listed_object_inner!($lum, $default, $props; @scope $scope, $reach; $static, $label, [$($alias),*], $desc, [$($verb => $h),+] $(, call: $call)?,
            extra: ($crate::commands::registry::Doc::new($label, $desc), None, $crate::commands::registry::Undo::ReadOnly, $crate::commands::registry::CommandClass::Commit));
    };
    ($lum:expr, $default:ident, $props:expr; @scope $scope:path, $reach:path; $static:ident, $label:literal, [$($alias:literal),*], $desc:literal, [$($verb:ident => $h:expr),+ $(,)?]
     $(, call: $call:path)?, extra: ($doc:expr, $family:expr, $undo:expr, $class:expr)) => {
        $crate::listed_object_inner!($lum, $default, $props; @rights $crate::rights::setup, $scope, $reach; $static, $label, [$($alias),*], $desc, [$($verb => $h),+] $(, call: $call)?,
            extra: ($doc, $family, $undo, $class));
    };
    // An object with its own levels (`@rights station_rights, …`).
    ($lum:expr, $default:ident, $props:expr; @rights $rights:path, $scope:path, $reach:path; $static:ident, $label:literal, [$($alias:literal),*], $desc:literal, [$($verb:ident => $h:expr),+ $(,)?]
     $(, call: $call:path)?) => {
        $crate::listed_object_inner!($lum, $default, $props; @rights $rights, $scope, $reach; $static, $label, [$($alias),*], $desc, [$($verb => $h),+] $(, call: $call)?,
            extra: ($crate::commands::registry::Doc::new($label, $desc), None, $crate::commands::registry::Undo::ReadOnly, $crate::commands::registry::CommandClass::Commit));
    };
    ($lum:expr, $default:ident, $props:expr; @rights $rights:path, $scope:path, $reach:path; $static:ident, $label:literal, [$($alias:literal),*], $desc:literal, [$($verb:ident => $h:expr),+ $(,)?]
     $(, call: $call:path)?, extra: ($doc:expr, $family:expr, $undo:expr, $class:expr)) => {
        $crate::listed_object_inner!($lum, $default, $props; @ask None; @rights $rights, $scope, $reach; $static, $label, [$($alias),*], $desc, [$($verb => $h),+] $(, call: $call)?,
            extra: ($doc, $family, $undo, $class));
    };
    // An object with its own Delete question (`@ask Some(f); @rights …`).
    ($lum:expr, $default:ident, $props:expr; @ask $ask:expr; @rights $rights:path, $scope:path, $reach:path; $static:ident, $label:literal, [$($alias:literal),*], $desc:literal, [$($verb:ident => $h:expr),+ $(,)?]
     $(, call: $call:path)?) => {
        $crate::listed_object_inner!($lum, $default, $props; @ask $ask; @rights $rights, $scope, $reach; $static, $label, [$($alias),*], $desc, [$($verb => $h),+] $(, call: $call)?,
            extra: ($crate::commands::registry::Doc::new($label, $desc), None, $crate::commands::registry::Undo::ReadOnly, $crate::commands::registry::CommandClass::Commit));
    };
    ($lum:expr, $default:ident, $props:expr; @ask $ask:expr; @rights $rights:path, $scope:path, $reach:path; $static:ident, $label:literal, [$($alias:literal),*], $desc:literal, [$($verb:ident => $h:expr),+ $(,)?]
     $(, call: $call:path)?, extra: ($doc:expr, $family:expr, $undo:expr, $class:expr)) => {
        pub static $static: ObjectDef = ObjectDef {
            decl: $crate::objects::ObjectDecl {
                keyword: $label,
                lum: $lum,
                default_verb: $crate::objects::Verb::$default,
                flags: &[],
                aliases: &[$($alias),*],
                doc: $doc,
                family: $family,
                call_undo: $undo,
                call_class: $class,
                scope: $scope,
                reach: $reach,
                rights: $rights,
                planned: &[],
            },
            pool: None,
            verbs: &[$(($crate::objects::Verb::Call, $call),)? $(($crate::objects::Verb::$verb, $h)),+],
            delete_ask: $ask,
            complete: None,
            props: $props,
            tree: None,
        };
    };
    ($lum:expr, $default:ident, $props:expr; $($rest:tt)*) => {
        $crate::listed_object_inner!($lum, $default, $props; @scope $crate::objects::on_master, $crate::objects::show_reach; $($rest)*);
    };
}

/// `listed_object!(lum <shape>, default <Verb>, …)`: its Lum shape (one of a kind without
/// `lum`) and its default verb (decisions §33 40).
#[macro_export]
macro_rules! listed_object {
    // `props X,`: its properties (decisions §33 58).
    (props $props:expr, lum $lum:expr, default $default:ident, $($rest:tt)*) => {
        $crate::listed_object_inner!($lum, $default, Some($props); $($rest)*);
    };
    (props $props:expr, default $default:ident, $($rest:tt)*) => {
        $crate::listed_object_inner!($crate::objects::LumShape::ONE, $default, Some($props); $($rest)*);
    };
    (lum $lum:expr, default $default:ident, $($rest:tt)*) => {
        $crate::listed_object_inner!($lum, $default, None; $($rest)*);
    };
    (default $default:ident, $($rest:tt)*) => {
        $crate::listed_object_inner!($crate::objects::LumShape::ONE, $default, None; $($rest)*);
    };
}

listed_object!(
default Dump,
PROGRAMMER,
"Programmer",
[],
"Your programmer (selection + values)",
[
    Dump => |s, a| out(show::dump_programmer(s, a)),
]);
listed_object!(
default List,
    SELECTION,
    "Selection",
    ["sel"],
    "The current fixture selection",
    [
        List => |s, a| out(list::selection(s, a)),
        On => |s, _| out(crate::commands::control::knock_in(s, &[])),
        Off => |s, _| out(crate::commands::control::knock_out(s, &[])),
    ],
    extra: (Doc {
        display: "Selection",
        category: HelpCategory::Programmer,
        also_applies_to: &[],
        syntax: "list selection | watch selection",
        description: "The current set of fixtures targeted by commands. \
                      Created by Select, Fixture, or Group commands. \
                      Persists between commands — `At Full` applies to the last selection. \
                      Watchable: `watch selection` shows selected fixtures live.",
        examples: &[
            "list selection     -- show current selection",
            "list sel           -- shorthand",
            "watch selection    -- live selection view",
        ],
        related: &["select", "fixture", "group"],
        alias_depth: AliasDepth::Base,
        hidden: false,
    }, Some(CommandFamily::Selection), Undo::ReadOnly, CommandClass::Commit));
listed_object!(
lum super::LumShape::ids(1),
default Info,
    station ERROR,
    "Error",
    ["errors"],
    "Error codes and what they mean",
    [
        List => |_, a| out(crate::errors::list(a)),
        Info => |_, a| out(crate::errors::info(a)),
    ],
    extra: (Doc {
        display: "Error",
        category: HelpCategory::System,
        also_applies_to: &[],
        syntax: "List Error [area] | Info Error <code>",
        description: "Every error has a stable code (E-AREA-NNN) shown after its message. \
                      `Info Error <code>` explains it; `List Error` lists all codes, \
                      `List Error Patch` those of one area.",
        examples: &[
            "info error E-PATCH-002   -- what an invalid DMX address means",
            "list error               -- every code",
            "list error cmd           -- command-line errors",
        ],
        related: &["info", "list"],
        alias_depth: AliasDepth::Base,
        hidden: false,
    }, None, Undo::ReadOnly, CommandClass::Commit)
);
listed_object!(
lum super::LumShape::ids(1),
default Info,
    REFERENCE,
    "Reference",
    ["references"],
    "The reference check: links that point at nothing",
    [
        List => |s, a| out(crate::references::list(s, a)),
    ],
    extra: (Doc {
        display: "Reference",
        category: HelpCategory::System,
        also_applies_to: &[],
        syntax: "List Reference [/Broken]",
        description: "Every link between objects is a GUID (decisions §21 \"6d\"). After a \
                      load, join, merge or undo the show is checked: a link to a missing \
                      object is kept (a later merge or undo can heal it) and listed here, \
                      as is anything that contains itself. `Info Show` gives the count.",
        examples: &[
            "list reference           -- check the show now",
            "list reference /broken   -- only the problems",
        ],
        related: &["info", "show"],
        alias_depth: AliasDepth::Base,
        hidden: false,
    }, None, Undo::ReadOnly, CommandClass::Commit)
);
listed_object!(
default List,
SHOW_DATA,
"ShowData",
[],
"All show data",
[
    List => |s, a| out(list::tree(s, crate::tree_path::Node::ShowData, a)),
    Dump => |s, a| out(show::dump_showdata(s, a)),
]);
listed_object!(
default List,
station SETTINGS,
"Settings",
["config"],
"Your settings",
[
    List => |s, a| out(list::settings(s, a)),
]);
listed_object!(
default List,
CLAIMS,
"Claims",
["claimed"],
"Claimed objects",
[
    List => |s, a| out(list::claims(s, a)),
]);
listed_object!(
default List,
UNDO,
"Undo",
[],
"The Oops list: List Undo [/<category>] [/ThisStation]",
[
    List => |s, a| out(list::undo(s, a)),
]);

fn global_var_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(list::tree(state, crate::tree_path::Node::GlobalVariables, args))
}

pub static GLOBAL_VAR: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "GlobalVar",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::List,
        flags: &[],
        aliases: &["globalvars", "globalvariables"],
        doc: crate::commands::registry::Doc::new("GlobalVar", "Runtime global variables"),
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: super::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::anyone,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::List, global_var_list)],
    delete_ask: None,
    complete: None,
    props: None,
    tree: None,
};

// ── Objects with a tree collection ───────────────────────────────────────────

use crate::vfs::{DynChild, NodeView, btree_to_children, object_view};
use lumacore_tree::Root;

fn profile_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(list::profiles(state, args))
}
fn profile_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(info::info_userprofile(state, args.first().copied()))
}
fn profile_dump(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(show::dump_profiles(state, args))
}
fn profiles_children(root: &Root) -> Vec<DynChild> {
    btree_to_children(&root.show_data.user_profiles)
}
fn profile_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.user_profiles.get(&id).map(|t| object_view(id, t))
}

/// Deleting profiles: one is asked by its handler (in use → refused, the
/// Default template locked); several get dispatch's one ask.
static PROFILE_ROWS: crate::commands::set_rows::RowDelete = crate::commands::set_rows::RowDelete {
    keyword: "UserProfile",
    noun: "profile",
    numbers: |s| s.root.show_data.user_profiles.keys().copied().collect(),
    refused: |_, _| None,
    affects: |_, _| None,
    own_ask: true,
};

pub static USER_PROFILE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "UserProfile",
        lum: crate::objects::LumShape::ids(1).children(&[
            &crate::commands::media::ICON.decl,
            &crate::commands::media::IMAGE.decl,
            &crate::commands::media::THEME.decl,
            &crate::commands::media::LOCK_SCREEN.decl,
            &crate::commands::window_presets::WINDOW_PRESET.decl,
            &crate::commands::attributes::ATTRIBUTE.decl,
            &crate::commands::attributes::FEATURE_GROUP.decl,
        ]),
        default_verb: crate::objects::Verb::Info,
        flags: &[crate::commands::profiles::COPY_OF.on(&[Verb::Store]), F::OVERWRITE.on(&[Verb::Import]), crate::commands::media::AS.on(&[Verb::Import]), F::DRIVE.on(&[Verb::Import, Verb::Export])],
        aliases: &["userprofiles", "profile", "profiles"],
        doc: crate::commands::registry::Doc {
            display: "UserProfile",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "List|Info UserProfile [<n>] | Store UserProfile [\"name\" [/CopyOf n]] | Copy UserProfile n At m|\"name\" | Label UserProfile n [\"name\"] | Delete UserProfile n [/NoConfirmation] | Set UserProfile [<n>|\"name\"] Theme.…|LockScreen.… <value> | \"Export\" UserProfile [\"name\"] | \"Import\" UserProfile [\"file\" [/As \"Name\" ] [/Overwrite]] | <verb> UserProfile [<n>|\"name\"] Image|Icon|WindowPreset … | Reset UserProfile [<n>|\"name\"] Window /Type <type>",
            description: "How users work: views, screen configs, keys, settings — and the look, which \
                          travels with it (decisions §30 \"12a2\"): Theme (preset, adjusters, palette), \
                          LockScreen (picture, clock, text, auto-lock) and its own Icons and Images. \
                          No rights: those are the user's. UserProfile \"Default\" is the locked template \
                          (decisions §30 \"12c2\"): every new profile starts as a copy of it (never blank), \
                          it can't be changed, renamed or deleted and nobody works in it. Store / Copy / \
                          Label / Delete / Import a profile need Admin; a bare Store, Label or Import asks, \
                          Delete always asks and is refused while a user works in the profile. Export / \
                          Import moves a profile with its pictures to another show (/As = a copy with new \
                          GUIDs). A bare UserProfile is your own; its users and Admins may change it.",
            examples: &[
                "Store UserProfile                    -- asks: name, from Default / copy of… / import file…",
                "Store UserProfile \"Stage\" /CopyOf \"Bob\"",
                "Copy UserProfile \"Bob\" At \"Stage 2\"",
                "Delete UserProfile \"Stage 2\"        -- asks",
                "Import UserProfile \"Stage\" /As \"Stage 2\"",
                "Set UserProfile Theme \"Base\" Night",
                "Set UserProfile Theme Paint \"Accent\" \"#9a7a08\"",
                "Reset UserProfile \"Bob\" Theme",
                "Import UserProfile Image \"bg.jpg\"",
                "Set UserProfile LockScreen \"Image\" 1",
                "Export UserProfile \"Bob\"",
                "Import UserProfile \"Bob\" /Overwrite",
                "UserProfile \"Bob\" WindowPreset 1  -- load Bob's preset 1 into this window",
                "Reset UserProfile \"Bob\" Window /Type DMX -- every DMX window of the profile (asks)",
            ],
            related: &["user", "image", "icon", "windowpreset"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: profile_scope,
        reach: profile_reach,
        rights: crate::commands::profiles::rights,
        planned: &[],
    },
    pool: None,
    verbs: &[
        // `UserProfile "Bob" WindowPreset 2` = Call WindowPreset 2 of that profile.
        (Verb::Call, |s, a| profile_presets(s, Verb::Call, a, profile_list)),
        (Verb::Store, |s, a| profile_presets(s, Verb::Store, a, |s, a| out(profiles::store(s, a)))),
        (Verb::List, |s, a| profile_presets(s, Verb::List, a, |s, a| media::profile_verb(s, Verb::List, a, profile_list))),
        (Verb::Info, |s, a| profile_presets(s, Verb::Info, a, |s, a| media::profile_verb(s, Verb::Info, a, profile_info))),
        (Verb::Dump, profile_dump),
        (Verb::Import, |s, a| profile_presets(s, Verb::Import, a, |s, a| media::profile_verb(s, Verb::Import, a, profile_list))),
        (Verb::Export, |s, a| profile_presets(s, Verb::Export, a, |s, a| media::profile_verb(s, Verb::Export, a, profile_list))),
        (Verb::Delete, |s, a| {
            // Several profiles: one ask for all; one keeps its own ask.
            crate::commands::set_rows::delete_rows(s, &PROFILE_ROWS, a, |s, a| {
                profile_presets(s, Verb::Delete, a, |s, a| media::profile_verb(s, Verb::Delete, a, |s, a| out(profiles::delete(s, a))))
            })
        }),
        (Verb::Label, |s, a| profile_presets(s, Verb::Label, a, |s, a| media::profile_verb(s, Verb::Label, a, |s, a| out(profiles::label(s, a))))),
        (Verb::Copy, |s, a| media::profile_verb(s, Verb::Copy, a, |s, a| out(profiles::copy(s, a)))),
        (Verb::Reset, |s, a| out(crate::commands::windows::cmd_reset_profile_windows(s, a))),
        // Owner first (decisions §30 "12f-a"): `Set|Move|Edit UserProfile [x] Attribute|FeatureGroup|AttributeRule|EncoderBar …`.
        (Verb::Move, |s, a| crate::commands::attributes::on_profile(s, Verb::Move, a)
            .unwrap_or_else(|| out(crate::errors::render_code("Move UserProfile [x] Attribute|FeatureGroup|AttributeRule|EncoderBar …", "E-CMD-006")))),
        (Verb::Edit, |s, a| crate::commands::attributes::on_profile(s, Verb::Edit, a)
            .unwrap_or_else(|| out(crate::errors::render_code("Edit UserProfile [x] Attribute|EncoderBar …", "E-CMD-006")))),
        // Your own grouping (decisions §33 86): Assign an attribute or a group (Sort / Split are
        // typed commands that read the owner themselves).
        (Verb::Assign, crate::commands::typed::typed::<crate::commands::attributes::AssignInProfile>),
    ],
    delete_ask: Some(|s, a| crate::commands::set_rows::delete_ask(s, &PROFILE_ROWS, a)),
    complete: None,
    props: Some(&crate::props::Props {
        lists: &[&media::PROFILE_PROPS],
        // `Set UserProfile "ClearBuffer" On` = your own (decisions §33 16).
        implicit: Some(|s| s.root.show_data.profile_id_for_user(s.effective_user_name()).map(|n| n.to_string()).unwrap_or_default()),
        apply: Some(media::profile_write),
        may: Some(media::profile_may),
        // Everything of a profile is its users' own (decisions §39: the User home).
        home: Some(crate::props::Home::User),
        // `UserProfile [x] Attribute|FeatureGroup …`: that profile's own grouping (§33 86).
        within: Some(crate::commands::attributes::profile_within),
        ..crate::props::Props::BASE
    }),
    tree: Some(super::TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: crate::tree_path::Node::UserProfiles,
        children: profiles_children,
        object: profile_view,
    }),
};

/// `<verb> UserProfile [<n>|"name"] WindowPreset …` (decisions §30 "12c",
/// owner first) goes to the profile's window presets; anything else to
/// `rest`.
fn profile_presets(
    state: &mut AppState,
    verb: Verb,
    args: &[&str],
    rest: fn(&mut AppState, &[&str]) -> DispatchResult,
) -> DispatchResult {
    crate::commands::attributes::on_profile(state, verb, args)
        .or_else(|| crate::commands::window_presets::on_profile(state, verb, args))
        .or_else(|| crate::commands::paint_slots::on_profile(state, verb, args))
        .unwrap_or_else(|| rest(state, args))
}

/// A profile's lines change the show on the master; opening its attribute
/// board (`Edit UserProfile Attribute`) stays on this station.
fn profile_scope(verb: Verb, args: &[&str]) -> crate::commands::registry::Scope {
    match verb {
        Verb::Edit => crate::commands::registry::Scope::Client,
        _ => super::on_master(verb, args),
    }
}

fn profile_reach(verb: Verb) -> crate::commands::registry::Reach {
    use crate::commands::registry::Reach;
    if matches!(verb, Verb::Import | Verb::Export) { Reach::Drive } else { Reach::Show }
}


fn window_types_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(list::tree(state, crate::tree_path::Node::WindowTypes, args))
}
fn window_types_children(root: &Root) -> Vec<DynChild> {
    root.resources
        .window_types
        .values()
        .filter(|t| !t.hidden)
        .map(|t| DynChild {
            display_name: format!(
                "{} \"{}\"  ({}, {} settings)",
                t.no,
                t.label,
                t.category,
                t.settings.len()
            ),
            id: t.no,
            node_type: "WindowType",
            is_leaf: true,
            child_count: 0,
            name: Some(t.label.clone()),
            guid: None,
        })
        .collect()
}
fn window_type_view(root: &Root, id: u32) -> Option<NodeView> {
    let t = root.resources.window_types.get(&id)?;
    Some(NodeView::Object {
        json: serde_json::to_value(t).unwrap_or(serde_json::Value::Null),
        node_type: "WindowType",
        id,
        guid: String::new(),
        name: Some(t.label.clone()),
    })
}

pub static WINDOW_TYPE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "WindowType",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::List,
        flags: &[],
        aliases: &["windowtypes"],
        doc: crate::commands::registry::Doc::new(
            "WindowType",
            "A kind of window (Terminal, Groups, …) and its settings",
        ),
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: super::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::anyone,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::List, window_types_list)],
    delete_ask: None,
    complete: None,
    props: None,
    tree: Some(super::TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: crate::tree_path::Node::WindowTypes,
        children: window_types_children,
        object: window_type_view,
    }),
};
