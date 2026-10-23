# 12d research: rights levels (one level per user)

Date: 2026-10-13. Read only: no repo file was changed, and no server was started.

User request: *"we need setup group, and group assignments for users, i think. admins have full acess. i think a single rights level is clean and would fit this system as well, we not want to assign users to multiple groups, just a level. guest, playback, admin etc.. what levels are there we need?"*

---

## 1. What the docs say (old and new)

### 1.1 Decisions (docs/archive/decisions-2026-10.md)

| Where | Quote (short) | Note |
|---|---|---|
| §4 :232 | "Admins can unclaim anyone" | claims, original 2026-03 design |
| §4 :257 | "`Clear Programmer User 2 Thru 5` (Admin + Setup; the user is notified and can undo)" | **Setup was already named as a level** in step 4 hardening, but never built |
| §10 :441 | "clears other users' programmers (Admin and Setup rights)" | same |
| §10 :498 | merge default + timeout "need Admin rights" | |
| §11 :523-524 | "An Input line (e.g. OSC) names a **user**; commands … run … with that user's rights" | |
| §11 :589 | "**Live Patch**: without the Patch claim (Setup/Admin rights) `Set` may change Name, Note…" | the second time Setup shows up: the full patch = Setup/Admin |
| §19 :863 | "Admin stays passwordless for now" | superseded by §30 12a3 |
| §19 :870 | security is about "**who did what and with which rights**, not about keeping attackers out" | |
| §19 :877-879 | "**One rights gate**: the `Rights` check (Guest / Programmer / Admin / Internal) runs in dispatch for every transport … OSC runs as its input line's `User` (Admin when none is set)" | |
| §19 :891-893 | "Session and look-around words a Guest needs are Guest: Login, Logout, Exit, … ListClaimed, Unwatch" | |
| §19 :894 | "OSC lines with no User run as Admin" | |
| §21 :945 | "'a show always has an Admin' stays as a rule" | |
| §21 :1079 | "taking a claimed object asks its owner … an Admin can force" | |
| §21 :1120 | "an Admin on any member can use [`Claim LumaSession`] any time" | |
| §21 :1127 | "`Delete Station "FOH-2"` removes one (master, Admin…)" | |
| §21 :1229 | merge default/timeout "Admin to change" | |
| §21 :1246 | "An Admin's `Claim … /Force` skips the prompt (this is §4's 'Admins can unclaim anyone')" | |
| §21 :1287 | "`Store Input … /User X` refuses an unknown user (a missing user never means Admin)" | |
| §25 :2371-2373 | sweep: "**Every role:** each line as Admin, Programmer and Guest (+ a second user holding claims)" | |
| §25 :2418 | finding 4: "A Guest line refused … in its second statement still changed the patch via the first" | fixed in 11b: all or nothing (:2660) |
| §30 :3075-3076 | descriptors carry "rights (disabled, not hidden, for a Guest)" | UI rule for missing rights |
| §30 :3218 | lock screen: "the show's other users as round avatars … rights badge" | |
| §30 :3339-3341 | "The User keeps identity (name, password, **rights**, …). **Rights**: users of that profile and Admin may change it" | |
| §30 :3360-3363 | "**admin:admin always works**, this is because if the LD/operator leaves, anyone needs to be able to login that is a poweruser" | |
| §30 :3368 | "no automatic Guest" | |
| §30 :3440-3441 | "Admin user always has admin as password and cannot be changed. It always asks for it." | |
| §30 :3519 | "Guest is an ordinary user when the show has one" | |
| §30 :3544 | `Set User … Color` "(own user or Admin)" | |
| §30 :3900-3901 | "Admin-role users get no history items (the TUI never recorded Admin history — unchanged)" | |
| §30 12c2 :4108-4109 | "a deleted user keeps Admin rights through the same-named profile's role (`role_for_user` fallback)" | the bug that started "rights only from the User" |
| §30 12c2 :4115-4116 | "New user = one station ask … Role (Admin / Programmer / Guest, default Programmer)" | |
| §30 12c2 :4126-4127 | Delete user "Refused, coded: yourself, the last Admin, Admin itself" | |
| §30 12c2 :4130-4131 | "**Rights only from the User**: the role on the profile and the name fallback go … Admin-only refusals answer E-RIGHTS-001" | |
| §30 12c2 :4184-4185 | "Rights come only from the User (`ObjectDef.rights`)" | |

### 1.2 Roadmap, handoffs, user and dev docs

| Where | Quote | Note |
|---|---|---|
| roadmap.md:50 | "One rights gate, Remote Login holds, passwords masked, logout revokes" | step 5 |
| roadmap.md:108 | "Rights all or nothing" | |
| roadmap.md:397 | "[ ] LumaSession password (join only with it)" | open |
| roadmap.md:412 | "[ ] Invite runs the join as the station's first logged-in user (else Admin) — pick the user properly" | open; touches rights |
| docs/archive/docs/plans/done/2026-10-10-decisions-qa.md:28 | "osc input needs the user so it runs on that user" | |
| decisions-qa.md:81 | Default Admin: "if we need external access we also need admin access and we will discuss this later" | |
| decisions-qa.md:120 | "a huge problem in other consoles is that i cannot clear someone elses programmer" → `Clear Programmer User 2 Thru 5`, **Admin + Setup** | the user's own words that started Setup |
| decisions-qa.md:316 | "web remote (must be enabled per station)" | phone/remote |
| decisions-qa.md:690 | "admin always have admin:admin so powerusers always can login. Isolated networks ok." | |
| decisions-qa.md:705 | profile rights: "Users of that profile + Admin" | |
| decisions-qa.md:849-851 | Delete user: "refuse self / last Admin / Admin" · Rights: "Only from the User" | |
| docs/plans/done/2026-10-10-thin-client.md:17 | a user record = name, password, rights, profile, screen config, web remote login | |
| thin-client.md:31 | "`LogIn ["user"] ["password"]` swaps user on the station; `LogOut` → Guest" | LumaCore chose "no automatic Guest" |
| thin-client.md:56 | "Split: `ShowData.users` {name, password hash, rights, remote login allowed, profile, …}" | |
| thin-client.md:104 | user commands "Admin only; last Admin protected" | |
| thin-client.md:87 | "Phones: a 'Phone' screen config is just a config with one small screen" | |
| docs/plans/done/2026-10-10-ui-keys.md:39 | "CleanStart rights: `CleanStart` command: Admin. **Home screen option**: anyone, **local server only**" | |
| docs/research/ui-kit/online-touch.md:167 | plugins "get the kit's rendering, gestures, rights and undo for free" | only plugin-rights mention |
| docs/user/terminal/commands.md:106-110 | "**Guest** — look only … **Programmer** — work on the show: select, set values, store, patch, play back · **Admin** — users, CleanStart, forcing claims, clearing other users' programmers" | today's user-facing ladder |
| commands.md:115-118 | "Rights come only from the **user** … Users and profiles are Admin's … (you may still set your own password and colour)" | |
| docs/user/object-tree/control.md:78 | "clears other users' programmers (Admin and Setup rights)" | stale vs code (Admin only) |
| docs/user/terminal/tree-shell.md:11 | "Every tree-shell word is read-only … needs only Guest rights" | |
| docs/dev/integrations/integrations.md:36,42 | `Store Input 2 "Companion" … /User Admin`; "Commands run as the line's user, through the same … rights gate" | |
| docs/dev/testing.md:205 | sweep "every case as Admin, LD (Programmer) and Guest" | |
| docs/dev/networking/lumasession.md:56,252 | Delete Station "(master, Admin)"; Claim LumaSession "(Admin, or a priority above the master's)" | |
| visions/notes/client-types.md:21-26 | "Playback wing: Cue playback, DMX output"; "Client declares its 'role' or 'profile' at connection time" | about data loading, not rights; roadmap :505 |
| deleted `docs/meta/phases/01-foundation/01-context.md:87-88` (git `8961464^`) | "Object locking/permissions: design for it, implement later"; "User roles: design for it, implement later" | 2026-03 |
| same file :284 | "**Tree access control per user profile** — visibility/edit permissions on tree branches per user (LD sees all, spot op sees only followspots)" | the only older idea of finer rights; not taken since |

### 1.3 Ideas that were floated

- **Setup** level: decisions §4/§10/§11 and decisions-qa :120 already say "Admin + Setup" for clearing others' programmers and "Setup/Admin" for the full patch. Never built.
- **Playback-only user**: `users.rs:43` already parses `/Role playback` as **Guest** (a placeholder). client-types.md lists a "Playback wing". A reference console's default guest user has playback rights.
- **Phone / remote user**: a reference console ships a remote user at a presets-level right; the web remote must be enabled per station (decisions-qa :316). LumaCore has a per-user `RemoteLogin` flag and decided "Guest is an ordinary user when the show has one".
- **A view-only user**: today's Guest ("look only").
- **Per-branch visibility** ("spot op sees only followspots", 2026-03): not taken by any later decision.
- **CleanStart from the home screen for anyone, local server only** (ui-keys :39): a recovery exception to rights.
- **LumaSession password** (roadmap :397): open. It's about stations joining, not user levels.

---

## 2. The code today

### 2.1 Mechanism

- `lumacore_tree::UserRole { Admin, #[default] Programmer, Guest }` (crates/show/lumacore-tree/src/programmer.rs:170), stored on `User.role` (:613). `ShowData::role_for_user(name)` (tree.rs:527) reads only the User (12c2).
- `registry::Rights { Guest, Programmer, Admin, Internal }` (core/commands/registry.rs:207). `CommandDef.rights: Rights`. `Word::rights()` (:3438) gives **`Rights::Programmer` for every bare object word** (`Fixture 3`, `DeskLock`, `Sequence 1`).
- `ObjectDef.rights: fn(Verb, &[&str]) -> Rights` (objects/mod.rs:191): extra rights per verb. 74 of the 76 objects use `no_extra_rights` (= Guest, i.e. only the word's own right counts). The only two that do more:
  - `User` (`users.rs:885 user_rights`): Store / Delete → Admin.
  - `UserProfile` (`profiles.rs:309 rights`): Store / Copy / Delete / Label / Import → Admin, except when the line names its Image / Icon / WindowPreset pool.
- The gate: `core/rights.rs` `check_statement` → `authorize(role, word)` (rank Guest 0 < Programmer 1 < Admin 2), then the object's `rights(verb, args)`. An unknown user counts as Guest. `INTERNAL_ROOT_USER` may do anything. The refusal is E-RIGHTS-001. Dispatch checks every statement before any runs.
- Property rights: `props::Prop.rights` + `props::allowed` (props.rs:198) map the role onto `Rights`.

### 2.2 Counts (generated reference, docs/user/reference/commands.md, 104 commands)

| Rights | Commands | Which |
|---|---|---|
| Guest | 34 | Dump, EditSetting, Help, Info, List, Shortcuts, Exit, banner, Clear, Disconnect, Echo, Env, History, Login, Logout, Menu, quicksheet, ReloadUI, Unwatch, WhoAmI, Cat, cd, Diff, Du, Find, Grep, Head, ls, pwd, root, Stat, Tail, Tree, Wc |
| Programmer | 68 | everything else (Store/Copy/Delete/Move/Label/Edit/Update/add/remove/rm/create, At/Full/Zero/Select/…, Go/Go-/Goto/Pause/Stop, Set, Patch/Unpatch/Assign/Unassign, Park/Unpark, DMX, Claim/Unclaim, Bind/Unbind, Join/Leave/Invite/Dismiss, LoadShow/NewShow/SaveShow/DeleteShow, Import/Export, Screenshot, Shutdown/Restart/LumaCore-Shutoff, CleanStart, Oops/Redo, Connect, Enter, On/Off/Toggle) |
| Admin | 1 | debug |
| Internal | 1 | HissMe |

Objects: 76 in `objects::OBJECTS`. A bare call needs Programmer for all of them. Extra Admin verbs exist only on User and UserProfile (above).

Writable properties (objects.md, `TreeDef::write`): 41 Guest (all the UserProfile look, lock screen and preferences fields), 36 Programmer (Fixture, Universe, Network, Output, Input, Image Name…), 2 Admin (`Input.User`, `UserProfile.Name`).

### 2.3 Admin-only today (word, object verb or handler check)

| What | Where it's checked |
|---|---|
| `Debug` | word rights Admin + `run.rs:249` |
| `CleanStart` | **handler only** (`backup.rs:451`); its word rights say Programmer |
| `Store User`, `Delete User` | `ObjectDef.rights` + `users.rs:31 require_admin` |
| `Set User … Name/Role/RemoteLogin/Profile/ScreenConfig`, another user's password or colour | `users.rs:421-425` (own password and colour are free) |
| `Store / Copy / Delete / Label / Import UserProfile` (not its pools) | `profiles.rs:309`; `media.rs:676, 696, 741` |
| Changing a profile you don't use | `media.rs:139 may_change_profile` (users of the profile + Admin) |
| `Clear Programmer User …` | `programmers.rs:174` (the docs say Admin + Setup; the code says Admin only) |
| `Claim … /Force`, `Unclaim` of another user's claim | `claims.rs:268`, `claims.rs:188` |
| `Claim LumaSession` (unless this station's priority outranks the master) | `session_link.rs:185` |
| `Delete Station` | `session_link.rs:218` |
| `Show /MergeDefault`, `/MergeTimeout` | `backup.rs:616` |
| `Input.User` property, `UserProfile.Name` | prop rights |
| Not a right but tied to Admin: the TUI doesn't record Admin's history | `tui_commands.rs:45,86` |
| Admin user: password always `admin` (E-USER-001), can't be renamed or deleted (E-USER-005), last Admin protected (E-USER-004) | `passwords.rs:197-211`, `users.rs:467-509, 608-619` |

So there are **about a dozen `is_admin()` checks inside handlers, outside the gate**. They all return E-RIGHTS-001, but help, completion and the sweep only see the word's declared right.

### 2.4 What a Guest can do today (and what it can't, though it probably should)

- Can: the 34 Guest words: look (List / Info / Dump / Help), the tree shell, Login / Logout / Exit / Disconnect, Menu, ReloadUI, Echo, Clear (terminal), Unwatch, EditSetting (it opens the settings).
- **Can't, probably by accident:**
  1. **`Set` is Programmer at the word level**, so a Guest can't set their **own password or colour**, or their profile's look / lock screen / preferences. The Guest prop rights (41 fields) and the "own password and colour" rule are never reached. `EditSetting` opens for a Guest, but every change it sends is a `Set` line, and that is refused.
  2. **`DeskLock` (F9 / Pause), `NormalTheme`, `Sidebars`, `ViewBar`, `FullScreen`** are object words, so they need Programmer (`Word::rights` → Programmer). A Guest can't lock their desk or go full screen. These are core keys that are meant to work for everyone. (Found by reading; not run.)
  3. `Oops` / `Redo` are Programmer, so a Guest can't undo their own UI steps.
  4. A bare view or window call (`View 3`) is Programmer.
- Sweep users: `sweep/world.rs:19 ROLES = ["Admin","LD","Guest"]`, created with `Store User … /Role …` (:134-136).

### 2.5 Other places that read the role

- `suggest/mod.rs:294`: suggestions mark a refused item ("no-rights").
- `help.rs:408`: "Needs: Guest/Programmer/Admin".
- `treeshell.rs:738`: WhoAmI.
- `/api/station` users[].role (`users.rs:918`, `server/state.rs:324`). Kit: `people.js:123`, `surface.js:369` (badge), avatar badge.
- `net/handler.rs:194`: the command port logs the role. The gate decides as usual.
- `users.rs:41-44 parse_role`: `admin`, `programmer|program`, `guest|playback`.
- `server/app.rs:202`: a new user defaults to Programmer. `UserRole` `#[default] Programmer`.
- `errors.rs:157`: E-RIGHTS-001's explanation names "Guest, Programmer, Admin".

---

## 4. Proposal: one ladder for LumaCore

### 4.1 The levels

`Guest < Playback < Presets < Program < Setup < Admin` (+ `Internal`, never a user's)

| Level | Meaning in LumaCore |
|---|---|
| **Guest** | Look, log in, and use their own device: List / Info / Help / tree shell, Menu, DeskLock / NormalTheme / Sidebars / ViewBar / FullScreen, call views and screens, their own password / colour / profile look. No programmer. |
| **Playback** | Run the show: Go / Go- / Goto / Pause / Stop / Off Sequence, executors, masters (grand, group, Highlight / Solo masters), call macros (each line gated as the user), select datapool / sequence, **and use the programmer** (Select, At, Full, Blind / Highlight / Freeze / Solo, Reset, Clear own). Arrange their own windows and views. Oops / Redo their own steps. Screenshot. No storing. |
| **Presets** (optional, see Q2) | Playback plus **Update** of existing presets, `Store … /Merge` on an existing preset, and plain `SaveShow` (same file). No new objects. |
| **Program** | Build the show: Store / Copy / Move / Delete / Label / Edit of show content (groups, presets, sequences, cues, macros, effects, timecodes, executors, images, views, window presets, UI keys), **Live Patch** props (`Set Fixture` Name / Note / Locked / Hidden / position / offsets), Park / Unpark, the `DMX` tester, Claim / Unclaim own, Bind / Unbind, Import / Export objects, `SaveShow "new name"`, DeleteShow. |
| **Setup** | The rig, the station and the session: **Claim Patch** (full patch: Patch / Unpatch / create / delete fixtures, universes, fixture types, `Move Fixture`), **Networks, Output / Input lines, routes** (Assign / Unassign Universe), merge modes, **station settings** (`Set Station …`, `Set Interface … LumaSession|Shell|HTTP`, Online, NIC ↔ network mapping, AssetRoot), **LumaSession** Join / Leave / Invite / Dismiss / Pause / `Claim LumaSession` / Delete Station, `Show /MergeDefault|/MergeTimeout|/Autosave`, LoadShow, Shutdown / Restart / Shutoff, `Clear Programmer User …`. |
| **Admin** | Everything, plus: users and profiles (Store / Delete / Set User Rights / Name / Profile / RemoteLogin, other users' passwords; Store / Copy / Delete / Label / Import UserProfile), `Input.User` (it hands a line someone's rights), `Claim … /Force` and unclaiming others, NewShow, CleanStart, Debug. |

Naming: **"Program", not "Programmer"**. `Programmer` is a registered object and `View` is one too, and §15 says one namespace. `presets` is an alias of `Preset`, so the level value would need `// tidy: not-keywords`. "Guest" is the user's word and isn't registered. The property is called **`Rights`** (`Set User "Bob" Rights Playback`, `Store User … /Rights Setup`). `Role` goes, forward only: show format +1, no migration.

### 4.2 Every current rights value mapped

**Guest words (34)** all stay Guest. Add to Guest: `Set` (the word; the target decides), `On / Off / Toggle` (the word; the object decides, so DeskLock works), `Oops / Redo` (only your own steps), `Enter`, `Connect`, and the bare calls of DeskLock / NormalTheme / Sidebars / ViewBar / FullScreen / View / ViewButton / Screen / Self / Device.

**Programmer words (68)**, split by what they act on. For a generic verb the word gets the lowest level, and `ObjectDef.rights(verb)` raises it.

| Current Programmer word | New level |
|---|---|
| Go, Go-, Goto, Pause, Stop | Playback |
| At, Attribute, Deselect, Full, Zero, Grid, Meow, Minus, Plus, Next, Previous, Reset, ResetActive, ResetAll, ResetSelection, Select, Flash (planned), If (planned) | Playback |
| On, Off, Toggle | Guest word; objects: modes / fixtures / groups → Playback, DeskLock etc. → Guest, `On DeskLock Station` → Setup |
| Store, Copy, Delete, Move, Label, Edit, Update, add, remove, rm, create | Word at Playback (own windows / views); object decides: show content → Program, `Update Preset` / `Store … /Merge` → Presets, patch objects + lines → Setup, User / UserProfile → Admin |
| Set | Guest word; prop rights decide (they exist already): profile look / prefs → Guest (users of that profile), fixture live props → Program, universe / line / network props → Setup, station / interface → Setup, User fields → Admin (own password / colour → Guest) |
| Assign, Unassign | Program word; universe routes → Setup |
| Patch, Unpatch, Fixture `Move` | Setup |
| Park, Unpark, DMX | Program |
| Claim, Unclaim | Program; `Claim Patch` → Setup; `Claim LumaSession` → Setup; `/Force` + others' claims → Admin |
| Bind, Unbind | Program (or the "users of the profile" rule) |
| Import, Export | Program; `Import UserProfile` → Admin |
| Screenshot | Playback |
| Join, Leave, Invite, Dismiss | Setup |
| LoadShow | **Setup** (hard case) |
| NewShow | **Admin** |
| SaveShow | Presets (plain), Program (new name / `/Overwrite`) |
| DeleteShow | Program |
| Shutdown, Restart, LumaCore-Shutoff | Setup (hard case) |
| CleanStart | **Admin at the word** (today: Programmer word + handler check) |
| Oops, Redo | Guest (own steps) |
| Connect | Guest |
| Enter | Guest |

**Admin (1)**: debug → Admin. **Internal (1)**: HissMe → Internal.

**Handler-side Admin checks (§2.3)**: they become declared rights (`ObjectDef.rights` per verb or flag, or prop rights), so help, completion and the sweep see them. `Clear Programmer User` → Setup (as decisions §4 / §10 already say). `Claim LumaSession` / `Delete Station` / merge settings → Setup. Users, profiles, `/Force` → Admin.

### 4.3 Rules to keep

- **Admin user**: always Admin, can't be changed (a locked default user), password always `admin`, can't be renamed or deleted. **Last-Admin rule** stays: lowering the last other Admin is E-USER-004. Since the Admin user can't be demoted the rule can't actually trip, but keep it as a backstop.
- **Rights only from the User** (12c2): the profile never carries a level. A missing or deleted user = Guest.
- **One gate in dispatch for every transport** (§19): level ranks replace `rank()`. All or nothing per line stays. Remove the handler `is_admin()` calls in favour of one `rights::at_least(state, Rights::X)` (or declared rights).
- **Disabled, not hidden** below the level (§30): descriptors, the pick list, suggestions ("no-rights") and Help "Needs:" show the level name.
- **Lock screen**: DeskLock / unlock work at every level. Unlock = the locking user's password, or admin:admin. The surface badge shows the level.
- **Remote / phone**: an ordinary user. `RemoteLogin` decides whether they may log in from elsewhere and the level decides what they may do. No separate cap needed.
- **Sweep**: `ROLES` becomes one user per level (Admin, Setup, Program, Presets, Playback, Guest). Cases go ~19.7k × 3 → × 6 (about 2 min). Checks are unchanged: a refusal is E-RIGHTS-001 and changes nothing; allowed lines behave as for Admin.
- **Machine lines** (agenda / timers later, OSC with no User): today OSC with no User runs as Admin (§19 :894). See Q5.
- **Plugins** (later): a plugin and its buttons declare a minimum level like `ObjectDef`. Their lines still pass the gate as the user who runs them.
- **New user default**: Program (today's Programmer).

### 4.4 Hard cases (flagged)

1. **LoadShow**: a reference console allows it at Playback. In a LumaSession, loading replaces everyone's show and DMX → Setup recommended. NewShow is Admin.
2. **DMX lines and networks**: a reference console puts them at Program. LumaCore's lines are live and need no claim (§11), and they decide what leaves the desk, so Setup is recommended. The station NIC mapping is station setup.
3. **Live patch vs full patch**: today the line is drawn by `Claim Patch`. The proposal: live props = Program, Claim Patch = Setup. It matches a common Patch / Edit split and decisions §11 :589 ("Patch claim (Setup/Admin rights)").
4. **Guest can't `Set` today**, so own password / colour / look is impossible. That's a bug in today's model. Fixing it needs `Set`'s word level to drop and the target's rights to decide.
5. **Core keys at every level**: DeskLock / FullScreen are object words → Programmer today. `Word::rights` for objects should read `ObjectDef.rights(Verb::Call)`, not a flat Programmer.
6. **Macros and UI keys** run as the pressing user, statement by statement (all or nothing). A Playback user's macro with a Store inside is refused whole. Option: a macro carries its own minimum level; see Q6.
7. **Oops after a demotion**: an Oops step filed when the user was Program, undone after they became Playback. Recommendation: Oops is always allowed for your own steps. They are your own history, and restore isn't a new command.
8. **Shutdown / Restart**: a Playback operator at the end of the night? Setup recommended, plus the local home-screen CleanStart exception already decided (ui-keys :39). Could add "local device may Shutdown".
9. **Clear Programmer User**: the docs say Admin + Setup and the code says Admin. Setup matches the user's own wish (decisions-qa :120).
10. **`Input.User` = Admin**: a Setup user can build an OSC line but can't make it run as someone. Keep, or allow Setup up to their own level (a line may name a user whose level ≤ mine).
11. **Granting levels**: Admin only (simple). Alternative: Setup may set levels up to Program. Not recommended; it gets complicated.
12. **"Admin gets no TUI history"** (`tui_commands.rs:45`): an odd leftover. Keep it or drop it, but it shouldn't be tied to the level ladder.
13. **Claim LumaSession by priority**: the station-priority bypass stays (a station right, not a user's).
14. **Presets level cost**: needs `Update` + `/Merge` gating on preset objects (planned verbs today: `Preset` Store / Call / Info are partly planned). If Presets is skipped, imported shows map Presets → Program on import (no import exists anyway).

### 4.5 Implementation sketch (for when it's a go)

- `lumacore_tree::UserRights` (Guest, Playback, Presets, Program, Setup, Admin) replaces `UserRole`. `User.rights`; show format +1; demo regenerated; `CLIENT_PROTOCOL` / `SESSION_PROTOCOL` bump (the station view and snapshots carry it).
- `registry::Rights` = the same ladder + Internal (one enum, `Ord`). `rank()` goes.
- `Word::Object` rights = `(obj.rights)(Verb::Call, args)`; objects declare per-verb levels (a small table per object kind: Content / Patch / Lines / UI / Device / Station / Users).
- `Set` / `On` / `Off` / `Toggle` / `Store`-family words at their lowest level; the object or prop declares the rest (prop rights already exist).
- Handler `is_admin()` calls → declared rights or `rights::at_least`.
- `parse_rights` accepts the six names (and `View` → Guest?, `Program`); E-RIGHTS-001's text names the level; Help "Needs: Setup".
- Sweep ROLES ×6; tests per level for the hard cases; generated reference updated.

---

## 5. Decision questions

1. **Ladder**: (a) six, Guest < Playback < Presets < Program < Setup < Admin: parity with established consoles, and Remote = Presets has a home *(recommended)*; (b) five, without Presets: simpler, and Presets needs Update / Merge gating that isn't built; (c) four, Guest / Playback / Program / Admin with setup inside Admin: misses the user's "setup group".
2. **Names**: (a) Guest, Playback, Presets, Program, Setup, Admin; property `Rights`: avoids the `Programmer` / `View` object words *(recommended)*; (b) View … Admin: `View` clashes with the View object; (c) keep "Programmer" / "Role": less churn, but it clashes with the namespace.
3. **Playback and the programmer**: (a) Playback may select and set values, never store *(recommended: busking / follow-spot operators need it)*; (b) Playback = executors and masters only.
4. **Setup scope**: (a) full patch (Claim Patch), networks / lines / routes, station settings and interfaces, LumaSession join / leave / invite / dismiss / claim, LoadShow, Shutdown / Restart, merge / autosave settings, Clear Programmer User *(recommended: "the rig, the station, the session")*; (b) the same but LoadShow and DMX lines at Program; (c) station and session only, the patch stays Program.
5. **Lines with no user (OSC, later agenda / timers)**: (a) a line must name a user, or it runs at Playback *(recommended: no hidden Admin)*; (b) Admin, as today; (c) refuse to start without a user.
6. **Macros / keys running higher-level lines**: (a) as the pressing user, all or nothing (today) *(recommended: one gate, no escalation)*; (b) a macro may carry a minimum level to grey it out early; (c) a macro runs at its storer's level: escalation, not recommended.
7. **Who grants levels**: (a) Admin only *(recommended: simple, matches 12c2)*; (b) Setup may grant up to Program.

