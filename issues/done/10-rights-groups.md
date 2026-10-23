# 10 — Rights groups

**Type:** design (decisions §19) · **Raised:** 2026-10-13 by the user, from issue 01.

## Today
Three roles (Guest / Programmer / Admin) and one gate in dispatch (`core/rights.rs`) that checks
only the **first word** of a line: 34 commands Guest, 68 Programmer, 1 Admin, 1 Internal. Some
Admin checks hide in handlers (issue 04). Rights never look at the target, so Guest tree-shell
words read the host disk (issue 01). Nothing between "can program" and "Admin" (setup, patch,
network).

## Reference (from memory — verify)
Established consoles use one level per user (None, Playback, Presets, Program, Setup, Admin), some
with worlds limiting which fixtures/attributes a user can touch. Patch and network setup need
Setup; other users and system need Admin.

## To do
1. Probe a reference console: user rights levels, what each level may run, worlds, the Guest
   user's defaults, how refusals read.
2. Decision in §19: levels, per command and per object verb, a target check in the gate
   (e.g. anything under `Root.Drives` = Admin/Setup), secrets never in the tree, how it maps to
   existing users and show files (format bump, no migration).
3. Build: registry `Rights`, the gate, issue 04's handler checks into the gate, docs/reference,
   tests (`planned`/rights word tests), sweep role list.
