// ██╗    ██╗██╗██████╗ ██╗███╗   ██╗ ██████╗
// ██║    ██║██║██╔══██╗██║████╗  ██║██╔════╝
// ██║ █╗ ██║██║██████╔╝██║██╔██╗ ██║██║  ███╗
// ██║███╗██║██║██╔══██╗██║██║╚██╗██║██║   ██║
// ╚███╔███╔╝██║██║  ██║██║██║ ╚████║╚██████╔╝
//  ╚══╝╚══╝ ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝
// FILE: wiring

//! A station's rig wiring (decisions §30 "12d part 4"): its row of the station table — which NIC
//! each network uses, which services each NIC serves — written for this station or others in one line.
//!
//! Owns:          the wiring: interfaces' services, the station's NIC for each network, wiring lines and their asks.
//! Doesn't own:   the station table's door (lumacore-state stations), the plan that applies it (the kernel's protocol_plan).
//! Entry points:  INTERFACE_SETTINGS, NETWORK_SETTINGS, names_a_station.
//! Decisions:     §11, §30 "12d part 4".

use lumacore_objects::AppState;
use lumacore_objects::commands::registry::Rights;
use lumacore_objects::errors::render_code;
use lumacore_objects::objects::ObjectDef;
use lumacore_objects::props::{Access, Home, Keyed, PropKind, Property, Props};
use lumacore_objects::settings_props::{STATION_WORDS, on, station_after, word};
use lumacore_tree::TreeNode;

use crate::settings::*;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

// *************************************************************************************************
// Wiring: interfaces and the network → NIC map (the station table)
// *************************************************************************************************
//
// A station's wiring lives in its row of the show's station table (decisions
// §30 "12d part 4"). `Set Interface "en0" "Shell" No` / `Set Network "Rig"
// Interface en1` write this station's row (made from its last-applied copy on
// the first change) and apply it at once; `Set Station "Tower" Interface …` /
// `Set Station "FOH" + "Tower" Network "Rig" "Interface" "en1"` write other
// stations' rows (pre-configured while they are offline) — the line sets the
// row it targets for the interface / network settings below.

thread_local! {
    /// The station-table row a wiring line targets (None = this station).
    static WIRING: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

/// Targets another station's row for the lifetime of the guard.
struct WiringTarget(Option<u32>);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl WiringTarget {
    fn set(row: Option<u32>) -> Self {
        Self(WIRING.with(|w| w.replace(row)))
    }
}

impl Drop for WiringTarget {
    fn drop(&mut self) {
        WIRING.with(|w| w.set(self.0));
    }
}

pub(crate) fn wiring_target() -> Option<u32> {
    WIRING.with(std::cell::Cell::get)
}

/// The row a wiring change writes: the targeted one, or this station's own
/// (made when missing). Returns it and whether it is this station's.
pub(crate) fn wiring_row(state: &mut AppState) -> (u32, bool) {
    match wiring_target() {
        Some(n) => (n, lumacore_objects::stations::own_row(state) == Some(n)),
        None => (lumacore_objects::stations::ensure_own_row(state), true),
    }
}

/// After a row write: this station applies its own row at once.
pub(crate) fn wired(state: &mut AppState, own: bool) {
    if own {
        lumacore_objects::stations::apply_own_row(state);
    }
}

pub(crate) fn target_name(state: &AppState) -> Option<String> {
    wiring_target()
        .and_then(|n| state.root.show_data.stations.get(&n))
        .and_then(|r| r.meta.identity.name.clone())
}

pub(crate) fn nic_rows(state: &AppState) -> Vec<String> {
    match wiring_target().and_then(|n| state.root.show_data.stations.get(&n)) {
        Some(row) => row.services.keys().cloned().collect(),
        None => lumacore_net::nics::list()
            .into_iter()
            .map(|n| n.name)
            .collect(),
    }
}

pub(crate) fn nic_row(state: &AppState, w: &str) -> Option<String> {
    let w = w.trim_matches('"');
    if wiring_target().is_some() {
        // Another station's interfaces aren't known here: any name is taken
        // (pre-configured), except a setting's.
        return (!w.is_empty() && INTERFACE_SETTINGS.get(w).is_none()).then(|| w.to_lowercase());
    }
    let _ = state;
    lumacore_net::nics::list()
        .into_iter()
        .map(|n| n.name)
        .find(|n| n.eq_ignore_ascii_case(w))
}

/// What NIC `nic` serves for the targeted station (its row), or for this
/// one (what it applies).
pub(crate) fn services_now(state: &AppState, nic: &str) -> lumacore_tree::NicServices {
    match wiring_target().and_then(|n| state.root.show_data.stations.get(&n)) {
        Some(row) => row.services_of(nic),
        None => state.root.station_settings.services_of(nic),
    }
}

macro_rules! service {
    ($name:literal, $field:ident, $doc:literal) => {
        Property {
            name: $name,
            kind: PropKind::Switch,
            named: &[],
            words: ("Yes", "No"),
            rights: Rights::Setup,
            section: "Services",
            unit: "",
            doc: $doc,
            default: None,
            access: Access::Keyed {
                get: |s, nic| {
                    if services_now(s, nic).$field {
                        "Yes"
                    } else {
                        "No"
                    }
                    .to_string()
                },
                set: |s, nic, v| {
                    let nic = nic.to_lowercase();
                    let (n, own) = wiring_row(s);
                    let row = s.root.show_data.stations.get_mut(&n).expect("wiring row");
                    let mut services = row.services_of(&nic);
                    services.$field = v == "Yes";
                    row.services.insert(nic.clone(), services);
                    let whose = if own {
                        String::new()
                    } else {
                        format!("Station \"{}\" ", row.meta.name().unwrap_or_default())
                    };
                    wired(s, own);
                    Ok(format!("{whose}{nic}: {} {v}", $name))
                },
            },
            home: Home::Show,
        }
    };
}

pub static INTERFACE_SETTINGS: Props = Props {
    lists: &[&[
        service!(
            "LumaSession",
            lumasession,
            "LumaSessions and discovery on this interface"
        ),
        service!(
            "Shell",
            shell,
            "the shell port (terminals, the CLI) on this interface; loopback always serves"
        ),
        service!(
            "HTTP",
            http,
            "the web UI and REST on this interface; loopback always serves"
        ),
    ]],
    keyed: Some(Keyed {
        rows: nic_rows,
        row: nic_row,
    }),
    label: Some(|s, k| match target_name(s) {
        Some(st) => format!("Station \"{st}\" Interface {k}"),
        None => format!("Interface {k}"),
    }),
    after: station_after,
    ..Props::BASE
};

// *************************************************************************************************
// The station's NIC for each show network
// *************************************************************************************************
pub(crate) fn network_name(state: &AppState, key: &str) -> String {
    key.parse::<u32>()
        .ok()
        .and_then(|n| state.root.show_data.protocols.networks.get(&n))
        .and_then(|n| n.name())
        .unwrap_or("")
        .to_string()
}

pub(crate) fn network_guid(state: &AppState, key: &str) -> Option<uuid::Uuid> {
    key.parse::<u32>()
        .ok()
        .and_then(|n| state.root.show_data.protocols.networks.get(&n))
        .map(|n| n.meta.identity.guid)
}

pub static NETWORK_SETTINGS: Props = Props {
    meta: lumacore_objects::props::MetaLevel::Node,
    lists: &[
        &[Property {
            name: "Interface",
            kind: PropKind::Text,
            named: &["None"],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Network",
            unit: "",
            doc: "the station's interface for the network (a name, address or subnet; its row in the station table); None = its lines never send here",
            default: None,
            access: Access::Keyed {
                get: |s, k| {
                    match wiring_target().and_then(|n| s.root.show_data.stations.get(&n)) {
                        Some(row) => network_guid(s, k).and_then(|g| row.networks.get(&g).cloned()),
                        None => s
                            .root
                            .station_settings
                            .networks
                            .get(&network_name(s, k).to_lowercase())
                            .cloned(),
                    }
                    .unwrap_or_else(|| "None".into())
                },
                set: |s, k, v| {
                    let shown = network_name(s, k);
                    let guid = network_guid(s, k).ok_or_else(|| {
                        render_code(format!("No Network {k} (List Network)."), "E-SET-004")
                    })?;
                    let (n, own) = wiring_row(s);
                    let row = s.root.show_data.stations.get_mut(&n).expect("wiring row");
                    let whose = if own {
                        "this station".to_string()
                    } else {
                        format!("Station \"{}\"", row.meta.name().unwrap_or_default())
                    };
                    let msg = if v.is_empty() || v == "None" {
                        row.networks.remove(&guid);
                        format!(
                            "Network \"{shown}\" unmapped on {whose}: its lines stop sending there"
                        )
                    } else {
                        row.networks.insert(guid, v.to_string());
                        let found = if own {
                            lumacore_net::nics::resolve(v, &lumacore_net::nics::list())
                                .map(|n| format!(" ({} {})", n.name, n.ip))
                                .unwrap_or_else(|| {
                                    " (no such interface right now — it is used once it appears)"
                                        .into()
                                })
                        } else {
                            String::new()
                        };
                        format!("Network \"{shown}\" → {v}{found} on {whose}")
                    };
                    wired(s, own);
                    Ok(msg)
                },
            },
            home: Home::Show,
        }],
    ],
    label: Some(|s, k| match target_name(s) {
        Some(st) => format!("Station \"{st}\" Network {k} \"{}\"", network_name(s, k)),
        None => format!("Network {k} \"{}\"", network_name(s, k)),
    }),
    apply: Some(crate::network::network_write),
    after: station_after,
    ..Props::BASE
};

// *************************************************************************************************
// Station: rows, wiring lines and the asks
// *************************************************************************************************
/// Whether a word after `Station <rows>` names a wiring object (`Interface`, `Network`).
pub(crate) fn wiring_def(word: &str) -> Option<&'static ObjectDef> {
    let obj = lumacore_objects::objects::find(word)?;
    (std::ptr::eq(obj, &crate::stations::INTERFACE) || std::ptr::eq(obj, &crate::network::NETWORK))
        .then_some(obj)
}

/// A station a wiring line names → its row (None = this station). A
/// member or a discovered station gets its row now (the same GUID it would
/// make itself); anything else needs `Store Station "x"` first.
pub(crate) fn wiring_station(state: &mut AppState, word: &str) -> Result<Option<u32>, String> {
    let w = word.trim_matches('"');
    if is_this_station(state, w) {
        return Ok(None);
    }
    if let Some(n) = lumacore_objects::stations::row_named(state, w) {
        return Ok((lumacore_objects::stations::own_row(state) != Some(n)).then_some(n));
    }
    let ls = &state.root.runtime.luma_sessions;
    let seen = ls
        .members
        .values()
        .find(|m| m.station.eq_ignore_ascii_case(w) && !m.station_id.is_empty())
        .map(|m| (m.station.clone(), m.station_id.clone()))
        .or_else(|| {
            ls.discovery
                .values()
                .find(|p| {
                    p.name.eq_ignore_ascii_case(w)
                        && !p.station_id.is_empty()
                        && p.ip != "this station"
                })
                .map(|p| (p.name.clone(), p.station_id.clone()))
        });
    let Some((name, id)) = seen else {
        return Err(render_code(
            format!(
                "No row for Station \"{w}\" in the station table (Store Station \"{w}\" makes one)."
            ),
            "E-SET-004",
        ));
    };
    let table = &mut state.root.show_data.stations;
    let no = (1..).find(|n| !table.contains_key(n)).unwrap_or(1);
    table.insert(
        no,
        lumacore_tree::StationWiring {
            meta: lumacore_base::NodeMeta::with_guid(lumacore_tree::StationWiring::guid_for(&id), Some(name)),
            station_id: id,
            ..Default::default()
        },
    );
    Ok(Some(no))
}

/// A wiring line's station (`Set Station "Tower" Network "Rig" "Interface" "en1"`): its
/// interface and network settings write that station's row while the guard lives.
pub(crate) fn station_within(
    state: &mut AppState,
    key: &str,
) -> Result<Box<dyn std::any::Any>, String> {
    let row = wiring_station(state, key)?;
    Ok(Box::new(WiringTarget::set(row)))
}

/// The ask a `Set Station` line needs before it runs, if any: Online Off,
/// HoldOutput On, Priority Never on the LumaSession's master — for this
/// station and for others (asked here, sent with /NoConfirmation).
pub(crate) fn station_ask(
    state: &mut AppState,
    rows: &[String],
    named: bool,
    s: &'static Property,
    raw: &str,
) -> Option<lumacore_objects::DispatchResult> {
    use lumacore_objects::DispatchResult;
    let raw = raw.trim().trim_matches('"');
    if raw.is_empty() {
        return None;
    }
    let Access::Keyed { get, .. } = s.access else {
        return None;
    };
    let master = if state.link.is_none() && !state.root.runtime.luma_sessions.members.is_empty() {
        Some(state.root.station_settings.station_name.clone())
    } else {
        state.link.as_ref().map(|l| l.master.clone())
    };
    let mut asked: Vec<(String, &'static str, String, &'static str)> = Vec::new();
    let mut value = String::new();
    for key in rows {
        let here = is_this_station(state, key);
        let who = if here {
            "this station".to_string()
        } else {
            format!("Station \"{key}\"")
        };
        let now = if here {
            Some(get(state, key))
        } else {
            lumacore_objects::stations::reported(state, key, s.name, "")
        };
        let want_on = if raw.eq_ignore_ascii_case("toggle") {
            match &now {
                Some(v) => !on(v),
                None => continue,
            }
        } else {
            on(raw)
        };
        let ask = match s.name {
            "Online" if !want_on && (!here || state.root.station_settings.online) => Some((
                "Go offline",
                if here {
                    "Online Off stops everything networked on this station: LumaSessions and discovery, Shell and HTTP on every interface (only localhost stays), and DMX protocols. A LumaSession is left.".to_string()
                } else {
                    format!(
                        "Online Off stops everything networked on {key}: LumaSessions and discovery, Shell and HTTP on every interface, and DMX protocols. It can be turned back on only at {key} itself."
                    )
                },
                "Go offline",
            )),
            "HoldOutput" if want_on && (!here || !state.root.station_settings.output_held) => {
                Some((
                    "Hold DMX output",
                    format!(
                        "Every output line of {who} stops sending (receivers hold or time out) until output is resumed."
                    ),
                    "Hold output",
                ))
            }
            "Priority"
                if raw.eq_ignore_ascii_case("never")
                    && master
                        .as_deref()
                        .is_some_and(|m| m.eq_ignore_ascii_case(key))
                    && now
                        .as_deref()
                        .is_none_or(|v| !v.eq_ignore_ascii_case("never")) =>
            {
                Some((
                    "Priority Never on the master",
                    format!(
                        "{who} is the master of this LumaSession. With priority Never it won't be chosen when the next election runs (nothing changes now; Claim LumaSession hands the role over)."
                    ),
                    "Set Never",
                ))
            }
            _ => None,
        };
        if let Some((title, text, button)) = ask {
            value = match s.kind {
                PropKind::OnOff | PropKind::Switch => word(s, want_on),
                _ => "Never".to_string(),
            };
            asked.push((who, title, text, button));
        }
    }
    let (who, title, _, button) = asked.first().cloned()?;
    // This station's own level is checked before asking; another station
    // checks its own when the line arrives.
    if rows.iter().any(|k| is_this_station(state, k))
        && let Err(e) = lumacore_objects::rights::require(
            state,
            lumacore_objects::props::rights_of(&crate::stations::STATION, s),
            &format!("Set Station \"{}\"", s.name),
        )
    {
        return Some(DispatchResult::Output(e.into()));
    }
    let text = asked
        .iter()
        .map(|(_, _, t, _)| t.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let meta = if asked.len() == 1 {
        who
    } else {
        asked
            .iter()
            .map(|(w, _, _, _)| w.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let row_words = if named {
        rows.iter()
            .map(|k| format!("\"{k}\""))
            .collect::<Vec<_>>()
            .join(" + ")
    } else {
        String::new()
    };
    let prop = format!("\"{}\"", s.name);
    let line = [
        "Set Station",
        row_words.as_str(),
        prop.as_str(),
        value.as_str(),
        "/NoConfirmation",
    ]
    .iter()
    .filter(|w| !w.is_empty())
    .copied()
    .collect::<Vec<_>>()
    .join(" ");
    Some(DispatchResult::Ask(Box::new(
        lumacore_objects::dialogs::Ask::new(title, text)
            .meta(meta, Some(lumacore_wire::Tone::Warn))
            .button(button, lumacore_wire::ButtonKind::Danger, line)
            .cancel("Cancel")
            .flags("/NoConfirmation"),
    )))
}

/// `Set Station <rows> …`: the line names stations (not this station's
/// setting or wiring word first).
pub fn names_a_station(args: &[&str]) -> bool {
    args.first().is_some_and(|a| {
        !a.starts_with('/') && STATION_SETTINGS.get(a).is_none() && wiring_def(a).is_none()
    })
}

#[cfg(test)]
mod tests;
