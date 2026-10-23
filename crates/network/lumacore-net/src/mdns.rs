// ███╗   ███╗██████╗ ███╗   ██╗███████╗
// ████╗ ████║██╔══██╗████╗  ██║██╔════╝
// ██╔████╔██║██║  ██║██╔██╗ ██║███████╗
// ██║╚██╔╝██║██║  ██║██║╚██╗██║╚════██║
// ██║ ╚═╝ ██║██████╔╝██║ ╚████║███████║
// ╚═╝     ╚═╝╚═════╝ ╚═╝  ╚═══╝╚══════╝
// FILE: mdns

//! mDNS (Bonjour / DNS-SD) advertising of this station's UI engine.
//!
//! `_lumacore._tcp.local.` → the HTTP port, with TXT `id` (station id),
//! `station` (name), `shell` (shell port), `version`. Thin clients and other
//! tools browse it; LumaCore stations also send the multicast session beacon
//! (`SessionDiscoveryWire`, now carrying the HTTP port too).
//!
//! A clean shutdown withdraws the record (`Advertisement::withdraw`, mDNS
//! goodbye) so the app's servers list drops the station at once instead of
//! keeping it until the record's TTL runs out (§30 "12b2 hardening").
//!
//! Owns:          the station's mDNS record (advertise, withdraw).
//! Doesn't own:   the discovery beacon (lumacore-wire, the server's spawn), when to advertise (the server).
//! Entry points:  advertise, Advertisement::withdraw.
//! Decisions:     §30 "12b2 hardening", §34 20.6.

use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use lumacore_wire::MDNS_SERVICE as SERVICE;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How long a withdraw waits for the daemon to send its goodbye.
const WITHDRAW_WAIT: Duration = Duration::from_millis(500);

/// The station's advertised record; kept on `ServerContext` for the life of
/// the server (dropping it without `withdraw` leaves the record to expire).
pub struct Advertisement {
    daemon: ServiceDaemon,
    fullname: String,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Advertisement {
    /// Send the mDNS goodbye for the record and stop the daemon. Waits at
    /// most `WITHDRAW_WAIT` for each step (mDNS is optional; shutdown goes on).
    pub fn withdraw(self) {
        match self.daemon.unregister(&self.fullname) {
            Ok(done) => {
                let _ = done.recv_timeout(WITHDRAW_WAIT);
            }
            Err(e) => log::debug!("mDNS unregister: {e}"),
        }
        if let Ok(done) = self.daemon.shutdown() {
            let _ = done.recv_timeout(WITHDRAW_WAIT);
        }
        log::info!("mDNS: withdrew {}", self.fullname);
    }
}

/// Advertise the station. Logs and gives up on errors (mDNS is optional).
pub fn advertise(
    station: &str,
    station_id: &str,
    http_port: u16,
    shell_port: u16,
) -> Option<Advertisement> {
    let daemon = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            log::warn!("mDNS unavailable: {e}");
            return None;
        }
    };
    // DNS-SD instance and host labels: no dots.
    let label = |s: &str| s.split('.').next().unwrap_or(s).replace(' ', "-");
    let host = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "lumacore".into());
    let short_id: String = station_id.chars().take(8).collect();
    let instance = format!("{} {short_id}", label(station));
    let properties = [
        ("id", station_id.to_string()),
        ("station", station.to_string()),
        ("shell", shell_port.to_string()),
        ("version", env!("CARGO_PKG_VERSION").to_string()),
        ("kind", lumacore_wire::StationKind::Server.name().to_string()),
        ("client", lumacore_wire::CLIENT_PROTOCOL.to_string()),
    ];
    let info = match ServiceInfo::new(
        SERVICE,
        &instance,
        &format!("{}.local.", label(&host)),
        "",
        http_port,
        &properties[..],
    ) {
        Ok(info) => info.enable_addr_auto(),
        Err(e) => {
            log::warn!("mDNS service info: {e}");
            return None;
        }
    };
    let fullname = info.get_fullname().to_string();
    match daemon.register(info) {
        Ok(()) => {
            log::info!("mDNS: advertising {instance} on {SERVICE} port {http_port}");
            Some(Advertisement { daemon, fullname })
        }
        Err(e) => {
            log::warn!("mDNS register: {e}");
            None
        }
    }
}
