//! The one change path (`docs/meta/decisions.md` §26).
//!
//! Handlers write the show directly; this module finds out what they changed.
//! Before a mutating command every ShowData subtree is fingerprinted (a
//! hashing pass, no copies); afterwards it is fingerprinted again, and exactly
//! the subtrees whose fingerprint moved are the change: they get the undo
//! frame, the replication op, the journal entry and a new published snapshot.
//! Nothing is listed by hand.
//!
//! The tracker keeps the serialized image of every subtree as of the last
//! change, so the "before" bytes of a change are already there. Writes that
//! happen outside commands (a show load, a timer) are noticed on the next
//! pass and the image is refreshed — they are not blamed on the next command.

use std::collections::HashMap;

use lumacore_tree::{ShowData, ShowSubtree, SubtreeSnapshot};

use super::AppState;
use super::oops;

/// Serialized form of a subtree that doesn't exist.
const ABSENT: &[u8] = b"null";
/// What a subtree that doesn't exist hashes as (no real walk starts with a bare u64).
const ABSENT_HASH: u64 = u64::MAX;

#[derive(Debug, Default)]
pub struct ChangeTracker {
    images: HashMap<ShowSubtree, Image>,
}

#[derive(Debug)]
struct Image {
    hash: u64,
    bytes: Vec<u8>,
}

/// A command in progress: the fingerprints from before it ran.
pub struct Pass {
    before: HashMap<ShowSubtree, u64>,
}

/// One changed subtree, as bytes before and after.
#[derive(Debug, Clone)]
pub struct Change {
    pub subtree: ShowSubtree,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
}

/// Every subtree of the show the change path watches — the only list,
/// generated from ShowData's fields (decisions §21): the destructure is
/// exhaustive, so a new field doesn't compile until it has a subtree here
/// and in `oops::{write_subtree, restore_subtree}`. Users come before their
/// programmers and selections so a restore re-creates a user before filling them.
pub fn subtrees(show: &ShowData) -> Vec<ShowSubtree> {
    use ShowSubtree as S;
    let ShowData {
        devices: _,
        datapools,
        datapool_priority_orders: _,
        show_settings: _,
        spatial_3d: _,
        playback: _,
        playbacks: _,
        session: _,
        user_profiles,
        users,
        protocols: _,
        effect_engines: _,
        variables: _,
        images: _,
        assets: _,
        attribute_defs: _,
        feature_groups: _,
        activation_groups: _,
        attribute_rules: _,
        stations: _,
        tags: _,
    } = show;
    let mut out = vec![
        S::Devices,
        S::DatapoolPriorityOrders,
        S::ShowSettings,
        S::Spatial3D,
        S::Playback,
        S::Playbacks,
        S::Session,
        S::Protocols,
        S::EffectEngines,
        S::Variables,
        S::Images,
        S::Assets,
        S::AttributeDefs,
        S::FeatureGroups,
        S::ActivationGroups,
        S::AttributeRules,
        S::Stations,
        S::Tags,
    ];
    out.extend(datapools.values().map(|d| S::DataPool(d.meta.identity.guid)));
    out.extend(user_profiles.values().map(|p| S::Profile(p.meta.identity.guid)));
    out.extend(users.values().map(|u| S::User(u.meta.identity.guid)));
    out.extend(users.values().map(|u| S::Programmer(u.meta.identity.guid)));
    out.extend(users.values().map(|u| S::Selection(u.meta.identity.guid)));
    out.extend(users.values().map(|u| S::Usage(u.meta.identity.guid)));
    out
}

/// A subtree's fingerprint: the walk into a hasher (`oops::fingerprint_subtree`); a pool entry
/// that doesn't exist hashes as absent.
fn fingerprint(state: &AppState, subtree: &ShowSubtree) -> u64 {
    oops::fingerprint_subtree(state, subtree).unwrap_or(ABSENT_HASH)
}

/// The fingerprint of every subtree that exists now, plus every one `also` names (gone ones
/// hash as absent), in the subtrees' one order; the ones in `skip` are left out.
fn fingerprints(state: &AppState, also: &HashMap<ShowSubtree, u64>, skip: &[ShowSubtree]) -> Vec<(ShowSubtree, u64)> {
    let mut candidates = subtrees(&state.root.show_data);
    for subtree in also.keys() {
        if !candidates.contains(subtree) {
            candidates.push(subtree.clone());
        }
    }
    candidates
        .into_iter()
        .filter(|s| !skip.contains(s))
        .map(|s| {
            let h = fingerprint(state, &s);
            (s, h)
        })
        .collect()
}

fn hash_of(now: &[(ShowSubtree, u64)], subtree: &ShowSubtree) -> Option<u64> {
    now.iter().find(|(s, _)| s == subtree).map(|(_, h)| *h)
}

fn image_bytes(state: &AppState, subtree: &ShowSubtree) -> Vec<u8> {
    oops::snapshot_subtree(state, subtree).unwrap_or_else(|| ABSENT.to_vec())
}

/// Fingerprint the show before a command; refresh images that went stale.
pub fn begin(state: &mut AppState) -> Pass {
    let mut tracker = std::mem::take(&mut state.changes);
    let all = subtrees(&state.root.show_data);
    tracker.images.retain(|k, _| all.contains(k));
    let mut before = HashMap::with_capacity(all.len());
    for subtree in all {
        let hash = fingerprint(state, &subtree);
        if tracker.images.get(&subtree).is_none_or(|i| i.hash != hash) {
            let bytes = image_bytes(state, &subtree);
            tracker
                .images
                .insert(subtree.clone(), Image { hash, bytes });
        }
        before.insert(subtree, hash);
    }
    state.changes = tracker;
    Pass { before }
}

/// Subtrees whose fingerprint moved since `pass` (including ones that
/// appeared or disappeared), with their new fingerprint.
fn moved(state: &AppState, pass: &Pass) -> Vec<(ShowSubtree, u64)> {
    moved_in(fingerprints(state, &pass.before, &[]), pass)
}

fn moved_in(now: Vec<(ShowSubtree, u64)>, pass: &Pass) -> Vec<(ShowSubtree, u64)> {
    now.into_iter()
        .filter(|(subtree, hash)| pass.before.get(subtree) != Some(hash))
        .collect()
}

/// After the command: what it changed. The images move forward.
pub fn finish(state: &mut AppState, pass: Pass) -> Vec<Change> {
    // A patch change may bring attributes the show has never seen: they get
    // their definition (placed by the resolver, marked new) in this same
    // step (decisions §30 "12f-a"). Not while a Devices draft is open — the
    // draft's attributes arrive with Keep, never before.
    state.synced_default = false;
    // Every subtree hashed once (the big one is Devices: a rig's thousand fixtures and their
    // types, decisions §40 "40f built"); the sync below touches everything but Devices, so
    // only the rest is hashed again after it.
    let mut now = fingerprints(state, &pass.before, &[]);
    let devices = ShowSubtree::Devices;
    if state.patch_draft.is_none() && pass.before.get(&devices).copied() != hash_of(&now, &devices) {
        // The locked Default learns of them too; that write is the sync's,
        // not the line's.
        let default = state.root.show_data.default_profile().map(|(_, p)| ShowSubtree::Profile(p.meta.identity.guid));
        let untouched = default
            .as_ref()
            .map(|d| (d.clone(), hash_of(&now, d).unwrap_or(ABSENT_HASH)))
            .filter(|(d, h)| pass.before.get(d) == Some(h));
        lumacore_tree::attribute_resolver::sync(&mut state.root.show_data);
        let devices_hash = hash_of(&now, &devices);
        now = fingerprints(state, &pass.before, std::slice::from_ref(&devices));
        if let Some(h) = devices_hash {
            now.insert(0, (devices.clone(), h));
        }
        state.synced_default = untouched.is_some_and(|(d, was)| hash_of(&now, &d) != Some(was));
    }
    let changed = moved_in(now, &pass);
    let mut tracker = std::mem::take(&mut state.changes);
    let mut out = Vec::with_capacity(changed.len());
    for (subtree, hash) in changed {
        let before = tracker
            .images
            .get(&subtree)
            .map(|i| i.bytes.clone())
            .unwrap_or_else(|| ABSENT.to_vec());
        let after = image_bytes(state, &subtree);
        tracker.images.insert(
            subtree.clone(),
            Image {
                hash,
                bytes: after.clone(),
            },
        );
        out.push(Change {
            subtree,
            before,
            after,
        });
    }
    state.changes = tracker;
    if !out.is_empty() {
        state.show_generation += 1;
    }
    if out.iter().any(|c| c.subtree == ShowSubtree::Devices) {
        state.patch_generation += 1;
    }
    out
}

/// After a failed (panicked) command: put every subtree it touched back.
/// Returns what was restored.
pub fn rollback(state: &mut AppState, pass: Pass) -> Result<Vec<ShowSubtree>, String> {
    let changed = moved(state, &pass);
    let snapshots: Vec<SubtreeSnapshot> = changed
        .iter()
        .map(|(subtree, _)| SubtreeSnapshot {
            subtree: subtree.clone(),
            data: state
                .changes
                .images
                .get(subtree)
                .map(|i| i.bytes.clone())
                .unwrap_or_else(|| ABSENT.to_vec()),
        })
        .collect();
    oops::restore_subtrees(state, &snapshots)?;
    Ok(changed.into_iter().map(|(s, _)| s).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_exactly_the_changed_subtrees() {
        let mut state = AppState::new();
        state.root.show_data.show_settings.show_name = "Before".to_string();
        let pass = begin(&mut state);
        state.root.show_data.show_settings.show_name = "After".to_string();
        let changes = finish(&mut state, pass);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].subtree, ShowSubtree::ShowSettings);
        assert!(String::from_utf8_lossy(&changes[0].before).contains("Before"));
        assert!(String::from_utf8_lossy(&changes[0].after).contains("After"));

        // Nothing changed: nothing reported.
        let pass = begin(&mut state);
        assert!(finish(&mut state, pass).is_empty());
    }

    #[test]
    fn rollback_restores_and_removes_new_datapools() {
        let mut state = AppState::new();
        state.root.show_data.show_settings.show_name = "Kept".to_string();
        let pass = begin(&mut state);
        state.root.show_data.show_settings.show_name = "Broken".to_string();
        let pool = lumacore_tree::Datapool::new(None);
        let guid = pool.meta.identity.guid;
        state.root.show_data.datapools.insert(77, pool);
        let restored = rollback(&mut state, pass).unwrap();
        assert!(restored.contains(&ShowSubtree::ShowSettings));
        assert!(restored.contains(&ShowSubtree::DataPool(guid)));
        assert_eq!(state.root.show_data.show_settings.show_name, "Kept");
        assert!(!state.root.show_data.datapools.contains_key(&77));
    }

    #[test]
    fn writes_outside_commands_are_not_blamed_on_the_next_one() {
        let mut state = AppState::new();
        let pass = begin(&mut state);
        let _ = finish(&mut state, pass);
        state.root.show_data.show_settings.show_name = "Loaded".to_string();
        let pass = begin(&mut state);
        assert!(finish(&mut state, pass).is_empty());
    }
}
