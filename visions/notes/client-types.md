---
title: Client Types
category: docs
---
# Client Types & Selective Loading

**Date:** 2026-03-11
**Status:** Design note (future phase)

## Concept

Different clients running LumaCore have different needs. Not every client needs the full show data.

## Examples

| Client | Needs | Doesn't Need |
|--------|-------|--------------|
| Full workstation | Everything | — |
| Raspberry Pi node | DMX output, network config | Show media, GDTF library, UI |
| Remote terminal | Command interface, state sync | Media, DMX output |
| Playback wing | Cue playback, DMX output | Editor state, GDTF parsing |

## Architecture Implications

- ShowData should support partial loading profiles
- Media (images, gobos, 3D models) should be lazy-loaded or skippable
- Client declares its "role" or "profile" at connection time
- Server only syncs relevant data subtree to each client

## Prompt Style Note

`:3 >` as an alternative prompt style — fits the cat personality theme (meow/hiss/prrr).
Could be a UserProfile setting: `set prompt cat` → `:3 >`
