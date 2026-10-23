//! Oops / Meow (undo / redo) command handlers and undo-list display.
//!
//! CMD-20: `Oops` — undo the last mutating command.
//! CMD-21: `Meow` — redo the last undone action.
//!
//! Subtree-granular undo: each frame records only the affected subtrees of
//! ShowData.  Conflict detection blocks undo when another user has touched
//! the same subtree since the original action.
//!
//! Each user's history lives in `Runtime.Users.{User}.Undo`: replicated,
//! never saved (decisions §21 "Step 6 hardening").

use lumacore_tree::{ShowSubtree, SubtreeSnapshot};
use nu_ansi_term::{Color, Style};

use crate::AppState;

pub use crate::oops::*;
use crate::claims::Claims;

/// After an Oops / Redo put a profile back: live screens are laid out from
/// the saved layouts again (a UI step undone; a screen the step first saved
/// goes back to the default layout).
fn follow_ui(state: &mut AppState, snapshots: &[SubtreeSnapshot]) {
    if snapshots
        .iter()
        .any(|s| matches!(s.subtree, ShowSubtree::Profile(_)))
    {
        crate::commands::screens::restore_all(state);
    }
}

// ─── Confirmation ────────────────────────────────────────────────────────────

/// The step count of `Oops [<n>]` and whether it was confirmed
/// (`/NoConfirmation`, `/NC`).
fn oops_args(args: &[&str]) -> (usize, bool) {
    let count = args
        .iter()
        .find_map(|a| a.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let confirmed = crate::commands::flags::has(args, &crate::commands::flags::NO_CONFIRMATION);
    (count, confirmed)
}

/// Oops Confirmation (profile setting): before an `Oops` line runs (on a
/// member too, before it goes to the master), ask when the setting is
/// Always, or Main and one of the steps it would undo is a Main action.
/// `None` = go ahead.
pub fn confirm_first(state: &mut AppState, line: &str) -> Option<crate::DispatchResult> {
    // Every statement of the line (R1-62: `Reset; Oops` asks too); the
    // answer re-runs the whole line with each Oops confirmed.
    let statements = super::tokens::statements(line);
    let mut count: usize = 0;
    let mut rewritten = Vec::with_capacity(statements.len());
    for statement in &statements {
        let tokens = super::tokenize(statement);
        match tokens.split_first() {
            Some((first, rest)) if super::registry::is_oops(first) => {
                let args: Vec<&str> = rest.iter().map(String::as_str).collect();
                let (n, confirmed) = oops_args(&args);
                if confirmed {
                    rewritten.push(statement.clone());
                } else {
                    count = count.saturating_add(n);
                    rewritten.push(format!("Oops {n} /NoConfirmation"));
                }
            }
            _ => rewritten.push(statement.clone()),
        }
    }
    if count == 0 {
        return None;
    }
    let user = state.effective_user_name().to_string();
    let setting = state
        .root
        .show_data
        .profile_for_user(&user)
        .map(|p| p.settings.preferences.oops.confirmation)?;
    let frames: Vec<&lumacore_tree::UndoFrame> = state
        .undo_stack(&user)?
        .frames
        .iter()
        .rev()
        .take(count)
        .collect();
    let ask = match setting {
        lumacore_tree::OopsConfirmation::Always => !frames.is_empty(),
        lumacore_tree::OopsConfirmation::Main => frames.iter().any(|f| f.main),
        lumacore_tree::OopsConfirmation::Never => false,
    };
    if !ask {
        return None;
    }
    let what = if frames.len() == 1 {
        format!("Oops \"{}\"?", frames[0].description)
    } else {
        format!(
            "Oops {} steps (newest: \"{}\")?",
            frames.len(),
            frames[0].description
        )
    };
    let command = rewritten.join("; ");
    let mut ask = crate::dialogs::Ask::new("Oops", what)
        .meta(format!("OopsConfirmation · {setting:?}"), None);
    for f in &frames {
        ask = ask.row(format!("{:?}", f.category), f.description.clone());
    }
    Some(crate::DispatchResult::Ask(Box::new(
        ask.button("Oops", lumacore_wire::ButtonKind::Primary, command)
            .cancel("Cancel"),
    )))
}

// ─── Conflict detection ──────────────────────────────────────────────────────

/// Check if undoing a frame would conflict with another user's modifications.
///
/// Returns a list of conflict messages for subtrees that have been modified
/// by a different user since the frame was created.
fn check_conflicts(
    state: &AppState,
    frame: &lumacore_tree::UndoFrame,
    current_user: &str,
) -> Vec<String> {
    let mut conflicts = Vec::new();
    for snap in &frame.snapshots {
        if let Some((owner, seq_id)) = state.subtree_owners.get(&snap.subtree)
            && !owner.eq_ignore_ascii_case(current_user)
            && *seq_id > frame.sequence_id
        {
            conflicts.push(format!(
                "{} was modified by {} since your action",
                snap.subtree, owner
            ));
        }
    }
    conflicts
}

// ─── cmd_oops ─────────────────────────────────────────────────────────────────

/// CMD-20: `Oops [count]` — undo the last mutating command.
///
/// Steps back through the undo history. `Oops 3` undoes 3 commands at once.
/// Operates silently on success; prints "Nothing to undo." if history is empty.
/// Blocks undo with a conflict message if another user has touched the same subtrees.
pub fn cmd_oops(state: &mut AppState, args: &[&str]) -> String {
    // "Oops 3" undoes 3 steps; `/NoConfirmation` was handled before it ran.
    let (count, _) = oops_args(args);

    let user = state.effective_user_name().to_string();

    let mut undone = 0;
    let mut warnings = Vec::new();

    for _ in 0..count {
        // Peek at the top frame to check for conflicts before popping.
        let frame_ref = state.undo_stack(&user).and_then(|u| u.frames.back());

        let frame_ref = match frame_ref {
            Some(f) => f,
            None => break, // No more frames
        };

        // Check conflicts
        let conflicts = check_conflicts(state, frame_ref, &user);
        if !conflicts.is_empty() {
            let msg = format!("Oops blocked — {}", conflicts.join("; "));
            if undone == 0 {
                return crate::errors::render_code(&msg, "E-CMD-010");
            } else {
                warnings.push((msg, "E-CMD-010"));
                break;
            }
        }

        // All or nothing (R1-64): restore from the peeked frame first; the
        // frame leaves the stack only when every subtree went back.
        let snapshots = frame_ref.snapshots.clone();
        let current_snapshots = match restore_all_or_nothing(state, &snapshots, "Undo") {
            Ok(current) => current,
            Err((msg, code)) => {
                if undone == 0 {
                    return crate::errors::render_code(&msg, code);
                }
                warnings.push((msg, code));
                break;
            }
        };
        let Some(frame) = state.undo_stack_mut(&user).pop_undo(current_snapshots, "") else {
            break;
        };
        state.note_undo_step(&user, crate::UndoKind::Undo);
        undone += 1;
        state.show_dirty = true;
        restore_local(state, &frame.local, Side::Before);
        follow_ui(state, &frame.snapshots);
        crate::references::check(state, "Oops");
    }

    if undone == 0 {
        Style::new().dimmed().paint("Nothing to undo.").to_string()
    } else {
        let mut msg = if undone == 1 {
            crate::text::render_success("Undone.")
        } else {
            crate::text::render_success(&format!("Undone {undone} steps."))
        };
        for (w, code) in warnings {
            msg.push('\n');
            msg.push_str(&crate::errors::render_code(&w, code));
        }
        msg
    }
}

// ─── cmd_meow ─────────────────────────────────────────────────────────────────

/// CMD-21: `Redo` — redo the last undone action. Like Oops it is refused
/// when another user changed one of its subtrees since (R1-63), and it is
/// all or nothing (R1-64).
/// `Redo [<n>]`: redo the newest n undone steps (the Oops window's "Redo n", decisions §30 "12f"),
/// stopping at the first that can't be redone.
/// `Redo [<steps>]` — the first typed verb (decisions §33 part 2).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Redo")]
pub struct Redo {
    #[lum(value)]
    pub steps: Option<u32>,
}

impl crate::commands::typed::Command for Redo {
    fn run(self, state: &mut AppState) -> crate::DispatchResult {
        crate::DispatchResult::Output(cmd_redo(state, self.steps).into())
    }
}

pub fn cmd_redo(state: &mut AppState, steps: Option<u32>) -> String {
    let count = match steps {
        None => 1,
        Some(0) => return crate::errors::render_code("'0' is not a number of steps.", "E-CMD-008"),
        Some(n) => (n as usize).min(1000),
    };
    if count == 1 {
        return cmd_meow(state, &[]);
    }
    let mut done = 0;
    for _ in 0..count {
        let out = cmd_meow(state, &[]);
        if !out.contains("Redone") {
            if done == 0 {
                return out;
            }
            break;
        }
        done += 1;
    }
    crate::text::render_success(&format!("Redone {done} step{}.", if done == 1 { "" } else { "s" }))
}

pub fn cmd_meow(state: &mut AppState, _args: &[&str]) -> String {
    let user = state.effective_user_name().to_string();

    // Peek at the redo frame: conflicts first, then restore, then pop.
    let Some(redo_ref) = state.undo_stack(&user).and_then(|u| u.redo_frames.back()) else {
        return Style::new().dimmed().paint("Nothing to redo.").to_string();
    };
    let conflicts = check_conflicts(state, redo_ref, &user);
    if !conflicts.is_empty() {
        return crate::errors::render_code(
            format!("Redo blocked — {}", conflicts.join("; ")),
            "E-CMD-010",
        );
    }
    let snapshots = redo_ref.snapshots.clone();
    let current_snapshots = match restore_all_or_nothing(state, &snapshots, "Redo") {
        Ok(current) => current,
        Err((msg, code)) => return crate::errors::render_code(&msg, code),
    };
    let Some(frame) = state.undo_stack_mut(&user).pop_redo(current_snapshots, "") else {
        return Style::new().dimmed().paint("Nothing to redo.").to_string();
    };
    state.note_undo_step(&user, crate::UndoKind::Redo);
    state.show_dirty = true;
    restore_local(state, &frame.local, Side::After);
    follow_ui(state, &frame.snapshots);
    crate::references::check(state, "Redo");
    crate::text::render_success("Redone.")
}

/// Put `snapshots` back as one unit (R1-64): the current state of the same
/// subtrees is taken first and returned (for the other stack); when any
/// subtree can't go back, everything is put back as it was and the reason
/// is returned — the show is never left half undone.
fn restore_all_or_nothing(
    state: &mut AppState,
    snapshots: &[SubtreeSnapshot],
    what: &str,
) -> Result<Vec<SubtreeSnapshot>, (String, &'static str)> {
    let subtrees: Vec<ShowSubtree> = snapshots.iter().map(|s| s.subtree.clone()).collect();
    let Some(current) = snapshot_subtrees(state, &subtrees) else {
        return Err((format!("{what} failed — could not serialise the current state."), "E-CMD-010"));
    };
    match restore_subtrees(state, snapshots) {
        Ok(()) => {
            // Another user's claimed object (R1-44): refused here, before the
            // frame moves to the other stack — dispatch's own claim check
            // would roll the show back after the step was already moved.
            let changes: Vec<crate::change::Change> = current
                .iter()
                .zip(snapshots)
                .map(|(before, after)| crate::change::Change {
                    subtree: before.subtree.clone(),
                    before: before.data.clone(),
                    after: after.data.clone(),
                })
                .collect();
            if let Some((label, owner)) = state.claim_changed_by(&changes) {
                if let Err(back) = restore_subtrees(state, &current) {
                    log::error!("{what}: putting the show back failed too: {back}");
                }
                state.root.show_data.after_load();
                return Err((
                    format!("{what} would change {label}, claimed by {owner} — nothing changed. Claim {label} asks them for it."),
                    "E-CLAIM-001",
                ));
            }
            Ok(current)
        }
        Err(e) => {
            if let Err(back) = restore_subtrees(state, &current) {
                log::error!("{what}: putting the show back failed too: {back}");
            }
            state.root.show_data.after_load();
            Err((format!("{what} failed, nothing changed — {e}"), "E-CMD-010"))
        }
    }
}

// ─── cmd_list_undo ────────────────────────────────────────────────────────────

/// `List Undo [/<category>] [/ThisStation]`: the user's Oops list, newest
/// first and numbered (`Oops <n>` undoes down to entry n), each with its
/// category (a coloured label, decisions §29 "11c"), Main mark, the station
/// it came from and when (SessionTime / InMySession, §21 "6c").
/// `/<category>` keeps only that category, `/ThisStation` this station's.
pub fn cmd_list_undo(state: &AppState, args: &[&str]) -> String {
    let flags: Vec<&str> = args.iter().map(|a| a.trim_start_matches('/')).collect();
    let only_here = flags.iter().any(|a| a.eq_ignore_ascii_case("thisstation"));
    let mut only_cat = None;
    for flag in flags.iter().filter(|a| !a.eq_ignore_ascii_case("thisstation")) {
        match lumacore_tree::OopsCategory::parse(flag) {
            Some(cat) => only_cat = Some(cat),
            None => {
                let names: Vec<&str> = lumacore_tree::OopsCategory::ALL
                    .iter()
                    .map(|c| c.label())
                    .collect();
                return crate::errors::render_code(
                    format!(
                        "'{flag}' isn't an Oops category: {} (or /ThisStation)",
                        names.join(", ")
                    ),
                    "E-CMD-006",
                );
            }
        }
    }
    let here = state.root.station_settings.station_name.clone();
    let user = state.effective_user_name();
    let Some(stack) = state.undo_stack(user) else {
        return Style::new()
            .dimmed()
            .paint("Undo history is empty.")
            .to_string();
    };

    if stack.is_empty() {
        return Style::new()
            .dimmed()
            .paint("Undo history is empty.")
            .to_string();
    }

    let mut lines = Vec::new();
    lines.push(Style::new().bold().paint("Undo History").to_string());

    for (i, frame) in stack.frames.iter().rev().enumerate() {
        if only_here && !frame.station.is_empty() && frame.station != here {
            continue;
        }
        if only_cat.is_some_and(|c| c != frame.category) {
            continue;
        }
        let mut desc = format!(
            "{} {}",
            category_color(frame.category).paint(format!("{:<10}", frame.category.label())),
            frame.description
        );
        if frame.main {
            desc.push_str(&Color::Yellow.paint("  Main").to_string());
        }
        if !frame.station.is_empty() {
            desc.push_str(&format!(
                "  {}",
                Style::new().dimmed().paint(format!(
                    "{} · {}",
                    frame.station,
                    super::list::local_time(frame.time_ms)
                ))
            ));
        }
        let marker = if i == 0 {
            Color::Cyan.paint(format!("  [{:>2}]", i + 1)).to_string()
        } else {
            Style::new()
                .dimmed()
                .paint(format!("  [{:>2}]", i + 1))
                .to_string()
        };
        lines.push(format!("{marker}  {desc}"));
    }

    // Memory summary.
    let usage = stack.memory_usage();
    let budget = stack.max_bytes;
    let usage_kb = usage / 1024;
    let budget_kb = budget / 1024;
    let pct = (usage * 100).checked_div(budget).unwrap_or(0);
    lines.push(String::new());
    lines.push(format!(
        "  {}  {} KB / {} KB  ({}%)  [{}/{} frames]",
        Style::new().dimmed().paint("Memory:"),
        Color::Green.paint(format!("{usage_kb}")),
        budget_kb,
        pct,
        stack.len(),
        stack.max_frames,
    ));

    lines.join("\n")
}

/// The colour of a category's label (terminal; the UI kit picks its own).
fn category_color(category: lumacore_tree::OopsCategory) -> Color {
    use lumacore_tree::OopsCategory as C;
    match category {
        C::Programmer => Color::Green,
        C::Selection => Color::Cyan,
        C::Show => Color::Blue,
        C::Patch => Color::Magenta,
        C::UI => Color::LightBlue,
        C::Users => Color::LightRed,
        C::Settings => Color::LightYellow,
        C::Keys => Color::LightPurple,
        C::Station => Color::Red,
    }
}
