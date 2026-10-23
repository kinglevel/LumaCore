//! Error codes of the server and the command line, and the one list of every
//! catalog (`docs/meta/decisions.md` §26). `Info Error <code>` and
//! `List Error` read [`CATALOGS`].

use lumacore_tree::{Coded, ErrorInfo};

/// Command-line and server errors.
pub const ERRORS: &[ErrorInfo] = &[
    ErrorInfo {
        code: "E-CMD-001",
        title: "Unknown word",
        explain: "The first word is not a command or object (nor an unambiguous short form of one). `help` lists them; the message suggests the closest names.",
    },
    ErrorInfo {
        code: "E-CMD-002",
        title: "Unterminated quote",
        explain: "A quoted name was opened but not closed. Close the quote or remove it.",
    },
    ErrorInfo {
        code: "E-CMD-003",
        title: "Not built yet (planned)",
        explain: "The word (or this verb of an object) is published as planned: it is in help, completion and the reference, but its behaviour is not built yet. It changes nothing (decisions §29).",
    },
    ErrorInfo {
        code: "E-CMD-004",
        title: "Object doesn't take this verb",
        explain: "The object does not support the verb used with it. The message lists the verbs it takes.",
    },
    ErrorInfo {
        code: "E-CMD-005",
        title: "Verb goes first",
        explain: "Mode objects take their verb first: `On Blind`, `Off Highlight`, `Toggle Freeze`, or the bare word to toggle (`Solo`). The postfix form `Blind On` is not used (decisions §16).",
    },
    ErrorInfo {
        code: "E-CMD-006",
        title: "Missing or wrong arguments",
        explain: "The word needs other arguments. The message shows its usage; `help <word>` and the reference (docs/user/reference/commands.md) show every form.",
    },
    ErrorInfo {
        code: "E-CMD-007",
        title: "No such object",
        explain: "Nothing exists with that number, name or path. `List <object>` (or `ls` in the tree) shows what does.",
    },
    ErrorInfo {
        code: "E-CMD-008",
        title: "Value not accepted",
        explain: "A number, name, time or option couldn't be read, or is out of range. The message says what is accepted.",
    },
    ErrorInfo {
        code: "E-CMD-009",
        title: "Nothing to act on",
        explain: "The command works on something that is empty right now: no fixtures selected, no programmer values, nothing to undo, nothing stored. Select or set something first.",
    },
    ErrorInfo {
        code: "E-CMD-010",
        title: "Not possible right now",
        explain: "The command can't run in the current state (the object is busy or in another mode, the station is a member, the word needs a different context). The message names what stands in the way.",
    },
    ErrorInfo {
        code: "E-CMD-011",
        title: "No user or profile",
        explain: "The command acts on the logged-in user's profile (or a named user) and there is none. Log in, or name an existing user (`List User`).",
    },
    ErrorInfo {
        code: "E-CMD-012",
        title: "File or disk error",
        explain: "Reading or writing a file failed (missing file, no permission, disk full, unreadable content). The message names the file and the system's reason.",
    },
    ErrorInfo {
        code: "E-CMD-013",
        title: "Only inside an editor",
        explain: "The word works only while an object is open in an editor (`Edit <object>`), on that object. Open the editor first.",
    },
    ErrorInfo {
        code: "E-CMD-014",
        title: "Select takes DataPool, Sequence or Timecode",
        explain: "Select sets a one-only default of its kind (decisions §29): the DataPool pool objects resolve in, the Sequence a bare Go acts on, the Timecode. Fixtures are selected with `Fixture <n>` or a group; other objects have no default.",
    },
    ErrorInfo {
        code: "E-CMD-015",
        title: "No settings",
        explain: "EditSetting opens an object's settings: what it declares writable (`Set <object> <n>` lists them). This object has none; `EditSetting Show` gives the show's.",
    },
    ErrorInfo {
        code: "E-CMD-016",
        title: "Self has no owner here",
        explain: "`Self` is the object the command line belongs to (decisions §30 \"12c\"): typed, the focused window; from a window bar, window menu or settings window, that window; a key in a window bar, its window. A terminal without a focused window (the CLI, a one-shot command) has none, and macros and plugins aren't owners until their runtimes exist. Name the window instead: `EditSetting Window 1.3`, `Set Window 1.4 \"Universe\" + 1`.",
    },
    ErrorInfo {
        code: "E-CMD-017",
        title: "Station busy",
        explain: "The station's state thread is on a long job (decisions §26, R1-67): a line from HTTP waits at most 10 s, from a shell 30 s. A line that didn't start didn't run — try again (HTTP answers 503); one that started finishes and reports in the System Monitor. Keys, taps and answers typed in a shell wait in order and run when it is free.",
    },
    ErrorInfo {
        code: "E-CMD-018",
        title: "Unknown option",
        explain: "A bare /word on the line is no option of its command or object (decisions §31): nothing in the line ran. The message names the options it takes; /NoConfirmation (/nc), /NoOops, /Verbose, /Debug and /Watch go on every line. A quoted \"/word\" is a value, not an option. An unknown option is never ignored silently; LumaCore refuses it, so a typo like /Overwite never stores with the default.",
    },
    ErrorInfo {
        code: "E-CMD-019",
        title: "Lines nested too deep",
        explain: "A line ran a line that ran a line … 32 deep (decisions §34 16): usually a key or a macro that runs itself, directly or through another. The innermost line didn't run; what ran before it stays (each statement is its own Oops step). Vim stops `:execute` loops the same way.",
    },
    ErrorInfo {
        code: "E-CMD-020",
        title: "Another kind",
        explain: "Copy, Move, Insert and Paste carry objects only into a pool of their own kind (decisions §35 6) — a Group into a DataPool's Groups, never into a Sequence; Unix refuses a rename across file systems the same way (EXDEV). Nothing moved. Import / Export carry objects between kinds of place; Paste takes what your clipboard holds.",
    },
    ErrorInfo {
        code: "E-PROG-001",
        title: "Fixture claimed by another user",
        explain: "`Claim Fixture <n>` locks a fixture for one user: nobody else can select it or set values for it until it is unclaimed (an Admin can unclaim anyone). Without a claim, programmers are LTP — the latest touch takes an attribute (decisions §4).",
    },
    ErrorInfo {
        code: "E-PROG-002",
        title: "Fixture is held by another programmer",
        explain: "A fixture can't be claimed while another user's programmer has a value for it that reaches the stage. Ask them to knock it out (`Off Fixture <n>`), or have an Admin clear it (`Clear Programmer User <n>`).",
    },
    ErrorInfo {
        code: "E-PLAY-001",
        title: "No such sequence or cue",
        explain: "The sequence or cue doesn't exist. `List Sequence` and `List Cue Sequence <n>` show what does. Cue numbers can have decimals (`Cue 1.5`).",
    },
    ErrorInfo {
        code: "E-PLAY-002",
        title: "Nothing to store",
        explain: "Store Cue records the active programmer values (only what changes from the tracked state). Set values first, or `On` to activate inactive ones.",
    },
    ErrorInfo {
        code: "E-PLAY-003",
        title: "No sequence selected",
        explain: "Go, Go-, Goto Cue and Off Sequence without `Sequence <n>` act on your selected sequence, and you have none (or it was deleted). `Select Sequence <n>` picks one, or name the sequence in the command. LumaCore never guesses Sequence 1. (Store Cue with none selected creates a new sequence after the highest and selects it.)",
    },
    ErrorInfo {
        code: "E-PLAY-004",
        title: "The cue exists — how to store?",
        explain: "Storing over an existing cue asks: Merge adds the programmer's values, Overwrite makes the cue exactly the programmer, Remove takes the programmer's attributes out, Create Second Cue stores a new cue right after it. A line from a key or macro answers with a flag: /Merge, /Overwrite, /Remove or /CreateSecondCue.",
    },
    ErrorInfo {
        code: "E-SET-001",
        title: "Property not found",
        explain: "The tree node has no property with that name. `Info <path>` lists its properties.",
    },
    ErrorInfo {
        code: "E-SET-002",
        title: "Property is read-only",
        explain: "The property can be read but not set: it is derived, an identity, or owned by the runtime. The message says why.",
    },
    ErrorInfo {
        code: "E-SET-003",
        title: "Invalid value",
        explain: "The value does not fit the property's type or range. The message shows what is accepted.",
    },
    ErrorInfo {
        code: "E-SET-004",
        title: "Path not found",
        explain: "No tree node lives at that path. `List <path>` and `cd` show what exists.",
    },
    ErrorInfo {
        code: "E-RIGHTS-001",
        title: "Not allowed at your rights level",
        explain: "Every user has one rights level (Guest < Playback < Presets < Program < Setup < Admin) and every command, object verb and property declares the level it needs; the message names it and yours. The same check runs for the Shell, the command port, HTTP, OSC, keys and macros (a line is refused whole). `Help <command>` shows the level; an Admin changes a user's level (`Set User \"x\" \"Rights\" Setup`).",
    },
    ErrorInfo {
        code: "E-RIGHTS-002",
        title: "Remote login is off for this user",
        explain: "The user may only log in on this station itself (Remote Login is off). Log in locally, or have an Admin turn Remote Login on for the user.",
    },
    ErrorInfo {
        code: "E-RIGHTS-003",
        title: "An OSC line needs a user",
        explain: "Commands from an OSC input line run as its user, with that user's rights level; a line without one would run as nobody (it never runs as Admin). Name one: `Store Input 2 \"Companion\" /Protocol Osc /Port 8000 /User \"LD\"` or `Set Input 2 \"User\" \"LD\"`. An existing line without a user doesn't start (the System Monitor says why).",
    },
    ErrorInfo {
        code: "E-GROUP-001",
        title: "Group exists",
        explain: "Storing or copying onto an existing group never overwrites it silently (decisions §22). Say what to do: `/Merge` adds the new fixtures (after the group's last grid X; `/GridMerge Off` keeps their own spots), `/Overwrite` replaces the members (the group keeps its GUID), `/Remove` takes the selected fixtures out.",
    },
    ErrorInfo {
        code: "E-GROUP-003",
        title: "Group not found",
        explain: "There is no group with that number or name in your selected datapool (`Group 3` is pool-relative, decisions §22). `List Group` shows them; `Select DataPool <n>` selects another datapool.",
    },
    ErrorInfo {
        code: "E-CLAIM-001",
        title: "Claimed by another user",
        explain: "Another user holds the object (or the object it lives in), for example because they opened it in an editor (decisions §21 \"6d\"). `Claim <object>` asks them for it: they can hand it over (`Unclaim`) or keep it (`Claim … /Keep`); without an answer within 10 s it is yours. An Admin can take it at once with `Claim … /Force`. `List Claims` shows who holds what.",
    },
    ErrorInfo {
        code: "E-USER-001",
        title: "Admin's password is fixed",
        explain: "The Admin user's password is always `admin` and can't be changed, so a power user can always log in when the operator has left (decisions §30 \"12a3\"; rigs are isolated networks). Give operators their own users with their own passwords.",
    },
    ErrorInfo {
        code: "E-USER-002",
        title: "Default is locked",
        explain: "UserProfile \"Default\" is the template every new profile starts from (decisions §30 \"12c2\", a locked profile): it can't be changed, renamed, imported over or deleted. Make a profile of your own (`Store UserProfile \"name\"` copies Default) and change that.",
    },
    ErrorInfo {
        code: "E-USER-003",
        title: "You can't delete yourself",
        explain: "The user you are logged in as can't be deleted from their own session. Log in as another Admin to delete it.",
    },
    ErrorInfo {
        code: "E-USER-004",
        title: "At least one Admin must remain",
        explain: "A show always has an Admin who can manage users and the station. Make another user Admin first (`Set User \"x\" \"Rights\" Admin`), then delete or lower this one.",
    },
    ErrorInfo {
        code: "E-USER-005",
        title: "Admin always exists",
        explain: "The user Admin (password always `admin`, rights always Admin) can't be deleted, renamed or given another level, so a power user can always log in (decisions §30 \"12a3\", \"12d\").",
    },
    ErrorInfo {
        code: "E-USER-006",
        title: "Profile is in use",
        explain: "Users work in this profile; the message names them. Give them another profile first (`Set User \"x\" \"Profile\" …`), then delete it.",
    },
    ErrorInfo {
        code: "E-USER-007",
        title: "Name taken",
        explain: "Another user (or profile) already has that name. Names are unique, ignoring case.",
    },
    ErrorInfo {
        code: "E-USER-008",
        title: "Nobody works in Default",
        explain: "UserProfile \"Default\" is only a template (decisions §30 \"12c2\"): no user may hold it. Give the user their own profile (`/Profile New`, a copy of Default) or share another one.",
    },
    ErrorInfo {
        code: "E-USER-009",
        title: "Reserved name",
        explain: "The name is reserved for the station itself (its internal runtime user, or the template profile Default). Pick another.",
    },
    ErrorInfo {
        code: "E-USER-010",
        title: "Accept the Meow License first",
        explain: "Every login asks the user to accept the Meow License before it is finished (decisions 0290): one dialog, held by the station. Until then the login runs only `Consent` (the dialog's \"I consent\"), `Logout` (\"Not me\") and `Login`; its HTTP token counts as no login. Unlocking a DeskLock is not a login and never asks.",
    },
    ErrorInfo {
        code: "E-LOCK-001",
        title: "This device is locked",
        explain: "DeskLock is on for this device (an app or a browser tab): the station runs nothing from it but the lock keys — F9 / Pause (DeskLock) and Meow+F9 (NormalTheme). Unlock it on its lock surface; shells, other devices, OSC and playback keep running.",
    },
    ErrorInfo {
        code: "E-LOCK-002",
        title: "Unlocking needs the password",
        explain: "The user on this device has a password (Admin always: `admin`), so F9, a tap or the corner taps don't unlock it: type the password on the lock surface.",
    },
    ErrorInfo {
        code: "E-LUMA-001",
        title: "LumaSession protocol differs",
        explain: "The station and the master speak different LumaSession stream protocols (different LumaCore versions). Run the same version on every station of a LumaSession (decisions §21 \"6b\").",
    },
    ErrorInfo {
        code: "E-LUMA-002",
        title: "Show format differs",
        explain: "The station and the master use different show formats (`format_version`). Members must read the master's show exactly, so a join needs the same format: run the same LumaCore version. A different build commit only warns.",
    },
    ErrorInfo {
        code: "E-LUMA-003",
        title: "That station can't take members",
        explain: "The station asked to be master has LumaSession off, is itself a member of another LumaSession (join its master instead), or has this station's name; or another station with the joining station's name is already in the session (connected or reconnecting), or the joining station sent no station id. Members are one per station name: give each station its own name (`Set Station \"Name\"`). The joining station keeps its show.",
    },
    ErrorInfo {
        code: "E-LUMA-004",
        title: "No link to the master",
        explain: "This member's link to its master is closed, so the command could not be sent. Leave LumaSession and join again. While the link is only reconnecting, commands wait in the outbox (marked pending) and run when the master confirms.",
    },
    ErrorInfo {
        code: "E-LUMA-005",
        title: "Merge isn't possible",
        explain: "A station rejoining with Merge was refused: the two sides hold different shows (show id), there is no show both had at the split (no split base), or both sides changed the patch since the split (decisions §21 \"6d\"). Choose Use Master (take the session's show, yours is autosaved), Use Mine, or Cancel (stay alone).",
    },
    ErrorInfo {
        code: "E-LUMA-006",
        title: "Another build",
        explain: "Every station of a LumaSession runs the exact same build — the same LumaCore version and commit (decisions §34 \"34f7b hardening\" 3): the stations replicate one show and must agree on its words, protocols and settings. The joining station runs another one; update it (or the master) so both run the same build. The message names both.",
    },
    ErrorInfo {
        code: "E-WIRE-001",
        title: "Client protocol differs",
        explain: "The client (CLI, desktop app) speaks another client protocol than the station (`lumacore_wire::CLIENT_PROTOCOL`): it was built from a different LumaCore version. The station refuses the attach; update the client. The app's server list shows such stations as \"Wrong version\" (decisions §23 \"Step 8\").",
    },
    ErrorInfo {
        code: "E-WIRE-002",
        title: "Reply too large for the command port",
        explain: "The answer to a request on the UDP command port doesn't fit in one UDP packet (65,507 bytes), so it wasn't sent. Run the line in a shell (the CLI, the web page's command line) instead of as a one-shot command (decisions §21: UDP keeps small messages, §31).",
    },
    ErrorInfo {
        code: "E-INTERNAL-001",
        title: "Internal error (panic)",
        explain: "A command crashed inside LumaCore. Its show changes were rolled back and the server kept running. The reply names the source file and line; the full backtrace is in the log under the report number. Please report it with that number.",
    },
];

/// The catalog entry for a code: every crate's error catalog is in the one catalog
/// (`lumacore_registry`, decisions §34 21) — each crate registers its own.
pub fn find(code: &str) -> Option<&'static ErrorInfo> {
    lumacore_registry::get().error(code)
}

/// Whether `text` is an error reply and its code: its first `Error:` line, the `[E-…]` at
/// its end (decisions §26). Only [`crate::Reply`]'s `From<String>` uses it — for
/// handlers that still answer with text (decisions §31 "typed replies").
pub fn outcome_of(text: &str) -> (bool, Option<String>) {
    let plain = crate::text::strip_ansi(text);
    let Some(error) = plain.lines().find(|l| l.trim_start().starts_with("Error:")) else {
        return (false, None);
    };
    let code = error
        .rsplit_once('[')
        .and_then(|(_, rest)| rest.trim_end().strip_suffix(']'))
        .filter(|c| c.starts_with("E-"))
        .map(str::to_string);
    (true, code)
}

/// `message [E-AREA-NNN]` (the formatter is the base's, decisions §34 25.10).
pub use lumacore_base::with_code;

/// A typed error as an error line with its code. The caller's location goes
/// to the log, so a code seen on screen can be traced to the line that
/// reported it.
#[track_caller]
pub fn render(err: &dyn Coded) -> String {
    let at = std::panic::Location::caller();
    log::debug!("{} reported at {}:{}", err.code(), at.file(), at.line());
    crate::text::render_error(&with_code(err, err.code()))
}

/// A command-line error with one of the codes above.
pub fn render_code(message: impl std::fmt::Display, code: &str) -> String {
    debug_assert!(!lumacore_registry::installed() || find(code).is_some(), "uncatalogued error code {code}");
    crate::text::render_error(&with_code(message, code))
}

/// An error reply for a message that may already be one: a message already
/// rendered as an error is passed on as it is, one that carries a code
/// (passed on from claims, a show load, the master) keeps it, anything else
/// gets `code`. Never "Error: Error:" and never two codes.
pub fn reply(message: impl std::fmt::Display, code: &str) -> String {
    let message = message.to_string();
    let plain = crate::text::strip_ansi(&message);
    if plain.starts_with("Error:") {
        message
    } else if plain.contains("[E-") {
        crate::text::render_error(&message)
    } else {
        render_code(message, code)
    }
}

/// [`reply`] as a typed [`crate::Reply`] (decisions §31): always failed; its code is the
/// one the passed-on message carries, else `code`.
pub fn reply_error(message: impl std::fmt::Display, code: &str) -> crate::Reply {
    let text = reply(message, code);
    let (_, carried) = outcome_of(&text);
    crate::Reply { text, failed: true, code: carried.or_else(|| Some(code.to_string())) }
}

/// `List Error [area]` — every code, or those of one area (`List Error Patch`).
pub fn list(args: &[&str]) -> String {
    use crate::text::PropertySheet;
    let area = args
        .first()
        .map(|a| a.trim_matches('"').to_ascii_uppercase());
    let mut sheet = PropertySheet::new("Error codes").key_width(16);
    let mut count = 0;
    for info in lumacore_registry::get().errors() {
        if let Some(area) = &area
            && !info.code[2..].starts_with(&format!("{area}-"))
        {
            continue;
        }
        sheet = sheet.field(info.code, info.title);
        count += 1;
    }
    if count == 0 {
        return render_code(
            format!("No error codes in area '{}'.", area.unwrap_or_default()),
            "E-SET-004",
        );
    }
    sheet
        .separator()
        .line("  Info Error <code> explains one.")
        .build()
}

/// `Info Error <code>` — what an error code means.
pub fn info(args: &[&str]) -> String {
    use crate::text::PropertySheet;
    let Some(code) = args.first() else {
        return list(&[]);
    };
    match find(code) {
        Some(info) => PropertySheet::new(info.code)
            .field("Title", info.title)
            .field("Meaning", info.explain)
            .build(),
        None => crate::text::render_error(&format!(
            "Unknown error code '{code}'. List Error shows all codes."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_crate_below_registers_its_catalog() {
            let catalog = lumacore_registry::get();
        for code in ["E-CMD-006", "E-SYN-001", "E-PATCH-002", "E-DMX-001", "E-GDTF-001"] {
            assert!(catalog.error(code).is_some(), "{code} isn't in the catalog");
        }
    }

    #[test]
    fn crate_errors_resolve_in_the_catalog() {
            let e = lumacore_patch::PatchError::InvalidAddress { address: 0 };
        let info = find(e.code()).expect("catalogued");
        assert_eq!(info.code, "E-PATCH-002");
        assert!(find("e-patch-2").is_some());
    }
}
