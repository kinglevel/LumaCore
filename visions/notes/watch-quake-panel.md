---
title: Watch Quake Panel
category: docs
---
# Watch Quake Panel

**Date:** 2026-03-11
**Status:** Design note (future enhancement)

## Concept

Watch output should display in a separate "Quake terminal" style panel that drops down from the top of the TUI screen. When no watches are active, the panel auto-closes and the output area reclaims full height.

## Behavior

- `/watch` on any command → opens watch panel (if not already open)
- Watch panel renders at top of screen, pushes output area down
- Multiple watches stack vertically inside the panel
- `unwatch` last watch → panel auto-closes with smooth transition
- Panel height auto-sizes to content (with a max, e.g., 40% of terminal height)

## Layout (ratatui)

```
┌─────────────────────────────────┐
│ ▼ WATCH (2 active)              │  ← Quake panel (dynamic height)
│  clock: 14:32:05                │
│  info universe 1: [sparkline]   │
├─────────────────────────────────┤
│ Path bar                        │
│ Scrollable output               │  ← Existing layout
│ Input line                      │
└─────────────────────────────────┘
```

When no watches:

```
┌─────────────────────────────────┐
│ Path bar                        │
│ Scrollable output (full height) │
│ Input line                      │
└─────────────────────────────────┘
```

## Implementation

- `Layout::vertical()` adds a top constraint only when `state.watches.len() > 0`
- Each watch gets its own row inside the panel
- Panel has a subtle border/separator (e.g., `─── WATCH ───`)
- Watch panel could be collapsible with a keybind (toggle visibility without stopping watches)

## Current Watch System (for reference)

Currently watches render inline in the output area with a 200ms poll loop. This redesign moves them to a dedicated persistent panel instead of mixing with command output.
