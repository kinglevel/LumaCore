//! `Self` — the object a command line belongs to (decisions §30 "12c
//! hardening"; one registry word, [`SELF`]).
//!
//! The line carries an owner ([`LineOwner`]): a window bar, window menu or
//! settings-window line names its window (the shell packet's `window`); a
//! UiKey placed in a window bar names that window; a key pressed on the
//! keyboard and a typed line belong to the client's focused window (never the
//! key itself). Macros and plugins are owners too, wired here — until their
//! runtimes exist they answer E-CMD-016, like a line with no owner (a
//! terminal without a focused window): never a guess.
//!
//! `Self` is **resolved on the station where the line starts**, before the
//! line can go to a LumaSession master (which can't see this station's
//! focus): [`resolve_line`] rewrites the word to `Window <screen>.<n>`, so the
//! forwarded line, the journal, Oops and the System Monitor show the window.
//! Macros, keys and Usage keep the `Self` text. The `Self` object's verbs
//! route to the Window object's (for a line that reaches them unresolved).

use crate::commands::registry::{AliasDepth, CommandClass, Doc, Undo};
use crate::commands::help::HelpCategory;
use crate::errors::render_code;
use crate::objects::{ObjectDef, Verb, out};
use crate::{AppState, DispatchResult};

pub use crate::app_state::LineOwner;

/// The one reply when `Self` has nothing to name.
pub fn no_owner() -> String {
    render_code(
        "Self has no owner here — type a window (EditSetting Window 1.3)",
        "E-CMD-016",
    )
}

/// The focused window of the client running the line, if it exists.
fn focused(state: &AppState) -> Option<(u32, u32)> {
    let (screen, no) = (state.current_live_screen?, state.current_window?);
    window_exists(state, screen, no).then_some((screen, no))
}

fn window_exists(state: &AppState, screen: u32, no: u32) -> bool {
    state
        .root
        .runtime
        .live_screens
        .read()
        .ok()
        .is_some_and(|s| s.get(screen).is_some_and(|s| s.windows.contains_key(&no)))
}

/// The window `Self` names for the running line: (live screen, window).
/// Err is the rendered E-CMD-016.
pub fn owner_window(state: &AppState) -> Result<(u32, u32), String> {
    match &state.line_owner {
        Some(LineOwner::Window { screen, no }) if window_exists(state, *screen, *no) => Ok((*screen, *no)),
        Some(LineOwner::Window { screen, no }) => Err(render_code(
            format!("Self is Window {screen}.{no}, which is gone"),
            "E-CMD-016",
        )),
        // A key from the keyboard acts on the focused window, never on itself.
        Some(LineOwner::Key(_)) | None => focused(state).ok_or_else(no_owner),
        Some(LineOwner::Macro(_)) | Some(LineOwner::Plugin(_)) => Err(no_owner()),
    }
}

/// `Window <screen>.<n>` of the line's owner.
pub fn owner_word(state: &AppState) -> Result<String, String> {
    owner_window(state).map(|(s, n)| format!("Window {s}.{n}"))
}

/// The line with every unquoted `Self` word replaced by its window
/// (`Set Self "Universe" + 1` → `Set Window 1.4 "Universe" + 1`). None when the
/// line has no `Self`; Err (E-CMD-016) when it has one and no owner.
pub fn resolve_line(state: &AppState, line: &str) -> Result<Option<String>, String> {
    use crate::commands::tokens::{Kind, lex};
    let Ok(tokens) = lex(line) else {
        return Ok(None);
    };
    let spans: Vec<(usize, usize)> = tokens
        .iter()
        .filter(|t| t.kind == Kind::Word { quoted: false } && SELF.matches(&t.text))
        .map(|t| (t.start, t.end))
        .collect();
    if spans.is_empty() {
        return Ok(None);
    }
    let word = owner_word(state)?;
    let mut out = String::with_capacity(line.len() + 8);
    let mut from = 0;
    for (start, end) in spans {
        out.push_str(&line[from..start]);
        out.push_str(&word);
        from = end;
    }
    out.push_str(&line[from..]);
    Ok(Some(out))
}

/// `<verb> Self …` → the Window object's verb on the owner's window.
fn on_window(state: &mut AppState, verb: Verb, args: &[&str]) -> DispatchResult {
    let (screen, no) = match owner_window(state) {
        Ok(w) => w,
        Err(e) => return out(e),
    };
    let address = format!("{screen}.{no}");
    let mut full: Vec<&str> = vec![&address];
    full.extend_from_slice(args);
    match crate::commands::windows::WINDOW.handler(verb) {
        Some(h) => h(state, &full),
        None => out(render_code(format!("Window doesn't take {}.", verb.label()), "E-CMD-004")),
    }
}

pub static SELF: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Self",
        lum: crate::objects::LumShape::ONE,
        default_verb: crate::objects::Verb::Select,
        flags: &[],
        aliases: &[],
        doc: Doc {
            display: "Self",
            category: HelpCategory::System,
            also_applies_to: &[],
            syntax: "Self | Set Self <setting> <value>|+ n|- n|Default|Toggle | EditSetting Self | Label|Delete|Reset Self",
            description: "The object the command line belongs to: a window. Typed on a command line it is \
                          the focused window; a window bar, window menu or settings-window line is that \
                          window; a key in a window bar is its window, a key from the keyboard the focused \
                          window. Resolved where the line starts — the journal, Oops and System Monitor \
                          show `Window <screen>.<n>`. No owner (a terminal without a focused window) → \
                          E-CMD-016. Macros and plugins become owners when they run.",
            examples: &[
                "Set Self \"Universe\" + 1        -- the DMX window's next universe",
                "Set Self \"Since\" Now           -- System Monitor: clear this window",
                "Reset Self \"WindowBarItems\"   -- the bar back to the type's",
                "EditSetting Self               -- this window's settings",
                "Reset Self                     -- every setting and the bar back to the defaults",
            ],
            related: &["window", "windowpreset", "editsetting"],
            alias_depth: AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: CommandClass::Commit,
        scope: crate::objects::on_station,
        reach: crate::objects::show_reach,
        rights: crate::rights::screen,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Select, |s, a| on_window(s, Verb::Select, a)),
        (Verb::Label, |s, a| on_window(s, Verb::Label, a)),
        (Verb::Delete, |s, a| on_window(s, Verb::Delete, a)),
        (Verb::Reset, |s, a| on_window(s, Verb::Reset, a)),
    ],
    delete_ask: None,
    complete: None,
    props: None,
    tree: None,
};
