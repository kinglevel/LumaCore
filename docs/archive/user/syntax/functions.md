---
title: Functions
category: docs
---
# Functions

Canonical language source: `docs/user/syntax/lum.md`

This document captures the current intended high-level definition of
`Function` in `Lum`.

## Principle

In `Lum`, a `Function` is the action or operation a sentence performs.

A function may:

- inspect
- change
- assign
- store
- recall
- navigate
- claim ownership

Examples:

```text
Info Fixture 1
Store Group 1
Delete Preset 2
Assign Universe 1 At sACN 1
At 50
Set Fixture 1 Label "Front Left"
```

## Main Function Families

- inspection
- control
- edit
- assignment
- navigation and scope
- ownership

## Ownership Functions

Ownership in LumaCore should use normal command language.

Primary ownership functions:

- `Claim`
- `Unclaim`

Rules:

- `Claim`
  - takes editable ownership of an object
  - with no explicit object, should act on the current scoped object
- `Unclaim`
  - releases editable ownership where policy allows it
- claimed objects are still viewable through inspection functions
- a user may claim multiple objects at once
- the `Programmer`
  - is always claimed by its owning user
  - is not a normal shared-edit claim

Examples:

```text
Claim
Claim Patch
Claim Timecode 1 Group 1
Claim ShowData
Unclaim Patch
Info Patch
List Patch
```
