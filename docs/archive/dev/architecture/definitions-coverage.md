---
title: Definitions Coverage
category: docs
---
# Definitions Coverage

`systemtest` should always follow the active definitions.

This is important because the definitions describe the intended language,
ownership rules, session behavior, object model, and runtime meaning of the
system. If the code changes but `systemtest` does not change with it, the
toolchain starts protecting old behavior instead of the intended behavior.

Rules:

- `systemtest` should treat the active definitions as test requirements, not as
  optional documentation
- when a definition changes command meaning, ownership behavior, session
  behavior, object identity, or runtime structure, `systemtest` should be
  updated in the same work
- [docs/user/syntax/lum.md](../user/syntax/lum.md)
  should be treated as the canonical language reference when command behavior is
  tested
- session, station, client, claim, derived selection claim, and authority rules
  should be represented in curated system tests
- examples in the command registry should continue to be swept, but curated
  definition-driven tests are just as important
- a passing systemtest run should mean the current implementation still matches
  the current definitions closely enough to trust

If a definition introduces a new first-class behavior, `systemtest` should gain
coverage for it as early as possible.
