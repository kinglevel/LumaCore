---
id: 0287
title: The view bar holds any object; one pool window type per pool
area: ui-kit
status: built
date: 2026-10-22
built_in: view bar (2026-10-23)
supersedes: []
superseded_by: —
confirmed_by: [views::buttons tests, pools_every_pool_has_a_window_type, kit-viewbar.spec.ts]
source: [session 2026-10-22, view bar round 1 — docs/dev/ui-kit/mockups/2026-10-22-view-bar/01-view-bar.html, approved]
---
# The view bar holds any object; one pool window type per pool

## Context
A stored view button never showed: the profile keeps buttons as view GUIDs and the sidebar matched them against view numbers, and `/api/views` found the profile by the login's name. A reference console keeps a bar per screen, addresses buttons `ViewButton <screen>.<n>`, and lets a button hold macros, users and screen configs besides views; the user's desk keeps buttons per display. One profile-wide set of buttons, a fixed object list only, assigning by command line only, clearing the screen on an empty tap, number-only pool cells and a pool window that switches kind were drawn and not taken.

## Decision
A profile keeps view buttons per screen: `ViewButton <screen>.<n>`, a bare `ViewButton n` meaning the line's screen. A button holds any object by kind and GUID; a tap runs that object's default verb (`Call View`, `Select Sequence`, a group's selection), a tap on an empty button only says so, and Store on an empty button stores this screen as a new view and assigns it. Assigning is one line, `Assign <Object> <n> At ViewButton <s>.<n>`, from a drag of a pool cell onto the bar, from Assign then two taps, or from a button's Assign… list. Each pool is its own window type on the one pool window: Views (cells draw the view's windows, its screens, the buttons it sits on, the current view gold; a window setting picks the profile) and Macros join Groups and Sequences. The bar shows as many buttons as fit, ▲ ▼ step a set; it rides a feed built by `lumacore-views` from the user's own profile.

## Consequences
The button map changes shape (a show format bump, no migration). A new object kind on the bar needs only a default verb; a new pool needs only a `PoolDef` and a window type.
