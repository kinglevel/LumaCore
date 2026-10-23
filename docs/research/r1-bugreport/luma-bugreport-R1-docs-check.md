# R1 questions vs the docs (2026-10-13)

Each question in [luma-bugreport-R1-questions.md](luma-bugreport-R1-questions.md) checked against
`docs/archive/decisions-2026-10.md` ("dec"), the roadmap, user/dev docs and CLAUDE.md (read only).

## Settled by the docs — built as the docs say unless overruled

| Q | Docs say | Source | vs ★ |
|---|---|---|---|
| Q1 | ahead → slow down; step only before the first render after joining or > 1 s (panic, sysmon). Fix the "never runs backwards" wording to match | dec:1894-1896 | ≈ ★b (hold-until-lock and rebasing fades not stated) |
| Q4 | fuzzing looks for panics and hangs, not memory use | dec:2318-2320 (§25) | **b**, against ★ |
| Q5 | no input size caps; only the 1 GiB preallocation + O(n²) shell scan are fixed | dec:2318-2319 (§25) | **b**, against ★ |
| Q7 | a refused line changes nothing | dec:2367 | ★a |
| Q12 | Off Sequence snaps (OffCue later) | dec:2169, 2532 | ★b for Off Sequence; release timing not stated |
| Q17 | Blind removes the programmer layer only | dec:2132, 798, engines.md:269 | **b**, against ★ |
| Q19 | the universe is created | fixtures-patch-output.md:63 | ★a |
| Q27 | only the station that sends opens input sockets; OSC is an input line | dec:630, 567, 640 | ★ |
| Q28 | unlock with the locker's password (Admin: `admin`); station-local, never saved | dec:3534-3538 | ★a |
| Q29 | a claim is refused only when another user holds the attribute *live* | dec:253-254 | **b**, against ★ |
| Q30 | broken link shows a "missing …" row until changed | dec:3631, 3924 | ★ |
| Q33 | layouts are profile data; sharing a profile shares the look | dec:995-997, 3340 | **a**, against ★ |
| Q35 | the station asks (line confirm), `/NoConfirmation` skips; no setting | dec:3580, presets.md:173 | **b**, against ★ |
| Q41 | stations map networks / asset roots by name | dec:553, 3322 | ★a (write the exception down) |
| Q43 | legacy parsing removed now (§21 no compat), fields with Effects | dec:943, 2459, roadmap:330 | ★c |
| Q46 | rights shown disabled, not hidden | dec:3074, 3588 | ★a |

Partly settled: Q6 typed order, `-` subtracts, open ranges allowed (dec:187, 1446, commands.md:46,
macros.md:227) — missing-id rule open. Q13 Goto asserts (dec:2136) — timing / Go- open.
Q18, Q20, Q23, Q31 — the docs exclude option c only.

## Docs contradict themselves
- Q8 `At + 10`: dec:3621 "against the current value" vs dec:3694 "programmer value (0 when none)".
- Q11 abbreviations: §14 dec:746 (shortest prefix) vs §15 dec:775 (aliases only).
- Q16 equal values: dec:2135 "blocked" vs dec:2168 "changes only".
- Q26 Invite: dec:674 "joins without asking and takes the session's show" vs dec:1107 / lumasession.md:246 "operator decides"; default `merge_default` = Merge, not Use Mine.
- Q37 autosave: §9 dec:402 `.autosave/` vs show-files.md:100 "saved in place"; LumaSession autosaves "kept until deleted" (dec:1233).
- Q44 Hi/Lo: §28 dec:2567 makes `Lo`/`Hi` registry words (Planned) — ★ makes them properties.
- Format number: CLAUDE.md says 10; crate-map.md:110, lumasession.md:121, dec:27 say 7.

## Not answered
Q2, Q3, Q9, Q10, Q14, Q15, Q21, Q22, Q24, Q25, Q32, Q34, Q36, Q38, Q39, Q40, Q42, Q45.
