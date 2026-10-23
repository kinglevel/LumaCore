---
id: 0230
title: The station only assembles: the catalog and thin binaries
area: station
status: built
date: 2026-10-16
built_in: 34f11
supersedes: [archive §34 L8064–8079 "station splits into", archive §34 L8080–8083 "Binaries live with", archive §34 L8084–8087 "station moves to"]
superseded_by: —
confirmed_by: [the_station_only_assembles, crates_sit_in_their_group]
source: [archive §34 L8064–8090 (items 19, 20.1, 20.2), archive §34 L8303–8314 (24.1), archive §34 L8332–8345 (24.3, 24.4), archive §34 L8798–8802, archive §34 L8866–8877, archive §34 L8892–8909]
---
# The station only assembles: the catalog and thin binaries

## Context
Item 19 planned the station's split in four steps; item 20 hardened it the same day to "the station holds no library code" and put each binary with its crate; item 24.1 reversed that last point, because every binary needs the whole catalog and a crate owning one would depend on every crate — a second assembler. 34f11 compared rustc's few-line `main` over `rustc_driver`, Bevy's facade, Servo's servoshell and the binary crates of cargo, uv, ruff, Zed and Deno.

## Decision
`apps/lumacore-station` builds the program and holds no library code: its `Cargo.toml` (every crate it builds with; the features `http`, `perfetto`, `tracy` forwarded to the crates), its README, `src/catalog.rs` (`install_catalog`, the one list of every crate's `register`) and a thin `main` per binary in `src/bin/` — `lumacore` (the REPL), `lumacore-server`, `lumacore-sweep`, `gen-demo`, `gen-docs`. Thin means parse arguments, install the catalog, call one library function. The server never installs the catalog: `require_catalog` refuses to start without one. There is no facade crate for our own crates; every crate names the crate it uses. The station is the one app outside `crates/`, beside the clients (see wire-crate-clients).

## Consequences
New code never goes into the station; it goes into a crate of its group and the station gains a `register` line. `the_station_only_assembles` allows only `src/catalog.rs` and `src/bin/*.rs`; a crate on the station needs no registry edge of its own. A build without the terminal or HTTP is a feature choice in the station's manifest. Open: none.
