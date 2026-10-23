// ███████╗███████╗ ██████╗ ██╗   ██╗███████╗███╗   ██╗ ██████╗███████╗
// ██╔════╝██╔════╝██╔═══██╗██║   ██║██╔════╝████╗  ██║██╔════╝██╔════╝
// ███████╗█████╗  ██║   ██║██║   ██║█████╗  ██╔██╗ ██║██║     █████╗
// ╚════██║██╔══╝  ██║▄▄ ██║██║   ██║██╔══╝  ██║╚██╗██║██║     ██╔══╝
// ███████║███████╗╚██████╔╝╚██████╔╝███████╗██║ ╚████║╚██████╗███████╗
// ╚══════╝╚══════╝ ╚══▀▀═╝  ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝╚══════╝
// FILE: sequence

//! The sequence sheet (decisions §30 "12f" sequence window A): one sequence's cues — CueZero
//! first, OffCue last — with their timing, and per cue its feature groups' own Fade · Delay ·
//! Path (unset = the cue's, which the page shows in brackets). Built for the `sequence` feed
//! from the published show; every edit the page makes is a line built from `line` + a word.
//!
//! Owns:          `resolve` (the sequence a sheet shows), `view` (the sequence sheet and timeline).
//! Doesn't own:   cue timing (`lumacore_tree`), the Sequence object (`lumacore-objects`).
//! Entry points:  resolve, view.
//! Decisions:     §30 "12f", §34 20.7.

use lumacore_tree::{CueObject, CueTrigger, FadePath, SequenceObject, ShowData, cue_label};
use serde_json::{Value, json};
use uuid::Uuid;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The sequence a sheet shows: `"selected"` (or nothing) = the user's selected sequence, a GUID
/// = that one. (DataPool, number, sequence).
pub fn resolve<'a>(show: &'a ShowData, user: &str, which: &Value) -> Option<(u32, u32, &'a SequenceObject)> {
    let guid = match which.as_str().and_then(|s| Uuid::parse_str(s).ok()) {
        Some(g) => g,
        None => show.user(user)?.selected.sequence?,
    };
    show.sequence_by_guid(guid).map(|(dp, no, s)| (dp, no, s.as_ref()))
}

fn trigger(t: &CueTrigger) -> String {
    match t {
        CueTrigger::Go => "Go".into(),
        CueTrigger::Follow => "Follow".into(),
        CueTrigger::Time(s) => format!("Time {}", trim(*s)),
        CueTrigger::Manual => "Manual".into(),
        CueTrigger::Sound => "Sound".into(),
        CueTrigger::Bpm(b) => format!("BPM {}", trim(*b)),
        CueTrigger::Midi => "MIDI".into(),
    }
}

fn trim(v: f64) -> String {
    let s = format!("{:.2}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// One row: the cue's part 0 timing and each feature group's own (null = the cue's).
fn row(show: &ShowData, owner: &str, seq_no: u32, label: &str, key: Option<u32>, cue: &CueObject, current: Option<u32>) -> Value {
    let part = cue.parts.get(&0).cloned().unwrap_or_default();
    let groups: Vec<Value> = show
        .feature_groups
        .values()
        .map(|g| {
            let t = part.group_timing.get(&g.meta.identity.guid).copied().unwrap_or_default();
            json!({
                "fade": t.fade_in, "delay": t.delay_in, "outfade": t.fade_out, "outdelay": t.delay_out,
                "path": t.path.map(FadePath::word),
            })
        })
        .collect();
    json!({
        "cue": label,
        "key": key,
        "name": cue.meta.identity.name,
        "trig": trigger(&cue.trigger),
        "fade": part.fade_in, "outfade": part.fade_out, "delay": part.delay_in, "outdelay": part.delay_out,
        "values": cue.values().count(),
        "parts": cue.parts.len(),
        "current": key.is_some() && key == current,
        // `current` is set only while the sequence runs.
        "run": key.is_some() && key == current,
        "line": format!("Set {owner}Sequence {seq_no} Cue {label}"),
        "groups": groups,
    })
}

/// The sheet's data for `user`: `{sequence, datapool, guid, name, running, current, feature_groups,
/// paths, cues: [CueZero, cues…, OffCue]}`; `{missing}` when there is none to show.
pub fn view(show: &ShowData, user: &str, which: &Value) -> Value {
    let Some((dp, no, seq)) = resolve(show, user, which) else {
        let note = if which.as_str().and_then(|s| Uuid::parse_str(s).ok()).is_some() {
            "The sequence is gone."
        } else {
            "No sequence selected: Select Sequence <n>."
        };
        return json!({ "missing": true, "note": note });
    };
    // A sequence outside your selected DataPool is named owner first (`Set DataPool 2 Cue …`).
    let owner = if dp == lumacore_objects::pools::user_datapool(show, user) { String::new() } else { format!("DataPool {dp} ") };
    let state = show.playbacks.live.sequences.get(&seq.meta.identity.guid);
    let running = state.is_some();
    let current = state.map(|s| s.current_cue);
    // Each cue row wears its colour as this viewer sees it (decisions 0294).
    let paint = lumacore_objects::paint::Painter::new(show, user);
    let painted = |mut r: Value, cue: &CueObject| {
        let shown = paint.of(&cue.meta);
        r["paint"] = shown.hex.clone().map_or(Value::Null, Value::from);
        r["paint_from"] = json!(shown.from_text());
        r
    };
    let mut cues = vec![painted(row(show, &owner, no, "0", Some(lumacore_tree::CUE_ZERO), &seq.cue_zero, None), &seq.cue_zero)];
    cues[0]["fixed"] = json!("CueZero");
    cues[0]["name"] = json!(seq.cue_zero.meta.name().filter(|n| !n.is_empty()).unwrap_or("CueZero"));
    for (&key, cue) in &seq.cues {
        cues.push(painted(row(show, &owner, no, &cue_label(key), Some(key), cue, current), cue));
    }
    let mut off = painted(row(show, &owner, no, "OffCue", None, &seq.off_cue, None), &seq.off_cue);
    off["fixed"] = json!("OffCue");
    cues.push(off);
    json!({
        "sequence": no,
        "datapool": dp,
        "guid": seq.meta.identity.guid.to_string(),
        "name": seq.meta.identity.name.clone().unwrap_or_else(|| format!("Sequence {no}")),
        "running": running,
        "current": current.map(cue_label),
        "select": format!("Select {owner}Sequence {no}"),
        "feature_groups": show.feature_groups.values().map(|g| json!({
            "name": g.meta.identity.name,
            "word": format!("FeatureGroup \"{}\"", g.meta.name().unwrap_or_default()),
            "color": crate::attribute_board::group_colour(&paint, g),
        })).collect::<Vec<_>>(),
        "paths": FadePath::ALL.iter().map(|p| p.word()).collect::<Vec<_>>(),
        "cues": cues,
    })
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
