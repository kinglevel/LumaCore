//  ██████╗ ██████╗      ██╗███████╗ ██████╗████████╗
// ██╔═══██╗██╔══██╗     ██║██╔════╝██╔════╝╚══██╔══╝
// ██║   ██║██████╔╝     ██║█████╗  ██║        ██║
// ██║   ██║██╔══██╗██   ██║██╔══╝  ██║        ██║
// ╚██████╔╝██████╔╝╚█████╔╝███████╗╚██████╗   ██║
//  ╚═════╝ ╚═════╝  ╚════╝ ╚══════╝ ╚═════╝   ╚═╝
// FILE: object
//! What an object declares (decisions §15, §33): its keyword and spellings, its Lum shape (ids,
//! the objects that follow it owner first), its default verb, help, family, how a call of it is
//! undone and scheduled, where and with what level each verb runs, what it reaches, its flags and
//! its planned verbs. Pure data: what each verb *does*, its pool, its tree collection, its
//! properties and its Delete question sit beside the row, in `lumacore-state`'s `ObjectDef`
//! (decisions §34 21).
//!
//! Owns:          ObjectDecl, LumShape
//! Doesn't own:   the verbs' handlers and the rest of an object's behaviour (`ObjectDef`)
//! Entry points:  ObjectDecl::{matches, is_planned, description}, LumShape
//! Decisions:     §15, §33, §34 21

use crate::command::{CommandClass, CommandFamily, Doc, Reach, Rights, Scope, Undo};
use crate::flag::FlagDef;
use crate::verb::Verb;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// An object as the catalog holds it.
pub struct ObjectDecl {
    /// Display form, e.g. "ViewButton".
    pub keyword: &'static str,
    /// How it is written in a Lum line (decisions §33 10, 27, 49): its ids and the objects
    /// that may follow it owner first. The parser's vocabulary reads it (`crate::lum`).
    pub lum: LumShape,
    /// What the object does with no verb (decisions §33 6, 40–48): `Group 3` =
    /// `SelectFixtures Group 3`, `Sequence 3` = `Select Sequence 3`, `Blind` = `Toggle Blind`.
    /// One per object; changing one is a lum.md hard discussion.
    pub default_verb: Verb,
    /// Other accepted spellings, lowercase: plurals, short forms ("vb").
    pub aliases: &'static [&'static str],
    /// Help documentation (description, syntax, examples, …).
    pub doc: Doc,
    /// Language family when the object starts a line (`Fixture 1 At 50`).
    pub family: Option<CommandFamily>,
    /// Undo/replication when the object is called (`Fixture 1 At 50`).
    pub call_undo: Undo,
    /// How the runtime schedules a call of the object.
    pub call_class: CommandClass,
    /// Where a verb on the object runs in a LumaSession (decisions §10): on
    /// the master (show data, users' state) or on this station (its settings,
    /// interfaces, live screens and windows). Given the verb and the
    /// arguments after the object word.
    pub scope: fn(Verb, &[&str]) -> Scope,
    /// What a verb on it touches besides the show (decisions §25 "Step 10
    /// hardening"); a line's reach is the furthest of its command's and this.
    pub reach: fn(Verb) -> Reach,
    /// The level each verb on it needs (decisions §30 "12d hardening"),
    /// given the verb and the arguments after the object word: it replaces
    /// the command word's own level (`Store View` Playback, `Store Group`
    /// Program, `Store Output` Setup, `Store User` Admin; `Verb::Call` is the
    /// bare object). The shared shapes are in `crate::rights` (`content`,
    /// `fixture`, `setup`, `screen`, `playback`, `anyone`). The dispatch gate,
    /// help, suggestions, the reference and the sweep read it.
    pub rights: fn(Verb, &[&str]) -> Rights,
    /// Flags per verb (`Store Group 3 /Overwrite`): [`FlagDef::on`] names the verbs (decisions §31).
    ///
    /// [`FlagDef::on`]: FlagDef::on
    pub flags: &'static [FlagDef],
    /// Verbs published but not built yet (decisions §29): no handler; they
    /// are in help, completion (dimmed) and the reference, and running one
    /// replies `isn't built yet (planned)` [E-CMD-003] and changes nothing.
    pub planned: &'static [Verb],
}

/// How an object reads a dotted id (lum.md "Every object declares how it reads a dotted id",
/// decisions §12 "Tags hardened"). The one resolver reads ids only through it, never as a float.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdReading {
    /// One of a kind: no ids.
    None,
    /// A whole number (`Fixture 3`); a dotted id is refused.
    Whole,
    /// A decimal number (`Cue 1.5`, so `1.10` = `1.1`), up to 3 decimals.
    Decimal,
    /// A fixed number of whole parts (`Master 2.1`, `Window 1.4`, `Preset 4.1`).
    Parts(u8),
    /// One whole part per tree level, any depth (`Tag 1.2.1`, so `1.10` ≠ `1.1`): the default of
    /// an object that nests itself.
    Path,
}

/// [`ObjectDecl::lum`]: what follows the object word in a line.
#[derive(Clone, Copy)]
pub struct LumShape {
    /// How many dot parts its numbers take at most (`Fixture 3` 1, `Cue 1.5` 2, a Path any:
    /// `u8::MAX`); 0 = one of a kind, no ids (`Blind`, `Programmer` — typed alone it runs its
    /// default verb, decision 27).
    pub id_parts: u8,
    /// How it reads a dotted id.
    pub reading: IdReading,
    /// The objects that may follow it in an owner-first path (`DataPool 2 Group 3`).
    pub children: &'static [&'static ObjectDecl],
    /// Words that may stand before its ids and name their numbering (`Fixture Houselights 4`:
    /// MVR's ID types, decisions §34 round 3 19).
    pub id_kinds: &'static [&'static str],
}

impl LumShape {
    /// One of a kind: no ids.
    pub const ONE: LumShape = LumShape { id_parts: 0, reading: IdReading::None, children: &[], id_kinds: &[] };

    /// Counted, its numbers whole (`parts` 1) or with a fixed number of whole parts (named
    /// members are quoted).
    pub const fn ids(parts: u8) -> LumShape {
        let reading = if parts <= 1 { IdReading::Whole } else { IdReading::Parts(parts) };
        LumShape { id_parts: parts, reading, children: &[], id_kinds: &[] }
    }

    /// Counted, its numbers decimal (`Cue 1.5`; `1.10` = `1.1`).
    pub const fn decimal() -> LumShape {
        LumShape { id_parts: 2, reading: IdReading::Decimal, children: &[], id_kinds: &[] }
    }

    /// Counted, its numbers a path of any depth (`Tag 1.2.1`): an object that nests itself.
    pub const fn path() -> LumShape {
        LumShape { id_parts: u8::MAX, reading: IdReading::Path, children: &[], id_kinds: &[] }
    }

    pub const fn children(self, children: &'static [&'static ObjectDecl]) -> LumShape {
        LumShape { children, ..self }
    }

    /// The words that may stand before its ids (`Fixture Houselights 4`).
    pub const fn id_kinds(self, id_kinds: &'static [&'static str]) -> LumShape {
        LumShape { id_kinds, ..self }
    }

    pub const fn counted(&self) -> bool {
        self.id_parts > 0
    }

    /// How the reference and help name its id reading (`whole`, `decimal`, `2 parts`, `path`).
    pub fn reading_text(&self) -> String {
        match self.reading {
            IdReading::None => "no ids".into(),
            IdReading::Whole => "whole".into(),
            IdReading::Decimal => "decimal (1.10 = 1.1)".into(),
            IdReading::Parts(n) => format!("{n} parts"),
            IdReading::Path => "path (1.10 ≠ 1.1)".into(),
        }
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl ObjectDecl {
    pub fn description(&self) -> &'static str {
        self.doc.description
    }

    /// The verb is published as planned (not built).
    pub fn is_planned(&self, verb: Verb) -> bool {
        self.planned.contains(&verb)
    }

    pub fn matches(&self, word: &str) -> bool {
        let word = word.trim_matches('"');
        self.keyword.eq_ignore_ascii_case(word)
            || self.aliases.iter().any(|a| a.eq_ignore_ascii_case(word))
    }
}
