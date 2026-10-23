// ███████╗██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// █████╗  ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗╚════██║
// ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║███████║
// ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: errors

//! Error types for the DMX protocols (lumacore-protocols)
//!
//! Owns:          DmxError and the E-DMX catalog.
//! Doesn't own:   the one catalog (lumacore-registry), how a code is rendered (lumacore-state errors).
//! Entry points:  DmxError, ERRORS.
//! Decisions:     §34 21.

use thiserror::Error;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Errors produced by the DMX output engine.
#[derive(Debug, Error)]
pub enum DmxError {
    /// Output timing drifted beyond acceptable tolerance.
    #[error("timing drift: expected {expected_us}µs, actual {actual_us}µs")]
    TimingDrift { expected_us: u64, actual_us: u64 },

    /// A socket-level error occurred during sACN transmission.
    #[error("socket error: {0}")]
    SocketError(String),

    /// The output agent thread terminated unexpectedly.
    #[error("output agent failed: {0}")]
    OutputAgentFailed(String),

    /// IO error wrapper.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl lumacore_base::Coded for DmxError {
    fn code(&self) -> &'static str {
        match self {
            Self::TimingDrift { .. } => "E-DMX-001",
            Self::SocketError(_) => "E-DMX-002",
            Self::OutputAgentFailed(_) => "E-DMX-003",
            Self::Io(_) => "E-DMX-004",
        }
    }
}

/// Error catalog of the DMX output (`Info Error <code>`).
pub const ERRORS: &[lumacore_base::ErrorInfo] = &[
    lumacore_base::ErrorInfo {
        code: "E-DMX-001",
        title: "Output timing drift",
        explain: "A DMX frame went out later than the 60 Hz render schedule allows. Check CPU load and Runtime.Health.",
    },
    lumacore_base::ErrorInfo {
        code: "E-DMX-002",
        title: "Output socket error",
        explain: "Sending a DMX packet failed at the network socket. Check the interface and network settings.",
    },
    lumacore_base::ErrorInfo {
        code: "E-DMX-003",
        title: "Output agent stopped",
        explain: "The DMX output thread ended unexpectedly; the log has the cause.",
    },
    lumacore_base::ErrorInfo {
        code: "E-DMX-004",
        title: "Output I/O error",
        explain: "An operating-system I/O error in the DMX output path.",
    },
];

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
