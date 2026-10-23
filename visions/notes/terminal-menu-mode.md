---
title: Terminal Menu Mode
category: docs
---
# Terminal Menu Mode

**Date:** 2026-03-11
**Status:** Design note (future phase)

## Concept

The terminal should have a second mode alongside the command prompt: a **menu mode** for common operations that don't need the full command syntax.

## Menu Structure

```
Terminal Menu Mode

Backup (Save/Load show)
Patch
Network
Protocols
Station settings
Logout
```

## Notes

- Simple navigable menu (not the full command prompt)
- Covers common setup/admin tasks
- Could be the default mode on first launch before a show is loaded
- Accessible via a key binding or command from prompt mode
