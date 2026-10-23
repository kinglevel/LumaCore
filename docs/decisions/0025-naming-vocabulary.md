---
id: 0025
title: LumaSession, Station, Shell, Client, Screen, CommandLine
area: naming
status: built
date: 2026-10-11
built_in: step 3, step 4d, step 8, step 11
supersedes: [archive §18 L1259–1260 "Session means mesh", archive §11 L684–691 "JoinSession LeaveSession go", archive §18 L1264–1266 "Runtime.Sessions ViewSession"]
superseded_by: —
confirmed_by: none
source: [archive §11 L684–699, archive §18 L1258–1268, archive §23 L1964–1970, archive §29 L3305–3307]
---
# LumaSession, Station, Shell, Client, Screen, CommandLine

## Context
The networking words drifted: "Session" meant the station mesh, a shell connection and a logged-in user at once, `engine/` named the layer that owned state, and `lumacore-core` clashed with Rust's built-in `core`. The renames landed in steps 3, 4d, 8 and 11.

## Decision
**LumaSession** is the server-to-server side (stations joined, join / invite / dismiss / leave, discovery, replication) — never "network session", never `Session`, **no aliases**: the object `LumaSession` (`Join LumaSession "Festival"`, `Leave LumaSession`, `Invite Station "X"`, `Dismiss Station "X"`, `List LumaSession`); `JoinSession` / `LeaveSession` are gone; its runtime is `Root.Runtime.LumaSessions`. **Station** = one running server. **Shell** = one connected terminal or window on it (the object `Shell`; `ShellSession` is gone). **Client** = one attached device or window (CLI, app window, browser tab). **CommandLine** = one command line's input, history and path, owned by a user; several clients can mirror one. **Screen** (was `ViewSession`) lives at `Runtime.Users.{User}.Screens.{n}`; who is logged in is `Runtime.Users.{User}` (was `Runtime.Sessions`). **Engine** keeps its meaning: a long-running subsystem (render, HTTP, Perfetto); protocols are not engines. The one-per-station layer that owns state and runs every command is **core** (today `lumacore-objects` on `lumacore-state`), never `engine/`; `kernel` stays reserved for later Linux work. `lumacore-core` became **`lumacore-base`**. **"Network"** means only a data network; the `Network` keyword keeps that meaning. **"WebUI"** in docs, logs and the UI; the package is lowercase like every crate.

## Consequences
A new word in this area is one of these or is decided first; no alias table exists outside the registry (see one-registry-one-namespace). Inside the state and objects crates the module is written `crate::…`, std's core as `::core::…`. Old names remain only in the archive.
