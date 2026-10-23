//! Error and warning types for GDTF import operations.
//!
//! `GdtfError` covers fatal errors that prevent import.
//! `GdtfWarning` covers non-fatal DIN SPEC 15800 schema violations that are
//! collected during import and returned to the caller for display.

use thiserror::Error;

/// Fatal errors that prevent GDTF import from completing.
#[derive(Debug, Error)]
pub enum GdtfError {
    /// File could not be opened or read.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// File is structurally invalid (not a ZIP, missing description.xml, etc.).
    #[error("Invalid GDTF file: {0}")]
    InvalidGdtf(String),

    /// XML inside the GDTF archive could not be parsed.
    #[error("Malformed XML in GDTF file: {0}")]
    MalformedXml(String),

    /// Generic parse error for individual field values.
    #[error("Parse error: {0}")]
    Parse(String),

    /// A virtual channel references a target channel that could not be resolved.
    ///
    /// This is returned only when all modes in a fixture file fail to resolve.
    /// Partial resolution (some modes fail) uses warnings instead.
    #[error("Virtual channel '{channel_name}' in mode '{mode_name}' could not be resolved")]
    VirtualChannelUnresolved {
        channel_name: String,
        mode_name: String,
    },

    /// JSON serialization/deserialization error (used by FixtureTypeIndex).
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl lumacore_base::Coded for GdtfError {
    fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "E-GDTF-001",
            Self::InvalidGdtf(_) => "E-GDTF-002",
            Self::MalformedXml(_) => "E-GDTF-003",
            Self::Parse(_) => "E-GDTF-004",
            Self::VirtualChannelUnresolved { .. } => "E-GDTF-005",
            Self::Json(_) => "E-GDTF-006",
        }
    }
}

/// Error catalog of GDTF import (`Info Error <code>`).
pub const ERRORS: &[lumacore_base::ErrorInfo] = &[
    lumacore_base::ErrorInfo {
        code: "E-GDTF-001",
        title: "Cannot read GDTF file",
        explain: "The file could not be opened or read. Check the path and permissions.",
    },
    lumacore_base::ErrorInfo {
        code: "E-GDTF-002",
        title: "Invalid GDTF file",
        explain: "The file is not a GDTF archive (not a ZIP, or description.xml is missing).",
    },
    lumacore_base::ErrorInfo {
        code: "E-GDTF-003",
        title: "Malformed GDTF XML",
        explain: "description.xml inside the archive is not valid XML.",
    },
    lumacore_base::ErrorInfo {
        code: "E-GDTF-004",
        title: "GDTF value parse error",
        explain: "A field in the fixture description has a value that cannot be parsed.",
    },
    lumacore_base::ErrorInfo {
        code: "E-GDTF-005",
        title: "Unresolved virtual channel",
        explain: "A virtual channel points at a channel that does not exist in any mode of the fixture.",
    },
    lumacore_base::ErrorInfo {
        code: "E-GDTF-006",
        title: "Fixture index JSON error",
        explain: "The fixture type index could not be read or written as JSON.",
    },
];

#[cfg(test)]
mod tests {
    #[test]
    fn catalog_is_valid() {
        lumacore_base::error_code::check_catalog(super::ERRORS).unwrap();
    }
}

/// Non-fatal DIN SPEC 15800 schema validation warning.
///
/// Warnings are collected during import and returned alongside the `FixtureType`.
/// The caller decides how to display or log them. Import still succeeds.
///
/// Each warning carries a code (e.g. `"DIN15800-MISSING-MANUFACTURER"`),
/// the offending field path, and a human-readable message.
#[derive(Debug, Clone)]
pub struct GdtfWarning {
    /// Machine-readable code identifying the violation type.
    /// Format: `DIN15800-<CATEGORY>-<DETAIL>` or `IMPORT-<DETAIL>`.
    pub code: String,
    /// Human-readable description of the violation.
    pub message: String,
    /// XPath-style field path identifying the offending value (e.g. `FixtureType/@Manufacturer`).
    pub field: String,
}

impl GdtfWarning {
    /// Construct a new warning.
    pub fn new(
        code: impl Into<String>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            field: field.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for GdtfWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.field, self.message)
    }
}
