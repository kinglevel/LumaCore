//  ██████╗ ██████╗ ███╗   ██╗███████╗███████╗███╗   ██╗████████╗
// ██╔════╝██╔═══██╗████╗  ██║██╔════╝██╔════╝████╗  ██║╚══██╔══╝
// ██║     ██║   ██║██╔██╗ ██║███████╗█████╗  ██╔██╗ ██║   ██║
// ██║     ██║   ██║██║╚██╗██║╚════██║██╔══╝  ██║╚██╗██║   ██║
// ╚██████╗╚██████╔╝██║ ╚████║███████║███████╗██║ ╚████║   ██║
//  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝  ╚═══╝   ╚═╝
// FILE: consent

//! The Meow License (decisions 0290): every login — the page's, the app's, `Login` on any
//! terminal, a user switch — asks the user to accept it before the login is finished. The
//! question is one server-held dialog (`dialogs`) whose buttons are lines: `Consent` finishes the
//! login, `Logout` drops it. Until then the login is pending: its client runs only the users
//! family's words (the gate, E-USER-010) and its HTTP token counts as no login at all.
//! Station-local, never replicated, never saved.
//!
//! Owns:          the license's text (the one door every client draws), the pending logins
//!                (by holder and by HTTP token), the consent dialog.
//! Doesn't own:   the gate's words (the users family), the board (dialogs), tokens (passwords).
//! Entry points:  Consents, ask, holder, begin, await_token, token_pending, token_waits, pending,
//!                consent, forget, forget_gone, forget_token.
//! Decisions:     docs/decisions/0290-meow-license-consent-on-every-login.

use std::collections::BTreeMap;

use lumacore_wire::{ButtonKind, Tone};

use crate::AppState;
use crate::dialogs::{self, Ask, Audience};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The dialog's title.
pub const TITLE: &str = "Meow License";
/// Its heading, the first line of its text.
pub const HEADING: &str = "Share all your good vibes and purrs with LumaCore";
/// Its body.
pub const BODY: &str = "By logging in you agree to share all your good vibes and purrs under the Meow License. \
Hisses prohibited. By proceeding you confirm you are at least three treats deep into a cosy evening.";
/// The warning above the buttons (an alert line).
pub const WARNING: &str = "Fully vibe-coded software. LumaCore was written with AI and comes with no reliability, \
no warranty and no promise that it works on your show. Never run anything that matters on it.";
/// The button that finishes the login (runs `Consent`).
pub const CONSENT_BUTTON: &str = "I consent — let the meow in";
/// The quiet button that drops it (runs `Logout`).
pub const NOT_ME: &str = "Not me";
/// The fine print under the buttons.
pub const FINE_PRINT: &str = "(LumaCore accepts no liability for spontaneous keyboard naps.)";
/// The dialog's topic: a client that waits for the consent (the page's surface) knows it by this.
pub const TOPIC: &str = "consent";

/// A login waiting for the consent.
#[derive(Debug, Clone)]
struct Pending {
    /// Who it is: a device (an app or a browser tab), a shell client, the station's console.
    holder: Audience,
    user: String,
    /// The signature of the HTTP token it came with, consented with it.
    token: Option<String>,
}

/// Every login on this station that waits for the Meow License (station-local).
#[derive(Debug, Default)]
pub struct Consents {
    /// Tokens from `POST /api/login` not consented yet: signature → expiry (unix s).
    tokens: BTreeMap<String, u64>,
    holders: Vec<Pending>,
    /// The waiting tokens changed since the station view was last published.
    changed: bool,
}

impl Consents {
    /// Did the waiting tokens change since this was last asked? (The station publishes its view
    /// for HTTP at once then.)
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// The signatures of the tokens that wait (the station view HTTP reads).
    pub fn token_signatures(&self) -> std::collections::BTreeSet<String> {
        self.tokens.keys().cloned().collect()
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The consent dialog for `user`: the text, the warning, `Consent` / `Logout`.
pub fn ask(user: &str) -> Ask {
    Ask::new(TITLE, format!("{HEADING}\n{BODY}"))
        .meta(format!("{user} · asks every login"), None)
        .detail(WARNING, Some(Tone::Alert))
        .button(CONSENT_BUTTON, ButtonKind::Primary, "Consent")
        .button(NOT_ME, ButtonKind::Plain, "Logout")
        .note(FINE_PRINT)
        .topic(TOPIC)
}

/// Who the running line belongs to, as a login: its device, its shell client, or the station's
/// console. None: a one-shot path (HTTP, UDP), which checks its token instead.
pub fn holder(state: &AppState) -> Option<Audience> {
    match (&state.current_device, state.current_client) {
        (Some(d), _) => Some(Audience::Device(d.clone())),
        (None, Some(c)) => Some(Audience::Client(c)),
        (None, None) if state.acting_user.is_none() => Some(Audience::Local),
        (None, None) => None,
    }
}

/// `user` logged in as `holder` (with the HTTP `token` it brought, if any): the login waits for
/// the consent, and the dialog asks for it. A login that was waiting there is replaced.
pub fn begin(state: &mut AppState, holder: Audience, user: &str, token: Option<&str>) {
    forget(state, &holder);
    let token = token.and_then(crate::passwords::token_signature).map(|(sig, _)| sig.to_string());
    state.consent.holders.push(Pending { holder: holder.clone(), user: user.to_string(), token });
    dialogs::open_for(state, ask(user), holder, user);
}

/// A token `POST /api/login` just issued: it counts as no login until a client that logged in
/// with it consents.
pub fn await_token(state: &mut AppState, token: &str) {
    let Some((sig, expiry)) = crate::passwords::token_signature(token) else {
        return;
    };
    let now = lumacore_base::clock::now_ms() / 1000;
    state.consent.tokens.retain(|_, until| *until >= now);
    state.consent.tokens.insert(sig.to_string(), expiry);
    state.consent.changed = true;
}

/// Does `token` wait for the consent? `pending` = the signatures that do (the station view).
pub fn token_pending(pending: &std::collections::BTreeSet<String>, token: &str) -> bool {
    crate::passwords::token_signature(token).is_some_and(|(sig, _)| pending.contains(sig))
}

/// Does this station hold `token` back for the consent?
pub fn token_waits(state: &AppState, token: &str) -> bool {
    crate::passwords::token_signature(token).is_some_and(|(sig, _)| state.consent.tokens.contains_key(sig))
}

/// The user whose login on the running line's holder waits for the consent, if one does.
pub fn pending(state: &AppState) -> Option<String> {
    let holder = holder(state)?;
    state.consent.holders.iter().find(|p| p.holder == holder).map(|p| p.user.clone())
}

/// The running line's holder consents: its login (and the token it came with) is finished and
/// the dialog closes. Returns the user, or None when nothing here waited.
pub fn consent(state: &mut AppState) -> Option<String> {
    let holder = holder(state)?;
    let at = state.consent.holders.iter().position(|p| p.holder == holder)?;
    let done = state.consent.holders.remove(at);
    if let Some(sig) = &done.token {
        state.consent.changed |= state.consent.tokens.remove(sig).is_some();
    }
    dialogs::close_topic(state, &holder, TOPIC, &done.user, "Consent");
    Some(done.user)
}

/// The holder logged out or went away: its waiting login (and its dialog) is dropped.
pub fn forget(state: &mut AppState, holder: &Audience) {
    let gone: Vec<Pending> = state.consent.holders.iter().filter(|p| p.holder == *holder).cloned().collect();
    state.consent.holders.retain(|p| p.holder != *holder);
    for p in gone {
        dialogs::close_topic(state, holder, TOPIC, &p.user, "Logout");
    }
}

/// Drop the waiting logins whose dialog the board forgot (their client has been gone a while,
/// `dialogs::forget_gone`); the station's console keeps its own.
pub fn forget_gone(state: &mut AppState) {
    let board = &state.dialogs;
    state
        .consent
        .holders
        .retain(|p| p.holder == Audience::Local || dialogs::holds_topic(board, &p.holder, TOPIC));
}

/// A token was revoked (`POST /api/logout`): it waits no more.
pub fn forget_token(state: &mut AppState, token: &str) {
    if let Some((sig, _)) = crate::passwords::token_signature(token) {
        state.consent.changed |= state.consent.tokens.remove(sig).is_some();
    }
}
