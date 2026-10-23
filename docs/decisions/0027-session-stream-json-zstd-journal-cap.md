---
id: 0027
title: The stream stays JSON plus zstd; journal capped
area: lumasession
status: built
date: 2026-10-14
built_in: the code review's "Data and LumaSession" (built 2026-10-14)
supersedes: [archive §21 L1398–1400 "length-prefixed JSON frames" (refined)]
superseded_by: —
confirmed_by: [lumacore_base::embedded_json, StreamMsg::Encoded]
source: [archive §31 L6738–6752]
---
# The stream stays JSON plus zstd; journal capped

## Context
The code review found waste on the LumaSession stream, not a wrong format: shows embedded as JSON inside a JSON string, snapshot bytes as number arrays, and every broadcast encoded once per member. 6b's choice of zstd-compressed JSON frames stands; a binary encoding was declined unless measurements still point at the stream.

## Decision
The stream (`lumacore_lumasession::stream`) carries length-prefixed, zstd-compressed JSON frames; `StreamMsg` is adjacently tagged (`kind` + `body`), because an internally tagged enum buffers its fields and can't carry raw JSON. Embedded shows (`Welcome` / `Preview` / `MergeOffer`) pass through as raw JSON (`lumacore_base::embedded_json`: `text` / `text_opt`, the fields stay `String`); snapshot bytes that are JSON travel as that JSON (zstd sees the text), other bytes as `{"b64": …}`. The master encodes a broadcast once for two or more members (`StreamMsg::Encoded`, local only, `encode_frame`) and sends the same bytes to each. Files a member lacks travel by SHA-256 (`FileWant` / `File`). The replication journal is capped by entry count or bytes, whichever comes first; a member behind the trimmed range gets a full snapshot (see master-executes-results-flow). Binary (postcard) only if measurements still point at the stream.

## Consequences
Every frame change bumps the current `SESSION_PROTOCOL` in `lumacore-wire`, no shims. A new message carrying a show or snapshot uses `embedded_json`, never a JSON string. The byte cap is decided; the archive does not state it built — check the journal before relying on it. UDP keeps only small messages (see one-state-thread for the command port's reply limit).
