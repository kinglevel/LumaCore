# lumacore-cmd-devices

The devices: what each app install or browser tab is to the station — its lock, its look, the
screen config it shows.

## Purpose

Every window of an app shares one device id, a browser tab has its own (decisions §30 "12a3").
The station keeps each device's DeskLock and normal theme — station-local, never replicated, not
an Oops step — so a reload or an app restart comes back as it was. `DeskLock` (F9 / Pause) locks
the device a line came from, `On DeskLock Station` every device of the station and every device
that connects later; the crate's `Hook::Gate` (the lock gate) then refuses every line but the lock
keys from it (E-LOCK-001), and only the user who locked it, or Admin, unlocks it. `NormalTheme`
(Meow+F9) shows the factory look; `Device` / `Set Device "ScreenConfig"` and the `ScreenConfig`
object pick which of the user's screen configs a device shows (§30 "12c2"). The device table is
the state's; the toggle shape and the window toggles (Sidebars, ViewBar, FullScreen) the kernel's
`devices`.

## Owns / Doesn't own

- **Owns:** the rows (`rows`), DeskLock / NormalTheme and the lock gate (`lock`), Device
  (`device`), ScreenConfig (`screen_config`).
- **Doesn't own:** the device table and unlocking (`lumacore-state` `device_table`), the toggle
  shape and the window toggles (the kernel's `devices`), screen configs themselves (the kernel's
  `screens`).

## Public surface

- `register` — the rows and the lock gate, after the kernel's.
- `allowed_while_locked` — what a locked device may still run.

## Rules

- A locked device runs only its lock keys. [test a_locked_device_runs_only_the_lock_keys]
- Only the user who locked a device, or Admin, unlocks it. [test only_the_locker_or_admin_unlocks_with_the_key]

## Edges

- **Depends on:** lumacore-registry, lumacore-sysmon, lumacore-objects, lumacore-cmd-station.
- **Used by:** lumacore-terminal, lumacore-server, lumacore-sweep, lumacore-tidy.

## Decisions

- [§30 "12a3"](../../../../docs/archive/decisions-2026-10.md) — devices, DeskLock, the core keys
- [§34 25.17](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate, before protocols

## Testing

- `cargo test -p lumacore-cmd-devices` — the lock, its gate, NormalTheme, Device, ScreenConfig.
