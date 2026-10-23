//  ██████╗  ██████╗ ██████╗ ███████╗
// ██╔═══██╗██╔═══██╗██╔══██╗██╔════╝
// ██║   ██║██║   ██║██████╔╝███████╗
// ██║   ██║██║   ██║██╔═══╝ ╚════██║
// ╚██████╔╝╚██████╔╝██║     ███████║
//  ╚═════╝  ╚═════╝ ╚═╝     ╚══════╝
// FILE: oops

//! Undo's state half (decisions §21, §29 "11c", §34 15): ShowData subtrees snapshot and put
//! back, and a step's station-local state restored on its own station. The change path and the
//! Oops / Redo commands build on it.
//!
//! Owns:          ShowData subtree snapshots and their restore, a step's station-local state restored.
//! Doesn't own:   the Oops / Redo / List Undo commands and the confirmation (commands/oops.rs), what changed (change.rs).
//! Entry points:  snapshot_subtree, snapshot_subtrees, write_subtree, restore_subtree, restore_subtrees, restore_local, Side.
//! Decisions:     §21, §29 "11c", §34 15.

use lumacore_tree::{LocalSnapshot, LocalSubtree, ShowSubtree, SubtreeSnapshot};

use std::hash::Hasher as _;

use crate::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

// *************************************************************************************************
// Subtree snapshot / restore
// *************************************************************************************************
/// Snapshot a single subtree of ShowData as JSON bytes.
pub fn snapshot_subtree(state: &AppState, subtree: &ShowSubtree) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    write_subtree(state, subtree, &mut bytes)?;
    Some(bytes)
}

/// Serialize a subtree into `out` (a byte buffer: the images the change path keeps, the undo
/// frames, the replication ops). A pool entry is written as `[slot, object]` (its number is its
/// key, decisions §12 "Tags part 1b hardened" 4); one that doesn't exist as `null`, so creating
/// and deleting one can be undone too.
pub fn write_subtree(
    state: &AppState,
    subtree: &ShowSubtree,
    out: &mut impl std::io::Write,
) -> Option<()> {
    with_subtree(state, subtree, &mut |v| v.write(out))
}

/// The change path's fingerprint of a subtree: the same walk as `write_subtree`, into a hasher
/// (`lumacore_base::fingerprint`, no text — decisions §40 "40f built"). `None` for a pool entry
/// that doesn't exist.
pub fn fingerprint_subtree(state: &AppState, subtree: &ShowSubtree) -> Option<u64> {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    with_subtree(state, subtree, &mut |v| {
        v.fingerprint(&mut h);
        Some(())
    })?;
    Some(h.finish())
}

/// The one match from a subtree to the value it names (the two readers above).
fn with_subtree(
    state: &AppState,
    subtree: &ShowSubtree,
    w: &mut dyn FnMut(&dyn erased::Ser) -> Option<()>,
) -> Option<()> {
    use ShowSubtree as S;
    let show = &state.root.show_data;
    match subtree {
        S::Devices => w(&show.devices),
        S::DataPool(g) => w(&show.datapools.iter().find(|(_, d)| d.meta.identity.guid == *g)),
        S::DatapoolPriorityOrders => w(&show.datapool_priority_orders),
        S::ShowSettings => w(&show.show_settings),
        S::Spatial3D => w(&show.spatial_3d),
        S::Playback => w(&show.playback),
        S::Playbacks => w(&show.playbacks),
        S::Session => w(&show.session),
        S::Profile(g) => w(&show.user_profiles.iter().find(|(_, p)| p.meta.identity.guid == *g)),
        S::User(g) => w(&show.users.iter().find(|(_, u)| u.meta.identity.guid == *g).map(|(n, u)| (n, UserIdentity::of(u)))),
        S::Programmer(g) => w(&show.user_by_guid(*g).map(|u| u.programmer_values())),
        S::Selection(g) => w(&show.user_by_guid(*g).map(|u| u.selection())),
        S::Usage(g) => w(&show.user_by_guid(*g).map(|u| &u.usage)),
        S::Bookmarks(g) => w(&show.user_by_guid(*g).map(|u| &u.bookmarks)),
        S::Protocols => w(&show.protocols),
        S::EffectEngines => w(&show.effect_engines),
        S::Variables => w(&show.variables),
        S::Images => w(&show.images),
        S::Assets => w(&show.assets),
        S::AttributeDefs => w(&show.attribute_defs),
        S::FeatureGroups => w(&show.feature_groups),
        S::ActivationGroups => w(&show.activation_groups),
        S::AttributeRules => w(&show.attribute_rules),
        S::Tags => w(&show.tags),
        S::Stations => w(&show.stations),
    }
}

/// Serialize through `dyn` (the match above hands out different types).
mod erased {
    pub trait Ser {
        fn write(&self, out: &mut dyn std::io::Write) -> Option<()>;
        fn fingerprint(&self, h: &mut std::collections::hash_map::DefaultHasher);
    }
    impl<T: serde::Serialize> Ser for T {
        fn write(&self, out: &mut dyn std::io::Write) -> Option<()> {
            serde_json::to_writer(out, self).ok()
        }
        fn fingerprint(&self, h: &mut std::collections::hash_map::DefaultHasher) {
            lumacore_base::fingerprint::into(self, h)
        }
    }
}

/// A user without their programmer and selection (their own subtrees). The
/// destructure is exhaustive: a new `User` field must be placed here too.
#[derive(serde::Serialize)]
struct UserIdentity<'a> {
    meta: &'a lumacore_base::NodeMeta,
    login: &'a String,
    #[serde(skip_serializing_if = "Option::is_none")]
    password_hash: &'a Option<String>,
    rights: &'a lumacore_tree::UserRights,
    remote_login: &'a bool,
    profile: &'a uuid::Uuid,
    color: &'a String,
    default_screen_config: &'a Option<uuid::Uuid>,
    default_object: &'a Option<String>,
}

impl<'a> UserIdentity<'a> {
    fn of(user: &'a lumacore_tree::User) -> Self {
        let lumacore_tree::User {
            meta,
            login,
            password_hash,
            rights,
            remote_login,
            profile,
            color,
            default_screen_config,
            default_object,
            programmer: _,
            selected: _,
            usage: _,
            bookmarks: _,
        } = user;
        Self {
            meta,
            login,
            password_hash,
            rights,
            remote_login,
            profile,
            color,
            default_screen_config,
            default_object,
        }
    }
}

/// Restore a single subtree of ShowData from JSON bytes (`null` = the pool
/// entry didn't exist: it is removed).
pub fn restore_subtree(
    state: &mut AppState,
    subtree: &ShowSubtree,
    data: &[u8],
) -> Result<(), String> {
    use ShowSubtree as S;
    fn parse<T: serde::de::DeserializeOwned>(subtree: &ShowSubtree, data: &[u8]) -> Result<T, String> {
        serde_json::from_slice(data).map_err(|e| format!("{subtree} restore failed: {e}"))
    }
    /// A pool entry by GUID (decisions §21 "6d"): the object leaves
    /// whatever slot it had and lands in the slot its data names; a slot
    /// held by another object is an error (never overwritten).
    fn entry<T>(
        subtree: &ShowSubtree,
        pool: &mut std::collections::BTreeMap<u32, T>,
        guid: uuid::Uuid,
        value: Option<(u32, T)>,
        guid_of: impl Fn(&T) -> uuid::Uuid,
    ) -> Result<(), String> {
        pool.retain(|_, v| guid_of(v) != guid);
        let Some((slot, v)) = value else {
            return Ok(());
        };
        if guid_of(&v) != guid {
            return Err(format!("{subtree} restore failed: the data is another object"));
        }
        if pool.contains_key(&slot) {
            return Err(format!(
                "{subtree} restore failed: slot {slot} holds another object"
            ));
        }
        pool.insert(slot, v);
        Ok(())
    }
    let show = &mut state.root.show_data;
    match subtree {
        S::Devices => show.devices = parse(subtree, data)?,
        S::DataPool(g) => entry(
            subtree,
            &mut show.datapools,
            *g,
            parse(subtree, data)?,
            |d| d.meta.identity.guid,
        )?,
        S::DatapoolPriorityOrders => show.datapool_priority_orders = parse(subtree, data)?,
        S::ShowSettings => show.show_settings = parse(subtree, data)?,
        S::Spatial3D => show.spatial_3d = parse(subtree, data)?,
        S::Playback => show.playback = parse(subtree, data)?,
        S::Playbacks => show.playbacks = parse(subtree, data)?,
        S::Session => show.session = parse(subtree, data)?,
        S::Profile(g) => entry(
            subtree,
            &mut show.user_profiles,
            *g,
            parse(subtree, data)?,
            |p| p.meta.identity.guid,
        )?,
        S::User(g) => {
            // Programmer and selection are their own subtrees: keep the
            // ones in place.
            let mut user: Option<(u32, lumacore_tree::User)> = parse(subtree, data)?;
            if let Some((_, new)) = user.as_mut()
                && let Some(old) = show.users.values_mut().find(|u| u.meta.identity.guid == *g)
            {
                new.programmer = std::mem::take(&mut old.programmer);
                new.selected = std::mem::take(&mut old.selected);
                new.usage = std::mem::take(&mut old.usage);
                new.bookmarks = std::mem::take(&mut old.bookmarks);
            }
            entry(subtree, &mut show.users, *g, user, |u| u.meta.identity.guid)?;
        }
        S::Programmer(g) => {
            // Null: the user didn't exist (their User subtree removes them).
            let values: Option<lumacore_tree::ProgrammerValues> = parse(subtree, data)?;
            if let (Some(v), Some(user)) = (values, show.users.values_mut().find(|u| u.meta.identity.guid == *g)) {
                user.set_programmer_values(v);
            }
        }
        S::Selection(g) => {
            let selection: Option<lumacore_tree::SelectionState> = parse(subtree, data)?;
            if let (Some(s), Some(user)) = (selection, show.users.values_mut().find(|u| u.meta.identity.guid == *g)) {
                user.set_selection(s);
            }
        }
        S::Usage(g) => {
            let usage: Option<lumacore_tree::Usage> = parse(subtree, data)?;
            if let (Some(u), Some(user)) = (usage, show.users.values_mut().find(|u| u.meta.identity.guid == *g)) {
                user.usage = u;
            }
        }
        S::Bookmarks(g) => {
            let marks: Option<lumacore_tree::Bookmarks> = parse(subtree, data)?;
            if let (Some(b), Some(user)) = (marks, show.users.values_mut().find(|u| u.meta.identity.guid == *g)) {
                user.bookmarks = b;
            }
        }
        S::Protocols => show.protocols = parse(subtree, data)?,
        S::EffectEngines => show.effect_engines = parse(subtree, data)?,
        S::Variables => show.variables = parse(subtree, data)?,
        S::Images => show.images = parse(subtree, data)?,
        S::Assets => show.assets = parse(subtree, data)?,
        S::AttributeDefs => show.attribute_defs = parse(subtree, data)?,
        S::FeatureGroups => show.feature_groups = parse(subtree, data)?,
        S::ActivationGroups => show.activation_groups = parse(subtree, data)?,
        S::AttributeRules => show.attribute_rules = parse(subtree, data)?,
        S::Tags => show.tags = parse(subtree, data)?,
        S::Stations => show.stations = parse(subtree, data)?,
    }
    Ok(())
}

/// Snapshot multiple subtrees at once.
pub fn snapshot_subtrees(
    state: &AppState,
    subtrees: &[ShowSubtree],
) -> Option<Vec<SubtreeSnapshot>> {
    let mut snapshots = Vec::with_capacity(subtrees.len());
    for subtree in subtrees {
        let data = snapshot_subtree(state, subtree)?;
        snapshots.push(SubtreeSnapshot {
            subtree: subtree.clone(),
            data,
        });
    }
    Some(snapshots)
}

/// Restore multiple subtrees from an undo frame's snapshots.
pub fn restore_subtrees(state: &mut AppState, snapshots: &[SubtreeSnapshot]) -> Result<(), String> {
    // Removals first: an object moving into a slot another one left in the
    // same change finds it free.
    let (gone, mut kept): (Vec<_>, Vec<_>) = snapshots.iter().partition(|s| s.data == b"null");
    // A user comes back before their programmer, selection and usage (a
    // deleted user's step, decisions §30 "12c2"): those fill the user.
    kept.sort_by_key(|s| {
        !matches!(s.subtree, lumacore_tree::ShowSubtree::Profile(_) | lumacore_tree::ShowSubtree::User(_))
    });
    for snap in gone.into_iter().chain(kept) {
        restore_subtree(state, &snap.subtree, &snap.data)?;
    }
    state.root.show_data.after_load();
    state.show_generation += 1;
    if snapshots
        .iter()
        .any(|s| s.subtree == lumacore_tree::ShowSubtree::Devices)
    {
        state.patch_generation += 1;
    }
    // Oops / Redo and replication put profiles back: the layer above refreshes their view bars.
    state.profiles_touched.extend(snapshots.iter().filter_map(|s| match s.subtree {
        lumacore_tree::ShowSubtree::Profile(g) => Some(g),
        _ => None,
    }));
    Ok(())
}

// *************************************************************************************************
// Station-local state
// *************************************************************************************************
/// Which side of a step's local state to put back: Oops → `Before`,
/// Redo → `After`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Before,
    After,
}

/// Put a step's station-local state back (station settings, watches) —
/// only the parts that belong to this station (decisions §29 "11c"); the
/// other stations do theirs when the step reaches them.
pub fn restore_local(state: &mut AppState, local: &[LocalSnapshot], side: Side) {
    let me = state.local_settings.station_id.clone();
    for snap in local.iter().filter(|l| l.station_id == me) {
        let data = match side {
            Side::Before => &snap.before,
            Side::After => &snap.after,
        };
        match &snap.what {
            LocalSubtree::Station => {
                match serde_json::from_slice::<lumacore_luma::station_config::StationImage>(data) {
                    Ok(image) => {
                        image.apply_to(&mut state.root.station_settings);
                        state
                            .local_settings
                            .store_from(&state.root.station_settings);
                        state.protocols_dirty = true;
                    }
                    Err(e) => log::error!("station settings restore failed: {e}"),
                }
            }
            LocalSubtree::Watches(user) => {
                match serde_json::from_slice::<Vec<lumacore_tree::WatchTarget>>(data) {
                    Ok(watches) => state.root.runtime.user(user).watches = watches,
                    Err(e) => log::error!("watches restore failed: {e}"),
                }
            }
        }
    }
}
