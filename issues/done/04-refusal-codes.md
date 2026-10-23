# 04 — Admin refusals inside handlers carry the wrong code

**Severity:** low · **Found:** sweep 2026-10-13 (`role-differs` on CleanStart, Store, Unclaim).

§25 "Step 10": a refusal is E-RIGHTS-001 / E-CLAIM-001; §29 codes everywhere.

| Line | Today | Where | Fix |
|---|---|---|---|
| `CleanStart` as LD | E-CMD-012 | registry says `rights: Programmer` (`registry.rs:~680`), `backup.rs:~451` refuses non-Admins via `out()` | `rights: Rights::Admin` in the registry (its doc already says "Admin only"), drop the handler check |
| `Store/Set/Delete User` as LD | E-CMD-010 | `users.rs:~27 require_admin` → `reply()` (`:78`) | E-RIGHTS-001 (stays in the handler: users may set their own password) |
| `Unclaim` someone else's object | E-CMD-010 | `claims.rs:~188` "Only Admins can unclaim other users", wrapped at `run.rs:~214` | E-CLAIM-001 |
| `Show /MergeDefault` as non-Admin | E-CMD-012 | `backup.rs:~616` | E-RIGHTS-001 (not flagged yet, same bug) |

Folds into issue 10 if the rights design moves these checks into the gate.

## Decision
As in the table. **Fixed** (2026-10-13, on branch `bugfixes`).
