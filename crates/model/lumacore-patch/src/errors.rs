//! PatchError: error types for all patch engine operations.
//!
//! Used by both UniverseAddressMap (conflict/boundary detection)
//! and PatchEngine (fixture lookup, patch, unpatch operations).

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during patch operations.
#[derive(Debug, Error)]
pub enum PatchError {
    /// A DMX address slot is already occupied by another fixture.
    #[error("Conflict at slot {slot}: occupied by fixture {occupant_guid}")]
    Conflict {
        /// The 1-indexed DMX slot that conflicted.
        slot: u16,
        /// GUID of the fixture that already owns this slot.
        occupant_guid: Uuid,
        /// Optional display name of the occupying fixture.
        occupant_name: Option<String>,
    },

    /// DMX addresses are 1-512; 0 is not an address.
    #[error("Invalid DMX address {address}: addresses are 1-512")]
    InvalidAddress { address: u16 },

    /// The fixture footprint would exceed DMX boundary (address 512).
    #[error(
        "Footprint exceeds boundary: need {needed} channels but only {available} available from this address; try address {suggestion}"
    )]
    ExceedsBoundary {
        /// Number of channels needed.
        needed: u16,
        /// Number of channels available from the requested start address.
        available: u16,
        /// Last valid start address for this footprint (512 - count + 1).
        suggestion: u16,
    },

    /// A universe with the given ID does not exist in ShowData.
    #[error("Universe {universe_id} not found")]
    UniverseNotFound { universe_id: u32 },

    /// A fixture type with the given GUID does not exist in ShowData.
    #[error("Fixture type {guid} not found")]
    FixtureTypeNotFound { guid: Uuid },

    /// A fixture with the given ID does not exist in ShowData.
    #[error("Fixture {fixture_id} not found")]
    FixtureNotFound { fixture_id: u32 },

    /// The fixture is already patched (future: prevent double-patch).
    #[error("Fixture {fixture_id} is already patched")]
    AlreadyPatched { fixture_id: u32 },

    /// No fixture number is left above the highest one in use.
    #[error("No fixture number is left above the highest one (a fixture at the top of the range): give the new fixture a number, or move that fixture down")]
    NoFreeNumber,
}

impl lumacore_tree::Coded for PatchError {
    fn code(&self) -> &'static str {
        match self {
            Self::Conflict { .. } => "E-PATCH-001",
            Self::InvalidAddress { .. } => "E-PATCH-002",
            Self::ExceedsBoundary { .. } => "E-PATCH-003",
            Self::UniverseNotFound { .. } => "E-PATCH-004",
            Self::FixtureTypeNotFound { .. } => "E-PATCH-005",
            Self::FixtureNotFound { .. } => "E-PATCH-006",
            Self::AlreadyPatched { .. } => "E-PATCH-007",
            Self::NoFreeNumber => "E-PATCH-009",
        }
    }
}

/// Error catalog of the patch engine (`Info Error <code>`).
pub const ERRORS: &[lumacore_tree::ErrorInfo] = &[
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-001",
        title: "Address conflict",
        explain: "A DMX slot the fixture needs is already used by another fixture. Patch to a free address or unpatch the other fixture first.",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-002",
        title: "Invalid DMX address",
        explain: "DMX addresses are 1-512; 0 and anything above 512 are not addresses.",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-003",
        title: "Footprint exceeds universe",
        explain: "The fixture needs more channels than are left from this address to 512. The message suggests the last start address that fits.",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-004",
        title: "Universe not found",
        explain: "The universe does not exist in the show. Create it first (Store Universe <n>).",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-005",
        title: "Fixture type not found",
        explain: "No fixture type with that identity is in the show. Import the fixture type first.",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-006",
        title: "Fixture not found",
        explain: "No fixture with that ID exists in the patch.",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-007",
        title: "Fixture already patched",
        explain: "The fixture already has an address; unpatch or move it instead of patching it again.",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-008",
        title: "Patch draft open",
        explain: "Another user has Patch claimed and is editing a draft of it. Patch edits wait until they keep or discard the draft (they leave the patch).",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-009",
        title: "No fixture number left",
        explain: "New fixtures are numbered above the highest one in use; a fixture at 4294967295 leaves none. Give the new fixture a number or move that fixture down.",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-010",
        title: "Fixture type name shared",
        explain: "Patch names a fixture type by name, and two or more types have that name. Rename one with Label FixtureType <n> \"new name\" (List FixtureType shows the numbers).",
    },
    lumacore_tree::ErrorInfo {
        code: "E-PATCH-011",
        title: "Not a place in the MVR tree",
        explain: "An MVR group nests in a layer or in another group, never under itself or one of its own groups; a layer is the top of the tree and nests nowhere; a fixture goes in a layer or a group (Assign Fixture … At MvrGroup <n>).",
    },
];

#[cfg(test)]
mod tests {
    #[test]
    fn catalog_is_valid() {
        lumacore_tree::error_code::check_catalog(super::ERRORS).unwrap();
    }
}
