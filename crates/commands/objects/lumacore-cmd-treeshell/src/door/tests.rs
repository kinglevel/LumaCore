// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the tree shell's mv / cp / rm: on the show tree the Lum lines they mean (Swap, the
//! asks, between DataPools, another kind refused); under Root.Drives files in a temp folder (the
//! taken-name ask, Keep both, rm to the trash with its Put back line).
//!
//! Owns:          the tests of shell.rs.
//! Doesn't own:   the door verbs' own tests (door/tests.rs).
//! Entry points:  the #[test] fns.
//! Decisions:     §35 19.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use std::path::PathBuf;

use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(r) => r.text,
        DispatchResult::Ask(a) => format!("ASK {}", a.view().text),
        _ => String::new(),
    }
}

fn with_groups(names: &[(u32, &str)]) -> AppState {
    let mut s = AppState::new();
    for (n, name) in names {
        run(&mut s, &format!("Store Group {n} \"{name}\""));
    }
    run(&mut s, "cd Root.ShowData.DataPools.1.Groups");
    s
}

fn groups(s: &AppState) -> Vec<(u32, String)> {
    s.user_groups()
        .iter()
        .map(|(n, g)| (*n, g.meta.identity.name.clone().unwrap_or_default()))
        .collect()
}

fn pairs(v: &[(u32, &str)]) -> Vec<(u32, String)> {
    v.iter().map(|(n, s)| (*n, s.to_string())).collect()
}

#[test]
fn mv_is_move_and_swaps() {
    let mut s = with_groups(&[(3, "C"), (7, "G")]);
    let out = run(&mut s, "mv 3 7");
    assert_eq!(groups(&s), pairs(&[(3, "G"), (7, "C")]), "{out}");
}

#[test]
fn the_write_out_is_the_lum_line() {
    let mut s = with_groups(&[(3, "C")]);
    s.root
        .show_data
        .datapools
        .insert(2, lumacore_tree::Datapool::new(Some("Two".into())));
    let lum = super::write_out(&s, "mv 3 7").unwrap().unwrap();
    assert_eq!(lum, "Move DataPool 1 Group 3 At 7");
    let lum = super::write_out(&s, "cp 3 Root.ShowData.DataPools.2.Groups.8 --overwrite")
        .unwrap()
        .unwrap();
    assert_eq!(
        lum,
        "Copy DataPool 1 Group 3 At DataPool 2 Group 8 /Overwrite"
    );
    assert_eq!(
        super::write_out(&s, "rm 3 -n").unwrap().unwrap(),
        "Delete DataPool 1 Group 3 /NoConfirmation"
    );
    assert!(super::write_out(&s, "ls").is_none());
}

#[test]
fn cp_copies_and_asks_over_an_existing_one() {
    let mut s = with_groups(&[(1, "A"), (5, "E")]);
    run(&mut s, "cp 1 3");
    assert_eq!(groups(&s), pairs(&[(1, "A"), (3, "A"), (5, "E")]));
    assert!(run(&mut s, "cp 1 5").starts_with("ASK"));
    run(&mut s, "cp 1 5 --overwrite");
    assert_eq!(groups(&s)[2], (5, "A".to_string()));
}

#[test]
fn rm_asks_like_delete() {
    let mut s = with_groups(&[(1, "A"), (2, "B")]);
    let out = run(&mut s, "rm 1");
    assert!(out.starts_with("ASK Delete Group 1"), "{out}");
    run(&mut s, "rm 1 -n");
    assert_eq!(groups(&s), pairs(&[(2, "B")]));
}

#[test]
fn mv_between_datapools_and_not_into_another_kind() {
    let mut s = with_groups(&[(1, "A")]);
    s.root
        .show_data
        .datapools
        .insert(2, lumacore_tree::Datapool::new(Some("Two".into())));
    run(&mut s, "mv 1 Root.ShowData.DataPools.2.Groups.4");
    assert!(groups(&s).is_empty());
    assert!(s.root.show_data.datapools[&2].groups.contains_key(&4));
    run(&mut s, "Store Group 1 \"B\"");
    assert!(run(&mut s, "mv 1 Root.ShowData.DataPools.1.Sequences.4").contains("E-CMD-020"));
}

/// The drive table is one per process (decisions §38): the drive tests take turns.
static DRIVE_TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// A temp folder mounted as a drive for the drive tests (decisions §38: `Root.Drives.<id>` is a
/// drive's id, the folder its volume), with the library (and so its trash) inside it. The guard
/// keeps the table ours until the test ends.
struct TestDrive {
    dir: PathBuf,
    _turn: std::sync::MutexGuard<'static, ()>,
}

impl TestDrive {
    fn new(tag: &str, s: &mut AppState) -> Self {
        let turn = DRIVE_TURN.lock().unwrap_or_else(|e| e.into_inner());
        // No dashes: in a fence line `-x` is a flag.
        let dir = std::env::temp_dir().join(format!("lumacoretestshell{tag}{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("files")).unwrap();
        lumacore_luma::backups::use_dir(Some(dir.join("library").join("shows")));
        let volume = lumacore_objects::drives::DriveVolume { key: format!("test:{tag}"), label: "TEST".into(), mount: dir.clone(), kind: lumacore_objects::drives::DriveVolumeKind::Removable, free: 1, total: 2 };
        lumacore_objects::drives::reconcile(s, &[volume], (1, 2));
        assert!(s.drives.iter().any(|d| d.mount == dir), "the folder is a drive now");
        Self { dir, _turn: turn }
    }
}

fn dot(path: &std::path::Path) -> String {
    super::drive_dot(path)
}

#[test]
fn drive_files_copy_with_a_taken_name_asked() {
    let mut s = AppState::new();
    let drive = TestDrive::new("cp", &mut s);
    let dir = drive.dir.clone();
    let files = dir.join("files");
    std::fs::write(files.join("a.txt"), "a").unwrap();
    let out = run(
        &mut s,
        &format!(
            "cp {} {}",
            dot(&files.join("a.txt")),
            dot(&files.join("b.txt"))
        ),
    );
    assert!(files.join("b.txt").exists(), "{out}");
    let again = format!(
        "cp {} {}",
        dot(&files.join("a.txt")),
        dot(&files.join("b.txt"))
    );
    assert!(run(&mut s, &again).starts_with("ASK"));
    run(&mut s, &format!("{again} --keep"));
    assert!(files.join("b 2.txt").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rm_on_a_drive_moves_to_the_trash_and_says_how_back() {
    let mut s = AppState::new();
    let drive = TestDrive::new("rm", &mut s);
    let dir = drive.dir.clone();
    let file = dir.join("files").join("old.txt");
    std::fs::write(&file, "x").unwrap();
    let line = format!("rm {}", dot(&file));
    assert!(run(&mut s, &line).starts_with("ASK"));
    let out = run(&mut s, &format!("{line} -n"));
    assert!(!file.exists(), "{out}");
    assert!(out.contains("Put back: mv "), "{out}");
    let trashed: Vec<_> =
        std::fs::read_dir(dir.join("library").join(lumacore_luma::backups::TRASH))
            .unwrap()
            .collect();
    assert_eq!(trashed.len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn between_drives_and_the_show_is_refused() {
    let mut s = with_groups(&[(1, "A")]);
    let drive = TestDrive::new("cross", &mut s);
    let dir = drive.dir.clone();
    let file = dir.join("files").join("a.txt");
    std::fs::write(&file, "a").unwrap();
    assert!(
        run(
            &mut s,
            &format!("cp {} Root.ShowData.DataPools.1.Groups.5", dot(&file))
        )
        .contains("E-CMD-020")
    );
    assert!(
        run(
            &mut s,
            &format!("cp Root.ShowData.DataPools.1.Groups.1 {}", dot(&dir))
        )
        .contains("E-CMD-020")
    );
    let _ = std::fs::remove_dir_all(&dir);
}
