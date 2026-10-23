//! What should run for the show's protocol lines on this station (decisions
//! §11 "Step 4b/4c hardening", §34 "34f7c hardened"): a pure plan of the show's lines,
//! this station's network mapping and its session role. `List Output`,
//! `Info Output` and the line states read it here; `lumacore-lines` starts the
//! drivers and listeners it names (decisions §23 "Step 8": the plan is core,
//! the runtimes are not).

use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use lumacore_net::nics::{self, Nic};
use lumacore_protocols::{InputPlan, LinePlan};
use lumacore_tree::{LineOps, LineRuntime, TreeNode};
use lumacore_protocols::protocol::{self, Kind};
use lumacore_protocols::Direction;

use crate::AppState;

/// Why a line does or doesn't work.
#[derive(Debug, Clone, PartialEq)]
pub enum LineState {
    Sending {
        universes: usize,
        nic: String,
    },
    /// OSC input.
    Listening {
        nic: String,
        port: u16,
    },
    /// DMX input: `live` of `universes` have a source right now.
    Receiving {
        universes: usize,
        live: usize,
        nic: String,
    },
    Disabled,
    NoRoutes,
    /// The station hasn't mapped the line's network.
    NotMapped,
    /// The mapping names an interface this machine doesn't have.
    NoInterface(String),
    /// In a session and not its master.
    NotMaster,
    /// Output held after leaving a LumaSession (or LumaSession off).
    Held,
    /// LumaSession (networking) is off on this station.
    Offline,
    /// An OSC line without a valid user.
    NoUser,
    Error(String),
}

impl LineState {
    pub fn label(&self, network: &str) -> String {
        match self {
            Self::Sending { universes, nic } => format!(
                "sending {universes} universe{} via {nic}",
                if *universes == 1 { "" } else { "s" }
            ),
            Self::Listening { nic, port } => format!("listening on {nic}:{port}"),
            Self::Receiving {
                universes,
                live,
                nic,
            } => {
                if live == universes {
                    format!("receiving {live}/{universes} via {nic}")
                } else if *live == 0 {
                    format!("no source (0/{universes}) — listening via {nic}")
                } else {
                    format!(
                        "receiving {live}/{universes} via {nic} — {} without a source",
                        universes - live
                    )
                }
            }
            Self::Disabled => "disabled".to_string(),
            Self::NoRoutes => "no universes assigned".to_string(),
            Self::NotMapped => format!(
                "Network \"{network}\" not mapped on this station — Set Network \"{network}\" \"Interface\" <nic>"
            ),
            Self::NoInterface(spec) => format!("no interface '{spec}' on this station"),
            Self::NotMaster => "silent: this station isn't the LumaSession master".to_string(),
            Self::Held => "silent: output held — Set Station \"HoldOutput\" Off".to_string(),
            Self::Offline => "silent: LumaSession is off on this station".to_string(),
            Self::NoUser => "no user: an OSC line runs as the user it names — Set Input <n> \"User\" <name> [E-RIGHTS-003]".to_string(),
            Self::Error(e) => format!("error: {e}"),
        }
    }

    /// The line works (sending, listening or receiving from a source).
    pub fn active(&self) -> bool {
        match self {
            Self::Sending { .. } | Self::Listening { .. } => true,
            Self::Receiving { live, .. } => *live > 0,
            _ => false,
        }
    }

    fn problem(&self) -> bool {
        matches!(
            self,
            Self::NotMapped | Self::NoInterface(_) | Self::NoUser | Self::Error(_)
        )
    }
}

#[derive(Debug, Clone)]
pub struct LineStatus {
    pub direction: Direction,
    pub line: u32,
    pub name: String,
    pub protocol: String,
    pub network: String,
    pub state: LineState,
}

impl LineStatus {
    pub fn runtime(&self) -> LineRuntime {
        LineRuntime {
            direction: match self.direction {
                Direction::Out => "Out",
                Direction::In => "In",
            }
            .to_string(),
            line: self.line,
            name: self.name.clone(),
            protocol: self.protocol.clone(),
            network: self.network.clone(),
            state: self.state.label(&self.network),
            problem: self.state.problem(),
            active: self.state.active(),
            unpinned: false,
        }
    }
}

/// What one driver (protocol, NIC) does.
#[derive(Debug, Clone, PartialEq)]
pub struct DriverPlan {
    pub nic: Nic,
    /// The network the NIC is mapped for (status).
    pub network: String,
    pub outputs: Vec<LinePlan>,
    pub inputs: Vec<InputPlan>,
}

/// A Control listener (OSC): the protocol row, where it listens and the user its lines run as.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlPlan {
    pub protocol: String,
    pub nic: Ipv4Addr,
    pub port: u16,
    pub user: String,
}

/// What should run.
#[derive(Debug, Default)]
pub struct Desired {
    /// Dmx drivers by (protocol row name, NIC address).
    pub drivers: BTreeMap<(String, Ipv4Addr), DriverPlan>,
    /// Data listeners (OSC) by (protocol, line).
    pub control: BTreeMap<(String, u32), ControlPlan>,
    pub status: Vec<LineStatus>,
}

/// What this station sends, in words.
pub fn distribution(state: &AppState, d: &Desired) -> String {
    let station = &state.root.station_settings.station_name;
    if let Some(why) = silence(state) {
        return format!("{station} sends nothing ({})", why.label(""));
    }
    if d.drivers.is_empty() {
        return format!("{station} sends nothing (no line on a mapped network)");
    }
    let parts: Vec<String> = d
        .drivers
        .iter()
        .map(|((protocol, _), plan)| {
            let universes: usize = plan.outputs.iter().map(|o| o.routes.len()).sum();
            format!(
                "{} on {} {} ({} line(s), {} universe(s) out, {} input line(s))",
                protocol,
                plan.nic.name,
                plan.nic.ip,
                plan.outputs.len(),
                universes,
                plan.inputs.len()
            )
        })
        .collect();
    format!("{station} sends {}", parts.join("; "))
}

/// Whether this station sends output and takes in input: standalone (never
/// joined a session) or the session's master (§10, §11), with networking on
/// and its output not held.
pub fn may_send(state: &AppState) -> bool {
    silence(state).is_none()
}

/// Why this station doesn't send, if it doesn't.
fn silence(state: &AppState) -> Option<LineState> {
    let st = &state.root.station_settings;
    if !st.online {
        Some(LineState::Offline)
    } else if st.output_held {
        Some(LineState::Held)
    } else if state.link.is_some()
        || !(state.root.runtime.joined_sessions.is_empty() || state.is_authority)
    {
        Some(LineState::NotMaster)
    } else {
        // §21 "6c": a fenced master never sends; an IdleMaster only with
        // "output when alone".
        match &state.session_role {
            crate::session_link::Role::Fenced { .. } => Some(LineState::NotMaster),
            crate::session_link::Role::IdleMaster { .. } if !st.output_when_alone => {
                Some(LineState::NotMaster)
            }
            _ => None,
        }
    }
}

/// The NIC a network uses on this station.
fn nic_for<'a>(state: &AppState, network: &str, nics: &'a [Nic]) -> Result<&'a Nic, LineState> {
    let spec = state
        .root
        .station_settings
        .networks
        .get(&network.to_lowercase())
        .ok_or(LineState::NotMapped)?;
    nics::resolve(spec, nics).ok_or_else(|| LineState::NoInterface(spec.clone()))
}

fn nic_label(nic: &Nic) -> String {
    format!("{} {}", nic.name, nic.ip)
}

/// What should run for this show, station and session role.
pub fn desired(state: &AppState, nics: &[Nic]) -> Desired {
    let show = &state.root.show_data;
    let silent = silence(state);
    let mut d = Desired::default();
    {
        for (protocol_name, id, line) in show.protocols.all_lines() {
            let direction = line.mode;
            let routes = LineOps::routes_of(show, protocol_name, id);
            let network = show.protocols.network_name(line.network);
            let st = if !line.enabled {
                LineState::Disabled
            } else if protocol::find(&line.protocol).is_some_and(|p| p.kind == Kind::Data) {
                control_state(state, &mut d, direction, id, line, nics, silent.as_ref())
            } else if routes.is_empty() {
                LineState::NoRoutes
            } else {
                match nic_for(state, network, nics) {
                    Err(st) => st,
                    Ok(_) if silent.is_some() => silent.clone().unwrap_or(LineState::NotMaster),
                    Ok(nic) => {
                        let plan = d.drivers.entry((line.protocol.clone(), nic.ip)).or_insert_with(|| {
                            DriverPlan {
                                nic: nic.clone(),
                                network: network.to_string(),
                                outputs: Vec::new(),
                                inputs: Vec::new(),
                            }
                        });
                        match direction {
                            Direction::Out => {
                                plan.outputs.push(LinePlan {
                                    line: id,
                                    unicast: line.target,
                                    priority: line.setting_u64("Priority").and_then(|p| u8::try_from(p).ok()).unwrap_or(100),
                                    routes: routes.clone(),
                                    rate_hz: line.rate.clamp(1, 60),
                                    send: line.send,
                                    packet_gap_us: capped_gap(line.packet_gap, line.rate, routes.len()),
                                    sync: line.sync,
                                });
                                LineState::Sending {
                                    universes: routes.len(),
                                    nic: nic_label(nic),
                                }
                            }
                            Direction::In => {
                                plan.inputs.push(InputPlan {
                                    line: id,
                                    source: line.target,
                                    band: line.input_priority,
                                    routes: routes.clone(),
                                });
                                LineState::Receiving {
                                    universes: routes.len(),
                                    live: 0,
                                    nic: nic_label(nic),
                                }
                            }
                        }
                    }
                }
            };
            d.status.push(LineStatus {
                direction,
                line: id,
                name: line.name().unwrap_or("").to_string(),
                protocol: line.protocol.clone(),
                network: network.to_string(),
                state: st,
            });
        }
    }
    for plan in d.drivers.values_mut() {
        cap_driver_gaps(plan);
    }
    d
}

/// An OSC input line follows the rule of every other input: it listens only
/// on the station that sends (standalone or the session's master,
/// [`silence`]), so a broadcast OSC message runs once. Same order of checks
/// as a DMX line: its own settings, the NIC, then the role.
fn control_state(
    state: &AppState,
    d: &mut Desired,
    direction: Direction,
    id: u32,
    line: &lumacore_protocols::Line,
    nics: &[Nic],
    silent: Option<&LineState>,
) -> LineState {
    if direction == Direction::Out {
        return LineState::Error(format!("{} lines are inputs (Set {} {id} \"Mode\" Input)", line.protocol, line.protocol));
    }
    // Commands run as the line's User, with its rights; none set = the line
    // doesn't start (never Admin, decisions §30 "12d").
    let show = &state.root.show_data;
    let user = match line.user {
        None => return LineState::NoUser,
        Some(g) => match show.user_name(g) {
            Some(name) => name,
            None => return LineState::NoUser,
        },
    };
    if show.user(user).is_none() {
        return LineState::NoUser;
    }
    match nic_for(state, show.protocols.network_name(line.network), nics) {
        Err(st) => st,
        Ok(_) if silent.is_some() => silent.cloned().unwrap_or(LineState::NotMaster),
        Ok(nic) => {
            let port = line
                .setting_u64("Port")
                .and_then(|p| u16::try_from(p).ok())
                .or_else(|| protocol::find(&line.protocol).map(|p| p.port))
                .unwrap_or(0);
            d.control.insert(
                (line.protocol.clone(), id),
                ControlPlan { protocol: line.protocol.clone(), nic: nic.ip, port, user: user.to_string() },
            );
            LineState::Listening {
                nic: nic_label(nic),
                port,
            }
        }
    }
}

/// Every line's state as of now: worked out from the show and station
/// (so it is right right after a command), with what the runtimes last
/// reported (socket errors, input sources) for lines that run.
pub fn line_states(state: &AppState) -> Vec<LineRuntime> {
    let live = &state.root.runtime.network.lines;
    desired(state, &nics::list())
        .status
        .iter()
        .map(|s| {
            let rt = s.runtime();
            let reported = live
                .iter()
                .find(|l| l.protocol == rt.protocol && l.line == rt.line);
            let unpinned = reported.is_some_and(|l| l.unpinned);
            let mut line = match (&s.state, reported) {
                (LineState::Receiving { .. }, Some(l))
                    if l.state.starts_with("error")
                        || l.state.starts_with("receiving")
                        || l.state.starts_with("no source") =>
                {
                    l.clone()
                }
                (LineState::Sending { .. } | LineState::Listening { .. }, Some(l))
                    if l.state.starts_with("error") =>
                {
                    l.clone()
                }
                _ => rt,
            };
            line.unpinned = unpinned;
            line
        })
        .collect()
}

/// A line's PacketGap as it is sent (R1-54, Q21): the gaps between its
/// universes never take more than half of one send period, so a line keeps
/// its rate; `Info Output` shows the capped value.
pub fn capped_gap(gap_us: u32, rate_hz: u8, universes: usize) -> u32 {
    let gaps = u32::try_from(universes.saturating_sub(1)).unwrap_or(u32::MAX).max(1);
    let half_period = 1_000_000 / u32::from(rate_hz.clamp(1, 60)) / 2;
    // A value saved above the setting's range counts as its top (2000 µs).
    gap_us.min(MAX_GAP_US).min(half_period / gaps)
}

/// The PacketGap setting's top (µs).
const MAX_GAP_US: u32 = 2_000;

/// One driver sends all its lines in turn: their gaps together never take
/// more than half of the fastest line's period (each line is scaled down
/// alike), so no line on the NIC loses its rate.
fn cap_driver_gaps(plan: &mut DriverPlan) {
    let Some(fastest) = plan.outputs.iter().map(|o| o.rate_hz).max() else {
        return;
    };
    let budget = u64::from(1_000_000 / u32::from(fastest.clamp(1, 60)) / 2);
    let total: u64 = plan
        .outputs
        .iter()
        .map(|o| u64::from(o.packet_gap_us) * o.routes.len().saturating_sub(1) as u64)
        .sum();
    if total > budget {
        for o in &mut plan.outputs {
            o.packet_gap_us = u32::try_from(u64::from(o.packet_gap_us) * budget / total).unwrap_or(0);
        }
    }
}

#[cfg(test)]
#[test]
fn a_packet_gap_never_takes_more_than_half_the_period() {
    // 30 Hz: 33 333 µs per frame, half = 16 666 µs.
    assert_eq!(capped_gap(500, 30, 4), 500, "fits");
    assert_eq!(capped_gap(2000, 30, 64), 16_666 / 63, "64 universes share half the period");
    assert_eq!(capped_gap(2000, 60, 1), 2000, "one universe has no gap to cap");
    assert_eq!(capped_gap(0, 30, 100), 0);
    assert_eq!(capped_gap(9000, 1, 2), 2000, "a saved value above the range counts as 2000");
}

#[cfg(test)]
#[test]
fn a_drivers_lines_share_half_the_period() {
    use std::net::Ipv4Addr;
    let line = |id: u32, universes: u16, gap: u32| LinePlan {
        line: id,
        unicast: None,
        priority: 100,
        routes: (1..=universes).map(|u| (u32::from(u), u)).collect(),
        rate_hz: 30,
        send: lumacore_protocols::LineSend::Full,
        packet_gap_us: gap,
        sync: false,
    };
    let nic = Nic {
        name: "en0".into(),
        ip: Ipv4Addr::LOCALHOST,
        netmask: Ipv4Addr::new(255, 0, 0, 0),
        broadcast: Ipv4Addr::BROADCAST,
        index: None,
    };
    // Two lines, each 11 gaps of 1000 µs: 22 ms together > 16.6 ms.
    let mut plan = DriverPlan { nic, network: "Rig".into(), outputs: vec![line(1, 12, 1000), line(2, 12, 1000)], inputs: Vec::new() };
    cap_driver_gaps(&mut plan);
    let total: u32 = plan.outputs.iter().map(|o| o.packet_gap_us * 11).sum();
    assert!(total <= 16_666, "{total}");
    assert_eq!(plan.outputs[0].packet_gap_us, plan.outputs[1].packet_gap_us, "scaled alike");
}
