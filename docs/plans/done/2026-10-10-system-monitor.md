# Handoff — System Monitor window (2026-10-10)

Branch: `termiham`. Researched and decided with the user; built (see Status).

## LumaCore today

- `remote::sysmon(ctx, subsystem, msg)` → `"<elapsed> <subsystem> : <msg>"` string; the last 512
  lines in `SYSMON_HISTORY`; live only to TCP monitor clients (`monitor_senders`). Level is a text
  prefix (`[ERROR]`, `[WARN]`).
- **`/ws/monitor` is broken**: nothing feeds its live channel (`sysmon_tx`). In the embedded server
  (`server.rs`, the app's `--server`) the diagnostics snapshot (its history) is never filled.

## Decisions

| Topic | Decision |
|---|---|
| Rendering | **Web list window** over a repaired feed (not the terminal compositor) |
| Feed | `/ws/monitor`: history then live, one JSON per line `{seq, time, level, subsystem, user?, message}`; requires login (any user) |
| Time | **Local wall clock** `21:04:11.203`; elapsed since start in the tooltip |
| Levels | Real field: info / warn / error (today's `[ERROR]` / `[WARN]` prefixes become levels); colours from a SystemMonitor palette |
| Window | Window type `sysmon` "System Monitor", category Info and System; rows time · subsystem · message; newest at the bottom; scrolled up = red background + auto-scroll paused; horizontal scroll |
| Settings | Font size (10–32, default 18), **level filter**, **subsystem filter**, **auto scroll**, **search** box |
| Content | Today's sysmon lines + **every executed command, all users**: `Cmd  [LD@Screen 1] Store Group 5 → ok` / `→ error: …` (one line, passwords masked, tokens never) + **`Echo "text"`** |
| Access | Everyone logged in (the station's log) |
| Retention | Memory ring **5000 lines**; **`library/logs/sysmon-YYYY-MM-DD.log`**, keep **7 days** |
| Keywords | **`Echo "text"`** (command line history + System Monitor). No `List SysMon` |

## Plan

1. **Structured sysmon** (`remote/sysmon.rs`): `SysmonLine {seq, unix_ms, level, subsystem, user,
   message}`; `sysmon()` / `sysmon_early()` build it (level from the prefix, or explicit
   `sysmon_warn` / `sysmon_error`); ring of 5000; text rendering for TCP monitor / stderr unchanged
   in content (plus wall-clock time); a broadcast hook other transports subscribe to.
2. **Log file**: append each line to `library/logs/sysmon-YYYY-MM-DD.log` (writer thread, daily
   rotation, delete files older than 7 days at start / rotation).
3. **Feed**: `/ws/monitor` sends the ring then live lines (JSON) — wired in both `server.rs` and
   `lumacore-server.rs`; `AuthUser` required. Fixes the dead live stream.
4. **Command lines**: after each dispatched command (typed, UI, UDP) one `Cmd` line: user, screen,
   masked command, `ok` / first line of the error.
5. **`Echo "text"`**: `CommandDef` (published); output to the command line and a sysmon line
   (subsystem `Echo`, user).
6. **Window type** `sysmon` in `window_types.rs` (settings: `font_size`, `level`, `subsystems`,
   `auto_scroll`, `search`), picker category Info and System; frontend window kind: virtualised
   row list, colours, red scrolled-up state, filters, search highlight.
7. **Tests**: unit (level parsing, masking, ring, rotation); e2e (`tests/e2e`): open the window from
   the picker, live lines on login, a `Cmd` line per command with the password masked, Echo,
   filters, search, scroll-up red + resume.

## Status (built; unit + e2e tests pass)

- `remote/sysmon.rs`: structured `SysmonLine {seq, time_ms, elapsed_ms, level, subsystem,
  message}`, ring of 5000, `subscribe()` (history + receiver without gaps), `forward_json`,
  `history_json`, `start_log_file` (`library/logs/sysmon-YYYY-MM-DD.log`, local time, 7 days).
  Level from `[ERROR]` / `[WARN]` prefixes. Text form for TCP monitor / stderr unchanged.
- Feed: `/ws/monitor` and `/api/sysmon` need a login; history from `ConsoleBridge::sysmon_history`,
  live JSON via `sysmon_tx`, fed in both `server.rs` and `lumacore-server`.
- `Cmd` lines (`terminal/commands/sysmon.rs::log_command`): typed and UI-dialog commands
  (`process_commands_logged`), UDP commands (both paths, now masked too); the "Tree modified"
  line masks passwords as well. `Echo "text"` is a common console keyword with its own `Echo` line.
- Window type 4 `sysmon` "System Monitor" (Others): `font_size`, `level`, `subsystems`, `search`,
  `auto_scroll`; new `SettingKind::Text`. Web view: shared feed, persistent per-window views,
  toolbar (search, level, LIVE/PAUSED, ⤓), red #400000 when scrolled up.
- Tests: `sysmon.rs` unit tests (levels, ring + subscribers, log pruning), window type settings;
  `tests/e2e/tests/sysmon.spec.ts` (10).

### Notes
- "Tree modified by …" also fires for some failed commands (the tree hash changes) — pre-existing.
- `cargo test -p lumacore-http` does not compile: its test fixtures build `HttpState` without the
  fields added in earlier sessions (`console`, `layout_tx`, `shell_port`, …) — pre-existing.
