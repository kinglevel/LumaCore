// ███████╗███████╗███████╗██████╗ ██████╗  █████╗  ██████╗██╗  ██╗
// ██╔════╝██╔════╝██╔════╝██╔══██╗██╔══██╗██╔══██╗██╔════╝██║ ██╔╝
// █████╗  █████╗  █████╗  ██║  ██║██████╔╝███████║██║     █████╔╝
// ██╔══╝  ██╔══╝  ██╔══╝  ██║  ██║██╔══██╗██╔══██║██║     ██╔═██╗
// ██║     ███████╗███████╗██████╔╝██████╔╝██║  ██║╚██████╗██║  ██╗
// ╚═╝     ╚══════╝╚══════╝╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
// FILE: feedback

//! Feedback that reads the show: the surfaces a profile picks per category and level, and the
//! station statuses (blackout, park, the protocols' lines, watches, debug).
//!
//! Owns:          the feedback that reads the show: a profile's surfaces per category and level, the station statuses.
//! Doesn't own:   the feedback events (lumacore-sysmon feedback.rs).
//! Entry points:  settings_json, publish_statuses.
//! Decisions:     §30 "12b2", §34 25.10.

use lumacore_sysmon::Level;
use lumacore_sysmon::feedback::{CATEGORIES, LEVELS, TIMED_MS, default_surface, status};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Every category × level with this profile's surface (overrides on top of
/// the defaults), for `/api/feedback`.
pub fn settings_json(
    prefs: Option<&lumacore_tree::programmer::UserPreferences>,
) -> serde_json::Value {
    let mut surfaces = serde_json::Map::new();
    for c in CATEGORIES {
        for l in LEVELS {
            let key = format!("{c}.{l}");
            let v = prefs
                .and_then(|p| p.notifications.get(&key))
                .map(String::as_str)
                .unwrap_or(default_surface(l));
            surfaces.insert(key, v.into());
        }
    }
    serde_json::json!({
        "mode": prefs.map_or("timed", |p| p.notification_mode.as_str()),
        "timed_ms": TIMED_MS,
        "surfaces": surfaces,
    })
}

/// Station statuses from the server state (command line icons; a Status
/// Center): emitted when they change. Per-user parts (mode objects such as
/// Blind / Highlight, selection, walk, claims) stay in each client's status
/// snapshot.
pub fn publish_statuses(state: &crate::AppState) {
    // Blackout = the grand master switched off (decisions §16).
    let (_, grand_on) = state.root.show_data.playbacks.grand();
    status("black", !grand_on, "BO", "●", Level::Alert);
    let park = &state.root.show_data.playback.park;
    let parked = park.fixtures.len() + park.channels.len();
    status(
        "parked",
        parked > 0,
        &format!("PARK {parked}"),
        "⊘",
        Level::Info,
    );
    // One icon per protocol: lines working / lines, red on a problem (§11);
    // DMX input lines receiving add "⇣n".
    let lines = &state.root.runtime.network.lines;
    for (key, protocol) in [("sacn", "sACN"), ("artnet", "Art-Net"), ("osc", "OSC")] {
        let of: Vec<_> = lines.iter().filter(|l| l.protocol == protocol).collect();
        let active = of.iter().filter(|l| l.active).count();
        let problem = of.iter().any(|l| l.problem);
        let receiving = of
            .iter()
            .filter(|l| l.direction == "In" && l.active && protocol != "OSC")
            .count();
        let input = if receiving > 0 {
            format!(" ⇣{receiving}")
        } else {
            String::new()
        };
        status(
            key,
            !of.is_empty(),
            &format!("{protocol} {active}/{}{input}", of.len()),
            "≋",
            if problem { Level::Alert } else { Level::Info },
        );
    }
    let watches = state.watches().len();
    status(
        "watches",
        watches > 0,
        &format!("WATCH {watches}"),
        "◔",
        Level::Info,
    );
    status("debug", state.debug_mode, "DEBUG", "⚑", Level::Alert);
}
