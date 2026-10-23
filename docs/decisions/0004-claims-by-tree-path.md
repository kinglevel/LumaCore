---
id: 0004
title: Claims by GUID: editors claim, cd never, owner asked
area: claims
status: built
date: 2026-10-11
built_in: step 6d
supersedes: [archive §4 L250–253 "bare keyword quick claim", archive §4 L245–249 "claim Patch by word", archive §21 L1670–1671 "ListClaimed replaced", archive §21 L1665–1668 "Allow Deny buttons"]
superseded_by: —
confirmed_by: [E-CLAIM-001]
source: [archive §4 L243–260, archive §10 L476–481, archive §21 L1497–1500, archive §21 L1658–1672, archive §21 L1712–1720]
---
# Claims by GUID: editors claim, cd never, owner asked

## Context
The March design locked tree areas by word (`claim Patch`, `claim ShowData`), auto-claimed on `cd` and made a bare keyword a "quick claim". The LumaSession work (§10, 2026-10-10) tied concurrent editing to claims with an ownership prompt; step 6d (2026-10-11) built it on GUIDs so a move keeps a claim, and dropped every auto-claim on navigation.

## Decision
Claims are keyed by GUID and live in `Runtime.claims` (replicated, never saved). Opening an editor claims — `Edit`, `Enter`, UI editor windows — directly and indirectly (editing a macro line claims the macro, a cue part its sequence); leaving releases. Plain commands (Store, Set, Label) never claim; changing another user's claimed object, or anything inside it, is refused with E-CLAIM-001. `cd` never claims. An area is claimed by its tree path, never by a word: `Claim Root.ShowData.Devices` (the staged patch, see its record). Taking a claimed object asks the owner — Hand over / Keep — with a 10 s countdown; no answer, or an owner logged out or whose station is gone, hands over at once; `Claim … /Force` is Admin's and skips the ask (this is "Admins can unclaim anyone"). `List Claims` lists them (no `ListClaimed` alias); `cd Claims` then `List` is the same place. `Claim Fixture <n>` is the one fixture lock (see programmers-ltp-fixture-claims).

## Consequences
A new editor goes through the claim path on enter and release on exit; never add a word-named area claim, an auto-claim on navigation, or a second ownership prompt. Claims replicate with the session runtime, so a takeover master keeps them. The ask is a server-held dialog like every question (see the dialogs record). The "Remember for this session" button was added in §30's 12b hardening.
