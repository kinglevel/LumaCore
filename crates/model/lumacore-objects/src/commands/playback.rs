//! Playback commands (decisions §27, step 4 skeleton): Store Cue with
//! tracking, Go, Go-, Goto, Off/On Sequence, and the grand master. Without
//! `Sequence <n>` they act on the user's selected sequence (decisions §29).
//!
//! Running state lives in `ShowData.Playbacks` (saved, resumed on load,
//! replicated, never undone — decisions §13). The render engine renders
//! it; nothing here touches DMX.

use lumacore_tree::{
    CueObject, CueValue, SequenceObject, SequenceState, cue_key, cue_label, next_stamp,
};

use crate::AppState;
use crate::errors::render_code;
use crate::objects::{self, ObjectDef};
use crate::text::render_success;

/// `Extract Sequence …` and `Update Sequence … Cue …` (decisions 0293).
pub mod extract;
pub mod update;

/// The sequence a command acts on, as (datapool, number): the one it names (in the user's
/// selected DataPool, or the line's owner — `DataPool 2 Sequence 3`; decisions §30 "12f"), else
/// the user's selected sequence (decisions §29, in whichever pool it is); none → E-PLAY-003
/// (never Sequence 1).
fn sequence_of(state: &AppState, target: &Target) -> Result<(u32, u32), String> {
    target.sequence.map(|n| (state.selected_datapool(), n)).or_else(|| state.selected_sequence()).ok_or_else(|| {
        render_code(
            "No sequence selected — Select Sequence <n>, or name it (Sequence <n>).",
            "E-PLAY-003",
        )
    })
}

/// `Sequence <n>` and `Cue <n>` in any order (`Cue 3 Sequence 2`,
/// `Sequence 2 Cue 3`). A leading bare number belongs to `first`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Target {
    pub sequence: Option<u32>,
    pub cue: Option<String>,
}

/// The OffCue's key in the commands (it is not in the cue map: `SequenceObject::off_cue`).
pub const OFF_CUE: u32 = u32::MAX;

/// A cue as typed: `0` / `CueZero` = CueZero, `OffCue`, else a number (decisions §32: every
/// sequence has both, fixed).
pub fn cue_ref(text: &str) -> Option<u32> {
    let t = text.trim_matches('"');
    if t == "0" || t.eq_ignore_ascii_case("cuezero") {
        Some(lumacore_tree::CUE_ZERO)
    } else if t.eq_ignore_ascii_case("offcue") {
        Some(OFF_CUE)
    } else {
        cue_key(t)
    }
}

/// How a cue key reads (`CueZero`, `OffCue`, `1.5`).
pub fn cue_name(key: u32) -> String {
    match key {
        lumacore_tree::CUE_ZERO => "CueZero".to_string(),
        OFF_CUE => "OffCue".to_string(),
        k => cue_label(k),
    }
}

/// The cue at `key`, CueZero and the OffCue included.
fn cue_mut(seq: &mut SequenceObject, key: u32) -> Option<&mut CueObject> {
    match key {
        lumacore_tree::CUE_ZERO => Some(&mut seq.cue_zero),
        OFF_CUE => Some(&mut seq.off_cue),
        k => seq.cues.get_mut(&k),
    }
}

fn is(word: &str, def: &ObjectDef) -> bool {
    objects::find(word).is_some_and(|o| std::ptr::eq(o, def))
}

pub fn parse_target(args: &[&str], first: &ObjectDef) -> Result<Target, String> {
    let mut t = Target::default();
    let mut i = 0;
    while i < args.len() {
        // A leading number (or, for a cue, CueZero / OffCue) belongs to `first`.
        let leading = args[i].parse::<f64>().is_ok() || (std::ptr::eq(first, &objects::CUE) && cue_ref(args[i]).is_some());
        let (def, value) = if leading && i == 0 {
            (first, args[i])
        } else {
            let Some(value) = args.get(i + 1) else {
                return Err(render_code(
                    format!("A number after '{}'?", args[i]),
                    "E-CMD-006",
                ));
            };
            let def = if is(args[i], &objects::SEQUENCE) {
                &objects::SEQUENCE
            } else if is(args[i], &objects::CUE) {
                &objects::CUE
            } else {
                return Err(render_code(
                    format!("Expected Sequence <n> or Cue <n>, found '{}'.", args[i]),
                    "E-CMD-006",
                ));
            };
            i += 1;
            (def, *value)
        };
        if std::ptr::eq(def, &objects::SEQUENCE) {
            t.sequence = Some(value.parse().map_err(|_| {
                render_code(format!("'{value}' is not a sequence number."), "E-CMD-008")
            })?);
        } else {
            cue_ref(value).ok_or_else(|| {
                render_code(format!("'{value}' is not a cue number (or 0 / OffCue)."), "E-CMD-008")
            })?;
            t.cue = Some(value.to_string());
        }
        i += 1;
    }
    Ok(t)
}

fn no_such(what: String) -> String {
    render_code(what, "E-PLAY-001")
}

/// `Store Sequence [<n>] ["name"]`: an empty sequence (CueZero + OffCue), or the next cue of an
/// existing one; none = a new one at the first free number.
pub fn store_sequence_at(state: &mut AppState, seq: Option<u32>, name: Option<String>) -> String {
    let dp = state.selected_datapool();
    // A bare `Store Sequence`: a new one at the first free number (decisions §34 13, §35 8).
    let seq_id = seq.unwrap_or_else(|| next_sequence_no(state, dp));
    let Some(pool) = state.root.show_data.datapools.get_mut(&dp) else {
        return render_code(format!("DataPool {dp} doesn't exist."), "E-CMD-007");
    };
    let existed = pool.sequences.contains_key(&seq_id);
    let seq = std::sync::Arc::make_mut(pool.sequences.entry(seq_id).or_insert_with(|| {
        std::sync::Arc::new(SequenceObject {
            meta: lumacore_tree::node::NodeMeta::new(None),
            ..SequenceObject::default()
        })
    }));
    if let Some(n) = &name {
        seq.meta.identity.name = Some(n.clone());
    }
    let next = seq.cues.keys().next_back().map_or(1000, |k| k + 1000);
    let shown = seq.meta.identity.name.as_ref().map(|n| format!(" \"{n}\"")).unwrap_or_default();
    let has_values = state.current_programmer().is_some_and(|p| p.values.values().any(|v| v.active));
    if has_values {
        let out = store_cue(state, Target { sequence: Some(seq_id), cue: Some(cue_label(next)) }, StoreMode::Merge, None);
        return if existed { out } else { format!("{}\n{out}", render_success(&format!("Sequence {seq_id}{shown} created."))) };
    }
    match (existed, name.is_some()) {
        (false, _) => render_success(&format!("Sequence {seq_id}{shown} created — empty (CueZero, OffCue).")),
        (true, true) => render_success(&format!("Sequence {seq_id} labeled{shown}; the programmer is empty, no cue stored.")),
        (true, false) => render_code(format!("Sequence {seq_id} exists and the programmer is empty — nothing stored."), "E-CMD-009"),
    }
}

/// What storing over an existing cue does (decisions §30 "Store Cue"): the store ask —
/// its answers are the flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreMode {
    Ask,
    Merge,
    Overwrite,
    Remove,
    SecondCue,
}

impl StoreMode {
    pub fn of(args: &[&str]) -> Self {
        use crate::commands::flags as F;
        if F::has(args, &F::OVERWRITE) {
            Self::Overwrite
        } else if F::has(args, &F::MERGE) {
            Self::Merge
        } else if F::has(args, &F::REMOVE) {
            Self::Remove
        } else if F::has(args, &F::CREATE_SECOND_CUE) {
            Self::SecondCue
        } else {
            Self::Ask
        }
    }
}

/// The first free sequence number of DataPool `dp` (a bare `Store Sequence`, a Store Cue with none
/// selected; decisions §34 13 — cues append instead, §35 8).
pub fn next_sequence_no(state: &AppState, dp: u32) -> u32 {
    let taken: Vec<u32> = state.root.show_data.datapools.get(&dp).map(|d| d.sequences.keys().copied().collect()).unwrap_or_default();
    crate::pools::door::next_free(&taken)
}

/// Sequence `id` of DataPool `dp`, created (CueZero + OffCue, unnamed) when missing; true = new.
fn ensure_sequence(state: &mut AppState, dp: u32, id: u32) -> Result<bool, String> {
    let Some(pool) = state.root.show_data.datapools.get_mut(&dp) else {
        return Err(render_code(format!("DataPool {dp} doesn't exist."), "E-CMD-007"));
    };
    let created = !pool.sequences.contains_key(&id);
    pool.sequences.entry(id).or_insert_with(|| {
        std::sync::Arc::new(SequenceObject {
            meta: lumacore_tree::node::NodeMeta::new(None),
            ..SequenceObject::default()
        })
    });
    Ok(created)
}

/// The cue a bare `Store Cue` makes: the last cue + 1 (2.5 → 3.5), Cue 1 in an empty sequence;
/// always before OffCue.
fn next_cue_key(seq: &SequenceObject) -> u32 {
    seq.cues.keys().next_back().map_or(1000, |k| k + 1000)
}

/// Create Second Cue's number: the next free point number after `key`, before the next cue
/// (2 → 2.1, then 2.2 …; 2.01 … when the tenths are taken). None = no room.
fn second_key(seq: &SequenceObject, key: u32) -> Option<u32> {
    let next = seq.cues.range(key + 1..).next().map_or(u32::MAX, |(k, _)| *k);
    for step in [100, 10, 1] {
        let mut c = key - key % step + step;
        while c < next && seq.cues.contains_key(&c) {
            c += step;
        }
        if c < next {
            return Some(c);
        }
    }
    None
}

/// `Store [Cue] [<n>] [Sequence <m>] ["name"] [/Merge | /Overwrite | /Remove | /CreateSecondCue]`
/// — also bare `Store` and `Store <n>` (Cue is Store's default object).
pub fn store_cue_line(state: &mut AppState, args: &[&str]) -> String {
    let name = args
        .iter()
        .find(|a| a.starts_with('"'))
        .map(|a| crate::commands::tokens::unquote(a).to_string())
        .filter(|n| !n.is_empty());
    let rest: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('"') && !a.starts_with('/')).collect();
    match parse_target(&rest, &objects::CUE) {
        Ok(t) => store_cue(state, t, StoreMode::of(args), name),
        Err(e) => e,
    }
}

/// `Store Cue` (decisions §30 "Store Cue"): the acting user's active programmer values go
/// into part 0 — only what changes from the tracked state before the cue (equal values are kept
/// as blocked values, decisions §27; with tracking off every value is stored). Without a number:
/// the next cue. An empty programmer makes an empty cue. Over an existing cue it asks (Merge /
/// Overwrite / Remove / Create Second Cue; empty programmer: Merge / Overwrite) unless a flag
/// answers. No sequence selected: a new one at the first free number, selected.
pub fn store_cue(state: &mut AppState, target: Target, mode: StoreMode, name: Option<String>) -> String {
    let mut notes: Vec<String> = Vec::new();
    let (dp, seq_id) = match target.sequence {
        Some(n) => (state.selected_datapool(), n),
        None => match state.selected_sequence() {
            Some(at) => at,
            None => {
                let dp = state.selected_datapool();
                let n = next_sequence_no(state, dp);
                if let Err(e) = ensure_sequence(state, dp, n) {
                    return e;
                }
                let guid = seq_guid(state, dp, n);
                let user = state.effective_user_name().to_string();
                if let Some(u) = state.root.show_data.user_mut(&user) {
                    u.selected.sequence = Some(guid);
                }
                notes.push(render_success(&format!("No sequence was selected: Sequence {n} created and selected.")));
                (dp, n)
            }
        },
    };
    let with_notes = |notes: Vec<String>, msg: String| if notes.is_empty() { msg } else { format!("{}\n{msg}", notes.join("\n")) };
    let key = match &target.cue {
        Some(c) => cue_ref(c).expect("checked by parse_target"),
        None => state
            .root
            .show_data
            .datapools
            .get(&dp)
            .and_then(|d| d.sequences.get(&seq_id))
            .map_or(1000, |s| next_cue_key(s)),
    };
    if key == OFF_CUE {
        return render_code(
            "The OffCue is always last and holds only the Off timing (Set Cue \"OffCue\" \"Fade\" 2); it stores no values.",
            "E-CMD-010",
        );
    }
    let Some(prog) = state.current_programmer() else {
        return render_code("No programmer available.", "E-CMD-011");
    };
    let mut active: Vec<CueValue> = prog
        .values
        .iter()
        .filter(|(_, pv)| pv.active)
        .map(|((fixture, attr), pv)| CueValue {
            fixture_id: *fixture,
            attribute: attr.clone(),
            timing: pv.timing,
            // A value that came from a preset is stored as the link only (decisions 0293).
            value: lumacore_tree::CueData::from_programmer(&pv.value),
        })
        .collect();
    active.sort_by(|a, b| (a.fixture_id, &a.attribute).cmp(&(b.fixture_id, &b.attribute)));

    match ensure_sequence(state, dp, seq_id) {
        Ok(true) if notes.is_empty() => notes.push(render_success(&format!("Sequence {seq_id} created."))),
        Ok(_) => {}
        Err(e) => return e,
    }
    // A sequence outside your DataPool is named owner first in the ask's lines.
    let owner = if dp == crate::pools::user_datapool(&state.root.show_data, state.effective_user_name()) {
        String::new()
    } else {
        format!("DataPool {dp} ")
    };
    let seq = std::sync::Arc::make_mut(
        state.root.show_data.datapools.get_mut(&dp).and_then(|d| d.sequences.get_mut(&seq_id)).expect("ensured above"),
    );
    let mut key = key;
    let mut mode = mode;
    // Only a cue holding values asks (an empty one — CueZero of a new sequence — just takes them).
    let exists = if key == lumacore_tree::CUE_ZERO {
        seq.cue_zero.parts.get(&0).is_some_and(|p| !p.values.is_empty())
    } else {
        seq.cues.get(&key).is_some_and(|c| c.values().next().is_some())
    };
    let shown = |k: u32| if k == lumacore_tree::CUE_ZERO { "CueZero".to_string() } else { format!("Cue {}", cue_label(k)) };
    if exists {
        match mode {
            StoreMode::Ask => {
                let label = if key == lumacore_tree::CUE_ZERO { "0".to_string() } else { cue_label(key) };
                let named = name.as_ref().map(|n| format!(" \"{n}\"")).unwrap_or_default();
                let base = format!("Store {owner}Sequence {seq_id} Cue {label}{named}");
                let cue = if key == lumacore_tree::CUE_ZERO { &seq.cue_zero } else { &seq.cues[&key] };
                let has = cue.values().count();
                let yours = if active.is_empty() { "your programmer is empty".to_string() } else { format!("your programmer has {}", active.len()) };
                let mut ask = crate::dialogs::Ask::new(
                    format!("Store {}", shown(key)),
                    format!("{} in Sequence {seq_id} has {has} value{}; {yours}.", shown(key), if has == 1 { "" } else { "s" }),
                )
                .button("Merge", lumacore_wire::ButtonKind::Primary, format!("{base} /Merge"))
                .button("Overwrite", lumacore_wire::ButtonKind::Danger, format!("{base} /Overwrite"));
                let flags = if active.is_empty() {
                    "/Merge or /Overwrite"
                } else {
                    ask = ask
                        .button("Remove", lumacore_wire::ButtonKind::Plain, format!("{base} /Remove"))
                        .button("Create Second Cue", lumacore_wire::ButtonKind::Plain, format!("{base} /CreateSecondCue"));
                    "/Merge, /Overwrite, /Remove or /CreateSecondCue"
                };
                state.asking = Some(ask.cancel("Cancel").flags(flags));
                return with_notes(notes, render_code(format!("{} exists. Store it with {flags}.", shown(key)), "E-PLAY-004"));
            }
            StoreMode::SecondCue => {
                let Some(k) = second_key(seq, key) else {
                    return render_code(format!("No free cue number between {} and the next cue.", shown(key)), "E-CMD-010");
                };
                key = k;
                mode = StoreMode::Merge;
            }
            StoreMode::Remove => {
                if active.is_empty() {
                    return render_code(format!("{}: the programmer is empty — nothing to remove.", shown(key)), "E-CMD-009");
                }
                let cue = if key == lumacore_tree::CUE_ZERO { &mut seq.cue_zero } else { seq.cues.get_mut(&key).expect("exists") };
                let mut removed = 0;
                for part in cue.parts.values_mut() {
                    let before = part.values.len();
                    part.values.retain(|old| !active.iter().any(|v| v.fixture_id == old.fixture_id && v.attribute == old.attribute));
                    removed += before - part.values.len();
                }
                fill_cue_zero(&mut state.root.show_data, dp, seq_id);
                return with_notes(notes, render_success(&format!("{} in Sequence {seq_id}: {removed} value{} removed.", shown(key), if removed == 1 { "" } else { "s" })));
            }
            StoreMode::Merge | StoreMode::Overwrite => {}
        }
    }
    let existed = key == lumacore_tree::CUE_ZERO || seq.cues.contains_key(&key);

    // What already tracks into this cue. A value equal to it is still
    // stored — a **blocked** value: a later change in an earlier cue
    // doesn't track through it (R1-31, Q16 b). With tracking off nothing
    // tracks in. A blocked value is told apart by comparing, not a flag.
    // Nothing tracks into CueZero: it is the start.
    // Links compare as links: a cue linking the preset that already tracks in is blocked.
    let tracked: Vec<(uuid::Uuid, String, lumacore_tree::CueData)> = if seq.settings.tracking && key != lumacore_tree::CUE_ZERO {
        seq.previous_cue(key)
            .or(Some(lumacore_tree::CUE_ZERO))
            .map(|prev| {
                seq.tracked(prev)
                    .into_iter()
                    .map(|((f, a), t)| (f, a.to_string(), t.value.clone()))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let blocked = active
        .iter()
        .filter(|v| tracked.iter().any(|(f, a, val)| *f == v.fixture_id && *a == v.attribute && *val == v.value))
        .count();
    let changes = active;
    let cue = if key == lumacore_tree::CUE_ZERO {
        &mut seq.cue_zero
    } else {
        seq.cues.entry(key).or_insert_with(|| CueObject::new(format!("Cue {}", cue_label(key))))
    };
    if let Some(n) = &name {
        cue.meta.identity.name = Some(n.clone());
    }
    let overwrite = existed && mode == StoreMode::Overwrite;
    // Overwrite: the cue becomes exactly the programmer. Else a stored value replaces the cue's
    // old one in every part.
    for part in cue.parts.values_mut() {
        if overwrite {
            part.values.clear();
        } else {
            part.values.retain(|old| !changes.iter().any(|v| old.fixture_id == v.fixture_id && old.attribute == v.attribute));
        }
    }
    let stored = changes.len();
    cue.part0().values.extend(changes);
    // CueZero's defaults follow what the sequence now uses (decisions §30 "12f echoes").
    fill_cue_zero(&mut state.root.show_data, dp, seq_id);
    let what = match (existed, overwrite) {
        (false, _) => "Stored",
        (true, true) => "Overwrote",
        (true, false) => "Merged into",
    };
    let mut msg = if stored == 0 && !existed {
        format!("Stored {} in Sequence {seq_id} — empty (the programmer is empty)", shown(key))
    } else {
        format!("{what} {} in Sequence {seq_id}: {stored} value{}", shown(key), if stored == 1 { "" } else { "s" })
    };
    if blocked > 0 {
        msg.push_str(&format!(" ({blocked} blocked: equal to what tracks in)"));
    }
    with_notes(notes, render_success(&msg))
}

fn sequence(state: &AppState, dp: u32, id: u32) -> Result<&SequenceObject, String> {
    state
        .root
        .show_data
        .datapools
        .get(&dp)
        .and_then(|d| d.sequences.get(&id))
        .map(|s| s.as_ref())
        .ok_or_else(|| no_such(format!("Sequence {id} doesn't exist.")))
}

/// The GUID of sequence `id` of DataPool `dp` (running state is keyed by it).
fn seq_guid(state: &AppState, dp: u32, id: u32) -> uuid::Uuid {
    state
        .root
        .show_data
        .datapools
        .get(&dp)
        .and_then(|d| d.sequences.get(&id))
        .map_or_else(uuid::Uuid::nil, |s| s.meta.identity.guid)
}

fn started(state: &mut AppState, dp: u32, seq_id: u32, cue: u32) -> String {
    let user = state
        .root
        .show_data
        .user(state.effective_user_name())
        .map_or_else(uuid::Uuid::nil, |u| u.meta.identity.guid);
    let guid = seq_guid(state, dp, seq_id);
    state
        .root
        .show_data
        .playbacks
        .live.sequences
        .insert(guid, SequenceState::asserted_at(cue, user, next_stamp()));
    render_success(&format!("Sequence {seq_id} → Cue {}", cue_label(cue)))
}

/// `Go [Sequence <n>]`: the next cue (the first when off).
pub fn go(state: &mut AppState, target: Target) -> String {
    // A cue number isn't silently ignored (R1-30, Q15): Goto goes to a cue.
    if let Some(cue) = &target.cue {
        return render_code(format!("Go doesn't take a cue: use Goto Cue {cue}."), "E-CMD-006");
    }
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    let seq = match sequence(state, dp, seq_id) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let current = state
        .root
        .show_data
        .playbacks
        .live.sequences
        .get(&seq_guid(state, dp, seq_id))
        .map(|s| s.current_cue);
    let Some(next) = seq.next_cue(current) else {
        return match current {
            Some(c) => render_success(&format!(
                "Sequence {seq_id} is at its last cue ({})",
                cue_label(c)
            )),
            None => no_such(format!("Sequence {seq_id} has no cues.")),
        };
    };
    let guid = seq_guid(state, dp, seq_id);
    match state.root.show_data.playbacks.live.sequences.get_mut(&guid) {
        Some(st) => {
            st.go_to(next, next_stamp());
            render_success(&format!("Sequence {seq_id} → Cue {}", cue_label(next)))
        }
        None => started(state, dp, seq_id, next),
    }
}

/// `Go- [Sequence <n>]`: the previous cue (asserted again).
pub fn go_back(state: &mut AppState, target: Target) -> String {
    // A cue number isn't silently ignored (R1-30, Q15): Goto goes to a cue.
    if let Some(cue) = &target.cue {
        return render_code(format!("Go- doesn't take a cue: use Goto Cue {cue}."), "E-CMD-006");
    }
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    let seq = match sequence(state, dp, seq_id) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let guid = seq_guid(state, dp, seq_id);
    let Some(st) = state.root.show_data.playbacks.live.sequences.get(&guid) else {
        return render_code(format!("Sequence {seq_id} isn't running."), "E-CMD-010");
    };
    let Some(prev) = seq.previous_cue(st.current_cue) else {
        return render_success(&format!(
            "Sequence {seq_id} is at its first cue ({})",
            cue_label(st.current_cue)
        ));
    };
    if let Some(st) = state.root.show_data.playbacks.live.sequences.get_mut(&guid) {
        st.go_to(prev, next_stamp());
    }
    render_success(&format!("Sequence {seq_id} → Cue {}", cue_label(prev)))
}

/// `Goto Cue <n> [Sequence <m>]`: jump there; every tracked value is
/// asserted (decisions §27).
/// A typed Store's answer to an existing cue (`/Merge` …), else it asks.
fn store_mode(merge: bool, overwrite: bool, remove: bool, second: bool) -> StoreMode {
    match (merge, overwrite, remove, second) {
        (_, true, _, _) => StoreMode::Overwrite,
        (true, _, _, _) => StoreMode::Merge,
        (_, _, true, _) => StoreMode::Remove,
        (_, _, _, true) => StoreMode::SecondCue,
        _ => StoreMode::Ask,
    }
}

/// `Store [DataPool <d>] [Sequence <n>] Cue [<m>] ["name"]`: the slot by the resolver's Store
/// rule under its owners (decisions §33 3d) — no Sequence = your selected one, no number = the
/// next cue.
fn store_cue_typed(state: &mut AppState, target: Option<&lumacore_syntax::ast::Target>, name: Option<String>, mode: StoreMode) -> String {
    use crate::resolve::Slot;
    let (slot, datapool, owners) = match crate::resolve::slot_owned(state, target, &objects::CUE) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let sequence = match owners.iter().find(|(o, _)| std::ptr::eq(*o, &objects::SEQUENCE)).map(|(_, k)| k.parse::<u32>()) {
        Some(Ok(n)) => Some(n),
        Some(Err(_)) => return render_code("Name the sequence by number.", "E-CMD-008"),
        None => None,
    };
    let cue = match slot {
        Slot::Key(k) => Some(k),
        Slot::Number(n) => Some(n.to_string()),
        Slot::Next => None,
        // CueZero / OffCue by name (`Store Sequence 1 Cue "OffCue"` is refused by store_cue).
        Slot::NewName(n) if cue_ref(&n).is_some() => Some(n),
        Slot::NewName(n) => return render_code(format!("A cue is named by its number (Store Cue 3 \"{n}\")."), "E-CMD-008"),
    };
    if let Some(c) = &cue
        && cue_ref(c).is_none()
    {
        return render_code(format!("'{c}' isn't a cue number."), "E-CMD-008");
    }
    let pool = datapool.or(state.datapool_override);
    let before = std::mem::replace(&mut state.datapool_override, pool);
    let out = store_cue(state, Target { sequence, cue }, mode, name);
    state.datapool_override = before;
    out
}

/// `Store [Sequence <n>] Cue [<m>] ["name"] [/Merge | /Overwrite | /Remove | /CreateSecondCue]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Cue")]
pub struct StoreCue {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
    #[lum(value)]
    pub name: Option<String>,
    #[lum(option = "Merge")]
    pub merge: bool,
    #[lum(option = "Overwrite")]
    pub overwrite: bool,
    #[lum(option = "Remove")]
    pub remove: bool,
    #[lum(option = "CreateSecondCue")]
    pub second: bool,
}

impl crate::commands::typed::Command for StoreCue {
    fn run(self, state: &mut AppState) -> crate::DispatchResult {
        let mode = store_mode(self.merge, self.overwrite, self.remove, self.second);
        crate::DispatchResult::Output(store_cue_typed(state, self.target.as_ref(), self.name, mode).into())
    }
}

/// `Store Sequence [<n>] ["name"]`, or a cue of it: `Store Sequence 4 Cue 2` (owner first).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Sequence")]
pub struct StoreSequence {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
    #[lum(value)]
    pub name: Option<String>,
    #[lum(option = "Merge")]
    pub merge: bool,
    #[lum(option = "Overwrite")]
    pub overwrite: bool,
    #[lum(option = "Remove")]
    pub remove: bool,
    #[lum(option = "CreateSecondCue")]
    pub second: bool,
}

impl crate::commands::typed::Command for StoreSequence {
    fn run(self, state: &mut AppState) -> crate::DispatchResult {
        use crate::resolve::Slot;
        let names_cue = self.target.as_ref().is_some_and(|t| match t.items.as_slice() {
            [(None, lumacore_syntax::ast::Item::Path(segs))] => segs.last().is_some_and(|l| {
                objects::find(&crate::lum::vocabulary().word(l.object.node).name).is_some_and(|o| std::ptr::eq(o, &objects::CUE))
            }),
            _ => false,
        });
        let text = if names_cue {
            let mode = store_mode(self.merge, self.overwrite, self.remove, self.second);
            store_cue_typed(state, self.target.as_ref(), self.name, mode)
        } else {
            match crate::resolve::slot_owned(state, self.target.as_ref(), &objects::SEQUENCE) {
                Ok((slot, datapool, _)) => {
                    let pool = datapool.or(state.datapool_override);
                    let before = std::mem::replace(&mut state.datapool_override, pool);
                    let out = match slot {
                        Slot::Number(n) => store_sequence_at(state, Some(n), self.name),
                        Slot::NewName(n) => store_sequence_at(state, None, Some(n)),
                        Slot::Next => store_sequence_at(state, None, self.name),
                        Slot::Key(k) => render_code(format!("'{k}' isn't a Sequence number."), "E-CMD-008"),
                    };
                    state.datapool_override = before;
                    out
                }
                Err(e) => e,
            }
        };
        crate::DispatchResult::Output(text.into())
    }
}

/// The cue a typed target names, owner first (`Sequence 2 Cue 3`; `Cue 3` = your selected
/// sequence's), through the one resolver (decisions §33 57, 3d).
pub fn cue_target(state: &mut AppState, target: &lumacore_syntax::ast::Target) -> Result<Target, String> {
    let (r, owners) = crate::resolve::resolve_owned(state, target, &objects::CUE)?;
    if !std::ptr::eq(r.object, &objects::CUE) {
        return Err(render_code(format!("Name a cue: Cue <n> (owner first: Sequence <n> Cue <m>), not {}.", r.object.keyword), "E-CMD-008"));
    }
    let sequence = owners
        .iter()
        .find(|(o, _)| std::ptr::eq(*o, &objects::SEQUENCE))
        .map(|(_, k)| k.parse::<u32>().map_err(|_| render_code(format!("No Sequence {k}."), "E-CMD-007")))
        .transpose()?;
    let cue = match r.members.as_slice() {
        [] if r.named => return Err(render_code("That cue doesn't exist (List Cue).", "E-PLAY-001")),
        [] => None,
        [one] => one.name.clone(),
        _ => return Err(render_code("Name one cue.", "E-CMD-008")),
    };
    Ok(Target { sequence, cue })
}

/// `Goto [Sequence <n>] Cue <m>` (decisions §33: owner first; `Cue 3` alone = Goto).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Goto")]
pub struct GotoCue {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
}

impl crate::commands::typed::Command for GotoCue {
    fn run(self, state: &mut AppState) -> crate::DispatchResult {
        let text = match cue_target(state, &self.target) {
            Ok(t) => goto(state, t),
            Err(e) => e,
        };
        crate::DispatchResult::Output(text.into())
    }
}

pub fn goto(state: &mut AppState, target: Target) -> String {
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    let Some(cue_no) = target.cue else {
        return render_code(
            "Goto which cue? Usage: Goto Cue 3 [Sequence 1]",
            "E-CMD-006",
        );
    };
    let key = cue_ref(&cue_no).expect("checked by parse_target");
    if key == lumacore_tree::CUE_ZERO || key == OFF_CUE {
        return render_code(
            format!("{} isn't a cue to go to: a run starts from CueZero; Off plays the OffCue.", cue_name(key)),
            "E-CMD-010",
        );
    }
    match sequence(state, dp, seq_id) {
        Ok(seq) if seq.cues.contains_key(&key) => {
            let guid = seq_guid(state, dp, seq_id);
            match state.root.show_data.playbacks.live.sequences.get_mut(&guid) {
                // Running: only the cue and the asserts change; fader,
                // rate, paused and who started it stay.
                Some(st) => {
                    st.assert_at(key, next_stamp());
                    render_success(&format!("Sequence {seq_id} → Cue {}", cue_label(key)))
                }
                None => started(state, dp, seq_id, key),
            }
        }
        Ok(_) => no_such(format!("Sequence {seq_id} has no Cue {cue_no}.")),
        Err(e) => e,
    }
}

/// `Off Sequence <n>`: stop it; its values release (the OffCue).
pub fn off(state: &mut AppState, target: Target) -> String {
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    let guid = seq_guid(state, dp, seq_id);
    // Off fades its echoes with it (their OffCue, decisions §32).
    let echoes = end_echoes(state, guid, |_| true);
    let tail = if echoes > 0 { format!(" ({echoes} echo{} ending)", if echoes == 1 { "" } else { "es" }) } else { String::new() };
    match state.root.show_data.playbacks.live.sequences.remove(&guid) {
        Some(_) => render_success(&format!("Sequence {seq_id} off{tail}")),
        None if echoes > 0 => render_success(&format!("Sequence {seq_id}: {echoes} echo{} ending", if echoes == 1 { "" } else { "es" })),
        None => render_success(&format!("Sequence {seq_id} wasn't running")),
    }
}

// ── Echoes (decisions §32 "Top = the echo multi-caller") ────────────────────

/// Start the OffCue of the running echoes of `guid` that `which` picks; how many.
fn end_echoes(state: &mut AppState, guid: uuid::Uuid, which: impl Fn(&lumacore_tree::Echo) -> bool) -> usize {
    let now = lumacore_base::clock::session_ms();
    let Some(list) = state.root.show_data.playbacks.echoes.sequences.get_mut(&guid) else { return 0 };
    let mut n = 0;
    for e in list.iter_mut().filter(|e| e.off_ms.is_none() && which(e)) {
        e.off_ms = Some(now);
        n += 1;
    }
    n
}

/// `Top On|Off Sequence <n>`: `On` starts an echo — a run from CueZero to the first cue with
/// the cue's timing, beside the live playback (which still takes Go / Off); `Off` plays that
/// echo's OffCue (the holder's newest, else the newest). A full list (`MaxEchoes`) refuses the
/// newest Top On; an old echo is never dropped. A bare `Top` is a usage error.
pub fn top(state: &mut AppState, args: &[&str]) -> String {
    let usage = || render_code("Usage: Top On|Off Sequence <n> (hold = On at press, Off at release).", "E-CMD-006");
    let (on, rest) = match args.first() {
        Some(w) if w.eq_ignore_ascii_case("on") => (true, &args[1..]),
        Some(w) if w.eq_ignore_ascii_case("off") => (false, &args[1..]),
        _ => return usage(),
    };
    if rest.is_empty() {
        return usage();
    }
    let target = match parse_target(rest, &objects::SEQUENCE) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    let (guid, first, max, ) = match sequence(state, dp, seq_id) {
        Ok(seq) => (seq.meta.identity.guid, seq.next_cue(None), seq.settings.max_echoes),
        Err(e) => return e,
    };
    let holder = state.current_client.map(|c| c.to_string());
    if !on {
        // The holder's newest running echo, else the newest running one.
        let list = state.root.show_data.playbacks.echoes.sequences.get(&guid);
        let running = |e: &&lumacore_tree::Echo| e.off_ms.is_none();
        let pick = list.and_then(|l| {
            l.iter().rev().filter(running).find(|e| holder.is_some() && e.holder == holder).or_else(|| l.iter().rev().find(running))
        });
        let Some(id) = pick.map(|e| e.id) else {
            return render_success(&format!("Sequence {seq_id} has no echo running"));
        };
        end_echoes(state, guid, |e| e.id == id);
        return render_success(&format!("Sequence {seq_id} echo {id} → OffCue"));
    }
    let Some(first) = first else {
        return no_such(format!("Sequence {seq_id} has no cues."));
    };
    if max == 0 {
        return render_code(format!("Sequence {seq_id} takes no echoes (MaxEchoes 0)."), "E-CMD-010");
    }
    let user = state.root.show_data.user(state.effective_user_name()).map_or_else(uuid::Uuid::nil, |u| u.meta.identity.guid);
    let list = state.root.show_data.playbacks.echoes.sequences.entry(guid).or_default();
    if list.len() >= max as usize {
        return render_code(
            format!("Sequence {seq_id} has {} echoes (MaxEchoes {max}): this Top On is refused.", list.len()),
            "E-CMD-010",
        );
    }
    let id = list.last().map_or(1, |e| e.id + 1);
    list.push(lumacore_tree::Echo {
        id,
        cue: first,
        started_ms: lumacore_base::clock::session_ms(),
        stamp: next_stamp(),
        holder,
        started_by: user,
        off_ms: None,
    });
    render_success(&format!("Sequence {seq_id} echo {id} → Cue {}", cue_label(first)))
}

/// The echoes' clock (the state thread's housekeeping, on the station that owns the show):
/// an echo running past `EchoMax` plays its OffCue, one whose holder went away (`holders`:
/// the attached clients on unlocked devices) too, and one whose OffCue has faded is gone.
/// Filed as an untracked change, so it replicates like any line (never undone).
pub fn echo_tick(state: &mut AppState, holders: &std::collections::HashSet<String>) {
    if state.link.is_some() || state.root.show_data.playbacks.echoes.sequences.is_empty() {
        return;
    }
    let now = lumacore_base::clock::session_ms();
    let show = &state.root.show_data;
    let mut ends = Vec::new();
    let mut gone = Vec::new();
    for (guid, list) in &show.playbacks.echoes.sequences {
        let Some((_, _, seq)) = show.sequence_by_guid(*guid) else {
            gone.push((*guid, None));
            continue;
        };
        let max_ms = (seq.settings.echo_max_s.max(0.0) * 1000.0) as u64;
        let off = seq.off_cue.parts.get(&0).map_or(0, |p| ((p.delay_out + p.fade_out).max(0.0) * 1000.0) as u64);
        for e in list {
            match e.off_ms {
                None if now.saturating_sub(e.started_ms) >= max_ms => ends.push((*guid, e.id)),
                None if e.holder.as_ref().is_some_and(|h| !holders.contains(h)) => ends.push((*guid, e.id)),
                Some(t) if now >= t.saturating_add(off) => gone.push((*guid, Some(e.id))),
                _ => {}
            }
        }
    }
    if ends.is_empty() && gone.is_empty() {
        return;
    }
    crate::commands::record_change(state, "Echo", super::registry::Undo::Untracked, |state| {
        let echoes = &mut state.root.show_data.playbacks.echoes.sequences;
        for (guid, id) in &ends {
            if let Some(e) = echoes.get_mut(guid).and_then(|l| l.iter_mut().find(|e| e.id == *id)) {
                e.off_ms = Some(now);
            }
        }
        for (guid, id) in &gone {
            match id {
                None => {
                    echoes.remove(guid);
                }
                Some(id) => {
                    if let Some(l) = echoes.get_mut(guid) {
                        l.retain(|e| e.id != *id);
                    }
                }
            }
        }
        echoes.retain(|_, l| !l.is_empty());
    });
}

/// `On Sequence <n>`: start it at its first cue unless it already runs.
pub fn on(state: &mut AppState, target: Target) -> String {
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    if state
        .root
        .show_data
        .playbacks
        .live.sequences
        .contains_key(&seq_guid(state, dp, seq_id))
    {
        return render_success(&format!("Sequence {seq_id} is already running"));
    }
    go(state, target)
}

/// `Toggle Sequence <n>`.
pub fn toggle(state: &mut AppState, target: Target) -> String {
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    if state
        .root
        .show_data
        .playbacks
        .live.sequences
        .contains_key(&seq_guid(state, dp, seq_id))
    {
        off(state, target)
    } else {
        on(state, target)
    }
}

// ── A cue's timing as properties (decisions §33 58, 87) ─────────────────────
//
// `Set Sequence 2 Cue 3 "Fade" 2`, `Set Sequence 2 Cue 3 FeatureGroup "Position" "Fade" 4`:
// the sequence scopes its cues, the cue its feature groups (`Props::within`); a cue alone is one
// of your selected sequence.

thread_local! {
    static CUE_SEQUENCE: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
    static CUE_SCOPE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

struct Scoped(Box<dyn FnMut()>);

impl Drop for Scoped {
    fn drop(&mut self) {
        (self.0)();
    }
}

/// `Sequence <n> …` scopes its cues.
pub fn sequence_within(_: &mut AppState, key: &str) -> Result<Box<dyn std::any::Any>, String> {
    let no = key.parse::<u32>().map_err(|_| render_code(format!("No Sequence {key}."), "E-CMD-007"))?;
    let before = CUE_SEQUENCE.with(|c| c.replace(Some(no)));
    Ok(Box::new(Scoped(Box::new(move || CUE_SEQUENCE.with(|c| c.set(before))))))
}

/// `… Cue <n> …` scopes its feature groups.
pub fn cue_within(_: &mut AppState, key: &str) -> Result<Box<dyn std::any::Any>, String> {
    let before = CUE_SCOPE.with(|c| c.replace(Some(key.to_string())));
    Ok(Box::new(Scoped(Box::new(move || CUE_SCOPE.with(|c| *c.borrow_mut() = before.clone())))))
}

/// The sequence the cue rows are of: the scoping one, else your selected one.
fn cue_sequence(state: &AppState) -> Option<(u32, u32)> {
    match CUE_SEQUENCE.with(std::cell::Cell::get) {
        Some(n) => Some((state.selected_datapool(), n)),
        None => state.selected_sequence(),
    }
}

/// The cue numbers of that sequence, as lines write them (`3`, `1.5`).
pub fn cue_rows(state: &AppState) -> Vec<String> {
    let Some((dp, seq)) = cue_sequence(state) else { return Vec::new() };
    state.root.show_data.datapools.get(&dp).and_then(|d| d.sequences.get(&seq)).map_or_else(Vec::new, |s| s.cues.keys().map(|&k| cue_label(k)).collect())
}

/// A typed cue (`3`, `1.5`, `0`, `OffCue`) → its row in that sequence.
pub fn cue_row(state: &AppState, w: &str) -> Option<String> {
    let key = cue_ref(w)?;
    let (dp, seq) = cue_sequence(state)?;
    let s = state.root.show_data.datapools.get(&dp)?.sequences.get(&seq)?;
    (s.cues.contains_key(&key) || key == lumacore_tree::CUE_ZERO || key == OFF_CUE).then(|| w.trim_matches('"').to_string())
}

/// One timing property of a cue (or, under a feature group, of that group in the cue).
fn cue_write(state: &mut AppState, key: &str, prop: &str, value: &str, group: Option<&str>) -> Result<String, String> {
    let (_, seq) = cue_sequence(state).ok_or_else(|| render_code("No sequence selected — Select Sequence <n>, or name it (Sequence <n> Cue …).", "E-PLAY-003"))?;
    let seq = seq.to_string();
    let mut args: Vec<&str> = vec![key, "Sequence", &seq];
    let quoted;
    if let Some(g) = group {
        quoted = format!("\"{g}\"");
        args.extend(["FeatureGroup", quoted.as_str()]);
    }
    args.extend([prop, value]);
    crate::props::from_reply(set_cue_timing(state, &args))
}

macro_rules! timing_prop {
    ($name:literal, $doc:literal, $group:expr) => {
        crate::props::Property {
            name: $name,
            kind: crate::props::PropKind::Declared,
            section: "Timing",
            doc: $doc,
            default: Some("Cue"),
            access: crate::props::Access::Named {
                get: |_, _, _| String::new(),
                set: |s, k, p, v| {
                    let cue = if $group { CUE_SCOPE.with(|c| c.borrow().clone()) } else { Some(k.to_string()) };
                    let Some(cue) = cue else {
                        return Err(render_code(format!("\"{p}\" of a feature group belongs to a cue: Set Cue <n> FeatureGroup \"{k}\" \"{p}\" …"), "E-SET-001"));
                    };
                    cue_write(s, &cue, p, v, $group.then_some(k))
                },
            },
            ..crate::props::Property::BASE
        }
    };
}

/// A cue's own timing (part 0): seconds, 0–3600.
pub static CUE_TIMING: [crate::props::Property; 4] = [
    timing_prop!("Fade", "fade in, seconds", false),
    timing_prop!("OutFade", "fade out, seconds", false),
    timing_prop!("Delay", "delay in, seconds", false),
    timing_prop!("OutDelay", "delay out, seconds", false),
];

/// A feature group's timing in a cue (unset = the cue's: `Reset … FeatureGroup "x"`).
pub static GROUP_TIMING: [crate::props::Property; 5] = [
    timing_prop!("Fade", "fade in, seconds (Reset = the cue's)", true),
    timing_prop!("OutFade", "fade out, seconds", true),
    timing_prop!("Delay", "delay in, seconds", true),
    timing_prop!("OutDelay", "delay out, seconds", true),
    crate::props::Property { named: &["Linear", "SCurve", "EaseIn", "EaseOut", "Snap"], ..timing_prop!("Path", "the shape of its fades: Linear, SCurve, EaseIn, EaseOut, Snap", true) },
];

/// `Set Cue <n> [Sequence <m>] [FeatureGroup <g>] Fade|OutFade|Delay|OutDelay <seconds>|Cue`
/// (and a feature group's `Path Linear|SCurve|EaseIn|EaseOut|Snap|Cue`):
/// part 0's timing (decisions §29: a dimmer going up and every other
/// attribute use the in fade/delay, a dimmer going down the out ones); with
/// a show feature group, that group's own timing in the part (decisions §30
/// "12f-a"; `Cue` = back to the part's).
pub fn set_cue_timing(state: &mut AppState, args: &[&str]) -> String {
    let usage = || {
        render_code(
            "Usage: Set Cue <n> [Sequence <m>] [FeatureGroup <g>] Fade|OutFade|Delay|OutDelay <seconds>|Cue \
             (FeatureGroup <g> Path Linear|SCurve|EaseIn|EaseOut|Snap|Cue)",
            "E-CMD-006",
        )
    };
    // `FeatureGroup <g>` anywhere before the property names the group.
    let mut args: Vec<&str> = args.to_vec();
    let mut group: Option<(uuid::Uuid, String)> = None;
    if let Some(i) = args.iter().position(|a| crate::commands::attributes::FEATURE_GROUP.matches(a)) {
        let Some(g) = args.get(i + 1).copied() else {
            return usage();
        };
        let w = crate::commands::tokens::unquote(g);
        let found = match w.parse::<u32>() {
            Ok(n) => state.root.show_data.feature_groups.get(&n),
            Err(_) => state.root.show_data.feature_groups.values().find(|x| x.meta.name().unwrap_or_default().eq_ignore_ascii_case(w)),
        };
        let Some(fg) = found else {
            return render_code(format!("No feature group \"{w}\" in the show."), "E-CMD-007");
        };
        group = Some((fg.meta.identity.guid, fg.meta.name().unwrap_or_default().to_string()));
        args.drain(i..i + 2);
    }
    let [target @ .., prop, value] = args.as_slice() else {
        return usage();
    };
    let inherit = group.is_some() && value.eq_ignore_ascii_case("cue");
    // A feature group's Path (the shape of its fades; sequence window A).
    let path = prop.eq_ignore_ascii_case("path");
    if path && group.is_none() {
        return render_code("A Path belongs to a feature group: Set Cue <n> FeatureGroup <g> Path <shape>.", "E-CMD-006");
    }
    let shape = if path && !inherit {
        let w = crate::commands::tokens::unquote(value);
        match lumacore_tree::FadePath::parse(w) {
            Some(p) => Some(p),
            None => {
                let all: Vec<&str> = lumacore_tree::FadePath::ALL.iter().map(|p| p.word()).collect();
                return render_code(format!("'{w}' is not a path ({}, or Cue).", all.join(", ")), "E-CMD-008");
            }
        }
    } else {
        None
    };
    let target = match parse_target(target, &objects::CUE) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let Some(cue_no) = target.cue.clone() else {
        return usage();
    };
    let seconds = if inherit || path {
        0.0
    } else {
        let Ok(seconds) = value.trim_end_matches(['s', 'S']).parse::<f64>() else {
            return render_code(format!("'{value}' is not a time in seconds."), "E-CMD-008");
        };
        seconds
    };
    if !(0.0..=3600.0).contains(&seconds) {
        return render_code("A cue time is 0–3600 s.", "E-CMD-008");
    }
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    let Some(key) = cue_ref(&cue_no) else {
        return render_code(format!("'{cue_no}' is not a cue number (or 0 / OffCue)."), "E-CMD-008");
    };
    let Some(cue) = state
        .root
        .show_data
        .sequence_mut(dp, seq_id)
        .and_then(|s| cue_mut(s, key))
    else {
        return no_such(format!("Cue {cue_no} in Sequence {seq_id} doesn't exist."));
    };
    let part = cue.part0();
    if let Some((g, name)) = group {
        let t = part.group_timing.entry(g).or_default();
        if path {
            t.path = shape;
            if t.is_empty() {
                part.group_timing.remove(&g);
            }
            return render_success(&match shape {
                Some(p) => format!("Cue {cue_no} Sequence {seq_id}: {name} Path {}", p.word()),
                None => format!("Cue {cue_no} Sequence {seq_id}: {name} Path follows the cue (Linear)"),
            });
        }
        let (field, label) = match prop.to_ascii_lowercase().as_str() {
            "fade" | "infade" | "fadein" => (&mut t.fade_in, "Fade"),
            "outfade" | "fadeout" => (&mut t.fade_out, "OutFade"),
            "delay" | "indelay" | "delayin" => (&mut t.delay_in, "Delay"),
            "outdelay" | "delayout" => (&mut t.delay_out, "OutDelay"),
            _ => return usage(),
        };
        *field = (!inherit).then_some(seconds);
        if t.is_empty() {
            part.group_timing.remove(&g);
        }
        return render_success(&if inherit {
            format!("Cue {cue_no} Sequence {seq_id}: {name} {label} follows the cue")
        } else {
            format!("Cue {cue_no} Sequence {seq_id}: {name} {label} {seconds} s")
        });
    }
    let (field, label) = match prop.to_ascii_lowercase().as_str() {
        "fade" | "infade" | "fadein" => (&mut part.fade_in, "Fade"),
        "outfade" | "fadeout" => (&mut part.fade_out, "OutFade"),
        "delay" | "indelay" | "delayin" => (&mut part.delay_in, "Delay"),
        "outdelay" | "delayout" => (&mut part.delay_out, "OutDelay"),
        _ => return usage(),
    };
    *field = seconds;
    render_success(&format!("Cue {cue_no} Sequence {seq_id}: {label} {seconds} s"))
}

/// `List Cue [Sequence <n>]`: the cues of a sequence, the current one marked.
pub fn list_cues(state: &AppState, target: Target) -> String {
    let (dp, seq_id) = match sequence_of(state, &target) {
        Ok(at) => at,
        Err(e) => return e,
    };
    let seq = match sequence(state, dp, seq_id) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let current = state
        .root
        .show_data
        .playbacks
        .live.sequences
        .get(&seq_guid(state, dp, seq_id))
        .map(|s| s.current_cue);
    let mut out = format!("Sequence {seq_id} \"{}\"", seq.meta.identity.name.as_deref().unwrap_or(""));
    let fixed = |label: &str, cue: &CueObject| {
        let n = cue.values().count();
        format!("\n  {label:<11} {:<16} {n} value{}", cue.meta.name().unwrap_or_default(), if n == 1 { "" } else { "s" })
    };
    out.push_str(&fixed("CueZero", &seq.cue_zero));
    if seq.cues.is_empty() {
        out.push_str("\n  (no cues)");
    }
    for (&key, cue) in &seq.cues {
        let n = cue.values().count();
        out.push_str(&format!(
            "\n{} Cue {:<6} {:<16} {n} value{}",
            if Some(key) == current { "▶" } else { " " },
            cue_label(key),
            cue.meta.name().unwrap_or_default(),
            if n == 1 { "" } else { "s" }
        ));
    }
    let off = seq.off_cue.parts.get(&0).map_or((0.0, 0.0), |p| (p.fade_out, p.delay_out));
    out.push_str(&format!("\n  OffCue      {:<16} OutFade {} s, OutDelay {} s", seq.off_cue.meta.name().unwrap_or_default(), off.0, off.1));
    out
}

/// Fill CueZero's defaults part (part 1) by the sequence's `CueZeroMode` (decisions §30
/// "12f echoes"): nothing, every attribute of the fixtures its cues use, or their dimmers — each
/// at the fixture type's default. What was stored into CueZero (part 0) stays and wins.
pub fn fill_cue_zero(show: &mut lumacore_tree::ShowData, dp: u32, seq_id: u32) {
    use lumacore_fixtures::fixture_type::AttributeType;
    use lumacore_tree::CueZeroMode;
    let Some(seq) = show.datapools.get(&dp).and_then(|d| d.sequences.get(&seq_id)) else { return };
    let mode = seq.settings.cue_zero_mode;
    let stored: std::collections::HashSet<(uuid::Uuid, String)> = seq
        .cue_zero
        .parts
        .get(&0)
        .map(|p| p.values.iter().map(|v| (v.fixture_id, v.attribute.clone())).collect())
        .unwrap_or_default();
    let used: std::collections::BTreeSet<uuid::Uuid> =
        seq.cues.values().flat_map(|c| c.values().map(|v| v.fixture_id)).collect();
    let mut defaults = Vec::new();
    if mode != CueZeroMode::Off {
        for guid in &used {
            let Some(fixture) = show.devices.dmx.patch.fixtures.values().find(|f| f.meta.identity.guid == *guid) else { continue };
            let mut seen = std::collections::HashSet::new();
            for cf in lumacore_render::merge::channels(show, fixture).iter().filter(|cf| !cf.is_virtual) {
                let wanted = mode == CueZeroMode::AllUsedAttributes || cf.attribute_type == AttributeType::Dimmer;
                if !wanted || !seen.insert(cf.attribute_name.clone()) || stored.contains(&(*guid, cf.attribute_name.clone())) {
                    continue;
                }
                defaults.push(CueValue {
                    fixture_id: *guid,
                    attribute: cf.attribute_name.clone(),
                    timing: Default::default(),
                    value: lumacore_tree::CueData::Value(lumacore_tree::Value::normalized(lumacore_render::merge::default_fraction(cf) * 100.0)),
                });
            }
        }
    }
    let Some(seq) = show.sequence_mut(dp, seq_id) else { return };
    if defaults.is_empty() {
        seq.cue_zero.parts.remove(&1);
    } else {
        seq.cue_zero.parts.entry(1).or_default().values = defaults;
    }
}

/// What runs: every sequence's current cue and who started it.
pub fn list_running(state: &AppState) -> String {
    let running = &state.root.show_data.playbacks.live.sequences;
    if running.is_empty() {
        return "No playbacks running.".to_string();
    }
    let mut out = String::from("Running playbacks:");
    let show = &state.root.show_data;
    for (guid, st) in running {
        let id = show
            .sequence_by_guid(*guid)
            .map_or_else(|| "?".to_string(), |(_, n, _)| n.to_string());
        let by = show
            .user_by_guid(st.started_by)
            .map_or("?", |u| u.login.as_str());
        out.push_str(&format!(
            "\n  Sequence {id:<4} Cue {:<6} started by {by}",
            cue_label(st.current_cue),
        ));
    }
    out
}

// ── Masters ───────────────────────────────────────────────────────────────

/// Which master a command names (decisions §30 "12f", the master groups): the grand group's
/// 2.1 Master and its 2.2 Highlight / 2.4 Solo caps on everybody, or the acting user's own
/// Highlight / Solo master (`Master "Highlight"`; decisions §16, §29).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Which {
    Grand,
    GrandCap(lumacore_tree::UserMode),
    Mode(lumacore_tree::UserMode),
}

/// The masters a full console has that we don't yet: their group and what they wait for.
fn planned_master(n: &str) -> Option<&'static str> {
    let group = n.split('.').next()?;
    Some(match (group, n) {
        (_, "2.3") => "2.3 Lowlight",
        (_, "2.5") => "2.5 Blind",
        ("1", _) => "group 1 Selected (follows the selected sequence)",
        ("3", _) => "group 3 Speed (with the effects, decisions §28)",
        ("4", _) => "group 4 Playback (with the executors)",
        ("5", _) => "group 5 Timing",
        _ => return None,
    })
}

fn which(args: &[&str]) -> Result<Which, String> {
    use lumacore_tree::UserMode;
    let Some(first) = args.first() else {
        return Err(render_code(
            "Which master? Master 2.1 (grand), 2.2 / 2.4 (Highlight / Solo for everybody), Master \"Highlight\" | Solo (yours).",
            "E-CMD-006",
        ));
    };
    let is = |def: &ObjectDef| objects::find(first).is_some_and(|o| std::ptr::eq(o, def));
    let word = first.trim_matches('"');
    match word {
        "2.1" => Ok(Which::Grand),
        _ if word.eq_ignore_ascii_case("grand") => Ok(Which::Grand),
        "2.2" => Ok(Which::GrandCap(UserMode::Highlight)),
        "2.4" => Ok(Which::GrandCap(UserMode::Solo)),
        _ if is(&objects::HIGHLIGHT) => Ok(Which::Mode(UserMode::Highlight)),
        _ if is(&objects::SOLO) => Ok(Which::Mode(UserMode::Solo)),
        n => Err(match planned_master(n) {
            Some(what) => render_code(
                format!(
                    "Master {n}: {what} isn't built yet (planned).{}",
                    if n == "1" { " The grand master is Master 2.1." } else { "" }
                ),
                "E-CMD-003",
            ),
            None => render_code(
                format!("Master {n}: the masters are 2.1 (grand), 2.2 / 2.4 and Master \"Highlight\" | Solo."),
                "E-CMD-007",
            ),
        }),
    }
}

/// A master's level after `At` (decisions §30 "12f": masters take relative input): `63.5`,
/// `Full`, `Zero`, `+ 10` / `- 10` / `* 2` / `/ 2` (glued `+10` too) from `now` (0–100).
/// The new level 0–100 and how many words it took; never 0 for a word it can't read (E-CMD-008).
pub fn level_after_at(words: &[&str], now: f64) -> Result<(f64, usize), String> {
    let bad = |v: &str| render_code(format!("'{v}' is not a level (0–100, Full, Zero, + 10 …)."), "E-CMD-008");
    let Some(first) = words.first() else {
        return Err(render_code("At what? A level 0–100 (or + 10 …).", "E-CMD-006"));
    };
    let number = |v: &str| v.trim_end_matches('%').parse::<f64>().ok().filter(|n| n.is_finite());
    let (raw, used) = match *first {
        // tidy: not-keywords (arithmetic operators of a relative level, not words)
        op @ ("+" | "-" | "*" | "/") => match words.get(1) {
            Some(by) => (format!("{op} {by}"), 2),
            None => return Err(bad(op)),
        },
        w if w.len() > 1 && w.starts_with(['+', '*', '/']) => (format!("{} {}", &w[..1], &w[1..]), 1),
        w if w.eq_ignore_ascii_case("full") => return Ok((100.0, 1)),
        w if w.eq_ignore_ascii_case("zero") => return Ok((0.0, 1)),
        w => return number(w).map(|n| (n.clamp(0.0, 100.0), 1)).ok_or_else(|| bad(w)),
    };
    let (op, by) = raw.split_at(1);
    let by = number(by.trim()).ok_or_else(|| bad(&raw))?;
    let next = match op {
        "+" => now + by,
        "-" => now - by,
        "*" => now * by,
        _ if by == 0.0 => return Err(render_code("A master can't be divided by 0.", "E-CMD-008")),
        _ => now / by,
    };
    Ok((next.clamp(0.0, 100.0), used))
}

/// `63.5` → "63.5", `100` → "100".
fn pct(level: f64) -> String {
    let r = (level * 10.0).round() / 10.0;
    if r.fract() == 0.0 { format!("{r:.0}") } else { format!("{r:.1}") }
}

/// The acting user's master of `mode`.
fn mode_master(state: &mut AppState, mode: lumacore_tree::UserMode) -> &mut lumacore_tree::ModeMaster {
    let user = state.effective_user_name().to_string();
    let modes = &mut state.root.runtime.user(&user).modes;
    match mode {
        lumacore_tree::UserMode::Highlight => &mut modes.highlight_master,
        _ => &mut modes.solo_master,
    }
}

/// A grand cap (2.2 Highlight, 2.4 Solo), 0–1.
fn grand_cap(state: &mut AppState, mode: lumacore_tree::UserMode) -> &mut f64 {
    let g = &mut state.root.show_data.playbacks.live.masters.grand;
    match mode {
        lumacore_tree::UserMode::Highlight => &mut g.highlight,
        _ => &mut g.solo,
    }
}

/// `Master 2.1 At <level>`: the grand master. `Master 2.2|2.4 At <level>`: Highlight / Solo for
/// everybody. `Master Highlight|Solo At <level>` / `… Mode Toggle|AlwaysOn|Disabled`: the acting
/// user's mode master. Masters are never undone (decisions §29, §30 "12f").
pub fn master_call(state: &mut AppState, args: &[&str]) -> String {
    let which = match which(args) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let rest = args.get(1..).unwrap_or_default();
    let at = rest.first().is_some_and(|w| w.eq_ignore_ascii_case("at"));
    let read = |now: f64| -> Result<f64, String> {
        let (level, used) = level_after_at(&rest[1..], now)?;
        match rest.get(1 + used) {
            None => Ok(level),
            Some(extra) => Err(render_code(format!("Unexpected '{extra}' after the level."), "E-CMD-006")),
        }
    };
    match which {
        Which::Grand if at => {
            let now = state.root.show_data.playbacks.live.masters.grand.master * 100.0;
            match read(now) {
                Ok(level) => {
                    state.root.show_data.playbacks.live.masters.grand.master = level / 100.0;
                    render_success(&format!("Master 2.1 Grand at {}%", pct(level)))
                }
                Err(e) => e,
            }
        }
        Which::Grand if rest.is_empty() => {
            let (level, on) = state.root.show_data.playbacks.grand();
            format!("Master 2.1 Grand: {}% {}", pct(level * 100.0), if on { "on" } else { "OFF (blackout)" })
        }
        Which::GrandCap(mode) if at => {
            let now = *grand_cap(state, mode) * 100.0;
            match read(now) {
                Ok(level) => {
                    *grand_cap(state, mode) = level / 100.0;
                    let no = if mode == lumacore_tree::UserMode::Highlight { "2.2" } else { "2.4" };
                    render_success(&format!("Master {no} {} (everybody) at {}%", mode.name(), pct(level)))
                }
                Err(e) => e,
            }
        }
        Which::GrandCap(mode) if rest.is_empty() => {
            let no = if mode == lumacore_tree::UserMode::Highlight { "2.2" } else { "2.4" };
            format!("Master {no} {} (everybody): {}%", mode.name(), pct(*grand_cap(state, mode) * 100.0))
        }
        Which::Mode(mode) if at => {
            let now = f64::from(mode_master(state, mode).level);
            match read(now) {
                Ok(level) => {
                    let m = mode_master(state, mode);
                    m.level = level.round() as u8;
                    render_success(&format!("{} master at {}%", mode.name(), m.level))
                }
                Err(e) => e,
            }
        }
        Which::Mode(mode) if rest.first().is_some_and(|w| w.eq_ignore_ascii_case("mode")) => {
            match rest.get(1).and_then(|v| lumacore_tree::MasterMode::parse(v)) {
                Some(m) => {
                    mode_master(state, mode).mode = m;
                    render_success(&format!("{} master mode {}", mode.name(), m.word()))
                }
                None => render_code(
                    format!("'{}': a master mode is Toggle, AlwaysOn or Disabled.", rest.get(1).unwrap_or(&"")),
                    "E-CMD-008",
                ),
            }
        }
        Which::Mode(mode) if rest.is_empty() => {
            let m = *mode_master(state, mode);
            format!("{} master (yours): {}% {}", mode.name(), m.level, m.mode.word())
        }
        _ => render_code(
            "Usage: Master 2.1|2.2|2.4|Highlight|Solo At <0-100 | + 10 …> · Master Highlight|Solo Mode Toggle|AlwaysOn|Disabled · On/Off/Toggle Master 2.1",
            "E-CMD-006",
        ),
    }
}

// ── Masters on the registry (decisions §33 10, 88, 90) ──────────────────────
//
// Rows by key: 2.1 (= "Grand"), 2.2, 2.4, "Highlight", "Solo"; the planned ones (1, 2.3, 2.5) are
// found too so their line says planned. Under `Master` are the group masters (`Master Group 3`).

const MASTER_ROWS: [&str; 5] = ["2.1", "2.2", "2.4", "Highlight", "Solo"];

fn master_row(_: &AppState, w: &str) -> Option<String> {
    let w = w.trim_matches('"');
    // tidy: not-keywords (the masters' own names and numbers)
    Some(match w.to_ascii_lowercase().as_str() {
        "2.1" | "grand" => "2.1".to_string(),
        // Every other number reaches the master code, which says planned (1, 2.3, 3.1 …) or none.
        n if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit() || c == '.') => w.to_string(),
        "highlight" => "Highlight".to_string(),
        "solo" => "Solo".to_string(),
        _ => return None,
    })
}

static MASTER_MODE: [crate::props::Property; 1] = [crate::props::Property {
    name: "Mode",
    kind: crate::props::PropKind::Choice(&["Toggle", "AlwaysOn", "Disabled"]),
    rights: crate::commands::registry::Rights::Playback,
    section: "Master",
    doc: "how your Highlight / Solo master acts: Toggle, AlwaysOn or Disabled (2.3 / 2.5)",
    access: crate::props::Access::Keyed {
        get: |s, k| match which(&[k]) {
            Ok(Which::Mode(mode)) => s.root.runtime.users.get(s.effective_user_name()).map_or_else(String::new, |u| {
                let m = if mode == lumacore_tree::UserMode::Highlight { &u.modes.highlight_master } else { &u.modes.solo_master };
                m.mode.word().to_string()
            }),
            _ => String::new(),
        },
        set: |s, k, v| match which(&[k]) {
            Ok(Which::Mode(_)) => crate::props::from_reply(master_call(s, &[k, "Mode", v])),
            _ => Err(render_code(format!("Master {k} has no Mode (only Master \"Highlight\" / \"Solo\")."), "E-SET-001")),
        },
    },
    ..crate::props::Property::BASE
}];

pub static MASTER_PROPS: crate::props::Props = crate::props::Props {
    lists: &[&MASTER_MODE],
    keyed: Some(crate::props::Keyed { rows: |_| MASTER_ROWS.iter().map(|r| r.to_string()).collect(), row: master_row }),
    // `Master Group 3 …`: the Master owns the group masters (§33 90); alone it is the grand one.
    implicit: Some(|_| "2.1".to_string()),
    label: Some(|_, k| format!("Master {k}")),
    within: Some(|_, _| Ok(Box::new(()))),
    ..crate::props::Props::BASE
};

/// A level value as the master code reads it (`80`, `+ 10`, `Full`).
fn level_words(v: &lumacore_syntax::ast::Value) -> Result<Vec<String>, String> {
    use lumacore_syntax::ast::Value;
    Ok(match v {
        Value::Number(n) => vec![n.node.clone()],
        Value::Relative { op, number } => vec![op.node.to_string(), number.node.clone()],
        Value::Word(w) => vec![crate::lum::vocabulary().word(w.node).name.clone()],
        _ => return Err(render_code("At a level: 0–100, Full, Zero, + 10 …", "E-CMD-008")),
    })
}

/// The master rows (or group masters) a typed target names.
enum Masters {
    Rows(Vec<String>),
    Groups(Vec<u32>),
}

fn masters_of(state: &mut AppState, target: &lumacore_syntax::ast::Target) -> Result<Masters, String> {
    let (r, _) = crate::resolve::resolve_owned(state, target, &objects::MASTER)?;
    if std::ptr::eq(r.object, &crate::commands::groups::GROUP) {
        if r.members.is_empty() {
            return Err(render_code("No such group.", "E-GROUP-003"));
        }
        return Ok(Masters::Groups(r.members.iter().map(|m| m.no).collect()));
    }
    if !std::ptr::eq(r.object, &objects::MASTER) {
        return Err(render_code(format!("Name a master (Master 2.1, Master Group 3), not {}.", r.object.keyword), "E-CMD-008"));
    }
    let rows: Vec<String> = if r.named { r.members.iter().filter_map(|m| m.name.clone()).collect() } else { vec!["2.1".into()] };
    if rows.is_empty() {
        return Err(render_code("The masters are 2.1 (\"Grand\"), 2.2, 2.4, \"Highlight\" and \"Solo\".", "E-CMD-007"));
    }
    Ok(Masters::Rows(rows))
}

/// `Master 2.1 At 80`, `Master "Highlight" At + 10`, `Master Group 4 At 50 Fade 5` (decisions §33 90).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "At", object = "Master")]
pub struct MasterAt {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "value")]
    pub level: lumacore_syntax::ast::Value,
    #[lum(helper = "Fade")]
    pub fade: Option<lumacore_syntax::ast::Value>,
}

impl crate::commands::typed::Command for MasterAt {
    fn run(self, state: &mut AppState) -> crate::DispatchResult {
        let text = (|| {
            let level = level_words(&self.level)?;
            let fade = match &self.fade {
                None => None,
                Some(lumacore_syntax::ast::Value::Number(n)) => Some(n.node.clone()),
                Some(_) => return Err(render_code("Fade: seconds, e.g. Fade 2", "E-CMD-008")),
            };
            Ok(match masters_of(state, &self.target)? {
                Masters::Groups(ids) => ids
                    .iter()
                    .map(|&id| {
                        let mut args: Vec<&str> = vec!["At"];
                        args.extend(level.iter().map(String::as_str));
                        if let Some(f) = &fade {
                            args.extend(["Fade", f.as_str()]);
                        }
                        crate::commands::groups::master(state, id, &args)
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                Masters::Rows(rows) => {
                    if fade.is_some() {
                        return Err(render_code("Fade is for a group master (Master Group 3 At 50 Fade 2).", "E-CMD-008"));
                    }
                    rows.iter()
                        .map(|k| {
                            let mut args: Vec<&str> = vec![k.as_str(), "At"];
                            args.extend(level.iter().map(String::as_str));
                            master_call(state, &args)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            })
        })();
        crate::DispatchResult::Output(text.unwrap_or_else(|e| e).into())
    }
}

macro_rules! master_switch_verb {
    ($name:ident, $verb:literal, $on:expr) => {
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = $verb, object = "Master")]
        pub struct $name {
            #[lum(target)]
            pub target: lumacore_syntax::ast::Target,
        }

        impl crate::commands::typed::Command for $name {
            fn run(self, state: &mut AppState) -> crate::DispatchResult {
                let text = match masters_of(state, &self.target) {
                    Ok(Masters::Rows(rows)) => rows.iter().map(|k| master_switch(state, &[k.as_str()], $on)).collect::<Vec<_>>().join("\n"),
                    Ok(Masters::Groups(_)) => render_code("A group master has a level only: Master Group 3 At 0.", "E-CMD-010"),
                    Err(e) => e,
                };
                crate::DispatchResult::Output(text.into())
            }
        }
    };
}

master_switch_verb!(OnMaster, "On", Some(true));
master_switch_verb!(OffMaster, "Off", Some(false));
master_switch_verb!(ToggleMaster, "Toggle", None);

pub fn master_switch(state: &mut AppState, args: &[&str], on: Option<bool>) -> String {
    match which(args) {
        Err(e) => e,
        Ok(Which::Mode(mode)) => {
            let user = state.effective_user_name().to_string();
            let modes = &mut state.root.runtime.user(&user).modes;
            let now = on.unwrap_or(!modes.get(mode));
            modes.set(mode, now);
            render_success(&format!("{} {}", mode.name(), if now { "On" } else { "Off" }))
        }
        Ok(Which::GrandCap(_)) => render_code(
            "Master 2.2 / 2.4 have a level only (Master 2.2 At …); switch your mode with On / Off Highlight | Solo.",
            "E-CMD-010",
        ),
        Ok(Which::Grand) => {
            let g = &mut state.root.show_data.playbacks.live.masters.grand;
            g.on = on.unwrap_or(!g.on);
            if g.on {
                render_success(&format!("Grand master on ({}%)", pct(g.master * 100.0)))
            } else {
                format!("{}", nu_ansi_term::Color::Red.bold().paint("BLACKOUT — grand master off"))
            }
        }
    }
}

// ── Sequence settings (Set Sequence <n> <Prop> <value>) ─────────────────────

/// A sequence's settings that are built (decisions §30 "12f echoes", §32).
pub const SEQUENCE_PROPS: [crate::props::Property; 4] = [
    crate::props::Property {
        name: "Tracking",
        access: crate::props::Access::Field("settings.tracking"),
        kind: crate::props::PropKind::OnOff,
        rights: crate::commands::registry::Rights::Program,
        doc: "values track into later cues",
        ..crate::props::Property::BASE
    },
    crate::props::Property {
        name: "CueZeroMode",
        access: crate::props::Access::Field("settings.cue_zero_mode"),
        kind: crate::props::PropKind::Choice(&["Off", "AllUsedAttributes", "OnlyUsedDimmers"]),
        rights: crate::commands::registry::Rights::Program,
        doc: "what CueZero is filled with: nothing, every used attribute's default, or the used dimmers'",
        ..crate::props::Property::BASE
    },
    crate::props::Property {
        name: "MaxEchoes",
        access: crate::props::Access::Field("settings.max_echoes"),
        kind: crate::props::PropKind::Int { min: 0, max: 64 },
        rights: crate::commands::registry::Rights::Program,
        doc: "how many echoes (Top On) may run at once; 0 = none",
        ..crate::props::Property::BASE
    },
    crate::props::Property {
        name: "EchoMax",
        access: crate::props::Access::Field("settings.echo_max_s"),
        kind: crate::props::PropKind::Int { min: 1, max: 3600 },
        rights: crate::commands::registry::Rights::Program,
        doc: "seconds an echo runs before its OffCue starts by itself",
        ..crate::props::Property::BASE
    },
];

/// Write a sequence's settings; CueZero's defaults follow its mode.
pub fn sequence_write(
    root: &mut lumacore_tree::Root,
    dp: Option<u32>,
    id: u32,
    edit: &mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>,
) -> Result<(), String> {
    let dp = dp.ok_or("no DataPool")?;
    let seq = root.show_data.sequence_mut(dp, id).ok_or("no such sequence")?;
    crate::props::edit_json(seq, edit)?;
    fill_cue_zero(&mut root.show_data, dp, id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_read_in_either_order() {
        let t = parse_target(&["3", "Sequence", "2"], &objects::CUE).unwrap();
        assert_eq!(t.sequence, Some(2));
        assert_eq!(t.cue.as_deref(), Some("3"));
        let t = parse_target(&["Sequence", "2", "Cue", "1.5"], &objects::CUE).unwrap();
        assert_eq!((t.sequence, t.cue.as_deref()), (Some(2), Some("1.5")));
        assert!(parse_target(&["Frog", "2"], &objects::CUE).is_err());
    }
}
