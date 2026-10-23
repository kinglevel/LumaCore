// ██╗   ██╗██████╗  ██████╗ ██████╗  █████╗ ██████╗ ███████╗
// ██║   ██║██╔══██╗██╔════╝ ██╔══██╗██╔══██╗██╔══██╗██╔════╝
// ██║   ██║██████╔╝██║  ███╗██████╔╝███████║██║  ██║█████╗
// ██║   ██║██╔═══╝ ██║   ██║██╔══██╗██╔══██║██║  ██║██╔══╝
// ╚██████╔╝██║     ╚██████╔╝██║  ██║██║  ██║██████╔╝███████╗
//  ╚═════╝ ╚═╝      ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: upgrade

//! Older show files upgrade step by step (decisions 0289): one step per format bump, each
//! rewriting the show's JSON (the tree `decode` put together again) from format N to N+1 before it
//! is parsed, and saying what it changed. A show older than `OLDEST_FORMAT` or newer than this
//! build is refused (E-SHOW-002 / 001).
//!
//! Owns:          OLDEST_FORMAT, STEPS, readable, upgrade, Upgraded.
//! Doesn't own:   the container (`show_file`), loading into a station (lumacore-state).
//! Entry points:  OLDEST_FORMAT, readable, upgrade, Upgraded.
//! Decisions:     docs/decisions/0289-show-files-upgrade-step-by-step.

use serde_json::{Map, Value, json};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The oldest show format this build reads (raising it deletes the steps below it).
pub const OLDEST_FORMAT: u32 = 31;

/// One format bump: the show's JSON from format `from` to `from + 1`; what it changed.
pub struct Step {
    pub from: u32,
    pub run: fn(&mut Value) -> Vec<String>,
}

/// Every step from `OLDEST_FORMAT` to `FORMAT_VERSION`, in order.
pub static STEPS: &[Step] = &[
    // 31 → 32: the theme's Tint goes; BackgroundHue and BackgroundSaturation replace it (0286).
    Step { from: 31, run: tint_gives_way },
    // 32 → 33: view buttons per screen, holding any object (decisions 0287).
    Step { from: 32, run: view_buttons_per_screen },
    // 33 → 34: one preset pool per DataPool (decisions 0288).
    Step { from: 33, run: one_preset_pool },
    // 34 → 35: a user's NAV bookmarks and a profile's NavLog, both new with their defaults
    // (decisions 0291).
    Step { from: 34, run: nav_bookmarks_and_log },
    // 35 → 36: a cue value is plain steps or a preset link (decisions 0293).
    Step { from: 35, run: cue_values_link_presets },
    // 36 → 37: a Paint is `#rrggbb` or a theme slot link; feature groups follow their slot
    // (decisions object-paint-palette-free-theme-links).
    Step { from: 36, run: paint_is_hex_or_a_theme_link },
];

/// What an upgrade did: the format it came from, this build's, and each step's notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upgraded {
    pub from: u32,
    pub to: u32,
    pub notes: Vec<String>,
}

impl Upgraded {
    /// One line for a reply or the System Monitor.
    pub fn line(&self) -> String {
        let steps = self.to - self.from;
        format!("saved in format {}, upgraded to {} ({steps} step{})", self.from, self.to, if steps == 1 { "" } else { "s" })
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Whether this build reads a file of `format`: Ok, or the refusal and its code.
pub fn readable(format: u64) -> Result<(), (String, &'static str)> {
    let ours = u64::from(lumacore_tree::FORMAT_VERSION);
    if format > ours {
        return Err((format!("saved in format {format}; this build reads formats {OLDEST_FORMAT} to {ours}"), "E-SHOW-001"));
    }
    if format < u64::from(OLDEST_FORMAT) {
        return Err((format!("saved in format {format}; this build reads formats {OLDEST_FORMAT} to {ours} — older shows can't be upgraded"), "E-SHOW-002"));
    }
    Ok(())
}

/// Upgrade a show's JSON to this build's format, step by step. None when it is this build's
/// already; an error when it can't be read (`readable`).
pub fn upgrade(show: &mut Value) -> Result<Option<Upgraded>, String> {
    let from = show["show_settings"]["format_version"].as_u64().unwrap_or(0);
    readable(from).map_err(|(msg, code)| lumacore_base::with_code(format!("This show was {msg}."), code))?;
    let to = lumacore_tree::FORMAT_VERSION;
    if from == u64::from(to) {
        return Ok(None);
    }
    let from = from as u32;
    let mut notes = Vec::new();
    for step in STEPS.iter().filter(|s| s.from >= from && s.from < to) {
        notes.extend((step.run)(show));
    }
    show["show_settings"]["format_version"] = json!(to);
    Ok(Some(Upgraded { from, to, notes }))
}

/// 31 → 32: a profile theme's Tint adjuster is gone (decisions 0286: the background's hue and
/// saturation replace it, both start at 0); a profile whose Tint wasn't 0 is named.
fn tint_gives_way(show: &mut Value) -> Vec<String> {
    let mut notes = Vec::new();
    for profile in members(show, "user_profiles") {
        let name = profile["meta"]["name"].as_str().unwrap_or("a profile").to_string();
        let Some(adjust) = profile["settings"]["theme"]["adjust"].as_object_mut() else { continue };
        let tint = adjust.remove("tint").and_then(|t| t.as_i64()).unwrap_or(0);
        adjust.entry("background_hue").or_insert(json!(0));
        adjust.entry("background_saturation").or_insert(json!(0));
        if tint != 0 {
            notes.push(format!("{name}: the theme's Tint {tint} is gone — BackgroundHue and BackgroundSaturation replace it"));
        }
    }
    notes
}

/// 32 → 33: a profile's view buttons were one set, slot → view GUID; now they are per screen and
/// hold an object by kind. The old set becomes screen 1's, each a view.
fn view_buttons_per_screen(show: &mut Value) -> Vec<String> {
    let mut moved = 0;
    for profile in members(show, "user_profiles") {
        let content = &mut profile["content"];
        let Some(old) = content.get("view_buttons").and_then(Value::as_object).cloned() else { continue };
        let screen: Map<String, Value> = old
            .into_iter()
            .filter_map(|(slot, guid)| guid.as_str().map(|g| (slot, json!({ "kind": "View", "guid": g }))))
            .collect();
        moved += screen.len();
        content["view_buttons"] = if screen.is_empty() { json!({}) } else { json!({ "1": screen }) };
    }
    if moved == 0 { Vec::new() } else { vec![format!("{moved} view buttons are screen 1's now")] }
}

/// 33 → 34: a DataPool's preset groups become its one preset pool — every preset of every group,
/// in group then preset order, numbered from 1; a profile's selected preset group goes.
fn one_preset_pool(show: &mut Value) -> Vec<String> {
    let mut notes = Vec::new();
    for (dp, pool) in members_keyed(show, "datapools") {
        let Some(groups) = pool.as_object_mut().and_then(|p| p.remove("preset_groups")) else { continue };
        let mut presets = Map::new();
        let mut renumbered = Vec::new();
        for (g, group) in sorted(&groups) {
            for (p, preset) in sorted(&group["presets"]) {
                let no = presets.len() + 1;
                renumbered.push(format!("{g}.{p} → {no}"));
                presets.insert(no.to_string(), preset.clone());
            }
        }
        let existing = pool.get("presets").and_then(Value::as_object).is_some_and(|m| !m.is_empty());
        if !existing {
            pool["presets"] = Value::Object(presets);
        }
        if !renumbered.is_empty() {
            let shown: Vec<&str> = renumbered.iter().take(8).map(String::as_str).collect();
            let more = renumbered.len().saturating_sub(shown.len());
            notes.push(format!(
                "DataPool {dp}: presets in one pool (Preset {}{})",
                shown.join(", "),
                if more > 0 { format!(", and {more} more") } else { String::new() }
            ));
        }
    }
    for profile in members(show, "user_profiles") {
        if let Some(content) = profile["content"].as_object_mut() {
            content.remove("selected_preset_group");
        }
    }
    notes
}

/// 34 → 35: nothing saved changes shape — a user's `bookmarks` and a profile's
/// `preferences.nav_log` are new and read as their defaults (none; On).
fn nav_bookmarks_and_log(_show: &mut Value) -> Vec<String> {
    Vec::new()
}

/// 35 → 36: a cue value becomes `{"Value": …}` (plain steps) or `{"Link": "<preset GUID>"}`
/// (decisions 0293). A value that carried the preset it came from keeps only the link — the cue
/// now plays the preset as it is; every other value keeps its steps. (A programmer value's new
/// `origin` reads as none.)
fn cue_values_link_presets(show: &mut Value) -> Vec<String> {
    let (mut linked, mut sequences) = (0, 0);
    for (_, pool) in members_keyed(show, "datapools") {
        let Some(seqs) = pool.get_mut("sequences").and_then(Value::as_object_mut) else { continue };
        for seq in seqs.values_mut() {
            let before = linked;
            let mut cues: Vec<&mut Value> = Vec::new();
            let Some(obj) = seq.as_object_mut() else { continue };
            for (k, v) in obj.iter_mut() {
                match k.as_str() {
                    "cue_zero" | "off_cue" => cues.push(v),
                    "cues" => cues.extend(v.as_object_mut().into_iter().flat_map(|m| m.values_mut())),
                    _ => {}
                }
            }
            for cue in cues {
                let Some(parts) = cue.get_mut("parts").and_then(Value::as_object_mut) else { continue };
                for part in parts.values_mut() {
                    let Some(values) = part.get_mut("values").and_then(Value::as_array_mut) else { continue };
                    for v in values {
                        let Some(old) = v.get_mut("value") else { continue };
                        let preset = old.get("preset").and_then(Value::as_str).map(str::to_string);
                        let release = old.get("release").and_then(Value::as_bool).unwrap_or(false);
                        *old = match preset {
                            Some(g) if !release => {
                                linked += 1;
                                json!({ "Link": g })
                            }
                            _ => {
                                let mut plain = old.take();
                                if let Some(m) = plain.as_object_mut() {
                                    m.remove("preset");
                                }
                                json!({ "Value": plain })
                            }
                        };
                    }
                }
            }
            if linked > before {
                sequences += 1;
            }
        }
    }
    if linked == 0 {
        Vec::new()
    } else {
        vec![format!(
            "{linked} cue value{} from presets now link{} to {} (in {sequences} sequence{}): the cues play the presets as they are",
            if linked == 1 { "" } else { "s" },
            if linked == 1 { "s" } else { "" },
            if linked == 1 { "its preset" } else { "their presets" },
            if sequences == 1 { "" } else { "s" }
        )]
    }
}

/// 36 → 37: every meta's `paint` was RGBA bytes and becomes `"#rrggbb"`; a feature group whose
/// colour is still its template's (LumaCore's colour of the slot of its name) links that theme slot
/// instead, so it follows each viewer's theme (decisions object-paint-palette-free-theme-links).
/// A tag's Priority, a theme's slots and the picker's mode and recent colours are new and read as
/// their defaults.
fn paint_is_hex_or_a_theme_link(show: &mut Value) -> Vec<String> {
    let (mut painted, mut linked) = (0, 0);
    hex_paints(show, &mut painted, &mut linked);
    let mut notes = Vec::new();
    if painted > 0 {
        notes.push(format!("{painted} paint{} now written #rrggbb", if painted == 1 { " is" } else { "s are" }));
    }
    if linked > 0 {
        notes.push(format!(
            "{linked} feature group{} now follow{} the theme's paint slot of {}",
            if linked == 1 { "" } else { "s" },
            if linked == 1 { "s" } else { "" },
            if linked == 1 { "its name" } else { "their names" }
        ));
    }
    notes
}

/// Walk the JSON: a meta's RGBA paint becomes hex; a feature group (a `groups` member with
/// `features`) still on its template colour becomes its slot's link.
fn hex_paints(v: &mut Value, painted: &mut usize, linked: &mut usize) {
    match v {
        Value::Object(map) => {
            if let Some(meta) = map.get_mut("meta").and_then(Value::as_object_mut)
                && let Some(hex) = bytes_hex(meta.get("paint"))
            {
                meta.insert("paint".into(), json!(hex));
                *painted += 1;
            }
            // The show's feature groups (`feature_groups`) and a profile's own (`attributes.groups`).
            for key in ["feature_groups", "groups"] {
                let Some(groups) = map.get_mut(key).and_then(Value::as_object_mut) else { continue };
                for g in groups.values_mut().filter(|g| g.get("features").is_some()) {
                    let Some(meta) = g.get_mut("meta").and_then(Value::as_object_mut) else { continue };
                    let hex = match bytes_hex(meta.get("paint")) {
                        Some(h) => {
                            *painted += 1;
                            h
                        }
                        None => meta.get("paint").and_then(Value::as_str).unwrap_or_default().to_string(),
                    };
                    let name = meta.get("name").and_then(Value::as_str).unwrap_or_default();
                    let slot = lumacore_tree::media::PAINT_SLOTS.iter().find(|(n, c)| n.eq_ignore_ascii_case(name) && c.eq_ignore_ascii_case(&hex));
                    match slot {
                        Some((n, _)) => {
                            meta.insert("paint".into(), json!({ "Theme": n }));
                            *linked += 1;
                        }
                        None if !hex.is_empty() => {
                            meta.insert("paint".into(), json!(hex));
                        }
                        None => {}
                    }
                }
            }
            for (key, child) in map.iter_mut() {
                if key != "meta" {
                    hex_paints(child, painted, linked);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|i| hex_paints(i, painted, linked)),
        _ => {}
    }
}

/// RGBA bytes (format 36's paint) as `#rrggbb`; None = not bytes.
fn bytes_hex(paint: Option<&Value>) -> Option<String> {
    let bytes = paint?.as_array().filter(|a| a.len() >= 3)?;
    let b = |i: usize| bytes[i].as_u64().unwrap_or(0).min(255) as u8;
    Some(lumacore_base::paint::hex_of([b(0), b(1), b(2)]))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The members of a numbered top-level pool (`user_profiles`, `datapools`).
fn members<'a>(show: &'a mut Value, field: &str) -> impl Iterator<Item = &'a mut Value> {
    show.get_mut(field).and_then(Value::as_object_mut).into_iter().flat_map(|m| m.values_mut())
}

fn members_keyed<'a>(show: &'a mut Value, field: &str) -> impl Iterator<Item = (String, &'a mut Value)> {
    show.get_mut(field).and_then(Value::as_object_mut).into_iter().flat_map(|m| m.iter_mut().map(|(k, v)| (k.clone(), v)))
}

/// A numbered map's entries in number order.
fn sorted(v: &Value) -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = v.as_object().map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default();
    out.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(u64::MAX));
    out
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
