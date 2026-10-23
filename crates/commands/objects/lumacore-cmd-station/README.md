# lumacore-cmd-station

The station's own words: what this running station is, shows and does to itself.

## Purpose

The station and its network — the console's Network screen (decisions §34 25.7, 25.17). The
station table: Station (this one's identity settings — Name, Priority, Online, HoldOutput, … —
another station's, sent to it, the rows made in advance, Store / Delete Station), Interface (the
NICs and the services each serves) and Network (a data network the show names, and this
station's NIC for it), the wiring lines for this and other stations with their asks, and the
Network / LumaSessions menus. And the running station's objects — Server (About), Shell, Client,
Clock, Journal, Health, UIEngine, Watch — with Restart and Shutdown (their "save first?" is
`lumacore-cmd-showfile`'s, which this crate sits on), Shutoff, Disconnect, Connect, Watch /
Unwatch and `<line> /Watch`, Echo, Debug, HissMe, Banner and the splash. The DMX lines on a
network are `lumacore-cmd-protocols`', which sits on this crate. Its hooks: `/Watch`, `Set Station
"Tower" …` checked on that station (`Hook::SetCheckedElsewhere`), Import Show's Network and
Station (`Hook::ImportByGuid`).

## Owns / Doesn't own

- **Owns:** the rows (`rows`), Station / Interface (`stations`), the table's rows (`table`),
  the station's settings (`settings`), the wiring (`wiring`), Network (`network`), the menus
  (`menus`), Shutoff / Echo / Debug (`words`), the running station's objects (`objects`), Watch
  (`watch`), the splash (`splash`).
- **Doesn't own:** the station table's door (`lumacore-state` `stations`), the plan that applies
  the wiring (the kernel's `protocol_plan`), the DMX lines (`lumacore-cmd-protocols`), saving
  before power (`lumacore-cmd-showfile`).

## Public surface

- `register` — the rows and the `/Watch` hook, after the show file's.
- `splash` — the REPL's start art; `handle_watch`, `watchable` — the TUI's watch keys.
- `STATION`, `INTERFACE`, `NETWORK`, their settings, `PRIORITY_CHOICES`, `cmd_delete_station`.

## Rules

- Only read-only lines are watched. [test only_read_only_lines_can_be_watched]
- HissMe is the station's own level, never a user's. [test hissme_is_internal]

## Edges

- **Depends on:** lumacore-registry, lumacore-base, lumacore-wire, lumacore-syntax,
  lumacore-syntax-derive, lumacore-tree, lumacore-protocols, lumacore-net, lumacore-objects,
  lumacore-cmd-showfile, lumacore-luma, lumacore-sysmon.
- **Used by:** lumacore-cmd-devices, lumacore-cmd-lumasession, lumacore-terminal, lumacore-sweep,
  lumacore-tidy.

## Decisions

- [§29](../../../../docs/archive/decisions-2026-10.md) — Watch, R1-65
- [§34 25.7](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-cmd-station` — the station's words line in, reply out.
