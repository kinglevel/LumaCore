// ███████╗████████╗ █████╗ ████████╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ███████╗   ██║   ███████║   ██║   █████╗         ██║   █████╗  ███████╗   ██║   ███████╗
// ╚════██║   ██║   ██╔══██║   ██║   ██╔══╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ███████║   ██║   ██║  ██║   ██║   ███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: state tests

//! Tests of `state`.
//!
//! Owns:          the unit tests of state.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §26, §31, R1-67, Q40, §34 24.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn context() -> ServerContext {
    ServerContext::new(lumacore_objects::AppState::new(), None, false, 0, 0, 0)
}

#[test]
fn a_panicking_job_is_reported_and_the_thread_keeps_running() {
    let published = Published::new(ShowData::default(), Default::default());
    let (_inbound_tx, inbound_rx) = crossbeam_channel::bounded(1);
    let (state, thread) = spawn(context(), inbound_rx, Vec::new(), published.clone());

    let err = state
        .call(|_ctx| -> u32 { panic!("job broke") })
        .unwrap_err();
    match err {
        CallError::Panicked(report) => {
            assert_eq!(report.message, "job broke");
            assert!(report.file.ends_with("state/tests.rs"), "{}", report.file);
        }
        CallError::Stopped | CallError::Reentrant | CallError::Busy { .. } | CallError::Full => panic!("unexpected {err:?}"),
    }

    // Still serving, and the show it publishes follows changes.
    state
        .call(|ctx| {
            ctx.state.root.show_data.show_settings.show_name = "Published".into();
            ctx.state.show_generation += 1;
        })
        .unwrap();
    let name = state
        .call(|ctx| ctx.state.root.show_data.show_settings.show_name.clone())
        .unwrap();
    assert_eq!(name, "Published");
    assert_eq!(published.show.load().show_settings.show_name, "Published");

    let panics = state
        .call(|ctx| {
            ctx.shutdown_requested = true;
            ctx.state.root.runtime.health.snapshot()[0].panics
        })
        .unwrap();
    assert_eq!(panics, 1);
    let ctx = thread.join().expect("state thread ends cleanly");
    assert!(ctx.shutdown_requested);
}

/// R1-67 (Q40): a call that waits too long answers Busy; a job not
/// started by then never runs later; the state thread serves again.
#[test]
fn a_call_that_waits_too_long_is_busy_and_never_runs_late() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let published = Published::new(ShowData::default(), Default::default());
    let (_inbound_tx, inbound_rx) = crossbeam_channel::bounded(1);
    let (state, thread) = spawn(context(), inbound_rx, Vec::new(), published);
    // A long job holds the state thread.
    let slow = state.clone();
    let blocker = std::thread::spawn(move || slow.call(|_| std::thread::sleep(Duration::from_millis(600))));
    std::thread::sleep(Duration::from_millis(100));
    assert!(state.busy_for().is_some(), "busy while the long job runs");
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();
    let result = state.call_within(Duration::from_millis(100), move |_| flag.store(true, Ordering::Relaxed));
    assert!(matches!(result, Err(CallError::Busy { running: false })), "{result:?}");
    assert!(result.unwrap_err().to_string().contains("E-CMD-017"));
    blocker.join().unwrap().unwrap();
    assert_eq!(state.call(|_| 7).unwrap(), 7);
    assert!(!ran.load(Ordering::Relaxed), "the given-up job never ran");
    assert!(state.busy_for().is_none());
    state.call(|ctx| ctx.shutdown_requested = true).unwrap();
    thread.join().expect("state thread ends cleanly");
}

/// Decisions §31: a full machine lane refuses at once (E-CMD-017, counted
/// in Health), while the shell lane still queues, never drops, and goes first.
#[test]
fn a_full_machine_lane_refuses_and_the_shell_lane_still_runs() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let published = Published::new(ShowData::default(), Default::default());
    let (_inbound_tx, inbound_rx) = crossbeam_channel::bounded(1);
    let (state, thread) = spawn(context(), inbound_rx, Vec::new(), published);
    let machine = state.machine();
    assert_eq!(machine.lane(), Lane::Machine);
    // A long job holds the state thread while both lanes fill.
    let (go_tx, go_rx) = crossbeam_channel::bounded::<()>(0);
    assert!(state.send(move |_| {
        let _ = go_rx.recv();
    }));
    std::thread::sleep(Duration::from_millis(50));
    let order = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ran = Arc::new(AtomicUsize::new(0));
    for _ in 0..MACHINE_LANE {
        let (ran, order) = (ran.clone(), order.clone());
        assert!(machine.send(move |_| {
            ran.fetch_add(1, Ordering::Relaxed);
            order.lock().unwrap().push("machine");
        }));
    }
    let refused = machine.call_within(Duration::from_secs(5), |_| 1);
    assert!(matches!(refused, Err(CallError::Full)), "{refused:?}");
    assert!(refused.unwrap_err().to_string().contains("E-CMD-017"));
    assert!(!machine.send(|_| unreachable!("a refused job never runs")));
    // The shell lane never refuses, and runs before the waiting machine jobs.
    for _ in 0..MACHINE_LANE * 2 {
        let order = order.clone();
        assert!(state.send(move |_| order.lock().unwrap().push("shell")));
    }
    go_tx.send(()).unwrap();
    let health = state.call(|ctx| ctx.state.root.runtime.health.snapshot()).unwrap();
    let state_row = health.iter().find(|t| t.name == "State").unwrap();
    assert_eq!(state_row.refused, 2);
    // The machine jobs all ran; the shell's came first.
    assert_eq!(machine.call(|_| ()).ok(), Some(()));
    assert_eq!(ran.load(Ordering::Relaxed), MACHINE_LANE);
    let order = order.lock().unwrap();
    assert!(order[..MACHINE_LANE * 2].iter().all(|o| *o == "shell"), "the shell lane goes first");
    state.call(|ctx| ctx.shutdown_requested = true).unwrap();
    thread.join().expect("state thread ends cleanly");
}

/// A job that calls back into the state thread gets an error instead of
/// waiting on itself forever (R1-67). Debug builds also assert, which
/// the job's panic guard turns into a report.
#[test]
fn a_call_from_a_job_is_refused_not_a_deadlock() {
    let published = Published::new(ShowData::default(), Default::default());
    let (_inbound_tx, inbound_rx) = crossbeam_channel::bounded(1);
    let (state, thread) = spawn(context(), inbound_rx, Vec::new(), published);
    let inner = state.clone();
    let (done_tx, done_rx) = crossbeam_channel::bounded(1);
    let outer = state.clone();
    std::thread::spawn(move || {
        let _ = done_tx.send(outer.call(move |_ctx| match inner.call(|_| 1u32) {
            Err(CallError::Reentrant) => "refused",
            _ => "something else",
        }));
    });
    let result = done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("the call came back (no deadlock)");
    if cfg!(debug_assertions) {
        assert!(matches!(result, Err(CallError::Panicked(_))), "{result:?}");
    } else {
        assert!(matches!(result, Ok("refused")), "{result:?}");
    }
    assert_eq!(state.call(|_| 7).unwrap(), 7, "the state thread still serves");
    assert!(CallError::Reentrant.to_string().ends_with("[E-INTERNAL-001]"));
    state.call(|ctx| ctx.shutdown_requested = true).unwrap();
    thread.join().expect("state thread ends cleanly");
}
