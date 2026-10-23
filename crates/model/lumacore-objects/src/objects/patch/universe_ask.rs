// ██╗   ██╗███╗   ██╗██╗██╗   ██╗███████╗██████╗ ███████╗███████╗     █████╗ ███████╗██╗  ██╗
// ██║   ██║████╗  ██║██║██║   ██║██╔════╝██╔══██╗██╔════╝██╔════╝    ██╔══██╗██╔════╝██║ ██╔╝
// ██║   ██║██╔██╗ ██║██║██║   ██║█████╗  ██████╔╝███████╗█████╗      ███████║███████╗█████╔╝
// ██║   ██║██║╚██╗██║██║╚██╗ ██╔╝██╔══╝  ██╔══██╗╚════██║██╔══╝      ██╔══██║╚════██║██╔═██╗
// ╚██████╔╝██║ ╚████║██║ ╚████╔╝ ███████╗██║  ██║███████║███████╗    ██║  ██║███████║██║  ██╗
//  ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝    ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
// FILE: universe_ask

//! A universe in the Devices draft (decisions §11 "Issue 11 hardening" 4, 5): the wiring — the
//! protocol lines — names slots by number and stays where it is, so Delete names the fixtures
//! patched on a universe (they lose their address there, in the draft) and the lines left on its
//! slot, and Move asks when input lines stay behind on the old slot.
//!
//! Owns:          delete_ask (Universe's Delete question), MoveUniverse, fixtures_on, lines_on.
//! Doesn't own:   the door's Move (`commands::door`), the delete itself (`commands::universe`).
//! Entry points:  delete_ask, MoveUniverse.
//! Decisions:     §11 "Issue 11 hardening" 4, 5; §34 "34g4 patch hardened" 8.

use lumacore_wire::ButtonKind;

use crate::commands::door::numbers_text;
use crate::objects::DeleteAsk;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Move Universe <n> At <m>`: the door's Move, asked first when input lines stay on a slot it
/// leaves (MergeMode goes with the universe, the inputs stay on the slot).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Move", object = "Universe")]
pub struct MoveUniverse {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "destination")]
    pub to: Option<lumacore_syntax::ast::Value>,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl crate::commands::typed::Command for MoveUniverse {
    fn run(self, state: &mut AppState) -> DispatchResult {
        // Asked once the move has a destination (without one the door asks where first).
        if !self.confirmed
            && let Some(to) = self.to.as_ref()
            && let Ok(slots) = super::numbers(state, &self.target, &crate::objects::UNIVERSE)
        {
            let inputs: Vec<String> = slots.iter().flat_map(|&n| lines_on(state, n, true)).collect();
            if !inputs.is_empty() {
                let at = match crate::commands::door::destination(state, &crate::objects::UNIVERSE, &super::UNIVERSE_DOOR, None, to) {
                    Ok((_, Some(start))) => format!(" At {start}"),
                    Ok((_, None)) => String::new(),
                    Err(e) => return crate::objects::out(e),
                };
                return DispatchResult::Ask(Box::new(
                    crate::dialogs::Ask::new(
                        "Move Universe",
                        format!(
                            "Input lines stay on the slot{} it leaves: {}.\nMergeMode goes with the universe; the inputs stay on the slot.",
                            if slots.len() == 1 { "" } else { "s" },
                            inputs.join(", ")
                        ),
                    )
                    .button("Move anyway", ButtonKind::Primary, format!("Move Universe {}{at} /NoConfirmation", numbers_text(&slots)))
                    .cancel("Cancel")
                    .flags("/NoConfirmation"),
                ));
            }
        }
        crate::commands::door::relocate(state, &crate::objects::UNIVERSE, &self.target, self.to.as_ref(), Default::default())
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Universe's one Delete question (`ObjectDef::delete_ask`): the universes, the fixtures patched
/// on them (they lose their address there, in the draft — the live ones named too while a draft
/// is open) and the lines left on their slots.
pub fn delete_ask(state: &AppState, args: &[&str]) -> DeleteAsk {
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    let target = match crate::resolve::target_of(&format!("Universe {}", words.join(" "))) {
        Ok(t) => t,
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let slots = match super::numbers(state, &target, &crate::objects::UNIVERSE) {
        Ok(s) if !s.is_empty() => s,
        Ok(_) => return DeleteAsk::Refuse(crate::errors::render_code("No such Universe.", "E-CMD-007")),
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let devices = &state.root.show_data.devices;
    let mut text = match slots.as_slice() {
        [one] => {
            let name = devices.dmx.patch.universes.get(one).and_then(|u| u.meta.identity.name.clone()).map(|n| format!(" \"{n}\"")).unwrap_or_default();
            format!("Delete Universe {one}{name}?")
        }
        _ => format!("Delete {} Universes ({})?", slots.len(), numbers_text(&slots)),
    };
    let here: Vec<u32> = slots.iter().flat_map(|&n| fixtures_on(devices, n)).collect();
    if !here.is_empty() {
        let draft = if state.patch_draft.is_some() { " in the draft" } else { "" };
        text.push_str(&format!("\nPatched on it: {} — they lose their address there{draft}.", fixtures_text(&here)));
    }
    if let Some(d) = &state.patch_draft {
        let live: Vec<u32> = slots.iter().flat_map(|&n| fixtures_on(&d.live, n)).collect();
        if !live.is_empty() && live != here {
            text.push_str(&format!("\nLive: {}.", fixtures_text(&live)));
        }
    }
    let lines: Vec<String> = slots.iter().flat_map(|&n| lines_on(state, n, false)).collect();
    if !lines.is_empty() {
        text.push_str(&format!("\nThe lines stay on the slot (nothing sent or taken until a universe is there): {}.", lines.join(", ")));
    }
    DeleteAsk::Ask(Box::new(
        crate::dialogs::Ask::new("Delete", text)
            .button("Delete", ButtonKind::Danger, format!("Delete Universe {} /NoConfirmation", numbers_text(&slots)))
            .cancel("Cancel")
            .flags("/NoConfirmation"),
    ))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The fixtures with a break on universe slot `slot` of these devices.
pub fn fixtures_on(devices: &lumacore_tree::DevicesData, slot: u32) -> Vec<u32> {
    let Some(guid) = devices.dmx.patch.universes.get(&slot).map(|u| u.meta.identity.guid) else { return Vec::new() };
    devices.dmx.patch.fixtures.values().filter(|f| f.mvr.breaks.iter().any(|b| b.universe_id == guid)).filter_map(|f| f.mvr.fid).collect()
}

/// The protocol lines whose run covers slot `slot` (`Sacn 2`); only inputs when `inputs`.
pub fn lines_on(state: &AppState, slot: u32, inputs: bool) -> Vec<String> {
    state
        .root
        .show_data
        .protocols
        .all_lines()
        .filter(|(_, _, l)| l.amount > 0 && (l.local_universe..l.local_universe + l.amount).contains(&slot))
        .filter(|(_, _, l)| !inputs || l.mode == lumacore_protocols::Direction::In)
        .map(|(p, n, _)| format!("{p} {n}"))
        .collect()
}

fn fixtures_text(nos: &[u32]) -> String {
    let shown: Vec<String> = nos.iter().take(8).map(u32::to_string).collect();
    let more = nos.len().saturating_sub(8);
    format!("Fixture {}{}", shown.join(", "), if more > 0 { format!(" and {more} more") } else { String::new() })
}
