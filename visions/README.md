---
title: Visions
category: docs
---
# Visions

Ideas, designs and visual targets for what LumaCore should become. Nothing here is built,
and nothing here is a rule: a design that is decided has a record in
[docs/decisions/](../docs/decisions/) (its history in
[docs/archive/decisions-2026-10.md](../docs/archive/decisions-2026-10.md)) and an item in
[docs/meta/roadmap.md](../docs/meta/roadmap.md), which links the vision it builds. The user
manual, [docs/user/](../docs/user/README.md), describes only what the station does today.

## Every file here

### Designs for features the station doesn't have yet

| File | What |
|---|---|
| [presets.md](presets.md) | The preset system as decided: pools and modes, Extract / Recast, recipe and embedded presets, relative and timing layers. Moved from the user manual in the md upgrade; its Built lines say what exists today. |
| [macros.md](macros.md) | The macro system as decided: macro lines, variables, the executor. Moved from the user manual; the macro pool and UI keys exist today, the executor doesn't. |
| [timelines.md](timelines.md) | Timecode timelines as decided: tracks, markers, transports, how a timeline drives sequences. Moved from the user manual; sequences and cues are built, the timeline isn't. |
| [effects.md](effects.md) | The effect generator as decided: lines, forms, calls, stomp, the deterministic anchors. Written from the frozen log's §28; supersedes `docs/archive/handoffs/effects.md`. |

### notes/ — design notes not yet planned

| File | What |
|---|---|
| [notes/README.md](notes/README.md) | The notes' own index: where each came from and what it says. |
| [notes/client-types.md](notes/client-types.md) | Client types and selective loading: not every client needs the whole show. |
| [notes/controllers-and-stations.md](notes/controllers-and-stations.md) | Stations with attached controllers (joysticks, MIDI, …) and a driver ecosystem. |
| [notes/effect-engines.md](notes/effect-engines.md) | Plugins, custom effect engines and a Commands Pool; the effects that will be built are [effects.md](effects.md). |
| [notes/global-users.md](notes/global-users.md) | Users and Teams known beyond one show, so people elsewhere can notify or chat with them. |
| [notes/terminal-menu-mode.md](notes/terminal-menu-mode.md) | A menu mode in the terminal beside the command prompt. |
| [notes/transport-types.md](notes/transport-types.md) | An address space beyond DMX universes for other transport protocols. |
| [notes/watch-quake-panel.md](notes/watch-quake-panel.md) | Watch output in a drop-down "Quake terminal" panel in the TUI. |

### metadata/ — imported reference material and analyses

| File | What |
|---|---|
| [metadata/LumaCore SB Concept.md](<metadata/LumaCore SB Concept.md>) | The UI style concept the Figma export was drawn from (the one copy; the export's own copy was removed as byte-identical). |
| [metadata/LumaCore SB+Lore Visionary.md](<metadata/LumaCore SB+Lore Visionary.md>) | "LumaCore Visionary UI": a design direction for a browser console, with the lore woven in. |
| [metadata/LumaCore SB tab bar analysis.md](<metadata/LumaCore SB tab bar analysis.md>) | An analysis of the tab system and navbar resizing in the concept screenshots (the one copy; the export's copy differed only in its title). |

### UI-UX/FIGMA/ — the Figma prototype

A React code bundle exported from Figma ("LumaCore Lighting Console"), kept as a visual
target. It is not the WebUI and shares no code with it; the kit the station ships is in
`crates/ui/lumacore-webui/web/kit/`. Paths below are under `UI-UX/FIGMA/LumaCore Lighting Console/`
unless named in full.

| File | What |
|---|---|
| `UI-UX/FIGMA/LumaCore Lighting Console.zip` | The export as downloaded, unchanged. |
| `README.md` | Figma's bundle readme: the design link and how to run it. |
| `CONSOLE_FEATURES.md` | The prototype's feature list, screen by screen. |
| `ATTRIBUTIONS.md` | Licences of the libraries the bundle uses. |
| `guidelines/Guidelines.md` | Figma's empty guidelines placeholder. |
| `index.html` | The bundle's HTML entry. |
| `package.json` | Its dependencies and scripts. |
| `postcss.config.mjs` | PostCSS for Tailwind. |
| `vite.config.ts` | The Vite build. |
| `src/main.tsx` | Mounts the app. |
| `src/app/App.tsx` | The app shell. |
| `src/app/routes.ts` | The routes to the pages. |
| `src/app/data/fixtures.ts` | Sample fixture data. |
| `src/app/types/lighting.ts` | Fixture, group, cue and preset types of the prototype. |
| `src/app/types/workspace.ts` | Pane, tab and window types of the prototype. |
| `src/app/pages/Root.tsx` | The page frame. |
| `src/app/pages/Home.tsx` | The home page. |
| `src/app/pages/Groups.tsx` | The groups page. |
| `src/app/pages/Cues.tsx` | The cues page. |
| `src/app/components/AttributeControls.tsx` | Encoders and attribute controls. |
| `src/app/components/CommandLine.tsx` | The command line bar. |
| `src/app/components/ConsoleTabs.tsx` | The console's tab strip. |
| `src/app/components/EffectsEngine.tsx` | An effects panel sketch. |
| `src/app/components/ExecutorBank.tsx` | Executor faders and buttons. |
| `src/app/components/FixtureCard.tsx` | One fixture as a card. |
| `src/app/components/GroupSelection.tsx` | The group pool and selection. |
| `src/app/components/PatchSpreadsheet.tsx` | The patch as a sheet. |
| `src/app/components/PresetPalettes.tsx` | Preset palettes. |
| `src/app/components/ShowInfoBar.tsx` | The show info bar. |
| `src/app/components/figma/ImageWithFallback.tsx` | Figma's image helper. |
| `src/app/components/workspace/Pane.tsx` | A workspace pane. |
| `src/app/components/workspace/TabBar.tsx` | The tab bar the analysis above describes. |
| `src/app/components/workspace/ViewRenderer.tsx` | Draws a view into a pane. |
| `src/app/components/workspace/ViewSelector.tsx` | Picks a view for a pane. |
| `src/app/components/workspace/WindowManager.tsx` | Splits and arranges panes. |
| `src/styles/fonts.css` | The fonts. |
| `src/styles/index.css` | The base styles. |
| `src/styles/tailwind.css` | Tailwind's entry. |
| `src/styles/theme.css` | The colour tokens. |
| `src/app/components/ui/accordion.tsx` | shadcn/ui component shipped with the export (the next 47 lines too). |
| `src/app/components/ui/alert-dialog.tsx` | shadcn/ui alert dialog. |
| `src/app/components/ui/alert.tsx` | shadcn/ui alert. |
| `src/app/components/ui/aspect-ratio.tsx` | shadcn/ui aspect ratio. |
| `src/app/components/ui/avatar.tsx` | shadcn/ui avatar. |
| `src/app/components/ui/badge.tsx` | shadcn/ui badge. |
| `src/app/components/ui/breadcrumb.tsx` | shadcn/ui breadcrumb. |
| `src/app/components/ui/button.tsx` | shadcn/ui button. |
| `src/app/components/ui/calendar.tsx` | shadcn/ui calendar. |
| `src/app/components/ui/card.tsx` | shadcn/ui card. |
| `src/app/components/ui/carousel.tsx` | shadcn/ui carousel. |
| `src/app/components/ui/chart.tsx` | shadcn/ui chart. |
| `src/app/components/ui/checkbox.tsx` | shadcn/ui checkbox. |
| `src/app/components/ui/collapsible.tsx` | shadcn/ui collapsible. |
| `src/app/components/ui/command.tsx` | shadcn/ui command palette. |
| `src/app/components/ui/context-menu.tsx` | shadcn/ui context menu. |
| `src/app/components/ui/dialog.tsx` | shadcn/ui dialog. |
| `src/app/components/ui/drawer.tsx` | shadcn/ui drawer. |
| `src/app/components/ui/dropdown-menu.tsx` | shadcn/ui dropdown menu. |
| `src/app/components/ui/form.tsx` | shadcn/ui form. |
| `src/app/components/ui/hover-card.tsx` | shadcn/ui hover card. |
| `src/app/components/ui/input-otp.tsx` | shadcn/ui one-time-code input. |
| `src/app/components/ui/input.tsx` | shadcn/ui input. |
| `src/app/components/ui/label.tsx` | shadcn/ui label. |
| `src/app/components/ui/menubar.tsx` | shadcn/ui menubar. |
| `src/app/components/ui/navigation-menu.tsx` | shadcn/ui navigation menu. |
| `src/app/components/ui/pagination.tsx` | shadcn/ui pagination. |
| `src/app/components/ui/popover.tsx` | shadcn/ui popover. |
| `src/app/components/ui/progress.tsx` | shadcn/ui progress bar. |
| `src/app/components/ui/radio-group.tsx` | shadcn/ui radio group. |
| `src/app/components/ui/resizable.tsx` | shadcn/ui resizable panels. |
| `src/app/components/ui/scroll-area.tsx` | shadcn/ui scroll area. |
| `src/app/components/ui/select.tsx` | shadcn/ui select. |
| `src/app/components/ui/separator.tsx` | shadcn/ui separator. |
| `src/app/components/ui/sheet.tsx` | shadcn/ui sheet. |
| `src/app/components/ui/sidebar.tsx` | shadcn/ui sidebar. |
| `src/app/components/ui/skeleton.tsx` | shadcn/ui skeleton. |
| `src/app/components/ui/slider.tsx` | shadcn/ui slider. |
| `src/app/components/ui/sonner.tsx` | shadcn/ui toasts. |
| `src/app/components/ui/switch.tsx` | shadcn/ui switch. |
| `src/app/components/ui/table.tsx` | shadcn/ui table. |
| `src/app/components/ui/tabs.tsx` | shadcn/ui tabs. |
| `src/app/components/ui/textarea.tsx` | shadcn/ui textarea. |
| `src/app/components/ui/toggle-group.tsx` | shadcn/ui toggle group. |
| `src/app/components/ui/toggle.tsx` | shadcn/ui toggle. |
| `src/app/components/ui/tooltip.tsx` | shadcn/ui tooltip. |
| `src/app/components/ui/use-mobile.ts` | shadcn/ui mobile breakpoint hook. |
| `src/app/components/ui/utils.ts` | shadcn/ui class-name helper. |

## Workflow

1. **Dump it** — drop ideas, sketches, screenshots or notes into the fitting folder; add a
   line to the index above.
2. **Discuss it** — when an idea gains traction, flesh it out here or as a mockup.
3. **Decide it** — when it is agreed, the rule becomes a record in
   [docs/decisions/](../docs/decisions/) and the work an item in
   [docs/meta/roadmap.md](../docs/meta/roadmap.md), which links the vision. The vision stays
   here until the feature is built and the user manual describes it.

## What does not belong here

- Rules — those are records in [docs/decisions/](../docs/decisions/).
- Tasks — those go in [docs/meta/roadmap.md](../docs/meta/roadmap.md).
- What the station does today — that is the user manual, [docs/user/](../docs/user/README.md),
  and the developer docs, [docs/dev/](../docs/dev/README.md).
