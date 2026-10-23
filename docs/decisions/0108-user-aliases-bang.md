---
id: 0108
title: Personal aliases on the profile, typed with !
area: syntax
status: decided
date: 2026-10-15
built_in: —
supersedes: [archive §15 L1159–1161 "short forms registry aliases"]
superseded_by: —
confirmed_by: none
source: [archive §33 L7077–7097 (item 13), archive §33 L7195–7199 (item 28)]
---
# Personal aliases on the profile, typed with !

## Context
With built-in short forms gone (see one-registry-one-namespace), operators still want shorthands, and the cat names wanted a home that costs no registry word. Established consoles have nothing like it. `Alias` / `Unalias` verbs were considered and dropped: `Alias` is an object with the normal verbs.

## Decision
An alias is yours: a word for a line, kept on your profile, typed with `!`. ``Store Alias "St" `Store Cue 1` ``, then `!St` runs `Store Cue 1`; `Delete Alias "sg"` removes one; the line is a backtick value (see quoting-values-options). Words typed after an alias are appended, Unix-like: `!sg 3 "Front"` runs `Store Group 3 "Front"`, `!St /Merge` runs `Store Cue 1 /Merge`; no placeholders (`$` stays for variables). An alias can't name another alias and every word in it must be exact, checked when it is set.

An alias expands only in a line typed on a command line, on the typing station before anything else; the forwarded, journalled and Oops-filed line is the expanded one written out by `print`. Never in macro, key or cue lines, OSC, HTTP, the UDP port or any stored or machine line (a stored `!x` is E-SYN-008). The `!` marker keeps aliases apart from every built-in word, now and after a new word is added; `!` means nothing else in Lum.

Every new user profile starts with `hiss` → `Oops`, `unmeow` → `Oops`, `turnoff` → `Shutdown` (typed `!hiss`); a bare `hiss` is unknown and the hint names `!hiss`. `hissme` stays its own verb.

## Consequences
Not built: the `Alias` object's lines sit in the catalog's `PENDING` list; the grammar's `alias` rule is in lum.md. The ready-made aliases come from the shipped default-profile data file (see tree-shell-unix-fence), never from strings in code.
