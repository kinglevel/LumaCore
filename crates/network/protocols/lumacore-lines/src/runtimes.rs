// ██████╗ ██╗   ██╗███╗   ██╗████████╗██╗███╗   ███╗███████╗███████╗
// ██╔══██╗██║   ██║████╗  ██║╚══██╔══╝██║████╗ ████║██╔════╝██╔════╝
// ██████╔╝██║   ██║██╔██╗ ██║   ██║   ██║██╔████╔██║█████╗  ███████╗
// ██╔══██╗██║   ██║██║╚██╗██║   ██║   ██║██║╚██╔╝██║██╔══╝  ╚════██║
// ██║  ██║╚██████╔╝██║ ╚████║   ██║   ██║██║ ╚═╝ ██║███████╗███████║
// ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝╚══════╝
// FILE: runtimes

//! Protocol runtimes from show content (decisions §11 "Step 4b/4c
//! hardening").
//!
//! Nothing protocol-related starts at boot. After every show change (and
//! once a second, for NIC changes) the state thread works out what should
//! run — `desired`: a pure function of the show's lines and routes, this
//! station's network mapping and its session role — and `ProtocolRuntimes`
//! starts, re-plans or stops the runtimes to match:
//!
//! - sACN and Art-Net: one driver per (protocol, NIC), attached to the
//!   render engine (`crate::lines`). It sends the output lines on
//!   that NIC and takes in the input lines; the engine merges the input.
//! - OSC input: one listener per line, commands run as the line's user.
//!
//! A line on a network this station hasn't mapped never sends or listens;
//! only a standalone station or the session master does either.
//!
//! Owns:          ProtocolRuntimes: starting, re-planning and stopping the line drivers and Control listeners to match the show, publishing line states, nodes and sources to Runtime.Network.
//! Doesn't own:   what should run (lumacore_objects::protocol_plan::desired), the drivers themselves (lines, the protocol crates).
//! Entry points:  ProtocolRuntimes::{new, reconcile, stop_all}.
//! Decisions:     §11 "Step 4b/4c hardening", §21 "6c", §34 22.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use lumacore_base::InboundEvent;
use lumacore_net::nics::{self, Nic};
use lumacore_protocols::protocol::{self, Capability, Kind, Open};
use lumacore_protocols::{ControlListener, ControlStart, DriverIdentity, SourceKey};
use lumacore_render::RenderHub;
use lumacore_tree::{NodeRuntime, SourceRuntime};
use lumacore_protocols::Direction;

use lumacore_objects::AppState;
use lumacore_objects::protocol_plan::{LineState, desired, distribution};

use crate::lines::NetworkDriver;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The running protocol runtimes (owned by the state thread).
pub struct ProtocolRuntimes {
    hub: Arc<RenderHub>,
    inbound: Sender<InboundEvent>,
    health: Option<Arc<lumacore_tree::runtime::HealthBoard>>,
    /// By (protocol row name, NIC address), with the NIC it was started on.
    drivers: BTreeMap<(String, Ipv4Addr), (Nic, NetworkDriver)>,
    /// Control listeners by input line.
    control: BTreeMap<(String, u32), ControlLine>,
    /// Socket errors from starting a runtime, by driver or `in<line>` (a Control line);
    /// retried on the next reconcile.
    start_errors: BTreeMap<String, String>,
    stopping: Vec<JoinHandle<()>>,
    nics: Vec<Nic>,
    nics_at: Option<Instant>,
    /// Input lines the System Monitor was told have no user (once per
    /// episode: said again after the line got one).
    no_user_said: std::collections::BTreeSet<u32>,
}

/// A running Control line and what it was started with (a change restarts it).
struct ControlLine {
    plan: lumacore_objects::protocol_plan::ControlPlan,
    listener: Box<dyn ControlListener>,
}

const NIC_REFRESH: Duration = Duration::from_secs(5);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// A Data line's key among the start errors (`Osc.3`).
fn data_key(protocol: &str, line: u32) -> String {
    format!("{protocol}.{line}")
}

fn driver_key(protocol: &str, ip: Ipv4Addr) -> String {
    format!("{protocol}-{ip}")
}

impl ProtocolRuntimes {
    pub fn new(
        hub: Arc<RenderHub>,
        inbound: Sender<InboundEvent>,
        health: Option<Arc<lumacore_tree::runtime::HealthBoard>>,
    ) -> Self {
        Self {
            hub,
            inbound,
            health,
            drivers: BTreeMap::new(),
            control: BTreeMap::new(),
            start_errors: BTreeMap::new(),
            stopping: Vec::new(),
            nics: Vec::new(),
            nics_at: None,
            no_user_said: Default::default(),
        }
    }

    /// Make the runtimes match the show; publish line states and input
    /// sources to `Runtime.Network`.
    pub fn reconcile(&mut self, state: &mut AppState) {
        if self.nics_at.is_none_or(|t| t.elapsed() >= NIC_REFRESH) {
            self.nics = nics::list();
            self.nics_at = Some(Instant::now());
        }
        let mut d = desired(state, &self.nics);
        // A Control line without a user doesn't start (never Admin, decisions
        // §30 "12d"): the System Monitor says why, once per episode.
        let no_user: std::collections::BTreeSet<u32> = d
            .status
            .iter()
            .filter(|l| l.direction == Direction::In && l.state == LineState::NoUser)
            .map(|l| l.line)
            .collect();
        for id in no_user.difference(&self.no_user_said) {
            state.sysmon_pending.push((
                "Input".to_string(),
                format!("Input {id} doesn't start: a Control line runs as a user, never Admin — Set Input {id} \"User\" \"name\" [E-RIGHTS-003]"),
            ));
        }
        self.no_user_said = no_user;
        // Output distribution ("Output: distribution changed"): one
        // line in the System Monitor whenever what this station sends
        // changes — show, NIC mapping, its session role (§21 "6c").
        let plan = distribution(state, &d);
        if plan != state.output_plan {
            state
                .sysmon_pending
                .push(("Output".to_string(), format!("distribution changed: {plan}")));
            state.output_plan = plan;
        }
        let cid = state.root.station_settings.sacn_cid;
        let name = state.root.station_settings.station_name.clone();
        self.start_errors.clear();

        // Dmx drivers (a NIC whose address details changed means
        // a new driver).
        let gone: Vec<(String, Ipv4Addr)> = self
            .drivers
            .iter()
            .filter(|(key, (nic, _))| d.drivers.get(key).is_none_or(|p| p.nic != *nic))
            .map(|(key, _)| key.clone())
            .collect();
        for key in gone {
            if let Some((_, mut drv)) = self.drivers.remove(&key)
                && let Some(t) = drv.stop()
            {
                self.stopping.push(t);
            }
        }
        for ((protocol, ip), plan) in &d.drivers {
            let ip = *ip;
            if let Some((_, drv)) = self.drivers.get(&(protocol.clone(), ip)) {
                drv.set_plan(plan.outputs.clone(), plan.inputs.clone());
                continue;
            }
            // The protocol's own row opens it (decisions §34 22): nothing here names one.
            let Some(Open::Dmx(open)) = protocol::find(protocol).map(|p| p.open) else {
                self.start_errors.insert(
                    driver_key(protocol, ip),
                    format!("{protocol} isn't a DMX protocol of this build"),
                );
                continue;
            };
            let identity = DriverIdentity {
                nic: plan.nic.clone(),
                name: name.clone(),
                cid,
            };
            match open(&identity) {
                Ok(sender) => {
                    let drv = NetworkDriver::start(
                        self.hub.clone(),
                        sender,
                        plan.outputs.clone(),
                        plan.inputs.clone(),
                        &format!("{}-{ip}", protocol.to_lowercase()),
                        self.health
                            .as_ref()
                            .map(|h| h.register(&format!("Line.{protocol}-{ip}"))),
                    );
                    self.drivers.insert((protocol.clone(), ip), (plan.nic.clone(), drv));
                }
                Err(e) => {
                    self.start_errors.insert(
                        driver_key(protocol, ip),
                        format!("{protocol} socket: {e}"),
                    );
                }
            }
        }

        // Control listeners (a changed plan restarts one).
        let stale: Vec<(String, u32)> = self
            .control
            .iter()
            .filter(|(id, l)| d.control.get(*id) != Some(&l.plan))
            .map(|(id, _)| id.clone())
            .collect();
        for id in stale {
            if let Some(mut l) = self.control.remove(&id)
                && let Some(t) = l.listener.stop()
            {
                self.stopping.push(t);
            }
        }
        for (key, plan) in &d.control {
            if self.control.contains_key(key) {
                continue;
            }
            let id = key.1;
            let health = self
                .health
                .as_ref()
                .map(|h| h.register(&format!("{}.{id}", plan.protocol)));
            let Some(Open::Control(open)) = protocol::find(&plan.protocol).map(|p| p.open) else {
                self.start_errors.insert(data_key(&plan.protocol, id), format!("{} isn't a Data protocol of this build", plan.protocol));
                continue;
            };
            let start = ControlStart {
                bind: plan.nic,
                port: plan.port,
                user: plan.user.clone(),
                inbound: self.inbound.clone(),
                health,
            };
            match open(start) {
                Ok(listener) => {
                    self.control.insert(key.clone(), ControlLine { plan: plan.clone(), listener });
                }
                Err(e) => {
                    self.start_errors.insert(data_key(&plan.protocol, id), e);
                }
            }
        }

        // Live errors and input sources into the line states.
        let ip_of = |nic: &str| -> Option<Ipv4Addr> { nic.rsplit(' ').next()?.parse().ok() };
        let mut unpinned: Vec<(String, u32)> = Vec::new();
        for st in &mut d.status {
            let row = protocol::find(&st.protocol);
            if row.is_some_and(|p| p.kind == Kind::Data) {
                if let (Direction::In, Some(e)) =
                    (st.direction, self.start_errors.get(&data_key(&st.protocol, st.line)))
                {
                    st.state = LineState::Error(e.clone());
                }
                continue;
            }
            let ip = match &st.state {
                LineState::Sending { nic, .. } | LineState::Receiving { nic, .. } => ip_of(nic),
                _ => None,
            };
            let Some(ip) = ip else {
                continue;
            };
            if let Some(e) = self.start_errors.get(&driver_key(&st.protocol, ip)) {
                st.state = LineState::Error(e.clone());
                continue;
            }
            let Some((_, drv)) = self.drivers.get(&(st.protocol.clone(), ip)) else {
                continue;
            };
            // sACN sends from its own port; only its receiver shares 5568.
            // Art-Net sends and receives on the shared 6454 (`SharedPort`).
            if drv.shared.unpinned.load(std::sync::atomic::Ordering::Relaxed)
                && (row.is_some_and(|p| p.can(Capability::SharedPort)) || st.direction == Direction::In)
            {
                unpinned.push((st.protocol.clone(), st.line));
            }
            if let LineState::Receiving { live, .. } = &mut st.state {
                *live = drv
                    .shared
                    .layers
                    .load()
                    .iter()
                    .filter(|l| l.line == st.line)
                    .count();
            }
            if let Some(e) = (**drv.shared.error.load()).clone() {
                st.state = LineState::Error(e);
            }
        }

        let mut nodes = Vec::new();
        let mut sources = Vec::new();
        let mut line_stats = Vec::new();
        for ((protocol, ip), (_, drv)) in &self.drivers {
            for (line, stats) in drv.shared.lines.load().iter() {
                line_stats.push((*line, protocol.clone(), stats.clone()));
            }
            let network = d
                .drivers
                .get(&(protocol.clone(), *ip))
                .map_or(String::new(), |p| p.network.clone());
            for n in drv.shared.nodes.load().iter() {
                nodes.push(NodeRuntime {
                    protocol: protocol.clone(),
                    ip: n.ip.to_string(),
                    name: n.name.clone(),
                    outputs: n.outputs.clone(),
                    network: network.clone(),
                });
            }
            for s in drv.shared.sources.load().iter() {
                sources.push(SourceRuntime {
                    protocol: protocol.clone(),
                    network: network.clone(),
                    name: s.name.clone(),
                    id: match s.key {
                        SourceKey::Cid(cid) => cid.to_string(),
                        SourceKey::Ip(ip) => ip.to_string(),
                    },
                    ip: s.ip.to_string(),
                    priority: s.priority,
                    universes: s.universes.clone(),
                });
            }
        }

        line_stats.sort_by_key(|(line, _, _)| *line);
        if let Some(h) = &self.health {
            h.set_lines(line_stats);
        }

        let net = &mut state.root.runtime.network;
        net.lines = d
            .status
            .iter()
            .map(|s| {
                let mut rt = s.runtime();
                rt.unpinned = unpinned.contains(&(s.protocol.clone(), s.line));
                rt
            })
            .collect();
        net.nodes = nodes;
        net.sources = sources;
        net.active_output_lines = d
            .status
            .iter()
            .filter(|s| s.direction == Direction::Out && s.state.active())
            .count() as u32;
        // Each running Dmx driver's protocol with its active output lines.
        let mut running: BTreeMap<String, u32> = self.drivers.keys().map(|(p, _)| (p.clone(), 0)).collect();
        for s in d.status.iter().filter(|s| s.direction == Direction::Out && s.state.active()) {
            if let Some(n) = running.get_mut(&s.protocol) {
                *n += 1;
            }
        }
        state.protocols_running = running;
        self.stopping.retain(|t| !t.is_finished());
    }

    /// Stop everything (outputs say goodbye) and wait until `deadline`.
    /// Returns what was still running.
    pub fn stop_all(&mut self, deadline: Instant) -> Vec<String> {
        let mut threads = std::mem::take(&mut self.stopping);
        for (_, mut l) in std::mem::take(&mut self.control) {
            threads.extend(l.listener.stop());
        }
        for (_, (_, mut drv)) in std::mem::take(&mut self.drivers) {
            threads.extend(drv.stop());
        }
        threads
            .into_iter()
            .filter_map(|t| {
                let name = t.thread().name().unwrap_or("protocol").to_string();
                lumacore_base::threads::join_until(t, deadline)
                    .err()
                    .map(|e| format!("{name} ({e})"))
            })
            .collect()
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
