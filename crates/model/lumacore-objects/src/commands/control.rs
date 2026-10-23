//! Control commands: fixture selection, value commands, and overlay commands for the LumaCore REPL.
//!
//! Implements the core lighting control grammar:
//!   Select Fixture 1 Thru 5          — update selection
//!   Fixture 1 Thru 5 At 50           — set dimmer 50% on fixtures 1–5
//!   At 80                            — apply to current selection
//!   Full / Zero                      — dimmer shortcuts
//!   On / Off [<fixtures>]            — knock in / knock out
//!   Attribute "Pan" At 45            — set named attribute
//!   Reset / ResetAll                 — programmer clear state machine
//!   Park Fixture <id> [<attr>] [At <pct>] — park fixture attributes
//!   Park DMX <u.a> [At <val>]        — park specific DMX address
//!   Unpark [Fixture <id> | DMX <u.a>] — release parked values
//!
//! Per CONTEXT.md: value commands (At/Full/Zero/On/Off/Attribute) do NOT
//! touch selected_fixtures. Only Select modifies selection.
//!
//! Values go through `programmers::program`: programmers are LTP and a touch
//! moves the attribute out of other users' programmers (decisions §4). The
//! render engine merges them (decisions §27).

use crate::AppState;
use crate::errors::render_code;
use crate::parser::parse_selection;
use crate::parser::resolve::{apply_if_filter, resolve_selection};
use crate::programmers;
use lumacore_tree::Value;

/// An error reply for `e` with `code` — unless `e` already is a rendered
/// (coded) error reply from below, which is passed on as it is.
pub fn error_reply(e: String, code: &str) -> String {
    crate::errors::reply(e, code)
}

// ─── Select ──────────────────────────────────────────────────────────────────

/// Handle: Select [Fixture] <selection>
///
/// Resolves the selection expression and selects it by the selection rule
/// (decisions §22: adds until a value was set, then replaces).
/// Returns a "Selected N fixtures" message.
pub fn cmd_select(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        // Bare select: show current selection
        let prog = match state.current_programmer() {
            Some(p) => p,
            None => return render_code("No programmer available.", "E-CMD-011"),
        };
        if prog.selected_fixtures.is_empty() {
            return nu_ansi_term::Style::new()
                .dimmed()
                .paint("No fixtures selected.")
                .to_string();
        }
        let ids: Vec<String> = prog
            .selected_fixtures
            .iter()
            .filter_map(|guid| state.root.show_data.fixture_id_for_guid(*guid))
            .map(|id| id.to_string())
            .collect();
        return format!("Selection: {}", ids.join(", "));
    }

    // Strip leading "fixture" keyword if present
    let sel_args = if args[0].to_lowercase() == "fixture" {
        &args[1..]
    } else {
        args
    };

    if sel_args.is_empty() {
        return render_code("Select what? Usage: Select Fixture 1 Thru 5", "E-CMD-006");
    }

    let tokens: Vec<String> = sel_args.iter().map(|s| s.to_string()).collect();
    let parsed = match parse_selection(&tokens) {
        Ok(p) => p,
        Err(e) => {
            return render_code(format!("Selection parse error: {e}"), "E-CMD-008");
        }
    };

    let ids = match resolve_parsed(state, &parsed) {
        Ok(ids) => ids,
        Err(e) => return e,
    };

    if ids.is_empty() {
        return render_code("No matching fixtures found.", "E-CMD-007");
    }
    let count = match state.select_fixtures(&ids, lumacore_tree::programmer::SelectMode::Plain) {
        Ok(count) => count,
        Err(e) => return error_reply(e, "E-CMD-011"),
    };

    if count == 0 {
        render_code("No matching fixtures found.", "E-CMD-007")
    } else {
        crate::text::render_success(&format!(
            "Selected {count} fixture{}",
            if count == 1 { "" } else { "s" }
        ))
    }
}

/// Add a single fixture to the current selection (stacking).
///
/// Used when a bare number is typed in Fixture context — each number
/// accumulates into the selection rather than replacing it.
pub fn cmd_select_add(state: &mut AppState, fixture_id: u32) -> String {
    // Verify the fixture exists
    if !state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .has_fid(fixture_id)
    {
        return render_code(format!("Fixture {fixture_id} not found."), "E-CMD-007");
    }

    let count = match state.add_fixture_to_selection(fixture_id) {
        Ok(count) => count,
        Err(e) => return error_reply(e, "E-CMD-011"),
    };
    crate::text::render_success(&format!(
        "Selected {count} fixture{}",
        if count == 1 { "" } else { "s" }
    ))
}

// ─── At ──────────────────────────────────────────────────────────────────────

/// Handle: At <value>  OR  [already-resolved fixture IDs] At <value>
///
/// This is called with either:
/// - Bare `At <value>` — apply to current selection
/// - With pre-resolved `fixture_ids` override (from Fixture <sel> At <val> routing in mod.rs)
///
/// Values are percentages (0–100). Converts to DMX via percent_to_dmx.
/// Does NOT modify selected_fixtures.
pub fn cmd_at(state: &mut AppState, args: &[&str]) -> String {
    cmd_at_with_override(state, args, None)
}

/// Internal At handler that accepts an optional pre-resolved fixture ID list.
///
/// If `id_override` is Some, apply to those IDs. Otherwise apply to current selection.
pub fn cmd_at_with_override(
    state: &mut AppState,
    args: &[&str],
    id_override: Option<Vec<u32>>,
) -> String {
    // `At Preset n` (decisions 0288): its values on the selection, or on the line's own target.
    if let Some((first, rest)) = args.split_first()
        && crate::objects::PRESET.matches(first)
    {
        return crate::objects::preset::apply::at(state, rest, id_override);
    }

    if args.is_empty() {
        return render_code("At what value? Usage: At 50", "E-CMD-006");
    }
    // `At 10 Attribute "Pan"` = `Attribute "Pan" At 10` (the usual
    // console convention: Pan 10, the Dimmer untouched).
    if id_override.is_none()
        && let Some(pos) = args.iter().position(|a| {
            // The word, its aliases and prefixes (`Att`), never a string match.
            crate::commands::attributes::ATTRIBUTE.matches(a)
        })
        && pos > 0
        && pos + 1 < args.len()
    {
        let mut line: Vec<&str> = args[pos + 1..].to_vec();
        line.push("At");
        line.extend_from_slice(&args[..pos]);
        return cmd_attribute(state, &line);
    }
    let amount = match parse_amount(args) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let fixture_guids = match id_override {
        Some(ids) => state.root.show_data.fixture_guids(&ids),
        None => match state.current_programmer() {
            Some(p) => p.selected_fixtures.clone(),
            None => return render_code("No programmer available.", "E-CMD-011"),
        },
    };

    if fixture_guids.is_empty() {
        return render_code(
            "No fixtures selected. Use 'Fixture 1 At 50' or select first.",
            "E-CMD-009",
        );
    }

    let readout = amount.readout();
    let values = amount.per_fixture(state, &fixture_guids, "Dimmer");
    let count = match programmers::program(state, values) {
        Ok(n) => n,
        Err(e) => return e,
    };

    crate::text::render_success(&format!(
        "{count} fixture{} → Dimmer {readout}",
        if count == 1 { "" } else { "s" },
    ))
}

/// An `At` amount: a value, or relative to each fixture's own —
/// `+ 10`, `- 10`, `* 2`, `/ 2` (decisions §30 "12b": the keypad sends what
/// was typed; the station works it out).
pub enum Amount {
    Abs(Value),
    Rel(char, f64),
}

impl Amount {
    fn readout(&self) -> String {
        match self {
            Amount::Abs(v) => v.readout(),
            Amount::Rel(op, by) => format!("{op} {by}"),
        }
    }

    /// The value for each fixture: relative ones from what the user's
    /// programmer holds for `attribute` (0 when nothing), kept in 0–100 %.
    fn per_fixture(
        &self,
        state: &AppState,
        guids: &[uuid::Uuid],
        attribute: &str,
    ) -> Vec<(uuid::Uuid, String, Value)> {
        // What a relative value adds to (Q8): your programmer value,
        // else what the fixture outputs now (before masters; R1-37).
        let output = matches!(self, Amount::Rel(..)).then(|| state.output_plan());
        guids
            .iter()
            .map(|&g| {
                let value = match self {
                    Amount::Abs(v) => v.clone(),
                    Amount::Rel(op, by) => {
                        // Under the fixture's own spelling (`pan` finds `Pan`),
                        // in the value's own unit: a physical value (degrees)
                        // stays physical; percent stays in 0–100.
                        let spelled = crate::programmers::attribute_spelling(state, g, attribute);
                        let now = state
                            .current_programmer()
                            .and_then(|p| p.get_value(g, &spelled))
                            .and_then(Value::absolute)
                            .or_else(|| output.as_ref()?.current_value(g, &spelled)?.absolute());
                        let apply = |now: f64| match op {
                            '+' => now + by,
                            '-' => now - by,
                            '*' => now * by,
                            _ => now / by,
                        };
                        match now {
                            Some(lumacore_base::Absolute::Physical(p)) => Value::physical(apply(p)),
                            Some(lumacore_base::Absolute::Normalized(p)) => Value::normalized(apply(p).clamp(0.0, 100.0)),
                            None => Value::normalized(apply(0.0).clamp(0.0, 100.0)),
                        }
                    }
                };
                (g, attribute.to_string(), value)
            })
            .collect()
    }
}

/// `50`, `Full`, `Zero`, `Release`, `+ 10`, `- 10`, `* 2`, `/ 2` (`-10`
/// glued is the absolute value, clamped to 0). Anything
/// else is refused (E-CMD-008) — never read as 0 %.
pub fn parse_amount(words: &[&str]) -> Result<Amount, String> {
    let bad = |w: &str| render_code(format!("'{w}' isn't a value (50, Full, Zero, + 10, * 2)."), "E-CMD-008");
    let first = *words.first().ok_or_else(|| bad(""))?;
    // tidy: not-keywords (relative operators)
    if matches!(first, "+" | "-" | "*" | "/") {
        let by = words.get(1).ok_or_else(|| bad(first))?;
        // Only a finite number (R1-06: `+ nan`, `* inf` are never stored).
        let by = by.parse::<f64>().ok().filter(|v| v.is_finite()).ok_or_else(|| bad(by))?;
        if first == "/" && by == 0.0 {
            return Err(bad("/ 0"));
        }
        return Ok(Amount::Rel(first.chars().next().unwrap_or('+'), by));
    }
    // `-5` glued is an absolute value (the common console convention: "to
    // obtain an absolute value, remove the space"); `- 5` is relative.
    if let Some(rest) = first.strip_prefix(['+', '*', '/'])
        && let Ok(by) = rest.parse::<f64>()
    {
        // Glued forms follow the same rules as spaced ones (R1-06: `/0`).
        if !by.is_finite() || (first.starts_with('/') && by == 0.0) {
            return Err(bad(first));
        }
        return Ok(Amount::Rel(first.chars().next().unwrap_or('+'), by));
    }
    parse_value(first).map(Amount::Abs).ok_or_else(|| bad(first))
}

// ─── Full ─────────────────────────────────────────────────────────────────────

/// Handle: Full [<selection>]
///
/// Sets dimmer to 100% (DMX 255). If selection args follow, apply to those.
/// Otherwise apply to current selection.
pub fn cmd_full(state: &mut AppState, args: &[&str]) -> String {
    cmd_level_shortcut(state, args, 100.0, "Full")
}

// ─── Zero ─────────────────────────────────────────────────────────────────────

/// Handle: Zero [<selection>]
///
/// Sets dimmer to 0% (DMX 0). If selection args follow, apply to those.
/// Otherwise apply to current selection.
pub fn cmd_zero(state: &mut AppState, args: &[&str]) -> String {
    cmd_level_shortcut(state, args, 0.0, "Zero")
}

// ─── On / Off (knock in / knock out) ──────────────────────────────────────────

/// `On [<fixtures>]` — knock in: activate the programmer values of the
/// selection, or of the fixtures named (`On Fixture 2`). Pulling values in
/// from cues and presets comes with the render engine (decisions §16).
pub fn knock_in(state: &mut AppState, args: &[&str]) -> String {
    match target_fixtures(state, args, "On") {
        Ok(guids) => knock_in_fixtures(state, &guids),
        Err(e) => e,
    }
}

/// Knock in these fixtures (`On Group 3` calls this with the group's).
pub fn knock_in_fixtures(state: &mut AppState, guids: &[uuid::Uuid]) -> String {
    // A knock in is a touch: refused on another user's claimed fixture,
    // like every other value path (R1-43).
    if let Err(e) = programmers::ensure_unclaimed_guids(state, guids) {
        return e;
    }
    let guids = guids.to_vec();
    let Some(prog) = state.current_programmer_mut() else {
        return render_code("No programmer available.", "E-CMD-011");
    };
    let count = prog.knock_in(&guids);
    // Knocking in is a touch: the attributes leave other programmers.
    if !programmers::in_blind(state) {
        programmers::take_over(state, Some(&guids));
    }
    crate::text::render_success(&format!(
        "On: {count} value{} activated ({} fixture{})",
        if count == 1 { "" } else { "s" },
        guids.len(),
        if guids.len() == 1 { "" } else { "s" }
    ))
}

/// `Off [<fixtures>]` — knock out: remove the selection's (or the named
/// fixtures') values from the programmer and deselect them.
pub fn knock_out(state: &mut AppState, args: &[&str]) -> String {
    match target_fixtures(state, args, "Off") {
        Ok(guids) => knock_out_fixtures(state, &guids),
        Err(e) => e,
    }
}

/// Knock out these fixtures (`Off Group 3` calls this with the group's).
pub fn knock_out_fixtures(state: &mut AppState, guids: &[uuid::Uuid]) -> String {
    let guids = guids.to_vec();
    let Some(prog) = state.current_programmer_mut() else {
        return render_code("No programmer available.", "E-CMD-011");
    };
    let count = prog.knock_out(&guids);
    crate::text::render_success(&format!(
        "Off: {} fixture{} knocked out ({count} value{} removed)",
        guids.len(),
        if guids.len() == 1 { "" } else { "s" },
        if count == 1 { "" } else { "s" }
    ))
}

// ─── Attribute ────────────────────────────────────────────────────────────────

/// Handle: Attribute "<name>" At <value>
///
/// Sets a named attribute on the current selection.
/// Attribute name may be quoted or bare. Value is percent.
pub fn cmd_attribute(state: &mut AppState, args: &[&str]) -> String {
    // Parse: Attribute "Pan" At 45
    // args after "attribute" verb: ["Pan", "At", "45"] or ["Pan", "45"]
    if args.is_empty() {
        return render_code(
            "Usage: Attribute \"Pan\" At 45\nSets named attribute on current selection.",
            "E-CMD-006",
        );
    }

    // `Attribute "Dimmer" Fade 2 | Delay 0.3 | Fade Cue`: the values' own timing (decisions §30
    // "12f" value timing).
    // tidy: not-keywords (Fade / Delay here name a value's timing layer, not the cue words)
    if let Some(pos) = args.iter().position(|s| s.eq_ignore_ascii_case("fade") || s.eq_ignore_ascii_case("delay"))
        && pos > 0
    {
        return attribute_timing(state, &args[..pos], args[pos], args.get(pos + 1..).unwrap_or_default());
    }

    // Find "at" keyword position
    let at_pos = args.iter().position(|s| s.to_lowercase() == "at");

    // Everything after `At` is the value (`At + 10` is two words, R1-37).
    // The name as typed, without its quotes (`"Pan"`, `pan`).
    let name = |words: &[&str]| words.iter().map(|w| w.trim_matches('"')).collect::<Vec<_>>().join(" ");
    let (attr_name, value_words) = match at_pos {
        Some(pos) if pos > 0 && pos + 1 < args.len() => (name(&args[..pos]), &args[pos + 1..]),
        None if args.len() >= 2 => {
            // No "at" keyword — treat last token as value
            (name(&args[..args.len() - 1]), &args[args.len() - 1..])
        }
        _ => {
            return render_code("Usage: Attribute \"Pan\" At 45", "E-CMD-006");
        }
    };

    let amount = match parse_amount(value_words) {
        Ok(a) => a,
        Err(e) => return e,
    };
    let readout = amount.readout();

    let fixture_guids = match state.current_programmer() {
        Some(p) => p.selected_fixtures.clone(),
        None => return render_code("No programmer available.", "E-CMD-011"),
    };

    if fixture_guids.is_empty() {
        return render_code("No fixtures selected. Select fixtures first.", "E-CMD-009");
    }

    let values = amount.per_fixture(state, &fixture_guids, &attr_name);
    let count = match programmers::program(state, values) {
        Ok(n) => n,
        Err(e) => return e,
    };

    crate::text::render_success(&format!(
        "{count} fixture{} → {attr_name} {readout}",
        if count == 1 { "" } else { "s" },
    ))
}

/// `Attribute "x" Fade|Delay <s>|Cue`: set (or give back to the cue) the Fade / Delay of the
/// selected fixtures' programmer values of attribute `x`. Only values the programmer holds take
/// timing; it travels with them into the cue they are stored in.
fn attribute_timing(state: &mut AppState, name_words: &[&str], layer: &str, rest: &[&str]) -> String {
    let name = name_words.iter().map(|w| w.trim_matches('"')).collect::<Vec<_>>().join(" ");
    let fade = layer.eq_ignore_ascii_case("fade");
    let label = if fade { "Fade" } else { "Delay" };
    let seconds = match rest {
        [w] if w.eq_ignore_ascii_case("cue") => None,
        [w] => match w.trim_end_matches(['s', 'S']).parse::<f64>() {
            Ok(v) if (0.0..=3600.0).contains(&v) => Some(v),
            _ => return render_code(format!("'{w}' is not a time (0–3600 s, or Cue)."), "E-CMD-008"),
        },
        _ => return render_code(format!("Usage: Attribute \"{name}\" {label} <seconds>|Cue"), "E-CMD-006"),
    };
    let Some(prog) = state.current_programmer_mut() else {
        return render_code("No programmer available.", "E-CMD-011");
    };
    let selected = prog.selected_fixtures.clone();
    if selected.is_empty() {
        return render_code("No fixtures selected. Select fixtures first.", "E-CMD-009");
    }
    let mut n = 0;
    for ((fixture, attr), pv) in prog.values.iter_mut() {
        if selected.contains(fixture) && attr.eq_ignore_ascii_case(&name) {
            if fade { pv.timing.fade = seconds } else { pv.timing.delay = seconds }
            n += 1;
        }
    }
    if n == 0 {
        return render_code(format!("No {name} values of the selection in the programmer: set a value first."), "E-CMD-009");
    }
    crate::text::render_success(&match seconds {
        Some(v) => format!("{n} {name} value{} → {label} {v} s", if n == 1 { "" } else { "s" }),
        None => format!("{n} {name} value{} → {label} from the cue", if n == 1 { "" } else { "s" }),
    })
}

// ─── Reset ────────────────────────────────────────────────────────────────────

/// Handle: Reset
///
/// Three-stage reset (industry-standard order):
/// 1st press: clears selection
/// 2nd press: deactivates values (still visible, won't store)
/// 3rd press: empties programmer completely
pub fn cmd_clear(state: &mut AppState, _args: &[&str]) -> String {
    let prog = match state.current_programmer_mut() {
        Some(p) => p,
        None => return render_code("No programmer available.", "E-CMD-011"),
    };

    let msg = prog.clear_layer();
    let is_empty = prog.is_empty();
    let was_echoed = prog.reset_echoed;

    // Track echo state for the 4th-stop clear-buffer behavior
    if msg == "Programmer cleared" && is_empty && was_echoed {
        prog.reset_echoed = false;
        return "\x1B[2J\x1B[1;1H".to_string();
    }
    if msg == "Programmer cleared" && is_empty {
        prog.reset_echoed = true;
    }
    crate::text::render_success(msg)
}

/// Handle: ResetAll
///
/// Unconditional full programmer clear (values + selection + state reset).
pub fn cmd_clear_all(state: &mut AppState, _args: &[&str]) -> String {
    let prog = match state.current_programmer_mut() {
        Some(p) => p,
        None => return render_code("No programmer available.", "E-CMD-011"),
    };

    prog.full_clear();
    crate::text::render_success("Programmer fully cleared.")
}

/// Handle: ResetSelection
///
/// Clear selection only, leave values untouched.
pub fn cmd_reset_selection(state: &mut AppState, _args: &[&str]) -> String {
    let prog = match state.current_programmer_mut() {
        Some(p) => p,
        None => return render_code("No programmer available.", "E-CMD-011"),
    };

    prog.clear_selection();
    crate::text::render_success("Selection cleared")
}

/// Handle: ResetActive
///
/// Deactivate all active values. Values remain in programmer but are dimmed.
pub fn cmd_reset_active(state: &mut AppState, _args: &[&str]) -> String {
    let prog = match state.current_programmer_mut() {
        Some(p) => p,
        None => return render_code("No programmer available.", "E-CMD-011"),
    };

    if !prog.has_active_values() {
        return nu_ansi_term::Style::new()
            .dimmed()
            .paint("No active values to deactivate.")
            .to_string();
    }

    prog.deactivate_values();
    crate::text::render_success("Values deactivated")
}

// ─── Please (Meow) ──────────────────────────────────────────────────────────

/// Handle: Please
///
/// The universal "do it" key. Behavior depends on programmer state:
/// - Nothing selected: echo "meow" (cat personality)
/// - Fixtures selected, no active values: knock-in (activate attributes from current output)
/// - Fixtures selected, has active values: deactivate all (toggle)
pub fn cmd_please(state: &mut AppState, _args: &[&str]) -> String {
    let prog = match state.current_programmer() {
        Some(p) => p.clone(),
        None => return "meow".to_string(),
    };

    if prog.selected_fixtures.is_empty() {
        return "meow".to_string();
    }

    // Fixtures are selected — check if we have active values
    if prog.has_active_values() {
        // Toggle: deactivate all values
        let prog_mut = state
            .current_programmer_mut()
            .expect("programmer checked above");
        prog_mut.deactivate_values();
        crate::text::render_success("Values deactivated")
    } else {
        // Knock-in: activate all inactive values, or if none exist, pull defaults
        let prog_mut = state
            .current_programmer_mut()
            .expect("programmer checked above");
        if !prog_mut.values.is_empty() {
            // Reactivate existing inactive values
            prog_mut.activate_values();
            crate::text::render_success("Values reactivated")
        } else {
            // No values at all — nothing to knock in yet
            let count = prog_mut.selected_fixtures.len();
            format!(
                "{}",
                nu_ansi_term::Style::new().dimmed().paint(format!(
                    "{count} fixture{} selected, no values to activate.",
                    if count == 1 { "" } else { "s" }
                ))
            )
        }
    }
}

// ─── Next / Previous ─────────────────────────────────────────────────────────

/// Handle: Next
///
/// Walk forward through the current selection (or all fixtures if none selected).
///
/// - No selection: select the lowest-ID fixture globally.
/// - Single fixture (not walking): jump to the next fixture by global ID.
/// - Multiple fixtures (or already walking): enter/continue walk mode,
///   stepping forward through the selection pool one fixture at a time.
/// - Wraps around at the end.
pub fn cmd_next(state: &mut AppState, _args: &[&str]) -> String {
    walk_selection(state, WalkDirection::Forward)
}

/// Handle: Previous
///
/// Walk backward through the current selection (or all fixtures if none selected).
///
/// - No selection: select the highest-ID fixture globally.
/// - Single fixture (not walking): jump to the previous fixture by global ID.
/// - Multiple fixtures (or already walking): enter/continue walk mode,
///   stepping backward through the selection pool one fixture at a time.
/// - Wraps around at the beginning.
pub fn cmd_previous(state: &mut AppState, _args: &[&str]) -> String {
    walk_selection(state, WalkDirection::Backward)
}

enum WalkDirection {
    Forward,
    Backward,
}

fn walk_selection(state: &mut AppState, dir: WalkDirection) -> String {
    let fixtures = state.root.show_data.devices.dmx.patch.by_number();
    if fixtures.is_empty() {
        return render_code("No fixtures patched.", "E-CMD-009");
    }

    let prog = match state.current_programmer() {
        Some(p) => p.clone(),
        None => return render_code("No programmer available.", "E-CMD-011"),
    };

    // ── Case 1: Nothing selected — pick first or last global fixture ─────
    if prog.selected_fixtures.is_empty() && !prog.is_walking() {
        let id = match dir {
            WalkDirection::Forward => *fixtures
                .keys()
                .next()
                .expect("fixtures non-empty checked above"),
            WalkDirection::Backward => *fixtures
                .keys()
                .next_back()
                .expect("fixtures non-empty checked above"),
        };
        if let Err(e) = state.replace_current_selection(vec![id]) {
            return error_reply(e, "E-CMD-011");
        }
        return fixture_walk_msg(state, id);
    }

    // ── Case 2: Single fixture, not walking — jump by global ID ──────────
    if prog.selected_fixtures.len() == 1 && !prog.is_walking() {
        let current_guid = prog.selected_fixtures[0];
        let current = state
            .root
            .show_data
            .fixture_id_for_guid(current_guid)
            .unwrap_or(0);
        let id = match dir {
            WalkDirection::Forward => {
                fixtures
                    .range((current + 1)..)
                    .next()
                    .map(|(&id, _)| id)
                    .unwrap_or_else(|| {
                        *fixtures
                            .keys()
                            .next()
                            .expect("fixtures non-empty checked above")
                    }) // wrap
            }
            WalkDirection::Backward => {
                fixtures
                    .range(..current)
                    .next_back()
                    .map(|(&id, _)| id)
                    .unwrap_or_else(|| {
                        *fixtures
                            .keys()
                            .next_back()
                            .expect("fixtures non-empty checked above")
                    }) // wrap
            }
        };
        if let Err(e) = state.replace_current_selection(vec![id]) {
            return error_reply(e, "E-CMD-011");
        }
        return fixture_walk_msg(state, id);
    }

    // ── Case 3: Multiple fixtures (or already walking) — walk the pool ───
    let prog_mut = state
        .current_programmer_mut()
        .expect("programmer checked at function entry");

    if !prog_mut.is_walking() {
        // Enter walk mode: stash full selection, start at first/last
        let pool = prog_mut.selected_fixtures.clone();
        let idx = match dir {
            WalkDirection::Forward => 0,
            WalkDirection::Backward => pool.len().saturating_sub(1),
        };
        prog_mut.selection_pool = Some(pool);
        prog_mut.walk_index = idx;
    } else {
        // Already walking: advance index
        let pool_len = prog_mut
            .selection_pool
            .as_ref()
            .expect("invariant: is_walking implies selection_pool is Some")
            .len();
        prog_mut.walk_index = match dir {
            WalkDirection::Forward => {
                if prog_mut.walk_index + 1 >= pool_len {
                    0
                } else {
                    prog_mut.walk_index + 1
                }
            }
            WalkDirection::Backward => {
                if prog_mut.walk_index == 0 {
                    pool_len - 1
                } else {
                    prog_mut.walk_index - 1
                }
            }
        };
    }

    let pool = prog_mut
        .selection_pool
        .as_ref()
        .expect("invariant: is_walking implies selection_pool is Some");
    let walked_guid = pool[prog_mut.walk_index];

    let idx = prog_mut.walk_index;
    let total = pool.len();
    let walked_id = state
        .root
        .show_data
        .fixture_id_for_guid(walked_guid)
        .unwrap_or(0);
    if let Err(e) = state.replace_current_selection(vec![walked_id]) {
        return error_reply(e, "E-CMD-011");
    }
    fixture_walk_msg_indexed(state, walked_id, idx, total)
}

/// Format a walk result message with fixture name.
fn fixture_walk_msg(state: &AppState, id: u32) -> String {
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(id)
        .and_then(|f| f.meta.name().map(|n| n.to_string()))
        .unwrap_or_else(|| format!("Fixture {id}"));
    crate::text::render_success(&format!("Fixture {id} \"{name}\""))
}

/// Format a walk result message with fixture name and pool position.
fn fixture_walk_msg_indexed(state: &AppState, id: u32, idx: usize, total: usize) -> String {
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(id)
        .and_then(|f| f.meta.name().map(|n| n.to_string()))
        .unwrap_or_else(|| format!("Fixture {id}"));
    crate::text::render_success(&format!("Fixture {id} \"{name}\" [{}/{}]", idx + 1, total))
}

// ─── Fixture <sel> At <val> routing ──────────────────────────────────────────

/// Handle the `Fixture <selection> [At <value>]` pattern.
///
/// If `At` is present: resolves selection and applies dimmer value without
/// modifying `selected_fixtures`.
/// If `At` is absent: treats as selection-only — updates `selected_fixtures`.
pub fn cmd_fixture_at(state: &mut AppState, args: &[&str]) -> String {
    // Find "at" keyword position
    let at_pos = args.iter().position(|s| s.to_lowercase() == "at");

    let at_pos = match at_pos {
        Some(p) => p,
        None => {
            // No "at" — treat as selection command: `Fixture 1 Thru 5` selects those fixtures
            if args.is_empty() {
                return cmd_select(state, args);
            }
            return cmd_select(state, args);
        }
    };

    if at_pos == 0 {
        return render_code(
            "No selection before 'At'. Usage: Fixture 1 At 50",
            "E-CMD-006",
        );
    }

    let sel_tokens: Vec<String> = args[..at_pos].iter().map(|s| s.to_string()).collect();
    let value_args = &args[at_pos + 1..];

    if value_args.is_empty() {
        return render_code("No value after 'At'. Usage: Fixture 1 At 50", "E-CMD-006");
    }

    // Parse selection
    let parsed = match parse_selection(&sel_tokens) {
        Ok(p) => p,
        Err(e) => {
            return render_code(format!("Selection parse error: {e}"), "E-CMD-008");
        }
    };

    let ids = match resolve_parsed(state, &parsed) {
        Ok(ids) => ids,
        Err(e) => return e,
    };

    if ids.is_empty() {
        return render_code("No matching fixtures found.", "E-CMD-007");
    }

    cmd_at_with_override(state, value_args, Some(ids))
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// The fixture ids a parsed selection names, its `If` filter applied (one
/// path for every word that takes a selection: R1-32, an `If` was parsed and
/// then dropped on value lines and On / Off / Full / Zero).
pub fn resolve_parsed(
    state: &AppState,
    parsed: &crate::parser::ParsedSelection,
) -> Result<Vec<u32>, String> {
    let fixtures = state.root.show_data.devices.dmx.patch.by_number();
    let groups = state.user_groups();
    let ids = resolve_selection(&parsed.expr, &fixtures, groups);
    if parsed.filters.is_empty() {
        return Ok(ids);
    }
    let programmer = state
        .current_programmer()
        .ok_or_else(|| render_code("No programmer available.", "E-CMD-011"))?;
    Ok(apply_if_filter(&parsed.filters, ids, groups, programmer, &fixtures))
}

/// Shared logic for Full/Zero/On/Off: apply a fixed dimmer level.
///
/// If args contain a selection expression, apply to those fixtures.
/// Otherwise apply to current selection.
fn cmd_level_shortcut(state: &mut AppState, args: &[&str], pct: f64, label: &str) -> String {
    let fixture_guids = match target_fixtures(state, args, label) {
        Ok(g) => g,
        Err(e) => return e,
    };

    let count = match set_dimmer_values(state, &fixture_guids, Value::normalized(pct)) {
        Ok(n) => n,
        Err(e) => return e,
    };

    crate::text::render_success(&format!(
        "{count} fixture{} → {label}",
        if count == 1 { "" } else { "s" }
    ))
}

/// The fixtures a level shortcut or knock acts on: the selection when `args`
/// are empty, else the selection expression in `args` (`Fixture 1 Thru 5`).
fn target_fixtures(
    state: &AppState,
    args: &[&str],
    label: &str,
) -> Result<Vec<uuid::Uuid>, String> {
    let fixture_guids = if args.is_empty() {
        match state.current_programmer() {
            Some(p) => p.selected_fixtures.clone(),
            None => {
                return Err(render_code("No programmer available.", "E-CMD-011"));
            }
        }
    } else {
        let tokens: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let parsed = parse_selection(&tokens)
            .map_err(|e| render_code(format!("Selection parse error: {e}"), "E-CMD-008"))?;
        let ids = resolve_parsed(state, &parsed)?;
        state.root.show_data.fixture_guids(&ids)
    };
    if fixture_guids.is_empty() {
        return Err(render_code(
            format!(
                "{label}: no fixtures selected. Select first or specify: {label} Fixture 1 Thru 5"
            ),
            "E-CMD-009",
        ));
    }
    Ok(fixture_guids)
}

/// Set the Dimmer of these fixtures (an LTP touch). Returns how many were
/// set. Does NOT modify selected_fixtures.
fn set_dimmer_values(
    state: &mut AppState,
    guids: &[uuid::Uuid],
    value: Value,
) -> Result<usize, String> {
    programmers::program(
        state,
        guids
            .iter()
            .map(|&g| (g, "Dimmer".to_string(), value.clone()))
            .collect(),
    )
}

/// Words that continue a fixture selection with a value (`Fixture 1 At 50`,
/// `Fixture 1 Full`) — the fixture value grammar, not command routing.
pub fn is_fixture_value_word(word: &str) -> bool {
    // tidy: not-keywords (fixture value grammar)
    matches!(
        word.to_lowercase().as_str(),
        "at" | "full" | "zero" | "on" | "off"
    )
}

/// Parse a value token as a percent.
///
/// Accepts:
/// - "full" / "100" → 100.0
/// - "zero" / "0"   → 0.0
/// - "50"           → 50.0
/// - "50.5"         → 50.5
fn parse_value_percent(s: &str) -> Option<f64> {
    // tidy: not-keywords (value words)
    match s.to_lowercase().as_str() {
        "full" => Some(100.0),
        "zero" => Some(0.0),
        // Clamped to 0–100 % (`At 150` → 100, `At -5` → 0; R1-12, Q2).
        other => other.parse::<f64>().ok().filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 100.0)),
    }
}

/// A value token: a percent, or `Release` (stored in cues: the attribute
/// goes back to lower sources, decisions §27). None = not a value.
fn parse_value(s: &str) -> Option<Value> {
    if s.eq_ignore_ascii_case("release") {
        Some(Value::release())
    } else {
        parse_value_percent(s).map(Value::normalized)
    }
}

// ─── Park ─────────────────────────────────────────────────────────────────────

/// Handle: Park [Fixture <sel>] [<attribute>] [At <pct>] | Park DMX <u.a> [At <val>]
///
/// Parked values win over everything and ignore masters (decisions §27).
/// They are saved with the show (`ShowData.Playback.Park`).
///
/// Forms:
/// - `Park` / `Park Fixture 1 Thru 3` — every attribute at its current output
/// - `Park Fixture 1 At 50`           — the Dimmer at 50% (`At` is intensity)
/// - `Park Fixture 1 Pan At 30`       — one attribute
/// - `Park DMX 1.255 [At 200]`        — a raw DMX channel (unpatched channels)
pub fn cmd_park(state: &mut AppState, args: &[&str]) -> String {
    if args.first().is_some_and(|a| a.eq_ignore_ascii_case("dmx")) {
        return park_dmx(state, &args[1..]);
    }
    let names_fixture = args.first().is_some_and(|a| {
        crate::objects::find(a)
            .is_some_and(|o| std::ptr::eq(o, &crate::objects::FIXTURE))
    });
    let rest = if names_fixture { &args[1..] } else { args };
    let (fixtures, attribute, at) = match park_target(state, rest) {
        Ok(t) => t,
        Err(e) => return e,
    };
    // The value each attribute outputs now (before masters): the engine's
    // plan once it has what was published (decisions §24).
    let output = state.output_plan();
    let mut parked = 0usize;
    for guid in &fixtures {
        let Some(fixture) = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .fixtures
            .values()
            .find(|f| f.meta.identity.guid == *guid)
            .cloned()
        else {
            continue;
        };
        let cfs = lumacore_render::merge::channels(&state.root.show_data, &fixture).to_vec();
        for cf in cfs.iter().filter(|cf| !cf.is_virtual) {
            let name = &cf.attribute_name;
            let wanted = match (&attribute, &at) {
                (Some(a), _) => a.eq_ignore_ascii_case(name),
                (None, Some(_)) => {
                    cf.attribute_type == lumacore_fixtures::fixture_type::AttributeType::Dimmer
                }
                (None, None) => true,
            };
            if !wanted {
                continue;
            }
            let value = match &at {
                Some(v) => v.clone(),
                None => output
                    .resolved(*guid, name)
                    .map(|r| r.value.clone())
                    .unwrap_or_else(|| {
                        Value::normalized(lumacore_render::merge::default_fraction(cf) * 100.0)
                    }),
            };
            state.root.show_data.playback.park.park(*guid, name, value);
            parked += 1;
        }
    }
    if parked == 0 {
        return render_code(
            "Nothing parked: no matching attributes on those fixtures.",
            "E-CMD-009",
        );
    }
    crate::text::render_success(&format!(
        "{} fixture{}: {parked} value{} parked",
        fixtures.len(),
        if fixtures.len() == 1 { "" } else { "s" },
        if parked == 1 { "" } else { "s" }
    ))
}

/// What a Park/Unpark names: fixtures, an attribute, a value.
type ParkTarget = (Vec<uuid::Uuid>, Option<String>, Option<Value>);

/// `[<selection>] [<attribute>] [At <value>]` → fixtures, attribute, value.
/// No selection = the current selection.
fn park_target(state: &AppState, args: &[&str]) -> Result<ParkTarget, String> {
    let at = args.iter().position(|a| a.eq_ignore_ascii_case("at"));
    let value = match at {
        Some(i) => match args.get(i + 1) {
            Some(v) => Some(parse_value(v).ok_or_else(|| {
                render_code(format!("'{v}' isn't a value (50, Full, Zero)."), "E-CMD-008")
            })?),
            None => return Err(render_code("At what value?", "E-CMD-006")),
        },
        None => None,
    };
    let head = &args[..at.unwrap_or(args.len())];
    // The selection is the leading numbers/ranges; a word after it names the attribute.
    // tidy: not-keywords (selection grammar)
    let split = head
        .iter()
        .position(|w| {
            !(w.parse::<f64>().is_ok() || matches!(w.to_lowercase().as_str(), "thru" | "+" | "-"))
        })
        .unwrap_or(head.len());
    let attribute = head.get(split).map(|a| a.to_string());
    let fixtures = target_fixtures(state, &head[..split], "Park")?;
    Ok((fixtures, attribute, value))
}

fn park_dmx(state: &mut AppState, args: &[&str]) -> String {
    let Some((universe_id, address)) = args.first().and_then(|a| parse_universe_address(a)) else {
        return render_code(
            "Usage: Park DMX <universe.address> [At <0-255>] (e.g. Park DMX 1.1 At 255)",
            "E-CMD-006",
        );
    };
    if !state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .universes
        .contains_key(&universe_id)
    {
        return render_code(
            format!("Universe {universe_id} does not exist."),
            "E-UNIVERSE-001",
        );
    }
    let value: u8 = if args.len() >= 3 && args[1].eq_ignore_ascii_case("at") {
        match args[2].parse::<u8>() {
            Ok(v) => v,
            Err(_) => {
                return render_code(
                    format!("Invalid DMX value: '{}'. Must be 0–255.", args[2]),
                    "E-CMD-008",
                );
            }
        }
    } else {
        // At the channel's current output.
        state
            .published_frame(universe_id)
            .map_or(0, |buf| buf[address as usize - 1])
    };
    state.root.show_data.playback.park.park_channel(
        state
            .root
            .show_data
            .universe_guid(universe_id)
            .unwrap_or_default(),
        address,
        value,
    );
    crate::text::render_success(&format!("Parked DMX {universe_id}.{address:03} at {value}"))
}

/// Handle: Unpark [Fixture <sel> [<attribute>] | DMX <u.a>]
///
/// - `Unpark`                   — release everything parked
/// - `Unpark Fixture 1 [Pan]`   — release a fixture's parked values (or one)
/// - `Unpark DMX 1.255`         — release a parked channel
pub fn cmd_unpark(state: &mut AppState, args: &[&str]) -> String {
    let park = &mut state.root.show_data.playback.park;
    if args.is_empty() {
        let count = park.fixtures.len() + park.channels.len();
        *park = Default::default();
        if count == 0 {
            return nu_ansi_term::Style::new()
                .dimmed()
                .paint("Nothing is parked.")
                .to_string();
        }
        return crate::text::render_success(&format!(
            "Released all {count} parked value{}",
            if count == 1 { "" } else { "s" }
        ));
    }
    if args[0].eq_ignore_ascii_case("dmx") {
        let Some((universe_id, address)) = args.get(1).and_then(|a| parse_universe_address(a))
        else {
            return render_code("Usage: Unpark DMX <u.a>", "E-CMD-006");
        };
        let guid = state
            .root
            .show_data
            .universe_guid(universe_id)
            .unwrap_or_default();
        let park = &mut state.root.show_data.playback.park;
        return if park.unpark_channel(guid, address) {
            crate::text::render_success(&format!("Released DMX {universe_id}.{address:03}"))
        } else {
            render_code(
                format!("DMX {universe_id}.{address:03} is not parked."),
                "E-CMD-009",
            )
        };
    }
    let names_fixture = crate::objects::find(args[0])
        .is_some_and(|o| std::ptr::eq(o, &crate::objects::FIXTURE));
    let rest = if names_fixture { &args[1..] } else { args };
    let (fixtures, attribute, _) = match park_target(state, rest) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let park = &mut state.root.show_data.playback.park;
    let released: usize = fixtures
        .iter()
        .map(|g| park.unpark(*g, attribute.as_deref()))
        .sum();
    if released == 0 {
        nu_ansi_term::Style::new()
            .dimmed()
            .paint("Nothing parked on those fixtures.")
            .to_string()
    } else {
        crate::text::render_success(&format!(
            "Released {released} parked value{}",
            if released == 1 { "" } else { "s" }
        ))
    }
}

/// Parse a "universe.address" string into (universe_id, address_1indexed).
///
/// Examples: "1.1" -> (1, 1), "2.100" -> (2, 100)
fn parse_universe_address(s: &str) -> Option<(u32, u16)> {
    let mut parts = s.splitn(2, '.');
    let universe: u32 = parts.next()?.parse().ok()?;
    let address: u16 = parts.next()?.parse().ok()?;
    if address == 0 || address > 512 {
        return None;
    }
    Some((universe, address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use lumacore_fixtures::Fixture;
    use uuid::Uuid;

    #[test]
    fn meow_with_nothing_selected_returns_meow() {
        let mut state = AppState::new();
        assert_eq!(cmd_please(&mut state, &[]), "meow");
    }

    #[test]
    fn meow_with_programmer_values_but_no_selection_returns_meow() {
        let mut state = AppState::new();
        let prog = state.current_programmer_mut().expect("admin programmer");
        prog.set_value(
            Uuid::new_v4(),
            "Dimmer",
            lumacore_tree::Value::normalized(100.0),
        );
        assert_eq!(cmd_please(&mut state, &[]), "meow");
    }

    #[test]
    fn fixture_thru_selects_all_fixtures() {
        let mut state = AppState::new();
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("One".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );
        state.root.show_data.devices.dmx.patch.insert_fid(3, Fixture::new(Some("Three".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );
        state.root.show_data.devices.dmx.patch.insert_fid(7, Fixture::new(Some("Seven".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );

        let output = cmd_fixture_at(&mut state, &["Thru"]);
        assert!(output.contains("Selected"), "unexpected output: {output}");

        let selection_guids = state
            .current_programmer()
            .expect("programmer")
            .selected_fixtures
            .clone();
        let selection_ids: Vec<u32> = selection_guids
            .iter()
            .filter_map(|g| state.root.show_data.fixture_id_for_guid(*g))
            .collect();
        assert_eq!(selection_ids, vec![1, 3, 7]);
    }
}
