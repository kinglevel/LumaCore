//! Pool windows (decisions §22 "Unified pool window"): one window for every
//! pool. A pool object declares a [`PoolDef`] on its `ObjectDef`; the server
//! turns its objects into generic [`PoolCell`]s (number, name, colour /
//! icon, who is editing it, whether something links to it) and the pool
//! fills in its own extras (Group: master bar and mode badge, how much of
//! the group is selected). The frontend has one PoolWindow component and
//! every gesture on it sends a real command line.

/// The one object door (decisions §35).
pub mod door;

use std::collections::{BTreeMap, HashSet};

use lumacore_tree::ShowData;
use lumacore_tree::node::NodeMeta;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

/// The highest number in a pool (1–9999; R1-11, Q3): a pool window
/// never has to lay out a million cells.
pub const MAX_NO: u32 = 9999;

/// What a pool object shows in a pool window.
pub struct PoolDef {
    /// Window type name (`Root.Resources.WindowTypes`), e.g. "groups".
    pub window: &'static str,
    /// What a tap can do (window setting `action`); the first is the default.
    /// Each is a command template: `{obj}` = keyword and number (`Group 3`).
    pub actions: &'static [PoolAction],
    /// The pool's objects as cells.
    pub cells: fn(&PoolContext) -> Vec<PoolCell>,
    /// One object's content for its editor (Group: members and grid).
    pub detail: Option<fn(&PoolContext, u32) -> Option<Value>>,
    /// What the whole window shows beside its cells (the Presets pool: the tags its presets carry,
    /// every number in use and the ones the filter hid).
    pub head: Option<fn(&PoolContext) -> Value>,
}

/// A tap action: its name and the command it sends.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct PoolAction {
    pub name: &'static str,
    /// `{obj}` is replaced by the object (`Group 3`).
    pub command: &'static str,
    /// A press-and-release action (Top: the echo while held, decisions §32): `command` at the
    /// press, this at the release — no hold sheet in between. Empty = a tap.
    #[serde(skip_serializing_if = "str::is_empty")]
    pub release: &'static str,
}

/// What cells are built from: the published show and claims, for one user
/// and datapool.
pub struct PoolContext<'a> {
    pub show: &'a ShowData,
    pub user: &'a str,
    pub datapool: u32,
    /// Object GUID → the user holding it.
    pub claims: &'a BTreeMap<Uuid, String>,
    /// GUIDs something else in the show links to.
    pub referenced: &'a HashSet<Uuid>,
    pub now_ms: u64,
    /// The profile a profile's pool shows (the Views pool's Profile setting); None = the user's.
    pub profile: Option<u32>,
    /// What the window narrows its pool to (the Presets window, decisions 0288).
    pub filter: PoolFilter,
}

/// A window's narrowing of its pool (the Presets window): the feature group its presets must
/// touch (None or `All` = every one) and the tags they must all carry.
#[derive(Debug, Clone, Default)]
pub struct PoolFilter {
    pub feature: Option<String>,
    pub tags: Vec<Uuid>,
}

impl PoolContext<'_> {
    /// The profile this pool shows: the one the window picked, else the user's own.
    pub fn profile(&self) -> Option<&lumacore_tree::UserProfile> {
        match self.profile {
            Some(no) => self.show.user_profiles.get(&no).map(|p| &**p),
            None => self.show.profile_for_user(self.user),
        }
    }
}

/// One pool button.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PoolCell {
    pub no: u32,
    pub guid: Uuid,
    pub name: Option<String>,
    /// `#rrggbbaa` from `NodeMeta.color`.
    pub color: Option<String>,
    pub icon: Option<Value>,
    /// Another user holding it (claimed / open in an editor) — the red lock.
    pub owner: Option<String>,
    /// Something else links to it (the reference icon).
    pub referenced: bool,
    pub locked: bool,
    /// How much of it fits the user's selection: `all`, `some`, `none`.
    pub applicability: Option<&'static str>,
    /// A level / progress bar (group master level, running macro, …).
    pub bar: Option<CellBar>,
    /// Small markers (group master mode, …).
    pub badges: Vec<String>,
    /// Executor style (the Sequence pool, decisions §30 "12f"): the cue before, the active cue and
    /// the next.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cues: Vec<PoolCue>,
    /// It runs (the blue mark along the bottom).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub running: bool,
    /// The user's selected one (a gold frame).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub selected: bool,
    /// What the cell draws of its object's layout (the Views pool: each screen's windows as
    /// boxes, decisions 0287).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Value>,
    /// What only this pool draws (the Presets pool: mode, feature groups, preview, use).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

/// One line of an executor-style cell: `state` `on` = the running sequence's active cue (gold),
/// `off` = where an off sequence stands (a thin frame), empty = before / next.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PoolCue {
    pub text: String,
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CellBar {
    pub kind: &'static str,
    /// 0–1.
    pub value: f64,
    pub text: String,
}

impl PoolCell {
    /// The generic part of a cell from an object's meta.
    pub fn of(no: u32, meta: &NodeMeta, ctx: &PoolContext) -> Self {
        Self {
            no,
            guid: meta.identity.guid,
            name: meta.identity.name.clone(),
            color: meta
                .annotation
                .paint
                .map(|[r, g, b, a]| format!("#{r:02x}{g:02x}{b:02x}{a:02x}")),
            icon: meta
                .annotation
                .icon
                .as_ref()
                .and_then(|i| serde_json::to_value(i).ok()),
            // Another user's claim is the red lock; your own isn't.
            owner: ctx
                .claims
                .get(&meta.identity.guid)
                .filter(|u| !u.eq_ignore_ascii_case(ctx.user))
                .cloned(),
            referenced: ctx.referenced.contains(&meta.identity.guid),
            locked: meta.annotation.locked,
            applicability: None,
            bar: None,
            badges: Vec::new(),
            cues: Vec::new(),
            running: false,
            selected: false,
            layout: None,
            extra: None,
        }
    }
}

/// The datapool a user works in (their selection, decisions §29, else the
/// first).
pub fn user_datapool(show: &ShowData, user: &str) -> u32 {
    let first = show.datapools.keys().next().copied().unwrap_or(1);
    show.user(user)
        .and_then(|u| u.selected.datapool)
        .and_then(|g| {
            show.datapools
                .iter()
                .find(|(_, d)| d.meta.identity.guid == g)
                .map(|(&n, _)| n)
        })
        .unwrap_or(first)
}

/// A pool window's DataPool setting (decisions §30 "12b2": a GUID link):
/// null, 0 or "Selected" follow the user's selected datapool; a number is
/// that datapool; a GUID string the datapool with that GUID. None when the
/// pinned datapool is gone: the window shows it missing, never silently
/// another one (R1-55, Q30).
pub fn resolve_datapool(show: &ShowData, user: &str, v: &Value) -> Option<u32> {
    match v {
        Value::Number(n) => match n.as_u64().filter(|n| *n > 0) {
            Some(n) => show.datapools.contains_key(&(n as u32)).then_some(n as u32),
            None => Some(user_datapool(show, user)),
        },
        Value::String(s) => match s.parse::<Uuid>() {
            Ok(g) => show.datapools.iter().find(|(_, d)| d.meta.identity.guid == g).map(|(&n, _)| n),
            Err(_) => Some(user_datapool(show, user)),
        },
        _ => Some(user_datapool(show, user)),
    }
}

/// A window's Profile setting (the Views pool): a profile's number or GUID, else the user's own.
pub fn resolve_profile(show: &ShowData, user: &str, v: &Value) -> Option<u32> {
    match v {
        Value::Number(n) => match n.as_u64().filter(|n| *n > 0) {
            Some(n) => show.user_profiles.contains_key(&(n as u32)).then_some(n as u32),
            None => show.profile_id_for_user(user),
        },
        Value::String(s) => match s.parse::<Uuid>() {
            Ok(g) => show.user_profiles.iter().find(|(_, p)| p.meta.identity.guid == g).map(|(&n, _)| n),
            Err(_) => show.profile_id_for_user(user),
        },
        _ => show.profile_id_for_user(user),
    }
}

/// Whether a pool window's DataPool setting follows the user's selection.
pub fn follows_selection(v: &Value) -> bool {
    match v {
        Value::Number(n) => n.as_u64().is_none_or(|n| n == 0),
        Value::String(s) => s.parse::<Uuid>().is_err(),
        _ => true,
    }
}

/// GUIDs something in the show links to (a group's own master level in
/// Playbacks doesn't count).
pub fn referenced(show: &ShowData) -> HashSet<Uuid> {
    show.walk_identity()
        .links
        .iter()
        .filter(|l| !l.path.starts_with("Playbacks."))
        .map(|l| l.target)
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_window_datapool_resolves_by_guid_number_or_selection() {
        let mut show = ShowData::default();
        show.datapools.insert(1, lumacore_tree::Datapool::new(Some("Default".into())));
        let other = lumacore_tree::Datapool::new(Some("Other".into()));
        let guid = other.meta.identity.guid;
        show.datapools.insert(2, other);
        let mine = user_datapool(&show, "Admin");
        for follow in [Value::Null, json!(0), json!("selected"), json!("Selected")] {
            assert_eq!(resolve_datapool(&show, "Admin", &follow), Some(mine), "{follow}");
            assert!(follows_selection(&follow));
        }
        assert_eq!(resolve_datapool(&show, "Admin", &json!(2)), Some(2));
        assert_eq!(resolve_datapool(&show, "Admin", &json!(guid.to_string())), Some(2));
        // Gone: missing, never the user's (Q30).
        assert_eq!(resolve_datapool(&show, "Admin", &json!(Uuid::new_v4().to_string())), None, "gone");
        assert_eq!(resolve_datapool(&show, "Admin", &json!(7)), None);
        assert!(!follows_selection(&json!(guid.to_string())));
    }
}
