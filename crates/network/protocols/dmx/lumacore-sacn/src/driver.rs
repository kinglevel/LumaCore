// ██████╗ ██████╗ ██╗██╗   ██╗███████╗██████╗
// ██╔══██╗██╔══██╗██║██║   ██║██╔════╝██╔══██╗
// ██║  ██║██████╔╝██║██║   ██║█████╗  ██████╔╝
// ██║  ██║██╔══██╗██║╚██╗ ██╔╝██╔══╝  ██╔══██╗
// ██████╔╝██║  ██║██║ ╚████╔╝ ███████╗██║  ██║
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝
// FILE: driver

//! sACN as a [`DmxDriver`] (decisions §34 22): one [`SacnSource`] sending from the driver's NIC,
//! and a [`SacnReceiver`] opened while there are input lines (retried each second when it can't).
//!
//! Owns:          SacnDriver, open (the row's opener).
//! Doesn't own:   the packets and the sockets (source, receiver), when a line sends (lumacore-lines).
//! Entry points:  open.
//! Decisions:     §11 "Step 4c hardening", §34 22.

use std::collections::BTreeSet;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use lumacore_net::Nic;
use lumacore_protocols::{DmxDriver, DmxError, DriverIdentity, LinePlan, Packet};
use uuid::Uuid;

use crate::receiver::SacnReceiver;
use crate::source::{Destination, SacnSource};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How long an input socket that couldn't open waits before the next try.
const RECEIVER_RETRY: Duration = Duration::from_secs(1);

/// sACN on one NIC.
pub struct SacnDriver {
    source: SacnSource,
    /// The input socket, open while there are input lines.
    receiver: Option<SacnReceiver>,
    receiver_retry: Option<Instant>,
    nic: Nic,
    /// Our CID: our own packets are ignored.
    cid: Uuid,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The row's opener: an sACN sender on the driver's NIC.
pub fn open(identity: &DriverIdentity) -> Result<Box<dyn DmxDriver>, DmxError> {
    Ok(Box::new(SacnDriver {
        source: SacnSource::new(identity.nic.ip, identity.cid, &identity.name)?,
        receiver: None,
        receiver_retry: None,
        nic: identity.nic.clone(),
        cid: identity.cid,
    }))
}

/// Multicast, or the line's unicast target.
fn destination(line: &LinePlan) -> Destination {
    line.unicast.map_or(Destination::Multicast, Destination::Unicast)
}

impl DmxDriver for SacnDriver {
    fn plan(&mut self, plan: &[LinePlan]) {
        let streams: BTreeSet<(Destination, u16)> = plan
            .iter()
            .flat_map(|l| {
                let dest = destination(l);
                l.routes.iter().map(move |&(_, w)| (dest, w))
            })
            .collect();
        self.source.prepare(&streams);
    }

    // tidy: render-begin (decisions §24: the per-frame path)
    fn send(&mut self, wire: u16, data: &[u8; 512], line: &LinePlan, _targets: &[Ipv4Addr]) -> std::io::Result<u64> {
        self.source
            .send(wire, data, line.priority, destination(line), line.sync)
            .map(|()| 1)
    }

    fn sync(&mut self, line: &LinePlan) -> std::io::Result<()> {
        self.source.sync(destination(line))
    }
    // tidy: render-end

    fn after_send(&mut self) {
        self.source.discovery();
    }

    fn receive(&mut self, wires: &BTreeSet<u16>, packets: &mut Vec<Packet>) -> Option<String> {
        if wires.is_empty() {
            self.receiver = None;
            return None;
        }
        let mut first_error = None;
        if self.receiver.is_none() && self.receiver_retry.is_none_or(|t| t.elapsed() >= RECEIVER_RETRY) {
            self.receiver_retry = Some(Instant::now());
            match SacnReceiver::new(&self.nic, self.cid) {
                Ok(r) => self.receiver = Some(r),
                Err(e) => first_error = Some(format!("sACN input socket: {e}")),
            }
        }
        if let Some(rx) = &mut self.receiver {
            if let Some(e) = rx.listen_to(wires) {
                first_error.get_or_insert(e);
            }
            rx.receive(packets);
        }
        first_error
    }

    fn pinned(&self) -> Option<bool> {
        self.receiver.as_ref().map(SacnReceiver::pinned)
    }
}
