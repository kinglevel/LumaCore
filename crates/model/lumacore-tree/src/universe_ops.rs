//! Universe lifecycle operations.
//!
//! `UniverseOps` is a stateless struct whose methods operate on `&mut ShowData`
//! and emit `TreeEvent`s for the caller. Routes to protocol lines are in
//! `output_ops` (decisions §11; universe ranges were dropped in step 4b).

use crate::events::TreeEvent;
use crate::tree::ShowData;
use lumacore_fixtures::universe::Universe;

/// Errors that can occur during universe operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UniverseError {
    /// A referenced universe ID does not exist in ShowData.
    #[error("universe {0} not found")]
    UniverseNotFound(u32),
}

impl crate::Coded for UniverseError {
    fn code(&self) -> &'static str {
        match self {
            Self::UniverseNotFound(_) => "E-UNIVERSE-001",
        }
    }
}

/// Error catalog of universe operations (`Info Error <code>`).
/// E-UNIVERSE-002 … 004 (ranges) are retired and never reused.
pub const UNIVERSE_ERRORS: &[crate::ErrorInfo] = &[crate::ErrorInfo {
    code: "E-UNIVERSE-001",
    title: "Universe not found",
    explain: "No universe with that ID exists in the show.",
}];

/// Stateless namespace for universe lifecycle operations.
pub struct UniverseOps;

impl UniverseOps {
    /// Create a new universe (next free ID) and insert it into ShowData.
    /// Returns the new universe ID; None when no number is left.
    pub fn create_universe(
        show: &mut ShowData,
        name: Option<String>,
        events: &mut Vec<TreeEvent>,
    ) -> Option<u32> {
        let id = show.devices.dmx.patch.universe_id_pool.allocate()?;
        show.devices
            .dmx
            .patch
            .universes
            .insert(id, Universe::new(name));
        events.push(TreeEvent::UniverseCreated { universe_id: id });
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_universe_allocates_ids() {
        let mut show = ShowData::new();
        let mut events = Vec::new();
        let a = UniverseOps::create_universe(&mut show, None, &mut events).unwrap();
        let b = UniverseOps::create_universe(&mut show, Some("B".into()), &mut events).unwrap();
        assert_eq!((a, b), (1, 2));
        assert_eq!(events.len(), 2);
    }
}
