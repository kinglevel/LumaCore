// ███████╗████████╗ █████╗ ████████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██║██╔═══██╗████╗  ██║
// ███████╗   ██║   ███████║   ██║   ██║██║   ██║██╔██╗ ██║
// ╚════██║   ██║   ██╔══██║   ██║   ██║██║   ██║██║╚██╗██║
// ███████║   ██║   ██║  ██║   ██║   ██║╚██████╔╝██║ ╚████║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: station

//! Per-station local settings, persisted next to the show library (the
//! startup show file name and friends).
//!
//! Stored in `library/station_settings.json`. Station-wide, not part of a show:
//! which show to load at startup, and the last show that was loaded or saved.
//! `lumacore-server --station B` keeps them in `library/stations/B/` instead,
//! so two servers can run side by side on one machine (decisions §21 "6b").
//!
//! Owns:          the station's local settings file: startup show, last show, the station folder.
//! Doesn't own:   the station settings' properties (objects settings_props).
//! Entry points:  use_station, load, save, dir.
//! Decisions:     §30 "12d part 4", §34 25.10.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// `--station <name>`: this server's own station directory.
static STATION: OnceLock<String> = OnceLock::new();

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Run as station `name` (before the settings are loaded): its files live in
/// `library/stations/<name>/` and it is called `name` until renamed.
pub fn use_station(name: &str) {
    let _ = STATION.set(name.to_string());
}

/// The directory of this station's own files.
pub fn dir() -> String {
    match STATION.get() {
        Some(name) => format!("library/stations/{name}"),
        None => "library".to_string(),
    }
}

fn path() -> String {
    format!("{}/station_settings.json", dir())
}

use lumacore_wire::StartupShow;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalSettings {
    #[serde(default)]
    pub startup_show: StartupShow,
    /// Name of the last show loaded or saved.
    #[serde(default)]
    pub last_show: Option<String>,
    /// Stable id of this station (discovery, thin clients remember servers by it).
    #[serde(default)]
    pub station_id: String,
    /// Secret that signs login tokens; persisted so tokens survive a restart.
    #[serde(default)]
    pub token_secret: String,
    /// The station's sACN CID: one per station, stable (decisions §11).
    #[serde(default)]
    pub sacn_cid: String,
    /// Network name (lowercase) → this station's interface (decisions §11).
    #[serde(default)]
    pub networks: std::collections::BTreeMap<String, String>,
    /// Asset root name (lowercase) → this station's folder (§8).
    #[serde(default)]
    pub asset_roots: std::collections::BTreeMap<String, String>,
    /// The normal theme for every device of this station (§30 "12a3").
    #[serde(default)]
    pub theme_override: Option<String>,
    /// Send output after losing the session (§10).
    #[serde(default)]
    pub output_when_alone: bool,
    /// LumaSession settings of this station (decisions §11 "Menus,
    /// LumaSession and staged patch"); None = defaults.
    #[serde(default)]
    pub station_name: Option<String>,
    #[serde(default)]
    pub station_priority: Option<lumacore_tree::StationPriority>,
    #[serde(default)]
    pub lumasession: Option<String>,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub invite: Option<bool>,
    #[serde(default)]
    pub online: Option<bool>,
    #[serde(default)]
    pub services: std::collections::BTreeMap<String, lumacore_tree::NicServices>,
    #[serde(default)]
    pub output_held: bool,
    /// Discovery on the Alternative multicast group (§21 "6c").
    #[serde(default)]
    pub multicast_alternative: bool,
    /// Backup cleanup (decisions §30 "12d part 5"): days versions and
    /// recovery copies are kept (None = 30) and the size cap in MB over all
    /// of them (None = 500).
    #[serde(default)]
    pub recovery_keep_days: Option<u32>,
    #[serde(default)]
    pub recovery_cap_mb: Option<u32>,
    /// This station's drives (decisions §38): id → the volume it is bound to.
    #[serde(default)]
    pub drives: std::collections::BTreeMap<String, lumacore_tree::DriveMemory>,
    /// Login tokens ended by logout: signature → expiry (unix s). Kept until
    /// the token would have expired (decisions §19).
    #[serde(default)]
    pub revoked_tokens: std::collections::BTreeMap<String, u64>,
    /// Set when the settings file exists but can't be read or parsed (R1-14):
    /// the station runs on defaults in memory and never writes over that
    /// file (it may be the only copy of the station id, CID and NIC map).
    #[serde(skip)]
    pub unreadable: Option<String>,
}

impl LocalSettings {
    /// Parse a settings file (pure: no defaults, no disk).
    pub fn parse(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }

    /// Load the settings, creating the station id and token secret on first
    /// start. A file that exists but doesn't parse is never written over:
    /// the station runs on defaults in memory and the System Monitor says so.
    pub fn load() -> Self {
        Self::load_from(&path())
    }

    fn load_from(file: &str) -> Self {
        let read = match crate::drive::read_to_string(file) {
            Ok(text) => Self::parse(&text).map(Some),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.to_string()),
        };
        let mut s = match read {
            Ok(s) => s.unwrap_or_default(),
            Err(e) => {
                lumacore_sysmon::sysmon_early(
                    "Station",
                    format!(
                        "[ERROR] station settings {file} can't be read ({e}); this station runs on \
                         defaults (new id, CID, no network mapping) and leaves the file as it is — \
                         fix or remove it and restart"
                    ),
                );
                Self { unreadable: Some(e), ..Default::default() }
            }
        };
        let mut created = false;
        if s.station_name.is_none()
            && let Some(name) = STATION.get()
        {
            s.station_name = Some(name.clone());
            created = true;
        }
        if s.station_id.is_empty() {
            s.station_id = uuid::Uuid::new_v4().to_string();
            created = true;
        }
        if s.token_secret.is_empty() {
            s.token_secret = format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            );
            created = true;
        }
        if uuid::Uuid::parse_str(&s.sacn_cid).is_err() {
            s.sacn_cid = uuid::Uuid::new_v4().to_string();
            created = true;
        }
        if created {
            s.save_to(file);
        }
        s
    }

    /// Put the station's output settings into `Root.StationSettings`.
    pub fn apply_to(&self, station: &mut lumacore_tree::StationSettings) {
        if let Ok(cid) = uuid::Uuid::parse_str(&self.sacn_cid) {
            station.sacn_cid = cid;
        }
        station.networks = self.networks.clone();
        station.asset_roots = self.asset_roots.clone();
        station.theme_override = self.theme_override.clone();
        station.output_when_alone = self.output_when_alone;
        if let Some(name) = self.station_name.clone().filter(|n| !n.is_empty()) {
            station.station_name = name;
        }
        if let Some(p) = self.station_priority {
            station.station_priority = p;
        }
        if let Some(s) = self.lumasession.clone().filter(|s| !s.is_empty()) {
            station.lumasession = s;
        }
        station.location = self.location.clone();
        station.invite = self.invite.unwrap_or(true);
        station.online = self.online.unwrap_or(true);
        station.services = self.services.clone();
        station.drives = self.drives.clone();
        station.output_held = self.output_held;
        station.multicast_alternative = self.multicast_alternative;
    }

    /// Take the station-local settings from `Root.StationSettings` (after a
    /// change) and save them.
    pub fn store_from(&mut self, station: &lumacore_tree::StationSettings) {
        self.networks = station.networks.clone();
        self.asset_roots = station.asset_roots.clone();
        self.theme_override = station.theme_override.clone();
        self.output_when_alone = station.output_when_alone;
        self.station_name = Some(station.station_name.clone());
        self.station_priority = Some(station.station_priority);
        self.lumasession = Some(station.lumasession.clone());
        self.location = station.location.clone();
        self.invite = Some(station.invite);
        self.online = Some(station.online);
        self.services = station.services.clone();
        self.drives = station.drives.clone();
        self.output_held = station.output_held;
        self.multicast_alternative = station.multicast_alternative;
        self.save();
    }

    pub fn save(&self) {
        // Unit tests never touch the station's real settings file.
        if cfg!(test) {
            return;
        }
        self.save_to(&path());
    }

    fn save_to(&self, file: &str) {
        if let Some(why) = &self.unreadable {
            log::warn!("station settings: not written, {file} couldn't be read at start ({why})");
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Some(parent) = std::path::Path::new(file).parent() {
                let _ = crate::drive::create_dir_all(parent);
            }
            if let Err(e) = crate::drive::write_atomic(file, json) {
                log::warn!("station settings: cannot write {file}: {e}");
            }
        }
    }

    /// The show to load at startup, if any.
    pub fn startup_show_name(&self) -> Option<String> {
        match &self.startup_show {
            StartupShow::Last => self.last_show.clone(),
            StartupShow::None => None,
            StartupShow::Named(name) => Some(name.clone()),
        }
    }
}

/// The station-local settings an Oops step can put back (decisions §29
/// "11c": interfaces, network → NIC, Online, name, priority, …) — the fields
/// `LocalSettings::store_from` keeps. The destructure is exhaustive: a new
/// `StationSettings` field is placed here or skipped on purpose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StationImage {
    networks: std::collections::BTreeMap<String, String>,
    asset_roots: std::collections::BTreeMap<String, String>,
    theme_override: Option<String>,
    output_when_alone: bool,
    station_name: String,
    station_priority: lumacore_tree::StationPriority,
    lumasession: String,
    location: String,
    invite: bool,
    online: bool,
    services: std::collections::BTreeMap<String, lumacore_tree::NicServices>,
    output_held: bool,
    multicast_alternative: bool,
}

impl StationImage {
    pub fn of(station: &lumacore_tree::StationSettings) -> Self {
        let lumacore_tree::StationSettings {
            sacn_cid: _,
            nic_addresses: _,
            data_dir: _,
            station_name,
            station_priority,
            networks,
            asset_roots,
            theme_override,
            output_when_alone,
            lumasession,
            location,
            invite,
            online,
            services,
            output_held,
            multicast_alternative,
            // The drive table is the watcher's, never an Oops step (§38).
            drives: _,
        } = station;
        Self {
            networks: networks.clone(),
            asset_roots: asset_roots.clone(),
            theme_override: theme_override.clone(),
            output_when_alone: *output_when_alone,
            station_name: station_name.clone(),
            station_priority: *station_priority,
            lumasession: lumasession.clone(),
            location: location.clone(),
            invite: *invite,
            online: *online,
            services: services.clone(),
            output_held: *output_held,
            multicast_alternative: *multicast_alternative,
        }
    }

    pub fn apply_to(self, station: &mut lumacore_tree::StationSettings) {
        station.networks = self.networks;
        station.asset_roots = self.asset_roots;
        station.theme_override = self.theme_override;
        station.output_when_alone = self.output_when_alone;
        station.station_name = self.station_name;
        station.station_priority = self.station_priority;
        station.lumasession = self.lumasession;
        station.location = self.location;
        station.invite = self.invite;
        station.online = self.online;
        station.services = self.services;
        station.output_held = self.output_held;
        station.multicast_alternative = self.multicast_alternative;
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
