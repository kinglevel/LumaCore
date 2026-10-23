// ███╗   ███╗███████╗██████╗  ██████╗ ███████╗    ████████╗███████╗███████╗████████╗███████╗
// ████╗ ████║██╔════╝██╔══██╗██╔════╝ ██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██╔████╔██║█████╗  ██████╔╝██║  ███╗█████╗         ██║   █████╗  ███████╗   ██║   ███████╗
// ██║╚██╔╝██║██╔══╝  ██╔══██╗██║   ██║██╔══╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║ ╚═╝ ██║███████╗██║  ██║╚██████╔╝███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: merge tests

//! Tests of `merge`.
//!
//! Owns:          the unit tests of merge.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §27, §29, §32.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn a_feature_groups_timing_overrides_the_parts_for_its_attributes() {
    let mut show = lumacore_tree::ShowData::new();
    let pos = show.feature_groups.values().find(|g| g.meta.name() == Some("Position")).unwrap().meta.identity.guid;
    for (name, g) in [("Pan", Some(pos)), ("Dimmer", None)] {
        show.attribute_defs.insert(
            name.into(),
            lumacore_fixtures::AttributeDefinition {
                meta: lumacore_base::NodeMeta::with_guid(lumacore_base::Guid::named("Attribute", name).0, None),
                name: name.into(),
                pretty: name.into(),
                physical_unit: Default::default(),
                natural_readout: Default::default(),
                feature_group: g,
                activation_group: None,
                main_attribute: None,
                encoder_resolution: Default::default(),
                intensity: 0.0,
                mib: false,
                feature: None,
                special: None,
                position: 1,
                placement: Default::default(),
                new: false,
            },
        );
    }
    let mut part = lumacore_tree::CuePart { fade_in: 3.0, delay_in: 1.0, ..Default::default() };
    part.group_timing.insert(pos, lumacore_tree::GroupTiming { fade_in: Some(0.5), ..Default::default() });
    let pan = Timing::of_attribute(&part, "Pan", &show);
    assert_eq!((pan.delay_in, pan.fade_in), (1000, 500));
    let dim = Timing::of_attribute(&part, "Dimmer", &show);
    assert_eq!((dim.delay_in, dim.fade_in), (1000, 3000));
}

use super::*;
use lumacore_protocols::input::InputLayer;
use lumacore_fixtures::fixture::Fixture;
use lumacore_fixtures::fixture_type::AttributeType;
use lumacore_protocols::{InputPriority, MergeMode};
use lumacore_fixtures::fixture_type::{FixtureType, Mode, Revision};
use lumacore_tree::node::NodeMeta;
use lumacore_fixtures::universe::Universe;
use lumacore_tree::{SequenceObject, SequenceState};

fn cf(name: &str, ty: AttributeType, offset: u16, bytes: u8) -> ChannelFunction {
    ChannelFunction {
        attribute_name: name.to_string(),
        attribute_type: ty,
        dmx_offset: offset,
        byte_count: bytes,
        default_value: 0,
        default_fraction: None,
        physical_range: None,
        is_virtual: false,
        resolved: true,
        ..Default::default()
    }
}

/// One universe, one fixture: Dimmer (8 bit) at 1, Pan (16 bit) at 2–3.
fn show() -> (ShowData, Uuid) {
    let mut show = ShowData::new();
    let universe = Universe::new(None);
    let ug = universe.meta.guid();
    show.devices.dmx.patch.universes.insert(1, universe);
    let mut pan = cf("Pan", AttributeType::Pan, 1, 2);
    pan.default_fraction = Some(0.5);
    let ft = FixtureType {
        meta: NodeMeta::new(Some("T".into())),
        manufacturer: "M".into(),
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
                channel_functions: vec![cf("Dimmer", AttributeType::Dimmer, 0, 1), pan],
                dmx_footprint: 3,
                geometry: String::new(),
                cells: Vec::new(),
                relations: Vec::new(),
            }],
        }],
    };
    let ftg = ft.meta.guid();
    let (rev, mode) = ft.mode_guids(0, 0).unwrap();
    show.devices.dmx.patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    let fixture = Fixture::new(None, ftg, rev, mode, ug, 1, 3);
    let g = fixture.meta.guid();
    show.devices.dmx.patch.insert_fid(1, fixture);
    (show, g)
}

fn ug_of(show: &ShowData) -> uuid::Uuid {
    show.devices.dmx.patch.universes[&1].meta.identity.guid
}

fn frame_of(show: ShowData, users: Vec<UserLayer>) -> [u8; 512] {
    let input = EngineInput {
        users,
        ..EngineInput::new(Arc::new(show))
    };
    *crate::render::render_now(&input, 0).get(1).unwrap()
}

#[test]
fn defaults_and_sixteen_bit() {
    let (show, _) = show();
    let f = frame_of(show, vec![]);
    assert_eq!(f[0], 0);
    assert_eq!((f[1], f[2]), (0x80, 0x00));
}

#[test]
fn latest_touch_wins_between_programmers_and_cues() {
    let (mut show, g) = show();
    let mut seq = SequenceObject::default();
    let mut cue = lumacore_tree::CueObject::new("");
    cue.part0().values.push(lumacore_tree::CueValue {
        fixture_id: g,
        attribute: "Dimmer".into(),
        timing: Default::default(),
        value: Value::normalized(100.0),
    });
    seq.cues.insert(1000, cue);
    let seq_guid = seq.meta.identity.guid;
    show.datapools.entry(1).or_insert_with(|| lumacore_tree::Datapool::new(None)).sequences.insert(1, std::sync::Arc::new(seq));
    let admin = show.users[&1].meta.identity.guid;
    let prog = &mut show.users.get_mut(&1).unwrap().programmer;
    prog.set_value(g, "Dimmer", Value::normalized(20.0));
    // The cue goes after the programmer value: the cue wins.
    let later = lumacore_tree::next_stamp();
    show.playbacks
        .live.sequences
        .insert(seq_guid, SequenceState::asserted_at(1000, admin, later));
    assert_eq!(frame_of(show.clone(), vec![])[0], 255);
    // Freeze lifts the programmer above the cue.
    let frozen = vec![UserLayer {
        user_no: 1,
        freeze: true,
        ..Default::default()
    }];
    assert_eq!(frame_of(show.clone(), frozen)[0], 51);
    // Blind takes the programmer out.
    let prog = &mut show.users.get_mut(&1).unwrap().programmer;
    prog.set_value(g, "Dimmer", Value::normalized(20.0));
    let blind = vec![UserLayer {
        user_no: 1,
        blind: true,
        ..Default::default()
    }];
    assert_eq!(frame_of(show, blind)[0], 255);
}

#[test]
fn grand_master_solo_and_park() {
    let (mut show, g) = show();
    let prog = &mut show.users.get_mut(&1).unwrap().programmer;
    prog.set_value(g, "Dimmer", Value::normalized(100.0));
    show.playbacks.live.masters.grand.master = 0.5;
    assert_eq!(frame_of(show.clone(), vec![])[0], 128);
    show.playbacks.live.masters.grand.on = false;
    assert_eq!(frame_of(show.clone(), vec![])[0], 0);
    // Parked values ignore masters.
    show.playback
        .park
        .park(g, "Dimmer", Value::normalized(100.0));
    assert_eq!(frame_of(show.clone(), vec![])[0], 255);
    show.playback.park.unpark(g, None);
    show.playbacks.live.masters.grand.on = true;
    show.playbacks.live.masters.grand.master = 1.0;
    // Solo by a user with nothing selected: everything else goes dark.
    let solo = vec![UserLayer {
        user_no: 2,
        solo: 1.0,
        ..Default::default()
    }];
    assert_eq!(frame_of(show.clone(), solo)[0], 0);
    // A Solo master at half: the others go to half.
    let half = vec![UserLayer {
        user_no: 2,
        solo: 0.5,
        ..Default::default()
    }];
    assert_eq!(frame_of(show.clone(), half)[0], 128);
    show.playback.park.park_channel(ug_of(&show), 10, 77);
    assert_eq!(frame_of(show, vec![])[9], 77);
}

fn layer(line: u32, band: InputPriority, value: u8, stamp: u64) -> InputLayer {
    InputLayer {
        line,
        universe: 1,
        band,
        data: [value; 512],
        stamps: Arc::new([stamp; 512]),
    }
}

fn merged(show: &ShowData, users: Vec<UserLayer>, layers: &[InputLayer]) -> ([u8; 512], [Winner; 512]) {
    let input = EngineInput {
        users,
        ..EngineInput::new(Arc::new(show.clone()))
    };
    let plan = crate::plan::Plan::compile(&input, None);
    let mut frame = crate::render::RenderedFrame::for_tier(&plan.channel);
    crate::render::render_into(&mut frame, &plan, 0, layers.iter(), &mut crate::render::NoStages);
    (*frame.get(1).unwrap(), *frame.winners(1).unwrap())
}

#[test]
fn input_merges_by_mode_band_and_stamp() {
    let (mut show, g) = show();
    let prog = &mut show.users.get_mut(&1).unwrap().programmer;
    prog.set_value(g, "Dimmer", Value::normalized(20.0)); // 51
    // Input changed long before / after the programmer's touch.
    let (older, newer) = (1, lumacore_tree::next_stamp() + 1_000_000);

    // Prio LTP: the latest change wins, against our dimmer.
    let (f, w) = merged(&show, vec![], &[layer(1, InputPriority::Ltp, 200, older)]);
    assert_eq!((f[0], w[0]), (51, Winner::Ours));
    // Pan is at its default and channel 10 is unpatched: input takes them.
    assert_eq!((f[1], w[1], f[9], w[9]), (200, Winner::Input(1), 200, Winner::Input(1)));
    let (f, w) = merged(&show, vec![], &[layer(1, InputPriority::Ltp, 200, newer)]);
    assert_eq!((f[0], w[0]), (200, Winner::Input(1)));
    // Low only fills where we give nothing.
    let (f, _) = merged(&show, vec![], &[layer(1, InputPriority::Low, 200, newer)]);
    assert_eq!((f[0], f[1]), (51, 200));
    // High beats our playbacks/programmers, but not a frozen programmer;
    // Super beats that too.
    let frozen = || {
        vec![UserLayer {
            user_no: 1,
            freeze: true,
            ..Default::default()
        }]
    };
    let (f, _) = merged(&show, vec![], &[layer(1, InputPriority::High, 200, older)]);
    assert_eq!(f[0], 200);
    let (f, _) = merged(&show, frozen(), &[layer(1, InputPriority::High, 200, newer)]);
    assert_eq!(f[0], 51);
    let (f, _) = merged(&show, frozen(), &[layer(1, InputPriority::Prog, 200, newer)]);
    assert_eq!(f[0], 200, "Prog: LTP with the frozen programmer");
    let (f, _) = merged(&show, frozen(), &[layer(1, InputPriority::Super, 200, older)]);
    assert_eq!(f[0], 200);
    // The higher band wins between lines; masters don't touch input.
    show.playbacks.live.masters.grand.on = false;
    let (f, w) = merged(
        &show,
        vec![],
        &[
            layer(2, InputPriority::Low, 250, newer),
            layer(3, InputPriority::Super, 90, older),
        ],
    );
    assert_eq!((f[0], w[0]), (90, Winner::Input(3)));
    show.playbacks.live.masters.grand.on = true;

    // HTP / LowTP / Off.
    let uni = show.devices.dmx.patch.universes.get_mut(&1).unwrap();
    uni.merge_mode = MergeMode::Htp;
    let (f, w) = merged(&show, vec![], &[layer(1, InputPriority::Lowest, 40, older)]);
    assert_eq!((f[0], w[0], f[9]), (51, Winner::Ours, 40));
    show.devices.dmx.patch.universes.get_mut(&1).unwrap().merge_mode = MergeMode::LowTp;
    let (f, _) = merged(&show, vec![], &[layer(1, InputPriority::Ltp, 40, older)]);
    assert_eq!((f[0], f[9]), (40, 0));
    show.devices.dmx.patch.universes.get_mut(&1).unwrap().merge_mode = MergeMode::Off;
    let (f, w) = merged(&show, vec![], &[layer(1, InputPriority::Super, 200, newer)]);
    assert_eq!((f[0], f[9], w[9]), (51, 0, Winner::Unpatched));

    // Park always wins, fixture attributes and raw channels alike.
    show.devices.dmx.patch.universes.get_mut(&1).unwrap().merge_mode = MergeMode::Prio;
    show.playback.park.park(g, "Dimmer", Value::normalized(0.0));
    show.playback.park.park_channel(ug_of(&show), 10, 7);
    let (f, w) = merged(&show, vec![], &[layer(1, InputPriority::Super, 200, newer)]);
    assert_eq!((f[0], w[0], f[9], w[9]), (0, Winner::Park, 7, Winner::Park));
}
