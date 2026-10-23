// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Every row's rules over the station's catalog (decisions §34 25.15): moved from the kernel's
//! tests when the command crates were cut out, so a cut crate's rows are checked too.
//!
//! Owns:          the row rules, the shim ratchet (SHIM_AT_MOST), the reference check.
//! Doesn't own:   the rows, the kernel's own tests.
//! Entry points:  the #[test] fns.
//! Decisions:     §15, §29, §33 57, 58, §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::registry::Rights;
use lumacore_objects::objects::Verb;
use lumacore_objects::tree_path::{Kind, Node};
use lumacore_objects::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match lumacore_objects::commands::dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// Handlers that still take raw words (`&[&str]`): every registry entry and object verb
/// minus the `typed::<…>` / `fenced::<…>` ones. It may only go down — lower it as handlers move (§33 part 3);
/// the branch merges at 0 (decision 33). 345 since the count became exact (2026-10-15: two
/// helper calls, `store_typed::<P>` and `delete_typed::<P>`, had been counted as typed — the
/// shown 343 was 345 raw handlers).
const SHIM_AT_MOST: usize = 337;

fn typed_uses() -> usize {
    let mut n = 0;
    let (mut built, mut per_object) = (0, 0);
    // Every crate with handlers: the kernel and the command crates (decisions §34 25).
    let root = std::path::Path::new(env!("LUMACORE_ROOT"));
    let mut dirs = vec![
        root.join("crates/model/lumacore-objects/src"),
        root.join("crates/commands"),
    ];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() && !path.ends_with("testdata") {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                && !path.ends_with("commands/typed.rs")
            {
                let text = std::fs::read_to_string(&path).unwrap();
                n += text
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .map(|l| uses(l, "typed::<") + uses(l, "fenced::<"))
                    .sum::<usize>();
                // A macro that declares a whole object (`line_object!`, decisions §34 "34f7c
                // hardened" 9) holds its typed handlers once in the text, once per object built.
                built += text.lines().filter(|l| l.trim_start().starts_with("lumacore_cmd_protocols::line_object!(")).count();
                if path.ends_with("lumacore-cmd-protocols/src/lines.rs") {
                    let body = text.split("macro_rules! line_object").nth(1).and_then(|b| b.split("macro_rules!").next()).unwrap_or("");
                    per_object = body.lines().filter(|l| !l.trim_start().starts_with("//")).map(|l| uses(l, "typed::<")).sum();
                }
            }
        }
    }
    n + built.saturating_sub(1) * per_object
}

/// `word` as itself, not the end of a longer name (`store_typed::<P>` is a helper, not a
/// typed handler — counted until 2026-10-15, which hid two raw handlers).
fn uses(line: &str, word: &str) -> usize {
    line.match_indices(word)
        .filter(|(i, _)| !line[..*i].ends_with(|c: char| c.is_alphanumeric() || c == '_'))
        .count()
}

/// Every example in the help parses with no unknown flag: what the docs tell people to
/// type is never refused (E-CMD-018).
#[test]
fn every_help_example_parses_without_unknown_flags() {
    let examples = lumacore_objects::commands::registry::all()
        .flat_map(|c| c.doc.examples.iter())
        .chain(lumacore_objects::objects::all().flat_map(|o| o.doc.examples.iter()));
    let mut bad = Vec::new();
    for example in examples {
        let line = example.split(" --").next().unwrap_or(example).trim();
        if let Ok(parsed) = lumacore_objects::commands::line::Line::parse(line) {
            for (_, flag) in parsed.unknown_flags() {
                bad.push(format!("{line}  ({flag})"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "help examples with flags their words don't declare:\n  {}",
        bad.join("\n  ")
    );
}

#[test]
fn verbs_are_lowercase() {
    for cmd in lumacore_objects::commands::registry::all() {
        assert_eq!(cmd.verb, cmd.verb.to_lowercase(), "verb '{}'", cmd.verb);
    }
}

#[test]
fn every_name_means_one_thing() {
    // One namespace: no spelling may belong to two different words.
    let mut seen: Vec<(String, lumacore_objects::commands::registry::Word)> = Vec::new();
    for (name, word) in lumacore_objects::commands::registry::all_names() {
        let lower = name.to_lowercase();
        if let Some((_, other)) = seen.iter().find(|(n, _)| *n == lower) {
            assert!(
                other.same(&word),
                "'{name}' names both {} and {}",
                other.label(),
                word.label()
            );
        } else {
            seen.push((lower, word));
        }
    }
}

#[test]
fn reading_is_anyones_on_every_object() {
    for obj in lumacore_objects::objects::all() {
        for verb in [Verb::List, Verb::Info, Verb::Dump] {
            assert_eq!(
                (obj.rights)(verb, &[]),
                Rights::Guest,
                "{} {verb:?}",
                obj.keyword
            );
        }
    }
    for word in ["List", "Info", "Dump", "Help"] {
        let c = lumacore_objects::commands::registry::all()
            .find(|c| c.verb.eq_ignore_ascii_case(word))
            .unwrap();
        assert_eq!(c.rights, Rights::Guest, "{word}");
    }
}

#[test]
fn the_shim_count_only_goes_down() {
    let handlers = lumacore_objects::commands::registry::all()
        .filter(|c| !c.verb.is_empty())
        .count()
        + lumacore_objects::objects::all()
            .map(|o| o.verbs.len())
            .sum::<usize>();
    let shim = handlers - typed_uses();
    assert!(
        shim <= SHIM_AT_MOST,
        "{shim} handlers take raw words, more than SHIM_AT_MOST ({SHIM_AT_MOST}) — new handlers are typed (`typed::<C>`)"
    );
    assert!(
        shim == SHIM_AT_MOST,
        "{shim} handlers take raw words now — lower SHIM_AT_MOST to {shim}"
    );
}

/// A planned word or verb says so (E-CMD-003) and changes nothing
/// (decisions §29): every Planned command and every object's planned verb.
#[test]
fn planned_words_reply_planned_and_change_nothing() {
    use lumacore_objects::commands::registry::Status;
    use lumacore_objects::objects::Verb;
    let mut state = AppState::new();
    let mut lines: Vec<String> = lumacore_objects::commands::registry::all()
        .filter(|c| c.status == Status::Planned)
        .map(|c| format!("{} 1", c.verb))
        .collect();
    for o in lumacore_objects::objects::all() {
        for v in o.planned {
            lines.push(match v {
                Verb::Call => format!("{} 1", o.keyword),
                // `<target> At <value>` (decision 43): `DMX 1.001 At 255`.
                Verb::At => format!("{} 1 At 1", o.keyword),
                v => format!("{} {} 1", v.label(), o.keyword),
            });
        }
    }
    assert!(lines.len() > 5, "planned words exist: {lines:?}");
    for line in lines {
        let before = serde_json::to_string(&state.root.show_data).unwrap();
        let out = run(&mut state, &line);
        assert!(
            out.contains("E-CMD-003") && out.contains("planned"),
            "{line}: {out}"
        );
        let after = serde_json::to_string(&state.root.show_data).unwrap();
        assert!(before == after, "{line} changed the show");
    }
}

/// One handler per verb (the first would hide a second), and a bare object always does
/// something: its default verb is built, planned, or a command of its own (§33 6, 40).
#[test]
fn every_object_has_one_handler_per_verb_and_a_default_verb() {
    for o in lumacore_objects::objects::all() {
        for (i, (v, _)) in o.verbs.iter().enumerate() {
            assert!(
                !o.verbs[..i].iter().any(|(w, _)| w == v),
                "{} declares {} twice",
                o.keyword,
                v.label()
            );
        }
        assert!(
            lumacore_objects::objects::has_bare(o)
                || o.is_planned(o.default_verb)
                || o.verbs.is_empty()
                || o.supports(Verb::List),
            "{}'s default verb {} does nothing",
            o.keyword,
            o.default_verb.label()
        );
    }
}

#[test]
fn keywords_and_aliases_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for o in lumacore_objects::objects::all() {
        for name in std::iter::once(o.keyword).chain(o.aliases.iter().copied()) {
            assert!(
                seen.insert(name.to_lowercase()),
                "duplicate object name {name}"
            );
        }
    }
}

#[test]
fn objects_are_not_also_commands() {
    // One namespace: an object is called by its own keyword (`Window 5`),
    // never through a duplicate CommandDef.
    for o in lumacore_objects::objects::all() {
        assert!(
            lumacore_objects::commands::registry::lookup_any(o.keyword).is_none(),
            "{} is both an object and a command",
            o.keyword
        );
    }
}

#[test]
fn every_object_takes_a_verb() {
    for o in lumacore_objects::objects::all() {
        assert!(
            !o.verbs.is_empty() || !o.planned.is_empty(),
            "{} has no verbs",
            o.keyword
        );
    }
}

/// The one registry (decisions §33 58): each object's property names are unique, every
/// property is documented, every keyed one has its section.
#[test]
fn every_property_is_unique_and_documented() {
    for o in lumacore_objects::objects::all() {
        let Some(props) = o.props else { continue };
        let mut seen = std::collections::BTreeSet::new();
        for p in props.all() {
            assert!(
                seen.insert(p.name.to_lowercase()),
                "{} twice on {}",
                p.name,
                o.keyword
            );
            assert!(!p.doc.is_empty(), "{} {} has no doc", o.keyword, p.name);
            if matches!(p.access, lumacore_objects::props::Access::Keyed { .. }) {
                assert!(
                    !p.section.is_empty(),
                    "{} {} has no section",
                    o.keyword,
                    p.name
                );
            }
        }
    }
}

/// Every collection node is some object's `TreeDef`, and every `TreeDef` names a collection.
#[test]
fn collections_and_tree_defs_match() {
    for &node in Node::ALL {
        if node.kind() == Kind::Collection {
            assert!(
                lumacore_objects::objects::tree_of(node).is_some(),
                "{} has no TreeDef",
                node.path()
            );
        }
    }
    for object in lumacore_objects::objects::all() {
        if let Some(tree) = &object.tree {
            assert_eq!(
                tree.node.kind(),
                Kind::Collection,
                "{}'s tree {}",
                object.keyword,
                tree.node.path()
            );
        }
    }
}

/// Every object with a tree gives its members' stored names (what a quoted name matches).
#[test]
fn every_pool_object_names_its_members() {
    let mut s = lumacore_objects::demo::fresh_state();
    let mut missing = std::collections::BTreeSet::new();
    for o in lumacore_objects::objects::all() {
        let Some(tree) = &o.tree else { continue };
        let dp = tree.datapool.as_ref().map(|_| s.selected_datapool());
        for c in tree.children_in(&s.root, dp) {
            if let Some(lumacore_objects::vfs::NodeView::Object {
                name: Some(name), ..
            }) = tree.object_in(&s.root, dp, c.id)
                && c.name.as_deref() != Some(name.as_str())
            {
                missing.insert(o.keyword);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "members that don't carry their stored name: {missing:?}"
    );
}

/// The committed reference is what the registries make today.
#[test]
fn the_reference_is_current() {
    use lumacore_objects::reference::{commands, objects, reference_dir};
    let dir = reference_dir(&lumacore_objects::demo::repo_root());
    for (file, made) in [("commands.md", commands()), ("objects.md", objects())] {
        let committed = std::fs::read_to_string(dir.join(file)).unwrap_or_default();
        assert!(
            committed == made,
            "docs/user/reference/{file} is stale (the registries changed). Run `cargo run -p lumacore-station --bin gen-docs`."
        );
    }
}

/// Names two words could share, settled: `u` is Update, `shortcuts` the command, `watch` the object.
#[test]
fn settled_clashes() {
    assert!(
        matches!(lumacore_objects::commands::registry::resolve("u").ok(), Some(lumacore_objects::commands::registry::Word::Command(c)) if c.verb == "update")
    );
    assert!(
        matches!(lumacore_objects::commands::registry::resolve("shortcuts").ok(), Some(lumacore_objects::commands::registry::Word::Command(c)) if c.verb == "shortcuts")
    );
    assert!(
        matches!(lumacore_objects::commands::registry::resolve("watch").ok(), Some(lumacore_objects::commands::registry::Word::Object(o)) if o.keyword == "Watch")
    );
}

/// Every ShowData object whose rows carry a NodeMeta gets its level's properties (decisions §12
/// "Tags part 1b hardened" 5): `Props.meta = Node`, so Set, EditSetting's Meta tab and the words
/// have Name, Note, Paint, … — or it declares its own (a tag: keyed by GUID).
#[test]
fn every_object_with_a_node_meta_has_its_level() {
    let mut state = lumacore_objects::demo::fresh_state();
    let dp = state.root.show_data.datapools.keys().next().copied();
    let mut missing = Vec::new();
    for o in lumacore_objects::objects::all() {
        let Some(tree) = o.tree.as_ref() else { continue };
        if !tree.node.path().starts_with(&Node::ShowData.path()) {
            continue;
        }
        let dp = tree.datapool.as_ref().and(dp);
        let Some(first) = tree.children_in(&state.root, dp).into_iter().next() else { continue };
        let Some(lumacore_objects::vfs::NodeView::Object { json, .. }) = tree.object_in(&state.root, dp, first.id) else { continue };
        if json["meta"].get("note").is_none() {
            continue;
        }
        let ok = o.props.is_some_and(|p| p.meta == lumacore_objects::props::MetaLevel::Node || (p.get("Name").is_some() && p.get("Note").is_some()));
        if !ok {
            missing.push(o.keyword);
        }
    }
    assert!(missing.is_empty(), "objects with a NodeMeta but not its properties: {missing:?}");
}

/// Resources are built from code: every node's GUID is derived from its key (decisions §12 "Tags
/// part 1b hardened" 6), the same on every station and every start, so a show may link to it.
#[test]
fn every_resource_has_a_derived_guid() {
    for t in lumacore_registry::get().window_types().values() {
        assert_eq!(t.meta.guid, lumacore_base::Guid::named("WindowType", &t.name).0, "WindowType {}", t.name);
    }
    let state = AppState::new();
    assert!(!state.root.resources.ui_keys.is_empty());
    for k in state.root.resources.ui_keys.values() {
        assert_eq!(k.meta.guid(), lumacore_base::Guid::named("UiKey", &k.id).0, "UIKey {}", k.id);
    }
}
