---
id: 0161
title: One rights gate, a six-level ladder, Admin's fixed password
area: rights
status: built
date: 2026-10-13
built_in: step 5 (the gate); 12a3 (Admin's password); 12d part 2 (the ladder)
supersedes: [archive §19 L1270–1278 "token on every request", archive §19 L1288–1290 "Guest Programmer Admin", archive §19 L1305–1306 "OSC runs as Admin"]
superseded_by: —
confirmed_by: [E-RIGHTS-001, E-RIGHTS-002, E-RIGHTS-003, E-USER-001, E-USER-004, E-USER-005, `passwords::mask_secrets`, `rights_tests`, the sweep's one user per level]
source: [archive §19 L1270–1318, archive §23 L2220–2227, archive §29 L3105–3107, archive §29 L3259, archive §30 L3815–3817, archive §30 L3898–3900, archive §30 L4000–4001, archive §30 L4597–4598, archive §30 L4753–4768, archive §30 L4956–4998, archive §30 L5388–5390]
---
# One rights gate, a six-level ladder, Admin's fixed password

## Context
Rigs are isolated, open networks: security is about who did what with which rights, not keeping attackers out. The first design (a token on every request, a three-step ladder with a role on the profile, OSC running as Admin by default) went in step 5 and 12d part 2. The user asked that `admin:admin` always works so a power user can get in when the operator leaves.

## Decision
One rights level per user, the ladder Guest < Playback < Presets < Program < Setup < Admin (+ Internal for the station), on `User.rights` (`lumacore_tree::UserRights`), never on a profile. Only Admin grants a level; the user Admin is always Admin (E-USER-005), the last Admin stays (E-USER-004). Every statement passes the one gate in dispatch (`core/rights.rs`) on every transport, all statements of a line checked before any runs; a refusal is E-RIGHTS-001 naming the level needed and yours. What a line needs is declared: `CommandDef.rights` and `arg_rights`, `ObjectDef.rights(verb, args)` for a named object, the property's level for `Set`. Only ownership rules (another's claim, password, colour) use `rights::require`. An OSC line names its user or is refused (E-RIGHTS-003, see osc-line-user-required). Remote Login holds: `client_addr` is believed only from a loopback peer, in-session `Login` too (E-RIGHTS-002). Passwords are masked once in dispatch (`passwords::mask_secrets`) before journal, undo, replication and sysmon; `POST /api/logout` revokes the token (versioned, carries a nonce). The command port, HTTP reads and station-to-station trust stay open. Admin's password is always `admin` (`passwords::ADMIN_PASSWORD`); `Set User Admin Password` is E-USER-001; the surface always asks for it.

## Consequences
Never an `is_admin()` or per-transport check in a handler; a new word or object declares its level. The sweep runs one user per level from `UserRights::ALL`. A LumaSession password is roadmap.
