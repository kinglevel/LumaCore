//! Command handlers that aren't owned by a feature module.
//!
//! Each `CommandDef` in the registry names its handler (`run`); the ones that
//! need more than a call into another module live here. Dispatch never matches
//! on verb strings — it looks the word up and calls `run`.

use crate::DispatchResult;
use crate::{AppState, ReplContext};

use super::claim::parse_claim_targets;
use super::{patch, universe};
use crate::claims::Claims;
use crate::navigation::Navigation;

/// Wrap a handler's text output.
pub fn out(text: String) -> DispatchResult {
    DispatchResult::Output(text.into())
}

/// An error reply with its code. Text that already carries one (an error
/// passed on from claims or programmers) keeps it.
fn error(text: &str, code: &str) -> DispatchResult {
    DispatchResult::Output(crate::errors::reply_error(text, code))
}

fn coded(text: &str, code: &str) -> String {
    crate::errors::reply(text, code)
}

fn success(text: &str) -> DispatchResult {
    DispatchResult::Output(crate::Reply::ok(crate::text::render_success(text)))
}

/// `Store` / `Delete` / … without an object it knows: list the objects that
/// take the verb (from the object registry).
pub fn objects_usage(verb: crate::objects::Verb) -> DispatchResult {
    let names: Vec<&str> = crate::objects::supporting(verb)
        .map(|o| o.keyword)
        .collect();
    error(
        &format!(
            "{label} what? {label} takes: {}",
            names.join(", "),
            label = verb.label()
        ),
        "E-CMD-006",
    )
}

/// Bare `Store` and `Store <n>`: Cue is Store's default object (decisions §30 "Store
/// Cue"). Any other word is still "Store what?".
pub(super) fn store_default(state: &mut AppState, args: &[&str]) -> DispatchResult {
    match args.iter().find(|a| !a.starts_with('/')) {
        Some(w) if !w.starts_with('"') && super::playback::cue_ref(w).is_none() => objects_usage(crate::objects::Verb::Store),
        _ => DispatchResult::Output(super::playback::store_cue_line(state, args).into()),
    }
}

/// `Pause` on anything but an object that pauses (`Pause LumaSession`):
/// pausing playbacks isn't built yet (decisions §29).
pub(super) fn pause(_state: &mut AppState, _args: &[&str]) -> DispatchResult {
    error(&super::planned_message("Pause", "pause"), "E-CMD-003")
}

/// Leave the current shell session and return to the client connector.
fn leave(state: &mut AppState, disconnect: bool) -> DispatchResult {
    // A login that waited for the Meow License ends with its dialog ("Not me", decisions 0290).
    if let Some(holder) = crate::consent::holder(state) {
        crate::consent::forget(state, &holder);
    }
    // A UI client forgets its login, so it doesn't reconnect as this user.
    if state.has_ui() {
        state
            .ui_requests
            .push(lumacore_wire::UiEvent::LoggedOut { disconnect });
    }
    DispatchResult::Logout
}

pub(super) fn logout(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    leave(state, false)
}

pub fn disconnect(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    leave(state, true)
}

/// Claim — bare: claim current scoped object; with args: claim explicit
/// object(s). Another user's object asks them first (10 s); `/Force`
/// (Admin) takes it; `/Keep` refuses someone asking for yours (§21 "6d").
/// `Unclaim Root.ShowData.Devices /Discard` (/DiscardChanges): drop the draft (decisions §31).
pub(super) const DISCARD: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("Discard", "Drop the changes");
pub(super) const DISCARD_CHANGES: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("DiscardChanges", "Drop the changes");
/// `Unclaim Root.ShowData.Devices /Keep /Default|/Edit [/Mine Default|Edit]`: the answer to the first dialog
/// for new attributes (decisions §30 "12f-a").
pub(super) const PATCH_DEFAULT: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("Default", "New attributes follow the standard grouping");
pub(super) const PATCH_EDIT: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("Edit", "Open the attribute boards for the new attributes");
pub(super) const PATCH_MINE: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("Mine", "Your own encoder bar's answer apart").value("Default|Edit");

pub(super) fn claim(state: &mut AppState, args: &[&str]) -> DispatchResult {
    use crate::commands::flags as F;
    let (force, keep, remember) = (F::has(args, &F::FORCE), F::has(args, &F::KEEP), F::has(args, &F::REMEMBER));
    let args = F::positional(args, &[]);
    if args.is_empty() {
        let Some(obj) = state.claimable_at_path() else {
            return error(
                "No claimable object in the current scope. List Claims shows the claims.",
                "E-CMD-009",
            );
        };
        return match state.ask_for_claim(&obj, force) {
            Ok(msg) => success(&msg),
            Err(e) => error(&e, "E-CMD-007"),
        };
    }

    let mut done: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for obj in parse_claim_targets(&args) {
        if keep {
            if remember {
                state.remember_claim_answer(&obj, false);
            }
            match state.keep_claim(&obj) {
                Ok(msg) => done.push(msg),
                Err(e) => errors.push(e),
            }
            continue;
        }
        // A fixture another user's programmer holds live can't be claimed.
        if let Some(id) = obj
            .strip_prefix("Fixture ")
            .and_then(|n| n.parse::<u32>().ok())
            && let Some((holder, attr)) = crate::programmers::live_holder(state, id)
        {
            errors.push(crate::errors::with_code(
                format!("Fixture {id} {attr} is held by {holder}."),
                "E-PROG-002",
            ));
            continue;
        }
        match state.ask_for_claim(&obj, force) {
            Ok(msg) => done.push(msg),
            Err(e) => errors.push(e),
        }
    }

    if errors.is_empty() {
        success(&done.join("\n"))
    } else if done.is_empty() {
        error(&errors.join("\n"), "E-CMD-007")
    } else {
        out(format!(
            "{}\n{}",
            crate::text::render_success(&done.join("\n")),
            coded(&errors.join("\n"), "E-CMD-007")
        ))
    }
}

/// Unclaim — release a claimed object (Admins can unclaim anyone).
pub(super) fn unclaim(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let args_full = args;
    // `/Keep` / `/Discard` answer "Leave the draft" without asking
    // (scripts, one-shot remote commands; /DiscardChanges).
    use crate::commands::flags as F;
    let remember = F::has(args, &F::REMEMBER);
    let words = F::positional(args, &[&PATCH_MINE]);
    let discard = F::has(args, &DISCARD) || F::has(args, &DISCARD_CHANGES);
    let keep = F::has(args, &F::KEEP);
    let args = words.as_slice();
    let patch = args.is_empty() || crate::claims::area_of(args[0]) == Some("Devices");
    // Leaving the patch ends what is left of your Identify (§34 round 3: the dialog sent its Off
    // lines first; this is the station's check). A leave that still asks keeps it until answered.
    if patch && state.is_claimed_by_me("Devices") && (keep || discard || !state.patch_draft_changed()) {
        let me = state.effective_user_name().to_string();
        crate::objects::identify::end_left_on(state, &me, "Unclaim");
    }
    if patch && state.is_claimed_by_me("Devices") && state.patch_draft_changed() {
        return if keep {
            // The draft brings attributes the show has never seen: the first
            // dialog (decisions §30 "12f-a"), unless the line answers.
            // `/Default` / `/Edit` answer for the show and the patcher's own
            // encoder bar; `/Mine Default|Edit` answers the bar apart.
            let new = new_attributes(state);
            let mine_word = F::value(args_full, &PATCH_MINE);
            let me = state.effective_user_name().to_string();
            let (default, edit) = (F::has(args_full, &PATCH_DEFAULT), F::has(args_full, &PATCH_EDIT));
            let result = if default || edit {
                let show_std = default;
                let mine_std = mine_word.as_deref().map_or(show_std, |w| w.eq_ignore_ascii_case("default"));
                let kept = super::dispatch::keep_patch_draft_with(state, show_std, mine_std);
                let mut lines = vec![kept];
                if !new.is_empty() {
                    lines.push(if show_std {
                        format!("{} new attributes follow the standard attribute grouping.", new.len())
                    } else {
                        format!("{} new attributes are placed by the standard grouping and marked new: Edit Attribute /New (or List Attribute /New).", new.len())
                    });
                    if !mine_std {
                        lines.push("Your encoder bar: Edit UserProfile Attribute /New (or List UserProfile Attribute /New).".into());
                    }
                    // `/Edit`: the boards open (the show's first, then yours).
                    use crate::commands::attributes::{open_board, Target};
                    if !show_std {
                        open_board(state, Target::Show, &["/New"]);
                    }
                    if !mine_std && let Some(no) = state.root.show_data.profile_id_for_user(&me) {
                        open_board(state, Target::Profile(no), &["/New"]);
                    }
                }
                out(lines.join("\n"))
            } else if !new.is_empty() {
                return crate::commands::attributes::patcher_prompt(&new);
            } else {
                out(super::dispatch::keep_patch_draft(state))
            };
            // Everyone else learns of the new attributes: their dialog.
            crate::commands::attributes::ask_everyone_waiting(state, &me);
            result
        } else if discard {
            state.discard_patch_draft();
            let _ = state.unclaim("Devices");
            success("Devices draft discarded; Devices released.")
        } else {
            patch_leave_prompt(state)
        };
    }
    if args.is_empty() {
        let released = state.unclaim_all_mine();
        let message = match released {
            0 => "No releasable claims held.".to_string(),
            1 => "Released 1 claim.".to_string(),
            n => format!("Released {n} claims."),
        };
        return success(&message);
    }
    let mut done = Vec::new();
    for obj in parse_claim_targets(args) {
        if remember {
            state.remember_claim_answer(&obj, true);
        }
        match state.unclaim(&obj) {
            Ok(()) => done.push(format!("{obj} released.")),
            Err(e) => return DispatchResult::Output((crate::errors::reply(&e, "E-CMD-010")).into()),
        }
    }
    success(&done.join("\n"))
}

/// Attributes the patch draft's fixtures use that the show doesn't know.
fn new_attributes(state: &AppState) -> Vec<String> {
    let show = &state.root.show_data;
    lumacore_tree::attribute_resolver::used_attributes(show)
        .into_keys()
        .filter(|n| !show.attribute_defs.contains_key(n))
        .collect()
}

/// "Leaving the patch" — ours "Leave the draft" (decisions §34 "34g4 patch entry hardened",
/// the locked `patch-lock.html`): the draft has changes. No countdown — a Devices draft is never
/// taken by a timer (only a holder lost from the session drops it, `release_lost_devices`). It names the lines Keep would leave on slots without a
/// universe (decisions §11 "Issue 11 hardening" 5).
pub fn patch_leave_prompt(state: &AppState) -> DispatchResult {
    let mut text = "You changed the patch in your draft. Save puts the changes live as one undo step; Discard keeps the live patch as it is.".to_string();
    let universes = &state.root.show_data.devices.dmx.patch.universes;
    let empty: Vec<String> = state
        .root
        .show_data
        .protocols
        .all_lines()
        .filter(|(_, _, l)| (l.local_universe..l.local_universe + l.amount).any(|u| !universes.contains_key(&u)))
        .map(|(p, n, _)| format!("{p} {n}"))
        .collect();
    if !empty.is_empty() {
        text.push_str(&format!("\nKeep leaves lines on slots without a universe (nothing sent or taken there): {}.", empty.join(", ")));
    }
    DispatchResult::Ask(Box::new(
        crate::dialogs::Ask::new("Leave the draft", text)
        .button("Save", lumacore_wire::ButtonKind::Primary, "Unclaim Root.ShowData.Devices /Keep")
        .button("Discard", lumacore_wire::ButtonKind::Danger, "Unclaim Root.ShowData.Devices /Discard")
        .cancel("Cancel")
        .flags("/Keep or /Discard"),
    ))
}

/// Patch — bare: navigate to Patch via enter; with args: patch fixtures (requires claim).
/// Standard error for commands that need the patch claim.
fn require_patch_claim() -> DispatchResult {
    error(
        "Devices are not claimed: Claim Root.ShowData.Devices (or 'enter patch') takes the draft.",
        "E-CMD-010",
    )
}

/// Delete <object> … is routed by the object registry; this is the context
/// form (`Delete 5` in Fixture / Universe context).
pub(super) fn delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    if !state.is_claimed_by_me("Devices") {
        return require_patch_claim();
    }
    if args.is_empty() {
        return objects_usage(crate::objects::Verb::Delete);
    }
    match &state.context {
        ReplContext::Universe => out(universe::handle_delete_universe(state, args)),
        _ => out(patch::delete_fixtures(state, args)),
    }
}

/// Add / Remove only exist inside edit mode (handled by `edit::dispatch_edit`).
pub(super) fn edit_only(_state: &mut AppState, _args: &[&str]) -> DispatchResult {
    error(
        "Only available while editing an object. Use 'Edit <object>' first.",
        "E-CMD-013",
    )
}

/// Clear the terminal output buffer.
/// `Clear` wipes the terminal; `Clear Programmer User 2 Thru 5` clears
/// other users' programmers (Admin, decisions §10).
pub(super) fn clear_terminal(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let names_programmer = args.first().is_some_and(|a| {
        crate::objects::find(a)
            .is_some_and(|o| std::ptr::eq(o, &crate::objects::PROGRAMMER))
    });
    if names_programmer {
        let rest = &args[1..];
        let rest = match rest.first() {
            Some(w)
                if crate::objects::find(w)
                    .is_some_and(|o| std::ptr::eq(o, &crate::commands::users::USER)) =>
            {
                &rest[1..]
            }
            _ => rest,
        };
        return out(crate::programmers::clear_users(state, rest));
    }
    out("\x1B[2J\x1B[1;1H".to_string())
}
