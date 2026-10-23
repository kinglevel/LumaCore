# Build times

Fresh builds of server + CLI + app, every crate and dependency (`./scripts/unix/build bench [profile]`,
decisions §34 "Build times hardened" 5). One row per run, appended by the script.

| Date | Machine | rustc | Profile | Jobs | Crates | Server | CLI | App | Time | Commit |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-10-18 12:21 | Apple M3 Pro · 12 threads · 18 GB · macOS 26.6.2 | 1.99.0 | local | 10 | 380 | — | — | — | 2m16s | d0118f3dac55 |
| 2026-10-18 12:43 | Apple M3 Pro · 12 threads · 18 GB · macOS 26.6.2 | 1.99.0 | local | 10 | 380 | — | — | — | 2m26s | c1276bfe3ef2 |
| 2026-10-18 12:44 | LumaCoreA · QEMU Virtual CPU version 2.5+ · 16 threads · 22 GB · Linux 6.12.85+deb13-amd64 | 1.99.0 | local | 14 | 408 | — | — | — | 2m56s | c1276bfe3ef2 |
| 2026-10-18 15:03 | Mac · Apple M3 Pro · 12 threads · 18 GB · macOS 26.6.2 | 1.99.0 | local | 10 | 380 | 1m27s | 7s | 11s | 1m45s | ab7fca5e8e9c |
| 2026-10-18 15:05 | LumaCoreA · AMD Ryzen Threadripper 1950X 16-Core Processor · 28 threads · 34 GB · Linux 6.12.85+deb13-amd64 | 1.99.0 | local | 26 | 408 | 1m47s | 13s | 50s | 2m50s | ab7fca5e8e9c |
