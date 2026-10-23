//! Info and Export command handlers.
//!
//! Info Fixture {id}         -- fixture attributes, DMX mapping, universe/address
//! Info FixtureType {id}     -- type details, all revisions and modes
//! Info Universe {id}        -- universe details, occupancy map, sACN status
//! Info Clock / Info Timing  -- timing diagnostics
//! Export {type} {id} "file" -- serialize to JSON

use lumacore_tree::TreeNode;
use nu_ansi_term::Color;

use crate::AppState;
use crate::errors::render_code;
use crate::text::{PropertySheet, render_channel_map_offset, render_success};
use lumacore_gdtf::ChannelMap;

/// Handle: Info <object> [<id>]
///
/// `Info <object> …` is routed to the object's own Info handler by dispatch
/// (objects publish themselves); reaching this means no object was named.
pub fn handle_info(_state: &AppState, args: &[&str]) -> String {
    let objects: Vec<&str> =
        crate::objects::supporting(crate::objects::Verb::Info)
            .map(|o| o.keyword)
            .collect();
    match args.first() {
        Some(target) => render_code(
            format!(
                "Unknown info target '{target}'. Use: {}",
                objects.join(", ")
            ),
            "E-CMD-001",
        ),
        None => render_code(format!("Info what? Info takes: {}", objects.join(", ")), "E-CMD-006"),
    }
}

pub fn info_server(state: &AppState) -> String {
    let station = &state.root.station_settings;
    let network = &state.root.runtime.network;
    let ls = &state.root.runtime.luma_sessions;
    let user_count = state.root.runtime.users.len();
    let cmdline = ls
        .listeners
        .get("Cmdline")
        .map(|listener| format!("{} {}", listener.transport, listener.port))
        .unwrap_or_else(|| "UDP ?".to_string());
    let shell = ls
        .listeners
        .get("Shell")
        .map(|listener| format!("{} {}", listener.transport, listener.port))
        .unwrap_or_else(|| "TCP ?".to_string());

    PropertySheet::new("Server")
        .field("Station", &station.station_name)
        .field("Authority", &station.station_name)
        .field("Cmdline", &cmdline)
        .field("Shell", &shell)
        .field("Users", &user_count.to_string())
        .field("Shell Clients", &ls.active_shell_clients.to_string())
        .field("Output Lines", &network.active_output_lines.to_string())
        .build()
}

/// `Info Network`: this station's data networks — interfaces, lines,
/// input sources, Art-Net nodes.
pub fn info_network(state: &AppState) -> String {
    let network = &state.root.runtime.network;
    let receiving = network
        .lines
        .iter()
        .filter(|l| l.direction == "In" && l.active)
        .count();
    PropertySheet::new("Network")
        .field("Interfaces", &network.interfaces.len().to_string())
        .field("Networks", &state.root.show_data.protocols.networks.len().to_string())
        .field(
            "Mapped here",
            &state.root.station_settings.networks.len().to_string(),
        )
        .field("Output Lines", &network.active_output_lines.to_string())
        .field("Inputs receiving", &receiving.to_string())
        .field("Input Sources", &network.sources.len().to_string())
        .field("Nodes", &network.nodes.len().to_string())
        .field(
            "Output",
            if state.root.station_settings.output_held {
                "held (Set Station \"HoldOutput\" Off)"
            } else {
                "free"
            },
        )
        .build()
}

/// `Info LumaSession`: this station's LumaSession — status, members,
/// discovery, listeners, clock.
/// The master: the linked one on a member, this station otherwise.
fn master_of(state: &AppState) -> String {
    match &state.link {
        Some(link) if link.master.is_empty() => link.target.clone(),
        Some(link) => format!("{} ({})", link.master, link.target),
        None if state.root.runtime.luma_sessions.members.is_empty() => "—".to_string(),
        None => format!("{} (this station)", state.root.station_settings.station_name),
    }
}

/// Members with who is logged in there (decisions §21 "6b").
fn members_line(ls: &lumacore_tree::LumaSessionsRuntime) -> String {
    if ls.members.is_empty() {
        return "0".to_string();
    }
    ls.members
        .values()
        .map(|m| {
            let users = if m.users.is_empty() {
                String::new()
            } else {
                format!(": {}", m.users.join(", "))
            };
            let lost = if m.connected { "" } else { " (reconnecting)" };
            format!("{}{lost}{users}", m.station)
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// The member's link: latency, silence, resends (§21 "6c").
fn link_line(state: &AppState) -> String {
    let Some(link) = &state.link else {
        return "—".to_string();
    };
    let silent = link.stats.silent_for().as_millis();
    let health = if link.stats.silent_for() >= crate::session_link::lost_after() {
        "LOST"
    } else if link.stats.silent_for() >= crate::session_link::WARN {
        "late"
    } else {
        "ok"
    };
    format!(
        "{health} · latency {} ms · last heard {silent} ms ago · {} resent",
        link.stats.rtt().as_millis(),
        link.stats.resends.load(std::sync::atomic::Ordering::Relaxed)
    )
}

/// Commands sent to the master and not answered yet.
fn pending_line(state: &AppState) -> String {
    match state.forwards.len() {
        0 => "0".to_string(),
        n => {
            let oldest = state
                .forwards
                .values()
                .map(|f| f.sent.elapsed().as_millis())
                .max()
                .unwrap_or(0);
            format!("{n} (oldest {oldest} ms)")
        }
    }
}

pub fn info_lumasession(state: &AppState) -> String {
    let ls = &state.root.runtime.luma_sessions;
    let station = &state.root.station_settings;
    let health = &ls.health;
    PropertySheet::new("LumaSession")
        .field("Name", &station.lumasession)
        .field("Station", &station.station_name)
        .field("Location", if station.location.is_empty() { "—" } else { &station.location })
        .field("Status", crate::session_link::status_word(state))
        .field("Priority", &station.station_priority.to_string())
        .field("Invite", if station.invite { "Yes" } else { "No" })
        .field("Online", if station.online { "Yes" } else { "No (localhost only)" })
        .field("Master", &master_of(state))
        .field("Members", &members_line(ls))
        .field("Applied", &state.last_applied.map_or("—".to_string(), |(_, seq)| seq.to_string()))
        .field("Pending", &pending_line(state))
        .field("Link", &link_line(state))
        .field("Epoch", &state.session_epoch.to_string())
        .field("Show Status", &state.show_status.to_string())
        .field("Output", if state.output_plan.is_empty() { "—" } else { &state.output_plan })
        .field("Decision", &decision_text(state))
        .field("Discovered", &ls.discovery.len().saturating_sub(1).to_string())
        .field("Listeners", &ls.listeners.len().to_string())
        .field("Connections", &ls.connections.len().to_string())
        .field(
            "State",
            if health.state.is_empty() {
                "idle"
            } else {
                &health.state
            },
        )
        .field("Last Event", health.last_event.as_deref().unwrap_or("—"))
        .build()
}

pub fn info_session(state: &AppState, id_arg: Option<&str>) -> String {
    let session = match resolve_session(state, id_arg) {
        Ok(session) => session,
        Err(err) => return render_code(&err, "E-CMD-007"),
    };

    let mut sheet = PropertySheet::new(&format!("Session {}", session.user))
        .field("ID", &session.meta.guid.to_string())
        .field("User", &session.user)
        .field("Authority", &session.authority)
        .field("Attached", if session.attached { "yes" } else { "no" })
        .field("Stations", &session.stations.len().to_string())
        .field("Clients", &session.clients.len().to_string())
        .field("Watches", &session.watches.len().to_string());

    // Show station details if any
    if !session.stations.is_empty() {
        sheet = sheet.separator().section("Stations");
        for station in session.stations.values() {
            sheet = sheet.line(&format!(
                "    {} [{}] {} ({})",
                station.name, station.status, station.priority, station.role
            ));
        }
    }

    sheet.build()
}

/// A station's row of the station table (decisions §30 "12d part 4"):
/// its NIC per network and its interfaces' services.
fn wiring_sheet(state: &AppState, name: &str) -> Option<String> {
    let id_arg = name.trim_matches('"');
    let me = state.root.station_settings.station_name.eq_ignore_ascii_case(id_arg);
    let n = if me { crate::stations::own_row(state) } else { crate::stations::row_named(state, id_arg) };
    let nets = crate::stations::network_names(state);
    let (title, networks, services, row) = match n.and_then(|n| state.root.show_data.stations.get(&n).map(|r| (n, r))) {
        Some((no, r)) => (
            r.meta.name().unwrap_or_default().to_string(),
            r.networks.iter().map(|(g, nic)| (nets.get(g).map_or_else(|| format!("{g} (gone)"), |(_, n)| n.clone()), nic.clone())).collect::<Vec<_>>(),
            r.services.clone(),
            format!("{no} · {}", if r.station_id.is_empty() { "made in advance" } else { "claimed" }),
        ),
        // No row (yet): this station's last-applied wiring.
        None if me => {
            let st = &state.root.station_settings;
            (st.station_name.clone(), st.networks.iter().map(|(k, v)| (k.clone(), v.clone())).collect(), st.services.clone(), "none yet (its own copy)".into())
        }
        None => return None,
    };
    let mut sheet = PropertySheet::new(&format!("Station {title} · wiring")).field("Table row", &row);
    for (net, nic) in networks {
        sheet = sheet.field(&format!("Network {net}"), &nic);
    }
    for (nic, sv) in services {
        let yn = |b: bool| if b { "Yes" } else { "No" };
        sheet = sheet.field(&format!("Interface {nic}"), &format!("LumaSession {} · Shell {} · HTTP {}", yn(sv.lumasession), yn(sv.shell), yn(sv.http)));
    }
    Some(sheet.build())
}

pub fn info_station(state: &AppState, id_arg: Option<&str>) -> String {
    let name = id_arg.map_or_else(|| state.root.station_settings.station_name.clone(), |a| a.trim_matches('"').to_string());
    let wiring = wiring_sheet(state, &name);
    let station = match resolve_station(state, Some(&name)) {
        Ok(station) => station,
        Err(err) => return wiring.unwrap_or_else(|| render_code(&err, "E-CMD-007")),
    };

    let sheet = PropertySheet::new(&format!("Station {}", station.name))
        .field("Name", &station.name)
        .field("Role", &station.role)
        .field("Endpoint", &station.endpoint)
        .field("Connected", if station.connected { "yes" } else { "no" })
        .field("Priority", &station.priority.to_string())
        .field("Status", &station.status.to_string())
        .field(
            "AuthorityState",
            if station.authority_state.is_empty() {
                "—"
            } else {
                &station.authority_state
            },
        )
        .field(
            "SyncState",
            if station.sync_state.is_empty() {
                "—"
            } else {
                &station.sync_state
            },
        )
        .build();
    match wiring {
        Some(w) => format!("{sheet}\n{w}"),
        None => sheet,
    }
}

pub fn info_client(state: &AppState, id_arg: Option<&str>) -> String {
    let client = match resolve_client(state, id_arg) {
        Ok(client) => client,
        Err(err) => return render_code(&err, "E-CMD-007"),
    };

    PropertySheet::new(&format!("Client {}", client.id))
        .field("ID", &client.id.to_string())
        .field("Kind", &client.kind)
        .field("Remote", &client.remote_addr)
        .field("Size", &format!("{}x{}", client.width, client.height))
        .field("Connected", if client.connected { "yes" } else { "no" })
        .build()
}

fn resolve_session<'a>(
    state: &'a AppState,
    id_arg: Option<&str>,
) -> Result<&'a lumacore_tree::RuntimeUser, String> {
    let mut sessions: Vec<&lumacore_tree::RuntimeUser> =
        state.root.runtime.users.values().collect();
    sessions.sort_by(|a, b| a.user.cmp(&b.user));
    if sessions.is_empty() {
        return Err("No runtime sessions.".to_string());
    }

    let key = id_arg.unwrap_or_else(|| state.effective_user_name());
    if let Ok(index) = key.parse::<usize>() {
        return sessions
            .get(index.saturating_sub(1))
            .copied()
            .ok_or_else(|| format!("Session {index} not found."));
    }

    sessions
        .into_iter()
        .find(|session| {
            session.user.eq_ignore_ascii_case(key)
                || session.meta.guid.to_string().eq_ignore_ascii_case(key)
        })
        .ok_or_else(|| format!("Session '{key}' not found."))
}

fn resolve_station<'a>(
    state: &'a AppState,
    id_arg: Option<&str>,
) -> Result<&'a lumacore_tree::SessionStationRuntime, String> {
    let key = id_arg.unwrap_or(&state.root.station_settings.station_name);
    for session in state.root.runtime.users.values() {
        if let Some(station) = session
            .stations
            .values()
            .find(|station| station.name.eq_ignore_ascii_case(key))
        {
            return Ok(station);
        }
    }
    Err(format!("Station '{key}' not found."))
}

fn resolve_client<'a>(
    state: &'a AppState,
    id_arg: Option<&str>,
) -> Result<&'a lumacore_tree::SessionClientRuntime, String> {
    let key = id_arg.ok_or_else(|| "Usage: Info Client {id}".to_string())?;
    for session in state.root.runtime.users.values() {
        if let Some(client) = session.clients.values().find(|client| {
            client.id.to_string().eq_ignore_ascii_case(key)
                || client.remote_addr.eq_ignore_ascii_case(key)
        }) {
            return Ok(client);
        }
    }
    Err(format!("Client '{key}' not found."))
}

pub fn info_fixture(state: &AppState, id_arg: Option<&str>) -> String {
    let id: u32 = match id_arg.and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => return render_code("Usage: Info Fixture {id}", "E-CMD-006"),
    };

    let fixture = match state.root.show_data.devices.dmx.patch.fixture(id) {
        Some(f) => f,
        None => return render_code(format!("Fixture {id} not found."), "E-PATCH-006"),
    };

    let name = fixture.meta.name().unwrap_or("<unnamed>").to_string();
    let guid = fixture.meta.guid();

    let (universe_guid, start_addr, footprint) = fixture
        .primary_break()
        .map(|b| (b.universe_id, b.start_address, b.channel_count))
        .unwrap_or((uuid::Uuid::nil(), 0, 0));
    let universe_int_id = state
        .root
        .show_data
        .universe_id_for_guid(universe_guid)
        .unwrap_or(0);

    let location = format!(
        "Universe {} Addr {:03}–{:03}",
        Color::Cyan.paint(universe_int_id.to_string()),
        start_addr,
        start_addr + footprint.saturating_sub(1)
    );

    let mut sheet = PropertySheet::new(&format!("Fixture {id}"))
        .field("ID", &Color::Cyan.paint(id.to_string()).to_string())
        .field("Name", &name)
        .field("GUID", &guid.to_string())
        .field("Location", &location);

    // Find fixture type for channel map
    if let Some(ft) = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture_types
        .values()
        .find(|ft| ft.meta.guid() == fixture.mvr.fixture_type)
    {
        let ft_name = ft.meta.name().unwrap_or("<unnamed>").to_string();
        sheet = sheet.field("Type", &format!("{} ({})", ft_name, ft.manufacturer));

        if let Some((_, mode)) = ft.mode_of(fixture.mvr.revision, fixture.mvr.mode) {
            let mode_name = mode.meta.name().unwrap_or("<unnamed>").to_string();
            sheet = sheet.field("Mode", &mode_name).separator();

            let channels = ChannelMap::by_dmx_offset(mode);
            let owned: Vec<lumacore_fixtures::ChannelFunction> =
                channels.iter().map(|cf| (*cf).clone()).collect();
            let channel_output = render_channel_map_offset(&mode_name, &owned);

            // Programmer values section
            let programmer_output = build_programmer_section(state, id, &owned);

            return sheet.build() + &channel_output + &programmer_output;
        }
    }

    sheet.build()
}

/// A fixture's facts beside its properties (the NAV inspector's info rows, decisions 0291):
/// its type, its mode and where it is patched — what `Info Fixture` heads its sheet with.
pub fn fixture_facts(state: &AppState, id: u32) -> Vec<(&'static str, String)> {
    let patch = &state.root.show_data.devices.dmx.patch;
    let Some(fixture) = patch.fixture(id) else {
        return Vec::new();
    };
    let mut facts = Vec::new();
    if let Some(ft) = patch.fixture_types.values().find(|ft| ft.meta.guid() == fixture.mvr.fixture_type) {
        facts.push(("Type", format!("{} ({})", ft.meta.name().unwrap_or("<unnamed>"), ft.manufacturer)));
        if let Some((_, mode)) = ft.mode_of(fixture.mvr.revision, fixture.mvr.mode) {
            facts.push(("Mode", mode.meta.name().unwrap_or("<unnamed>").to_string()));
        }
    }
    let location = match fixture.primary_break() {
        Some(b) => format!(
            "Universe {} Addr {:03}–{:03}",
            state.root.show_data.universe_id_for_guid(b.universe_id).unwrap_or(0),
            b.start_address,
            b.start_address + b.channel_count.saturating_sub(1)
        ),
        None => "not patched".to_string(),
    };
    facts.push(("Location", location));
    facts
}

/// Build a section showing current programmer values for a fixture.
fn build_programmer_section(
    state: &AppState,
    fixture_id: u32,
    channels: &[lumacore_fixtures::ChannelFunction],
) -> String {
    // Get programmer for current user
    let Some(programmer) = state.current_programmer() else {
        return String::new();
    };

    // Resolve fixture ID to GUID for programmer lookup
    let fixture_guid = match state.root.show_data.fixture_guid(fixture_id) {
        Some(g) => g,
        None => return String::new(),
    };

    // Collect programmer values for this fixture (active shown bold, inactive dimmed)
    let mut value_lines: Vec<String> = Vec::new();
    for cf in channels {
        let key = (fixture_guid, cf.attribute_name.clone());
        if let Some(pv) = programmer.values.get(&key) {
            let readout = pv.value.readout();
            if pv.active {
                value_lines.push(format!(
                    "    {} {}",
                    Color::Cyan.paint(format!("{:<20}", cf.attribute_name)),
                    Color::Green.paint(format!("{readout:>5}")),
                ));
            } else {
                // Inactive values shown dimmed
                value_lines.push(format!(
                    "    {}",
                    nu_ansi_term::Style::new()
                        .dimmed()
                        .paint(format!("{:<20} {readout:>5} [inactive]", cf.attribute_name))
                ));
            }
        }
    }

    if value_lines.is_empty() {
        return String::new();
    }

    let mut sheet = crate::text::PropertySheet::new("")
        .separator()
        .section("Programmer Values");
    for line in &value_lines {
        sheet = sheet.line(line);
    }
    sheet.build()
}

pub fn info_fixture_type(state: &AppState, id_arg: Option<&str>) -> String {
    let id: u32 = match id_arg.and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => return render_code("Usage: Info FixtureType {id}", "E-CMD-006"),
    };

    let ft = match state.root.show_data.devices.dmx.patch.fixture_types.get(&id) {
        Some(ft) => ft,
        None => return render_code(format!("Fixture type {id} not found."), "E-PATCH-005"),
    };

    let name = ft.meta.name().unwrap_or("<unnamed>").to_string();

    let mut sheet = PropertySheet::new("FixtureType")
        .field("ID", &Color::Cyan.paint(id.to_string()).to_string())
        .field("Manufacturer", &ft.manufacturer)
        .field("Name", &name)
        .field("GUID", &ft.meta.guid().to_string());

    for (ri, rev) in ft.revisions.iter().enumerate() {
        let rev_name = rev.meta.name().unwrap_or("<unnamed>").to_string();
        sheet = sheet
            .separator()
            .section(&format!("Revision: {} (index {ri})", rev_name));

        for mode in &rev.modes {
            let mode_name = mode.meta.name().unwrap_or("<unnamed>").to_string();
            sheet = sheet.line(&format!(
                "    {} — {} ch",
                Color::Cyan.paint(&mode_name),
                mode.dmx_footprint
            ));

            let channels = ChannelMap::by_dmx_offset(mode);
            for cf in &channels {
                let offset_str = if cf.is_virtual {
                    Color::Yellow.paint("virt").to_string()
                } else {
                    Color::Cyan
                        .paint(format!("{:03}", cf.dmx_offset + 1))
                        .to_string()
                };
                sheet = sheet.line(&format!(
                    "      [{}] {} ({}b){}",
                    offset_str,
                    cf.attribute_name,
                    cf.byte_count,
                    if cf.is_virtual { " [virtual]" } else { "" }
                ));
            }
        }
    }

    sheet.build()
}

pub fn info_universe(state: &AppState, id_arg: Option<&str>) -> String {
    let id: u32 = match id_arg.and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => return render_code("Usage: Info Universe {id}", "E-CMD-006"),
    };

    let universe = match state.root.show_data.devices.dmx.patch.universes.get(&id) {
        Some(u) => u,
        None => return render_code(format!("Universe {id} not found."), "E-UNIVERSE-001"),
    };

    let name = universe.meta.name().unwrap_or("<unnamed>").to_string();
    let occupied = universe.occupied_count();
    let pct = if occupied > 0 {
        ((occupied as u32) * 100) / 512
    } else {
        0
    };

    let routes_info = {
        let routes: Vec<String> = lumacore_tree::LineOps::lines_on(&state.root.show_data, id)
            .iter()
            .map(|(p, no, mode, w)| crate::commands::list::route_label(p, *no, *mode, *w))
            .collect();
        if routes.is_empty() {
            Color::Yellow.paint("not assigned").to_string()
        } else {
            Color::Green.paint(routes.join(", ")).to_string()
        }
    };

    let mut sheet = PropertySheet::new(&format!("Universe {id}"))
        .field("Name", &name)
        .field("Lines", &routes_info)
        .field("Merge Mode", universe.merge_mode.label())
        .field("Usage", &format!("{occupied}/512 slots"))
        .bar("Map", pct, 32);

    // List fixtures in this universe
    let uid_guid_for_filter = state.root.show_data.universe_guid(id);
    let fixtures_here: Vec<String> = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixtures
        .iter()
        .filter(|(_, f)| {
            uid_guid_for_filter
                .map(|guid| {
                    f.primary_break()
                        .map(|b| b.universe_id == guid)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
        .map(|(_, f)| {
            let b = f.primary_break().unwrap();
            let fname = f.meta.name().unwrap_or("<unnamed>").to_string();
            format!(
                "    [{:03}–{:03}] {} {}",
                b.start_address,
                b.start_address + b.channel_count.saturating_sub(1),
                f.handle(),
                fname
            )
        })
        .collect();

    if !fixtures_here.is_empty() {
        sheet = sheet.separator().section("Fixtures");
        for line in &fixtures_here {
            sheet = sheet.line(line);
        }
    } else {
        sheet = sheet.separator().line(&format!(
            "  {}",
            Color::Yellow.paint("No fixtures in this universe.")
        ));
    }

    sheet.build()
}

pub fn info_group(state: &AppState, id_arg: Option<&str>) -> String {
    let id: u32 = match id_arg.and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => return render_code("Usage: Info Group {id}", "E-CMD-006"),
    };

    crate::commands::groups::property_sheet(state, id)
}

/// `Info Clock`: the session clock (decisions §24 "Step 9 hardening") —
/// local and session time, the offset to the master and how the stream
/// heartbeat measured it, and linuxptp when it runs.
pub fn info_clock(state: &AppState) -> String {
    let c = state.root.runtime.health.clock_view();
    let local = lumacore_base::clock::local_us();
    let session = lumacore_base::clock::session_us();
    let lt = crate::commands::list::local_time;
    let mut sheet = PropertySheet::new("Clock")
        .field(
            "Role",
            &if c.role == "Member" {
                format!("Member of {} (epoch {})", c.master, c.epoch)
            } else {
                c.role.to_string()
            },
        )
        .field("State", c.state)
        .field("Session time", &format!("{} (+{} µs)", lt(session / 1000), session % 1000))
        .field("Local time", &format!("{} (+{} µs)", lt(local / 1000), local % 1000))
        .field("offsetFromMaster", &format!("{:+} µs", c.offset_us))
        .field("meanPathDelay", &format!("{} µs", c.path_delay_us))
        .field("Jitter", &format!("{} µs", c.jitter_us))
        .field(
            "Frequency",
            &format!("{:+.3} ppm (skew {:.3} ppm)", c.frequency_ppb as f64 / 1000.0, c.skew_ppb as f64 / 1000.0),
        )
        .field("Correction", &format!("{:+.1} ppm", c.correction_ppb as f64 / 1000.0))
        .field("Samples", &format!("{} used · {} rejected", c.samples_used, c.samples_rejected))
        .field(
            "Last sync",
            &if c.last_sync_ms == 0 { "—".to_string() } else { lt(c.last_sync_ms) },
        )
        .field(
            "Steps",
            &if c.steps == 0 {
                "none".to_string()
            } else {
                format!("{} (last {:+} µs at {})", c.steps, c.last_step_us, lt(c.last_step_ms))
            },
        );
    for m in &c.members {
        sheet = sheet.field(
            &format!("Member {}", m.station),
            &format!(
                "{} · offset {:+} µs · path delay {} µs · jitter {} µs · {:+.3} ppm",
                m.state,
                m.offset_us,
                m.path_delay_us,
                m.jitter_us,
                m.frequency_ppb as f64 / 1000.0
            ),
        );
    }
    sheet = match (&c.ptp, &c.ptp_note) {
        (Some(p), _) => sheet
            .section("PTP (linuxptp)")
            .field("Port state", &p.port_state)
            .field("Offset", &format!("{:+} ns", p.offset_ns))
            .field("Path delay", &format!("{} ns", p.path_delay_ns))
            .field("Grandmaster", &format!("{} ({})", p.grandmaster, if p.gm_present { "present" } else { "absent" }))
            .field("Steps removed", &p.steps_removed.to_string()),
        (None, Some(note)) => sheet.field("PTP", note),
        (None, None) => sheet,
    };
    sheet.build()
}

/// `Info Output <n>` / `Info Input <n>`: one line's settings, routes and
/// live state; inputs add the sources heard on their network.
/// A line's protocol settings: each declared one with its value, then the ones it keeps from
/// another protocol, unused (decisions §34 "34f7b hardening" 1).
fn protocol_settings(mut sheet: PropertySheet, line: &lumacore_protocols::Line) -> PropertySheet {
    let declared = line.declared();
    for (key, value) in &declared {
        sheet = sheet.field(key, &value.to_string());
    }
    for (key, value) in &line.settings {
        if !declared.iter().any(|(k, _)| k == key) {
            sheet = sheet.field(key, &format!("{value} (unused: not a {} setting)", line.protocol));
        }
    }
    sheet
}

/// `Info ArtNet`: the protocol itself — what it is and can do, its own settings, what runs
/// (decisions §34 "34f7c hardened" 6).
pub fn info_protocol(state: &AppState, protocol: &str) -> String {
    use lumacore_protocols::protocol;
    let Some(row) = protocol::find(protocol) else {
        return render_code(format!("This build has no protocol {protocol}."), "E-CMD-007");
    };
    let directions = match (row.outputs, row.inputs) {
        (true, true) => "out + in",
        (true, false) => "out",
        _ => "in",
    };
    let caps: Vec<String> = row.capabilities.iter().map(|c| format!("{c:?}")).collect();
    let node = state.root.show_data.protocols.node(row.name);
    let wide = (row.protocol_settings)();
    let settings = if wide.is_empty() {
        "none".to_string()
    } else {
        wide.iter()
            .map(|d| format!("{} {}", d.key, node.and_then(|n| n.settings.get(&d.key)).unwrap_or(&d.default)))
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let mut sheet = PropertySheet::new(row.name)
        .field("Kind", &format!("{:?} ({})", row.kind, row.kind.group()))
        .field("Lines", &format!("{directions} · port {}", row.port))
        .field("Capabilities", &if caps.is_empty() { "—".to_string() } else { caps.join(", ") });
    if let Some(u) = row.universes {
        sheet = sheet.field("Universes", &format!("{} {}", u.property, u.range_text()));
    }
    let running = state.protocols_running.get(row.name).copied().unwrap_or(0);
    let nodes = state.root.runtime.network.nodes.iter().filter(|n| n.protocol == row.name).count();
    sheet
        .field("Settings", &settings)
        .field("Show", &format!("{} line(s)", node.map_or(0, |n| n.lines.len())))
        .field("Running", &format!("{running} output line(s) · {nodes} node(s) heard"))
        .build()
}

pub fn info_line(state: &AppState, protocol: &str, id_arg: Option<&str>) -> String {
    use lumacore_tree::{LineOps};
    use lumacore_protocols::{Direction, Kind};
    let Some(id_text) = id_arg else {
        return info_protocol(state, protocol);
    };
    let Ok(id) = id_text.parse::<u32>() else {
        return render_code(format!("Usage: Info {protocol} [<n>]"), "E-CMD-006");
    };
    let show = &state.root.show_data;
    let Some(line) = show.protocols.line(protocol, id) else {
        return render_code(format!("{protocol} {id} not found."), "E-OUTPUT-002");
    };
    let direction = line.mode;
    let routes = LineOps::routes_of(show, protocol, id);
    let line_state = crate::protocol_plan::line_states(state)
        .into_iter()
        .find(|s| s.protocol == line.protocol && s.line == id);
    let unpinned = line_state.as_ref().is_some_and(|s| s.unpinned);
    let state_label = line_state.map_or_else(String::new, |s| s.state);
    let wire = |w: u16| lumacore_protocols::wire_label(&line.protocol, w);
    let control = lumacore_protocols::protocol::find(&line.protocol).is_some_and(|p| p.kind == Kind::Data);
    let mut sheet = PropertySheet::new(&format!("{} {id}", line.protocol))
        .field("Name", line.meta.name().unwrap_or("<unnamed>"))
        .field("Mode", line.mode.label())
        .field(
            "Network",
            state.root.show_data.protocols.network_name(line.network),
        )
        .field("Enabled", if line.enabled { "yes" } else { "no" });
    sheet = protocol_settings(sheet, line);
    match (direction, control) {
        (Direction::Out, _) => {
            sheet = sheet
                .field("Via", &lumacore_protocols::mode_label(&line.protocol, line.target))
                .field(
                    "Rate",
                    &{
                        // The gap as sent (R1-54, Q21: never more than half the period).
                        let sent = crate::protocol_plan::capped_gap(line.packet_gap, line.rate, routes.len());
                        let gap = if sent < line.packet_gap {
                            format!("{} µs (capped to {sent} µs: half the period)", line.packet_gap)
                        } else {
                            format!("{} µs", line.packet_gap)
                        };
                        format!(
                            "{} Hz · {} · gap {gap} · sync {}",
                            line.rate,
                            line.send.label(),
                            if line.sync { "on" } else { "off" }
                        )
                    },
                )
                .field(
                    "Universes",
                    &routes
                        .iter()
                        .map(|(u, w)| format!("{u}→{}", wire(*w)))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
        }
        (Direction::In, true) => {
            sheet = sheet
                .field(
                    "User",
                    line.user
                        .and_then(|u| state.root.show_data.user_name(u))
                        .unwrap_or("—"),
                );
        }
        (Direction::In, false) => {
            let modes: std::collections::BTreeSet<&str> = routes
                .iter()
                .filter_map(|(u, _)| show.devices.dmx.patch.universes.get(u))
                .map(|u| u.merge_mode.label())
                .collect();
            let merge = match modes.len() {
                0 => "—".to_string(),
                1 => modes.into_iter().next().unwrap_or_default().to_string(),
                _ => "…".to_string(),
            };
            sheet = sheet
                .field("Input Priority", line.input_priority.label())
                .field("Merge Mode", &merge)
                .field(
                    "Accept",
                    &line
                        .target
                        .map_or("any source".to_string(), |ip| format!("only {ip}")),
                )
                .field(
                    "Universes",
                    &routes
                        .iter()
                        .map(|(u, w)| format!("{}→{u}", wire(*w)))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            let wires: Vec<u16> = routes.iter().map(|(_, w)| *w).collect();
            let sources: Vec<String> = state
                .root
                .runtime
                .network
                .sources
                .iter()
                .filter(|s| {
                    s.protocol == line.protocol
                        && s.network.eq_ignore_ascii_case(
                            state.root.show_data.protocols.network_name(line.network),
                        )
                        && s.universes.iter().any(|u| wires.contains(u))
                })
                .map(|s| {
                    format!(
                        "\"{}\" {} prio {} universes {}",
                        s.name,
                        s.ip,
                        s.priority,
                        s.universes
                            .iter()
                            .map(|u| wire(*u))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect();
            sheet = sheet.field(
                "Sources",
                &if sources.is_empty() {
                    "none heard".to_string()
                } else {
                    sources.join("; ")
                },
            );
        }
    }
    sheet = sheet.field("State", &state_label);
    if unpinned {
        sheet = sheet.field(
            "Socket",
            "not pinned: unicast may be lost if another program uses the port",
        );
    }
    sheet.build()
}

/// Info Show "name" — read a .luma show file from disk and display stats.
/// The pending LumaSession decision: its answers, the default with its
/// countdown, and what Merge would do (§21 "6d").
fn decision_text(state: &AppState) -> String {
    let Some(d) = &state.decision else {
        return "—".to_string();
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_millis() as u64);
    let when = match (d.deadline_ms, d.paused) {
        (Some(t), _) => format!("{} in {} s", d.default, t.saturating_sub(now) / 1000),
        (None, true) => format!("{} (paused)", d.default),
        (None, false) => format!("{} (waits)", d.default),
    };
    let mut out = format!("{} — default {when}", d.title);
    for (label, command) in &d.options {
        if command.is_empty() {
            out.push_str(&format!("\n    {label}: not possible ({})", d.merge_blocked.as_deref().unwrap_or("?")));
        } else {
            out.push_str(&format!("\n    {label}: {command}"));
        }
    }
    for line in d.details.iter().take(12) {
        out.push_str(&format!("\n    · {line}"));
    }
    out
}

/// `Info Show`: the loaded show — identity, format, references, merge.
pub fn info_loaded_show(state: &AppState) -> String {
    let s = &state.root.show_data.show_settings;
    PropertySheet::new("Show")
        .field("Name", state.current_show.as_deref().unwrap_or("untitled"))
        .field("Show Id", &s.show_id.to_string())
        .field(
            "Format",
            &format!("{} (saved by {})", s.format_version, s.saved_with_commit),
        )
        .field("Status", &state.show_status.to_string())
        .field("References", &crate::references::summary(state))
        .field("Merge Default", s.merge_default.word())
        .field(
            "Merge Timeout",
            &match s.merge_timeout {
                None => "Unlimited".to_string(),
                Some(0) => "0 (no prompt)".to_string(),
                Some(n) => format!("{n} s"),
            },
        )
        .build()
}

pub fn info_show_file(name_arg: Option<&str>) -> String {
    let name = match name_arg {
        Some(n) => n,
        None => return render_code(
            "Usage: Info Show \"name\"\nExample: Info Show \"demo\"",
            "E-CMD-006",
        ),
    };

    let name_clean = name.trim_end_matches(".luma");
    let path = format!("library/shows/{name_clean}.luma");

    let contents = match lumacore_luma::drive::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| lumacore_luma::show_file::decode(&bytes).map(|c| (bytes.len(), c.json)))
    {
        Ok(c) => c,
        Err(e) => return render_code(format!("Cannot read '{path}': {e}"), "E-CMD-012"),
    };

    let (file_size, contents) = contents;
    let sd: lumacore_tree::ShowData = match serde_json::from_str(&contents) {
        Ok(sd) => sd,
        Err(e) => return render_code(format!("Invalid show file '{path}': {e}"), "E-CMD-012"),
    };

    let size_str = if file_size > 1024 * 1024 {
        format!("{:.1} MB", file_size as f64 / (1024.0 * 1024.0))
    } else if file_size > 1024 {
        format!("{:.1} KB", file_size as f64 / 1024.0)
    } else {
        format!("{file_size} B")
    };

    let show_name = if sd.show_settings.show_name.is_empty() {
        "untitled"
    } else {
        &sd.show_settings.show_name
    };

    PropertySheet::new(&format!("Show: {name_clean}"))
        .field("File", &path)
        .field("Size", &size_str)
        .field("Name", show_name)
        .field(
            "Description",
            if sd.show_settings.description.is_empty() {
                "—"
            } else {
                &sd.show_settings.description
            },
        )
        .field("Fixtures", &{
            let folders = sd.devices.dmx.patch.fixtures.values().filter(|f| f.mvr.fid.is_none()).count();
            let n = sd.devices.dmx.patch.fids().len();
            if folders > 0 { format!("{n} ({folders} folders)") } else { n.to_string() }
        })
        .field(
            "FixtureTypes",
            &sd.devices.dmx.patch.fixture_types.len().to_string(),
        )
        .field("Universes", &sd.devices.dmx.patch.universes.len().to_string())
        .field("Lines", &sd.protocols.all_lines().count().to_string())
        .field("Users", &sd.user_profiles.len().to_string())
        .build()
}

pub fn info_userprofile(state: &AppState, id_arg: Option<&str>) -> String {
    // Bare = current user; with ID = specific profile
    let profile = match id_arg.and_then(|s| s.parse::<u32>().ok()) {
        Some(id) => state.root.show_data.user_profiles.get(&id).map(|p| (id, p)),
        None => {
            let user = state.effective_user_name();
            state
                .root
                .show_data
                .profile_id_for_user(user)
                .and_then(|id| state.root.show_data.user_profiles.get(&id).map(|p| (id, p)))
        }
    };

    let (id, p) = match profile {
        Some(pair) => pair,
        None => {
            return render_code(
                format!(
                    "UserProfile not found. {} profiles exist.",
                    state.root.show_data.user_profiles.len()
                ),
                "E-CMD-011",
            );
        }
    };

    let name = p.name().unwrap_or("<unnamed>").to_string();

    let sheet = PropertySheet::new(&format!("UserProfile {id}"))
        .field("ID", &Color::Cyan.paint(id.to_string()).to_string())
        .field("Name", &name)
        .field("GUID", &p.meta.guid().to_string())
        .field("Locked", if p.locked { "yes (template for new profiles)" } else { "no" })
        .field("AliasDepth", &p.settings.alias_depth)
        .field("ProgLang", &p.settings.prog_lang)
        .field("ClearBuffer", &p.settings.clear_buffer.to_string())
        .field("ShowDescriptions", &p.settings.show_descriptions.to_string())
        .field("Verbosity", &format!("{:?}", p.settings.verbosity))
        .field("KeyBindings", &format!("{} custom", p.content.keybindings.len()));

    sheet.build()
}

/// Handle: Export {type} {id} "filename.json"
/// Where `Export … "file"` writes.
const EXPORT_DIR: &str = "library/export";

pub fn handle_export(state: &AppState, args: &[&str]) -> String {
    if args.len() < 3 {
        return render_code("Usage: Export {type} {id} \"filename.json\"", "E-CMD-006");
    }

    let obj_type = args[0].to_lowercase();
    let id: u32 = match args[1].parse() {
        Ok(id) => id,
        Err(_) => return render_code(
            format!("Invalid ID '{}'. Expected a number.", args[1]),
            "E-CMD-008",
        ),
    };
    // A file in library/export/ (sub-folders allowed; issue 07).
    let path = match lumacore_luma::drive::library_file(EXPORT_DIR, args[2]) {
        Ok(path) => path,
        Err(e) => return render_code(e, if args[2].starts_with('/') { "E-CMD-006" } else { "E-CMD-008" }),
    };
    let filename = path.display().to_string();

    let json = match obj_type.as_str() {
        "fixture" => match state.root.show_data.devices.dmx.patch.fixture(id) {
            Some(f) => serde_json::to_string_pretty(f).ok(),
            None => return render_code(format!("Fixture {id} not found."), "E-PATCH-006"),
        },
        "fixturetype" => match state.root.show_data.devices.dmx.patch.fixture_types.get(&id) {
            Some(ft) => serde_json::to_string_pretty(ft).ok(),
            None => return render_code(format!("Fixture type {id} not found."), "E-PATCH-005"),
        },
        "universe" => match state.root.show_data.devices.dmx.patch.universes.get(&id) {
            Some(u) => serde_json::to_string_pretty(u).ok(),
            None => return render_code(format!("Universe {id} not found."), "E-UNIVERSE-001"),
        },
        _ => {
            return render_code(
                format!(
                    "Cannot export '{obj_type}'. Use: Fixture, FixtureType, Universe"
                ),
                "E-CMD-004",
            );
        }
    };

    match json {
        Some(data) => match path
            .parent()
            .map_or(Ok(()), lumacore_luma::drive::create_dir_all)
            .and_then(|_| lumacore_luma::drive::write_atomic(&path, &data))
        {
            Ok(()) => render_success(&format!("Exported to '{filename}' ({} bytes)", data.len())),
            Err(e) => render_code(format!("Write failed: {e}"), "E-CMD-012"),
        },
        None => render_code("Serialization failed.", "E-CMD-012"),
    }
}
