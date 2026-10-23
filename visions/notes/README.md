---
title: Vision Notes
category: docs
---
# Vision Notes

Design notes that describe ideas, not built features. They moved here from `docs/` in
step 11a (decisions §29) so `docs/` only describes what is built or decided. They are
from 2026-03 and may not match the current code.

| Note | Moved from | What |
|---|---|---|
| [client-types.md](client-types.md) | `docs/dev/design/` | Client types and selective loading: not every client needs the whole show. |
| [global-users.md](global-users.md) | (new, 2026-10-15) | Users and Teams known beyond one show, so people elsewhere can notify / chat with them; what today's design keeps open. |
| [controllers-and-stations.md](controllers-and-stations.md) | `docs/dev/design/` | Stations with attached controllers (joysticks, MIDI, …) and a driver ecosystem. |
| [terminal-menu-mode.md](terminal-menu-mode.md) | `docs/dev/design/` | A menu mode in the terminal beside the command prompt. |
| [transport-types.md](transport-types.md) | `docs/dev/design/` | An address space beyond DMX universes for other transport protocols. |
| [watch-quake-panel.md](watch-quake-panel.md) | `docs/dev/design/` | Watch output in a drop-down "Quake terminal" panel in the TUI. |
| [effect-engines.md](effect-engines.md) | `docs/user/effects/` | Plugins, custom effect engines and a Commands Pool. The effects that will be built are decisions §28. |

The ideas are also listed in [docs/meta/roadmap.md](../../docs/meta/roadmap.md) → Ideas.
When one is picked up, its spec goes into
[docs/archive/decisions-2026-10.md](../../docs/archive/decisions-2026-10.md).
