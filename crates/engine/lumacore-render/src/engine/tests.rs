// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The render rules (decisions §24 "Step 9 hardening"): the tick allocates
//! nothing, stays inside its budget, and the engine publishes frames from
//! the compiled plan, holds the last frame on a panic and frees nothing on
//! its own thread.
//!
//! Owns:          the render rules' tests: no allocation in the tick, the 35k-parameter budget, the engine's frames, panics and hand-back.
//! Doesn't own:   the benchmark (benches/render.rs).
//! Entry points:  the #[test] fns.
//! Decisions:     §24 "Step 9 hardening", §27, §31 "Render budget".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::bench_show;
use crate::engine::{RenderHub, spawn_engine};
use crate::plan::Plan;
use crate::merge::EngineInput;
use crate::render::{NoStages, RenderedFrame, render_into};

/// The large rig probed for §24: 35,451 parameters (here 1364 × 26).
const RIG: u32 = 1364;

#[test]
#[cfg(debug_assertions)]
fn the_tick_allocates_nothing() {
    let input = bench_show::input(64);
    let plan = Plan::compile(&input, None);
    let mut frame = RenderedFrame::for_tier(&plan.channel);
    let layers = [lumacore_protocols::input::InputLayer {
        line: 1,
        universe: 1,
        band: lumacore_protocols::InputPriority::Ltp,
        data: [7; 512],
        stamps: Arc::new([1; 512]),
    }];
    let before = assert_no_alloc::violation_count();
    assert_no_alloc::assert_no_alloc(|| {
        render_into(&mut frame, &plan, 1234, layers.iter(), &mut NoStages);
    });
    assert_eq!(assert_no_alloc::violation_count(), before, "the tick allocated");
    // It did render: the cue's values, the parked raw channel.
    assert!(frame.get(1).unwrap().iter().any(|&v| v != 0));
    assert_eq!(frame.get(1).unwrap()[511], 9);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "the budget is for release builds")]
fn a_35k_parameter_frame_renders_under_3_ms() {
    let input = bench_show::input(RIG);
    let plan = Plan::compile(&input, None);
    assert!(plan.channel.channels.len() >= 35_000);
    let mut frame = RenderedFrame::for_tier(&plan.channel);
    let mut worst = Duration::ZERO;
    for n in 0..200 {
        let t = Instant::now();
        render_into(&mut frame, &plan, n * 16, std::iter::empty(), &mut NoStages);
        worst = worst.max(t.elapsed());
    }
    assert!(worst < Duration::from_millis(3), "worst frame {worst:?}");
}

fn wait_for(what: &str, mut ok: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(2);
    while !ok() {
        assert!(Instant::now() < until, "{what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn engine_renders_published_input_and_merges_dmx_input() {
    let hub = RenderHub::new(EngineInput::new(Arc::new(lumacore_tree::tree::ShowData::new())));
    let mut engine = spawn_engine(hub.clone(), None, None, Box::new(|_| {}));

    let mut input = bench_show::input(4);
    input.generation = 7;
    hub.publish(input);
    assert!(hub.wait_rendered(7, Duration::from_secs(2)), "plan 7 never rendered");
    let frame = hub.frame.load_full();
    assert_eq!(frame.generation, 7);
    assert!(frame.frame_no > 0);
    assert_eq!(frame.get(1).unwrap()[511], 9);
    assert_eq!(hub.plan.load().generation, 7);

    // DMX input merges in (Prio, Super beats ours).
    let id = hub.driver_id();
    hub.set_inputs(
        id,
        Arc::new(vec![lumacore_protocols::input::InputLayer {
            line: 3,
            universe: 1,
            band: lumacore_protocols::InputPriority::Super,
            data: [200; 512],
            stamps: Arc::new([1; 512]),
        }]),
    );
    wait_for("input never merged", || hub.frame.load().get(1).unwrap()[0] == 200);
    assert_eq!(
        hub.frame.load().winners(1).unwrap()[0],
        crate::merge::Winner::Input(3)
    );
    assert_eq!(hub.frame.load().ours(1).unwrap()[511], 9);

    // Frames keep coming at the render rate.
    let n = hub.frame.load().frame_no;
    std::thread::sleep(Duration::from_millis(200));
    let m = hub.frame.load().frame_no;
    assert!((5..=20).contains(&(m - n)), "{} frames in 200 ms", m - n);
    let stats = hub.stats.snapshot();
    assert_eq!(stats.rate_hz, 60);
    assert!(stats.compiles >= 1 && stats.channels > 0);
    // Debug builds panic on an allocation in the tick: none happened.
    assert_eq!((stats.panics, stats.busy), (0, 0));

    engine.stop();
    for t in engine.take_threads() {
        t.join().unwrap();
    }
}

/// R1-66: a compile that panics is caught and counted; the engine keeps the
/// last plan, the generation isn't retried in a loop, the next one compiles;
/// the compiler is on the health board.
#[test]
fn a_compiler_panic_keeps_the_last_plan_and_the_next_input_compiles() {
    use std::sync::atomic::Ordering;
    let hub = RenderHub::new(EngineInput::new(Arc::new(lumacore_tree::tree::ShowData::new())));
    let board = lumacore_tree::runtime::HealthBoard::default();
    let mut engine = spawn_engine(
        hub.clone(),
        None,
        Some(board.register("Render.Compiler")),
        Box::new(|_| {}),
    );
    let mut input = bench_show::input(2);
    input.generation = 1;
    hub.publish(input);
    assert!(hub.wait_rendered(1, Duration::from_secs(2)));
    let compiles = hub.stats.snapshot().compiles;

    hub.panic_next_compile.store(true, Ordering::Relaxed);
    let mut input = bench_show::input(2);
    input.generation = 2;
    hub.publish(input);
    wait_for("the panic was never counted", || hub.stats.snapshot().compile_panics == 1);
    // Not retried; the engine still renders plan 1.
    std::thread::sleep(Duration::from_millis(150));
    let v = hub.stats.snapshot();
    assert_eq!((v.compile_panics, v.compiles), (1, compiles));
    assert_eq!(hub.plan.load().generation, 1);
    assert_eq!(hub.frame.load().generation, 1);

    let mut input = bench_show::input(2);
    input.generation = 3;
    hub.publish(input);
    assert!(hub.wait_rendered(3, Duration::from_secs(2)), "plan 3 never rendered");
    let threads = board.snapshot();
    let compiler = threads.iter().find(|t| t.name == "Render.Compiler").expect("on the board");
    assert_eq!(compiler.panics, 1);
    assert_eq!(hub.stats.snapshot().stale_since, 0, "a compile that works ends the stale episode");

    // Q39: the stale flag latches; three panics within 60 s (change 2 above,
    // 4 and 5 here) stop the compiler (the last plan stays), a restart
    // brings it back.
    for generation in 4..=5 {
        hub.panic_next_compile.store(true, Ordering::Relaxed);
        let mut input = bench_show::input(2);
        input.generation = generation;
        hub.publish(input);
        wait_for("panic not counted", || hub.stats.snapshot().compile_panics == generation - 2);
        assert_eq!(hub.stats.snapshot().stale_since, 4, "latched at the first failing change");
    }
    assert!(hub.stats.snapshot().compiler_down);
    let mut input = bench_show::input(2);
    input.generation = 6;
    hub.publish(input);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(hub.plan.load().generation, 3, "a stopped compiler compiles nothing; plan 3 stays");
    hub.restart_compiler();
    assert!(hub.wait_rendered(6, Duration::from_secs(2)), "restarted, it compiles the newest input");
    let v = hub.stats.snapshot();
    assert!(!v.compiler_down && v.stale_since == 0);

    engine.stop();
    for t in engine.take_threads() {
        t.join().unwrap();
    }
}

#[test]
fn a_panic_holds_the_last_frame_and_restarts() {
    use std::sync::atomic::{AtomicU32, Ordering};
    let hub = RenderHub::new(EngineInput::new(Arc::new(lumacore_tree::tree::ShowData::new())));
    let panics = Arc::new(AtomicU32::new(0));
    let seen = panics.clone();
    let mut engine = spawn_engine(
        hub.clone(),
        None,
        None,
        Box::new(move |_| {
            seen.fetch_add(1, Ordering::Relaxed);
        }),
    );
    let mut input = bench_show::input(2);
    input.generation = 1;
    hub.publish(input);
    assert!(hub.wait_rendered(1, Duration::from_secs(2)));

    // A plan whose channel points past its universes makes the tick panic.
    let mut bad = Plan::compile(&bench_show::input(2), None);
    let mut tier = (*bad.channel).clone();
    tier.channels[0].fixture = 9_999;
    tier.channels[0].dimmer = true;
    bad.channel = Arc::new(tier);
    bad.value.channels[0].owner = crate::merge::Owner::Default;
    bad.generation = 2;
    let pool = (0..crate::engine::POOL)
        .map(|_| Arc::new(RenderedFrame::for_tier(&bad.channel)))
        .collect();
    hub.bundle_tx
        .send(crate::engine::Bundle {
            plan: Arc::new(bad),
            pool,
        })
        .unwrap();
    wait_for("no restart", || hub.stats.snapshot().restarts == 1);
    assert_eq!(panics.load(Ordering::Relaxed), 1);
    // The last good frame is still published; the recompiled plan renders.
    assert_eq!(hub.frame.load().get(1).unwrap()[511], 9);
    wait_for("no fresh plan", || hub.frame.load().generation == 1 && hub.stats.snapshot().compiles >= 2);
    assert!(!hub.stats.snapshot().down);

    engine.stop();
    for t in engine.take_threads() {
        t.join().unwrap();
    }
}

/// A fade's arithmetic saturates (decisions §26 "checked arithmetic"): a
/// start, delay or fade near `u64::MAX` never overflows in the tick.
#[test]
fn a_fade_near_u64_max_never_overflows() {
    use crate::plan::Fade;
    let huge = Fade {
        from: 0.0,
        start_ms: u64::MAX - 10,
        delay_ms: u64::MAX,
        fade_ms: u64::MAX,
        snap: false,
        path: Default::default(),
    };
    assert_eq!(huge.at(1.0, 0), 0.0, "not started yet");
    assert_eq!(huge.at(1.0, u64::MAX - 1), 0.0, "the delay never ends");
    assert!(huge.running(u64::MAX - 1));
    let endless = Fade {
        from: 0.0,
        start_ms: 1_000,
        delay_ms: 0,
        fade_ms: u64::MAX,
        snap: false,
        path: Default::default(),
    };
    assert!(endless.at(1.0, u64::MAX) <= 1.0);
    assert!(endless.running(u64::MAX - 1));
    let snap = Fade { snap: true, ..endless };
    assert_eq!(snap.at(1.0, 1_000), 1.0);
    assert!(!snap.running(1_000));
}

/// A fade's Path shapes it (decisions §30 "12f" sequence window A): the same ends, a different
/// middle; Snap jumps at the end of its delay.
#[test]
fn a_fade_follows_its_path() {
    use crate::plan::Fade;
    use lumacore_tree::FadePath;
    let f = |path| Fade { from: 0.0, start_ms: 0, delay_ms: 100, fade_ms: 1_000, snap: false, path };
    for p in FadePath::ALL {
        assert_eq!(f(p).at(1.0, 50), 0.0, "{p:?} waits for its delay");
        assert_eq!(f(p).at(1.0, 1_100), 1.0, "{p:?} arrives");
    }
    let quarter = |p| f(p).at(1.0, 350);
    assert!((quarter(FadePath::Linear) - 0.25).abs() < 1e-9);
    assert!(quarter(FadePath::EaseIn) < 0.25 && quarter(FadePath::EaseOut) > 0.25);
    assert!(quarter(FadePath::SCurve) < 0.25);
    assert_eq!(f(FadePath::Snap).at(1.0, 100), 1.0);
    assert!(!f(FadePath::Snap).running(100));
}

/// The compiler's sequence cache (decisions §31) answers exactly what the merge without it
/// does: after a programmer change (cache kept), a Go to another cue and an edit of the
/// sequence (worked out again).
#[test]
fn the_sequence_cache_merges_like_no_cache() {
    use crate::merge::{SequenceCache, resolve_by, resolve_cached};
    let base = bench_show::input(32);
    let plan = Plan::compile(&base, None);
    let key_of = |f: uuid::Uuid, a: &str| plan.channel.attr_of(f, a);
    let mut cache = SequenceCache::default();
    let same = |input: &crate::merge::EngineInput, cache: &mut SequenceCache<u32>| {
        let cached = resolve_cached(input, key_of, |_| false, cache);
        let plain = resolve_by(input, key_of);
        assert_eq!(cached.len(), plain.len());
        for (k, r) in &plain {
            let c = &cached[k];
            assert_eq!((c.stamp, c.source, &c.value, c.timing), (r.stamp, r.source, &r.value, r.timing), "attr {k}");
        }
    };
    same(&base, &mut cache);
    // A programmer change: the sequence's Arc is shared, the cache is used.
    let mut show = base.show.as_ref().clone();
    let g = show.devices.dmx.patch.fixture(3).expect("the fixture").meta.identity.guid;
    show.users.get_mut(&1).unwrap().programmer.set_value(g, "Pan", lumacore_tree::Value::normalized(10.0));
    let changed = crate::merge::EngineInput { show: Arc::new(show.clone()), ..base.clone() };
    same(&changed, &mut cache);
    // A second cue and a Go to it: the cue changed, worked out again.
    let mut cue = lumacore_tree::CueObject::new("");
    cue.part0().values.push(lumacore_tree::CueValue {
        fixture_id: g,
        attribute: "Dimmer".into(),
        timing: Default::default(),
        value: lumacore_tree::CueData::Value(lumacore_tree::Value::normalized(42.0)),
    });
    show.sequence_mut(1, 1).unwrap().cues.insert(2000, cue);
    let seq_guid = show.datapools[&1].sequences[&1].meta.identity.guid;
    let admin = show.users[&1].meta.identity.guid;
    show.playbacks.live.sequences.insert(seq_guid, lumacore_tree::SequenceState::asserted_at(2000, admin, lumacore_tree::next_stamp()));
    let gone = crate::merge::EngineInput { show: Arc::new(show.clone()), ..base.clone() };
    same(&gone, &mut cache);
    // An edit of the sequence at the same cue: a new Arc, worked out again.
    show.sequence_mut(1, 1).unwrap().cues.get_mut(&2000).unwrap().part0().values[0].value =
        lumacore_tree::CueData::Value(lumacore_tree::Value::normalized(7.0));
    let edited = crate::merge::EngineInput { show: Arc::new(show), ..base };
    same(&edited, &mut cache);
}
