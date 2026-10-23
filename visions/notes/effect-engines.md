---
title: Effect Engines And Plugins
category: docs
---
# Plugins EffectEngines And CommandsPool

## Goal

Create a future extension system that supports custom effect logic, generators,
and user-defined commands without destabilizing the core.

## What Planning Suggests

- Plugin system is a future differentiator.
- Custom effect generators matter.
- A Commands Pool backed by Lua and a visual editor is a future idea.
- Node-based logic belongs in a separate branch, not mixed into basic playback.

## What We Should Implement

- `Root.ShowData.EffectEngines`
  - node-based engines
  - generators
  - shader-style logic
  - custom plugins
- datapool-level `Plugins` for reusable show content
- later command-extension systems on top of the stable `Lum` core

## Suggested Next Steps

1. Keep `EffectEngines` separate from basic playback effects.
2. Define a minimal plugin boundary before adding scripting hooks.
3. Preserve `Lum` verbs while allowing user-defined higher-level commands later.
