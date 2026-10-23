// ███████╗███╗   ██╗ ██████╗ ██████╗ ██████╗ ███████╗██████╗
// ██╔════╝████╗  ██║██╔════╝██╔═══██╗██╔══██╗██╔════╝██╔══██╗
// █████╗  ██╔██╗ ██║██║     ██║   ██║██║  ██║█████╗  ██████╔╝
// ██╔══╝  ██║╚██╗██║██║     ██║   ██║██║  ██║██╔══╝  ██╔══██╗
// ███████╗██║ ╚████║╚██████╗╚██████╔╝██████╔╝███████╗██║  ██║
// ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝
// FILE: encoder

//! What a screen's encoder bar shows (decisions §30 "12f": the bar per screen above the command
//! line, Mode above Layer), as the `encoder` feed sends it to `<luma-encoder-bar>`. Pure: it reads
//! a published show, the live screen (its `encoder` state), the engine's plan (output readouts)
//! and the users' own Highlight / Solo masters. Every gesture on the bar is a command line
//! (`Set Screen "Encoder…"`, `Attribute "x" At | Fade | Delay`, `Master … At`).
//!
//! Owns:          `view` (the encoder bar: strip, encoders, masters).
//! Doesn't own:   `Set Screen`'s encoder settings (`lumacore_objects::commands::screens`), the merge (`lumacore_render::merge`).
//! Entry points:  view.
//! Decisions:     §30 "12f", §34 20.7.

use lumacore_tree::ShowData;
use lumacore_tree::runtime::{EncoderLayer, EncoderMode, Screen};
use serde_json::{Value, json};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Four encoders a page (round 9).
const PER_PAGE: usize = 4;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The bar for `user` on live `screen`. `plan` = the engine's (output readouts; None = defaults).
/// `own` = the user's own Highlight / Solo masters (`{"highlight": {level, mode}, "solo": …}`).
pub fn view(show: &ShowData, user: &str, screen: &Screen, plan: Option<&lumacore_render::Plan>, own: &Value) -> Value {
    let profile = show.profile_for_user(user);
    let setting = profile
        .and_then(|p| p.content.screen_configs.get(&screen.config))
        .and_then(|c| c.screens.get(&screen.screen))
        .map(|s| s.encoder_bar.clone())
        .unwrap_or_default();
    let e = &screen.encoder;
    let modes: Vec<Value> = EncoderMode::ALL
        .iter()
        .map(|m| json!({ "word": m.word(), "label": mode_label(*m), "planned": matches!(m, EncoderMode::Effects) }))
        .collect();
    let mut out = json!({
        "on": setting.shown(screen.screen),
        // Dim the interface behind a pop-up (the profile's PopupDim, decisions §30 "12f").
        "popup_dim": profile.is_none_or(|p| p.settings.preferences.popup_dim),
        "screen": screen.screen,
        "mode": e.mode.word(),
        "modes": modes,
        "layer": e.layer.word(),
        "layers": EncoderLayer::ALL.iter().map(|l| l.word()).collect::<Vec<_>>(),
        "group": e.group,
        "page": e.page,
    });
    let body = match e.mode {
        EncoderMode::Attributes => attributes(show, user, &setting, e.group, e.page, plan),
        EncoderMode::Masters => masters(show, user, own),
        EncoderMode::CueTimingVertical | EncoderMode::CueTimingHorizontal => cue_timing(show, user, e.group, e.page),
        EncoderMode::Effects => json!({ "planned": "Effects come with the effects build (decisions §28)" }),
    };
    if let (Some(o), Value::Object(b)) = (out.as_object_mut(), body) {
        o.extend(b);
    }
    out
}

fn mode_label(m: EncoderMode) -> &'static str {
    match m {
        EncoderMode::Attributes => "Attributes",
        EncoderMode::CueTimingVertical => "Cue timing · vertical",
        EncoderMode::CueTimingHorizontal => "Cue timing · horizontal",
        EncoderMode::Masters => "Masters",
        EncoderMode::Effects => "Effects",
    }
}

/// Percent text: `48%`, `48.5%`.
fn pct(v: f64) -> String {
    let r = (v * 10.0).round() / 10.0;
    if r.fract() == 0.0 { format!("{r:.0}%") } else { format!("{r:.1}%") }
}

/// The selection's values of one attribute: per fixture (that has it) the percent of its range,
/// whether the user's programmer holds it, and the programmer values' own timing.
struct Values {
    levels: Vec<f64>,
    readouts: Vec<String>,
    programmed: bool,
    fade: Vec<Option<f64>>,
    delay: Vec<Option<f64>>,
}

fn values_of(show: &ShowData, user: &str, attr: &str, plan: Option<&lumacore_render::Plan>) -> Values {
    let mut v = Values { levels: Vec::new(), readouts: Vec::new(), programmed: false, fade: Vec::new(), delay: Vec::new() };
    let Some(prog) = show.programmer_of(user) else { return v };
    for guid in &prog.selected_fixtures {
        let Some(fixture) = show.devices.dmx.patch.fixtures.values().find(|f| f.meta.identity.guid == *guid) else { continue };
        let Some(cf) = lumacore_render::merge::channels(show, fixture)
            .iter()
            .find(|cf| !cf.is_virtual && cf.attribute_name.eq_ignore_ascii_case(attr))
        else {
            continue;
        };
        let held = prog.values.get(&(*guid, cf.attribute_name.clone())).filter(|pv| pv.active);
        let value = match held {
            Some(pv) => {
                v.programmed = true;
                v.fade.push(pv.timing.fade);
                v.delay.push(pv.timing.delay);
                Some(pv.value.clone())
            }
            None => plan.and_then(|p| p.current_value(*guid, &cf.attribute_name)),
        };
        let fraction = value
            .as_ref()
            .and_then(|val| lumacore_render::merge::fraction(val, cf))
            .unwrap_or_else(|| lumacore_render::merge::default_fraction(cf));
        v.levels.push(fraction * 100.0);
        v.readouts.push(value.map_or_else(|| pct(fraction * 100.0), |val| val.readout()));
    }
    v
}

/// One timing layer's text: the values' own seconds when they agree, `mixed`, else `cue`.
fn timing_of(seconds: &[Option<f64>]) -> Value {
    let own: Vec<f64> = seconds.iter().flatten().copied().collect();
    match own.first() {
        None => json!({ "text": "cue", "value": Value::Null }),
        Some(&first) if own.len() == seconds.len() && own.iter().all(|s| (*s - first).abs() < 1e-9) => {
            json!({ "text": format!("{}s", trim(first)), "value": first })
        }
        Some(&first) => json!({ "text": "mixed", "value": first }),
    }
}

fn trim(s: f64) -> String {
    let r = (s * 100.0).round() / 100.0;
    let t = format!("{r:.2}");
    t.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn attributes(
    show: &ShowData,
    user: &str,
    setting: &lumacore_tree::programmer::EncoderBarSetting,
    group: u32,
    page: u32,
    plan: Option<&lumacore_render::Plan>,
) -> Value {
    let Some(profile) = show.profile_for_user(user) else {
        return json!({ "strip": [], "encoders": [], "note": "No user profile." });
    };
    let attrs = &profile.content.attributes;
    let layout = setting
        .layout
        .and_then(|g| attrs.encoder_bars.iter().find(|(_, b)| b.meta.identity.guid == g))
        .or_else(|| attrs.encoder_bars.iter().next());
    let Some((&layout_no, layout)) = layout else {
        return json!({ "strip": [], "encoders": [], "note": "No encoder bar layout: Store UserProfile EncoderBar." });
    };
    let selected = show.programmer_of(user).map_or(0, |p| p.selected_fixtures.len());
    let paint = lumacore_objects::paint::Painter::new(show, user);
    let strip: Vec<Value> = layout
        .buttons
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let g = attrs.groups.values().find(|g| g.meta.identity.guid == b.group);
            let names: Vec<&str> = b.slots.iter().flatten().map(String::as_str).collect();
            // The mini: the first attribute's level per selected fixture.
            let first = names.first().map(|a| values_of(show, user, a, plan));
            let programmed = names.iter().any(|a| values_of(show, user, a, None).programmed);
            json!({
                "no": i + 1,
                "name": g.map_or("?", |g| g.meta.name().unwrap_or_default()),
                "color": g.map_or(Value::Null, |g| crate::attribute_board::group_colour(&paint, g)),
                "icon": g.and_then(|g| crate::attribute_board::icon_word(&g.meta)),
                "programmed": programmed,
                "mini": first.map_or(Value::Null, |v| json!({ "kind": "levels", "values": v.levels })),
            })
        })
        .collect();
    let index = (group.max(1) as usize - 1).min(layout.buttons.len().saturating_sub(1));
    let button = layout.buttons.get(index);
    let slots: Vec<Option<String>> = button.map(|b| b.slots.clone()).unwrap_or_default();
    let pages = slots.len().div_ceil(PER_PAGE).max(1);
    let page = (page.max(1) as usize).min(pages);
    let encoder = |attr: &String| -> Value {
            let v = values_of(show, user, attr, plan);
            let def = show.attribute_defs.get(attr);
            let placed = attrs.placed.get(attr);
            let readout = match (v.readouts.first(), v.levels.iter().copied().reduce(f64::min), v.levels.iter().copied().reduce(f64::max)) {
                (None, ..) => "–".to_string(),
                (Some(r), Some(lo), Some(hi)) if (hi - lo).abs() < 0.05 => r.clone(),
                (_, Some(lo), Some(hi)) => format!("{}…{}", pct(lo), pct(hi)),
                (Some(r), ..) => r.clone(),
            };
            let quoted = format!("\"{attr}\"");
            json!({
                "attr": attr,
                "pretty": def.map_or(attr.as_str(), |d| if d.pretty.is_empty() { attr.as_str() } else { d.pretty.as_str() }),
                "fixtures": v.levels.len(),
                "readout": readout,
                "value": v.levels.first().copied(),
                "levels": v.levels,
                "programmed": v.programmed,
                "fade": timing_of(&v.fade),
                "delay": timing_of(&v.delay),
                "resolution": format!("{:?}", placed.and_then(|p| p.resolution.clone()).unwrap_or_default()),
                "lines": {
                    "value": format!("Attribute {quoted} At {{value}}"),
                    "fade": format!("Attribute {quoted} Fade {{value}}"),
                    "delay": format!("Attribute {quoted} Delay {{value}}"),
                },
            })
    };
    let encoders: Vec<Value> = (0..PER_PAGE)
        .map(|k| match slots.get((page - 1) * PER_PAGE + k) {
            Some(Some(attr)) => encoder(attr),
            _ => Value::Null,
        })
        .collect();
    // A phone has no pages: it swipes through every attribute of the feature (round 10).
    let all: Vec<Value> = slots.iter().flatten().map(encoder).collect();
    json!({
        "layout": { "no": layout_no, "name": layout.meta.identity.name },
        "strip": strip,
        "group": index + 1,
        "page": page,
        "pages": pages,
        "feature": button.and_then(|b| attrs.groups.values().find(|g| g.meta.identity.guid == b.group)).map(|g| g.meta.identity.name.clone()),
        "selection": selected,
        "encoders": encoders,
        "all": all,
    })
}

/// Cue timing (round 12): the strip holds the selected sequence's cues (a mini of each cue's
/// delay and fade), the encoders are the cue and the show's feature groups, each with its In
/// (Delay, Fade) and Out (Delay, Fade) — a group's unset time inherits the cue's. `group` 0 =
/// the sequence's current cue (its first when off; decisions §30 "12f").
fn cue_timing(show: &ShowData, user: &str, group: u32, page: u32) -> Value {
    let Some(guid) = show.user(user).and_then(|u| u.selected.sequence) else {
        return json!({ "strip": [], "encoders": [], "note": "No sequence selected: Select Sequence <n>." });
    };
    let Some((_, seq_no, seq)) = show.sequence_by_guid(guid) else {
        return json!({ "strip": [], "encoders": [], "note": "The selected sequence is gone." });
    };
    let cues: Vec<(u32, &lumacore_tree::CueObject)> = seq.cues.iter().map(|(&k, c)| (k, c)).collect();
    if cues.is_empty() {
        return json!({ "strip": [], "encoders": [], "note": format!("Sequence {seq_no} has no cues.") });
    }
    let current = show.playbacks.live.sequences.get(&guid).map(|s| s.current_cue);
    let index = if group == 0 {
        current.and_then(|c| cues.iter().position(|(k, _)| *k == c)).unwrap_or(0)
    } else {
        (group as usize - 1).min(cues.len() - 1)
    };
    let strip: Vec<Value> = cues
        .iter()
        .enumerate()
        .map(|(i, (k, c))| {
            let p = c.parts.get(&0);
            json!({
                "no": i + 1,
                "name": format!("Cue {}", lumacore_tree::cue_label(*k)),
                "sub": c.meta.identity.name,
                "current": Some(*k) == current,
                "mini": { "kind": "timing", "delay": p.map_or(0.0, |p| p.delay_in), "fade": p.map_or(0.0, |p| p.fade_in) },
            })
        })
        .collect();
    let (key, cue) = cues[index];
    let label = lumacore_tree::cue_label(key);
    let part = cue.parts.get(&0).cloned().unwrap_or_default();
    let field = |own: Option<f64>, cue_value: f64| match own {
        Some(v) => json!({ "text": format!("{}s", trim(v)), "value": v, "inherited": false }),
        None => json!({ "text": format!("[{}s]", trim(cue_value)), "value": cue_value, "inherited": true }),
    };
    let base = format!("Set Sequence {seq_no} Cue {label}");
    let paint = lumacore_objects::paint::Painter::new(show, user);
    let mut list = vec![json!({
        "label": format!("Cue {label}"),
        "sub": cue.meta.identity.name,
        "in": { "delay": field(Some(part.delay_in), 0.0), "fade": field(Some(part.fade_in), 0.0) },
        "out": { "delay": field(Some(part.delay_out), 0.0), "fade": field(Some(part.fade_out), 0.0) },
        "lines": {
            "in_delay": format!("{base} \"Delay\" {{value}}"), "in_fade": format!("{base} \"Fade\" {{value}}"),
            "out_delay": format!("{base} \"OutDelay\" {{value}}"), "out_fade": format!("{base} \"OutFade\" {{value}}"),
        },
    })];
    for g in show.feature_groups.values() {
        let t = part.group_timing.get(&g.meta.identity.guid).cloned().unwrap_or_default();
        let gl = format!("{base} FeatureGroup \"{}\"", g.meta.name().unwrap_or_default());
        list.push(json!({
            "label": g.meta.identity.name,
            "sub": "feature group",
            "color": crate::attribute_board::group_colour(&paint, g),
            "in": { "delay": field(t.delay_in, part.delay_in), "fade": field(t.fade_in, part.fade_in) },
            "out": { "delay": field(t.delay_out, part.delay_out), "fade": field(t.fade_out, part.fade_out) },
            "lines": {
                "in_delay": format!("{gl} \"Delay\" {{value}}"), "in_fade": format!("{gl} \"Fade\" {{value}}"),
                "out_delay": format!("{gl} \"OutDelay\" {{value}}"), "out_fade": format!("{gl} \"OutFade\" {{value}}"),
                // Back to the cue's own timing (decisions §33 61, 87).
                "inherit": format!("Reset{} \"{{field}}\"", gl.strip_prefix("Set").unwrap_or(&gl)),
            },
        }));
    }
    let pages = list.len().div_ceil(PER_PAGE).max(1);
    let page = (page.max(1) as usize).min(pages);
    let encoders: Vec<Value> = (0..PER_PAGE).map(|k| list.get((page - 1) * PER_PAGE + k).cloned().unwrap_or(Value::Null)).collect();
    json!({
        "sequence": seq_no,
        "cue": label,
        "feature": format!("Cue {label}"),
        "strip": strip,
        "group": index + 1,
        "page": page,
        "pages": pages,
        "encoders": encoders,
        "all": list,
    })
}

fn masters(show: &ShowData, user: &str, own: &Value) -> Value {
    let g = &show.playbacks.live.masters.grand;
    let mut list = vec![
        json!({ "label": "Grand", "sub": "Master 2.1", "line": "Master 2.1 At {value}", "value": g.master * 100.0, "off": !g.on }),
        json!({ "label": "Highlight", "sub": "Master 2.2", "line": "Master 2.2 At {value}", "value": g.highlight * 100.0 }),
        json!({ "label": "Solo", "sub": "Master 2.4", "line": "Master 2.4 At {value}", "value": g.solo * 100.0 }),
    ];
    for (key, label, word) in [("highlight", "My HL", "Highlight"), ("solo", "My Solo", "Solo")] {
        if let Some(level) = own[key]["level"].as_f64() {
            list.push(json!({ "label": label, "sub": "yours", "line": format!("Master {word} At {{value}}"), "value": level }));
        }
    }
    // The group masters of the user's selected DataPool (pool-relative numbers, as typed).
    let now = lumacore_base::clock::session_ms();
    if let Some(dp) = show.datapools.get(&lumacore_objects::pools::user_datapool(show, user)) {
        for (no, group) in &dp.groups {
            if group.master == lumacore_tree::GroupMasterMode::None {
                continue;
            }
            let level = show.playbacks.live.masters.groups.get(&group.meta.identity.guid).copied().unwrap_or_default().at(now);
            list.push(json!({
                "label": group.meta.identity.name.as_deref().unwrap_or("Group"),
                "sub": format!("Group {no}"),
                "line": format!("Master Group {no} At {{value}}"),
                "value": level * 100.0,
            }));
        }
    }
    json!({ "masters": list })
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
