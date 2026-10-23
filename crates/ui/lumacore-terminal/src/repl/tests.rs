// ██████╗ ███████╗██████╗ ██╗         ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██╔════╝██╔══██╗██║         ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██████╔╝█████╗  ██████╔╝██║            ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══██╗██╔══╝  ██╔═══╝ ██║            ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║  ██║███████╗██║     ███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═╝╚══════╝╚═╝     ╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: repl tests

//! Tests of `repl`.
//!
//! Owns:          the unit tests of repl.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §23 "Step 8", §30 "12b"

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use crossterm::event::{KeyCode, KeyEvent as CtKeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;

fn render_plain(app: &mut TuiApp, state: &AppState, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test backend");
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

/// A terminal (the CLI, the station's console) gets the station's dialog
/// as a centered window over the dimmed shell, in the user's theme
/// (decisions §30 "12b"); keys answer it the same way any client does.
#[test]
fn a_dialog_is_a_centered_window_in_the_users_theme_and_keys_answer_it() {
    let mut state = AppState::new();
    let mut app = TuiApp::new(&state);
    app.execute_command(&mut state, "Debug");
    let frame = render_plain(&mut app, &state, 100, 30);
    let title = frame.lines().position(|l| l.contains(" Debug ")).expect("window title");
    let row = frame.lines().nth(title).unwrap();
    let left = row.find('╭').expect("rounded border");
    let right = row.chars().count() - row.chars().rev().position(|c| c == '╮').unwrap() - 1;
    assert!((left as i32 - (99 - right) as i32).abs() <= 2, "centered: {row}");
    assert!(frame.contains("1 Enable") && frame.contains("2 Cancel"), "{frame}");
    // Truecolor in the profile theme: Dark's accent gold on the border.
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let _ = terminal.draw(|f| app.draw(f, &state));
    let gold = ratatui::style::Color::Rgb(0x9a, 0x7a, 0x08);
    assert!(terminal.backend().buffer().content.iter().any(|c| c.fg == gold));
    // A Daylight profile: the ground is light.
    state.root.show_data.user_profiles.values_mut().map(std::sync::Arc::make_mut).for_each(|p| p.settings.theme.preset = "Daylight".into());
    let _ = terminal.draw(|f| app.draw(f, &state));
    let ground = terminal.backend().buffer()[(0, 0)].bg;
    assert!(matches!(ground, ratatui::style::Color::Rgb(r, _, _) if r > 150), "{ground:?}");
    // Enter presses the primary button: the line runs, the window goes.
    app.handle_key(CtKeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &mut state);
    assert!(state.debug_mode);
    assert!(!render_plain(&mut app, &state, 100, 30).contains("╭"), "closed");
    // Esc = Cancel.
    app.execute_command(&mut state, "Debug");
    app.handle_key(CtKeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut state);
    assert!(state.debug_mode, "cancel ran nothing");
}

/// The suggestion rows a terminal's TUI draws (its live list), read off
/// the rendered screen: the title column of each row inside the box.
fn tui_suggestion_titles(app: &mut TuiApp, state: &AppState) -> Vec<String> {
    let screen = render_plain(app, state, 120, 40);
    let mut in_box = false;
    let mut out = Vec::new();
    for line in screen.lines() {
        if line.contains(" Suggestions ") && line.contains("Tab takes") {
            in_box = true;
            continue;
        }
        if !in_box {
            continue;
        }
        let Some(at) = line.find('│') else { break };
        let row: String = line[at + '│'.len_utf8()..].chars().skip(3).take(24).collect();
        if row.trim().is_empty() {
            break;
        }
        out.push(row.trim_end().to_string());
    }
    out
}

/// The CLI's TUI and the web command line bar show the same suggestions
/// (decisions §30 "12b2": one engine in `lumacore-suggest`): what a terminal
/// draws while typing equals, row for row, what the station answers a
/// UI client's `Suggest` packet for the same line.
#[test]
fn the_terminal_and_the_bar_show_the_same_suggestions() {
    let mut state = crate::core::demo::fresh_state();
    state.user = "Admin".into();
    // Keyboard mode: letters type (in PRG they are shortcuts).
    state.shortcut_mode = crate::core::app_state::ShortcutMode::Keyboard;
    let lines = ["Gr", "Gro", "Store Gr", "Group 1 At ", "Fix", "lo", "Set Win", "cd Ro"];
    let mut compared = 0;
    for line in lines {
        // The terminal (lumacore-cli): typed key by key, the live list.
        let mut app = TuiApp::new(&state);
        for c in line.chars() {
            app.handle_key(CtKeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE), &mut state);
        }
        let tui = tui_suggestion_titles(&mut app, &state);
        // The web page: the shell packet a UI client sends for the bar.
        let mut remote = command_line::RemoteCommandLine::new("Admin".into(), &mut state, 120, 40);
        let mut client = command_line::ClientView::default();
        let web: Vec<String> = remote
            .suggest(&mut client, &mut state, 1, Some(line.to_string()), None, lumacore_wire::Surface::Bar)
            .items
            .iter()
            .map(|i| i.title.chars().take(24).collect::<String>().trim_end().to_string())
            .collect();
        if tui.is_empty() {
            // The terminal hides a list that only repeats what was typed.
            assert!(web.len() <= 1, "'{line}': the terminal shows nothing, the bar {web:?}");
            continue;
        }
        assert!(!web.is_empty(), "'{line}': the terminal shows {tui:?}, the bar nothing");
        let n = tui.len().min(web.len());
        assert_eq!(tui[..n], web[..n], "'{line}': terminal vs bar");
        compared += 1;
    }
    assert!(compared >= 5, "only {compared} lines had a list to compare");
}

/// PRG's letters are LumaCore's keys (decisions 0295): a key types its word, the same key again
/// takes the next row of its cycle in place, a press that runs completes the line, and anything
/// typed between starts over.
#[test]
fn prg_keys_cycle_their_word_in_place_and_a_run_completes_the_line() {
    let mut state = crate::core::demo::fresh_state();
    state.user = "Admin".into();
    state.shortcut_mode = crate::core::app_state::ShortcutMode::Programmer;
    let mut app = TuiApp::new(&state);
    let key = |app: &mut TuiApp, state: &mut AppState, code: KeyCode, mods: KeyModifiers| app.handle_key(CtKeyEvent::new(code, mods), state);
    let ch = |app: &mut TuiApp, state: &mut AppState, c: char| key(app, state, KeyCode::Char(c), KeyModifiers::NONE);
    ch(&mut app, &mut state, 'f');
    ch(&mut app, &mut state, '1');
    assert_eq!(app.get_input(), "Fixture 1");
    ch(&mut app, &mut state, 'e');
    assert_eq!(app.get_input(), "Fixture 1 Edit ");
    ch(&mut app, &mut state, 'e');
    assert_eq!(app.get_input(), "Fixture 1 EditSetting ");
    ch(&mut app, &mut state, 'e');
    assert_eq!(app.get_input(), "Fixture 1 Edit ", "it wraps");
    key(&mut app, &mut state, KeyCode::Char('a'), KeyModifiers::ALT);
    key(&mut app, &mut state, KeyCode::Char('a'), KeyModifiers::ALT);
    assert_eq!(app.get_input(), "Fixture 1 Edit Label ");
    // A dot glues to its number; a second dot runs Zero with the line.
    app.set_input("");
    ch(&mut app, &mut state, 'f');
    ch(&mut app, &mut state, '1');
    ch(&mut app, &mut state, '.');
    assert_eq!(app.get_input(), "Fixture 1.");
    ch(&mut app, &mut state, '.');
    assert_eq!(app.get_input(), "", "the line ran");
    assert!(app.output.iter().any(|l| l.contains("Fixture 1 Zero")), "{:?}", app.output.iter().rev().take(3).collect::<Vec<_>>());
    // Inside a quote every key types.
    app.set_input("Label Group 3 \"");
    ch(&mut app, &mut state, 'e');
    assert_eq!(app.get_input(), "Label Group 3 \"e");
    // Space is Go only on an empty line; with words it types.
    app.set_input("Group 3");
    ch(&mut app, &mut state, ' ');
    assert_eq!(app.get_input(), "Group 3 ");
    // KBD: the letter types.
    state.shortcut_mode = crate::core::app_state::ShortcutMode::Keyboard;
    app.set_input("");
    ch(&mut app, &mut state, 'e');
    assert_eq!(app.get_input(), "e");
}

#[test]
fn meow_f9_toggles_the_normal_theme_of_the_device() {
    let mut state = AppState::new();
    state.current_device = Some("tab-1".into());
    state.devices.entry("tab-1".into()).or_default();
    let mut app = TuiApp::new(&state);
    // A browser's Shift+F9 — Shift is the default Meow key (decisions §30
    // "12b2"; wire bits: Shift 1, Alt 2, Ctrl 4).
    let web = tui_types::to_crossterm(&lumacore_wire::RemoteShellKey {
        code: lumacore_wire::RemoteShellCode::F(9),
        modifiers: lumacore_wire::shell::modifiers::SHIFT,
        repeat: None,
    });
    assert_eq!(web.modifiers, KeyModifiers::SHIFT);
    app.handle_key(web, &mut state);
    assert!(state.devices["tab-1"].normal_theme, "{:?}", app.output.last());
    app.handle_key(CtKeyEvent::new(KeyCode::F(9), KeyModifiers::NONE), &mut state);
    assert!(state.devices["tab-1"].locked);
    // Works while locked (decisions §30 "12a3").
    app.handle_key(CtKeyEvent::new(KeyCode::F(9), KeyModifiers::SHIFT), &mut state);
    assert!(!state.devices["tab-1"].normal_theme, "{:?}", app.output.last());
}

#[test]
fn typing_sho_renders_completion_popup() {
    let mut state = AppState::new();
    state.shortcut_mode = crate::core::app_state::ShortcutMode::Keyboard;
    let mut app = TuiApp::new(&state);

    for ch in ['s', 'h', 'o'] {
        app.handle_key(
            CtKeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
            &mut state,
        );
    }

    assert!(
        app.completion_menu_visible(),
        "completion popup should be active after typing 'sho'"
    );

    let rendered = render_plain(&mut app, &state, 100, 24);
    assert!(
        rendered.contains("Suggestions"),
        "expected popup title in rendered frame, got:\n{rendered}"
    );
    assert!(
        rendered.to_lowercase().contains("show"),
        "expected 'show' suggestion in rendered frame, got:\n{rendered}"
    );
}

/// The prompt shrinks step by step on a narrow line and never loses its
/// `>` (decisions §30 "12e hardening").
#[test]
fn the_prompt_shrinks_on_a_narrow_line() {
    let f = |w| fit_prompt("admin", "desk1", "DataPools.1.Groups", w);
    assert_eq!(f(80), "[admin@desk1] DataPools.1.Groups> ");
    assert_eq!(f(44), "DataPools.1.Groups> ");
    assert_eq!(f(25), "‹s.1.Groups> ");
    assert_eq!(f(16), "> ");
    for w in 1..100 {
        let p = f(w);
        assert!(p.ends_with("> "), "{w}: {p}");
        assert!(p == "> " || w as usize - p.chars().count() >= 12.min(w as usize), "{w}: {p}");
    }
}

/// The line scrolls sideways so the cursor stays in view.
#[test]
fn the_line_scrolls_to_keep_the_cursor_in_view() {
    assert_eq!(input_offset(10, 10, 20), 0, "fits");
    for (len, w) in [(40, 13), (100, 25), (30, 5)] {
        for cursor in 0..=len {
            let off = input_offset(len, cursor, w);
            assert!(cursor >= off && cursor < off + w, "{len} {w} {cursor} {off}");
            assert!(off + w <= len + 1, "no empty room past the end");
        }
    }
}

/// A 25-column bar shows `>`, the end of a long line and the `‹` mark —
/// never a blank row.
#[test]
fn a_narrow_bar_shows_the_prompt_and_the_end_of_a_long_line() {
    let state = AppState::new();
    let mut app = TuiApp::new(&state);
    app.textarea.insert_str("Group 3 Thru 7 At 50 Fade 2");
    let frame = render_plain(&mut app, &state, 25, 10);
    let line = frame.lines().find(|l| l.contains("Fade 2")).unwrap_or_else(|| panic!("{frame}"));
    assert!(line.contains("> ") && line.contains('‹'), "{line}");
}
