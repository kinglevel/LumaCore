//! Attributes, feature groups, the resolver's rules and encoder bar layouts
//! (decisions §30 "12f-a syntax hardening"). Two layers that never mix: the
//! show's (cue timing reads it; bare words) and each user profile's own
//! (owner first: `UserProfile [<n>|"name"] Attribute|FeatureGroup|AttributeRule|EncoderBar …`,
//! bare = your own). `Attribute` is an object: its Call sets a programmer
//! value. Names are values: quoted when they hold spaces, `+` joins several,
//! a quoted `*` is a pattern. `Default [UserProfile] Attribute /New` keeps
//! what the resolver placed.

use std::collections::BTreeSet;

use lumacore_tree::attribute_resolver::{self, matches, AttributeLayer, Ctx, Place};
use lumacore_tree::programmer::SubAttributesFollow;
use lumacore_tree::ShowData;
use lumacore_base::{IconRef, NodeMeta};
use lumacore_fixtures::{AttributeRule, FeatureGroup, PlaceHow, Placement, RuleKind};

use crate::commands::registry::{Doc, Undo};
use crate::commands::tokens::unquote;
use crate::errors::render_code;
use crate::objects::{out, ObjectDef, Verb};
use crate::text::render_success;
use crate::commands::flags as F;
use crate::{AppState, DispatchResult};

// ─── Words ───────────────────────────────────────────────────────────────────

// The flags these words take (decisions §31: declared, read with `flags`).
const NEW: F::FlagDef = F::FlagDef::new("New", "Only the attributes a patch brought, not answered yet");
const UNUSED: F::FlagDef = F::FlagDef::new("Unused", "Only attributes no patched fixture uses");
const HINT: F::FlagDef = F::FlagDef::new("Hint", "Only placements from a hint or a guess");
const WITH_SUBS: F::FlagDef = F::FlagDef::new("WithSubs", "Move its sub-attributes with it");
const ONLY: F::FlagDef = F::FlagDef::new("Only", "Move only this attribute");
const LEARNED: F::FlagDef = F::FlagDef::new("Learned", "Only the rules learned here");
const KEYWORD: F::FlagDef = F::FlagDef::new("Keyword", "A keyword rule (the word anywhere in the name)");
/// What `Attribute` (and `Edit` / `List` / `Default` on it) take.
pub const ATTRIBUTE_FLAGS: &[F::FlagDef] = &[NEW, UNUSED, HINT, F::BY, WITH_SUBS, ONLY];

fn no_confirmation(args: &[&str]) -> bool {
    F::has(args, &F::NO_CONFIRMATION)
}

fn plain<'a>(args: &[&'a str]) -> Vec<&'a str> {
    F::positional(args, &[])
}

/// The words up to the first of `stops` (case-insensitive), and the rest.
fn split_at_word<'a>(args: &'a [&'a str], stops: &[&str]) -> (&'a [&'a str], &'a [&'a str]) {
    let i = args
        .iter()
        .position(|a| stops.iter().any(|s| a.eq_ignore_ascii_case(s)))
        .unwrap_or(args.len());
    (&args[..i], &args[i..])
}

fn quoted(n: &str) -> String {
    format!("\"{n}\"")
}

// ─── Targets: the show's layer or a profile's ────────────────────────────────

/// Which layer a line works on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Show,
    Profile(u32),
}

impl Target {
    /// The line's prefix for this target (`""`, `UserProfile `, `UserProfile 4 `).
    fn prefix(self, state: &AppState) -> String {
        match self {
            Target::Show => String::new(),
            Target::Profile(no) if Some(no) == own_profile(state) => "UserProfile ".into(),
            Target::Profile(no) => format!("UserProfile {no} "),
        }
    }
}

fn own_profile(state: &AppState) -> Option<u32> {
    state.root.show_data.profile_id_for_user(state.effective_user_name())
}

fn layer(show: &ShowData, t: Target) -> &dyn AttributeLayer {
    match t {
        Target::Show => show,
        Target::Profile(no) => &show.user_profiles[&no].content.attributes,
    }
}

/// The layer to change, and the context the resolver reads (the show as it is).
fn with_layer<R>(state: &mut AppState, t: Target, f: impl FnOnce(&mut dyn AttributeLayer, &Ctx) -> R) -> R {
    let show = &mut state.root.show_data;
    let ctx = Ctx::of(show);
    match t {
        Target::Show => f(show, &ctx),
        Target::Profile(no) => {
            let p = show.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).expect("a known profile");
            f(&mut p.content.attributes, &ctx)
        }
    }
}

/// A profile without a layer of its own starts from the show's.
fn ensure_seeded(state: &mut AppState, t: Target) {
    if let Target::Profile(no) = t {
        let show = &mut state.root.show_data;
        if show.user_profiles.get(&no).is_some_and(|p| !p.content.attributes.is_seeded()) {
            let fresh = attribute_resolver::profile_from_show(show);
            show.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes = fresh;
        }
    }
}

/// After a profile's places changed: each moved attribute sits at the
/// bottom of its group's strip button on every layout.
fn relayout(state: &mut AppState, t: Target, names: &[String]) {
    if let Target::Profile(no) = t {
        let p = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes;
        for n in names {
            if let Some(g) = p.placed.get(n).map(|x| x.group) {
                attribute_resolver::add_to_layouts(p, n, g);
            }
        }
    }
}

/// Attribute names a line names: `"Zoom" + "Iris"`, `"Shutter2*"` (a quoted
/// pattern), case-insensitive. Unknown names are errors.
fn attribute_names(l: &dyn AttributeLayer, words: &[&str]) -> Result<Vec<String>, String> {
    let known = l.names();
    let mut out: Vec<String> = Vec::new();
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    for w in words {
        if *w == "+" {
            if !cur.is_empty() {
                parts.push(std::mem::take(&mut cur));
            }
            continue;
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(unquote(w));
    }
    if !cur.is_empty() {
        parts.push(cur);
    }
    if parts.is_empty() {
        return Err(render_code("Name an attribute: Attribute \"Zoom\" (or \"Shutter2*\").", "E-CMD-006"));
    }
    for p in parts {
        let found: Vec<&String> = if p.contains('*') {
            known.iter().filter(|n| matches(&p, n)).collect()
        } else {
            known.iter().filter(|n| n.eq_ignore_ascii_case(&p)).collect()
        };
        if found.is_empty() {
            return Err(render_code(format!("No attribute \"{p}\" in that grouping."), "E-CMD-007"));
        }
        for n in found {
            if !out.contains(n) {
                out.push(n.clone());
            }
        }
    }
    Ok(out)
}

/// A feature group by name or number: (id, guid, name).
fn group_ref(l: &dyn AttributeLayer, word: &str) -> Result<(u32, uuid::Uuid, String), String> {
    let w = unquote(word);
    let g = match w.parse::<u32>() {
        Ok(n) => l.groups().get_key_value(&n),
        Err(_) => l.groups().iter().find(|(_, g)| g.meta.name().unwrap_or_default().eq_ignore_ascii_case(w)),
    };
    g.map(|(id, g)| (*id, g.meta.identity.guid, g.meta.name().unwrap_or_default().to_string())).ok_or_else(|| {
        let names: Vec<&str> = l.groups().values().map(|g| g.meta.name().unwrap_or_default()).collect();
        render_code(format!("No feature group \"{w}\" (there are {}).", names.join(", ")), "E-CMD-007")
    })
}

fn group_name(l: &dyn AttributeLayer, guid: Option<uuid::Uuid>) -> String {
    guid.and_then(|g| l.groups().values().find(|f| f.meta.identity.guid == g))
        .map(|f| f.meta.name().unwrap_or_default().to_string())
        .unwrap_or_else(|| "—".into())
}

/// The attributes of a group, in order.
fn members(l: &dyn AttributeLayer, group: uuid::Uuid) -> Vec<String> {
    let mut v: Vec<(u32, String)> = l
        .names()
        .into_iter()
        .filter_map(|n| l.place_of(&n).filter(|p| p.group == group).map(|p| (p.position, n)))
        .collect();
    v.sort();
    v.into_iter().map(|(_, n)| n).collect()
}

/// Write positions 1… for a group's attributes in `order`.
fn renumber(l: &mut dyn AttributeLayer, order: &[String]) {
    for (i, n) in order.iter().enumerate() {
        if let Some(mut p) = l.place_of(n) {
            p.position = i as u32 + 1;
            l.set_place(n, p);
        }
    }
}

/// Sub-attributes of `main` in the same group: its MainAttribute children
/// (from the show), and names that continue it with a capital.
fn subs_of(l: &dyn AttributeLayer, ctx: &Ctx, main: &str) -> Vec<String> {
    let Some(mp) = l.place_of(main) else { return Vec::new() };
    l.names()
        .into_iter()
        .filter(|n| n != main && l.place_of(n).is_some_and(|p| p.group == mp.group))
        .filter(|n| {
            ctx.main.get(n).is_some_and(|m| m.eq_ignore_ascii_case(main))
                || n.strip_prefix(main).and_then(|rest| rest.chars().next()).is_some_and(|c| c.is_ascii_uppercase())
        })
        .collect()
}

/// Move `names` into `group` at `at` (1-based) or the bottom, placed by a user.
fn place(l: &mut dyn AttributeLayer, names: &[String], group: uuid::Uuid, at: Option<usize>) {
    for n in names {
        if let Some(p) = l.place_of(n) {
            l.set_place(
                n,
                Place {
                    group,
                    feature: p.feature,
                    position: p.position,
                    placement: Placement { how: PlaceHow::User, why: "placed by a user".into(), file_feature: p.placement.file_feature },
                    new: false,
                },
            );
        }
    }
    let mut order: Vec<String> = members(l, group).into_iter().filter(|n| !names.contains(n)).collect();
    let i = at.map(|a| a.saturating_sub(1).min(order.len())).unwrap_or(order.len());
    for (k, n) in names.iter().enumerate() {
        order.insert(i + k, n.clone());
    }
    renumber(l, &order);
}

// ─── Attribute ───────────────────────────────────────────────────────────────

fn follow_setting(state: &AppState) -> SubAttributesFollow {
    state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| p.settings.preferences.sub_attributes_follow)
        .unwrap_or_default()
}

/// `Set [UserProfile] Attribute <names> FeatureGroup <g> [/WithSubs | /Only]` ·
/// `… Readout Percent|Physical` · `… Resolution Coarse|Fine|Ultra`.
fn attribute_set(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let (names_w, rest) = split_at_word(&words, &["FeatureGroup", "Readout", "Resolution"]);
    let names = match attribute_names(layer(&state.root.show_data, t), names_w) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let Some(prop) = rest.first() else {
        return out(render_code(
            "Usage: Assign [UserProfile] Attribute \"Zoom\" At FeatureGroup \"FX\" · Set Attribute \"Zoom\" \"Readout\" Physical | \"Resolution\" Fine",
            "E-CMD-006",
        ));
    };
    let value = rest.get(1).copied().unwrap_or("");
    match prop.to_ascii_lowercase().as_str() {
        "featuregroup" => {
            let (gid, gname) = match group_ref(layer(&state.root.show_data, t), value) {
                Ok((_, g, n)) => (g, n),
                Err(e) => return out(e),
            };
            let mut names = names;
            // A main attribute brings its sub-attributes (decisions §30
            // "12f-a": the profile's SubAttributesFollow, default Ask).
            if names.len() == 1 && !F::has(args, &ONLY) {
                let ctx = Ctx::of(&state.root.show_data);
                let l = layer(&state.root.show_data, t);
                let subs: Vec<String> =
                    subs_of(l, &ctx, &names[0]).into_iter().filter(|s| l.place_of(s).is_some_and(|p| p.group != gid)).collect();
                if !subs.is_empty() {
                    let follow = follow_setting(state);
                    if F::has(args, &WITH_SUBS) || no_confirmation(args) || follow == SubAttributesFollow::Always {
                        names.extend(subs);
                    } else if follow == SubAttributesFollow::Ask {
                        let line = format!("Assign {p}Attribute {} At {p}FeatureGroup {}", quoted(&names[0]), quoted(&gname), p = t.prefix(state));
                        return DispatchResult::Ask(Box::new(
                            crate::dialogs::Ask::new(
                                format!("Move {} with it?", subs.len()),
                                format!("{} is the main attribute of {}. Move them to {gname} too?", names[0], subs.join(", ")),
                            )
                            .button(&format!("Move {}", subs.len() + 1), lumacore_wire::ButtonKind::Primary, format!("{line} /WithSubs"))
                            .button(&format!("Only {}", names[0]), lumacore_wire::ButtonKind::Plain, format!("{line} /Only"))
                            .cancel("Cancel")
                            .flags("/WithSubs or /Only"),
                        ));
                    }
                }
            }
            with_layer(state, t, |l, _| place(l, &names, gid, None));
            relayout(state, t, &names);
            out(render_success(&format!("{} → {gname} (at the bottom).", names.join(", "))))
        }
        "readout" | "resolution" => {
            let readout = prop.eq_ignore_ascii_case("readout");
            use lumacore_fixtures::{EncoderResolution as R, NaturalReadout as N};
            let (r, s): (Option<N>, Option<R>) = match value.to_ascii_lowercase().as_str() {
                "percent" if readout => (Some(N::Percent), None),
                "physical" if readout => (Some(N::Physical), None),
                "coarse" if !readout => (None, Some(R::Coarse)),
                "fine" if !readout => (None, Some(R::Fine)),
                "ultra" if !readout => (None, Some(R::Ultra)),
                _ => {
                    return out(render_code(
                        if readout { "Readout is Percent or Physical." } else { "Resolution is Coarse, Fine or Ultra." },
                        "E-CMD-008",
                    ))
                }
            };
            let show = &mut state.root.show_data;
            for n in &names {
                match t {
                    Target::Show => {
                        let d = show.attribute_defs.get_mut(n).unwrap();
                        if let Some(r) = r.clone() {
                            d.natural_readout = r;
                        }
                        if let Some(s) = s.clone() {
                            d.encoder_resolution = s;
                        }
                    }
                    Target::Profile(no) => {
                        let p = show.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes.placed.get_mut(n).unwrap();
                        if r.is_some() {
                            p.readout = r.clone();
                        }
                        if s.is_some() {
                            p.resolution = s.clone();
                        }
                    }
                }
            }
            out(render_success(&format!("{}: {} {value}.", names.join(", "), prop.to_ascii_lowercase())))
        }
        other => out(render_code(
            format!("Attribute has no property \"{other}\": FeatureGroup, Readout or Resolution."),
            "E-CMD-006",
        )),
    }
}

/// `Move [UserProfile] Attribute <names> At <n>` (inside its group) ·
/// `… At FeatureGroup <g> [<n>]`.
fn attribute_move(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let (names_w, rest) = split_at_word(&words, &["At"]);
    let l = layer(&state.root.show_data, t);
    let names = match attribute_names(l, names_w) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let usage = || out(render_code("Usage: Move [UserProfile] Attribute \"Zoom\" At 3 | At FeatureGroup \"FX\" 3", "E-CMD-006"));
    let rest = rest.get(1..).unwrap_or(&[]);
    let (group, at) = match rest {
        [fg, g, tail @ ..] if FEATURE_GROUP.matches(fg) => match group_ref(l, g) {
            Ok((_, gid, _)) => (gid, tail.first().and_then(|n| n.parse::<usize>().ok())),
            Err(e) => return out(e),
        },
        [n] => match (n.parse::<usize>(), l.place_of(&names[0])) {
            (Ok(n), Some(p)) => (p.group, Some(n)),
            _ => return usage(),
        },
        _ => return usage(),
    };
    with_layer(state, t, |l, _| place(l, &names, group, at.or(Some(usize::MAX))));
    relayout(state, t, &names);
    let l = layer(&state.root.show_data, t);
    let pos = l.place_of(&names[0]).map(|p| p.position).unwrap_or(0);
    out(render_success(&format!("{} → {}, position {pos}.", names.join(", "), group_name(l, Some(group)))))
}

/// The attributes a list shows: `/New`, `/Unused`, `/Hint` (hints and
/// guesses), `FixtureType "x"`.
fn listed(state: &AppState, t: Target, args: &[&str]) -> Result<Vec<String>, String> {
    let show = &state.root.show_data;
    let l = layer(show, t);
    let used = attribute_resolver::used_attributes(show);
    let words = plain(args);
    let of_type: Option<BTreeSet<String>> = match words.as_slice() {
        [ft, name, ..] if ft.eq_ignore_ascii_case("FixtureType") => {
            let name = unquote(name);
            let ty = show
                .devices
                .dmx
                .patch
                .fixture_types
                .values()
                .find(|x| x.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(name)))
                .ok_or_else(|| render_code(format!("No fixture type \"{name}\"."), "E-CMD-007"))?;
            Some(
                ty.revisions
                    .iter()
                    .flat_map(|r| r.modes.iter())
                    .flat_map(|m| m.channel_functions.iter())
                    .map(|c| c.attribute_name.clone())
                    .collect(),
            )
        }
        _ => None,
    };
    let unused = F::has(args, &UNUSED);
    Ok(l
        .names()
        .into_iter()
        .filter(|n| {
            let Some(p) = l.place_of(n) else { return false };
            (!F::has(args, &NEW) || p.new)
                && (!unused || !used.contains_key(n))
                && (!F::has(args, &HINT) || matches!(p.placement.how, PlaceHow::Hint | PlaceHow::Guess))
                && of_type.as_ref().is_none_or(|ty| ty.contains(n))
        })
        .collect())
}

fn attribute_list(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let names = match listed(state, t, args) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let l = layer(&state.root.show_data, t);
    if names.is_empty() {
        return out("No attributes.".into());
    }
    let mut rows: Vec<(String, u32, String)> = names
        .iter()
        .filter_map(|n| l.place_of(n).map(|p| (group_name(l, Some(p.group)), p.position, n.clone())))
        .collect();
    rows.sort();
    let lines: Vec<String> = rows
        .into_iter()
        .map(|(g, _, n)| {
            let p = l.place_of(&n).unwrap();
            format!("{g:<10} {n:<28} {:<7} {}{}", p.placement.how.word(), p.placement.why, if p.new { "  [new]" } else { "" })
        })
        .collect();
    out(lines.join("\n"))
}

/// `Edit [UserProfile [x]] Attribute [/New|/Hint|/Unused] [/By FixtureType]`:
/// open the board (decisions §30 "12f-a"); a terminal gets the list.
fn attribute_edit(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    open_board(state, t, args);
    let words: Vec<&str> = F::without(args, &F::BY);
    attribute_list(state, t, &words)
}

/// `/By FixtureType`: the board by fixture type instead of by group (decisions round 3 19 — one
/// `/FixtureType`, the Store option).
fn by_type(args: &[&str]) -> bool {
    F::value(args, &F::BY).is_some_and(|v| v.eq_ignore_ascii_case("FixtureType"))
}

/// Ask this client to open the board of `t`.
pub fn open_board(state: &mut AppState, t: Target, args: &[&str]) {
    let filter = if F::has(args, &NEW) {
        "new"
    } else if F::has(args, &HINT) {
        "doubt"
    } else if F::has(args, &UNUSED) {
        "unused"
    } else {
        "all"
    };
    let board = match t {
        Target::Show => crate::attribute_board::BoardTarget::Show,
        Target::Profile(no) => crate::attribute_board::BoardTarget::Profile(no),
    };
    let owner = crate::attribute_board::owner_words(&state.root.show_data, state.effective_user_name(), board);
    state.ui_requests.push(lumacore_wire::UiEvent::AttributeBoard {
        profile: match t {
            Target::Show => 0,
            Target::Profile(no) => no,
        },
        owner,
        filter: filter.into(),
        view: if by_type(args) { "type" } else { "group" }.into(),
    });
}

/// Attribute lines: Set / Move change the master's show; opening a board
/// and reading stay on this station.
fn attribute_scope(verb: Verb, _: &[&str]) -> crate::commands::registry::Scope {
    match verb {
        Verb::Edit | Verb::List | Verb::Info => crate::commands::registry::Scope::Client,
        _ => crate::commands::registry::Scope::Master,
    }
}

fn attribute_info(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let show = &state.root.show_data;
    let l = layer(show, t);
    let names = match attribute_names(l, &plain(args)) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let used = attribute_resolver::used_attributes(show);
    let mut blocks = Vec::new();
    for n in names {
        let p = l.place_of(&n).unwrap();
        let d = show.attribute_defs.get(&n);
        let types: Vec<String> = show
            .devices
            .dmx
            .patch
            .fixture_types
            .values()
            .filter(|ty| {
                ty.revisions
                    .iter()
                    .flat_map(|r| r.modes.iter())
                    .any(|m| m.channel_functions.iter().any(|c| c.attribute_name == n))
            })
            .filter_map(|ty| ty.meta.name().map(str::to_string))
            .collect();
        let show_group = d.and_then(|d| d.feature_group).map(|g| group_name(show, Some(g)));
        blocks.push(format!(
            "Attribute \"{n}\" ({})\n  FeatureGroup {}{}, position {}{}\n  placed: {} — {}\n  file says: {}\n  special: {} · main: {}\n  fixture types: {}{}",
            d.map(|d| d.pretty.as_str()).unwrap_or("—"),
            group_name(l, Some(p.group)),
            p.feature.as_deref().map(|f| format!(" · {f}")).unwrap_or_default(),
            p.position,
            match (t, show_group) {
                (Target::Profile(_), Some(g)) => format!(" (the show: {g})"),
                _ => String::new(),
            },
            p.placement.how.word(),
            p.placement.why,
            p.placement.file_feature.as_deref().unwrap_or("—"),
            d.and_then(|d| d.special.as_deref()).unwrap_or("—"),
            d.and_then(|d| d.main_attribute.as_deref()).unwrap_or("—"),
            if types.is_empty() { "—".into() } else { types.join(", ") },
            if used.contains_key(&n) { "" } else { "\n  unused: no patched fixture has it" },
        ));
    }
    out(blocks.join("\n"))
}

fn complete_attributes(state: &AppState, prefix: &str) -> Vec<String> {
    let p = prefix.trim_matches('"').to_lowercase();
    state
        .root
        .show_data
        .attribute_defs
        .keys()
        .filter(|n| n.to_lowercase().starts_with(&p))
        .map(|n| if n.contains(' ') { quoted(n) } else { n.clone() })
        .collect()
}

/// Calling sets a programmer value (Playback); the show's definition is Setup.
fn attribute_rights(verb: Verb, args: &[&str]) -> crate::commands::registry::Rights {
    match verb {
        Verb::Select | Verb::At => crate::commands::registry::Rights::Playback,
        // Opening the board is a look; each change it sends is checked.
        Verb::Edit => crate::commands::registry::Rights::Guest,
        _ => crate::rights::setup(verb, args),
    }
}

pub static ATTRIBUTE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Attribute",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Select,
        aliases: &["att", "attributes"],
        doc: Doc {
            display: "Attribute",
            category: crate::commands::help::HelpCategory::Programmer,
            also_applies_to: &[],
            syntax: "Attribute <name> At <value> | Attribute <name> Fade|Delay <s>|Cue | Assign [UserProfile [<n>|\"name\"]] Attribute <names> At [UserProfile] FeatureGroup <g> [/WithSubs|/Only] | Set Attribute <name> \"Readout\" Percent|Physical | \"Resolution\" Coarse|Fine|Ultra | Move [UserProfile] Attribute <names> At <n> | At FeatureGroup <g> [<n>] | List [UserProfile] Attribute [/New] [/Unused] [/Hint] [FixtureType \"x\"] | Info [UserProfile] Attribute <names> | Default [UserProfile] Attribute /New | Edit [UserProfile] Attribute [/New|/Hint|/Unused] [/By FixtureType]",
            description: "A fixture attribute (decisions §30 \"12f-a\"). Calling it sets a programmer value on \
                          the selection (Attribute \"Pan\" At 120). The show's definition says which \
                          feature group it is in — cue timing per feature group reads it; your profile has \
                          its own grouping (UserProfile Attribute …) for your encoder bar. A patch that brings \
                          an attribute nobody has seen places it by the resolver's rules (marked new) in the \
                          show and in every profile, and each user is asked. Names: quoted when they hold \
                          spaces, + joins, a quoted * is a pattern.",
            examples: &[
                "Attribute \"Dimmer\" Fade 2   -- the selection's Dimmer values fade 2 s when stored",
                "Attribute \"Pan\" At 120                              -- programmer",
                "Assign Attribute \"Zoom\" At FeatureGroup \"FX\"              -- the show's, at the bottom",
                "Assign UserProfile Attribute \"Zoom\" At UserProfile FeatureGroup \"Beam\" -- yours",
                "Assign Attribute \"Shutter2*\" At FeatureGroup \"Shutter\"",
                "Move Attribute \"Zoom\" At FeatureGroup \"FX\" 3",
                "Info Attribute \"Pan Invert\"                         -- why it is where it is",
                "List UserProfile Attribute /New",
                "Default UserProfile Attribute /New                  -- keep where your rules put them",
                "Edit UserProfile Attribute /New                     -- your board, the new ones",
            ],
            related: &["featuregroup", "attributerule", "encoderbar", "default", "userprofile"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: Some(crate::commands::registry::CommandFamily::Action),
        call_undo: crate::commands::registry::PROGRAMMER,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: attribute_scope,
        reach: crate::objects::show_reach,
        rights: attribute_rights,
        planned: &[],
        flags: ATTRIBUTE_FLAGS,
    },
    pool: None,
    verbs: &[
        (Verb::Select, |s, a| out(crate::commands::control::cmd_attribute(s, a))),
        (Verb::At, |s, a| out(crate::commands::control::cmd_attribute(s, a))),
        (Verb::Assign, crate::commands::typed::typed::<AssignAttribute>),
        (Verb::Move, |s, a| attribute_move(s, Target::Show, a)),
        (Verb::List, |s, a| attribute_list(s, Target::Show, a)),
        (Verb::Info, |s, a| attribute_info(s, Target::Show, a)),
        (Verb::Edit, |s, a| attribute_edit(s, Target::Show, a)),
    ],
    delete_ask: None,
    complete: Some(complete_attributes),
    props: Some(&ATTRIBUTE_SETTINGS),
    tree: None,
};

/// `Default [UserProfile [x]] Attribute /New`: keep where the resolver put
/// the new attributes (the Default keyword: back to the standard).
pub fn default_attributes(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let words = plain(args);
    let (t, rest) = match profile_target(state, &words) {
        Ok(Some((t, rest))) => (t, rest),
        Ok(None) => (Target::Show, words.as_slice()),
        Err(e) => return out(e),
    };
    if !rest.first().is_some_and(|w| ATTRIBUTE.matches(w)) {
        return out(render_code("Usage: Default [UserProfile] Attribute /New", "E-CMD-006"));
    }
    if let Target::Profile(no) = t
        && let Err(e) = crate::commands::media::may_change_profile(state, no)
    {
        return out(e);
    }
    let n = clear_new(state, t);
    out(render_success(&if n == 0 {
        "No new attributes.".to_string()
    } else {
        format!(
            "{n} new attribute{} kept where {} put them.",
            if n == 1 { "" } else { "s" },
            if t == Target::Show { "the standard grouping" } else { "your rules" }
        )
    }))
}

/// Clear the new marks of a layer; returns how many.
/// A kept patch draft (a `Hook::PatchKept` of the programmer family): the attributes follow the
/// new patch, and Keep's `/Standard` / `/Mine` clear what is new in the show / your profile.
pub(crate) fn after_patch_kept(state: &mut AppState, standard: bool, mine: bool) {
    if !(standard || mine) {
        return;
    }
    lumacore_tree::attribute_resolver::sync(&mut state.root.show_data);
    if standard {
        clear_new(state, Target::Show);
    }
    if mine && let Some(no) = state.root.show_data.profile_id_for_user(state.effective_user_name()) {
        clear_new(state, Target::Profile(no));
    }
}

pub fn clear_new(state: &mut AppState, t: Target) -> usize {
    let show = &mut state.root.show_data;
    match t {
        Target::Show => {
            let mut n = 0;
            for d in show.attribute_defs.values_mut().filter(|d| d.new) {
                d.new = false;
                n += 1;
            }
            n
        }
        Target::Profile(no) => {
            let Some(p) = show.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut) else { return 0 };
            let mut n = 0;
            for a in p.content.attributes.placed.values_mut().filter(|a| a.new) {
                a.new = false;
                n += 1;
            }
            n
        }
    }
}

// ─── FeatureGroup ────────────────────────────────────────────────────────────

fn renumber_groups(l: &mut dyn AttributeLayer, list: Vec<FeatureGroup>) {
    *l.groups_mut() = list
        .into_iter()
        .enumerate()
        .map(|(i, g)| (i as u32 + 1, g))
        .collect();
}

/// `Store [UserProfile] FeatureGroup "Haze"` — a new, empty group at the end.
fn group_store(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    if words.is_empty() {
        // Bare (the board's "+" column): one ask for the name.
        let prefix = t.prefix(state);
        return DispatchResult::Ask(Box::new(
            crate::dialogs::Ask::new("New feature group", "It is added at the end, empty.")
                .meta(format!("Store {prefix}FeatureGroup"), None)
                .input("name", "Name", "", lumacore_wire::InputKind::Text)
                .button("Create", lumacore_wire::ButtonKind::Primary, format!("Store {prefix}FeatureGroup \"{{name}}\""))
                .cancel("Cancel")
                .flags("the name: Store FeatureGroup \"name\""),
        ));
    }
    let Some(name) = words.first().map(|w| unquote(w).to_string()).filter(|n: &String| !n.is_empty() && n.parse::<u32>().is_err()) else {
        return out(render_code("Usage: Store [UserProfile] FeatureGroup \"name\"", "E-CMD-006"));
    };
    if layer(&state.root.show_data, t).groups().values().any(|g| g.meta.name().unwrap_or_default().eq_ignore_ascii_case(&name)) {
        return out(render_code(format!("A feature group \"{name}\" exists already."), "E-CMD-010"));
    }
    let id = with_layer(state, t, |l, _| {
        let id = l.groups().keys().max().copied().unwrap_or(0) + 1;
        l.groups_mut().insert(id, FeatureGroup { meta: NodeMeta::new(Some(name.clone())), features: Vec::new() });
        id
    });
    out(render_success(&format!("FeatureGroup {id} \"{name}\" stored (empty, at the end).")))
}

fn group_label(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let [g, name, ..] = words.as_slice() else {
        return out(render_code("Usage: Label [UserProfile] FeatureGroup \"FX\" \"Effects\"", "E-CMD-006"));
    };
    let l = layer(&state.root.show_data, t);
    let id = match group_ref(l, g) {
        Ok((id, _, _)) => id,
        Err(e) => return out(e),
    };
    let name = unquote(name).to_string();
    if l.groups().iter().any(|(n, x)| *n != id && x.meta.name().unwrap_or_default().eq_ignore_ascii_case(&name)) {
        return out(render_code(format!("A feature group \"{name}\" exists already."), "E-CMD-010"));
    }
    let old = with_layer(state, t, |l, _| l.groups_mut().get_mut(&id).unwrap().meta.identity.name.replace(name.clone())).unwrap_or_default();
    out(render_success(&format!("FeatureGroup \"{old}\" is now \"{name}\".")))
}

/// `Set [UserProfile] FeatureGroup <g> Icon "name" | Sort Name|Family|MainFirst | Merge <other> | Split Family` (its
/// Paint is the one Paint property, decisions 0294).
fn group_set(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let [g, rest @ ..] = words.as_slice() else {
        return out(render_code("Usage: Set FeatureGroup \"FX\" \"Paint\" \"#b07be6\" | Icon \"sparkles\" | Sort Family | Merge \"Gobo\" | Split Family", "E-CMD-006"));
    };
    let (id, guid, name) = match group_ref(layer(&state.root.show_data, t), g) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    let prefix = t.prefix(state);
    let mut done = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        let prop = rest[i].to_ascii_lowercase();
        let value = rest.get(i + 1).copied().unwrap_or("");
        i += 2;
        match prop.as_str() {
            "icon" => {
                let icon = unquote(value).to_string();
                with_layer(state, t, |l, _| l.groups_mut().get_mut(&id).unwrap().meta.annotation.icon = Some(IconRef::Builtin(icon.clone())));
                done.push(format!("icon {}", unquote(value)));
            }
            "sort" => {
                let how = value.to_ascii_lowercase();
                if !matches!(how.as_str(), "name" | "family" | "mainfirst") {
                    return out(render_code("Sort Name, Family or MainFirst.", "E-CMD-008"));
                }
                let names = with_layer(state, t, |l, ctx| {
                    let mut order = members(l, guid);
                    let num = |n: &str| n.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().unwrap_or(0);
                    let root = |n: &str| n.chars().take_while(|c| c.is_ascii_alphabetic()).collect::<String>().to_lowercase();
                    match how.as_str() {
                        "name" => order.sort_by_key(|a| a.to_lowercase()),
                        "family" => order.sort_by(|a, b| root(a).cmp(&root(b)).then(num(a).cmp(&num(b))).then(a.len().cmp(&b.len())).then(a.cmp(b))),
                        _ => {
                            let first = order.clone();
                            let main_of = |n: &String| ctx.main.get(n).cloned().unwrap_or_else(|| n.clone());
                            order.sort_by_key(|n| (first.iter().position(|x| *x == main_of(n)).unwrap_or(usize::MAX), n.len()));
                        }
                    }
                    renumber(l, &order);
                    order
                });
                // The layouts follow the group's new order.
                if let Target::Profile(no) = t {
                    let p = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes;
                    for bar in p.encoder_bars.values_mut() {
                        if let Some(b) = bar.buttons.iter_mut().find(|b| b.group == guid) {
                            b.slots = names.iter().map(|n| Some(n.clone())).collect();
                        }
                    }
                }
                done.push(format!("sorted by {value}"));
            }
            "merge" => {
                let (to_guid, to_name) = match group_ref(layer(&state.root.show_data, t), value) {
                    Ok(x) if x.0 != id => (x.1, x.2),
                    Ok(_) => return out(render_code("A group can't merge into itself.", "E-CMD-008")),
                    Err(e) => return out(e),
                };
                let moving = members(layer(&state.root.show_data, t), guid);
                if !no_confirmation(args) {
                    let line = format!("Assign {prefix}FeatureGroup {} At {prefix}FeatureGroup {} /NoConfirmation", quoted(&name), quoted(&to_name));
                    return DispatchResult::Ask(Box::new(
                        crate::dialogs::Ask::new(
                            format!("Merge {name} into {to_name}?"),
                            format!(
                                "{} attributes move to the bottom of {to_name}, and the group {name} is removed.{}",
                                moving.len(),
                                if t == Target::Show { " Cue timing set for it stays with the cues and is listed as broken." } else { "" }
                            ),
                        )
                        .button("Merge", lumacore_wire::ButtonKind::Danger, line)
                        .cancel("Cancel")
                        .flags("/NoConfirmation"),
                    ));
                }
                with_layer(state, t, |l, _| {
                    place(l, &moving, to_guid, None);
                    for r in l.rules_mut().values_mut().filter(|r| r.group == guid) {
                        r.group = to_guid;
                    }
                    l.groups_mut().remove(&id);
                    let list: Vec<FeatureGroup> = l.groups().values().cloned().collect();
                    renumber_groups(l, list);
                });
                if let Target::Profile(no) = t {
                    let p = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes;
                    for n in &moving {
                        attribute_resolver::add_to_layouts(p, n, to_guid);
                    }
                    for bar in p.encoder_bars.values_mut() {
                        bar.buttons.retain(|b| b.group != guid);
                    }
                }
                return out(render_success(&format!("{name} merged into {to_name} ({} attributes).", moving.len())));
            }
            "split" => {
                if !value.eq_ignore_ascii_case("Family") {
                    return out(render_code("Split Family.", "E-CMD-008"));
                }
                let mut fams: Vec<(String, Vec<String>)> = Vec::new();
                for n in members(layer(&state.root.show_data, t), guid) {
                    let root: String = n.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
                    match fams.iter_mut().find(|(r, _)| *r == root) {
                        Some((_, v)) => v.push(n),
                        None => fams.push((root, vec![n])),
                    }
                }
                if fams.len() < 2 {
                    return out(render_code(format!("{name} has only one family."), "E-CMD-010"));
                }
                if !no_confirmation(args) {
                    let line = format!("Split {prefix}FeatureGroup {} /By Family /NoConfirmation", quoted(&name));
                    let list: Vec<String> = fams.iter().map(|(r, v)| format!("{r} ({})", v.len())).collect();
                    return DispatchResult::Ask(Box::new(
                        crate::dialogs::Ask::new(
                            format!("Split {name} by family?"),
                            format!("{name} becomes {} groups: {}.", fams.len(), list.join(", ")),
                        )
                        .button(&format!("Split into {}", fams.len()), lumacore_wire::ButtonKind::Primary, line)
                        .cancel("Cancel")
                        .flags("/NoConfirmation"),
                    ));
                }
                // The first family keeps the group (its GUID); the others are new groups after it.
                let made = with_layer(state, t, |l, _| {
                    let base = l.groups()[&id].clone();
                    let mut out_list: Vec<FeatureGroup> = Vec::new();
                    let mut made: Vec<(uuid::Uuid, Vec<String>)> = Vec::new();
                    for (n, g) in l.groups().clone() {
                        out_list.push(g.clone());
                        if n == id {
                            for (root, names) in fams.iter().skip(1) {
                                let mut ng = FeatureGroup { meta: NodeMeta::new(Some(root.clone())), features: Vec::new() };
                                // The new group keeps the look of the one it split from.
                                ng.meta.annotation.paint = Some(group_paint(&base));
                                ng.meta.annotation.icon = base.meta.annotation.icon.clone();
                                made.push((ng.meta.identity.guid, names.clone()));
                                out_list.push(ng);
                            }
                        }
                    }
                    renumber_groups(l, out_list);
                    for (g, names) in &made {
                        place(l, names, *g, None);
                    }
                    renumber(l, &fams[0].1);
                    made
                });
                if let Target::Profile(no) = t {
                    let p = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes;
                    for (g, names) in &made {
                        for n in names {
                            attribute_resolver::add_to_layouts(p, n, *g);
                        }
                    }
                }
                return out(render_success(&format!("{name} split into {} groups.", fams.len())));
            }
            other => {
                return out(render_code(
                    format!("FeatureGroup has no property \"{other}\": Color, Icon, Sort, Merge or Split."),
                    "E-CMD-006",
                ))
            }
        }
    }
    if done.is_empty() {
        return out(render_code("Usage: Set FeatureGroup \"FX\" \"Paint\" \"#b07be6\" | Icon \"sparkles\" | Sort Family", "E-CMD-006"));
    }
    out(render_success(&format!("FeatureGroup \"{name}\": {}.", done.join(", "))))
}

/// `Move [UserProfile] FeatureGroup <g> At <n>`: the column / strip order.
fn group_move(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let (g, rest) = split_at_word(&words, &["At"]);
    let (Some(g), Some(n)) = (g.first(), rest.get(1).and_then(|n| n.parse::<usize>().ok())) else {
        return out(render_code("Usage: Move FeatureGroup \"Shutter\" At 2", "E-CMD-006"));
    };
    let (id, _, name) = match group_ref(layer(&state.root.show_data, t), g) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    let at = with_layer(state, t, |l, _| {
        let i = l.groups().keys().position(|n| *n == id).unwrap();
        let mut list: Vec<FeatureGroup> = l.groups().values().cloned().collect();
        let grp = list.remove(i);
        let at = n.saturating_sub(1).min(list.len());
        list.insert(at, grp);
        renumber_groups(l, list);
        at
    });
    out(render_success(&format!("FeatureGroup \"{name}\" is now number {}.", at + 1)))
}

fn group_delete(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let Some(g) = words.first() else {
        return out(render_code("Usage: Delete [UserProfile] FeatureGroup \"Haze\"", "E-CMD-006"));
    };
    let (id, guid, name) = match group_ref(layer(&state.root.show_data, t), g) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    let n = members(layer(&state.root.show_data, t), guid).len();
    if n > 0 {
        return out(render_code(
            format!("FeatureGroup \"{name}\" holds {n} attributes: move them first (Assign FeatureGroup \"{name}\" At FeatureGroup \"…\")."),
            "E-CMD-010",
        ));
    }
    with_layer(state, t, |l, _| {
        if l.rules().values().any(|r| r.group == guid) {
            let rules: Vec<AttributeRule> = l.rules().values().filter(|r| r.group != guid).cloned().collect();
            *l.rules_mut() = numbered(rules);
        }
        l.groups_mut().remove(&id);
    });
    if let Target::Profile(no) = t {
        for bar in state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes.encoder_bars.values_mut() {
            bar.buttons.retain(|b| b.group != guid);
        }
    }
    out(render_success(&format!("FeatureGroup \"{name}\" deleted.")))
}

fn group_list(state: &mut AppState, t: Target, _args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let show = &state.root.show_data;
    let l = layer(show, t);
    let used = attribute_resolver::used_attributes(show);
    let lines: Vec<String> = l
        .groups()
        .iter()
        .map(|(no, g)| {
            let m = members(l, g.meta.identity.guid);
            let in_use = m.iter().filter(|n| used.contains_key(*n)).count();
            format!("{no:>2}  {:<10} {in_use:>3} in use · {:>3} known   {}", g.meta.name().unwrap_or_default(), m.len(), g.features.join(" · "))
        })
        .collect();
    out(format!(
        "Feature groups{} (template {}):\n{}",
        if t == Target::Show { "" } else { " of the profile" },
        if l.template().is_empty() { "—" } else { l.template() },
        lines.join("\n")
    ))
}

fn group_info(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let Some(g) = words.first() else {
        return group_list(state, t, args);
    };
    let l = layer(&state.root.show_data, t);
    let (id, guid, _) = match group_ref(l, g) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    let grp = &l.groups()[&id];
    let m = members(l, guid);
    out(format!(
        "FeatureGroup {} \"{}\"\n  features: {}\n  colour: {} · icon: {}\n  attributes ({}): {}",
        id,
        grp.meta.name().unwrap_or_default(),
        if grp.features.is_empty() { "—".into() } else { grp.features.join(", ") },
        group_paint(grp).word(),
        match &grp.meta.annotation.icon {
            Some(IconRef::Builtin(i)) => i.as_str(),
            _ => "—",
        },
        m.len(),
        if m.is_empty() { "—".into() } else { m.join(", ") },
    ))
}

fn complete_groups(state: &AppState, prefix: &str) -> Vec<String> {
    let p = prefix.trim_matches('"').to_lowercase();
    state
        .root
        .show_data
        .feature_groups
        .values()
        .filter(|g| g.meta.name().unwrap_or_default().to_lowercase().starts_with(&p))
        .map(|g| quoted(g.meta.name().unwrap_or_default()))
        .collect()
}

// ── The attribute words on the registry (decisions §33 58, 86, 88) ──────────
//
// `UserProfile [x]` in front scopes them to that profile's own grouping (`Props::within`);
// without it they are the show's (cue timing per feature group).

thread_local! {
    static ATTR_LAYER: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

struct LayerScope(Option<u32>);

impl Drop for LayerScope {
    fn drop(&mut self) {
        ATTR_LAYER.with(|l| l.set(self.0));
    }
}

/// `UserProfile <x> …` scopes the attribute words to that profile.
pub fn profile_within(_: &mut AppState, key: &str) -> Result<Box<dyn std::any::Any>, String> {
    let no = key.parse::<u32>().map_err(|_| render_code(format!("No UserProfile {key}."), "E-CMD-007"))?;
    Ok(Box::new(LayerScope(ATTR_LAYER.with(|l| l.replace(Some(no))))))
}

/// The profile a `UserProfile <x> …` owner in front scopes to now (Theme, LockScreen).
pub fn scoped_profile() -> Option<u32> {
    ATTR_LAYER.with(std::cell::Cell::get)
}

/// The layer the attribute words act on now.
fn scoped() -> Target {
    ATTR_LAYER.with(std::cell::Cell::get).map_or(Target::Show, Target::Profile)
}

/// A profile's layer is changed only by its users and Admins.
fn may(state: &AppState, t: Target) -> Result<(), String> {
    match t {
        Target::Profile(no) => crate::commands::media::may_change_profile(state, no),
        Target::Show => Ok(()),
    }
}

/// A rendered attribute reply as a property write's result.
fn written(r: DispatchResult) -> Result<String, String> {
    match r {
        DispatchResult::Output(o) => crate::props::from_reply(o.text),
        _ => Err(render_code("That needs an answer first — run it without the property.", "E-CMD-010")),
    }
}

macro_rules! attr_prop {
    ($name:literal, $words:expr, $doc:literal) => {
        crate::props::Property {
            name: $name,
            kind: crate::props::PropKind::Choice($words),
            rights: crate::commands::registry::Rights::Setup,
            section: "Attribute",
            doc: $doc,
            access: crate::props::Access::Named {
                get: |_, _, _| String::new(),
                set: |s, k, p, v| {
                    let t = scoped();
                    may(s, t)?;
                    written(attribute_set(s, t, &[&format!("\"{k}\""), p, v]))
                },
            },
            ..crate::props::Property::BASE
        }
    };
}

static ATTRIBUTE_PROPS: [crate::props::Property; 2] = [
    attr_prop!("Readout", &["Percent", "Physical"], "how its value reads: Percent or Physical"),
    attr_prop!("Resolution", &["Coarse", "Fine", "Ultra"], "its encoder resolution"),
];

/// The attributes of the layer, by name (`Attribute "Gobo1"`).
static ATTRIBUTE_SETTINGS: crate::props::Props = crate::props::Props {
    lists: &[&ATTRIBUTE_PROPS],
    keyed: Some(crate::props::Keyed {
        rows: |s| layer(&s.root.show_data, scoped()).names(),
        row: |s, w| {
            let w = unquote(w);
            layer(&s.root.show_data, scoped()).names().into_iter().find(|n| n.eq_ignore_ascii_case(w))
        },
    }),
    label: Some(|_, k| format!("Attribute \"{k}\"")),
    ..crate::props::Props::BASE
};

macro_rules! group_prop {
    ($name:literal, $word:literal, $doc:literal) => {
        crate::props::Property {
            name: $name,
            section: "FeatureGroup",
            doc: $doc,
            access: crate::props::Access::Named {
                get: |_, _, _| String::new(),
                set: |s, k, _, v| {
                    let t = scoped();
                    may(s, t)?;
                    written(group_set(s, t, &[k, $word, &format!("\"{v}\"")]))
                },
            },
            ..crate::props::Property::BASE
        }
    };
}

/// A feature group's Paint (decisions 0294): its own, else the theme slot of its name.
pub fn group_paint(g: &FeatureGroup) -> lumacore_base::Paint {
    g.meta.annotation.paint.clone().unwrap_or_else(|| lumacore_base::Paint::Theme(g.meta.name().unwrap_or_default().to_string()))
}

/// `Set FeatureGroup "FX" "Paint" …`: the one Paint parser; `none` = the theme slot of its name.
fn set_group_paint(state: &mut AppState, key: &str, _: &str, value: &str) -> Result<String, String> {
    let t = scoped();
    may(state, t)?;
    let paint = lumacore_base::Paint::parse(value).map_err(|e| render_code(format!("FeatureGroup \"Paint\": {e}."), "E-SET-003"))?;
    // The row is the group's number (the keyed rows found it).
    let (id, name) = key.parse::<u32>().ok().and_then(|id| layer(&state.root.show_data, t).groups().get(&id).map(|g| (id, g.meta.name().unwrap_or_default().to_string()))).ok_or_else(|| render_code(format!("No FeatureGroup {key}."), "E-CMD-007"))?;
    with_layer(state, t, |l, _| l.groups_mut().get_mut(&id).expect("a found group").meta.annotation.paint = paint.clone());
    let shown = paint.map_or_else(|| format!("none (Theme \"{name}\")"), |p| p.word());
    Ok(format!("FeatureGroup \"{name}\" Paint = {shown}"))
}

fn get_group_paint(state: &AppState, key: &str, _: &str) -> String {
    let l = layer(&state.root.show_data, scoped());
    key.parse::<u32>().ok().and_then(|id| l.groups().get(&id)).and_then(|g| g.meta.annotation.paint.as_ref()).map(|p| p.word()).unwrap_or_default()
}

static FEATURE_GROUP_PROPS: [crate::props::Property; 2] = [
    crate::props::Property {
        name: "Paint",
        kind: crate::props::PropKind::Paint,
        section: "FeatureGroup",
        doc: "its colour on the encoder bar, the sheets and the preset pool: #rrggbb, a palette colour, Theme \"<slot>\", or none = the theme slot of its name",
        access: crate::props::Access::Named { get: get_group_paint, set: set_group_paint },
        ..crate::props::Property::BASE
    },
    group_prop!("Icon", "Icon", "its icon on the encoder bar"),
];

/// The layer's feature groups by number or name; under a cue they carry its timing (§33 87).
static FEATURE_GROUP_SETTINGS: crate::props::Props = crate::props::Props {
    lists: &[&FEATURE_GROUP_PROPS],
    keyed: Some(crate::props::Keyed {
        rows: |s| layer(&s.root.show_data, scoped()).groups().keys().map(u32::to_string).collect(),
        row: |s, w| {
            let w = unquote(w);
            let groups = layer(&s.root.show_data, scoped()).groups();
            w.parse::<u32>().ok().filter(|n| groups.contains_key(n)).or_else(|| groups.iter().find(|(_, g)| g.meta.name().unwrap_or_default().eq_ignore_ascii_case(w)).map(|(n, _)| *n)).map(|n| n.to_string())
        },
    }),
    scoped: &[&crate::commands::playback::GROUP_TIMING],
    label: Some(|s, k| {
        let name = k.parse::<u32>().ok().and_then(|n| layer(&s.root.show_data, scoped()).groups().get(&n).and_then(|g| g.meta.name().map(str::to_string)));
        format!("FeatureGroup \"{}\"", name.unwrap_or_else(|| k.to_string()))
    }),
    ..crate::props::Props::BASE
};

// ── Typed: Assign (into a group, a merge), Sort, Split (decisions §33 86, 88) ──

use lumacore_syntax::ast::{Id, Item, Range, Segment, Target as LumTarget, Value as LumValue};

/// The layer a target names (`UserProfile [x] …` = that profile's grouping, else the show's)
/// and its last segment (`Attribute "a" + "b"`, `FeatureGroup "G"`).
fn layer_of(state: &AppState, target: &LumTarget) -> Result<(Target, Segment), String> {
    let [(None, Item::Path(segments))] = target.items.as_slice() else {
        return Err(render_code("Name the attributes or the feature group (owner first: UserProfile … FeatureGroup …).", "E-CMD-008"));
    };
    let (last, owners) = segments.split_last().expect("a path has a segment");
    let t = match owners {
        [] => Target::Show,
        [profile] => {
            let words = crate::lum::vocabulary();
            if !crate::objects::USER_PROFILE.matches(&words.word(profile.object.node).name) {
                return Err(render_code("Only a UserProfile owns its own grouping.", "E-CMD-008"));
            }
            match &profile.ids {
                None => own_profile(state).map(Target::Profile).ok_or_else(|| render_code("You have no UserProfile.", "E-CMD-011"))?,
                Some(_) => {
                    let one = LumTarget { items: vec![(None, Item::Path(vec![profile.clone()]))] };
                    let r = crate::resolve::resolve_one(state, &one, &crate::objects::USER_PROFILE)?;
                    let [m] = r.members.as_slice() else {
                        return Err(render_code("Name one UserProfile.", "E-CMD-008"));
                    };
                    Target::Profile(m.no)
                }
            }
        }
        _ => return Err(render_code("UserProfile [x] Attribute|FeatureGroup … — one owner.", "E-CMD-008")),
    };
    Ok((t, last.clone()))
}

/// A segment's ids as the attribute words take them: quoted names (patterns too: `"Shutter*"`)
/// and numbers, joined by `+`.
fn id_words(segment: &Segment) -> Vec<String> {
    let mut out = Vec::new();
    for (op, range) in segment.ids.iter().flat_map(|i| i.ranges.iter()) {
        if op.is_some() {
            out.push("+".to_string());
        }
        if let Range::One(Id::Name(n) | Id::Number(n)) = range {
            out.push(quoted(&n.node));
        }
    }
    out
}

fn segment_is(segment: &Segment, obj: &ObjectDef) -> bool {
    obj.matches(&crate::lum::vocabulary().word(segment.object.node).name)
}

/// `Assign [UserProfile [x]] Attribute <names> | FeatureGroup <g> At [UserProfile] FeatureGroup <h>`.
fn assign_in(state: &mut AppState, target: &LumTarget, to: &LumValue, flags: &[&str]) -> DispatchResult {
    let (t, source) = match layer_of(state, target) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    if let Err(e) = may(state, t) {
        return out(e);
    }
    let dest = match to {
        LumValue::Target(d) => match d.items.as_slice() {
            [(None, Item::Path(segs))] => segs.last().filter(|s| segment_is(s, &FEATURE_GROUP)).map(id_words),
            _ => None,
        },
        _ => None,
    };
    let Some(dest) = dest.filter(|d| d.len() == 1) else {
        return out(render_code("… At FeatureGroup \"G\" — name one feature group.", "E-CMD-008"));
    };
    let names = id_words(&source);
    let mut line: Vec<&str> = names.iter().map(String::as_str).collect();
    if segment_is(&source, &ATTRIBUTE) {
        line.extend(["FeatureGroup", &dest[0]]);
        line.extend_from_slice(flags);
        attribute_set(state, t, &line)
    } else if segment_is(&source, &FEATURE_GROUP) && names.len() == 1 {
        line.extend(["Merge", &dest[0]]);
        line.extend_from_slice(flags);
        group_set(state, t, &line)
    } else {
        out(render_code("Assign puts an Attribute into a FeatureGroup, or merges one FeatureGroup into another.", "E-CMD-008"))
    }
}

/// `Assign Attribute <names> At FeatureGroup <g> [/WithSubs | /Only]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Assign", object = "Attribute")]
pub struct AssignAttribute {
    #[lum(target)]
    pub target: LumTarget,
    #[lum(at = "destination")]
    pub to: LumValue,
    #[lum(option = "WithSubs")]
    pub with_subs: bool,
    #[lum(option = "Only")]
    pub only: bool,
}

impl crate::commands::typed::Command for AssignAttribute {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let flags: Vec<&str> = [(self.with_subs, "/WithSubs"), (self.only, "/Only")].into_iter().filter(|(on, _)| *on).map(|(_, f)| f).collect();
        assign_in(state, &self.target, &self.to, &flags)
    }
}

/// `Assign FeatureGroup <g> At FeatureGroup <h>`: g merges into h (asks).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Assign", object = "FeatureGroup")]
pub struct AssignFeatureGroup {
    #[lum(target)]
    pub target: LumTarget,
    #[lum(at = "destination")]
    pub to: LumValue,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl crate::commands::typed::Command for AssignFeatureGroup {
    fn run(self, state: &mut AppState) -> DispatchResult {
        assign_in(state, &self.target, &self.to, if self.confirmed { &["/NoConfirmation"] } else { &[] })
    }
}

/// `Assign UserProfile [x] Attribute|FeatureGroup … At UserProfile FeatureGroup …`: your own grouping.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Assign", object = "UserProfile")]
pub struct AssignInProfile {
    #[lum(target)]
    pub target: LumTarget,
    #[lum(at = "destination")]
    pub to: LumValue,
    #[lum(option = "WithSubs")]
    pub with_subs: bool,
    #[lum(option = "Only")]
    pub only: bool,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl crate::commands::typed::Command for AssignInProfile {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let flags: Vec<&str> =
            [(self.with_subs, "/WithSubs"), (self.only, "/Only"), (self.confirmed, "/NoConfirmation")].into_iter().filter(|(on, _)| *on).map(|(_, f)| f).collect();
        assign_in(state, &self.target, &self.to, &flags)
    }
}

/// The `/By` key as written (a declared word or quoted text).
fn by_word(v: &Option<LumValue>) -> Option<String> {
    match v.as_ref()? {
        LumValue::Word(w) => Some(crate::lum::vocabulary().word(w.node).name.clone()),
        LumValue::Text(t) => Some(t.node.clone()),
        _ => None,
    }
}

/// `Sort|Split [UserProfile [x]] FeatureGroup "G" /By <key>`.
fn regroup(state: &mut AppState, target: &LumTarget, how: &str, by: Option<String>, confirmed: bool) -> DispatchResult {
    let (t, group) = match layer_of(state, target) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    if !segment_is(&group, &FEATURE_GROUP) {
        return out(render_code(format!("{how} takes a FeatureGroup today: {how} FeatureGroup \"G\" /By …"), "E-CMD-004"));
    }
    if let Err(e) = may(state, t) {
        return out(e);
    }
    let names = id_words(&group);
    let [g] = names.as_slice() else {
        return out(render_code(format!("{how} one FeatureGroup at a time."), "E-CMD-008"));
    };
    let Some(by) = by else {
        return out(render_code(format!("{how} FeatureGroup {g} /By {}", if how == "Sort" { "Name|Family|MainFirst" } else { "Family" }), "E-CMD-006"));
    };
    let mut line: Vec<&str> = vec![g, how, &by];
    if confirmed {
        line.push("/NoConfirmation");
    }
    group_set(state, t, &line)
}

/// `Sort <object> /By <key>` (decisions §33 88): a feature group's attributes today.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Sort")]
pub struct Sort {
    #[lum(target)]
    pub target: LumTarget,
    #[lum(option = "By")]
    pub by: Option<LumValue>,
}

impl crate::commands::typed::Command for Sort {
    fn run(self, state: &mut AppState) -> DispatchResult {
        regroup(state, &self.target, "Sort", by_word(&self.by), false)
    }
}

/// `Split <object> /By <key>` (decisions §33 88): a feature group by attribute family (asks).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Split")]
pub struct Split {
    #[lum(target)]
    pub target: LumTarget,
    #[lum(option = "By")]
    pub by: Option<LumValue>,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl crate::commands::typed::Command for Split {
    fn run(self, state: &mut AppState) -> DispatchResult {
        regroup(state, &self.target, "Split", by_word(&self.by), self.confirmed)
    }
}

pub static FEATURE_GROUP: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "FeatureGroup",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Select,
        aliases: &["featuregroups", "fg"],
        doc: Doc {
            display: "FeatureGroup",
            category: crate::commands::help::HelpCategory::Programmer,
            also_applies_to: &[],
            syntax: "Store [UserProfile] FeatureGroup \"name\" | Label FeatureGroup <g> \"new\" | Set FeatureGroup <g> \"Paint\" \"#rrggbb\" | \"Icon\" \"name\" | Sort FeatureGroup <g> /By Name|Family|MainFirst | Assign FeatureGroup <g> At FeatureGroup <g2> | Split FeatureGroup <g> /By Family | Move FeatureGroup <g> At <n> | Delete FeatureGroup <g> | List|Info FeatureGroup [<g>]",
            description: "A group of attributes (decisions §30 \"12f-a\"): a bare FeatureGroup is the show's — \
                          cue timing per feature group reads it; UserProfile FeatureGroup … is your own \
                          grouping for your encoder bar. Named (\"FX\") or numbered (its column order). Merge \
                          and Split ask first (/NoConfirmation skips); Delete only an empty group.",
            examples: &[
                "Store FeatureGroup \"Haze\"",
                "Store UserProfile FeatureGroup \"Busk\"",
                "Label FeatureGroup \"FX\" \"Effects\"",
                "Set FeatureGroup \"FX\" \"Paint\" \"#b07be6\"; Set FeatureGroup \"FX\" \"Icon\" \"sparkles\"",
                "Sort FeatureGroup \"Beam\" /By Family",
                "Assign UserProfile FeatureGroup \"Gobo\" At UserProfile FeatureGroup \"FX\"",
                "Move FeatureGroup \"Shutter\" At 2",
                "List UserProfile \"Bob\" FeatureGroup",
            ],
            related: &["attribute", "attributerule", "encoderbar", "cue"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::setup,
        planned: &[Verb::Select],
        flags: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Store, |s, a| group_store(s, Target::Show, a)),
        (Verb::Label, |s, a| group_label(s, Target::Show, a)),
        (Verb::Assign, crate::commands::typed::typed::<AssignFeatureGroup>),
        (Verb::Move, |s, a| group_move(s, Target::Show, a)),
        (Verb::Delete, |s, a| group_delete(s, Target::Show, a)),
        (Verb::List, |s, a| group_list(s, Target::Show, a)),
        (Verb::Info, |s, a| group_info(s, Target::Show, a)),
    ],
    delete_ask: None,
    complete: Some(complete_groups),
    props: Some(&FEATURE_GROUP_SETTINGS),
    tree: None,
};

// ─── AttributeRule ───────────────────────────────────────────────────────────

fn numbered(rules: Vec<AttributeRule>) -> std::collections::BTreeMap<u32, AttributeRule> {
    rules
        .into_iter()
        .enumerate()
        .map(|(i, r)| (i as u32 + 1, r))
        .collect()
}

/// `Store [UserProfile] AttributeRule "Pixel*" FeatureGroup "FX" [Feature "x"] [/Keyword]`:
/// a learned rule, placed before the keyword rules (a /Keyword rule at the end).
fn rule_store(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let usage = || out(render_code("Usage: Store [UserProfile] AttributeRule \"Pixel*\" FeatureGroup \"FX\" [Feature \"Effects\"] [/Keyword]", "E-CMD-006"));
    let [pattern, fg, g, rest @ ..] = words.as_slice() else { return usage() };
    if !FEATURE_GROUP.matches(fg) {
        return usage();
    }
    let (gid, gname) = match group_ref(layer(&state.root.show_data, t), g) {
        Ok((_, g, n)) => (g, n),
        Err(e) => return out(e),
    };
    let feature = match rest {
        [f, v, ..] if f.eq_ignore_ascii_case("Feature") => Some(unquote(v).to_string()),
        _ => None,
    };
    let keyword = F::has(args, &KEYWORD);
    let rule = AttributeRule {
        meta: NodeMeta::default(),
        pattern: unquote(pattern).to_string(),
        group: gid,
        feature,
        kind: if keyword { RuleKind::Keyword } else { RuleKind::Family },
        class: None,
        learned: true,
    };
    let at = with_layer(state, t, |l, _| {
        let mut rules: Vec<AttributeRule> = l.rules().values().cloned().collect();
        let at = if keyword { rules.len() } else { rules.iter().position(|r| r.kind == RuleKind::Keyword).unwrap_or(rules.len()) };
        rules.insert(at, rule);
        *l.rules_mut() = numbered(rules);
        at
    });
    out(render_success(&format!("AttributeRule {} \"{}\" → {gname} (learned).", at + 1, unquote(pattern))))
}

fn rule_no(l: &dyn AttributeLayer, word: Option<&&str>) -> Result<u32, String> {
    word.and_then(|w| w.parse::<u32>().ok())
        .filter(|n| l.rules().contains_key(n))
        .ok_or_else(|| render_code("Name a rule by its number (List AttributeRule).", "E-CMD-007"))
}

fn rule_move(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let (n, rest) = split_at_word(&words, &["At"]);
    let from = match rule_no(layer(&state.root.show_data, t), n.first()) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let Some(to) = rest.get(1).and_then(|n| n.parse::<usize>().ok()) else {
        return out(render_code("Usage: Move AttributeRule 15 At 3", "E-CMD-006"));
    };
    let at = with_layer(state, t, |l, _| {
        let i = l.rules().keys().position(|n| *n == from).unwrap();
        let mut rules: Vec<AttributeRule> = l.rules().values().cloned().collect();
        let r = rules.remove(i);
        let at = to.saturating_sub(1).min(rules.len());
        rules.insert(at, r);
        *l.rules_mut() = numbered(rules);
        at
    });
    out(render_success(&format!("AttributeRule {from} is now {}.", at + 1)))
}

fn rule_delete(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let n = match rule_no(layer(&state.root.show_data, t), words.first()) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let pattern = with_layer(state, t, |l, _| {
        let r = l.rules_mut().remove(&n).unwrap();
        let rules: Vec<AttributeRule> = l.rules().values().cloned().collect();
        *l.rules_mut() = numbered(rules);
        r.pattern
    });
    out(render_success(&format!("AttributeRule {n} \"{pattern}\" deleted.")))
}

fn rule_list(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let l = layer(&state.root.show_data, t);
    let learned_only = F::has(args, &LEARNED);
    let lines: Vec<String> = l
        .rules()
        .iter()
        .filter(|(_, r)| !learned_only || r.learned)
        .map(|(no, r)| {
            format!(
                "{:>3}  {:<26} → {:<10}{} {}{}",
                no,
                r.pattern,
                group_name(l, Some(r.group)),
                r.feature.as_deref().map(|f| format!(" · {f}")).unwrap_or_default(),
                if r.kind == RuleKind::Keyword { "keyword" } else { "family" },
                if r.learned { " · learned" } else { "" }
            )
        })
        .collect();
    out(if lines.is_empty() { "No rules.".into() } else { lines.join("\n") })
}

fn rule_info(state: &mut AppState, t: Target, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, t);
    let words = plain(args);
    let show = &state.root.show_data;
    let l = layer(show, t);
    if let Some(w) = words.first().filter(|w| w.parse::<u32>().is_err()) {
        // `Info AttributeRule "PixelMSpeed"`: which rule would place a name.
        let ctx = Ctx::of(show);
        let r = attribute_resolver::resolve_in(l, &ctx, &lumacore_fixtures::FileAttribute { name: unquote(w).to_string(), ..Default::default() });
        return out(format!("\"{}\" → {} ({}: {})", unquote(w), group_name(l, Some(r.group)), r.placement.how.word(), r.placement.why));
    }
    let n = match rule_no(l, words.first()) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let r = &l.rules()[&n];
    let hits: Vec<String> = l.names().into_iter().filter(|a| matches(&r.pattern, a)).collect();
    out(format!(
        "AttributeRule {n}: \"{}\" → {} ({}{})\n  matches: {}",
        r.pattern,
        group_name(l, Some(r.group)),
        if r.kind == RuleKind::Keyword { "keyword" } else { "family" },
        if r.learned { ", learned" } else { "" },
        if hits.is_empty() { "—".into() } else { hits.join(", ") }
    ))
}

/// `Call AttributeTemplate "<name>" At ShowData | UserProfile [x]` (decisions §33 89): regroup a
/// layer by a built-in grouping recipe, once (asks; learned rules stay).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Call", object = "AttributeTemplate")]
pub struct CallTemplate {
    #[lum(target)]
    pub target: LumTarget,
    #[lum(at = "destination")]
    pub to: LumValue,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl crate::commands::typed::Command for CallTemplate {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let [(None, Item::Path(segments))] = self.target.items.as_slice() else {
            return out(render_code("Call AttributeTemplate \"<name>\" At ShowData | UserProfile", "E-CMD-006"));
        };
        let names = segments.last().map(id_words).unwrap_or_default();
        let [name] = names.as_slice() else {
            return out(render_code("Name one template: List AttributeTemplate.", "E-CMD-006"));
        };
        // Where it goes: the show's grouping, or a profile's own.
        let t = match &self.to {
            LumValue::Target(d) => match d.items.as_slice() {
                [(None, Item::Path(segs))] if segs.len() == 1 && segment_is(&segs[0], &crate::objects::SHOW_DATA) => Ok(Target::Show),
                [(None, Item::Path(segs))] if segs.len() == 1 && segment_is(&segs[0], &crate::objects::USER_PROFILE) => {
                    match &segs[0].ids {
                        None => own_profile(state).map(Target::Profile).ok_or_else(|| render_code("You have no UserProfile.", "E-CMD-011")),
                        Some(_) => crate::resolve::resolve_one(state, d, &crate::objects::USER_PROFILE).and_then(|r| match r.members.as_slice() {
                            [m] => Ok(Target::Profile(m.no)),
                            _ => Err(render_code("Name one UserProfile.", "E-CMD-008")),
                        }),
                    }
                }
                _ => Err(render_code("A template goes At ShowData or At UserProfile [x].", "E-CMD-008")),
            },
            _ => Err(render_code("A template goes At ShowData or At UserProfile [x].", "E-CMD-008")),
        };
        let t = match t {
            Ok(t) => t,
            Err(e) => return out(e),
        };
        if let Err(e) = may(state, t) {
            return out(e);
        }
        set_template(state, t, name, if self.confirmed { &["/NoConfirmation"] } else { &[] })
    }
}

/// Into the show's grouping it's Setup work (cue timing); into a profile its users' (§30 "12f-a").
fn template_rights(verb: Verb, args: &[&str]) -> crate::commands::registry::Rights {
    use crate::commands::registry::Rights;
    match verb {
        Verb::Call if args.iter().any(|a| crate::objects::USER_PROFILE.matches(a)) => Rights::Guest,
        Verb::Call => Rights::Setup,
        _ => Rights::Guest,
    }
}

fn template_list(_: &mut AppState, _: &[&str]) -> DispatchResult {
    let std = attribute_resolver::standard();
    out(format!(
        "Attribute templates (Call AttributeTemplate \"<name>\" At ShowData | UserProfile):\n{}",
        std.template_names().iter().map(|n| format!("  \"{n}\"")).collect::<Vec<_>>().join("\n")
    ))
}

/// The built-in grouping recipes ("GDTF standard", "LumaCore FX"), by name.
static TEMPLATE_ROWS: crate::props::Props = crate::props::Props {
    keyed: Some(crate::props::Keyed {
        rows: |_| attribute_resolver::standard().template_names().into_iter().map(str::to_string).collect(),
        row: |_, w| attribute_resolver::standard().template(unquote(w)).map(|t| t.name.clone()),
    }),
    ..crate::props::Props::BASE
};

pub static ATTRIBUTE_TEMPLATE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "AttributeTemplate",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::List,
        aliases: &["attributetemplates"],
        doc: Doc {
            display: "AttributeTemplate",
            category: crate::commands::help::HelpCategory::Programmer,
            also_applies_to: &[],
            syntax: "List AttributeTemplate | Call AttributeTemplate \"<name>\" At ShowData | UserProfile [<n>|\"name\"]",
            description: "A built-in grouping recipe (decisions §33 89): feature groups and the rules \
                          that place each attribute (\"GDTF standard\": GDTF's groups; \"LumaCore \
                          FX\": gobos, prisms and spins under FX). Call applies one once — the show's \
                          grouping (cue timing per feature group, Setup) or a profile's own (its encoder \
                          bar); learned rules stay. Asks first.",
            examples: &[
                "List AttributeTemplate",
                "Call AttributeTemplate \"LumaCore FX\" At ShowData",
                "Call AttributeTemplate \"GDTF standard\" At UserProfile",
            ],
            related: &["attribute", "featuregroup", "attributerule"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: template_rights,
        planned: &[],
        flags: &[],
    },
    pool: None,
    verbs: &[(Verb::Call, crate::commands::typed::typed::<CallTemplate>), (Verb::List, template_list)],
    delete_ask: None,
    complete: None,
    props: Some(&TEMPLATE_ROWS),
    tree: None,
};

pub static ATTRIBUTE_RULE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "AttributeRule",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        aliases: &["attributerules"],
        doc: Doc {
            display: "AttributeRule",
            category: crate::commands::help::HelpCategory::Programmer,
            also_applies_to: &[],
            syntax: "Store [UserProfile] AttributeRule \"pattern\" FeatureGroup <g> [Feature \"f\"] [/Keyword] | Move AttributeRule <n> At <m> | Delete AttributeRule <n> | List AttributeRule [/Learned] | Info AttributeRule <n> | \"name\"",
            description: "The resolver's rules (decisions §30 \"12f-a\"): where an attribute nobody has seen \
                          goes — the show's, or a profile's own (UserProfile AttributeRule …). Tried in \
                          order, the first match wins: * = any characters, {n} = digits, case-insensitive, \
                          whole name. A stored rule is learned: it is someone's own choice and wins over the \
                          standard table. Info with a name says which rule would place it.",
            examples: &[
                "Store AttributeRule \"Pixel*\" FeatureGroup \"FX\"",
                "Store UserProfile AttributeRule \"Gobo{n}*\" FeatureGroup \"Gobo\"",
                "Move AttributeRule 15 At 3",
                "List AttributeRule /Learned",
                "Info AttributeRule \"PixelMSpeed\"",
            ],
            related: &["attribute", "featuregroup"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: crate::rights::setup,
        planned: &[],
        flags: &[LEARNED, KEYWORD],
    },
    pool: None,
    verbs: &[
        (Verb::Store, |s, a| rule_store(s, Target::Show, a)),
        (Verb::Move, |s, a| rule_move(s, Target::Show, a)),
        (Verb::Delete, |s, a| rule_delete(s, Target::Show, a)),
        (Verb::List, |s, a| rule_list(s, Target::Show, a)),
        (Verb::Info, |s, a| rule_info(s, Target::Show, a)),
    ],
    delete_ask: None,
    complete: None,
    props: Some(&RULE_SETTINGS),
    tree: None,
};

// ── AttributeRule properties (the Rules window's cells, decisions 0294) ──

/// A rule of the scoped layer (the show's, or `UserProfile x`'s), by its number.
static RULE_SETTINGS: crate::props::Props = crate::props::Props {
    lists: &[&RULE_PROPS],
    keyed: Some(crate::props::Keyed {
        rows: |s| layer(&s.root.show_data, scoped()).rules().keys().map(u32::to_string).collect(),
        row: |s, w| {
            let n = unquote(w).parse::<u32>().ok()?;
            layer(&s.root.show_data, scoped()).rules().contains_key(&n).then(|| n.to_string())
        },
    }),
    label: Some(|_, k| format!("AttributeRule {k}")),
    ..crate::props::Props::BASE
};

macro_rules! rule_prop {
    ($name:literal, $doc:literal) => {
        crate::props::Property {
            name: $name,
            rights: crate::commands::registry::Rights::Setup,
            section: "AttributeRule",
            doc: $doc,
            access: crate::props::Access::Named { get: rule_get, set: rule_set },
            ..crate::props::Property::BASE
        }
    };
}

static RULE_PROPS: [crate::props::Property; 3] = [
    rule_prop!("Pattern", "the names it places: * = any characters, {n} = digits, case-insensitive, whole name (a change makes it learned)"),
    rule_prop!("FeatureGroup", "the feature group it places into, by name or number (a change makes it learned)"),
    rule_prop!("Feature", "the feature inside the group, or none"),
];

fn rule_get(state: &AppState, key: &str, name: &str) -> String {
    let l = layer(&state.root.show_data, scoped());
    let Some(r) = key.parse::<u32>().ok().and_then(|n| l.rules().get(&n)) else { return String::new() };
    // tidy: not-keywords (the rule's own property names)
    match name {
        "Pattern" => r.pattern.clone(),
        "FeatureGroup" => group_name(l, Some(r.group)),
        _ => r.feature.clone().unwrap_or_default(),
    }
}

/// A rule's cell: the pattern, its feature group or its feature. A user's change is their own
/// choice: the rule becomes learned (it beats the standard table).
fn rule_set(state: &mut AppState, key: &str, name: &str, value: &str) -> Result<String, String> {
    let t = scoped();
    may(state, t)?;
    ensure_seeded(state, t);
    let n: u32 = key.parse().map_err(|_| render_code("Name a rule by its number (List AttributeRule).", "E-CMD-007"))?;
    let value = unquote(value.trim()).to_string();
    let group = match name {
        // The group by name or number, as the FeatureGroup rows read it.
        "FeatureGroup" => {
            let row = FEATURE_GROUP_SETTINGS.keyed.as_ref().and_then(|k| (k.row)(state, &value)).and_then(|n| n.parse::<u32>().ok());
            let l = layer(&state.root.show_data, t);
            let g = row.and_then(|n| l.groups().get(&n)).ok_or_else(|| render_code(format!("No FeatureGroup \"{value}\" in this layer."), "E-CMD-007"))?;
            Some((g.meta.identity.guid, g.meta.name().unwrap_or_default().to_string()))
        }
        _ => None,
    };
    if name == "Pattern" && value.is_empty() {
        return Err(render_code("A rule needs a pattern (Delete AttributeRule removes it).", "E-CMD-008"));
    }
    let shown = group.as_ref().map_or_else(|| value.clone(), |(_, n)| n.clone());
    with_layer(state, t, |l, _| {
        let Some(r) = l.rules_mut().get_mut(&n) else { return Err(render_code(format!("No AttributeRule {n}."), "E-CMD-007")) };
        match (name, group) {
            ("Pattern", _) => r.pattern = value.clone(),
            (_, Some((g, _))) => r.group = g,
            _ => r.feature = Some(value.clone()).filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("none")),
        }
        if name != "Feature" {
            r.learned = true;
            r.class = None;
        }
        Ok(())
    })?;
    Ok(format!("AttributeRule {n} {name} = {shown}"))
}

// ─── EncoderBar (a profile's layouts) ────────────────────────────────────────

fn bar_no(state: &AppState, no: u32, word: Option<&&str>) -> Result<u32, String> {
    let bars = &state.root.show_data.user_profiles[&no].content.attributes.encoder_bars;
    let found = word.and_then(|w| {
        let w = unquote(w);
        match w.parse::<u32>() {
            Ok(n) => bars.contains_key(&n).then_some(n),
            Err(_) => bars.iter().find(|(_, b)| b.meta.name().unwrap_or_default().eq_ignore_ascii_case(w)).map(|(n, _)| *n),
        }
    });
    found.ok_or_else(|| {
        let names: Vec<&str> = bars.values().map(|b| b.meta.name().unwrap_or_default()).collect();
        render_code(format!("No EncoderBar {} (there are {}).", word.map(|w| unquote(w)).unwrap_or(""), names.join(", ")), "E-CMD-007")
    })
}

/// `Store UserProfile EncoderBar "Busking"`: a copy of the first layout (or a
/// fresh one from the groups).
fn bar_store(state: &mut AppState, no: u32, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, Target::Profile(no));
    let words = plain(args);
    if words.is_empty() {
        // Bare (the Encoder bar tab's New): one ask for the name.
        let prefix = Target::Profile(no).prefix(state);
        return DispatchResult::Ask(Box::new(
            crate::dialogs::Ask::new("New encoder bar layout", "A copy of the first layout.")
                .meta(format!("Store {prefix}EncoderBar"), None)
                .input("name", "Name", "", lumacore_wire::InputKind::Text)
                .button("Create", lumacore_wire::ButtonKind::Primary, format!("Store {prefix}EncoderBar \"{{name}}\""))
                .cancel("Cancel")
                .flags("the name: Store UserProfile EncoderBar \"name\""),
        ));
    }
    let Some(name) = words.first().map(|w| unquote(w).to_string()).filter(|n: &String| !n.is_empty()) else {
        return out(render_code("Usage: Store UserProfile EncoderBar \"name\"", "E-CMD-006"));
    };
    let p = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes;
    if p.encoder_bars.values().any(|b| b.meta.name().unwrap_or_default().eq_ignore_ascii_case(&name)) {
        return out(render_code(format!("An EncoderBar \"{name}\" exists already."), "E-CMD-010"));
    }
    let id = p.encoder_bars.keys().max().copied().unwrap_or(0) + 1;
    let mut bar = p.encoder_bars.values().next().cloned().unwrap_or_else(|| attribute_resolver::default_layout(p, &name));
    bar.meta.identity.guid = uuid::Uuid::new_v4();
    bar.meta.identity.name = Some(name.clone());
    p.encoder_bars.insert(id, bar);
    out(render_success(&format!("EncoderBar {id} \"{name}\" stored.")))
}

fn bar_label(state: &mut AppState, no: u32, args: &[&str]) -> DispatchResult {
    let words = plain(args);
    let [b, name, ..] = words.as_slice() else {
        return out(render_code("Usage: Label UserProfile EncoderBar \"Busking\" \"Festival\"", "E-CMD-006"));
    };
    let id = match bar_no(state, no, Some(b)) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let bar = state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes.encoder_bars.get_mut(&id).unwrap();
    let old = bar.meta.identity.name.replace(unquote(name).to_string()).unwrap_or_default();
    out(render_success(&format!("EncoderBar \"{old}\" is now \"{}\".", unquote(name))))
}

fn bar_delete(state: &mut AppState, no: u32, args: &[&str]) -> DispatchResult {
    let words = plain(args);
    let id = match bar_no(state, no, words.first()) {
        Ok(n) => n,
        Err(e) => return out(e),
    };
    let bars = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes.encoder_bars;
    if bars.len() == 1 {
        return out(render_code("A profile keeps at least one encoder bar layout.", "E-CMD-010"));
    }
    let b = bars.remove(&id).unwrap();
    out(render_success(&format!("EncoderBar \"{}\" deleted.", b.meta.name().unwrap_or_default())))
}

/// `Move UserProfile EncoderBar <b> FeatureGroup <g> At <n>` (strip order) ·
/// `… Attribute <a> At <n>` (slot n of its strip button; swaps with what is there).
fn bar_move(state: &mut AppState, no: u32, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, Target::Profile(no));
    let words = plain(args);
    let usage = || out(render_code("Usage: Move UserProfile EncoderBar \"Default\" FeatureGroup \"Shutter\" At 2 | Attribute \"Zoom\" At 5", "E-CMD-006"));
    let [b, kind, what, at, n, ..] = words.as_slice() else { return usage() };
    if !at.eq_ignore_ascii_case("At") {
        return usage();
    }
    let Ok(n) = n.parse::<usize>() else { return usage() };
    let id = match bar_no(state, no, Some(b)) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    let p = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes;
    if FEATURE_GROUP.matches(kind) {
        let gid = match group_ref(p, what) {
            Ok((_, g, _)) => g,
            Err(e) => return out(e),
        };
        let bar = p.encoder_bars.get_mut(&id).unwrap();
        let Some(i) = bar.buttons.iter().position(|x| x.group == gid) else {
            return out(render_code("That group has no strip button on this layout.", "E-CMD-007"));
        };
        let btn = bar.buttons.remove(i);
        let at = n.saturating_sub(1).min(bar.buttons.len());
        bar.buttons.insert(at, btn);
        return out(render_success(&format!("Strip button {} is now number {}.", unquote(what), at + 1)));
    }
    if ATTRIBUTE.matches(kind) {
        let name = unquote(what).to_string();
        let bar = p.encoder_bars.get_mut(&id).unwrap();
        let Some(btn) = bar.buttons.iter_mut().find(|x| x.slots.iter().any(|s| s.as_deref() == Some(name.as_str()))) else {
            return out(render_code(format!("\"{name}\" isn't on this layout."), "E-CMD-007"));
        };
        let from = btn.slots.iter().position(|s| s.as_deref() == Some(name.as_str())).unwrap();
        let to = n.max(1) - 1;
        while btn.slots.len() <= to {
            btn.slots.push(None);
        }
        btn.slots.swap(from, to);
        while btn.slots.last().is_some_and(Option::is_none) {
            btn.slots.pop();
        }
        return out(render_success(&format!("\"{name}\" is now in slot {n}.")));
    }
    usage()
}

fn bar_list(state: &mut AppState, no: u32, _args: &[&str]) -> DispatchResult {
    ensure_seeded(state, Target::Profile(no));
    let p = &state.root.show_data.user_profiles[&no].content.attributes;
    let lines: Vec<String> = p.encoder_bars.iter().map(|(n, b)| format!("{n:>2}  {:<14} {} strip buttons", b.meta.name().unwrap_or_default(), b.buttons.len())).collect();
    out(lines.join("\n"))
}

fn bar_info(state: &mut AppState, no: u32, args: &[&str]) -> DispatchResult {
    ensure_seeded(state, Target::Profile(no));
    let words = plain(args);
    let id = match bar_no(state, no, words.first()) {
        Ok(x) => x,
        Err(e) => return out(e),
    };
    let p = &state.root.show_data.user_profiles[&no].content.attributes;
    let used = attribute_resolver::used_attributes(&state.root.show_data);
    let bar = &p.encoder_bars[&id];
    let mut lines = vec![format!("EncoderBar {id} \"{}\"", bar.meta.name().unwrap_or_default())];
    for b in &bar.buttons {
        let slots: Vec<String> = b
            .slots
            .iter()
            .map(|s| match s {
                Some(n) if used.contains_key(n) => n.clone(),
                Some(n) => format!("({n})"),
                None => "·".into(),
            })
            .collect();
        lines.push(format!("  {:<10} {}", group_name(p, Some(b.group)), slots.chunks(4).map(|c| c.join(" ")).collect::<Vec<_>>().join(" | ")));
    }
    lines.push("  (name) = no patched fixture has it: hidden on the bar".into());
    out(lines.join("\n"))
}

/// `Edit [UserProfile] EncoderBar`: Users & Profiles at the Encoder bar tab
/// (your own layouts; another profile's are changed by their lines).
fn bar_edit(state: &mut AppState, no: u32, _args: &[&str]) -> DispatchResult {
    if own_profile(state) != Some(no) {
        return out(render_code(
            "The Encoder bar tab shows your own layouts. Another profile's: List / Move UserProfile \"x\" EncoderBar …",
            "E-CMD-010",
        ));
    }
    let open = format!("\"{no} Content EncoderBars\"");
    crate::commands::menus::menu_call(state, &["\"Users\"", "/Tab", "Profiles", "/Open", &open])
}

/// Encoder bar lines change the master's show; opening the tab stays here.
fn bar_scope(verb: Verb, args: &[&str]) -> crate::commands::registry::Scope {
    attribute_scope(verb, args)
}

fn own(state: &mut AppState, f: fn(&mut AppState, u32, &[&str]) -> DispatchResult, args: &[&str]) -> DispatchResult {
    match own_profile(state) {
        Some(no) => f(state, no, args),
        None => out(render_code("No user profile.", "E-CMD-011")),
    }
}

pub static ENCODER_BAR: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "EncoderBar",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Select,
        aliases: &["encoderbars"],
        doc: Doc {
            display: "EncoderBar",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Store [UserProfile] EncoderBar \"name\" | Label|Delete|Info EncoderBar <b> | List EncoderBar | Move [UserProfile] EncoderBar <b> FeatureGroup <g> At <n> | Attribute <a> At <n> | Edit EncoderBar",
            description: "Your encoder bar layouts (decisions §30 \"12f-a\", an EncoderBarPool): a pool on \
                          your profile, each screen shows one. A layout is your groups as strip buttons with \
                          4-slot pages; a new attribute lands at the bottom of its group's button on every \
                          layout. Attributes no patched fixture has stay in their slot, hidden on the bar.",
            examples: &[
                "Store UserProfile EncoderBar \"Busking\"",
                "Move UserProfile EncoderBar \"Busking\" FeatureGroup \"Shutter\" At 2",
                "Move UserProfile EncoderBar \"Busking\" Attribute \"Zoom\" At 5",
                "Info EncoderBar \"Default\"",
                "Edit EncoderBar                                     -- Users & Profiles, Encoder bar tab",
            ],
            related: &["attribute", "featuregroup", "userprofile"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: bar_scope,
        reach: crate::objects::show_reach,
        rights: crate::rights::screen,
        planned: &[Verb::Select],
        flags: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Store, |s, a| own(s, bar_store, a)),
        (Verb::Label, |s, a| own(s, bar_label, a)),
        (Verb::Delete, |s, a| own(s, bar_delete, a)),
        (Verb::Move, |s, a| own(s, bar_move, a)),
        (Verb::List, |s, a| own(s, bar_list, a)),
        (Verb::Info, |s, a| own(s, bar_info, a)),
        (Verb::Edit, |s, a| own(s, bar_edit, a)),
    ],
    delete_ask: None,
    complete: None,
    props: None,
    tree: None,
};

// ─── Owner first: UserProfile [x] Attribute | FeatureGroup | AttributeRule | EncoderBar … ─

/// `UserProfile [<n>|"name"] <sub> …`: the profile (bare = yours) and the
/// words from the sub-object on. None when no sub-object is named.
fn profile_target<'a>(state: &AppState, words: &'a [&'a str]) -> Result<Option<(Target, &'a [&'a str])>, String> {
    let sub = |w: &str| ATTRIBUTE.matches(w) || FEATURE_GROUP.matches(w) || ATTRIBUTE_RULE.matches(w) || ENCODER_BAR.matches(w);
    let words = match words {
        [first, rest @ ..] if crate::objects::USER_PROFILE.matches(first) => rest,
        _ => return Ok(None),
    };
    match words {
        [s, ..] if sub(s) => match own_profile(state) {
            Some(no) => Ok(Some((Target::Profile(no), words))),
            None => Err(render_code("No user profile.", "E-CMD-011")),
        },
        [owner, s, ..] if sub(s) => {
            let w = unquote(owner);
            let profiles = &state.root.show_data.user_profiles;
            let no = match w.parse::<u32>() {
                Ok(n) => profiles.contains_key(&n).then_some(n),
                Err(_) => profiles.iter().find(|(_, p)| p.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(w))).map(|(n, _)| *n),
            };
            no.map(|no| Some((Target::Profile(no), &words[1..])))
                .ok_or_else(|| render_code(format!("No UserProfile {owner}."), "E-CMD-007"))
        }
        _ => Ok(None),
    }
}

/// `<verb> UserProfile [x] Attribute|FeatureGroup|AttributeRule|EncoderBar …`
/// (args after the `UserProfile` word): handled here, or None.
pub fn on_profile(state: &mut AppState, verb: Verb, args: &[&str]) -> Option<DispatchResult> {
    let mut words: Vec<&str> = vec!["UserProfile"];
    words.extend(args.iter().copied());
    let (t, rest) = match profile_target(state, &words) {
        Ok(Some(x)) => x,
        Ok(None) => return None,
        Err(e) => return Some(out(e)),
    };
    let Target::Profile(no) = t else { return None };
    let changes = !matches!(verb, Verb::List | Verb::Info | Verb::Edit);
    if changes && let Err(e) = crate::commands::media::may_change_profile(state, no) {
        return Some(out(e));
    }
    let sub = rest[0];
    let rest: Vec<&str> = rest[1..].to_vec();
    let rest = rest.as_slice();
    Some(if ATTRIBUTE.matches(sub) {
        match verb {
            Verb::Move => attribute_move(state, t, rest),
            Verb::List => attribute_list(state, t, rest),
            Verb::Info => attribute_info(state, t, rest),
            Verb::Edit => attribute_edit(state, t, rest),
            _ => out(render_code("UserProfile Attribute takes Set, Move, List, Info (and Default … /New).", "E-CMD-006")),
        }
    } else if FEATURE_GROUP.matches(sub) {
        match verb {
            Verb::Store => group_store(state, t, rest),
            Verb::Label => group_label(state, t, rest),
            Verb::Move => group_move(state, t, rest),
            Verb::Delete => group_delete(state, t, rest),
            Verb::Info => group_info(state, t, rest),
            _ => group_list(state, t, rest),
        }
    } else if ATTRIBUTE_RULE.matches(sub) {
        match verb {
            Verb::Store => rule_store(state, t, rest),
            Verb::Move => rule_move(state, t, rest),
            Verb::Delete => rule_delete(state, t, rest),
            Verb::Info => rule_info(state, t, rest),
            _ => rule_list(state, t, rest),
        }
    } else {
        match verb {
            Verb::Store => bar_store(state, no, rest),
            Verb::Label => bar_label(state, no, rest),
            Verb::Delete => bar_delete(state, no, rest),
            Verb::Move => bar_move(state, no, rest),
            Verb::Info => bar_info(state, no, rest),
            Verb::Edit => bar_edit(state, no, rest),
            _ => bar_list(state, no, rest),
        }
    })
}

// ─── Templates ───────────────────────────────────────────────────────────────

/// Regroup a layer by a template (asks; learned rules stay).
pub fn set_template(state: &mut AppState, t: Target, name: &str, args: &[&str]) -> DispatchResult {
    let std = attribute_resolver::standard();
    let Some(tpl) = std.template(unquote(name)) else {
        return out(render_code(
            format!("No attribute template \"{}\" (there are {}).", unquote(name), std.template_names().join(", ")),
            "E-CMD-007",
        ));
    };
    let tname = tpl.name.clone();
    if !no_confirmation(args) {
        let line = match t {
            Target::Show => format!("Call AttributeTemplate \"{tname}\" At ShowData /NoConfirmation"),
            Target::Profile(no) => format!("Call AttributeTemplate \"{tname}\" At UserProfile {no} /NoConfirmation"),
        };
        return DispatchResult::Ask(Box::new(
            crate::dialogs::Ask::new(
                format!("Apply template \"{tname}\"?"),
                if t == Target::Show {
                    "Every attribute is regrouped by the template's rules; learned rules stay. Cue timing per feature group follows the new groups."
                } else {
                    "Your attributes are regrouped by the template's rules; your learned rules and remembered attributes stay."
                },
            )
            .button("Apply", lumacore_wire::ButtonKind::Danger, line)
            .cancel("Cancel")
            .flags("/NoConfirmation"),
        ));
    }
    ensure_seeded(state, t);
    let r = match t {
        Target::Show => attribute_resolver::seed(&mut state.root.show_data, &tname),
        Target::Profile(_) => with_layer(state, t, |l, ctx| attribute_resolver::seed_layer(l, ctx, &tname)),
    };
    if let (Ok(()), Target::Profile(no)) = (&r, t) {
        // The layouts follow the regrouping.
        let p = &mut state.root.show_data.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes;
        let names: Vec<(String, uuid::Uuid)> = p.placed.iter().map(|(n, a)| (n.clone(), a.group)).collect();
        let groups: Vec<uuid::Uuid> = p.groups.values().map(|g| g.meta.identity.guid).collect();
        for bar in p.encoder_bars.values_mut() {
            bar.buttons.retain(|b| groups.contains(&b.group));
        }
        for (n, g) in names {
            attribute_resolver::add_to_layouts(p, &n, g);
        }
    }
    match r {
        Ok(()) => out(render_success(&format!("The attributes follow the template \"{tname}\"."))),
        Err(e) => out(render_code(e, "E-CMD-008")),
    }
}

// ─── The new-attribute dialogs ───────────────────────────────────────────────

/// Ask `user` about the attributes waiting in their profile (decisions §30
/// "12f-a": Follow standard · Edit · Later). Nothing waiting: nothing asked.
pub fn ask_waiting(state: &mut AppState, user: &str) {
    let Some(no) = state.root.show_data.profile_id_for_user(user) else { return };
    let p = &state.root.show_data.user_profiles[&no];
    if p.locked {
        return;
    }
    let waiting = p.content.attributes.waiting();
    if waiting.is_empty() {
        return;
    }
    let shown: Vec<&str> = waiting.iter().take(6).copied().collect();
    let more = if waiting.len() > 6 { format!(" and {} more", waiting.len() - 6) } else { String::new() };
    let template = if p.content.attributes.template.is_empty() { "the standard".to_string() } else { p.content.attributes.template.clone() };
    let ask = crate::dialogs::Ask::new(
        "New attributes",
        format!(
            "{} attribute{} your encoder bar has never seen ({}{more}). Standard = your rules + {template}.",
            waiting.len(),
            if waiting.len() == 1 { "" } else { "s" },
            shown.join(", ")
        ),
    )
    .button("Follow standard attribute grouping", lumacore_wire::ButtonKind::Primary, "Default UserProfile Attribute /New")
    .button("Edit attributes", lumacore_wire::ButtonKind::Plain, "Edit UserProfile Attribute /New")
    .cancel("Later");
    crate::dialogs::open_for(state, ask, crate::dialogs::Audience::User(user.to_string()), user);
}

/// After a patch was kept: every logged-in user but `except` with attributes
/// waiting is asked.
pub fn ask_everyone_waiting(state: &mut AppState, except: &str) {
    let users: Vec<String> = state.root.runtime.users.keys().filter(|u| !u.eq_ignore_ascii_case(except)).cloned().collect();
    for u in users {
        ask_waiting(state, &u);
    }
}

/// The patcher's first dialog (decisions §30 "12f-a"): the show's grouping
/// and their own encoder bar in one, Follow standard for both, or Cancel
/// (back to the patch — nothing saved, nothing sent).
pub fn patcher_prompt(new: &[String]) -> DispatchResult {
    let shown: Vec<&str> = new.iter().take(6).map(String::as_str).collect();
    let more = if new.len() > 6 { format!(" and {} more", new.len() - 6) } else { String::new() };
    DispatchResult::Ask(Box::new(
        crate::dialogs::Ask::new(
            "New attributes",
            format!(
                "The patch brings {} attribute{} the show has never seen ({}{more}).",
                new.len(),
                if new.len() == 1 { "" } else { "s" },
                shown.join(", ")
            ),
        )
        .selector("show", "Show grouping (cue timing uses it)", &[("Follow standard", "/Default"), ("Edit", "/Edit")], "Follow standard")
        .selector("mine", "Your encoder bar", &[("Follow standard", "Default"), ("Edit", "Edit")], "Follow standard")
        .button("Follow standard for both", lumacore_wire::ButtonKind::Primary, "Unclaim Root.ShowData.Devices /Keep /Default")
        .button("Continue", lumacore_wire::ButtonKind::Plain, "Unclaim Root.ShowData.Devices /Keep {show} /Mine {mine}")
        .cancel("Cancel")
        .flags("/Keep /Default or /Keep /Edit [/Mine Default|Edit]"),
    ))
}

/// Profile lifecycle (decisions §30 "12f-a"): a new profile from Default is
/// built from the show's groups; an imported or copied one keeps its own
/// layer and learns the show's attributes it has never seen (its users are
/// asked).
pub fn adopt_profile(state: &mut AppState, no: u32, fresh: bool) {
    let show = &mut state.root.show_data;
    if !show.user_profiles.contains_key(&no) {
        return;
    }
    if fresh || !show.user_profiles[&no].content.attributes.is_seeded() {
        let layer = attribute_resolver::profile_from_show(show);
        show.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap().content.attributes = layer;
        return;
    }
    let ctx = Ctx::of(show);
    let names: Vec<String> = show.attribute_defs.keys().cloned().collect();
    let p = show.user_profiles.get_mut(&no).map(std::sync::Arc::make_mut).unwrap();
    let ask = !p.locked;
    let added = attribute_resolver::push_to_profile(&mut p.content.attributes, &ctx, &names, ask);
    if !added.is_empty() {
        let show = &state.root.show_data;
        let users: Vec<String> = show
            .users
            .values()
            .filter(|u| show.profile_id_for_user(&u.login) == Some(no))
            .map(|u| u.login.clone())
            .collect();
        for u in users {
            if state.root.runtime.users.contains_key(&u) {
                ask_waiting(state, &u);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::AppState;
    use crate::commands::dispatch;

    fn run(state: &mut AppState, line: &str) -> String {
        match dispatch(state, line) {
            crate::DispatchResult::Output(s) => crate::text::strip_ansi(&s),
            crate::DispatchResult::Ask(a) => format!("ASK {}: {}", a.title(), a.view().text),
            _ => String::new(),
        }
    }

    fn demo() -> AppState {
        crate::demo::fresh_state()
    }

    fn group_of(state: &AppState, attr: &str) -> String {
        let s = &state.root.show_data;
        let g = s.attribute_defs[attr].feature_group.unwrap();
        s.feature_groups.values().find(|x| x.meta.identity.guid == g).unwrap().meta.name().unwrap_or_default().to_string()
    }

    /// Every flag the 12f-a and Users & Profiles lines carry is declared (decisions §31: an
    /// undeclared flag is refused, E-CMD-018) — the page and the dialogs send these.
    #[test]
    fn the_attribute_and_profile_lines_flags_are_declared() {
        let mut state = demo();
        state.ui_client = true;
        for line in [
            "List Attribute /New",
            "List Attribute /Unused",
            "List Attribute /Hint",
            "List UserProfile Attribute /Unused",
            "Edit Attribute /New",
            "Edit UserProfile Attribute /New /By FixtureType",
            "Default Attribute /New",
            "Default UserProfile Attribute /New",
            "List AttributeRule /Learned",
            "Assign Attribute \"Pan\" At FeatureGroup \"Position\" /Only",
            "Assign Attribute \"Pan\" At FeatureGroup \"Position\" /WithSubs",
            "Unclaim Root.ShowData.Devices /Keep /Default /Mine Edit",
            "Menu \"Users\" /Tab Profiles /Open \"1 Content\"",
        ] {
            let out = run(&mut state, line);
            assert!(!out.contains("E-CMD-018"), "{line}: {out}");
        }
    }

    #[test]
    fn set_attribute_feature_group_moves_to_the_bottom() {
        let mut s = demo();
        assert_eq!(group_of(&s, "ColorAdd_W"), "Color", "where the rig's GDTFs put it");
        let r = run(&mut s, "Assign Attribute \"ColorAdd_W\" At FeatureGroup \"FX\" /Only");
        assert!(r.contains("FX"), "{r}");
        assert_eq!(group_of(&s, "ColorAdd_W"), "FX");
        assert_eq!(group_of(&s, "ColorAdd_WW"), "Color", "/Only leaves the sub where it was");
        let d = &s.root.show_data.attribute_defs["ColorAdd_W"];
        assert_eq!(d.placement.how, lumacore_fixtures::PlaceHow::User);
        let max = s.root.show_data.attribute_defs.values().filter(|x| x.feature_group == d.feature_group).map(|x| x.position).max().unwrap();
        assert_eq!(d.position, max);
    }

    #[test]
    fn patterns_and_plus_name_several() {
        let mut s = demo();
        run(&mut s, "Assign Attribute \"ColorAdd_*\" At FeatureGroup \"Video\"");
        for n in ["ColorAdd_R", "ColorAdd_G", "ColorAdd_B", "ColorAdd_W"] {
            assert_eq!(group_of(&s, n), "Video");
        }
        let r = run(&mut s, "Assign Attribute \"Nope*\" At FeatureGroup \"FX\"");
        assert!(r.contains("E-CMD-007"), "{r}");
    }

    #[test]
    fn feature_groups_store_label_move_and_delete_only_empty() {
        let mut s = demo();
        run(&mut s, "Store FeatureGroup \"Haze\"");
        assert!(s.root.show_data.feature_groups.values().any(|g| g.meta.name() == Some("Haze")));
        run(&mut s, "Label FeatureGroup \"Haze\" \"Smoke\"");
        run(&mut s, "Move FeatureGroup \"Smoke\" At 1");
        assert_eq!(s.root.show_data.feature_groups[&1].meta.name(), Some("Smoke"));
        let r = run(&mut s, "Delete FeatureGroup \"Color\" /NoConfirmation");
        assert!(r.contains("E-CMD-010"), "{r}");
        run(&mut s, "Delete FeatureGroup \"Smoke\" /NoConfirmation");
        assert!(!s.root.show_data.feature_groups.values().any(|g| g.meta.name() == Some("Smoke")));
    }

    #[test]
    fn merge_asks_then_moves_everything() {
        let mut s = demo();
        let r = run(&mut s, "Assign FeatureGroup \"Shutter\" At FeatureGroup \"FX\"");
        assert!(r.contains("Merge"), "asks: {r}");
        assert!(s.root.show_data.feature_groups.values().any(|g| g.meta.name() == Some("Shutter")));
        run(&mut s, "Assign FeatureGroup \"Shutter\" At FeatureGroup \"FX\" /NoConfirmation");
        assert!(!s.root.show_data.feature_groups.values().any(|g| g.meta.name() == Some("Shutter")));
        assert_eq!(group_of(&s, "Shutter1"), "FX");
    }

    #[test]
    fn a_learned_rule_sits_before_the_keyword_rules_and_places_new_names() {
        let mut s = demo();
        run(&mut s, "Store AttributeRule \"Pixel*\" FeatureGroup \"Video\"");
        let show = &s.root.show_data;
        let learned = show.attribute_rules.values().find(|r| r.learned).unwrap();
        let no_of = |r: &lumacore_fixtures::AttributeRule| show.attribute_rules.iter().find(|(_, x)| x.meta.identity.guid == r.meta.identity.guid).map(|(n, _)| *n).unwrap();
        let first_kw = show.attribute_rules.values().find(|r| r.kind == lumacore_fixtures::RuleKind::Keyword).unwrap();
        assert!(no_of(learned) < no_of(first_kw));
        let r = run(&mut s, "Info AttributeRule \"PixelPattern\"");
        assert!(r.contains("Video"), "{r}");
    }

    #[test]
    fn the_attribute_call_still_sets_a_programmer_value() {
        let mut s = demo();
        run(&mut s, "Fixture 101");
        let r = run(&mut s, "Attribute \"Dimmer\" At 40");
        assert!(!r.contains("E-"), "{r}");
    }

    #[test]
    fn default_attribute_new_keeps_the_placements() {
        let mut s = demo();
        s.root.show_data.attribute_defs.values_mut().for_each(|d| d.new = true);
        run(&mut s, "Default Attribute /New");
        assert!(s.root.show_data.attribute_defs.values().all(|d| !d.new));
    }

    #[test]
    fn keeping_a_patch_with_new_attributes_asks_first() {
        let root = std::path::Path::new(env!("LUMACORE_ROOT"));
        let mut s = AppState::new();
        crate::commands::navigate::cmd_enter(&mut s, &["patch"]);
        run(&mut s, "Claim Root.ShowData.Devices");
        crate::commands::import::import_gdtf_file(&mut s, &root.join("library/gdtf/AuraXB.gdtf"));
        run(&mut s, "Universe 1");
        let name = s.root.show_data.devices.dmx.patch.fixture_types.values().next()
            .and_then(|t| t.meta.name().map(str::to_string)).unwrap();
        run(&mut s, &format!("Store Fixture 1 Thru 2 At DMX 1.001 /FixtureType \"{name}\""));
        // Nothing reaches the show's definitions while the draft is open.
        assert!(s.root.show_data.attribute_defs.is_empty());
        let r = run(&mut s, "Unclaim Root.ShowData.Devices /Keep");
        assert!(r.starts_with("ASK New attributes"), "{r}");
        run(&mut s, "Unclaim Root.ShowData.Devices /Keep /Default");
        let defs = &s.root.show_data.attribute_defs;
        assert!(!defs.is_empty());
        assert!(defs.values().all(|d| !d.new && d.feature_group.is_some()));
    }

    #[test]
    fn a_cue_part_times_a_feature_group_on_its_own() {
        let mut s = demo();
        run(&mut s, "Fixture 101 At 50");
        run(&mut s, "Store Sequence 1 Cue 1");
        let r = run(&mut s, "Set Sequence 1 Cue 1 FeatureGroup \"Dimmer\" \"Fade\" 2");
        assert!(!r.contains("E-"), "{r}");
        let dim = s.root.show_data.feature_groups.values().find(|g| g.meta.name() == Some("Dimmer")).unwrap().meta.identity.guid;
        let cue = &s.root.show_data.datapools[&1].sequences[&1].cues.values().next().unwrap();
        assert_eq!(cue.parts[&0].group_timing[&dim].fade_in, Some(2.0));
        run(&mut s, "Reset Sequence 1 Cue 1 FeatureGroup \"Dimmer\" \"Fade\"");
        let cue = &s.root.show_data.datapools[&1].sequences[&1].cues.values().next().unwrap();
        assert!(cue.parts[&0].group_timing.is_empty());
    }

    #[test]
    fn a_feature_group_path_shapes_its_fades() {
        let mut s = demo();
        run(&mut s, "Fixture 101 At 50");
        run(&mut s, "Store Sequence 1 Cue 1");
        let dim = s.root.show_data.feature_groups.values().find(|g| g.meta.name() == Some("Dimmer")).unwrap().meta.identity.guid;
        let path = |s: &AppState| s.root.show_data.datapools[&1].sequences[&1].cues[&1000].parts[&0].group_timing.get(&dim).and_then(|t| t.path);
        let r = run(&mut s, "Set Sequence 1 Cue 1 FeatureGroup \"Dimmer\" \"Path\" \"S-curve\"");
        assert!(!r.contains("E-"), "{r}");
        assert_eq!(path(&s), Some(lumacore_tree::FadePath::SCurve));
        assert!(run(&mut s, "Set Sequence 1 Cue 1 FeatureGroup \"Dimmer\" \"Path\" \"Wobble\"").contains("E-CMD-008"));
        // A Path belongs to a feature group in the cue, not to the cue.
        let r = run(&mut s, "Set Sequence 1 Cue 1 \"Path\" Snap");
        assert!(r.contains("E-SET-001"), "{r}");
        run(&mut s, "Reset Sequence 1 Cue 1 FeatureGroup \"Dimmer\" \"Path\"");
        assert_eq!(path(&s), None);
        assert!(s.root.show_data.datapools[&1].sequences[&1].cues[&1000].parts[&0].group_timing.is_empty());
    }

    fn profile_group_of(state: &AppState, attr: &str) -> String {
        let no = state.root.show_data.profile_id_for_user("Admin").unwrap();
        let p = &state.root.show_data.user_profiles[&no].content.attributes;
        let g = p.placed[attr].group;
        p.groups.values().find(|x| x.meta.identity.guid == g).unwrap().meta.name().unwrap_or_default().to_string()
    }

    #[test]
    fn your_profile_groups_on_its_own_and_the_show_stays() {
        let mut s = demo();
        let r = run(&mut s, "Assign UserProfile Attribute \"Shutter1\" At UserProfile FeatureGroup \"FX\"");
        assert!(!r.contains("E-"), "{r}");
        assert_eq!(profile_group_of(&s, "Shutter1"), "FX");
        assert_eq!(group_of(&s, "Shutter1"), "Shutter", "the show's grouping is untouched");
        run(&mut s, "Store UserProfile FeatureGroup \"Busk\"");
        let no = s.root.show_data.profile_id_for_user("Admin").unwrap();
        assert!(s.root.show_data.user_profiles[&no].content.attributes.groups.values().any(|g| g.meta.name() == Some("Busk")));
        assert!(!s.root.show_data.feature_groups.values().any(|g| g.meta.name() == Some("Busk")));
        // The moved attribute sits at the bottom of FX's strip button on the layout.
        let p = &s.root.show_data.user_profiles[&no].content.attributes;
        let fx = p.groups.values().find(|g| g.meta.name() == Some("FX")).unwrap().meta.identity.guid;
        let btn = p.encoder_bars[&1].buttons.iter().find(|b| b.group == fx).unwrap();
        assert_eq!(btn.slots.last().unwrap().as_deref(), Some("Shutter1"));
    }

    #[test]
    fn encoder_bar_layouts_store_move_and_list() {
        let mut s = demo();
        let r = run(&mut s, "Store UserProfile EncoderBar \"Busking\"");
        assert!(r.contains("Busking"), "{r}");
        let r = run(&mut s, "Move UserProfile EncoderBar \"Busking\" FeatureGroup \"Shutter\" At 1");
        assert!(!r.contains("E-"), "{r}");
        let r = run(&mut s, "List EncoderBar");
        assert!(r.contains("Default") && r.contains("Busking"), "{r}");
        let r = run(&mut s, "Info EncoderBar \"Busking\"");
        assert!(r.lines().nth(1).unwrap().contains("Shutter"), "{r}");
    }

    #[test]
    fn the_patcher_can_answer_their_own_bar_apart() {
        let root = std::path::Path::new(env!("LUMACORE_ROOT"));
        let mut s = AppState::new();
        crate::commands::navigate::cmd_enter(&mut s, &["patch"]);
        run(&mut s, "Claim Root.ShowData.Devices");
        crate::commands::import::import_gdtf_file(&mut s, &root.join("library/gdtf/AuraXB.gdtf"));
        run(&mut s, "Universe 1");
        let name = s.root.show_data.devices.dmx.patch.fixture_types.values().next()
            .and_then(|t| t.meta.name().map(str::to_string)).unwrap();
        run(&mut s, &format!("Store Fixture 1 At DMX 1.001 /FixtureType \"{name}\""));
        run(&mut s, "Unclaim Root.ShowData.Devices /Keep /Default /Mine Edit");
        let no = s.root.show_data.profile_id_for_user("Admin").unwrap();
        assert!(s.root.show_data.attribute_defs.values().all(|d| !d.new), "the show followed the standard");
        assert!(!s.root.show_data.user_profiles[&no].content.attributes.waiting().is_empty(), "the bar waits for the edit");
        let r = run(&mut s, "List UserProfile Attribute /New");
        assert!(!r.contains("No attributes"), "{r}");
        run(&mut s, "Default UserProfile Attribute /New");
        assert!(s.root.show_data.user_profiles[&no].content.attributes.waiting().is_empty());
    }

    /// A patch change bringing a new attribute writes it into the locked
    /// Default too — that is the sync's write, never refused (E-USER-002).
    #[test]
    fn a_patch_change_teaches_the_locked_default_without_a_refusal() {
        let mut s = crate::commands::coverage_tests::with_fixture();
        let show = &mut s.root.show_data;
        show.attribute_defs.remove("Dimmer");
        for p in show.user_profiles.values_mut().map(std::sync::Arc::make_mut) {
            p.content.attributes.placed.remove("Dimmer");
        }
        let r = run(&mut s, "Set Fixture 1 \"Name\" \"Spot\"");
        assert!(!r.contains("E-USER-002"), "{r}");
        let show = &s.root.show_data;
        assert!(show.attribute_defs.contains_key("Dimmer"));
        let (_, default) = show.default_profile().unwrap();
        assert!(default.content.attributes.placed.contains_key("Dimmer"));
        assert!(run(&mut s, "Set UserProfile 1 \"PopupDim\" Off").contains("E-USER-002"), "a user's own edit still is");
    }

    /// `Edit [UserProfile] Attribute` opens the board on this client (12f-a).
    #[test]
    fn edit_attribute_opens_the_board_of_the_show_or_yours() {
        let mut s = demo();
        s.ui_requests.clear();
        let r = run(&mut s, "Edit Attribute /New /By FixtureType");
        assert!(!r.contains("E-"), "{r}");
        let mine = s.root.show_data.profile_id_for_user("Admin").unwrap();
        run(&mut s, "Edit UserProfile Attribute");
        let boards: Vec<_> = s
            .ui_requests
            .iter()
            .filter_map(|e| match e {
                lumacore_wire::UiEvent::AttributeBoard { profile, owner, filter, view } => {
                    Some((*profile, owner.clone(), filter.clone(), view.clone()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            boards,
            [(0, String::new(), "new".into(), "type".into()), (mine, "UserProfile".into(), "all".into(), "group".into())]
        );
    }

    /// The "+" column and the Encoder bar tab's New ask the name (12f-a).
    #[test]
    fn bare_store_asks_the_name_and_the_tab_sees_your_layouts() {
        let mut s = demo();
        assert!(run(&mut s, "Store FeatureGroup").starts_with("ASK New feature group"));
        assert!(run(&mut s, "Store UserProfile EncoderBar").starts_with("ASK New encoder bar layout"));
        run(&mut s, "Store UserProfile EncoderBar \"Busking\"");
        let no = s.root.show_data.profile_id_for_user("Admin").unwrap();
        let bars = &s.root.show_data.user_profiles[&no].content.attributes.encoder_bars;
        assert_eq!(bars.len(), 2);
        assert_eq!(bars.values().nth(1).map(|b| b.meta.name().unwrap_or_default()), Some("Busking"));
    }
}
