---
id: 0134
title: Copy and Cut fill a clipboard of live GUIDs
area: door
status: built
date: 2026-10-15
built_in: 35b, 35g2
supersedes: [archive §35 L11070–11077 "client clipboard of the slab"]
superseded_by: —
confirmed_by: [`Runtime.Users.{User}.clipboard`, `SessionRuntime.clipboards`]
source: [archive §35 L10967–10975, archive §35 L10991–11000, archive §35 L11070–11077]
---
# Copy and Cut fill a clipboard of live GUIDs

## Context
The grid slab kept a client-side clipboard, so what was put aside lived in one browser tab and never passed the door. Established consoles have `Cut` and `Paste` with the destination as the target, and a pasted object carries every edit made after copying — a live reference. A snapshot at Copy was declined.

## Decision
A bare `Copy Group 1 Thru 5` remembers the rows' GUIDs as *copy*, `Cut Group 3` as *cut*; `Paste Group 10` (or `Paste DataPool 2 Group 10`) finishes them through the door (see one-object-door): a copy duplicates with new GUIDs, a cut moves with kept ones, spacing kept, swap and the existing-object ask applying. The clipboard holds live GUIDs with their mode and pool on `Runtime.Users.{User}.clipboard` — session runtime, replicated in `SessionRuntime.clipboards`, never saved, cleared by a show load; Paste takes the objects as they are then, a deleted one is skipped and named. `Paste`'s destination is a slot like Store's; it has no default object of its own, so `Paste 10` follows your default object (see default-object-per-user), another kind than the clipboard's is refused, and a stored line with a bare number is E-SYN-008. A bare `Paste`: a copy goes to the first free block of its own pool, a cut asks where; a stored bare Paste of a cut is refused naming the target. `Cut` is its own word — Move always moves. `Copy|Move … At …` stay the one-line forms.

## Consequences
A setup grid of an object on the door runs `Copy {object} {rows}` / `Cut {object} {rows}` / `Paste {object} {at}`; the page remembers only that something was put aside — never a client clipboard again. Network and the station table wait for the door.
