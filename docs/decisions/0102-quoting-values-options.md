---
id: 0102
title: A quoted word is a value, a bare /word an option
area: syntax
status: built
date: 2026-10-15
built_in: issue 07 (2026-10-13), lum part 1 (quoted properties, backticks)
supersedes: [archive §15 L1180–1181 "one word unquoted", archive §33 L7138–7144 "quoted-line examples", archive §33 L7200–7204 "Watch quoted line"]
superseded_by: —
confirmed_by: [E-CMD-006, E-CMD-008, E-SYN-005, E-SYN-014, `tokens::unquote`, `drive::library_file`, `Value::Line` / `Kind::Line`]
source: [archive §15 L1171–1191, archive §33 L7013–7020 (item 7), archive §33 L7698–7708 (item 91)]
---
# A quoted word is a value, a bare /word an option

## Context
R1 issue 07 found option checks swallowing quoted text and names colliding with keywords; Established consoles treat a quoted word as a value everywhere. The lum discussion made property names quoted and chose backticks for a line given as a value (doubled quotes, backslash escapes and no inline lines were declined).

## Decision
A quoted word is text — a value, never a keyword; a bare `/word` is an option. The `quoted` mark goes through dispatch: a handler gets the quotes and takes them off only when it stores the word (`tokens::unquote`), so `Label Group 3 "/nc"` names it `/nc` and `Group "All"` is a group named All.

Whether a quoted word is a name, a value or a property name comes only from the verb's declared signature, never a guess. Property names are always quoted: `Set Fixture 3 "Name" "Spot SR"`; a bare property is E-SYN-005 with the quoted form as hint. A property is never nested inside its quotes (see owner-first-syntax). A declared choice is bare; quoted text where a choice goes is E-SYN-014.

A command line given as a value is in backticks, nothing else: ``Bind "F5" `Set Self "AutoScroll" Toggle` ``, ``Watch `List Fixture` /Every 500``; `"…"` is never a line (``Bind "F5" "List Fixture"`` is E-SYN-014); a backtick can't stand inside a line (no escapes); the signature says which slots take a line.

A filename a line names is relative to its library folder through `drive::library_file` (sub-folders allowed; an absolute path or `..` is E-CMD-008).

## Consequences
A name with spaces must be quoted (unquoted → E-CMD-006). Code that imports by itself (the demo builder) calls the function, never a line. Never add a second quote rule or an option check that sees quoted text.
