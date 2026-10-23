---
id: 0186
title: Every question the station asks is one server-held dialog
area: dialogs
status: built
date: 2026-10-13
built_in: 12b
supersedes: [archive §30 L3595–3598 "One standard dialog", archive §30 L4053–4058 "answer packet UiEvent"]
superseded_by: —
confirmed_by: [tidy frontend_asks_only_through_the_kit_dialog, E-CMD-008 (template fill), e2e kit-dialogs.spec.ts]
source: [archive §30 L3595–3598, archive §30 L3956–3958, archive §30 L4041–4064, archive §30 L4108–4110, archive §30 L4128–4160, archive §30 L4599–4600, archive §30 L5124–5129, archive §30 L5380–5383]
---
# Every question the station asks is one server-held dialog

## Context
Before 12b the page had `openAsk`, `openDecision`, `openClaimAsk`, `confirmAction` and `promptText`, and the station sent `UiEvent::Confirm`, `PasswordPrompt`, `StoreView` and `Status.claim_asks`. 12b replaced them with one message-box dialog the station keeps; 12c2 and 12d moved every page prompt and menu confirm onto it.

## Decision
Every question — line confirms, show load / save / delete, power, Unclaim, claim asks, the merge decision, the password prompt, a verb missing its value, Store over a used object, New user, Import — is an `Ask` in `core::dialogs`: title, meta, text, rows, details, typed inputs, toggles, selectors, buttons that are command-line templates (named answers, never 0-based), an optional countdown whose default the server presses (`take_due`). A command returns `DispatchResult::Ask` (a text handler sets `AppState::asking`). The board (`AppState::dialogs`, station-local) keeps each dialog with its audience: the device that ran the line, every device of the owning user for a claim ask, every client for the merge decision. Dialogs ride in each client's `Status.dialogs` / `dialogs_closed`, so reload and reconnect come free. The answer (`DialogAnswer`) closes the dialog on every client first, then the template is filled (text loses `"` and `;`, numbers only digits and operators, else E-CMD-008) and the line runs as the answering user through the ordinary change path. First answer wins. No invented timeouts: without a countdown it waits until answered or its client is gone; locked devices show "n waiting". Web draws the kit dialog, terminals a centred window. One-shot paths answer with `/NoConfirmation`. Delete asks once, in dispatch (see oops-categories-statements).

## Consequences
Never a client-only confirm, a browser dialog or a second ask path — tidy `frontend_asks_only_through_the_kit_dialog`. A new question is an `Ask` with templates; claim asks keep "Remember for this session" (`/Remember`).
