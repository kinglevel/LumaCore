// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗    ██████╗  ██████╗ ██╗    ██╗███████╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║    ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║    ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║    ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝    ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝     ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: menu rows

//! What the Network, DMX Protocols, LumaSessions and Settings › Assets menus
//! show (decisions §11, §30 "12d part 4"), built on the state thread and
//! published in the `StationView`: the HTTP engine only reads it.
//!
//! Every grid row carries every field its `MenuDef` columns declare (a
//! field a row lacks is drawn read-only), its row key (`guid`, or a NIC's
//! name) and, where rows mix objects, its object word.
//!
//! Owns:          `network`, `lumasessions` (the rows of the `network` and `lumasessions` feeds).
//! Doesn't own:   the menus' shape (`menus`), the state thread that builds them (the server).
//! Entry points:  network, lumasessions.
//! Decisions:     §11, §30 "12d part 4", §34 20.7.

use lumacore_tree::{LineOps, TreeNode};
use lumacore_protocols::{Direction};
use serde_json::{Value, json};

use lumacore_objects::AppState;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS


/// The dot of a line state.
fn line_tone(active: bool, problem: bool, direction: Direction) -> &'static str {
    match (problem, active, direction) {
        (true, _, _) => "alert",
        (false, true, Direction::In) => "warn",
        (false, true, Direction::Out) => "ok",
        _ => "off",
    }
}

/// `GET /api/network`: interfaces with their services, the show's networks
/// with this station's map, every line with its routes and live state, the
/// routing rows, the nodes and sources found, the universes (merge mode)
/// and the asset drives with this station's folders.
pub fn network(state: &AppState) -> Value {
    let st = &state.root.station_settings;
    let show = &state.root.show_data;
    let rt = &state.root.runtime.network;
    let nics = lumacore_net::nics::list();
    let interfaces: Vec<Value> = rt
        .interfaces
        .values()
        .enumerate()
        .map(|(i, nic)| {
            let up = nic.active && !nic.address.is_empty();
            json!({
                "no": i + 1, "name": nic.name, "object": "Interface", "address": nic.address,
                "link": if up { "up" } else { "no link" }, "link_tone": if up { "ok" } else { "warn" },
                "lumasession": nic.lumasession, "shell": nic.shell, "http": nic.http,
                "loopback": nic.address.starts_with("127."),
            })
        })
        .collect();
    // On a member the master sends; its line states come with the session
    // runtime ("sent by FOH").
    let master = state.link.as_ref().map(|l| l.master.clone());
    let master_lines = &state.root.runtime.luma_sessions.master_lines;
    let states = lumacore_objects::protocol_plan::line_states(state);
    let network_name = |g: &uuid::Uuid| {
        show.protocols.networks.values().find(|n| n.meta.identity.guid == *g).and_then(|n| n.name()).unwrap_or("").to_string()
    };
    let mut lines: Vec<Value> = Vec::new();
    let mut routes: Vec<Value> = Vec::new();
    for (protocol, id, l) in show.protocols.all_lines() {
        let dir = l.mode;
        let (state_text, active, problem) = match &master {
            Some(m) => match master_lines.iter().find(|s| s.protocol == protocol && s.line == id) {
                Some(s) => (format!("sent by {m}: {}", s.state), s.active, s.problem),
                None => (format!("sent by {m}"), false, false),
            },
            None => match states.iter().find(|s| s.protocol == protocol && s.line == id) {
                Some(s) => (s.state.clone(), s.active, s.problem),
                None => (String::new(), false, false),
            },
        };
        let line_routes = LineOps::routes_of(show, protocol, id);
        let net = network_name(&l.network);
        let dmx = lumacore_protocols::protocol::find(protocol).is_some_and(|p| p.kind == lumacore_protocols::Kind::Dmx);
        let mut row = json!({
            "guid": l.meta.identity.guid, "no": id, "line": id, "object": protocol, "name": l.name().unwrap_or(""),
            "direction": if dir == Direction::Out { "out" } else { "in" }, "mode": dir.label(),
            "protocol": protocol, "network": net, "network_guid": l.network,
            "universes": if dmx { l.run_label() } else { "—".to_string() },
            "target": l.target.map_or_else(|| "none".to_string(), |t| t.to_string()), "enabled": l.enabled,
            "routes": line_routes,
            "state": state_text, "tone": line_tone(active, problem, dir), "active": active, "problem": problem,
            "rate_hz": l.rate, "send": l.send.label(), "sync": l.sync, "packet_gap": l.packet_gap,
            "input_priority": l.input_priority.label(),
            "user": l.user.and_then(|g| show.user_by_guid(g)).map(|u| u.login.clone()),
        });
        // Its protocol's settings (sACN's Priority, OSC's Port), by their declared names.
        for (key, value) in l.declared() {
            row[key] = value;
        }
        lines.push(row);
        for (u, w) in line_routes {
            let uni = show.devices.dmx.patch.universes.get(&u);
            routes.push(json!({
                "key": format!("{u}:{protocol}{id}"), "guid": uni.map(|x| x.meta.identity.guid),
                "universe": u, "object": "Universe", "network": net,
                "line": format!("{protocol} {id}"), "line_name": l.name().unwrap_or(""),
                "line_universe": lumacore_protocols::wire_label(protocol, w),
                "direction": if dir == Direction::Out { "out" } else { "in" },
                "merge_mode": uni.map(|x| x.merge_mode.label()),
            }));
        }
    }
    // Universes no line routes: a row on every network ("not routed").
    for (&u, uni) in &show.devices.dmx.patch.universes {
        if !routes.iter().any(|r| r["universe"] == json!(u)) {
            routes.push(json!({
                "key": format!("{u}:-"), "guid": uni.meta.identity.guid, "universe": u, "object": "Universe", "network": "*",
                "line": "—", "line_universe": "—", "direction": "", "merge_mode": uni.merge_mode.label(), "tone": "off",
            }));
        }
    }
    let networks: Vec<Value> = show
        .protocols
        .networks
        .iter()
        .map(|(&id, n)| {
            let name = n.name().unwrap_or("");
            let spec = st.networks.get(&name.to_lowercase());
            let nic = spec.and_then(|s| lumacore_net::nics::resolve(s, &nics));
            let on: Vec<&Value> = lines.iter().filter(|l| l["network_guid"] == json!(n.meta.identity.guid)).collect();
            let out = on.iter().any(|l| l["direction"] == "out" && l["active"] == true);
            let inn = on.iter().any(|l| l["direction"] == "in" && l["active"] == true);
            let (state_text, tone) = match (spec, &nic) {
                (None, _) => ("not mapped — nothing is sent".to_string(), "alert"),
                (Some(s), None) => (format!("no interface '{s}' here"), "alert"),
                (Some(_), Some(n)) => (format!("{} {}", n.name, n.ip), "ok"),
            };
            json!({
                "no": id, "id": id, "guid": n.meta.identity.guid, "object": "Network", "name": name,
                "interface": spec.cloned().unwrap_or_else(|| "None".into()),
                "resolved": nic.as_ref().map(|n| format!("{} {}", n.name, n.ip)),
                "state": state_text, "tone": tone,
                "lines": on.iter().map(|l| format!("{} {}", l["direction"].as_str().unwrap_or(""), l["line"])).collect::<Vec<_>>().join(" · "),
                "counts": format!("{} out · {} in", on.iter().filter(|l| l["direction"] == "out").count(), on.iter().filter(|l| l["direction"] == "in").count()),
                // The stripe of the left tab: what is really sent / heard.
                "flow": match (out, inn) { (true, true) => "both", (true, false) => "out", (false, true) => "in", _ => "off" },
                "note": n.meta.annotation.note.clone().unwrap_or_default(),
            })
        })
        .collect();
    let universes: Vec<Value> = show
        .devices
        .dmx
        .patch
        .universes
        .iter()
        .map(|(&id, u)| json!({"id": id, "name": u.name().unwrap_or(""), "merge_mode": u.merge_mode.label()}))
        .collect();
    let mut nodes: Vec<Value> = rt
        .sources
        .iter()
        .map(|s| {
            json!({
                "ip": format!("{} {}", s.ip, s.id), "address": s.ip, "name": s.name, "network": s.network,
                "protocol": format!("{} source", s.protocol), "universes": s.universes.iter().map(u16::to_string).collect::<Vec<_>>().join(", "),
                "seen": "now",
            })
        })
        .collect();
    nodes.extend(rt.nodes.iter().map(|n| {
        json!({
            "ip": n.ip, "address": n.ip, "name": n.name, "network": n.network, "protocol": format!("{} node", n.protocol),
            "universes": n.outputs.iter().map(|w| lumacore_protocols::wire_label(&n.protocol, *w)).collect::<Vec<_>>().join(", "),
            "seen": "now",
        })
    }));
    let asset_roots: Vec<Value> = show
        .assets
        .roots
        .iter()
        .map(|(&no, r)| {
            let folder = st.asset_roots.get(&r.name.to_lowercase());
            let found = folder.is_some_and(|f| lumacore_luma::drive::metadata(f).is_ok());
            json!({
                "no": no, "guid": r.meta.identity.guid, "object": "AssetRoot", "name": r.name,
                "folder": folder.cloned().unwrap_or_else(|| "None".into()),
                "state": match (folder, found) { (None, _) => "not mapped", (Some(_), true) => "found", (Some(_), false) => "missing" },
                "tone": if found { "ok" } else { "warn" },
            })
        })
        .collect();
    json!({
        "station": st.station_name,
        "online": st.online,
        "output_held": st.output_held,
        "output_when_alone": st.output_when_alone,
        "master": master,
        "interfaces": interfaces,
        "networks": networks,
        "lines": lines,
        "routes": routes,
        "nodes": nodes,
        "universes": universes,
        "sources": rt.sources,
        "asset_roots": asset_roots,
    })
}

/// `GET /api/lumasessions`: this station's settings and status, every
/// station discovery sees, the LumaSession's stations with the station
/// table's wiring (the Stations grid), every station that ever joined.
pub fn lumasessions(state: &AppState) -> Value {
    let st = &state.root.station_settings;
    let show = &state.root.show_data;
    let ls = &state.root.runtime.luma_sessions;
    let stations: Vec<Value> = ls
        .discovery
        .values()
        .filter(|p| p.cmd_port != 0)
        .map(|p| {
            json!({
                "name": p.name, "ip": p.ip, "lumasession": p.lumasession,
                "location": p.location, "status": p.status, "priority": p.priority,
                "invite": p.invite, "version": p.version, "me": p.ip == "this station",
                "reachable": p.reachable, "show_status": p.show_status, "epoch": p.epoch,
                "stream_port": p.stream_port,
            })
        })
        .collect();
    let status = lumacore_objects::session_link::status_word(state);
    let joined = state.link.as_ref().map(|l| l.master.clone());
    let me = st.station_name.clone();
    let networks: Vec<Value> = show
        .protocols
        .networks
        .iter()
        .map(|(&no, n)| json!({"no": no, "guid": n.meta.identity.guid, "name": n.name().unwrap_or("")}))
        .collect();
    let clock = state.root.runtime.health.clock_view();
    // The Stations grid: this station, the session's stations, then rows of
    // the station table that aren't here (made in advance, or offline).
    let mut names: Vec<String> = vec![me.clone()];
    let mut push = |n: &str| {
        if !n.is_empty() && !names.iter().any(|x| x.eq_ignore_ascii_case(n)) {
            names.push(n.to_string());
        }
    };
    if let Some(m) = &joined {
        push(m);
    }
    for m in ls.members.keys() {
        push(m);
    }
    for r in show.stations.values() {
        push(r.meta.name().unwrap_or_default());
    }
    let own_row = lumacore_objects::stations::own_row(state);
    let members: Vec<Value> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_me = name.eq_ignore_ascii_case(&me);
            let member = ls.members.values().find(|m| m.station.eq_ignore_ascii_case(name));
            let seen = ls.discovery.values().find(|p| p.name.eq_ignore_ascii_case(name) && p.ip != "this station");
            let row_no = if is_me {
                own_row.filter(|n| show.stations.contains_key(n))
            } else {
                show.stations.iter().find(|(_, r)| r.meta.name().unwrap_or_default().eq_ignore_ascii_case(name)).map(|(n, _)| *n)
            };
            let row = row_no.and_then(|n| show.stations.get(&n));
            let role = if is_me {
                status.to_string()
            } else if joined.as_deref().is_some_and(|m| m.eq_ignore_ascii_case(name)) {
                "Master".to_string()
            } else if let Some(m) = member {
                if m.connected { "Member".into() } else { "Lost".into() }
            } else if seen.is_some() {
                "Seen".into()
            } else {
                "Offline".into()
            };
            let tone = match role.as_str() {
                "Master" => "warn",
                "Member" | "Standalone" => "ok",
                "Lost" | "Offline" => "alert",
                _ => "off",
            };
            let (priority, location) = if is_me {
                (st.station_priority.to_string(), st.location.clone())
            } else {
                seen.map_or((String::new(), String::new()), |p| (p.priority.clone(), p.location.clone()))
            };
            let station_id = if is_me {
                state.local_settings.station_id.clone()
            } else {
                member.map(|m| m.station_id.clone()).filter(|s| !s.is_empty()).or_else(|| seen.map(|p| p.station_id.clone())).unwrap_or_default()
            };
            let guid = row.map(|r| r.meta.identity.guid).unwrap_or_else(|| {
                if station_id.is_empty() {
                    lumacore_base::Guid::named("Station", name).0
                } else {
                    lumacore_tree::StationWiring::guid_for(&station_id)
                }
            });
            let rtt = clock.members.iter().find(|c| c.station.eq_ignore_ascii_case(name)).map(|c| {
                let ms = c.path_delay_us as f64 * 2.0 / 1000.0;
                format!("{ms:.1}")
            });
            let users: Vec<String> = if is_me {
                state.root.runtime.users.iter().filter(|(_, u)| u.attached).map(|(n, _)| n.clone()).collect()
            } else if joined.as_deref().is_some_and(|m| m.eq_ignore_ascii_case(name)) {
                ls.master_users.clone()
            } else {
                member.map(|m| m.users.clone()).unwrap_or_default()
            };
            let mut v = json!({
                "no": i + 1, "guid": guid, "object": "Station", "name": name, "me": is_me, "role": role, "tone": tone,
                "priority": priority, "location": location, "rtt": rtt.unwrap_or_else(|| if is_me { "—".into() } else { String::new() }),
                "users": if users.is_empty() { "—".to_string() } else { users.join(", ") },
                "row": row_no, "station_id": station_id,
                "address": member.map(|m| m.ip.clone()).or_else(|| seen.map(|p| p.ip.clone())).unwrap_or_default(),
            });
            // Edits sent to it, not yet reported back / refused (decisions
            // §30 "Remote station edits"): the grid's pending cells.
            if !is_me {
                let (pending, refused) = lumacore_objects::stations::remote_marks(state, name);
                if !pending.is_empty() {
                    v["pending"] = Value::Object(pending);
                }
                if !refused.is_empty() {
                    v["refused"] = Value::Object(refused);
                }
            }
            // One column per show network: the station's NIC for it (its
            // row; this station: what it applies).
            for n in show.protocols.networks.values() {
                let nic = if is_me {
                    st.networks.get(&n.name().unwrap_or("").to_lowercase()).cloned()
                } else {
                    row.and_then(|r| r.networks.get(&n.meta.identity.guid).cloned())
                };
                v[format!("nic:{}", n.meta.identity.guid)] = json!(nic.unwrap_or_else(|| "None".into()));
            }
            // The first network's NIC (the board's "Main → NIC").
            if let Some(first) = show.protocols.networks.values().next() {
                v["nic"] = v[format!("nic:{}", first.meta.identity.guid)].clone();
            }
            v
        })
        .collect();
    let session_members: Vec<Value> = ls
        .members
        .values()
        .map(|m| json!({"station": m.station, "ip": m.ip, "connected": m.connected, "users": m.users}))
        .collect();
    let known: Vec<Value> = show
        .session
        .stations
        .values()
        .map(|k| {
            let what = match ls.members.get(k.meta.name().unwrap_or_default()) {
                Some(m) if m.connected => "In session",
                Some(_) => "Lost",
                None if k.meta.name().unwrap_or_default().eq_ignore_ascii_case(&me) => "This station",
                None => "Known",
            };
            json!({
                "guid": k.meta.identity.guid, "object": "Station", "name": k.meta.identity.name, "address": k.ip, "state": what,
                "tone": match what { "In session" | "This station" => "ok", "Lost" => "alert", _ => "off" },
                "seen": k.joined_unix_ms, "priority": k.priority.to_string(),
                "seen_text": chrono::DateTime::from_timestamp_millis(k.joined_unix_ms as i64)
                    .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default(),
            })
        })
        .collect();
    json!({
        "station": {
            "name": st.station_name, "lumasession": st.lumasession,
            "location": st.location, "priority": st.station_priority.to_string(),
            "invite": st.invite, "online": st.online,
            "status": status,
            "master": joined,
            "show_status": state.show_status.to_string(),
            "multicast_base": if st.multicast_alternative { "Alternative" } else { "Default" },
        },
        "decision": state.decision,
        "stations": stations,
        "members": members,
        "session_members": session_members,
        "known": known,
        "networks": networks,
    })
}
