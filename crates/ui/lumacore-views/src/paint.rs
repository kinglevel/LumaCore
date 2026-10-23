// ██████╗  █████╗ ██╗███╗   ██╗████████╗
// ██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝
// ██████╔╝███████║██║██╔██╗ ██║   ██║
// ██╔═══╝ ██╔══██║██║██║╚██╗██║   ██║
// ██║     ██║  ██║██║██║ ╚████║   ██║
// ╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝
// FILE: paint

//! The paint picker's and the theme editor's Paint tab's data (decisions 0294), for the `paint`
//! feed: the palette (12 hues in four tones, the named tone, the greys — the station's, never a
//! client's), the viewer's theme paint slots (their colour, LumaCore's, changed, added), the
//! viewer's recent colours and the mode the picker opens on. `profile` = whose theme's slots (the
//! theme editor on another profile); none = the viewer's own.
//!
//! Owns:          view (the `paint` feed).
//! Doesn't own:   the palette and the parser (lumacore_base::paint), the slots (lumacore_tree::media).
//! Entry points:  view.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links.

use lumacore_base::paint::{GREYS, NAMED, NAMED_TONE, PALETTE};
use lumacore_tree::ShowData;
use serde_json::{Value, json};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// `{palette: {hues: [{name, tones}], named_tone, greys, named}, slots: [{name, colour, default,
/// changed, added, line, reset}], recent, mode, profile, line}`.
pub fn view(show: &ShowData, user: &str, profile: Option<u32>) -> Value {
    let own = show.profile_id_for_user(user);
    let no = profile.filter(|n| *n > 0).or(own);
    let p = no.and_then(|n| show.user_profiles.get(&n));
    let theme = p.map(|p| p.settings.theme.clone()).unwrap_or_default();
    // The lines name the profile owner first unless it is the viewer's own.
    let owner = match (no, p) {
        (Some(n), Some(p)) if Some(n) != own => format!("UserProfile \"{}\" ", p.meta.identity.name.clone().unwrap_or_default()),
        _ => String::new(),
    };
    let viewer = own.and_then(|n| show.user_profiles.get(&n));
    let slots: Vec<Value> = theme
        .slot_views()
        .into_iter()
        .map(|s| {
            json!({
                "name": s.name,
                "colour": s.colour,
                "default": s.default,
                "changed": s.changed,
                "added": s.default.is_none(),
                "line": format!("Set {owner}Theme Paint \"{}\" {{value}}", s.name),
                "reset": format!("Reset {owner}Theme Paint \"{}\"", s.name),
            })
        })
        .collect();
    json!({
        "palette": {
            "hues": PALETTE.iter().map(|(name, tones)| json!({ "name": name, "tones": tones })).collect::<Vec<_>>(),
            "named_tone": NAMED_TONE,
            "greys": GREYS,
            "named": NAMED.iter().map(|(n, h)| json!({ "name": n, "hex": h })).collect::<Vec<_>>(),
        },
        "slots": slots,
        "recent": viewer.map(|p| p.settings.preferences.paint_recent.clone()).unwrap_or_default(),
        "mode": viewer.map_or_else(|| "Palette".to_string(), |p| p.settings.preferences.paint_mode.clone()),
        // By number: `Theme` after a bare `UserProfile "…"` would read as its child.
        "mode_line": own.map(|n| format!("Set UserProfile {n} \"PaintMode\" {{value}}")),
        "profile": p.map(|p| p.meta.identity.name.clone().unwrap_or_default()),
        "add": format!("Store {owner}Theme Paint {{name}} {{value}}"),
        "reset_all": slots_reset(&owner, &theme),
    })
}

/// "Reset to LumaCore": the line putting every changed or added slot back (one Oops step each).
fn slots_reset(owner: &str, theme: &lumacore_tree::media::Theme) -> String {
    theme.slots.iter().map(|s| format!("Reset {owner}Theme Paint \"{}\"", s.name)).collect::<Vec<_>>().join("; ")
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
