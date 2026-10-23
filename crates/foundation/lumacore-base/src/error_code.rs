//! Stable error codes (`docs/meta/decisions.md` §26).
//!
//! Every typed error has a code `E-AREA-NNN`: the area is the crate or
//! subsystem, the number is never reused. The terminal shows the message with
//! its code (`Error: … [E-PATCH-002]`) and `Info Error <code>` explains it from
//! the catalog each crate publishes (`ERRORS`).

/// One catalog entry: a code and what it means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorInfo {
    /// `E-AREA-NNN`.
    pub code: &'static str,
    /// Short name of the error.
    pub title: &'static str,
    /// What it means and what to do about it.
    pub explain: &'static str,
}

/// An error that carries a stable code from its crate's catalog.
pub trait Coded: std::error::Error {
    fn code(&self) -> &'static str;
}

/// `message [E-AREA-NNN]` — how a coded error reads on the command line.
pub fn with_code(message: impl std::fmt::Display, code: &str) -> String {
    format!("{message} [{code}]")
}

/// Find a code in a set of catalogs (case-insensitive; `e-patch-2` and
/// `E-PATCH-002` both match).
pub fn find(catalogs: &[&'static [ErrorInfo]], code: &str) -> Option<&'static ErrorInfo> {
    let wanted = normalize(code)?;
    catalogs
        .iter()
        .flat_map(|c| c.iter())
        .find(|info| normalize(info.code).as_deref() == Some(wanted.as_str()))
}

/// `e-patch-2` → `E-PATCH-002`; None when it is not shaped like a code.
pub fn normalize(code: &str) -> Option<String> {
    let upper = code.trim().to_ascii_uppercase();
    let rest = upper.strip_prefix("E-")?;
    let (area, number) = rest.rsplit_once('-')?;
    if area.is_empty() || !area.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let number: u32 = number.parse().ok()?;
    Some(format!("E-{area}-{number:03}"))
}

/// Check a catalog: well-formed, unique codes (used by every crate's tests).
pub fn check_catalog(catalog: &[ErrorInfo]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for info in catalog {
        if normalize(info.code).as_deref() != Some(info.code) {
            return Err(format!("malformed code '{}'", info.code));
        }
        if !seen.insert(info.code) {
            return Err(format!("duplicate code '{}'", info.code));
        }
        if info.title.is_empty() || info.explain.is_empty() {
            return Err(format!("'{}' needs a title and an explanation", info.code));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATALOG: &[ErrorInfo] = &[ErrorInfo {
        code: "E-TEST-001",
        title: "Test",
        explain: "A test entry.",
    }];

    #[test]
    fn normalizes_and_finds_codes() {
        assert_eq!(normalize("e-test-1").as_deref(), Some("E-TEST-001"));
        assert_eq!(normalize("E-TEST-001").as_deref(), Some("E-TEST-001"));
        assert_eq!(normalize("TEST-1"), None);
        assert_eq!(normalize("E-T3ST-1"), None);
        assert_eq!(find(&[CATALOG], "e-test-01").map(|i| i.title), Some("Test"));
        assert!(find(&[CATALOG], "E-TEST-002").is_none());
    }

    #[test]
    fn catalog_check_catches_duplicates() {
        assert!(check_catalog(CATALOG).is_ok());
        let dup = [CATALOG[0], CATALOG[0]];
        assert!(check_catalog(&dup).is_err());
    }
}
