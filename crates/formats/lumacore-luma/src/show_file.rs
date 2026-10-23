// ███████╗██╗  ██╗ ██████╗ ██╗    ██╗    ███████╗██╗██╗     ███████╗
// ██╔════╝██║  ██║██╔═══██╗██║    ██║    ██╔════╝██║██║     ██╔════╝
// ███████╗███████║██║   ██║██║ █╗ ██║    █████╗  ██║██║     █████╗
// ╚════██║██╔══██║██║   ██║██║███╗██║    ██╔══╝  ██║██║     ██╔══╝
// ███████║██║  ██║╚██████╔╝╚███╔███╔╝    ██║     ██║███████╗███████╗
// ╚══════╝╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝     ╚═╝     ╚═╝╚══════╝╚══════╝
// FILE: show_file

//! The show file (`.luma`, format 7 — decisions §30 "12a2 / 12a3
//! hardening", §8): a zip holding the ShowData tree as folders and files.
//!
//! ```text
//! manifest.json                    format, show_id, commit; every entry + sha256 + size;
//!                                  the save's facts, counts, the GUID → hash index and
//!                                  the last 20 history rows (format 15, `crate::backups`)
//! Settings.json  Playback.json  Protocols.json  …   one file per top-level field
//! Devices/Devices.json             fixtures, universes, Metadata.MVR (all but the types)
//! Devices/Dmx/Patch/FixtureTypes/12 Aura XB/fixturetype.json
//!                                          /Revisions/1/source.gdtf
//! DataPools/1 Default/datapool.json + Groups.json, PresetGroups.json, Macros.json …
//! Users/1 Admin/user.json + Programmer.json + Selected.json + Usage.json
//! UserProfiles/1 Admin/profile.json  (name, locked)
//!                     /Settings.json  (preferences, theme, lock screen, menus …)
//!                     /Content.json   (attributes, encoder bars, keys, views, screens …)
//!                     /Content/Icons/1 Logo/icon.json + icon.png
//!                     /Content/Images/1 Stage/image.json + image.jpg
//! Images/4 Stage/image.json + image.jpg
//! ```
//!
//! A pool member's folder is `<no> <name>` for reading; the loader trusts only
//! the number and the GUIDs inside. `manifest.json` is written first and read
//! first: a newer or older format is refused before anything else, and every
//! entry's SHA-256 is checked (a damaged JSON part refuses the file,
//! E-SHOW-005; a damaged picture is only left out). Files are pretty JSON.
//!
//! The `.gdtf` is the source of truth of a fixture type: on load each revision
//! is derived again from its file (`lumacore_gdtf::refresh_revision`, GUIDs
//! kept). The station keeps the files it has seen in content-addressed stores:
//! `library/fixturetypes/<sha256>.gdtf` and `library/media/.store/`
//! (`crate::media`). Everything goes through `crate::drive`.
//!
//! Owns:          the .luma format: encode, decode, the manifest, parse_show (format and GUID checks), stored GDTF revisions.
//! Doesn't own:   where shows are kept (backups.rs), the media store (media.rs), loading into a station (lumacore-state).
//! Entry points:  encode_with, decode, from_bytes, from_bytes_upgraded, read_upgraded, manifest_of,
//!                parse_show, parse_show_upgraded.
//! Decisions:     §20, §21 "6d", §30 "12a2", §34 25.10.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use lumacore_tree::ShowData;
use serde_json::{Map, Value};

pub mod upgrade;
pub use upgrade::{OLDEST_FORMAT, Upgraded};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The station's store of GDTF files, by SHA-256.
pub const STORE: &str = "library/fixturetypes";

const MANIFEST: &str = "manifest.json";
/// Format 6 kept the whole show in one entry; it is only read to refuse it
/// with its own format number (E-SHOW-002).
const LEGACY_SHOW: &str = "show.json";

/// Top-level ShowData fields that are one file each.
const FILES: &[(&str, &str)] = &[
    ("show_settings", "Settings.json"),
    ("datapool_priority_orders", "PriorityOrders.json"),
    ("spatial_3d", "Spatial3D.json"),
    ("playback", "Playback.json"),
    ("playbacks", "Playbacks.json"),
    ("session", "LumaSession.json"),
    ("protocols", "Protocols.json"),
    ("effect_engines", "EffectEngines.json"),
    ("variables", "Variables.json"),
    ("assets", "Assets.json"),
    ("attribute_defs", "AttributeDefinitions.json"),
    ("feature_groups", "FeatureGroups.json"),
    ("activation_groups", "ActivationGroups.json"),
    ("attribute_rules", "AttributeRules.json"),
    ("tags", "Tags.json"),
    ("stations", "Stations.json"),
];

/// Top-level fields that are folders (one per pool member).
const FOLDERS: &[&str] = &["devices", "datapools", "users", "user_profiles", "images"];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The store directory (tests keep theirs out of the repo).
fn store_dir() -> String {
    if cfg!(test) {
        std::env::temp_dir().join("lumacore-test-fixturetypes").to_string_lossy().into_owned()
    } else {
        STORE.to_string()
    }
}

fn store_path(hash: &str) -> String {
    format!("{}/{hash}.gdtf", store_dir())
}

/// Keep a GDTF file in the store; returns its hash.
pub fn store_gdtf(bytes: &[u8]) -> Result<String, String> {
    let hash = lumacore_gdtf::file_hash(bytes);
    crate::media::keep_in_store(&store_dir(), &store_path(&hash), &hash, bytes)?;
    Ok(hash)
}

/// A GDTF file from the store (None when missing or damaged).
pub fn stored_gdtf(hash: &str) -> Option<Vec<u8>> {
    if !crate::media::is_hash(hash) {
        return None;
    }
    crate::media::read_from_store(&store_path(hash), hash)
}

// *************************************************************************************************
// Writing
// *************************************************************************************************
/// A pool member's folder name: `<no> <name>` with characters a file system
/// may refuse replaced (the number alone when it has no name).
fn folder(no: &str, name: Option<&str>) -> String {
    let name: String = name
        .unwrap_or("")
        .chars()
        .map(|c| if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
        .take(60)
        .collect();
    let name = name.trim().trim_start_matches('.');
    if name.is_empty() {
        no.to_string()
    } else {
        format!("{no} {name}")
    }
}

/// The number at the start of a folder name.
fn folder_no(folder: &str) -> Option<String> {
    let no = folder.split(' ').next()?;
    no.parse::<u32>().ok().map(|n| n.to_string())
}

/// `preset_groups` → `PresetGroups.json`.
fn pascal(field: &str) -> String {
    field
        .split('_')
        .map(|p| {
            let mut c = p.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default()
        })
        .collect()
}

/// `PresetGroups` → `preset_groups`; a digit after a letter starts a part
/// (`Views3d` → `views_3d`, the field [`pascal`] made it from).
fn snake(name: &str) -> String {
    let mut out = String::new();
    let mut prev: Option<char> = None;
    for c in name.chars() {
        if prev.is_some() && (c.is_uppercase() || (c.is_ascii_digit() && prev.is_some_and(char::is_alphabetic))) {
            out.push('_');
        }
        out.extend(c.to_lowercase());
        prev = Some(c);
    }
    out
}

fn take(map: &mut Map<String, Value>, key: &str) -> Value {
    map.remove(key).unwrap_or(Value::Null)
}

/// A folder's label: an object's own name key, its display name, or a user's login.
fn name_of(v: &Value) -> Option<&str> {
    v.get("name")
        .or_else(|| v.get("meta").and_then(|m| m.get("name")).filter(|n| !n.is_null()))
        .or_else(|| v.get("login"))
        .and_then(Value::as_str)
}

/// What the container holds while it is written: path → (bytes, compress).
#[derive(Default)]
struct Entries {
    files: BTreeMap<String, (Vec<u8>, bool)>,
    missing: Vec<String>,
}

impl Entries {
    fn json(&mut self, path: String, v: &Value) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(v).map_err(|e| format!("Serialization error: {e}"))?;
        self.files.insert(path, (bytes, true));
        Ok(())
    }

    /// A picture's manifest and its file (`<dir>/<kind>.json` + `<kind>.<ext>`).
    fn media(&mut self, dir: &str, kind: &str, mf: &Value) -> Result<(), String> {
        self.json(format!("{dir}/{kind}.json"), mf)?;
        let sha = mf["sha256"].as_str().unwrap_or("");
        let ext = mf["ext"].as_str().unwrap_or("");
        match crate::media::stored(sha, ext) {
            // Pictures are compressed already (SVG is text).
            Some(bytes) => {
                self.files.insert(format!("{dir}/{kind}.{ext}"), (bytes, ext == "svg"));
            }
            None => self.missing.push(format!("{sha}.{ext}")),
        }
        Ok(())
    }

    /// `UserProfiles/<no> <name>/`: profile.json, Settings.json, Content.json
    /// and the content's own pools — as the tree (`….Settings | Content`).
    fn profile(&mut self, no: &str, mut profile: Value) -> Result<(), String> {
        let dir = format!("UserProfiles/{}", folder(no, name_of(&profile)));
        if let Value::Object(p) = &mut profile {
            let settings = take(p, "settings");
            let mut content = take(p, "content");
            if let Value::Object(c) = &mut content {
                self.media_pool(&format!("{dir}/Content/Icons"), "icon", take(c, "icons"))?;
                self.media_pool(&format!("{dir}/Content/Images"), "image", take(c, "images"))?;
            }
            self.json(format!("{dir}/Settings.json"), &settings)?;
            self.json(format!("{dir}/Content.json"), &content)?;
        }
        self.json(format!("{dir}/profile.json"), &profile)
    }

    /// The zip: `manifest` (with the entry list added) first, then the
    /// entries in path order — the same content gives the same bytes.
    fn pack(self, mut manifest: Value) -> Result<(Vec<u8>, Vec<String>), String> {
        manifest["entries"] = Value::Array(
            self.files
                .iter()
                .map(|(path, (bytes, _))| {
                    serde_json::json!({
                        "path": path,
                        "sha256": crate::media::sha256(bytes),
                        "size": bytes.len(),
                    })
                })
                .collect(),
        );
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let deflated = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(zip::DateTime::default());
        let stored = deflated.compression_method(zip::CompressionMethod::Stored);
        let manifest = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
        zip.start_file(MANIFEST, deflated).map_err(|e| e.to_string())?;
        zip.write_all(&manifest).map_err(|e| e.to_string())?;
        for (path, (bytes, compress)) in &self.files {
            zip.start_file(path.as_str(), if *compress { deflated } else { stored })
                .map_err(|e| e.to_string())?;
            zip.write_all(bytes).map_err(|e| e.to_string())?;
        }
        let bytes = zip.finish().map_err(|e| e.to_string())?.into_inner();
        let mut missing = self.missing;
        missing.sort();
        missing.dedup();
        Ok((bytes, missing))
    }

    /// A pool of pictures under `dir` (`Images`, `Icons`).
    fn media_pool(&mut self, dir: &str, kind: &str, pool: Value) -> Result<(), String> {
        if let Value::Object(pool) = pool {
            for (no, mf) in pool {
                self.media(&format!("{dir}/{}", folder(&no, name_of(&mf))), kind, &mf)?;
            }
        }
        Ok(())
    }
}

/// The container's bytes, and the files the store didn't have (GDTF hashes,
/// `<sha256>.<ext>` pictures): the show is still written, those parts can't
/// be restored elsewhere.
pub fn encode(show: &ShowData) -> Result<(Vec<u8>, Vec<String>), String> {
    encode_with(show, &Value::Null)
}

/// [`encode`] with the save's facts in the manifest (decisions §30 "12d
/// part 5": `saved_at`, `reason`, `station`, `user`, `note`, `history`;
/// see `crate::backups`). The manifest always carries `counts` and
/// the per-kind GUID → hash `index`.
pub fn encode_with(show: &ShowData, facts: &Value) -> Result<(Vec<u8>, Vec<String>), String> {
    let Value::Object(mut top) = serde_json::to_value(show).map_err(|e| format!("Serialization error: {e}"))? else {
        return Err("Serialization error: the show is not an object".into());
    };
    // Echoes are replicated, never saved (decisions §32).
    if let Some(p) = top.get_mut("playbacks").and_then(Value::as_object_mut) {
        p.remove("echoes");
    }
    let mut e = Entries::default();
    for (field, file) in FILES {
        e.json((*file).to_string(), &take(&mut top, field))?;
    }

    // Devices: the patch, and a folder per fixture type with its GDTF files.
    let mut devices = take(&mut top, "devices");
    let types = devices
        .get_mut("dmx")
        .and_then(|d| d.get_mut("patch"))
        .and_then(|p| p.get_mut("fixture_types"))
        .map(|t| std::mem::replace(t, Value::Object(Map::new())))
        .unwrap_or(Value::Null);
    e.json("Devices/Devices.json".into(), &devices)?;
    if let Value::Object(types) = types {
        for (no, ft) in types {
            let dir = format!("Devices/Dmx/Patch/FixtureTypes/{}", folder(&no, name_of(&ft)));
            for (i, rev) in ft["revisions"].as_array().into_iter().flatten().enumerate() {
                let hash = rev["file_hash"].as_str().unwrap_or("");
                if hash.is_empty() {
                    continue;
                }
                // A revision's folder is its place in the list, from 1 (the walk numbers it so).
                let rev_no = i + 1;
                match stored_gdtf(hash) {
                    // GDTF files are zips already: stored, not compressed again.
                    Some(bytes) => {
                        e.files.insert(format!("{dir}/Revisions/{rev_no}/source.gdtf"), (bytes, false));
                    }
                    None => e.missing.push(hash.to_string()),
                }
            }
            e.json(format!("{dir}/fixturetype.json"), &ft)?;
        }
    }

    // Data pools: datapool.json + one file per pool kind.
    if let Value::Object(pools) = take(&mut top, "datapools") {
        for (no, dp) in pools {
            let dir = format!("DataPools/{}", folder(&no, name_of(&dp)));
            let Value::Object(dp) = dp else { continue };
            let mut rest = Map::new();
            for (k, v) in dp {
                if k != "meta" && v.is_object() {
                    e.json(format!("{dir}/{}.json", pascal(&k)), &v)?;
                } else {
                    rest.insert(k, v);
                }
            }
            e.json(format!("{dir}/datapool.json"), &Value::Object(rest))?;
        }
    }

    // Users: identity, programmer (with the fixture selection), one-only
    // selections, usage (decisions §30 "12b2").
    if let Value::Object(users) = take(&mut top, "users") {
        for (no, mut user) in users {
            let dir = format!("Users/{}", folder(&no, name_of(&user)));
            if let Value::Object(u) = &mut user {
                e.json(format!("{dir}/Programmer.json"), &take(u, "programmer"))?;
                e.json(format!("{dir}/Selected.json"), &take(u, "selected"))?;
                e.json(format!("{dir}/Usage.json"), &take(u, "usage"))?;
            }
            e.json(format!("{dir}/user.json"), &user)?;
        }
    }

    // Profiles: the profile with its own Icons and Images.
    if let Value::Object(profiles) = take(&mut top, "user_profiles") {
        for (no, profile) in profiles {
            e.profile(&no, profile)?;
        }
    }

    // The show's pictures.
    e.media_pool("Images", "image", take(&mut top, "images"))?;

    if let Some(field) = top.keys().next() {
        return Err(format!("The show field `{field}` has no place in the .luma (show_file::FILES)"));
    }

    let settings = &show.show_settings;
    let index = crate::backups::index(show);
    let mut manifest = serde_json::json!({
        "format": lumacore_tree::FORMAT_VERSION,
        "kind": "show",
        "show_id": settings.show_id,
        "show_name": settings.show_name,
        "saved_with_commit": settings.saved_with_commit,
        "description": settings.description,
        "counts": crate::backups::counts(&index),
        "index": index,
    });
    if let (Some(m), Some(f)) = (manifest.as_object_mut(), facts.as_object()) {
        for (k, v) in f {
            m.insert(k.clone(), v.clone());
        }
    }
    e.pack(manifest)
}

/// A show file's manifest (the first entry; nothing else is read or
/// checked). None = not a container.
pub fn manifest_of(path: &std::path::Path) -> Option<Value> {
    let file = crate::drive::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).ok()?;
    let mut entry = zip.by_name(MANIFEST).ok()?;
    let mut data = Vec::new();
    entry.read_to_end(&mut data).ok()?;
    serde_json::from_slice(&data).ok()
}

/// A profile file (`.lumaprofile`, `Export UserProfile`): the same layout as
/// a show's `UserProfiles/<no> <name>/` with its pictures, and a manifest.
pub fn encode_profile(no: u32, profile: &lumacore_tree::UserProfile) -> Result<(Vec<u8>, Vec<String>), String> {
    let mut e = Entries::default();
    e.profile(&no.to_string(), serde_json::to_value(profile).map_err(|e| e.to_string())?)?;
    e.pack(serde_json::json!({
        "format": lumacore_tree::FORMAT_VERSION,
        "kind": "userprofile",
        "profile": profile.meta.identity.guid,
        "saved_with_commit": crate::build_info().git_commit,
    }))
}

/// A profile file's pictures: `(ext, bytes)`.
pub type Pictures = Vec<(String, Vec<u8>)>;

/// Read a profile file: the profile and its pictures.
pub fn decode_profile(bytes: &[u8]) -> Result<(lumacore_tree::UserProfile, Pictures), String> {
    let contents = decode(bytes)?;
    let mut raw: Value = serde_json::from_str(&contents.json).map_err(|e| e.to_string())?;
    // An older profile file upgrades like a show (decisions 0289); one without a format is today's.
    if raw["show_settings"]["format_version"].is_u64() {
        upgrade::upgrade(&mut raw).map_err(|e| e.replace("This show was", "This profile was"))?;
    }
    let profile = raw["user_profiles"]
        .as_object()
        .and_then(|p| p.values().next())
        .ok_or_else(|| damaged("no profile in the file".into()))?;
    let profile = serde_json::from_value(profile.clone()).map_err(|e| damaged(format!("profile.json: {e}")))?;
    Ok((profile, contents.media))
}

// *************************************************************************************************
// Reading
// *************************************************************************************************
/// What a container holds: the show as one JSON document (the tree put
/// together again), its GDTF files and its pictures.
pub struct Contents {
    pub json: String,
    pub gdtf: Vec<(String, Vec<u8>)>,
    /// `(ext, bytes)` of every picture whose bytes match the manifest.
    pub media: Vec<(String, Vec<u8>)>,
}

fn damaged(detail: String) -> String {
    lumacore_base::with_code(format!("The show file is damaged: {detail}"), "E-SHOW-005")
}

/// Put `value` at `map[key]` (an object created on the way).
fn put(map: &mut Map<String, Value>, key: &str, value: Value) {
    map.insert(key.to_string(), value);
}

fn object<'a>(map: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    let v = map.entry(key.to_string()).or_insert_with(|| Value::Object(Map::new()));
    if !v.is_object() {
        *v = Value::Object(Map::new());
    }
    v.as_object_mut().expect("made an object above")
}

/// Open a container: check the manifest (format first, then every entry's
/// SHA-256) and put the tree together again. A show of a format this build
/// can't read (`upgrade::readable`) comes back as its settings alone, so it is
/// refused with its own number (E-SHOW-001 / 002); an older readable one comes
/// whole, for `parse_show` to upgrade.
pub fn decode(bytes: &[u8]) -> Result<Contents, String> {
    if !bytes.starts_with(b"PK") {
        let json = String::from_utf8(bytes.to_vec()).map_err(|_| "Invalid show file: not a show".to_string())?;
        return Ok(Contents { json, gdtf: Vec::new(), media: Vec::new() });
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("Invalid show file: {e}"))?;
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| format!("Invalid show file: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| damaged(format!("{name}: {e}")))?;
        entries.insert(name, data);
    }
    let Some(manifest) = entries.remove(MANIFEST) else {
        // Format 6 and older: one show.json (refused by its format).
        let json = entries
            .remove(LEGACY_SHOW)
            .ok_or_else(|| format!("Invalid show file: no {MANIFEST}"))?;
        let json = String::from_utf8(json).map_err(|e| format!("Invalid show file: {e}"))?;
        return Ok(Contents { json, gdtf: Vec::new(), media: Vec::new() });
    };
    let manifest: Value = serde_json::from_slice(&manifest).map_err(|e| damaged(format!("{MANIFEST}: {e}")))?;
    let format = manifest["format"].as_u64().unwrap_or(0);
    // A format this build can't read comes back as its settings alone, refused by its number.
    if upgrade::readable(format).is_err() {
        let settings = serde_json::json!({ "show_settings": {
            "format_version": format,
            "saved_with_commit": manifest["saved_with_commit"],
            "show_name": manifest["show_name"],
        }});
        return Ok(Contents { json: settings.to_string(), gdtf: Vec::new(), media: Vec::new() });
    }

    // Every listed entry is there and is what was saved.
    let mut bad_pictures = std::collections::BTreeSet::new();
    for listed in manifest["entries"].as_array().into_iter().flatten() {
        let path = listed["path"].as_str().unwrap_or("");
        let want = listed["sha256"].as_str().unwrap_or("");
        let is_json = path.ends_with(".json");
        match entries.get(path) {
            Some(data) if crate::media::sha256(data) == want => {}
            _ if !is_json && !path.ends_with(".gdtf") => {
                bad_pictures.insert(path.to_string());
            }
            Some(_) => return Err(damaged(format!("{path} doesn't match its SHA-256"))),
            None => return Err(damaged(format!("{path} is missing"))),
        }
    }

    let parse = |path: &str, data: &[u8]| -> Result<Value, String> {
        serde_json::from_slice(data).map_err(|e| damaged(format!("{path}: {e}")))
    };
    let mut top = Map::new();
    let mut gdtf = Vec::new();
    let mut media = Vec::new();
    for (field, file) in FILES {
        if let Some(data) = entries.get(*file) {
            put(&mut top, field, parse(file, data)?);
        }
    }
    let mut devices = Value::Null;
    let mut types = Map::new();
    for (path, data) in &entries {
        let parts: Vec<&str> = path.split('/').collect();
        match parts.as_slice() {
            ["Devices", "Devices.json"] => devices = parse(path, data)?,
            ["Devices", "Dmx", "Patch", "FixtureTypes", dir, "fixturetype.json"] => {
                let no = folder_no(dir).ok_or_else(|| damaged(format!("{path}: no number")))?;
                types.insert(no, parse(path, data)?);
            }
            ["Devices", "Dmx", "Patch", "FixtureTypes", _, "Revisions", _, "source.gdtf"] => {
                gdtf.push((lumacore_gdtf::file_hash(data), data.clone()));
            }
            ["DataPools", dir, file] if file.ends_with(".json") => {
                let no = folder_no(dir).ok_or_else(|| damaged(format!("{path}: no number")))?;
                let dp = object(object(&mut top, "datapools"), &no);
                if *file == "datapool.json" {
                    if let Value::Object(rest) = parse(path, data)? {
                        dp.extend(rest);
                    }
                } else {
                    put(dp, &snake(file.trim_end_matches(".json")), parse(path, data)?);
                }
            }
            ["Users", dir, file] => {
                let no = folder_no(dir).ok_or_else(|| damaged(format!("{path}: no number")))?;
                let user = object(object(&mut top, "users"), &no);
                match *file {
                    "user.json" => {
                        if let Value::Object(rest) = parse(path, data)? {
                            user.extend(rest);
                        }
                    }
                    "Programmer.json" => put(user, "programmer", parse(path, data)?),
                    "Selected.json" => put(user, "selected", parse(path, data)?),
                    "Usage.json" => put(user, "usage", parse(path, data)?),
                    _ => {}
                }
            }
            ["UserProfiles", dir, "profile.json"] => {
                let no = folder_no(dir).ok_or_else(|| damaged(format!("{path}: no number")))?;
                let profile = object(object(&mut top, "user_profiles"), &no);
                if let Value::Object(rest) = parse(path, data)? {
                    profile.extend(rest);
                }
            }
            ["UserProfiles", dir, part @ ("Settings.json" | "Content.json")] => {
                let no = folder_no(dir).ok_or_else(|| damaged(format!("{path}: no number")))?;
                let profile = object(object(&mut top, "user_profiles"), &no);
                let key = if *part == "Settings.json" { "settings" } else { "content" };
                if let Value::Object(rest) = parse(path, data)? {
                    object(profile, key).extend(rest);
                }
            }
            // tidy: not-keywords (folder names of the file layout)
            ["UserProfiles", dir, "Content", pool @ ("Icons" | "Images"), item, file] => {
                let no = folder_no(dir).ok_or_else(|| damaged(format!("{path}: no number")))?;
                let item_no = folder_no(item).ok_or_else(|| damaged(format!("{path}: no number")))?;
                let key = if *pool == "Icons" { "icons" } else { "images" };
                if file.ends_with(".json") {
                    let profile = object(object(&mut top, "user_profiles"), &no);
                    put(object(object(profile, "content"), key), &item_no, parse(path, data)?);
                } else if !bad_pictures.contains(path.as_str()) {
                    media.push((extension(file), data.clone()));
                }
            }
            ["Images", item, file] => {
                let item_no = folder_no(item).ok_or_else(|| damaged(format!("{path}: no number")))?;
                if file.ends_with(".json") {
                    put(object(&mut top, "images"), &item_no, parse(path, data)?);
                } else if !bad_pictures.contains(path.as_str()) {
                    media.push((extension(file), data.clone()));
                }
            }
            _ => {}
        }
    }
    // The fixture types go back into the patch.
    if let Some(p) = devices.get_mut("dmx").and_then(|d| d.get_mut("patch")).and_then(Value::as_object_mut) {
        p.insert("fixture_types".into(), Value::Object(types));
    }
    put(&mut top, "devices", devices);
    for f in FOLDERS {
        if !top.contains_key(*f) {
            put(&mut top, f, Value::Object(Map::new()));
        }
    }
    // In the show's own field order (readers compare documents); a tree that
    // doesn't read as a show goes on as it is, so `parse_show` names the error.
    let top = Value::Object(top);
    let json = match serde_json::from_value::<ShowData>(top.clone()) {
        Ok(show) => serde_json::to_string(&show),
        Err(_) => serde_json::to_string(&top),
    }
    .map_err(|e| e.to_string())?;
    Ok(Contents { json, gdtf, media })
}

fn extension(file: &str) -> String {
    file.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default()
}

/// Write `show` to `path` (through a temporary file); returns the files the
/// store was missing (see [`encode`]).
pub fn write(path: &std::path::Path, show: &ShowData, on_chunk: impl FnMut(usize)) -> Result<Vec<String>, String> {
    let (bytes, missing) = encode(show)?;
    write_bytes(path, &bytes, on_chunk)?;
    Ok(missing)
}

/// Write an encoded container to `path` atomically (`drive::write_atomic_with`:
/// a temporary file, synced, renamed over it, the folder synced; R1-14/15).
pub fn write_bytes(path: &std::path::Path, bytes: &[u8], on_chunk: impl FnMut(usize)) -> Result<(), String> {
    crate::drive::write_atomic_with(path, bytes, on_chunk)
        .map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

/// Read a show file: unpack its GDTF files and pictures into the stores,
/// check the format (`parse_show`) and derive every fixture type revision
/// again from its file.
pub fn read(path: &std::path::Path) -> Result<ShowData, String> {
    let bytes = crate::drive::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
    from_bytes(&bytes)
}

/// [`read`] from bytes.
pub fn from_bytes(bytes: &[u8]) -> Result<ShowData, String> {
    from_bytes_upgraded(bytes).map(|(show, _)| show)
}

/// [`read`], and what an upgrade did to an older show (decisions 0289).
pub fn read_upgraded(path: &std::path::Path) -> Result<(ShowData, Option<Upgraded>), String> {
    let bytes = crate::drive::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
    from_bytes_upgraded(&bytes)
}

/// [`from_bytes`], and what an upgrade did to an older show.
pub fn from_bytes_upgraded(bytes: &[u8]) -> Result<(ShowData, Option<Upgraded>), String> {
    let contents = decode(bytes)?;
    let (mut show, upgraded) = parse_show_upgraded(&contents.json)?;
    // The stores are caches: a show still loads when they can't be written.
    for (_, data) in &contents.gdtf {
        let _ = store_gdtf(data);
    }
    for (ext, data) in &contents.media {
        let _ = crate::media::store(data, ext);
    }
    refresh(&mut show, &contents.gdtf);
    Ok((show, upgraded))
}

/// Derive every revision that has a file again from it (the container's
/// copy, else the store's). A revision without its file keeps what the show
/// says.
pub fn refresh(show: &mut ShowData, packed: &[(String, Vec<u8>)]) {
    for ft in show.devices.dmx.patch.fixture_types.values_mut().map(std::sync::Arc::make_mut) {
        for rev in &mut ft.revisions {
            if rev.file_hash.is_empty() {
                continue;
            }
            let bytes = packed
                .iter()
                .find(|(h, _)| *h == rev.file_hash)
                .map(|(_, b)| b.clone())
                .or_else(|| stored_gdtf(&rev.file_hash));
            if let Some(bytes) = bytes
                && let Err(e) = lumacore_gdtf::refresh_revision(rev, &bytes)
            {
                log::warn!("fixture type revision {}: {e}", rev.file_hash);
            }
        }
    }
}

/// The show's format and the build that saved it, read from a show file.
pub fn format_of(path: &std::path::Path) -> Option<(u64, String)> {
    let bytes = crate::drive::read(path).ok()?;
    let raw: serde_json::Value = serde_json::from_str(&decode(&bytes).ok()?.json).ok()?;
    let settings = &raw["show_settings"];
    Some((
        settings["format_version"].as_u64()?,
        settings["saved_with_commit"]
            .as_str()
            .filter(|c| !c.is_empty())
            .unwrap_or("unknown")
            .to_string(),
    ))
}

/// Parse a show file (decisions 0289): a newer format than this build reads is refused, naming the
/// build that saved it (E-SHOW-001); one older than `OLDEST_FORMAT` too (E-SHOW-002); one in
/// between is upgraded step by step first.
pub fn parse_show(text: &str) -> Result<ShowData, String> {
    parse_show_upgraded(text).map(|(show, _)| show)
}

/// [`parse_show`], and what an upgrade did.
pub fn parse_show_upgraded(text: &str) -> Result<(ShowData, Option<Upgraded>), String> {
    let mut raw: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Invalid show file: {e}"))?;
    let settings = &raw["show_settings"];
    let version = settings["format_version"].as_u64().unwrap_or(0);
    if version > u64::from(lumacore_tree::FORMAT_VERSION) {
        let commit = settings["saved_with_commit"].as_str().unwrap_or("unknown");
        return Err(lumacore_base::with_code(
            format!(
                "This show was saved in format {version} (build {commit}); this build reads formats {OLDEST_FORMAT} to {}.",
                lumacore_tree::FORMAT_VERSION
            ),
            "E-SHOW-001",
        ));
    }
    let upgraded = upgrade::upgrade(&mut raw)?;
    let show: ShowData = serde_json::from_value(raw).map_err(|e| {
        let e = e.to_string();
        if e.contains("missing field `guid`") || e.contains("missing field `show_id`") {
            lumacore_base::with_code(format!("An object has no GUID: {e}"), "E-SHOW-003")
        } else {
            format!("Invalid show file: {e}")
        }
    })?;
    // Every object has its own GUID: missing or shared ones are refused.
    let id = show.identity();
    if !id.duplicates.is_empty() || !id.nil.is_empty() {
        let mut paths: Vec<String> = id.nil.iter().map(|p| format!("{p} (no GUID)")).collect();
        paths.extend(
            id.duplicates
                .iter()
                .map(|(g, ps)| format!("{} share {g}", ps.join(" and "))),
        );
        paths.truncate(5);
        return Err(lumacore_base::with_code(
            format!("Objects without a unique GUID: {}", paths.join("; ")),
            "E-SHOW-003",
        ));
    }
    lumacore_tree::LineOps::check_protocols(&show)?;
    Ok((show, upgraded))
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
