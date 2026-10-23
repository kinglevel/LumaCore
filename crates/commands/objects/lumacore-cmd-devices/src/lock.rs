// ██╗      ██████╗  ██████╗██╗  ██╗
// ██║     ██╔═══██╗██╔════╝██║ ██╔╝
// ██║     ██║   ██║██║     █████╔╝
// ██║     ██║   ██║██║     ██╔═██╗
// ███████╗╚██████╔╝╚██████╗██║  ██╗
// ╚══════╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝
// FILE: lock

//! DeskLock (F9 / Pause) locks the device a line came from; a locked device runs only the lock
//! keys (R1-41). `On DeskLock Station` locks every device, and every device that connects later
//! (R1-42). NormalTheme (Meow+F9) shows the factory look on this device.
//!
//! Owns:          DeskLock (this device, or every device of the station), NormalTheme, the lock gate.
//! Doesn't own:   the device table and unlocking (lumacore-state device_table), the toggle shape (the kernel's devices).
//! Entry points:  DESK_LOCK, NORMAL_THEME, lock_gate, allowed_while_locked.
//! Decisions:     §30 "12a3", R1-41, R1-42.

use lumacore_objects::AppState;
use lumacore_objects::devices::{Switch, is_locked, known, locked_refusal, may_unlock, unlock};
use lumacore_objects::errors::render_code;
use lumacore_objects::objects::{ObjectDef, Verb};
use lumacore_objects::text::render_success;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// A locked device may still run exactly the lock keys (R1-41): the
/// `DeskLock` / `NormalTheme` object bare, or after `On` / `Off` / `Toggle`;
/// the only argument is `Station`, and only for DeskLock. One statement,
/// nothing else — words resolve through the one namespace.
/// The lock's gate (a `Hook::Gate` of the devices family): a locked device runs nothing but the
/// lock keys (decisions §30 "12a3"); the page holds keys back too, this is the station's half.
pub(crate) fn lock_gate(state: &AppState, line: &str) -> Option<String> {
    (is_locked(state) && !allowed_while_locked(line)).then(locked_refusal)
}

pub fn allowed_while_locked(line: &str) -> bool {
    use lumacore_objects::commands::registry::{self, Word};
    if line.contains(';') {
        return false;
    }
    let tokens = lumacore_objects::commands::tokenize(line);
    let words: Vec<Option<Word>> = tokens.iter().map(|t| registry::resolve(t).ok()).collect();
    let is = |w: &Option<Word>, def: &'static ObjectDef| {
        w.as_ref().is_some_and(|w| w.same(&Word::Object(def)))
    };
    let rest = match words.first() {
        Some(Some(Word::Command(c)))
            if matches!(c.object_verb, Some(Verb::On | Verb::Off | Verb::Toggle)) =>
        {
            &words[1..]
        }
        _ => &words[..],
    };
    match rest {
        [object] => is(object, &DESK_LOCK) || is(object, &NORMAL_THEME),
        [object, station] => {
            is(object, &DESK_LOCK) && is(station, &lumacore_cmd_station::STATION)
        }
        _ => false,
    }
}

/// `On|Off|Toggle DeskLock [Station]`.
pub(crate) fn desk_lock(state: &mut AppState, how: Switch, args: &[&str]) -> String {
    let station = args
        .first()
        .is_some_and(|a| lumacore_cmd_station::STATION.matches(a));
    if station {
        let on = !matches!(how, Switch::Off);
        if !on {
            return render_code(
                "Each device unlocks on its own (its key, its corner taps or its password).",
                "E-CMD-010",
            );
        }
        let by = state.effective_user_name().to_string();
        let ids: Vec<String> = state.devices.keys().cloned().collect();
        for device in state.devices.values_mut() {
            device.lock(&by);
        }
        // Devices that connect later start locked too (R1-42).
        state.station_lock.get_or_insert(by);
        lumacore_sysmon::sysmon_note(
            "Session",
            format!(
                "DeskLock: every device of this station locked ({})",
                ids.len()
            ),
        );
        return render_success(&format!("Locked {} device(s) of this station", ids.len()));
    }
    let Some(id) = state.current_device.clone() else {
        return render_code(
            "DeskLock locks a device (an app or a browser tab); this client isn't one.",
            "E-CMD-010",
        );
    };
    let device = known(state, &id);
    let on = match how {
        Switch::On => true,
        Switch::Off => false,
        Switch::Toggle => !device.locked,
    };
    if !on
        && device.locked
        && lumacore_objects::passwords::has_password(state, state.effective_user_name())
    {
        return render_code(
            format!(
                "Unlocking needs {}'s password.",
                state.effective_user_name()
            ),
            "E-LOCK-002",
        );
    }
    let by = state.effective_user_name().to_string();
    if !on {
        if let Err(e) = may_unlock(state, &id, &by) {
            return render_code(e, "E-LOCK-002");
        }
        unlock(state, &id);
        return render_success("DeskLock Off");
    }
    known(state, &id).lock(&by);
    render_success(if on { "DeskLock On" } else { "DeskLock Off" })
}

pub(crate) fn normal_theme(state: &mut AppState, how: Switch) -> String {
    let Some(id) = state.current_device.clone() else {
        return render_code(
            "NormalTheme is per device (an app or a browser tab).",
            "E-CMD-010",
        );
    };
    let device = known(state, &id);
    device.normal_theme = match how {
        Switch::On => true,
        Switch::Off => false,
        Switch::Toggle => !device.normal_theme,
    };
    render_success(if device.normal_theme {
        "NormalTheme On"
    } else {
        "NormalTheme Off"
    })
}

/// The device words are anyone's (core keys at every level, decisions §30
/// "12d"); `On DeskLock Station` locks every device of the station (Setup).
pub(crate) fn desk_rights(_: Verb, args: &[&str]) -> lumacore_objects::commands::registry::Rights {
    use lumacore_objects::commands::registry::Rights;
    if args
        .iter()
        .any(|a| lumacore_cmd_station::STATION.matches(a))
    {
        Rights::Setup
    } else {
        Rights::Guest
    }
}

lumacore_objects::device_object!(
    DESK_LOCK,
    "DeskLock",
    lumacore_objects::devices::on_station,
    desk_rights,
    "Lock this device (a desk lock): its screens show the lock surface, the station refuses \
     its lines; playback, DMX, shells and other devices run on. F9 or Pause toggles it; a user \
     with a password unlocks with it. `On DeskLock Station` locks every device of the station. \
     Station-local, never replicated, not an Oops step (decisions §30 \"12a3\").",
    "DeskLock | On DeskLock [Station] | Off DeskLock | Toggle DeskLock",
    &[
        "DeskLock               -- toggle (F9 / Pause)",
        "On DeskLock Station    -- lock every device of this station"
    ],
    desk_lock
);

lumacore_objects::device_object!(
    NORMAL_THEME,
    "NormalTheme",
    lumacore_objects::devices::on_station,
    desk_rights,
    "Show the normal theme (the factory preset, nothing of the profile's look) on this device; \
     Meow+F9, held = while held. Works while locked. `Set Station \"ThemeOverride\" Dark` makes it \
     the default for every device of this station.",
    "NormalTheme | On NormalTheme | Off NormalTheme | Toggle NormalTheme",
    &["NormalTheme    -- toggle (Meow+F9)"],
    |s, how, _| normal_theme(s, how)
);
