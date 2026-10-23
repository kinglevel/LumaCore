---
id: 0152
title: "One attribute board: show, profile, new attributes"
area: attributes
status: built
date: 2026-10-14
built_in: 12f-a parts 3–4b
supersedes: []
superseded_by: —
confirmed_by: [attributes feed, Status.attributes_waiting]
source: [archive §30 L6048–6063, archive §30 L6072–6082, archive §30 L6097–6100, archive §30 L6114–6120, archive §30 L6141–6150, archive §30 L6243–6245, archive §30 L6593–6608]
---
# One attribute board: show, profile, new attributes

## Context
In mockups the board won over a tray, an inbox and a one-at-a-time dialog.

## Decision
One kit element `<luma-attr-board>` (`kit/attrboard.js`) with a target (the show or a profile) and a filter (All / New / Hints / Unused, By fixture type) is the profile's Attributes tab, the patch's attribute definitions and the new-attribute dialog. It is fed by the `attributes` State feed (`core::attribute_board`) and opened by the `attribute_board` UI event from `Edit [UserProfile] Attribute [/New|/Hint|/Unused] [/FixtureType]` and `Unclaim Patch /Keep /Edit` (the show's board, then yours); a terminal gets the list. Every gesture is a line: a drop on a header is `Set … FeatureGroup`, between tiles `Move … At FeatureGroup "G" n`, "+" a bare `Store [UserProfile] FeatureGroup` (the station asks), the learning toast `Store … AttributeRule`, the header menu `Set FeatureGroup … Sort | Merge | Split`; Delete stays a command line. Hold a tile opens a popover; no live fixture context. A column orders its group; a placed attribute lands at the bottom on every layout; the board never reshuffles a layout. Unused attributes sit behind an "Unused n" chip. A group carries Paint and a Lucide icon.

The person patching gets one two-line first dialog (show grouping and your bar: Follow standard / Edit, Follow standard for both, Cancel back to the still-claimed patch); other users get Follow / Edit / Later and "n WAIT" in the sidebar (`Status.attributes_waiting`). Sub-attributes follow per `SubAttributesFollow Ask | Always | Never` (`/WithSubs`, `/Only`; `/NoConfirmation` = Always). The Encoder bar tab is `<luma-encoder-bars>` (`Edit EncoderBar`).

## Consequences
Not built: the "Remember" tick on "Move n with it?"; sharing a grouping (export / import, copy, reset, a template library — each asks), a screen's layout, another profile's board, the TUI board, keyboard use. Everything on the board is possible by syntax, see featuregroup-attribute-syntax.
