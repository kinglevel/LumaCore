// ███████╗██╗   ██╗ ██████╗  ██████╗ ███████╗███████╗████████╗
// ██╔════╝██║   ██║██╔════╝ ██╔════╝ ██╔════╝██╔════╝╚══██╔══╝
// ███████╗██║   ██║██║  ███╗██║  ███╗█████╗  ███████╗   ██║
// ╚════██║██║   ██║██║   ██║██║   ██║██╔══╝  ╚════██║   ██║
// ███████║╚██████╔╝╚██████╔╝╚██████╔╝███████╗███████║   ██║
// ╚══════╝ ╚═════╝  ╚═════╝  ╚═════╝ ╚══════╝╚══════╝   ╚═╝
// FILE: suggest

//! The one suggestion engine (decisions §30 "12b2 hardening"): what the
//! command line can take at the cursor, as typed items, ranked on the
//! station. The bottom bar's list, the full-screen palette, the TUI's
//! overlay, the sweep and the fuzz targets all ask here.
//!
//! Ranking, in this order:
//! 1. **grammar** — only what is valid at the cursor ([`grammar`], from the
//!    registries; never a hand-written word list);
//! 2. the **fuzzy score** of the typed fragment ([`nucleo_matcher`]);
//! 3. **usage** — the acting user's `ShowData.Users.{n}.Usage` (decayed);
//! 4. a fixed tie-break: kind, then number, then title.
//!
//! Picking never runs code in a client: an item writes the command line
//! (`insert` over `replace`) or its actions are command lines.
//!
//! Owns:          the suggestion engine: items at the cursor, their ranking, the ghost, the mode, Usage recording.
//! Doesn't own:   the grammar of what may follow (grammar.rs), drawing the items (the kit's <luma-suggest>, the TUI).
//! Entry points:  suggest, ghost, mode, complete_input, hint_input, record_usage, register.
//! Decisions:     §30 "12b2", §34 25.9.

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog: its hook, after the crates it builds on (decisions §34 21,
/// 25.13).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_objects::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(HOOKS);
}

/// What the suggestion engine adds to how a line runs: a user's show line ranks their words.
pub static HOOKS: &[lumacore_objects::commands::hooks::Hook] = &[lumacore_objects::commands::hooks::Hook::Used(record_now)];

/// A line went through now (`Hook::Used`).
fn record_now(state: &mut AppState, line: &str) {
    record_usage(state, line, now_ms());
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

pub mod grammar;
#[cfg(test)]
mod tests;

pub use grammar::{collect_attribute_names, complete_input, hint_input};

use lumacore_tree::usage::word_key;
use lumacore_tree::SuggestionsMode;
use lumacore_objects::commands::registry::Rights;
use lumacore_wire::{ItemAction, ItemKind, ItemStatus, SuggestItem, Suggestions, Surface};
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use lumacore_objects::AppState;
use lumacore_objects::commands::registry::{self, Args, Word};
use lumacore_objects::commands::tokens;
use lumacore_objects::objects::{self, ObjectDef};
use lumacore_objects::vfs::DynChild;

/// What to suggest for.
pub struct Query<'a> {
    pub line: &'a str,
    /// The cursor, in chars.
    pub cursor: usize,
    pub surface: Surface,
    /// The session's command history, oldest first.
    pub history: &'a [String],
    /// Unix ms, for usage decay (tests pass a fixed time).
    pub now_ms: u64,
}

/// Items in a reply: the bar shows a short list, the palette a long one.
const BAR_ITEMS: usize = 12;
const PALETTE_ITEMS: usize = 50;
/// History lines offered at most.
const HISTORY_ITEMS: usize = 3;
/// Instances offered per object keyword at the start of a line (`Gr` →
/// `Group 3`, `Group 1` …).
const INSTANCES_PER_KEYWORD: usize = 5;

/// The acting user's profile setting.
pub fn mode(state: &AppState) -> SuggestionsMode {
    state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| p.settings.preferences.suggestions)
        .unwrap_or_default()
}

/// Wall-clock ms (the one copy is the base's, decisions §34 25.14).
pub use lumacore_base::clock::now_ms;

fn char_to_byte(s: &str, char_pos: usize) -> usize {
    s.char_indices().nth(char_pos).map(|(i, _)| i).unwrap_or(s.len())
}

fn byte_to_char(s: &str, byte: usize) -> usize {
    s[..byte.min(s.len())].chars().count()
}

/// Where the cursor is in the grammar.
struct Cursor<'a> {
    line: &'a str,
    /// Byte offsets of the word being typed.
    word_start: usize,
    pos: usize,
    word: &'a str,
    /// Words before it.
    tok_pos: usize,
    prev: String,
    first: Option<Word>,
}

impl<'a> Cursor<'a> {
    fn new(line: &'a str, cursor: usize) -> Self {
        let pos = char_to_byte(line, cursor);
        let word_start = tokens::word_start(line, pos);
        let before = tokens::lex_lenient(&line[..word_start]);
        Cursor {
            line,
            word_start,
            pos,
            word: &line[word_start..pos],
            tok_pos: before.len(),
            prev: before.last().map(|t| t.text.clone()).unwrap_or_default(),
            first: tokens::lex_lenient(line)
                .first()
                .and_then(|t| registry::resolve(&t.text).ok()),
        }
    }

    fn args(&self) -> Args {
        match self.first {
            Some(Word::Command(c)) => c.args,
            _ => Args::Objects,
        }
    }

    /// The part of the word fixed by the grammar (a tree path up to its last
    /// dot) and the fragment the fuzzy match runs on.
    fn stem_and_fragment(&self) -> (&'a str, &'a str) {
        // A name being typed (`"Ev`) matches without its quotes.
        match self.word.rfind('.') {
            Some(dot) => (&self.word[..=dot], &self.word[dot + 1..]),
            None => ("", self.word.trim_matches('"')),
        }
    }
}

/// One candidate before it is typed and ranked.
struct Candidate {
    insert: String,
    /// What the fuzzy match runs against.
    names: Vec<String>,
    item: SuggestItem,
    /// The usage key (an object GUID or `word:<Label>`).
    usage: Option<String>,
}

/// Suggestions for `query` as the acting user sees them.
pub fn suggest(state: &AppState, query: &Query) -> Suggestions {
    let line = query.line;
    let cursor = query.cursor.min(line.chars().count());
    let at = Cursor::new(line, cursor);
    let mut out = Suggestions {
        seq: 0,
        surface: query.surface,
        line: line.to_string(),
        cursor: cursor as u32,
        replace: [byte_to_char(line, at.word_start) as u32, cursor as u32],
        hint: None,
        syntax: at.first.map(|w| w.doc().syntax).filter(|s| !s.is_empty()).map(str::to_string),
        items: Vec::new(),
        total: 0,
        mode: mode(state).label().to_string(),
    };

    let mut history = history_items(&at, query);
    let at_end = at.pos == line.len();
    if line.trim().is_empty() && query.surface == Surface::Bar {
        return out;
    }

    let (_, fragment) = at.stem_and_fragment();
    let mut candidates = if line.trim().is_empty() { Vec::new() } else { grammar_candidates(state, &at) };
    let role = role(state);
    let usage = state.root.show_data.user(state.effective_user_name()).map(|u| &u.usage);

    let mut matcher = Matcher::new({
        let mut c = Config::DEFAULT;
        c.prefer_prefix = true;
        c
    });
    let atom = (!fragment.is_empty()).then(|| {
        Atom::new(fragment, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy, false)
    });
    let mut buf = Vec::new();
    let mut ranked: Vec<SuggestItem> = Vec::new();
    for mut c in candidates.drain(..) {
        let fuzzy = match &atom {
            None => Some(0u16),
            Some(atom) => c
                .names
                .iter()
                .filter_map(|n| atom.score(Utf32Str::new(n, &mut buf), &mut matcher))
                .max(),
        };
        let Some(fuzzy) = fuzzy else { continue };
        let used = c
            .usage
            .as_deref()
            .and_then(|k| usage.map(|u| u.rank(k, query.now_ms)))
            .unwrap_or(0.0);
        if let (Some(key), Some(u)) = (c.usage.as_deref(), usage)
            && let Some(e) = u.get(key)
            && c.item.kind == ItemKind::Object
        {
            c.item.subtitle = join(&c.item.subtitle, &format!("used {}×", e.count));
        }
        c.item.score = i64::from(fuzzy) * 1_000_000 + ((used * 1000.0).round() as i64).min(999_999);
        rights_and_status(&mut c.item, role.as_ref());
        c.item.insert = c.insert;
        ranked.push(c.item);
    }
    // Number for the tie-break, from the title (`Group 12` → 12).
    let number = |i: &SuggestItem| {
        i.title
            .split_whitespace()
            .find_map(|w| w.parse::<u32>().ok())
            .unwrap_or(u32::MAX)
    };
    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.kind.cmp(&b.kind))
            .then(number(a).cmp(&number(b)))
            // An object by number before the same object by name.
            .then(a.insert.starts_with('"').cmp(&b.insert.starts_with('"')))
            .then(a.title.cmp(&b.title))
            .then(a.insert.cmp(&b.insert))
    });
    ranked.dedup_by(|a, b| a.insert.eq_ignore_ascii_case(&b.insert) && a.kind == b.kind);
    // One row per object: its number and its name both complete to it.
    let mut seen_guids: Vec<String> = Vec::new();
    ranked.retain(|i| match &i.guid {
        Some(g) if seen_guids.contains(g) => false,
        Some(g) => {
            seen_guids.push(g.clone());
            true
        }
        None => true,
    });

    // Ghost text: the newest history line that extends what is typed, else
    // the first item that extends the word.
    if at_end && !line.is_empty() {
        out.hint = history
            .first()
            .map(|h| h.insert.chars().skip(at.word.chars().count()).collect::<String>())
            .filter(|s: &String| !s.is_empty())
            .or_else(|| {
                let typed = at.word.to_lowercase();
                ranked
                    .iter()
                    .filter(|i| i.kind != ItemKind::History && i.disabled.is_none())
                    .map(|i| i.insert.trim_end())
                    .find(|ins| ins.to_lowercase().starts_with(&typed) && ins.len() > typed.len())
                    .map(|ins| ins.chars().skip(at.word.chars().count()).collect())
            });
    }

    // A number right after a word with a default object (`S 1` → `Store Cue 1`): the first
    // item names it in full, and the items replace from the line's start so it can.
    if let Some(full) = default_object_line(&at) {
        let prefix = &line[..at.word_start];
        for i in ranked.iter_mut().chain(history.iter_mut()) {
            i.insert = format!("{prefix}{}", i.insert);
        }
        out.replace[0] = 0;
        ranked.insert(
            0,
            SuggestItem {
                kind: ItemKind::Object,
                title: full.clone(),
                subtitle: "the default object".to_string(),
                icon: icon(ItemKind::Object).to_string(),
                insert: full,
                guid: None,
                actions: Vec::new(),
                status: ItemStatus::Ready,
                disabled: None,
                section: section(ItemKind::Object).to_string(),
                score: i64::MAX / 2,
            },
        );
    }

    let mut items = history;
    items.extend(ranked);
    out.total = items.len() as u32;
    items.truncate(match query.surface {
        Surface::Bar => BAR_ITEMS,
        Surface::Palette => PALETTE_ITEMS,
    });
    if query.surface == Surface::Bar {
        for i in &mut items {
            i.actions.clear();
        }
    }
    out.items = items;
    out
}

/// `Store 1` / `S 1` → `Store Cue 1`: the line with the word's default object written out, when the
/// word at the cursor is a number right after it.
fn default_object_line(at: &Cursor) -> Option<String> {
    let Some(Word::Command(c)) = at.first else { return None };
    let object = c.default_object?;
    let number = !at.word.is_empty() && at.word.chars().all(|ch| ch.is_ascii_digit() || ch == '.');
    (at.tok_pos == 1 && number && at.pos == at.line.len()).then(|| format!("{} {object} {}", c.doc.display, at.word))
}

/// The ghost text alone (the TUI draws it and Right takes it).
pub fn ghost(state: &AppState, line: &str, cursor: usize, history: &[String]) -> Option<String> {
    if line.is_empty() || cursor < line.chars().count() {
        return None;
    }
    suggest(
        state,
        &Query {
            line,
            cursor,
            surface: Surface::Bar,
            history,
            now_ms: now_ms(),
        },
    )
    .hint
}

fn join(a: &str, b: &str) -> String {
    if a.is_empty() { b.to_string() } else { format!("{a} · {b}") }
}

fn role(state: &AppState) -> Option<Rights> {
    Some(lumacore_objects::rights::level(state)).filter(|r| *r != Rights::Internal)
}

/// Planned words and words the user may not run (decisions §19, §29): still
/// listed, dimmed, with the reason.
fn rights_and_status(item: &mut SuggestItem, role: Option<&Rights>) {
    if !matches!(item.kind, ItemKind::Verb | ItemKind::Keyword) || item.title.contains(' ') {
        return;
    }
    if let Some(role) = role
        && let Err(why) = lumacore_objects::rights::authorize(*role, &item.title)
    {
        item.disabled = Some("E-RIGHTS-001".to_string());
        item.subtitle = why;
    }
}

// *************************************************************************************************
// History
// *************************************************************************************************
/// History lines that start with what is typed (newest first), as items
/// replacing the word at the cursor.
fn history_items(at: &Cursor, query: &Query) -> Vec<SuggestItem> {
    let typed = &at.line[..at.pos];
    if at.pos != at.line.len() || (typed.trim().is_empty() && query.surface == Surface::Bar) {
        return Vec::new();
    }
    let lower = typed.to_lowercase();
    let from = byte_to_char(at.line, at.word_start);
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for (i, h) in query.history.iter().rev().enumerate() {
        let hl = h.to_lowercase();
        if !hl.starts_with(&lower) || hl.len() <= lower.len() || seen.contains(&hl) {
            continue;
        }
        seen.push(hl);
        let insert: String = h.chars().skip(from).collect();
        out.push(SuggestItem {
            kind: ItemKind::History,
            title: insert.clone(),
            subtitle: format!("last: {h}"),
            icon: icon(ItemKind::History).to_string(),
            insert,
            guid: None,
            actions: Vec::new(),
            status: ItemStatus::Ready,
            disabled: None,
            section: section(ItemKind::History).to_string(),
            score: i64::MAX - i as i64,
        });
        if out.len() == HISTORY_ITEMS {
            break;
        }
    }
    out
}

// *************************************************************************************************
// Grammar → typed candidates
// *************************************************************************************************
/// Everything valid at the cursor whatever the fragment (the fuzzy match
/// filters), typed.
fn grammar_candidates(state: &AppState, at: &Cursor) -> Vec<Candidate> {
    let (stem, _) = at.stem_and_fragment();
    // A flag or option: the grammar's own prefix match.
    let raw: Vec<String> = if at.word.starts_with('-') || at.word.starts_with('/') {
        complete_input(at.line, byte_to_char(at.line, at.pos), state).1
    } else {
        let probe = format!("{}{stem}", &at.line[..at.word_start]);
        complete_input(&probe, probe.chars().count(), state).1
    };
    // An object's children once for every candidate of it (a name is found among them; on a
    // rig of a thousand fixtures a walk per candidate was the whole second a Tab took).
    let kids: Option<Vec<DynChild>> = objects::find(&at.prev)
        .and_then(|obj| obj.tree.as_ref().map(|tree| tree.children_in(&state.root, datapool_of(state, obj))));
    let mut out: Vec<Candidate> = raw.into_iter().map(|c| typed(state, at, stem, c, kids.as_deref())).collect();
    if at.word_start == 0 {
        out.extend(first_word_instances(state, at.word));
    }
    out
}

/// One grammar candidate, typed by where it stands (`kids`: the previous word's object's
/// children, when it is one).
fn typed(state: &AppState, at: &Cursor, stem: &str, raw: String, kids: Option<&[DynChild]>) -> Candidate {
    let tail = if raw.len() >= stem.len() && raw[..stem.len()].eq_ignore_ascii_case(stem) {
        raw[stem.len()..].to_string()
    } else {
        raw.clone()
    };
    let space = if raw.ends_with('.') { "" } else { " " };
    let insert = format!("{raw}{space}");
    // A flag or option.
    if raw.starts_with('-') || raw.starts_with('/') {
        return plain(ItemKind::Flag, raw.clone(), String::new(), insert, vec![tail]);
    }
    // A word of the namespace: the first word, `Help ▸`, `Store ▸`.
    let word_place = at.word_start == 0
        || at.args() == Args::Words
        || matches!(at.first, Some(Word::Command(c)) if c.object_verb.is_some() && at.tok_pos == 1);
    if word_place && let Some(word) = registry::lookup_word(raw.trim_matches('"')) {
        return word_candidate(at, word, raw, insert);
    }
    // An object of the show: `Group ▸`, `Store Group ▸`.
    if let Some(obj) = objects::find(&at.prev)
        && (raw.parse::<u32>().is_ok() || raw.starts_with('"'))
    {
        return instance_candidate(state, obj, &raw, insert, vec![raw.trim_matches('"').to_string()], kids);
    }
    let tree_args = matches!(at.args(), Args::TreeNav { .. } | Args::TreeFromRoot | Args::TreeItems { .. });
    if tree_args || raw.contains('.') {
        return plain(ItemKind::TreeNode, raw.clone(), String::new(), insert, vec![tail]);
    }
    if let Some(word) = registry::lookup_word(&raw) {
        return word_candidate(at, word, raw, insert);
    }
    plain(ItemKind::Value, raw.clone(), String::new(), insert, vec![tail])
}

fn plain(kind: ItemKind, title: String, subtitle: String, insert: String, names: Vec<String>) -> Candidate {
    Candidate {
        insert,
        names,
        item: SuggestItem {
            kind,
            title,
            subtitle,
            icon: icon(kind).to_string(),
            insert: String::new(),
            guid: None,
            actions: Vec::new(),
            status: ItemStatus::Ready,
            disabled: None,
            section: section(kind).to_string(),
            score: 0,
        },
        usage: None,
    }
}

/// A command or object word.
fn word_candidate(at: &Cursor, word: Word, raw: String, insert: String) -> Candidate {
    let kind = match word {
        Word::Command(_) => ItemKind::Verb,
        Word::Object(_) => ItemKind::Keyword,
    };
    let doc = word.doc();
    // `Store ▸` offers objects whose Store is only planned.
    let planned = word.is_planned()
        || matches!((at.first, word), (Some(Word::Command(c)), Word::Object(o))
            if at.tok_pos == 1 && c.object_verb.is_some_and(|v| !o.supports(v) && o.is_planned(v)));
    let subtitle = if planned {
        "isn't built yet (planned)".to_string()
    } else if !doc.syntax.is_empty() {
        doc.syntax.to_string()
    } else {
        first_sentence(doc.description)
    };
    let mut names: Vec<String> = word.names().into_iter().map(str::to_string).collect();
    names.insert(0, word.label().to_string());
    let mut c = plain(kind, raw, subtitle, insert, names);
    if planned {
        c.item.status = ItemStatus::Planned;
    }
    c.usage = Some(word_key(word.label()));
    c
}

fn first_sentence(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    let end = line.find(". ").map_or(line.len(), |i| i + 1);
    let s = &line[..end];
    if s.chars().count() > 72 {
        format!("{}…", s.chars().take(71).collect::<String>())
    } else {
        s.to_string()
    }
}

/// The datapool an object's tree lives in for the acting user.
fn datapool_of(state: &AppState, obj: &ObjectDef) -> Option<u32> {
    obj.tree
        .as_ref()
        .and_then(|t| t.datapool.as_ref())
        .map(|_| state.selected_datapool())
}

/// An object of the show (`Group 3 "Even"`): title, name, GUID, actions.
fn instance_candidate(
    state: &AppState,
    obj: &'static ObjectDef,
    raw: &str,
    insert: String,
    names: Vec<String>,
    kids: Option<&[DynChild]>,
) -> Candidate {
    let found = obj.tree.as_ref().and_then(|tree| {
        let dp = datapool_of(state, obj);
        let id = raw.parse::<u32>().ok().or_else(|| {
            let name = raw.trim_matches('"');
            let named = |c: &DynChild| {
                c.display_name.trim_matches('"').eq_ignore_ascii_case(name)
                    || c.display_name.split('"').nth(1).is_some_and(|n| n.eq_ignore_ascii_case(name))
            };
            match kids {
                Some(kids) => kids.iter().find(|c| named(c)).map(|c| c.id),
                None => tree.children_in(&state.root, dp).into_iter().find(|c| named(c)).map(|c| c.id),
            }
        })?;
        match tree.object_in(&state.root, dp, id)? {
            lumacore_objects::vfs::NodeView::Object { guid, name, .. } => Some((id, guid, name)),
            _ => None,
        }
    });
    let (title, subtitle, guid) = match &found {
        Some((id, guid, name)) => (
            format!("{} {id}", obj.keyword),
            name.as_deref().map(|n| format!("\"{n}\"")).unwrap_or_default(),
            Some(guid.clone()),
        ),
        None => (format!("{} {}", obj.keyword, raw), String::new(), None),
    };
    let mut names = names;
    names.push(title.clone());
    let mut c = plain(ItemKind::Object, title, subtitle, insert, names);
    if let Some((id, _, _)) = &found {
        c.item.actions = actions(obj, *id);
    }
    c.usage = guid.clone();
    c.item.guid = guid;
    c
}

/// What an object can do, as command lines (the palette's actions): its
/// built verbs, each through the command that is that verb.
fn actions(obj: &ObjectDef, id: u32) -> Vec<ItemAction> {
    let target = format!("{} {id}", obj.keyword);
    let mut out: Vec<ItemAction> = Vec::new();
    for (label, verb) in action_words(obj) {
        let line = match verb {
            // The bare line runs it (`Group 3` = SelectFixtures Group 3).
            None => target.clone(),
            Some(verb) => format!("{verb} {target}"),
        };
        if !out.iter().any(|a| a.line == line) {
            out.push(ItemAction { label: label.clone(), line });
        }
    }
    out
}

/// An object's actions as (label, the verb before the target — none for the bare line), found
/// in the registry once per object: the registry is fixed for the process, and a candidate per
/// fixture of a rig would otherwise scan it for every verb.
fn action_words(obj: &ObjectDef) -> &'static [(String, Option<String>)] {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<&'static str, &'static [(String, Option<String>)]>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(found) = cache.lock().ok().and_then(|m| m.get(obj.keyword).copied()) {
        return found;
    }
    let mut words: Vec<(String, Option<String>)> = Vec::new();
    for (verb, _) in obj.verbs {
        if *verb == obj.default_verb {
            words.push((verb.label().to_string(), None));
        } else if let Some(cmd) =
            lumacore_objects::commands::registry::all().find(|c| c.object_verb == Some(*verb) && c.status == registry::Status::Ready)
        {
            words.push((cmd.label().to_string(), Some(cmd.label().to_string())));
        }
    }
    let leaked: &'static [(String, Option<String>)] = Box::leak(words.into_boxed_slice());
    if let Ok(mut m) = cache.lock() {
        m.insert(obj.keyword, leaked);
    }
    leaked
}

/// At the start of a line, an object keyword the fragment matches brings a
/// few of its objects (`Gr` → `Group 3`), most used first.
fn first_word_instances(state: &AppState, word: &str) -> Vec<Candidate> {
    if word.chars().count() < 2 {
        return Vec::new();
    }
    let lower = word.to_lowercase();
    let usage = state.root.show_data.user(state.effective_user_name()).map(|u| &u.usage);
    let mut out = Vec::new();
    for obj in objects::all() {
        let Some(tree) = &obj.tree else { continue };
        if !lumacore_objects::objects::has_bare(obj)
            || !std::iter::once(obj.keyword)
                .chain(obj.aliases.iter().copied())
                .any(|n| n.len() > 1 && n.to_lowercase().starts_with(&lower))
        {
            continue;
        }
        let dp = datapool_of(state, obj);
        let ids = tree.children_in(&state.root, dp).into_iter().map(|c| c.id);
        // GUIDs only matter when the user has used something.
        let mut children: Vec<(u32, String)> = if usage.is_some_and(|u| !u.entries.is_empty()) {
            ids.filter_map(|id| match tree.object_in(&state.root, dp, id)? {
                lumacore_objects::vfs::NodeView::Object { guid, .. } => Some((id, guid)),
                _ => None,
            })
            .collect()
        } else {
            ids.take(INSTANCES_PER_KEYWORD).map(|id| (id, String::new())).collect()
        };
        let now = now_ms();
        children.sort_by(|a, b| {
            let ua = usage.map_or(0.0, |u| u.rank(&a.1, now));
            let ub = usage.map_or(0.0, |u| u.rank(&b.1, now));
            ub.total_cmp(&ua).then(a.0.cmp(&b.0))
        });
        for (id, _) in children.into_iter().take(INSTANCES_PER_KEYWORD) {
            let insert = format!("{} {id} ", obj.keyword);
            let names = vec![obj.keyword.to_string()];
            out.push(instance_candidate(state, obj, &id.to_string(), insert, names, None));
        }
    }
    out
}

fn icon(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::History => "history",
        ItemKind::Verb => "play",
        ItemKind::Keyword => "command",
        ItemKind::Object => "box",
        ItemKind::Value => "hash",
        ItemKind::TreeNode => "folder-open",
        ItemKind::Flag => "flag",
    }
}

fn section(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::History => "History",
        ItemKind::Verb | ItemKind::Keyword => "Commands",
        ItemKind::Object => "Objects",
        ItemKind::Value => "Values",
        ItemKind::TreeNode => "Tree",
        ItemKind::Flag => "Options",
    }
}

// *************************************************************************************************
// Usage
// *************************************************************************************************
/// Count what a line that ran used (decisions §30 "12b2"): its words
/// (`word:Store`) and the objects it names (by GUID), on the acting user's
/// `Usage`. Called by dispatch inside the change pass, so it replicates and
/// is saved — and never undone (`ShowSubtree::Usage`).
pub fn record_usage(state: &mut AppState, line: &str, now_ms: u64) {
    let mut keys: Vec<String> = Vec::new();
    for statement in tokens::statements(line) {
        let words = tokens::words(&tokens::lex_lenient(&statement));
        if let Some(first) = words.first().and_then(|w| registry::resolve(w).ok()) {
            keys.push(word_key(first.label()));
        }
        for pair in words.windows(2) {
            let Some(obj) = objects::find(&pair[0]) else { continue };
            if obj.tree.is_none() {
                continue;
            }
            let raw = if pair[1].parse::<u32>().is_ok() { pair[1].clone() } else { format!("\"{}\"", pair[1]) };
            let c = instance_candidate(state, obj, &raw, String::new(), Vec::new(), None);
            if let Some(guid) = c.item.guid {
                keys.push(guid);
            }
        }
    }
    if keys.is_empty() {
        return;
    }
    let user = state.effective_user_name().to_string();
    if let Some(u) = state.root.show_data.user_mut(&user) {
        for key in keys {
            u.usage.record(&key, now_ms);
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

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    // The tree shell's words too: completion takes whatever the catalog holds (`cd …`).
    lumacore_registry::install_with(|r| {
        crate::register(r);
        lumacore_cmd_treeshell::register(r);
        lumacore_sacn::register(r);
        lumacore_artnet::register(r);
        lumacore_osc::register(r);
    });
}
