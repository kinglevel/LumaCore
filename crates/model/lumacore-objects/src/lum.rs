//! The station's Lum vocabulary (decisions §33): the words `lumacore_syntax` parses with, built
//! once from the registries — never a second word list.
//!
//! - verbs: every [`CommandDef`] by its one name (`hold`, `fence` from the entry);
//! - objects: every [`ObjectDef`] with its [`ObjectDef::lum`] shape (ids, owner-first children);
//! - helpers and value words that no registry entry carries: [`HELPERS`], [`VALUE_WORDS`];
//! - value words from the declared properties (choices, named numbers, on/off words) and the
//!   window types' names;
//! - property names from every writable property (`props`, `settings_props`), so a bare one is
//!   refused with its quoted form (decision 7);
//! - options from every declared flag; a fence word's flags are its `-x` / `--name`.
//!
//! Short forms (registry `aliases`) are not words of Lum (decision 12) and are left out.
//!
//! [`CommandDef`]: crate::commands::registry::CommandDef
//! [`ObjectDef`]: crate::objects::ObjectDef
//! [`ObjectDef::lum`]: crate::objects::ObjectDef::lum

use std::sync::OnceLock;

use lumacore_syntax::{FenceDef, ObjectDef as LumObject, VerbDef, Vocabulary};

use crate::commands::flags::{FlagDef, FlagValue};
use crate::commands::registry::CommandDef;
use crate::objects::ObjectDef;
use crate::props::PropKind;

/// Words that start a slot (`At 50`, `Fade 2`, `Delay 1`) and are no command of their own.
/// `At` is a verb too (decision 43).
pub const HELPERS: &[&str] = &["At", "Fade", "Delay"];

/// Value words no property declares: levels and the on / off words (decision 14), `Ask`
/// (decision 17).
pub const VALUE_WORDS: &[&str] = &["Full", "Zero", "On", "Off", "Toggle", "Yes", "No", "Ask", "Groups"]; // Groups: what Import PoolXml takes (decisions §40 13)

/// The vocabulary, built on first use.
pub fn vocabulary() -> &'static Vocabulary {
    static VOCAB: OnceLock<Vocabulary> = OnceLock::new();
    VOCAB.get_or_init(build)
}

/// A command's one name as written: its display name, else the verb.
fn verb_name(c: &CommandDef) -> &'static str {
    if c.doc.display.is_empty() { c.verb } else { c.doc.display }
}

fn is_keyword(name: &str) -> bool {
    lumacore_syntax::KEYWORDS.iter().any(|k| k.eq_ignore_ascii_case(name))
}

fn build() -> Vocabulary {
    let mut v = Vocabulary::new();
    for c in crate::commands::registry::all() {
        let name = verb_name(c);
        if c.verb.is_empty() || is_keyword(name) || !lumacore_syntax::is_word(name) {
            continue;
        }
        if c.fence {
            // Unix words keep their lowercase spelling (`cat`, `ls`).
            v.fence(c.verb, FenceDef { flags: c.flags.iter().flat_map(fence_flags).collect() });
        } else {
            v.verb(name, VerbDef { hold: c.hold });
        }
    }
    for o in crate::objects::all() {
        if is_keyword(o.keyword) {
            continue;
        }
        v.object(o.keyword, lum_object(o));
    }
    for h in HELPERS {
        v.helper(h);
    }
    let mut values: Vec<&str> = VALUE_WORDS.to_vec();
    let mut properties: Vec<&str> = Vec::new();
    for p in props() {
        properties.push(p.0);
        values.extend(p.1);
    }
    let types = lumacore_registry::get().window_types();
    let type_names: Vec<String> = types.values().map(|t| t.label.clone()).collect();
    values.extend(type_names.iter().map(String::as_str));
    // A window setting's choices are the values its bar writes (`Set Self "Layout" Fixed`).
    let choices: Vec<String> = types
        .values()
        .flat_map(|t| t.settings.iter())
        .filter_map(|d| match &d.kind {
            lumacore_tree::window_types::SettingKind::Enum { options } => Some(options.iter().filter_map(|(v, _)| v.as_str().map(str::to_string)).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect();
    for value in values {
        if lumacore_syntax::is_word(value) && !is_keyword(value) {
            v.value(value);
        }
    }
    // A property name that is also another word keeps that word's meaning (it is quoted anyway).
    for p in properties {
        if lumacore_syntax::is_word(p) && !is_keyword(p) && v.lookup(p).is_none() {
            v.property(p);
        }
    }
    // A window setting's choices that aren't words already (`Fixed`; `At` stays the verb).
    for c in &choices {
        if lumacore_syntax::is_word(c) && !is_keyword(c) && v.lookup(c).is_none() {
            v.value(c);
        }
    }
    // An option's value words; one that is a property's name stays that (`/By Name`).
    for c in all_flags().flat_map(|f| f.choices.iter()) {
        if lumacore_syntax::is_word(c) && !is_keyword(c) && v.lookup(c).is_none() {
            v.value(c);
        }
    }
    for f in all_flags() {
        v.option(f.name, !matches!(f.value, FlagValue::None));
    }
    v
}

fn lum_object(o: &ObjectDef) -> LumObject {
    LumObject {
        counted: o.lum.counted(),
        id_parts: o.lum.id_parts,
        children: o.lum.children.iter().map(|c| c.keyword.to_string()).collect(),
        id_kinds: o.lum.id_kinds.iter().map(|k| k.to_string()).collect(),
    }
}

/// A fence word's flag as `-x` / `--name`: its name and every short form, lowercase.
fn fence_flags(f: &FlagDef) -> Vec<(String, bool)> {
    let takes = !matches!(f.value, FlagValue::None);
    std::iter::once(f.name).chain(f.aliases.iter().copied()).map(|n| (n.to_lowercase(), takes)).collect()
}

/// Every declared flag: the global ones, each command's (fence words aside), each object's.
fn all_flags() -> impl Iterator<Item = &'static FlagDef> {
    lumacore_registry::get().global_flags()
        .chain(crate::commands::registry::all().filter(|c| !c.fence).flat_map(|c| c.flags.iter()))
        .chain(crate::objects::all().flat_map(|o| o.flags.iter()))
}

/// Every writable property's name with the value words it declares.
fn props() -> Vec<(&'static str, Vec<&'static str>)> {
    let mut out = Vec::new();
    let kind_words = |k: &PropKind| match k {
        PropKind::Choice(words) => words.to_vec(),
        // The palette's colour words (decisions 0294): `Set Group 3 "Paint" Blue`.
        PropKind::Paint => lumacore_base::paint::NAMED.iter().map(|(n, _)| *n).collect(),
        _ => Vec::new(),
    };
    for p in crate::objects::all().filter_map(|o| o.props).flat_map(|p| p.every()) {
        let mut words = kind_words(&p.kind);
        words.extend(p.named);
        if p.kind.is_on_off() {
            words.extend([p.words.0, p.words.1]);
        }
        out.push((p.name, words));
    }
    out
}
