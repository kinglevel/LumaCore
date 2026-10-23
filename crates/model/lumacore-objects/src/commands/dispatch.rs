//! Main command dispatch — routes raw command lines to handlers.
//!
//! Dispatch never matches on verb strings: the first word is looked up in the
//! one namespace of commands and objects (`registry::resolve`) and its handler
//! runs. Whether it mutates (and its undo category) comes from the same entry;
//! *what* it changed comes from the change path (`terminal::change`), which
//! files the undo frame, the replication op and the journal entry. Every line
//! runs inside a panic guard: a panic rolls the show back and the reply says
//! where it happened (decisions.md §26).

use lumacore_tree::runtime::{JournalEntry, JournalResult};
use lumacore_tree::{
    LocalSnapshot, LocalSubtree, OopsCategory, OopsKind, ShowSubtree, SubtreeSnapshot, UndoFrame,
};

use crate::change::{self, Change};
use crate::DispatchResult;
use crate::{AppState, ReplContext};

use super::registry::{self, Undo, Word};
use super::sysmon::{report_tree_changes, tree_snapshot};
use super::hooks::{Hook, hooks};
use super::{edit, help, tokenize};
use crate::claims::Claims;

/// Dispatch a raw command line to the appropriate handler.
///
/// Supports semicolon-separated commands: `list; help` dispatches two commands.
/// The first non-Output result from any sub-command short-circuits the chain
/// (e.g., Exit or ContextSwitch propagates immediately). The whole line is one
/// change: one undo frame, one replication op, one journal entry.
///
/// Returns a `DispatchResult` indicating output, exit, or context switch.
/// A handler that answers with text but has a question (decisions §30
/// "12b") leaves it in `AppState::asking`: the line's result is the ask.
fn asked(state: &mut AppState, result: DispatchResult) -> DispatchResult {
    let result = match state.asking.take() {
        Some(ask) => DispatchResult::Ask(Box::new(ask)),
        None => result,
    };
    // An answer is a line like any other: a flag its words don't declare would be refused
    // (decisions §31, E-CMD-018) — a bug in the ask, caught where it leaves dispatch.
    if let DispatchResult::Ask(ask) = &result {
        for line in ask.button_lines() {
            debug_assert!(
                super::line::Line::parse(line).map_or(true, |l| l.unknown_flags().next().is_none()),
                "dialog button line with an undeclared flag: {line}"
            );
        }
    }
    result
}

/// How deep lines may run lines (decisions §34 16, Vim's `do_cmdline` guard): a key or a macro
/// that runs itself stops here with E-CMD-019 instead of overflowing the stack.
pub const MAX_NESTED_DEPTH: u32 = 32;

pub fn dispatch(state: &mut AppState, line: &str) -> DispatchResult {
    // Nested dispatch (macros, confirmations) is part of the outer line.
    if state.dispatch_depth > 0 {
        if state.dispatch_depth >= MAX_NESTED_DEPTH {
            let msg = format!("`{line}` runs lines {MAX_NESTED_DEPTH} deep — a line that runs itself? It didn't run");
            return DispatchResult::Output(crate::Reply::error(msg, "E-CMD-019"));
        }
        state.dispatch_depth += 1;
        let result = dispatch_inner(state, line);
        state.dispatch_depth -= 1;
        return asked(state, result);
    }

    // A family's gate refuses a line before anything runs (a locked device runs only its lock
    // keys, decisions §30 "12a3").
    for hook in hooks() {
        if let Hook::Gate(gate) = hook
            && let Some(refusal) = gate(state, line)
        {
            return DispatchResult::Output(refusal.into());
        }
    }

    // A flag no word declares is refused before anything runs, forwards or
    // asks (decisions §31, E-CMD-018) — ignoring it silently would let a typo like
    // `/Overwite` would store with the default.
    if let Some(refused) = super::line::Line::parse(line).ok().and_then(|l| l.unknown_flag_refusal()) {
        return DispatchResult::Output(refused.into());
    }

    // Every statement of a line is its own step (R1-62, Q10): each
    // runs through the whole path below — Oops confirmation, the Delete
    // question, forwarding — when it is reached, so `Clear; Oops` confirms
    // the step Oops really undoes. Rights: all or nothing, checked first.
    let parsed = super::line::Line::parse(line).ok();
    if let Some(parsed) = parsed.as_ref().filter(|p| p.statements.len() > 1) {
        if let Err(refused) = rights_of_all(state, parsed) {
            return DispatchResult::Output((format!("{refused} — nothing in `{line}` ran")).into());
        }
        let parts: Vec<String> = parsed.statements.iter().map(|s| s.text.clone()).collect();
        let mut outputs: Vec<crate::Reply> = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            match dispatch(state, part) {
                DispatchResult::Output(s) => outputs.push(s),
                // A statement asks: the rest of the line runs with the answer.
                DispatchResult::Ask(ask) => {
                    let rest = parts[i + 1..].join("; ");
                    return DispatchResult::Ask(Box::new(if rest.is_empty() { *ask } else { ask.then_run(&rest) }));
                }
                other => return other,
            }
        }
        return DispatchResult::Output(crate::Reply::joined(outputs));
    }

    // `Self` is the line's owner (decisions §30 "12c"): resolved here, where
    // the line starts — the master can't see this station's focus — so the
    // forwarded line, the journal, Oops and sysmon name the window. Usage
    // keeps what was typed.
    // A bare number or name is written out first (decisions §33 21; issue 02).
    let written = write_out_bare(state, line);
    let line = written.as_deref().unwrap_or(line);
    // A family may write the line out as the Lum line it means (the tree shell's `mv` / `cp` /
    // `rm` on the show tree, §35 19).
    let mut fenced = None;
    for hook in super::hooks::hooks() {
        if let super::hooks::Hook::Rewrite(rewrite) = hook {
            match rewrite(state, line) {
                Some(Ok(lum)) => {
                    fenced = Some(lum);
                    break;
                }
                Some(Err(e)) => return DispatchResult::Output(e.into()),
                None => {}
            }
        }
    }
    let line = fenced.as_deref().unwrap_or(line);
    let typed = line;
    state.resolved_line = None;
    let resolved = match crate::self_ref::resolve_line(state, line) {
        Ok(r) => r,
        Err(e) => return DispatchResult::Output(e.into()),
    };
    state.resolved_line.clone_from(&resolved);
    let line = resolved.as_deref().unwrap_or(line);

    // Oops Confirmation: asked here, before the line runs or goes to
    // the master (a member holds a mirror of the user's Oops list).
    if let Some(asked) = super::oops::confirm_first(state, line) {
        return asked;
    }

    // Delete is asked by the station (R1-61, Q35; Delete /NoConfirmation):
    // here, before the line runs or goes to the master. The answer carries
    // `/NoConfirmation` (also when forwarded, so the master doesn't ask
    // again); `dispatch_single` takes it out just before the handler.
    if !state.running_stored_line
        && let Some(ask) = delete_first(state, line)
    {
        return ask;
    }

    // A member sends what changes the show or a user's state to its master,
    // which runs it as this user and sends the result back (decisions §10,
    // §21 "6b"). Station-local lines run here.
    if state.link.is_some()
        && !state.replaying_remote_op
        && registry::line_scope(line) == registry::Scope::Master
    {
        return crate::session_link::forward(state, line);
    }

    // `/NoOops`: the line runs as usual but files no Oops step.
    let stripped = strip_no_oops(line);
    let line = stripped.as_deref().unwrap_or(line);
    state.no_oops = stripped.is_some();

    // Passwords are masked once, before the line reaches the journal, undo,
    // replication or sysmon (decisions §19).
    let shown = crate::passwords::mask_secrets(line);

    // Capture tree counts before dispatch for sysmon change detection.
    // Intentionally excludes spatial_3d (too noisy) and effect_engines.
    let pre = tree_snapshot(state);
    let undo = undo_of_line(line);
    // A gesture's line (decisions §30 "12f"): its end starts from where the drag began.
    crate::gesture::before_line(state);
    // While someone holds the Devices draft, a live line runs against the live devices: the
    // normal change path records it, then the draft follows (decisions §34 "34g4 patch
    // hardened" 8). `live_base` is the live devices before it.
    let mut live_base = super::live_draft::runs_live(state, line).then(|| {
        state.swap_live_devices();
        serde_json::to_value(&state.root.show_data.devices).unwrap_or_default()
    });
    let mut ran_live = false;
    let pass = undo.is_mutating().then(|| change::begin(state));
    // The template profile is locked (decisions §30 "12c2"): whatever path
    // a line takes, it never changes Default. A line that replaces the whole
    // show (LoadShow, NewShow, CleanStart) brings the new show's Default.
    let default_profile = state.root.show_data.default_profile().map(|(_, p)| p.meta.identity.guid);
    let show_loads = state.show_loads;
    // Station-local state a step can put back (decisions §29 "11c").
    let local_before = undo.kind().map(|_| local_images(state));

    state.dispatch_depth += 1;
    state.undo_to_owners = false;
    let mutating = pass.is_some();
    let run = lumacore_sysmon::panic_report::guarded(|| {
        let result = dispatch_inner(state, line);
        // A deleted DataPool / Sequence / Timecode leaves no selection
        // behind (decisions §29); inside the change pass, so it replicates.
        // A read-only line deletes nothing (and has no pass to record it).
        if mutating {
            state.clear_dangling_selections();
        }
        result
    });
    state.dispatch_depth = 0;

    // Whether the change path filed anything (the sysmon line below).
    let mut modified = false;
    let result = match run {
        Ok(result) => {
            let result = asked(state, result);
            if let Some(before) = local_before {
                let after = local_images(state);
                state.pending_local.extend(local_changes(state, before, after));
            }
            if let Some(pass) = pass {
                // What the user used ranks their suggestions (decisions §30
                // "12b2"): inside the pass, so it is saved and replicated
                // like any change — and never undone (`ShowSubtree::Usage`).
                // A member leaves it to its master, which runs the user's
                // show lines.
                if !state.replaying_remote_op
                    && state.link.is_none()
                    && state.gesture.is_none()
                    && !matches!(result, DispatchResult::Ask(_))
                    && matches!(journal_result(&result), JournalResult::Ok)
                {
                    let typed = crate::passwords::mask_secrets(&strip_no_oops(typed).unwrap_or_else(|| typed.to_string()));
                    for hook in hooks() {
                        if let Hook::Used(used) = hook {
                            used(state, &typed);
                        }
                    }
                }
                let mut changes = finish_changes(state, pass);
                modified = changes.iter().any(|c| super::sysmon::reported(&c.subtree));
                if let Some(base) = live_base.take() {
                    let rebased = super::live_draft::rebase_draft(state, &base);
                    if let Err(msg) = rebased {
                        refuse(state, &changes);
                        state.swap_live_devices();
                        journal(state, &shown, Vec::new(), journal_result(&DispatchResult::Output(msg.clone().into())));
                        return DispatchResult::Output(msg.into());
                    }
                    state.swap_live_devices();
                    // The draft read back from its JSON: its id pools are rebuilt.
                    state.root.show_data.after_load();
                    ran_live = true;
                }
                if let Some(guid) = default_profile
                    && !state.replaying_remote_op
                    && state.show_loads == show_loads
                    && state.effective_user_name() != crate::INTERNAL_ROOT_USER
                    && !state.synced_default
                    && changes.iter().any(|c| c.subtree == lumacore_tree::ShowSubtree::Profile(guid))
                {
                    refuse(state, &changes);
                    state.pending_local.clear();
                    state.no_oops = false;
                    let msg = crate::errors::render_code(DEFAULT_LOCKED, "E-USER-002");
                    journal(state, &shown, Vec::new(), journal_result(&DispatchResult::Output(msg.clone().into())));
                    return DispatchResult::Output(msg.into());
                }
                // A Devices draft is open (decisions §11 "staged patch"): the
                // owner's patch edits (the Devices subtree) stay in the draft
                // (no undo step, no replication until Keep); anyone else's
                // are refused.
                if state.patch_draft.is_some()
                    && !ran_live
                    && changes.iter().any(|c| c.subtree == lumacore_tree::ShowSubtree::Devices)
                {
                    if let Some(owner) = state.patch_draft_of_other().map(str::to_string) {
                        refuse(state, &changes);
                        let msg = crate::errors::render_code(
                            format!(
                                "Devices are being edited by {owner} (draft): try again after they keep or discard it."
                            ),
                            "E-PATCH-008",
                        );
                        journal(state, &shown, Vec::new(), journal_result(&DispatchResult::Output(msg.clone().into())));
                        return DispatchResult::Output(msg.into());
                    }
                    changes.retain(|c| c.subtree != lumacore_tree::ShowSubtree::Devices);
                }
                // A line that changed another user's claimed object — in
                // whatever shape it named it — is rolled back (R1-44, §21
                // "6d"); `held_elsewhere` is only the early refusal. The
                // master already checked what it sends back.
                let held = (!state.replaying_remote_op)
                    .then(|| state.claim_changed_by(&changes))
                    .flatten();
                if let Some((label, owner)) = held {
                    refuse(state, &changes);
                    let msg = crate::errors::render_code(
                        format!("{label} is claimed by {owner}: this line would change it. Claim {label} asks them for it."),
                        "E-CLAIM-001",
                    );
                    journal(state, &shown, Vec::new(), journal_result(&DispatchResult::Output(msg.clone().into())));
                    DispatchResult::Output(msg.into())
                } else {
                    let outcome = journal_result(&result);
                    commit(state, &shown, undo, &changes, outcome);
                    result
                }
            } else {
                result
            }
        }
        Err(report) => {
            // A handler that panicked mid owner-first line left its DataPool set.
            state.datapool_override = None;
            let user = state.effective_user_name().to_string();
            let build = state.root.diagnostics.build_info.build_id.clone();
            let restored = match pass.map(|pass| change::rollback(state, pass)) {
                Some(Ok(subtrees)) => subtrees.iter().map(|s| s.to_string()).collect(),
                Some(Err(e)) => {
                    log::error!("rollback after report #{} failed: {e}", report.number);
                    Vec::new()
                }
                None => Vec::new(),
            };
            journal(
                state,
                &shown,
                restored,
                JournalResult::Internal {
                    report: report.number,
                },
            );
            DispatchResult::Output(crate::errors::render_code(report.text(&shown, &user, &build), "E-INTERNAL-001").into())
        }
    };

    // A live line that filed nothing (read only, or it panicked): the draft comes back.
    if live_base.is_some() {
        state.swap_live_devices();
    }
    state.no_oops = false;
    state.pending_local.clear();

    // Detect and report tree mutations via sysmon_pending.
    if let DispatchResult::Output(ref output) = result {
        let post = tree_snapshot(state);
        report_tree_changes(state, &pre, &post, &shown, output, modified);
    }

    result
}

/// The refusal for the Default profile, the template nobody holds (E-USER-002): dispatch
/// refuses every change to it (decisions §30 "12c2").
pub const DEFAULT_LOCKED: &str =
    "Default is locked: a template for new profiles. Make your own (Store UserProfile \"name\") and change that.";

/// The undo/replication behaviour of one command: its first word's entry.
fn undo_of(words: &[String]) -> Undo {
    // A grid's columns on your profile are never an Oops step (decisions
    // §30 "12d"): replicated and saved, like the menus' last tab.
    if words.len() > 1
        && matches!(registry::resolve(&words[0]), Ok(registry::Word::Command(c)) if matches!(c.object_verb, Some(crate::objects::Verb::Set | crate::objects::Verb::Reset)))
        && hooks().any(|h| matches!(h, Hook::SetUntracked(o) if o.matches(&words[1])))
    {
        return Undo::Untracked;
    }
    // A keyword alone sets your default object: replicated, never an Oops step (§33 21).
    if let [only] = words
        && let Ok(registry::Word::Object(obj)) = registry::resolve(only)
        && obj.lum.counted()
    {
        return Undo::Untracked;
    }
    words
        .first()
        .and_then(|w| registry::resolve(w).ok())
        .map_or(Undo::ReadOnly, |word| word.undo())
}

/// A bare number or name names a default object, written out before anything else sees the
/// line (decisions §33 21, 25, 39): it runs, forwards, journals and files as the full line.
/// At the start, your default object (`3 At 50` = `Group 3 At 50` with `Group>`, else
/// `Fixture 3 At 50`); after a verb, the verb's own default object (`Store 2` = `Store Cue 2`),
/// else yours when it takes that verb (`Delete 3` = `Delete Group 3`).
pub fn write_out_bare(state: &AppState, line: &str) -> Option<String> {
    use lumacore_syntax::TokenKind as K;
    let tokens = lumacore_syntax::lex(line);
    let first = tokens.first()?;
    let is_thru = |t: &lumacore_syntax::Token| t.kind == K::Word && t.text(line).eq_ignore_ascii_case("thru");
    if matches!(first.kind, K::Number | K::Quoted { .. }) || is_thru(first) {
        return Some(format!("{} {}", crate::objects::default_object(state).keyword, line.trim_start()));
    }
    let second = tokens.get(1).filter(|t| t.kind == K::Number)?;
    if first.kind != K::Word {
        return None;
    }
    let Ok(registry::Word::Command(cmd)) = registry::resolve(first.text(line)) else {
        return None;
    };
    let verb = cmd.object_verb?;
    let object = match cmd.default_object.and_then(crate::objects::find) {
        Some(own) => own,
        None => {
            let mine = crate::objects::default_object(state);
            if !(mine.supports(verb) || mine.is_planned(verb)) {
                return None;
            }
            mine
        }
    };
    let at = second.span.start as usize;
    Some(format!("{}{} {}", &line[..at], object.keyword, &line[at..]))
}

/// A whole line (`a; b`): tracked if any part is, filed under the last
/// tracked part's category.
pub fn undo_of_line(line: &str) -> Undo {
    match super::line::Line::parse(line) {
        Ok(parsed) => undo_of_parsed(&parsed),
        // A line that doesn't lex: its first word decides, as typed.
        Err(_) => undo_of(&tokenize(line)),
    }
}

/// [`undo_of_line`] of a line parsed once (decisions §31).
pub fn undo_of_parsed(line: &super::line::Line) -> Undo {
    line.statements
        .iter()
        .map(|s| undo_of(&s.words))
        .fold(Undo::ReadOnly, |acc, undo| match (acc, undo) {
            (_, Undo::Tracked(_)) => undo,
            (Undo::ReadOnly, Undo::Untracked) => undo,
            _ => acc,
        })
}

/// How a command ended, from its output: an error line (with its code if it
/// has one) or ok.
fn journal_result(result: &DispatchResult) -> JournalResult {
    match result {
        DispatchResult::Output(reply) if reply.failed => JournalResult::Error(reply.code.clone()),
        _ => JournalResult::Ok,
    }
}

/// File what a mutating line (already masked) changed: undo frame,
/// replication op, journal. Suppressed (except the journal) while replaying a remote op.
fn commit(
    state: &mut AppState,
    line: &str,
    undo: Undo,
    changes: &[Change],
    outcome: JournalResult,
) {
    // Usage rides along with every line (decisions §30 "12b2"): the
    // journal names what the command itself changed.
    journal(
        state,
        line,
        changes
            .iter()
            .filter(|c| !matches!(c.subtree, ShowSubtree::Usage(_)))
            .map(|c| c.subtree.to_string())
            .collect(),
        outcome,
    );
    let local = std::mem::take(&mut state.pending_local);
    if (changes.is_empty() && local.is_empty()) || state.replaying_remote_op {
        return;
    }
    // `/NoOops`: replicated and journaled, no Oops step.
    // A gesture's move files no step; its end files the one (from where it began).
    let moving = matches!(state.gesture, Some(crate::gesture::GestureLine::Move { .. }));
    if moving {
        crate::gesture::remember(state, changes);
    }
    let undo = if (state.no_oops || moving) && undo.kind().is_some() {
        Undo::Untracked
    } else {
        undo
    };
    // A member's own change (live screens writing through, a station-local
    // command that touched the show or the station): the master files it
    // (§21 "6b").
    if state.link.is_some() {
        let after: Vec<SubtreeSnapshot> = changes
            .iter()
            .map(|c| SubtreeSnapshot {
                subtree: c.subtree.clone(),
                data: c.after.clone(),
            })
            .collect();
        crate::session_link::submit_changes(state, line, undo, &after, local);
        return;
    }
    // Running playback state is never undone (Oops never undoes a Go).
    if let Some(kind) = undo.kind()
        && (changes.iter().any(|c| c.subtree.is_undoable()) || !local.is_empty())
    {
        let category = step_category(kind.category, changes, &local);
        if files_step(state, category) {
            let before = changes
                .iter()
                .filter(|c| c.subtree.is_undoable())
                .map(|c| SubtreeSnapshot {
                    subtree: c.subtree.clone(),
                    data: c.before.clone(),
                })
                .collect();
            push_subtree_undo_frame(
                state,
                line.to_string(),
                before,
                OopsKind {
                    category,
                    main: kind.main,
                },
                local,
            );
        }
    }
    if changes.is_empty() {
        return;
    }
    let mut after: Vec<SubtreeSnapshot> = changes
        .iter()
        .map(|c| SubtreeSnapshot {
            subtree: c.subtree.clone(),
            data: c.after.clone(),
        })
        .collect();
    // A live patch change raises the patch version (never undone, so a
    // rejoin can tell whether the patch changed since a split — §21 "6c").
    if changes.iter().any(|c| c.subtree == lumacore_tree::ShowSubtree::Devices) {
        state.root.show_data.session.patch_version += 1;
        after.retain(|s| s.subtree != lumacore_tree::ShowSubtree::Session);
        if let Some(mut session) =
            super::oops::snapshot_subtrees(state, &[lumacore_tree::ShowSubtree::Session])
        {
            after.append(&mut session);
        }
    }
    state
        .replication_outbox
        .push(lumacore_tree::PendingReplicationOp {
            user: state.effective_user_name().to_string(),
            command: line.to_string(),
            affected_subtrees: after.iter().map(|s| s.subtree.clone()).collect(),
            post_snapshots: after,
            ..Default::default()
        });
}

/// The label of a step (decisions §29 "11c"): a command's own category, or —
/// for the generic ones (`Show`, `Settings`: Set, Delete, Store …) — what it
/// actually changed (the patch, a user, a profile, the station).
pub fn step_category(
    declared: OopsCategory,
    changes: &[Change],
    local: &[LocalSnapshot],
) -> OopsCategory {
    use ShowSubtree as S;
    // A programmer word that only moved the selection (`Fixture 3`, a group
    // recall) is a Selection step.
    if matches!(declared, OopsCategory::Programmer | OopsCategory::Selection) {
        let values = changes.iter().any(|c| matches!(c.subtree, S::Programmer(_)));
        let selection = changes.iter().any(|c| matches!(c.subtree, S::Selection(_)));
        match (values, selection) {
            (false, true) => return OopsCategory::Selection,
            (true, _) => return OopsCategory::Programmer,
            // A programmer word on something else (`Reset UserProfile …
            // Window`, decisions §30 "12c") is labelled by what it changed.
            _ if changes.iter().any(|c| matches!(c.subtree, S::Profile(_) | S::User(_))) => {}
            _ => return declared,
        }
    } else if !matches!(declared, OopsCategory::Show | OopsCategory::Settings) {
        return declared;
    }
    let of = |s: &ShowSubtree| match s {
        S::Devices | S::Protocols | S::AttributeDefs | S::FeatureGroups | S::ActivationGroups | S::AttributeRules => {
            Some(OopsCategory::Patch)
        }
        S::User(_) => Some(OopsCategory::Users),
        S::Profile(_) | S::ShowSettings => Some(OopsCategory::Settings),
        S::Programmer(_) => Some(OopsCategory::Programmer),
        S::Selection(_) => Some(OopsCategory::Selection),
        S::DataPool(_)
        | S::DatapoolPriorityOrders
        | S::Spatial3D
        | S::Playback
        | S::EffectEngines
        | S::Variables
        | S::Images
        | S::Assets
        | S::Tags => Some(OopsCategory::Show),
        S::Stations => Some(OopsCategory::Station),
        S::Playbacks | S::Session | S::Usage(_) => None,
    };
    // The most telling change wins: patch, then users, show, settings.
    let found: Vec<OopsCategory> = changes.iter().filter_map(|c| of(&c.subtree)).collect();
    for wanted in [
        OopsCategory::Patch,
        OopsCategory::Users,
        OopsCategory::Show,
        OopsCategory::Settings,
        OopsCategory::Station,
        OopsCategory::Programmer,
        OopsCategory::Selection,
    ] {
        if found.contains(&wanted) {
            return wanted;
        }
    }
    match local.first().map(|l| &l.what) {
        Some(LocalSubtree::Station) => OopsCategory::Station,
        Some(LocalSubtree::Watches(_)) => OopsCategory::UI,
        None => declared,
    }
}

/// Whether the acting user's profile files a step of `category` (
/// CreateOops / OopsViews / OopsProgrammer / OopsSelection).
fn files_step(state: &AppState, category: OopsCategory) -> bool {
    let Some(oops) = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| p.settings.preferences.oops)
    else {
        return true;
    };
    oops.create_oops
        && match category {
            OopsCategory::UI => oops.oops_views,
            OopsCategory::Programmer => oops.oops_programmer,
            OopsCategory::Selection => oops.oops_selection,
            _ => true,
        }
}

/// Whether `tokens` are a Delete the station asks about: not `Delete Show`
/// (it asks itself) or `Delete ViewButton` (clearing a button is harmless).
fn asked_delete(tokens: &[String]) -> bool {
    let is_delete = tokens
        .first()
        .is_some_and(|w| matches!(registry::resolve(w), Ok(registry::Word::Command(c)) if c.verb == "delete"));
    // Show and ViewButton never ask; a planned Delete only says so.
    let exempt = tokens.get(1).is_some_and(|w| {
        hooks().any(|h| matches!(h, Hook::DeleteNeverAsks(o) if o.matches(w)))
            || crate::objects::find(w).is_some_and(|o| o.planned.contains(&crate::objects::Verb::Delete))
    });
    is_delete && !exempt
}

/// What the object of a Delete statement says about its question (its
/// `ObjectDef::delete_ask`; the plain question when it has none).
fn delete_question(state: &AppState, tokens: &[String]) -> crate::objects::DeleteAsk {
    // Owner first (`Delete DataPool 2 Group 3`): the object asked about is the last in the path,
    // and its question gets the whole target (decisions §35 7).
    let last = tokens.iter().enumerate().skip(1).filter(|(_, w)| !w.starts_with('"')).filter_map(|(i, w)| crate::objects::find(w).map(|o| (i, o))).next_back();
    let hook = match last {
        Some((1, o)) => o.delete_ask.map(|ask| (ask, 2)),
        Some((_, o)) => o.delete_ask.map(|ask| (ask, 1)),
        None => None,
    };
    match hook {
        Some((ask, from)) => {
            let args: Vec<&str> = tokens[from..].iter().map(String::as_str).collect();
            ask(state, &args)
        }
        None => crate::objects::DeleteAsk::Plain,
    }
}

/// Whether dispatch asks for this Delete statement (and so takes the
/// answer's `/NoConfirmation` off before the handler): every Delete but the
/// exempt ones and those whose handler asks its own question.
fn dispatch_asks_delete(state: &AppState, tokens: &[String]) -> bool {
    asked_delete(tokens) && !matches!(delete_question(state, tokens), crate::objects::DeleteAsk::Handler)
}

/// The station's Delete question for one statement, or None (not a Delete,
/// exempt, its handler asks, or answered with `/NoConfirmation`). It is the
/// only Delete question (decisions §30 "Merge of bugfixes"): a row object's
/// names the count and the rows ("Delete 3 lines (Output 1, 4, 7)?") and
/// refuses the rows that can't go before asking; otherwise it shows how many
/// objects an open or long fixture range names.
fn delete_first(state: &AppState, line: &str) -> Option<DispatchResult> {
    use crate::objects::DeleteAsk;
    // Quoted words keep their quotes (issue 07): the answer names the rows
    // as typed.
    let tokens = super::tokens::args(&super::tokens::lex_lenient(line));
    if !asked_delete(&tokens) || tokens.iter().any(|w| crate::commands::flags::is(w, &crate::commands::flags::NO_CONFIRMATION)) {
        return None;
    }
    match delete_question(state, &tokens) {
        DeleteAsk::Handler => return None,
        DeleteAsk::Refuse(reply) => return Some(DispatchResult::Output(reply.into())),
        DeleteAsk::Ask(ask) => return Some(DispatchResult::Ask(ask)),
        DeleteAsk::Plain => {}
    }
    let count = (tokens.get(1).is_some_and(|w| crate::objects::FIXTURE.matches(w)))
        .then(|| {
            let rest: Vec<&str> = tokens[2..].iter().map(String::as_str).collect();
            crate::parser::number_list(&rest, &state.root.show_data.devices.dmx.patch.by_number())
        })
        .flatten()
        .filter(|ids| ids.len() > 1)
        .map(|ids| format!(" ({} fixtures)", ids.len()))
        .unwrap_or_default();
    Some(DispatchResult::Ask(Box::new(
        crate::dialogs::Ask::new("Delete", format!("{line}{count}?"))
            .button("Delete", lumacore_wire::ButtonKind::Danger, format!("{line} /NoConfirmation"))
            .cancel("Cancel")
            .flags("/NoConfirmation"),
    )))
}

/// `/NoOops` or `/NU` (any case) as a word of `line`: the line without it.
fn strip_no_oops(line: &str) -> Option<String> {
    let is_flag = |w: &str| crate::commands::flags::is(w, &crate::commands::flags::NO_OOPS);
    let tokens = super::tokens::lex(line).ok()?;
    if !super::tokens::words(&tokens).iter().any(|w| is_flag(w)) {
        return None;
    }
    // Drop the flag outside quotes.
    let mut out = String::with_capacity(line.len());
    let mut quoted = false;
    for piece in line.split_inclusive(|c: char| c.is_whitespace() || c == ';') {
        let word = piece.trim_end_matches(|c: char| c.is_whitespace() || c == ';');
        if !quoted && is_flag(word) {
            out.push_str(&piece[word.len()..]);
        } else {
            out.push_str(piece);
        }
        if piece.matches('"').count() % 2 == 1 {
            quoted = !quoted;
        }
    }
    Some(out.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// The station-local state a step can change: this station's settings and
/// the acting user's watches here.
fn local_images(state: &AppState) -> (Vec<u8>, String, Vec<u8>) {
    let station = serde_json::to_vec(&lumacore_luma::station_config::StationImage::of(
        &state.root.station_settings,
    ))
    .unwrap_or_default();
    let user = state.effective_user_name().to_string();
    let watches = serde_json::to_vec(state.watches()).unwrap_or_default();
    (station, user, watches)
}

fn local_changes(
    state: &AppState,
    before: (Vec<u8>, String, Vec<u8>),
    after: (Vec<u8>, String, Vec<u8>),
) -> Vec<LocalSnapshot> {
    let station_id = state.local_settings.station_id.clone();
    let mut out = Vec::new();
    if before.0 != after.0 {
        out.push(LocalSnapshot {
            what: LocalSubtree::Station,
            station_id: station_id.clone(),
            before: before.0,
            after: after.0,
        });
    }
    if before.1 == after.1 && before.2 != after.2 {
        out.push(LocalSnapshot {
            what: LocalSubtree::Watches(before.1),
            station_id,
            before: before.2,
            after: after.2,
        });
    }
    out
}

/// File a change made outside a command line (show writes the server does
/// itself, a member's changes on the master): the change path records it
/// like a command (`line` is its label).
pub fn record_change(
    state: &mut AppState,
    line: &str,
    undo: Undo,
    f: impl FnOnce(&mut AppState),
) {
    let pass = change::begin(state);
    f(state);
    let changes = finish_changes(state, pass);
    if !changes.is_empty() || !state.pending_local.is_empty() {
        commit(state, line, undo, &changes, JournalResult::Ok);
    }
    state.pending_local.clear();
}

/// Put back what a refused command changed.
fn refuse(state: &mut AppState, changes: &[Change]) {
    let before: Vec<SubtreeSnapshot> = changes
        .iter()
        .map(|c| SubtreeSnapshot {
            subtree: c.subtree.clone(),
            data: c.before.clone(),
        })
        .collect();
    if let Err(e) = super::oops::restore_subtrees(state, &before) {
        log::error!("restoring a refused command failed: {e}");
    }
}

/// Keep the Devices draft: the whole draft becomes one change (one undo
/// step, one replication op) and Patch is released.
pub fn keep_patch_draft(state: &mut AppState) -> String {
    keep_patch_draft_with(state, false, false)
}

/// Keep the draft; `standard` also keeps the new attributes where the
/// standard grouping put them (`Unclaim Root.ShowData.Devices /Keep /Default`, decisions §30
/// "12f-a") — in the same step.
pub fn keep_patch_draft_with(state: &mut AppState, standard: bool, mine: bool) -> String {
    // An MVR review stands between the draft and the rig (decisions §34 "Import MVR hardened" 3).
    if let Some(r) = state.patch_draft.as_ref().and_then(|d| d.review.as_ref()) {
        return crate::errors::render_code(format!("The MVR review of \"{}\" is open: Import Mvr /Apply or Import Mvr /Cancel first.", r.file), "E-CMD-010");
    }
    let Some(draft) = state.patch_draft.take() else {
        return crate::errors::render_code("No Patch draft is open.", "E-CMD-009");
    };
    let drafted = std::mem::replace(&mut state.root.show_data.devices, draft.live.clone());
    let pass = change::begin(state);
    state.root.show_data.devices = drafted.clone();
    state.root.show_data.after_load();
    for hook in hooks() {
        if let Hook::PatchKept(kept) = hook {
            kept(state, standard, mine);
        }
    }
    // Fixtures the draft deleted leave the programmers in this same step
    // (R1-38, Q9).
    state.clear_dangling_selections();
    let changes = finish_changes(state, pass);
    // A draft that changes another user's claimed fixture is refused like
    // any line (R1-44): nothing is kept, the draft stays open.
    if let Some((label, owner)) = state.claim_changed_by(&changes) {
        refuse(state, &changes);
        state.root.show_data.devices = drafted;
        state.root.show_data.after_load();
        state.patch_draft = Some(draft);
        return crate::errors::render_code(
            format!("{label} is claimed by {owner}: the draft would change it, nothing was kept. Claim {label} asks them for it."),
            "E-CLAIM-001",
        );
    }
    commit(
        state,
        "Keep Patch",
        super::registry::PATCH_MAIN,
        &changes,
        JournalResult::Ok,
    );
    let _ = state.unclaim("Devices");
    crate::text::render_success("Patch changes kept; Patch released.")
}

/// Add an entry to `Runtime.Journal` (`line` is already masked).
fn journal(state: &mut AppState, line: &str, changed: Vec<String>, result: JournalResult) {
    // A gesture files one entry at its end (decisions §30 "12f"), naming what moved.
    let line = match &state.gesture {
        Some(crate::gesture::GestureLine::Move { .. }) => return,
        Some(crate::gesture::GestureLine::End { summary, .. }) => {
            let summary = summary.clone();
            let who = state.effective_user_name().to_string();
            state.sysmon_pending.push((crate::gesture::SYSMON_CATEGORY.to_string(), format!("{summary} · {who}")));
            summary
        }
        None => line.to_string(),
    };
    let line = line.as_str();
    let entry = JournalEntry {
        time_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        epoch: 0,
        seq: 0,
        user: state.effective_user_name().to_string(),
        station: state.root.station_settings.station_name.clone(),
        command: line.to_string(),
        changed,
        result,
    };
    state.root.runtime.journal.record(entry);
}

fn dispatch_inner(state: &mut AppState, line: &str) -> DispatchResult {
    // Split on unquoted semicolons — each part is dispatched independently.
    let parsed = super::line::Line::parse(line).ok();
    if let Some(parsed) = parsed.as_ref().filter(|p| p.statements.len() > 1) {
        // All or nothing (decisions §29): every statement passes the rights
        // gate before any of them runs; one refused → nothing runs.
        if let Err(refused) = rights_of_all(state, parsed) {
            return DispatchResult::Output((format!("{refused} — nothing in `{line}` ran")).into());
        }
        let parts: Vec<String> = parsed.statements.iter().map(|s| s.text.clone()).collect();
        let mut outputs: Vec<crate::Reply> = Vec::new();
        for part in &parts {
            match dispatch_single(state, part) {
                DispatchResult::Output(s) => outputs.push(s),
                // Non-output results (Exit, ContextSwitch, etc.) propagate immediately
                other => return other,
            }
        }
        return DispatchResult::Output(crate::Reply::joined(outputs));
    }
    dispatch_single(state, line)
}

/// The global /Verbose and /Debug a statement carries (as typed, with the level), as the
/// parse attributes them: a word's own flag of the same name (`Tree /d=2` = depth) isn't one
/// (decisions §31).
fn verbosity_flags(statement: &super::line::Statement) -> Vec<(String, lumacore_tree::Verbosity)> {
    statement
        .flags
        .iter()
        .filter_map(|f| {
            if f.def.name == crate::commands::flags::VERBOSE.name {
                Some((f.typed.clone(), lumacore_tree::Verbosity::Verbose))
            } else if f.def.name == crate::commands::flags::DEBUG.name {
                Some((f.typed.clone(), lumacore_tree::Verbosity::Debug))
            } else {
                None
            }
        })
        .collect()
}

/// The rights gate for every statement of a line, all before any runs (decisions §29): each
/// checked with what the single path checks — its args as handlers get them (a quoted
/// `"/Force"` is a value, §15), without the verbosity flags `dispatch_single` strips.
fn rights_of_all(state: &AppState, line: &super::line::Line) -> Result<(), String> {
    for statement in &line.statements {
        let Some(word) = statement.words.first() else { continue };
        let verbosity = verbosity_flags(statement);
        let args: Vec<&str> = statement
            .args
            .iter()
            .map(String::as_str)
            .filter(|a| !verbosity.iter().any(|(typed, _)| typed == a))
            .collect();
        crate::rights::check_statement(state, word, &args)?;
    }
    Ok(())
}

/// Push a subtree-granular undo frame onto the current user's undo stack.
fn push_subtree_undo_frame(
    state: &mut AppState,
    description: String,
    snapshots: Vec<SubtreeSnapshot>,
    kind: OopsKind,
    local: Vec<LocalSnapshot>,
) {
    let user = state.effective_user_name().to_string();

    // Any undoable change makes the show unsaved.
    state.show_dirty = true;

    // Bump sequence counter
    state.undo_sequence += 1;
    let seq = state.undo_sequence;

    // Update subtree ownership
    for snap in &snapshots {
        state
            .subtree_owners
            .insert(snap.subtree.clone(), (user.clone(), seq));
    }

    // `Clear Programmer User …`: each cleared programmer's step goes on its
    // own user's history (decisions §10); the rest stays with the actor.
    let mut stacks: Vec<(String, String, Vec<SubtreeSnapshot>)> = Vec::new();
    for snap in snapshots {
        let owner = match &snap.subtree {
            lumacore_tree::ShowSubtree::Programmer(guid) | lumacore_tree::ShowSubtree::Selection(guid)
                if state.undo_to_owners =>
            {
                state
                .root
                .show_data
                .user_by_guid(*guid)
                .map(|u| u.login.clone())
                .filter(|name| !name.eq_ignore_ascii_case(&user))
            }
            _ => None,
        };
        let (who, what) = match owner {
            Some(name) => (name, format!("{description} (by {user})")),
            None => (user.clone(), description.clone()),
        };
        match stacks.iter_mut().find(|(w, _, _)| *w == who) {
            Some((_, _, list)) => list.push(snap),
            None => stacks.push((who, what, vec![snap])),
        }
    }
    state.undo_to_owners = false;
    // Which station the command came from, and when (§21 "6c").
    let station = state
        .command_station
        .clone()
        .unwrap_or_else(|| state.root.station_settings.station_name.clone());
    let time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64);
    // Only station state changed (Set Station, Watch): one step of the actor.
    if stacks.is_empty() {
        stacks.push((user.clone(), description.clone(), Vec::new()));
    }
    let mut local = Some(local);
    for (who, description, snapshots) in stacks {
        // Station state belongs to the actor's step.
        let local = if who == user { local.take().unwrap_or_default() } else { Vec::new() };
        let frame = UndoFrame {
            description,
            category: kind.category,
            user: who.clone(),
            sequence_id: seq,
            station: station.clone(),
            time_ms,
            main: kind.main,
            local,
            snapshots: Vec::new(),
            size_bytes: 0,
        }
        .with_snapshots(snapshots, "");
        state.undo_stack_mut(&who).push_frame(frame);
        state.note_undo_step(&who, crate::UndoKind::Push);
    }
}

/// Dispatch a single (non-semicolon) command line.
fn dispatch_single(state: &mut AppState, line: &str) -> DispatchResult {
    // A nested line (macro, answer) is written out here too (decisions §33 21).
    let written = write_out_bare(state, line);
    let line = written.as_deref().unwrap_or(line);
    // Tokenize: respect quoted strings; an unterminated quote is an error.
    // Quoted words keep their quotes: a value, never a keyword or an option.
    let mut tokens = match super::tokens::lex(line) {
        Ok(tokens) => super::tokens::args(&tokens),
        Err(e) => {
            return DispatchResult::Output(crate::Reply::error(e.render(line), "E-CMD-002"));
        }
    };
    if tokens.is_empty() {
        return DispatchResult::Output((String::new()).into());
    }
    // `Label … ""` names nothing: it asks for the name (decisions §30 "12b"), so the empty text
    // doesn't reach a Label handler (elsewhere `""` is a value: `Set User "x" "Password" ""`).
    if matches!(registry::resolve(&tokens[0]), Ok(registry::Word::Command(c)) if c.object_verb == Some(crate::objects::Verb::Label)) {
        tokens.retain(|t| t != "\"\"");
    }

    // An answered Delete's `/NoConfirmation` goes just before the handler
    // (`delete_first` asked, R1-61): Delete Show keeps its own.
    if dispatch_asks_delete(state, &tokens) {
        tokens.retain(|w| !crate::commands::flags::is(w, &crate::commands::flags::NO_CONFIRMATION));
    }

    // Intercept verbosity flags (/v, /d) — strip from tokens, set per-command override.
    // Reset to None first so flags from a previous command don't bleed through.
    state.current_verbosity = None;
    {
        let parsed = super::line::Line::parse(line).ok().and_then(|l| l.statements.into_iter().next());
        for (typed, level) in parsed.as_ref().map(verbosity_flags).unwrap_or_default() {
            state.current_verbosity = Some(level);
            if let Some(i) = tokens.iter().position(|t| *t == typed) {
                tokens.remove(i);
            }
        }
        if tokens.is_empty() {
            state.current_verbosity = None;
            return DispatchResult::Output((String::new()).into());
        }
    }

    // A family may take the line before its words run (`<line> /Watch`).
    for hook in hooks() {
        if let Hook::Intercept(intercept) = hook
            && let Some(result) = intercept(state, &tokens, line)
        {
            return result;
        }
    }

    // In Custom (edit mode) context: try dispatch_edit first.
    // If it handles the command, return that result.
    // If it returns None, fall through to normal dispatch.
    if matches!(state.context, ReplContext::Custom(_)) {
        let token_refs: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
        if let Some(result) = edit::dispatch_edit(state, &token_refs) {
            return result;
        }
    }

    let raw_word = tokens[0].as_str();
    let args: Vec<&str> = tokens[1..].iter().map(|s| s.as_str()).collect();

    // `word?` quick-hint shortcut — strip trailing `?` and return one-liner syntax.
    // Matches bare `at?`, `store?`, `fixture?`, etc. (no spaces, only trailing ?).
    if raw_word.ends_with('?') && raw_word.len() > 1 && args.is_empty() {
        let stripped = raw_word.trim_end_matches('?');
        if let Some(hint) = help::format_quick_hint(stripped) {
            return DispatchResult::Output(hint.into());
        }
        // Fall through to normal resolution (will produce "unknown command" error)
    }

    // The one rights gate, for every transport (decisions §19).
    if let Err(refused) = crate::rights::check_statement(state, raw_word, &args) {
        return DispatchResult::Output(refused.into());
    }

    match registry::resolve(raw_word) {
        Ok(Word::Object(obj)) => crate::objects::call(state, obj, &args),
        Ok(Word::Command(cmd)) if cmd.status == registry::Status::Planned => {
            DispatchResult::Output(crate::Reply::error(
                planned_message(cmd.label(), cmd.verb),
                "E-CMD-003",
            ))
        }
        Ok(Word::Command(cmd)) => {
            // Changing an object another user holds is refused (§21 "6d").
            if !matches!(cmd.undo, registry::Undo::ReadOnly)
                && let Some(refused) = crate::objects::held_elsewhere(state, &args)
            {
                return DispatchResult::Output(refused.into());
            }
            // `Set <target> "<Property>" <value>` on the one property registry, every line
            // (decisions §33 58; the target-less forms went in 3c3).
            // `Reset <target> ["<Property>"]` on the registry (decisions §33 61).
            if cmd.object_verb == Some(crate::objects::Verb::Reset)
                && let Some(result) = super::set::reset_route(state, &args)
            {
                return result;
            }
            if cmd.object_verb == Some(crate::objects::Verb::Set) {
                return super::set::route(state, &args);
            }
            let args: &[&str] = &args;
            // `Store Window 5 …` — the command as an object verb goes to the object.
            if let Some(verb) = cmd.object_verb
                && let Some(result) = crate::objects::dispatch_verb(state, verb, args)
            {
                return result;
            }
            (registry::run_of(cmd))(state, args)
        }
        Err(msg) => DispatchResult::Output(crate::Reply::error(msg, "E-CMD-001")),
    }
}

/// The one reply of a word that isn't built yet (decisions §29, E-CMD-003).
pub fn planned_message(label: &str, help_word: &str) -> String {
    format!("{label} isn't built yet (planned). See 'help {help_word}'.")
}

/// The change path's end for a line (`change::finish`), then what follows a change above the
/// state (decisions §34 15): a profile holds the views and view buttons, so the view bars
/// refresh.
fn finish_changes(state: &mut AppState, pass: change::Pass) -> Vec<change::Change> {
    let out = change::finish(state, pass);
    let profiles: Vec<uuid::Uuid> = out
        .iter()
        .filter_map(|c| match c.subtree {
            lumacore_tree::ShowSubtree::Profile(g) => Some(g),
            _ => None,
        })
        .collect();
    state.profiles_touched.extend(profiles);
    for hook in hooks() {
        if let Hook::Changed(changed) = hook {
            changed(state);
        }
    }
    out
}

#[cfg(test)]
mod tests;
