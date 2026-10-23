// ███████╗ ██████╗ ██████╗ ███╗   ███╗ █████╗ ████████╗
// ██╔════╝██╔═══██╗██╔══██╗████╗ ████║██╔══██╗╚══██╔══╝
// █████╗  ██║   ██║██████╔╝██╔████╔██║███████║   ██║
// ██╔══╝  ██║   ██║██╔══██╗██║╚██╔╝██║██╔══██║   ██║
// ██║     ╚██████╔╝██║  ██║██║ ╚═╝ ██║██║  ██║   ██║
// ╚═╝      ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝
// FILE: format

//! Chrome Trace Event JSON serialization for Perfetto.
//!
//! Owns:          event_to_json
//! Doesn't own:   the events (collector)
//! Entry points:  event_to_json
//! Decisions:     §23 "Step 8"

use serde_json::{Value, json};

use crate::collector::TraceEvent;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Convert a TraceEvent to Chrome Trace Event JSON format.
pub fn event_to_json(event: &TraceEvent) -> Value {
    let mut obj = json!({
        "name": event.name,
        "cat": event.category,
        "ph": event.phase.to_string(),
        "ts": event.timestamp_us,
        "pid": event.pid,
        "tid": event.tid,
    });

    // Add duration for complete events
    if event.phase == 'X' {
        obj["dur"] = json!(event.duration_us);
    }

    obj
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
