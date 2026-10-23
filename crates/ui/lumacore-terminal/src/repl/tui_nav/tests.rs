// ███╗   ██╗ █████╗ ██╗   ██╗    ████████╗███████╗███████╗████████╗███████╗
// ████╗  ██║██╔══██╗██║   ██║    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██╔██╗ ██║███████║██║   ██║       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║╚██╗██║██╔══██║╚██╗ ██╔╝       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║ ╚████║██║  ██║ ╚████╔╝        ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═══╝╚═╝  ╚═╝  ╚═══╝         ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: nav tests

//! NAV on the demo rig (decisions 0291): its rows are `ls`'s, every object opens, the
//! inspector follows the cursor, the copy keys queue a copy, marked rows make one range, a
//! bookmark finds its object after it moved, the frame draws the tree and the inspector.
//!
//! Owns:          the tests of tui_nav.rs and its drawing.
//! Doesn't own:   the rows (tree_rows' tests), the inspector's rows (lumacore-views' tests).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crossterm::event::{KeyCode, KeyEvent as CtKeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::prelude::Terminal;

use super::{Focus, range_line};
use crate::core::AppState;
use crate::repl::TuiApp;

fn key(app: &mut TuiApp, state: &mut AppState, code: KeyCode) {
    let mods = match code {
        KeyCode::Char(c) if c.is_ascii_uppercase() || "?".contains(c) => KeyModifiers::SHIFT,
        _ => KeyModifiers::NONE,
    };
    app.handle_key(CtKeyEvent::new(code, mods), state);
}

fn keys(app: &mut TuiApp, state: &mut AppState, text: &str) {
    for c in text.chars() {
        key(app, state, KeyCode::Char(c));
    }
}

/// The rig, the shell at `path`, NAV on.
fn nav_at(path: &str) -> (TuiApp, AppState) {
    let mut state = crate::core::demo::fresh_state();
    let mut app = TuiApp::new(&state);
    app.execute_command(&mut state, &format!("cd {path}"));
    app.execute_command(&mut state, "Shortcuts Navigator");
    (app, state)
}

/// What `ls` prints below the shell's place, without colours or connectors.
fn ls_rows(app: &mut TuiApp, state: &mut AppState) -> Vec<String> {
    app.output.clear();
    app.execute_command(state, "ls");
    app.output
        .iter()
        .skip(1)
        .map(|l| crate::core::text::strip_ansi(l).trim_start_matches(['├', '└', '─', ' ']).to_string())
        .collect()
}

fn select(app: &mut TuiApp, path: &str) {
    app.nav.cursor = app.nav.entries.iter().position(|e| e.row.path == path).unwrap_or_else(|| panic!("no {path}"));
}

/// NAV's rows are `ls`'s rows, for these places on the rig.
#[test]
fn nav_rows_are_ls_rows() {
    for path in ["/", "ShowData", "ShowData.DataPools", "ShowData.DataPools.1", "ShowData.DataPools.1.Groups", "ShowData.Devices.Dmx.Patch"] {
        let (mut app, mut state) = nav_at(path);
        let nav: Vec<String> = app.nav.entries.iter().map(|e| e.row.text()).collect();
        state.shortcut_mode = crate::core::app_state::ShortcutMode::Keyboard;
        assert_eq!(nav, ls_rows(&mut app, &mut state), "at {path}");
    }
}

/// DataPool 1 opens by its number (its label is never a path), and every row below it opens
/// in the inspector with its path.
#[test]
fn every_object_opens() {
    let (mut app, mut state) = nav_at("ShowData.DataPools");
    select(&mut app, "Root.ShowData.DataPools.1");
    key(&mut app, &mut state, KeyCode::Right);
    assert!(app.nav.entries.iter().any(|e| e.row.path == "Root.ShowData.DataPools.1.Groups"), "DataPool 1 opened");
    select(&mut app, "Root.ShowData.DataPools.1.Groups");
    key(&mut app, &mut state, KeyCode::Right);
    for i in 0..app.nav.entries.len() {
        app.nav.cursor = i;
        app.nav_inspect(&mut state);
        let path = app.nav.entries[i].row.path.clone();
        assert_eq!(app.nav.inspector.path, path);
        assert!(app.nav.inspector.rows.iter().any(|r| r.name == "Path" && r.value == path), "{path}");
    }
}

/// The inspector follows the cursor; Tab reaches Slot, Kind, GUID, Path; Enter on a property
/// puts its Set line on the command line and NAV's keys rest while the user types the value.
#[test]
fn the_inspector_follows_and_enter_gives_the_set_line() {
    let (mut app, mut state) = nav_at("ShowData.DataPools.1.Groups");
    keys(&mut app, &mut state, "1");
    assert_eq!(app.nav.inspector.target.as_deref(), Some("Group 1"));
    key(&mut app, &mut state, KeyCode::Tab);
    assert_eq!(app.nav.focus, Focus::Inspector);
    assert_eq!(app.nav.focused_row().map(|r| r.name.as_str()), Some("Slot"));
    let mode = app.nav.visible().iter().position(|&i| app.nav.inspector.rows[i].name == "Mode").expect("Mode");
    app.nav.icursor = mode;
    key(&mut app, &mut state, KeyCode::Enter);
    assert_eq!(app.get_input(), "Set Group 1 \"Mode\" ");
    keys(&mut app, &mut state, "Additive");
    assert_eq!(app.get_input(), "Set Group 1 \"Mode\" Additive", "typing reaches the line");
    assert!(app.nav.log.iter().any(|l| l.contains("Set Group 1 \"Mode\"")), "{:?}", app.nav.log);
}

/// yg / yp / ya queue the GUID, the path and the target for the user's clipboard, and the log
/// says what was copied; y in the inspector copies the focused row.
#[test]
fn copy_keys_queue_a_copy() {
    let (mut app, mut state) = nav_at("ShowData.Devices.Dmx.Patch.Fixtures");
    keys(&mut app, &mut state, "101");
    let guid = state.root.show_data.fixture_guid(101).unwrap().to_string();
    keys(&mut app, &mut state, "ygypya");
    let copies: Vec<(String, String)> = app
        .ui_outbox
        .iter()
        .filter_map(|e| match e {
            lumacore_wire::UiEvent::CopyText { label, text } => Some((label.clone(), text.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        copies,
        vec![
            ("GUID".to_string(), guid.clone()),
            ("path".to_string(), "Root.ShowData.Devices.Dmx.Patch.Fixtures.101".to_string()),
            ("target".to_string(), "Fixture 101".to_string()),
        ]
    );
    assert!(app.nav.log.iter().any(|l| *l == format!("copied GUID: {guid}")), "{:?}", app.nav.log);
}

/// Space marks rows: one range line on the command line, NAV's keys still on.
#[test]
fn marked_rows_make_one_range() {
    assert_eq!(range_line(&[("Group".into(), 3), ("Group".into(), 1), ("Group".into(), 2), ("Group".into(), 7)]), "Group 1 Thru 3 + 7");
    assert_eq!(range_line(&[("Group".into(), 1), ("Fixture".into(), 7)]), "Group 1 + Fixture 7");
    let (mut app, mut state) = nav_at("ShowData.DataPools.1.Groups");
    keys(&mut app, &mut state, "1   ");
    keys(&mut app, &mut state, "7 ");
    assert_eq!(app.get_input(), "Group 1 Thru 3 + 7");
    key(&mut app, &mut state, KeyCode::Esc);
    assert_eq!(app.get_input(), "", "Esc drops the marks and their line");
    assert_eq!(state.shortcut_mode, crate::core::app_state::ShortcutMode::Navigator, "and stays in NAV");
}

/// A bookmark is the object's GUID: after the group moved to another number, ' + letter still
/// finds it. It is saved on the user in the show.
#[test]
fn a_bookmark_survives_renumbering() {
    let (mut app, mut state) = nav_at("ShowData.DataPools.1.Groups");
    keys(&mut app, &mut state, "5");
    let guid = app.nav.current().and_then(|e| e.row.guid).expect("Group 5");
    keys(&mut app, &mut state, "mb");
    let user = state.effective_user_name().to_string();
    assert_eq!(state.root.show_data.user(&user).and_then(|u| u.bookmarks.get('b')), Some(guid));
    app.execute_command(&mut state, "Move Group 5 At 300");
    keys(&mut app, &mut state, "1'b");
    assert_eq!(app.nav.current().map(|e| e.row.path.as_str()), Some("Root.ShowData.DataPools.1.Groups.300"));
}

/// The frame: the tree's rows and the inspector's identity rows, the gesture log, the help.
#[test]
fn the_frame_draws_tree_inspector_and_log() {
    let (mut app, mut state) = nav_at("ShowData.DataPools.1.Groups");
    keys(&mut app, &mut state, "1yn");
    app.nav_refresh(&mut state);
    let frame = render(&mut app, &state);
    assert!(frame.contains("1  \"Photon\"  (Group)") || frame.contains("1  \"Photon\""), "{frame}");
    assert!(frame.contains("GUID") && frame.contains("Slot") && frame.contains("─ Properties"), "{frame}");
    assert!(frame.contains("copied name: Photon"), "{frame}");
    keys(&mut app, &mut state, "?");
    assert!(render(&mut app, &state).contains("NAV keys"));
}

fn render(app: &mut TuiApp, state: &AppState) -> String {
    let mut terminal = Terminal::new(TestBackend::new(140, 40)).expect("test backend");
    let _ = terminal.draw(|frame| app.draw(frame, state));
    let buffer = terminal.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}
