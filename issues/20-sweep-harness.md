# 20 — Sweep checks that report false findings

**Found:** 2026-10-13 analysis of the sweep run (`target/fuzz-report/`). Code in
`crates/lumacore/src/sweep/`.

| # | Problem | Findings it caused | Fix |
|---|---|---|---|
| a | `world.rs:~292` `std::mem::take(&mut self.master.show_replaced)` clears the flag before `replicate()` reads it (`:~382`), so after NewShow / LoadShow / CleanStart the simulated member never gets the snapshot. The real server is right (`link::flush`, `lumasession/link.rs:~677`). | all 7 `replication` ("op N out of order", CleanStart "slot 1 holds another object"), 7 `member` | read the flag instead of taking it (one line; verified in a scratch build: replays clean) |
| b | No resync model: after one failed apply every later op is "out of order" (the real member requests a resync, `link.rs:~414`) | cascades | on a gap or failed apply re-snapshot the member and report only the first failure |
| c | Sandbox shared across roles: Admin's `NewShow Q` makes LD's fail (E-CMD-012) | `role-differs` NewShow, DeleteShow, LoadShow | `sandbox.refresh()` before each role when the case's `line_reach` is Drive |
| d | `role-differs` compares `changed` literally (`run.rs:~474`) | `show "demo" /load`, `set descriptions off` | compare error-or-not; compare `changed` without user-owned subtrees (Profile, Programmer, Selection, User, Usage); skip when the show was replaced |
| e | `read-back` on relative input: `expected_value` calls `props::parse("+ 1")` → None | `Set Output 1 Priority + 1` → 101 (correct, default 100) | apply `props::relative` for numeric props, or skip when the raw value starts with an operator |
| f | Open Patch draft: `heavy()` compares the drafted master with the member | `member` Devices (`Full Solo Zero`) | compare `patch_draft::live_view` |
| g | Tree-shell words get in when a line names `Drives`: `line_reach` takes the max (Show < TreeShell < Drive), `vocab::swept` only excludes `== TreeShell` (`lines.rs:~195`, `vocab.rs:~44`) | 11 `drive-escape` (`ls/cat/du/… Drives`) | `line_reach` reports "touches the tree shell" as a flag (or reject lines whose statement word is TreeShell) |
| h | `order_diff` (`world.rs:~500`) doesn't track closed objects, prints a wrong path and says "order" | `save-load-order` label | report the first differing text line |
| i | A `;` line's finding is filed under its first word | `Join ;`, `Library ;` | key `read-only-changed` by the statement that changed something (cosmetic) |
| j | One `master.context` shared by all users (ContextSwitch) | made issue 08 reachable | per-user context, like the server |

Already fixed (2026-10-13, this commit): the sandbox grew without limit (every `SaveShow` kept
a ~1 MB show; 403 MB filled the disk) — now copied fresh per random sequence and above 64 MB in
the sweep phase (`SANDBOX_CAP`). Also: the random phase said "done" when stopped.

Not real: 7 `crashed` on `Video Shell UIEngine UIEngine Sequence` — the disk was full; alone the
line replies E-CMD-004.

## Decision
Fix a–h as in the table. **Awaiting go.** After a–c, rerun the sweep (`--fresh`): any member
finding left is real.
