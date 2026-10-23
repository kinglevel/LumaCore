use crate::core::commands::registry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageMode {
    Neutral,
    Lum,
    TreeShell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandFamily {
    Selection,
    Action,
    Assignment,
    StoreUpdate,
    Settings,
    Inspection,
    Navigation,
    Session,
    System,
    Runtime,
    Playback,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashModifierKind {
    Recursive,
    Watch,
    Save,
    Load,
    Verbose,
    Debug,
    Other,
}

impl CommandFamily {
    pub fn label(self) -> &'static str {
        match self {
            CommandFamily::Selection => "Selection",
            CommandFamily::Action => "Action",
            CommandFamily::Assignment => "Assignment",
            CommandFamily::StoreUpdate => "Store/Update",
            CommandFamily::Settings => "Settings",
            CommandFamily::Inspection => "Inspection",
            CommandFamily::Navigation => "Navigation",
            CommandFamily::Session => "Session",
            CommandFamily::System => "System",
            CommandFamily::Runtime => "Runtime",
            CommandFamily::Playback => "Playback",
            CommandFamily::Edit => "Edit",
        }
    }

    pub fn sentence_preview(self) -> &'static str {
        match self {
            CommandFamily::Selection => "Lum: Selection sentence",
            CommandFamily::Action => "Lum: Action sentence",
            CommandFamily::Assignment => "Lum: Object assignment sentence",
            CommandFamily::StoreUpdate => "Lum: Store/update sentence",
            CommandFamily::Settings => "Lum: Setting sentence",
            CommandFamily::Inspection => "Lum: Inspection sentence",
            CommandFamily::Navigation => "TreeShell: Navigation sentence",
            CommandFamily::Session => "TreeShell: Session sentence",
            CommandFamily::System => "TreeShell: System sentence",
            CommandFamily::Runtime => "TreeShell: Runtime sentence",
            CommandFamily::Playback => "Lum: Playback sentence",
            CommandFamily::Edit => "Lum: Edit sentence",
        }
    }

    pub fn mode(self) -> LanguageMode {
        match self {
            CommandFamily::Navigation
            | CommandFamily::Session
            | CommandFamily::System
            | CommandFamily::Runtime => LanguageMode::TreeShell,
            _ => LanguageMode::Lum,
        }
    }
}

/// The language family of a command or object word (exact name or alias),
/// as its registry entry declares it.
pub fn family_for_verb(verb: &str) -> Option<CommandFamily> {
    registry::lookup_word(verb)?.family()
}

fn first_token(input: &str) -> Option<&str> {
    input.split_whitespace().next()
}

fn slash_modifier_token(input: &str) -> Option<&str> {
    input
        .split_whitespace()
        .find(|token| token.starts_with('/'))
}

pub fn infer_mode(input: &str) -> LanguageMode {
    let Some(token) = first_token(input) else {
        return LanguageMode::Neutral;
    };
    if let Some(family) = family_for_partial(token) {
        return family.mode();
    }
    LanguageMode::Neutral
}

pub fn sentence_preview(input: &str) -> Option<&'static str> {
    let token = first_token(input)?;
    let family = family_for_partial(token)?;
    if slash_modifier_token(input).is_some() {
        return Some(modifier_sentence_preview(family));
    }
    Some(family.sentence_preview())
}

pub fn family_label(input: &str) -> Option<&'static str> {
    let token = first_token(input)?;
    family_for_partial(token).map(CommandFamily::label)
}

pub fn slash_modifier_kind(input: &str) -> Option<SlashModifierKind> {
    let token = slash_modifier_token(input)?.to_ascii_lowercase();
    Some(match token.as_str() {
        "/r" => SlashModifierKind::Recursive,
        "/watch" => SlashModifierKind::Watch,
        "/save" => SlashModifierKind::Save,
        "/load" => SlashModifierKind::Load,
        "/v" | "/verbose" => SlashModifierKind::Verbose,
        "/d" | "/debug" => SlashModifierKind::Debug,
        _ => SlashModifierKind::Other,
    })
}

pub fn slash_modifier_preview(input: &str) -> Option<&'static str> {
    let kind = slash_modifier_kind(input)?;
    Some(match kind {
        SlashModifierKind::Recursive => "Slash modifier: recursive option",
        SlashModifierKind::Watch => "Slash modifier: watch option",
        SlashModifierKind::Save => "Slash modifier: save option",
        SlashModifierKind::Load => "Slash modifier: load option",
        SlashModifierKind::Verbose => "Slash modifier: verbose option",
        SlashModifierKind::Debug => "Slash modifier: debug option",
        SlashModifierKind::Other => "Slash modifier: command-specific option",
    })
}

fn modifier_sentence_preview(family: CommandFamily) -> &'static str {
    match family {
        CommandFamily::Selection => "Lum: Selection sentence with slash modifier",
        CommandFamily::Action => "Lum: Action sentence with slash modifier",
        CommandFamily::Assignment => "Lum: Object assignment sentence with slash modifier",
        CommandFamily::StoreUpdate => "Lum: Store/update sentence with slash modifier",
        CommandFamily::Settings => "Lum: Setting sentence with slash modifier",
        CommandFamily::Inspection => "Lum: Inspection sentence with slash modifier",
        CommandFamily::Navigation => "TreeShell: Navigation sentence with slash modifier",
        CommandFamily::Session => "TreeShell: Session sentence with slash modifier",
        CommandFamily::System => "TreeShell: System sentence with slash modifier",
        CommandFamily::Runtime => "TreeShell: Runtime sentence with slash modifier",
        CommandFamily::Playback => "Lum: Playback sentence with slash modifier",
        CommandFamily::Edit => "Lum: Edit sentence with slash modifier",
    }
}

/// The family of a partly typed word: the one family every matching name
/// shares, else the family of the exact word.
fn family_for_partial(token: &str) -> Option<CommandFamily> {
    let lower = token.to_lowercase();
    let mut families: Vec<CommandFamily> = Vec::new();
    for (name, word) in registry::all_names() {
        if name.to_lowercase().starts_with(&lower)
            && let Some(family) = word.family()
            && !families.contains(&family)
        {
            families.push(family);
        }
    }
    match families.as_slice() {
        [family] => Some(*family),
        _ => family_for_verb(&lower),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_prefix_infers_lum_mode() {
        assert_eq!(infer_mode("fixt"), LanguageMode::Lum);
        assert_eq!(sentence_preview("fixt"), Some("Lum: Selection sentence"));
    }

    #[test]
    fn cd_prefix_infers_tree_mode() {
        assert_eq!(infer_mode("cd sho"), LanguageMode::TreeShell);
        assert_eq!(
            sentence_preview("cd sho"),
            Some("TreeShell: Navigation sentence")
        );
    }

    #[test]
    fn set_is_a_setting_sentence() {
        assert_eq!(family_for_verb("set"), Some(CommandFamily::Settings));
        assert_eq!(sentence_preview("set user"), Some("Lum: Setting sentence"));
    }

    #[test]
    fn assign_is_an_assignment_sentence() {
        assert_eq!(family_for_verb("assign"), Some(CommandFamily::Assignment));
        assert_eq!(
            sentence_preview("assign universe 1"),
            Some("Lum: Object assignment sentence")
        );
    }

    #[test]
    fn slash_modifier_changes_sentence_preview() {
        assert_eq!(
            sentence_preview("ls /r"),
            Some("TreeShell: Navigation sentence with slash modifier")
        );
        assert_eq!(
            slash_modifier_preview("ls /r"),
            Some("Slash modifier: recursive option")
        );
    }

    #[test]
    fn slash_modifier_detects_save() {
        assert_eq!(
            sentence_preview("show \"demo\" /save"),
            Some("Lum: Inspection sentence with slash modifier")
        );
        assert_eq!(
            slash_modifier_preview("show \"demo\" /save"),
            Some("Slash modifier: save option")
        );
    }
}
