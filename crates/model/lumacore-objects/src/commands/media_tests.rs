//! Step 12a2 (decisions §30 "12a2 / 12a3 hardening"): pictures, the look on
//! the profile, profile files and asset drives — through real dispatch.

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// A PNG header of `w` × `h` with a tag, so each test has its own file.
fn png(w: u32, h: u32, tag: &str) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    b.extend(tag.as_bytes());
    b
}

/// Put a file into the (test) import folder.
fn put(name: &str, bytes: &[u8]) {
    let dir = lumacore_luma::media::library_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/{name}"), bytes).unwrap();
}

#[test]
fn images_import_label_copy_and_delete() {
    let mut state = AppState::new();
    put("t12a2-stage.png", &png(640, 480, "stage"));
    let out = run(&mut state, "Import Image \"t12a2-stage.png\"");
    assert!(out.contains("Image 1") && out.contains("640×480"), "{out}");
    let image = state.root.show_data.images[&1].clone();
    assert_eq!((image.width, image.height, image.ext.as_str()), (640, 480, "png"));
    assert!(lumacore_luma::media::has(&image.sha256, "png"), "kept in the store");
    // Not a picture, or outside library/media/, is refused.
    put("t12a2-notes.txt", b"hello");
    assert!(run(&mut state, "Import Image \"t12a2-notes.txt\"").contains("E-MEDIA-002"));
    assert!(run(&mut state, "Import Image \"../secret.png\"").contains("E-MEDIA-001"));
    // A taken number asks for /Overwrite (which keeps the GUID).
    assert!(run(&mut state, "Import Image 1 \"t12a2-stage.png\"").contains("/Overwrite"));
    run(&mut state, "Import Image 1 \"t12a2-stage.png\" /Overwrite");
    assert_eq!(state.root.show_data.images[&1].meta.identity.guid, image.meta.identity.guid);

    run(&mut state, "Label Image 1 \"Stage\"");
    assert_eq!(state.root.show_data.images[&1].meta.name(), Some("Stage"));
    // Owner first: into your own profile's icons, a copy with a new GUID.
    let out = run(&mut state, "Copy Image 1 At UserProfile Icon 5");
    assert!(out.contains("Icon 5"), "{out}");
    let profile = state.root.show_data.profile_for_user("Admin").unwrap();
    assert_eq!(profile.content.icons[&5].sha256, image.sha256);
    assert_ne!(profile.content.icons[&5].meta.identity.guid, image.meta.identity.guid);
    assert!(run(&mut state, "List Icon").contains("Stage"));
    assert!(run(&mut state, "Delete Image 1 /nc").contains("Deleted Image 1"));
    assert!(state.root.show_data.images.is_empty());
    assert!(run(&mut state, "Delete UserProfile Icon 5 /nc").contains("Deleted"));
}

#[test]
fn the_look_is_set_on_the_profile_by_its_users() {
    let mut state = AppState::new();
    run(&mut state, "Store User \"LD\" /Rights Program");
    put("t12a2-bg.png", &png(1920, 1080, "bg"));
    let out = run(&mut state, "Import UserProfile Image \"t12a2-bg.png\"");
    assert!(out.contains("Image 1"), "{out}");
    let out = run(&mut state, "Set UserProfile LockScreen \"Image\" 1");
    assert!(!out.contains("Error"), "{out}");
    let admin = state.root.show_data.profile_for_user("Admin").unwrap().clone();
    assert_eq!(admin.settings.lock_screen.image, Some(admin.content.images[&1].meta.identity.guid), "a GUID link");
    assert!(run(&mut state, "Set UserProfile LockScreen \"Image\" 9").contains("no Image 9"));

    run(&mut state, "Set UserProfile Theme \"Base\" Night");
    run(&mut state, "Set UserProfile Theme Slot \"Accent\" \"#9A7A08\"");
    // A negative value after an implicit owner reads as a glued list operator (E-SYN-007,
    // open — §33 3c1): the profile is named here.
    let out = run(&mut state, "Set UserProfile \"Admin\" Theme \"AccentHue\" -30");
    assert!(!out.contains("Error"), "{out}");
    let theme = &state.root.show_data.profile_for_user("Admin").unwrap().settings.theme;
    assert_eq!(theme.preset, "Night");
    assert_eq!(theme.palette.accent.as_deref(), Some("#9a7a08"));
    assert_eq!(theme.adjust.accent_hue, -30);
    assert!(run(&mut state, "Set UserProfile Theme Slot \"Accent\" \"red\"").contains("E-SET-003"));
    // Your own without the owner; one colour back, then the whole theme (decisions §33 60, 61).
    assert!(!run(&mut state, "Set Theme Slot \"Accent\" \"#112233\"").contains("Error"));
    assert_eq!(state.root.show_data.profile_for_user("Admin").unwrap().settings.theme.palette.accent.as_deref(), Some("#112233"));
    let out = run(&mut state, "Reset Theme Slot \"Accent\"");
    assert!(!out.contains("Error"), "{out}");
    assert_eq!(state.root.show_data.profile_for_user("Admin").unwrap().settings.theme.palette.accent, None);
    let out = run(&mut state, "Reset UserProfile Theme");
    assert!(!out.contains("Error"), "{out}");
    assert_eq!(state.root.show_data.profile_for_user("Admin").unwrap().settings.theme, lumacore_tree::Theme::default());

    // LD may change their own profile, not Admin's; Admin may change LD's.
    state.user = "LD".into();
    assert!(!run(&mut state, "Set UserProfile Theme \"Base\" Daylight").contains("Error"));
    let out = run(&mut state, "Set UserProfile \"Admin\" Theme \"Base\" Daylight");
    assert!(out.contains("E-RIGHTS-001"), "{out}");
    state.user = "Admin".into();
    let ld_profile = state.root.show_data.profile_id_for_user("LD").unwrap();
    let out = run(&mut state, &format!("Set UserProfile {ld_profile} Theme \"Base\" Night"));
    assert!(!out.contains("Error"), "{out}");
    assert_eq!(state.root.show_data.profile_for_user("LD").unwrap().settings.theme.preset, "Night");
}

#[test]
fn a_profile_moves_to_another_show_with_its_pictures() {
    let mut state = AppState::new();
    put("t12a2-move.png", &png(32, 32, "move"));
    run(&mut state, "Import UserProfile Icon \"t12a2-move.png\"");
    run(&mut state, "Set UserProfile Theme \"Base\" Daylight");
    let out = run(&mut state, "Export UserProfile \"Admin\" \"t12a2-admin\"");
    assert!(out.contains("t12a2-admin.lumaprofile"), "{out}");
    let source = state.root.show_data.profile_for_user("Admin").unwrap().clone();

    // Another show: the same GUID needs /Overwrite, then replaces it.
    let mut other = AppState::new();
    let out = run(&mut other, "Import UserProfile \"t12a2-admin\"");
    let mine = other.root.show_data.profile_for_user("Admin").unwrap().meta.identity.guid;
    if mine == source.meta.identity.guid {
        assert!(out.contains("/Overwrite"), "{out}");
        run(&mut other, "Import UserProfile \"t12a2-admin\" /Overwrite");
    }
    let imported = other
        .root
        .show_data
        .user_profiles
        .values()
        .find(|p| p.meta.identity.guid == source.meta.identity.guid)
        .expect("the profile is in the other show");
    assert_eq!(imported.settings.theme.preset, "Daylight");
    assert_eq!(imported.content.icons[&1].sha256, source.content.icons[&1].sha256, "its icon came along");
}

/// The background job's follow-up line containing `needle` (waits ≤ 5 s).
fn follow_up(needle: &str) -> String {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut seen = Vec::new();
    while std::time::Instant::now() < until {
        seen.extend(lumacore_luma::background::take());
        if let Some(i) = seen.iter().position(|d| d.line.contains(needle)) {
            return seen.swap_remove(i).line;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    panic!("no follow-up line with {needle}");
}

#[test]
fn assets_stay_on_drives_and_are_checked_per_station() {
    let mut state = AppState::new();
    let folder = std::env::temp_dir().join("lumacore-test-assets-12a2");
    std::fs::create_dir_all(folder.join("clips")).unwrap();
    std::fs::write(folder.join("clips/intro.mov"), vec![7u8; 4096]).unwrap();
    run(&mut state, "Store AssetRoot \"Video\"");
    let out = run(&mut state, "Import Asset \"Video\" \"clips/intro.mov\"");
    assert!(out.contains("isn't mapped"), "{out}");
    let out = run(&mut state, &format!("Set AssetRoot \"Video\" \"Folder\" \"{}\"", folder.display()));
    assert!(out.contains("on this station"), "{out}");
    assert_eq!(
        state.root.station_settings.asset_roots.get("video").map(String::as_str),
        Some(folder.to_string_lossy().as_ref()),
        "station-local mapping"
    );
    // R1-70 (Q42): hashed on a worker; the follow-up line stores it.
    let out = run(&mut state, "Import Asset \"Video\" \"clips/intro.mov\"");
    assert!(out.contains("in the background"), "{out}");
    assert!(state.root.show_data.assets.files.is_empty(), "not yet");
    let follow = follow_up("clips/intro.mov");
    assert!(follow.contains("/Hash") && follow.contains("/Size 4096"), "{follow}");
    let out = run(&mut state, &follow);
    assert!(out.contains("Asset 1"), "{out}");
    let asset = &state.root.show_data.assets.files[&1];
    assert_eq!((asset.size, asset.path.as_str()), (4096, "clips/intro.mov"));
    assert!(run(&mut state, "List Asset /Verify").contains("in the background"));
    assert_eq!(follow_up("List Asset"), "List Asset");
    assert!(
        lumacore_sysmon::history().iter().any(|l| l.text().contains("Asset 1 \"intro.mov\": ok (verified)")),
        "the result is in the System Monitor"
    );
    // A member lists and verifies its own drives; Import still goes to the master.
    use crate::commands::registry::{Scope, line_scope};
    assert_eq!(line_scope("List Asset /Verify"), Scope::Station);
    assert_eq!(line_scope("Import Asset \"Video\" \"x.mov\""), Scope::Master);
    assert!(run(&mut state, "Import Asset \"Video\" \"../x\"").contains("E-MEDIA-001"));
    // R1 review 17: the options are checked, and the follow-up names the
    // root by GUID (a rename while hashing can't break it).
    let guid = state.root.show_data.assets.roots.values().next().unwrap().meta.identity.guid.to_string();
    assert!(follow.contains(&guid), "{follow}");
    for bad in ["/Hash xyz /Size 1", "/Hash 00 /Size 1", &format!("/Hash {} /Size big", "a".repeat(64)), &format!("/Hash {}", "a".repeat(64))] {
        let out = run(&mut state, &format!("Import Asset \"Video\" \"clips/intro.mov\" {bad}"));
        assert!(out.contains("E-CMD-008"), "{bad}: {out}");
    }
    assert_eq!(state.root.show_data.assets.files.len(), 1, "nothing stored");
    std::fs::remove_file(folder.join("clips/intro.mov")).unwrap();
    let out = run(&mut state, "List Asset /Missing");
    assert!(out.contains("missing"), "listed, never a crash: {out}");
    assert!(run(&mut state, "Delete AssetRoot \"Video\" /nc").contains("Assets use this root"));
}

#[test]
fn image_steps_are_undone_by_oops() {
    let mut state = AppState::new();
    put("t12a2-oops.png", &png(8, 8, "oops"));
    run(&mut state, "Import Image \"t12a2-oops.png\"");
    assert_eq!(state.root.show_data.images.len(), 1);
    run(&mut state, "Oops");
    assert!(state.root.show_data.images.is_empty(), "the import was one step");
}

/// R1-69 (Q41): a station's folder for an AssetRoot follows a rename made
/// anywhere (here or replicated from the master), by GUID.
#[test]
fn a_renamed_asset_root_keeps_this_stations_folder() {
    let mut state = AppState::new();
    run(&mut state, "Store AssetRoot \"Video\"");
    state.root.station_settings.asset_roots.insert("video".into(), "/media/video".into());
    state.follow_renames(); // first look: names recorded
    // The rename arrives as a show change (as from the master).
    let root = state.root.show_data.assets.roots.values_mut().next().unwrap();
    root.name = "Clips".into();
    assert!(state.follow_renames());
    let roots = &state.root.station_settings.asset_roots;
    assert_eq!(roots.get("clips").map(String::as_str), Some("/media/video"));
    assert!(!roots.contains_key("video"));

    // A show file whose root (same GUID) has another name is no rename:
    // this station's folders stay as they are.
    let mut older = state.root.show_data.clone();
    older.assets.roots.values_mut().next().unwrap().name = "Old".into();
    let path = std::env::temp_dir().join(format!("lumacore-renames-{}.luma", std::process::id()));
    lumacore_luma::show_file::write(&path, &older, |_| {}).unwrap();
    crate::objects::load_show(&mut state, &path.to_string_lossy()).unwrap();
    let _ = std::fs::remove_file(&path);
    assert!(!state.follow_renames(), "a load isn't a rename");
    assert!(state.root.station_settings.asset_roots.contains_key("clips"));
}
