---
id: 0012
title: User, Show and Global variables
area: commands
status: decided
date: 2026-10-10
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §5 L281–291, archive §15 L1189–1191]
---
# User, Show and Global variables

## Context
Macros and keys need values that outlive one line. The March design gave variables three scopes with different lifetimes, following where each kind of state has its home (see state-homes-by-type). The one-registry work (§15, 2026-10-10) deferred the syntax to a hardening pass after the step 12 UI work.

## Decision
Three tiers. A UserVariable lives on the user's profile, is saved and follows the profile on `Export` / `Import UserProfile`. A ShowVariable lives on `ShowData` (`Variables`), saved with the show. A GlobalVariable is session runtime — volatile, gone on restart, replicated with `Runtime` (step 6b lists global variables among the runtime split). The syntax is `$name`: an unquoted `$name` expands, a quoted one is text; the setter is `SetUserVar` for the user tier, with `Show.$name` and `Global.$name` addressing the other two. The words are declared in the catalog like every verb and follow lum.md (see the Lum syntax record) when they are built.

## Consequences
Not built: the step 11a audit records only the global tier existing in code; the user and show tiers, the expansion and the setter are open. Building it is a syntax change, so lum.md and its hard discussion come first; the show tier is a new `ShowData` field with its subtree arms and file entry (see change-path-undo-subtrees); the user tier is a profile field, never on `User` (see programmer-owned-by-user); the global tier stays in `Runtime` and is never saved. Any value a variable holds in a stored line stays command text.
