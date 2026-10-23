// ████████╗ ██████╗ ██████╗ ██╗ ██████╗
// ╚══██╔══╝██╔═══██╗██╔══██╗██║██╔════╝
//    ██║   ██║   ██║██████╔╝██║██║
//    ██║   ██║   ██║██╔═══╝ ██║██║
//    ██║   ╚██████╔╝██║     ██║╚██████╗
//    ╚═╝    ╚═════╝ ╚═╝     ╚═╝ ╚═════╝
// FILE: topic

//! The board's dialogs by topic (decisions 0290): a line that did what a dialog asked closes it
//! (`Consent` typed instead of pressed), and a holder's dialog about a topic is found.
//!
//! Owns:          close_topic, holds_topic.
//! Doesn't own:   the board and its answers (dialogs), what a topic means (its asker: consent).
//! Entry points:  close_topic, holds_topic.
//! Decisions:     docs/decisions/0290-meow-license-consent-on-every-login.

use lumacore_wire::DialogClosed;

use super::{Audience, Board};
use crate::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Close the dialogs about `topic` that `to` holds for `user`, as answered with `answer`.
pub fn close_topic(state: &mut AppState, to: &Audience, topic: &str, user: &str, answer: &str) {
    let ids: Vec<u64> = state
        .dialogs
        .open
        .iter()
        .filter(|(_, o)| o.to == *to && o.user.eq_ignore_ascii_case(user) && o.ask.view.topic.as_deref() == Some(topic))
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(o) = state.dialogs.open.remove(&id) {
            let title = o.ask.view.title.clone();
            let notice = DialogClosed { id: format!("d{id}"), title, by: user.to_string(), answer: answer.to_string() };
            state.dialogs.close(o.to, notice);
        }
    }
}

/// Does `to` hold an open dialog about `topic`?
pub fn holds_topic(board: &Board, to: &Audience, topic: &str) -> bool {
    board.open.values().any(|o| o.to == *to && o.ask.view.topic.as_deref() == Some(topic))
}
