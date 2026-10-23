---
id: 0295
title: Keys bind per shortcut mode; a press can cycle
area: keys
status: built
date: 2026-10-23
built_in: the profile and key editor, steps 2–3 (docs/plans/active/2026-10-23-profile-and-key-editor.md)
supersedes: [0170 "letters are never bound" — that sentence only: letters bind in PRG and NAV]
superseded_by: —
confirmed_by: [E-KEY-006, E-KEY-007, a_mode_binding_wins_over_all_and_an_empty_one_turns_a_key_off, a_letter_binds_in_a_mode_as_a_press_cycle, prg_keys_cycle_their_word_in_place_and_a_run_completes_the_line, every_built_in_keys_lines_are_the_catalogs_words, lines_in_a_row_are_one_value, every_format_from_the_oldest_has_its_step]
source: [docs/plans/active/2026-10-23-profile-and-key-editor.md, the user in session 2026-10-23 ("looks good, implement" on mockups docs/dev/ui-kit/mockups/2026-10-23-profile-editor/02-keys.html, 03-cycles.html), docs/research/keys/2026-10-23-key-editor.md]
---

## Context

The terminal held PRG's letters and the `aa` double tap in a hard-coded table; only a handful of keys were real bindings, all in every mode, and nothing could be seen or changed in an editor. The user asked for a keyboard per KBD / PRG / NAV and for keys that cycle their word (E: Edit, again EditSetting), the reference console's model. A timed cycle and a `Runs` list were rejected: every key's press runs today, so the list names the presses that type.

## Decision

A `KeyBinding` carries its `Layer` (All, KBD, PRG, NAV; `slot`); a mode's binding wins over an All one, a profile's over LumaCore's, debug over both; a profile binding with no UiKey turns LumaCore's key off. ``Bind <key> `<line>` … /Shortcuts "PRG"``; `Unbind … /Shortcuts`. A letter, digit, sign or Space without Ctrl / Alt types in KBD and binds only in PRG or NAV (E-KEY-006). A UiKey's Press is a cycle (`cycle`), `types` lists the presses that type; Meow alike; `empty_only` keys act only on an empty line. Lines in a row are one value (`Value::Lines`). The next press of a key replaces what it typed while the line still ends with it, wraps, no timer; a press that runs completes the line when its cycle types, else runs alone. PRG's keys are `keybindings::PRG_KEYS` rows; a profile's keys are set with `Set UIKey "<id>" "Press" …` (E-KEY-007 for a locked key).

## Consequences

A new default key is a `PRG_KEYS` row, never a terminal table. The shortcut mode stays one value; per screen, NAV's own keys as rows, hold = repeat and line-guarded rows are open.
