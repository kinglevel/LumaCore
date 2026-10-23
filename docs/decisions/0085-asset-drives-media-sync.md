---
id: 0085
title: Media in two tiers; asset drives; files travel by hash
area: drives
status: built
date: 2026-10-12
built_in: 12a2
supersedes: [archive §8 L380–389 "paths/references only", archive §8 L391–412 "auto-discovery sync GUI"]
superseded_by: —
confirmed_by: [E-MEDIA-001…004, media_tests, the link / stream file-exchange tests, e2e media.spec.ts]
source: [archive §8 L380–412, archive §11 L618–620, archive §30 L3770–3782, archive §30 L3834–3838, archive §30 L3853–3865, archive §30 L3869–3882]
---
# Media in two tiers; asset drives; files travel by hash

## Context
§8 kept all media outside the show as paths. 12a2 rewrote it: small files belong in the `.luma`; only large show-critical media stays on drives. §8's operator-approved sync remains the direction for that tier.

## Decision
Embedded tier: images, icons and GDTF files live inside the `.luma` beside their manifests (`image.json`); the per-file and per-show limits are constants in the luma crate, an import past them is refused with a code pointing to asset drives (E-MEDIA-001…004). The station keeps one copy per hash in `library/media/.store/<sha256>.<ext>` and `library/fixturetypes/<sha256>.gdtf`. `ShowData.images` is the show's pool; a profile carries its own images and icons.

Asset tier: the show names `AssetRoots` (`ShowData.assets {roots, files}`); each station maps a root to a place of its own (`Set AssetRoot "Video" Folder …`; since §38 a drive and a folder on it); station maps follow a rename by GUID (`AppState::follow_renames`, Q41). `Import Asset "Video" "clips/intro.mov"` hashes on the one background worker, never the state thread; the result is a follow-up `Import Asset … /Hash <sha> /Size <n>` line run as the user (Q42). Missing assets are listed (`List Asset [/Missing] [/Verify]`), never a crash.

Files travel by hash: a member asks `FileWant` for pictures and GDTF files its stores lack, the master answers `File` (checked by SHA-256, written on the stream thread); an upload on a member goes to the master first (`Upload` frame ahead of the forwarded Import line; master lost → the Import fails). Every station keeps everything embedded.

## Consequences
Not built: preload into memory, the operator-approved sync of §8, asset uploads over HTTP, taking asset-linked items offline — roadmap. Never a second file-transfer path; a frame change bumps `SESSION_PROTOCOL`.
