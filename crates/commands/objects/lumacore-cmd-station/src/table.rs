// ████████╗ █████╗ ██████╗ ██╗     ███████╗
// ╚══██╔══╝██╔══██╗██╔══██╗██║     ██╔════╝
//    ██║   ███████║██████╔╝██║     █████╗
//    ██║   ██╔══██║██╔══██╗██║     ██╔══╝
//    ██║   ██║  ██║██████╔╝███████╗███████╗
//    ╚═╝   ╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝
// FILE: table

//! The station table's rows (decisions §30 "12d part 4"): made before the station exists, forgotten
//! with Delete (this station, or one online in the LumaSession, is refused first).
//!
//! Owns:          Store / Delete Station: a row made in advance, forgetting a station and its row.
//! Doesn't own:   the table itself (lumacore-state stations), the delete ask (the kernel's set_rows).
//! Entry points:  cmd_store_station, cmd_delete_station, station_not_deletable.
//! Decisions:     §30 "12d part 4".

// ███████╗████████╗ █████╗ ████████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██║██╔═══██╗████╗  ██║
// ███████╗   ██║   ███████║   ██║   ██║██║   ██║██╔██╗ ██║
// ╚════██║   ██║   ██╔══██║   ██║   ██║██║   ██║██║╚██╗██║
// ███████║   ██║   ██║  ██║   ██║   ██║╚██████╔╝██║ ╚████║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: station

//! The Station verbs (decisions §30 "12d part 4"): Store / Delete Station. Claim and Pause
//! LumaSession are `lumacore-cmd-lumasession`'s.
//!
//! Owns:          cmd_store_station, cmd_delete_station, station_not_deletable.
//! Doesn't own:   the outbox and the session's state (`lumacore_objects::session_link`), the station table (`lumacore_objects::stations`).
//! Entry points:  the handlers the STATION ObjectDef names.
//! Decisions:     §21 "6c", §30 "12d", §34 15.

use lumacore_objects::AppState;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

// *************************************************************************************************
// Station
// Rows of the station table.
// *************************************************************************************************

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Store Station "Tower"`: a row of the station table made before the
/// station exists (decisions §30 "12d part 4": pre-configure the rig).
pub fn cmd_store_station(state: &mut AppState, args: &[&str]) -> String {
    use lumacore_objects::text::render_success;
    let name = args
        .iter()
        .copied()
        .filter(|a| !a.starts_with('/'))
        .collect::<Vec<_>>()
        .join(" ");
    match lumacore_objects::stations::store_row(state, &name) {
        Ok(n) => render_success(&format!(
            "Station table row {n} \"{}\": wire it with Set Station {} Network … \"Interface\" … (it applies when the station joins)",
            name.trim_matches('"'),
            lumacore_objects::stations::word(name.trim_matches('"'))
        )),
        Err(e) => e,
    }
}

/// A station that can't be forgotten now (refused before any ask): one
/// that is neither known nor in the table, or still in the LumaSession.
pub fn station_not_deletable(state: &AppState, word: &str) -> Option<String> {
    use lumacore_objects::errors::render_code;
    let w = word.trim_matches('"');
    let row = lumacore_objects::stations::row_named(state, w);
    let name = row.map_or(w.to_string(), |n| {
        state.root.show_data.stations[&n].meta.name().unwrap_or_default().to_string()
    });
    let known = state
        .root
        .show_data
        .session
        .stations
        .values()
        .any(|s| s.meta.name().unwrap_or_default().eq_ignore_ascii_case(&name));
    if !known && row.is_none() {
        return Some(render_code(
            format!("{w} is neither a known station nor in the station table (List Station)."),
            "E-CMD-007",
        ));
    }
    state
        .root
        .runtime
        .luma_sessions
        .members
        .get(&name)
        .is_some_and(|m| m.connected)
        .then(|| {
            render_code(
                format!("{name} is in the LumaSession: Dismiss Station \"{name}\" first."),
                "E-CMD-010",
            )
        })
}

/// `Delete Station "FOH-2"`: forget a station — its known-stations entry
/// (no more re-invites) and its row of the station table.
pub fn cmd_delete_station(state: &mut AppState, args: &[&str]) -> String {
    use lumacore_objects::errors::render_code;
    use lumacore_objects::text::render_success;
    let Some(word) = args.first().map(|a| a.trim_matches('"').to_string()) else {
        return render_code("Usage: Delete Station \"name\"", "E-CMD-006");
    };
    let row = lumacore_objects::stations::row_named(state, &word);
    let name = row.map_or(word.clone(), |n| {
        state.root.show_data.stations[&n].meta.name().unwrap_or_default().to_string()
    });
    let stations = &state.root.show_data.session.stations;
    let known = stations
        .iter()
        .find(|(_, s)| s.meta.name().unwrap_or_default().eq_ignore_ascii_case(&name))
        .map(|(id, _)| id.clone());
    if known.is_none() && row.is_none() {
        return render_code(
            format!("{word} is neither a known station nor in the station table (List Station)."),
            "E-CMD-007",
        );
    }
    if state
        .root
        .runtime
        .luma_sessions
        .members
        .get(&name)
        .is_some_and(|m| m.connected)
    {
        return render_code(
            format!("{name} is in the LumaSession: Dismiss Station \"{name}\" first."),
            "E-CMD-010",
        );
    }
    let mut gone = Vec::new();
    if let Some(id) = known {
        state.root.show_data.session.stations.remove(&id);
        gone.push("the known stations (no more re-invites)");
    }
    if let Some(n) = row {
        state.root.show_data.stations.remove(&n);
        gone.push("the station table");
    }
    render_success(&format!("{name} removed from {}.", gone.join(" and ")))
}
