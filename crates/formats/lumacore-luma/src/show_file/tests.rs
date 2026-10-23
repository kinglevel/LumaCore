// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `show_file`.
//!
//! Owns:          the unit tests of show_file.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §20, §21 "6d", §30 "12a2", §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn entry_names(bytes: &[u8]) -> Vec<String> {
    let zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    zip.file_names().map(str::to_string).collect()
}

#[test]
fn a_container_holds_the_tree_and_reads_back() {
    let mut show = ShowData::new();
    show.show_settings.format_version = lumacore_tree::FORMAT_VERSION;
    let (bytes, missing) = encode(&show).unwrap();
    assert!(bytes.starts_with(b"PK") && missing.is_empty());
    assert_eq!(encode(&show).unwrap().0, bytes, "same show, same bytes");
    let names = entry_names(&bytes);
    assert_eq!(names[0], MANIFEST, "the manifest comes first");
    for want in [
        "Settings.json",
        "Devices/Devices.json",
        "DataPools/1 Default/datapool.json",
        "DataPools/1 Default/Groups.json",
        "Users/1 Admin/user.json",
        "Users/1 Admin/Programmer.json",
        "Users/1 Admin/Usage.json",
        // UserProfile 1 is the locked Default template (decisions §30 "12c2").
        "UserProfiles/1 Default/profile.json",
        "UserProfiles/2 Admin/profile.json",
        // The tree's split (decisions §30 "Users & Profiles follows storage").
        "UserProfiles/2 Admin/Settings.json",
        "UserProfiles/2 Admin/Content.json",
    ] {
        assert!(names.iter().any(|n| n == want), "{want} in {names:?}");
    }
    let back = from_bytes(&bytes).unwrap();
    assert_eq!(back.show_settings.show_id, show.show_settings.show_id);
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&show).unwrap(),
        "the tree reads back as it was"
    );
}

#[test]
fn pictures_travel_beside_their_manifests() {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend(2u32.to_be_bytes());
    png.extend(3u32.to_be_bytes());
    let sha = crate::media::store(&png, "png").unwrap();
    let mf = |_: u32| lumacore_tree::MediaFile {
        meta: lumacore_base::NodeMeta::new(Some("Stage".into())),
        sha256: sha.clone(),
        ext: "png".into(),
        size: png.len() as u64,
        width: 2,
        height: 3,
        source: "stage.png".into(),
    };
    let mut show = ShowData::new();
    show.images.insert(4, mf(4));
    show.profile_mut(2).unwrap().content.images.insert(1, mf(1));
    let (bytes, missing) = encode(&show).unwrap();
    assert!(missing.is_empty(), "{missing:?}");
    let names = entry_names(&bytes);
    for want in [
        "Images/4 Stage/image.json",
        "Images/4 Stage/image.png",
        "UserProfiles/2 Admin/Content/Images/1 Stage/image.json",
        "UserProfiles/2 Admin/Content/Images/1 Stage/image.png",
    ] {
        assert!(names.iter().any(|n| n == want), "{want} in {names:?}");
    }
    let contents = decode(&bytes).unwrap();
    assert_eq!(contents.media.len(), 2);
    let back = parse_show(&contents.json).unwrap();
    assert_eq!(back.images[&4].sha256, sha);
    assert_eq!(back.user_profiles[&2].content.images[&1].meta.name(), Some("Stage"));
}

#[test]
fn a_cut_store_picture_is_missing_and_never_packed() {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend(7u32.to_be_bytes());
    png.extend(9u32.to_be_bytes());
    png.extend(b"cut-store-test");
    let sha = crate::media::store(&png, "png").unwrap();
    let path = crate::media::store_path(&sha, "png");
    std::fs::write(&path, &png[..png.len() / 2]).unwrap();
    // A save packs from the store: the damaged file is left out, listed.
    let mut show = ShowData::new();
    show.images.insert(
        4,
        lumacore_tree::MediaFile {
            meta: lumacore_base::NodeMeta::new(Some("Cut".into())),
            sha256: sha.clone(),
            ext: "png".into(),
            size: png.len() as u64,
            width: 7,
            height: 9,
            source: "cut.png".into(),
        },
    );
    let (bytes, missing) = encode(&show).unwrap();
    assert_eq!(missing, [format!("{sha}.png")]);
    assert!(!entry_names(&bytes).iter().any(|n| n == "Images/4 Cut/image.png"));
    assert!(!std::path::Path::new(&path).exists(), "the damaged file is removed");
    assert_eq!(crate::media::stored(&sha, "png"), None);
    // Storing the real bytes again heals it.
    crate::media::store(&png, "png").unwrap();
    assert_eq!(crate::media::stored(&sha, "png").as_deref(), Some(&png[..]));
    // A damaged file that still exists is written again by `store`.
    std::fs::write(&path, b"x").unwrap();
    crate::media::store(&png, "png").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), png);
}

#[test]
fn a_cut_store_gdtf_is_missing() {
    let gdtf = b"PK not really a gdtf, cut-store-test".to_vec();
    let hash = store_gdtf(&gdtf).unwrap();
    std::fs::write(store_path(&hash), &gdtf[..10]).unwrap();
    assert_eq!(stored_gdtf(&hash), None);
    assert!(!std::path::Path::new(&store_path(&hash)).exists());
    std::fs::write(store_path(&hash), b"x").unwrap();
    store_gdtf(&gdtf).unwrap();
    assert_eq!(stored_gdtf(&hash), Some(gdtf));
}

#[test]
fn a_changed_part_is_refused_as_damaged() {
    let show = ShowData::new();
    let (bytes, _) = encode(&show).unwrap();
    // Rewrite one JSON entry behind the manifest's back.
    let mut zip = zip::ZipArchive::new(Cursor::new(&bytes[..])).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..zip.len() {
        let mut e = zip.by_index(i).unwrap();
        let name = e.name().to_string();
        let mut data = Vec::new();
        e.read_to_end(&mut data).unwrap();
        if name == "Variables.json" {
            data = b"{\"changed\": true}".to_vec();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
        out.write_all(&data).unwrap();
    }
    let changed = out.finish().unwrap().into_inner();
    let err = from_bytes(&changed).err().unwrap();
    assert!(err.contains("E-SHOW-005"), "{err}");
}

#[test]
fn older_formats_are_refused_with_their_number() {
    let err = from_bytes(br#"{"show_settings": {"format_version": 1}}"#).err().unwrap();
    assert!(err.contains("E-SHOW-002"), "{err}");
    // A format 6 container: one show.json, no manifest.
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("show.json", zip::write::SimpleFileOptions::default()).unwrap();
    zip.write_all(br#"{"show_settings": {"format_version": 6}}"#).unwrap();
    let six = zip.finish().unwrap().into_inner();
    let err = from_bytes(&six).err().unwrap();
    assert!(err.contains("E-SHOW-002") && err.contains("format 6"), "{err}");
}

#[test]
fn folder_names_are_readable_and_safe() {
    assert_eq!(folder("3", Some("Intro")), "3 Intro");
    assert_eq!(folder("3", Some("a/b: c?")), "3 a_b_ c_");
    assert_eq!(folder("3", None), "3");
    assert_eq!(folder("3", Some("..")), "3");
    assert_eq!(folder_no("12 Aura XB").as_deref(), Some("12"));
    assert_eq!(snake(&pascal("preset_groups")), "preset_groups");
    assert_eq!(pascal("distribute_presets"), "DistributePresets");
}

/// A data pool kind is a file named from its field (`Groups.json`); the
/// loader turns the name back into the field. A field with a digit in
/// its name must come back too (R1-18: `views_3d` came back `views3d`
/// and its pool was lost on load).
#[test]
fn data_pool_files_name_their_field_back() {
    for field in ["views_3d", "layouts_2d_grid", "spatial_3d", "preset_groups"] {
        assert_eq!(snake(&pascal(field)), field);
    }
    let show = serde_json::to_value(ShowData::new()).unwrap();
    let dp = show["datapools"]["1"].as_object().unwrap();
    for (k, v) in dp {
        if k != "meta" && v.is_object() {
            assert_eq!(snake(&pascal(k)), *k, "every Datapool kind file reads back to its field");
        }
    }
}

/// A revision's folder is its place in its type's list, from 1: a revision without a stored file
/// writes nothing but keeps its number, so the next one is still `3` (the round 3 audit).
#[test]
fn revision_folders_are_numbered_by_their_place() {
    use lumacore_tree::fixtures::fixture_type::{FixtureType, Revision};
    let (one, three) = (b"PK revision one, folder-test".to_vec(), b"PK revision three, folder-test".to_vec());
    let revision = |hash: String| Revision {
        meta: lumacore_base::NodeMeta::new(None),
        geometries: Vec::new(),
        wheels: Vec::new(),
        emitters: Vec::new(),
        filters: Vec::new(),
        attributes: Vec::new(),
        gdtf_version: "1.2".into(),
        file_hash: hash,
        modes: Vec::new(),
    };
    let ft = FixtureType {
        meta: lumacore_base::NodeMeta::new(Some("Spot".into())),
        manufacturer: "Test".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![revision(store_gdtf(&one).unwrap()), revision(String::new()), revision(store_gdtf(&three).unwrap())],
    };
    let mut show = ShowData::new();
    show.show_settings.format_version = lumacore_tree::FORMAT_VERSION;
    show.devices.dmx.patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    let (bytes, missing) = encode(&show).unwrap();
    assert!(missing.is_empty(), "{missing:?}");
    let revisions: Vec<String> = entry_names(&bytes).into_iter().filter(|n| n.contains("/Revisions/")).collect();
    assert_eq!(
        revisions,
        vec![
            "Devices/Dmx/Patch/FixtureTypes/1 Spot/Revisions/1/source.gdtf".to_string(),
            "Devices/Dmx/Patch/FixtureTypes/1 Spot/Revisions/3/source.gdtf".to_string(),
        ]
    );
}
