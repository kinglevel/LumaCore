---
id: 0038
title: Clients never draw 3D; one renderer on the station
area: webui
status: decided
date: 2026-10-18
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §36 L11130–11160]
---
# Clients never draw 3D; one renderer on the station

## Context
Drawing 3D in every client engine would meet WebKitGTK's WebGL slow paths and NVIDIA / Wayland breakage, and a phone can't shade a rig. The March DESK-02 note and `visions/notes/client-types.md` point the same way.

## Decision
**Clients never draw 3D.** Every 3D window is a video stream rendered on the station; the client only shows it. Camera gestures are command lines or window settings sent to the station. The WebUI uses no WebGL / WebGPU in any view — live views (faders, DMX, Realtime) stay DOM and 2D canvas. **One wgpu renderer, two levels**: Basic (wireframe, never shaded) and Full (shaded); quality is a `RenderQuality` pool on the profile (locked defaults plus the user's own), picked per 3D window, each object with `Render: Basic | Full` and graded levels. **RenderNode is a station service** like LumaSession / Shell / HTTP: `Set Interface "en1" "RenderNode" Yes|No`, station-local, filtered like the others; a 3D window streams from a render node in the LumaSession and says so when none is reachable; the DMX render thread keeps priority on a station that is both. Source `Output | Blind | Preview` with `Follow` (default on) showing Blind or Preview while you are in it; Preview — sequences running without touching the output — is a new feature hardened as its own section first. The stream pipe is decided later: low-latency one-way video to any client engine, the station's hardware encoder where there is one, gestures back as command lines, reusable for media previews and cameras; transport and codec researched when the 3D step starts.

## Consequences
**Not built.** Nothing in the kit may add WebGL / WebGPU meanwhile; a 3D window waits for the stream pipe. The service follows `ServiceAccess` (see lumasession-membership-services); Preview and the pipe each get their own record first.
