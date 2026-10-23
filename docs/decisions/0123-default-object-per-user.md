---
id: 0123
title: A counted keyword alone sets the user's default object
area: dispatch
status: built
date: 2026-10-15
built_in: 3b2
supersedes: [archive §33 L7910–7923 "context_number shortcut gone"]
superseded_by: —
confirmed_by: [E-SYN-008, the prompt `Root [Group]>`]
source: [archive §33 L7145–7153, archive §33 L7177–7181, archive §33 L7189–7194, archive §33 L7250–7256, archive §33 L7396–7400, archive §33 L7910–7923]
---
# A counted keyword alone sets the user's default object

## Context
A reference console lets a keyword alone change what a bare number means, limited there to patch id types. Issue 02 decided LumaCore's design; the `context_number` shortcut it replaced resolved a bare number inside the handler, so journal, forwarding and Oops saw a line that never said what it did.

## Decision
An object that takes ids typed alone sets your default object, `User.default_object` on `ShowData.Users` (replicated, never an Oops step, `Undo::Untracked`, no rights needed; None = Fixture). Every counted object can be it — `Group` → `Group>`, `Menu` → `Menu>`, `Root` → `Root>` (its default verb is List). A bare number, quoted name or `Thru` at the start of a statement is written out with your default object before anything reads the line (`dispatch::write_out_bare`: `3 At 50` = `Group 3 At 50`, `"Tower"` = `Info Station "Tower"`); a number right after a verb takes the verb's own `default_object` first (`Store 2` = `Store Cue 2`, Go / Go- → Sequence, Goto → Cue), else yours when it takes that verb (`Delete 3` = `Delete Group 3`), else Fixture. Nested lines are written out again, so the full line takes the change path, journal, forwarding and Oops. A keyword with ids runs its default verb instead (see bare-object-default-verb); a one-of-a-kind object alone acts. A stored key or macro line starting with a bare number is refused on save — it can't know whose default it runs with — E-SYN-008. The terminal prompt shows it: `[LD@host] Root [Group]>`.

## Consequences
Never resolve a bare number inside a handler; the line a handler sees names its object. Open: the web command bar's prompt (mockup first), the profile's start place (issue 02), `Root>` as a cd place (3e2).
