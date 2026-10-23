//! Fixture type library index — flat JSON file cache.
//!
//! Implements the "Structured index + original .gdtf" storage strategy from the
//! locked decisions. The index is a JSON file for Phase 1 (MessagePack migration
//! deferred if performance becomes a concern).
//!
//! The index stores parsed `FixtureType` objects for fast library browsing without
//! re-parsing the ZIP archives each time. Original .gdtf files remain as source of truth.

use crate::errors::GdtfError;
use lumacore_fixtures::FixtureType;
use std::path::Path;

/// Save a slice of `FixtureType`s as a JSON index file.
///
/// The file is written atomically by serializing to a string first, then
/// writing the string to disk. On error, the file may not exist or may be
/// partially written.
pub fn save_index(types: &[FixtureType], path: &Path) -> Result<(), GdtfError> {
    let json = serde_json::to_string_pretty(types)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load `FixtureType`s from a JSON index file.
///
/// Returns an error if the file does not exist, cannot be read, or contains
/// invalid JSON that doesn't match the `FixtureType` schema.
pub fn load_index(path: &Path) -> Result<Vec<FixtureType>, GdtfError> {
    let json = std::fs::read_to_string(path)?;
    let types: Vec<FixtureType> = serde_json::from_str(&json)?;
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumacore_base::{TreeNode};
    use lumacore_fixtures::{FixtureCategory, FixtureType};

    fn make_fixture_type(name: &str, manufacturer: &str) -> FixtureType {
        FixtureType::new(
            manufacturer.to_string(),
            Some(name.to_string()),
            FixtureCategory::Wash,
        )
    }

    #[test]
    fn round_trip_empty_index() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let path = dir.path().join("index.json");

        let types: Vec<FixtureType> = vec![];
        save_index(&types, &path).expect("save should succeed");
        let loaded = load_index(&path).expect("load should succeed");
        assert!(loaded.is_empty());
    }

    #[test]
    fn round_trip_with_fixture_types() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let path = dir.path().join("index.json");

        let types = vec![
            make_fixture_type("Mac Aura XB", "Robe"),
            make_fixture_type("Spiider", "Robe"),
        ];

        save_index(&types, &path).expect("save should succeed");
        let loaded = load_index(&path).expect("load should succeed");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].manufacturer, "Robe");
        assert_eq!(loaded[0].name(), Some("Mac Aura XB"));
        assert_eq!(loaded[1].name(), Some("Spiider"));
    }

    #[test]
    fn load_nonexistent_file_returns_error() {
        let path = Path::new("/tmp/lumacore_nonexistent_index_12345.json");
        assert!(load_index(path).is_err());
    }

    #[test]
    fn load_invalid_json_returns_error() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not valid json {{}}").unwrap();
        assert!(load_index(&path).is_err());
    }
}
