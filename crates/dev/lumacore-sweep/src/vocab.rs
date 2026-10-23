// ██╗   ██╗ ██████╗  ██████╗ █████╗ ██████╗
// ██║   ██║██╔═══██╗██╔════╝██╔══██╗██╔══██╗
// ██║   ██║██║   ██║██║     ███████║██████╔╝
// ╚██╗ ██╔╝██║   ██║██║     ██╔══██║██╔══██╗
//  ╚████╔╝ ╚██████╔╝╚██████╗██║  ██║██████╔╝
//   ╚═══╝   ╚═════╝  ╚═════╝╚═╝  ╚═╝╚═════╝
// FILE: vocab

//! What the sweep can say, read from the registries at run time ("it needs to be dynamic, so if we
//! add new features down the line we dont need to update it"). Nothing here names a command,
//! keyword or property.
//!
//! Owns:          what the sweep can say: words, keywords, flags, writable properties, their values, read from the registries at run time.
//! Doesn't own:   the lines built from it (lines.rs).
//! Entry points:  Vocab, swept, swept_line, values_of, args_for, verb_word.
//! Decisions:     §25 "Step 10 hardening", §34 21.

use std::collections::BTreeSet;

use lumacore_objects::AppState;
use lumacore_objects::commands::registry::{Args, CommandDecl, Reach};
use lumacore_objects::objects::{self, ObjectDef, Verb};
use lumacore_objects::props::{PropKind, Property};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One writable property of one object kind, with the ids to try it on.
#[derive(Debug, Clone)]
pub struct PropTarget {
    pub object: &'static str,
    pub ids: Vec<u32>,
    pub prop: &'static Property,
}

/// Everything the generator draws from.
pub struct Vocab {
    /// Commands the sweep runs (`Reach::Show`, and `Drive` in the sandbox).
    pub commands: Vec<&'static CommandDecl>,
    /// Objects (every keyword; a verb on one whose reach isn't Show is skipped).
    pub objects: Vec<&'static ObjectDef>,
    /// Every `/Flag` the docs mention, lowercase, sorted.
    pub flags: Vec<String>,
    /// Writable properties per object.
    pub props: Vec<PropTarget>,
    /// Fields an object's JSON has that `Set` must refuse (read-only),
    /// `(object keyword, id, field)`.
    pub read_only: Vec<(&'static str, u32, String)>,
    /// Ids that exist in the base show, per object keyword (first, last).
    pub ids: Vec<(&'static str, Vec<u32>)>,
    /// Example lines from every doc.
    pub examples: Vec<String>,
    /// Words the sweep leaves alone, with their reach (for the report).
    pub skipped: Vec<(String, Reach)>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// What the sweep runs (decisions §29): `Show` lines with the disk shut,
/// `Drive` lines in the sandbox; the tree shell stays out.
pub fn swept(reach: Reach) -> bool {
    reach != Reach::TreeShell
}

/// Whether a whole line is swept: its reach, and no statement whose word is
/// a tree-shell word — a line naming `Drives` reaches Drive, but `ls Drives`
/// is still the tree shell (issue 20 g).
pub fn swept_line(line: &str) -> bool {
    use lumacore_objects::commands::registry::{self, Word};
    swept(registry::line_reach(line))
        && !lumacore_objects::commands::tokens::statements(line).iter().any(|st| {
            st.split_whitespace().next().is_some_and(|w| {
                matches!(registry::resolve(w), Ok(Word::Command(c)) if c.reach == Reach::TreeShell)
            })
        })
}

impl Vocab {
    /// Read the registries; `base` is the show the sweep starts from (ids,
    /// read-only fields).
    pub fn new(base: &AppState) -> Self {
        let mut commands = Vec::new();
        let mut skipped = Vec::new();
        for def in lumacore_registry::get().commands() {
            if swept(def.reach) {
                commands.push(def);
            } else {
                skipped.push((def.verb.to_string(), def.reach));
            }
        }
        let objects: Vec<&'static ObjectDef> = objects::all().collect::<Vec<_>>();
        for obj in &objects {
            for (verb, _) in obj.verbs {
                let reach = (obj.reach)(*verb);
                if !swept(reach) {
                    skipped.push((format!("{verb:?} {}", obj.keyword), reach));
                }
            }
        }

        // Every declared flag, with a value where it takes one (decisions §31): the random
        // sequences mix them in anywhere.
        let flags: BTreeSet<String> = lumacore_objects::commands::flags::all().map(flag_case).collect();
        let mut examples = Vec::new();
        let docs = commands
            .iter()
            .map(|c| &c.doc)
            .chain(objects.iter().map(|o| &o.doc));
        for doc in docs {
            for ex in doc.examples {
                let line = ex.split("--").next().unwrap_or("").trim();
                if !line.is_empty() && swept_line(line) {
                    examples.push(line.to_string());
                }
            }
        }

        let mut ids = Vec::new();
        let mut props = Vec::new();
        let mut read_only = Vec::new();
        for obj in &objects {
            let Some(tree) = obj.tree.as_ref() else {
                continue;
            };
            let have: Vec<u32> = (tree.children)(&base.root)
                .iter()
                .map(|c| c.id)
                .collect();
            let mut some: Vec<u32> = have.first().into_iter().chain(have.last()).copied().collect();
            some.dedup();
            ids.push((obj.keyword, some.clone()));
            let first = some.first().copied().unwrap_or(1);
            // The tree fields it declares (keyed ones are swept as lines, not read back).
            let writable: Vec<&'static Property> = obj
                .props
                .map(|p| p.all().filter(|p| matches!(p.access, lumacore_objects::props::Access::Field(_))).collect())
                .unwrap_or_default();
            for prop in &writable {
                props.push(PropTarget {
                    object: obj.keyword,
                    ids: if some.is_empty() { vec![1] } else { some.clone() },
                    prop,
                });
            }
            if let Some(lumacore_objects::vfs::NodeView::Object { json, .. }) = (tree.object)(&base.root, first)
                && let Some(map) = json.as_object()
            {
                for key in map.keys() {
                    if !writable.iter().any(|p| p.name.eq_ignore_ascii_case(key)) {
                        read_only.push((obj.keyword, first, key.clone()));
                    }
                }
            }
        }

        Self {
            commands,
            objects,
            flags: flags.into_iter().collect(),
            props,
            read_only,
            ids,
            examples,
            skipped,
        }
    }

    /// Ids to address `object` with: what the show has, then edge cases.
    pub fn ids_of(&self, object: &str) -> Vec<String> {
        let mut out: Vec<String> = self
            .ids
            .iter()
            .find(|(k, _)| *k == object)
            .map(|(_, ids)| ids.iter().map(u32::to_string).collect())
            .unwrap_or_default();
        for edge in ["1", "1 Thru 3", "999", "0", "\"sweep\""] {
            if !out.iter().any(|o| o == edge) {
                out.push(edge.to_string());
            }
        }
        out
    }

    /// The flags a command takes on its own, as the sweep writes them (`Claim … /Force`).
    pub fn command_flags(cmd: &CommandDecl) -> Vec<String> {
        cmd.flags.iter().map(flag_case).collect()
    }

    /// The flags `obj` takes with `verb`, besides the command's own (`Store Group 3 /Overwrite`).
    pub fn object_flags(obj: &ObjectDef, verb: lumacore_objects::objects::Verb) -> Vec<String> {
        obj.flags.iter().filter(|f| f.with_verb(verb)).map(flag_case).collect()
    }

}

/// Values to try for a property: valid ones, the edges and wrong ones. A
/// new `PropKind` doesn't compile until it has values here.
pub fn values_of(kind: PropKind) -> Vec<String> {
    match kind {
        PropKind::Declared => ["On", "Off", "Toggle", "1", "+ 1", "\"sweep\""].map(String::from).to_vec(),
        PropKind::Text => vec![
            "\"sweep\"".into(),
            "\"\"".into(),
            "none".into(),
            "\"Ünïcødé 🐱\"".into(),
            format!("\"{}\"", "x".repeat(300)),
        ],
        PropKind::OnOff | PropKind::Switch => ["Yes", "No", "On", "Off", "Toggle", "maybe"].map(String::from).to_vec(),
        PropKind::Int { min, max } => {
            let mut v = vec![min, max, min.saturating_sub(1), max.saturating_add(1), min / 2 + max / 2];
            v.dedup();
            let mut out: Vec<String> = v.into_iter().map(|n| n.to_string()).collect();
            out.push("abc".into());
            // Relative input (decisions §30 "12b").
            out.extend(["+ 1", "- 1", "* 2", "/ 0"].map(String::from));
            out
        }
        PropKind::Choice(options) => options
            .iter()
            .map(|o| o.to_string())
            .chain(["Bogus".to_string()])
            .collect(),
        PropKind::Link { .. } | PropKind::OwnLink { .. } => {
            ["1", "999", "\"Missing\"", "none"].map(String::from).to_vec()
        }
        PropKind::Paint => ["#3c8f3c", "#FFFFFF", "none", "#12", "red", "Blue", "Theme \"Position\""].map(String::from).to_vec(),
        PropKind::Icon => ["star", "grid-3x3", "none", "\"two words\""].map(String::from).to_vec(),
        PropKind::Colour => ["\"#9a7a08\"", "#FFFFFF", "none", "#12345", "red"].map(String::from).to_vec(),
    }
}

/// Values for a word whose argument is a value (`At`).
pub const VALUES: &[&str] = &["50", "Full", "Zero", "0", "100", "-1", "101", "abc"];

/// Arguments for a command by its declared argument shape. A new `Args`
/// variant doesn't compile until it has arguments here.
pub fn args_for(shape: Args, vocab: &Vocab, _base: &AppState) -> Vec<String> {
    match shape {
        Args::Objects | Args::ObjectsOrTree => vocab
            .objects
            .iter()
            .flat_map(|o| [o.keyword.to_string(), format!("{} 1", o.keyword)])
            .collect(),
        // Tree navigation belongs to the tree shell; from a Show word, try
        // the root and a made-up path.
        Args::TreeNav { .. } | Args::TreeFromRoot => vec!["Root".into(), "..".into(), "Nowhere.At.All".into()],
        Args::TreeItems { options, .. } => std::iter::once("Root".to_string())
            .chain(options.iter().map(|o| o.to_string()))
            .collect(),
        Args::Words => vocab.commands.iter().map(|c| c.verb.to_string()).collect(),
        Args::Value => VALUES.iter().map(|v| v.to_string()).collect(),
        Args::Watches => vec!["all".into(), "1".into()],
    }
}

/// An object verb's word (`Store` for `Verb::Store`): the command that is it.
pub fn verb_word(verb: Verb) -> Option<&'static str> {
    lumacore_registry::get()
        .commands()
        .find(|c| c.object_verb == Some(verb))
        .map(|c| c.verb)
}

/// A declared flag as the sweep writes it: `/overwrite`, `/port 1`, `/watch=1` (a sample value
/// where it takes one; decisions §31).
fn flag_case(f: &lumacore_objects::commands::flags::FlagDef) -> String {
    use lumacore_objects::commands::flags::FlagValue;
    let name = f.name.to_ascii_lowercase();
    match f.value {
        FlagValue::None => format!("/{name}"),
        FlagValue::One(_) | FlagValue::Rest(_) => format!("/{name} 1"),
        FlagValue::Inline(_) => format!("/{name}=1"),
    }
}
