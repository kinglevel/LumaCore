// ██╗███╗   ██╗███████╗███████╗██████╗ ████████╗
// ██║████╗  ██║██╔════╝██╔════╝██╔══██╗╚══██╔══╝
// ██║██╔██╗ ██║███████╗█████╗  ██████╔╝   ██║
// ██║██║╚██╗██║╚════██║██╔══╝  ██╔══██╗   ██║
// ██║██║ ╚████║███████║███████╗██║  ██║   ██║
// ╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝  ╚═╝   ╚═╝
// FILE: insert

//! The Insert fixtures dialog's data (decisions §34 round 3, `insert-fixtures.html` v6): the
//! show's fixture types by number — each revision, the newest revision's modes with their
//! footprints, how many fixtures use it — and the library's files beside them (the same file as
//! a show revision says so); then one mode's channel map (break, offset, attribute with its pretty
//! name and feature group, geometry, virtual) and the file's details. `GET /api/fixture-types`,
//! `GET /api/fixture-type`.
//!
//! Owns:          list, mode.
//! Doesn't own:   reading the library (`commands::gdtf_library`), the import (Import FixtureType).
//! Entry points:  list, mode.
//! Decisions:     §34 round 3 2–4.

use lumacore_objects::commands::gdtf_library::LibraryFile;
use lumacore_tree::DevicesData;
use lumacore_tree::fixtures::fixture_type::{FixtureType, Revision};
use serde_json::{Value, json};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Every fixture type the dialog can choose: `{show: [...], library: [...]}`.
pub fn list(devices: &DevicesData, library: &[LibraryFile]) -> Value {
    let patch = &devices.dmx.patch;
    let show: Vec<Value> = patch
        .fixture_types
        .iter()
        .map(|(no, t)| {
            let used = patch.fixtures.values().filter(|f| f.mvr.fixture_type == t.meta.identity.guid).count();
            json!({
                "source": "show", "no": no, "guid": t.meta.identity.guid,
                "name": t.meta.identity.name.clone().unwrap_or_default(), "manufacturer": t.manufacturer,
                "revisions": t.revisions.iter().enumerate().map(|(i, r)| json!({ "no": i + 1, "name": r.meta.identity.name.clone().unwrap_or_default() })).collect::<Vec<_>>(),
                "modes": t.revisions.last().map(modes).unwrap_or_default(),
                "used": used,
            })
        })
        .collect();
    let library: Vec<Value> = library
        .iter()
        .map(|f| {
            // The same file as a revision in the show: it is already there.
            let in_show = patch
                .fixture_types
                .iter()
                .find(|(_, t)| t.revisions.iter().any(|r| r.file_hash == f.sha256))
                .map(|(no, _)| *no);
            match &f.fixture_type {
                Ok(t) => json!({
                    "source": "library", "file": f.file, "size": f.size, "in_show": in_show,
                    "name": t.meta.identity.name.clone().unwrap_or_default(), "manufacturer": t.manufacturer,
                    "revision": f.details["revisions"].as_array().and_then(|r| r.last()).map(|r| r["text"].clone()).unwrap_or_default(),
                    "modes": t.revisions.last().map(modes).unwrap_or_default(),
                }),
                Err(e) => json!({ "source": "library", "file": f.file, "size": f.size, "error": e }),
            }
        })
        .collect();
    json!({ "show": show, "library": library })
}

/// One mode's channel map and the file's details: `revision` / `mode` count from 1 (0 = the
/// newest revision).
pub fn mode(ft: &FixtureType, revision: usize, mode: usize, details: &Value) -> Option<Value> {
    let rev = if revision == 0 { ft.revisions.last()? } else { ft.revisions.get(revision - 1)? };
    let m = rev.modes.get(mode.checked_sub(1)?)?;
    let channels: Vec<Value> = m
        .channel_functions
        .iter()
        .map(|c| {
            let file = rev.attributes.iter().find(|a| a.name.eq_ignore_ascii_case(&c.attribute_name));
            let feature = file.and_then(|a| a.feature.clone()).unwrap_or_default();
            json!({
                "break": c.dmx_break, "offset": c.dmx_offset, "bytes": c.byte_count, "virtual": c.is_virtual,
                "attribute": c.attribute_name,
                "pretty": file.map(|a| a.pretty.clone()).filter(|p| !p.is_empty()).unwrap_or_else(|| c.attribute_name.clone()),
                "feature_group": feature.split('.').next().unwrap_or_default(),
                "feature": feature,
                "geometry": c.geometry,
            })
        })
        .collect();
    Some(json!({
        "name": ft.meta.identity.name.clone().unwrap_or_default(), "manufacturer": ft.manufacturer,
        "revision": rev.meta.identity.name.clone().unwrap_or_default(), "gdtf_version": rev.gdtf_version,
        "mode": m.meta.identity.name.clone().unwrap_or_default(), "footprint": m.dmx_footprint,
        "channels": channels, "details": details,
    }))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// A revision's modes as the dialog's chips: number, name, footprint.
fn modes(rev: &Revision) -> Vec<Value> {
    rev.modes
        .iter()
        .enumerate()
        .map(|(i, m)| json!({ "no": i + 1, "name": m.meta.identity.name.clone().unwrap_or_default(), "footprint": m.dmx_footprint }))
        .collect()
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
