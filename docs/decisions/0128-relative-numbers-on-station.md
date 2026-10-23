---
id: 0128
title: Relative numbers are parsed on the station
area: commands
status: built
date: 2026-10-13
built_in: 12b (Set Window / Set Self in 12c)
supersedes: [archive §30 L4161–4164 "unparseable was 0 %"]
superseded_by: —
confirmed_by: [E-CMD-008, `relative_tests`]
source: [archive §30 L4082–4084, archive §30 L4161–4164, archive §30 L4538]
---
# Relative numbers are parsed on the station

## Context
The keypad and the encoders need `+ 10` style input (R1 Q8). Before 12b a value the station couldn't read silently became 0 %, and a relative step would have meant the client computing the result — a second arithmetic beside the station's.

## Decision
The client sends the typed expression as is; the station parses relative numbers against the current value. `At + 10`, `At - 10`, `At * 2`, `At / 2` apply per fixture from the user's programmer value, else the fixture's current output, kept in 0–100 %; `Attribute … At` the same; `Set … + 3` works on any whole-number property; `Set Window` / `Set Self` take `+ - * /`, `Default`, `Toggle`, `Now`. The space matters: `- 3` is relative, `-3` glued is an absolute negative value (clamped). An unparseable value is E-CMD-008, never 0. The typed `Set` handler parses the value once by the property's kind (see set-one-typed-handler); Toggle is only for a `Switch`.

## Consequences
Never compute a relative step in a client, a kit control or a handler of its own; a new numeric property gets relative input for free through the one `Set` and the At verb (see at-verb-on-target). A value that cannot be read is a refusal with its code, never a silent default; `relative_tests` holds the spaced / glued cases.
