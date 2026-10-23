---
id: 0070
title: The encoder bar: concept A, Mode above Layer, per screen
area: ui-kit
status: built
date: 2026-10-14
built_in: 12f parts 4b, 4c
supersedes: [archive §30 L3560–3561 "encoders are drag", archive §30 L5767–5769 "hold spawns slider", archive §30 L5886–5888 "horizontal indicators option"]
superseded_by: —
confirmed_by: [e2e `kit-12f.spec.ts`, `Revisions.modes`]
source: [archive §30 L3557–3561, archive §30 L5767–5769, archive §30 L5800–5801, archive §30 L5810–5812, archive §30 L5833–5835, archive §30 L5859–5866, archive §30 L5877–5917, archive §30 L5954–5969, archive §30 L6195–6207, archive §30 L6443–6500]
---
# The encoder bar: concept A, Mode above Layer, per screen

## Context
Twelve artifact rounds shaped the bar: concept A, the classic encoder bar made modern and smarter, with four fixed encoders and smart cells from the GDTF. Horizontal indicators, a hold delay and a bar below the command line were tried and dropped.

## Decision
`<luma-encoder-bar>` (`kit/encoderbar.js`) sits above the command line bar, per screen: `Set Screen [n] EncoderBar On|Off|Toggle|"layout"` (saved on the profile's screen config `ScreenContent.encoder_bar`; a phone swipes without pages). What the bar shows is runtime on the live screen, `Screen.encoder: EncoderState { mode, group, page, layer }` — switching is never an Oops step. Words: `EncoderMode Attributes | CueTimingVertical | CueTimingHorizontal | Masters | Effects` (Effects planned), `EncoderGroup <n>|Next|Previous`, `EncoderPage <n>|Next|Previous`, `EncoderLayer Value|Fade|Delay|Next` — stepping is Next / Previous, never `+ 1`. The feed `encoder` (`core::encoder_view`) follows `Revisions.modes`. The feature strip, MODE above LAYER, four encoders each with Value · F · D vertical faders side by side, the layer's one wide, red when programmed (`--lc-programmer`). Press and drag adjusts at once (one gesture, one Oops step); a tap on the wide fader opens the slider panel (`kit/sliderpanel.js`). Every pop-up closes on a tap outside; dimming behind it is the profile's `PopupDim`. The Cue timing modes open on the selected sequence's current cue; the strip is its cues, the encoders its part and feature-group times on one shared, held scale, lines `Set Cue <c> Sequence <s> [FeatureGroup "g"] Fade|Delay|OutFade|OutDelay`. Masters mode is a fader bank. Layouts are decided as a profile-owned pool (`Store UserProfile EncoderBar "Busking"`, `Set Screen 1 EncoderBar "Busking"`, `Edit EncoderBar`); the scope word (`/ShowData` or `/UserProfile`) is open.

## Consequences
Not built: smart minis beyond levels, GDTF channel functions / sets, the colour picker and shapers, Prog time, the Effect layers, the suggested modes (Selection, Pawtern, Stage, Executors) — each its own step.
