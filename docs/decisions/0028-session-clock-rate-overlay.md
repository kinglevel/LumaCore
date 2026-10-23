---
id: 0028
title: Session clock: a rate overlay that never runs backwards
area: engine
status: built
date: 2026-10-11
built_in: step 9
supersedes: [archive §24 L2318–2324 "clock over the stream heartbeat"]
superseded_by: —
confirmed_by: [R1-04 Q1 property test, Runtime.Health.Clock]
source: [archive §24 L2318–2353, archive §25 L2770–2772]
---
# Session clock: a rate overlay that never runs backwards

## Context
Fades, group-master fades and effect anchors need one time across a LumaSession. The first plan read the offset from the stream heartbeat; a study of Ableton Link, Snapcast, nqptp, PipeWire, chrony, ntpd-rs, statime, linuxptp and game netcode replaced it with a disciplined clock and pings that never queue behind snapshots.

## Decision
`lumacore_base::clock` is a rate overlay on `Instant`: session = base + (Instant − anchor) × (1 + frequency + correction), re-anchored on every change so it stays continuous. It **never runs backwards except one step**: the first lock after joining, or when more than a second off. A member ahead of the master slows down (R1-04, Q1). Fades, group-master fades and effect anchors use `clock::session_ms()`, never wall clock; standalone = the local monotonic clock. Members discipline it from `Time` pings on the master's UDP command port (`lumacore_clocksync`): four timestamps stamped at the socket, a burst on joining or on a new master, then steady pings; an RTT gate, a median offset and an RTT-weighted regression for frequency; correction and frequency are clamped (the bands are constants in the crate); a step, a gap or repeated outliers start a new burst. On master loss every station free-runs at its last frequency (holdover); the elected master keeps its time; members drop their samples, keep their frequency and slew onto it. States and stats are in PTP / NTP terms (Unlocked / Acquiring / Locked / Stable / Holdover; offsetFromMaster, meanPathDelay, jitter, frequency, correction, samples used / rejected, last step) in `Runtime.Health.Clock`, `Info Clock`, `List Health` and the Realtime window; linuxptp is read-only via `lumacore_net::ptp`.

## Consequences
Nothing timed in playback reads `SystemTime` or `Instant` directly. A ping is answered at the socket, never on the state thread (see one-state-thread); the stream heartbeat is for liveness and election only.
