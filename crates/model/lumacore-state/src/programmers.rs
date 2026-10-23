//! Programmers between users (decisions §4, step 4 hardening).
//!
//! - Programmers are LTP, like sequences: any user may set any attribute and
//!   the latest touch wins the output. Touching an attribute **moves** it:
//!   the value leaves every other programmer.
//! - Selecting claims nothing; two users can select the same fixture.
//! - Values set in Blind don't touch the stage and take nothing; leaving
//!   Blind is a fresh touch of every value (`take_over_all`).
//! - `Claim Fixture <n>` locks a whole fixture: nobody else may select it or
//!   set values for it. A claim is refused while another user's programmer
//!   holds a live value for the fixture.

use lumacore_tree::Value;
use uuid::Uuid;

use crate::errors::render_code;
use crate::AppState;

/// Who (other than the acting user) has claimed this fixture.
fn claimed_by_other(state: &AppState, fixture_id: u32) -> Option<String> {
    let me = state.effective_user_name();
    let guid = state.root.show_data.fixture_guid(fixture_id)?;
    state
        .root
        .runtime
        .claims
        .get(&guid)
        .map(|c| c.user.clone())
        .filter(|owner| !owner.eq_ignore_ascii_case(me))
}

/// Refuse fixtures another user has claimed (selection and values).
pub fn ensure_unclaimed(state: &AppState, fixture_ids: &[u32]) -> Result<(), String> {
    let taken: Vec<String> = fixture_ids
        .iter()
        .filter_map(|&id| {
            claimed_by_other(state, id).map(|owner| format!("Fixture {id} ({owner})"))
        })
        .collect();
    if taken.is_empty() {
        Ok(())
    } else {
        Err(render_code(
            format!("Claimed by another user: {}.", taken.join(", ")),
            "E-PROG-001",
        ))
    }
}

/// Same, by fixture GUID.
pub fn ensure_unclaimed_guids(state: &AppState, guids: &[Uuid]) -> Result<(), String> {
    let ids: Vec<u32> = guids
        .iter()
        .filter_map(|g| state.root.show_data.fixture_id_for_guid(*g))
        .collect();
    ensure_unclaimed(state, &ids)
}

/// Whether the acting user is in Blind.
pub fn in_blind(state: &AppState) -> bool {
    state
        .root
        .runtime
        .user_ref(state.effective_user_name())
        .is_some_and(|u| u.modes.blind)
}

/// Users whose programmers reach the stage: everyone not in Blind.
fn live_users(state: &AppState) -> Vec<u32> {
    let show = &state.root.show_data;
    show.users
        .iter()
        .filter(|(_, u)| {
            !state
                .root
                .runtime
                .user_ref(&u.login)
                .is_some_and(|r| r.modes.blind)
        })
        .map(|(no, _)| *no)
        .collect()
}

fn my_user(state: &AppState) -> Option<u32> {
    state.root.show_data.user_no(state.effective_user_name())
}

/// Set values in the acting user's programmer (latest touch). Unless the
/// user is in Blind, the attributes leave every other programmer. Claimed
/// fixtures are refused (nothing is set). Returns how many values were set.
pub fn program(state: &mut AppState, values: Vec<(Uuid, String, Value)>) -> Result<usize, String> {
    let values = only_attributes_they_have(state, values)?;
    let guids: Vec<Uuid> = values.iter().map(|(g, _, _)| *g).collect();
    ensure_unclaimed_guids(state, &guids)?;
    let Some(me) = my_user(state) else {
        return Err(crate::errors::render_code(
            "No programmer available.",
            "E-CMD-011",
        ));
    };
    let blind = in_blind(state);
    let users = &mut state.root.show_data.users;
    if !blind {
        for (no, user) in users.iter_mut() {
            if *no != me {
                for (g, attr, _) in &values {
                    user.programmer.values.remove(&(*g, attr.clone()));
                }
            }
        }
    }
    let prog = &mut users
        .get_mut(&me)
        .expect("user looked up above")
        .programmer;
    let count = values.len();
    for (g, attr, value) in values {
        prog.set_value(g, attr, value);
    }
    Ok(count)
}

/// The attribute names of a fixture's mode, or None when its type or mode
/// can't be found (a broken link: then nothing is checked).
fn mode_attributes<'a>(state: &'a AppState, fixture: &lumacore_fixtures::Fixture) -> Option<Vec<&'a str>> {
    let patch = &state.root.show_data.devices.dmx.patch;
    let ty = patch.fixture_types.values().find(|t| t.meta.identity.guid == fixture.mvr.fixture_type)?;
    let (_, mode) = ty.mode_of(fixture.mvr.revision, fixture.mvr.mode)?;
    Some(mode.channel_functions.iter().map(|cf| cf.attribute_name.as_str()).collect())
}

/// `attribute` as the fixture's mode spells it (`pan` → `Pan`); as typed when
/// the fixture doesn't have it or its mode can't be found.
pub fn attribute_spelling(state: &AppState, fixture: Uuid, attribute: &str) -> String {
    state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixtures
        .values()
        .find(|f| f.meta.identity.guid == fixture)
        .and_then(|f| mode_attributes(state, f))
        .and_then(|names| names.into_iter().find(|n| n.eq_ignore_ascii_case(attribute)).map(str::to_string))
        .unwrap_or_else(|| attribute.to_string())
}

/// Keep the values for attributes the fixture has, under the mode's own
/// spelling (`pan` → `Pan`); a fixture without the attribute is skipped
/// (R1-36: its value was stored and never output). None left: refused.
fn only_attributes_they_have(
    state: &AppState,
    values: Vec<(Uuid, String, Value)>,
) -> Result<Vec<(Uuid, String, Value)>, String> {
    let asked = values.len();
    let mut missing: Option<String> = None;
    // Looked up once per fixture, not per value.
    let fixtures: std::collections::HashMap<Uuid, &lumacore_fixtures::Fixture> =
        state.root.show_data.devices.dmx.patch.fixtures.values().map(|f| (f.meta.identity.guid, f)).collect();
    let mut modes: std::collections::HashMap<Uuid, Option<Vec<&str>>> = std::collections::HashMap::new();
    let kept: Vec<(Uuid, String, Value)> = values
        .into_iter()
        .filter_map(|(g, attr, v)| {
            // A fixture that isn't in the patch any more gets no value (it
            // would never output, and Store Cue would copy it).
            fixtures.contains_key(&g).then_some(())?;
            Some((g, attr, v))
        })
        .filter_map(|(g, attr, v)| match modes
            .entry(g)
            .or_insert_with(|| fixtures.get(&g).and_then(|f| mode_attributes(state, f)))
        {
            // Its type or mode can't be found (a broken link): not checked.
            None => Some((g, attr, v)),
            Some(names) => match names.iter().find(|n| n.eq_ignore_ascii_case(&attr)) {
                Some(name) => Some((g, (*name).to_string(), v)),
                None => {
                    missing.get_or_insert(attr);
                    None
                }
            },
        })
        .collect();
    match missing {
        Some(attr) if kept.is_empty() && asked > 0 => Err(crate::errors::render_code(
            format!("No fixture here has {attr}."),
            "E-CMD-009",
        )),
        _ => Ok(kept),
    }
}

/// The acting user's values for `fixtures` (all when `None`) become the
/// latest touch and leave every other programmer (knock in, leaving Blind).
/// Fixtures another user has claimed are skipped (R1-43): their values stay
/// with the claimer.
pub fn take_over(state: &mut AppState, fixtures: Option<&[Uuid]>) {
    let Some(me) = my_user(state) else {
        return;
    };
    let me_name = state.effective_user_name().to_string();
    let claims = &state.root.runtime.claims;
    let claimed_by_other =
        |fixture: &Uuid| claims.get(fixture).is_some_and(|c| !c.user.eq_ignore_ascii_case(&me_name));
    let users = &mut state.root.show_data.users;
    let Some(mine) = users.get_mut(&me) else {
        return;
    };
    let mut taken = Vec::new();
    for (key, pv) in mine.programmer.values.iter_mut() {
        if fixtures.is_none_or(|f| f.contains(&key.0)) && !claimed_by_other(&key.0) {
            pv.stamp = lumacore_tree::next_stamp();
            taken.push(key.clone());
        }
    }
    for (no, user) in users.iter_mut() {
        if *no != me {
            for key in &taken {
                user.programmer.values.remove(key);
            }
        }
    }
}

/// Another user whose programmer holds a live value for this fixture.
pub fn live_holder(state: &AppState, fixture_id: u32) -> Option<(String, String)> {
    let guid = state.root.show_data.fixture_guid(fixture_id)?;
    let me = my_user(state);
    let live = live_users(state);
    state
        .root
        .show_data
        .users
        .iter()
        .filter(|(no, _)| Some(**no) != me && live.contains(no))
        .find_map(|(_, u)| {
            let attr = u
                .programmer
                .values
                .keys()
                .find(|(g, _)| *g == guid)
                .map(|(_, a)| a.clone())?;
            Some((u.login.clone(), attr))
        })
}

/// `Clear Programmer User 2 Thru 5` — clear other users' programmers
/// (Setup, declared by `rights::clear_rights`). The undo step goes on each **affected** user's history, so they
/// can take their work back with Oops (decisions §10).
pub fn clear_users(state: &mut AppState, args: &[&str]) -> String {
    let numbers = match parse_user_numbers(state, args) {
        Some(n) if !n.is_empty() => n,
        _ => {
            return crate::errors::render_code(
                "Usage: Clear Programmer User <n> [Thru <m>]",
                "E-CMD-006",
            );
        }
    };
    let mut cleared = Vec::new();
    for no in numbers {
        let by = state.effective_user_name().to_string();
        let Some(user) = state.root.show_data.users.get_mut(&no) else {
            continue;
        };
        user.programmer.full_clear();
        cleared.push(user.login.clone());
        let note = format!("Your programmer was cleared by {by} (Oops undoes it).");
        lumacore_sysmon::sysmon_note("Programmer", format!("{}: {note}", user.login));
    }
    state.undo_to_owners = true;
    if cleared.is_empty() {
        return crate::errors::render_code("No such user.", "E-CMD-011");
    }
    crate::text::render_success(&format!(
        "Cleared the programmer of {}",
        cleared.join(", ")
    ))
}

/// `2`, `2 Thru 5`, `2 + 4` (user numbers; the one range rule,
/// `parser::number_list`: users that don't exist are skipped).
fn parse_user_numbers(state: &AppState, args: &[&str]) -> Option<Vec<u32>> {
    crate::parser::number_list(args, &state.root.show_data.users)
}
