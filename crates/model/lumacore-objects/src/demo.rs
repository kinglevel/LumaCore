//! The demo show (`library/shows/demo.luma`, decisions §40 14–17): The International 2026 rig,
//! built from the sources in git through the station's own words, so it always matches the
//! current `ShowData` — `Import Mvr "ti2026.mvr"` (1 348 fixtures and 23 folders, 106
//! universes), one sACN line per run of universes, the user LD, `Import PoolXml "ti2026-groups.xml"
//! Groups` (77 groups), then the manual demo's lines (`library/demo/manual.lum`, the user's own
//! script: the profile, what is to be kept). The built file is not committed: `ensure` makes it
//! when it is missing or stale (the station's start, the e2e support, `gen-demo --if-stale`);
//! the catalog test checks its counts and a golden of the normalized JSON (`fingerprint`).

use std::path::{Path, PathBuf};

use lumacore_tree::DriveKind;

use crate::commands::dispatch;
use crate::{AppState, DispatchResult};

/// The repo root (`LUMACORE_ROOT`, set in `.cargo/config.toml`; decisions §34 23.6).
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("LUMACORE_ROOT"))
}

/// Where the built demo show lives (gitignored).
pub fn demo_path(root: &Path) -> PathBuf {
    root.join("library/shows/demo.luma")
}

/// The sources the show is built from, all in git: the MVR, the exported groups, the manual demo.
pub fn sources(root: &Path) -> [PathBuf; 3] {
    [
        root.join("library/mvr/ti2026.mvr"),
        root.join("library/poolxml/ti2026-groups.xml"),
        root.join("library/demo/manual.lum"),
    ]
}

/// The demo show and the command lines that complained while building it.
pub struct Demo {
    pub state: AppState,
    pub warnings: Vec<String>,
}

impl Demo {
    /// The show as pretty JSON (the container's `show.json`).
    pub fn json(&self) -> String {
        serde_json::to_string_pretty(&self.state.root.show_data).expect("ShowData serializes")
    }

    /// The show file: the container with the GDTF files (`lumacore_luma::show_file`).
    pub fn file(&self) -> Vec<u8> {
        lumacore_luma::show_file::encode(&self.state.root.show_data)
            .expect("the demo show encodes")
            .0
    }
}

/// Build the demo show from the sources: the rig imported into a claimed Devices draft and
/// kept, an sACN line per run of universes, the user LD, the exported groups, then every line of the
/// manual demo.
pub fn build(root: &Path) -> Demo {
    let mut state = AppState::new();
    let mut warnings = Vec::new();
    let mut run = |state: &mut AppState, cmd: &str| match dispatch(state, cmd) {
        DispatchResult::Output(text) => {
            let plain = crate::text::strip_ansi(&text);
            if plain.to_lowercase().contains("error") {
                warnings.push(format!("{cmd} → {}", plain.trim()));
            }
        }
        DispatchResult::ContextSwitch(ctx) => state.context = ctx,
        _ => warnings.push(format!("{cmd} → unexpected result")),
    };

    // The repo's library as the local library, wherever the process runs (the file words read
    // `/Drive`'s library, `AppState::library_folder`).
    state.drives.push(crate::drives::Drive {
        id: crate::drives::LOCAL.to_string(),
        kind: DriveKind::LocalLibrary,
        label: "Internal".to_string(),
        mount: root.join("library"),
        present: true,
        free: 0,
        total: 0,
        since_ms: 0,
    });
    let drive = format!("/Drive \"{}\"", crate::drives::LOCAL);

    // ----- The rig -----
    // Enter Patch (the tree shell's word; the walk and its claim are the kernel's).
    let _ = crate::commands::navigate::cmd_enter(&mut state, &["patch"]);
    run(&mut state, "Claim Root.ShowData.Devices");
    run(&mut state, &format!("Import Mvr \"ti2026.mvr\" {drive} /IdType Media"));
    run(&mut state, "Import Mvr /Apply /Gone Recreate");
    // Keep the draft: the patch goes live and the fixtures' attributes get
    // their definitions in the show's grouping (decisions §30 "12f-a").
    run(&mut state, "Unclaim Root.ShowData.Devices /Keep /Default");

    // ----- Output -----
    // One sACN line per run of universes, each starting on the wire at its first universe.
    let universes: Vec<u32> = state.root.show_data.devices.dmx.patch.universes.keys().copied().collect();
    for (n, (first, last)) in runs(&universes).into_iter().enumerate() {
        let line = n + 1;
        run(&mut state, &format!("Store Sacn {line}"));
        run(&mut state, &format!("Assign Universe {first} Thru {last} At Sacn {line} /Universe {first}"));
    }

    // ----- Users -----
    // Admin exists from AppState::new(); LD programs (rights Program).
    run(&mut state, "Store User \"LD\" /Rights Program");

    // ----- Groups -----
    run(&mut state, &format!("Import PoolXml \"ti2026-groups.xml\" Groups {drive}"));

    // ----- The manual demo -----
    let script = lumacore_luma::drive::read_to_string(&sources(root)[2]);
    for line in script.iter().flat_map(|s| s.lines()).map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        run(&mut state, line);
    }
    run(&mut state, "ResetAll");
    drop(run);
    if let Err(e) = script {
        warnings.push(format!("library/demo/manual.lum → {e}"));
    }

    state.root.show_data.show_settings.show_name = "Demo".to_string();
    // A new show has learned nothing yet (building it counted what it ran,
    // decisions §30 "12b2"): no usage, so every build is the same.
    for user in state.root.show_data.users.values_mut() {
        user.usage = Default::default();
    }
    Demo { state, warnings }
}

/// A fresh station with the demo show in place, for tests: the show is built once per process
/// (20 s in debug — every tracked line fingerprints the whole rig) and each caller gets a copy
/// put in place the way a load does (`replace_show_data`).
pub fn fresh_state() -> AppState {
    static SHOW: std::sync::OnceLock<Template> = std::sync::OnceLock::new();
    SHOW.get_or_init(|| Template::of(build(&repo_root()).state)).fresh()
}

/// What a built show leaves on the station besides the show: the GDTF files it imported
/// (`gdtf_sources`, the file names an MVR import matches).
struct Template {
    show: lumacore_tree::ShowData,
    gdtf_sources: std::collections::HashMap<u32, crate::FixtureTypeSource>,
}

impl Template {
    fn of(state: AppState) -> Self {
        Template { show: state.root.show_data, gdtf_sources: state.gdtf_sources }
    }

    fn fresh(&self) -> AppState {
        let mut state = AppState::new();
        state.replace_show_data(self.show.clone());
        state.gdtf_sources = self.gdtf_sources.clone();
        state
    }
}

/// The runs of consecutive numbers: `[1, 2, 3, 5]` → `[(1, 3), (5, 5)]`.
pub fn runs(numbers: &[u32]) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for &n in numbers {
        match out.last_mut() {
            Some((_, last)) if *last + 1 == n => *last = n,
            _ => out.push((n, n)),
        }
    }
    out
}

/// Build the show and write it, when the file is missing or stale: a newer source, or a file of
/// another format. The one door every start takes (the station, the e2e support, the scripts);
/// a build that complains or fails `check` writes nothing. Returns the path and whether it wrote.
pub fn ensure(root: &Path) -> Result<(PathBuf, bool), String> {
    let path = demo_path(root);
    if !stale(root) {
        return Ok((path, false));
    }
    let demo = build(root);
    if !demo.warnings.is_empty() {
        return Err(format!("building the demo complained:\n{}", demo.warnings.join("\n")));
    }
    check(&demo.state)?;
    write(root, &demo)?;
    Ok((path, true))
}

/// When `path` is the demo show: build it from its sources if it is missing or stale (the one door,
/// for a start with it and `LoadShow "demo"`). A note when it was built or couldn't be.
pub fn ensure_if_demo(path: &Path) -> Option<String> {
    let root = repo_root();
    let demo = demo_path(&root);
    // The demo by its place: the show a load or a start names as `library/shows/demo.luma`.
    let same = path == demo || path.ends_with("library/shows/demo.luma");
    if !same {
        return None;
    }
    match ensure(&root) {
        Ok((built, true)) => Some(format!("demo show built from its sources: {}", built.display())),
        Ok((_, false)) => None,
        Err(e) => Some(format!("[ERROR] the demo show could not be built: {e}")),
    }
}

/// Write the built show as `library/shows/demo.luma` (atomically: two stations starting at
/// once never read a torn file).
pub fn write(root: &Path, demo: &Demo) -> Result<PathBuf, String> {
    let path = demo_path(root);
    if let Some(dir) = path.parent() {
        lumacore_luma::drive::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    lumacore_luma::drive::write_atomic(&path, demo.file()).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// Whether the built file is missing, older than a source, or of another format.
pub fn stale(root: &Path) -> bool {
    let path = demo_path(root);
    let Ok(meta) = lumacore_luma::drive::metadata(&path) else { return true };
    let Ok(written) = meta.modified() else { return true };
    let newer_source = sources(root)
        .iter()
        .filter_map(|s| lumacore_luma::drive::metadata(s).ok()?.modified().ok())
        .any(|m| m > written);
    if newer_source {
        return true;
    }
    let format = lumacore_luma::show_file::format_of(&path).map(|(f, _)| f);
    format != Some(u64::from(lumacore_tree::FORMAT_VERSION))
}

/// What a show must have to count as the demo.
pub fn check(state: &AppState) -> Result<(), String> {
    let sd = &state.root.show_data;
    let patch = &sd.devices.dmx.patch;
    let fixtures = patch.fids().len();
    let folders = patch.fixtures.values().filter(|f| f.mvr.fid.is_none()).count();
    let universes = patch.universes.len();
    let outputs = sd.protocols.all_lines().filter(|(_, _, l)| l.mode == lumacore_protocols::Direction::Out).count();
    let users = sd.user_profiles.len();
    let sacn = sd.protocols.lines("Sacn").is_some_and(|m| m.values().any(|l| l.amount > 0));
    let groups: usize = sd.datapools.values().map(|d| d.groups.len()).sum();
    let problems: Vec<String> = [
        (fixtures >= 1000, format!("≥1000 numbered fixtures, got {fixtures}")),
        (folders >= 10, format!("≥10 folders, got {folders}")),
        (universes >= 100, format!("≥100 universes, got {universes}")),
        (outputs >= 1, format!("an output line, got {outputs}")),
        (sd.users.values().any(|u| u.login == "LD"), "the user LD".to_string()),
        (users >= 1, format!("a user profile, got {users}")),
        (sacn, "an sACN line carrying universes".to_string()),
        (groups >= 50, format!("≥50 groups, got {groups}")),
        (sd.show_settings.show_name == "Demo", "the show named Demo".to_string()),
        (
            patch.fixtures.values().count() > 0
                && lumacore_tree::attribute_resolver::used_attributes(sd)
                    .keys()
                    .all(|n| sd.attribute_defs.get(n).is_some_and(|d| d.feature_group.is_some())),
            "every patched attribute defined and grouped (decisions §30 \"12f-a\")".to_string(),
        ),
    ]
    .into_iter()
    .filter(|(ok, _)| !ok)
    .map(|(_, want)| format!("expected {want}"))
    .collect();
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// The show's counts and the SHA-256 of its normalized JSON: the golden the catalog test keeps
/// (`tests/catalog/testdata/demo.golden.txt`), small where the JSON is megabytes.
pub fn fingerprint(demo: &Demo) -> String {
    use sha2::{Digest, Sha256};
    let sd = &demo.state.root.show_data;
    let patch = &sd.devices.dmx.patch;
    let json = serde_json::to_string(&normalized(&demo.json())).expect("JSON");
    let hash = Sha256::digest(json.as_bytes());
    let hash: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    let lines = [
        format!("format {}", sd.show_settings.format_version),
        format!("fixtures {}", patch.fids().len()),
        format!("folders {}", patch.fixtures.values().filter(|f| f.mvr.fid.is_none()).count()),
        format!("fixture_types {}", patch.fixture_types.len()),
        format!("universes {}", patch.universes.len()),
        format!("lines {}", sd.protocols.all_lines().count()),
        format!("users {}", sd.users.len()),
        format!("profiles {}", sd.user_profiles.len()),
        format!("groups {}", sd.datapools.values().map(|d| d.groups.len()).sum::<usize>()),
        format!("sha256 {hash}"),
    ];
    lines.join("\n") + "\n"
}

/// The show as JSON with every GUID replaced by its order of appearance, so
/// two builds compare equal when only their fresh GUIDs differ (links still
/// have to point at the same objects).
pub fn normalized(json: &str) -> serde_json::Value {
    fn is_guid(s: &str) -> bool {
        s.len() == 36
            && s.char_indices().all(|(i, c)| match i {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            })
    }
    fn walk(v: &mut serde_json::Value, seen: &mut std::collections::HashMap<String, usize>) {
        match v {
            serde_json::Value::String(s) if is_guid(s) => {
                let n = seen.len();
                let n = *seen.entry(s.clone()).or_insert(n);
                *s = format!("guid#{n}");
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(|i| walk(i, seen)),
            serde_json::Value::Object(map) => {
                // Keys can be GUIDs too (maps keyed by object).
                let entries: Vec<(String, serde_json::Value)> = std::mem::take(map).into_iter().collect();
                for (k, mut val) in entries {
                    let key = if is_guid(&k) {
                        let n = seen.len();
                        format!("guid#{}", *seen.entry(k).or_insert(n))
                    } else {
                        k
                    };
                    walk(&mut val, seen);
                    map.insert(key, val);
                }
            }
            _ => {}
        }
    }
    let mut v: serde_json::Value = serde_json::from_str(json).unwrap_or(serde_json::Value::Null);
    walk(&mut v, &mut Default::default());
    v
}
