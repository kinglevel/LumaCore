// ██╗    ██╗ ██████╗ ██████╗ ██████╗ ███████╗
// ██║    ██║██╔═══██╗██╔══██╗██╔══██╗██╔════╝
// ██║ █╗ ██║██║   ██║██████╔╝██║  ██║███████╗
// ██║███╗██║██║   ██║██╔══██╗██║  ██║╚════██║
// ╚███╔███╔╝╚██████╔╝██║  ██║██████╔╝███████║
//  ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: words
//! The registry is the one source of command and object words (decisions §15, §31): no keyword
//! list outside it, every registry entry states what decides it, every flag and error code in the
//! source is declared, and every bound line (a TUI hotkey, a core UI key, the built-in
//! keyboard's word row) starts with a registered word.
//!
//! Owns:          the word, flag, error-code and bound-line rules
//! Doesn't own:   the registry (lumacore-objects), the file scanners (files)
//! Entry points:  the #[test] rules below
//! Decisions:     §15, §26, §29, §31

use lumacore_objects::commands::registry;

use crate::files::{line_of, non_test_source, objects_dir, rust_files, crate_dir, station_files, station_rel, is_test_file, string_literals, workspace};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The marker a grammar or value domain that shares names carries.
const MARKER: &str = "tidy: not-keywords";

/// Files that define the words themselves.
const DEFINITIONS: &[&str] = &["core/commands/registry.rs"];

/// The fields a registry entry must always state; the rest may come from
/// `..CommandDef::BASE` (decisions §31).
const STATED: &[&str] = &["verb", "doc", "run", "undo", "scope", "rights", "reach"];

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

// *************************************************************************************************
// Keyword lists
// Runs of string literals joined by `|`.
// *************************************************************************************************

/// Runs of string literals joined by `|` (match alternatives).
fn alternations(src: &str) -> Vec<(usize, Vec<String>)> {
    let literals = string_literals(src);
    let mut groups: Vec<(usize, Vec<String>)> = Vec::new();
    for (pos, text) in literals {
        if let Some((start, group)) = groups.last_mut() {
            let last_end = *start + group_len(src, *start, group.len());
            let between = &src[last_end.min(pos)..pos];
            if between.trim() == "|" {
                group.push(text);
                continue;
            }
        }
        groups.push((pos, vec![text]));
    }
    groups.retain(|(_, g)| g.len() >= 2);
    groups
}

/// Byte length of the first `n` literals of a group starting at `start`.
fn group_len(src: &str, start: usize, n: usize) -> usize {
    let mut end = start;
    for (pos, text) in string_literals(&src[start..]).into_iter().take(n) {
        end = start + pos + text.len() + 2;
    }
    end - start
}

/// A marker on the pattern's line or up to eight lines above (one marker
/// covers a whole `match` of a value domain).
fn exempt(src: &str, pos: usize) -> bool {
    let line = line_of(src, pos);
    src.lines()
        .skip(line.saturating_sub(9))
        .take(9)
        .any(|l| l.contains(MARKER))
}

// *************************************************************************************************
// Registry entries and flags
// *************************************************************************************************

/// Which of [`STATED`] each `CommandDef` literal in `src` leaves out, by verb.
fn unstated_fields(src: &str) -> Vec<String> {
    let mut missing = Vec::new();
    for entry in src.split("\n    CommandDef {\n").skip(1) {
        let entry = entry.split("\n    },").next().unwrap_or(entry);
        // `run` beside the row (8 spaces), the row's own fields in `decl: CommandDecl {` (12).
        let at = |l: &str, n: usize| l.len() > n && l[..n].trim().is_empty() && !l[n..].starts_with(' ');
        let fields: Vec<&str> = entry
            .lines()
            .filter(|l| at(l, 8) || at(l, 12))
            .filter_map(|l| l.trim().split(':').next())
            .collect();
        let verb = entry
            .lines()
            .find_map(|l| l.trim().strip_prefix("verb: "))
            .unwrap_or("?")
            .trim_end_matches(',');
        for field in STATED {
            if !fields.contains(field) {
                missing.push(format!("{verb}: {field}"));
            }
        }
    }
    missing
}

/// Whether `word` (with its slash, any case, an optional `=value`) names a declared flag.
fn declared_flag(word: &str) -> bool {
    let name = word.trim_start_matches('/');
    let name = name.split('=').next().unwrap_or(name);
    lumacore_objects::commands::flags::all().any(|f| f.named(name))
}

// *************************************************************************************************
// Bound lines
// *************************************************************************************************

/// The first word of a bound command line must be a registered word.
fn check_command_line(origin: &str, line: &str, failures: &mut Vec<String>) {
    let Some(first) = line.split_whitespace().next() else {
        return;
    };
    if registry::resolve(first).is_err() {
        failures.push(format!(
            "{origin}: '{line}' — '{first}' is not a command or object"
        ));
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

// *************************************************************************************************
// Keyword lists
// Words are published through CommandDef / ObjectDef (§15).
// *************************************************************************************************

#[test]
fn no_keyword_lists_outside_the_registry() {
    let mut files = Vec::new();
    files.extend(station_files(""));
    let mut violations = Vec::new();
    for file in files {
        let rel = station_rel(&file);
        if DEFINITIONS.contains(&rel.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        let src = non_test_source(&text);
        for (pos, group) in alternations(src) {
            let words: Vec<&String> = group
                .iter()
                .filter(|w| !w.is_empty() && registry::lookup_word(w).is_some())
                .collect();
            if words.len() >= 2 && !exempt(src, pos) {
                violations.push(format!(
                    "{rel}:{}: {}",
                    line_of(src, pos),
                    words
                        .iter()
                        .map(|w| w.as_str())
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "keyword lists outside the registry (publish words via CommandDef/ObjectDef, \
         or mark a grammar/value domain with `// {MARKER} (<reason>)`):\n  {}",
        violations.join("\n  ")
    );
}

#[test]
fn scanner_finds_alternations() {
    let src = "match x { \"store\" | \"delete\" => 1, \"a\" => 2 }\n// \"list\" | \"info\"\n";
    let groups = alternations(src);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].1, ["store", "delete"]);

    // Spread over lines, as rustfmt writes long patterns.
    let src = "Some(\n    \"help\" | \"list\"\n    | \"info\",\n) => 1";
    assert_eq!(alternations(src)[0].1, ["help", "list", "info"]);
}

// *************************************************************************************************
// Registry entries
// Every entry states what decides it (§31).
// *************************************************************************************************

#[test]
fn every_command_states_what_decides_it() {
    // Every family's rows (decisions §34 25.13: `commands/registry/rows/<family>.rs`).
    let mut files = Vec::new();
    rust_files(&objects_dir().join("commands/registry/rows"), &mut files);
    assert!(!files.is_empty(), "the command families");
    let mut missing = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read a family");
        let start = text.find("pub(crate) static ROWS").expect("the family's rows");
        missing.extend(unstated_fields(&text[start..]));
    }
    assert!(
        missing.is_empty(),
        "registry entries must state {STATED:?} themselves (only the rest comes from \
         `..CommandDef::BASE`):\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn stated_field_scanner_finds_a_missing_field() {
    let src = "x\n    CommandDef {\n        run: go,\n        decl: CommandDecl {\n            verb: \"go\",\n            doc: Doc {\n                \
               display: \"Go\",\n            },\n            undo: Undo::ReadOnly,\n            scope: Scope::Master,\n            \
               reach: Reach::Show,\n            ..CommandDecl::BASE\n        },\n    },\n";
    assert_eq!(unstated_fields(src), vec!["\"go\": rights".to_string()]);
}

// *************************************************************************************************
// Flags and codes
// Every flag is declared, every error code catalogued (§26, §31).
// *************************************************************************************************

#[test]
fn every_flag_in_core_is_declared() {
    // Literals that start with a slash but are paths (asset folders in help, test paths).
    const PATHS: &[&str] = &["/volumes", "/tmp", "/etc", "/media", "/bogus"];
    let mut files = Vec::new();
    files.extend(station_files("core"));
    let mut undeclared = Vec::new();
    for file in files {
        let rel = station_rel(&file);
        if is_test_file(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        let src = non_test_source(&text);
        for (pos, lit) in string_literals(src) {
            let word = lit.trim_end_matches(['=', ' ']);
            let flag_like = word.len() > 1
                && word.starts_with('/')
                && word[1..].chars().all(|c| c.is_ascii_alphanumeric() || c == '=')
                && word[1..].starts_with(|c: char| c.is_ascii_alphabetic());
            if flag_like && !PATHS.contains(&word.to_ascii_lowercase().as_str()) && !declared_flag(word) {
                undeclared.push(format!("{rel}:{}: {lit}", line_of(src, pos)));
            }
        }
    }
    assert!(
        undeclared.is_empty(),
        "flags used in core but declared nowhere (CommandDef / ObjectDef `flags`, or \
         the global flags state registers — decisions §31):\n  {}",
        undeclared.join("\n  ")
    );
}

#[test]
fn every_flag_in_the_help_is_declared() {
    use lumacore_objects::commands::registry::Doc;
    let docs = lumacore_registry::get()
        .commands()
        .map(|c| (c.verb, &c.doc))
        .chain(lumacore_objects::objects::all().map(|o| (o.keyword, &o.doc)));
    let mut undeclared = Vec::new();
    for (word, doc) in docs {
        let doc: &Doc = doc;
        for text in std::iter::once(doc.syntax).chain(doc.examples.iter().copied()) {
            for token in text.split(|c: char| c.is_whitespace() || matches!(c, '[' | ']' | '|' | '(' | ')' | ',')) {
                let token = token.trim_end_matches(['.', ':', ';']);
                let flag_like = token.len() > 1
                    && token.starts_with('/')
                    && token[1..].starts_with(|c: char| c.is_ascii_alphabetic())
                    && !token[1..].contains('/');
                if flag_like && !token.contains('<') && !declared_flag(token) {
                    undeclared.push(format!("{word}: {token}   ({text})"));
                }
            }
        }
    }
    assert!(undeclared.is_empty(), "flags in help text but declared nowhere (decisions §31):\n  {}", undeclared.join("\n  "));
}

/// Every `E-AREA-NNN` code the station's source names is in a catalog (decisions §26, §31): an
/// uncatalogued code was only caught by a debug assertion when the line ran.
#[test]
fn every_error_code_in_the_source_is_catalogued() {
    let mut files = Vec::new();
    files.extend(station_files(""));
    let mut missing = Vec::new();
    let mut seen = 0;
    for file in files {
        let rel = station_rel(&file);
        if is_test_file(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        let src = non_test_source(&text);
        for (pos, lit) in string_literals(src) {
            let mut i = 0;
            while let Some(at) = lit[i..].find("E-") {
                let start = i + at;
                let code: String = lit[start..]
                    .chars()
                    .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '-')
                    .collect();
                i = start + 2;
                let well_formed = code.len() >= 9
                    && code[2..].split('-').count() == 2
                    && code.rsplit('-').next().is_some_and(|n| n.len() == 3 && n.bytes().all(|b| b.is_ascii_digit()));
                // The HTTP engine's codes are in its own catalog (the `http` feature).
                let http_only = cfg!(not(feature = "http")) && (code.starts_with("E-FEED-") || code.starts_with("E-HTTP-"));
                seen += usize::from(well_formed);
                if well_formed && !http_only && lumacore_registry::get().error(&code).is_none() {
                    missing.push(format!("{rel}:{}: {code}", line_of(src, pos)));
                }
            }
        }
    }
    assert!(seen > 300, "the scan found only {seen} codes");
    missing.sort();
    missing.dedup();
    assert!(missing.is_empty(), "error codes in the source but in no catalog (add them, decisions §26):\n  {}", missing.join("\n  "));
}

// *************************************************************************************************
// Bound lines
// A key's line starts with a registered word.
// *************************************************************************************************

#[test]
fn tui_hotkeys_run_registered_commands() {
    let mut failures = Vec::new();
    let mut files = Vec::new();
    rust_files(&crate_dir("lumacore-terminal").join("repl"), &mut files);
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read source");
        let src = non_test_source(&text);
        for (i, _) in src.match_indices("execute_command(state, \"") {
            let start = i + "execute_command(state, \"".len();
            let end = src[start..].find('"').map_or(start, |e| start + e);
            let origin = format!(
                "{}:{}",
                file.file_name().unwrap_or_default().to_string_lossy(),
                line_of(src, i)
            );
            check_command_line(&origin, &src[start..end], &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn core_ui_keys_run_registered_commands() {
    let mut failures = Vec::new();
    for key in lumacore_tree::builtin_ui_keys().values() {
        for line in [&key.press, &key.double_press, &key.hold] {
            check_command_line(&format!("UIKey {}", key.id), line, &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The built-in keyboard's command-word row (the page's `KB_WORDS`) is
/// registry words.
#[test]
fn keyboard_words_are_registry_words() {
    let app = workspace().join("crates/ui/lumacore-webui/web/index.html");
    let text = std::fs::read_to_string(app).expect("read index.html");
    let line = text.lines().find(|l| l.contains("const KB_WORDS")).expect("KB_WORDS in index.html");
    let words: Vec<&str> = line.split('\'').skip(1).step_by(2).collect();
    assert!(!words.is_empty());
    let mut failures = Vec::new();
    for w in words {
        // `+`, `-` and `Thru` are the selection grammar (keys), not words.
        // tidy: not-keywords (selection grammar)
        if matches!(w, "+" | "-" | "Thru") {
            continue;
        }
        if registry::resolve(w).is_err() {
            failures.push(w.to_string());
        }
    }
    assert!(failures.is_empty(), "KB_WORDS not in the registry: {failures:?}");
}
