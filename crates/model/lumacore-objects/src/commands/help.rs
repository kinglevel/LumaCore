//! Help system for the LumaCore REPL.
//!
//! Object-polymorphic help model:
//! - Commands declare which object type(s) they operate on
//! - Help overview groups by object, then shows global commands
//! - Each object shows its available functions (like reference console's button function enums)
//!
//! All command metadata lives in the unified registry (registry.rs).
//! This module only handles rendering — no hardcoded command lists.
//!
//! Alias depth filtering:
//! - "Base" — show only core commands
//! - "ProgLang" — Base + Edit/Undo
//! - "Full" — all commands including System

use super::registry::{self, AliasDepth, Word};
use crate::AppState;
use crate::defs::command::HelpColour;
use nu_ansi_term::{Color, Style};

// ── Category constants ───────────────────────────────────────────────────────

pub use crate::defs::command::HelpCategory;


// ── Public API ───────────────────────────────────────────────────────────────

/// Dispatch `Help [<command>]` — categorized overview or per-command detail.
///
/// - Bare `Help` → `categorized_overview(state)`
/// - `Help <verb>` → full documentation for that command
pub fn cmd_help(state: &AppState, args: &[&str]) -> String {
    // Check for /all flag — show everything including hidden and power-user commands
    let has_all = crate::commands::flags::has(args, &crate::commands::flags::ALL);
    let filtered: Vec<&str> = args
        .iter()
        .filter(|a| !crate::commands::flags::is(a, &crate::commands::flags::ALL))
        .copied()
        .collect();

    if filtered.is_empty() {
        if has_all {
            categorized_overview_all()
        } else {
            categorized_overview(state)
        }
    } else {
        let query = filtered.join(" ").to_lowercase();
        detail_page(&query)
    }
}

/// Return a one-liner syntax hint for a given verb.
///
/// Used by the `verb?` shortcut in dispatch. Returns `None` if the verb is unknown.
pub fn format_quick_hint(verb: &str) -> Option<String> {
    let word = registry::lookup_word(verb)?;
    if word.doc().hidden {
        return None;
    }
    Some(format!(
        "{}  {}{}",
        Color::Cyan.bold().paint(word.label()),
        Style::new().dimmed().paint(word.doc().syntax),
        planned_mark(word),
    ))
}

/// Categorized help overview, filtered by the user's `alias_depth`.
///
/// "Base" → Control, Inspection, Navigation, and Shutdown only.
/// "ProgLang" → Base + Edit, Undo.
/// "Full" → all categories.
pub fn categorized_overview(state: &AppState) -> String {
    let depth = user_alias_depth(state);
    let mut text = String::new();

    let header = Style::new().bold().paint("LumaCore Commands").to_string();
    text.push_str(&format!("{header}\n"));
    text.push_str(&format!(
        "{}\n",
        Style::new()
            .dimmed()
            .paint("Object-polymorphic grouping — commands grouped by the object they act on.")
    ));

    // Depth note
    if depth == AliasDepth::Base {
        text.push_str(&format!(
            "{}\n\n",
            Style::new()
                .dimmed()
                .paint("Showing Base commands. User alias_depth controls visible command set.")
        ));
    }

    for cat in &OVERVIEW_ORDER {
        // Filter by alias_depth
        if depth < cat.min_depth() {
            continue;
        }

        let entries: Vec<Word> = overview_words()
            .filter(|w| !w.doc().hidden)
            .filter(|w| w.doc().category == *cat && depth >= w.doc().alias_depth)
            .collect();

        if entries.is_empty() {
            continue;
        }

        // Section header
        let color = cat.color();
        text.push_str(&format!(
            "\n{}\n",
            color.bold().paint(format!("── {} ──", cat.label()))
        ));

        // Command rows: verb  syntax-hint
        for entry in entries {
            let verb_col = if entry.is_planned() {
                Style::new().dimmed().paint(entry.label()).to_string()
            } else {
                color.paint(entry.label()).to_string()
            };
            let visible_len = entry.label().len();
            let pad = if visible_len < 16 {
                16 - visible_len
            } else {
                1
            };
            text.push_str(&format!(
                "  {}{}{}{}\n",
                verb_col,
                " ".repeat(pad),
                Style::new().dimmed().paint(entry.doc().syntax),
                planned_mark(entry),
            ));
        }
    }

    text.push_str(&format!(
        "\n{}",
        Style::new()
            .dimmed()
            .paint("Type 'help <command>' for full docs. Type '<command>?' for quick syntax.")
    ));

    text
}

/// "  (planned)" after a published, not yet implemented command.
fn planned_mark(word: Word) -> String {
    if word.is_planned() {
        format!("  {}", Color::Yellow.paint("(planned)"))
    } else {
        String::new()
    }
}

/// Commands and objects documented with a syntax line, for the overviews.
/// (Objects with only a description are still shown by `Help <object>`.)
fn overview_words() -> impl Iterator<Item = Word> {
    registry::all_words().filter(|w| !w.doc().syntax.is_empty())
}

/// Full help overview — all commands, all categories, including hidden and power-user.
fn categorized_overview_all() -> String {
    let mut text = String::new();

    let header = Style::new()
        .bold()
        .paint("LumaCore Commands (all)")
        .to_string();
    text.push_str(&format!("{header}\n"));
    text.push_str(&format!(
        "{}\n\n",
        Style::new()
            .dimmed()
            .paint("Showing all commands including hidden and power-user.")
    ));

    for cat in &OVERVIEW_ORDER {
        let entries: Vec<Word> = overview_words()
            .filter(|w| w.doc().category == *cat)
            .collect();

        if entries.is_empty() {
            continue;
        }

        let color = cat.color();
        text.push_str(&format!(
            "\n{}\n",
            color.bold().paint(format!("── {} ──", cat.label()))
        ));

        for entry in entries {
            let hidden = entry.doc().hidden;
            let label = if hidden {
                format!("({})", entry.names()[0])
            } else {
                entry.label().to_string()
            };
            let verb_col = if hidden {
                Style::new().dimmed().paint(&label).to_string()
            } else {
                color.paint(&label).to_string()
            };
            let visible_len = label.len();
            let pad = if visible_len < 16 {
                16 - visible_len
            } else {
                1
            };
            text.push_str(&format!(
                "  {}{}{}\n",
                verb_col,
                " ".repeat(pad),
                Style::new().dimmed().paint(entry.doc().syntax),
            ));
        }
    }

    text.push_str(&format!(
        "\n{}",
        Style::new()
            .dimmed()
            .paint("Type 'help <command>' for full docs. Type '<command>?' for quick syntax.")
    ));

    text
}

/// Quicksheet — the tree here (`ls`) and every command and object, one
/// colour-coded line per help category (Tab on an empty prompt).
pub fn quicksheet(state: &AppState) -> String {
    let dim = |s: &str| Style::new().dimmed().paint(s).to_string();
    let mut text = super::navigate::cmd_ls(state, &lumacore_syntax::FenceLine::default());
    text.push('\n');

    let sep = dim(" \u{00b7} ");
    for cat in OVERVIEW_ORDER {
        let color = cat.color();
        let words: Vec<String> = overview_words()
            .filter(|w| !w.doc().hidden && w.doc().category == cat)
            .map(|w| color.paint(w.label()).to_string())
            .collect();
        if words.is_empty() {
            continue;
        }
        text.push_str(&format!(
            "{} {}\n",
            color.bold().paint(format!("{}:", cat.label())),
            words.join(&sep),
        ));
    }
    text.push_str(&format!(
        "\n{}",
        dim("Tab = this view  |  help <cmd> = details")
    ));
    text
}

/// Category order of the help overviews and the quicksheet.
const OVERVIEW_ORDER: [HelpCategory; 11] = [
    // Objects first — the things you work with
    HelpCategory::Fixture,
    HelpCategory::Programmer,
    HelpCategory::Playback,
    HelpCategory::DataPool,
    // Command families — actions you perform on objects
    HelpCategory::Universe,
    HelpCategory::Inspection,
    HelpCategory::ProgrammerNav,
    HelpCategory::TreeNav,
    HelpCategory::Undo,
    HelpCategory::Group,
    HelpCategory::System,
];

// ── Private helpers ──────────────────────────────────────────────────────────

/// Return detailed help for a single command verb.
fn detail_page(query: &str) -> String {
    match query {
        "lum" => return lum_page(),
        "lum assign" => return lum_assign_page(),
        "lum set" => return lum_set_page(),
        "lum variables" => return lum_variables_page(),
        "treeshell" => return treeshell_page(),
        "treeshell navigation" => return treeshell_navigation_page(),
        "lum modifiers" | "modifiers" => return lum_modifiers_page(),
        _ => {}
    }

    // Exact name (verb, keyword or alias), then the resolver's prefix rules.
    let entry = registry::lookup_word(query).or_else(|| registry::resolve(query).ok());

    match entry {
        Some(e) if !e.doc().hidden => format_detail(e),
        _ => crate::errors::render_code(
            format!("Unknown command: '{}'. Type 'help' for a list.", query),
            "E-CMD-001",
        ),
    }
}

/// Format full documentation for a command or object.
fn format_detail(word: Word) -> String {
    let mut text = String::new();
    let e = word.doc();

    // Title: verb [category] (planned)
    text.push_str(&format!(
        "{}  {}{}\n",
        Style::new().bold().paint(word.label()),
        e.category
            .color()
            .paint(format!("[{}]", e.category.label())),
        planned_mark(word),
    ));

    // Syntax
    if !e.syntax.is_empty() {
        text.push_str(&format!("  {}\n", Color::Cyan.paint(e.syntax)));
    }

    // Description
    text.push_str(&format!("\n{}\n", e.description));

    // The level it needs (the rights gate, decisions §19, §30 "12d").
    let needs = match word {
        Word::Object(o) => crate::rights::describe_object(o),
        Word::Command(c) => {
            let mut needs = match c.rights {
                registry::Rights::Internal => "the internal root monitor".to_string(),
                r => r.label().to_string(),
            };
            if c.object_verb.is_some() {
                needs.push_str(" (a named object decides: `Help <object>`)");
            }
            if c.arg_rights.is_some() {
                needs.push_str("; some arguments need more");
            }
            needs
        }
    };
    text.push_str(&format!(
        "\n{}  {needs}\n",
        Style::new().dimmed().paint("Needs:")
    ));

    // An object's verbs, built and planned (decisions §29).
    if let Word::Object(o) = word {
        let built: Vec<&str> = o.verbs.iter().map(|(v, _)| v.label()).collect();
        text.push_str(&format!(
            "\n{}  {}\n",
            Style::new().dimmed().paint("Verbs:"),
            built.join(", ")
        ));
        if !o.planned.is_empty() {
            let planned: Vec<&str> = o.planned.iter().map(|v| v.label()).collect();
            text.push_str(&format!(
                "{}  {}\n",
                Color::Yellow.paint("Planned:"),
                planned.join(", ")
            ));
        }
    }

    // Its options (decisions §31): the declared flags, with the verbs an object's go with.
    let own: &[crate::commands::flags::FlagDef] = match word {
        Word::Object(o) => o.flags,
        Word::Command(c) => c.flags,
    };
    if !own.is_empty() {
        text.push_str(&format!("\n{}\n", Style::new().bold().paint("Options:")));
        let width = own.iter().map(|f| f.usage().chars().count()).max().unwrap_or(0);
        for f in own {
            let verbs = f.verbs_label();
            let verbs = if verbs.is_empty() { String::new() } else { format!("  [{verbs}]") };
            let usage = f.usage();
            let pad = " ".repeat(width - usage.chars().count());
            text.push_str(&format!(
                "  {}{pad}  {}{}\n",
                Color::Cyan.paint(usage),
                f.doc,
                Style::new().dimmed().paint(verbs),
            ));
        }
        text.push_str(&format!(
            "{}  {}\n",
            Style::new().dimmed().paint("On every line:"),
            crate::commands::flags::globals_line()
        ));
    }

    // Aliases (every name but the first)
    let names = word.names();
    if names.len() > 1 {
        text.push_str(&format!(
            "\n{}  {}\n",
            Style::new().dimmed().paint("Aliases:"),
            names[1..].join(", "),
        ));
    }

    // Examples
    if !e.examples.is_empty() {
        text.push_str(&format!("\n{}\n", Style::new().bold().paint("Examples:")));
        for ex in e.examples {
            text.push_str(&format!("  {}\n", Color::Green.paint(*ex),));
        }
    }

    // Related
    if !e.related.is_empty() {
        text.push_str(&format!(
            "\n{}  {}\n",
            Style::new().dimmed().paint("See also:"),
            e.related.join(", "),
        ));
    }

    text
}

fn lum_page() -> String {
    format!(
        "{}\n\n{}\n\n{}\n{}\n{}\n{}\n{}\n",
        Style::new().bold().paint("Lum"),
        "Lum is LumaCore's object-based operator language: verb first, then the object.",
        "Sentence families:",
        "  Selection     Fixture 1 Thru 4",
        "  Action        At 50",
        "  Assignment    Assign Universe 1 At Output 1 /Universe 1",
        "  Settings      Set Fixture 3 \"Name\" \"Spot SR\"",
    ) + "\nUse 'help lum assign', 'help lum set', 'help lum variables', or 'help lum modifiers' for more detail.\nThe full list: docs/user/reference/commands.md."
}

fn lum_assign_page() -> String {
    format!(
        "{}\n\n{}\n\n{}\n{}\n{}\n{}\n",
        Style::new().bold().paint("Lum Assign"),
        "Assign links one object to a typed target. It stays one of the most explicit parts of Lum.",
        "Canonical form:",
        "  Assign <Object> <id> At <Target> [/<option> <value>]",
        "Examples:",
        "  Assign Universe 1 Thru 6 At Output 1 /Universe 101\n  Assign Universe 2 At Output 3 /Universe 0.1.2\n  Assign Universe 5 At Input 1 /Universe 7",
    ) + "\nUse explicit protocol targets instead of loose free-form values."
}

fn lum_set_page() -> String {
    format!(
        "{}\n\n{}\n\n{}\n{}\n{}\n{}\n",
        Style::new().bold().paint("Lum Set"),
        "Set changes a property the object declares writable; everything else is read-only and Set says why.",
        "Canonical form:",
        "  Set <object> <id> <property> <value>   (or Set <tree path> <property> <value>)",
        "Examples:",
        "  Set Fixture 3 \"Name\" \"Spot SR\" \n Set Output 1 \"Priority\" 120\n Set Station \"Name\" \"Luma01\"",
    ) + "\n'Set Fixture 3' lists what can be set; Info shows the current values."
}

fn lum_variables_page() -> String {
    format!(
        "{}\n\n{}\n{}\n{}\n{}\n",
        Style::new().bold().paint("Lum Variables"),
        "Variables aren't built yet (planned):",
        "  UserVar    saved with the user profile",
        "  ShowVar    saved with the show",
        "  GlobalVar  runtime only",
    )
}

fn treeshell_page() -> String {
    format!(
        "{}\n\n{}\n{}\n{}\n{}\n",
        Style::new().bold().paint("TreeShell"),
        "TreeShell is the tree-aware system side of the mixed LumaCore shell.",
        "Use it for navigation, inspection, and direct structure work.",
        "Examples:",
        "  cd ShowData\n  ls\n  pwd\n  dump ShowData",
    )
}

fn treeshell_navigation_page() -> String {
    format!(
        "{}\n\n{}\n{}\n{}\n{}\n{}\n{}\n",
        Style::new().bold().paint("TreeShell Navigation"),
        "Navigation commands move through and inspect the root tree and runtime structure.",
        "Core commands:",
        "  cd    change location",
        "  ls    list children or objects",
        "  pwd   print current path",
        "  enter jump into a known object path",
    )
}

fn lum_modifiers_page() -> String {
    format!(
        "{}\n\n{}\n{}\n{}\n{}\n{}\n",
        Style::new().bold().paint("Lum Slash Modifiers"),
        "Adding '/' to a command introduces a modifier or option keyword.",
        "Examples:",
        "  SaveShow \"demo\" /Overwrite\n  Store Group 3 /Merge",
        "Rule:",
        "  The main command still defines the sentence. The slash part modifies how that sentence behaves.",
    )
}

/// The current user's alias depth, defaulting to Base.
fn user_alias_depth(state: &AppState) -> AliasDepth {
    let user = state.effective_user_name();
    state
        .root
        .show_data
        .profile_for_user(user)
        .map_or(AliasDepth::Base, |p| AliasDepth::parse(&p.settings.alias_depth))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_expected_verbs() {
        assert!(registry::lookup("at").is_some(), "missing 'at'");
        assert!(
            registry::lookup_word("fixture").is_some(),
            "missing 'fixture'"
        );
        assert!(registry::lookup("store").is_some(), "missing 'store'");
        assert!(registry::lookup("help").is_some(), "missing 'help'");
        assert!(registry::lookup("list").is_some(), "missing 'list'");
    }

    #[test]
    fn format_quick_hint_known_verb() {
        let hint = format_quick_hint("at");
        assert!(hint.is_some(), "should return hint for 'at'");
        let text = hint.unwrap();
        assert!(text.contains("At") || text.contains("at") || text.len() > 2);
    }

    #[test]
    fn format_quick_hint_unknown_verb() {
        let hint = format_quick_hint("unknownverb123");
        assert!(hint.is_none(), "should return None for unknown verb");
    }

    #[test]
    fn format_quick_hint_hidden_verb() {
        let hint = format_quick_hint("quicksheet");
        assert!(hint.is_none(), "should return None for hidden verb");
    }

    #[test]
    fn depth_order() {
        assert!(AliasDepth::Base >= AliasDepth::Base);
        assert!(AliasDepth::Base < AliasDepth::Full);
        assert!(AliasDepth::Full >= AliasDepth::ProgLang);
        assert_eq!(AliasDepth::parse("proglang"), AliasDepth::ProgLang);
        assert_eq!(AliasDepth::parse("nonsense"), AliasDepth::Base);
    }

    #[test]
    fn categorized_overview_base_user() {
        let state = AppState::new();
        let text = categorized_overview(&state);
        // Should have object-polymorphic subtitle
        assert!(text.contains("Object-polymorphic"));
        // System commands should NOT appear for Base users
        assert!(
            !text.contains("Unpatch"),
            "Base user should not see Unpatch"
        );
    }

    #[test]
    fn cmd_help_bare_produces_output() {
        let state = AppState::new();
        let output = cmd_help(&state, &[]);
        assert!(!output.is_empty());
    }

    #[test]
    fn cmd_help_specific_command() {
        let state = AppState::new();
        let output = cmd_help(&state, &["at"]);
        assert!(output.contains("At") || output.contains("at"));
        assert!(output.contains("50") || output.contains("Full"));
    }

    #[test]
    fn cmd_help_unknown_command() {
        let state = AppState::new();
        let output = cmd_help(&state, &["unknownxyz"]);
        assert!(output.contains("Unknown"));
    }

    #[test]
    fn cmd_help_shows_aliases() {
        let state = AppState::new();
        let output = cmd_help(&state, &["oops"]);
        assert!(
            output.contains("hiss") || output.contains("unmeow"),
            "Oops help should show its aliases"
        );
    }

    #[test]
    fn cmd_help_lum_pages_exist() {
        let state = AppState::new();
        assert!(cmd_help(&state, &["lum"]).contains("Sentence families"));
        assert!(cmd_help(&state, &["lum", "assign"]).contains("Canonical form"));
        assert!(cmd_help(&state, &["treeshell", "navigation"]).contains("cd"));
        assert!(cmd_help(&state, &["lum", "modifiers"]).contains("/Overwrite"));
    }
}
