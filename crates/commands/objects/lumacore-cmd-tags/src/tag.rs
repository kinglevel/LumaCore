// ████████╗ █████╗  ██████╗
// ╚══██╔══╝██╔══██╗██╔════╝
//    ██║   ███████║██║  ███╗
//    ██║   ██╔══██║██║   ██║
//    ██║   ██║  ██║╚██████╔╝
//    ╚═╝   ╚═╝  ╚═╝ ╚═════╝
// FILE: tag

//! The Tag object (decisions §12 "Tags hardened"): its row, its tree node `Root.ShowData.Tags`
//! (the top tags; a tag's children are in its JSON), and the path row the one resolver walks it
//! by. The shared lookups every verb uses: the tags a target names, what carries them.
//!
//! Owns:          TAG, PATHS, the tree views, targeted, carriers, fixture_numbers, the flags.
//! Doesn't own:   the tree's operations (lumacore-tags), each verb's work (its file).
//! Entry points:  TAG, PATHS, targeted, carriers.
//! Decisions:     §12 "Tags hardened", lum.md (Path ids, /R).

use lumacore_objects::commands::flags::{FlagDef, RECURSIVE};
use lumacore_objects::commands::help::HelpCategory;
use lumacore_objects::commands::registry::{CommandClass, Doc, SELECTION};
use lumacore_objects::commands::typed::typed;
use lumacore_objects::objects::{LumShape, ObjectDecl, ObjectDef, PathDef, TreeDef, Verb, json_view};
use lumacore_objects::tree_path::Node;
use lumacore_objects::vfs::{DynChild, NodeView};
use lumacore_objects::AppState;
use lumacore_syntax::ast::Target;
use lumacore_tags::tag::{self as tree, Member};
use lumacore_tags::{Tag, TagPath};
use lumacore_tree::Root;
use uuid::Uuid;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

const FLAGS: &[FlagDef] = &[
    FlagDef::new("WithChildren", "Delete its child tags too").on(&[Verb::Delete]),
    FlagDef::new("KeepChildren", "Move its child tags up one level").on(&[Verb::Delete]),
    RECURSIVE.on(&[Verb::SelectFixtures, Verb::Info, Verb::On, Verb::Off, Verb::Toggle, Verb::Go, Verb::Goto, Verb::Pause, Verb::At]),
];

/// The objects that may follow a tag, owner first: another tag (`Tag "Lx1" Tag "Right"`).
static OWNED: [&ObjectDecl; 1] = [&TAG.decl];

/// `Tag` (decisions §12 "Tags hardened"): a node of the tag tree, its ids a path.
pub static TAG: ObjectDef = ObjectDef::new(ObjectDecl {
    keyword: "Tag",
    lum: LumShape::path().children(&OWNED),
    default_verb: Verb::SelectFixtures,
    aliases: &["tags"],
    doc: Doc {
        category: HelpCategory::Group,
        syntax: "Tag <path> [/R] | Store Tag [<path>] [\"name\"] | Label Tag <path> \"name\" | Delete Tag <path> [/WithChildren | /KeepChildren] | Move Tag <path> At Tag <path> | Root | <n> | Assign Tag <path> At <objects> | Unassign Tag <path> At <objects> | List Tag | Info Tag <path> [/R]",
        examples: &[
            "Store Tag \"All\"                     -- a top tag",
            "Store Tag 1 Tag \"Lx1\"               -- a child of tag 1",
            "Assign Tag 1.1 At Fixture 1 Thru 12   -- tag twelve fixtures",
            "Tag 1.1                               -- select its fixtures",
            "Tag 1 /R                              -- and those of every tag under it",
            "Move Tag 1.2 At Tag 2                 -- put it under tag 2",
            "Delete Tag 1.1 /KeepChildren          -- its child tags move up",
        ],
        related: &["group", "assign", "unassign"],
        ..Doc::new(
            "Tag",
            "A tag in the tag tree: a tag has one parent and is numbered within it, so its number is its path (Tag 1.2.1 = the first child of the second child of tag 1). Any show object carries tags. A tag reaches the objects tagged with it; /R adds those of every tag under it. Alone it selects its fixtures.",
        )
    },
    family: None,
    call_undo: SELECTION,
    call_class: CommandClass::Commit,
    scope: lumacore_objects::objects::on_master,
    reach: lumacore_objects::objects::show_reach,
    rights: lumacore_objects::rights::content,
    flags: FLAGS,
    planned: &[],
})
.verbs(&[
    (Verb::SelectFixtures, typed::<crate::select::SelectTag>),
    (Verb::Store, typed::<crate::store::StoreTag>),
    (Verb::Label, typed::<crate::label::LabelTag>),
    (Verb::Delete, typed::<crate::delete::DeleteTag>),
    (Verb::Move, typed::<crate::nest::MoveTag>),
    (Verb::Assign, typed::<crate::assign::AssignTag>),
    (Verb::Unassign, typed::<crate::assign::UnassignTag>),
    (Verb::List, typed::<crate::list::ListTag>),
    (Verb::Info, typed::<crate::list::InfoTag>),
    (Verb::On, typed::<crate::members::OnTag>),
    (Verb::Off, typed::<crate::members::OffTag>),
    (Verb::Toggle, typed::<crate::members::ToggleTag>),
    (Verb::Go, typed::<crate::members::GoTag>),
    (Verb::Goto, typed::<crate::members::GotoTag>),
    (Verb::Pause, typed::<crate::members::PauseTag>),
    (Verb::At, typed::<crate::members::AtTag>),
])
.delete_ask(crate::delete::ask)
.props(&crate::props::PROPS)
.tree(TreeDef { node: Node::Tags, name: None, children: top, object: view, datapool: None, door: None });

/// How the one resolver walks the tag tree (`IdReading::Path`).
pub static PATHS: &[PathDef] = &[PathDef { object: &TAG.decl, nodes }];

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

fn child(path: &TagPath, tag: &Tag) -> DynChild {
    let name = tag.meta.identity.name.clone();
    DynChild {
        display_name: match &name {
            Some(n) => format!("Tag {path} \"{n}\""),
            None => format!("Tag {path}"),
        },
        id: path.last(),
        node_type: "Tag",
        is_leaf: tag.children.is_empty(),
        child_count: tag.children.len(),
        name,
        guid: Some(tag.meta.identity.guid),
    }
}

/// Every tag with its path, depth first: the resolver's `PathDef`.
fn nodes(root: &Root) -> Vec<(Vec<u32>, DynChild)> {
    tree::all(&root.show_data.tags).into_iter().map(|(p, t)| (p.0.clone(), child(&p, t))).collect()
}

/// `Root.ShowData.Tags`: the top tags.
fn top(root: &Root) -> Vec<DynChild> {
    root.show_data.tags.iter().map(|(n, t)| child(&TagPath(vec![*n]), t)).collect()
}

fn view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.tags.get(&id).map(|t| json_view("Tag", id, t))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The tags `target` names, by path and GUID, in typed order (the one resolver).
pub(crate) fn targeted(state: &AppState, target: &Target) -> Result<Vec<(TagPath, Uuid)>, String> {
    let found = lumacore_objects::resolve::resolve_one(state, target, &TAG)?;
    let tags = &state.root.show_data.tags;
    let out: Vec<(TagPath, Uuid)> =
        found.members.iter().filter_map(|m| m.guid).filter_map(|g| tree::path_of(tags, g).map(|p| (p, g))).collect();
    if out.is_empty() {
        return Err(lumacore_objects::errors::render_code("No such Tag.", "E-TAG-001"));
    }
    Ok(out)
}

/// The tags to look for: these, and with `deep` every tag under them too.
pub(crate) fn reach(state: &AppState, tags: &[Uuid], deep: bool) -> Vec<Uuid> {
    if !deep {
        return tags.to_vec();
    }
    let tree_tags = &state.root.show_data.tags;
    let mut out: Vec<Uuid> = Vec::new();
    for g in tags {
        if let Some(t) = tree::path_of(tree_tags, *g).and_then(|p| tree::find(tree_tags, &p)) {
            for s in tree::subtree(t) {
                if !out.contains(&s) {
                    out.push(s);
                }
            }
        }
    }
    out
}

/// What carries any of `tags` (each object once), from the show's identity walk.
pub(crate) fn carriers(state: &AppState, tags: &[Uuid]) -> Vec<Member> {
    tree::members(&state.root.show_data.walk_identity(), tags)
}

/// The fixture numbers of the carriers that are fixtures, in the walk's order.
pub(crate) fn fixture_numbers(state: &AppState, carriers: &[Member]) -> Vec<u32> {
    let patch = &state.root.show_data.devices.dmx.patch;
    carriers
        .iter()
        .filter(|m| m.kind == "Fixture")
        .filter_map(|m| patch.fixture_by_guid(m.guid).and_then(|(_, f)| f.mvr.fid))
        .collect()
}

/// `Tag 1.1 "Lx1"`, for replies.
pub(crate) fn label(state: &AppState, path: &TagPath) -> String {
    label_in(&state.root.show_data.tags, path)
}

pub(crate) fn label_in(tags: &lumacore_tags::TagTree, path: &TagPath) -> String {
    match tree::find(tags, path).and_then(|t| t.meta.identity.name.clone()) {
        Some(n) => format!("Tag {path} \"{n}\""),
        None => format!("Tag {path}"),
    }
}

/// "12 fixtures, 1 sequence": a count per kind.
pub(crate) fn kinds_text(carriers: &[Member]) -> String {
    let mut kinds: Vec<(&str, usize)> = Vec::new();
    for m in carriers {
        match kinds.iter_mut().find(|(k, _)| *k == m.kind) {
            Some((_, n)) => *n += 1,
            None => kinds.push((m.kind, 1)),
        }
    }
    if kinds.is_empty() {
        return "nothing".into();
    }
    kinds.iter().map(|(k, n)| format!("{n} {}{}", k.to_lowercase(), if *n == 1 { "" } else { "s" })).collect::<Vec<_>>().join(", ")
}
