---
id: 0104
title: Ids are numbers or quoted text; glued signs; case
area: syntax
status: built
date: 2026-10-15
built_in: lum part 1 (lexer), 3d7 (numbers start with a digit), 3d8 (named masters quoted)
supersedes: [archive §33 L7337–7339 "goback go- names", archive §33 L7786–7787 ".5 numbers", archive §33 L7788–7790 "bare Master Highlight"]
superseded_by: —
confirmed_by: [E-SYN-007, E-SYN-012, E-SYN-001, the `NUMBER` / `WORD` tokens of lum.md's grammar block]
source: [archive §33 L7039–7056 (items 10, 11), archive §33 L7171–7176 (item 24), archive §33 L7337–7339 (item 51), archive §33 L7475–7478 (item 59)]
---
# Ids are numbers or quoted text; glued signs; case

## Context
A reference console reads `Fixture 1+3` as 1, silently, and its values are case-sensitive. Lum takes that lexer rule and makes the silent cases refusals.

## Decision
An id is a number or quoted text. A number is digits with optional dots and starts with a digit (`0.5`, never `.5`): `Cue 1.5`, `Master 2.1`, `DMX 1.001`; how many parts an object takes is declared (see dotted-ids-per-object; a wrong count is E-SYN-012). Everything else is quoted: `Master "Grand"`, `Group "Front"`, `Interface "en1"`.

A sign glued to digits is part of the number; a sign standing alone is an operator: `Attribute "Pan" At -120` absolute · `At - 10`, `At + 10`, `At * 2`, `At / 2` relative · `Group 1 + 3 - 2` set operators · `/` before a letter is an option. A glued operator inside a list (`Group 1+3`) is E-SYN-007 with a hint to space it; `*` is only multiply. A range keeps its typed order (`Fixture 5 Thru 2` = 5, 4, 3, 2). One glued trailing `+` or `-` belongs to a word only before a space, `;` or the end: `Go-` is the back verb's one name.

Words ignore case, text keeps it: `store group 3` = `Store Group 3`; declared values are written as declared (see declared-words-written-as-declared); a lookup by name ignores case; two names of one kind differing only in case are refused at the second; `print` writes canonical case.

## Consequences
A negative value right after an implicit owner's quoted property stays refused (the glued `-30` reads as a list operator): name the owner or use the child-object form (see owner-first-syntax). `Root.` and `Self.` are the only places a dot nests (see tree-paths-typed-node, self-reads-in-values); Root's ids take any number of parts.
