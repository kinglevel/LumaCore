// ██╗    ██╗ ██████╗ ██████╗ ██╗     ██████╗
// ██║    ██║██╔═══██╗██╔══██╗██║     ██╔══██╗
// ██║ █╗ ██║██║   ██║██████╔╝██║     ██║  ██║
// ██║███╗██║██║   ██║██╔══██╗██║     ██║  ██║
// ╚███╔███╔╝╚██████╔╝██║  ██║███████╗██████╔╝
//  ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═════╝
// FILE: world

//! The sweep's world: a master and an in-process member, no server, no engine threads, no sockets,
//! the disk shut. World::run runs one line as one user and checks what must hold after it.
//!
//! Owns:          the sweep's world (a master + an in-process member), running one line as one user, the checks after it, findings.
//! Doesn't own:   which lines run (lines.rs, run.rs), the change path itself (lumacore-state).
//! Entry points:  World, Finding, Outcome, Expect, ROLES, round_trip.
//! Decisions:     §25 "Step 10 hardening", §21, §34 24.2.

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, Instant};

use lumacore_tree::{ReplicationJournal, ShowSubtree, UserRights};
use serde::{Deserialize, Serialize};

use lumacore_objects::commands::{dispatch, oops, registry::Undo};
use lumacore_objects::props::{PropKind, Property};
use lumacore_objects::{AppState, DispatchResult, change};
use lumacore_luma::drive;
use lumacore_sysmon::panic_report;
use lumacore_lumasession::link::{OpOrder, op_order};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The users lines run as: one per rights level (decisions §30 "12d"),
/// highest first — Admin is the reference the others are compared with —
/// each named after its level, generated from [`UserRights::ALL`]; plus
/// one who holds a claim.
pub const ROLES: [&str; UserRights::ALL.len()] = {
    let mut out = [""; UserRights::ALL.len()];
    let mut i = 0;
    while i < out.len() {
        out[i] = UserRights::ALL[out.len() - 1 - i].label();
        i += 1;
    }
    out
};
/// Holds a claim on the first group, so others' edits of it are refused.
pub const OWNER: &str = "Owner";

/// A line slower than this is a finding (a hang stops the process: the
/// runner's watchdog).
pub const SLOW: Duration = Duration::from_secs(1);

/// What a line must do besides the general checks.
#[derive(Debug, Clone)]
pub enum Expect {
    /// `Set <object> <id> <prop> <raw>`: reads back, or refuses with a code.
    ReadBack { object: &'static str, id: u32, prop: &'static Property, raw: String },
    /// `Set <object> <id> <field> …` on a field that isn't writable: refused
    /// with a code, nothing changed.
    ReadOnly { object: &'static str, id: u32, field: String },
}

/// Something that must not happen. `key` groups repeats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Which check failed (`panic`, `undo`, `read-back`, …).
    pub check: String,
    /// The line's first word (and object), for grouping.
    pub word: String,
    pub detail: String,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Finding {
    /// Repeats of one cause share a key: a panic by where it happened; a
    /// show-wide check (member, save/load) by what differs; the others by
    /// check, first word and the kinds of subtrees involved.
    pub fn key(&self) -> String {
        if self.check == "panic"
            && let Some(at) = self.detail.split_whitespace().find(|w| w.contains(".rs:"))
        {
            return format!("panic · {}", at.trim_end_matches(['—', ',', ':']));
        }
        let kinds = kinds_of(&self.detail);
        let first = self.word.split(' ').next().unwrap_or("");
        // tidy: not-keywords (the sweep's check names)
        match self.check.as_str() {
            "member" | "save-load" | "save-load-order" => format!("{} · {kinds}", self.check),
            "undo" | "redo" | "read-only-changed" | "refused-but-changed" | "identity" => {
                format!("{} · {first} · {kinds}", self.check)
            }
            _ => format!("{} · {first}", self.check),
        }
    }
}

/// A detail with GUIDs and numbers taken out (`Programmer(…)` → `Programmer`).
fn kinds_of(detail: &str) -> String {
    let mut out = String::new();
    let mut chars = detail.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '(' {
            // Drop a parenthesised GUID.
            let inner: String = chars.clone().take_while(|c| *c != ')').collect();
            if inner.len() == 36 && inner.chars().filter(|c| *c == '-').count() == 4 {
                for _ in 0..=inner.len() {
                    chars.next();
                }
                continue;
            }
        }
        if c.is_ascii_digit() {
            out.push('#');
            while chars.peek().is_some_and(char::is_ascii_digit) {
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    let mut parts: Vec<&str> = out.split(", ").collect();
    parts.sort();
    parts.dedup();
    parts.join(", ")
}

/// How a line ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// An `Error:` line's code (`Some(None)`: an error without one).
    pub error: Option<Option<String>>,
    /// The changed subtrees, with the acting user's own named `self`.
    pub changed: Vec<String>,
    /// The line ended the session (Shutdown, Restart, Logout): the world
    /// starts over.
    pub ended: bool,
    /// The line replaced the show (LoadShow, NewShow, CleanStart).
    pub replaced: bool,
}

/// A master, a member and what has been sent between them.
pub struct World {
    pub master: AppState,
    pub member: AppState,
    journal: ReplicationJournal,
    base: std::sync::Arc<String>,
    /// Lines since the last heavy check.
    since_heavy: usize,
    /// Lines run since the world was made (`role\tline`), for replays.
    pub history: Vec<String>,
}

/// Every how many lines the heavy checks run (save/load, member, render).
pub const HEAVY_EVERY: usize = 25;

impl World {
    /// A fresh world from the base show (JSON of a `.luma`).
    pub fn new(base: std::sync::Arc<String>) -> Result<Self, String> {
        crate::require_catalog()?;
        let mut master = AppState::new();
        lumacore_objects::objects::replace_show(&mut master, lumacore_objects::parse_show(&base)?);
        master.clear_all_undo_stacks();
        let users = UserRights::ALL
            .iter()
            .filter(|r| **r != UserRights::Admin)
            .map(|r| (r.label(), *r))
            .chain([(OWNER, UserRights::Program)]);
        for (name, rights) in users {
            as_user(&mut master, "Admin");
            let _ = dispatch(&mut master, &format!("Store User \"{name}\" /Rights {}", rights.label()));
        }
        for user in ROLES.iter().chain([&OWNER]) {
            master.root.runtime.user(user);
        }
        let first_group = master
            .root
            .show_data
            .datapools
            .values()
            .find_map(|d| d.groups.keys().next().copied());
        if let Some(g) = first_group {
            as_user(&mut master, OWNER);
            let _ = dispatch(&mut master, &format!("Claim Group {g}"));
        }
        master.clear_acting_user();
        master.replication_outbox.clear();
        master.undo_log.clear();
        master.show_replaced = false;
        master.clear_all_undo_stacks();

        let journal = ReplicationJournal::new(64);
        let member = member_of(&master, &journal)?;
        Ok(Self {
            master,
            member,
            journal,
            base,
            since_heavy: 0,
            history: Vec::new(),
        })
    }

    /// Start over from the base show.
    pub fn reset(&mut self) -> Result<(), String> {
        *self = Self::new(self.base.clone())?;
        Ok(())
    }

    /// Run `line` as `user` and check it. `expect` adds the line's own check.
    pub fn run(&mut self, user: &str, line: &str, expect: Option<&Expect>) -> (Outcome, Vec<Finding>) {
        let word = word_of(line);
        let mut findings = Vec::new();
        let mut found = |check: &str, detail: String| {
            findings.push(Finding {
                check: check.to_string(),
                word: word.clone(),
                detail,
            })
        };
        self.history.push(format!("{user}\t{line}"));

        as_user(&mut self.master, user);
        let before = prints(&self.master);
        let problems_before = problems(&self.master);
        // The acting user's own subtrees (user, programmer, profile) are
        // named `self`, so roles compare.
        let own: Vec<String> = self
            .master
            .root
            .show_data
            .user(user)
            .map(|u| u.meta.identity.guid.to_string())
            .into_iter()
            .chain(
                self.master
                    .root
                    .show_data
                    .profile_for_user(user)
                    .map(|p| p.meta.identity.guid.to_string()),
            )
            .collect();
        let expected = expect.and_then(|e| expected_value(&mut self.master, e));

        // A Show line finds the door shut; a Drive line finds it open inside
        // the sandbox only (decisions §29).
        let reach = lumacore_objects::commands::registry::line_reach(line);
        drive::set_off(reach == lumacore_objects::commands::registry::Reach::Show);
        let _ = drive::take_refused();
        let filed_before = self.master.undo_sequence;
        let started = Instant::now();
        let result = panic_report::guarded(|| dispatch(&mut self.master, line));
        let elapsed = started.elapsed();
        let mut output = match result {
            Ok(r) => r,
            Err(report) => {
                found("panic", format!("outside the command guard: {}:{} {}", report.file, report.line, report.message));
                DispatchResult::Output((String::new()).into())
            }
        };
        // A question is answered with its primary button: the destructive
        // path runs too.
        if let DispatchResult::Ask(ask) = &output
            && let Some(answer) = ask.primary_line()
        {
            output = panic_report::guarded(|| dispatch(&mut self.master, &answer))
                .unwrap_or(DispatchResult::Output((String::new()).into()));
        }
        // A login waits for the Meow License on the board (decisions 0290): the user consents,
        // so the lines after it run as that user.
        if lumacore_objects::consent::pending(&self.master).is_some() {
            let _ = panic_report::guarded(|| dispatch(&mut self.master, "Consent"));
        }
        let refused = drive::take_refused();
        if !refused.is_empty() {
            if reach == lumacore_objects::commands::registry::Reach::Show {
                found("drive", format!("a Show line reached the disk: {}", refused.join(", ")));
            } else {
                found("drive-escape", format!("left the sandbox: {}", refused.join(", ")));
            }
        }
        if elapsed > SLOW {
            found("slow", format!("took {} ms", elapsed.as_millis()));
        }

        let (text, ended) = match &output {
            DispatchResult::Output(s) => (lumacore_objects::text::strip_ansi(s), false),
            DispatchResult::ContextSwitch(ctx) => {
                self.master.context = ctx.clone();
                (String::new(), false)
            }
            DispatchResult::Exit | DispatchResult::Logout => (String::new(), true),
            DispatchResult::Ask(_) => (String::new(), false),
        };
        // A command's panic comes back as its report (an `Info Error`
        // listing only names the code).
        if text.contains("E-INTERNAL-001") && text.contains("Internal error in ") {
            found("panic", first_line_with(&text, "Internal error in "));
        }
        let error = error_of(&text);
        // Every error reply carries a code (decisions §29), on every line.
        if matches!(error, Some(None)) && expect.is_none() {
            found("uncoded-error", first_line_with(&text, "Error:"));
        }

        let after = prints(&self.master);
        let changed_keys = diff(&before, &after);
        let changed: Vec<String> = changed_keys
            .iter()
            .map(|k| own.iter().fold(k.clone(), |k: String, g: &String| k.replace(g.as_str(), "self")))
            .collect();
        let undoable_changed = changed_keys.iter().any(|k| undoable(&after, &before, k));

        // A refusal changes nothing.
        if let Some(code) = &error
            && code.as_deref().is_some_and(|c| c == "E-RIGHTS-001" || c == "E-CLAIM-001")
            && !changed.is_empty()
        {
            found("refused-but-changed", format!("{} changed {}", code.clone().unwrap_or_default(), changed.join(", ")));
        }

        // Undo: a read-only line changes nothing; a tracked one comes back
        // with Oops and goes again with Redo (decisions §21).
        let undo = lumacore_objects::commands::undo_of_line(line);
        let draft_only = self.master.patch_draft.is_some()
            && changed_keys.iter().all(|k| k == &format!("{:?}", ShowSubtree::Devices));
        // (Oops and Redo are Untracked: they are checked by the lines they undo.)
        // A loaded / new show reaches members as a full snapshot, not undo.
        // Read, not take: `replicate` still needs it to send the snapshot
        // (issue 20 a).
        let replaced = self.master.show_replaced;
        if !ended && undoable_changed && !draft_only && !replaced {
            match undo {
                Undo::ReadOnly => found("read-only-changed", format!("changed {}", changed.join(", "))),
                // No step filed (/NoOops, the profile's Oops filters):
                // the line is simply not undoable (decisions §29 "11c").
                Undo::Tracked(_) if self.master.undo_sequence == filed_before => {}
                Undo::Tracked(_) => self.check_oops(&before, &after, &mut found),
                Undo::Untracked => {}
            }
        }

        if let Some(expect) = expect {
            self.check_expect(expect, expected, &error, &changed, &mut found);
        }

        // Identity: no new duplicate GUIDs or broken links.
        let problems_after = problems(&self.master);
        let new: Vec<&String> = problems_after.iter().filter(|p| !problems_before.contains(*p)).collect();
        if !new.is_empty() {
            found("identity", new.iter().take(5).map(|s| s.as_str()).collect::<Vec<_>>().join("; "));
        }

        self.replicate(&mut found);
        self.since_heavy += 1;
        if self.since_heavy >= HEAVY_EVERY {
            self.since_heavy = 0;
            self.heavy(&mut found);
        }
        drive::set_off(false);
        self.master.clear_acting_user();
        // Background follow-ups (an asset hash's Import line) aren't part of
        // what this line did: dropped, so the queue never grows over a run.
        let _ = lumacore_luma::background::take();

        (Outcome { error, changed, ended, replaced }, findings)
    }

    fn check_oops(
        &mut self,
        before: &BTreeMap<String, u64>,
        after: &BTreeMap<String, u64>,
        found: &mut impl FnMut(&str, String),
    ) {
        let _ = dispatch(&mut self.master, "Oops /NoConfirmation");
        let undone = prints(&self.master);
        let wrong = undoable_diff(before, &undone);
        if !wrong.is_empty() {
            found("undo", format!("Oops left {} different", wrong.join(", ")));
            return;
        }
        let _ = dispatch(&mut self.master, "Redo");
        let redone = prints(&self.master);
        let wrong = undoable_diff(after, &redone);
        if !wrong.is_empty() {
            found("redo", format!("Redo left {} different", wrong.join(", ")));
        }
    }

    fn check_expect(
        &self,
        expect: &Expect,
        expected: Option<serde_json::Value>,
        error: &Option<Option<String>>,
        changed: &[String],
        found: &mut impl FnMut(&str, String),
    ) {
        match (expect, error) {
            (_, Some(None)) => found("uncoded-error", "an error without an E-code".to_string()),
            (Expect::ReadOnly { field, .. }, None) => found("read-only-accepted", format!("Set {field} was accepted")),
            (Expect::ReadOnly { field, .. }, Some(_)) if !changed.is_empty() => {
                found("read-only-changed", format!("refused Set {field} changed {}", changed.join(", ")))
            }
            (Expect::ReadBack { object, id, prop, raw }, None) => {
                let lumacore_objects::props::Access::Field(field) = prop.access else { return };
                let have = field_of(&self.master, object, *id, field);
                let ok = match (&prop.kind, &expected) {
                    (PropKind::Link { .. }, _) => have.is_null() || have.is_string(),
                    (_, Some(want)) => &have == want,
                    (_, None) => false,
                };
                if !ok {
                    found(
                        "read-back",
                        format!("Set {object} {id} {} {raw} accepted, reads back {have}", prop.name),
                    );
                }
            }
            _ => {}
        }
    }

    /// Send what the master filed to the member, strictly in order.
    fn replicate(&mut self, found: &mut impl FnMut(&str, String)) {
        if std::mem::take(&mut self.master.show_replaced) {
            self.master.replication_outbox.clear();
            self.master.undo_log.clear();
            self.journal.restart();
            match member_of(&self.master, &self.journal) {
                Ok(m) => self.member = m,
                Err(e) => found("replication", format!("snapshot to the member failed: {e}")),
            }
            return;
        }
        let mut pending = std::mem::take(&mut self.master.replication_outbox);
        let undo = std::mem::take(&mut self.master.undo_log);
        if pending.is_empty() && undo.is_empty() {
            return;
        }
        if pending.is_empty() {
            pending.push(lumacore_tree::PendingReplicationOp::default());
        }
        if let Some(last) = pending.last_mut() {
            last.undo = undo;
        }
        // Like a real member (`link.rs`: a gap or a failed apply asks for a
        // resync): the first failure is reported, then the member starts
        // over from a snapshot — later ops never cascade (issue 20 b).
        let mut failed = false;
        for p in pending {
            let op = self.journal.commit(p, "sweep", 0);
            if failed {
                continue;
            }
            if op_order(self.member.last_applied, &op) != OpOrder::Next {
                found("replication", format!("op {} out of order", op.seq));
                failed = true;
                continue;
            }
            if let Err(e) = lumacore_lumasession::replication::apply_replicated_op(&mut self.member, &op) {
                found("replication", format!("op {} ({}) failed on the member: {e}", op.seq, op.command));
                failed = true;
            }
        }
        if failed {
            match member_of(&self.master, &self.journal) {
                Ok(m) => self.member = m,
                Err(e) => found("replication", format!("resync snapshot failed: {e}")),
            }
        }
    }

    /// Save → load → save, the member against the master, a render.
    pub fn heavy(&mut self, found: &mut impl FnMut(&str, String)) {
        if let Err((check, e)) = round_trip(&self.master) {
            found(check, e);
        }
        let mut master = prints(&self.master);
        let member = prints(&self.member);
        // An open Patch draft isn't replicated until Keep (decisions §11):
        // the member has the live patch: compare it with the master's live
        // view (`patch_draft::live_view`, issue 20 f), not the draft.
        if self.master.patch_draft.is_some() {
            let devices = format!("{:?}", ShowSubtree::Devices);
            let live = lumacore_objects::patch_draft::live_view(&self.master, self.master.root.show_data.clone());
            let drafted = std::mem::replace(&mut self.master.root.show_data, live);
            let live_prints = prints(&self.master);
            self.master.root.show_data = drafted;
            match live_prints.get(&devices) {
                Some(&p) => master.insert(devices, p),
                None => master.remove(&devices),
            };
        }
        let differ = diff(&master, &member);
        if !differ.is_empty() {
            found("member", format!("member differs from the master in {}", differ.join(", ")));
        }
        if let Err(report) = panic_report::guarded(|| self.master.output_frame()) {
            found("render", format!("{}:{} {}", report.file, report.line, report.message));
        }
    }
}

/// A member: the master's show as a snapshot, ready for the journal's ops.
fn member_of(master: &AppState, journal: &ReplicationJournal) -> Result<AppState, String> {
    let json = serde_json::to_string(&master.root.show_data).map_err(|e| e.to_string())?;
    let mut member = AppState::new();
    lumacore_objects::objects::replace_show(&mut member, serde_json::from_str(&json).map_err(|e| e.to_string())?);
    member.clear_all_undo_stacks();
    member.show_replaced = false;
    member.last_applied = Some((journal.id(), journal.next_seq() - 1));
    Ok(member)
}

/// The real save path (`show_file::encode`, the `.luma` container), loaded
/// back through the real load path (`show_file::from_bytes`) and saved
/// again: the same bytes (decisions §25 property, R1-18).
pub fn round_trip(state: &AppState) -> Result<(), (&'static str, String)> {
    use lumacore_luma::show_file;
    let err = |e: String| ("save-load", e);
    let (saved, _) = show_file::encode(&state.root.show_data).map_err(err)?;
    let mut loaded = AppState::new();
    lumacore_objects::objects::replace_show(&mut loaded, show_file::from_bytes(&saved).map_err(err)?);
    let (again, _) = show_file::encode(&loaded.root.show_data).map_err(err)?;
    if saved != again {
        return Err(container_diff(&saved, &again));
    }
    let dups = loaded.root.show_data.identity();
    if !dups.duplicates.is_empty() || !dups.nil.is_empty() {
        return Err(err(format!(
            "the loaded show has {} duplicate and {} nil GUIDs",
            dups.duplicates.len(),
            dups.nil.len()
        )));
    }
    Ok(())
}

/// Where two `.luma` containers differ: the first entry only one has, or the
/// first entry whose bytes differ (a JSON entry with its path inside).
fn container_diff(a: &[u8], b: &[u8]) -> (&'static str, String) {
    use std::io::Read;
    fn entries(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        (0..zip.len())
            .map(|i| {
                let mut e = zip.by_index(i).map_err(|e| e.to_string())?;
                let mut data = Vec::new();
                e.read_to_end(&mut data).map_err(|e| e.to_string())?;
                Ok((e.name().to_string(), data))
            })
            .collect()
    }
    // The manifest only lists the other entries' hashes: name the entry.
    let manifest_last = |mut v: Vec<(String, Vec<u8>)>| {
        v.sort_by_key(|(n, _)| n == "manifest.json");
        v
    };
    let (a, b) = match (entries(a), entries(b)) {
        (Ok(a), Ok(b)) => (manifest_last(a), manifest_last(b)),
        (Err(e), _) | (_, Err(e)) => return ("save-load", format!("container unreadable: {e}")),
    };
    for (name, x) in &a {
        let Some((_, y)) = b.iter().find(|(n, _)| n == name) else {
            return ("save-load", format!("{name} (only before)"));
        };
        if x == y {
            continue;
        }
        let (Ok(xv), Ok(yv)) = (
            serde_json::from_slice::<serde_json::Value>(x),
            serde_json::from_slice::<serde_json::Value>(y),
        ) else {
            return ("save-load", format!("{name}: bytes differ"));
        };
        if xv == yv {
            let (xs, ys) = (String::from_utf8_lossy(x), String::from_utf8_lossy(y));
            return ("save-load-order", format!("same content, other order in {name} at {}", order_diff(&xs, &ys)));
        }
        return ("save-load", format!("content differs in {name} at {}", first_difference(&xv, &yv, String::new())));
    }
    match b.iter().find(|(n, _)| !a.iter().any(|(m, _)| m == n)) {
        Some((name, _)) => ("save-load", format!("{name} (only after)")),
        None => ("save-load-order", "same entries, other order".into()),
    }
}

/// The JSON path of the first difference (numbers and GUIDs in keys kept).
fn first_difference(a: &serde_json::Value, b: &serde_json::Value, path: String) -> String {
    use serde_json::Value;
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in x {
                match y.get(k) {
                    Some(w) if w == v => {}
                    Some(w) => return first_difference(v, w, format!("{path}/{k}")),
                    None => return format!("{path}/{k} (only before)"),
                }
            }
            match y.keys().find(|k| !x.contains_key(*k)) {
                Some(k) => format!("{path}/{k} (only after)"),
                None => path,
            }
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                if v != w {
                    return first_difference(v, w, format!("{path}/{i}"));
                }
            }
            path
        }
        _ => path,
    }
}

/// Where two texts of the same content first differ: the enclosing keys.
/// The first line that differs, as text (issue 20 h: a guessed path was
/// often wrong).
fn order_diff(a: &str, b: &str) -> String {
    match a.lines().zip(b.lines()).enumerate().find(|(_, (x, y))| x != y) {
        Some((n, (x, y))) => format!("line {}: `{}` vs `{}`", n + 1, x.trim(), y.trim()),
        None => format!("lengths {} vs {} lines", a.lines().count(), b.lines().count()),
    }
}

fn as_user(state: &mut AppState, user: &str) {
    state.acting_user = Some(user.to_string());
    state.root.runtime.user(user);
}

/// A fingerprint per show subtree (its undo/replication snapshot, hashed).
pub fn prints(state: &AppState) -> BTreeMap<String, u64> {
    change::subtrees(&state.root.show_data)
        .into_iter()
        .map(|s| {
            // By content, not bytes: a map that serializes in another order
            // is the same subtree (that order is its own finding, save-load).
            let value: serde_json::Value = oops::snapshot_subtree(state, &s)
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default();
            let mut h = DefaultHasher::new();
            value.to_string().hash(&mut h);
            (format!("{s:?}"), h.finish())
        })
        .collect()
}

fn diff(a: &BTreeMap<String, u64>, b: &BTreeMap<String, u64>) -> Vec<String> {
    let mut out: Vec<String> = a.iter().filter(|(k, v)| b.get(*k) != Some(v)).map(|(k, _)| k.clone()).collect();
    out.extend(b.keys().filter(|k| !a.contains_key(*k)).cloned());
    out
}

fn is_undoable_key(k: &str) -> bool {
    // Usage(…) is per user and never undone (decisions §30 "12b2").
    k != format!("{:?}", ShowSubtree::Playbacks)
        && k != format!("{:?}", ShowSubtree::Session)
        && !k.starts_with("Usage(")
}

fn undoable(_: &BTreeMap<String, u64>, _: &BTreeMap<String, u64>, k: &str) -> bool {
    is_undoable_key(k)
}

fn undoable_diff(a: &BTreeMap<String, u64>, b: &BTreeMap<String, u64>) -> Vec<String> {
    diff(a, b).into_iter().filter(|k| is_undoable_key(k)).collect()
}

/// Identity problems the sweep counts. Broken links are not among them: a
/// deleted object's links are kept by design and work again when it comes
/// back (decisions §22, §29) — duplicates, nil GUIDs, wrong kinds and loops
/// still count.
fn problems(state: &AppState) -> Vec<String> {
    let mut report = state.root.show_data.identity();
    report.broken.clear();
    lumacore_objects::references::problems(&report)
}

/// The line's first word, lowercase, with the object it names.
pub fn word_of(line: &str) -> String {
    let words: Vec<String> = lumacore_objects::commands::tokens::lex_lenient(line)
        .iter()
        .take(2)
        .map(|t| t.text.trim_matches('"').to_ascii_lowercase())
        .collect();
    match words.as_slice() {
        [first, second, ..] if lumacore_objects::objects::find(second).is_some() => format!("{first} {second}"),
        [first, ..] => first.clone(),
        [] => String::new(),
    }
}

/// The `Error:` line's code: `None` = no error, `Some(None)` = no code.
pub fn error_of(text: &str) -> Option<Option<String>> {
    let line = text.lines().find(|l| l.trim_start().starts_with("Error:"))?;
    Some(
        line.rsplit_once('[')
            .and_then(|(_, rest)| rest.trim_end().strip_suffix(']'))
            .filter(|c| c.starts_with("E-"))
            .map(str::to_string),
    )
}

fn first_line_with(text: &str, needle: &str) -> String {
    text.lines().find(|l| l.contains(needle)).unwrap_or(text).trim().to_string()
}

/// What a property reads back as after `Set` (None when it can't be told).
fn expected_value(state: &mut AppState, expect: &Expect) -> Option<serde_json::Value> {
    match expect {
        // A relative value (`+ 1`, `* 2`, `- 3` with a space) on a number
        // reads back as the result, not as typed (issue 20 e): not checked
        // here. On text it is literal text, checked like any value.
        Expect::ReadBack { prop, raw, .. }
            if matches!(prop.kind, PropKind::Int { .. }) && lumacore_objects::props::relative(raw, 0).is_some() =>
        {
            None
        }
        Expect::ReadBack { prop, raw, .. } if !matches!(prop.kind, PropKind::Link { .. }) => {
            lumacore_objects::props::parse(prop.kind, raw, &mut state.root).ok()
        }
        _ => None,
    }
}

/// A field of an object, by its JSON path (missing = null).
fn field_of(state: &AppState, object: &str, id: u32, field: &str) -> serde_json::Value {
    let Some(tree) = lumacore_objects::objects::find(object).and_then(|o| o.tree.as_ref()) else {
        return serde_json::Value::Null;
    };
    let Some(lumacore_objects::vfs::NodeView::Object { json, .. }) = (tree.object)(&state.root, id) else {
        return serde_json::Value::Null;
    };
    field
        .split('.')
        .try_fold(&json, |v, k| v.get(k))
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}
