// ████████╗██╗   ██╗██╗    ████████╗ █████╗ ██████╗
// ╚══██╔══╝██║   ██║██║    ╚══██╔══╝██╔══██╗██╔══██╗
//    ██║   ██║   ██║██║       ██║   ███████║██████╔╝
//    ██║   ██║   ██║██║       ██║   ██╔══██║██╔═══╝
//    ██║   ╚██████╔╝██║       ██║   ██║  ██║██║
//    ╚═╝    ╚═════╝ ╚═╝       ╚═╝   ╚═╝  ╚═╝╚═╝
// FILE: tui tap

//! A tapped UI object on the command line: a pool cell, a view button, a tag's square. With a
//! verb pending it completes the line, on an empty line it runs the object's action; a tag arms
//! `Assign Tag <path> At` and the line stays armed for the next cell (decisions 0297).
//!
//! Owns:          TuiApp::handle_tap, the armed tag's tap rules and its `armed_tag` event.
//! Doesn't own:   what an armed line is (lumacore_cmd_tags::armed), the lines a tap runs.
//! Entry points:  TuiApp::handle_tap, TuiApp::armed_event.
//! Decisions:     §30 "12b", docs/decisions/0287-view-bar-any-object-pool-windows, docs/decisions/0297-tag-squares-and-picker.

use lumacore_wire::UiEvent;

use super::TuiApp;
use crate::core::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl TuiApp {
    /// Object tap: with a pending command on the line (`Store`, `Delete`,
    /// `Label` …) the tapped object completes it and it executes
    /// (`Store` + tap → `Store ViewButton 4`); on an empty line the object's
    /// default action runs (`ViewButton 4` = call/recall). Tap-and-hold stores.
    pub(super) fn handle_tap(&mut self, state: &mut AppState, tap: &crate::Tap) {
        self.completion = None;
        let input = self.get_input();
        let input = input.trim_end();
        let (object, hold, action) = (tap.object.as_str(), tap.hold, tap.action.as_deref());
        // A dotted id stands in for the number (a tag's path, decisions 0297).
        let no = tap.id.clone().unwrap_or_else(|| tap.no.to_string());
        if !hold && self.tap_tag(state, input, object, &no) {
            return;
        }

        // A UI client (it has a screen) gets the Store View dialog — a
        // station ask (decisions §30 "12b") — instead of an immediate store;
        // terminal-only sessions store directly.
        use crate::core::commands::registry::{self, Word};
        let is_view = crate::core::objects::find(object).is_some_and(|o| {
            std::ptr::eq(o, &crate::core::commands::views::VIEW)
                || std::ptr::eq(o, &crate::core::commands::views::VIEW_BUTTON)
        });
        let pending_store = matches!(
            registry::resolve(input),
            Ok(Word::Command(c)) if c.object_verb == Some(crate::core::objects::Verb::Store)
        );
        let has_ui = state.current_live_screen.is_some();
        if is_view && has_ui && (hold || pending_store) {
            if !input.is_empty() {
                self.snapshot_input();
                self.set_input("");
            }
            let button =
                crate::core::objects::find(object).is_some_and(|o| std::ptr::eq(o, &crate::core::commands::views::VIEW_BUTTON));
            match crate::core::commands::views::store_ask(state, button, tap.no) {
                Ok(ask) => {
                    crate::core::dialogs::open(state, ask);
                }
                Err(e) => self.add_output(&crate::core::errors::render_code(&e, "E-CMD-010")),
            }
            return;
        }

        // A pending Move / Copy / Assign takes the tapped object, then
        // waits for its target (`Move Group 3` + tap 7 → `… At 7`), the usual console behaviour.
        let first = input.split_whitespace().next().unwrap_or("");
        let wants_target = matches!(
            registry::resolve(first),
            Ok(Word::Command(c)) if matches!(
                c.object_verb,
                Some(crate::core::objects::Verb::Move
                    | crate::core::objects::Verb::Copy
                    | crate::core::objects::Verb::Assign)
            )
        );
        let words = input.split_whitespace().count();
        let has_at = input.split_whitespace().any(|w| w.eq_ignore_ascii_case("at"));
        if hold {
            self.run_ui_command(state, &format!("store {object} {no}"));
        } else if input.is_empty() {
            let line = match action {
                None => format!("{object} {no}"),
                Some(template) => template.replace("{obj}", &format!("{object} {no}")),
            };
            if !line.trim().is_empty() {
                self.run_ui_command(state, &line);
            }
        } else if wants_target && words == 1 {
            self.snapshot_input();
            self.set_input(&format!("{input} {object} {no}"));
        } else if wants_target && !has_at {
            // A target of another kind names itself (`Assign Sequence 3` + a view button tap →
            // `… At ViewButton 6`, decisions 0287); the same kind is a bare number.
            let pending = input.split_whitespace().nth(1).and_then(crate::core::objects::find);
            let tapped = crate::core::objects::find(object);
            let target = match (pending, tapped) {
                (Some(p), Some(t)) if !std::ptr::eq(p, t) => format!("{} {no}", t.keyword),
                _ => no.to_string(),
            };
            self.snapshot_input();
            self.set_input(&format!("{input} At {target}"));
            self.submit_command(state);
        } else {
            self.snapshot_input();
            self.set_input(&format!("{input} {object} {no}"));
            self.submit_command(state);
        }
    }

    /// The armed tag's rules (decisions 0297). With `Assign` / `Unassign` alone on the line a tag
    /// arms it (`Assign Tag 1.2 At`); armed, another tag switches the tag and any other object
    /// finishes the line, which runs and comes back armed for the next. True when the tap is done.
    fn tap_tag(&mut self, state: &mut AppState, input: &str, object: &str, no: &str) -> bool {
        use crate::core::commands::registry::{self, Word};
        let is_tag = matches!(registry::resolve(object), Ok(Word::Object(o)) if std::ptr::eq(o, &lumacore_cmd_tags::TAG));
        let verb = input.split_whitespace().next().unwrap_or("");
        if lumacore_cmd_tags::arming(input).is_some() {
            self.snapshot_input();
            if is_tag {
                let tag_word = input.split_whitespace().nth(1).unwrap_or(object);
                self.set_input(&format!("{verb} {tag_word} {no} At "));
            } else {
                self.set_input(&format!("{input} {object} {no}"));
                self.submit_command(state);
                self.set_input(&format!("{input} "));
            }
            return true;
        }
        let tagging = matches!(
            registry::resolve(verb),
            Ok(Word::Command(c)) if matches!(c.object_verb, Some(crate::core::objects::Verb::Assign | crate::core::objects::Verb::Unassign))
        );
        if is_tag && tagging && input.split_whitespace().count() == 1 {
            self.snapshot_input();
            self.set_input(&format!("{input} {object} {no} At "));
            return true;
        }
        false
    }

    /// The `armed_tag` event when the line's armed tag (or what carries it) changed since the
    /// last one: the pools mark its carriers while the line waits (decisions 0297).
    pub(super) fn armed_event(&mut self, state: &AppState) -> Option<UiEvent> {
        let input = self.get_input();
        let now = lumacore_cmd_tags::arming(&input).and_then(|_| lumacore_cmd_tags::armed(state, &input));
        if now == self.armed_tag {
            return None;
        }
        self.armed_tag = now.clone();
        Some(match now {
            Some(a) => UiEvent::ArmedTag { tag: a.path, colour: a.colour, carriers: a.carriers.iter().map(uuid::Uuid::to_string).collect() },
            None => UiEvent::ArmedTag { tag: String::new(), colour: String::new(), carriers: Vec::new() },
        })
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
