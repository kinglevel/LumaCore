//! What the parser refuses, with codes (decisions §33 decision 34; the table in lum.md "Errors").

use crate::Span;
use lumacore_base::error_code::ErrorInfo;

/// One refusal: its code, where, and the message the operator reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

/// The codes, one constant each, so the parser never spells a code twice.
pub mod codes {
    pub const UNKNOWN_WORD: &str = "E-SYN-001";
    pub const MISSING: &str = "E-SYN-002";
    pub const UNEXPECTED: &str = "E-SYN-003";
    pub const CHILD_FIRST: &str = "E-SYN-004";
    pub const BARE_PROPERTY: &str = "E-SYN-005";
    pub const OPTION: &str = "E-SYN-006";
    pub const GLUED_OPERATOR: &str = "E-SYN-007";
    pub const PERSONAL_IN_STORED: &str = "E-SYN-008";
    pub const FENCE_IN_CODE: &str = "E-SYN-009";
    pub const AMBIGUOUS: &str = "E-SYN-010";
    pub const UNTERMINATED: &str = "E-SYN-011";
    pub const ID_PARTS: &str = "E-SYN-012";
    pub const HOLD_PHASE: &str = "E-SYN-013";
    pub const VALUE_WORD: &str = "E-SYN-014";
}

/// The catalog (`Info Error "E-SYN-001"`).
pub const ERRORS: &[ErrorInfo] = &[
    ErrorInfo {
        code: codes::UNKNOWN_WORD,
        title: "Unknown word",
        explain: "The line names a word Lum doesn't know. Every word has exactly one name — no short forms, no prefixes (decisions §33). The message names the closest words; typing fuzzily on a command line shows the full word as ghost text before Enter runs it.",
    },
    ErrorInfo {
        code: codes::MISSING,
        title: "Something is missing",
        explain: "The line stops where the grammar needs more: a value after a helper (`Copy Group 1 At`), ids after `Thru`, the end of a quote. The message says what was expected (docs/user/syntax/lum.md).",
    },
    ErrorInfo {
        code: codes::UNEXPECTED,
        title: "A word in the wrong place",
        explain: "A word that can't stand where it is: neither a declared child of the object before it nor a slot of the verb, an option before a slot (options come last), or a dotted keyword path (`DataPool.2.Group.3` — nesting is `DataPool 2 Group 3` or the tree path `Root.ShowData.DataPools.2.Groups.3`).",
    },
    ErrorInfo {
        code: codes::CHILD_FIRST,
        title: "Child before its owner",
        explain: "A nested object is written owner first, in tree order: `Goto Sequence 2 Cue 3`, not `Goto Cue 3 Sequence 2`. A child alone uses your selected owner (`Cue 3` = a cue of your selected sequence).",
    },
    ErrorInfo {
        code: codes::BARE_PROPERTY,
        title: "Property name not quoted",
        explain: "Property names are always quoted: `Set Fixture 3 \"Name\" \"Spot SR\"`. A bare keyword after an object only continues its path (decisions §33 decision 7).",
    },
    ErrorInfo {
        code: codes::OPTION,
        title: "Unknown option",
        explain: "The `/Name` isn't a declared option, or the line puts a value on an option that takes none. Every word's options are in the command reference; an option changes how a line runs, never what it does.",
    },
    ErrorInfo {
        code: codes::GLUED_OPERATOR,
        title: "Operator glued in a list",
        explain: "Inside a list the `+` / `-` stand alone: `Group 1 + 3`, not `Group 1+3` (some consoles read that as Group 1 alone, silently). A sign glued to digits after a space is part of the number (`At -120`).",
    },
    ErrorInfo {
        code: codes::PERSONAL_IN_STORED,
        title: "Personal shorthand in a stored line",
        explain: "A key, macro or cue command is stored exactly: it can't use a bare number (it can't know whose default object it will run with), an alias (`!St` is yours) or a fuzzy word. Write the full line.",
    },
    ErrorInfo {
        code: codes::FENCE_IN_CODE,
        title: "Tree-shell word in LumaCore's own line",
        explain: "The tree-shell words (`cd`, `ls`, …) are for what users write. A line LumaCore's code emits — a template, a registry line, a generated line — is pure Lum (decisions §33 decision 18).",
    },
    ErrorInfo {
        code: codes::AMBIGUOUS,
        title: "Ambiguous word",
        explain: "Fuzzy typing found more than one word for what was typed; the ghost text shows the choices. Type a little more, or pick one.",
    },
    ErrorInfo {
        code: codes::UNTERMINATED,
        title: "Quote not closed",
        explain: "The line ends inside quoted text. Close it with `\"`.",
    },
    ErrorInfo {
        code: codes::ID_PARTS,
        title: "Wrong number of id parts",
        explain: "The object's numbers take fewer dot parts than written (`Fixture 1.2.3`). `Cue 1.5`, `Master 2.1`, `DMX 1.001` take two; a dot never nests (decisions §33 decision 10).",
    },
    ErrorInfo {
        code: codes::HOLD_PHASE,
        title: "Hold verb without On / Off",
        explain: "Top, Flash, Temp and Swap say press or release right after the verb: `Flash On Sequence 1`, `Flash Off Sequence 1` (the usual console form; decisions §33 decision 15).",
    },
    ErrorInfo {
        code: codes::VALUE_WORD,
        title: "Value not accepted",
        explain: "The property doesn't take that value. On / off values are `On`, `Off`, `Yes`, `No` (and `Toggle` where declared); an enum lists its values.",
    },
];

#[cfg(test)]
mod tests {
    #[test]
    fn the_catalog_is_well_formed() {
        lumacore_base::error_code::check_catalog(super::ERRORS).unwrap();
        assert_eq!(super::ERRORS.len(), 14);
    }
}
