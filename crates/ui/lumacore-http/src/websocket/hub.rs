//! Shared builds of the counter-driven State feeds (decisions §30 "12d
//! hardening": "Shared builds").
//!
//! One build per (feed, user, params), only while someone subscribes it: the
//! first subscriber starts a producer task, the last one leaving stops it.
//! The producer builds when the station's own counters move
//! ([`crate::Revisions`]: the published show, the station view, claims, the
//! show library, the screen events) — never on a polling timer — serializes
//! the result once and hands it to every subscriber through a `watch`
//! (a slow subscriber only ever sees the newest). Each build carries a `rev`
//! (a fingerprint of its JSON, so the same state has the same rev, across
//! reconnects and restarts); a keyed list also carries the `set` / `del`
//! diff from the build before (HA's `subscribe_entities`, Centrifugo's
//! cache recovery).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use serde_json::{Map, Value, json};
use tokio::sync::watch;

use crate::{ConsoleBridge, HttpState, Revisions};

/// A keyed list in a feed's data: `list` is an array of objects, `key` the
/// field naming each one (a number or a string).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keyed {
    pub list: &'static str,
    pub key: &'static str,
}

/// What one build says.
#[derive(Debug)]
pub struct Built {
    /// Fingerprint of `text` (53 bits: exact in JavaScript).
    pub rev: u64,
    /// The data, serialized once for every subscriber.
    pub text: Arc<str>,
    /// `(base rev, diff JSON)`: the change from the build before, when it is
    /// a keyed list and the diff is less than half the data.
    pub diff: Option<(u64, Arc<str>)>,
}

/// One feed's build parameters (the key of a shared build).
#[derive(Debug, Clone, PartialEq)]
pub enum BuildSpec {
    Station,
    Network,
    LumaSessions,
    Shows { history: String, drive: String },
    /// A drive's library folder by kind (decisions §38 5).
    Files { drive: String, folder: String, kind: String },
    Pools { user: String, pool: String, datapool: Value, profile: Value, filter: Value },
    PoolEditor { user: String, pool: String, datapool: Value, no: u32 },
    Windows { user: String, screen: u32 },
    Encoder { user: String, screen: u32 },
    ViewBar { user: String, screen: u32 },
    Oops { user: String },
    Sequence { user: String, sequence: Value },
    Tags { user: String, tag: Value, deep: bool },
    Patch { user: String },
    Attributes { user: String, profile: Value },
    Detail { kind: String, no: u32 },
    /// A Tree window's rows (decisions 0292): its place, open branches, filter and Follow.
    Tree { user: String, path: String, open: String, search: String, follow: bool },
    /// A Tree window's inspector (decisions 0292).
    Inspector { user: String, path: String },
    /// The paint picker and the theme editor's Paint tab (decisions 0294).
    Paint { user: String, profile: Value },
    /// The Rules window (decisions 0294).
    Rules { user: String, profile: Value, which: String },
}

impl BuildSpec {
    fn key(&self) -> String {
        match self {
            Self::Station => "station".into(),
            Self::Network => "network".into(),
            Self::LumaSessions => "lumasessions".into(),
            Self::Shows { history, drive } => format!("shows|{history}|{}", drive.to_lowercase()),
            Self::Files { drive, folder, kind } => format!("files|{}|{folder}|{kind}", drive.to_lowercase()),
            Self::Pools { user, pool, datapool, profile, filter } => {
                format!("pools|{}|{}|{datapool}|{profile}|{filter}", user.to_lowercase(), pool.to_lowercase())
            }
            Self::PoolEditor { user, pool, datapool, no } => {
                format!("pool_editor|{}|{}|{datapool}|{no}", user.to_lowercase(), pool.to_lowercase())
            }
            Self::Windows { user, screen } => format!("windows|{}|{screen}", user.to_lowercase()),
            Self::Encoder { user, screen } => format!("encoder|{}|{screen}", user.to_lowercase()),
            Self::ViewBar { user, screen } => format!("viewbar|{}|{screen}", user.to_lowercase()),
            Self::Oops { user } => format!("oops|{}", user.to_lowercase()),
            Self::Sequence { user, sequence } => format!("sequence|{}|{sequence}", user.to_lowercase()),
            Self::Tags { user, tag, deep } => format!("tags|{user}|{tag}|{deep}"),
            Self::Patch { user } => format!("patch|{}", user.to_lowercase()),
            Self::Attributes { user, profile } => format!("attributes|{}|{profile}", user.to_lowercase()),
            Self::Detail { kind, no } => format!("detail|{kind}|{no}"),
            Self::Tree { user, path, open, search, follow } => format!("tree|{}|{path}|{open}|{search}|{follow}", user.to_lowercase()),
            Self::Inspector { user, path } => format!("inspector|{}|{path}", user.to_lowercase()),
            Self::Paint { user, profile } => format!("paint|{}|{profile}", user.to_lowercase()),
            Self::Rules { user, profile, which } => format!("rules|{}|{profile}|{which}", user.to_lowercase()),
        }
    }

    /// The counters this build follows: it is rebuilt when they move.
    fn inputs(&self, r: &Revisions, screens: u64) -> [u64; 3] {
        match self {
            Self::Station => [r.station, 0, 0],
            Self::Network => [r.network, 0, 0],
            Self::LumaSessions => [r.lumasessions, 0, 0],
            Self::Shows { .. } => [r.show_info, r.library, r.station],
            // A stick in or out moves the station; a save or an upload moves the library.
            Self::Files { .. } => [r.station, r.library, 0],
            Self::Pools { .. } | Self::PoolEditor { .. } => [r.show, r.claims, 0],
            Self::Windows { .. } | Self::ViewBar { .. } => [r.show, screens, 0],
            // Your own Highlight / Solo masters ride in the station view.
            Self::Encoder { .. } => [r.show, screens, r.modes],
            // The profile's filters are show data.
            Self::Oops { .. } => [r.undo, r.show, 0],
            // Running state (the current cue) is show data too.
            Self::Attributes { .. } | Self::Detail { .. } | Self::Sequence { .. } | Self::Tags { .. } => [r.show, 0, 0],
            // Who holds the Devices draft decides draft or live; Paste counts the clipboard.
            Self::Patch { .. } => [r.show, r.claims, r.clipboard],
            // The tree and the inspector read the show as the user (bookmarks, usage are show data).
            Self::Tree { .. } | Self::Inspector { .. } => [r.show, 0, 0],
            // The palette is fixed; the slots, recent colours and rules are show data.
            Self::Paint { .. } | Self::Rules { .. } => [r.show, 0, 0],
        }
    }

    fn keyed(&self) -> Option<Keyed> {
        match self {
            Self::Shows { .. } => Some(Keyed { list: "shows", key: "name" }),
            Self::Files { .. } => Some(Keyed { list: "files", key: "name" }),
            Self::Pools { .. } => Some(Keyed { list: "cells", key: "no" }),
            Self::PoolEditor { .. } => Some(Keyed { list: "members", key: "fixture" }),
            Self::Tree { .. } => Some(Keyed { list: "rows", key: "path" }),
            _ => None,
        }
    }

    /// The menu views the state thread builds only while asked for.
    fn demand(&self) -> Option<&'static str> {
        match self {
            Self::Network => Some("network"),
            Self::LumaSessions => Some("lumasessions"),
            _ => None,
        }
    }

    /// Build it now (a blocking call: reads what the station published, the
    /// show library lists the disk).
    fn build(&self, state: &HttpState) -> Value {
        let console = &state.console;
        match self {
            Self::Station => console.station(),
            Self::Network => console.network(),
            Self::LumaSessions => console.lumasessions(),
            Self::Shows { history, drive } => console.shows_for(Some(history.as_str()).filter(|h| !h.is_empty()), Some(drive.as_str()).filter(|d| !d.is_empty())),
            Self::Files { drive, folder, kind } => console.files(drive, folder, kind),
            Self::Pools { user, pool, datapool, profile, filter } => {
                // A pinned DataPool that is gone shows missing, never the
                // user's (R1-55, Q30).
                let follows = datapool.is_null()
                    || datapool.as_u64() == Some(0)
                    || datapool.as_str().is_some_and(|s| s.is_empty() || s.eq_ignore_ascii_case("selected"));
                match console.resolve_datapool(user, datapool) {
                    None if !follows => json!({ "missing": pool, "datapool_missing": true }),
                    resolved => console.pool(user, pool, resolved, profile, filter).unwrap_or_else(|| json!({ "missing": pool })),
                }
            }
            Self::PoolEditor { user, pool, datapool, no } => {
                let datapool = console.resolve_datapool(user, datapool);
                console
                    .pool_object(user, pool, datapool, *no)
                    .unwrap_or_else(|| json!({ "missing": no }))
            }
            Self::Windows { user, screen } => super::feeds::windows_data(state, user, *screen),
            Self::Encoder { user, screen } => super::feeds::encoder_data(state, user, *screen),
            Self::ViewBar { user, screen } => super::feeds::viewbar_data(state, user, *screen),
            Self::Oops { user } => state.console.oops(user),
            Self::Sequence { user, sequence } => console.sequence_sheet(user, sequence),
            Self::Tags { user, tag, deep } => console.tags(user, tag, *deep),
            Self::Patch { user } => console.patch(user),
            Self::Attributes { user, profile } => {
                console.attribute_board(user, profile).unwrap_or_else(|| json!({ "missing": profile }))
            }
            Self::Detail { kind, no } => console.detail_view(kind, *no).unwrap_or_else(|| json!({ "missing": no })),
            Self::Tree { user, path, open, search, follow } => {
                console.tree_window(user, &json!({ "path": path, "open": open, "search": search, "follow": follow }))
            }
            Self::Inspector { user, path } => console.inspector(user, path),
            Self::Paint { user, profile } => console.paint(user, profile),
            Self::Rules { user, profile, which } => console.rules(user, profile, which),
        }
    }
}

/// While a build lives, the station builds its menu view (`demand`).
struct Demand {
    console: Arc<dyn ConsoleBridge>,
    feed: &'static str,
}

impl Drop for Demand {
    fn drop(&mut self) {
        self.console.feed_demand(self.feed, false);
    }
}

/// A running shared build; dropping the last handle stops its producer.
pub struct SharedBuild {
    rx: watch::Receiver<Option<Arc<Built>>>,
    task: tokio::task::AbortHandle,
    _demand: Option<Demand>,
}

impl SharedBuild {
    /// The newest build (None until the first one is done).
    pub fn receiver(&self) -> watch::Receiver<Option<Arc<Built>>> {
        self.rx.clone()
    }
}

impl Drop for SharedBuild {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Every shared build, by key.
#[derive(Default)]
pub struct FeedHub {
    builds: Mutex<HashMap<String, Weak<SharedBuild>>>,
}

impl FeedHub {
    /// The shared build for `spec`, started when nobody has it yet. Call
    /// from inside the runtime.
    pub fn subscribe(&self, state: &HttpState, spec: BuildSpec) -> Arc<SharedBuild> {
        let key = spec.key();
        let mut builds = self.builds.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(build) = builds.get(&key).and_then(Weak::upgrade) {
            return build;
        }
        builds.retain(|_, b| b.strong_count() > 0);
        let demand = spec.demand().map(|feed| {
            state.console.feed_demand(feed, true);
            Demand { console: state.console.clone(), feed }
        });
        let (tx, rx) = watch::channel(None);
        let task = tokio::spawn(produce(spec, state.clone(), tx)).abort_handle();
        let build = Arc::new(SharedBuild { rx, task, _demand: demand });
        builds.insert(key, Arc::downgrade(&build));
        build
    }

    /// How many builds run (tests).
    pub fn running(&self) -> usize {
        self.builds
            .lock()
            .map(|b| b.values().filter(|w| w.strong_count() > 0).count())
            .unwrap_or(0)
    }
}

/// How often a build whose data says `"moving": true` (a group master
/// fading) is made again until it rests.
const MOVING_EVERY: Duration = Duration::from_millis(100);

async fn produce(spec: BuildSpec, state: HttpState, tx: watch::Sender<Option<Arc<Built>>>) {
    let mut revs = state.console.revisions();
    let mut screens = state.screen_events.watch();
    let follows_screens = matches!(spec, BuildSpec::Windows { .. } | BuildSpec::Encoder { .. } | BuildSpec::ViewBar { .. });
    let keyed = spec.keyed();
    let mut last_inputs = None;
    let mut prev: Option<(u64, Value)> = None;
    let mut moving = false;
    loop {
        let inputs = spec.inputs(&revs.borrow_and_update(), *screens.borrow_and_update());
        if last_inputs != Some(inputs) || moving {
            last_inputs = Some(inputs);
            let (s, st) = (spec.clone(), state.clone());
            let Ok(data) = tokio::task::spawn_blocking(move || s.build(&st)).await else {
                break;
            };
            moving = data.get("moving").and_then(Value::as_bool).unwrap_or(false);
            let text = data.to_string();
            let rev = fingerprint(&text);
            if prev.as_ref().map(|p| p.0) != Some(rev) {
                let diff = match (keyed, &prev) {
                    (Some(k), Some((base, before))) => diff(before, &data, k)
                        .map(|d| d.to_string())
                        .filter(|d| d.len() * 2 < text.len())
                        .map(|d| (*base, Arc::from(d))),
                    _ => None,
                };
                tx.send_replace(Some(Arc::new(Built { rev, text: Arc::from(text), diff })));
                prev = Some((rev, data));
            }
        }
        tokio::select! {
            changed = revs.changed() => if changed.is_err() { break },
            changed = screens.changed(), if follows_screens => if changed.is_err() { break },
            _ = tokio::time::sleep(MOVING_EVERY), if moving => {}
        }
    }
}

/// FNV-1a over the JSON text, 53 bits (a JavaScript number holds it).
pub fn fingerprint(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h & ((1 << 53) - 1)
}

/// A list item's key as text (`3` and `"3"` are the same row).
fn key_of(item: &Value, key: &str) -> Option<String> {
    match &item[key] {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// The change from `before` to `now` of a keyed list:
/// `{list, key, fields: {top-level field: new value}, set: {key: item},
/// del: [key], order?: [key]}` — `order` only when the keys' order isn't the old order
/// with the deleted keys taken out and the new ones appended. None when the
/// data isn't shaped for it (then the full data goes).
pub fn diff(before: &Value, now: &Value, keyed: Keyed) -> Option<Value> {
    let (b, n) = (before.as_object()?, now.as_object()?);
    let mut fields = Map::new();
    for (k, v) in n {
        if k != keyed.list && b.get(k) != Some(v) {
            fields.insert(k.clone(), v.clone());
        }
    }
    if b.keys().any(|k| k != keyed.list && !n.contains_key(k)) {
        return None;
    }
    let rows = |o: &Map<String, Value>| -> Option<Vec<(String, Value)>> {
        o.get(keyed.list)?
            .as_array()?
            .iter()
            .map(|item| Some((key_of(item, keyed.key)?, item.clone())))
            .collect()
    };
    let (old, new) = (rows(b)?, rows(n)?);
    let old_map: HashMap<&str, &Value> = old.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let new_keys: std::collections::HashSet<&str> = new.iter().map(|(k, _)| k.as_str()).collect();
    if old_map.len() != old.len() || new_keys.len() != new.len() {
        return None; // duplicate keys
    }
    let mut set = Map::new();
    for (k, v) in &new {
        if old_map.get(k.as_str()) != Some(&v) {
            set.insert(k.clone(), v.clone());
        }
    }
    let del: Vec<&str> = old.iter().map(|(k, _)| k.as_str()).filter(|k| !new_keys.contains(k)).collect();
    let mut expected: Vec<&str> = old.iter().map(|(k, _)| k.as_str()).filter(|k| new_keys.contains(k)).collect();
    expected.extend(new.iter().map(|(k, _)| k.as_str()).filter(|k| !old_map.contains_key(k)));
    let order: Vec<&str> = new.iter().map(|(k, _)| k.as_str()).collect();
    let mut d = json!({ "list": keyed.list, "key": keyed.key, "fields": fields, "set": set, "del": del });
    if expected != order {
        d["order"] = json!(order);
    }
    Some(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const K: Keyed = Keyed { list: "cells", key: "no" };

    /// What the kit's FeedClient does with a diff (`applyDiff` in feeds.js).
    fn apply(before: &Value, d: &Value) -> Value {
        let mut out = before.as_object().unwrap().clone();
        for (k, v) in d["fields"].as_object().unwrap() {
            out.insert(k.clone(), v.clone());
        }
        let old: Vec<Value> = before[K.list].as_array().unwrap().clone();
        let mut rows: Vec<(String, Value)> = old.iter().map(|v| (key_of(v, K.key).unwrap(), v.clone())).collect();
        let del: Vec<&str> = d["del"].as_array().unwrap().iter().map(|k| k.as_str().unwrap()).collect();
        rows.retain(|(k, _)| !del.contains(&k.as_str()));
        for (k, v) in d["set"].as_object().unwrap() {
            match rows.iter_mut().find(|(rk, _)| rk == k) {
                Some(row) => row.1 = v.clone(),
                None => rows.push((k.clone(), v.clone())),
            }
        }
        if let Some(order) = d["order"].as_array() {
            let by: HashMap<String, Value> = rows.into_iter().collect();
            rows = order.iter().map(|k| { let k = k.as_str().unwrap().to_string(); (k.clone(), by[&k].clone()) }).collect();
        }
        out.insert(K.list.into(), Value::Array(rows.into_iter().map(|(_, v)| v).collect()));
        Value::Object(out)
    }

    #[test]
    fn a_keyed_diff_sets_deletes_and_orders_rows() {
        let before = json!({"pool": "Group", "datapool": 1, "cells": [
            {"no": 1, "name": "a"}, {"no": 2, "name": "b"}, {"no": 3, "name": "c"}]});
        let now = json!({"pool": "Group", "datapool": 2, "cells": [
            {"no": 1, "name": "a"}, {"no": 3, "name": "C"}, {"no": 4, "name": "d"}]});
        let d = diff(&before, &now, K).unwrap();
        assert_eq!(d["fields"], json!({"datapool": 2}));
        assert_eq!(d["del"], json!(["2"]));
        assert_eq!(d["set"], json!({"3": {"no": 3, "name": "C"}, "4": {"no": 4, "name": "d"}}));
        assert!(d.get("order").is_none(), "old order, deletes out, new appended");
        assert_eq!(apply(&before, &d), now);

        // A new row in the middle: the order goes along.
        let middle = json!({"pool": "Group", "datapool": 2, "cells": [
            {"no": 1, "name": "a"}, {"no": 2, "name": "x"}, {"no": 3, "name": "C"}, {"no": 4, "name": "d"}]});
        let d = diff(&now, &middle, K).unwrap();
        assert_eq!(d["order"], json!(["1", "2", "3", "4"]));
        assert_eq!(apply(&now, &d), middle);

        // A field gone, or no list: the full data instead.
        assert!(diff(&before, &json!({"cells": []}), K).is_none());
        assert!(diff(&json!({"missing": "x"}), &now, K).is_none());
    }

    #[test]
    fn the_same_json_has_the_same_rev() {
        assert_eq!(fingerprint("{\"a\":1}"), fingerprint("{\"a\":1}"));
        assert_ne!(fingerprint("{\"a\":1}"), fingerprint("{\"a\":2}"));
        assert!(fingerprint("x") < 1 << 53);
    }
}
