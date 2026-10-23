//! Inbound event types for integration engines.
//!
//! Any external system (OSC, MIDI, REST, timecode) can push events into
//! the LumaCore command path via a channel carrying `InboundEvent` values.
//! The server main loop drains the channel and dispatches events alongside
//! UDP commands and shell sessions.

use serde::{Deserialize, Serialize};

/// Transport control actions for playback/timecode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportAction {
    Go,
    Back,
    Stop,
    Goto(f64),
}

/// An inbound event from an integration engine.
///
/// Integration engines (OSC, MIDI, REST, timecode) produce these events
/// and send them over a `crossbeam_channel::Sender<InboundEvent>` stored
/// in `EngineContext`.  The server main loop receives and dispatches them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InboundEvent {
    /// A Lum command string to be parsed and executed (from OSC, REST, etc.).
    Command(String),

    /// A command from an input line, run as the line's user with that
    /// user's rights (decisions §11).
    UserCommand { user: String, command: String },

    /// Direct attribute control — bypasses command parsing.
    AttributeSet {
        fixture_id: u32,
        attribute: String,
        value: f64,
    },

    /// Playback transport control.
    TransportControl(TransportAction),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbound_event_command_roundtrip() {
        let event = InboundEvent::Command("fixture 1 at 80".to_string());
        let json = serde_json::to_string(&event).unwrap();
        let decoded: InboundEvent = serde_json::from_str(&json).unwrap();
        match decoded {
            InboundEvent::Command(s) => assert_eq!(s, "fixture 1 at 80"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn inbound_event_attribute_set() {
        let event = InboundEvent::AttributeSet {
            fixture_id: 42,
            attribute: "Dimmer".to_string(),
            value: 0.75,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("AttributeSet"));
        assert!(json.contains("Dimmer"));
    }

    #[test]
    fn inbound_event_transport_control() {
        let event = InboundEvent::TransportControl(TransportAction::Goto(12.5));
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Goto"));
    }
}
