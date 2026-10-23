// ██████╗  █████╗  ██████╗██╗  ██╗ ██████╗ ██████╗  ██████╗ ██╗   ██╗███╗   ██╗██████╗
// ██╔══██╗██╔══██╗██╔════╝██║ ██╔╝██╔════╝ ██╔══██╗██╔═══██╗██║   ██║████╗  ██║██╔══██╗
// ██████╔╝███████║██║     █████╔╝ ██║  ███╗██████╔╝██║   ██║██║   ██║██╔██╗ ██║██║  ██║
// ██╔══██╗██╔══██║██║     ██╔═██╗ ██║   ██║██╔══██╗██║   ██║██║   ██║██║╚██╗██║██║  ██║
// ██████╔╝██║  ██║╚██████╗██║  ██╗╚██████╔╝██║  ██║╚██████╔╝╚██████╔╝██║ ╚████║██████╔╝
// ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝╚═════╝
// FILE: background

//! Background work a command starts (R1-70, Q42): long disk work — hashing a
//! multi-GB asset — runs on a worker thread, never on the state thread, so
//! no command waits for it. The worker meets the same disk door as the line
//! that started it (`drive::Door`). When it is done it queues a follow-up
//! **command line**, run as the user who started it on the state thread
//! (`take`, drained by the server), so the result goes through the one
//! change path like any line; a failure is said in the System Monitor.
//!
//! Owns:          the one background worker for long disk work and its follow-up command lines.
//! Doesn't own:   running the follow-up (the state thread), the disk door (drive.rs).
//! Entry points:  spawn, take, wait_idle.
//! Decisions:     R1-70, Q42, §34 25.10.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A follow-up line to run as `user`.
pub struct Done {
    pub user: String,
    pub line: String,
}

static QUEUE: Mutex<Vec<Done>> = Mutex::new(Vec::new());

type Job = Box<dyn FnOnce() + Send>;

/// The one worker: jobs run one after another (hashing two multi-GB files at
/// once only makes both slower), never one thread per job.
static WORKER: Mutex<Option<std::sync::mpsc::Sender<Job>>> = Mutex::new(None);

/// Jobs queued or running.
static PENDING: AtomicUsize = AtomicUsize::new(0);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Run `work` on the worker; its `Ok(line)` runs as `user` afterwards, its
/// `Err` goes to the System Monitor (`what` names the job).
pub fn spawn(user: &str, what: &str, work: impl FnOnce() -> Result<String, String> + Send + 'static) {
    let user = user.to_string();
    let job = what.to_string();
    let door = crate::drive::door();
    let run: Job = Box::new(move || {
        crate::drive::enter(&door);
        match work() {
            Ok(line) => QUEUE.lock().unwrap_or_else(|e| e.into_inner()).push(Done { user, line }),
            Err(e) => lumacore_sysmon::sysmon_note("Background", format!("[ERROR] {job} ({user}): {e}")),
        }
    });
    PENDING.fetch_add(1, Ordering::SeqCst);
    let mut worker = WORKER.lock().unwrap_or_else(|e| e.into_inner());
    // The worker is started on the first job (and again if it ever ended).
    let run = match worker.as_ref() {
        Some(tx) => match tx.send(run) {
            Ok(()) => return,
            Err(std::sync::mpsc::SendError(run)) => run,
        },
        None => run,
    };
    let (tx, rx) = std::sync::mpsc::channel::<Job>();
    let started = std::thread::Builder::new().name("background".to_string()).spawn(move || {
        for job in rx {
            // A job that panics is reported by the panic hook; the worker
            // goes on with the next one.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job));
            PENDING.fetch_sub(1, Ordering::SeqCst);
        }
    });
    match started {
        Ok(_) => {
            let _ = tx.send(run);
            *worker = Some(tx);
        }
        Err(e) => {
            PENDING.fetch_sub(1, Ordering::SeqCst);
            lumacore_sysmon::sysmon_note("Background", format!("[ERROR] {what}: no worker thread ({e})"));
        }
    }
}

/// The follow-up lines that are ready (oldest first).
pub fn take() -> Vec<Done> {
    std::mem::take(&mut *QUEUE.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Wait until no job is queued or running (at most `limit`); whether it is
/// idle. The sweep waits before it replaces the files a job may be reading.
pub fn wait_idle(limit: std::time::Duration) -> bool {
    let until = std::time::Instant::now() + limit;
    while PENDING.load(Ordering::SeqCst) > 0 {
        if std::time::Instant::now() >= until {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    true
}
