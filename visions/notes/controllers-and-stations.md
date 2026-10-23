---
title: Controllers And Stations
category: docs
---
# Controllers, Stations & Driver Ecosystem

**Date:** 2026-03-11
**Status:** Design note (future phase)

## Station → Controller Hierarchy

```
Station
 └─ Attached Controllers
     ├─ Joystick
     ├─ OSC Surface
     ├─ MIDI Surface
     ├─ 3D Mouse
     └─ DAW Controller
```

## Per-User Controller Mappings

```
UserProfile
 └─ Controller Mappings
     ├─ Joystick        → User-defined behavior
     ├─ OSC Surface     → User-defined behavior
     ├─ MIDI Surface    → User-defined behavior
     ├─ 3D Mouse        → User-defined behavior
     └─ DAW Controller  → User-defined behavior
```

**Key insight:** Controllers are attached to a Station (hardware), but their *behavior* is defined per UserProfile. A 3D mouse could work totally differently for another user on another station.

## Driver Ecosystem

- Some controllers need drivers (e.g., 3D mice, DAW controllers)
- Drivers should be importable from a **driver pool** in ShowData
- Driver pool can be populated from the **library** (like GDTF fixtures)
- Flow: Library → import driver → ShowData driver pool → controller uses driver

## Architecture

```
Library
 └─ drivers/          (shared driver definitions)

ShowData
 └─ DriverPool        (imported drivers for this show)

Station
 └─ Controller        (physical device)
     └─ uses Driver from DriverPool

UserProfile
 └─ ControllerMapping (per-user behavior overrides)
```
