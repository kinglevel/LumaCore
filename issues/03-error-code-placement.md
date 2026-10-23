# 03 — Error code not at the end of the first line

**Severity:** medium (Journal records errors as uncoded; sysmon hides the code) · **Found:** sweep
2026-10-13, 140 `uncoded-error` findings — 139 do carry a code, in the wrong place.

The Journal (`dispatch.rs:~219 journal_result`), the System Monitor (`sysmon.rs:~211`, first line
only) and the sweep (`sweep/world.rs:~589 error_of`) read the code from the end of the first
`Error:` line. decisions "### Errors" (~line 2077) sets `Error: <message> [E-AREA-NNN]`. Clients
(`console_bridge.rs:50`, frontend) only test the `Error:` prefix.

## Causes
1. **`;` lines refused by rights** (131 findings): `dispatch.rs:~567` appends
   ``— nothing in `<line>` ran`` after the already rendered `[E-RIGHTS-001]`
   (`rights::check`, `rights.rs:64`). Repro: `Guest  Fixture 1 Thru 10 ; Next`.
2. **Multi-line messages** (Usage + Example) get the code on the last line: Attribute
   (`control.rs:~369`), Bind (`keys.rs:~145`), Unbind (`keys.rs:~202`), `ASSIGN_USAGE`
   (`universe.rs:~127`), Label (`edit.rs:~297`), `List` unknown target (`list/mod.rs:~74`),
   not hit yet: `info.rs:~872`. Repro: `Admin  Unbind`.
3. **`Info Error 1`**: really uncoded — `errors.rs:~378` calls `text::render_error` directly.
4. **`Claim Fixture 1 Thru 3 /force`**: `run.rs:~168/173` join one error per target with `\n`
   and code only the end; `errors::reply` (`errors.rs:~331`) adds no code if any line already has
   `[E-`.
5. **tidy `error_replies_carry_a_code`** (`tidy.rs:~517`) only flags `render_error(` calls and
   skips all of `core/errors.rs` (line ~524), so 3 slipped through.

## Fix (decided)
- `errors::with_code` / `render_code` put `[E-…]` at the end of the **first** line
  (`Error: Usage: Unbind <key> [E-CMD-006]` then the example). Unit test for a multi-line message.
- Dispatch builds the `;` refusal from the plain message: `rights::refusal(...)` returns the text,
  dispatch renders `"<msg> Nothing in `<line>` ran."` with E-RIGHTS-001.
- `errors.rs:378` → `render_code(…, "E-CMD-007")`.
- `claim`: code each line (`errors::with_code`) before joining.
- tidy: exempt only the bodies of `render` / `render_code` / `reply`, not the whole file.

**Decided** (user, 2026-10-13): code at the end of the first line. Build awaiting go.
