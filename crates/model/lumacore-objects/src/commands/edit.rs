//! Edit mode command handler for the LumaCore REPL.
//!
//! Edit <type> <id> — enter edit mode for an object, scoping context to that object.
//! In edit mode (ReplContext::Custom("Group 1")), sub-commands apply to the target:
//!   Name/Label "New Name" — rename the object
//!   Add Fixture <id>      — add fixture to group (groups only)
//!   Remove Fixture <id>   — remove fixture from group (groups only)
//!   exit / .. / q        — return to parent context

use nu_ansi_term::Color;

use crate::AppState;
use crate::ReplContext;
use crate::errors::render_code;
use crate::text::{PropertySheet, render_success};
use crate::DispatchResult;
use crate::claims::Claims;
use crate::navigation::Navigation;

/// Handle: Edit <object> <id> — the object passes its editor (`edit_group`,
/// `edit_fixture`). Sets context to ReplContext::Custom("<Object> <id>") and
/// displays the property sheet.
pub fn edit_with(
    state: &mut AppState,
    args: &[&str],
    editor: fn(&mut AppState, u32) -> DispatchResult,
) -> DispatchResult {
    let Some(raw) = args.first() else {
        return DispatchResult::Output(crate::Reply::error("Usage: Edit <object> <id>", "E-CMD-006"));
    };
    let Ok(id) = raw.parse::<u32>() else {
        return DispatchResult::Output(crate::Reply::error(format!(
            "Invalid ID '{raw}'. Expected a number."
        ), "E-CMD-008"));
    };
    let before = state.context.clone();
    let result = editor(state, id);
    // Opening an editor claims the object (decisions §21 "6d"): another
    // user's object stays closed (Claim it to ask them).
    if let ReplContext::Custom(context) = state.context.clone()
        && state.context != before
        && let label = owner_of(&context).1.to_string()
        && let Err(e) = state.claim_for_editor(&label)
    {
        state.context = before;
        let message = format!("{e} Claim {label} asks them for it.");
        // A claim conflict arrives already coded (E-CLAIM-001).
        return DispatchResult::Output((crate::errors::reply(message, "E-CMD-007")).into());
    }
    result
}

pub fn edit_fixture(state: &mut AppState, id: u32) -> DispatchResult {
    let fixture = match state.root.show_data.devices.dmx.patch.fixture(id) {
        Some(f) => f.clone(),
        None => return DispatchResult::Output(crate::Reply::error(format!("Fixture {id} not found."), "E-PATCH-006")),
    };

    let name = fixture.meta.name().unwrap_or("<unnamed>").to_string();
    let (universe_guid, start_addr, footprint) = fixture
        .primary_break()
        .map(|b| (b.universe_id, b.start_address, b.channel_count))
        .unwrap_or((uuid::Uuid::nil(), 0, 0));
    let universe_int_id = state
        .root
        .show_data
        .universe_id_for_guid(universe_guid)
        .unwrap_or(0);

    let location = format!(
        "Universe {} Addr {:03}–{:03}",
        Color::Cyan.paint(universe_int_id.to_string()),
        start_addr,
        start_addr + footprint.saturating_sub(1)
    );

    let mut sheet = PropertySheet::new(&format!("Fixture {id}"))
        .field("ID", &Color::Cyan.paint(id.to_string()).to_string())
        .field("Name", &name)
        .field("Location", &location);

    if let Some(ft) = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture_types
        .values()
        .find(|ft| ft.meta.guid() == fixture.mvr.fixture_type)
    {
        let ft_name = ft.meta.name().unwrap_or("<unnamed>").to_string();
        sheet = sheet.field("Type", &format!("{} ({})", ft_name, ft.manufacturer));

        if let Some((_, mode)) = ft.mode_of(fixture.mvr.revision, fixture.mvr.mode) {
            let mode_name = mode.meta.name().unwrap_or("<unnamed>").to_string();
            sheet = sheet.field("Mode", &mode_name);
        }
    }

    sheet = sheet.separator().line("Edit mode: Name \"...\", exit");

    // Switch into edit mode context
    state.context = ReplContext::Custom(format!("Fixture {id}"));
    DispatchResult::Output(sheet.build().into())
}

/// Dispatch sub-commands when in a Custom context (edit mode).
///
/// Handles: Label (and its aliases), Add Fixture, Remove Fixture, Exit and
/// `..`, and value words in a fixture (`At 50`). Words resolve through the
/// registry like everywhere else; the edited object comes from the context
/// ("Group 1" → the Group object). For anything else, returns None so the
/// caller falls through to normal dispatch.
pub fn dispatch_edit(state: &mut AppState, tokens: &[&str]) -> Option<DispatchResult> {
    use crate::commands::registry::{self, Word};

    let first = *tokens.first()?;
    let ReplContext::Custom(ctx) = state.context.clone() else {
        return None;
    };
    let command = match registry::resolve(first) {
        Ok(Word::Command(c)) => Some(c.verb),
        _ => None,
    };
    let is = |verb: &str| command == Some(verb);

    // Exit edit mode — go up one level in tree
    if first == ".." || is("exit") {
        // The editor closes: its claim goes.
        state.release_editor_claims();
        if !state.navigation_path.is_empty() {
            if let Some(obj) = state.claimable_at_path() {
                let _ = state.unclaim(&obj);
            }
            state.navigation_path.pop();
        }
        state.context = state.context_from_path();
        return Some(DispatchResult::Output((render_success(&format!(
            "Left edit mode. {}",
            state.path_string()
        ))).into()));
    }

    // The edited object: context "Group 1" / "Fixture 1" / "DataPool 2 Group 3".
    let (datapool, ctx) = owner_of(&ctx);
    if datapool.is_some() {
        let previous = std::mem::replace(&mut state.datapool_override, datapool);
        let result = edit_in(state, tokens, ctx, is);
        state.datapool_override = previous;
        return result;
    }
    edit_in(state, tokens, ctx, is)
}

/// An editor's context split into its owner DataPool (owner first,
/// `DataPool 2 Group 3`) and the edited object (`Group 3`).
fn owner_of(context: &str) -> (Option<u32>, &str) {
    let mut words = context.splitn(3, ' ');
    match (words.next(), words.next(), words.next()) {
        (Some(owner), Some(no), Some(rest))
            if crate::objects::find(owner)
                .is_some_and(|o| std::ptr::eq(o, &crate::objects::DATAPOOL)) =>
        {
            match no.parse() {
                Ok(no) => (Some(no), rest),
                Err(_) => (None, context),
            }
        }
        _ => (None, context),
    }
}

/// Edit-mode words on the edited object `ctx` ("Group 1" / "Fixture 1").
fn edit_in(
    state: &mut AppState,
    tokens: &[&str],
    ctx: &str,
    is: impl Fn(&str) -> bool,
) -> Option<DispatchResult> {
    use crate::objects::{self, Verb};
    let first = *tokens.first()?;
    let (keyword, id) = ctx.split_once(' ')?;
    let object = objects::find(keyword)?;
    let id_num: u32 = id.parse().ok()?;
    let is_group = std::ptr::eq(object, &super::groups::GROUP);
    let is_fixture = std::ptr::eq(object, &crate::objects::FIXTURE);

    // Label — rename the object, through its own Label handler.
    if is("label") {
        if tokens.len() < 2 {
            return Some(DispatchResult::Output(crate::Reply::error(
                "Usage: Label \"New Label\"",
                "E-CMD-006",
            )));
        }
        let handler = object.handler(Verb::Label)?;
        let mut args = vec![id];
        args.extend_from_slice(&tokens[1..]);
        return Some(handler(state, &args));
    }

    // Add / Remove Fixture <id> — groups only
    if is("add") || is("remove") {
        let adding = is("add");
        let label = if adding { "Add" } else { "Remove" };
        if !is_group {
            return Some(DispatchResult::Output(crate::Reply::error(format!(
                "{label} Fixture is only available in Group edit mode."
            ), "E-CMD-010")));
        }
        let names_fixture = tokens
            .get(1)
            .and_then(|w| objects::find(w))
            .is_some_and(|o| std::ptr::eq(o, &crate::objects::FIXTURE));
        let fixture_id = tokens.get(2).and_then(|s| s.parse::<u32>().ok());
        let (true, Some(fixture_id)) = (names_fixture, fixture_id) else {
            return Some(DispatchResult::Output(crate::Reply::error(format!(
                "Usage: {label} Fixture <id>"
            ), "E-CMD-006")));
        };
        return Some(DispatchResult::Output((super::groups::membership(
            state, id_num, fixture_id, adding,
        )).into()));
    }

    // In a fixture: bare "At <val>" -> Fixture N At <val>
    if is_fixture && super::control::is_fixture_value_word(first) {
        let new_args: Vec<&str> = std::iter::once(id).chain(tokens.iter().copied()).collect();
        return Some(DispatchResult::Output(
            (crate::commands::control::cmd_fixture_at(state, &new_args)).into(),
        ));
    }

    // Unknown sub-command in edit mode — fall through to normal dispatch
    None
}

// ── Standalone Label command ────────────────────────────────────────────────

/// The name words of a Label (issue 07): one word, quoted or not;
/// a name with spaces must be quoted (`Label Group 3 Wash Front` is refused).
fn name_of(words: &[&str]) -> Result<String, String> {
    match words {
        [] => Ok(String::new()),
        [one] => Ok(super::tokens::unquote(one).to_string()),
        _ => Err(render_code(
            format!("A name with spaces goes in quotes: \"{}\".", words.iter().map(|w| super::tokens::unquote(w)).collect::<Vec<_>>().join(" ")),
            "E-CMD-006",
        )),
    }
}

/// Characters forbidden in labels.
const FORBIDDEN_LABEL_CHARS: &[char] = &[
    '\\', '"', '$', '&', '*', '?', ',', '.', ';', '^', '{', '|', '}', '~',
];

/// Strip restricted characters from a label string.
pub(super) fn sanitize_label(name: &str) -> String {
    name.chars()
        .filter(|c| !FORBIDDEN_LABEL_CHARS.contains(c))
        .collect()
}

/// The numbers of `start..=end` not in `existing` (sorted), as `n` or
/// `n Thru m` runs.
fn missing_ranges(start: u32, end: u32, existing: &[u32]) -> Vec<String> {
    let run = |a: u32, b: u32| if a == b { a.to_string() } else { format!("{a} Thru {b}") };
    let mut out = Vec::new();
    let mut next = Some(start);
    for &n in existing {
        if let Some(from) = next
            && n > from
        {
            out.push(run(from, n - 1));
        }
        next = n.checked_add(1).filter(|&m| m <= end);
    }
    if let Some(from) = next {
        out.push(run(from, end));
    }
    out
}

/// Generate an enumerated name from a base pattern and a zero-based index.
///
/// Finds the last free-standing number in `base` and replaces it with `start + index`.
/// If no number is found, returns the base unchanged for all indices.
///
/// Examples:
///   enumerate_name("Wash 1", 0) → "Wash 1"
///   enumerate_name("Wash 1", 3) → "Wash 4"
///   enumerate_name("hello 0", 1) → "hello 1"
pub(super) fn enumerate_name(base: &str, index: usize) -> String {
    // Find last free-standing number: a sequence of digits bounded by non-digit or string boundary
    let mut last_start = None;
    let mut last_end = None;
    let mut i = 0;
    let chars: Vec<char> = base.chars().collect();
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            last_start = Some(start);
            last_end = Some(i);
        } else {
            i += 1;
        }
    }

    match (last_start, last_end) {
        (Some(s), Some(e)) => {
            let num: usize = base[s..e].parse().unwrap_or(0);
            format!("{}{}{}", &base[..s], num + index, &base[e..])
        }
        _ => base.to_string(),
    }
}

/// One object's labeler: set (or clear, with None) the name of object `id`.
pub type Labeler = fn(&mut AppState, u32, Option<String>) -> Result<String, String>;

/// `Label <object> <id>` without a name asks for it (the Label dialog,
/// decisions §30 "12b"; a text input opens): Please labels it, an empty
/// name clears it (`/Clear`, since an empty `""` doesn't reach a command).
/// `/Clear` also clears from a script. Anything else goes to `label_with`.
pub fn label_or_ask(
    state: &mut AppState,
    keyword: &str,
    args: &[&str],
    labeler: Labeler,
    current: fn(&AppState, u32) -> Option<String>,
    ids: fn(&AppState) -> Vec<u32>,
) -> String {
    let clear = crate::commands::flags::has(args, &crate::commands::flags::CLEAR);
    let rest: Vec<&str> = args.iter().copied().filter(|a| !crate::commands::flags::is(a, &crate::commands::flags::CLEAR)).collect();
    if let [id] = rest.as_slice()
        && !clear
        && let Ok(n) = id.parse::<u32>()
    {
        let name = current(state, n).unwrap_or_default();
        state.asking = Some(
            crate::dialogs::Ask::new(
                format!("Label {keyword} {n}"),
                "A new name. Empty clears the label.",
            )
            .input("name", "Name", name, lumacore_wire::InputKind::Text)
            .button("Please", lumacore_wire::ButtonKind::Please, format!("Label {keyword} {n} \"{{name}}\""))
            .or_when_empty(format!("Label {keyword} {n} /Clear"))
            .cancel("Cancel")
            .flags(format!("the name (Label {keyword} {n} \"name\"), or /Clear")),
        );
        return render_code(format!("Usage: Label {keyword} {n} \"name\" | /Clear"), "E-CMD-006");
    }
    // A range with no name clears only when asked to (`/Clear`).
    if let [_, thru, _] = rest.as_slice()
        && thru.eq_ignore_ascii_case("thru")
        && !clear
    {
        return render_code(
            format!("Usage: Label {keyword} <n> Thru <m> \"name\" | /Clear"),
            "E-CMD-006",
        );
    }
    label_with(state, keyword, &rest, labeler, ids)
}

/// `Label <object> <id> [Thru <end>] "name"` — the arguments after the object
/// keyword; the object passes its labeler (`label_fixture`, …).
///
/// Labels one or more objects without entering edit mode.
/// Supports auto-enumeration for Thru ranges. Only the objects that exist
/// (`ids`) are visited; the missing numbers of a range are named in one
/// line (R1-09: `Thru 4294967295` never loops over every number).
pub fn label_with(
    state: &mut AppState,
    keyword: &str,
    args: &[&str],
    labeler: Labeler,
    ids: fn(&AppState) -> Vec<u32>,
) -> String {
    // Shift so the parsing below keeps its `Label <type> <id> …` indexes.
    let args: Vec<&str> = std::iter::once("").chain(args.iter().copied()).collect();
    if args.len() < 2 {
        return render_code(
            "Usage: Label <object> <id> [Thru <end>] \"name\"\nExamples: Label Fixture 1 \"Spot\", Label Universe 1 Thru 5 \"Uni 1\"",
            "E-CMD-006",
        );
    }

    let id: u32 = match args[1].parse() {
        Ok(id) => id,
        Err(_) => return render_code(format!("Invalid ID '{}'. Expected a number.", args[1]), "E-CMD-008"),
    };

    // Parse optional Thru and the name
    // Note: tokenizer strips quotes, so `Label Fixture 1 ""` arrives as just [Fixture, 1].
    // Bare `Label <type> <id>` (no name) clears the label — same as `Label <type> <id> ""`.
    // `Thru <end>` without a name is the range with no name (R1-35: it was
    // read as the name "Thru 5" of the first object).
    let (end_id, name_raw) = if args.len() >= 4 && args[2].to_lowercase() == "thru" {
        // Label <type> <id> Thru <end> "name"
        let end: u32 = match args[3].parse() {
            Ok(e) => e,
            Err(_) => {
                return render_code(format!("Invalid end ID '{}'. Expected a number.", args[3]), "E-CMD-008");
            }
        };
        let name = match name_of(&args[4..]) {
            Ok(name) => name,
            Err(e) => return e,
        };
        (Some(end), name)
    } else if args.len() >= 3 {
        // Label <type> <id> "name"
        let name = match name_of(&args[2..]) {
            Ok(name) => name,
            Err(e) => return e,
        };
        (None, name)
    } else {
        // Bare Label <type> <id> — clear the label
        (None, String::new())
    };

    // Sanitize the name (strip forbidden characters)
    let sanitized = sanitize_label(&name_raw);
    let was_stripped = sanitized.len() != name_raw.len();

    // Empty string clears the name
    let clear_mode = sanitized.is_empty();

    let start = id;
    let end = end_id.unwrap_or(id);
    if end < start {
        return render_code(format!(
            "Invalid range: {start} Thru {end} (end must be >= start)."
        ), "E-CMD-008");
    }

    let mut results: Vec<String> = Vec::new();
    let count = (end - start) as usize + 1;
    let existing: Vec<u32> = ids(state).into_iter().filter(|n| (start..=end).contains(n)).collect();
    let missing = missing_ranges(start, end, &existing);
    if !missing.is_empty() {
        results.push(render_code(format!("{keyword} {} not found.", missing.join(", ")), "E-CMD-007"));
    }

    for obj_id in existing {
        let idx = (obj_id - start) as usize;
        let label = if clear_mode {
            None
        } else if count > 1 {
            Some(enumerate_name(&sanitized, idx))
        } else {
            Some(sanitized.clone())
        };

        let result = labeler(state, obj_id, label);

        match result {
            Ok(msg) => results.push(msg),
            Err(msg) => results.push(render_code(&msg, "E-CMD-007")),
        }
    }

    if was_stripped {
        results.push(render_success(
            "Note: Some restricted characters were removed from the label.",
        ));
    }

    results.join("\n")
}

pub fn label_fixture_type(
    state: &mut AppState,
    id: u32,
    name: Option<String>,
) -> Result<String, String> {
    match state.root.show_data.devices.dmx.patch.fixture_type_mut(id) {
        Some(ft) => {
            let display = match &name {
                Some(n) => format!("FixtureType {id} labeled \"{n}\"."),
                None => format!("FixtureType {id} label cleared."),
            };
            ft.meta.identity.name = name;
            Ok(render_success(&display))
        }
        None => Err(format!("FixtureType {id} not found.")),
    }
}

