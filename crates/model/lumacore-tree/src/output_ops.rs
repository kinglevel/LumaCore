//! Protocol lines and the run of universes each carries (decisions §11, §34 "34f7c hardened"):
//! a line is one DMX Protocols row — `Assign Universe 5 Thru 8 At Sacn 1 /Universe 101` sets its
//! LocalUniverse 5, Amount 4, its first protocol universe 101 and Mode Output;
//! `Assign Sacn 2 At Universe 40` Mode Input. Every protocol's lines share every operation.
//!
//! `LineOps` is stateless and works on `&mut ShowData`; the caller emits
//! the returned `TreeEvent`s.

use uuid::Uuid;
use crate::events::TreeEvent;
use lumacore_protocols::{Direction, Line, Network};
use crate::tree::ShowData;

/// Errors of lines and their universes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineError {
    /// A referenced universe ID does not exist in ShowData.
    #[error("universe {0} not found")]
    UniverseNotFound(u32),
    /// A referenced line does not exist in ShowData.
    #[error("{0} {1} not found")]
    LineNotFound(String, u32),
    /// Another output line of the protocol already sends this protocol
    /// universe on the same network to the same destination (R1-54).
    #[error(
        "{protocol} {other} already sends {protocol} universe {wire} on this network to the same destination — {protocol} {line} can't send it too"
    )]
    WireClash {
        line: u32,
        other: u32,
        protocol: String,
        wire: u16,
    },
    /// The protocol universe number is out of range.
    #[error("{protocol} universe {wire} is out of range ({range})")]
    BadWireUniverse {
        protocol: String,
        wire: u32,
        range: String,
    },
    /// A Data line (OSC) carries commands, not universes.
    #[error("{0} carries commands, not universes")]
    NotDmx(String),
    /// A line carries one run of universes (LocalUniverse + Amount).
    #[error("a line carries one run of universes: {0}")]
    NotOneRun(String),
}

impl crate::Coded for LineError {
    fn code(&self) -> &'static str {
        match self {
            Self::UniverseNotFound(_) => "E-OUTPUT-001",
            Self::LineNotFound(..) => "E-OUTPUT-002",
            Self::WireClash { .. } => "E-OUTPUT-003",
            Self::BadWireUniverse { .. } => "E-OUTPUT-005",
            Self::NotDmx(_) => "E-OUTPUT-006",
            Self::NotOneRun(_) => "E-OUTPUT-007",
        }
    }
}

/// Error catalog of lines (`Info Error <code>`).
pub const OUTPUT_ERRORS: &[crate::ErrorInfo] = &[
    crate::ErrorInfo {
        code: "E-OUTPUT-001",
        title: "Universe not found",
        explain: "The command refers to a universe that does not exist in the show.",
    },
    crate::ErrorInfo {
        code: "E-OUTPUT-002",
        title: "Line not found",
        explain: "No line of that protocol has that number (List Sacn, List ArtNet, List Osc show them).",
    },
    crate::ErrorInfo {
        code: "E-OUTPUT-003",
        title: "Protocol universe already sent",
        explain: "Only one output line of a protocol may send a protocol universe (sACN universe, Art-Net Port-Address) on a network to the same destination (multicast / broadcast, or the same unicast target): two would fight in every receiver. Pick another protocol universe, or give the lines different unicast targets.",
    },
    crate::ErrorInfo {
        code: "E-OUTPUT-005",
        title: "Protocol universe out of range",
        explain: "sACN universes are 1–63999; Art-Net Port-Addresses are 0–32767 (Net 0–127, SubNet 0–15, Universe 0–15). A line's run of universes must end inside the range too.",
    },
    crate::ErrorInfo {
        code: "E-OUTPUT-006",
        title: "Not a DMX line",
        explain: "OSC lines carry commands; only DMX protocols' lines (sACN, Art-Net) take universes.",
    },
    crate::ErrorInfo {
        code: "E-OUTPUT-007",
        title: "Not one run of universes",
        explain: "A line carries one run of local universes, LocalUniverse and the Amount after it (DMX Protocols rows): assign consecutive universes (Universe 5 Thru 8), take universes off either end, or make another line for another run.",
    },
];

/// Stateless namespace for line operations.
pub struct LineOps;

impl LineOps {
    /// The GUID of line `line` of `protocol`.
    pub fn line_guid(show: &ShowData, protocol: &str, line: u32) -> Result<Uuid, LineError> {
        show.protocols
            .line(protocol, line)
            .map(|l| l.meta.identity.guid)
            .ok_or_else(|| LineError::LineNotFound(protocol.to_string(), line))
    }

    /// The ID of the network called `name`, created if the show has none.
    pub fn ensure_network(show: &mut ShowData, name: &str) -> u32 {
        if let Some((&id, _)) = show.protocols.networks.iter().find(|(_, n)| {
            n.meta
                .name()
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        }) {
            return id;
        }
        let id = show
            .protocols
            .networks
            .keys()
            .next_back()
            .map_or(1, |n| n + 1);
        show.protocols.networks.insert(
            id,
            Network {
                meta: crate::node::NodeMeta::new(Some(name.to_string())),
            },
        );
        id
    }

    /// Every protocol node names a protocol of this build, sits in its kind's group and holds only
    /// settings its protocol declares (decisions §34 "34f7b hardening" 2, 5, "34f7c hardened" 5):
    /// a show from a build with another protocol set is refused, never loaded with silent lines or
    /// dropped settings.
    pub fn check_protocols(show: &ShowData) -> Result<(), String> {
        use lumacore_protocols::protocol;
        let p = &show.protocols;
        for (group, nodes) in [("DMXProtocols", &p.dmx_protocols), ("DataProtocols", &p.data_protocols)] {
            for (name, node) in nodes {
                let Some(row) = protocol::find(name).filter(|r| r.name == name) else {
                    let known: Vec<&str> = protocol::all().map(|p| p.name).collect();
                    return Err(lumacore_base::with_code(
                        format!("The show has {group} {name}, which this build doesn't have ({}).", known.join(", ")),
                        "E-SHOW-007",
                    ));
                };
                if row.kind.group() != group {
                    return Err(lumacore_base::with_code(
                        format!("The show has {name} under {group}; this build's {name} is one of the {}.", row.kind.group()),
                        "E-SHOW-007",
                    ));
                }
                let wide = (row.protocol_settings)();
                if let Some(key) = node.settings.keys().find(|k| !wide.iter().any(|d| &d.key == *k)) {
                    return Err(lumacore_base::with_code(
                        format!("{name} has the setting {key}, which it doesn't declare in this build."),
                        "E-SHOW-008",
                    ));
                }
                let declared = (row.settings)();
                for (no, line) in &node.lines {
                    if let Some(key) = line.settings.keys().find(|k| !declared.iter().any(|d| &d.key == *k)) {
                        return Err(lumacore_base::with_code(
                            format!("{name} {no} has the setting {key}, which {name} doesn't declare in this build."),
                            "E-SHOW-008",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Create a line of `protocol` (its network is created if missing). None when the protocol
    /// isn't this build's or no line number is left (nothing is created).
    pub fn create_line(
        show: &mut ShowData,
        protocol: &str,
        mode: Direction,
        id: Option<u32>,
        name: Option<String>,
        network: &str,
    ) -> Option<u32> {
        let row = lumacore_protocols::protocol::find(protocol)?;
        let node = show.protocols.node_mut(protocol)?;
        let id = match id {
            Some(id) => id,
            None => node.id_pool.allocate()?,
        };
        node.id_pool.reserve(id);
        let net = Self::ensure_network(show, network);
        let network = show.protocols.networks[&net].meta.identity.guid;
        show.protocols
            .lines_mut(protocol)?
            .insert(id, Line::new(name, mode, row.name, network));
        Some(id)
    }

    /// Output line `line` and another output line of its protocol send the same protocol
    /// universe on one network to the same destination (E-OUTPUT-003) — checked after a line's
    /// run, Mode, Network or Target changes.
    pub fn wire_clash(show: &ShowData, protocol: &str, line: u32) -> Option<LineError> {
        let l = show.protocols.line(protocol, line)?;
        if l.mode != Direction::Out {
            return None;
        }
        let mine = l.wires();
        for (&other_no, other) in show.protocols.lines(protocol)? {
            if other_no == line
                || other.mode != Direction::Out
                || other.network != l.network
                || other.target != l.target
            {
                continue;
            }
            let theirs = other.wires();
            if let Some(wire) = mine.clone().find(|w| theirs.contains(w)) {
                return Some(LineError::WireClash {
                    line,
                    other: other_no,
                    protocol: l.protocol.clone(),
                    wire: u16::try_from(wire).unwrap_or(u16::MAX),
                });
            }
        }
        None
    }

    /// The run's protocol universes fit the protocol's range (E-OUTPUT-005); a Data line takes
    /// none (E-OUTPUT-006).
    pub fn check_range(show: &ShowData, protocol: &str, line: u32) -> Result<(), LineError> {
        let l = show
            .protocols
            .line(protocol, line)
            .ok_or_else(|| LineError::LineNotFound(protocol.to_string(), line))?;
        if l.amount == 0 {
            return Ok(());
        }
        let Some(u) = lumacore_protocols::protocol::find(protocol).and_then(|p| p.universes) else {
            return Err(LineError::NotDmx(format!("{protocol} {line}")));
        };
        let wires = l.wires();
        let (first, last) = (wires.start, wires.end - 1);
        for wire in [first, last] {
            if wire < u32::from(u.first) || wire > u32::from(u.last) {
                return Err(LineError::BadWireUniverse { protocol: protocol.to_string(), wire, range: u.range_text() });
            }
        }
        Ok(())
    }

    /// Give line `line` the run `ids` (consecutive universes of the show) in `mode`, starting on
    /// the wire at `start` (None: the line's own first protocol universe). The universes must
    /// exist and follow on without a gap (E-OUTPUT-001 / 007); the run must fit the protocol's
    /// range (E-OUTPUT-005) and not clash with another output line (E-OUTPUT-003). Nothing changes
    /// when refused. Returns the (universe, protocol universe) pairs.
    pub fn set_run(
        show: &mut ShowData,
        protocol: &str,
        line: u32,
        mode: Direction,
        ids: &[u32],
        start: Option<u32>,
        events: &mut Vec<TreeEvent>,
    ) -> Result<Vec<(u32, u16)>, LineError> {
        let before = show
            .protocols
            .line(protocol, line)
            .ok_or_else(|| LineError::LineNotFound(protocol.to_string(), line))?
            .clone();
        if lumacore_protocols::protocol::find(protocol).and_then(|p| p.universes).is_none() {
            return Err(LineError::NotDmx(format!("{protocol} {line}")));
        }
        let mut sorted = ids.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        let (Some(&first), Some(&last)) = (sorted.first(), sorted.last()) else {
            return Err(LineError::NotOneRun("no universe given".into()));
        };
        if let Some(missing) = (first..=last).find(|u| !show.devices.dmx.patch.universes.contains_key(u)) {
            return Err(if sorted.contains(&missing) || sorted.len() == 1 {
                LineError::UniverseNotFound(missing)
            } else {
                LineError::NotOneRun(format!("Universe {missing} doesn't exist between {first} and {last}"))
            });
        }
        if u32::try_from(sorted.len()).ok() != Some(last - first + 1) {
            return Err(LineError::NotOneRun(format!("Universe {first} Thru {last} has gaps")));
        }
        let start = match start {
            Some(s) => u16::try_from(s).map_err(|_| LineError::BadWireUniverse {
                protocol: protocol.to_string(),
                wire: s,
                range: lumacore_protocols::protocol::find(protocol)
                    .and_then(|p| p.universes)
                    .map(|u| u.range_text())
                    .unwrap_or_default(),
            })?,
            None => before.universe,
        };
        {
            let l = show.protocols.lines_mut(protocol).and_then(|m| m.get_mut(&line)).expect("checked");
            l.mode = mode;
            l.local_universe = first;
            l.amount = last - first + 1;
            l.universe = start;
        }
        let refused = Self::check_range(show, protocol, line).err().or_else(|| Self::wire_clash(show, protocol, line));
        if let Some(e) = refused {
            show.protocols.lines_mut(protocol).expect("checked").insert(line, before);
            return Err(e);
        }
        let l = show.protocols.line(protocol, line).expect("checked");
        let pairs: Vec<(u32, u16)> = l.pairs().collect();
        if mode == Direction::Out {
            for &(id, _) in &pairs {
                events.push(TreeEvent::OutputAssigned { output_id: line, universe_id: id });
            }
        }
        Ok(pairs)
    }

    /// Take universes `ids` off line `line`'s run: from either end (the protocol universes of
    /// the rest stay where they were) or all of it (Amount 0). A universe in the middle would
    /// split the run (E-OUTPUT-007). Returns how many went.
    pub fn take_off(show: &mut ShowData, protocol: &str, line: u32, ids: &[u32]) -> Result<u32, LineError> {
        let l = show
            .protocols
            .lines_mut(protocol)
            .and_then(|m| m.get_mut(&line))
            .ok_or_else(|| LineError::LineNotFound(protocol.to_string(), line))?;
        let run = l.slots();
        let mut off: Vec<u32> = ids.iter().copied().filter(|u| run.contains(u)).collect();
        off.sort_unstable();
        off.dedup();
        let Some((&lo, &hi)) = off.first().zip(off.last()) else {
            return Ok(0);
        };
        let n = u32::try_from(off.len()).unwrap_or(u32::MAX);
        if n != hi - lo + 1 {
            return Err(LineError::NotOneRun(format!("taking off {} would leave gaps", off.iter().map(u32::to_string).collect::<Vec<_>>().join(", "))));
        }
        if lo == run.start && hi + 1 == run.end {
            l.amount = 0;
        } else if lo == run.start {
            l.local_universe = hi + 1;
            l.universe = l.universe.saturating_add(u16::try_from(n).unwrap_or(u16::MAX));
            l.amount -= n;
        } else if hi + 1 == run.end {
            l.amount -= n;
        } else {
            return Err(LineError::NotOneRun(format!(
                "Universe {lo} Thru {hi} is inside {protocol} {line}'s run {}–{}: take universes off its ends, or make another line",
                run.start,
                run.end - 1
            )));
        }
        Ok(n)
    }

    /// Every (local universe, protocol universe) a line carries whose universe the show has.
    pub fn routes_of(show: &ShowData, protocol: &str, line: u32) -> Vec<(u32, u16)> {
        show.protocols
            .line(protocol, line)
            .map(|l| l.pairs().filter(|(u, _)| show.devices.dmx.patch.universes.contains_key(u)).collect())
            .unwrap_or_default()
    }

    /// The lines whose run covers universe `slot`: (protocol, number, its protocol universe).
    pub fn lines_on(show: &ShowData, slot: u32) -> Vec<(String, u32, Direction, u16)> {
        show.protocols
            .all_lines()
            .filter_map(|(p, no, l)| {
                l.pairs().find(|(s, _)| *s == slot).map(|(_, w)| (p.to_string(), no, l.mode, w))
            })
            .collect()
    }

    /// Delete a line.
    pub fn delete_line(show: &mut ShowData, protocol: &str, line: u32) -> Result<(), LineError> {
        show.protocols
            .lines_mut(protocol)
            .and_then(|m| m.remove(&line))
            .map(|_| ())
            .ok_or_else(|| LineError::LineNotFound(protocol.to_string(), line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe_ops::UniverseOps;

    fn show_with(n: u32) -> ShowData {
        let mut show = ShowData::new();
        let mut ev = Vec::new();
        for _ in 0..n {
            UniverseOps::create_universe(&mut show, None, &mut ev).unwrap();
        }
        show
    }

    fn sacn(show: &mut ShowData, network: &str) -> u32 {
        LineOps::create_line(show, "Sacn", Direction::Out, None, None, network).unwrap()
    }

    #[test]
    fn a_line_carries_one_run_of_universes() {
        let mut show = show_with(6);
        let line = sacn(&mut show, "Rig");
        let mut ev = Vec::new();
        let pairs = LineOps::set_run(&mut show, "Sacn", line, Direction::Out, &[1, 2, 3], Some(10), &mut ev).unwrap();
        assert_eq!(pairs, vec![(1, 10), (2, 11), (3, 12)]);
        assert_eq!(LineOps::routes_of(&show, "Sacn", line).len(), 3);
        assert_eq!(show.protocols.networks.len(), 1);
        // A gap is refused: a line is one run.
        let gap = LineOps::set_run(&mut show, "Sacn", line, Direction::Out, &[1, 3], None, &mut ev).unwrap_err();
        assert!(matches!(gap, LineError::NotOneRun(_)));
        assert_eq!(LineOps::routes_of(&show, "Sacn", line).len(), 3, "a refusal changes nothing");
        // A missing universe is named.
        assert_eq!(
            LineOps::set_run(&mut show, "Sacn", line, Direction::Out, &[9], None, &mut ev),
            Err(LineError::UniverseNotFound(9))
        );
        // Without a start the line keeps its first protocol universe.
        LineOps::set_run(&mut show, "Sacn", line, Direction::Out, &[4, 5], None, &mut ev).unwrap();
        assert_eq!(LineOps::routes_of(&show, "Sacn", line), vec![(4, 10), (5, 11)]);
        // Out of the protocol's range.
        let range = LineOps::set_run(&mut show, "Sacn", line, Direction::Out, &[1, 2], Some(63999), &mut ev).unwrap_err();
        assert!(matches!(range, LineError::BadWireUniverse { .. }));
    }

    #[test]
    fn two_output_lines_never_send_one_protocol_universe_to_one_destination() {
        let mut show = show_with(4);
        let (a, b) = (sacn(&mut show, "Rig"), sacn(&mut show, "Rig"));
        let mut ev = Vec::new();
        LineOps::set_run(&mut show, "Sacn", a, Direction::Out, &[1, 2], Some(1), &mut ev).unwrap();
        let clash = LineOps::set_run(&mut show, "Sacn", b, Direction::Out, &[3, 4], Some(2), &mut ev).unwrap_err();
        assert_eq!(clash, LineError::WireClash { line: b, other: a, protocol: "Sacn".into(), wire: 2 });
        // The same local universes on two lines are fine (decisions §34 "34f7c hardened" 4).
        LineOps::set_run(&mut show, "Sacn", b, Direction::Out, &[1, 2], Some(3), &mut ev).unwrap();
        assert_eq!(LineOps::lines_on(&show, 1).len(), 2);
        // An input of the same protocol universes doesn't clash.
        let input = LineOps::create_line(&mut show, "Sacn", Direction::In, None, None, "Rig").unwrap();
        LineOps::set_run(&mut show, "Sacn", input, Direction::In, &[4], Some(1), &mut ev).unwrap();
    }

    #[test]
    fn universes_come_off_the_ends_of_a_run() {
        let mut show = show_with(6);
        let line = sacn(&mut show, "Rig");
        let mut ev = Vec::new();
        LineOps::set_run(&mut show, "Sacn", line, Direction::Out, &[1, 2, 3, 4, 5], Some(101), &mut ev).unwrap();
        assert!(matches!(LineOps::take_off(&mut show, "Sacn", line, &[3]), Err(LineError::NotOneRun(_))));
        assert_eq!(LineOps::take_off(&mut show, "Sacn", line, &[1]), Ok(1));
        assert_eq!(LineOps::take_off(&mut show, "Sacn", line, &[5]), Ok(1));
        assert_eq!(LineOps::routes_of(&show, "Sacn", line), vec![(2, 102), (3, 103), (4, 104)]);
        assert_eq!(LineOps::take_off(&mut show, "Sacn", line, &[2, 3, 4]), Ok(3));
        assert_eq!(show.protocols.line("Sacn", line).unwrap().amount, 0);
        LineOps::delete_line(&mut show, "Sacn", line).unwrap();
        assert_eq!(
            LineOps::delete_line(&mut show, "Sacn", line),
            Err(LineError::LineNotFound("Sacn".into(), line))
        );
    }
}
