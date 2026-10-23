---
title: API WebUI And UIEngines
category: docs
---
# API WebUI And UIEngines

## Goal

Expose the same show/runtime truth through browser clients and other UI engines
without weakening the server-owned model.

## What Planning Suggests

- Phase 4 is headless server plus web UI.
- Multiple clients should see updates.
- UI engines are distinct from saved user views.

## What We Should Implement

- keep saved views in `ShowData.UserProfiles`
- keep live rendered workspaces in `UIEngines`
- add REST + WebSocket layers later
- treat browser clients as consumers of server truth, not separate truth owners

## Suggested Next Steps

1. Keep `UIEngines` as the live presentation branch.
2. Add a stable server event stream that browser and terminal clients can both
   consume.
3. Reuse `Lum`/tree concepts instead of inventing a separate web-only model.
