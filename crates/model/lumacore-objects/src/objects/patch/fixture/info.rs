// ██╗███╗   ██╗███████╗ ██████╗
// ██║████╗  ██║██╔════╝██╔═══██╗
// ██║██╔██╗ ██║█████╗  ██║   ██║
// ██║██║╚██╗██║██╔══╝  ██║   ██║
// ██║██║ ╚████║██║     ╚██████╔╝
// ╚═╝╚═╝  ╚═══╝╚═╝      ╚═════╝
// FILE: info

//! `List Fixture …` and `Info Fixture …` (decisions §40 "40d built"): a number lists or describes
//! the fixture; a folder — `Fixture Media 1` — lists what is under it at any depth, or its own
//! sheet (where it is, what it holds); `Info Fixture 201 Multipatch` and `Info Fixture 3 DMX`
//! stay the round-3 forms.
//!
//! Owns:          the Fixture object's List and Info handlers, the folder's sheet.
//! Doesn't own:   the listing itself (`commands::list`), the fixture's sheet (`commands::info`),
//!                the nodes (`devices_meta::nodes`).
//! Entry points:  fixture_list, fixture_info.
//! Decisions:     §34 round 3 19, §40 "40d hardened" 8.

use crate::AppState;
use crate::DispatchResult;
use crate::commands::{info, list};
use crate::objects::out;
use crate::objects::patch::devices_meta::nodes;
use crate::objects::patch::dmx;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

pub(in crate::objects::patch) fn fixture_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    // `List Fixture Media 1`: what is under the node, at any depth (decisions §40 "40d hardened").
    if !args.is_empty() && !args[0].chars().all(|c| c.is_ascii_digit()) {
        let words: Vec<&str> = std::iter::once("Fixture").chain(args.iter().copied()).collect();
        return out(crate::resolve::fixture_guid_words(state, &words).map_or_else(|e| e, |guids| {
            let devices = &state.root.show_data.devices;
            let mut nos: Vec<u32> = Vec::new();
            for g in guids {
                match devices.dmx.patch.fixture_by_guid(g).and_then(|(_, f)| f.mvr.fid) {
                    Some(n) => nos.push(n),
                    None => nos.extend(nodes::fixtures_under(devices, g)),
                }
            }
            nos.sort_unstable();
            nos.dedup();
            list::fixtures_of(state, &nos)
        }));
    }
    out(list::fixtures(state, args))
}

/// `Info Fixture Media 1`: a folder — where it is, what is under it.
fn folder_info(state: &AppState, g: uuid::Uuid) -> String {
    let devices = &state.root.show_data.devices;
    let Some((_, f)) = devices.dmx.patch.fixture_by_guid(g) else { return crate::errors::render_code("No such Fixture.", "E-CMD-007") };
    let ft = devices.dmx.patch.fixture_types.values().find(|t| t.meta.identity.guid == f.mvr.fixture_type);
    let under = f.mvr.parent.map(|p| nodes::node_label(devices, p)).unwrap_or_else(|| "No MVR node".to_string());
    let folders = nodes::folders_under(devices, g);
    let fixtures = nodes::fixtures_under(devices, g).len();
    crate::text::PropertySheet::new(&f.handle())
        .field("Name", f.meta.name().unwrap_or("<unnamed>"))
        .field("GUID", &f.meta.identity.guid.to_string())
        .field("Type", &ft.and_then(|t| t.meta.identity.name.clone()).unwrap_or_default())
        .field("In", &under)
        .field("Holds", &format!("{folders} folders, {fixtures} fixtures"))
        .build()
}
pub(in crate::objects::patch) fn fixture_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    // `Info Fixture 201 Multipatch`: its copies (round 3 19).
    if let Some(i) = args.iter().position(|a| a.eq_ignore_ascii_case(super::multipatch::MULTIPATCH.keyword)) {
        return out(crate::resolve::fixture_words(state, &args[..i]).map_or_else(|e| e, |nos| super::multipatch::info(state, &nos)));
    }
    // `Info Fixture Media 1`: a folder's own sheet (decisions §40 "40d hardened").
    if args.first().is_some_and(|a| !a.chars().all(|c| c.is_ascii_digit()))
        && let Ok(guids) = crate::resolve::fixture_guid_words(state, &std::iter::once("Fixture").chain(args.iter().copied()).collect::<Vec<_>>())
        && let [g] = guids.as_slice()
        && state.root.show_data.devices.dmx.patch.fixture_by_guid(*g).is_some_and(|(_, f)| f.mvr.fid.is_none())
    {
        return out(folder_info(state, *g));
    }
    // `Info Fixture 3 DMX [n]`: its addresses (round 3 19).
    if let Some(i) = args.iter().position(|a| a.eq_ignore_ascii_case(dmx::DMX.keyword)) {
        let brk = args.get(i + 1).and_then(|b| b.parse().ok());
        return out(crate::resolve::fixture_words(state, &args[..i]).map_or_else(|e| e, |nos| dmx::addresses(state, &nos, brk)));
    }
    out(info::info_fixture(state, args.first().copied()))
}
