// ██████╗ ██╗      █████╗ ██╗   ██╗██████╗  █████╗  ██████╗██╗  ██╗
// ██╔══██╗██║     ██╔══██╗╚██╗ ██╔╝██╔══██╗██╔══██╗██╔════╝██║ ██╔╝
// ██████╔╝██║     ███████║ ╚████╔╝ ██████╔╝███████║██║     █████╔╝
// ██╔═══╝ ██║     ██╔══██║  ╚██╔╝  ██╔══██╗██╔══██║██║     ██╔═██╗
// ██║     ███████╗██║  ██║   ██║   ██████╔╝██║  ██║╚██████╗██║  ██╗
// ╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
// FILE: playback

//! The command rows of playback: Go, Go-, Goto, Top, Pause, Stop, Stomp, Shuffle (decisions §34
//! 25.7, 25.13: each crate of 34g4 takes its file).
//!
//! Owns:          the playback family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use crate::commands::registry::*;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This family's commands with their handlers; `commands::register` adds them to the catalog.
pub(crate) static ROWS: &[CommandDef] = &[
    CommandDef {
        run: |s, a| match playback::parse_target(a, &crate::objects::SEQUENCE) {
            Ok(t) => out(playback::go(s, t)),
            Err(e) => out(e),
        },
        decl: CommandDecl {
            verb: "go",
            doc: Doc {
                display: "Go",
                category: HelpCategory::Playback,
                syntax: "Go [Sequence <id>]",
                description: "Go to the next cue of a sequence (the first one when it is off). Without a \
                          sequence: your selected sequence (none selected: E-PLAY-003). Each cue part \
                          fades by its own timing (Set Cue <n> Fade|OutFade|Delay|OutDelay).",
                examples: &["Go", "Go Sequence 2"],
                related: &["goback", "goto", "sequence"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            object_verb: Some(Verb::Go),
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            default_object: Some("Sequence"),
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| match playback::parse_target(a, &crate::objects::SEQUENCE) {
            Ok(t) => out(playback::go_back(s, t)),
            Err(e) => out(e),
        },
        decl: CommandDecl {
            verb: "goback",
            aliases: &["go-"],
            doc: Doc {
                display: "Go-",
                category: HelpCategory::Playback,
                syntax: "Go- [Sequence <id>]",
                description: "Go back to the previous cue of a running sequence (its tracked state is asserted again).",
                examples: &["Go-", "Go- Sequence 2"],
                related: &["go", "goto"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            default_object: Some("Sequence"),
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<playback::GotoCue>,
        decl: CommandDecl {
            verb: "goto",
            doc: Doc {
                display: "Goto",
                category: HelpCategory::Playback,
                syntax: "Goto [Sequence <n>] Cue <m>",
                description: "Jump to a cue (owner first; no Sequence = your selected one). Every value \
                          that tracks into it is asserted.",
                examples: &["Goto Cue 3", "Goto Sequence 2 Cue 1.5"],
                related: &["go", "cue"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            object_verb: Some(Verb::Goto),
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            default_object: Some("Cue"),
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(playback::top(s, a)),
        decl: CommandDecl {
            verb: "top",
            hold: true,
            doc: Doc {
                display: "Top",
                category: HelpCategory::Playback,
                syntax: "Top On|Off Sequence <id>",
                description: "Echoes (decisions §32): Top On starts an echo of the sequence — a run from \
                          CueZero to its first cue with the cue's timing, beside the live playback \
                          (which still takes Go / Off). Top Off plays that echo's OffCue (yours, else \
                          the newest). Held pool cells and executor buttons send On at press, Off at \
                          release; several holds overlap (intensity HTP, the rest LTP). MaxEchoes (8; \
                          0 = none) refuses the newest Top On when full; EchoMax (30 s) ends an echo by \
                          itself; a holder that goes away ends its echo. Off Sequence ends them all.",
                examples: &["Top On Sequence 3", "Top Off Sequence 3"],
                related: &["go", "goto", "off", "sequence"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: run::pause,
        decl: CommandDecl {
            verb: "pause",
            doc: Doc {
                display: "Pause",
                category: HelpCategory::Playback,
                syntax: "Pause [Sequence <id>] | Pause LumaSession",
                description: "Pause the currently running cue fade in a sequence. \
                              `Pause LumaSession` stops the countdown of a LumaSession decision \
                              (the merge prompt waits for you).",
                examples: &["Pause", "Pause Sequence 1", "Pause LumaSession"],
                related: &["go", "stop"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            object_verb: Some(Verb::Pause),
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| match playback::parse_target(a, &crate::objects::SEQUENCE) {
            Ok(t) => out(playback::off(s, t)),
            Err(e) => out(e),
        },
        decl: CommandDecl {
            verb: "stop",
            doc: Doc {
                display: "Stop",
                category: HelpCategory::Playback,
                syntax: "Stop [Sequence <id>]",
                description: "Stop a running sequence; its values release (same as Off Sequence).",
                examples: &["Stop", "Stop Sequence 1"],
                related: &["go", "pause"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            rights: Rights::Playback,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "stomp",
            doc: Doc {
                display: "Stomp",
                category: HelpCategory::Playback,
                syntax: "Stomp Fixture <sel> | Group <n>",
                description: "Stop every effect on these fixtures, holding the value they show (decisions §28). Planned with Effects.",
                examples: &["Stomp Fixture 3"],
                related: &["effect"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "unstomp",
            doc: Doc {
                display: "Unstomp",
                category: HelpCategory::Playback,
                syntax: "Unstomp Fixture <sel> | Group <n>",
                description: "Let stomped effects run again (decisions §28). Planned with Effects.",
                examples: &["Unstomp Fixture 3"],
                related: &["effect"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "ifstomp",
            doc: Doc {
                display: "IfStomp",
                category: HelpCategory::Playback,
                syntax: "IfStomp",
                description: "Select the fixtures whose effects are stomped (decisions §28). Planned with Effects.",
                examples: &["IfStomp"],
                related: &["effect"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "disable",
            doc: Doc {
                display: "Disable",
                category: HelpCategory::Playback,
                syntax: "Disable Effect <n>[.<line>] Fixture <sel>",
                description: "Take fixtures out of an effect line without changing the cue (decisions §28). Planned with Effects.",
                examples: &["Disable Effect 5.2 Fixture 3"],
                related: &["effect"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "enable",
            doc: Doc {
                display: "Enable",
                category: HelpCategory::Playback,
                syntax: "Enable Effect <n>[.<line>] Fixture <sel>",
                description: "Put disabled fixtures back into an effect line (decisions §28). Planned with Effects.",
                examples: &["Enable Effect 5.2 Fixture 3"],
                related: &["effect"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: planned_run,
        decl: CommandDecl {
            verb: "shuffle",
            doc: Doc {
                display: "Shuffle",
                category: HelpCategory::Playback,
                syntax: "Shuffle [<seed>]",
                description: "Shuffle the selection order with a seed (Pawtern, decisions §3). Planned with Effects.",
                examples: &["Shuffle 7"],
                related: &["effect"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Playback),
            scope: Scope::Master,
            status: Status::Planned,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::objects::playback::SEQUENCE,
    &crate::objects::playback::CUE,
    &crate::objects::presets::TIMING_PRESET,
    &crate::objects::presets::CURVE_PRESET,
    &crate::objects::playback::EXECUTOR,
    &crate::objects::playback::MASTER,
    &crate::objects::playback::EFFECT,
    &crate::objects::playback::MACRO,
    &crate::objects::playback::TIMECODE,
    &crate::objects::playback::EFFECT_ENGINE,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[];
