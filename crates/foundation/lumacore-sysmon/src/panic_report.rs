// ██████╗  █████╗ ███╗   ██╗██╗ ██████╗    ██████╗ ███████╗██████╗  ██████╗ ██████╗ ████████╗
// ██╔══██╗██╔══██╗████╗  ██║██║██╔════╝    ██╔══██╗██╔════╝██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝
// ██████╔╝███████║██╔██╗ ██║██║██║         ██████╔╝█████╗  ██████╔╝██║   ██║██████╔╝   ██║
// ██╔═══╝ ██╔══██║██║╚██╗██║██║██║         ██╔══██╗██╔══╝  ██╔═══╝ ██║   ██║██╔══██╗   ██║
// ██║     ██║  ██║██║ ╚████║██║╚██████╗    ██║  ██║███████╗██║     ╚██████╔╝██║  ██║   ██║
// ╚═╝     ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝ ╚═════╝    ╚═╝  ╚═╝╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝
// FILE: panic_report

//! Internal-error reports: a panic anywhere gets a report number, and the log gets its location,
//! message, thread and backtrace under it. Work inside guarded keeps running after a panic and gets
//! the report back, so it can roll back and tell the user where it broke.
//!
//! Owns:          panic reports: the hook, report numbers, guarded, the report's one line.
//! Doesn't own:   rolling the show back (dispatch), the reply's E-INTERNAL-001 rendering (lumacore-state errors).
//! Entry points:  install, guarded, PanicReport.
//! Decisions:     §26, §34 25.10.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Once;
use std::sync::atomic::{AtomicU64, Ordering};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a panic left behind.
#[derive(Debug, Clone)]
pub struct PanicReport {
    pub number: u64,
    pub file: String,
    pub line: u32,
    pub message: String,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// How a panic reads on the command line (`CallError::Panicked`, decisions §29).
impl std::fmt::Display for PanicReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Internal error in {}:{} — {} · report #{} [E-INTERNAL-001]", self.file, self.line, self.message, self.number)
    }
}

static NEXT_REPORT: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static LAST: RefCell<Option<PanicReport>> = const { RefCell::new(None) };
    static GUARDED: Cell<u32> = const { Cell::new(0) };
}

/// Install the panic hook (once per process). Guarded panics are logged here
/// instead of printed by the default hook; unguarded ones (a thread that
/// dies) are logged and then go to the default hook as before.
pub fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let number = NEXT_REPORT.fetch_add(1, Ordering::Relaxed);
            let (file, line) = info
                .location()
                .map(|l| (l.file().to_string(), l.line()))
                .unwrap_or_else(|| ("<unknown>".to_string(), 0));
            let payload = info.payload();
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panic".to_string());
            let thread = std::thread::current()
                .name()
                .unwrap_or("<unnamed>")
                .to_string();
            let backtrace = std::backtrace::Backtrace::force_capture();
            log::error!(
                "Internal error report #{number} in {file}:{line} on thread '{thread}': \
                 {message}\n{backtrace}"
            );
            crate::sysmon_early(
                "Error",
                format!("Internal error report #{number} in {file}:{line}: {message}"),
            );
            let guarded = GUARDED.with(|g| g.get() > 0);
            LAST.with(|last| {
                *last.borrow_mut() = Some(PanicReport {
                    number,
                    file,
                    line,
                    message,
                })
            });
            if !guarded {
                previous(info);
            }
        }));
    });
}

/// Run `f`; a panic inside it comes back as its report instead of unwinding
/// further.
pub fn guarded<R>(f: impl FnOnce() -> R) -> Result<R, PanicReport> {
    install();
    GUARDED.with(|g| g.set(g.get() + 1));
    let result = catch_unwind(AssertUnwindSafe(f));
    GUARDED.with(|g| g.set(g.get() - 1));
    result.map_err(|_| {
        LAST.with(|last| last.borrow_mut().take())
            .unwrap_or_else(|| PanicReport {
                number: NEXT_REPORT.fetch_add(1, Ordering::Relaxed),
                file: "<unknown>".to_string(),
                line: 0,
                message: "panic".to_string(),
            })
    })
}

impl PanicReport {
    /// The one line every user sees, without its code: the caller renders it with E-INTERNAL-001
    /// (`errors::render_code`, decisions §34 25.10).
    pub fn text(&self, command: &str, user: &str, build: &str) -> String {
        format!(
            "Internal error in {}:{} — {} · command '{}' · user {} · build {} · report #{}",
            self.file, self.line, self.message, command, user, build, self.number
        )
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
