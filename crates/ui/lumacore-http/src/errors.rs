//! Error codes of the feed socket (`/ws/feeds`, decisions §30 "12d
//! hardening": every `sub` is answered `subbed` or `nosub` with a code).
//! `Info Error E-FEED-…` reads this catalog (listed in the station's
//! the one catalog through `lumacore_http::register` when the HTTP engine is built in, decisions §34 21).

use lumacore_tree::ErrorInfo;

/// An unknown feed name.
pub const UNKNOWN_FEED: &str = "E-FEED-001";
/// The feed needs a login and the socket has none.
pub const LOGIN_REQUIRED: &str = "E-FEED-002";
/// The subscription's parameters are missing or wrong.
pub const BAD_PARAMS: &str = "E-FEED-003";
/// The socket holds the most subscriptions it may.
pub const TOO_MANY: &str = "E-FEED-004";
/// The client doesn't read fast enough (closes the socket).
pub const TOO_SLOW: &str = "E-FEED-005";
/// An `auth` with a token the station doesn't accept.
pub const AUTH_REFUSED: &str = "E-FEED-006";
/// A message that isn't a feed op.
pub const BAD_MESSAGE: &str = "E-FEED-007";

/// The feed socket's catalog.
pub const ERRORS: &[ErrorInfo] = &[
    ErrorInfo {
        code: UNKNOWN_FEED,
        title: "No such feed",
        explain: "A page subscribed a feed the station doesn't publish. The socket's `hello` lists every feed (the feed registry in `websocket/feeds.rs`); a page from another LumaCore version reloads.",
    },
    ErrorInfo {
        code: LOGIN_REQUIRED,
        title: "Feed needs a login",
        explain: "Only feeds marked open (the station: users, presence, show status for the lock screen) work before a login. Send `auth` with a login token on the socket; the subscription is kept by the page and sent again after `authed`. A logout or a revoked token ends every feed that needs a login with this code.",
    },
    ErrorInfo {
        code: BAD_PARAMS,
        title: "Feed parameters are wrong",
        explain: "The subscription lacks a parameter the feed needs (a pool name, an object number, a screen) or an op lacks its `id`. The `hello` lists each feed's parameters.",
    },
    ErrorInfo {
        code: TOO_MANY,
        title: "Too many subscriptions",
        explain: "One feed socket holds at most 64 subscriptions (one page). Close windows the page no longer shows, or reuse a subscription id (a `sub` with an id in use replaces it).",
    },
    ErrorInfo {
        code: TOO_SLOW,
        title: "Feed client too slow",
        explain: "The page didn't read its feeds fast enough: its send queue filled, or stayed over the peak for 5 s. The station closes the socket (close code 4008); the page reconnects and resumes (samples by `since`, states by `rev`). A slow page never holds anything back for others: a state it hasn't taken is replaced by the newest.",
    },
    ErrorInfo {
        code: AUTH_REFUSED,
        title: "Login token refused",
        explain: "The `auth` op carried a token the station doesn't accept (expired, revoked by a logout, or from another station). The socket stays as it was; log in again.",
    },
    ErrorInfo {
        code: BAD_MESSAGE,
        title: "Not a feed message",
        explain: "The feed socket takes JSON ops only: `auth`, `sub` and `unsub` (see docs/dev/api/http-api.md).",
    },
];

#[cfg(test)]
mod tests {
    #[test]
    fn catalog_is_valid() {
        lumacore_tree::error_code::check_catalog(super::ERRORS).unwrap();
    }
}
