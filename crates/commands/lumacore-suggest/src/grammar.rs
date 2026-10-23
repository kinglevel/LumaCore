//  ██████╗ ██████╗  █████╗ ███╗   ███╗███╗   ███╗ █████╗ ██████╗
// ██╔════╝ ██╔══██╗██╔══██╗████╗ ████║████╗ ████║██╔══██╗██╔══██╗
// ██║  ███╗██████╔╝███████║██╔████╔██║██╔████╔██║███████║██████╔╝
// ██║   ██║██╔══██╗██╔══██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██╔══██╗
// ╚██████╔╝██║  ██║██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║  ██║
//  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝
// FILE: grammar

//! The grammar half of the suggestion engine (decisions §30 "12b2", moved
//! down from `terminal/completer.rs`): what is valid at the cursor, from the
//! registries (`registry::Args`, objects' completers, the object tree).
//! [`super::suggest`] ranks and types it.
//!
//! - `complete_input()`: the candidates that extend the word at the cursor
//! - `hint_input()`: fish-style ghost text suffix for the current input
//!
//! Context-aware completion:
//! - Position 0: complete against registered command verbs
//! - After `cd`: tree path completions
//! - After `fixture`, `group`: fixture/group IDs and names
//! - After `at` or `fixture N at`: attribute names from GDTF + value keywords
//! - After `attribute`: attribute names from selected fixtures' GDTF modes
//! - After `store`: Group, Universe, Preset subtypes
//! - After `help`: all command names
//! - After `info`, `dump`: object type keywords
//! - After `list`: listable types
//! - After `enter`: scope type keywords
//! - After `unwatch`: active watch names
//!
//! Owns:          what may follow at the cursor, read from the registries (grammar first).
//! Doesn't own:   ranking (lib.rs).
//! Entry points:  complete_input, hint_input, collect_attribute_names.
//! Decisions:     §15, §30 "12b2", §34 25.9.

use lumacore_objects::AppState;
use lumacore_objects::commands::registry::{self, Args, Word};
use lumacore_objects::commands::tokens;
use lumacore_objects::tree_path::{Node, TreePath};
use lumacore_tree::{Root, TreeNode};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Every structural node name in the object tree (for `cd Patch` /
/// `ls Playback` shorthands), walked from the tree itself.
pub(super) fn tree_node_names() -> Vec<String> {
    let mut names = Vec::new();
    let mut stack = vec!["Root".to_string()];
    while let Some(path) = stack.pop() {
        for child in lumacore_objects::nav_children_at(&path) {
            if !names
                .iter()
                .any(|n: &String| n.eq_ignore_ascii_case(&child))
            {
                names.push(child.clone());
            }
            if path.matches('.').count() < 6 {
                stack.push(format!("{path}.{child}"));
            }
        }
    }
    names
}

/// Root's children in the object tree, plus Root (for `List root`, `List ShowData`).
fn tree_top_nodes() -> impl Iterator<Item = String> {
    std::iter::once("Root".to_string()).chain(lumacore_objects::nav_children_at("Root"))
}

// *************************************************************************************************
// Tree path definitions
// *************************************************************************************************
/// Walk the object tree (`lumacore_objects::tree_path`) along a dot-path. Returns the children at
/// the deepest matched node — its declared children, then its numbered entries (datapools,
/// objects; never a disk read) — and the unmatched remainder (the partial segment).
fn resolve_tree_path<'a>(path: &'a str, root: &Root) -> (Vec<String>, &'a str) {
    let segments: Vec<&str> = path.split('.').collect();
    let Some((first, rest)) = segments.split_first() else {
        return (vec!["Root".to_string()], "");
    };
    if !first.eq_ignore_ascii_case("root") {
        return (vec!["Root".to_string()], first);
    }
    let mut current = TreePath::root();
    for seg in rest {
        match current.join(seg) {
            Some(next) => current = next,
            None => return (tree_children(&current, root), seg),
        }
    }
    (tree_children(&current, root), "")
}

fn tree_children(path: &TreePath, root: &Root) -> Vec<String> {
    let dot = path.to_string();
    let mut names = lumacore_objects::nav_children_at(&dot);
    if path.node() != Node::Drives
        && let Some(entries) = lumacore_objects::vfs::dynamic_children_at(&dot, root)
    {
        names.extend(entries.iter().map(|c| c.id.to_string()));
    }
    names
}

fn complete_tree_path(typed: &str, root: &Root) -> Vec<String> {
    complete_tree_path_from(typed, &[], root)
}

/// Complete a tree path relative to a given navigation position.
///
/// `cwd` is the current navigation_path (e.g. `["ShowData"]`).
/// Typed input is resolved relative to cwd unless it starts with "root" or "/".
fn complete_tree_path_from(typed: &str, cwd: &[String], root: &Root) -> Vec<String> {
    let typed_lower = typed.to_lowercase();

    // Absolute paths go through the normal resolver
    if typed_lower.starts_with("root") || typed_lower.starts_with("/") {
        return complete_tree_path_absolute(typed, root);
    }

    // Build the full dot-path by prepending cwd
    let cwd_prefix = lumacore_objects::tree_path::dot(cwd);

    let full_typed = if typed.is_empty() {
        cwd_prefix.clone()
    } else {
        format!("{cwd_prefix}.{typed}")
    };

    let (children, partial) = resolve_tree_path(&full_typed, root);
    let partial_lower = partial.to_lowercase();

    // Build prefix for the completion candidates (relative to cwd, not absolute)
    let prefix = if partial.is_empty() {
        if typed.ends_with('.') {
            typed.to_string()
        } else if typed.is_empty() {
            String::new()
        } else {
            format!("{typed}.")
        }
    } else {
        match typed.rfind('.') {
            Some(pos) => format!("{}.", &typed[..pos]),
            None => String::new(),
        }
    };

    let mut results = Vec::new();
    for child in children {
        if child.to_lowercase().starts_with(&partial_lower) {
            results.push(format!("{prefix}{child}"));
        }
    }
    results
}

/// Complete an absolute tree path (starting with "Root" or "/").
fn complete_tree_path_absolute(typed: &str, root: &Root) -> Vec<String> {
    let (children, partial) = resolve_tree_path(typed, root);
    let partial_lower = partial.to_lowercase();

    let prefix = if partial.is_empty() {
        if typed.ends_with('.') {
            typed.to_string()
        } else if typed.is_empty() {
            String::new()
        } else {
            format!("{typed}.")
        }
    } else {
        match typed.rfind('.') {
            Some(pos) => format!("{}.", &typed[..pos]),
            None => String::new(),
        }
    };

    let mut results = Vec::new();
    for child in children {
        if child.to_lowercase().starts_with(&partial_lower) {
            results.push(format!("{prefix}{child}"));
        }
    }
    results
}

/// Collect all unique attribute names from a fixture's GDTF mode.
///
/// Looks up the fixture's FixtureType by GUID, finds the correct revision and mode,
/// then returns the unique attribute_name values from all ChannelFunctions.
pub fn collect_attribute_names(fixture_id: u32, state: &AppState) -> Vec<String> {
    let fixture = match state.root.show_data.devices.dmx.patch.fixture(fixture_id) {
        Some(f) => f,
        None => return Vec::new(),
    };

    // Find the FixtureType by GUID
    let fixture_type = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture_types
        .values()
        .find(|ft| ft.guid() == fixture.mvr.fixture_type);

    let ft = match fixture_type {
        Some(ft) => ft,
        None => return Vec::new(),
    };

    // Get the revision (fall back to last if index out of bounds)
    let revision = match ft.revisions.iter().find(|r| r.meta.identity.guid == fixture.mvr.revision) {
        Some(r) => r,
        None => match ft.revisions.last() {
            Some(r) => r,
            None => return Vec::new(),
        },
    };

    let mode = match revision.modes.iter().find(|m| m.meta.identity.guid == fixture.mvr.mode) {
        Some(m) => m,
        None => match revision.modes.first() {
            Some(m) => m,
            None => return Vec::new(),
        },
    };

    // Collect unique attribute names
    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();
    for cf in &mode.channel_functions {
        if seen.insert(cf.attribute_name.clone()) {
            names.push(cf.attribute_name.clone());
        }
    }
    names
}

/// Collect attribute names from all currently selected fixtures.
fn collect_selected_attribute_names(state: &AppState) -> Vec<String> {
    let selected = state
        .current_programmer()
        .map(|p| p.selected_fixtures.clone())
        .unwrap_or_default();

    if selected.is_empty() {
        // No selection — collect from all fixtures
        let mut seen = std::collections::HashSet::new();
        let mut names = Vec::new();
        for (fixture_id, _) in state.root.show_data.devices.dmx.patch.fids() {
            for name in collect_attribute_names(fixture_id, state) {
                if seen.insert(name.clone()) {
                    names.push(name);
                }
            }
        }
        return names;
    }

    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();
    for fixture_guid in &selected {
        if let Some(fixture_id) = state.root.show_data.fixture_id_for_guid(*fixture_guid) {
            for name in collect_attribute_names(fixture_id, state) {
                if seen.insert(name.clone()) {
                    names.push(name);
                }
            }
        }
    }
    names
}

/// The command or object the line starts with, resolved like dispatch does
/// (aliases and unique prefixes), so completion works while the first word is
/// still being typed (`fixtur 1 ▸`).
fn first_word(line: &str) -> Option<Word> {
    registry::resolve(&tokens::lex_lenient(line).first()?.text).ok()
}

/// Count how many whitespace-separated tokens appear before the current word being typed.
/// word_start is the byte offset where the current word starts.
fn token_position(line: &str, word_start: usize) -> usize {
    tokens::lex_lenient(&line[..word_start]).len()
}

// *************************************************************************************************
// Public API
// *************************************************************************************************
// tui-textarea cursor().1 returns a char index, not a byte offset.
// Convert so string slices don't panic on multi-byte input.
fn char_to_byte(s: &str, char_pos: usize) -> usize {
    s.char_indices()
        .nth(char_pos)
        .map(|(byte_i, _)| byte_i)
        .unwrap_or(s.len())
}

/// Compute tab completion candidates for the given input line and cursor position.
///
/// `pos` is the cursor column as a char index (from tui-textarea `cursor().1`).
/// Returns `(word_start, candidates)` where `word_start` is the byte offset
/// of the word being completed, and `candidates` are replacement strings.
///
/// Everything comes from the registries: the first word completes from all
/// command and object names, later words from the object being addressed or
/// the argument shape the command declares (`CommandDef::args`).
pub fn complete_input(line: &str, pos: usize, state: &AppState) -> (usize, Vec<String>) {
    let pos = char_to_byte(line, pos);
    let word_start = tokens::word_start(line, pos);
    let word = &line[word_start..pos];
    let word_lower = word.to_lowercase();

    let mut candidates = Vec::new();

    if word_start == 0 {
        // Position 0: one suggestion per command or object
        candidates.extend(lumacore_objects::commands::verb_suggestions(word));
        candidates.dedup();
        return (word_start, candidates);
    }

    // A flag: the ones this statement declares (decisions §31), before anything else.
    if word.starts_with('/')
        && let Some(flags) = lumacore_objects::commands::line::Line::flags_to_offer(&line[..word_start])
    {
        let typed = &word_lower[1..];
        candidates.extend(flags.into_iter().filter(|f| f[1..].to_lowercase().starts_with(typed)));
        return (word_start, candidates);
    }

    let tok_pos = token_position(line, word_start);
    let prev = get_prev_token(line, word_start);

    // `/FixtureType ▸`: the show's types (number, then quoted name), then the library's GDTF files
    // (the one library file index, decisions §34 round 3 3).
    if prev.eq_ignore_ascii_case("/FixtureType") {
        let typed = word_lower.trim_start_matches('"');
        let types = &state.root.show_data.devices.dmx.patch.fixture_types;
        for (no, ft) in types {
            if no.to_string().starts_with(typed) {
                candidates.push(no.to_string());
            }
            if let Some(name) = ft.meta.name().filter(|n| n.to_lowercase().starts_with(typed)) {
                candidates.push(format!("\"{name}\""));
            }
        }
        let files = lumacore_objects::commands::import::gdtf_files();
        candidates.extend(files.into_iter().filter(|f| f.to_lowercase().starts_with(typed)).map(|f| format!("\"{f}\"")));
        return (word_start, candidates);
    }
    let first = first_word(line);
    let args = match first {
        Some(Word::Command(c)) => c.args,
        _ => Args::Objects,
    };

    // Objects publish their completions (terminal/objects): `Store ▸`,
    // `Store Window ▸`, `Window ▸`.
    if let Some(first) = first
        && let Some(objects) =
            lumacore_objects::objects::complete(state, first, &prev, tok_pos, word)
    {
        candidates.extend(objects);
        if args == Args::ObjectsOrTree && tok_pos == 1 {
            candidates
                .extend(tree_top_nodes().filter(|n| n.to_lowercase().starts_with(&word_lower)));
        }
        candidates.dedup();
        return (word_start, candidates);
    }

    // After `At` anywhere (`Fixture 1 At ▸`, `At ▸`): values and attributes.
    let after_value_word =
        matches!(registry::lookup_word(&prev), Some(Word::Command(c)) if c.args == Args::Value);
    if args == Args::Value || after_value_word {
        complete_value(state, line, word_start, &word_lower, &mut candidates);
        candidates.dedup();
        return (word_start, candidates);
    }

    match args {
        Args::TreeNav { up } => {
            // Tree paths relative to the cwd; never a trailing dot — the user
            // types `.` to go deeper.
            candidates.extend(complete_tree_path_from(word, &state.navigation_path, &state.root));
            // Always offer root as a global navigation target.
            if "root".starts_with(&word_lower)
                && !candidates.iter().any(|c| c.eq_ignore_ascii_case("root"))
            {
                candidates.push("root".to_string());
            }
            if up
                && !word.contains('.')
                && "..".starts_with(&word_lower)
                && !candidates.iter().any(|c| c == "..")
            {
                candidates.push("..".to_string());
            }
        }
        Args::TreeFromRoot if tok_pos == 1 => {
            let completions = complete_tree_path(word, &state.root);
            if completions.len() == 1 {
                let full = &completions[0];
                let (children, _) = resolve_tree_path(full, &state.root);
                if !children.is_empty() {
                    candidates.push(format!("{full}."));
                } else {
                    candidates.push(full.clone());
                }
            } else {
                candidates.extend(completions);
            }
            // Shorthand node names
            if !word.contains('.') {
                for s in tree_node_names() {
                    if s.to_lowercase().starts_with(&word_lower)
                        && !candidates.iter().any(|c| c.eq_ignore_ascii_case(&s))
                    {
                        candidates.push(s);
                    }
                }
            }
        }
        Args::TreeFromRoot => {}
        Args::TreeItems { from, options } => {
            if tok_pos >= from {
                candidates.extend(complete_tree_path_from(word, &state.navigation_path, &state.root));
                complete_vfs_children(state, &word_lower, &mut candidates);
            }
            for option in options {
                if option.starts_with(&word_lower) && word_lower.starts_with('-') {
                    candidates.push(option.to_string());
                }
            }
        }
        Args::Words => {
            candidates.extend(lumacore_objects::commands::verb_suggestions(word));
        }
        Args::Watches => {
            if tok_pos == 1 {
                if "all".starts_with(&word_lower) {
                    candidates.push("all".to_string());
                }
                for w in state.watches() {
                    if w.command.to_lowercase().starts_with(&word_lower) {
                        candidates.push(w.command.clone());
                    }
                }
            }
        }
        Args::Value | Args::Objects | Args::ObjectsOrTree => {
            // Object type keywords + object names from ShowData
            for ot in lumacore_objects::objects::keywords() {
                if ot.to_lowercase().starts_with(&word_lower) {
                    candidates.push(ot.to_string());
                }
            }
            for name in collect_object_names(state) {
                if name.to_lowercase().starts_with(&word_lower) {
                    candidates.push(format!("\"{name}\""));
                }
            }
        }
    }

    candidates.dedup();
    (word_start, candidates)
}

/// Values after `At`: value words, common levels, and attribute names — of
/// fixture N in `Fixture N At ▸`, else of the selection.
fn complete_value(
    state: &AppState,
    line: &str,
    word_start: usize,
    word_lower: &str,
    candidates: &mut Vec<String>,
) {
    // tidy: not-keywords (value grammar suggestions)
    for kw in ["Full", "Zero", "50", "100", "75", "25"] {
        if kw.to_lowercase().starts_with(word_lower) {
            candidates.push(kw.to_string());
        }
    }
    let tokens: Vec<&str> = line[..word_start].split_whitespace().collect();
    let fixture_id = match (first_word(line), tokens.get(1)) {
        (Some(Word::Object(o)), Some(id))
            if std::ptr::eq(o, &lumacore_objects::objects::FIXTURE) =>
        {
            id.parse::<u32>().ok()
        }
        _ => None,
    };
    let attrs = match fixture_id {
        Some(fid) => collect_attribute_names(fid, state),
        None => collect_selected_attribute_names(state),
    };
    for attr in attrs {
        if attr.to_lowercase().starts_with(word_lower) {
            candidates.push(attr);
        }
    }
}

/// Compute the fish-style ghost text hint for the current input: the rest of
/// the first completion that extends what's typed (so hints and Tab never
/// disagree).
///
/// Returns the suffix to append after the cursor, or None if no hint.
pub fn hint_input(line: &str, pos: usize, state: &AppState) -> Option<String> {
    let byte_pos = char_to_byte(line, pos);
    if byte_pos < line.len() || line.is_empty() {
        return None;
    }
    let (word_start, candidates) = complete_input(line, pos, state);
    let typed = &line[word_start..byte_pos];
    let typed_lower = typed.to_lowercase();
    let typed_chars = typed.chars().count();
    candidates
        .into_iter()
        .find(|c| c.to_lowercase().starts_with(&typed_lower) && c.chars().count() > typed_chars)
        .map(|c| c.chars().skip(typed_chars).collect())
}

// *************************************************************************************************
// Internal helpers
// *************************************************************************************************
/// Collect all fixture type, fixture, and universe names for generic completions.
fn collect_object_names(state: &AppState) -> Vec<String> {
    let mut names = Vec::new();

    for ft in state.root.show_data.devices.dmx.patch.fixture_types.values() {
        if let Some(name) = ft.name() {
            names.push(name.to_string());
        }
    }
    for f in state.root.show_data.devices.dmx.patch.fixtures.values() {
        if let Some(name) = f.name() {
            names.push(name.to_string());
        }
    }
    for u in state.root.show_data.devices.dmx.patch.universes.values() {
        if let Some(name) = u.name() {
            names.push(name.to_string());
        }
    }
    names
}

/// Get the token immediately before the word being typed.
fn get_prev_token(line: &str, word_start: usize) -> String {
    tokens::lex_lenient(&line[..word_start])
        .pop()
        .map(|t| t.text)
        .unwrap_or_default()
}

/// Complete with VFS dynamic children (object IDs/names) at the current cwd.
fn complete_vfs_children(state: &AppState, word_lower: &str, candidates: &mut Vec<String>) {
    let cwd = lumacore_objects::tree_path::dot(&state.navigation_path);
    if let Some(dyn_children) = lumacore_objects::vfs::dynamic_children_at(&cwd, &state.root) {
        for dc in &dyn_children {
            let id_str = dc.id.to_string();
            if id_str.starts_with(word_lower) {
                candidates.push(id_str);
            }
            // Also complete by name
            if let Some(name) = dc.display_name.split('"').nth(1)
                && name.to_lowercase().starts_with(word_lower)
            {
                candidates.push(format!("\"{name}\""));
            }
        }
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
