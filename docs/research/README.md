# Research

Research is input, never a rule; the rules are in `docs/decisions/`. Nothing here is loaded by default: a plan or a record links the file it drew on. Moved here on 2026-10-20 from `handoffs/ui-kit/research/`, `handoffs/2026-10-18-patch-round3/research/` and `issues/` (`docs/plans/active/moves-handoffs.md`). One line per file: topic · date · words.

## ui-kit/ — the touch UI kit (step 12, decisions §30)

| File | Topic | Date | Words |
|---|---|---|---|
| [lumacore-inventory.md](ui-kit/lumacore-inventory.md) | Inventory and critique of the frontend before the kit (every window, overlay, dialog; the touch breakages; what a kit can render) | 2026-10-12 | 5,685 |
| [online-touch.md](ui-kit/online-touch.md) | Touch-first element kits online (Open Stage Control, QLC+, Companion, …), Pointer Events, webview quirks, the recommended architecture | 2026-10-12 | 4,575 |
| [themes.md](ui-kit/themes.md) | Theme editing (REAPER, VS Code, Ableton, Material, Radix); the proposed LumaCore theme model | 2026-10-12 | 3,052 |
| [inspiration-style.md](ui-kit/inspiration-style.md) | The user's style inspiration analysed — the look the kit follows | 2026-10-12 | 729 |
| [layout-format.md](ui-kit/layout-format.md) | Layout description formats (KDL, XML, JSON): why the registry → JSON, screens as the split tree | 2026-10-12 | 2,932 |
| [12d-docs.md](ui-kit/12d-docs.md) | 12d menus: what LumaCore's own docs said about setup, network, protocols, backup before the hardening | 2026-10-13 | 5,030 |
| [12d-backup.md](ui-kit/12d-backup.md) | The Backup menu (Show · Library · Recovery): versions, recovery copies, cleanup rules | 2026-10-13 | 3,612 |
| [12d-feeds.md](ui-kit/12d-feeds.md) | One live socket (`/ws/feeds`) for the whole web UI: State vs Samples, seq / gap, limits | 2026-10-13 | 4,279 |
| [12d-grid-projects.md](ui-kit/12d-grid-projects.md) | `<luma-grid>` + function block: lessons from open-source grids and apps | 2026-10-13 | 3,100 |
| [12d-grid-lasso.md](ui-kit/12d-grid-lasso.md) | One-finger lasso vs scrolling on touch (grids, pools, layout canvases) | 2026-10-13 | 3,939 |
| [12d-rights.md](ui-kit/12d-rights.md) | Rights levels, one level per user; every rights mention in the docs tabulated | 2026-10-13 | 4,439 |
| [12d-remote-station-edits.md](ui-kit/12d-remote-station-edits.md) | Editing another station's own settings: direct, via the master, or both (what other projects do) | 2026-10-13 | 2,635 |

## keys/ — the key editor (the UserProfile editor's Keys tab)

| File | Topic | Date | Words |
|---|---|---|---|
| [2026-10-23-key-editor.md](keys/2026-10-23-key-editor.md) | The KBD / PRG / NAV modes and every key today; how good key editors work (visual keyboards, scopes, modifier layers); the open questions | 2026-10-23 | 1,255 |

## patch/ — the patch overhaul, round 3 (decisions §34; plan `docs/plans/active/2026-10-18-patch-round3.md`)

| File | Topic | Date | Words |
|---|---|---|---|
| [attributes/sources.md](patch/attributes/sources.md) | The attribute dataset's sources (GDTF spec, GDTF Share usage, community.smidn.com) and how `scripts/build.py` rebuilds it | 2026-10-18 | 516 |
| attributes/*.json, attributes/scripts/ | The dataset itself: `attributes.json` (279 GDTF attributes), `feature_groups*.json`, `observed_gdtf_share.json` (attribute names in use on GDTF Share); `build.py` / `cmp.py` | 2026-10-18 | data |
| [community-fixtures/report.md](patch/community-fixtures/report.md) | Map & clone and the parts model tested against real fixture data (part-kind rules, values keyed by part + instance, resampling) | 2026-10-18 | 5,558 |
| community-fixtures/*.py, *.json | The fetch / build scripts against community.smidn.com and their samples (`broad_ids.json`, `sample.json`) | 2026-10-18 | data |

## r1-bugreport/ — the R1 bug report set (the source of the `R1-nn` / `Qnn` ids; `r1-bugreport/README.md`)

| File | Topic | Date | Words |
|---|---|---|---|
| [luma-bugreport-R1.md](r1-bugreport/luma-bugreport-R1.md) | Published console bug fixes vs LumaCore: 72 risks, 10 doc drifts, spec gaps; the Status section tracks the fixes | 2026-10-13 | 4,942 |
| [luma-bugreport-R1-questions.md](r1-bugreport/luma-bugreport-R1-questions.md) | The report's open questions (grey items) with the user's answers, `Q1` … | 2026-10-13 | 2,782 |
| [luma-bugreport-R1-review.md](r1-bugreport/luma-bugreport-R1-review.md) | The R1 fixes reviewed against the docs | 2026-10-13 | 1,036 |
| [luma-bugreport-R1-docs-check.md](r1-bugreport/luma-bugreport-R1-docs-check.md) | The R1 questions checked against the docs | 2026-10-13 | 547 |
