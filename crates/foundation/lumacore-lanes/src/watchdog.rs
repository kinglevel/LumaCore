// ██╗    ██╗ █████╗ ████████╗ ██████╗██╗  ██╗██████╗  ██████╗  ██████╗
// ██║    ██║██╔══██╗╚══██╔══╝██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝
// ██║ █╗ ██║███████║   ██║   ██║     ███████║██║  ██║██║   ██║██║  ███╗
// ██║███╗██║██╔══██║   ██║   ██║     ██╔══██║██║  ██║██║   ██║██║   ██║
// ╚███╔███╔╝██║  ██║   ██║   ╚██████╗██║  ██║██████╔╝╚██████╔╝╚██████╔╝
//  ╚══╝╚══╝ ╚═╝  ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝╚═════╝  ╚═════╝  ╚═════╝
// FILE: watchdog

//! The state thread's watchdog (R1-67, Q40): busy on one piece of work for more than 5 s → one
//! warning per episode, and every `on_busy_change` watcher told; told again when it answers.
//!
//! Owns:          spawn_watchdog, WATCHDOG_AFTER_MS.
//! Doesn't own:   where the warning goes (the caller's warn: the System Monitor), the busy clock (handle::Busy).
//! Entry points:  spawn_watchdog.
//! Decisions:     R1-67, Q40, §30 "Merge of bugfixes", §34 24.2.

use std::time::Duration;

use crate::handle::StateHandle;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Busy this long: the watchdog warns (once per episode).
pub const WATCHDOG_AFTER_MS: u64 = 5_000;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Watch `handle`'s state thread until `alive` is gone (the thread ended). `warn` takes each
/// line the watchdog says.
pub fn spawn_watchdog<C>(handle: &StateHandle<C>, alive: std::sync::Weak<()>) {
    let busy = handle.busy.clone();
    let watchers = handle.busy_watchers.clone();
    let tell = move |state: bool| {
        for f in watchers.lock().unwrap_or_else(|e| e.into_inner()).iter() {
            f(state);
        }
    };
    let _ = std::thread::Builder::new().name("state_watchdog".to_string()).spawn(move || {
        let mut warned = false;
        while alive.upgrade().is_some() {
            std::thread::sleep(Duration::from_millis(500));
            match busy.for_ms() {
                Some(for_ms) if for_ms > WATCHDOG_AFTER_MS => {
                    if !warned {
                        warned = true;
                        lumacore_sysmon::sysmon_early("State", format!(
                            "[WARN] the state thread has been busy for {} s: commands wait (HTTP gives up after 10 s, shells after 30 s)",
                            for_ms / 1000
                        ));
                        tell(true);
                    }
                }
                None if warned => {
                    warned = false;
                    lumacore_sysmon::sysmon_early("State", "the state thread answers again");
                    tell(false);
                }
                _ => {}
            }
        }
    });
}
