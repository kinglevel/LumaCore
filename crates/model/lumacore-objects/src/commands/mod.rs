//! Command dispatch table for the LumaCore REPL.
//!
//! Each command handler takes (&mut AppState, &[&str]) and returns a String output.
//! The dispatch function maps the first token (verb) to the appropriate handler.

// ── Existing submodules ──
/// The verbs on the object door (decisions §35).
pub mod door;
pub mod attributes;
pub mod control;
pub mod edit;
pub mod label;
/// Live lines under the Devices draft (decisions §34 "34g4 patch hardened" 8).
pub mod live_draft;
pub mod edit_setting;
pub mod gdtf_library;
pub mod mvr_library;
pub mod flags;
pub mod line;
pub mod groups;
pub mod help;
pub mod hooks;
pub mod import;
pub mod info;
pub mod list;
pub mod media;
pub mod menus;
pub mod meta_paint;
pub mod oops;
pub mod output;
pub mod paint_object;
pub mod paint_slots;
pub mod patch;
pub mod playback;
pub mod profiles;
pub mod registry;
pub mod screens;
pub mod selections;
pub mod set;
pub mod set_rows;
pub mod tokens;
pub mod typed;
pub mod universe;
pub mod users;
pub mod views;
pub mod windows;
pub mod window_presets;
pub mod ui_files;

// ── New submodules (extracted from mod.rs) ──
mod claim;
mod dispatch;
pub mod navigate;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod default_object_tests;
#[cfg(test)]
mod media_tests;
#[cfg(test)]
mod oops_tests;
#[cfg(test)]
mod patch_draft_tests;
#[cfg(test)]
mod relative_tests;
#[cfg(test)]
mod quoting_tests;
#[cfg(test)]
mod flags_tests;
#[cfg(test)]
mod window_settings_tests;
#[cfg(test)]
mod window_bar_tests;
#[cfg(test)]
mod view_ask_tests;
#[cfg(test)]
mod security_tests;
#[cfg(test)]
mod rights_tests;
#[cfg(test)]
mod menus_tests;
#[cfg(test)]
mod stations_tests;
#[cfg(test)]
mod merge_tests;
#[cfg(test)]
mod groups_tests;
#[cfg(test)]
pub mod playback_tests;
#[cfg(test)]
mod protocol_tests;
pub mod run;
#[cfg(test)]
pub(crate) mod test_words;
pub mod screenshot;
pub mod session;
pub mod consent;
pub mod settings;
pub mod show;
pub mod sysmon;
#[cfg(test)]
mod word_tests;
#[cfg(test)]
mod users_tests;

// ── Re-exports ──
// Public items that external code depends on.
pub use dispatch::dispatch;
pub use dispatch::{planned_message, undo_of_line};
pub use dispatch::record_change;
pub use registry::{completion_keywords, verb_suggestions};
#[cfg(test)]
use crate::claims::Claims;

// Items used by sibling submodules (e.g. list.rs references super::handle_list_settings).
pub use claim::list_claims;
pub use settings::handle_list_settings;

// ─── Word Resolution ─────────────────────────────────────────────────────────
//
// Commands and objects share one namespace (registry.rs); there are no
// hardcoded verb lists here.

/// Resolve what the user typed to the canonical lowercase name of a command
/// (`"at"`) or object (`"fixture"`). See [`registry::resolve`].
pub fn resolve_verb(input: &str) -> Result<String, String> {
    registry::resolve(input).map(|word| match word {
        registry::Word::Command(c) => c.verb.to_string(),
        registry::Word::Object(o) => o.keyword.to_lowercase(),
    })
}

/// The words of a command line, respecting double-quoted strings (lenient:
/// an unterminated quote runs to the end). Dispatch uses the strict lexer in
/// [`tokens`], which reports the error instead.
///
/// `Patch "My Fixture" At 1.001` → ["Patch", "My Fixture", "At", "1.001"]
pub fn tokenize(line: &str) -> Vec<String> {
    tokens::words(&tokens::lex_lenient(line))
}

/// Parse `universe.address` dot notation (e.g., "4.001", "4.1", "1.1").
///
/// Returns (universe_id, start_address) — address is 1-indexed.
pub fn parse_universe_address(s: &str) -> Option<(u32, u16)> {
    let parts: Vec<&str> = s.splitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }
    let universe: u32 = parts[0].parse().ok()?;
    let address: u16 = parts[1].parse().ok()?;
    if address == 0 || address > 512 {
        return None;
    }
    Some((universe, address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use crate::DispatchResult;
    use lumacore_fixtures::Fixture;
    use uuid::Uuid;

    // Re-import extracted items for test access
    use super::dispatch::dispatch;

    /// Fixture 1 and Group 1 exist (claims name real objects, §21 "6d").
    fn with_fixture_and_group() -> AppState {
        let mut state = AppState::new();
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(None, Uuid::new_v4(), Uuid::nil(), Uuid::nil(), Uuid::nil(), 1, 1),
        );
        dispatch(&mut state, "Fixture 1");
        dispatch(&mut state, "Store Group 1");
        dispatch(&mut state, "Clear");
        state
    }

    #[test]
    fn bare_unclaim_releases_all_normal_claims() {
        let mut state = with_fixture_and_group();
        state.claim("Devices").unwrap();
        state.claim("Fixture 1").unwrap();

        let result = dispatch(&mut state, "unclaim");
        match result {
            DispatchResult::Output(s) => {
                assert!(
                    s.contains("Released 2 claims"),
                    "Expected release summary: {s}"
                )
            }
            _ => panic!("Expected Output"),
        }

        assert!(!state.is_claimed("Devices"));
        assert!(!state.is_claimed("Fixture 1"));
    }

    #[test]
    fn claim_supports_multiple_object_handles() {
        let mut state = with_fixture_and_group();

        let result = dispatch(&mut state, "claim Root.ShowData.Devices fixture 1 group 1");
        match result {
            DispatchResult::Output(s) => {
                assert!(s.contains("Claimed"), "Expected success summary: {s}")
            }
            _ => panic!("Expected Output"),
        }

        assert!(state.is_claimed("Devices"));
        assert!(state.is_claimed("Fixture 1"));
        assert!(state.is_claimed("Group 1"));
        // Claims name objects by GUID: one that doesn't exist can't be claimed.
        assert!(state.claim("Group 9").unwrap_err().contains("doesn't exist"));
    }

    #[test]
    fn set_clearbuffer_toggles() {
        let mut state = AppState::new();
        assert!(!state.clear_buffer_active());

        let out = match dispatch(&mut state, "Set UserProfile \"ClearBuffer\" On") {
            DispatchResult::Output(s) => s.text,
            _ => String::new(),
        };
        assert!(state.clear_buffer_active(), "{out}");

        dispatch(&mut state, "Set UserProfile \"ClearBuffer\" Off");
        assert!(!state.clear_buffer_active());
    }

    #[test]
    fn set_descriptions_toggles() {
        let mut state = AppState::new();
        // Admin profile defaults to show_descriptions: false
        assert!(!state.show_descriptions());

        dispatch(&mut state, "Set UserProfile \"ShowDescriptions\" On");
        assert!(state.show_descriptions());

        dispatch(&mut state, "Set UserProfile \"ShowDescriptions\" Off");
        assert!(!state.show_descriptions());
    }

    #[test]
    fn set_names_its_target() {
        // Bare `Set` and the target-less forms are gone (decisions §33 16, 3c3): the reply says
        // the owner-first line.
        let mut state = AppState::new();
        let reply = |state: &mut AppState, line: &str| match dispatch(state, line) {
            DispatchResult::Output(s) => crate::text::strip_ansi(&s),
            _ => panic!("Expected Output"),
        };
        assert!(reply(&mut state, "Set").contains("E-CMD-006"));
        let out = reply(&mut state, "Set ClearBuffer On");
        assert!(out.contains("Set UserProfile \"ClearBuffer\" On") && out.contains("E-CMD-006"), "{out}");
        assert!(!state.clear_buffer_active());
    }

    #[test]
    fn completion_keywords_includes_registry_commands() {
        let keywords = completion_keywords();
        // Spot-check key commands are present
        assert!(keywords.contains(&"set".to_string()), "Missing set");
        assert!(keywords.contains(&"store".to_string()), "Missing store");
        assert!(keywords.contains(&"help".to_string()), "Missing help");
    }

    #[test]
    fn completion_keywords_includes_aliases() {
        let keywords = completion_keywords();
        assert!(
            keywords.contains(&"?".to_string()),
            "Missing ? alias for help"
        );
        assert!(
            keywords.contains(&"hiss".to_string()),
            "Missing hiss alias"
        );
        assert!(
            keywords.contains(&"clearterm".to_string()),
            "Missing clearterm alias"
        );
    }

    #[test]
    fn verb_suggestions_offer_each_command_once() {
        let log = verb_suggestions("log");
        assert_eq!(
            log.iter()
                .filter(|s| s.eq_ignore_ascii_case("logout"))
                .count(),
            1,
            "{log:?}"
        );
        assert_eq!(
            log.iter()
                .filter(|s| s.eq_ignore_ascii_case("login"))
                .count(),
            1,
            "{log:?}"
        );
        assert!(
            !log.iter().any(|s| s.eq_ignore_ascii_case("logoff")),
            "alias shown next to its command: {log:?}"
        );
        assert!(
            log.contains(&"Logout".to_string()),
            "display name expected: {log:?}"
        );
        // An alias is offered when only it matches.
        assert_eq!(verb_suggestions("logof"), vec!["logoff".to_string()]);
        // No command twice in the whole list (any casing).
        let all = verb_suggestions("");
        let mut seen = std::collections::HashSet::new();
        for s in &all {
            assert!(seen.insert(s.to_lowercase()), "duplicate suggestion {s}");
        }
    }

    #[test]
    fn completion_keywords_includes_display_names() {
        let keywords = completion_keywords();
        // Commands with display names different from primary name
        assert!(
            keywords.contains(&"EditSetting".to_string()),
            "Missing EditSetting display name"
        );
    }

    // ── resolve_verb tests ──────────────────────────────────────────────────

    #[test]
    fn resolve_verb_exact_match() {
        assert_eq!(resolve_verb("fixture").unwrap(), "fixture");
        assert_eq!(resolve_verb("list").unwrap(), "list");
        assert_eq!(resolve_verb("help").unwrap(), "help");
    }

    #[test]
    fn resolve_verb_case_insensitive_exact() {
        assert_eq!(resolve_verb("Fixture").unwrap(), "fixture");
        assert_eq!(resolve_verb("LIST").unwrap(), "list");
        assert_eq!(resolve_verb("HELP").unwrap(), "help");
    }

    #[test]
    fn resolve_verb_prefix_fix_to_fixture() {
        assert_eq!(resolve_verb("Fix").unwrap(), "fixture");
        assert_eq!(resolve_verb("fix").unwrap(), "fixture");
        assert_eq!(resolve_verb("fixtu").unwrap(), "fixture");
    }

    #[test]
    fn resolve_verb_single_letter_a_to_at() {
        assert_eq!(resolve_verb("a").unwrap(), "at");
    }

    #[test]
    fn resolve_verb_single_letter_d_to_delete() {
        assert_eq!(resolve_verb("d").unwrap(), "delete");
    }

    #[test]
    fn resolve_verb_short_co_to_copy() {
        assert_eq!(resolve_verb("co").unwrap(), "copy");
    }

    #[test]
    fn resolve_verb_ambiguous_prefix() {
        // "un" matches multiple verbs (unwatch, unpatch, unpark, unmeow)
        let result = resolve_verb("un");
        assert!(result.is_err(), "Expected ambiguous error for 'un'");
        let msg = result.unwrap_err();
        assert!(msg.contains("Ambiguous"), "Should say Ambiguous: {msg}");
    }

    #[test]
    fn resolve_verb_unknown_with_suggestion() {
        // "hlep" is close to "help" — Levenshtein should suggest it
        let result = resolve_verb("hlep");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("help"), "Should suggest 'help': {msg}");
    }

    #[test]
    fn resolve_verb_totally_unknown() {
        let result = resolve_verb("xyzzy");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("xyzzy"),
            "Error should contain the unknown verb: {msg}"
        );
    }

    // ── Semicolon separator tests ────────────────────────────────────────────

    #[test]
    fn split_semicolons_two_commands() {
        let parts = tokens::statements("list; help");
        assert_eq!(parts, vec!["list".to_string(), "help".to_string()]);
    }

    #[test]
    fn split_semicolons_single_command() {
        let parts = tokens::statements("list");
        assert_eq!(parts, vec!["list".to_string()]);
    }

    #[test]
    fn split_semicolons_preserves_quotes() {
        // Semicolons inside quotes are NOT separators
        let parts = tokens::statements(r#"echo "hello; world""#);
        assert_eq!(parts.len(), 1);
        assert!(parts[0].contains("hello; world"));
    }

    #[test]
    fn dispatch_semicolon_two_outputs() {
        let mut state = AppState::new();
        let result = dispatch(&mut state, "TestEcho hello; TestEcho world");
        match result {
            DispatchResult::Output(s) => {
                assert!(s.contains("hello"), "Missing 'hello': {s}");
                assert!(s.contains("world"), "Missing 'world': {s}");
            }
            _ => panic!("Expected Output from semicolon dispatch"),
        }
    }

    // ── SYSMON tree change detection tests ────────────────────────────

    #[test]
    fn sysmon_reports_store_group() {
        let mut state = AppState::new();
        // Add a fixture and select it
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("Spot".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );
        let _ = dispatch(&mut state, "fixture 1");
        state.sysmon_pending.clear();

        let _ = dispatch(&mut state, "store group 1");
        assert!(
            !state.sysmon_pending.is_empty(),
            "sysmon_pending should contain a tree change after Store Group"
        );
        let (subsystem, message) = &state.sysmon_pending[0];
        assert_eq!(subsystem, "Show", "subsystem should be Show");
        assert!(
            message.contains("Tree changed") || message.contains("Tree modified"),
            "message should report tree change: {message}"
        );
        assert!(
            message.contains("Group") && message.contains("stored"),
            "message should describe the store result: {message}"
        );
    }

    #[test]
    fn sysmon_no_report_for_read_only_commands() {
        let mut state = AppState::new();
        state.sysmon_pending.clear();

        let _ = dispatch(&mut state, "list");
        assert!(
            state.sysmon_pending.is_empty(),
            "sysmon_pending should be empty after read-only 'list': {:?}",
            state.sysmon_pending
        );

        let _ = dispatch(&mut state, "help");
        assert!(
            state.sysmon_pending.is_empty(),
            "sysmon_pending should be empty after read-only 'help': {:?}",
            state.sysmon_pending
        );
    }

    #[test]
    fn sysmon_reports_label_modification() {
        let mut state = AppState::new();
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("OldName".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );
        state.sysmon_pending.clear();

        let _ = dispatch(&mut state, "label fixture 1 \"NewName\"");
        assert!(
            !state.sysmon_pending.is_empty(),
            "sysmon_pending should report tree modification after Label"
        );
        let (subsystem, message) = &state.sysmon_pending[0];
        assert_eq!(subsystem, "Show");
        assert!(
            message.contains("Tree modified") || message.contains("Tree changed"),
            "message should report tree change: {message}"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // ── Standalone Label command tests ───────────────────────────────────
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn label_fixture_single() {
        let mut state = AppState::new();
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("Old".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );
        let result = dispatch(&mut state, "Label Fixture 1 \"FixtureLabel\"");
        match result {
            DispatchResult::Output(s) => {
                assert!(s.contains("labeled"), "Expected success: {s}");
                assert!(s.contains("FixtureLabel"), "Expected name in output: {s}");
            }
            _ => panic!("Expected Output"),
        }
        let name = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .fixture(1)
            .unwrap()
            .meta
            .name();
        assert_eq!(name, Some("FixtureLabel"));
    }

    #[test]
    fn label_universe_single() {
        let mut state = AppState::new();
        state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .universes
            .insert(1, lumacore_fixtures::Universe::new(None));
        let result = dispatch(&mut state, "Label Universe 1 \"simpleName\"");
        match result {
            DispatchResult::Output(s) => assert!(s.contains("labeled"), "Expected success: {s}"),
            _ => panic!("Expected Output"),
        }
        let name = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .universes
            .get(&1)
            .unwrap()
            .meta
            .name();
        assert_eq!(name, Some("simpleName"));
    }

    #[test]
    fn label_universe_thru_with_enumeration() {
        let mut state = AppState::new();
        for id in 1..=10 {
            state
                .root
                .show_data
                .devices
                .dmx
                .patch
                .universes
                .insert(id, lumacore_fixtures::Universe::new(None));
        }
        let result = dispatch(&mut state, "Label Universe 1 Thru 10 \"hello 0\"");
        match result {
            DispatchResult::Output(s) => assert!(s.contains("labeled"), "Expected success: {s}"),
            _ => panic!("Expected Output"),
        }
        // The usual console behaviour: "hello 0" -> "hello 0", "hello 1", ..., "hello 9"
        for id in 1u32..=10 {
            let name = state
                .root
                .show_data
                .devices
                .dmx
                .patch
                .universes
                .get(&id)
                .unwrap()
                .meta
                .name();
            let expected = format!("hello {}", id - 1);
            assert_eq!(
                name,
                Some(expected.as_str()),
                "Universe {id} should be '{expected}'"
            );
        }
    }

    #[test]
    fn label_fixture_thru_auto_enumerate() {
        let mut state = AppState::new();
        for id in 1..=5 {
            state.root.show_data.devices.dmx.patch.insert_fid(id, Fixture::new(None, Uuid::new_v4(), uuid::Uuid::nil(), uuid::Uuid::nil(), Uuid::nil(), id as u16, 1),
            );
        }
        let result = dispatch(&mut state, "Label Fixture 1 Thru 5 \"Wash 1\"");
        match result {
            DispatchResult::Output(s) => assert!(s.contains("labeled"), "Expected success: {s}"),
            _ => panic!("Expected Output"),
        }
        // "Wash 1" base, enumerate: "Wash 1", "Wash 2", "Wash 3", "Wash 4", "Wash 5"
        for id in 1u32..=5 {
            let name = state
                .root
                .show_data
                .devices
                .dmx
                .patch
                .fixture(id)
                .unwrap()
                .meta
                .name();
            let expected = format!("Wash {id}");
            assert_eq!(name, Some(expected.as_str()), "Fixture {id}");
        }
    }

    #[test]
    fn label_empty_string_clears_name() {
        let mut state = AppState::new();
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("HasName".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );
        // An empty "" doesn't reach Label (it asks for the name, decisions §30 "12b");
        // /Clear clears.
        assert!(matches!(dispatch(&mut state, "Label Fixture 1 \"\""), DispatchResult::Ask(_)));
        let result = dispatch(&mut state, "Label Fixture 1 /Clear");
        match result {
            DispatchResult::Output(s) => assert!(s.contains("cleared"), "Expected clear: {s}"),
            _ => panic!("Expected Output"),
        }
        let name = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .fixture(1)
            .unwrap()
            .meta
            .name();
        assert_eq!(name, None);
    }

    #[test]
    fn label_strips_restricted_characters() {
        let mut state = AppState::new();
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(None, Uuid::new_v4(), uuid::Uuid::nil(), uuid::Uuid::nil(), Uuid::nil(), 1, 1),
        );
        let result = dispatch(&mut state, "Label Fixture 1 \"test$name\"");
        match result {
            DispatchResult::Output(s) => {
                assert!(s.contains("labeled"), "Expected success: {s}");
                assert!(s.contains("restricted"), "Expected stripping note: {s}");
            }
            _ => panic!("Expected Output"),
        }
        let name = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .fixture(1)
            .unwrap()
            .meta
            .name();
        assert_eq!(name, Some("testname"));
    }

    #[test]
    fn label_group_single() {
        let mut state = AppState::new();
        // Store a group first
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("F1".to_string()),
                Uuid::new_v4(),
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                1,
            ),
        );
        let _ = dispatch(&mut state, "fixture 1");
        let _ = dispatch(&mut state, "store group 1");

        let result = dispatch(&mut state, "Label Group 1 \"MyGroup\"");
        match result {
            DispatchResult::Output(s) => assert!(s.contains("labeled"), "Expected success: {s}"),
            _ => panic!("Expected Output"),
        }
        let dp_id = state.selected_datapool();
        let name = state
            .root
            .show_data
            .datapools
            .get(&dp_id)
            .unwrap()
            .groups
            .get(&1)
            .unwrap()
            .meta
            .name();
        assert_eq!(name, Some("MyGroup"));
    }

    #[test]
    fn label_name_alias_works() {
        let mut state = AppState::new();
        state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(None, Uuid::new_v4(), uuid::Uuid::nil(), uuid::Uuid::nil(), Uuid::nil(), 1, 1),
        );
        let result = dispatch(&mut state, "Name Fixture 1 \"AliasTest\"");
        match result {
            DispatchResult::Output(s) => assert!(s.contains("labeled"), "Expected success: {s}"),
            _ => panic!("Expected Output"),
        }
        let name = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .fixture(1)
            .unwrap()
            .meta
            .name();
        assert_eq!(name, Some("AliasTest"));
    }

    #[test]
    fn label_not_found_returns_error() {
        let mut state = AppState::new();
        let result = dispatch(&mut state, "Label Fixture 999 \"Ghost\"");
        match result {
            // The one resolver answers (§33 57).
            DispatchResult::Output(s) => assert!(s.contains("E-CMD-007"), "Expected error: {s}"),
            _ => panic!("Expected Output"),
        }
    }

    // ── enumerate_name unit tests ────────────────────────────────────────

    #[test]
    fn enumerate_name_basic() {
        assert_eq!(edit::enumerate_name("Wash 1", 0), "Wash 1");
        assert_eq!(edit::enumerate_name("Wash 1", 1), "Wash 2");
        assert_eq!(edit::enumerate_name("Wash 1", 4), "Wash 5");
    }

    #[test]
    fn enumerate_name_zero_based() {
        assert_eq!(edit::enumerate_name("hello 0", 0), "hello 0");
        assert_eq!(edit::enumerate_name("hello 0", 1), "hello 1");
        assert_eq!(edit::enumerate_name("hello 0", 9), "hello 9");
    }

    #[test]
    fn enumerate_name_no_number() {
        assert_eq!(edit::enumerate_name("Spot", 0), "Spot");
        assert_eq!(edit::enumerate_name("Spot", 3), "Spot");
    }

    #[test]
    fn enumerate_name_multiple_numbers_uses_last() {
        assert_eq!(edit::enumerate_name("Mac700 1", 0), "Mac700 1");
        assert_eq!(edit::enumerate_name("Mac700 1", 1), "Mac700 2");
        assert_eq!(edit::enumerate_name("Mac700 1", 9), "Mac700 10");
    }

    #[test]
    fn sanitize_label_strips_forbidden() {
        assert_eq!(edit::sanitize_label("test$name"), "testname");
        assert_eq!(edit::sanitize_label("hello.world"), "helloworld");
        assert_eq!(edit::sanitize_label("no|pipes{here}"), "nopipeshere");
        assert_eq!(edit::sanitize_label("clean"), "clean");
    }
}
