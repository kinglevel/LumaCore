// ██████╗  █████╗  ██████╗██╗  ██╗██╗   ██╗██████╗ ███████╗
// ██╔══██╗██╔══██╗██╔════╝██║ ██╔╝██║   ██║██╔══██╗██╔════╝
// ██████╔╝███████║██║     █████╔╝ ██║   ██║██████╔╝███████╗
// ██╔══██╗██╔══██║██║     ██╔═██╗ ██║   ██║██╔═══╝ ╚════██║
// ██████╔╝██║  ██║╚██████╗██║  ██╗╚██████╔╝██║     ███████║
// ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝
// FILE: backups

//! What keeps a show safe on this station's disk (decisions §30 "12d part 5
//! built", the Backup bullets of "12d hardening"):
//!
//! ```text
//! library/shows/<show>.luma                         the show: changes only when someone saves
//! library/shows/.versions/<show>/<stamp>.luma       every save keeps the file it replaces
//! library/shows/.autosave/<show>_<reason>_<station>_<stamp>.luma
//!                                                   recovery copies: timed, before-load,
//!                                                   signal, lost merges, …
//! library/shows/.trash/<stamp>_<file>.luma          deleted shows / versions / copies, 30 days
//! library/shows/.pins.json                          pinned files (never cleaned up) + a note
//! ```
//!
//! Every list reads only manifests (`show_file::manifest_of`): saved-at,
//! reason, station, user, note, counts, a per-kind GUID → hash index and the
//! last [`HISTORY_ROWS`] history rows — so History, its matrix and the
//! Library need no show load. Stamps are UTC `YYYYMMDD-hhmmss`; a file's name
//! is for people in a file browser, the manifest is the truth.
//!
//! Cleanup ([`plan_cleanup`]) is a station setting: everything from the last
//! hour, one per hour today, one per day for 7 days, one per week up to
//! `RecoveryKeepDays` (30), a size cap `RecoveryCap` (500 MB) over versions
//! and copies; signal / merge / format copies stay ≥ 30 days; pinned never;
//! files whose manifest doesn't say why they were written (older formats,
//! copies made by hand) are never removed. It runs after every write here.
//! Deleting moves to `.trash`.
//! Everything goes through `crate::drive`.
//!
//! Owns:          versions, recovery copies, the trash, pins, manifests (cached), cleanup, the Backup menu's rows.
//! Doesn't own:   who and why a copy is written, the startup ask (lumacore-state recovery.rs).
//! Entry points:  keep_version, versions, recovery, trash, manifest, plan_cleanup, cleanup, history_rows, library_view.
//! Decisions:     §30 "12d part 5 built", §34 25.10.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use lumacore_tree::ShowData;
use serde_json::{Value, json};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// History rows a manifest carries (the newest first).
pub const HISTORY_ROWS: usize = 20;
/// Objects named per kind and change in a history row's detail.
const DETAIL_NAMES: usize = 12;
pub const HOUR: u64 = 3_600_000;
pub const DAY: u64 = 24 * HOUR;
/// Trash and the long-kept copies (signal, merges, formats): 30 days.
const LONG: u64 = 30 * DAY;

pub const VERSIONS: &str = ".versions";
pub const AUTOSAVE: &str = ".autosave";
pub const TRASH: &str = ".trash";
const PINS: &str = ".pins.json";

thread_local! {
    /// Tests keep their show library in a temporary folder.
    static DIR: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Put this thread's show library somewhere else (tests), or back.
pub fn use_dir(dir: Option<PathBuf>) {
    DIR.with(|d| *d.borrow_mut() = dir);
}

/// The shows folder for the rest of a line (`/Drive "usb1"`, decisions §38): the drive's
/// `shows/`, with its versions, recovery copies and trash beside it; back when dropped.
pub struct DirGuard(Option<PathBuf>);

impl DirGuard {
    pub fn enter(dir: PathBuf) -> Self {
        Self(DIR.with(|d| d.replace(Some(dir))))
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let prev = self.0.take();
        DIR.with(|d| *d.borrow_mut() = prev);
    }
}

/// Where the station keeps its show files.
pub const SHOWS_DIR: &str = "library/shows";

/// A show's file in the library.
pub fn show_path(name: &str) -> PathBuf {
    shows_dir().join(format!("{name}.luma"))
}

/// The show files in the library, newest first:
/// `[{name, size, modified (unix seconds)}]`.
pub fn list_show_files() -> Vec<serde_json::Value> {
    let mut shows: Vec<(String, u64, u64)> = crate::drive::read_dir(shows_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let name = path
                .file_name()?
                .to_str()?
                .strip_suffix(".luma")?
                .to_string();
            let meta = e.metadata().ok()?;
            let modified = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            Some((name, meta.len(), modified))
        })
        .collect();
    shows.sort_by_key(|s| std::cmp::Reverse(s.2));
    shows
        .into_iter()
        .map(|(name, size, modified)| serde_json::json!({ "name": name, "size": size, "modified": modified }))
        .collect()
}

/// The show library (`library/shows`).
pub fn shows_dir() -> PathBuf {
    DIR.with(|d| d.borrow().clone())
        .unwrap_or_else(|| PathBuf::from(SHOWS_DIR))
}

/// A thread with its own library (tests) writes even under `cfg(test)`.
pub fn writes_allowed() -> bool {
    !cfg!(test) || DIR.with(|d| d.borrow().is_some())
}

/// Wall-clock ms (the one copy is the base's, decisions §34 25.14).
pub use lumacore_base::clock::now_ms;

/// `20261013-140203` (UTC).
pub fn stamp(ms: u64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms as i64)
        .map(|t| t.format("%Y%m%d-%H%M%S").to_string())
        .unwrap_or_else(|| "00000000-000000".into())
}

/// The time a stamp names (its first 15 characters).
pub fn stamp_ms(s: &str) -> Option<u64> {
    let head = s.get(..15)?;
    chrono::NaiveDateTime::parse_from_str(head, "%Y%m%d-%H%M%S")
        .ok()
        .map(|t| t.and_utc().timestamp_millis() as u64)
}

/// A time for people: `today 14:02`, `yesterday 22:40`, `3 days ago`, a date.
pub fn when_text(ms: u64, now: u64) -> String {
    use chrono::TimeZone;
    if ms == 0 {
        return "—".into();
    }
    let Some(t) = chrono::Local.timestamp_millis_opt(ms as i64).single() else {
        return "—".into();
    };
    let Some(today) = chrono::Local.timestamp_millis_opt(now as i64).single() else {
        return t.format("%Y-%m-%d %H:%M").to_string();
    };
    let days = (today.date_naive() - t.date_naive()).num_days();
    match days {
        0 => format!("today {}", t.format("%H:%M")),
        1 => format!("yesterday {}", t.format("%H:%M")),
        2..=13 => format!("{days} days ago {}", t.format("%H:%M")),
        _ => t.format("%Y-%m-%d %H:%M").to_string(),
    }
}

/// `1.1 MB`, `48 kB`.
pub fn size_text(bytes: u64) -> String {
    match bytes {
        b if b >= 1_000_000_000 => format!("{:.1} GB", b as f64 / 1e9),
        b if b >= 100_000_000 => format!("{:.0} MB", b as f64 / 1e6),
        b if b >= 1_000_000 => format!("{:.1} MB", b as f64 / 1e6),
        b => format!("{} kB", b.div_ceil(1000)),
    }
}

/// A file-name part from a station or show name (`Mac.local` → `Mac-local`).
pub fn sane(s: &str) -> String {
    let out: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '-' })
        .take(60)
        .collect();
    // At most 100 bytes, so a recovery or trash name built from two of
    // these still fits in 255 bytes (a long name, R1-16).
    let out = fit(out.trim(), 100).trim().to_string();
    if out.is_empty() { "-".into() } else { out }
}

/// `s` shortened to at most `bytes` bytes, on a character boundary.
fn fit(s: &str, bytes: usize) -> &str {
    if s.len() <= bytes {
        return s;
    }
    let mut end = bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

// *************************************************************************************************
// Reasons
// *************************************************************************************************
/// Why a file was written, as people read it (decisions §30 "Show history").
pub fn reason_word(code: &str) -> &'static str {
    match code {
        "save" => "Save",
        "quick" => "Quick",
        "saveas" => "Save As",
        "new" => "New",
        "timed" => "Recovery",
        "before-load" => "Before load",
        "shutdown" => "Shut down",
        "signal" => "Signal",
        "format" => "Format",
        "restored" => "Restored",
        "use-master" | "use-mine" | "before-merge" | "split-base" | "invite" | "merge" => "Lost merge",
        "" => "—",
        _ => "Other",
    }
}

/// The tone of a reason's dot (merges alert, signal / before-load warn).
fn reason_tone(code: &str) -> Option<&'static str> {
    match reason_word(code) {
        "Lost merge" => Some("alert"),
        "Signal" | "Before load" | "Shut down" | "Format" => Some("warn"),
        "Recovery" => Some("ok"),
        _ => None,
    }
}

/// Copies someone lost something to (a signal, a merge, a format): kept at
/// least 30 days whatever the tiers say.
fn long_kept(code: &str) -> bool {
    matches!(reason_word(code), "Lost merge" | "Signal" | "Format")
}

// *************************************************************************************************
// The GUID → hash index (the History matrix)
// *************************************************************************************************
/// The object kinds the History matrix shows, in its column order.
pub const KINDS: [&str; 10] = [
    "Fixtures", "Patch", "Groups", "Presets", "Sequences", "Cues", "Macros", "Lines", "Users", "Profiles",
];

/// Kinds the index also keeps for the Load show dialog's pick lists (not
/// matrix columns).
const PICK_KINDS: [&str; 3] = ["FixtureTypes", "Networks", "Stations"];

/// FNV-1a 64 of a value's canonical JSON (sorted keys).
fn hash(v: &Value) -> String {
    let bytes = serde_json::to_vec(v).unwrap_or_default();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn named(word: &str, no: impl std::fmt::Display, name: Option<&str>) -> String {
    match name.filter(|n| !n.is_empty()) {
        Some(n) => format!("{word} {no} \"{n}\""),
        None => format!("{word} {no}"),
    }
}

/// Per kind: GUID → `[hash, label]` (decisions §30 "History matrix": the
/// manifest keeps it so a diff needs no show load).
pub fn index(show: &ShowData) -> Value {
    let mut out: BTreeMap<&str, BTreeMap<String, Value>> = KINDS.iter().chain(PICK_KINDS.iter()).map(|k| (*k, BTreeMap::new())).collect();
    let mut put = |kind: &str, guid: String, v: Value, label: String| {
        if let Some(m) = out.get_mut(kind) {
            m.insert(guid, json!([hash(&v), label]));
        }
    };
    for f in show.devices.dmx.patch.fixtures.values() {
        // The manifest names a fixture as a line does (decisions §40 7): its number, else its
        // second id or name.
        let handle = f.handle();
        let no = handle.trim_start_matches("Fixture ");
        let mut v = serde_json::to_value(f).unwrap_or_default();
        let address = json!({
            "type": v["mvr"]["fixture_type"].take(), "revision": v["mvr"]["revision"].take(),
            "mode": v["mvr"]["mode"].take(), "breaks": v["mvr"]["breaks"].take(),
        });
        let label = named("Fixture", no, f.meta.identity.name.as_deref());
        put("Patch", f.meta.identity.guid.to_string(), address, handle.clone());
        put("Fixtures", f.meta.identity.guid.to_string(), v, label);
    }
    for (dp_no, dp) in &show.datapools {
        let pre = if *dp_no == 1 { String::new() } else { format!("DataPool {dp_no} ") };
        for (no, g) in &dp.groups {
            put("Groups", g.meta.identity.guid.to_string(), serde_json::to_value(g).unwrap_or_default(), format!("{pre}{}", named("Group", no, g.meta.identity.name.as_deref())));
        }
        for (no, p) in &dp.presets {
            put("Presets", p.meta.identity.guid.to_string(), serde_json::to_value(p).unwrap_or_default(), format!("{pre}{}", named("Preset", no, p.meta.identity.name.as_deref())));
        }
        for (no, m) in &dp.macros {
            put("Macros", m.meta.identity.guid.to_string(), serde_json::to_value(m).unwrap_or_default(), format!("{pre}{}", named("Macro", no, m.meta.identity.name.as_deref())));
        }
    }
    for (dp_no, no, s) in show.sequences() {
        let pre = if dp_no == 1 { String::new() } else { format!("DataPool {dp_no} ") };
        put("Sequences", s.meta.identity.guid.to_string(), serde_json::to_value(s).unwrap_or_default(), format!("{pre}{}", named("Sequence", no, s.meta.identity.name.as_deref())));
        for (key, c) in &s.cues {
            let cue = lumacore_tree::cue_label(*key);
            put("Cues", c.meta.identity.guid.to_string(), serde_json::to_value(c).unwrap_or_default(), format!("{pre}Sequence {no} Cue {cue}"));
        }
    }
    for (protocol, no, l) in show.protocols.all_lines() {
        put("Lines", l.meta.identity.guid.to_string(), serde_json::to_value(l).unwrap_or_default(), named(protocol, no, l.meta.identity.name.as_deref()));
    }
    for u in show.users.values() {
        let mut v = serde_json::to_value(u).unwrap_or_default();
        // A user's work changes every minute; the matrix shows who they are.
        if let Some(o) = v.as_object_mut() {
            for k in ["programmer", "selected", "usage"] {
                o.remove(k);
            }
        }
        put("Users", u.meta.identity.guid.to_string(), v, format!("User \"{}\"", u.login));
    }
    for (no, t) in &show.devices.dmx.patch.fixture_types {
        put("FixtureTypes", t.meta.identity.guid.to_string(), json!(t.meta.identity.guid), named("FixtureType", no, t.meta.identity.name.as_deref()));
    }
    for (no, n) in &show.protocols.networks {
        put("Networks", n.meta.identity.guid.to_string(), serde_json::to_value(n).unwrap_or_default(), named("Network", no, n.meta.identity.name.as_deref()));
    }
    for (no, r) in &show.stations {
        put("Stations", r.meta.identity.guid.to_string(), serde_json::to_value(r).unwrap_or_default(), named("Station", no, r.meta.name()));
    }
    for (no, p) in &show.user_profiles {
        put("Profiles", p.meta.identity.guid.to_string(), serde_json::to_value(p).unwrap_or_default(), named("UserProfile", no, p.meta.identity.name.as_deref()));
    }
    serde_json::to_value(out).unwrap_or_default()
}

/// Objects per kind.
pub fn counts(index: &Value) -> Value {
    let mut out = serde_json::Map::new();
    for k in KINDS.iter().chain(PICK_KINDS.iter()) {
        out.insert((*k).into(), json!(index[*k].as_object().map_or(0, |m| m.len())));
    }
    Value::Object(out)
}

/// What changed from `old` to `new` per kind: `{kind: [added, changed,
/// removed]}` and the objects' names `{kind: {"a": […], "c": […], "r": […]}}`.
pub fn diff(old: &Value, new: &Value) -> (Value, Value) {
    let mut changes = serde_json::Map::new();
    let mut detail = serde_json::Map::new();
    let empty = serde_json::Map::new();
    for k in KINDS {
        let o = old[k].as_object().unwrap_or(&empty);
        let n = new[k].as_object().unwrap_or(&empty);
        let label = |v: &Value| v[1].as_str().unwrap_or("").to_string();
        let mut a = Vec::new();
        let mut c = Vec::new();
        let mut r = Vec::new();
        for (g, v) in n {
            match o.get(g) {
                None => a.push(label(v)),
                Some(was) if was[0] != v[0] => c.push(label(v)),
                _ => {}
            }
        }
        for (g, v) in o {
            if !n.contains_key(g) {
                r.push(label(v));
            }
        }
        if a.is_empty() && c.is_empty() && r.is_empty() {
            continue;
        }
        changes.insert(k.into(), json!([a.len(), c.len(), r.len()]));
        let cap = |mut l: Vec<String>| {
            l.sort();
            let more = l.len().saturating_sub(DETAIL_NAMES);
            l.truncate(DETAIL_NAMES);
            if more > 0 {
                l.push(format!("… {more} more"));
            }
            l
        };
        detail.insert(k.into(), json!({"a": cap(a), "c": cap(c), "r": cap(r)}));
    }
    (Value::Object(changes), Value::Object(detail))
}

/// `+12 fixtures · −1 sequence` from a row's changes.
pub fn changes_text(changes: &Value) -> String {
    let mut parts = Vec::new();
    for k in KINDS {
        let Some(m) = changes[k].as_array() else { continue };
        let word = k.to_lowercase();
        let one = word.trim_end_matches('s');
        let n = |i: usize| m.get(i).and_then(Value::as_u64).unwrap_or(0);
        for (i, sign) in [(0, "+"), (1, "~"), (2, "−")] {
            let c = n(i);
            if c > 0 {
                parts.push(format!("{sign}{c} {}", if c == 1 && k != "Patch" { one } else { &word }));
            }
        }
    }
    if parts.is_empty() { "no object changes".into() } else { parts.join(" · ") }
}

/// A matrix cell: `+12 ~3 −1`.
fn cell_text(m: &Value) -> String {
    let n = |i: usize| m.get(i).and_then(Value::as_u64).unwrap_or(0);
    [(0, "+"), (1, "~"), (2, "−")]
        .iter()
        .filter(|(i, _)| n(*i) > 0)
        .map(|(i, s)| format!("{s}{}", n(*i)))
        .collect::<Vec<_>>()
        .join(" ")
}

// *************************************************************************************************
// What a save carries
// *************************************************************************************************
/// The loaded show's file facts (station-local; from its manifest on load,
/// renewed by every save): the history rows and the index the next save
/// compares with.
#[derive(Debug, Clone, Default)]
pub struct Saved {
    pub history: Vec<Value>,
    pub index: Value,
    /// When the loaded file was saved (0 = never: a new or untitled show).
    pub saved_at: u64,
    pub by: String,
    pub station: String,
    /// The last recovery copy (the timer counts from it or the last save).
    pub last_copy: Option<std::time::Instant>,
    /// Opened from a version or a recovery copy (its id): the next save's
    /// reason is Restored.
    pub restored: Option<String>,
}

impl Saved {
    /// From a file's manifest.
    pub fn from_manifest(m: &Value) -> Self {
        Self {
            history: m["history"].as_array().cloned().unwrap_or_default(),
            index: m["index"].clone(),
            saved_at: m["saved_at"].as_u64().unwrap_or(0),
            by: m["user"].as_str().unwrap_or("").to_string(),
            station: m["station"].as_str().unwrap_or("").to_string(),
            last_copy: None,
            restored: None,
        }
    }
}

// *************************************************************************************************
// Manifests (cached)
// *************************************************************************************************
type Cache = HashMap<PathBuf, (u64, u128, Arc<Value>)>;
static MANIFESTS: Mutex<Option<Cache>> = Mutex::new(None);

fn mtime_ns(meta: &std::fs::Metadata) -> u128 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos())
}

/// A file's manifest without its entry list (read once per change of the
/// file), plus its size and modified time.
pub fn manifest(path: &Path) -> Option<(Arc<Value>, u64, u64)> {
    let meta = crate::drive::metadata(path).ok()?;
    let (size, mtime) = (meta.len(), mtime_ns(&meta));
    let ms = (mtime / 1_000_000) as u64;
    let mut guard = MANIFESTS.lock().unwrap_or_else(|e| e.into_inner());
    let cache = guard.get_or_insert_with(HashMap::new);
    if let Some((s, m, v)) = cache.get(path)
        && *s == size
        && *m == mtime
    {
        return Some((v.clone(), size, ms));
    }
    let mut v = crate::show_file::manifest_of(path).unwrap_or(Value::Null);
    if let Some(o) = v.as_object_mut() {
        o.remove("entries");
    }
    let v = Arc::new(v);
    if cache.len() > 512 {
        cache.clear();
    }
    cache.insert(path.to_path_buf(), (size, mtime, v.clone()));
    Some((v, size, ms))
}

// *************************************************************************************************
// The stores
// *************************************************************************************************
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    Version,
    Recovery,
    Trash,
}

/// One file of the stores.
#[derive(Debug, Clone)]
pub struct Copy {
    pub store: Store,
    /// A version's stamp; a recovery copy's or trash file's name without `.luma`.
    pub id: String,
    pub show: String,
    pub path: PathBuf,
    /// The path inside the library (the pins' key).
    pub rel: String,
    pub at: u64,
    pub size: u64,
    pub reason: String,
    pub station: String,
    pub user: String,
    pub note: String,
    pub format: u64,
    pub pinned: Option<String>,
}

fn luma_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = crate::drive::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "luma"))
        .collect();
    out.sort();
    out
}

pub fn rel(path: &Path) -> String {
    path.strip_prefix(shows_dir()).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn copy_of(store: Store, path: PathBuf, show_hint: &str, pins: &BTreeMap<String, String>) -> Option<Copy> {
    let (m, size, mtime) = manifest(&path)?;
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let id = stem.clone();
    let at = m["saved_at"].as_u64().or_else(|| stamp_ms(&stem)).unwrap_or(mtime);
    let r = rel(&path);
    Some(Copy {
        store,
        id,
        show: m["show_name"].as_str().filter(|s| !s.is_empty()).unwrap_or(show_hint).to_string(),
        at,
        size,
        reason: m["reason"].as_str().unwrap_or("").to_string(),
        station: m["station"].as_str().unwrap_or("").to_string(),
        user: m["user"].as_str().unwrap_or("").to_string(),
        note: m["note"].as_str().unwrap_or("").to_string(),
        format: m["format"].as_u64().unwrap_or(0),
        pinned: pins.get(&r).cloned(),
        rel: r,
        path,
    })
}

/// The versions of `show`, newest first.
pub fn versions(show: &str) -> Vec<Copy> {
    let pins = pins();
    let mut out: Vec<Copy> = luma_files(&shows_dir().join(VERSIONS).join(show))
        .into_iter()
        .filter_map(|p| copy_of(Store::Version, p, show, &pins))
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.at));
    out
}

/// Every show that has versions.
fn version_shows() -> Vec<String> {
    crate::drive::read_dir(shows_dir().join(VERSIONS))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

/// Every recovery copy, newest first.
pub fn recovery() -> Vec<Copy> {
    let pins = pins();
    let mut out: Vec<Copy> = luma_files(&shows_dir().join(AUTOSAVE))
        .into_iter()
        .filter_map(|p| {
            let hint = p.file_stem()?.to_string_lossy().split('_').next()?.to_string();
            copy_of(Store::Recovery, p, &hint, &pins)
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.at));
    out
}

/// What is in the trash, newest first (its `at` = when it was deleted).
pub fn trash() -> Vec<Copy> {
    let pins = BTreeMap::new();
    let mut out: Vec<Copy> = luma_files(&shows_dir().join(TRASH))
        .into_iter()
        .filter_map(|p| {
            let stem = p.file_stem()?.to_string_lossy().into_owned();
            let mut c = copy_of(Store::Trash, p, &stem, &pins)?;
            c.at = stamp_ms(&stem).unwrap_or(c.at);
            Some(c)
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.at));
    out
}

/// A version by its stamp (`20261013-140203`) or a recovery copy by its
/// name; None = no such file.
pub fn find(store: Store, show: &str, id: &str) -> Option<Copy> {
    let id = id.trim_matches('"');
    match store {
        Store::Version => versions(show).into_iter().find(|c| c.id == id),
        Store::Recovery => recovery().into_iter().find(|c| c.id == id),
        Store::Trash => trash().into_iter().find(|c| c.id == id),
    }
}

/// Before `path` (the show file of `show`) is replaced: keep it as a
/// version. Returns where it went.
pub fn keep_version(show: &str, path: &Path) -> Result<Option<PathBuf>, String> {
    // Only a missing file is "nothing to keep"; a file that can't be read
    // is an error, never a save over it without its version (R1-16).
    match crate::drive::metadata(path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Cannot read {} to keep it as a version: {e}", path.display())),
    }
    let at = manifest(path)
        .and_then(|(m, _, mtime)| m["saved_at"].as_u64().or(Some(mtime)))
        .unwrap_or_else(now_ms);
    let dir = shows_dir().join(VERSIONS).join(show);
    crate::drive::create_dir_all(&dir).map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
    // Never over a version from the same second: `-2`, `-3` … (they sort
    // after the first).
    let base = stamp(at);
    let mut to = dir.join(format!("{base}.luma"));
    let mut n = 2;
    while crate::drive::metadata(&to).is_ok() {
        to = dir.join(format!("{base}-{n}.luma"));
        n += 1;
    }
    // Copied by the OS (never read into memory here) through a temporary
    // name, so the named show stays whole whatever happens to the save.
    let tmp = to.with_extension("luma.tmp");
    crate::drive::copy(path, &tmp)
        .and_then(|_| crate::drive::rename(&tmp, &to))
        .map_err(|e| {
            let _ = crate::drive::remove_file(&tmp);
            format!("Cannot keep the previous file as a version: {e}")
        })?;
    Ok(Some(to))
}

/// Move a file to the trash (30 days). Returns its new path.
pub fn to_trash(path: &Path) -> Result<PathBuf, String> {
    let dir = shows_dir().join(TRASH);
    crate::drive::create_dir_all(&dir).map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
    let file = path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    // A version's file is only its stamp: name the show too.
    let file = match path.parent().and_then(|p| p.parent()) {
        Some(pp) if pp.ends_with(VERSIONS) => {
            let show = path.parent().and_then(|p| p.file_name()).map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
            format!("{}_version_{file}", fit(&show, 150))
        }
        _ => file,
    };
    let to = dir.join(format!("{}_{file}", stamp(now_ms())));
    crate::drive::rename(path, &to).map_err(|e| format!("Cannot move {} to the trash: {e}", path.display()))?;
    set_pin(&rel(path), None);
    Ok(to)
}

// *************************************************************************************************
// Pins
// *************************************************************************************************
/// Pinned files (library path → note): never cleaned up.
pub fn pins() -> BTreeMap<String, String> {
    crate::drive::read_to_string(shows_dir().join(PINS))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Pin (`Some(note)`) or unpin a file.
pub fn set_pin(rel: &str, note: Option<String>) {
    let mut p = pins();
    let changed = match note {
        Some(n) => p.insert(rel.to_string(), n.clone()) != Some(n),
        None => p.remove(rel).is_some(),
    };
    if changed && let Ok(json) = serde_json::to_string_pretty(&p) {
        let _ = crate::drive::write(shows_dir().join(PINS), json);
    }
}

// *************************************************************************************************
// Cleanup
// *************************************************************************************************
/// The station's cleanup rules (`Set Station RecoveryKeepDays|RecoveryCap`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
    pub keep_days: u32,
    pub cap_mb: u32,
}

impl Default for Rules {
    fn default() -> Self {
        Self { keep_days: 30, cap_mb: 500 }
    }
}

/// One file as the cleanup sees it.
#[derive(Debug, Clone)]
pub struct Item {
    pub at: u64,
    pub size: u64,
    pub reason: String,
    pub pinned: bool,
    /// Files are thinned per group (a show's versions, a show's copies).
    pub group: String,
    /// Written by this scheme (its manifest says why). Other files in the
    /// folders — older formats, copies made by hand — are never removed.
    pub managed: bool,
}

/// Which items the rules remove (indices), and why each one stays (the
/// Kept column: `1 h`, `today`, `7 d`, `30 d`, `pinned`).
pub fn plan_cleanup(items: &[Item], rules: Rules, now: u64) -> (Vec<usize>, Vec<&'static str>) {
    let keep_for = u64::from(rules.keep_days.max(1)) * DAY;
    let mut kept: Vec<&'static str> = vec![""; items.len()];
    let mut gone = vec![false; items.len()];
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|&a, &b| items[b].at.cmp(&items[a].at));
    let mut buckets: std::collections::HashSet<(String, &'static str, u64)> = Default::default();
    for &i in &order {
        let it = &items[i];
        let age = now.saturating_sub(it.at);
        if it.pinned {
            kept[i] = "pinned";
            continue;
        }
        if !it.managed {
            kept[i] = "kept";
            continue;
        }
        if long_kept(&it.reason) && age < LONG {
            kept[i] = "30 d";
            continue;
        }
        if age < HOUR {
            kept[i] = "1 h";
            continue;
        }
        if age >= keep_for {
            gone[i] = true;
            continue;
        }
        let (tier, size, word) = if age < DAY {
            ("h", HOUR, "today")
        } else if age < 7 * DAY {
            ("d", DAY, "7 d")
        } else {
            ("w", 7 * DAY, "30 d")
        };
        if buckets.insert((it.group.clone(), tier, it.at / size)) {
            kept[i] = word;
        } else {
            gone[i] = true;
        }
    }
    // The size cap: the oldest unpinned go first, the long-kept last.
    let cap = u64::from(rules.cap_mb) * 1_000_000;
    let mut total: u64 = (0..items.len()).filter(|&i| !gone[i]).map(|i| items[i].size).sum();
    for long in [false, true] {
        for &i in order.iter().rev() {
            if total <= cap {
                break;
            }
            let it = &items[i];
            if gone[i] || it.pinned || !it.managed || long_kept(&it.reason) != long {
                continue;
            }
            gone[i] = true;
            total = total.saturating_sub(it.size);
        }
    }
    let out: Vec<usize> = (0..items.len()).filter(|&i| gone[i]).collect();
    for &i in &out {
        kept[i] = "goes";
    }
    (out, kept)
}

fn items(copies: &[Copy]) -> Vec<Item> {
    copies
        .iter()
        .map(|c| Item {
            at: c.at,
            size: c.size,
            reason: c.reason.clone(),
            pinned: c.pinned.is_some(),
            group: format!("{:?}:{}", c.store, c.show),
            managed: !c.reason.is_empty(),
        })
        .collect()
}

/// Every version and recovery copy.
fn stored() -> Vec<Copy> {
    let mut all: Vec<Copy> = version_shows().iter().flat_map(|s| versions(s)).collect();
    all.extend(recovery());
    all
}

/// What `cleanup` would remove now.
pub fn would_clean(rules: &Rules) -> Vec<Copy> {
    let all = stored();
    let (gone, _) = plan_cleanup(&items(&all), *rules, now_ms());
    gone.into_iter().map(|i| all[i].clone()).collect()
}

/// Apply the rules: remove what they drop, empty the trash of what is older
/// than 30 days. Returns the library paths removed.
pub fn cleanup(rules: &Rules) -> Vec<String> {
    if !writes_allowed() {
        return Vec::new();
    }
    let mut removed = Vec::new();
    for c in would_clean(rules) {
        if crate::drive::remove_file(&c.path).is_ok() {
            removed.push(c.rel.clone());
        }
    }
    let now = now_ms();
    for c in trash() {
        if now.saturating_sub(c.at) >= LONG && crate::drive::remove_file(&c.path).is_ok() {
            removed.push(c.rel);
        }
    }
    if !removed.is_empty() {
        lumacore_sysmon::sysmon_note("Show", format!("Cleanup removed {} old copies", removed.len()));
    }
    removed
}

/// Bytes the versions and copies use.
pub fn usage() -> u64 {
    stored().iter().map(|c| c.size).sum()
}

// *************************************************************************************************
// Startup recovery
// *************************************************************************************************
pub fn marker() -> String {
    format!("{}/running", crate::station_config::dir())
}

/// The station stopped cleanly: no ask next time.
pub fn stopped() {
    let _ = crate::drive::remove_file(marker());
}

// *************************************************************************************************
// The Backup menu's feed
// *************************************************************************************************
/// Free bytes on the library's volume (cached 10 s): `(bytes, tone)`.
fn disk() -> Option<(u64, &'static str)> {
    static FREE: Mutex<Option<(std::time::Instant, Option<u64>)>> = Mutex::new(None);
    let mut g = FREE.lock().unwrap_or_else(|e| e.into_inner());
    let fresh = g.as_ref().is_some_and(|(t, _)| t.elapsed() < std::time::Duration::from_secs(10));
    if !fresh {
        *g = Some((std::time::Instant::now(), crate::drive::free_bytes(&shows_dir())));
    }
    let free = g.as_ref()?.1?;
    let tone = match free {
        f if f < 5_000_000_000 => "alert",
        f if f < 15_000_000_000 => "warn",
        _ => "ok",
    };
    Some((free, tone))
}

fn kept_word(c: &Copy, why: &str) -> (String, &'static str) {
    if c.pinned.is_some() {
        ("★ pinned".into(), "pin")
    } else if why == "goes" {
        ("goes at cleanup".into(), "warn")
    } else {
        (why.to_string(), "")
    }
}

/// History rows of `show`: its file's manifest history, plus older rows
/// that only versions still carry; Kept from what is on disk.
pub fn history_rows(show: &str, current_name: Option<&str>) -> Vec<Value> {
    let now = now_ms();
    let file = show_path(show);
    let head = manifest(&file).map(|(m, _, _)| m);
    let vs = versions(show);
    let mut rows: BTreeMap<u64, Value> = BTreeMap::new();
    let mut take = |m: &Value| {
        for r in m["history"].as_array().into_iter().flatten() {
            if let Some(at) = r["saved_at"].as_u64() {
                rows.entry(at).or_insert_with(|| r.clone());
            }
        }
    };
    if let Some(m) = &head {
        take(m);
    }
    for v in &vs {
        if let Some((m, _, _)) = manifest(&v.path) {
            take(&m);
        }
    }
    let file_at = head.as_ref().and_then(|m| m["saved_at"].as_u64());
    let rules_plan: HashMap<u64, &Copy> = vs.iter().map(|c| (c.at, c)).collect();
    let mut out: Vec<Value> = rows
        .into_values()
        .rev()
        .map(|r| {
            let at = r["saved_at"].as_u64().unwrap_or(0);
            let version = rules_plan.get(&at);
            let saved_as = r["name"].as_str().unwrap_or(show);
            let (kept, tone) = if Some(at) == file_at {
                ("current".to_string(), "ok")
            } else if version.is_none() && !saved_as.eq_ignore_ascii_case(show) {
                // A save of the show it came from (Save As keeps the history).
                (format!("in \"{saved_as}\""), "off")
            } else if let Some(v) = version {
                if v.pinned.is_some() { ("★ pinned".to_string(), "pin") } else { ("version".to_string(), "") }
            } else {
                ("pruned".to_string(), "off")
            };
            let reason = r["reason"].as_str().unwrap_or("");
            let mut row = json!({
                "id": r["id"], "saved_at": at, "saved": when_text(at, now),
                "by": r["by"], "station": r["station"], "reason": reason_word(reason),
                "reason_tone": reason_tone(reason),
                "note": version.and_then(|v| v.pinned.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| r["note"].as_str().unwrap_or("").to_string()),
                "changes": changes_text(&r["changes"]),
                "kept": kept, "kept_tone": tone,
                "version": version.map(|v| v.id.clone()),
                // What lines name: the version's file, else the save's stamp.
                "vid": version.map_or_else(|| stamp(at), |v| v.id.clone()),
                "pinned": version.is_some_and(|v| v.pinned.is_some()),
                "show": show,
                "loaded": current_name == Some(show),
                "current": Some(at) == file_at,
            });
            for k in KINDS {
                let m = &r["changes"][k];
                row[format!("k_{k}")] = json!(if m.is_array() { cell_text(m) } else { String::new() });
                row[format!("t_{k}")] = json!(match m.as_array() {
                    Some(a) if a.get(2).and_then(Value::as_u64).unwrap_or(0) > 0 => Some("alert"),
                    Some(a) if a.get(1).and_then(Value::as_u64).unwrap_or(0) > 0 => Some("warn"),
                    Some(_) => Some("ok"),
                    None => None,
                });
                let d = &r["detail"][k];
                if d.is_object() {
                    let mut parts = Vec::new();
                    for (key, sign) in [("a", "+"), ("c", "~"), ("r", "−")] {
                        for n in d[key].as_array().into_iter().flatten() {
                            parts.push(format!("{sign} {}", n.as_str().unwrap_or("")));
                        }
                    }
                    row[format!("d_{k}")] = json!(parts.join(" · "));
                }
            }
            row
        })
        .collect();
    out.truncate(200);
    out
}

/// The rules from [`station_facts`].
pub fn rules_of(facts: &Value) -> Rules {
    let d = Rules::default();
    Rules {
        keep_days: facts["keep_days"].as_u64().map_or(d.keep_days, |v| v as u32),
        cap_mb: facts["cap_mb"].as_u64().map_or(d.cap_mb, |v| v as u32),
    }
}

/// The `shows` feed: the loaded show and its facts, the library, the
/// history of `history_of` (default: the loaded show), the recovery copies,
/// what versions and copies use, the free disk.
pub fn library_view(current: &lumacore_wire::ShowStatus, history_of: Option<&str>, facts: &Value) -> Value {
    let rules = rules_of(facts);
    let now = now_ms();
    let shows: Vec<Value> = list_show_files()
        .into_iter()
        .map(|s| {
            let name = s["name"].as_str().unwrap_or("").to_string();
            let path = show_path(&name);
            let m = manifest(&path).map(|(m, _, _)| m);
            let m = m.as_deref().unwrap_or(&Value::Null);
            let fmt = m["format"].as_u64().unwrap_or(0);
            let ours = u64::from(lumacore_tree::FORMAT_VERSION);
            let (fmt_text, tone) = match fmt {
                0 => ("not a show".to_string(), "alert"),
                f if f == ours => (format!("{f} · loads"), "ok"),
                // An older readable show upgrades on load (decisions 0289).
                f if f < ours && crate::show_file::upgrade::readable(f).is_ok() => (format!("{f} · older, upgrades on load"), "warn"),
                f if f < ours => (format!("{f} · too old, refused"), "alert"),
                f => (format!("{f} · newer, refused"), "alert"),
            };
            let at = m["saved_at"].as_u64().unwrap_or(s["modified"].as_u64().unwrap_or(0) * 1000);
            let size = s["size"].as_u64().unwrap_or(0);
            json!({
                "name": name, "size": size, "modified": s["modified"],
                "saved": when_text(at, now), "saved_at": at, "size_text": size_text(size),
                "format": fmt, "format_text": fmt_text, "format_tone": tone, "loads": fmt == ours,
                "by": m["user"], "station": m["station"],
                "versions": versions(&name).len(),
                "loaded": current.name.as_deref() == Some(name.as_str()),
                "counts": m["counts"],
            })
        })
        .collect();
    let history_show = history_of.filter(|s| !s.is_empty()).map(str::to_string).or_else(|| current.name.clone());
    let history = history_show.as_deref().map(|s| history_rows(s, current.name.as_deref())).unwrap_or_default();
    // Recovery copies with what the rules keep them for.
    let all = stored();
    let (_, why) = plan_cleanup(&items(&all), rules, now);
    let kept_of: HashMap<String, &'static str> = all.iter().zip(why).map(|(c, w)| (c.rel.clone(), w)).collect();
    let recovery: Vec<Value> = recovery()
        .into_iter()
        .map(|c| {
            let (kept, tone) = kept_word(&c, kept_of.get(&c.rel).copied().unwrap_or(""));
            json!({
                "id": c.id, "show": c.show, "time": when_text(c.at, now), "at": c.at,
                "reason": reason_word(&c.reason), "reason_tone": reason_tone(&c.reason),
                "by": format!("{} · {}", if c.station.is_empty() { "—" } else { &c.station }, if c.user.is_empty() { "—" } else { &c.user }),
                "size_text": size_text(c.size), "kept": kept, "kept_tone": tone,
                "note": c.pinned.clone().unwrap_or_default(), "pinned": c.pinned.is_some(),
                "format": c.format,
            })
        })
        .collect();
    let used: u64 = all.iter().map(|c| c.size).sum();
    let cap = u64::from(rules.cap_mb) * 1_000_000;
    let disk = disk();
    let saved_at = facts["saved_at"].as_u64().unwrap_or(0);
    json!({
        "current": current,
        "facts": {
            "saved_at": saved_at,
            "saved": if saved_at == 0 { "never saved".to_string() } else { when_text(saved_at, now) },
            "by": facts["by"], "station": facts["station"], "restored": facts["restored"],
            "counts": facts["counts"],
        },
        "kinds": KINDS,
        "shows": shows,
        "history_of": history_show,
        "history": history,
        "recovery": recovery,
        "trash": trash().len(),
        "usage": { "bytes": used, "cap": cap, "text": format!("{} of {}", size_text(used), size_text(cap)),
                   "tone": if used * 10 > cap * 9 { "warn" } else { "ok" } },
        "disk": disk.map(|(free, tone)| json!({ "free": free, "text": format!("{} free", size_text(free)), "tone": tone })),
        "rules": { "keep_days": rules.keep_days, "cap_mb": rules.cap_mb },
    })
}
