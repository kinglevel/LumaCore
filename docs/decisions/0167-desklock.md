---
id: 0167
title: DeskLock locks the device it is pressed on
area: devices
status: built
date: 2026-10-12
built_in: 12a3 (decided in 12a hardening)
supersedes: [archive §30 L3658–3666 "scope every screen", archive §30 L3660–3661 "F9 cd fixture"]
superseded_by: —
confirmed_by: [E-LOCK-001, E-LOCK-002, `lumacore_cmd_devices::allowed_while_locked`, `devices::may_unlock`]
source: [archive §30 L3658–3666, archive §30 L3810–3824, archive §30 L3893–3897, archive §30 L3911–3915, archive §30 L3991–3999]
---
# DeskLock locks the device it is pressed on

## Context
A reference console's desk lock is a per-station UI overlay on Pause / F9, unlocked with the key or four corner taps; its `Lock` keyword locks pool objects. 12a scoped it to every screen of the station; 12a3 made it per device (see device-lock-theme-state) and gave users with a password a real unlock (R1-42).

## Decision
`DeskLock` is a mode-style object (`On DeskLock`, `Off DeskLock`, bare toggles), station-local, never replicated, not an Oops step; `Lock` / `Unlock` stay free for an object lock. F9 / Pause toggle it (core keys, see keys-uikey-gestures), plus the Lock tile on the touch sidebar and in the Shutdown menu. `On DeskLock` from a shell locks the client it came from; `On DeskLock Station` locks every device, and a device that connects while it is on starts locked until the last device is unlocked (`AppState::station_lock`). While locked the page forwards only the lock keys; the station refuses any other line from a locked device in dispatch, before forwarding (E-LOCK-001; `lumacore_cmd_devices::allowed_while_locked` admits the lock words bare or after On / Off / Toggle, `Station` only for DeskLock, one statement). Unlock: a user without a password with the key, a tap or the corner taps; one with a password only with it (`POST /api/unlock`); only the user who locked it or Admin (`devices::may_unlock`; E-LOCK-002 names the case). Nothing is shown over the lock: asks run to their defaults, the surface shows "n waiting". Playback, DMX, shells, other devices and OSC run on. Auto-lock after idle minutes is a LockScreen setting of the profile, default off.

## Consequences
Never a second lock check in a transport or a client-only lock; a word that must work while locked is admitted in `allowed_while_locked`. What the locked device shows is one-surface-login-lock-connect.
