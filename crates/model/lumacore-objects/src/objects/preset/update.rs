// ██╗   ██╗██████╗ ██████╗  █████╗ ████████╗███████╗
// ██║   ██║██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔════╝
// ██║   ██║██████╔╝██║  ██║███████║   ██║   █████╗
// ██║   ██║██╔═══╝ ██║  ██║██╔══██║   ██║   ██╔══╝
// ╚██████╔╝██║     ██████╔╝██║  ██║   ██║   ███████╗
//  ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: update

//! `Update` (decisions 0293): puts what you changed in your programmer back where it came from.
//! `Update Preset <ids> [/Merge|/Add|/Overwrite]` writes a preset (`links::update_preset`); a bare
//! `Update` asks first — one station ask holding a tree: each preset your values came from with
//! the cues that follow it (untick a cue: it keeps its old look, an Extract runs first), and the
//! cues whose plain values you changed; Content and the two switches; the lines it will run.
//! Its Update button runs ``Update `<lines>` ``: the lines as one statement, so the whole update
//! is one Oops step. The ask starts with the choices of the last update (the profile's
//! `UpdateContent` / `UpdateRecast` / `UpdateShield`).
//!
//! Owns:          UpdatePreset, update_line (the bare word and the grouped run), the Update ask.
//! Doesn't own:   the writes (`links`), a cue's update (`commands::playback::update`).
//! Entry points:  UpdatePreset, update_line, ask.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

use std::collections::BTreeMap;

use lumacore_tree::{PresetMode, PresetUses};
use lumacore_wire::{ButtonKind, DialogTree, DialogTreeNode, Tone, TreeStatement};
use uuid::Uuid;

use super::links::{self, Content};
use crate::errors::render_code;
use crate::{AppState, DispatchResult};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The lanes the ask's statements run in: the Extracts before the updates they protect cues
/// from, then the presets, the Recasts that follow them, the cues last.
const LANE_EXTRACT: u8 = 0;
const LANE_PRESET: u8 = 1;
const LANE_RECAST: u8 = 2;
const LANE_CUE: u8 = 3;

/// The words a grouped update may run.
const GROUPED: [&str; 3] = ["Update", "Recast", "Extract"];

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Update Preset <ids> [/Merge | /Add | /Overwrite]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Update", object = "Preset")]
pub struct UpdatePreset {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(option = "Merge")]
    pub merge: bool,
    #[lum(option = "Add")]
    pub add: bool,
    #[lum(option = "Overwrite")]
    pub overwrite: bool,
}

impl crate::commands::typed::Command for UpdatePreset {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let content = match (self.merge, self.add, self.overwrite) {
            (_, true, false) => Content::Add,
            (_, false, true) => Content::Overwrite,
            (_, true, true) => return out_err(render_code("Update Preset takes one of /Merge, /Add, /Overwrite.", "E-CMD-006")),
            _ => Content::Merge,
        };
        let picked = match crate::resolve::resolve_one(state, &self.target, &super::super::PRESET) {
            Ok(p) => p,
            Err(e) => return out_err(e),
        };
        let dp = picked.datapool.unwrap_or_else(|| state.selected_datapool());
        if picked.members.is_empty() {
            return out_err(render_code("No such preset.", "E-CMD-007"));
        }
        let replies: Vec<crate::Reply> =
            picked.members.iter().map(|m| links::update_preset(state, dp, m.no, content).map_or_else(|e| e.into(), |m| m.into())).collect();
        DispatchResult::Output(crate::Reply::joined(replies))
    }
}

/// The bare `Update` row: alone it asks (`/NoConfirmation`: the ask's lines as they start);
/// with a line in backticks it runs those lines as one statement — one Oops step.
pub fn update_line(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let confirmed = crate::commands::flags::has(args, &crate::commands::flags::NO_CONFIRMATION);
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    match words.as_slice() {
        [] => match ask(state) {
            Ok(ask) if confirmed => match ask.primary_line() {
                Some(line) => crate::commands::dispatch(state, &line),
                None => out_err(render_code("Update: nothing to update.", "E-CMD-009")),
            },
            Ok(ask) => DispatchResult::Ask(Box::new(ask)),
            Err(e) => out_err(e),
        },
        [line] if line.starts_with('`') => grouped(state, line.trim_matches('`')),
        _ => crate::commands::run::objects_usage(crate::objects::Verb::Update),
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// ``Update `a; b; c` ``: each statement (an Update, Recast or Extract) runs inside this one line,
/// its question answered; the first refusal stops the rest. The choices the lines carry are
/// remembered on the profile.
fn grouped(state: &mut AppState, lines: &str) -> DispatchResult {
    let parsed = match crate::commands::line::Line::parse(lines) {
        Ok(p) => p,
        Err(e) => return out_err(render_code(format!("Update: {e:?}"), "E-CMD-006")),
    };
    let statements: Vec<String> = parsed.statements.iter().map(|s| s.text.trim().to_string()).filter(|s| !s.is_empty()).collect();
    if statements.is_empty() {
        return out_err(render_code("Update: nothing ticked, nothing to update.", "E-CMD-009"));
    }
    for s in &statements {
        let first = s.split_whitespace().next().unwrap_or_default();
        let ok = GROUPED.iter().any(|w| w.eq_ignore_ascii_case(first)) && !s.contains('`');
        if !ok {
            return out_err(render_code(format!("Update runs Update, Recast and Extract lines only, not `{s}`."), "E-CMD-008"));
        }
    }
    remember(state, &statements);
    let mut replies = Vec::new();
    for s in &statements {
        match crate::commands::dispatch(state, &format!("{s} /NoConfirmation")) {
            DispatchResult::Output(r) => {
                let failed = r.failed;
                replies.push(r);
                if failed {
                    break;
                }
            }
            DispatchResult::Ask(_) => {
                replies.push(crate::Reply::error(format!("`{s}` asked a question inside Update"), "E-CMD-010"));
                break;
            }
            _ => {}
        }
    }
    DispatchResult::Output(crate::Reply::joined(replies))
}

/// The choices a grouped update's lines carry, kept on the acting user's profile for the next
/// ask: the Content of its preset updates, Recast (when the Content could add), Shield (when a
/// cue was updated).
fn remember(state: &mut AppState, statements: &[String]) {
    let flag = |s: &str, f: &str| s.split_whitespace().any(|w| w.eq_ignore_ascii_case(&format!("/{f}")));
    let starts = |s: &str, w: &str| s.split_whitespace().next().is_some_and(|x| x.eq_ignore_ascii_case(w));
    let presets: Vec<&String> = statements.iter().filter(|s| starts(s, "Update") && s.contains("Preset ")).collect();
    let cues: Vec<&String> = statements.iter().filter(|s| starts(s, "Update") && s.contains(" Cue ")).collect();
    let content = presets.first().and_then(|s| Content::ALL.into_iter().find(|c| flag(s, c.word())));
    let recast = (content.is_some_and(|c| c != Content::Merge)).then(|| statements.iter().any(|s| starts(s, "Recast")));
    let shield = (!cues.is_empty()).then(|| cues.iter().any(|s| flag(s, "Shield")));
    let user = state.effective_user_name().to_string();
    let Some(n) = state.root.show_data.profile_id_for_user(&user) else { return };
    let Some(profile) = state.root.show_data.profile_mut(n) else { return };
    let choices = &mut profile.settings.preferences.update;
    if let Some(c) = content {
        choices.content = c.choice().to_string();
    }
    if let Some(r) = recast {
        choices.recast = r;
    }
    if let Some(s) = shield {
        choices.shield = s;
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The Update ask for the acting user's programmer (decisions 0293, mockup round 3).
pub fn ask(state: &AppState) -> Result<crate::dialogs::Ask, String> {
    let show = &state.root.show_data;
    let user = state.effective_user_name();
    let prog = state.current_programmer().ok_or_else(|| render_code("No programmer available.", "E-CMD-011"))?;
    let choices = show.profile_for_user(user).map(|p| p.settings.preferences.update.clone()).unwrap_or_default();
    let uses = PresetUses::build(show);

    // ----- The presets the values came from, and whether the programmer changed them -----
    let mut presets: BTreeMap<(u32, u32), (Uuid, bool, usize)> = BTreeMap::new();
    for ((f, a), pv) in &prog.values {
        let Some(g) = pv.origin.filter(|_| pv.active && !pv.value.release) else { continue };
        let Some((dp, no, p)) = show.preset_by_guid(g) else { continue };
        let changed = pv.value.preset != Some(g) || p.value_for(*f, a) != Some(&links::plain(&pv.value));
        let e = presets.entry((dp, no)).or_insert((g, false, 0));
        e.1 |= changed;
        e.2 += 1;
    }

    // ----- The cues whose plain values the programmer changed (running sequences) -----
    let mut cues: Vec<(u32, u32, u32, String, Vec<String>, Vec<u32>)> = Vec::new();
    for (guid, st) in &show.playbacks.live.sequences {
        let Some((dp, no, seq)) = show.sequence_by_guid(*guid) else { continue };
        let tracked = seq.tracked(st.current_cue);
        let mut attrs: Vec<String> = Vec::new();
        let mut from: Vec<u32> = Vec::new();
        for ((f, a), pv) in &prog.values {
            if !pv.active || pv.value.release || pv.origin.is_some() || pv.value.preset.is_some() {
                continue;
            }
            let Some(t) = tracked.get(&(*f, a.as_str())) else { continue };
            let Some(stored) = t.value.plain() else { continue };
            if *stored == links::plain(&pv.value) {
                continue;
            }
            if !attrs.contains(a) {
                attrs.push(a.clone());
            }
            if !from.contains(&t.cue) {
                from.push(t.cue);
            }
        }
        if !attrs.is_empty() {
            let name = seq.cue(st.current_cue).and_then(|c| c.meta.name()).unwrap_or_default().to_string();
            cues.push((dp, no, st.current_cue, name, attrs, from));
        }
    }
    cues.sort();
    if presets.is_empty() && cues.is_empty() {
        return Err(render_code("Update: your programmer holds nothing that came from a preset or a running cue.", "E-CMD-009"));
    }

    // ----- The tree -----
    let plural = |n: usize, one: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {one}s") };
    let mut nodes: Vec<DialogTreeNode> = Vec::new();
    for (&(dp, no), &(g, changed, _)) in &presets {
        let p = show.datapools.get(&dp).and_then(|d| d.presets.get(&no)).expect("found above");
        let id = format!("p{dp}_{no}");
        let line = links::preset_line(state, dp, no);
        let following = uses.cues(g);
        nodes.push(DialogTreeNode {
            id: id.clone(),
            depth: 0,
            kind: "preset".into(),
            no: format!("P{no}"),
            name: p.meta.name().unwrap_or_default().to_string(),
            badge: level(&p.mode).into(),
            note: if following.is_empty() { "no cues use it".into() } else { plural(following.len(), "cue") },
            on: changed,
            tickable: true,
            when_on: vec![
                TreeStatement { lane: LANE_PRESET, text: format!("Update {line} {{content}}") },
                // Recast only when the update can bring attributes (Add, Overwrite) and its switch is on.
                TreeStatement { lane: LANE_RECAST, text: format!("{{?recast}}{{?content!=/Merge}}Recast {line}") },
            ],
            ..DialogTreeNode::default()
        });
        for u in following {
            let cue = links::cue_line(state, u.datapool, u.sequence, u.cue);
            nodes.push(DialogTreeNode {
                id: format!("c{dp}_{no}_{}_{}_{}", u.datapool, u.sequence, u.cue),
                depth: 1,
                kind: "cue".into(),
                no: cue_no(u.cue),
                name: u.name.clone(),
                badge: format!("Seq {}", u.sequence),
                note: "follows".into(),
                note_off: "keeps its old look".into(),
                on: true,
                tickable: true,
                when_off: vec![TreeStatement { lane: LANE_EXTRACT, text: format!("{{?{id}}}Extract {line} {cue}") }],
                ..DialogTreeNode::default()
            });
        }
    }
    if !cues.is_empty() {
        nodes.push(DialogTreeNode {
            id: "plain".into(),
            kind: "dir".into(),
            name: "Cues with plain values".into(),
            note: plural(cues.len(), "cue"),
            ..DialogTreeNode::default()
        });
        for (dp, seq, cue, name, attrs, from) in &cues {
            let stored = match from.as_slice() {
                [one] if one != cue => format!(" · stored in {}", cue_no(*one)),
                _ => String::new(),
            };
            nodes.push(DialogTreeNode {
                id: format!("q{dp}_{seq}_{cue}"),
                depth: 1,
                kind: "plaincue".into(),
                no: cue_no(*cue),
                name: name.clone(),
                badge: format!("Seq {seq}"),
                note: format!("{}{stored}", attrs.join(" · ")),
                on: true,
                tickable: true,
                when_on: vec![TreeStatement { lane: LANE_CUE, text: format!("Update {} /Original {{shield}}", links::cue_line(state, *dp, *seq, *cue)) }],
                ..DialogTreeNode::default()
            });
        }
    }
    let tree = DialogTree { nodes };

    // ----- The ask -----
    let content = Content::parse(&choices.content).unwrap_or_default();
    let choices_list: Vec<(&str, String)> = Content::ALL.iter().map(|c| (c.choice(), format!("/{}", c.word()))).collect();
    let choices_ref: Vec<(&str, &str)> = choices_list.iter().map(|(l, t)| (*l, t.as_str())).collect();
    let mut ask = crate::dialogs::Ask::new(
        "Update",
        "Your programmer changed values that came from these presets and cues. The cues under a preset follow it. \
         Tick what to update; untick a cue to keep its old look.",
    )
    .meta(format!("{user} · your programmer"), None)
    .tree(tree)
    .selector("content", "Content", &choices_ref, content.choice())
    .toggle("recast", "Add new attributes to the cues that follow (Recast)", choices.recast, "")
    .toggle("shield", "Protect the next cues", choices.shield, "/Shield");
    ask = ask
        .cancel("Cancel")
        .button("Preview in Blind", ButtonKind::Plain, "Update")
        .disabled("Preview in Blind isn't built yet (planned): there is no Blind output to show the result on.")
        .button("Update", ButtonKind::Primary, "Update `{tree}`")
        .note("one Oops step")
        .flags("/NoConfirmation");
    // The lines it runs as the ask starts (a client redraws them as you tick).
    for line in ask.line_of("Update").map(|l| statements_of(&l)).unwrap_or_default() {
        ask = ask.detail(line, Some(Tone::Ok));
    }
    Ok(ask)
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

fn out_err(msg: String) -> DispatchResult {
    DispatchResult::Output(msg.into())
}

/// A preset's level badge (Selective, Global, Universal).
fn level(mode: &PresetMode) -> &'static str {
    match mode {
        PresetMode::Selective => "S",
        PresetMode::Global => "G",
        PresetMode::Universal => "U",
    }
}

/// A cue's number as shown (`Cue 2.5`, `Cue 0` for CueZero).
fn cue_no(key: u32) -> String {
    if key == lumacore_tree::CUE_ZERO { "Cue 0".into() } else { format!("Cue {}", lumacore_tree::cue_label(key)) }
}

/// The statements inside ``Update `…` ``.
fn statements_of(line: &str) -> Vec<String> {
    let inner = line.split_once('`').and_then(|(_, rest)| rest.rsplit_once('`')).map_or("", |(inner, _)| inner);
    inner.split("; ").map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect()
}
