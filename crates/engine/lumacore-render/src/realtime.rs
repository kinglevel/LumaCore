// ██████╗ ███████╗ █████╗ ██╗     ████████╗██╗███╗   ███╗███████╗
// ██╔══██╗██╔════╝██╔══██╗██║     ╚══██╔══╝██║████╗ ████║██╔════╝
// ██████╔╝█████╗  ███████║██║        ██║   ██║██╔████╔██║█████╗
// ██╔══██╗██╔══╝  ██╔══██║██║        ██║   ██║██║╚██╔╝██║██╔══╝
// ██║  ██║███████╗██║  ██║███████╗   ██║   ██║██║ ╚═╝ ██║███████╗
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝
// FILE: realtime

//! Real-time scheduling for the render thread and the lines' send threads (decisions §24,
//! §31 "Thread priority").
//!
//! On Linux a thread asks for `SCHED_FIFO` at a modest priority — below audio and IRQ threads
//! (JACK, PipeWire and the kernel's threaded IRQs sit at 80–99). That needs `RLIMIT_RTPRIO`
//! (`scripts/unix/realtime.conf` grants it) or `CAP_SYS_NICE`; without either the thread keeps
//! normal scheduling and says why. Elsewhere the thread asks for the platform's highest normal
//! priority. Either way the result is a line for `Info Health Threads`, never only a log line.
//!
//! Owns:          promote_current and the priorities of the render and send threads.
//! Doesn't own:   what runs on those threads (engine, lumacore_lines::lines).
//! Entry points:  promote_current, ENGINE_PRIORITY, DRIVER_PRIORITY.
//! Decisions:     §24, §31 "Thread priority".

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The render engine's `SCHED_FIFO` priority.
pub const ENGINE_PRIORITY: u8 = 70;
/// The lines' send / receive threads: just below the engine they take frames from.
pub const DRIVER_PRIORITY: u8 = 65;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Ask for real-time scheduling for the calling thread; returns what it got, for Health
/// (`SCHED_FIFO 70`, or `normal — …why…`). Call it once when the thread starts, never in a
/// render loop (it may allocate and read a file).
pub fn promote_current(priority: u8) -> String {
    imp::promote_current(priority)
}

#[cfg(target_os = "linux")]
mod imp {
    use thread_priority::unix::{
        RealtimeThreadSchedulePolicy, ThreadSchedulePolicy, set_thread_priority_and_policy,
        thread_native_id, thread_schedule_policy_param,
    };
    use thread_priority::{ThreadPriority, ThreadPriorityValue};

    pub fn promote_current(priority: u8) -> String {
        let Ok(value) = ThreadPriorityValue::try_from(priority) else {
            return format!("normal — priority {priority} is out of range");
        };
        let id = thread_native_id();
        let fifo = ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo);
        match set_thread_priority_and_policy(id, ThreadPriority::Crossplatform(value), fifo) {
            Ok(()) => match thread_schedule_policy_param(id) {
                Ok((ThreadSchedulePolicy::Realtime(_), params)) => {
                    format!("SCHED_FIFO {}", params.sched_priority)
                }
                Ok((policy, _)) => format!("normal — asked for SCHED_FIFO {priority}, got {policy:?}"),
                Err(err) => format!("SCHED_FIFO {priority} (not read back: {err:?})"),
            },
            Err(err) => format!(
                "normal — SCHED_FIFO {priority} refused ({err:?}; max real-time priority {}); \
                 install scripts/unix/realtime.conf and log in again",
                rt_limit().unwrap_or_else(|| "unknown".into())
            ),
        }
    }

    /// The process's `RLIMIT_RTPRIO`, from `/proc/self/limits` ("Max realtime priority").
    fn rt_limit() -> Option<String> {
        let limits = std::fs::read_to_string("/proc/self/limits").ok()?;
        limits
            .lines()
            .find(|l| l.starts_with("Max realtime priority"))?
            .split_whitespace()
            .nth(3)
            .map(str::to_string)
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use thread_priority::{ThreadPriority, set_current_thread_priority};

    pub fn promote_current(_priority: u8) -> String {
        match set_current_thread_priority(ThreadPriority::Max) {
            Ok(()) => "highest normal priority".to_string(),
            Err(err) => format!("normal — highest priority refused ({err:?})"),
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
