// ██╗   ██╗██╗███████╗██╗    ██╗    ██████╗  █████╗ ██████╗
// ██║   ██║██║██╔════╝██║    ██║    ██╔══██╗██╔══██╗██╔══██╗
// ██║   ██║██║█████╗  ██║ █╗ ██║    ██████╔╝███████║██████╔╝
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║    ██╔══██╗██╔══██║██╔══██╗
//  ╚████╔╝ ██║███████╗╚███╔███╔╝    ██████╔╝██║  ██║██║  ██║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝     ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
// FILE: view_bar

//! The view bar (decisions 0287): one screen's view buttons of the user's own profile, for the
//! `viewbar` feed. Each button names what it holds — kind, number, name, the DataPool it lives in
//! — or that its object is gone; the page draws them in the kind's colour and sends a tap as
//! `ViewButton <n>` (the server writes it out as the object's line).
//!
//! Owns:          view (the `viewbar` feed's content).
//! Doesn't own:   the buttons and what they hold (`lumacore_objects::commands::views::buttons`).
//! Entry points:  view.
//! Decisions:     docs/decisions/0287-view-bar-any-object-pool-windows.

use lumacore_tree::ShowData;
use serde_json::{Value, json};

use lumacore_objects::commands::views::buttons;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The bar of the user's profile screen `screen` (the profile's number of a live screen):
/// `{screen, profile, buttons: [{slot, kind, no, name, datapool, missing}]}`.
pub fn view(show: &ShowData, user: &str, screen: u32) -> Value {
    let Some(profile) = show.profile_for_user(user) else {
        return json!({ "screen": screen, "buttons": [] });
    };
    let rows: Vec<Value> = buttons::bar(show, profile, screen)
        .into_iter()
        .map(|b| match b.held {
            Some(h) => json!({
                "slot": b.slot,
                "kind": b.kind.keyword(),
                "no": h.no,
                "name": h.name,
                "datapool": h.datapool,
            }),
            None => json!({ "slot": b.slot, "kind": b.kind.keyword(), "missing": true }),
        })
        .collect();
    json!({ "screen": screen, "profile": profile.meta.identity.name, "buttons": rows })
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
