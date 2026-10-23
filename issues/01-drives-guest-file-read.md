# 01 — Guests read host files through `Drives`

**Severity:** security · **Found:** 2026-10-13, while analysing the sweep's drive-escape findings
(the sweep's sandbox refused them; on a real server nothing does).

## What happens
`Root.Drives` maps to the real filesystem root (`vfs::drives_root`, `core/vfs.rs:180`), and the
tree-shell words (`ls`, `cat`, `tree`, `du`, `head`, `tail`, `wc`, `stat`, `grep`, `diff`) are
`Rights::Guest`. So a Guest shell or browser tab can list `/` and read any text file the server
user can read (`cat` reads up to 500 lines, `treeshell.rs:104`; `resolve_drive_path` matches
dotted names, `vfs.rs:255`) — including `library/station_settings.json`, which holds
`token_secret` (the HMAC key for login tokens) and `revoked_tokens`, and e.g. `~/.ssh` keys.

Verified by code path, not on a live server.

## Repro
As Guest: `ls Drives`, then `cat Drives.<abs path>.library.station_settings.json`.

## Docs
§19 "Step 5 hardening": one rights gate; rigs are isolated, security light. §29: Quicksheet,
List, Enter, Watch never walk into Drives unless the path names it. Nothing says a Guest may
read the host disk.

## Options
1. Anything reading or listing under `Root.Drives` needs Admin, checked in the one rights gate
   (by target, not only by word).
2. Same with Programmer instead of Admin.
3. Map `Drives` to `library/` and the mapped `AssetRoot`s only, never `/`.
4. In any case: never show station secrets (`token_secret`, `revoked_tokens`, keystore) in the tree.

## Decision
The user wants rights groups modelled on a reference console's levels (issue 10) rather than a one-off. Proposed interim:
option 1 + 4 now (small), replaced by the rights-group design later. **Awaiting go.**
