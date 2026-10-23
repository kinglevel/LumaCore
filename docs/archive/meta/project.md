---
title: Project
category: docs
---
# LumaCore — Open Source Lighting Control Framework

## What This Is

LumaCore is an open-source, modular lighting control framework backed by internal planning docs, protocol references, and fixture research. It provides a full console replacement with a console-inspired object-oriented terminal, GDTF fixture support, and DMX output — accessible to hobbyists yet powerful enough for professional lighting designers.

## Core Value

A terminal-first lighting console with a console-style object tree that lets users patch GDTF fixtures, organize them into groups, and control dimmers with DMX output — all from an open, extensible framework.

## Requirements

### Validated

<!-- Inferred from existing codebase -->

- ✓ DMX protocol reference documentation — existing in references/
- ✓ GDTF/MVR specification access — existing in external_repos/spec/
- ✓ Fixture definition references — existing in references/
- ✓ Color system references — existing in references/
- ✓ Beam type fundamentals — existing in references/
- ✓ External tool ecosystem cataloged — existing in external_repos/

### Active

- [ ] Rust-based DMX engine core with protocol abstraction
- [ ] console-style object-oriented terminal (command line)
- [ ] Object tree: Fixture objects with patch, select, control
- [ ] Object tree: Group objects for fixture grouping
- [ ] GDTF import — parse .gdtf files, extract channel modes, use for patching
- [ ] DMX output via USB (FTDI/ENTTEC dongles)
- [ ] DMX output via Art-Net (UDP)
- [ ] DMX output via sACN (E1.31)
- [ ] Lua plugin system for user-extensible modules
- [ ] Headless server with API (REST/WebSocket)
- [ ] Web-based UI as primary graphical interface
- [ ] Desktop shell wrapping web UI (native-looking)
- [ ] Basic dimmer control commands
- [ ] console-oriented command syntax with familiar operator patterns
- [ ] Lua system test framework (like reference console's system test scripts, for debugging)
- [ ] Cross-platform support (Linux, macOS, Windows)

### Out of Scope

- Preset/sequence/cue system — deferred to v2 (v1 focuses on terminal + patch + groups + dimmer)
- Full GDTF geometry/wheels/physical properties — v1 imports channel maps only
- Mobile app — web UI accessible on mobile browsers is sufficient
- Video/media server integration — future scope
- Visualizer/3D rendering — future scope (vision: Rust GPU rendering streamed to web UI)
- DAW-style timeline with tracks triggering sequences — v2+ (Reaper/Cubase-style grid with looping, MIDI import/export, timecode sync)
- MIDI input/output/import/export — v2+ (timeline integration)
- Timecode (LTC/MTC) — v2+ (essential for timeline features)
- Network protocol modules beyond Art-Net/sACN — future scope

## Context

- Internal planning docs collect curated reference data: DMX protocols, GDTF/MVR specs, fixture definitions, color systems, beam types, and ecosystem documentation
- LumaCore consumes that reference material during planning and implementation
- User has JSON dumps of a reference console object tree in an internal reference archive — key input for terminal root and object model design
- reference console Lua system test scripts — reference for command primitives (`test_fixture_at.lua`, `test_group_update.lua`)
- reference console manual available for parsing to determine bare minimum feature set and document command set
- reference console forum should be crawled for pitfalls — what they did well and poorly; note: userbase has been unhappy with the system lately (opportunity for LumaCore)
- LumaCore commands should closely mirror reference console command syntax for familiarity
- Existing external repos include GDTF spec, parsers (gdtf.rs in Rust), and related tools
- The ecosystem already has Rust GDTF tooling (zeevonk) that may be leveraged

## Constraints

- **Tech stack**: Rust core engine, React 19 + Vite 7 + Tailwind 4 for Web UI, Lua 5.4 for plugin scripting
- **Cross-platform**: Linux, macOS, Windows as primary targets
- **Terminal UX**: Must feel like reference console command line — object-oriented selection and action syntax
- **Open source**: Framework must be open and extensible — others should be able to build on it
- **Module architecture**: Protocol outputs, effects, UI panels, and fixture profiles all as pluggable modules
- **Standards compliance**: GDTF (DIN SPEC 15800) for fixture data, Art-Net/sACN for network DMX

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust for core engine | Performance-critical DMX timing, memory safety, systems-level control | — Pending |
| Lua for plugins | Industry-standard embeddable scripting, familiar to lighting/game communities | — Pending |
| Terminal-first v1 | Core object model and command system before GUI — validates architecture | — Pending |
| Web-based primary UI | Modern web capabilities, network-accessible, cross-platform | — Pending |
| GDTF for fixture data | Open standard, growing adoption, replaces private fixture libraries | — Pending |
| reference console object model as inspiration | Industry-leading console, user has firmware intel | — Pending |

---
*Last updated: 2026-03-10 after research phase*
