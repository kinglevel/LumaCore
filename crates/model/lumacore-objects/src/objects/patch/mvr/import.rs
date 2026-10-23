// ██╗███╗   ███╗██████╗  ██████╗ ██████╗ ████████╗
// ██║████╗ ████║██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝
// ██║██╔████╔██║██████╔╝██║   ██║██████╔╝   ██║
// ██║██║╚██╔╝██║██╔═══╝ ██║   ██║██╔══██╗   ██║
// ██║██║ ╚═╝ ██║██║     ╚██████╔╝██║  ██║   ██║
// ╚═╝╚═╝     ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝
// FILE: import

//! `Import Mvr "file" [/Drive "usb1"] [/Scope <target>] [/Match …] [/Lost …] [/IgnoreRotation]
//! [/FidOffset n] [/UniverseOffset n] [/X /Y /Z mm] [/Rotate °] [/Addresses …] … — the file read
//! against the holder's draft, the review opened on it (decisions §34 "Import MVR hardened" 1–3);
//! `Import Mvr /Cancel` drops the review. A bare `Import Mvr` asks for the file with the files
//! element. `/Apply`, a cell's `/Take` / `/Offer` and the placement asks come with part 4b.
//!
//! Owns:          ImportMvr, the settings from the preset and the line, the scope from the resolver.
//! Doesn't own:   the review's rule (review.rs), the file (`lumacore-mvr`), the draft.
//! Entry points:  ImportMvr.
//! Decisions:     §34 "Import MVR hardened" 1–3.

use std::collections::BTreeMap;

use lumacore_syntax::ast::{Id, Item, Range, Target, Value};
use lumacore_tree::{MvrLost, MvrMatch, MvrRotation};

use super::review::{self, Scope};
use super::{Settings, take_mode, whole, word};
use crate::errors::render_code;
use crate::objects::{out, patch_claimed};
use crate::{AppState, DispatchResult};
//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS


/// `Import Mvr …`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Import", object = "Mvr")]
pub struct ImportMvr {
    #[lum(target)]
    pub target: Option<Target>,
    #[lum(option = "Drive")]
    pub drive: Option<Value>,
    #[lum(option = "Scope")]
    pub scope: Option<Value>,
    #[lum(option = "Match")]
    pub match_key: Option<Value>,
    #[lum(option = "Lost")]
    pub lost: Option<Value>,
    #[lum(option = "IgnoreRotation")]
    pub ignore_rotation: bool,
    #[lum(option = "IdType")]
    pub id_type: Option<Value>,
    #[lum(option = "FidOffset")]
    pub fid_offset: Option<Value>,
    #[lum(option = "UniverseOffset")]
    pub universe_offset: Option<Value>,
    #[lum(option = "X")]
    pub x: Option<Value>,
    #[lum(option = "Y")]
    pub y: Option<Value>,
    #[lum(option = "Z")]
    pub z: Option<Value>,
    #[lum(option = "Rotate")]
    pub rotate: Option<Value>,
    #[lum(option = "Addresses")]
    pub addresses: Option<Value>,
    #[lum(option = "Ids")]
    pub ids: Option<Value>,
    #[lum(option = "Names")]
    pub names: Option<Value>,
    #[lum(option = "Position")]
    pub position: Option<Value>,
    #[lum(option = "Revisions")]
    pub revisions: Option<Value>,
    #[lum(option = "NewFixtures")]
    pub new_fixtures: Option<Value>,
    #[lum(option = "Apply")]
    pub apply: bool,
    #[lum(option = "Cancel")]
    pub cancel: bool,
    #[lum(option = "Take")]
    pub take: Option<Value>,
    #[lum(option = "Offer")]
    pub offer: Option<Value>,
    #[lum(option = "Gone")]
    pub gone: Option<Value>,
    #[lum(option = "Group")]
    pub group: Option<Value>,
}

impl crate::commands::typed::Command for ImportMvr {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if self.cancel {
            return out(cancel(state));
        }
        if self.apply || self.take.is_some() || self.offer.is_some() {
            if let Err(e) = patch_claimed(state) {
                return e;
            }
            return if self.apply { super::apply::apply(state, &self) } else { super::apply::take_cell(state, &self) };
        }
        let Some(file) = self.target.as_ref().and_then(file_name) else {
            // The station's "which file?" ask (decisions §38 5): the files element on mvr/.
            state.asking = Some(
                crate::dialogs::Ask::new("Import MVR", "An MVR file of the library against your Devices draft: the review opens in the patch grid, nothing is written until Apply.")
                    .file("file", "File", "mvr")
                    .button("Review", lumacore_wire::ButtonKind::Primary, "Import Mvr {file}")
                    .cancel("Cancel")
                    .flags("the file: Import Mvr \"file.mvr\" [/Drive \"usb1\"]"),
            );
            return out(render_code("Import Mvr \"file.mvr\": a file of library/mvr/ (Claim Root.ShowData.Devices first).", "E-CMD-006"));
        };
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        out(import(state, &self, &file).unwrap_or_else(|e| e))
    }
}
// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC


/// The quoted file name of the target, if that is what it is.
fn file_name(target: &Target) -> Option<String> {
    match target.items.as_slice() {
        [(None, Item::Path(segs))] => segs.last().and_then(|s| s.ids.as_ref()).and_then(|ids| match ids.ranges.as_slice() {
            [(None, Range::One(Id::Name(n)))] => Some(n.node.clone()),
            _ => None,
        }),
        [(None, Item::Ids(ids))] => match ids.ranges.as_slice() {
            [(None, Range::One(Id::Name(n)))] => Some(n.node.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn cancel(state: &mut AppState) -> String {
    let Some(review) = state.patch_draft.as_mut().and_then(|d| d.review.take()) else {
        return render_code("No MVR review to cancel (Import Mvr \"file\" opens one).", "E-CMD-009");
    };
    state.show_generation += 1;
    crate::text::render_success(&format!("Import review of \"{}\" cancelled; the draft as it was.", review.file))
}

/// The settings: the user's preset, then the line's options.
fn settings(state: &AppState, cmd: &ImportMvr) -> Result<Settings, String> {
    let preset = state
        .root
        .show_data
        .profile_id_for_user(state.effective_user_name())
        .and_then(|no| state.root.show_data.user_profiles.get(&no))
        .map(|p| p.settings.preferences.mvr.clone())
        .unwrap_or_default();
    let mut s = Settings::from_preset(&preset);
    if let Some(v) = &cmd.match_key {
        s.match_key = match word(v).to_ascii_lowercase().as_str() {
            "auto" => MvrMatch::Auto,
            "uuid" => MvrMatch::Uuid,
            "fid" => MvrMatch::Fid,
            "fixturename" => MvrMatch::FixtureName,
            other => return Err(render_code(format!("/Match takes Auto, UUID, FID or FixtureName, not {other}."), "E-CMD-008")),
        };
    }
    if let Some(v) = &cmd.lost {
        s.lost = match word(v).to_ascii_lowercase().as_str() {
            "mark" => MvrLost::Mark,
            "delete" => MvrLost::Delete,
            other => return Err(render_code(format!("/Lost takes Mark or Delete, not {other}."), "E-CMD-008")),
        };
    }
    if cmd.ignore_rotation {
        s.rotation = MvrRotation::Ignore;
    }
    if let Some(v) = &cmd.id_type {
        s.id_type = lumacore_fixtures::IdType::from_name(&word(v)).filter(|t| *t != lumacore_fixtures::IdType::None).ok_or_else(|| render_code(format!("/IdType takes one of {}, not {}.", lumacore_fixtures::IdType::NAMES[1..].join(", "), word(v)), "E-CMD-008"))?;
    }
    if let Some(v) = &cmd.fid_offset {
        s.fid_offset = whole(v, "FidOffset")?;
    }
    if let Some(v) = &cmd.universe_offset {
        s.universe_offset = whole(v, "UniverseOffset")?;
    }
    for (i, v) in [&cmd.x, &cmd.y, &cmd.z].into_iter().enumerate() {
        if let Some(v) = v {
            s.offset_mm[i] = whole(v, ["X", "Y", "Z"][i])?;
        }
    }
    if let Some(v) = &cmd.rotate {
        s.rotate_deg = whole(v, "Rotate")?;
    }
    for (v, slot, what) in [
        (&cmd.addresses, &mut s.addresses, "Addresses"),
        (&cmd.ids, &mut s.ids, "Ids"),
        (&cmd.names, &mut s.names, "Names"),
        (&cmd.position, &mut s.position, "Position"),
        (&cmd.revisions, &mut s.revisions, "Revisions"),
        (&cmd.new_fixtures, &mut s.new_fixtures, "NewFixtures"),
    ] {
        if let Some(v) = v {
            *slot = take_mode(v, what)?;
        }
    }
    Ok(s)
}

/// `/Scope <target>` through the one resolver: an MVR layer or group, a class, a tag.
fn scope(state: &AppState, v: &Value) -> Result<(Scope, String), String> {
    let Value::Target(target) = v else {
        return Err(render_code("/Scope takes an MvrLayer, MvrGroup, MvrClass or Tag (MvrGroup 3, Tag \"Truss\").", "E-CMD-008"));
    };
    let resolved = crate::resolve::resolve_one(state, target, &super::MVR)?;
    let member = resolved.members.first().ok_or_else(|| render_code("/Scope names nothing.", "E-CMD-007"))?;
    let guid = member.guid.ok_or_else(|| render_code(format!("{} {} has no GUID to scope by.", resolved.object.keyword, member.no), "E-CMD-008"))?;
    let text = format!("{} {}", resolved.object.keyword, member.name.as_deref().map(|n| format!("\"{n}\"")).unwrap_or_else(|| member.no.to_string()));
    let scope = match resolved.object.keyword {
        "MvrLayer" | "MvrGroup" => Scope::Node(guid), // tidy: not-keywords (the scope's kinds, found by the one resolver)
        "MvrClass" => Scope::Class(guid),
        "Tag" => Scope::Tag(guid),
        other => return Err(render_code(format!("/Scope takes an MvrLayer, MvrGroup, MvrClass or Tag, not {other}."), "E-CMD-008")),
    };
    Ok((scope, text))
}

fn import(state: &mut AppState, cmd: &ImportMvr, file: &str) -> Result<String, String> {
    let drive = cmd.drive.as_ref().map(word);
    let folder = state.library_folder(drive.as_deref(), "mvr", false)?;
    let path = lumacore_luma::drive::library_file(&folder, file).map_err(|e| render_code(e, "E-CMD-008"))?;
    let path = if path.extension().is_some() { path } else { path.with_extension("mvr") };
    let open = || lumacore_luma::drive::open(&path).map(std::io::BufReader::new).map_err(|e| render_code(format!("No MVR \"{file}\" in {folder}/: {e}"), "E-CMD-012"));
    let mvr = lumacore_mvr::read_from(open()?).map_err(|e| render_code(format!("\"{file}\": {e}"), "E-CMD-008"))?;
    let s = settings(state, cmd)?;
    let scope = cmd.scope.as_ref().map(|v| scope(state, v)).transpose()?;
    // The GDTFs the zip holds, hashed: a type the show has is known by its revision's hash.
    let mut hashes: BTreeMap<String, String> = BTreeMap::new();
    for spec in mvr.head().gdtf_named {
        if mvr.has_gdtf(&spec)
            && let Ok(Some(bytes)) = lumacore_mvr::gdtf_entry(open()?, &spec)
        {
            hashes.insert(spec, lumacore_gdtf::file_hash(&bytes));
        }
    }
    let review = review::build(&state.root.show_data.devices, &mvr, &s, scope.as_ref().map(|(s, _)| s), &hashes, file, drive.as_deref(), scope.as_ref().map(|(_, t)| t.as_str()));
    let summary = review.summary();
    let replaced = state.patch_draft.as_mut().and_then(|d| d.review.replace(review)).map(|r| r.file);
    state.show_generation += 1;
    Ok(crate::text::render_success(&format!(
        "Review {summary}{} — List Mvr shows it; Import Mvr /Apply writes the ticked rows (part 4b), /Cancel drops it.",
        replaced.map(|f| format!(" (the review of \"{f}\" replaced)")).unwrap_or_default()
    )))
}
