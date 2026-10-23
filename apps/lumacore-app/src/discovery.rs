//! Finds LumaCore servers on the network for the Connect view: mDNS
//! (`_lumacore._tcp`, `lumacore_wire::MDNS_SERVICE`) and the stations'
//! multicast session beacon (`SessionDiscoveryWire`, which carries the HTTP
//! port). Both are merged by station id; the list goes to the event loop
//! whenever it changes.
//!
//! The beacon decides (§30 "12b2 hardening", Servers list): a station that
//! ever beaconed is listed only while its beacon is fresh — late (> 1.5 s)
//! is "not answering", gone after 4 s even while mDNS still lists it (a
//! killed server never withdraws its record), and a `leaving` beacon (clean
//! shutdown) removes it at once. mDNS alone only lists stations whose beacon
//! never reached us.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

/// How long a server stays listed after its last beacon (beacons are 750 ms).
const BEACON_TTL: Duration = Duration::from_secs(4);
/// A beacon older than this is late: the row turns amber, "not answering".
const BEACON_LATE: Duration = Duration::from_millis(1500);

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub show: Option<String>,
    /// What the station is (its host type), e.g. "Server".
    pub kind: String,
    /// It speaks another client protocol (another LumaCore version): the
    /// list greys it out as "Wrong version" (decisions §23 "Step 8").
    pub wrong_version: bool,
    /// Whole seconds since its last beacon once that is late (> 1.5 s): the
    /// page shows an amber dot, "not answering N s". None = fresh, or a
    /// station known only from mDNS.
    pub not_answering_s: Option<u64>,
}

#[derive(Default)]
struct Entry {
    server: Option<Server>,
    /// mDNS fullname while resolved (removed on goodbye).
    mdns: Option<String>,
    beacon_at: Option<Instant>,
    /// Its last beacon said it was shutting down; a new beacon (it started
    /// again) clears it.
    left: bool,
}

impl Entry {
    /// Listed now? `Some(late seconds)` when listed (None = fresh), `None`
    /// when not.
    fn listed(&self, now: Instant) -> Option<Option<u64>> {
        if self.left {
            return None;
        }
        match self.beacon_at {
            // The beacon decides, whatever mDNS says.
            Some(at) => {
                let age = now.saturating_duration_since(at);
                if age >= BEACON_TTL {
                    None
                } else if age > BEACON_LATE {
                    Some(Some(age.as_secs_f32().round() as u64))
                } else {
                    Some(None)
                }
            }
            // Only mDNS found it: listed while the record lives.
            None => self.mdns.as_ref().map(|_| None),
        }
    }
}

type Table = Arc<Mutex<BTreeMap<String, Entry>>>;

/// Start browsing; `on_change` gets the full list (sorted by name).
pub fn start(on_change: impl Fn(Vec<Server>) + Send + 'static) {
    let table: Table = Arc::default();
    browse_mdns(table.clone());
    listen_beacons(table.clone());
    std::thread::spawn(move || {
        let mut last: Vec<Server> = Vec::new();
        loop {
            let now = match table.lock() {
                Ok(t) => list(&t, Instant::now()),
                Err(_) => Vec::new(),
            };
            if now != last {
                on_change(now.clone());
                last = now;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

fn list(t: &BTreeMap<String, Entry>, now: Instant) -> Vec<Server> {
    let mut out: Vec<Server> = t
        .values()
        .filter_map(|e| {
            let late = e.listed(now)?;
            let server = e.server.clone()?;
            Some(Server { not_answering_s: late, ..server })
        })
        .collect();
    out.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.host.cmp(&b.host))
    });
    out
}

fn key(id: &str, host: &str, port: u16) -> String {
    if id.is_empty() {
        format!("{host}:{port}")
    } else {
        id.to_string()
    }
}

fn browse_mdns(table: Table) {
    use mdns_sd::{ServiceDaemon, ServiceEvent};
    let daemon = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            log::warn!("mDNS browse unavailable: {e}");
            return;
        }
    };
    let receiver = match daemon.browse(lumacore_wire::MDNS_SERVICE) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("mDNS browse: {e}");
            return;
        }
    };
    std::thread::spawn(move || {
        let _daemon = daemon; // keeps browsing alive
        while let Ok(event) = receiver.recv() {
            let Ok(mut t) = table.lock() else { return };
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    // Prefer IPv4; mDNS also reports link-local IPv6.
                    let mut addrs: Vec<_> = info.get_addresses().iter().copied().collect();
                    addrs.sort_by_key(|a| !a.is_ipv4());
                    let Some(addr) = addrs.first() else { continue };
                    let prop =
                        |k: &str| info.get_property_val_str(k).unwrap_or_default().to_string();
                    let server = Server {
                        id: prop("id"),
                        name: prop("station"),
                        host: addr.to_string(),
                        port: info.get_port(),
                        version: prop("version"),
                        show: None,
                        kind: prop("kind"),
                        wrong_version: prop("client").parse::<u32>().ok()
                            != Some(lumacore_wire::CLIENT_PROTOCOL),
                        not_answering_s: None,
                    };
                    let entry = t
                        .entry(key(&server.id, &server.host, server.port))
                        .or_default();
                    entry.mdns = Some(info.get_fullname().to_string());
                    // Keep the show name a beacon may have filled in.
                    let show = entry.server.as_ref().and_then(|s| s.show.clone());
                    entry.server = Some(Server { show, ..server });
                }
                ServiceEvent::ServiceRemoved(_, fullname) => {
                    for e in t.values_mut() {
                        if e.mdns.as_deref() == Some(fullname.as_str()) {
                            e.mdns = None;
                        }
                    }
                }
                _ => {}
            }
        }
    });
}

fn listen_beacons(table: Table) {
    let socket = match lumacore_wire::bind_discovery_listener_at(
        lumacore_wire::SESSION_DISCOVERY_PORT,
    ) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("discovery beacon listener: {e}");
            return;
        }
    };
    std::thread::spawn(move || {
        let mut buf = vec![0u8; 65535];
        loop {
            let Ok((len, from)) = socket.recv_from(&mut buf) else {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            };
            let Ok(beacon) =
                serde_json::from_slice::<lumacore_wire::SessionDiscoveryWire>(&buf[..len])
            else {
                continue;
            };
            // Stations without a UI engine can't be used by the app.
            let Some(port) = beacon.http_port else {
                continue;
            };
            let host = from.ip().to_string();
            let Ok(mut t) = table.lock() else { return };
            apply_beacon(&mut t, &beacon, host, port, Instant::now());
        }
    });
}

/// One beacon heard from `host` (its HTTP port `port`) at `now`.
fn apply_beacon(
    t: &mut BTreeMap<String, Entry>,
    beacon: &lumacore_wire::SessionDiscoveryWire,
    host: String,
    port: u16,
    now: Instant,
) {
    let entry = t.entry(key(&beacon.station_id, &host, port)).or_default();
    // A clean shutdown: gone at once (a later beacon brings it back).
    entry.left = beacon.leaving;
    if beacon.leaving {
        return;
    }
    entry.beacon_at = Some(now);
    match &mut entry.server {
        Some(s) => s.show = beacon.show.clone(),
        None => {
            entry.server = Some(Server {
                id: beacon.station_id.clone(),
                name: beacon.station_name.clone(),
                host,
                port,
                version: beacon.version.clone(),
                show: beacon.show.clone(),
                kind: beacon.kind.name().to_string(),
                wrong_version: beacon.client_protocol != lumacore_wire::CLIENT_PROTOCOL,
                not_answering_s: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn beacon(id: &str, leaving: bool) -> lumacore_wire::SessionDiscoveryWire {
        serde_json::from_value(serde_json::json!({
            "station_name": "desk1", "authority": "", "cmd_port": 47653, "shell_port": 47654,
            "session_count": 0, "active_shell_clients": 0, "build_id": "", "sent_unix_ms": 0,
            "http_port": 47656, "station_id": id, "leaving": leaving,
            "client_protocol": lumacore_wire::CLIENT_PROTOCOL,
        }))
        .unwrap()
    }

    fn heard(t: &mut BTreeMap<String, Entry>, id: &str, leaving: bool, at: Instant) {
        apply_beacon(t, &beacon(id, leaving), "192.0.2.40".into(), 47656, at);
    }

    /// Resolved over mDNS (as `browse_mdns` does).
    fn resolved(t: &mut BTreeMap<String, Entry>, id: &str) {
        let entry = t.entry(id.to_string()).or_default();
        entry.mdns = Some(format!("desk1 {id}._lumacore._tcp.local."));
        entry.server.get_or_insert_with(|| Server {
            id: id.into(),
            name: "desk1".into(),
            host: "192.0.2.40".into(),
            port: 47656,
            version: String::new(),
            show: None,
            kind: "Server".into(),
            wrong_version: false,
            not_answering_s: None,
        });
    }

    #[test]
    fn a_fresh_beacon_is_listed_green() {
        let (mut t, start) = (BTreeMap::new(), Instant::now());
        heard(&mut t, "a", false, start);
        let l = list(&t, start + Duration::from_millis(800));
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].not_answering_s, None);
    }

    #[test]
    fn a_late_beacon_is_amber_with_its_age() {
        let (mut t, start) = (BTreeMap::new(), Instant::now());
        heard(&mut t, "a", false, start);
        resolved(&mut t, "a");
        assert_eq!(list(&t, start + Duration::from_millis(1600))[0].not_answering_s, Some(2));
        assert_eq!(list(&t, start + Duration::from_millis(3100))[0].not_answering_s, Some(3));
        // A new beacon: green again.
        heard(&mut t, "a", false, start + Duration::from_secs(3));
        assert_eq!(list(&t, start + Duration::from_millis(3200))[0].not_answering_s, None);
    }

    #[test]
    fn no_beacon_for_4_s_is_gone_even_while_mdns_lists_it() {
        let (mut t, start) = (BTreeMap::new(), Instant::now());
        heard(&mut t, "a", false, start);
        resolved(&mut t, "a");
        assert!(list(&t, start + Duration::from_millis(3900)).len() == 1);
        assert!(list(&t, start + Duration::from_secs(4)).is_empty());
    }

    #[test]
    fn a_leaving_beacon_removes_it_at_once_until_it_beacons_again() {
        let (mut t, start) = (BTreeMap::new(), Instant::now());
        heard(&mut t, "a", false, start);
        resolved(&mut t, "a");
        heard(&mut t, "a", true, start + Duration::from_millis(100));
        assert!(list(&t, start + Duration::from_millis(100)).is_empty());
        // mDNS resolving it again doesn't bring it back; a beacon does.
        resolved(&mut t, "a");
        assert!(list(&t, start + Duration::from_millis(200)).is_empty());
        heard(&mut t, "a", false, start + Duration::from_secs(10));
        assert_eq!(list(&t, start + Duration::from_secs(10)).len(), 1);
    }

    #[test]
    fn mdns_only_stays_listed_while_its_record_lives() {
        let (mut t, start) = (BTreeMap::new(), Instant::now());
        resolved(&mut t, "b");
        let l = list(&t, start + Duration::from_secs(60));
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].not_answering_s, None);
        t.get_mut("b").unwrap().mdns = None; // goodbye
        assert!(list(&t, start + Duration::from_secs(61)).is_empty());
    }
}
