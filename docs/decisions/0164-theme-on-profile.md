---
id: 0164
title: Theme on the UserProfile; the TUI wears it
area: users
status: built
date: 2026-10-12
built_in: 12a (tokens); 12a2 (on the profile); 12b (the TUI look); the editor and the background adjusters wait for 12g
supersedes: [archive §30 L3604–3608 "stored on the user", archive §30 L3649–3652 "ShowData.Users.n.Theme", archive §30 L3839 "Tint adjuster"]
superseded_by: —
confirmed_by: [tidy `terminal_colours_only_in_the_look`, tidy `frontend_colours_only_in_the_theme`, `look_tests`]
source: [archive §30 L3599–3608, archive §30 L3649–3652, archive §30 L3789–3795, archive §30 L3839–3846, archive §30 L4072–4081, archive §30 L4170–4176, archive §30 L4199–4210]
---
# Theme on the UserProfile; the TUI wears it

## Context
A reference console has over a thousand colour entries; LumaCore wanted a REAPER-style theme: a small editable palette plus adjusters, everything else derived. 12a stored it on the User; 12a2 moved it to the profile with the rest of the look (see profile-carries-look). The user then asked that the terminal TUI follow the same theme, which settled the conflict between the old cli-terminal and ui-framework docs.

## Decision
The theme is the profile's `Theme`: a preset, the editable palette colours (only the changes stored) and adjusters, declared as `UserProfile` properties and written with `Set UserProfile … Theme…` (`Theme Default` resets). Every other colour (hover, pressed, alt row, focus, terminal roles) is derived; the derivation exists once in the frontend (`theme.js`) and once in Rust (`core/look.rs`), checked against each other by tests. The whole TUI (shell, command line, completion, dialog window) is drawn from the logged-in user's profile theme, resolved on the station and sent as truecolor, the background painted too; colours in `terminal/` are named by `tui_look::role`, never a raw colour. A station's normal-theme override shows the factory preset and never writes a user's theme (see device-lock-theme-state). Users of the profile and Admin may change it; a drag previews in the client and files one Oops step (UI) on release.

## Consequences
A raw colour in `terminal/` fails tidy `terminal_colours_only_in_the_look`; one in the frontend outside the theme module fails `frontend_colours_only_in_the_theme`. Decided, not built (12g): the theme editor (pick an element to find its colour, reset per colour, A/B, import / export JSON) and two background adjusters, hue and saturation, moving all neutral greys together and replacing Tint (a format bump then, forward only); Tint is what the code declares today.
