//! Deleting fixtures: `Delete Fixture …` and the context form `Delete 5` in Fixture context.
//! Fixtures are made with `Store Fixture` and addressed with `Assign … At DMX` (decisions §33 48,
//! §34 round 3 19; `objects::patch::fixture`).

use lumacore_patch::PatchEngine;

use crate::AppState;
use crate::errors::render_code;
use crate::text::render_success;

/// `Delete 5` in Fixture context (the context form of `Delete Fixture`).
pub fn delete_fixtures(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        return render_code("Usage: Delete Fixture <n> [Thru <m>]", "E-CMD-006");
    }
    // The one range rule (R1-35, Q6): every fixture of `1 Thru 5` that
    // exists, not only the first.
    let Some(ids) = crate::parser::number_list(args, &state.root.show_data.devices.dmx.patch.by_number()) else {
        return render_code(format!("Invalid fixture ID '{}'. Expected a number.", args.join(" ")), "E-CMD-008");
    };
    if ids.is_empty() {
        return render_code(format!("Fixture {} not found.", args.join(" ")), "E-PATCH-006");
    }
    unpatch_ids(state, &ids)
}

/// Delete these fixtures, one reply line each.
pub fn unpatch_ids(state: &mut AppState, ids: &[u32]) -> String {
    ids.iter().map(|n| unpatch_one(state, *n)).collect::<Vec<_>>().join("\n")
}

fn unpatch_one(state: &mut AppState, fixture_id: u32) -> String {
    // Get fixture name before unpatching (for the success message)
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(fixture_id)
        .and_then(|f| f.meta.name().map(|n| n.to_string()))
        .unwrap_or_else(|| format!("Fixture {fixture_id}"));

    match PatchEngine::unpatch_fixture(&mut state.root.show_data, fixture_id) {
        Ok(()) => render_success(&format!(
            "Deleted Fixture {fixture_id} '{name}'."
        )),
        Err(lumacore_patch::PatchError::FixtureNotFound { fixture_id }) => {
            render_code(format!("Fixture {fixture_id} not found."), "E-PATCH-006")
        }
        Err(e) => {
            let code = lumacore_tree::Coded::code(&e);
            render_code(format!("Delete failed: {e:?}"), code)
        }
    }
}
