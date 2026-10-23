// ██████╗  █████╗ ██╗███╗   ██╗████████╗    ███████╗██╗      ██████╗ ████████╗███████╗
// ██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝    ██╔════╝██║     ██╔═══██╗╚══██╔══╝██╔════╝
// ██████╔╝███████║██║██╔██╗ ██║   ██║       ███████╗██║     ██║   ██║   ██║   ███████╗
// ██╔═══╝ ██╔══██║██║██║╚██╗██║   ██║       ╚════██║██║     ██║   ██║   ██║   ╚════██║
// ██║     ██║  ██║██║██║ ╚████║   ██║       ███████║███████╗╚██████╔╝   ██║   ███████║
// ╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝       ╚══════╝╚══════╝ ╚═════╝    ╚═╝   ╚══════╝
// FILE: paint_slots

//! A theme's paint slots (decisions 0294): the colours an object painted `Theme "<slot>"` shows,
//! one per slot, in each viewer's own theme. LumaCore's slots (`media::PAINT_SLOTS`: Dimmer …
//! Special 4) are declared properties of the theme's `Paint`, beside its 14 screen colours; a slot
//! the theme adds is the same property under its own name (`Props::open`). `Set [UserProfile x]
//! Theme Paint "Position" "#3a62b8"` changes one, `Store … Theme Paint "Haze" ["#rrggbb"]` adds
//! one, `Reset … Theme Paint "Position"` puts LumaCore's back (an added slot goes).
//!
//! Owns:          SLOT_PROPS, OPEN_SLOT, StorePaintSlot, the slots' get / set.
//! Doesn't own:   the theme and its slots' data (lumacore_tree::media), the Paint object (media.rs),
//!                how a viewer resolves a link (lumacore_state::paint).
//! Entry points:  SLOT_PROPS, OPEN_SLOT, StorePaintSlot.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links, 0145, 0164.

use lumacore_base::paint::{check_slot, hex_of, hex_rgb};
use lumacore_syntax::ast::{Item, Target, Value};
use lumacore_tree::media::{PAINT_SLOTS, PaintSlot, is_builtin_slot};

use crate::errors::render_code;
use crate::objects::out;
use crate::props::{Access, PropKind, Property};
use crate::{AppState, DispatchResult};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

macro_rules! slot {
    ($i:expr) => {
        Property {
            name: PAINT_SLOTS[$i].0,
            kind: PropKind::Colour,
            rights: crate::commands::registry::Rights::Guest,
            section: "Paint slots",
            doc: "a theme paint slot: what an object painted Theme \"<slot>\" shows you (#rrggbb; none = LumaCore's)",
            access: Access::Named { get: slot_get, set: slot_set },
            ..Property::BASE
        }
    };
}

/// LumaCore's paint slots as the theme's `Paint` properties.
pub static SLOT_PROPS: [Property; 15] = [
    slot!(0),
    slot!(1),
    slot!(2),
    slot!(3),
    slot!(4),
    slot!(5),
    slot!(6),
    slot!(7),
    slot!(8),
    slot!(9),
    slot!(10),
    slot!(11),
    slot!(12),
    slot!(13),
    slot!(14),
];

/// A slot the theme added, under its own name.
pub static OPEN_SLOT: Property = Property {
    name: "Slot",
    kind: PropKind::Colour,
    rights: crate::commands::registry::Rights::Guest,
    section: "Paint slots",
    doc: "a paint slot the theme added (Store Theme Paint \"<name>\" adds one; Reset removes it)",
    access: Access::Named { get: slot_get, set: slot_set },
    ..Property::BASE
};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Store [UserProfile x] Theme Paint "Haze" ["#rrggbb"]`: a paint slot of the theme's own (or a
/// new colour for one it has).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store")]
pub struct StorePaintSlot {
    #[lum(target)]
    pub target: Target,
    #[lum(property)]
    pub slot: String,
    #[lum(value)]
    pub colour: Option<Value>,
}

impl crate::commands::typed::Command for StorePaintSlot {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(store(state, &self.target, &self.slot, self.colour.as_ref()).unwrap_or_else(|e| e))
    }
}

/// The Store of the object that starts the line (`Theme`, `Paint`; the words after it): the
/// line again from that word, bound to [`StorePaintSlot`].
pub fn store_from(word: &'static str) -> impl Fn(&mut AppState, &[&str]) -> DispatchResult {
    move |state, args| {
        let words: Vec<&str> = std::iter::once(word).chain(args.iter().copied()).collect();
        crate::commands::typed::typed::<StorePaintSlot>(state, &words)
    }
}

/// `Store UserProfile [x] Theme Paint "<slot>" …` (args after `UserProfile`): a profile's slot;
/// None when the line names no Theme.
pub fn on_profile(state: &mut AppState, verb: crate::objects::Verb, args: &[&str]) -> Option<DispatchResult> {
    let theme = args.iter().any(|w| crate::objects::find(w).is_some_and(|o| std::ptr::eq(o, &crate::commands::media::THEME)));
    if verb != crate::objects::Verb::Store || !theme {
        return None;
    }
    Some(store_from("UserProfile")(state, args))
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn store(state: &mut AppState, target: &Target, slot: &str, colour: Option<&Value>) -> Result<String, String> {
    let slot = crate::tokens::unquote(slot).trim().to_string();
    check_slot(&slot).map_err(|e| render_code(e, "E-CMD-008"))?;
    let no = owner(state, target)?;
    crate::commands::media::may_change_profile(state, no).map_err(|e| crate::errors::reply(e, "E-SET-002"))?;
    let theme = &state.root.show_data.user_profiles.get(&no).ok_or_else(|| render_code("No such UserProfile.", "E-CMD-007"))?.settings.theme;
    let had = theme.slot(&slot).map(str::to_string);
    let hex = match colour {
        Some(v) => {
            let text = match v {
                Value::Text(t) => t.node.clone(),
                Value::Number(n) => n.node.clone(),
                Value::Word(w) => crate::lum::vocabulary().word(w.node).name.clone(),
                _ => String::new(),
            };
            hex_rgb(&text).map(hex_of).ok_or_else(|| render_code(format!("'{text}' is not a colour #rrggbb."), "E-CMD-008"))?
        }
        None if had.is_some() => {
            return Err(render_code(format!("The theme has Paint \"{slot}\" already: Set Theme Paint \"{slot}\" \"#rrggbb\" changes it."), "E-CMD-010"));
        }
        // A new slot without a colour takes the next palette hue.
        None => {
            let n = theme.slots.iter().filter(|s| !is_builtin_slot(&s.name)).count();
            lumacore_base::paint::PALETTE[n % lumacore_base::paint::PALETTE.len()].1[lumacore_base::paint::NAMED_TONE].to_string()
        }
    };
    put(state, no, &slot, Some(&hex));
    let what = if had.is_some() { "Stored" } else { "Added" };
    Ok(crate::text::render_success(&format!("{what} UserProfile {no} Theme Paint \"{slot}\" {hex}")))
}

/// The profile a Store line names (`UserProfile x` in front), else your own.
fn owner(state: &AppState, target: &Target) -> Result<u32, String> {
    let first = match target.items.as_slice() {
        [(None, Item::Path(segments))] => segments.first().cloned(),
        _ => None,
    };
    if let Some(seg) = first.filter(|s| s.ids.is_some()) {
        let words = crate::lum::vocabulary();
        if crate::objects::USER_PROFILE.matches(&words.word(seg.object.node).name) {
            let one = Target { items: vec![(None, Item::Path(vec![seg]))] };
            let r = crate::resolve::resolve_one(state, &one, &crate::objects::USER_PROFILE)?;
            let [m] = r.members.as_slice() else {
                return Err(render_code("Name one UserProfile.", "E-CMD-008"));
            };
            return Ok(m.no);
        }
    }
    state.root.show_data.profile_id_for_user(state.effective_user_name()).ok_or_else(|| render_code("You have no UserProfile.", "E-CMD-011"))
}

/// A slot's colour in the theme of profile `key`.
fn slot_get(state: &AppState, key: &str, name: &str) -> String {
    key.parse::<u32>()
        .ok()
        .and_then(|no| state.root.show_data.user_profiles.get(&no))
        .and_then(|p| p.settings.theme.slot(name).map(str::to_string))
        .unwrap_or_default()
}

/// `Set … Theme Paint "<slot>" "#rrggbb"` (none / Reset: LumaCore's colour back; an added slot
/// goes).
fn slot_set(state: &mut AppState, key: &str, name: &str, value: &str) -> Result<String, String> {
    let no: u32 = key.parse().map_err(|_| render_code("Name a UserProfile.", "E-CMD-007"))?;
    let theme = &state.root.show_data.user_profiles.get(&no).ok_or_else(|| render_code("No such UserProfile.", "E-CMD-007"))?.settings.theme;
    let builtin = is_builtin_slot(name);
    if !builtin && theme.slot(name).is_none() {
        return Err(render_code(
            format!("The theme has no paint slot \"{name}\": Store Theme Paint \"{name}\" \"#rrggbb\" adds it."),
            "E-CMD-007",
        ));
    }
    let text = value.trim().trim_matches('"');
    if text.is_empty() || text.eq_ignore_ascii_case("none") {
        put(state, no, name, None);
        return Ok(if builtin { format!("UserProfile {no} Paint \"{name}\" = LumaCore's") } else { format!("UserProfile {no} Paint \"{name}\" removed") });
    }
    let hex = hex_rgb(text).map(hex_of).ok_or_else(|| render_code(format!("UserProfile {no} \"{name}\": '{text}' is not a colour #rrggbb."), "E-SET-003"))?;
    put(state, no, name, Some(&hex));
    Ok(format!("UserProfile {no} Paint \"{name}\" = {hex}"))
}

/// Write a slot of profile `no`: a colour, or None = LumaCore's again (an added slot goes). A
/// LumaCore slot set back to its own colour keeps no entry.
fn put(state: &mut AppState, no: u32, name: &str, hex: Option<&str>) {
    let Some(profile) = state.root.show_data.profile_mut(no) else { return };
    let slots = &mut profile.settings.theme.slots;
    let default = PAINT_SLOTS.iter().find(|(n, _)| n.eq_ignore_ascii_case(name));
    let at = slots.iter().position(|s| s.name.eq_ignore_ascii_case(name));
    match (hex, at) {
        (None, Some(i)) => {
            slots.remove(i);
        }
        (Some(h), _) if default.is_some_and(|(_, d)| d.eq_ignore_ascii_case(h)) => {
            if let Some(i) = at {
                slots.remove(i);
            }
        }
        (Some(h), Some(i)) => slots[i].colour = h.to_string(),
        (Some(h), None) => {
            let name = default.map_or(name, |(n, _)| n).to_string();
            slots.push(PaintSlot { name, colour: h.to_string() });
        }
        (None, None) => {}
    }
    state.show_dirty = true;
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
