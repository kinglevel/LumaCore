// ██████╗ ███████╗███╗   ██╗██████╗ ███████╗██████╗     ██████╗ ███████╗███╗   ██╗ ██████╗██╗  ██╗
// ██╔══██╗██╔════╝████╗  ██║██╔══██╗██╔════╝██╔══██╗    ██╔══██╗██╔════╝████╗  ██║██╔════╝██║  ██║
// ██████╔╝█████╗  ██╔██╗ ██║██║  ██║█████╗  ██████╔╝    ██████╔╝█████╗  ██╔██╗ ██║██║     ███████║
// ██╔══██╗██╔══╝  ██║╚██╗██║██║  ██║██╔══╝  ██╔══██╗    ██╔══██╗██╔══╝  ██║╚██╗██║██║     ██╔══██║
// ██║  ██║███████╗██║ ╚████║██████╔╝███████╗██║  ██║    ██████╔╝███████╗██║ ╚████║╚██████╗██║  ██║
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝    ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝╚═╝  ╚═╝
// FILE: render bench

//! The render tick and the plan compiler on a show the size of a probed large rig (35,464
//! parameters): `cargo bench -p lumacore-render --bench render`.
//!
//! Besides the plain tick, the paths that cost the most (decisions §31 "Render budget"): the
//! tick with DMX input merging, the tick while half the rig fades, a compile after one
//! programmer change on a 500-cue tracking sequence, a full compile (a patch change rebuilds
//! the channel tier) and the copy of the show the state thread publishes after a change.
//! Baselines: `-- --save-baseline main`, then compare with `-- --baseline main` (criterion
//! reports a regression); the numbers of record are in `docs/dev/benchmarks.md`.
//!
//! Owns:          the render benchmarks (tick, input merge, fades, compiles, the show copy).
//! Doesn't own:   the budget test (engine/tests.rs), the numbers of record (docs/dev/benchmarks.md).
//! Entry points:  the criterion groups.
//! Decisions:     §24, §31 "Render budget".

use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use lumacore_render::bench_show;
use lumacore_protocols::input::InputLayer;
use lumacore_render::plan::Plan;
use lumacore_render::merge::EngineInput;
use lumacore_render::render::{NoStages, RenderedFrame, render_into};
use lumacore_tree::{CueObject, CueValue, SequenceState, Value};
use lumacore_protocols::{InputPriority};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

const FIXTURES: u32 = 1364;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn render(c: &mut Criterion) {
    let input = bench_show::input(FIXTURES);
    let plan = Plan::compile(&input, None);
    let mut frame = RenderedFrame::for_tier(&plan.channel);
    let mut now = 0u64;
    c.bench_function("render_tick_35k", |b| {
        b.iter(|| {
            now += 16;
            render_into(&mut frame, &plan, now, std::iter::empty(), &mut NoStages);
        })
    });

    // 64 input layers: two lines on each of the first 32 universes, changing every channel.
    let layers: Vec<InputLayer> = (0..64u32)
        .map(|i| InputLayer {
            line: 1 + i % 2,
            universe: 1 + i / 2,
            band: if i % 2 == 0 { InputPriority::Ltp } else { InputPriority::High },
            data: [(i * 3) as u8; 512],
            stamps: Arc::new([u64::from(i) + 1; 512]),
        })
        .collect();
    c.bench_function("render_tick_35k_input_64_layers", |b| {
        b.iter(|| {
            now += 16;
            render_into(&mut frame, &plan, now, layers.iter(), &mut NoStages);
        })
    });

    // Half the rig fading: the cue's values change with a long fade, compiled after the plan
    // that renders now, so those channels carry a running fade.
    let faded = with_cue(&input, |cue| {
        cue.part0().fade_in = 600.0;
        for (i, v) in cue.part0().values.iter_mut().enumerate() {
            if i % 2 == 0 {
                v.value = Value::normalized(100.0);
            }
        }
    });
    let fading = Plan::compile_after(&faded, Some(&plan.channel), Some(&plan), 1_000);
    let fades = fading.value.channels.iter().filter(|v| v.fade.is_some()).count();
    eprintln!("half_fading: {fades} channels fading");
    let mut fade_frame = RenderedFrame::for_tier(&fading.channel);
    let mut t = 1_000u64;
    c.bench_function("render_tick_35k_half_fading", |b| {
        b.iter(|| {
            t += 16;
            render_into(&mut fade_frame, &fading, t, std::iter::empty(), &mut NoStages);
        })
    });

    // Where a compile's time goes: the merge and the release pass on their own.
    c.bench_function("resolve_35k", |b| b.iter(|| lumacore_render::merge::resolve(&input)));
    c.bench_function("releases_35k", |b| b.iter(|| lumacore_render::merge::releases(&input)));
    c.bench_function("resolve_by_attr_index_35k", |b| {
        b.iter(|| lumacore_render::merge::resolve_by(&input, |f, a| plan.channel.attr_of(f, a)))
    });
    let seq = &input.show.datapools[&1].sequences[&1];
    c.bench_function("tracked_one_cue_35k", |b| b.iter(|| seq.tracked(1000).len()));
    c.bench_function("compile_value_tier_35k", |b| {
        b.iter(|| Plan::compile(&input, Some(&plan.channel)))
    });
    c.bench_function("compile_full_35k (channel tier rebuilt)", |b| b.iter(|| Plan::compile(&input, None)));

    // A 500-cue tracking sequence at its last cue; one programmer value changes.
    let tracking = tracking_input(&input, 500);
    let before = Plan::compile(&tracking, None);
    let changed = one_programmer_change(&tracking);
    c.bench_function("compile_after_one_change_500_cue_tracking", |b| {
        b.iter(|| Plan::compile_after(&changed, Some(&before.channel), Some(&before), 2_000))
    });
    // What the compiler thread does: its sequence cache is warm (the sequence's Arc and cue
    // didn't change), only the programmers are merged again (decisions §31).
    let mut cache = lumacore_render::merge::SequenceCache::default();
    let warm = Plan::compile_cached(&tracking, None, None, 1_000, &mut cache);
    c.bench_function("compile_cached_one_change_500_cue_tracking", |b| {
        b.iter(|| Plan::compile_cached(&changed, Some(&warm.channel), Some(&warm), 2_000, &mut cache))
    });

    // What the state thread copies on every published change today (decisions §31 "Show
    // snapshots": a deep clone of the whole show).
    c.bench_function("publish_show_clone_35k", |b| b.iter(|| input.show.as_ref().clone()));
}

/// `input` with sequence 1's running cue changed by `f`.
fn with_cue(input: &EngineInput, f: impl FnOnce(&mut CueObject)) -> EngineInput {
    let mut show = input.show.as_ref().clone();
    let seq = show.sequence_mut(1, 1).expect("sequence 1");
    f(seq.cues.get_mut(&1000).expect("cue 1"));
    // Gone to again (a newer stamp): the compiler fades what changed.
    let seq_guid = seq.meta.identity.guid;
    let admin = show.users[&1].meta.identity.guid;
    show.playbacks.live.sequences.insert(seq_guid, SequenceState::asserted_at(1000, admin, lumacore_tree::next_stamp()));
    EngineInput { show: Arc::new(show), generation: input.generation + 1, ..input.clone() }
}

/// `input` whose sequence 1 has `cues` cues — cue 1 the full look, every later cue moves the
/// dimmers of 1 fixture in 50 (tracking keeps the rest) — running at its last cue.
fn tracking_input(input: &EngineInput, cues: u32) -> EngineInput {
    let mut show = input.show.as_ref().clone();
    let fixtures: Vec<_> = show.devices.dmx.patch.fixtures.values().map(|f| f.meta.identity.guid).collect();
    let seq = show.sequence_mut(1, 1).expect("sequence 1");
    for n in 2..=cues {
        let mut cue = CueObject::new("");
        for (i, g) in fixtures.iter().enumerate() {
            if i % 50 == (n as usize % 50) {
                cue.part0().values.push(CueValue {
                    fixture_id: *g,
                    attribute: "Dimmer".into(),
                    timing: Default::default(),
                    value: Value::normalized(f64::from(n % 100)),
                });
            }
        }
        seq.cues.insert(n * 1000, cue);
    }
    let seq_guid = seq.meta.identity.guid;
    let admin = show.users[&1].meta.identity.guid;
    show.playbacks
        .live.sequences
        .insert(seq_guid, SequenceState::asserted_at(cues * 1000, admin, lumacore_tree::next_stamp()));
    EngineInput { show: Arc::new(show), generation: input.generation + 1, ..input.clone() }
}

/// `input` with one more programmer value (a fader move).
fn one_programmer_change(input: &EngineInput) -> EngineInput {
    let mut show = input.show.as_ref().clone();
    let g = show.devices.dmx.patch.fixtures[&2].meta.identity.guid;
    show.users.get_mut(&1).expect("user 1").programmer.set_value(g, "Dimmer", Value::normalized(33.0));
    EngineInput { show: Arc::new(show), generation: input.generation + 1, ..input.clone() }
}

criterion_group!(benches, render);
criterion_main!(benches);
