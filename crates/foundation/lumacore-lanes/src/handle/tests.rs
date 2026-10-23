// ██╗  ██╗ █████╗ ███╗   ██╗██████╗ ██╗     ███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██║  ██║██╔══██╗████╗  ██║██╔══██╗██║     ██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ███████║███████║██╔██╗ ██║██║  ██║██║     █████╗         ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══██║██╔══██║██║╚██╗██║██║  ██║██║     ██╔══╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║  ██║██║  ██║██║ ╚████║██████╔╝███████╗███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: handle tests

//! Tests of `handle`.
//!
//! Owns:          the unit tests of handle.rs, with a small test host and loop.
//! Doesn't own:   the station's state thread (its own tests).
//! Entry points:  the #[test] fns.
//! Decisions:     §31, R1-67, §34 24.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

/// A context that counts its jobs and panics.
#[derive(Default)]
struct Counter {
    jobs: u32,
    panics: u32,
}

impl Host for Counter {
    type Panic = String;
    fn guarded<R>(f: impl FnOnce() -> R) -> Result<R, String> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).map_err(|e| {
            e.downcast_ref::<&str>().map(|s| s.to_string()).unwrap_or_else(|| "panic".into())
        })
    }
    fn panicked(&mut self) {
        self.panics += 1;
    }
}

/// A state thread serving the shell lane first, like the station's.
fn serve(lanes: Lanes<Counter>) -> std::thread::JoinHandle<Counter> {
    std::thread::spawn(move || {
        enter_state_thread();
        let mut ctx = Counter::default();
        loop {
            let job = match lanes.shell.try_recv() {
                Ok(job) => job,
                Err(_) => crossbeam_channel::select! {
                    recv(lanes.shell) -> j => match j { Ok(j) => j, Err(_) => break },
                    recv(lanes.machine) -> j => match j { Ok(j) => j, Err(_) => break },
                },
            };
            lanes.busy.start();
            job(&mut ctx);
            ctx.jobs += 1;
            lanes.busy.end();
        }
        ctx
    })
}

#[test]
fn a_call_runs_on_the_state_thread_and_a_panic_is_reported() {
    let (handle, lanes) = lanes::<Counter>(|| {});
    let thread = serve(lanes);
    assert_eq!(handle.call(|ctx| ctx.jobs + 41).unwrap(), 41);
    let err = handle.call(|_| -> u32 { panic!("job broke") }).unwrap_err();
    assert!(matches!(&err, CallError::Panicked(m) if m == "job broke"), "{err:?}");
    assert_eq!(handle.call(|ctx| ctx.panics).unwrap(), 1, "the host noted the panic");
    drop(handle);
    assert_eq!(thread.join().unwrap().jobs, 3);
}

#[test]
fn a_full_machine_lane_refuses_at_once_and_counts_it() {
    let refused = Arc::new(AtomicU64::new(0));
    let count = refused.clone();
    let (handle, lanes) = lanes::<Counter>(move || {
        count.fetch_add(1, Ordering::Relaxed);
    });
    // No loop yet: the machine lane fills up.
    let machine = handle.machine();
    for _ in 0..MACHINE_LANE {
        assert!(machine.send(|_| {}));
    }
    assert!(!machine.send(|_| {}));
    assert!(matches!(machine.call_within(Duration::from_millis(10), |_| ()), Err(CallError::Full)));
    assert_eq!(refused.load(Ordering::Relaxed), 2);
    // The shell lane never refuses.
    assert!(handle.send(|_| {}));
    drop(lanes);
}

#[test]
fn a_call_that_waits_too_long_never_runs_late() {
    let (handle, lanes) = lanes::<Counter>(|| {});
    // Nothing serves yet: the call gives up, and its job is dropped unrun.
    let err = handle.call_within(Duration::from_millis(20), |ctx| ctx.jobs += 100).unwrap_err();
    assert!(matches!(err, CallError::Busy { running: false }), "{err:?}");
    let thread = serve(lanes);
    assert_eq!(handle.call(|ctx| ctx.jobs).unwrap(), 1, "the late job ran but did nothing");
    drop(handle);
    thread.join().unwrap();
}

#[test]
fn a_call_from_the_state_thread_is_refused_not_a_deadlock() {
    let (handle, lanes) = lanes::<Counter>(|| {});
    let thread = serve(lanes);
    let inner = handle.clone();
    let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handle.call(move |_| matches!(inner.call(|_| ()), Err(CallError::Reentrant)))
    }));
    // Debug builds assert (the panic is the report); release builds refuse with Reentrant.
    match refused {
        Ok(Ok(was_refused)) => assert!(was_refused),
        Ok(Err(CallError::Panicked(m))) => assert!(m.contains("wait on itself"), "{m}"),
        other => panic!("{:?}", other.map(|r| r.map(|_| ()))),
    }
    drop(handle);
    thread.join().unwrap();
}
