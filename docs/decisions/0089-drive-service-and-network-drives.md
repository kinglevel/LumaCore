---
id: 0089
title: A drives watcher; library on first write; own network drives
area: drives
status: built
date: 2026-10-19
built_in: 38a (network drives: 38d, not built)
supersedes: []
superseded_by: —
confirmed_by: [E-CMD-012, the sysmon "Drives:" lines, tests/e2e/tests/kit-drives.spec.ts]
source: [archive §38 L11401–11417, archive §38 L11426–11428]
---
# A drives watcher; library on first write; own network drives

## Context
§38 point 6 settled how the station notices a stick, when it writes to one, and what a network drive is. The user: the library is made on the first write, never after an ask; network drives are LumaCore's own and "never exposed to the OS", not now.

## Decision
The server's `drives` thread (never the state thread) reads the OS's mounts on a short period and sends the state thread one message only when something changed; the state thread keeps the table (`reconcile`), publishes the drives in `StationView` and the `station` feed and writes a sysmon line (`Drives: usb1 "KINGSTON" in (… free, library ready)`, `usb1 out`, `usb1 library created`). Free space is read on the tick; a save that would not fit is refused before writing (E-CMD-012 with the numbers).

The library tree `LumaCore/library/` is created on the first write — a save, an export, a copy-in, a tap on the Library cell — never by a read, never by an ask: a stick only looked at stays untouched; a read-only stick's first write fails.

Network drives are LumaCore's own clients, in process, multiplatform — SMB, FTP, SFTP — never the OS's mounts: `Store Drive "smb1" /Path "//nas/shows" /User …` with the credentials in the station's keystore. An OS-mounted share is only ever a Folder drive someone added by path; `smb1` / `ftp1` / `sftp1` are reserved for LumaCore's connections.

## Consequences
Network drives are decided, not built (planned rows, part 38d). Never probe the OS on the state thread; never create the library on a read. A test gives the watcher a folder to treat as a volume (`LUMACORE_DRIVES_PROBE`). The table and ids: see drives-are-the-os-volumes-probed.
