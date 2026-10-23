// ███████╗███████╗███████╗██████╗
// ██╔════╝██╔════╝██╔════╝██╔══██╗
// █████╗  █████╗  █████╗  ██║  ██║
// ██╔══╝  ██╔══╝  ██╔══╝  ██║  ██║
// ██║     ███████╗███████╗██████╔╝
// ╚═╝     ╚══════╝╚══════╝╚═════╝
// FILE: feed
//! A live feed a page can subscribe on `/ws/feeds` (decisions §30 "12b2", "12d"): its name, State
//! or Samples, whether it works before a login, the parameters a `sub` takes, its default period
//! and its help line. Pure data: the HTTP engine declares them as catalog rows and binds each to
//! its builder by name (decisions §34 21).
//!
//! Owns:          FeedDecl, FeedKind
//! Doesn't own:   building a feed (lumacore-http), the socket (lumacore-http)
//! Entry points:  FeedDecl, FeedKind
//! Decisions:     §30 "12b2", "12d", §34 21

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// State (latest wins, `rev` + diffs) or Samples (every item, with a `seq`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedKind {
    State,
    Samples,
}

impl FeedKind {
    /// As the `hello` names it.
    pub fn label(self) -> &'static str {
        match self {
            Self::State => "state",
            Self::Samples => "samples",
        }
    }
}

/// One feed as the catalog holds it (the `hello` lists them).
#[derive(Debug)]
pub struct FeedDecl {
    /// Its name on the socket (`health.render`).
    pub name: &'static str,
    pub kind: FeedKind,
    /// Works before a login (the lock screen).
    pub open: bool,
    /// The parameters a `sub` takes besides `since` / `epoch` / `rev` / `period_ms`.
    pub params: &'static [&'static str],
    /// The send period when the page asks for none (ms).
    pub period_ms: u64,
    pub doc: &'static str,
}
