//! Show, dump, and DMX commands.

use crate::AppState;

/// Handle: Dump <object> [<id>]
///
/// `Dump <object> …` is routed to the object's own Dump handler by dispatch;
/// reaching this means no object was named.
pub fn handle_dump(_state: &AppState, args: &[&str]) -> String {
    let objects: Vec<&str> =
        crate::objects::supporting(crate::objects::Verb::Dump)
            .map(|o| o.keyword)
            .collect();
    match args.first() {
        Some(target) => crate::errors::render_code(
            format!(
                "Unknown dump target: '{target}'. Try: {}",
                objects.join(", ")
            ),
            "E-CMD-001",
        ),
        None => crate::errors::render_code(
            format!(
                "Usage: Dump <object> [<id>]. Dump takes: {}",
                objects.join(", ")
            ),
            "E-CMD-006",
        ),
    }
}

/// `title` and the value as pretty JSON.
fn dump_json<T: serde::Serialize>(title: &str, value: &T) -> String {
    match serde_json::to_string_pretty(value) {
        Ok(json) => format!(
            "{}\n{}",
            nu_ansi_term::Style::new().bold().paint(title),
            json
        ),
        Err(e) => crate::errors::render_code(
            format!("Serialization error: {e}"),
            "E-CMD-010",
        ),
    }
}

/// One object of a pool by the id in `args`: `Dump <label> <id>`.
fn dump_by_id<T: serde::Serialize>(
    label: &str,
    args: &[&str],
    get: impl FnOnce(u32) -> Option<T>,
) -> String {
    let Some(id) = args.first().and_then(|s| s.parse::<u32>().ok()) else {
        return crate::errors::render_code(
            format!("Usage: Dump {label} {{id}}"),
            "E-CMD-006",
        );
    };
    match get(id) {
        Some(value) => dump_json(&format!("{label} {id}"), &value),
        None => crate::errors::render_code(format!("{label} {id} not found."), "E-CMD-007"),
    }
}

pub fn dump_fixture(state: &AppState, args: &[&str]) -> String {
    dump_by_id("Fixture", args, |id| {
        state.root.show_data.devices.dmx.patch.fixture(id)
    })
}

pub fn dump_universe(state: &AppState, args: &[&str]) -> String {
    dump_by_id("Universe", args, |id| {
        state.root.show_data.devices.dmx.patch.universes.get(&id)
    })
}

pub fn dump_fixture_type(state: &AppState, args: &[&str]) -> String {
    dump_by_id("FixtureType", args, |id| {
        state.root.show_data.devices.dmx.patch.fixture_types.get(&id)
    })
}

pub fn dump_group(state: &AppState, args: &[&str]) -> String {
    let dp_id = state.selected_datapool();
    dump_by_id("Group", args, |id| {
        state
            .root
            .show_data
            .datapools
            .get(&dp_id)
            .and_then(|dp| dp.groups.get(&id))
    })
}

pub fn dump_showdata(state: &AppState, _args: &[&str]) -> String {
    dump_json("ShowData", &state.root.show_data)
}

pub fn dump_programmer(state: &AppState, _args: &[&str]) -> String {
    match state.current_programmer() {
        Some(p) => dump_json("Programmer", p),
        None => crate::errors::render_code(
            "No programmer found for current user.",
            "E-CMD-011",
        ),
    }
}

/// `Dump UserProfile [<id> | "name" | Thru]` — all profiles without an id.
pub fn dump_profiles(state: &AppState, args: &[&str]) -> String {
    use lumacore_tree::TreeNode;
    let profiles = &state.root.show_data.user_profiles;
    let first = args.first().map(|s| s.to_lowercase());
    if first.is_none() || first.as_deref() == Some("thru") {
        let parts: Vec<String> = profiles
            .iter()
            .map(|(id, p)| {
                let label = p.name().unwrap_or("<unnamed>");
                dump_json(&format!("UserProfile {id} '{label}'"), p)
            })
            .collect();
        return if parts.is_empty() {
            crate::errors::render_code("No user profiles found.", "E-CMD-011")
        } else {
            parts.join("\n\n")
        };
    }
    let wanted = first.unwrap_or_default();
    let profile = match wanted.parse::<u32>() {
        Ok(id) => profiles.get(&id),
        Err(_) => profiles
            .values()
            .find(|p| p.name().is_some_and(|n| n.eq_ignore_ascii_case(&wanted))),
    };
    match profile {
        Some(p) => dump_json(
            &format!("UserProfile '{}'", p.name().unwrap_or("<unnamed>")),
            p,
        ),
        None => crate::errors::render_code(
            format!(
                "UserProfile '{}' not found. {} profiles exist.",
                args.first().unwrap_or(&""),
                profiles.len()
            ),
            "E-CMD-011",
        ),
    }
}
