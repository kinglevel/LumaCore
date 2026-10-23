//! The station table (decisions §30 "12d part 4": the show's
//! output stations vs the station's own socket). A station's **rig wiring** — its NIC
//! for each of the show's networks and what each NIC serves — lives in the
//! show (`ShowData.Stations`, one GUID row per station, pre-configurable
//! while a station is offline, replicated and undone like any show data).
//! Its **identity** (name, location, priority, LumaSession, Online, …)
//! stays on the station (`station_settings.json`).
//!
//! Each station applies only its own row ([`apply_own_row`], after every
//! show change) into `Root.StationSettings` — the last-applied copy kept in
//! `station_settings.json`, so a show without a row for this station (or a
//! standalone boot) keeps working with what it had.
//!
//! A station's own row is found by its station id, else (a row made before
//! the station existed) by its name with no station id yet; it is made on
//! the first wiring change from the last-applied copy ([`ensure_own_row`]).

use std::collections::BTreeMap;

use lumacore_tree::{NicServices, StationWiring, TreeNode};

use crate::AppState;

/// This station's row in the show's table, if it has one.
pub fn own_row(state: &AppState) -> Option<u32> {
    let id = &state.local_settings.station_id;
    let name = &state.root.station_settings.station_name;
    let table = &state.root.show_data.stations;
    table
        .iter()
        .find(|(_, r)| !id.is_empty() && r.station_id == *id)
        .or_else(|| table.iter().find(|(_, r)| r.station_id.is_empty() && r.meta.name().unwrap_or_default().eq_ignore_ascii_case(name)))
        .map(|(&n, _)| n)
}

/// The row a line names (`Station "Tower"`, `Station 2`): by number or name.
pub fn row_named(state: &AppState, word: &str) -> Option<u32> {
    let table = &state.root.show_data.stations;
    let word = word.trim_matches('"');
    word.parse::<u32>()
        .ok()
        .filter(|n| table.contains_key(n))
        .or_else(|| table.iter().find(|(_, r)| r.meta.name().unwrap_or_default().eq_ignore_ascii_case(word)).map(|(&n, _)| n))
}

/// The next free row number.
fn next_no(table: &BTreeMap<u32, StationWiring>) -> u32 {
    (1..).find(|n| !table.contains_key(n)).unwrap_or(1)
}

/// A new row made in advance for station `name` (`Store Station "Tower"`):
/// matched to the station of that name when it shows up.
pub fn store_row(state: &mut AppState, name: &str) -> Result<u32, String> {
    let name = name.trim_matches('"').trim();
    if name.is_empty() {
        return Err(crate::errors::render_code("Usage: Store Station \"name\"", "E-CMD-006"));
    }
    if let Some(n) = row_named(state, name) {
        return Err(crate::errors::render_code(
            format!("Station \"{name}\" already has row {n} in the station table."),
            "E-CMD-010",
        ));
    }
    let table = &mut state.root.show_data.stations;
    let no = next_no(table);
    table.insert(
        no,
        StationWiring {
            meta: lumacore_base::NodeMeta::new(Some(name.to_string())),
            ..StationWiring::default()
        },
    );
    Ok(no)
}

/// This station's row, made from the last-applied copy when the show has
/// none yet (a wiring change needs it). A row made in advance for this
/// station's name is claimed: it takes the station id.
pub fn ensure_own_row(state: &mut AppState) -> u32 {
    let id = state.local_settings.station_id.clone();
    let name = state.root.station_settings.station_name.clone();
    if let Some(n) = own_row(state) {
        let row = state.root.show_data.stations.get_mut(&n).expect("own row");
        if row.station_id.is_empty() && !id.is_empty() {
            row.station_id = id;
        }
        if row.meta.name() != Some(name.as_str()) {
            row.meta.identity.name = Some(name);
        }
        return n;
    }
    let networks = wiring_networks(state);
    let services = state.root.station_settings.services.clone();
    let table = &mut state.root.show_data.stations;
    let no = next_no(table);
    table.insert(
        no,
        StationWiring {
            meta: lumacore_base::NodeMeta::with_guid(
                if id.is_empty() { uuid::Uuid::new_v4() } else { StationWiring::guid_for(&id) },
                Some(name),
            ),
            station_id: id,
            networks,
            services,
        },
    );
    no
}

/// The last-applied network map as GUID links (the show's networks only).
fn wiring_networks(state: &AppState) -> BTreeMap<uuid::Uuid, String> {
    let map = &state.root.station_settings.networks;
    state
        .root
        .show_data
        .protocols
        .networks
        .values()
        .filter_map(|n| {
            let name = n.name()?.to_lowercase();
            map.get(&name).map(|spec| (n.meta.identity.guid, spec.clone()))
        })
        .collect()
}

/// What this station's row says, as `StationSettings` holds it: network
/// name (lowercase) → NIC, and the services per NIC. Networks of the show
/// the row doesn't map are unmapped here; maps for networks this show
/// doesn't have are kept (another show may have them).
fn derived(state: &AppState, row: &StationWiring) -> (BTreeMap<String, String>, BTreeMap<String, NicServices>) {
    let show_nets = &state.root.show_data.protocols.networks;
    let mut networks: BTreeMap<String, String> = state
        .root
        .station_settings
        .networks
        .iter()
        .filter(|(name, _)| !show_nets.values().any(|n| n.name().is_some_and(|x| x.eq_ignore_ascii_case(name))))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for n in show_nets.values() {
        if let (Some(name), Some(spec)) = (n.name(), row.networks.get(&n.meta.identity.guid)) {
            networks.insert(name.to_lowercase(), spec.clone());
        }
    }
    (networks, row.services.clone())
}

/// Apply this station's row to its station settings (after a load, a
/// replication, an Oops, any show change). Returns whether anything moved;
/// then the last-applied copy is saved and the protocols re-planned. No row
/// = nothing changes (the station keeps its last-applied wiring).
pub fn apply_own_row(state: &mut AppState) -> bool {
    let Some(n) = own_row(state) else {
        return false;
    };
    let row = state.root.show_data.stations[&n].clone();
    let (networks, services) = derived(state, &row);
    let st = &mut state.root.station_settings;
    if st.networks == networks && st.services == services {
        return false;
    }
    st.networks = networks;
    st.services = services;
    state.local_settings.store_from(&state.root.station_settings);
    state.protocols_dirty = true;
    true
}

/// A station row word: its name, quoted.
pub fn word(name: &str) -> String {
    // A name is always quoted (lum.md "Targets": an id is a number or quoted text).
    format!("\"{}\"", name.replace('"', ""))
}

/// The networks of the show by GUID → (number, name).
pub fn network_names(state: &AppState) -> BTreeMap<uuid::Uuid, (u32, String)> {
    state
        .root
        .show_data
        .protocols
        .networks
        .iter()
        .map(|(&no, n)| (n.meta.identity.guid, (no, n.name().unwrap_or("").to_string())))
        .collect()
}

// ── Remote station edits (decisions §30 "Remote station edits") ─────────────
//
// `Set Station "Tower" "Priority" High` on FOH goes straight to Tower's command
// port (its own dispatch, rights gate and log). FOH keeps what it sent here
// until Tower answers and — for a setting Tower reports in its beacon — until
// discovery shows the new value: the Stations grid's "pending" cell. Nothing
// of it is stored as desired state; the reported value is the truth.

/// How long a sent edit waits for the station to report its new value.
pub const PENDING_FOR: std::time::Duration = std::time::Duration::from_secs(30);
/// How long a refused edit stays marked in its cell.
pub const REFUSED_FOR: std::time::Duration = std::time::Duration::from_secs(10);

/// One edit of another station's own setting, sent from here.
#[derive(Debug, Clone)]
pub struct RemoteEdit {
    pub id: u64,
    pub station: String,
    /// The setting as declared (`Priority`).
    pub setting: &'static str,
    /// The value sent (canonical, as the setting takes it).
    pub value: String,
    pub since: std::time::Instant,
    /// The station said yes; waiting only for its report.
    pub answered: bool,
    /// The station refused (or didn't answer): the reason, shown for a while.
    pub refused: Option<String>,
}

/// What `station` reports of `setting` here (its discovery beacon), if it
/// reports that setting at all. `Name`: the new name once a station of that
/// name is seen.
pub fn reported(state: &AppState, station: &str, setting: &str, value: &str) -> Option<String> {
    let ls = &state.root.runtime.luma_sessions;
    let seen = |name: &str| ls.discovery.values().find(|p| p.name.eq_ignore_ascii_case(name) && p.ip != "this station");
    match setting {
        "Name" => Some(if seen(value).is_some() { value.to_string() } else { station.to_string() }),
        "Priority" => seen(station).map(|p| p.priority.clone()),
        "Location" => seen(station).map(|p| p.location.clone()),
        "LumaSession" => seen(station).map(|p| p.lumasession.clone()),
        "Invite" => seen(station).map(|p| if p.invite { "On" } else { "Off" }.to_string()),
        _ => None,
    }
}

/// Note a sent edit (one per station + setting: the newest wins); returns
/// its id for the answer.
pub fn remote_sent(state: &mut AppState, station: &str, setting: &'static str, value: &str) -> u64 {
    let id = state.remote_edits.iter().map(|e| e.id).max().unwrap_or(0) + 1;
    state.remote_edits.retain(|e| !(e.station.eq_ignore_ascii_case(station) && e.setting == setting));
    state.remote_edits.push(RemoteEdit {
        id,
        station: station.to_string(),
        setting,
        value: value.to_string(),
        since: std::time::Instant::now(),
        answered: false,
        refused: None,
    });
    id
}

/// The station answered edit `id`: Ok = done (pending until reported, if
/// the setting is reported), Err = the refusal, marked in its cell.
pub fn remote_answered(state: &mut AppState, id: u64, outcome: Result<(), String>) {
    let Some(i) = state.remote_edits.iter().position(|e| e.id == id) else {
        return;
    };
    match outcome {
        Ok(()) => {
            let e = &state.remote_edits[i];
            let reports = reported(state, &e.station, e.setting, &e.value).is_some();
            if reports {
                state.remote_edits[i].answered = true;
            } else {
                state.remote_edits.remove(i);
            }
        }
        Err(why) => {
            let e = &mut state.remote_edits[i];
            e.refused = Some(why);
            e.since = std::time::Instant::now();
        }
    }
}

/// Drop edits the station now reports, and old ones (about once a second).
pub fn tidy_remote_edits(state: &mut AppState) {
    if state.remote_edits.is_empty() {
        return;
    }
    let now = std::time::Instant::now();
    let edits = std::mem::take(&mut state.remote_edits);
    state.remote_edits = edits
        .into_iter()
        .filter(|e| match &e.refused {
            Some(_) => now.duration_since(e.since) < REFUSED_FOR,
            None => {
                let done = e.answered
                    && reported(state, &e.station, e.setting, &e.value).is_some_and(|v| v.eq_ignore_ascii_case(&e.value));
                !done && now.duration_since(e.since) < PENDING_FOR
            }
        })
        .collect();
}

/// The Stations grid's marks of one station: `pending` (field → value
/// sent) and `refused` (field → why), fields named like the grid's
/// columns (the setting's name in lower case).
pub fn remote_marks(state: &AppState, station: &str) -> (serde_json::Map<String, serde_json::Value>, serde_json::Map<String, serde_json::Value>) {
    let mut pending = serde_json::Map::new();
    let mut refused = serde_json::Map::new();
    for e in state.remote_edits.iter().filter(|e| e.station.eq_ignore_ascii_case(station)) {
        let field = e.setting.to_lowercase();
        match &e.refused {
            Some(why) => {
                refused.insert(field, why.clone().into());
            }
            None => {
                let now = reported(state, &e.station, e.setting, &e.value);
                if !(e.answered && now.is_some_and(|v| v.eq_ignore_ascii_case(&e.value))) {
                    pending.insert(field, e.value.clone().into());
                }
            }
        }
    }
    (pending, refused)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> AppState {
        let mut s = AppState::new();
        s.local_settings.station_id = "6f1c2b9e-3a4d-4e5f-8a7b-1c2d3e4f5a6b".into();
        s.root.station_settings.station_name = "FOH".into();
        lumacore_tree::LineOps::ensure_network(&mut s.root.show_data, "Main");
        s
    }

    #[test]
    fn a_station_without_a_row_keeps_its_last_applied_wiring() {
        let mut s = state();
        s.root.station_settings.networks.insert("main".into(), "en1".into());
        assert!(own_row(&s).is_none());
        assert!(!apply_own_row(&mut s));
        assert_eq!(s.root.station_settings.networks["main"], "en1");
    }

    #[test]
    fn the_own_row_is_made_from_the_last_applied_copy_and_applies_back() {
        let mut s = state();
        s.root.station_settings.networks.insert("main".into(), "en1".into());
        s.root.station_settings.networks.insert("elsewhere".into(), "en2".into());
        let n = ensure_own_row(&mut s);
        let row = &s.root.show_data.stations[&n];
        assert_eq!(row.meta.identity.guid, StationWiring::guid_for(&s.local_settings.station_id));
        assert_eq!(row.networks.len(), 1, "only the show's networks are in the row");
        // The row changes (a replication, an Oops): the station follows.
        let main = s.root.show_data.protocols.networks.values().next().unwrap().meta.identity.guid;
        s.root.show_data.stations.get_mut(&n).unwrap().networks.insert(main, "en0".into());
        assert!(apply_own_row(&mut s));
        assert_eq!(s.root.station_settings.networks["main"], "en0");
        assert_eq!(s.root.station_settings.networks["elsewhere"], "en2", "other shows' maps are kept");
        assert!(!apply_own_row(&mut s), "nothing moves twice");
    }

    #[test]
    fn a_row_made_in_advance_is_claimed_by_name() {
        let mut s = state();
        let n = store_row(&mut s, "foh").unwrap();
        assert!(store_row(&mut s, "FOH").is_err(), "one row per station");
        assert_eq!(own_row(&s), Some(n));
        assert_eq!(ensure_own_row(&mut s), n);
        assert_eq!(s.root.show_data.stations[&n].station_id, s.local_settings.station_id);
        assert_eq!(s.root.show_data.stations[&n].meta.name(), Some("FOH"));
        assert!(s.root.show_data.identity().is_clean(), "{:?}", s.root.show_data.identity());
    }
}
