//! `Screenshot [Screen <n>] [User <x>]` (decisions §29 "11d"; Print
//! Screen / F11): every UI client showing the user's open screens renders its
//! screen as seen to PNG and uploads it (`POST /api/screenshots`); the files
//! go to `library/screenshots/<station>/YYYYMMDD_hhmmss_<user>_screen<n>.png`.
//!
//! The command only asks (`ScreenStore::ask_capture`) and replies at once;
//! each saved file is a System Monitor line and a notice ([`saved`]); a
//! client that stays silent for [`WAIT`] is a warning ([`tick`]).

use std::path::PathBuf;
use std::time::Duration;

use crate::AppState;
use crate::errors::render_code;
use lumacore_sysmon::feedback::{self, Importance, category};
use crate::text::render_success;

/// Where the files go, under the station's own folder.
pub const DIR: &str = "library/screenshots";

/// How long a client may take to upload its capture.
/// A page renders itself as seen: with a rig's windows in it (a pool of 77 groups, a patch of
/// 1 371 rows) a capture can take well over ten seconds (decisions §40 "40f built").
pub const WAIT: Duration = Duration::from_secs(30);

/// `Screenshot [Screen <n>] [User <x>]`.
pub fn cmd_screenshot(state: &mut AppState, args: &[&str]) -> String {
    const USAGE: &str = "Usage: Screenshot [Screen <n>] [User <name>]";
    let mut screen: Option<u32> = None;
    let mut user = state.effective_user_name().to_string();
    let mut it = args.iter();
    while let Some(word) = it.next() {
        let obj = crate::objects::find(word);
        let value = it.next().map(|v| v.trim_matches('"'));
        match (obj, value) {
            (Some(o), Some(v)) if std::ptr::eq(o, &super::screens::SCREEN) => {
                match v.parse::<u32>() {
                    Ok(n) if n > 0 => screen = Some(n),
                    _ => return render_code(format!("'{v}' is not a screen number."), "E-CMD-008"),
                }
            }
            (Some(o), Some(v)) if std::ptr::eq(o, &super::users::USER) => {
                match state.root.show_data.user(v) {
                    Some(u) => user = u.login.clone(),
                    None => {
                        return render_code(format!("User \"{v}\" doesn't exist."), "E-CMD-007");
                    }
                }
            }
            _ => return render_code(USAGE, "E-CMD-006"),
        }
    }
    let asked_by = state.effective_user_name().to_string();
    let Ok(mut store) = state.root.runtime.live_screens.write() else {
        return render_code("Screens are unavailable.", "E-CMD-010");
    };
    let screens: Vec<u32> = store
        .for_user(&user)
        .filter(|s| screen.is_none_or(|n| s.screen == n))
        .map(|s| s.no)
        .collect();
    let which = screen.map_or_else(|| "screens".to_string(), |n| format!("screen {n}"));
    if screens.is_empty() {
        return render_code(format!("{user} has no {which} open."), "E-CMD-010");
    }
    let (_, clients) = store.ask_capture(&asked_by, &screens);
    if clients == 0 {
        return render_code(
            format!("No UI client shows {user}'s {which} on this station."),
            "E-CMD-010",
        );
    }
    render_success(&format!(
        "Screenshot asked from {clients} client{} — files go to {}/",
        if clients == 1 { "" } else { "s" },
        folder(&state.root.station_settings.station_name).display()
    ))
}

/// This station's screenshot folder.
pub fn folder(station: &str) -> PathBuf {
    let safe: String = station
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    PathBuf::from(DIR).join(if safe.is_empty() {
        "station".to_string()
    } else {
        safe
    })
}

/// The file for `user`'s screen `screen` taken at `time`
/// (`YYYYMMDD_hhmmss_<user>_screen<n>.png`; `_2`, `_3` … when taken
/// again in the same second).
pub fn file_name(
    time: chrono::DateTime<chrono::Local>,
    user: &str,
    screen: u32,
    taken: impl Fn(&str) -> bool,
) -> String {
    let user: String = user
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let base = format!("{}_{user}_screen{screen}", time.format("%Y%m%d_%H%M%S"));
    let mut name = format!("{base}.png");
    let mut n = 2;
    while taken(&name) {
        name = format!("{base}_{n}.png");
        n += 1;
    }
    name
}

/// Write an uploaded capture of live screen `live` (owned by `user`,
/// screen `screen` of their config) into `dir`. Runs on the HTTP worker,
/// not a command.
pub fn write(
    dir: &std::path::Path,
    user: &str,
    screen: u32,
    png: &[u8],
) -> Result<PathBuf, String> {
    // tidy: drive-ok (an upload saved by the HTTP worker, not a command)
    lumacore_luma::drive::create_dir_all(dir)
        .map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
    let name = file_name(chrono::Local::now(), user, screen, |n| {
        lumacore_luma::drive::metadata(dir.join(n)).is_ok()
    });
    let path = dir.join(name);
    lumacore_luma::drive::write_atomic(&path, png)
        .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// A capture was saved: a System Monitor line and a notice for whoever asked.
pub fn saved(asked_by: &str, path: &str) {
    feedback::message(
        lumacore_sysmon::Level::Info,
        category::SYSTEM,
        Importance::Normal,
        "Screenshot saved",
        path,
        Some(asked_by),
    );
}

/// Captures still missing after [`WAIT`]: a warning for whoever asked.
pub fn tick(state: &mut AppState) {
    let late = match state.root.runtime.live_screens.write() {
        Ok(mut store) => store.overdue(WAIT),
        Err(_) => return,
    };
    for capture in late {
        let screens: Vec<String> = capture.screens.iter().map(u32::to_string).collect();
        feedback::message(
            lumacore_sysmon::Level::Warn,
            category::SYSTEM,
            Importance::Normal,
            "Screenshot incomplete",
            &format!(
                "no capture from a client of screen {} within {} s",
                screens.join(", "),
                WAIT.as_secs()
            ),
            Some(&capture.asked_by),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DispatchResult;

    fn run(state: &mut AppState, line: &str) -> String {
        match crate::commands::dispatch(state, line) {
            DispatchResult::Output(s) => crate::text::strip_ansi(&s),
            _ => panic!("{line}"),
        }
    }

    #[test]
    fn screenshot_asks_the_clients_viewing_the_users_screens() {
        let mut state = AppState::new();
        assert!(
            run(&mut state, "Screenshot").contains("E-CMD-010"),
            "no screen open"
        );
        let no = {
            let mut store = state.root.runtime.live_screens.write().unwrap();
            let no = store.open_screen("Admin", 1, 2, None);
            store.viewer(no, true);
            no
        };
        let asked = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let seen = asked.clone();
        state
            .root
            .runtime
            .live_screens
            .write()
            .unwrap()
            .subscribe_capture(move |r| seen.lock().unwrap().push(r.screens.clone()));
        assert!(run(&mut state, "Screenshot").contains("asked from 1 client"));
        assert!(run(&mut state, "Screenshot Screen 2").contains("asked from 1 client"));
        assert!(run(&mut state, "Screenshot Screen 1").contains("E-CMD-010"));
        assert!(run(&mut state, "Screenshot User \"Nobody\"").contains("E-CMD-007"));
        assert!(run(&mut state, "Screenshot Banana").contains("E-CMD-006"));
        assert_eq!(*asked.lock().unwrap(), vec![vec![no], vec![no]]);
    }

    #[test]
    fn files_are_named_by_time_user_and_screen() {
        use chrono::TimeZone;
        let t = chrono::Local
            .with_ymd_and_hms(2026, 10, 12, 15, 30, 12)
            .unwrap();
        assert_eq!(
            file_name(t, "Admin", 1, |_| false),
            "20261012_153012_Admin_screen1.png"
        );
        let taken = [
            "20261012_153012_Admin_screen1.png",
            "20261012_153012_Admin_screen1_2.png",
        ];
        assert_eq!(
            file_name(t, "Admin", 1, |n| taken.contains(&n)),
            "20261012_153012_Admin_screen1_3.png"
        );
        assert_eq!(
            file_name(t, "A/B", 3, |_| false),
            "20261012_153012_A_B_screen3.png"
        );
        assert_eq!(folder("FOH 1"), PathBuf::from("library/screenshots/FOH_1"));
    }
}
