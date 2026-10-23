// ██████╗ ███████╗██╗   ██╗██╗ ██████╗███████╗███████╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██╔════╝██╔════╝
// ██║  ██║█████╗  ██║   ██║██║██║     █████╗  ███████╗
// ██║  ██║██╔══╝  ╚██╗ ██╔╝██║██║     ██╔══╝  ╚════██║
// ██████╔╝███████╗ ╚████╔╝ ██║╚██████╗███████╗███████║
// ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝ ╚═════╝╚══════╝╚══════╝
// FILE: devices

//! The station's devices (decisions §30 "12a3 hardening", R1-42): each device's DeskLock,
//! normal theme and screen config — station-local, never replicated, not an Oops step.
//!
//! Owns:          DeviceState, Devices, the lock checks, unlocking, a device's status.
//! Doesn't own:   the DeskLock / NormalTheme / Device objects and what a locked device may run (`crate::devices`).
//! Entry points:  Devices, DeviceState, status, is_locked, device_locked, locked_refusal, may_unlock, unlock, known.
//! Decisions:     §30 "12a3", §34 15.

use std::collections::BTreeMap;

use crate::AppState;
use crate::errors::render_code;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One device's state on this station.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceState {
    /// Who it is on this station: a fresh GUID while the station runs.
    pub meta: lumacore_base::Identity,
    pub locked: bool,
    pub normal_theme: bool,
    /// The screen config it shows; None = Follow user.
    pub screen_config: Option<u32>,
    /// Who switched the lock on (R1-42), `None` while unlocked: only they or
    /// Admin unlock it (`may_unlock`).
    pub locked_by: Option<String>,
}

impl DeviceState {
    pub fn lock(&mut self, by: &str) {
        if !self.locked {
            self.locked_by = Some(by.to_string());
        }
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
        self.locked_by = None;
    }
}

/// Every device this station knows (by id), station-local.
pub type Devices = BTreeMap<String, DeviceState>;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The status a client's device gets (in its `Status`).
pub fn status(state: &AppState) -> lumacore_wire::DeviceStatus {
    let id = state.current_device.clone().unwrap_or_default();
    let d = state.devices.get(&id).cloned().unwrap_or_default();
    let theme_override = state.root.station_settings.theme_override.clone();
    lumacore_wire::DeviceStatus {
        id,
        locked: d.locked,
        normal_theme: d.normal_theme || theme_override.is_some(),
        theme_override,
        screen_config: d.screen_config,
    }
}

/// Whether the device the current line came from is locked (one the station
/// hasn't seen yet is, while the station is locked: R1-42).
pub fn is_locked(state: &AppState) -> bool {
    state.current_device.as_ref().is_some_and(|id| device_locked(state, id))
}

/// The reply to anything a locked device asks for (E-LOCK-001): its lines
/// and its REST writes (the layout PUT) alike.
pub fn locked_refusal() -> String {
    render_code("This device is locked (DeskLock): unlock it first.", "E-LOCK-001")
}

/// Whether device `id` is locked (one the station hasn't seen yet is, while
/// the station is locked: R1-42).
pub fn device_locked(state: &AppState, id: &str) -> bool {
    match state.devices.get(id) {
        Some(d) => d.locked,
        None => state.station_lock.is_some(),
    }
}

/// Whether `user` may unlock `device` (decisions §30 "12a3", R1-42): the user
/// who locked it, or Admin. The password is checked by the caller.
pub fn may_unlock(state: &AppState, device: &str, user: &str) -> Result<(), String> {
    match state.devices.get(device).and_then(|d| d.locked_by.as_deref()) {
        Some(locker) if !locker.eq_ignore_ascii_case(user) && !crate::passwords::is_admin_user(user) => {
            Err(format!("Locked by {locker}: only {locker} or Admin can unlock it"))
        }
        _ => Ok(()),
    }
}

/// Unlock the device after its user's password was checked (`/api/unlock`).
/// The station lock ends with the last locked device (R1-42).
pub fn unlock(state: &mut AppState, device: &str) {
    if let Some(d) = state.devices.get_mut(device) {
        d.unlock();
    }
    if !state.devices.values().any(|d| d.locked) {
        state.station_lock = None;
    }
}

/// The device `id`, made when it is new: while the station is locked
/// (`On DeskLock Station`), a new device starts locked (R1-42).
pub fn known<'a>(state: &'a mut AppState, id: &str) -> &'a mut DeviceState {
    let station_lock = state.station_lock.clone();
    state.devices.entry(id.to_string()).or_insert_with(|| {
        let mut device = DeviceState::default();
        if let Some(by) = station_lock {
            device.lock(&by);
        }
        device
    })
}

