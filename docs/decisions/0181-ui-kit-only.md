---
id: 0181
title: Every UI surface is built only from kit elements
area: ui-kit
status: built
date: 2026-10-12
built_in: 12a1
supersedes: []
superseded_by: —
confirmed_by: [tidy frontend_colours_only_in_the_theme, tidy frontend_icons_come_from_the_kit, tidy frontend_font_sizes_are_tokens, tidy frontend_gestures_only_in_the_kit, e2e kit.spec.ts]
source: [archive §30 L3521–3526, archive §30 L3609–3612, archive §30 L3624–3632, archive §30 L3636–3642, archive §30 L3696–3700, archive §30 L3701–3717]
---
# Every UI surface is built only from kit elements

## Context
The touch UI kit step chose plain custom elements as ES modules — no framework, no build step, nothing from the internet — updated in place by id so nothing re-renders under a finger, with one pointer core for every gesture. On the restyled mockups the user made it strict: "all windows etc needs to use elements. So if i want a new type of window that is missing elements we need to design the elements first with artifacts."

## Decision
Every window type, menu, dialog, pop-up, bar and plugin UI is composed only of `luma-*` kit elements in `crates/ui/lumacore-webui/web/kit/`; no hand-built controls, `innerHTML` templates or raw colours outside the kit. A window that needs something the kit lacks waits: the element is designed first as a mockup artifact, approved by the user, added to the kit and the Showroom, and only then used. The Showroom is a registry-driven window type that shows every element in every state; it is also the screenshot set and the plugin reference. Kit elements carry `data-kit` / `data-id`, and the e2e `tap()` checks the element is really hit. Playwright runs desktop, tablet, phone and WebKit projects.

## Consequences
Old code was exempt only until the part that owned it replaced it; since 12d part 5 no overlay lives outside the kit (see menudef-luma-setup). Tidy catches raw colours, foreign icons, raw font sizes and gestures outside `kit/` (the four rules above); lists and action rows have their own rules (see luma-grid, slab). UI work is verified in Chrome and with `./scripts/unix/screenshots` against the approved mockup. Every gesture ends as a command line (see descriptors-from-registries); the pointer core's rules are one-hold-one-owner and pointer-core-one-lift-gesture.
