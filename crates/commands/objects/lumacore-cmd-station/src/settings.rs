// ███████╗███████╗████████╗████████╗██╗███╗   ██╗ ██████╗ ███████╗
// ██╔════╝██╔════╝╚══██╔══╝╚══██╔══╝██║████╗  ██║██╔════╝ ██╔════╝
// ███████╗█████╗     ██║      ██║   ██║██╔██╗ ██║██║  ███╗███████╗
// ╚════██║██╔══╝     ██║      ██║   ██║██║╚██╗██║██║   ██║╚════██║
// ███████║███████╗   ██║      ██║   ██║██║ ╚████║╚██████╔╝███████║
// ╚══════╝╚══════╝   ╚═╝      ╚═╝   ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
// FILE: settings

//! A station's own settings (decisions §30 "12d"): typed keyed properties of the Station object;
//! another station's go straight to its command port, checked and logged there.
//!
//! Owns:          this station's settings (Name, Priority, Online, HoldOutput, …) and another station's, sent to it.
//! Doesn't own:   the wiring (wiring.rs), the show's settings (the kernel's settings_props).
//! Entry points:  STATION_SETTINGS, PRIORITY_CHOICES, THEME_OVERRIDE_CHOICES.
//! Decisions:     §30 "12d", "Remote station edits".

use lumacore_objects::AppState;
use lumacore_objects::commands::registry::Rights;
use lumacore_objects::commands::set::Row;
use lumacore_objects::errors::render_code;
use lumacore_objects::props::{Access, Home, Keyed, PropKind, Property, Props};
use lumacore_objects::settings_props::{STATION_WORDS, on, station_after, word};

use crate::wiring::*;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// This station's own settings and another station's (sent to it): a station's `Set` writes
/// its own way (decisions §30 "Remote station edits").
pub(crate) fn station_write(
    state: &mut AppState,
    key: &str,
    s: &'static Property,
    raw: &str,
) -> Option<Result<String, String>> {
    // Another station's own settings change on that station: its rights gate, its log — no
    // check here.
    if !is_this_station(state, key) {
        return Some(remote_station_set(state, key, s, raw));
    }
    let station = &crate::stations::STATION;
    Some((|| {
        lumacore_objects::rights::require(
            state,
            lumacore_objects::props::rights_of(station, s),
            &format!("Set Station \"{}\"", s.name),
        )?;
        let row = Row {
            key: key.to_string(),
            no: 0,
            datapool: None,
            label: "Station".to_string(),
        };
        let value = lumacore_objects::commands::set::checked(state, station, &row, s, raw)?;
        station_apply(state, key, s, &value)
    })())
}

/// This station's own setting: `Priority Normal → High` (+ what follows
/// from it). Sent from another station, it is logged with the sender and
/// the reply carries the line that puts it back (Oops on the sender
/// doesn't: identity is outside the change path).
pub(crate) fn station_apply(
    state: &mut AppState,
    key: &str,
    s: &Property,
    value: &str,
) -> Result<String, String> {
    let Access::Keyed { get, set } = s.access else {
        return Err(render_code(
            format!("Station \"{}\" isn't a station setting.", s.name),
            "E-SET-001",
        ));
    };
    let old = get(state, key);
    let note = set(state, key, value)?;
    station_after(state);
    let new = get(state, key);
    let shown = |v: &str| {
        if v.is_empty() {
            "\"\"".to_string()
        } else if matches!(s.kind, PropKind::Text) {
            format!("\"{v}\"")
        } else {
            v.to_string()
        }
    };
    let change = if old == new {
        format!("{} stays {}", s.name, shown(&new))
    } else {
        format!("{} {} → {}", s.name, shown(&old), shown(&new))
    };
    let mut msg = if note.is_empty() {
        change.clone()
    } else {
        format!("{change} — {note}")
    };
    if let Some(from) = state.remote_from.clone() {
        state
            .sysmon_pending
            .push(("Station".into(), format!("{from}: {change}")));
        if old != new {
            let me = lumacore_objects::stations::word(&state.root.station_settings.station_name);
            let back = if old.is_empty()
                || old.contains(char::is_whitespace)
                || matches!(s.kind, PropKind::Text)
            {
                format!("\"{old}\"")
            } else {
                old
            };
            msg.push_str(&format!(
                "\nPut back: Set Station {me} \"{}\" {back}",
                s.name
            ));
        }
    }
    Ok(msg)
}

// *************************************************************************************************
// This station
// *************************************************************************************************
pub(crate) fn this_station(state: &AppState) -> String {
    state.root.station_settings.station_name.clone()
}

/// Every station a `Set Station` line may name: this one, the station
/// table's rows, the LumaSession's members and the stations discovery sees.
pub(crate) fn station_rows(state: &AppState) -> Vec<String> {
    let mut out = vec![this_station(state)];
    let mut push = |n: &str| {
        if !n.is_empty() && !out.iter().any(|o| o.eq_ignore_ascii_case(n)) {
            out.push(n.to_string());
        }
    };
    for r in state.root.show_data.stations.values() {
        push(r.meta.name().unwrap_or_default());
    }
    let ls = &state.root.runtime.luma_sessions;
    for m in ls.members.keys() {
        push(m);
    }
    for p in ls.discovery.values().filter(|p| p.cmd_port != 0) {
        push(&p.name);
    }
    out
}

pub(crate) fn station_row(state: &AppState, w: &str) -> Option<String> {
    let w = w.trim_matches('"');
    if let Some(n) = w
        .parse::<u32>()
        .ok()
        .and_then(|n| state.root.show_data.stations.get(&n))
    {
        return n.meta.identity.name.clone();
    }
    station_rows(state)
        .into_iter()
        .find(|n| n.eq_ignore_ascii_case(w))
}

pub(crate) fn is_this_station(state: &AppState, name: &str) -> bool {
    state
        .root
        .station_settings
        .station_name
        .eq_ignore_ascii_case(name)
}

/// Where another station answers: a member of this LumaSession, else a
/// station discovery sees (a standalone neighbour too).
pub(crate) fn reachable(state: &AppState, name: &str) -> Option<(String, String, u16)> {
    let ls = &state.root.runtime.luma_sessions;
    ls.members
        .values()
        .find(|m| m.station.eq_ignore_ascii_case(name) && m.cmd_port != 0)
        .map(|m| (m.station.clone(), m.ip.clone(), m.cmd_port))
        .or_else(|| {
            ls.discovery
                .values()
                .find(|p| {
                    p.name.eq_ignore_ascii_case(name) && p.cmd_port != 0 && p.ip != "this station"
                })
                .map(|p| (p.name.clone(), p.ip.clone(), p.cmd_port))
        })
}

/// `Set Station "Tower" "Priority" High` on another station (decisions §30
/// "Remote station edits"): sent to its command port, where it runs as this
/// user through its own dispatch — its rights gate, its log. The value goes
/// absolute (a retry sets the same again); `Toggle` and relative numbers
/// need the value the station has, so only a value it reports takes them.
/// The answer comes back as a follow-up (`old → new` and the line that puts
/// it back); the grid cell is pending until the station reports it.
pub(crate) fn remote_station_set(
    state: &mut AppState,
    name: &str,
    s: &'static Property,
    raw: &str,
) -> Result<String, String> {
    let Some((station, ip, port)) = reachable(state, name) else {
        return Err(render_code(
            format!(
                "Station \"{name}\" isn't reachable: its own settings ({}) change only on it. Its wiring row changes here: Set Station {} Interface … / Network … \"Interface\" …",
                s.name,
                lumacore_objects::stations::word(name)
            ),
            "E-CMD-010",
        ));
    };
    let unquoted = raw.trim().trim_matches('"');
    let value = if matches!(s.kind, PropKind::Switch) && unquoted.eq_ignore_ascii_case("toggle") {
        match lumacore_objects::stations::reported(state, &station, s.name, "") {
            Some(now) => word(s, !on(&now)),
            None => {
                return Err(render_code(
                    format!(
                        "Station \"{station}\" {}: say {} or {} — its value isn't reported here, so Toggle can't be sent.",
                        s.name, s.words.0, s.words.1
                    ),
                    "E-CMD-008",
                ));
            }
        }
    } else if matches!(s.kind, PropKind::Int { .. })
        && lumacore_objects::props::relative(unquoted, 0).is_some()
    {
        return Err(render_code(
            format!(
                "Station \"{station}\" {}: give the new value — its value isn't reported here, so {unquoted} can't be sent.",
                s.name
            ),
            "E-CMD-008",
        ));
    } else {
        let row = Row {
            key: station.clone(),
            no: 0,
            datapool: None,
            label: format!("Station \"{station}\""),
        };
        lumacore_objects::commands::set::checked(state, &crate::stations::STATION, &row, s, raw)?
    };
    let quoted = if value.is_empty()
        || value.contains(char::is_whitespace)
        || matches!(s.kind, PropKind::Text)
    {
        format!("\"{value}\"")
    } else {
        value.clone()
    };
    // Asks happened here; the station runs it without asking again.
    let cmd = format!("Set Station \"{}\" {quoted} /NoConfirmation", s.name);
    let id = lumacore_objects::stations::remote_sent(state, &station, s.name, &value);
    let mut reply_to = state.reply_to.clone();
    // A one-shot request (UDP / HTTP) waits for the station's answer.
    if let Some(waits) = &mut state.forward_waits {
        let (tx, rx) = crossbeam_channel::bounded(1);
        waits.push(rx);
        reply_to = lumacore_objects::session_link::ReplyTo::Waiter(tx);
    }
    state.session_requests.push(
        lumacore_objects::session_link::SessionRequest::EditStation {
            id,
            station: station.clone(),
            ip,
            port,
            request: lumacore_wire::RemoteRequest::Command {
                user: state.effective_user_name().to_string(),
                cmd,
            },
            reply_to,
        },
    );
    Ok(format!(
        "Station \"{station}\": {} → {quoted} sent, waiting for its answer",
        s.name
    ))
}

/// The station's theme override (`Off` or a theme preset).
pub const THEME_OVERRIDE_CHOICES: &[&str] = &["Off", "Dark", "Daylight", "Night"];
/// The election priority's words (`lumacore_runtime::StationPriority`).
// tidy: not-keywords (the priority ladder's words, as Set takes them)
pub const PRIORITY_CHOICES: &[&str] = &["Never", "VeryLow", "Low", "Normal", "High"];
pub(crate) const MULTICAST_CHOICES: &[&str] = &["Default", "Alternative"];

macro_rules! station_bool {
    ($name:literal, $doc:literal, $field:ident, $msg:expr) => {
        Property {
            name: $name,
            kind: PropKind::Switch,
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Station",
            unit: "",
            doc: $doc,
            default: None,
            access: Access::Keyed {
                get: |s, _| {
                    if s.root.station_settings.$field {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string()
                },
                set: |s, _, v| {
                    let on = v == "On";
                    s.root.station_settings.$field = on;
                    Ok(($msg)(on))
                },
            },
            home: Home::Station,
        }
    };
}

pub static STATION_SETTINGS: Props = Props {
    lists: &[&[
        Property {
            name: "Name",
            kind: PropKind::Text,
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Admin,
            section: "Station",
            unit: "",
            doc: "this station's name (discovery, the LumaSessions grid)",
            default: None,
            access: Access::Keyed {
                get: |s, _| s.root.station_settings.station_name.clone(),
                set: |s, _, v| {
                    if v.is_empty() {
                        return Err(render_code("A station needs a name.", "E-CMD-008"));
                    }
                    s.root.station_settings.station_name = v.to_string();
                    Ok(String::new())
                },
            },
            home: Home::Station,
        },
        Property {
            name: "Location",
            kind: PropKind::Text,
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Station",
            unit: "",
            doc: "where the station is (a label in the LumaSessions grid)",
            default: None,
            access: Access::Keyed {
                get: |s, _| s.root.station_settings.location.clone(),
                set: |s, _, v| {
                    s.root.station_settings.location = v.to_string();
                    Ok(String::new())
                },
            },
            home: Home::Station,
        },
        Property {
            name: "Priority",
            kind: PropKind::Choice(PRIORITY_CHOICES),
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Station",
            unit: "",
            doc: "who becomes master when the master is lost",
            default: None,
            access: Access::Keyed {
                get: |s, _| s.root.station_settings.station_priority.to_string(),
                set: |s, _, v| {
                    let p = v
                        .parse::<lumacore_tree::StationPriority>()
                        .map_err(|e| render_code(e, "E-CMD-008"))?;
                    let changed = s.root.station_settings.station_priority != p;
                    s.root.station_settings.station_priority = p;
                    // Election runs only when a master is lost (§21 "6c").
                    Ok(if changed {
                        "takes effect at the next election (Claim LumaSession hands the master role over now)".to_string()
                    } else {
                        String::new()
                    })
                },
            },
            home: Home::Station,
        },
        Property {
            name: "ThemeOverride",
            kind: PropKind::Choice(THEME_OVERRIDE_CHOICES),
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Station",
            unit: "",
            doc: "every device of this station shows the normal theme with this preset (Off = each user's look)",
            default: None,
            access: Access::Keyed {
                get: |s, _| {
                    s.root
                        .station_settings
                        .theme_override
                        .clone()
                        .unwrap_or_else(|| "Off".into())
                },
                set: |s, _, v| {
                    if v == "Off" {
                        s.root.station_settings.theme_override = None;
                        Ok("each user's own look".to_string())
                    } else {
                        s.root.station_settings.theme_override = Some(v.to_string());
                        Ok("on every device of this station".to_string())
                    }
                },
            },
            home: Home::Station,
        },
        Property {
            name: "MulticastBase",
            kind: PropKind::Choice(MULTICAST_CHOICES),
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Admin,
            section: "Station",
            unit: "",
            doc: "the discovery group (a multicast base): stations on another group don't see this one",
            default: None,
            access: Access::Keyed {
                get: |s, _| {
                    if s.root.station_settings.multicast_alternative {
                        "Alternative"
                    } else {
                        "Default"
                    }
                    .to_string()
                },
                set: |s, _, v| {
                    let alternative = v == "Alternative";
                    s.root.station_settings.multicast_alternative = alternative;
                    Ok(format!(
                        "group {}: stations on another group no longer see this one",
                        lumacore_wire::discovery_group(alternative)
                    ))
                },
            },
            home: Home::Station,
        },
        Property {
            name: "StartupShow",
            kind: PropKind::Text,
            named: &["Last", "None"],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Station",
            unit: "",
            doc: "the show this station loads when it starts: Last, None or a show's name",
            default: None,
            access: Access::Keyed {
                get: |s, _| match &s.local_settings.startup_show {
                    lumacore_wire::StartupShow::Last => "Last".to_string(),
                    lumacore_wire::StartupShow::None => "None".to_string(),
                    lumacore_wire::StartupShow::Named(n) => n.clone(),
                },
                set: |s, _, v| {
                    let (startup, msg) = match v {
                        "Last" | "" => (
                            lumacore_wire::StartupShow::Last,
                            "the last show".to_string(),
                        ),
                        "None" => (
                            lumacore_wire::StartupShow::None,
                            "an empty show".to_string(),
                        ),
                        n => (
                            lumacore_wire::StartupShow::Named(n.to_string()),
                            String::new(),
                        ),
                    };
                    s.local_settings.startup_show = startup;
                    s.local_settings.save();
                    Ok(msg)
                },
            },
            home: Home::Station,
        },
        Property {
            name: "RecoveryKeepDays",
            kind: PropKind::Int { min: 1, max: 365 },
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Cleanup",
            unit: "days",
            doc: "how long this station keeps show versions and recovery copies (thinned: all of the last hour, one per hour today, one per day for 7 days, then one per week); pinned ones always, signal / merge copies at least 30 days",
            default: None,
            access: Access::Keyed {
                get: |s, _| lumacore_objects::recovery::rules(s).keep_days.to_string(),
                set: |s, _, v| {
                    let days = v.parse::<u32>().unwrap_or(30);
                    s.local_settings.recovery_keep_days = Some(days);
                    s.local_settings.save();
                    Ok(format!("versions and recovery copies kept {days} days"))
                },
            },
            home: Home::Station,
        },
        Property {
            name: "RecoveryCap",
            kind: PropKind::Int {
                min: 50,
                max: 100_000,
            },
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Cleanup",
            unit: "MB",
            doc: "the most this station's show versions and recovery copies may use together; the oldest unpinned go first",
            default: None,
            access: Access::Keyed {
                get: |s, _| lumacore_objects::recovery::rules(s).cap_mb.to_string(),
                set: |s, _, v| {
                    let mb = v.parse::<u32>().unwrap_or(500);
                    s.local_settings.recovery_cap_mb = Some(mb);
                    s.local_settings.save();
                    Ok(format!("versions and recovery copies capped at {mb} MB"))
                },
            },
            home: Home::Station,
        },
        Property {
            name: "LumaSession",
            kind: PropKind::Text,
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Admin,
            section: "Station",
            unit: "",
            doc: "the LumaSession this station belongs to (found by discovery)",
            default: None,
            access: Access::Keyed {
                get: |s, _| s.root.station_settings.lumasession.clone(),
                set: |s, _, v| {
                    if v.is_empty() {
                        return Err(render_code("A LumaSession needs a name.", "E-CMD-008"));
                    }
                    if !s.root.runtime.joined_sessions.is_empty() {
                        return Err(render_code(
                            "Leave the LumaSession before renaming it on this station.",
                            "E-CMD-010",
                        ));
                    }
                    s.root.station_settings.lumasession = v.to_string();
                    Ok(String::new())
                },
            },
            home: Home::Station,
        },
        Property {
            name: "Online",
            kind: PropKind::Switch,
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Admin,
            section: "Station",
            unit: "",
            doc: "everything networked on; Off serves only localhost and stops DMX protocols",
            default: None,
            access: Access::Keyed {
                get: |s, _| {
                    if s.root.station_settings.online {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string()
                },
                set: |s, _, v| {
                    let on = v == "On";
                    let mut note = String::new();
                    if !on && !s.root.runtime.joined_sessions.is_empty() {
                        note = format!(
                            "\n{}",
                            lumacore_objects::text::strip_ansi(
                                &lumacore_objects::commands::session::leave_lumasession(s)
                            )
                        );
                    }
                    s.root.station_settings.online = on;
                    Ok(if on {
                        note
                    } else {
                        format!("only localhost is served, DMX protocols stop{note}")
                    })
                },
            },
            home: Home::Station,
        },
        station_bool!(
            "Invite",
            "other masters may invite this station",
            invite,
            |_: bool| String::new()
        ),
        station_bool!(
            "HoldOutput",
            "hold DMX output (after leaving or being dismissed)",
            output_held,
            |on: bool| if on {
                "DMX output held".to_string()
            } else {
                "DMX output resumed".to_string()
            }
        ),
        station_bool!(
            "OutputWhenAlone",
            "send DMX after losing the LumaSession (IdleMaster)",
            output_when_alone,
            |_: bool| String::new()
        ),
    ]],
    keyed: Some(Keyed {
        rows: station_rows,
        row: station_row,
    }),
    implicit: Some(this_station),
    label: Some(|s, k| {
        if is_this_station(s, k) {
            "Station".to_string()
        } else {
            format!("Station \"{k}\"")
        }
    }),
    after: station_after,
    own_write: Some(station_write),
    ask: Some(station_ask),
    within: Some(station_within),
    ..Props::BASE
};
