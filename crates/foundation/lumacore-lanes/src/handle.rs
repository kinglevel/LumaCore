// ██╗  ██╗ █████╗ ███╗   ██╗██████╗ ██╗     ███████╗
// ██║  ██║██╔══██╗████╗  ██║██╔══██╗██║     ██╔════╝
// ███████║███████║██╔██╗ ██║██║  ██║██║     █████╗
// ██╔══██║██╔══██║██║╚██╗██║██║  ██║██║     ██╔══╝
// ██║  ██║██║  ██║██║ ╚████║██████╔╝███████╗███████╗
// ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚══════╝
// FILE: handle

//! The way to a state thread (decisions §26 "one state thread", §31 "two lanes"): every other
//! thread reaches the context only through a [`StateHandle`], whose jobs wait in one of two
//! lanes — the shell lane (unbounded, never dropped, served first) or the machine lane (bounded at
//! [`MACHINE_LANE`], refused at once when full). `call` waits for the result, `call_within` at
//! most a while (a job not started by then never runs), `send` doesn't wait; a call from the
//! state thread itself is refused instead of waiting on itself. Generic over the context: the
//! station's server is one [`Host`].
//!
//! Owns:          StateHandle, Lane, Lanes (the receiving end), Busy (the busy clock), CallError, Job, Host, MACHINE_LANE, enter_state_thread.
//! Doesn't own:   the loop that runs the jobs (the server's state thread), how long a caller waits (its policy), the panic report (the host's guard).
//! Entry points:  lanes, StateHandle::{call, call_within, send, machine, shell, busy_for, on_busy_change}.
//! Decisions:     §26, §31 "State job queue: two lanes", R1-67, Q40, §34 24.2.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a state thread's context does around a job: run it under its panic guard, and note a
/// panic (its health entry). The station's `ServerContext` is the one host.
pub trait Host: 'static {
    /// What a guarded job's panic leaves behind (the station's panic report).
    type Panic: std::fmt::Debug + std::fmt::Display + Send + 'static;
    /// Run `f`, turning a panic into its report.
    fn guarded<R>(f: impl FnOnce() -> R) -> Result<R, Self::Panic>;
    /// A job panicked on this context.
    fn panicked(&mut self);
}

/// Work for the state thread.
pub type Job<C> = Box<dyn FnOnce(&mut C) + Send>;

/// A job could not deliver its result.
#[derive(Debug)]
pub enum CallError<P> {
    /// The state thread has stopped (shutdown).
    Stopped,
    /// The job panicked; its show changes were rolled back by dispatch.
    Panicked(P),
    /// `call` from the state thread itself (a job), which would wait on itself forever (R1-67):
    /// refused instead.
    Reentrant,
    /// The state thread didn't get to the job in time (R1-67, Q40). `running`: it had started
    /// and is still running; otherwise it was dropped unrun.
    Busy { running: bool },
    /// The machine lane was full (decisions §31): refused at once, never run.
    Full,
}

/// Which queue a handle's jobs wait in (decisions §31 "State job queue: two lanes").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lane {
    /// Unbounded, never dropped: a shell's keys, lines, taps and answers, a logout's revoke, and
    /// the station's own work (the LumaSession stream, elections, signals) whose order matters
    /// (§26). A person typing can't outrun the state thread.
    Shell,
    /// Bounded at [`MACHINE_LANE`]: machine senders (HTTP, feeds, the UDP command port,
    /// discovery). A full lane refuses at once — `Full` (E-CMD-017 / HTTP 503) — and counts in
    /// `refused` (Health). OSC has its own bounded channel.
    Machine,
}

/// How many machine jobs may wait (decisions §31).
pub const MACHINE_LANE: usize = 256;

/// When the state thread's current work started (ms on the local clock), 0 when idle: the
/// watchdog's and `busy_for`'s view (R1-67, Q40).
#[derive(Clone, Default)]
pub struct Busy(Arc<AtomicU64>);

type BusyWatchers = Arc<std::sync::Mutex<Vec<Box<dyn Fn(bool) + Send + Sync>>>>;

/// A way to reach the state thread. Cheap to clone; one per thread.
pub struct StateHandle<C> {
    shell: Sender<Job<C>>,
    machine: Sender<Job<C>>,
    lane: Lane,
    /// Counts a machine job turned away (the state thread's Health `refused`).
    on_refused: Arc<dyn Fn() + Send + Sync>,
    pub(crate) busy: Busy,
    /// Told when a busy episode starts and ends (the watchdog's view).
    pub(crate) busy_watchers: BusyWatchers,
}

/// The state thread's end of the two lanes, and its busy clock.
pub struct Lanes<C> {
    pub shell: Receiver<Job<C>>,
    pub machine: Receiver<Job<C>>,
    pub busy: Busy,
}

thread_local! {
    /// Set on the state thread ([`enter_state_thread`]).
    static ON_STATE_THREAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The two lanes of a new state thread: the handle every other thread clones, and the end the
/// loop receives from. `on_refused` counts a machine job turned away.
pub fn lanes<C>(on_refused: impl Fn() + Send + Sync + 'static) -> (StateHandle<C>, Lanes<C>) {
    let (shell_tx, shell_rx) = crossbeam_channel::unbounded::<Job<C>>();
    let (machine_tx, machine_rx) = crossbeam_channel::bounded::<Job<C>>(MACHINE_LANE);
    let busy = Busy::default();
    let handle = StateHandle {
        shell: shell_tx,
        machine: machine_tx,
        lane: Lane::Shell,
        on_refused: Arc::new(on_refused),
        busy: busy.clone(),
        busy_watchers: BusyWatchers::default(),
    };
    (handle, Lanes { shell: shell_rx, machine: machine_rx, busy })
}

/// Mark this thread as the state thread: a `call` from it is refused, not a deadlock.
pub fn enter_state_thread() {
    ON_STATE_THREAD.with(|on| on.set(true));
}

/// Ms on the local clock (never 0 for a start).
fn now_ms() -> u64 {
    lumacore_base::clock::local_us() / 1000
}

impl Busy {
    /// Work starts now.
    pub fn start(&self) {
        self.0.store(now_ms().max(1), Ordering::Relaxed);
    }

    /// Work starts now unless it already did (the rest of a loop turn).
    pub fn start_if_idle(&self) {
        if self.0.load(Ordering::Relaxed) == 0 {
            self.start();
        }
    }

    /// Idle again.
    pub fn end(&self) {
        self.0.store(0, Ordering::Relaxed);
    }

    /// For how many ms the current work has run (None: idle).
    pub fn for_ms(&self) -> Option<u64> {
        let since = self.0.load(Ordering::Relaxed);
        (since != 0).then(|| now_ms().saturating_sub(since))
    }
}

impl<C> Clone for StateHandle<C> {
    fn clone(&self) -> Self {
        Self {
            shell: self.shell.clone(),
            machine: self.machine.clone(),
            lane: self.lane,
            on_refused: self.on_refused.clone(),
            busy: self.busy.clone(),
            busy_watchers: self.busy_watchers.clone(),
        }
    }
}

impl<C: Host> StateHandle<C> {
    /// This handle on the machine lane (bounded; refuses when full).
    pub fn machine(&self) -> Self {
        Self { lane: Lane::Machine, ..self.clone() }
    }

    /// This handle on the shell lane (unbounded; never refuses).
    pub fn shell(&self) -> Self {
        Self { lane: Lane::Shell, ..self.clone() }
    }

    pub fn lane(&self) -> Lane {
        self.lane
    }

    /// Queue a job on this handle's lane. `Full` when the machine lane is full.
    fn queue(&self, job: Job<C>) -> Result<(), CallError<C::Panic>> {
        match self.lane {
            Lane::Shell => self.shell.send(job).map_err(|_| CallError::Stopped),
            Lane::Machine => self.machine.try_send(job).map_err(|e| match e {
                crossbeam_channel::TrySendError::Full(_) => {
                    (self.on_refused)();
                    CallError::Full
                }
                crossbeam_channel::TrySendError::Disconnected(_) => CallError::Stopped,
            }),
        }
    }

    /// Refused on the state thread itself (it would wait on itself).
    fn reentrant() -> Result<(), CallError<C::Panic>> {
        if ON_STATE_THREAD.with(std::cell::Cell::get) {
            debug_assert!(false, "StateHandle::call from the state thread would wait on itself");
            log::error!("{}", CallError::<C::Panic>::Reentrant);
            return Err(CallError::Reentrant);
        }
        Ok(())
    }

    /// Run `f` on the state thread and wait for its result. Never call this from a job (the
    /// state thread would wait on itself).
    pub fn call<R: Send + 'static>(
        &self,
        f: impl FnOnce(&mut C) -> R + Send + 'static,
    ) -> Result<R, CallError<C::Panic>> {
        Self::reentrant()?;
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.queue(Box::new(move |ctx| {
            let result = C::guarded(|| f(ctx));
            if result.is_err() {
                ctx.panicked();
            }
            let _ = tx.send(result);
        }))?;
        match rx.recv() {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(report)) => Err(CallError::Panicked(report)),
            Err(_) => Err(CallError::Stopped),
        }
    }

    /// [`Self::call`] that waits at most `wait` (R1-67, Q40). A job not started by then is
    /// dropped and never runs (`Busy { running: false }`); one already running finishes on its
    /// own (`running: true`).
    pub fn call_within<R: Send + 'static>(
        &self,
        wait: Duration,
        f: impl FnOnce(&mut C) -> R + Send + 'static,
    ) -> Result<R, CallError<C::Panic>> {
        Self::reentrant()?;
        // 0 queued, 1 started, 2 given up.
        let phase = Arc::new(AtomicU8::new(0));
        let job_phase = phase.clone();
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.queue(Box::new(move |ctx| {
            if job_phase.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
                return; // the caller gave up: never run late
            }
            let result = C::guarded(|| f(ctx));
            if result.is_err() {
                ctx.panicked();
            }
            let _ = tx.send(result);
        }))?;
        match rx.recv_timeout(wait) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(report)) => Err(CallError::Panicked(report)),
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => Err(CallError::Stopped),
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                let running = phase.compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire).is_err();
                Err(CallError::Busy { running })
            }
        }
    }

    /// How long the state thread has been on its current work (None: idle).
    pub fn busy_for(&self) -> Option<Duration> {
        self.busy.for_ms().map(Duration::from_millis)
    }

    /// Call `f(true)` when a busy episode starts (the watchdog's 5 s) and `f(false)` when the
    /// state thread answers again. Called off the state thread.
    pub fn on_busy_change(&self, f: impl Fn(bool) + Send + Sync + 'static) {
        self.busy_watchers.lock().unwrap_or_else(|e| e.into_inner()).push(Box::new(f));
    }

    /// Queue `f` without waiting. False when the state thread has stopped (or, on the machine
    /// lane, the lane is full: `f` never runs).
    pub fn send(&self, f: impl FnOnce(&mut C) + Send + 'static) -> bool {
        self.queue(Box::new(f)).is_ok()
    }
}

impl<P: std::fmt::Display> std::fmt::Display for CallError<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => f.write_str("the server is shutting down"),
            Self::Busy { running: false } => f.write_str(
                "The station is busy (a long job is running): your line didn't run. Try again. [E-CMD-017]",
            ),
            Self::Busy { running: true } => f.write_str(
                "The station is busy: your line started and is still running; its result will show in the System Monitor. [E-CMD-017]",
            ),
            Self::Full => f.write_str("The station is busy: too many requests are waiting for it. Try again. [E-CMD-017]"),
            Self::Reentrant => f.write_str(
                "Internal error — a state-thread job waited on the state thread (StateHandle::call from a job) [E-INTERNAL-001]",
            ),
            Self::Panicked(report) => write!(f, "{report}"),
        }
    }
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
