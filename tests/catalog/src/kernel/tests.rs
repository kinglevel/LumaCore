// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The kernel's machinery on real words from several crates (decisions §34 25.16): the change
//! path, the demo show, the reference check, the menu listing — until stand-in words cover them.
//!
//!
//! Owns:          the change path, the demo show, the reference check and the menu list on real words.
//! Doesn't own:   the demo builder (the kernel's demo), the reference (the kernel's references).
//! Entry points:  the #[test] fns.
//! Decisions:     §20, §21, §34 25.16.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

#[test]
fn the_change_path_journals_undoes_and_replicates_what_changed() {
    // Protocol lines change live (patch edits under the claim are a draft,
    // filed only when kept: patch_draft_tests).
    let mut state = AppState::new();
    let count = |s: &AppState| s.root.show_data.protocols.all_lines().count();
    let before = count(&state);
    let out = run(&mut state, "Store Sacn 7");
    assert_eq!(
        count(&state),
        before + 1,
        "{out}"
    );

    let entry = state
        .root
        .runtime
        .journal
        .entries()
        .last()
        .expect("journaled");
    assert_eq!(entry.command, "Store Sacn 7");
    assert_eq!(entry.changed, vec!["Protocols".to_string()]);
    assert_eq!(state.replication_outbox.len(), 1);

    run(&mut state, "oops");
    assert_eq!(count(&state), before);
    // Read-only commands leave no entry.
    let n = state.root.runtime.journal.len();
    run(&mut state, "list universe");
    assert_eq!(state.root.runtime.journal.len(), n);

    // (`List Journal` and `Watch Events` are the station's words: its tests read them.)
    assert!(
        state
            .root
            .runtime
            .journal
            .entries()
            .any(|e| e.command.contains("Store Sacn 7"))
    );
}

/// The demo show builds cleanly from its sources, survives save → load through the real load
/// path (format check, GUIDs, `after_load`, references) and matches its golden: the counts and
/// the SHA-256 of the normalized JSON (`tests/catalog/testdata/demo.golden.txt`, rewrite with
/// `LUMACORE_BLESS=1`, read the diff). The built file is written beside it when it differs
/// (decisions §40 15: the file is gitignored, made when missing or stale).
#[test]
fn the_demo_show_builds_loads_and_matches_its_golden() {
    use lumacore_objects::demo::{build, check, fingerprint, repo_root};
    let root = repo_root();
    let demo = build(&root);
    assert!(
        demo.warnings.is_empty(),
        "building the demo complained:\n{}",
        demo.warnings.join("\n")
    );
    check(&demo.state).expect("the built demo");

    // Save → load as the server does.
    let dir = std::env::temp_dir().join(format!("lumacore-demo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("demo.luma");
    std::fs::write(&path, demo.file()).unwrap();
    let mut loaded = AppState::new();
    loaded
        .load_show(&path.to_string_lossy())
        .expect("the generated demo loads");
    let _ = std::fs::remove_dir_all(&dir);
    check(&loaded).expect("the loaded demo");
    let report = loaded.root.show_data.identity();
    assert!(
        report.is_clean(),
        "{:#?}",
        lumacore_objects::references::problems(&report)
    );

    // The golden.
    let golden = root.join("tests/catalog/testdata/demo.golden.txt");
    let got = fingerprint(&demo);
    if std::env::var("LUMACORE_BLESS").is_ok() {
        std::fs::write(&golden, &got).unwrap();
    }
    let want = std::fs::read_to_string(&golden).unwrap_or_default();
    assert!(
        got == want,
        "the demo differs from tests/catalog/testdata/demo.golden.txt (LUMACORE_BLESS=1 rewrites it):\n--- golden\n{want}--- built\n{got}"
    );

    // The built file on disk follows the build.
    let built = lumacore_objects::demo::demo_path(&root);
    let current = std::fs::read(&built)
        .ok()
        .and_then(|b| lumacore_luma::show_file::decode(&b).ok())
        .is_some_and(|c| lumacore_objects::demo::normalized(&c.json) == lumacore_objects::demo::normalized(&demo.json()));
    if !current {
        lumacore_objects::demo::write(&root, &demo).expect("write library/shows/demo.luma");
    }
}

#[test]
fn references_broken_links_are_kept_and_listed() {
    let mut state = AppState::new();
    dispatch(&mut state, "Store Sacn 1 /Network \"Rig\"");
    use lumacore_objects::references::{list, summary};
    assert!(list(&mut state, &[]).contains("every link resolves"));
    // Remove the network behind the line's back: the link stays, listed.
    state.root.show_data.protocols.networks.clear();
    let out = list(&mut state, &["/Broken"]);
    assert!(
        out.contains("Protocols.DMXProtocols.Sacn.Lines.1.Network → missing Network"),
        "{out}"
    );
    assert!(state.root.show_data.protocols.line("Sacn", 1).unwrap().network != uuid::Uuid::nil());
    assert!(summary(&state).contains("1 broken"));
}

#[test]
fn list_and_info_menu_publish_the_registry() {
    let mut state = AppState::new();
    let list = run(&mut state, "List Menu");
    for m in lumacore_registry::get().menus() {
        assert!(list.contains(m.name), "{list}");
    }
    let info = run(&mut state, "Info Menu \"DMXProtocols\"");
    assert!(
        info.contains("Tab Lines") && info.contains("dmx.lines") && info.contains("HoldOutput"),
        "{info}"
    );
    assert!(run(&mut state, "Info Menu \"x\"").contains("E-CMD-007"));
}

/// An older show upgrades step by step (decisions 0289): today's rig taken back to the oldest
/// readable format — the theme's Tint, a view button in the flat set, its presets in a preset
/// group, the selected preset group — comes up again equal to today's.
#[test]
fn an_older_show_upgrades_step_by_step_to_today() {
    let mut state = lumacore_objects::demo::fresh_state();
    // A view button on the Admin profile's screen 1, holding one of its views.
    let admin = state.root.show_data.profile_id_for_user("Admin").expect("Admin's profile");
    let profile = std::sync::Arc::make_mut(state.root.show_data.user_profiles.get_mut(&admin).unwrap());
    // The rig's Admin has no views: a link to none is kept and loads (broken links are listed).
    let view = profile.content.views.values().next().map_or_else(uuid::Uuid::new_v4, |v| v.meta.identity.guid);
    profile.content.view_buttons.entry(1).or_default().insert(4, lumacore_tree::ViewButton { kind: lumacore_tree::ButtonKind::View, guid: view });
    let today = serde_json::to_value(&state.root.show_data).unwrap();

    let mut old = today.clone();
    old["show_settings"]["format_version"] = lumacore_luma::show_file::OLDEST_FORMAT.into();
    for pool in old["datapools"].as_object_mut().unwrap().values_mut() {
        let presets = pool.as_object_mut().unwrap().remove("presets").unwrap_or_default();
        let group = serde_json::json!({ "meta": lumacore_base::NodeMeta::new(Some("All".into())), "preset_mode": "Selective", "presets": presets });
        pool["preset_groups"] = serde_json::json!({ "1": group });
    }
    for p in old["user_profiles"].as_object_mut().unwrap().values_mut() {
        let flat: serde_json::Map<String, serde_json::Value> = p["content"]["view_buttons"]
            .as_object()
            .and_then(|s| s.get("1"))
            .and_then(|b| b.as_object())
            .map(|b| b.iter().map(|(slot, v)| (slot.clone(), v["guid"].clone())).collect())
            .unwrap_or_default();
        p["content"]["view_buttons"] = serde_json::Value::Object(flat);
        p["content"]["selected_preset_group"] = serde_json::Value::Null;
        let adjust = p["settings"]["theme"]["adjust"].as_object_mut().unwrap();
        adjust.remove("background_hue");
        adjust.remove("background_saturation");
        adjust.insert("tint".into(), 0.into());
    }

    let (show, upgraded) = lumacore_luma::show_file::parse_show_upgraded(&old.to_string()).expect("the older show loads");
    let upgraded = upgraded.expect("it was upgraded");
    assert_eq!((upgraded.from, upgraded.to), (lumacore_luma::show_file::OLDEST_FORMAT, lumacore_tree::FORMAT_VERSION));
    assert!(upgraded.notes.iter().any(|n| n.contains("view buttons")), "{:?}", upgraded.notes);
    assert!(upgraded.notes.iter().any(|n| n.contains("presets in one pool")), "{:?}", upgraded.notes);
    assert_eq!(serde_json::to_value(&show).unwrap(), today, "the upgraded show is today's");
    // One older than the oldest is refused, naming why.
    old["show_settings"]["format_version"] = (lumacore_luma::show_file::OLDEST_FORMAT - 1).into();
    assert!(lumacore_luma::show_file::parse_show(&old.to_string()).unwrap_err().contains("E-SHOW-002"));
}

/// A real show file of an older format, named by `LUMACORE_OLD_SHOW`, loads upgraded (a hand
/// check: `LUMACORE_OLD_SHOW=path cargo test -p lumacore-catalog-tests -- --ignored old_show`).
#[test]
#[ignore = "needs a real older show file: LUMACORE_OLD_SHOW"]
fn an_old_show_file_loads_upgraded() {
    let path = std::env::var("LUMACORE_OLD_SHOW").expect("LUMACORE_OLD_SHOW names a show file");
    let (show, upgraded) = lumacore_luma::show_file::read_upgraded(std::path::Path::new(&path)).expect("it loads");
    let u = upgraded.expect("an older format");
    eprintln!("{path}: {} — {} fixtures, notes: {:?}", u.line(), show.devices.dmx.patch.fixtures.len(), u.notes);
}
