---
id: 0092
title: Import FixtureType from the library; revisions by hash
area: files
status: built
date: 2026-10-19
built_in: 34g4 round 3 part 2, 4c1
supersedes: [archive §34 L9712–9713 "old Import go"]
superseded_by: —
confirmed_by: [LinkOut::Upload, GET /api/fixture-types, the Import FixtureType reply]
source: [archive §34 L9709–9728, archive §34 L10648–10657, archive §34 L10560–10574, archive §34 L10379–10387, archive §34 L10416–10419]
---
# Import FixtureType from the library; revisions by hash

## Context
The bare `Import "x.gdtf"` and `Import library` went; the user asked that import "works with GDTF revisions and is easy", and that an upload on a member reaches every station, since all members must save the same `.luma`.

## Decision
`Import FixtureType "file.gdtf"` (the Devices claim; reach Drive; `/Drive "usb1"` as every file word) reads `library/gdtf/` through the disk door; `Store Fixture … /FixtureType "x.gdtf"` imports in the same step; `Import "x"`, `Import library` and `Import GDTF` are refused naming the kinds. A type holds its revisions (the same FixtureTypeID, `FixtureType.gdtf_fixture_type_id`): a re-import of the same FixtureTypeID adds a Revision, never a second type; revisions are told apart by the file's SHA-256 and revision text, never by name; `/Type` takes the newest, `/Revision n` picks one; a newer revision changes no fixture — the reply names how many stay and offers `Set Fixture … "Revision" n`; changing a revision needs the claimed patch.

One file index: `gdtf_library::library_in(dir)` (`lumacore_gdtf::file_details`; cached by path, size and time) feeds `/FixtureType <Tab>` (the show's types first, then the library's files), `GET /api/fixture-types[?drive=]` (`lumacore_views::insert`: the show's types with revisions, modes, footprints and use counts; the library's files with `in_show` by SHA-256, or the file's error), `GET /api/fixture-type?no|file&revision&mode` (one mode's channel map and details), the thumbnail (Details only) and `POST /api/gdtf?name=` (Upload from this device, Setup and up). A member's upload goes to the master over the LumaSession stream (`LinkOut::Upload`) before its Store / Import line runs there; the type then rides in the show with its file.

## Consequences
Never a second GDTF list or route; later file kinds join the one `files` feed (see one-files-element-on-one-feed). The derived model and the reader: see gdtf-native.
