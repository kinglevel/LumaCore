---
name: build-bench
description: The build profiles, the timed benchmark build and the remote build servers. Use when measuring build time, building for shipping, or building on a build server.
---

# Build profiles and the bench

The scripts build `[profile.local]` by default (release plus incremental, `target/local/`); `--ship` is the full release profile, `--debug` the dev profile. `jobs` is set in `.cargo/config.toml` (all cores minus two). Every build ends with one line: finished, duration, crates.

```bash
./scripts/unix/build                                   # the server, local profile
./scripts/unix/build bench [local|release|dev]         # a fresh timed build of every crate → one row in docs/dev/build-times.md
./scripts/unix/build bench --remote                    # the same on every build server; their rows land in the same file
```

The git commit is stamped only by the station (`apps/lumacore-station/build.rs`); never add a `build.rs` or an `env!` stamp lower in the graph, a commit would recompile everything above it.

**Build servers** are listed in `buildservers.local.env` at the repo root (machine-local, gitignored: `LC_BUILD_HOSTS`, `LC_BUILD_DIR`; key auth, use `ssh -o BatchMode=yes`). Never copy them into a tracked file. Remote login shells are fish: wrap commands in `bash -lc '…'`. A server builds what is pushed: push, then `git pull --ff-only` there; never edit code on it.

Keep one cargo process per machine; two builds on one target directory take turns on cargo's lock and double the time. Disk fills before RAM does: watch `target/` (`du -sh target`; `scripts/unix/prune_objects` runs from the hook).
