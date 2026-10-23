---
id: 0002
title: Programmer and selection belong to the User
area: programmer
status: built
date: 2026-10-11
built_in: step 6a
supersedes: [archive §2 L171–176 "Arc-wrapped RwLock Programmer", archive §10 L459–461 "programmer on UserProfile"]
superseded_by: —
confirmed_by: [ShowSubtree::Programmer, ShowSubtree::Selection, ShowData::programmer_of]
source: [archive §2 L171–192, archive §10 L450–469, archive §21 L1360–1362, archive §21 L1372–1380]
---
# Programmer and selection belong to the User

## Context
The programmer was first an `Arc<RwLock<Programmer>>` shared with the output thread and kept on the `UserProfile`. The LumaSession design (§10, 2026-10-10) found that a profile is only how a user works — views, screens, keys, settings — while the programmer is the user's work, which must survive a profile switch and be the same on every station the user is logged in on. Step 6a (2026-10-11) moved it; old shows start with empty programmers, no conversion. The March prompt modes (`[Programmer:Select]`, `[Programmer:Grid]`) were never built.

## Decision
The programmer and the fixture selection live on `ShowData.Users.{n}` (`User.programmer`; read through `ShowData::programmer_of` / `programmer_of_mut`, `AppState::current_programmer`). They are saved with the show and replicated; the same user on several stations shares one programmer, one selection and one set of modes. A `UserProfile` carries no programmer and no undo stack. One programmer per user, stored so it can become several later. The state thread owns it; the engine reads published snapshots, one `UserLayer` per user, never a lock. `Store` records the acting user's own active values. `Clear Programmer User 2 Thru 5` clears other users' programmers (Admin and Setup); each cleared programmer's undo step is filed on its owner (`AppState::undo_to_owners`) and the owner is told.

## Consequences
Never put programmer or selection state on a profile, in `Runtime` or in a station file. Undo and replication address them as `ShowSubtree::Programmer(user)` (values) and `ShowSubtree::Selection(user)` (fixture + one-only selections); `User(n)` is the user without its programmer. Any new per-user working state goes beside them on `User`, and the compiler refuses a new `ShowData` field until it has its subtree arms (see change-path-undo-subtrees). Mode flags are not here: they are session runtime (see mode-objects-verb-first). How values enter a programmer is programmers-ltp-fixture-claims.
