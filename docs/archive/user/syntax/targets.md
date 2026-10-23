---
title: Targets
category: docs
---
# Targets

Canonical language source: `docs/user/syntax/lum.md`

This document captures the current intended high-level definition of `Target`
in `Lum`.

## Principle

In `Lum`, a `Target` is where an object or function is directed.

A target is usually used when a sentence expresses:

- assignment
- routing
- destination
- relationship

Examples:

```text
Universe 1 Assign sACN Universe 1
Copy Group 1 At 2
Assign Timecode 1 Sequence 4
Set Fixture 1 Label "Front Left"
```

## Main Target Families

- object targets
- protocol targets
- property targets
- slot or position targets
