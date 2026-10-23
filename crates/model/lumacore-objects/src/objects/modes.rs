//! Mode objects — Blind, Highlight, Freeze, Solo (decisions §16).
//!
//! Verb-first like every other object: `On Blind`, `Off Highlight`,
//! `Toggle Freeze`; the bare word toggles (`Blind`, like pressing the key).
//! The state is per user (`Root.Runtime.Users.{User}.Modes`), shown in the
//! status bar and on key LEDs. The output effects come with the playback
//! engine (step 4, §27); until then Highlight and Solo drive the old overlay.

use lumacore_tree::UserMode;

use super::{Doc, ObjectDef, Undo, Verb, out};
use crate::commands::registry::CommandClass;
use crate::AppState;
use crate::commands::help::HelpCategory;
use crate::commands::registry::{self, AliasDepth, Word};
use crate::commands::registry::CommandFamily;
use crate::text::render_success;

/// `Toggle Blind` and the bare `Blind` (its default verb, decisions §33 40); a word after it
/// (`Blind On`, the postfix form) points at the verb-first one.
fn toggle(state: &mut AppState, mode: UserMode, args: &[&str]) -> crate::DispatchResult {
    let Some(first) = args.first() else {
        return out(switch(state, mode, Switch::Toggle));
    };
    let verb = match registry::resolve(first) {
        Ok(Word::Command(c)) => c.object_verb,
        _ => None,
    };
    let hint = match verb {
        Some(v @ (Verb::On | Verb::Off | Verb::Toggle)) => {
            format!("Use `{} {}` (verb first).", v.label(), mode.name())
        }
        _ => format!("{} takes no arguments: `{0}`, `On {0}`, `Off {0}`, `Toggle {0}`.", mode.name()),
    };
    out(crate::errors::render_code(hint, "E-CMD-005"))
}

/// What a mode verb does.
#[derive(Clone, Copy)]
enum Switch {
    On,
    Off,
    Toggle,
}

/// Switch `mode` for the acting user.
fn switch(state: &mut AppState, mode: UserMode, how: Switch) -> String {
    let user = state.effective_user_name().to_string();
    let modes = &mut state.root.runtime.user(&user).modes;
    let on = match how {
        Switch::On => true,
        Switch::Off => false,
        Switch::Toggle => !modes.get(mode),
    };
    let was = modes.get(mode);
    modes.set(mode, on);
    // The render engine applies the modes (decisions §27).
    let detail = match mode {
        // Leaving Blind is a fresh touch: the user's values take over (§4).
        UserMode::Blind if was && !on => {
            crate::programmers::take_over(state, None);
            None
        }
        UserMode::Highlight => {
            on.then(|| " (mode only — output follows with the GDTF highlight values)".to_string())
        }
        _ => None,
    };
    render_success(&format!(
        "{} {}{}",
        mode.name(),
        if on { "On" } else { "Off" },
        detail.unwrap_or_default()
    ))
}

/// `List Blind` / `Info Blind` — the acting user's state.
fn show(state: &AppState, mode: UserMode) -> String {
    let on = state
        .root
        .runtime
        .user_ref(state.effective_user_name())
        .is_some_and(|u| u.modes.get(mode));
    format!(
        "{} is {} for {}",
        mode.name(),
        if on { "On" } else { "Off" },
        state.effective_user_name()
    )
}

macro_rules! mode_object {
    ($static:ident, $mode:expr, $label:literal, [$($alias:literal),*], $desc:literal, $examples:expr) => {
        pub static $static: ObjectDef = ObjectDef {
            decl: crate::objects::ObjectDecl {
                keyword: $label,
                lum: crate::objects::LumShape::ONE,
                default_verb: crate::objects::Verb::Toggle,
                flags: &[],
                aliases: &[$($alias),*],
                doc: Doc {
                    display: $label,
                    category: HelpCategory::Programmer,
                    also_applies_to: &[],
                    syntax: concat!($label, " | On ", $label, " | Off ", $label, " | Toggle ", $label),
                    description: $desc,
                    examples: $examples,
                    related: &["blind", "highlight", "freeze", "solo"],
                    alias_depth: AliasDepth::Base,
                    hidden: false,
                },
                family: Some(CommandFamily::Action),
                call_undo: Undo::ReadOnly,
                call_class: CommandClass::Commit,
                scope: crate::objects::on_master,
                reach: crate::objects::show_reach,
                rights: crate::rights::playback,
                planned: &[],
            },
            pool: None,
            verbs: &[
                (Verb::On, |s, _| out(switch(s, $mode, Switch::On))),
                (Verb::Off, |s, _| out(switch(s, $mode, Switch::Off))),
                (Verb::Toggle, |s, a| toggle(s, $mode, a)),
                (Verb::List, |s, _| out(show(s, $mode))),
                (Verb::Info, |s, _| out(show(s, $mode))),
            ],
            delete_ask: None,
            complete: None,
            props: None,
            tree: None,
        };
    };
}

mode_object!(
    BLIND,
    UserMode::Blind,
    "Blind",
    [],
    "Program without changing the live output: while Blind is on, your \
     programmer stays out of the DMX output. Per user. The output effect comes with the \
     render engine; today Blind keeps the state, status and key LED.",
    &[
        "Blind         -- toggle",
        "On Blind      -- switch on",
        "Off Blind     -- switch off"
    ]
);
mode_object!(
    HIGHLIGHT,
    UserMode::Highlight,
    "Highlight",
    ["hi"],
    "Bring your selected fixtures to their highlight values (from their GDTF, \
     `DMXChannel@Highlight`) without touching the programmer; a channel \
     with no highlight value stays as it is. Above programmers and cues, below park; the \
     masters still scale intensity. Per user; `Master \"Highlight\"` sets its strength.",
    &[
        "Highlight     -- toggle",
        "On Highlight  -- switch on",
        "Off Highlight -- switch off"
    ]
);
mode_object!(
    FREEZE,
    UserMode::Freeze,
    "Freeze",
    [],
    "Freeze your programmer's place in the output merge (how it ranks \
     against the playbacks is settled with the render engine, decisions §7/§27). \
     Per user. Today Freeze keeps the state, status and key LED.",
    &[
        "Freeze        -- toggle",
        "On Freeze     -- switch on",
        "Off Freeze    -- switch off"
    ]
);
mode_object!(
    SOLO,
    UserMode::Solo,
    "Solo",
    [],
    "Show only the selected fixtures: the others go dark without touching the programmer \
     Per user; `Master \"Solo\"` sets how far they go down.",
    &[
        "Solo          -- toggle",
        "On Solo       -- switch on",
        "Off Solo      -- switch off"
    ]
);
