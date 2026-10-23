// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! 12d part 5 (decisions §30 "12d part 5 built"): versions on every save,
//! the manifest's facts and history, recovery copies, before-load copies,
//! the trash, the cleanup tiers, the startup ask and `Import Show` — in a
//! temporary show library (never the real one).
//!
//! And the show file's halves of the kernel's tests: show settings and their levels, EditSetting
//! Show, CleanStart is Admin's, Restart / Shutdown untracked, `turnoff`.
//!
//! Owns:          the tests of the show file's words.
//! Doesn't own:   recovery's own rules (lumacore-state), the file format's (lumacore-luma).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d part 5", §33 46.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_luma::backups::*;
use lumacore_objects::commands::dispatch;
use lumacore_objects::recovery::{recovery_ask, recovery_copy, station_facts};
use lumacore_objects::{AppState, DispatchResult};
use serde_json::json;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// A fresh library for this test's thread; removed when dropped.
struct Library(std::path::PathBuf);

impl Library {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("lumacore-backups-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        use_dir(Some(dir.clone()));
        Self(dir)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        use_dir(None);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn station() -> AppState {
    let mut s = AppState::new();
    s.root.station_settings.station_name = "FOH".into();
    s
}

fn add_user(state: &mut AppState, no: u32, name: &str) {
    let mut u = state.root.show_data.users.values().next().unwrap().clone();
    u.meta = lumacore_base::NodeMeta::default();
    u.login = name.into();
    state.root.show_data.users.insert(no, u);
}

#[test]
fn stamps_read_back_and_times_read_well() {
    let ms = 1_791_837_123_000;
    let s = stamp(ms);
    assert_eq!(s.len(), 15);
    assert_eq!(stamp_ms(&s), Some(ms));
    assert_eq!(stamp_ms("demo_timed"), None);
    assert!(when_text(now_ms(), now_ms()).starts_with("today "));
    assert_eq!(size_text(1_100_000), "1.1 MB");
}

#[test]
fn every_save_keeps_a_version_and_writes_its_facts() {
    let _lib = Library::new("versions");
    let mut s = station();
    let users = s.root.show_data.users.len();
    assert!(
        run(&mut s, "SaveShow \"t\"").contains("saved"),
        "first save"
    );
    assert!(versions("t").is_empty(), "nothing replaced yet");
    add_user(&mut s, 9, "LD");
    let out = run(&mut s, "SaveShow /Note \"before rehearsal\"");
    assert!(out.contains("kept as a version"), "{out}");
    let v = versions("t");
    assert_eq!(v.len(), 1, "the replaced file is a version");
    // The manifest: facts, counts, the index and two history rows.
    let (m, _, _) = manifest(&crate::show_path("t")).unwrap();
    assert_eq!(m["reason"], "quick");
    assert_eq!(m["station"], "FOH");
    assert_eq!(m["note"], "before rehearsal");
    assert_eq!(m["counts"]["Users"], users + 1);
    assert_eq!(m["index"]["Users"].as_object().unwrap().len(), users + 1);
    let h = m["history"].as_array().unwrap();
    assert_eq!(h.len(), 2);
    assert_eq!(
        h[0]["changes"]["Users"],
        json!([1, 0, 0]),
        "one user added since the save before"
    );
    assert!(
        h[0]["detail"]["Users"]["a"][0]
            .as_str()
            .unwrap()
            .contains("LD")
    );
    // History: the current file, the version.
    let rows = history_rows("t", Some("t"));
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["kept"], "current");
    assert_eq!(rows[0]["reason"], "Quick");
    assert_eq!(rows[0]["k_Users"], "+1");
    assert_eq!(rows[1]["kept"], "version");
    assert_eq!(rows[1]["version"], json!(v[0].id));
    // `List Version` reads it without loading.
    assert!(run(&mut s, "List Version \"t\"").contains("+1 user"));
}

#[test]
fn the_timed_copy_is_a_recovery_copy_never_the_show() {
    let _lib = Library::new("timed");
    let mut s = station();
    run(&mut s, "SaveShow \"t\"");
    let before = std::fs::read(crate::show_path("t")).unwrap();
    s.root.show_data.show_settings.autosave_minutes = 1;
    s.show_dirty = true;
    s.last_save_at = std::time::Instant::now() - std::time::Duration::from_secs(61);
    crate::autosave_tick(&mut s);
    let copies = recovery();
    assert_eq!(copies.len(), 1);
    assert_eq!(copies[0].reason, "timed");
    assert_eq!(copies[0].station, "FOH");
    assert_eq!(copies[0].show, "t");
    assert!(s.show_dirty, "still unsaved");
    assert_eq!(
        std::fs::read(crate::show_path("t")).unwrap(),
        before,
        "the show file is untouched"
    );
    // Not again until the next interval.
    crate::autosave_tick(&mut s);
    assert_eq!(recovery().len(), 1);
    // After an unclean stop, one ask: the copy is newer than the file.
    let ask = recovery_ask(&s).expect("a newer copy asks");
    assert!(ask.line_of("Open copy").unwrap().contains("/Recovery"));
    // Opening it: unsaved changes of "t"; the next save is Restored.
    let id = copies[0].id.clone();
    let out = run(
        &mut s,
        &format!("LoadShow /Recovery \"{id}\" /NoSave /NoConfirmation"),
    );
    assert!(out.contains("as unsaved changes"), "{out}");
    assert_eq!(s.current_show.as_deref(), Some("t"));
    assert!(s.show_dirty);
    run(&mut s, "SaveShow");
    let (m, _, _) = manifest(&crate::show_path("t")).unwrap();
    assert_eq!(m["reason"], "restored");
}

#[test]
fn a_load_over_unsaved_work_keeps_a_copy_and_deletes_go_to_the_trash() {
    let _lib = Library::new("beforeload");
    let mut s = station();
    run(&mut s, "SaveShow \"a\"");
    run(&mut s, "SaveShow \"b\"");
    s.show_dirty = true;
    let out = run(&mut s, "LoadShow \"a\" /NoSave /NoConfirmation");
    assert!(out.contains("recovery copy"), "{out}");
    assert_eq!(recovery()[0].reason, "before-load");
    assert!(run(&mut s, "DeleteShow \"b\" /NoConfirmation").contains("trash"));
    assert!(!crate::show_path("b").exists());
    assert_eq!(trash().len(), 1);
    assert!(run(&mut s, "DeleteShow \"a\" /NoConfirmation").contains("loaded show"));
}

#[test]
fn cleanup_thins_by_age_keeps_pins_and_lost_work_and_holds_the_cap() {
    let now = 100 * DAY;
    let item = |age: u64, reason: &str, pinned: bool| Item {
        at: now - age,
        size: 1_000_000,
        reason: reason.into(),
        pinned,
        group: "v:t".into(),
        managed: true,
    };
    let items = vec![
        item(10 * 60_000, "save", false),       // 0: last hour → kept
        item(20 * 60_000, "save", false),       // 1: last hour → kept
        item(2 * HOUR + 1000, "save", false),   // 2: today, its hour's newest → kept
        item(2 * HOUR + 2000, "save", false),   // 3: same hour, older → goes
        item(3 * DAY + 1000, "save", false),    // 4: this week, its day → kept
        item(3 * DAY + 2000, "save", false),    // 5: same day → goes
        item(40 * DAY, "save", false),          // 6: past 30 days → goes
        item(40 * DAY, "save", true),           // 7: pinned → kept
        item(20 * DAY, "signal", false),        // 8: signal < 30 d → kept
        item(20 * DAY + 5000, "signal", false), // 9: signal < 30 d → kept too
    ];
    let (gone, why) = plan_cleanup(&items, Rules::default(), now);
    assert_eq!(gone, vec![3, 5, 6]);
    assert_eq!(why[0], "1 h");
    assert_eq!(why[7], "pinned");
    assert_eq!(why[8], "30 d");
    // A 5 MB cap: the oldest unpinned plain ones go first, lost work last.
    let (gone, _) = plan_cleanup(
        &items,
        Rules {
            keep_days: 30,
            cap_mb: 5,
        },
        now,
    );
    assert_eq!(gone, vec![2, 3, 4, 5, 6]);
    assert!(!gone.contains(&7), "pinned never");
    // A file that doesn't say why it was written (made by hand, older) stays.
    let mut foreign = item(60 * DAY, "", false);
    foreign.managed = false;
    let (gone, why) = plan_cleanup(
        &[foreign],
        Rules {
            keep_days: 1,
            cap_mb: 0,
        },
        now,
    );
    assert!(gone.is_empty());
    assert_eq!(why[0], "kept");
    assert!(!gone.contains(&0), "the newest stay");
}

#[test]
fn pins_and_notes_on_versions() {
    let _lib = Library::new("pins");
    let mut s = station();
    run(&mut s, "SaveShow \"t\"");
    run(&mut s, "SaveShow");
    let id = versions("t")[0].id.clone();
    let out = run(&mut s, &format!("Set Version \"{id}\" \"Note\" \"show 1\""));
    assert!(out.contains("pinned"), "{out}");
    let v = &versions("t")[0];
    assert_eq!(v.pinned.as_deref(), Some("show 1"));
    let rows = history_rows("t", Some("t"));
    assert_eq!(rows[1]["kept"], "★ pinned");
    assert_eq!(rows[1]["note"], "show 1");
    assert!(run(&mut s, &format!("Set Version \"{id}\" \"Pinned\" No")).contains("unpinned"));
    assert!(
        run(&mut s, &format!("Delete Version \"{id}\"")).starts_with("ASK"),
        "deleting asks"
    );
    assert!(
        run(
            &mut s,
            &format!("Delete Version \"t\" \"{id}\" /NoConfirmation")
        )
        .contains("trash")
    );
    assert!(versions("t").is_empty());
    assert_eq!(
        history_rows("t", Some("t"))[1]["kept"],
        "pruned",
        "the row stays"
    );
    assert!(run(&mut s, "Delete Recovery /Cleanup").starts_with("ASK"));
}

#[test]
fn the_feed_lists_the_library_history_and_copies() {
    let _lib = Library::new("feed");
    let mut s = station();
    run(&mut s, "SaveShow \"t\"");
    run(&mut s, "SaveShow");
    let status = crate::show_info(&s);
    let v = library_view(&status, None, &station_facts(&s));
    assert_eq!(v["shows"][0]["name"], "t");
    assert_eq!(v["shows"][0]["loaded"], true);
    assert_eq!(v["shows"][0]["versions"], 1);
    assert!(
        v["shows"][0]["format_text"]
            .as_str()
            .unwrap()
            .ends_with("loads")
    );
    assert_eq!(v["history_of"], "t");
    assert_eq!(v["history"].as_array().unwrap().len(), 2);
    assert_eq!(v["kinds"].as_array().unwrap().len(), 10);
    assert!(v["usage"]["text"].as_str().unwrap().contains("of 500 MB"));
}

/// Merge of bugfixes (decisions §30): the branch's backup hardening lives in
/// the versions — copied, never over one from the same second, a long name
/// fits, a read error stops the save.
#[test]
fn versions_are_copied_never_overwritten_and_a_read_error_stops_the_save() {
    let lib = Library::new("hardening");
    let mut s = station();
    run(&mut s, "SaveShow \"h\"");
    let path = crate::show_path("h");
    // Two versions in the same second: the second gets `-2`.
    let a = keep_version("h", &path).unwrap().unwrap();
    let b = keep_version("h", &path).unwrap().unwrap();
    assert_ne!(a, b, "never over a version from the same second");
    assert!(path.exists(), "copied, the named show stays");
    assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&path).unwrap());
    // Only a missing file is "nothing to keep".
    assert_eq!(
        keep_version("h", &lib.0.join("shows/none.luma")).unwrap(),
        None
    );
    let blocked = path.join("x.luma"); // under a file: not "missing"
    assert!(
        keep_version("h", &blocked).is_err(),
        "a read error is reported"
    );
    // A long name still fits a recovery copy's and the trash's file name.
    let long = "Ä".repeat(200);
    s.current_show = Some(long.clone());
    s.show_dirty = true;
    let copy = recovery_copy(&s, &s.root.show_data, "timed").expect("written");
    let name = std::path::Path::new(&copy)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert!(name.len() <= 255, "{}", name.len());
}

/// Merge of bugfixes (decisions §30): a start that died while loading its
/// show skips it (`.starting`, E-SHOW-006) first; then the recovery ask
/// offers that show's newer copy.
#[test]
fn a_skipped_startup_show_offers_its_newer_copy() {
    let _lib = Library::new("skipped");
    let mut s = station();
    run(&mut s, "SaveShow \"k\"");
    s.show_dirty = true;
    recovery_copy(&s, &s.root.show_data, "timed").expect("written");
    // The next start: nothing loaded, "k" skipped.
    let mut next = station();
    next.skipped_startup_show = Some("k".into());
    let ask = recovery_ask(&next).expect("the copy is offered");
    assert!(
        ask.view().text.contains("E-SHOW-006"),
        "{}",
        ask.view().text
    );
    assert!(
        ask.line_of("Open copy")
            .unwrap()
            .starts_with("LoadShow \"k\" /Recovery")
    );
    // Without the skip and with nothing loaded: no ask for another show.
    assert!(recovery_ask(&station()).is_none());
}

// *************************************************************************************************
// names (was backup.rs's own tests)
// *************************************************************************************************
use crate::ask::generated_name;
use crate::ask::show_name;

#[test]
fn names_are_sanitised() {
    let s = |v: &str| show_name(&[v.to_string()]);
    assert_eq!(s("\"My Show.luma\"").unwrap(), Some("My Show".into()));
    assert!(s("../etc").is_err());
    assert!(s("a/b").is_err());
    assert_eq!(s("").unwrap(), None);
}

#[test]
fn generated_name_shape() {
    let n = generated_name();
    assert!(
        n.starts_with("NewShow_")
            && n.ends_with("UTC")
            && n.len() == "NewShow_2026.10.10_14.03.07UTC".len()
    );
}

#[test]
fn show_names_keep_apostrophes_and_refuse_what_windows_refuses() {
    let name = |s: &str| show_name(&[s.to_string()]);
    assert_eq!(name("\"Rock'n'\"").unwrap().as_deref(), Some("Rock'n'"));
    assert_eq!(
        name("Festival 26.luma").unwrap().as_deref(),
        Some("Festival 26")
    );
    for bad in [
        "a:b", "a?b", "a*b", "a|b", "a<b", "x.", "CON", "com1", "a/x", ".hidden",
    ] {
        assert!(name(bad).is_err(), "{bad}");
    }
}

// *************************************************************************************************
// what the kernel's tests checked through the show file's words
// *************************************************************************************************
fn ladder_user(state: &mut AppState, user: &str, level: &str) {
    run(state, &format!("Store User \"{user}\" /Rights {level}"));
}

/// Show settings are typed and need their level (`Set Show …`, decisions §33 46).
#[test]
fn show_settings_are_typed_and_need_their_level() {
    let mut state = AppState::new();
    ladder_user(&mut state, "P", "Program");
    ladder_user(&mut state, "S", "Setup");
    state.user = "P".into();
    assert!(run(&mut state, "Set Show \"Description\" \"Tour\"").contains("Tour"));
    assert!(run(&mut state, "Set Show \"Autosave\" 5").contains("E-RIGHTS-001"));
    state.user = "S".into();
    assert!(run(&mut state, "Set Show \"Autosave\" Off").contains("Autosave off"));
    let out = run(&mut state, "Set Show \"Autosave\" + 10");
    assert!(out.contains("every 10 min"), "{out}");
    assert!(run(&mut state, "Set Show \"MergeTimeout\" Unlimited").contains("Unlimited"));
    assert!(run(&mut state, "Set Show \"MergeDefault\" UseMine").contains("Use Mine"));
}

/// `EditSetting Show`: each field says its home, unit and level.
#[test]
fn edit_setting_show_says_home_unit_and_level() {
    use lumacore_wire::{FieldInput, UiEvent};
    let mut state = AppState::new();
    run(&mut state, "EditSetting Show");
    let Some(UiEvent::EditSettings { fields, .. }) = state.ui_requests.last() else {
        panic!()
    };
    let auto = fields.iter().find(|f| f.name == "Autosave").unwrap();
    assert_eq!(
        (auto.home.as_str(), auto.unit.as_str(), auto.line.as_str()),
        ("show", "min", "Set Show \"Autosave\" {value}")
    );
    assert!(matches!(&auto.input, FieldInput::Number { named, .. } if named == &["Off"]));
    let desc = fields.iter().find(|f| f.name == "Description").unwrap();
    assert_eq!(desc.rights, "Program");
}

/// CleanStart is Admin's (E-RIGHTS-001), and so is a show setting's level (issue 04).
#[test]
fn clean_start_and_show_settings_carry_the_rights_code() {
    let mut state = AppState::new();
    ladder_user(&mut state, "LD", "Program");
    run(&mut state, "Claim Root.ShowData.Devices");
    state.user = "LD".into();
    for line in [
        "CleanStart /NoConfirmation",
        "Set Show \"MergeDefault\" UseMine",
    ] {
        let out = run(&mut state, line);
        assert!(out.contains("E-RIGHTS-001"), "{line}: {out}");
    }
}

