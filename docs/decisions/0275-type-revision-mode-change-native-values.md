---
id: 0275
title: Type, revision and mode changes keep native values
area: patch
status: decided
date: 2026-10-18
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §34 L9725–9732, archive §34 L10040–10049]
---
# Type, revision and mode changes keep native values

## Context
"changing fixture type and mode of a patched fixture is not well built": no mode-change path existed and values risked being rewritten as DMX; "we need to be very very careful".

## Decision
A type holds its revisions (same FixtureTypeID); `/FixtureType` takes the newest, `/Revision n` picks one; revisions are told apart by the file's SHA-256 and revision text, never by name; importing a newer revision changes no fixture. Changing needs the claimed patch: `Set Fixture <sel> "FixtureType" | "Revision" | "Mode" …`, from the grid's cells through the same picker as Insert fixtures. The address is kept; where the new footprint collides or runs past the universe's end the address shows red with a warning until the user resolves it, no question at change time; the attribute map opens when values lose a target (see map-and-clone-attribute-maps). Values are stored native — strict: degrees, Hz, kelvin, CIE, wheel slots by GUID, never a DMX byte; DMX is computed from the fixture's current GDTF range at every plan compile, so a revision that changes a range recalculates the DMX and never touches the value; a held out-of-range value keeps what it wanted, is listed, and is retried when a range grows. Update fixture types lists the show's types with a newer library revision and a diff, by severity: must-resolve (a mode in use is gone) blocks Update; warnings carry a pre-filled fix; info needs nothing.

## Consequences
Not built (round 3 part 5): the draft's Fixture type / Mode cells wait for it. Never convert a stored value to DMX at change time; never a silent type exchange.
