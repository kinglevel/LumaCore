---
title: Roadmap
category: docs
---
# Roadmap

What is being built now, next and later — one line per item, each with a link to its plan, record, issue or vision. Done work is in [CHANGELOG.md](../../CHANGELOG.md); the designs are the records in [docs/decisions/](../decisions/README.md) and the plans in [docs/plans/](../plans/active/); ideas not yet decided are in [visions/notes/ideas-2026-10.md](../../visions/notes/ideas-2026-10.md).

## Now

- Finish the Patch tree after 40d: New folder here, Duplicate, the desk capture, a folder's Metadata tab, `List Mvr` paging, a grid `reveal(key)` API ([record](../decisions/0272-nested-fixtures-folders.md))
- Finish 34g4 patch round 3: the missing tests, type / revision / mode change (part 5), Map & clone (part 6), the live columns (part 7), Swap / Multipatch in Edit Patch, the review cell and Bring into scope ([plan](../plans/active/2026-10-18-patch-round3.md))
- Build RunTool / EditTool and the lasso everywhere through the pointer core ([record](../decisions/0196-runtool-edittool-tool-object.md))
- Build the programmer with its Direct / Down switch; the device sheet shows its values ([vision](../../visions/notes/ideas-2026-10.md#programmer-and-device-sheet))
- The device sheet's next round: Dimmer+ and the Auto layer, the attribute board's marks, the marks the engine has no data for yet (relative, phaser, recipes, the DMX tester, parts) ([record](../decisions/0284-device-sheet-window.md))
- Cut the change path's next lever on a big show: per-type fingerprints or reused hashes, the Tab cost on `Fixture 1` ([record](../decisions/0052-one-line-costs-two-fingerprints.md))
- Cut the remaining 34g4 domain crates: patch, media, windows, users, programmer, playback; rename the kernel `lumacore-cmd-dispatch` ([record](../decisions/0245-objects-split-kernel-and-domain-crates.md))
- Finish Lum part 3: the 3b2 leftovers, the new words (3e), the `Root` object and node numbers (3e2), fuzzy Enter (3f) ([record](../decisions/0101-lum-grammar-one-parser.md))
- Finish the Tags follow-ups: the members pane's kind filter, drag to re-parent, Expand all, Go+ / Off, the Draft column, a typed walk for tag links, the Tags cell in grids ([record](../decisions/0208-tags-ui.md))
- Run test speed steps 3–5: no idle doctests, one test binary per crate, desktop-only by default, no fixed sleeps ([plan](../plans/active/2026-10-18-test-speed.md))

## Next

- Settle the sweep's security findings: guests and the `Drives` context, rights groups ([issues 01, 08, 10](../../issues/README.md))
- Put the error code at the end of a refusal's first line ([issue 03](../../issues/03-error-code-placement.md))
- Rerun the long sweep `--fresh` and close the harness leftovers and the build leftovers ([issue 20](../../issues/20-sweep-harness.md), [issue 30](../../issues/30-build-leftovers.md))
- Give every user a default keyword and every profile a start place, the web bar showing the default object ([issue 02](../../issues/done/02-bare-number-change-path.md))
- Validate a save by hash across every station of a LumaSession ([record](../decisions/0021-master-executes-results-flow.md))
- Type the remaining replies: every handler answers `Reply`, `outcome_of` only for text from outside ([record](../decisions/0031-typed-replies-text-at-edge.md))
- Land the one `cargo fmt --all` commit, then `clippy::mod_module_files` ([record](../decisions/0251-build-profiles-local-checks-hygiene.md))
- Give the LumaSession record, the GDTF attribute tables and the station table tree nodes; `navigation_path` as a `TreePath` ([record](../decisions/0106-tree-paths-typed-node.md))
- Finish the snapshot work: per-kind feed revisions, the winning `Value` copy, the §26 programmer snapshot ([record](../decisions/0030-show-snapshot-arc-parts.md))
- Lock output lines to engine frames (line timing step 2), the RTKit fallback, `scheduling` in the Realtime window ([record](../decisions/0044-line-rates-send-pacing.md))
- Group `AppState` gradually and decide the `Walk` derive ([record](../decisions/0237-code-structure-later-items.md))
- Build live links: the `Link` object, `Wire` / `Unwire`, typed nodes, `lumacore-links` ([record](../decisions/0148-links-live-property-links.md))
- Build notifications and the `Team` object; chat as its own design later ([record](../decisions/0149-notifications-teams.md))
- Finish Lum parts 4–5: every line rewritten, the old lexers and copies gone, the shim count at 0 ([record](../decisions/0101-lum-grammar-one-parser.md))
- Decide how a light colour is written and reaches each fixture's emitters ([record](../decisions/0144-color-light-paint-screen.md))
- Build the shared doors and the §35 door leftovers, one pass per handler as its area moves ([record](../decisions/0226-shared-code-one-door.md))
- Move the kernel's dispatch tests into `tests/catalog` feature modules (34g5) ([record](../decisions/0249-three-homes-for-a-test.md))
- Close the protocol leftovers: words written as declared, `Set ArtNet "<Setting>"`, a protocol's own grid columns ([record](../decisions/0048-declared-words-written-as-declared.md))
- Build Export MVR and MVR-xchange after presets and the programmer ([record](../decisions/0268-export-mvr-and-xchange.md))
- Close the drives leftovers: a two-line review cell, Bring into scope, the phone's review, `Eject`, network drives (38d) ([record](../decisions/0089-drive-service-and-network-drives.md))
- Sort every flag into flag groups after an audit of the vocabulary golden ([record](../decisions/0113-flags-and-options-declared.md))
- Give the suggestion engine a free-value index per object kind ([record](../decisions/0173-one-suggestion-engine.md))
- Build Effects: the phaser fields out of `Value`, the effect band, Form / Effect pools, overrides, Pawtern spread, Stomp, cell resolution ([record](../decisions/0071-effects-model-and-control.md))
- Finish the console study's decided items: Effects and Executors as DataPool objects, effect targets per line, the `Copy Group` pawtern bug ([record](../decisions/0071-effects-model-and-control.md))
- Finish step 12's leftovers: the encoder bar, the sequence sheet, the 12d follow-ups, the kit-only tidy rule ([plan](../plans/done/ui-kit.md))

## Later

- Build times: drop `prune_objects` once the toolchain fix ships ([record](../decisions/0250-build-times-stamp-profile-jobs.md))
- Playback depth: priority bands, HTP / Super, executors, speed masters, Lowlight, preview programmer, cue triggers, preset depth ([vision](../../visions/notes/ideas-2026-10.md#playback-depth))
- One `List` / `Info` / `Dump` reading the registered objects ([record](../decisions/0126-list-info-dump-one-command.md))
- Show, references and ownership: export any object as JSON, a references view, network-synced autosave, a LumaSession password, asset drives, the Import dialog, the rest of the tree ([vision](../../visions/notes/ideas-2026-10.md#show-references-ownership))
- LumaSession and network: join by address, patch version negotiation, Invite's user, `Time` pings by host, PTP on Linux, CORS / TLS, per-client overlays ([vision](../../visions/notes/ideas-2026-10.md#lumasession-and-network))
- Patch and DMX: live Pan / Tilt offsets, a DMX tester, input limits, discovery paging, lost sources, TRU, the parameter budget, a GDTF fuzz target ([vision](../../visions/notes/ideas-2026-10.md#patch-and-dmx))
- Control input: an OSC address space of our own ([vision](../../visions/notes/ideas-2026-10.md#control-input-osc))
- Object paint's next round: a gradient and "by fixture type" across a selection, tinted rows in the station-wide menus (Network, DMX Protocols: built once for every viewer), the pool cell drawing its icon beside the paint ([record](../decisions/0294-object-paint-palette-free-theme-links.md))
- UI leftovers: readouts, shortcut mode per screen, encoder bar modes, echoes, keyboard tabbing, the frontend review findings ([vision](../../visions/notes/ideas-2026-10.md#ui-leftovers))
- Developer scripts: a second station in the launcher, remote hosts, the Windows twins ([vision](../../visions/notes/ideas-2026-10.md#developer-scripts))
- Playback words from the R1 report: relations / virtual dimmer, Go- / Goto timings, the window scroll report ([report](../research/r1-bugreport/luma-bugreport-R1.md))
- After the merge of bugfixes: open ranges in `Set <rows>`, a `busy` test, a retried remote edit; the known failing e2e ([vision](../../visions/notes/ideas-2026-10.md#tests-and-known-failures))
- Feedback system leftovers: the Message Center, the envelope, the `client` audience, CLI feedback lines, the progress API ([plan](../plans/done/2026-10-10-feedback-system.md))
- Thin client leftovers: HttpOnly cookie, per-user feeds, the macOS bundle, `Connect` completion, a manual GUI pass ([plan](../plans/done/2026-10-10-thin-client.md))
- UI keys leftovers: plugins, double presses, safe mode, the `--clean-start` test ([plan](../plans/done/2026-10-10-ui-keys.md)); the key editor's leftovers — the shortcut mode per screen, NAV's own keys as rows, hold = repeat, line-guarded and state-computed cycles, executors on Ctrl / Alt + F1–F8, the numpad's names, the F10 cheat sheet, export / import ([plan](../plans/active/2026-10-23-profile-and-key-editor.md))
- WebUI: the e2e under Playwright's WebKit on Linux, a Windows build of the app ([record](../decisions/0037-wire-crate-clients.md))
- The console study's open questions, in order ([vision](../../visions/notes/ideas-2026-10.md))
- Later words from §40 and the `Part` word for cue parts ([vision](../../visions/notes/ideas-2026-10.md#later-words))
