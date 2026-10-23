//! Claims and ownership (decisions §4, §10, §21 "6d hardening").
//!
//! - Claims are keyed by the object's **GUID** (a move keeps the claim);
//!   areas (`Patch`, `DataPools`, …) have a GUID derived from their name.
//! - **Opening an editor claims** (`Edit`, `Enter`, UI editor windows),
//!   indirectly too (a cue's editor claims its sequence); leaving releases
//!   it. `cd` never claims.
//! - Plain commands don't claim but are refused on an object (or anything
//!   inside an object) another user holds (E-CLAIM-001).
//! - Taking a claimed object asks its owner: Allow (`Unclaim …`) / Deny
//!   (`Claim … /Keep`) within 10 s, else — or when the owner is logged out —
//!   it is handed over. An Admin's `Claim … /Force` skips the question.

use lumacore_base::Guid;
use lumacore_tree::{Claim, ClaimAsk, UserRights};
use uuid::Uuid;

use super::INTERNAL_ROOT_USER;
use super::app_state::AppState;

pub use crate::dialogs::TAKEOVER;

/// Areas claimed as a whole (`Claim Root.ShowData.Devices`, `Enter` into them). Devices holds every device
/// kind's draft (decisions §34 "34g4 patch hardened" 8; it replaced the Patch claim).
pub const AREAS: [&str; 7] = [
    "ShowData",
    "Devices",
    "DataPools",
    "Protocols",
    "UserProfiles",
    "ShowSettings",
    "StationSettings",
];

/// The level claiming a whole area needs (`CommandDef::arg_rights` of
/// `Claim`): the datapools are Program, every other area — the patch, the
/// lines, the settings, the profiles — is Setup. None = not an area.
pub fn area_rights(word: &str) -> Option<crate::commands::registry::Rights> {
    use crate::commands::registry::Rights;
    let area = area_of(word)?;
    Some(if area == "DataPools" { Rights::Program } else { Rights::Setup })
}

/// How long the Devices holder may be gone before their draft goes (a page reload reconnects
/// well within it; the LumaSession calls a station lost after 5 s too).
pub const HOLDER_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

/// The lost-holder watch (decisions §34 "34g4 patch entry hardened" 5), fed once a second with
/// who holds the Devices and who is present anywhere: it answers true once the holder has been
/// gone for [`HOLDER_GRACE`]; a holder back in time, or another holder, starts it over.
#[derive(Default)]
pub struct HolderWatch {
    gone_since: Option<(String, std::time::Instant)>,
}

impl HolderWatch {
    /// Whether the draft's holder is lost now (then [`Claims::release_lost_devices`] runs).
    pub fn lost(&mut self, holder: Option<&str>, present: &[String], now: std::time::Instant) -> bool {
        let lost = holder.filter(|h| !present.iter().any(|u| u.eq_ignore_ascii_case(h)));
        match (lost, &self.gone_since) {
            (None, _) => {
                self.gone_since = None;
                false
            }
            (Some(h), Some((was, since))) if was == h => {
                let gone = now.duration_since(*since) >= HOLDER_GRACE;
                if gone {
                    self.gone_since = None;
                }
                gone
            }
            (Some(h), _) => {
                self.gone_since = Some((h.to_string(), now));
                false
            }
        }
    }
}

/// Every JSON object that is the object with this GUID: one whose `guid`
/// (or `meta.guid`) names it — with everything inside it.
fn objects_with<'a>(value: &'a serde_json::Value, guid: &str) -> Vec<&'a serde_json::Value> {
    fn walk<'a>(v: &'a serde_json::Value, guid: &str, out: &mut Vec<&'a serde_json::Value>) {
        match v {
            serde_json::Value::Object(map) => {
                let named = |m: &serde_json::Map<String, serde_json::Value>| {
                    m.get("guid").and_then(|g| g.as_str()) == Some(guid)
                };
                if named(map) || map.get("meta").and_then(|m| m.as_object()).is_some_and(named) {
                    out.push(v);
                    return;
                }
                map.values().for_each(|x| walk(x, guid, out));
            }
            serde_json::Value::Array(items) => items.iter().for_each(|x| walk(x, guid, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(value, guid, &mut out);
    out
}

use lumacore_base::clock::now_ms;

/// The area a word names: its name (`Devices`) or its tree path (`Root.ShowData.Devices`, the
/// form lines write — the areas aren't Lum words, decisions §34 "34g4 patch hardened" 8).
pub fn area_of(word: &str) -> Option<&'static str> {
    let word = word.trim().trim_matches('"');
    if let Some(area) = AREAS.iter().find(|a| a.eq_ignore_ascii_case(word)) {
        return Some(area);
    }
    let path = crate::tree_path::TreePath::parse_dot(word)?;
    // Exactly an area's node: a path below it names an object, not the area.
    AREAS.iter().copied().find(|a| path.entry().is_none() && path.node().claim() == Some(*a) && path.node().name() == *a)
}

/// What a claim target names: (GUID, label as shown).
pub fn resolve(state: &AppState, label: &str) -> Result<(Uuid, String), String> {
    let label = label.trim();
    if let Some(area) = area_of(label) {
        return Ok((Guid::named("Area", area).0, area.to_string()));
    }
    let mut words = label.split_whitespace();
    let (Some(word), Some(id)) = (words.next(), words.next()) else {
        return Err(format!("Can't claim '{label}': name an object and its number (Group 3)."));
    };
    let obj = super::objects::find(word).ok_or_else(|| format!("'{word}' is not an object."))?;
    let shown = format!("{} {id}", obj.keyword);
    let id: u32 = id
        .parse()
        .map_err(|_| format!("Can't claim '{label}': '{id}' is not a number."))?;
    let tree = obj
        .tree
        .as_ref()
        .ok_or_else(|| format!("{} can't be claimed.", obj.keyword))?;
    let dp = tree.datapool.as_ref().map(|_| state.selected_datapool());
    match tree.object_in(&state.root, dp, id) {
        Some(super::vfs::NodeView::Object { guid, .. }) => Uuid::parse_str(&guid)
            .map(|g| (g, shown.clone()))
            .map_err(|_| format!("{shown} has no GUID.")),
        _ => Err(format!("{shown} doesn't exist.")),
    }
}

/// Claims on AppState (decisions §4, §21 "6d"): an extension of the state, since the rules read the rights gate above it (decisions §34 15).
pub trait Claims {
    fn claims_of(&self, user: &str) -> Vec<String>;
    fn claims_in_order(&self) -> Vec<(String, String, bool)>;
    fn claim_holder(&self, label: &str) -> Option<&str>;
    fn is_claimed(&self, object: &str) -> bool;
    fn is_claimed_by_me(&self, object: &str) -> bool;
    fn is_admin(&self) -> bool;
    fn rights_of(&self, user: &str) -> Option<UserRights>;
    fn claim(&mut self, object: &str) -> Result<(), String>;
    fn claim_for_editor(&mut self, object: &str) -> Result<(), String>;
    fn claim_as(&mut self, object: &str, editor: bool) -> Result<(), String>;
    fn grant(&mut self, guid: Uuid, label: String, user: String, editor: bool);
    fn drop_claim(&mut self, guid: Uuid);
    fn unclaim(&mut self, object: &str) -> Result<(), String>;
    fn release_claims_of(&mut self, user: &str) -> usize;
    fn release_lost_devices(&mut self, present: &[String]) -> Option<String>;
    fn unclaim_all_mine(&mut self) -> usize;
    fn release_editor_claims(&mut self) -> usize;
    fn release_where(&mut self, pick: impl Fn(&Claim) -> bool) -> usize;
    fn ask_for_claim(&mut self, object: &str, force: bool) -> Result<String, String>;
    fn remember_claim_answer(&mut self, object: &str, hand_over: bool);
    fn keep_claim(&mut self, object: &str) -> Result<String, String>;
    fn expire_claim_asks(&mut self) -> bool;
    fn held_by_other(&self, guid: Uuid) -> Option<(String, String)>;
    fn claim_changed_by(&self, changes: &[super::change::Change]) -> Option<(String, String)>;
    fn notify_user(&mut self, user: &str, text: &str);
}

impl Claims for AppState {
    /// The labels of `user`'s claims, oldest first.
    fn claims_of(&self, user: &str) -> Vec<String> {
        let rt = &self.root.runtime;
        rt.claim_order
            .iter()
            .filter_map(|g| rt.claims.get(g))
            .filter(|c| c.user.eq_ignore_ascii_case(user))
            .map(|c| c.label.clone())
            .collect()
    }

    /// Every claim in order: (label, holder, editor).
    fn claims_in_order(&self) -> Vec<(String, String, bool)> {
        let rt = &self.root.runtime;
        rt.claim_order
            .iter()
            .filter_map(|g| rt.claims.get(g))
            .map(|c| (c.label.clone(), c.user.clone(), c.editor))
            .collect()
    }

    /// Who holds `label` (by GUID), if anyone.
    fn claim_holder(&self, label: &str) -> Option<&str> {
        let (guid, _) = resolve(self, label).ok()?;
        self.root.runtime.claims.get(&guid).map(|c| c.user.as_str())
    }

    /// Check if an object is claimed (by anyone).
    fn is_claimed(&self, object: &str) -> bool {
        self.claim_holder(object).is_some()
    }

    /// Check if an object is claimed by the current user.
    fn is_claimed_by_me(&self, object: &str) -> bool {
        self.claim_holder(object) == Some(self.effective_user_name())
    }

    /// The acting user is at the Admin level (or the internal root user).
    fn is_admin(&self) -> bool {
        crate::rights::at_least(self, crate::commands::registry::Rights::Admin)
    }

    /// The rights level of a visible show user.
    fn rights_of(&self, user: &str) -> Option<UserRights> {
        if user.trim().is_empty() || user.eq_ignore_ascii_case(INTERNAL_ROOT_USER) {
            return None;
        }
        self.root.show_data.rights_of(user)
    }

    /// Claim an object for the current user. Err when another user holds it.
    fn claim(&mut self, object: &str) -> Result<(), String> {
        self.claim_as(object, false)
    }

    /// Claim for an editor (released when the editor closes).
    fn claim_for_editor(&mut self, object: &str) -> Result<(), String> {
        self.claim_as(object, true)
    }

    fn claim_as(&mut self, object: &str, editor: bool) -> Result<(), String> {
        let (guid, label) = resolve(self, object)?;
        let me = self.effective_user_name().to_string();
        if let Some(c) = self.root.runtime.claims.get(&guid) {
            return if c.user == me {
                Ok(())
            } else {
                Err(crate::errors::render_code(
                    format!("{label} is claimed by {}.", c.user),
                    "E-CLAIM-001",
                ))
            };
        }
        self.grant(guid, label, me, editor);
        Ok(())
    }

    /// Give `guid` to `user` (the claims table and its side effects).
    fn grant(&mut self, guid: Uuid, label: String, user: String, editor: bool) {
        // Claiming Devices edits a draft; the rig keeps the live devices.
        if label == "Devices" {
            self.begin_patch_draft();
        }
        self.root.runtime.claims.insert(
            guid,
            Claim {
                meta: lumacore_base::Identity::default(),
                label,
                user,
                since_ms: now_ms(),
                editor,
            },
        );
        self.root.runtime.claim_order.retain(|g| *g != guid);
        self.root.runtime.claim_order.push(guid);
        self.root.runtime.claim_asks.retain(|a| a.object != guid);
    }

    fn drop_claim(&mut self, guid: Uuid) {
        self.root.runtime.claims.remove(&guid);
        self.root.runtime.claim_order.retain(|g| *g != guid);
    }

    /// Unclaim an object. Admins can unclaim anyone's claim. A user asking
    /// for it gets it (the owner allowed the takeover).
    ///
    /// Devices with a changed draft is refused: the draft is kept or discarded
    /// first (`Unclaim Root.ShowData.Devices` asks; decisions §11 "staged patch"). An
    /// unchanged draft just ends.
    fn unclaim(&mut self, object: &str) -> Result<(), String> {
        let (guid, label) = resolve(self, object)?;
        let Some(owner) = self.root.runtime.claims.get(&guid).map(|c| c.user.clone()) else {
            return Err(format!("{label} is not claimed."));
        };
        // Another user's claim is Admin's (an ownership rule; refused with
        // E-RIGHTS-001 and both levels).
        if owner != self.effective_user_name() {
            crate::rights::require(
                self,
                crate::commands::registry::Rights::Admin,
                &format!("Unclaim {label} (held by {owner})"),
            )?;
        }
        if label == "Devices" {
            if self.patch_draft_changed() {
                return Err(
                    "The Devices draft has changes: Unclaim Root.ShowData.Devices to keep or discard them."
                        .to_string(),
                );
            }
            self.forget_patch_draft();
        }
        self.drop_claim(guid);
        if let Some(ask) = self.root.runtime.claim_asks.iter().find(|a| a.object == guid).cloned() {
            self.grant(guid, label.clone(), ask.from.clone(), false);
            self.notify_user(&ask.from, &format!("{owner} handed {label} over to you."));
        }
        Ok(())
    }

    /// Release `user`'s claims (they logged out or switched user).
    fn release_claims_of(&mut self, user: &str) -> usize {
        let user = user.to_string();
        self.release_where(|c| c.user.eq_ignore_ascii_case(&user))
    }

    /// The Devices holder left the session (decisions §34 "34g4 patch entry hardened" 5): no
    /// longer present anywhere (`present`: every user logged in on this station or another of
    /// the LumaSession), their claim is released and the draft discarded — nothing kept. Returns
    /// the holder when it did.
    fn release_lost_devices(&mut self, present: &[String]) -> Option<String> {
        let (guid, holder) =
            self.root.runtime.claims.iter().find(|(_, c)| c.label == "Devices").map(|(g, c)| (*g, c.user.clone()))?;
        if present.iter().any(|u| u.eq_ignore_ascii_case(&holder)) {
            return None;
        }
        let changed = self.patch_draft_changed();
        self.discard_patch_draft();
        self.drop_claim(guid);
        self.sysmon_pending.push((
            "Patch".to_string(),
            format!(
                "{holder} left the session with the patch claimed: {}, the patch is Live.",
                if changed { "the draft was discarded" } else { "the claim was released" }
            ),
        ));
        Some(holder)
    }

    /// Release all claims owned by the effective user.
    fn unclaim_all_mine(&mut self) -> usize {
        let user = self.effective_user_name().to_string();
        self.release_where(|c| c.user == user)
    }

    /// Release the editor claims of the effective user (the editor closed).
    fn release_editor_claims(&mut self) -> usize {
        let user = self.effective_user_name().to_string();
        self.release_where(|c| c.user == user && c.editor)
    }

    fn release_where(&mut self, pick: impl Fn(&Claim) -> bool) -> usize {
        let user = self.effective_user_name().to_string();
        let gone: Vec<(Uuid, String)> = self
            .root
            .runtime
            .claims
            .iter()
            .filter(|(_, c)| pick(c))
            .map(|(g, c)| (*g, c.label.clone()))
            .collect();
        for (guid, label) in &gone {
            // Released without the prompt (logout, user switch): the draft
            // never reached the rig, so it is dropped.
            if label == "Devices" {
                if self.patch_draft_changed() {
                    self.sysmon_pending.push((
                        "Patch".to_string(),
                        format!("[{user}] Patch draft discarded (claim released)"),
                    ));
                }
                self.discard_patch_draft();
            }
            self.drop_claim(*guid);
        }
        gone.len()
    }

    /// `Claim <object>` when another user holds it: ask them (10 s), hand it
    /// over at once when they're logged out, or take it (`/Force`, Admin).
    fn ask_for_claim(&mut self, object: &str, force: bool) -> Result<String, String> {
        let (guid, label) = resolve(self, object)?;
        let me = self.effective_user_name().to_string();
        let Some(owner) = self.root.runtime.claims.get(&guid).map(|c| c.user.clone()) else {
            self.grant(guid, label.clone(), me, false);
            return Ok(format!("Claimed {label}."));
        };
        if owner == me {
            return Ok(format!("You hold {label} already."));
        }
        // `/Force` is declared Admin (`rights::claim_rights`): the gate
        // refused it below Admin.
        if force {
            self.grant(guid, label.clone(), me.clone(), false);
            self.notify_user(&owner, &format!("{me} (Admin) took {label} from you."));
            return Ok(format!("Took {label} from {owner} (/Force)."));
        }
        let owner_online = self
            .root
            .runtime
            .users
            .keys()
            .any(|u| u.eq_ignore_ascii_case(&owner));
        let key = (owner.to_lowercase(), me.to_lowercase());
        if !owner_online {
            self.claim_remember.retain(|(o, _), _| *o != key.0);
            self.grant(guid, label.clone(), me, false);
            return Ok(format!("Claimed {label} ({owner} isn't logged in)."));
        }
        // The owner answered this user "for this session" (decisions §30 "12b").
        match self.claim_remember.get(&key) {
            Some(true) => {
                self.grant(guid, label.clone(), me.clone(), false);
                self.notify_user(&owner, &format!("{me} took {label} (you hand over to {me} this session)."));
                return Ok(format!("Claimed {label} ({owner} hands over to you this session)."));
            }
            Some(false) => {
                return Err(format!("{owner} keeps their objects from you this session ({label})."));
            }
            None => {}
        }
        self.root.runtime.claim_asks.retain(|a| a.object != guid);
        self.root.runtime.claim_asks.push(ClaimAsk {
            object: guid,
            label: label.clone(),
            from: me.clone(),
            owner: owner.clone(),
            deadline_ms: now_ms() + TAKEOVER.as_millis() as u64,
            due_session_ms: lumacore_base::clock::session_ms() + TAKEOVER.as_millis() as u64,
        });
        self.notify_user(
            &owner,
            &format!(
                "{me} wants {label}: Unclaim {label} hands it over, Claim {label} /Keep keeps it \
                 (handed over in {} s otherwise).",
                TAKEOVER.as_secs()
            ),
        );
        Ok(format!(
            "{label} is claimed by {owner}: asked them (handed over in {} s unless they keep it).",
            TAKEOVER.as_secs()
        ))
    }

    /// "Remember for this session" on a claim ask for `object`: later asks
    /// by the same user are answered `hand_over` at once until this user
    /// logs out.
    fn remember_claim_answer(&mut self, object: &str, hand_over: bool) {
        let Ok((guid, _)) = resolve(self, object) else {
            return;
        };
        let me = self.effective_user_name().to_lowercase();
        if let Some(ask) = self.root.runtime.claim_asks.iter().find(|a| a.object == guid) {
            self.claim_remember.insert((me, ask.from.to_lowercase()), hand_over);
        }
    }

    /// `Claim <object> /Keep`: the owner refuses a takeover.
    fn keep_claim(&mut self, object: &str) -> Result<String, String> {
        let (guid, label) = resolve(self, object)?;
        let me = self.effective_user_name().to_string();
        let Some(ask) = self
            .root
            .runtime
            .claim_asks
            .iter()
            .find(|a| a.object == guid && a.owner.eq_ignore_ascii_case(&me))
            .cloned()
        else {
            return Err(format!("Nobody is asking for {label}."));
        };
        self.root.runtime.claim_asks.retain(|a| a.object != guid);
        self.notify_user(&ask.from, &format!("{me} keeps {label}."));
        Ok(format!("You keep {label} ({} was told).", ask.from))
    }

    /// Takeovers nobody answered in time are handed over (master, every
    /// tick). Returns whether anything changed.
    fn expire_claim_asks(&mut self) -> bool {
        // The session clock, not the wall clock (R1-02): continuous across a
        // master handover, never stepped by NTP.
        let now = lumacore_base::clock::session_ms();
        let due: Vec<ClaimAsk> = self
            .root
            .runtime
            .claim_asks
            .iter()
            .filter(|a| a.due_session_ms <= now)
            .cloned()
            .collect();
        for ask in &due {
            if self.root.runtime.claims.get(&ask.object).is_some_and(|c| c.label == "Devices") {
                // A Devices draft is never taken away unasked: it stays.
                self.root.runtime.claim_asks.retain(|a| a.object != ask.object);
                continue;
            }
            self.grant(ask.object, ask.label.clone(), ask.from.clone(), false);
            self.notify_user(
                &ask.owner,
                &format!("{} took {} (no answer in {} s).", ask.from, ask.label, TAKEOVER.as_secs()),
            );
            self.notify_user(&ask.from, &format!("You have {} now.", ask.label));
        }
        !due.is_empty()
    }

    /// The other user holding `guid` or an object it lives in (the walk's
    /// parents), if any: plain commands on it are refused.
    fn held_by_other(&self, guid: Uuid) -> Option<(String, String)> {
        let me = self.effective_user_name();
        let others = |c: &&Claim| !c.user.eq_ignore_ascii_case(me);
        if let Some(c) = self.root.runtime.claims.get(&guid).filter(others) {
            return Some((c.label.clone(), c.user.clone()));
        }
        if !self.root.runtime.claims.values().any(|c| others(&c)) {
            return None;
        }
        let walk = self.root.show_data.walk_identity();
        let parent: std::collections::HashMap<Uuid, Option<Uuid>> =
            walk.objects.iter().map(|o| (o.guid, o.parent)).collect();
        let mut at = parent.get(&guid).copied().flatten();
        while let Some(g) = at {
            if let Some(c) = self.root.runtime.claims.get(&g).filter(others) {
                return Some((c.label.clone(), c.user.clone()));
            }
            at = parent.get(&g).copied().flatten();
        }
        None
    }

    /// The claim (label, owner) of another user whose object a line changed
    /// (R1-44): judged on the outcome, so every line shape counts — Thru,
    /// names, tree paths, owner-first forms, Copy/Move targets, imports, the
    /// object as verb. An object is found in the change path's subtree images
    /// by its GUID (`guid` or `meta.guid`); it changed when its JSON (with
    /// everything inside it) differs or it is gone. Areas (`Claim Root.ShowData.Devices` …)
    /// have no object here: the Devices draft guards Patch (E-PATCH-008).
    fn claim_changed_by(&self, changes: &[super::change::Change]) -> Option<(String, String)> {
        let me = self.effective_user_name();
        let others: Vec<(String, &Claim)> = self
            .root
            .runtime
            .claims
            .iter()
            .filter(|(_, c)| !c.user.eq_ignore_ascii_case(me))
            .map(|(g, c)| (g.to_string(), c))
            .collect();
        if others.is_empty() || changes.is_empty() {
            return None;
        }
        let parse = |bytes: &[u8]| serde_json::from_slice::<serde_json::Value>(bytes).unwrap_or_default();
        for change in changes {
            let mut images: Option<(serde_json::Value, serde_json::Value)> = None;
            for (guid, claim) in &others {
                let inside = |bytes: &[u8]| bytes.windows(guid.len()).any(|w| w == guid.as_bytes());
                if !inside(&change.before) && !inside(&change.after) {
                    continue;
                }
                let (before, after) = images.get_or_insert_with(|| (parse(&change.before), parse(&change.after)));
                if objects_with(before, guid) != objects_with(after, guid) {
                    return Some((claim.label.clone(), claim.user.clone()));
                }
            }
        }
        None
    }

    /// A notice to every session of `user` (Shell output, UI cards).
    fn notify_user(&mut self, user: &str, text: &str) {
        self.sysmon_pending
            .push(("Claims".to_string(), format!("[{user}] {text}")));
        self.claim_notices.push((user.to_string(), text.to_string()));
    }
}
