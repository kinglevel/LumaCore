---
id: 0036
title: App webview locked to the station; CLI ends with the stream
area: station
status: decided
date: 2026-10-14
built_in: — (the IPC enum built 2026-10-14)
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §31 L6871–6879]
---
# App webview locked to the station; CLI ends with the stream

## Context
The code review checked the clients against Electron's security checklist and Tauri: the app's webview could navigate anywhere, its IPC was untyped strings, and the CLI hung forever when the station died.

## Decision
The desktop app's webview may load only `lumacore://localhost` and the connected station; anything else opens in the system browser and new windows are refused; the login token rides an initialization script that runs on every page. IPC messages are one typed enum — built: `apps/lumacore-app/src/ipc.rs` (`Ipc`, tagged by `type`: connect page, station screen, overview); a message that doesn't parse is ignored and logged at debug. The app's HTTP replies are size-capped. The station's web origin checks stay open by the rights decision (isolated rigs). CLI: the end of the shell stream is a disconnect, the terminal is restored on a panic, and sentinel strings become an enum.

## Consequences
The navigation lock, the reply cap and the CLI fixes are decided; the archive confirms only the IPC enum built — check the app and CLI before relying on them. A new IPC message is a variant of `Ipc`, never a string. See app-thin-client for what the app is.
