//! Object registry — objects publish themselves.
//!
//! Verbs publish themselves in the command registry (`commands/registry.rs`).
//! Objects (Fixture, Group, Window, User, …) publish themselves here: their
//! keyword and aliases, which verbs they take (with handlers), how to complete
//! their instances, and where they live in the object tree. Dispatch, tab
//! completion, `List`, `Info` and help all derive from this — so an object
//! can't exist without being discoverable.
//!
//! Each object is defined next to the code that owns it (`windows.rs`
//! defines Window, `users.rs` defines User, …) and listed once in its family's `OBJECTS` (`commands::registry::rows`).
//! See `handoffs/2026-10-10-object-registry.md`.

pub(crate) mod modes;
pub(crate) mod patch;
pub(crate) mod playback;
pub mod preset;
pub(crate) mod presets;
pub(crate) mod root;
pub mod poolxml;
pub mod protocols;
pub mod system;

pub use patch::FIXTURE;
pub use patch::devices_meta::MVR_CLASS;
/// Identify's ends: the station's tick and the patch's Unclaim (§34 round 3).
pub use patch::identify;
pub use modes::{HIGHLIGHT, SOLO};
pub use patch::{DATAPOOL, FIXTURE_TYPE, ROUTE_UNIVERSE, UNIVERSE, line_of, wire_text};
pub use playback::{CUE, MACRO, MASTER, SEQUENCE, TIMECODE};
pub use preset::PRESET;
pub use system::{PROGRAMMER, SHOW_DATA, USER_PROFILE};

use crate::AppState;
pub use crate::commands::registry::{Doc, Scope, Undo};
pub use crate::defs::object::*;
use crate::DispatchResult;
use crate::claims::Claims;

/// The collection at a tree `path` (a numbered datapool resolves to its
/// `DataPool` schema) and that datapool.
pub fn tree_and_datapool(path: &str) -> Option<(&'static TreeDef, Option<u32>)> {
    let path = crate::tree_path::TreePath::parse_dot(path)?;
    if path.entry().is_some() {
        return None;
    }
    tree_at(&path).map(|t| (t, path.datapool()))
}

/// Whether a bare `<Object> <ids>` does something: its default verb is built here or is a
/// command of its own (`Sequence 3` = `Select Sequence 3`).
pub fn has_bare(obj: &ObjectDef) -> bool {
    obj.supports(obj.default_verb)
        || crate::commands::registry::all()
            .any(|c| c.object_verb == Some(obj.default_verb) && c.status == crate::commands::registry::Status::Ready)
}

/// The acting user's default object (decisions §33 21, 39): what a bare number or name
/// names; `Fixture` when none is set.
pub fn default_object(state: &AppState) -> &'static ObjectDef {
    state.root
        .show_data
        .user(state.effective_user_name())
        .and_then(|u| u.default_object.as_deref())
        .and_then(find)
        .filter(|o| o.lum.counted())
        .unwrap_or(&FIXTURE)
}

/// What follows another show above the state (decisions §34 15): what the families add (the
/// key lines checked, §17), every link must resolve (broken ones listed, never dropped), every
/// view bar refreshes.
pub fn after_show_replaced(state: &mut AppState) {
    for hook in crate::commands::hooks::hooks() {
        if let crate::commands::hooks::Hook::ShowReplaced(f) = hook {
            f(state);
        }
    }
    crate::references::check(state, "load");
    crate::commands::views::profiles_changed(state, None);
}

/// Replace the show (a load, a sync, a merge, NewShow) and run what follows it.
pub fn replace_show(state: &mut AppState, show: lumacore_tree::ShowData) {
    state.replace_show_data(show);
    after_show_replaced(state);
}

/// Load a `.luma` file as the show and run what follows it.
pub fn load_show(state: &mut AppState, path: &str) -> Result<(), String> {
    state.load_show(path)?;
    after_show_replaced(state);
    Ok(())
}

/// Every object with its behaviour, from the catalog (each family's `OBJECTS`, decisions §34 25.13).
pub fn all() -> impl Iterator<Item = &'static ObjectDef> {
    lumacore_registry::get().rows::<&'static ObjectDef>().copied()
}

/// The object whose row this is: a row and its behaviour are one `ObjectDef` in the catalog
/// (decisions §34 21, found by the row's address).
///
/// # Panics
/// When the row is no listed object's — every object is in a family's `OBJECTS` (a test holds it).
pub fn def_of(decl: &ObjectDecl) -> &'static ObjectDef {
    all()
        .find(|o| std::ptr::eq(&o.decl, decl))
        .unwrap_or_else(|| panic!("{} is in no family's OBJECTS", decl.keyword))
}

/// What a statement with no verb runs (decisions §33 6, 43): `<target> At <value>` is the At verb
/// where the object takes it, anything else its default verb. Typed alone, an object whose
/// default verb isn't its own lists (`Sequence`) — until a counted object alone sets your default
/// object (§33 21, 27 — part 3b2).
pub fn bare_verb(obj: &ObjectDef, args: &[&str]) -> Verb {
    // `UserProfile "Bob" WindowPreset 1`: a declared child's own bare verb, where this object
    // passes it on to its children.
    let names_child = |w: &str| !w.starts_with('"') && obj.lum.children.iter().any(|c| c.matches(w));
    if let Some(i) = args.iter().position(|w| names_child(w)) {
        let child = obj.lum.children.iter().find(|c| c.matches(args[i])).expect("found");
        let verb = bare_verb(def_of(child), &args[i + 1..]);
        if obj.supports(verb) {
            return verb;
        }
    }
    // `<target> At <value>` is the At verb, a planned one too (`DMX 1.001 At 255` replies planned).
    if (obj.supports(Verb::At) || obj.is_planned(Verb::At)) && args.iter().any(|w| w.eq_ignore_ascii_case("at")) {
        return Verb::At;
    }
    if args.is_empty() && !obj.supports(obj.default_verb) && obj.supports(Verb::List) {
        return Verb::List;
    }
    obj.default_verb
}

/// The object a word names (keyword or alias, case-insensitive).
pub fn find(word: &str) -> Option<&'static ObjectDef> {
    // One namespace (§15): a name is a command or an object, never both; the registry's
    // index answers it (decisions §31).
    match crate::commands::registry::lookup_word(word.trim_matches('"')) {
        Some(crate::commands::registry::Word::Object(obj)) => Some(obj),
        _ => None,
    }
}

/// Objects that take `verb`.
pub fn supporting(verb: Verb) -> impl Iterator<Item = &'static ObjectDef> {
    all().filter(move |o| o.supports(verb))
}

fn doesnt_take(obj: &ObjectDef, verb: Verb) -> DispatchResult {
    DispatchResult::Output(crate::Reply::error(
        format!(
            "{} doesn't take {}. It takes: {}",
            obj.keyword,
            verb.label(),
            obj.verbs
                .iter()
                .map(|(v, _)| v.label())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        "E-CMD-004",
    ))
}

/// Route `<verb> <object> …` to the object's handler. None = the first
/// argument isn't an object that this form applies to; the command's own
/// handler takes it (tree paths, settings, context shortcuts like `Delete 5`
/// in Fixture context).
pub fn dispatch_verb(state: &mut AppState, verb: Verb, args: &[&str]) -> Option<DispatchResult> {
    let obj = args.first().and_then(|a| find(a))?;
    // `Set <setting> …` shares names with objects (Station), and `Claim`
    // claims any tree path (`Claim Root.ShowData.Devices`, `Claim Fixture 3`): only objects
    // that take the verb route it.
    // `Import "fixture.gdtf"` and `Export Fixture 1 "f.json"` keep their own
    // forms for objects without the verb.
    // `Select Sequence 3`, `Goto Sequence 2 Cue 3`, `Go Sequence 2` keep their command's own form
    // for objects without the verb (decisions §33 26, 40).
    // `Sort` / `Split` are typed commands that read their target themselves (decisions §33 88).
    if matches!(verb, Verb::Set | Verb::Claim | Verb::Pause | Verb::Import | Verb::Export | Verb::Reset | Verb::Select | Verb::Goto | Verb::Go | Verb::Sort | Verb::Split | Verb::EditSetting | Verb::Label)
        && !obj.supports(verb)
        && !obj.is_planned(verb)
    {
        return None;
    }
    Some(match obj.handler(verb) {
        Some(h) => h(state, &args[1..]),
        None if obj.is_planned(verb) => planned(obj, verb),
        None => doesnt_take(obj, verb),
    })
}

/// The reply of a planned verb: nothing runs (decisions §29, E-CMD-003).
fn planned(obj: &ObjectDef, verb: Verb) -> DispatchResult {
    let label = match verb {
        Verb::Call => obj.keyword.to_string(),
        v => format!("{} {}", v.label(), obj.keyword),
    };
    DispatchResult::Output(crate::Reply::error(
        crate::commands::planned_message(&label, &obj.keyword.to_lowercase()),
        "E-CMD-003",
    ))
}

/// A changing command naming an object (`Label Group 3 …`) another user
/// claimed — or anything inside one — is refused (decisions §21 "6d";
/// `Claim` asks them for it). `args` start at the object keyword.
pub fn held_elsewhere(state: &AppState, args: &[&str]) -> Option<String> {
    if state.root.runtime.claims.is_empty() {
        return None;
    }
    let obj = find(args.first()?)?;
    let id: u32 = args.get(1)?.parse().ok()?;
    let tree = obj.tree.as_ref()?;
    // `Group 3` is in the user's selected datapool.
    let dp = tree.datapool.as_ref().map(|_| state.selected_datapool());
    let Some(crate::vfs::NodeView::Object { guid, .. }) = tree.object_in(&state.root, dp, id) else {
        return None;
    };
    let guid = uuid::Uuid::parse_str(&guid).ok()?;
    let (label, owner) = state.held_by_other(guid)?;
    Some(crate::errors::render_code(
        format!(
            "{} {id} is in {label}, claimed by {owner}. Claim {label} asks them for it.",
            obj.keyword
        ),
        "E-CLAIM-001",
    ))
}

/// `<object> …` — the object used as a verb (`Window 5`, `Fixture 1 At 50`).
pub fn call(state: &mut AppState, obj: &ObjectDef, args: &[&str]) -> DispatchResult {
    // A counted object typed alone becomes your default object (decisions §33 21, 27, 39).
    if args.is_empty() && obj.lum.counted() {
        return set_default_object(state, obj);
    }
    let verb = crate::objects::bare_verb(obj, args);
    if let Some(h) = obj.handler(verb) {
        return h(state, args);
    }
    if obj.is_planned(verb) {
        return planned(obj, verb);
    }
    // A default verb that is a command of its own (`Sequence 3` = `Select Sequence 3`).
    if let Some(cmd) = crate::commands::registry::all().find(|c| c.object_verb == Some(verb)) {
        let line: Vec<&str> = std::iter::once(obj.keyword).chain(args.iter().copied()).collect();
        return (cmd.run)(state, &line);
    }
    doesnt_take(obj, verb)
}

/// `Group` alone: a bare number or name now names a Group (`3 At 50` = `Group 3 At 50`).
fn set_default_object(state: &mut AppState, obj: &ObjectDef) -> DispatchResult {
    let user = state.effective_user_name().to_string();
    let keyword = (!std::ptr::eq(obj, &FIXTURE as &ObjectDef)).then(|| obj.keyword.to_string());
    match state.root.show_data.user_mut(&user) {
        Some(u) => u.default_object = keyword,
        None => return DispatchResult::Output(crate::Reply::error("No user to set a default object for.", "E-CMD-011")),
    }
    let k = obj.keyword;
    DispatchResult::Output(
        crate::text::render_success(&format!("{k}> — a bare number or name is a {k} now (3 = {k} 3)")).into(),
    )
}

/// Completions for the argument at `tok_pos` of a line starting with
/// `first`. Some(candidates) when the object registry is responsible for this
/// position.
pub fn complete(
    state: &AppState,
    first: crate::commands::registry::Word,
    prev: &str,
    tok_pos: usize,
    word: &str,
) -> Option<Vec<String>> {
    use crate::commands::registry::Word;
    let lower = word.to_lowercase();
    if let Word::Command(c) = first
        && let Some(v) = c.object_verb
    {
        return match tok_pos {
            // `Store ▸` → every object that can be stored (planned ones
            // too: they are discoverable, and say so when run)
            1 => Some(
                all()
                    .filter(|o| takes(o, v))
                    .map(|o| o.keyword.to_string())
                    .filter(|k| k.to_lowercase().starts_with(&lower))
                    .collect(),
            ),
            // `Store Window ▸` → the object's instances
            2 => find(prev)
                .filter(|o| takes(o, v))
                .and_then(|o| o.complete)
                .map(|c| c(state, &lower)),
            _ => None,
        };
    }
    // `Window ▸` (an object used as a verb) → its instances
    let Word::Object(obj) = first else {
        return None;
    };
    if !has_bare(obj) {
        return None;
    }
    (tok_pos == 1).then(|| obj.complete.map(|c| c(state, &lower)).unwrap_or_default())
}

/// Whether `Set ▸` / `Store ▸` offers `obj`: it has the verb (or plans it) —
/// and every object with declared properties takes `Set`, which is one typed
/// route reading the registry, never an object's own handler (decisions §33
/// 3c3, 58).
fn takes(obj: &ObjectDef, verb: Verb) -> bool {
    obj.supports(verb) || obj.is_planned(verb) || (verb == Verb::Set && obj.props.is_some())
}

/// The object whose tree collection is `node`.
/// The object whose tree is `node` (a lower layer finds a row by its declared collection, never
/// by naming its static — decisions §34 25.17).
pub fn object_of(node: crate::tree_path::Node) -> Option<&'static ObjectDef> {
    all().find(|o| o.tree.as_ref().is_some_and(|t| t.node == node))
}

pub fn tree_of(node: crate::tree_path::Node) -> Option<&'static TreeDef> {
    all().filter_map(|o| o.tree.as_ref()).find(|t| t.node == node)
}

/// The tree collection a path stands on: by its node, and inside a named node by the entry's
/// name too (`Protocols.DMXProtocols.Sacn` is `Sacn`'s, decisions §34 "34f7c hardened" 5).
pub fn tree_at(path: &crate::tree_path::TreePath) -> Option<&'static TreeDef> {
    let node = path.node();
    match path.named() {
        Some(name) => all()
            .filter_map(|o| o.tree.as_ref())
            .find(|t| t.node == node && t.name.is_some_and(|n| n.eq_ignore_ascii_case(name))),
        None => tree_of(node),
    }
}

/// The object a tree path stands on, its datapool and its number (`…DataPools.2.Groups.3` →
/// Group in DataPool 2, 3; `…DataPools.2` → DataPool 2); None for a node or a live entry.
pub fn at_path(path: &crate::tree_path::TreePath) -> Option<(&'static ObjectDef, Option<u32>, u32)> {
    if path.node() == crate::tree_path::Node::DataPool {
        return Some((&DATAPOOL, None, path.datapool()?));
    }
    let id = path.entry()?;
    let tree = tree_at(path)?;
    let object = all().find(|o| o.tree.as_ref().is_some_and(|t| std::ptr::eq(t, tree)))?;
    Some((object, path.datapool(), id))
}

/// The named entries of a named node (`DMXProtocols` → `Sacn`, `ArtNet`): the objects whose
/// tree is its shape, in catalog order.
pub fn named_in(node: crate::tree_path::Node) -> Vec<&'static str> {
    let crate::tree_path::Kind::Named(shape) = node.kind() else { return Vec::new() };
    all().filter_map(|o| o.tree.as_ref()).filter(|t| t.node == shape).filter_map(|t| t.name).collect()
}

/// Every object keyword (for argument completion anywhere).
pub fn keywords() -> impl Iterator<Item = &'static str> {
    all().map(|o| o.keyword)
}

// ── Helpers for object definitions ──────────────────────────────────────────

pub fn out(text: String) -> DispatchResult {
    DispatchResult::Output(text.into())
}

/// Completions from numbers / names, filtered by prefix.
pub fn filter<I: IntoIterator<Item = String>>(items: I, prefix: &str) -> Vec<String> {
    items
        .into_iter()
        .filter(|s| {
            s.to_lowercase()
                .trim_start_matches('"')
                .starts_with(prefix.trim_start_matches('"'))
        })
        .collect()
}

/// A name for the command line: quoted when it has spaces.
pub fn quote(name: &str) -> String {
    if name.contains(' ') {
        format!("\"{name}\"")
    } else {
        name.to_string()
    }
}

/// An object view from any serializable value (for objects that aren't
/// `TreeNode`s): name from `name` or `meta.name`.
pub fn json_view<T: serde::Serialize>(
    node_type: &'static str,
    id: u32,
    value: &T,
) -> crate::vfs::NodeView {
    let json = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    let name = json
        .get("name")
        .or_else(|| json.get("meta").and_then(|m| m.get("name")))
        .and_then(|n| n.as_str())
        .map(str::to_string);
    // Its GUID from its meta (the inspector's links need it, decisions 0293).
    let guid = json
        .get("guid")
        .or_else(|| json.get("meta").and_then(|m| m.get("guid")))
        .and_then(|g| g.as_str())
        .unwrap_or_default()
        .to_string();
    crate::vfs::NodeView::Object {
        json,
        node_type,
        id,
        guid,
        name,
    }
}

/// Patch-changing handlers need the Devices claim (decisions §34 "34g4 patch hardened" 8).
pub fn patch_claimed(state: &AppState) -> Result<(), DispatchResult> {
    if state.is_claimed_by_me("Devices") {
        Ok(())
    } else {
        Err(out(crate::errors::render_code(
            "Devices are not claimed: Claim Root.ShowData.Devices (or 'enter patch') takes the draft.",
            "E-CMD-010",
        )))
    }
}
