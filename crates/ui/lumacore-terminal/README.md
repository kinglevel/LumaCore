# lumacore-terminal

The terminals: one per Shell — the command line, suggestions, the TUI and the registry of shell
sessions.

## Purpose

Everything a user types into or looks at in a shell, the same for the CLI, the page's terminal
window and the station's own console: line editing and history, the suggestion list, the
full-screen TUI (path bar, output, watches, input line), the station's dialogs drawn as a centred
window, and the session registry that keeps every Shell's frames, keys, lines and gestures in
order. It turns keys into command lines and hands them to the state thread; it runs nothing itself.

## Owns / Doesn't own

- **Owns:** `SessionRegistry` (`session_manager`), `TuiApp` and its drawing, keys and commands
  (`repl`, `repl/tui_*`), the remote shell's ANSI buffer (`repl/command_line`), the TUI's colours by
  role (`repl/tui_look`), `Gesture`.
- **Doesn't own:** the shell socket and the HTTP shell (`lumacore-station`'s `net/`), the state
  thread (`server/`), what a line does (`lumacore-objects`), the dialogs' content
  (`core::dialogs`), the suggestion engine (`lumacore-suggest`), the user's theme (the profile).

## Public surface

- `session_manager::SessionRegistry` — the station's table of shell sessions; the shell server
  and the state thread feed it.
- `repl::TuiApp`, `repl::run_tui` — the station's own console.
- `repl::{ClientView, RemoteCommandLine, RenderedShellFrame}` — a remote shell's frame and line.
- `repl::tui_look::{role, paint}` — colours by role, then the theme pass.
- `Gesture` — a kit control's line as the shell receives it (the gesture channel).

## Rules

- The TUI names colours by role (`tui_look::role`), never a raw colour; one pass paints the user's
  theme. [tidy terminal_colours_only_in_the_look]
- Every hotkey runs a registered command. [tidy tui_hotkeys_run_registered_commands]
- It depends on the objects crate and below only — never on the server, the network or the
  LumaSession code. [compiler + tidy crate_edges_point_down]
- A shell's keys, lines, taps and answers wait in one queue, in order, and are retried, never
  dropped. [unchecked]
- A dialog is the station's one dialog, drawn as a centred window in the profile's theme; the
  answer goes the way every client's does. [unchecked]

## Edges

- **Depends on:** lumacore-cmd-treeshell, lumacore-cmd-keys, lumacore-cmd-tags, lumacore-cmd-showfile, lumacore-cmd-station, lumacore-cmd-devices, lumacore-cmd-protocols, lumacore-objects, lumacore-state, lumacore-tree, lumacore-base, lumacore-wire,
  lumacore-syntax, lumacore-registry, lumacore-suggest.
- **Used by:** lumacore-station.

## Decisions

- [§23 "Step 8"](../../../docs/archive/decisions-2026-10.md) — the layers: terminal below the server, above core
- [§30 "12b"](../../../docs/archive/decisions-2026-10.md) — dialogs, the TUI in the user's theme, the keyboard
- [§30 "12b2"](../../../docs/archive/decisions-2026-10.md) — suggestions in the shell
- [§30 "12f"](../../../docs/archive/decisions-2026-10.md) — the gesture channel
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate (19), its README (18)

## Testing

- `cargo test -p lumacore-terminal` — the session registry and the TUI's units.
- Its tidy rules: `cargo test -p lumacore-tidy` (docs/meta/tidy.md).
