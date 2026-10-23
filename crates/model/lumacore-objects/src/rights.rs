//! The one rights gate (decisions §19 "Step 5 hardening", §30 "12d
//! hardening"): every command line from every transport — Shell, UDP, HTTP,
//! OSC, keys, macros — passes `dispatch`, which checks here what the line
//! needs against the acting user's level (the ladder: Guest < Playback <
//! Presets < Program < Setup < Admin, one level per user, only from the
//! User — §30 "12c2").
//!
//! What a line needs is **declared**: the command's `rights` (and its
//! `arg_rights` for `Claim Root.ShowData.Devices`, `/Force`, `Clear Programmer User`), the
//! object's `rights` for the verb when the line names an object (`Store
//! View` is Playback, `Store Group` Program, `Store Output` Setup, `Store
//! User` Admin), and a property's level for `Set`. Help, completion,
//! suggestions, the reference and the sweep read the same declarations.
//! Only rules that depend on who owns what (another user's claim, password
//! or colour, a profile you don't use) are checked by their handlers, with
//! [`require`].

use super::commands::registry::{self, Rights, Word};
use super::errors::render_code;
use super::objects::{self, ObjectDef, Verb};
use super::{AppState, INTERNAL_ROOT_USER};

/// The acting user's level: the internal root user may do anything, a user
/// missing from the show counts as a Guest. Rights come from the User only.
pub fn level(state: &AppState) -> Rights {
    let user = state.effective_user_name();
    if user == INTERNAL_ROOT_USER {
        return Rights::Internal;
    }
    state
        .root
        .show_data
        .rights_of(user)
        .map_or(Rights::Guest, Rights::from)
}

/// The acting user has at least `needed`.
pub fn at_least(state: &AppState, needed: Rights) -> bool {
    level(state) >= needed
}

/// The refusal [E-RIGHTS-001]: what was refused, the level it needs and
/// the user's own ("'Claim Root.ShowData.Devices' needs Setup — you are Program.").
pub fn refusal(what: &str, needed: Rights, have: Rights) -> String {
    if needed == Rights::Internal {
        return render_code(
            format!("'{what}' is reserved for the internal root monitor."),
            "E-RIGHTS-001",
        );
    }
    render_code(
        format!("'{what}' needs {} — you are {}.", needed.label(), have.label()),
        "E-RIGHTS-001",
    )
}

/// A handler's check of an ownership rule (another user's claim, password
/// or colour): Ok, or the coded refusal naming both levels.
pub fn require(state: &AppState, needed: Rights, what: &str) -> Result<(), String> {
    let have = level(state);
    if have >= needed {
        Ok(())
    } else {
        Err(refusal(what, needed, have))
    }
}

/// What a verb on `obj` needs, with `args` after the object word: the
/// object's declared level, raised by a `Set` property's own level.
fn object_level(obj: &ObjectDef, verb: Verb, args: &[&str]) -> Rights {
    let mut needed = (obj.rights)(verb, args);
    // The property's own level, after the rows (`Set Output 1 Thru 3 Rate
    // 44`; a keyed setting like `Set Station Name …` too).
    if verb == Verb::Set
        && !set_checked_elsewhere(obj, args)
        && let Some(prop) = crate::commands::set::prop_rights(obj, args)
    {
        needed = needed.max(prop);
    }
    needed
}

/// Whether a family says this `Set` is checked where it is written (`Set Station "Tower" …`: the
/// station that owns it checks its own level, decisions §30 "Remote station edits").
fn set_checked_elsewhere(obj: &ObjectDef, args: &[&str]) -> bool {
    crate::commands::hooks::hooks().any(|h| matches!(h, crate::commands::hooks::Hook::SetCheckedElsewhere(o, f) if std::ptr::eq(*o, obj) && f(args)))
}

/// The level a statement starting with `word` needs, and what it names
/// (`Store Group`, `Claim`). A bare number (a Fixture-context shortcut)
/// needs Playback; another unknown word passes, and dispatch refuses it as
/// unknown (E-CMD-001).
pub fn needed(word: &str, args: &[&str]) -> (Rights, String) {
    match registry::resolve(word) {
        // Typed alone it sets your default object: anyone's (decisions §33 21).
        Ok(Word::Object(obj)) if args.is_empty() && obj.lum.counted() => (Rights::Guest, obj.keyword.to_string()),
        Ok(Word::Object(obj)) => (object_level(obj, crate::objects::bare_verb(obj, args), args), obj.keyword.to_string()),
        Ok(Word::Command(c)) => {
            let (base, what) = match (c.object_verb, args.first().and_then(|a| objects::find(a))) {
                (Some(verb), Some(obj)) => (
                    object_level(obj, verb, &args[1..]),
                    format!("{} {}", c.label(), obj.keyword),
                ),
                _ => (c.rights, c.label().to_string()),
            };
            let more = c.arg_rights.map_or(Rights::Guest, |f| f(args));
            (base.max(more), what)
        }
        // A bare number is a context shortcut (select / set a fixture);
        // any other unknown word is refused by dispatch as unknown.
        Err(_) if word.parse::<u32>().is_ok() => (Rights::Playback, word.to_string()),
        Err(_) => (Rights::Guest, word.to_string()),
    }
}

/// May a user at `have` run a bare `word`? (Suggestions mark what's out of
/// reach.) The refusal names both levels.
pub fn authorize(have: Rights, word: &str) -> Result<(), String> {
    let (needed, what) = needed(word, &[]);
    if have >= needed {
        return Ok(());
    }
    Err(refusal(&what, needed, have))
}

/// The gate `dispatch` runs before a word's handler, for a bare word.
pub fn check(state: &AppState, word: &str) -> Result<(), String> {
    check_statement(state, word, &[])
}

/// The gate for a whole statement: what [`needed`] says against the acting
/// user's [`level`]. The internal root user may do anything but Internal
/// words are for it alone.
pub fn check_statement(state: &AppState, word: &str, args: &[&str]) -> Result<(), String> {
    let have = level(state);
    if have == Rights::Internal {
        return Ok(());
    }
    let (needed, what) = needed(word, args);
    if have >= needed {
        return Ok(());
    }
    Err(refusal(&what, needed, have))
}

// ── Declared levels of a line's arguments (`CommandDef::arg_rights`) ───────

/// `Claim`: `/Force` takes from another user (Admin); a whole area (`Claim
/// Patch`: the full patch, the lines, the station) is Setup, the datapools
/// Program.
pub fn claim_rights(args: &[&str]) -> Rights {
    if crate::commands::flags::has(args, &crate::commands::flags::FORCE) {
        return Rights::Admin;
    }
    args.first()
        .and_then(|a| super::claims::area_rights(a))
        .unwrap_or(Rights::Guest)
}

/// `Clear Programmer [User] <n>` clears other users' programmers (Setup,
/// decisions §4 / §10); a bare `Clear` clears the terminal.
pub fn clear_rights(args: &[&str]) -> Rights {
    if args.first().is_some_and(|a| objects::PROGRAMMER.matches(a)) {
        Rights::Setup
    } else {
        Rights::Guest
    }
}

/// `SaveShow` over the same file is Presets; a new name or a flag
/// (`/Overwrite`) is Program.
pub fn saveshow_rights(args: &[&str]) -> Rights {
    if args.is_empty() { Rights::Guest } else { Rights::Program }
}

// ── Object levels (`ObjectDef::rights`) ─────────────────────────────────────
//
// Each object declares the level of every verb on it. These are the shared
// shapes; an object with its own rules (User, UserProfile, Show, DeskLock)
// has its own function next to it.

/// The reading verbs: anyone may look. `EditSetting` only opens the editor —
/// every `Set` it sends is gated by its property's level, and a cell above
/// the user's level shows greyed with the reason (decisions §30 "12d").
fn reads(verb: Verb) -> bool {
    matches!(verb, Verb::List | Verb::Info | Verb::Dump | Verb::EditSetting)
}

/// Show content programmers build (groups, sequences, cues, macros,
/// effects, images, assets): reading is anyone's, calling it runs the show
/// (Playback), changing it is Program.
pub fn content(verb: Verb, _: &[&str]) -> Rights {
    match verb {
        v if reads(v) => Rights::Guest,
        Verb::Call | Verb::SelectFixtures | Verb::Select | Verb::Goto | Verb::Go | Verb::At | Verb::On | Verb::Off | Verb::Toggle | Verb::Pause => {
            Rights::Playback
        }
        _ => Rights::Program,
    }
}

/// Presets: content, but `Update` and `Store … /Merge` on an existing
/// preset are the Presets level — creating one stays Program (the
/// handler refuses a /Merge that would create).
pub fn presets(verb: Verb, args: &[&str]) -> Rights {
    match verb {
        Verb::Update => Rights::Presets,
        Verb::Store if crate::commands::flags::has(args, &crate::commands::flags::MERGE) => Rights::Presets,
        _ => content(verb, args),
    }
}

/// Fixtures: select and knock in (Playback), the live patch — name, note,
/// address, claim, edit (Program), the full patch — create, delete, move,
/// renumber (Setup, Patch vs Live Patch).
pub fn fixture(verb: Verb, args: &[&str]) -> Rights {
    match verb {
        Verb::Set | Verb::Label | Verb::Claim | Verb::Edit | Verb::Export => Rights::Program,
        Verb::Call | Verb::SelectFixtures | Verb::At | Verb::On | Verb::Off | Verb::Toggle | Verb::List | Verb::Info | Verb::Dump => {
            content(verb, args)
        }
        _ => Rights::Setup,
    }
}

/// The rig, the station and the LumaSession (universes, fixture types,
/// networks, lines, routes, interfaces, stations, asset roots): anyone may
/// look, changing is Setup. A `Selection` knock in / out is Playback.
pub fn setup(verb: Verb, _: &[&str]) -> Rights {
    match verb {
        v if reads(v) => Rights::Guest,
        Verb::Call | Verb::Select => Rights::Guest,
        Verb::On | Verb::Off | Verb::Toggle => Rights::Playback,
        _ => Rights::Setup,
    }
}

/// The user's own screen (windows, views, view buttons, screens, screen
/// configs, window presets): calling a view or a screen is anyone's,
/// arranging them is Playback.
pub fn screen(verb: Verb, _: &[&str]) -> Rights {
    match verb {
        v if reads(v) => Rights::Guest,
        Verb::Call | Verb::Select => Rights::Guest,
        _ => Rights::Playback,
    }
}

/// Playback and the modes (sequences' Go, masters, Blind, Highlight,
/// Freeze, Solo): reading is anyone's, everything else Playback.
pub fn playback(verb: Verb, _: &[&str]) -> Rights {
    if reads(verb) { Rights::Guest } else { Rights::Playback }
}

/// Objects anyone may use: reading, the device (lock, theme, full screen).
pub fn anyone(_: Verb, _: &[&str]) -> Rights {
    Rights::Guest
}

/// Every object's declared level per verb, for the reference ("Store
/// Program · Delete Setup").
pub fn object_levels(obj: &ObjectDef) -> Vec<(Verb, Rights)> {
    obj.verbs
        .iter()
        .map(|(v, _)| *v)
        .chain(obj.planned.iter().copied())
        .map(|v| (v, (obj.rights)(v, &[])))
        .collect()
}

/// An object's levels for help, grouped lowest first ("Guest: List, Info ·
/// Playback: Call · Program: Store, Delete").
pub fn describe_object(obj: &ObjectDef) -> String {
    let levels = object_levels(obj);
    let mut out: Vec<String> = Vec::new();
    let mut seen: Vec<Rights> = levels.iter().map(|(_, r)| *r).collect();
    seen.sort();
    seen.dedup();
    for r in seen {
        let verbs: Vec<&str> = levels.iter().filter(|(_, l)| *l == r).map(|(v, _)| v.label()).collect();
        out.push(format!("{}: {}", r.label(), verbs.join(", ")));
    }
    if out.is_empty() { Rights::Guest.label().to_string() } else { out.join(" · ") }
}
