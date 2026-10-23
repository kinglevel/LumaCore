---
title: LumaCore SB tab bar analysis
category: vision
---
# Tab and Navbar Resizing Analysis

## Scope
This document analyzes **only** the tab system and top/navbar resizing behavior visible in the provided UI screenshots (`A1 S1`, `A1 S2`, `A1 S3`). It does **not** evaluate color, workflow, or overall product strategy.

---

## 1. Structural Model

The UI uses a **stacked strip model**:

1. **Global top information bar**
   - Contains screen title, firmware/show/user/world/zone/sound output metadata, clock, and status indicator.
   - Runs full width.
   - Behaves as a persistent, non-scrolling header.

2. **Local tab bars per view/panel**
   - Each major region has its own tab strip.
   - Examples:
     - `Positions / Shapers`
     - `Colors / Beams / Gobo / Focus / Control / Shapers`
     - `Groups / Layout`
     - `FixtureType Sheet / Channel / Fixture / DMX / Sequence Exec`
     - `Timeline Overview`, `Timeline Transport`, `Timeline Editor`
     - `Effects / Zones / Worlds / Pawterns / Users / Backup / Patch`
   - This means resizing is **panel-local**, not driven by one giant app-wide tab bar.

3. **Micro-control row under tabs**
   - Small toggles such as `Preset Sort`, `Icons`, `Fixture Sort`, `Mask`, `Sort by:`.
   - These behave like a secondary utility row and should remain visually separate from primary tabs.

This layered structure is good because it allows the UI to scale by **region**, not only by screen size.

---

## 2. Tab Sizing Behavior Observed

### Fixed-height strips
The tab bars appear to use a **consistent, fixed height** across panels. This is important because it gives the whole interface a machine-like rhythm and avoids vertical instability when panels resize.

### Width appears content-driven, but capped by available panel width
Each tab looks sized according to its label length, with some shared padding. When many tabs exist in a narrow panel, the system compresses them tightly rather than expanding their touch targets.

### Add-tab button as terminal fixed item
A `+` button appears at the end of several tab rows. This suggests the layout treats tabs as a flowing list with a fixed utility action anchored at the far end.

### No evidence of responsive priority rules
In narrow contexts, the current system does **not** appear to:
- collapse lower-priority tabs into overflow,
- truncate labels elegantly,
- switch to icon+tooltip mode,
- or promote the active tab while demoting inactive ones.

Instead, it simply keeps shrinking the available text space.

---

## 3. Strengths of the Current Resizing Approach

### A. Predictable strip geometry
The user can quickly scan where tabs live because every panel uses the same general grammar:
- title/tabs on top,
- controls below,
- content below that.

### B. Good containment
Because each module owns its own tabs, resizing one area does not destroy the navigation model of the whole screen.

### C. Dense information efficiency
For expert users on large screens, many tabs can remain visible at once without opening extra menus.

### D. Works well with tiled console layouts
The system is compatible with lighting-console thinking: multiple tool windows, each with a compact local navigation strip.

---

## 4. Weaknesses in the Current Resizing System

### A. Tabs become too small too quickly
When panel width decreases, the tab labels compress aggressively. This creates three problems:
- reduced readability,
- reduced click confidence,
- weak active/inactive differentiation.

### B. No visible overflow strategy
The screenshots suggest the system assumes there will always be enough width, but panels like `Colors / Beams / Gobo / Focus / Control / Shapers` already show the pressure point. In a browser-based UI, this will break faster than on a fixed hardware console display.

### C. Information bar is width-hungry
The top navbar contains many metadata items in one horizontal line:
- firmware,
- showfile,
- revision,
- user,
- world,
- zone,
- audio output,
- date/time,
- status.

This is fine at large sizes, but it has no visible responsive degradation path. On smaller widths it will either wrap badly, truncate unpredictably, or force the title area to shrink.

### D. Tab role and utility role compete for the same strip
Some rows mix:
- navigation tabs,
- sort toggles,
- add buttons,
- dropdown-like options.

When resized, this makes it harder to know what should remain visible first.

---

## 5. Navbar Resizing Analysis

## Global top bar behavior
The top bar is essentially a **status ribbon** plus screen identity. It is horizontally elegant on wide screens, but fragile on narrower layouts.

### Current characteristics
- left: screen title / main context
- center: show and environment metadata
- right: clock and status

### Resizing problem
All items appear to share the same horizontal importance, but they should not. In practice:
- the current screen name,
- showfile,
- and critical status
should survive longest.

Low-priority metadata should degrade first.

### What the current design lacks
- priority-based hiding,
- compact grouping,
- breadcrumb compression,
- adaptive abbreviation.

---

## 6. Panel Tab Resizing Patterns by Screenshot

## A1 S1
This screen shows multiple concurrent tab regions. The important lesson is that **many short tab strips can coexist**, but only if each strip has its own overflow logic. The current version is visually stable, but already near its density limit.

## A1 S2
This timeline screen is the clearest stress test for navbar scaling:
- global top bar,
- timeline section headers,
- transport controls,
- editor row,
- attributes row.

Because the content area needs width for time-based data, navigation chrome must become more disciplined. This is the strongest argument for a responsive tab priority system.

## A1 S3
This screen works better because it has fewer competing tab groups and stronger content focus. It suggests the system is most successful when:
- one main work area is dominant,
- tabs are few,
- and utility controls are not mixed into the same line.

---

## 7. Recommended Resizing Rules

## Rule 1 — Fixed height, variable width, minimum hit size
Keep tab height fixed, but never allow width below a usable minimum.

Recommended minimums:
- mouse-first UI: **72–88 px** per tab
- touch-capable UI: **96–120 px** per tab

If width drops below minimum, move to overflow instead of shrinking further.

## Rule 2 — Priority-based overflow
Every tab strip should support priorities:
- **Priority 1:** active tab, most-used tabs
- **Priority 2:** common secondary tabs
- **Priority 3:** rare/admin tabs

On resize:
1. preserve active tab,
2. preserve pinned tabs,
3. move low-priority tabs into `More`.

## Rule 3 — Separate navigation from utility controls
Do not let sort toggles and mode checkboxes compete with navigation tabs in the same compression model.

Use:
- row 1 = tabs
- row 2 = utility toggles / sort / filters

or
- left = tabs
- right = utility cluster

with separate overflow logic.

## Rule 4 — Navbar priority collapse
For the global top bar, collapse in this order:
1. keep screen title,
2. keep showfile,
3. keep status + clock,
4. group user/world/zone into one compact environment chip,
5. hide firmware/revision/audio route first.

Example compact state:

`ShowInfo | BigassShow | Admin / World Full / Zone Full | 00:00 | ●`

## Rule 5 — Truncate with intent
Do not compress text endlessly. Use:
- full label at large widths,
- abbreviated label at medium widths,
- icon or overflow at small widths.

Example:
- `FixtureType Sheet`
- `Fixture Sheet`
- `Fixture`
- icon + tooltip / overflow

## Rule 6 — Active tab should grow, inactive tabs may shrink
A strong responsive pattern for console UIs is:
- active tab gets more width,
- inactive tabs get compact width,
- hidden tabs move into overflow.

This maintains context even in narrow panel sizes.

---

## 8. Best Responsive Model for a React/Web Version

For a browser-based implementation, the best model is:

### Global navbar
A **priority-aware flex row** with collapsing metadata groups.

### Panel tabs
A **scroll-free tab bar** with:
- minimum tab width,
- overflow menu,
- pinned active tab,
- optional abbreviation map.

### Utility controls
A separate responsive cluster that can wrap, collapse to icons, or move into a panel menu.

This is better than horizontal scrolling tabs because lighting workflows depend on immediate legibility and fast pointer landing.

---

## 9. React Implementation Guidance

## Recommended tab model
Each panel tab system should support metadata like:

- `id`
- `label`
- `shortLabel`
- `icon`
- `priority`
- `pinned`
- `active`

Then the resize algorithm can decide:
- show full label,
- show short label,
- show icon,
- or move to overflow.

## Recommended navbar model
The top bar should be broken into chunks:
- `screen`
- `show`
- `systemMeta`
- `environment`
- `clock`
- `status`

Each chunk gets a visibility priority and compact variant.

---

## 10. Final Verdict

The tab/navbar system in these screenshots is **structurally strong but not yet truly responsive**.

Its main strengths are:
- clear modular panel ownership,
- consistent strip hierarchy,
- strong console-like density.

Its main weakness is that resizing currently seems to rely on **shrinking everything horizontally** instead of applying **priority, overflow, and compact states**.

For a modern React/browser console, the correct evolution is:
- keep the same multi-panel tab grammar,
- preserve fixed strip heights,
- add minimum tab widths,
- introduce overflow menus,
- and make the top metadata bar collapse by importance.

That would preserve the familiar console feel while making the system robust on different browser sizes, docked layouts, and multi-monitor workflows.
