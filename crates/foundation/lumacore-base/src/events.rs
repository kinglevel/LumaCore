//! TreeEvent enum and observer pattern for the reactive object tree.
//!
//! Any change to the tree emits a TreeEvent. Observers (closures) are registered
//! on Root and called synchronously on each emit.
//!
//! Foundation for Phase 3 Watch commands and Phase 4 WebSocket push.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Events emitted by the LumaCore object tree.
///
/// Events are emitted synchronously by Root::emit() after any tree mutation.
/// Phase 1 events cover the core patch and output operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreeEvent {
    /// A new fixture type was imported from a GDTF file.
    FixtureTypeImported {
        /// GUID of the newly created FixtureType.
        guid: Uuid,
    },
    /// A fixture was patched to a universe.
    FixturePatched {
        /// Integer ID of the patched fixture.
        fixture_id: u32,
        /// Integer ID of the target universe.
        universe_id: u32,
    },
    /// A fixture was unpatched (removed from all universes).
    FixtureUnpatched {
        /// Integer ID of the unpatched fixture.
        fixture_id: u32,
    },
    /// A new universe was created.
    UniverseCreated {
        /// Integer ID of the new universe.
        universe_id: u32,
    },
    /// A universe was assigned to an output line.
    OutputAssigned {
        /// Integer ID of the output line.
        output_id: u32,
        /// Integer ID of the universe assigned.
        universe_id: u32,
    },
    /// The fixture selection changed for a user.
    SelectionChanged {
        /// Integer IDs of the now-selected fixtures.
        fixture_ids: Vec<u32>,
    },
    /// A group was stored to a datapool.
    GroupStored { group_id: u32, datapool_id: u32 },
    /// A preset was stored to a pool within a datapool.
    PresetStored {
        preset_id: u32,
        pool_id: u32,
        datapool_id: u32,
    },
    /// A new sequence was created.
    SequenceCreated { sequence_id: u32 },
    /// A cue was stored into a sequence.
    CueStored { sequence_id: u32, cue_id: u32 },
    /// A macro was stored to a datapool.
    MacroStored { macro_id: u32, datapool_id: u32 },
    /// A playback (sequence executor) started running.
    PlaybackStarted { sequence_id: u32 },
    /// A playback (sequence executor) stopped.
    PlaybackStopped { sequence_id: u32 },
    /// This station joined a remote session.
    SessionJoined { session_id: String },
    /// This station left a remote session.
    SessionLeft { session_id: String },
    /// A new datapool was created.
    DatapoolCreated { datapool_id: u32 },
    /// A runtime variable changed value.
    VariableChanged { scope: String, name: String },
}

/// An observer function that receives tree events.
///
/// Stored in Root as `Box<dyn Fn(&TreeEvent) + Send + Sync>`.
pub type TreeObserver = Box<dyn Fn(&TreeEvent) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn tree_event_is_serializable() {
        let event = TreeEvent::FixturePatched {
            fixture_id: 1,
            universe_id: 2,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("FixturePatched"));
    }

    #[test]
    fn observer_called_with_event() {
        let received = Arc::new(Mutex::new(Vec::<TreeEvent>::new()));
        let received_clone = received.clone();

        let observer: TreeObserver = Box::new(move |event| {
            received_clone.lock().unwrap().push(event.clone());
        });

        let event = TreeEvent::UniverseCreated { universe_id: 1 };
        observer(&event);

        let events = received.lock().unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            TreeEvent::UniverseCreated { universe_id } => assert_eq!(*universe_id, 1),
            _ => panic!("Wrong event type"),
        }
    }
}
