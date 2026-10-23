---
id: 0087
title: The Drive object and /Drive on every file word
area: drives
status: built
date: 2026-10-19
built_in: 38a, 38b-4
supersedes: []
superseded_by: —
confirmed_by: [lum.md "### Drives", AppState::library_folder, tests/catalog/testdata/lum_words.golden.txt, E-CMD-003 (Eject planned)]
source: [archive §38 L11356–11384, archive §38 L11429–11451, archive §38 L11472–11490]
---
# The Drive object and /Drive on every file word

## Context
A drive needs two ways onto it, its root and its library, and a line needs to say which library. A lum.md discussion settled the form; the drive inside the quoted name (`"usb1:rig"`) and file words taking tree paths were not chosen (only the fence enters `Root.Drives`). The id is quoted: a bare word is a vocabulary word.

## Decision
`/Drive "<id>"` is one option declared once (`F::DRIVE`, like `/Version`); every file word takes it; absent means the local library. The file name stays relative to the word's library folder (`drive::library_file`: sub-folders allowed, no absolute path or `..`): `LoadShow "rig" /Drive "usb1"`, `Import Mvr "Rig 2026 v3.mvr" /Drive "usb1"`. One door resolves it, `AppState::library_folder(drive, folder, write)`; show words go through `backups::DirGuard` (that drive's `shows/` with versions, recovery copies and trash beside it). In the tree shell `cd ~usb1` is that drive's library, `cd ~` the local one.

The `Drive` object (`lumacore-cmd-station` `drives.rs`): station-local keyed rows like `Interface`, properties `Label`, `Kind` (Set re-ids), `Path`, `Library` (Switch — Set Yes creates the tree, No is refused), `Free`, `Total`, `State` (In · Out), `Since`. `List Drive`, `Info Drive "usb1"`, `Store Drive "folder1" /Path "/srv/shows"`, `Delete Drive "usb1"` (forgets or removes; asks when an AssetRoot maps to it). `Menu "Drives"` is a `MenuDef` grid on these properties.

## Consequences
A new file word declares `F::DRIVE` and reads through `library_folder` — never its own path join. `Eject Drive "usb1"` is planned, not built (E-CMD-003). A file picked outside the library is copied into the right library folder first, asked once, so a line stays a library name.
