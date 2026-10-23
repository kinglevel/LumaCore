---
id: 0090
title: Every command declares reach; the disk has one door
area: drives
status: built
date: 2026-10-12
built_in: Step 10, 11b
supersedes: []
superseded_by: —
confirmed_by: [tidy core_reaches_the_disk_only_through_drive, registry::line_reach, the sweep's report]
source: [archive §25 L2788–2798, archive §25 L2836–2841, archive §29 L3122–3130, archive §29 L3272–3276]
---
# Every command declares reach; the disk has one door

## Context
The command sweep had to run every word without touching the tree shell or the disk ("it should NOT touch the unix-like commands and drive access") and stay dynamic — generated from the registries, never a hand-written list of what to skip. 11b added the sandbox so the Drive words are swept too.

## Decision
Every `CommandDef` declares `reach: Show | TreeShell | Drive` and every `ObjectDef` a `reach: fn(Verb) -> Reach` (`show_reach` / `drive_reach`); the compiler forces it on a new command. `TreeShell` is the shell's walk (pwd, ls, cd, cat, find, grep, …); `Drive` is anything that reads or writes disk (Import, Export, SaveShow, LoadShow, Set Station, autosave …); `create` stays `Show` (an alias of Store). `registry::line_reach` also counts an object argument (`Watch Show`) and a path naming `Root.Drives`, which only tree-shell words and such paths enter.

`core::drive` wraps every file call of `core/`; a filename a line names goes through `drive::library_file`, a drive through `AppState::library_folder` (see drive-object-and-slash-drive-option). Tidy `core_reaches_the_disk_only_through_drive` fails on `std::fs` outside it; `// tidy: drive-ok (<reason>)` marks a non-command thread such as the sysmon log writer.

The sweep runs `Show` lines with the door shut per thread — what knocked is a finding — and `Drive` lines in its sandbox (`drive::set_sandbox`: a throwaway copy of `library/` as the working directory; only paths inside it pass, an absolute path or `../` is refused and reported); the tree shell stays out and is listed as skipped.

## Consequences
A handler that opens a file is `Drive` and calls `drive`, or tidy fails; a new command does not compile without its reach. The sandbox is the only place sweep or fuzz lines may write.
