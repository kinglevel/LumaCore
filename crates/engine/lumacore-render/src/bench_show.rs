// ██████╗ ███████╗███╗   ██╗ ██████╗██╗  ██╗    ███████╗██╗  ██╗ ██████╗ ██╗    ██╗
// ██╔══██╗██╔════╝████╗  ██║██╔════╝██║  ██║    ██╔════╝██║  ██║██╔═══██╗██║    ██║
// ██████╔╝█████╗  ██╔██╗ ██║██║     ███████║    ███████╗███████║██║   ██║██║ █╗ ██║
// ██╔══██╗██╔══╝  ██║╚██╗██║██║     ██╔══██║    ╚════██║██╔══██║██║   ██║██║███╗██║
// ██████╔╝███████╗██║ ╚████║╚██████╗██║  ██║    ███████║██║  ██║╚██████╔╝╚███╔███╔╝
// ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝╚═╝  ╚═╝    ╚══════╝╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝
// FILE: bench show

//! A generated show the size of the large rig probed for decisions §24
//! (35,451 parameters): the render budget test and the render benchmark
//! use it. Not part of the console.
//!
//! Owns:          show and input: a generated 35k-parameter show for the budget test and the benchmark.
//! Doesn't own:   the console's shows (the demo, lumacore-objects).
//! Entry points:  show, input.
//! Decisions:     §24, §31 "Render budget".

#![doc(hidden)]

use std::sync::Arc;

use lumacore_fixtures::fixture::Fixture;
use lumacore_fixtures::fixture_type::{AttributeType, ChannelFunction, FixtureType, Mode, Revision};
use lumacore_tree::node::NodeMeta;
use lumacore_tree::tree::ShowData;
use lumacore_fixtures::universe::Universe;
use lumacore_tree::{
    CueObject, CueValue, GroupMasterMode, MasterLevel, SequenceObject, SequenceState, Value,
};

use crate::merge::{EngineInput, UserLayer};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Channel functions per fixture (the probed rig: ~26 parameters per fixture).
const FUNCTIONS: u16 = 26;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn function(i: u16) -> ChannelFunction {
    let (name, ty, bytes) = match i {
        0 => ("Dimmer".to_string(), AttributeType::Dimmer, 1),
        1 => ("Pan".to_string(), AttributeType::Pan, 2),
        2 => ("Tilt".to_string(), AttributeType::Tilt, 2),
        n => (format!("Attr{n}"), AttributeType::Control, 1),
    };
    ChannelFunction {
        attribute_name: name,
        attribute_type: ty,
        dmx_offset: 0,
        byte_count: bytes,
        default_value: 0,
        default_fraction: Some(0.5),
        physical_range: None,
        is_virtual: false,
        resolved: true,
        ..Default::default()
    }
}

/// `fixtures` fixtures of 26 channel functions (28 DMX channels each),
/// packed into universes; one running cue gives every attribute a value,
/// user 1's programmer holds a quarter of the dimmers, a group master fades
/// over half the rig and one raw channel is parked.
pub fn show(fixtures: u32) -> ShowData {
    let mut show = ShowData::new();
    let mut cfs = Vec::new();
    let mut offset = 0u16;
    for i in 0..FUNCTIONS {
        let mut cf = function(i);
        cf.dmx_offset = offset;
        offset += u16::from(cf.byte_count);
        cfs.push(cf);
    }
    let footprint = offset;
    let ft = FixtureType {
        meta: NodeMeta::new(Some("Bench".into())),
        manufacturer: "LumaCore".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![Revision {
            meta: NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![Mode {
                meta: NodeMeta::new(None),
                channel_functions: cfs.clone(),
                dmx_footprint: footprint,
                geometry: String::new(),
                cells: Vec::new(),
                relations: Vec::new(),
            }],
        }],
    };
    let ftg = ft.meta.identity.guid;
    let (rev, mode) = ft.mode_guids(0, 0).expect("mode 0");
    show.devices.dmx.patch.fixture_types.insert(1, std::sync::Arc::new(ft));

    let per_universe = u32::from(512 / footprint);
    let mut cue = CueObject::new("");
    let mut group = lumacore_tree::Group::new(Some("Half".into()));
    group.master = GroupMasterMode::Scaling;
    for n in 0..fixtures {
        let universe_no = n / per_universe + 1;
        let address = (n % per_universe) * u32::from(footprint) + 1;
        show.devices
            .dmx
            .patch
            .universes
            .entry(universe_no)
            .or_insert_with(|| Universe::new(None));
        let ug = show.devices.dmx.patch.universes[&universe_no].meta.identity.guid;
        let mut fixture = Fixture::new(None, ftg, rev, mode, ug, address as u16, footprint);
        fixture.mvr.fid = Some(n + 1);
        let g = fixture.meta.identity.guid;
        show.devices.dmx.patch.insert_fixture(fixture);
        for cf in &cfs {
            cue.part0().values.push(CueValue {
                fixture_id: g,
                attribute: cf.attribute_name.clone(),
                timing: Default::default(),
                value: lumacore_tree::CueData::Value(Value::normalized(f64::from(n % 100))),
            });
        }
        if n % 4 == 0 {
            show.users
                .get_mut(&1)
                .expect("user 1")
                .programmer
                .set_value(g, "Dimmer", Value::normalized(75.0));
        }
        if n % 2 == 0 {
            group.members.push(lumacore_tree::GroupMember {
                fixture: g,
                cell: Default::default(),
            });
        }
    }
    let group_guid = group.meta.identity.guid;
    show.datapools
        .values_mut()
        .next()
        .expect("a datapool")
        .groups
        .insert(1, group);
    show.playbacks.live.masters.groups.insert(
        group_guid,
        MasterLevel {
            from: 0.0,
            to: 1.0,
            start_ms: 0,
            fade_ms: u64::MAX / 2,
        },
    );

    let mut seq = SequenceObject::default();
    seq.cues.insert(1000, cue);
    let seq_guid = seq.meta.identity.guid;
    show.datapools.entry(1).or_insert_with(|| lumacore_tree::Datapool::new(None)).sequences.insert(1, Arc::new(seq));
    let admin = show.users[&1].meta.identity.guid;
    show.playbacks.live.sequences.insert(
        seq_guid,
        SequenceState::asserted_at(1000, admin, lumacore_tree::next_stamp()),
    );
    let ug = show.devices.dmx.patch.universes[&1].meta.identity.guid;
    show.playback.park.park_channel(ug, 512, 9);
    show
}

/// The engine input for [`show`] (user 1 logged in, no modes).
pub fn input(fixtures: u32) -> EngineInput {
    EngineInput {
        show: Arc::new(show(fixtures)),
        users: vec![UserLayer {
            user_no: 1,
            ..Default::default()
        }],
        identify: Vec::new(),
        generation: 1,
        patch_generation: 1,
    }
}
