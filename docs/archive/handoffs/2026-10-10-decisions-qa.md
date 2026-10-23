# All questions answered — 2026-10-10 review session

Every question asked in the review → decision session, with the user's answer (quoted where
the user wrote their own words). The consolidated decisions are in `docs/meta/decisions.md`;
this file keeps the reasoning trail. Resume point: [`docs/meta/roadmap.md`](../docs/meta/roadmap.md) (the one plan since step 11a; the step 0–10 table is in `handoffs/done/2026-10-10-review-roadmap.md`).

## User corrections to the first review

- **sACN/Art-Net duplicate renderers (P1 #6):** "sacn settings and all other output protocols
  are stored in the showdata and starts the runtimes as needed, this is because some shows and
  stations does not need to start protocols if they are not needed." Confirmed in
  decisions.md §11 (Protocols in ShowData); the code started all engines at boot — a code gap,
  not a design mistake.
- **OSC (P1 #7):** a protocol that may be used per show (same lifecycle as outputs).
- **Blind hotkey (P1 #9):** "blind is to be implemented but not right now."
- **Registry (P3 #15):** "very important … it should not be possible to have rogue lists with
  keywords and commands, we need one struct and table that all other commands keywords and
  modules registers to."
- "Ask me more about all this to harden all decisions." / "Don't implement anything yet before
  I say so."

## Protocols, stations, inputs

| Question | Answer |
|---|---|
| What starts a protocol runtime? | Show content drives it: starts when the show has an enabled line of that protocol, stops when the last is removed / show unloaded; nothing at boot. |
| Which station sends DMX in a multi-station session? | "for now, like a reference console, only the master in a luma-session sends data … session = multiple servers joined together in a mesh … maybe user sessions needs to be renamed so this is not confusing?" |
| What may inputs (OSC) do? | "osc input needs the user so it runs on that user" — an Input line names a user; commands run with that user's rights. |
| Registry shape? | Two defs (CommandDef + ObjectDef), one registry with one namespace. "it makes sense to have command and object separate, i guess we already have it? do we need to change anything?" → yes: handlers on CommandDef, rogue lists removed. |

## Naming

| Question | Answer |
|---|---|
| "Session" means four things — how to name them? | Session = network mesh only (the reference console). |
| Attached-surface noun (after explanation of Client / ShellSession / UserSession / ViewSession) | ShellSession → **CommandLine**; UserSession → **Runtime.Users.{User}**; ViewSession → **Screen**; **Client** stays the attached device/window (earlier docs). |
| Distribution tool name (cat options) | **Pawtern** — "makes purrfect sense with the cat distribution system"; later: **Pawtern replaces Distribute** (§3). |
| Modifier key name | "Modifier needs to be called **Meow**." |

## Registry, words, clashes

| Question | Answer |
|---|---|
| Can a word be both command and object? | No — one namespace. |
| `u`: Update or Universe? | "it depends on KBD, NAV or PRG mode … in PRG we want update like a reference console" → `u` = Update (typed and PRG hotkey). |
| `type` | FixtureType (`cat` keeps `cat`). |
| `history` | Command history (undo list is Oops/Undo). |
| `shortcuts` | The KBD/PRG/NAV mode switch. |
| `watch` | "watch clock, list watch. we dont want too many keywords" → Watch is one callable object; no separate Watches keyword; principle: as few keywords as possible. |
| Abbreviations: registry aliases or separate TUI table? | Registry aliases only. |
| Planned commands vs "don't publish unimplemented"? | Allowed, clearly marked (`status: Planned`). |
| Blind now? | Register it as **planned** (step 1); later: keyword + state now, output suppression later. |
| May tests run during step 1? | Yes, during step 1. |

## Hotkeys / keys

| Question | Answer |
|---|---|
| Where do hotkeys live? | "users needs their own keymaps that then maps to commands … a hotkey editor that can point to commands that are hotkey available." |
| Keymap storage | In the user profile in ShowData. |
| Hotkey value | "everything is a command in this system … a keymap is like firing a macro … users need full commands on keymaps." |
| When to check a hotkey's command line? | Parse when saved, re-check on show load. |
| Keyboard → actions | Keyboard → UiKey → command lines (virtual keys). |
| Key behaviours | Release + ReleaseAfterHold, Modifier (**Meow**), Key state LEDs. (ShCuts on/off toggle not chosen.) Also: "virtual keys also have double press" — yes. |
| User request | "probe the reference console … toggle on/off keywords now … turn highlight and blind on and off from commandline now." |

## Mode objects

| Question | Answer |
|---|---|
| Syntax: postfix `Blind On` or reversed? | "it would make more sense to do `on blind` … sequences use this" → verb-first: `On Blind`, `Off Blind`, `Toggle Blind`. |
| Grammar detail | Verb-first, bare word toggles (`Blind`). |
| Mode state scope | Per user. |
| Bare `On`/`Off` | "later we will be able to have groups, sequences, fixtures and other things selected, so off would act on selected objects." |

## Security

| Question | Answer |
|---|---|
| Network auth model | Login token on every request (UDP, TCP, WS). |
| Default Admin | "lighting networks are mostly isolated, if we need external access we also need admin access and we will discuss this later" — keep passwordless for now. |
| Password hashes over the wire | Never send hashes. |
| `hissme` | Keep as is. |
| Web origin / CORS | Later, with external access. |
| TCP shell `client_addr` | Trust it only from local (loopback) bridges. |
| Connection limits | No limits. |

## Show files, undo, groups

| Question | Answer |
|---|---|
| Show file version | Version + migrations, "and also add commit version so we can track issues easier." |
| Undo/replication coverage | Generated from ShowData (exhaustive). |
| Group models | "groups lives only inside showdata datapool, fixturegroup sounds like old code? probe how the reference console defines groups we want it similar." |
| Group colour/icon | "we already made this discussion that all our objects should have icon and colour only" (TODO.md: icon + color on NodeMeta, no Appearance pool). |
| Group selection data | Order + grid positions. |
| Group Pawterns | Reserve a (Pawtern) slot now. |
| Group members | Fixtures only — "we have an idea at Meow Corp that subfixtures is not what we want … discussion for later." |

## Structure, render thread, tooling

| Question | Answer |
|---|---|
| Crate split | "in my head we have servers and thin clients, the terminals runs on the server and the clients stream them, right?" (yes) → proto crate first, then engine/terminal modules. |
| Render thread strictness | Strict, like audio code. |
| Tooling guards | Registry tidy test; fuzz + property tests. (CI and workspace lints not chosen.) |
| Where to record decisions | docs/meta/decisions.md + memory. |
| Work order | Registry first; later amended to quick fixes first, then registry, then engine thread. |
| "Harden Claude" (station-local vs replicated) | CLAUDE.md rule — "only do it after this session is done and I say go" (done after go). |

## Network session (master executes)

| Question | Answer |
|---|---|
| What travels between stations? (after example) | "all commands gets send to the server that is the master and it executes there, other servers when slaves forwards connected devices and other events needed." |
| Where do programmer/selection/flags live? | "it does not make sense to have it on the userprofile, a userprofile is mostly views and settings … have programmer, selection and flags live on a user directly so a user can keep its data and change profile." (Matches decisions §2 from March.) |
| Programmer persistence | Saved with the show (modes not saved). |
| Same user on two stations | Shared programmer. |
| Programmers per user | One now, room for more later. |
| Clearing others' programmers | "a huge problem in the reference console is that i cannot clear someone elses programmer" → `Clear Programmer User 2 Thru 5`, Admin + Setup. |
| Station-local vs replicated marker | By type/subtree — "and we need to harden claude to understand this." |
| Where commands run | Declared on CommandDef (`scope`). |
| Master loss | Automatic + epoch fencing; "also research the keep master only data options a user gets when two masters meet" → researched. |
| Session merge | The usual four options done right (GUID-based, every losing side saved first, show UUID + lineage). |
| Feedback latency | Programmer instant, structure waits for the master. |
| Master unreachable | "the reference console does 2 … I can still use everything, but nothing happens on the rig until all commands have been sent to the master; after the 5s timeout it is disconnected … gone rogue and needs a resync" → researched; partly confirmed. |
| Edit locks | Claims + timed takeover (10 s). |
| Under 5 s outage | Keep working, hold and resend. |
| After 5 s | IdleMaster, editable, silent (network output off unless "output when alone"). |
| Rejoin | Auto re-invite + merge prompt. |
| Health/timeouts | Named constants + visible health. |

## Bugs and robustness (§26)

| Question | Answer |
|---|---|
| State ownership (global lock) | One engine thread + command queue. |
| OSC shutdown | Yes, if its user has the right. |
| DMX on exit | Proper sACN/Art-Net goodbye. |
| Art-Net | Spec-correct + unicast to ArtPoll-discovered nodes. |
| Panic handling | Catch per command and keep running — "and we need good responses instead of just internal error … so it points us in the code" (file:line, command, user, build, report #). |
| Errors | Typed errors with codes. |
| Server startup | One ServerBuilder. |
| Tree access | "we want as much public available data as possible so the users can see the internals working, that's the whole concept with lumacore" → read everything, write through one door. |
| Generic Set | Yes, any writable property. |
| Live internals | Change log/events, undo stacks, engine/thread health (not ID pool counters). |
| ID pools | Rebuilt by the show itself (`after_load`). |
| Cleanup | cargo fmt once, clippy fixes, repo hygiene, replace unsafe localtime_r. |
| Work order for new items | Quick fixes first, then registry. |

## Session actions

- "go" → step 0 implemented; "commit and push" → `533f7c4`.
- "go step 1" → step 1 implemented; "commit and push, fix systemtest temp dir" → `90bebb2`,
  `888b408`.
- Open question for the user: regenerate `list_tree.rs` hand-drawn overviews from the real tree?

## Step 2 hardening (after "go step 2" → "harden it")

| Question | Answer |
|---|---|
| Change path: how to find what a command changed? | Automatic diff of ShowData subtrees before/after each mutating command. |
| How do readers read state? | "we dont want http engine to be on the same thread as protocoll and dmx rendering" → clarified: state thread ≠ DMX thread. HTTP reads a published snapshot, never the queue. |
| Panic rollback | ShowData only. |
| Error codes scope | Foundation + crate errors; handler strings convert gradually. |
| DMX renderer reading the snapshot? | "later, we will make the playback engine … root arcswap → playbackengine (does all tracking and effect renderings) → dmx output to the dmx protocols … playbackengine just need basic structure for now" → researched a reference console (manual, live console) → decisions §27. |
| Programmer path to the engine | Own small programmer snapshot (1). "we need to figure out how the programmer and playbackengine work together so we can have effects in the programmer." |
| Value type | "presets, programmer and sequences needs the same values" → one Value. |
| Where the playback engine goes | Merged into step 4 (playback engine skeleton + protocol lifecycle). |
| Skeleton contents | Static cues + tracking; "programmer needs its own phase later". |
| Thread health | All threads, in Runtime.Health. |
| Set: what is writable? | Declared per node type. |
| Journal | Ring of 1000 in Runtime.Journal, not saved. |
| Error code format | E-AREA-NNN. |
| Shutdown deadline | 5 seconds. |
| Who sees full internal errors | Everyone. |
| Repeat panics | Just report. |
| Tree place for health/journal | Runtime.Health / Runtime.Journal. |
| Programmer levels | As on established consoles (inactive values output, only active store). |
| Cue parts | From the start. |
| Preview programmer | "we want a preview programmer per user, later". |
| Priorities | "we already have it defined in a pool so we can rearrange it easier, much better" → §7 Priority pool stays; the usual bands are its default content. |

## Step 3 hardening ("lets harden step 3")

| Question | Answer |
|---|---|
| Bare `On`/`Off` on fixtures | "research the reference console manual, on selection activates all values in the programmer, on blind we want to turn blind on" → researched (`keyword_off`, `keyword_knockin`, live console) → knock in / knock out, bare = selection; bare `Off` also deselects; dimmer zero/restore removed. |
| `Black` as a mode | "we dont need black, we want it to be apart of the grandmaster, research masters in the reference console manual and probe it … does not need to be implemented now" → blackout = grand master (Toggle) switched off; recorded in §16, built later. |
| Highlight/Solo set | Follow live selection — "is it really tied to this step?" → no: effects move to step 4. |
| Mode scope in step 3 | Objects + per-user state + status + LEDs now; effects in step 4. |
| Where per-user state lives | Rename `Runtime.Sessions` → `Runtime.Users`. |
| Modes vs grand masters | "modes per user, and mode-masters per user that turns on like a reference console". |
| Lowlight | Not now. |
| Postfix `Blind On` | Verb-first only. |
| KeyBinding vs UiKey | Binding points at a UiKey; `Bind` makes a user UiKey; immediate. |
| Meow key | Held key, configurable (default Option/Alt); latching on-screen button. |
| Gestures | All five; TUI press-only. |

## Step 4 hardening ("lets harden step 4")

Research: a reference console manual (priorities, freeze, live patch, sACN),
live console (Sequence has `CURRENTCUE` + `USER` in ShowData; `Temp/LoadedPlaybacks`), and all
repo `*.md` on output configuration.

| Question | Answer |
|---|---|
| Freeze direction | Programmer LTP with playbacks, Freeze lifts above all but Super. |
| Playback state on load | "i believe that the playbackengine state has to be saved in the showfile, so therefore it must be in showdata" → `ShowData.Playbacks`; "the playback tree needs to be flexible" (`Playbacks.Sequences`, later `Effects`, …); not undoable. |
| On load | Resume, snapped. |
| Cue parts | Sequence → Cue → Part. |
| Users' programmers | First "attributes should get locked to users who activates them" → then "programmers are like sequences with LTP … multiple users can reach for values at any time and take over the ownership … programmers transfer the values between them … following actors on stage and nexting thru fixtures fast without collisions". |
| Takeover | Value moves out of the other programmer (transfer). |
| Blind Off | On top (fresh touch). Blind values don't touch the stage. |
| Store | All own active values. |
| Selection vs claim | "it should be possible for other users to select a claimed fixture … selecting should not auto-claim"; `Claim Fixture` blocks selection too; claim refused if another user holds an attribute. |
| Lock life / store | While the value is in the programmer; derived from programmers. |
| Value type | Full shape, one step. |
| Commands | Store Cue, Go/Go-/Goto/Off, Release in cues (no executors yet). |
| Masters | Grand master + blackout; `Black` goes. |
| Highlight | "do not implement highlight now, it comes from the imported gdtf file." Solo now. |
| Output mapping | "per universe can be more powerful and we can build a smart UI around this idea better than rows"; a universe has several output lines. |
| Assign syntax | `Assign Universe 1 At sACN 1`. |
| Assign line | Route per Assign, creates the line; clashes are errors. |
| Station-local | As usual (NIC, enable, send-when-alone, CID). |
| Art-Net | Port-Address + reserve Input. |
| Future transports | Postpone generalising, but "prepare for when fixtures talk bi-directional with something similar to a json protocol/API with feedback, dmx will still be king" → engine outputs Values ("lumacore should go with physical/normalized as much as possible thru the whole system"), fixture connection slot, `Runtime.Devices`. |
| Protocol lifecycle | Reconcile + master-only; ArtPoll unicast, stable sACN CID, OSC input names a user. |
| Clear Programmer User | Step 4. |
| Park | `At` = intensity; bare Park = all attributes at output; raw channel park stays — "park is not related to dmx tester". |
| MEOW button | Rename the command-history button to **PURR**. |
| list_tree | Generate from the tree. |
| Set + Patch claim | "probe the reference console … some values on fixtures can be set, but creating and deleting patch objects is forbidden" → a Live Patch split; live Sets undoable. |
| Build order | 4a engine, 4b protocols. |

## Step 4b hardening ("lets harden step 4b")

Research: a reference console manual (port config: input merge Off/Prio/HTP/LowTP, input priority
Super…Lowest; `dmx_ethernet_sacn`), live console (`DmxUniverse.MERGEMODE`, `PORTIN`, `PORTOUT`).

| Question | Answer |
|---|---|
| Ranges | Drop ranges. |
| Who sends | Standalone + master; IdleMaster only with "output when alone". |
| Station settings / NICs | "i want to be able to have multiple nics with for example sacn on all of them, but just some local universes assigned to them … input sacn 1-6 on one nic, output local 1-6 to another nic … merge dmx universes from some guests, we would act like a gateway … all nic settings live on the station settings, protocols needs to be binded to the nics in its turn, and then the routes needs to be using these protocols" |
| Migration of old lines | "no migrations yet, we are still working on this" |
| NIC binding | Named networks, mapped per station. |
| Input in 4b? | 4b output, 4c input. |
| Merge | "probe the reference console" → the common model (universe merge mode, input priority on the line). |
| Direction | One direction per line. |
| Where networks live | "a showfile will be prepped with input and outputs, and a station will just setup its nic and what to use later" → names in the show, mapping in the station. |
| Art-Net fallback | Directed broadcast. |
| OSC | No OSC line by default. |
| Name | Network. |
| Commands | Store + Set + Assign. |
| Unmapped network | Don't send. |
| First run | "this i a safe software, we cannot randomly start send until a user really wants it to a certain place" → nothing sends until mapped, also Default. |
| Status | Per-line state in Info/List + status bar icon per protocol. |

## Step 4c hardening ("lets harden 4c")

Research: a reference console manual (priorities, port config, sACN / Art-Net,
the DMX sheet), its DMX sheet window, live console
(`DmxUniverse.MERGEMODE/PORTIN`, the open DMX sheet's `DMXSheetSettings`).

| Question | Answer |
|---|---|
| Merge level | Per DMX channel, after render (winning source + stamp per channel). |
| Park / masters / Solo | Park wins; grand master, blackout, Solo leave input alone. |
| Source loss | Release after 2.5 s (E1.31), stream-terminated at once. |
| Several sources | E1.31: sACN priority, then HTP; Art-Net HTP. |
| Who listens | Only the station that sends. |
| ArtPollReply | Reply on networks with an Art-Net input line. |
| Syntax | "we want to unify this in code a little bit so we dont write it twice i think" → one Line type, two keywords. |
| Unify lines | One `Line` struct + handlers by Direction; `Output`/`Input` keywords and pools stay. |
| Merge mode | On the universe, `Set Input n MergeMode` sets all its universes. |
| Visibility | Status + sources + `DMX In` — "and this is when we need to implement the dmx window, check the reference console's DMX window". |
| DMX window | A DMX sheet + source marks (settings as on a reference console), View Output/Input. |
| Window data | WebSocket subscription, ≤ ~20 fps, from ArcSwap. |
| Input bands | Mapped onto today's layers (Super, Prog, Highest/High, LTP, Low/Lowest). |
| Build order | One step (input + merge + window together). |

## DMX Protocols window hardening ("before security, we need to implement the DMX Protocols window")

Research: a reference console's network setup screens (network, sACN, Art-Net, DMX), live
`DeviceConfigurations.DMXProtocols.{sACN, ArtNet}` (Data/Discovery/Nodes/Timecode collects).

| Question | Answer |
|---|---|
| Placement | Menu overlay. |
| Organisation | "2. but is it a good idea to place multiple protocols inside a network?" → yes (where vs how); still open after the mockups. |
| Routes | "i think i need an artifact for all this to decide" → mockups published. |
| Header | "i think we want all interface settings to belong in the network menu (check the reference console), in there we set all nic settings, and the networks for dmx protocols is defined with its protocolls, right?" |
| Naming | "its confusing with different network wordings now, we need to differanciate, network and sessions" → "lets call it LumaSession is a better word, change it." |
| Menus | Three: LumaSessions, Network ("and on the network menu also settings for the meow-session nics" → LumaSession interface), DMX Protocols. |
| Network keyword | Keep. |
| Organisation | Network tabs. |
| Routes | Routing matrix; "clicking a cell opens a editor." |
| Rename reach | "1. but no alias" → LumaSession everywhere, no aliases. |
| Words | LumaSession object, `Shell` for terminal sessions. |
| Leave prompt | "1. and patch changes does not happen until it is saved from this prompt, editing the patch in this menu does not happen live on the rig" |
| Staged patch | "claiming drafts etc 1 yes, but networks and routes can be edited live and does not need claim. claim only for entering exiting the patch for now" |
| Conflicts | Live Patch edits by others refused while a draft is held. |
| Session NIC | "1. but we want to be able to host other services like the shell and http on other nics as well from this menu … every nic should have Shell and HTTP for example as a list Yes/No item" |
| Services | LumaSession / Shell / HTTP per NIC, localhost always on. |
| LumaSessions menu | The full menu now. |
| Build order | All in one step. |
| Identity | Named sessions found by discovery; join by address still possible. |
| Invite | Joins if Invite is on. |
| Dismiss | Standalone, DMX output held until told. |
| On/off | Everything networked (localhost stays). |

## Step 5 hardening — security ("lets do step 5")

User hint: "main hint here is, this will mostly live on isolated lighting networks openly".
Research: a reference console's web remote (must be enabled per station; Admin, Guest and 3D can't log in to it),
LogIn keyword, Admin password reset on show load.

| Question | Answer |
|---|---|
| Commit 4d first | Yes (`3b65944`). |
| UDP command port login | Leave UDP open (no token on requests). |
| Member logins / hashes | Members keep hashes (station-to-station traffic may carry them). |
| Station token | Keep source-IP trust. |
| Rights | One gate for every transport (Shell, UDP, HTTP, OSC, keys); OSC runs as its line's User. |
| Remote Login | Make it hold: reported client address believed only from loopback; in-session Login checks it. |
| Password masking | Mask at the source (journal, sysmon, replication, history); fix the masking test. |
| HTTP | Leave open; only logout that really ends the token. |

## Step 6 hardening ("lets do step 6")

Research: code survey (change path, replication types, join, runtime, programmer location,
versioning) and a scan of every .md for step 6 decisions and conflicts.

| Question | Answer |
|---|---|
| Split | 6a–6d (groundwork, link, master loss/election, merge). |
| Undo frames | On the master, replicated (sending user's stack). |
| Undo saved | Not saved. |
| Station states | Standalone / Master / Member / IdleMaster. |
| Coverage | One subtree per field, all undoable (except Playbacks). |
| Old programmers | Start empty. |
| Versioning | "we only look forward, no migrations and backward capabillity for now" |
| Old load fix-ups | Remove them. |
| Load check | Refuse a newer format. |
| Undo home | `Runtime.Users.{User}.Undo`. |
| Stale docs | Fix in 6a. |

## Step 6b — the link (2026-10-11)

| Question | Answer |
|---|---|
| Stream | New TCP port 47658, LumaSession service. |
| Encoding | JSON + zstd, length-prefixed, versioned. |
| Join checks | Same protocol + format; commit only warns. |
| Local feedback | Everything waits for the master for now. |
| Runtime split | "session of a user is how a user can have different screenconfigs etc" / "if they have the same screen-session/screenconfig active then the same thing happens, right? i made decisions earlier about this 1 day ago" → screen configs are profile show data (thin-client handoff), write-through goes to the master; open screens/positions per station. |
| Undo sync | As steps in each result. |
| Master doesn't answer | Build the pending outbox now. |
| Old UDP pieces | "i sm noy sure about 1 and 2, have we made decisions to re-design all of this?" → explained §26/§10 → replace the 3 UDP requests in 6b, JoinNegotiation/JoinPolicy in 6d. |
| Proxmox | "we are not going to use proxmox" → remove scripts + doc. |
| Two servers on one machine | "we need a flag to run two servers on the same machine so you can test it easier" → `--station <name>` (+ `--port-base`). |
| Probe the reference console | "also probe the reference console about sessions" → live console + manual (recorded in §21 "6b hardening"). |
| Priority on join | "it joins and if we are a high prio we get a prompt to take over" → prompt + handover with the 6c election. |
| Failed join download | Keep the old show (staging copy, swap when complete). |

## System Monitor research → 6c / 6d (2026-10-11)

| Question | Answer |
|---|---|
| Dump | "before 6c we need to open the system monitor with app_terminal and get a dump of the full system monitor so we can save it to a gitignored intel/ folder" → `intel/` (SYSMON 127.0.0.1); "check again i have made some changes dump again" → second dump. |
| Research | "reseach these features we can see and lets talk about them and add them to 6c and 6d" |
| Patch version | Yes, in 6c; 6d uses it. |
| Show status | Yes, in 6c. |
| Reference check | 6d after merge, then everywhere. |
| Ownership | In 6d. |
| Output distribution log | Yes, in 6c. |
| Link down | Yes (immediate pending; 5 s still decides IdleMaster). |
| Overload warning | Later, step 9. |
| Startup macro / Agenda | Skip both. |
| Multicast | "what about the multicast question?" → session data stays TCP per member; discovery group configurable (Default / Alternative) in 6c. |

## Step 6c — health, master loss, election (2026-10-11)

| Question | Answer |
|---|---|
| Election | Each member computes the same winner (replicated list + discovery). |
| Up to date | Priority, then most results applied, then online time, then id. |
| Diverged IdleMaster before 6d | Stay IdleMaster, operator decides. |
| Take over | `Claim LumaSession`. |
| Master lost | Election first, IdleMaster only if alone. |
| Known list | "on the reference console all joined stations are in the lists so all client knows who was joined, ad they are removeable, this can have effects such as auto join" → every joined station listed in the show; removed with `Delete Station`. |
| Two masters | "i think we need a prompt so the operator can decide what will happen?" → lower epoch fenced; prompt Join / Keep mine / Stay alone. |
| Heartbeat | 500 ms beat, warn 1.5 s, lost 5 s. |
| Prompt research | "research the session prompts in the reference console" → prompts are native MessageBoxes; modes Cancel / Use Master / Merge / Use Mine. |
| Prompt words | The usual modes (Use Master / Use Mine / Cancel; 6d adds Merge). |
| Show status | The fuller set. |
| Undo info | Station + time on undo frames, in 6c. |
| 6d tests | Copy the reference console's data-negotiation test cases. |

## Step 6d hardening (2026-10-11)

"lets load 6d and make sure to read all relevant md files to this step" / "no only 6d, but also
find md files with related features to 6d". Decisions in `docs/meta/decisions.md` §21 "6d hardening".

| Question | Answer |
|---|---|
| GUIDs | "all objects in our tree needs a strict guid req" → every object has a GUID. |
| Strictness | Type bound (`Identified`) + tree-walk test; duplicates refused on load. |
| Links | "strict guid links with tests as well" → every stored link is a GUID, tested. |
| Merge base | Split base copy, 3-way merge. |
| Both sides changed | Master wins, listed. |
| Show identity | `show_id` in ShowSettings; only the same show can Merge. |
| Defaults | Merge, 60 s. |
| Patch on both sides | Merge greyed with reason; timeout falls back to Cancel. |
| Merge unit | Smallest GUID object. |
| Same slot | Mine to the next free slot. |
| Delete vs edit | Edit wins, listed. |
| Playbacks | Always the master's. |
| Reference check | Keep link, mark missing; `List Reference /Broken`, `Info Show`. |
| Auto-claim | Opening an editor (indirect too); plain commands refused on others' claims. |
| Takeover | 10 s, no answer = granted; Admin `/Force`. |
| Claims | By GUID; `List Claims` replaces `ListClaimed`. |
| Split 6d | One step 6d. |
| Old shows | Older format refused (forward only); demo regenerated. |
| cd | "cd into claim should be like being inside of the claim object so we just can enter list and see all claimed objects" → cd never claims; `cd Claims` + `List`. |
| Autosave | Names say why (before-merge / use-master / use-mine / invite / split-base). |
| Go | "go" → built as one step (§21 "6d built"). |
| Reference GUIDs | "probe the reference console and their objects and the guids to make sure we have implemented it well" → the reference has GUIDs on fewer objects (none on Cue, Executor, DMXMode, Attribute) and links by name path / FID; ours is stricter on purpose (§21). |
| Strict distribution | "when merge and other types of session reference object distribution system is used, guids needs to be used strict" → replication, undo and members' changes name pool entries by GUID (`ShowSubtree::DataPool(Uuid)` …), restores never overwrite another object's slot. |

## Step 7 hardening (2026-10-11)

"lets do step 7, make sure to analyze all md files with related features topics decisions".
Decisions in `docs/meta/decisions.md` §22 "Step 7 hardening".

| Question | Answer |
|---|---|
| Recall add / replace | "it depends like the reference console, when we have a new active value calling a group replaces the selection. but if we have not touched a group it just adds to the selection. probe this behavior on the reference console." → probed live (executing commands on it): adds until a value is set, then the first plain select replaces; `+` always adds; `A + B` replaces with the union. Adopted exactly, for every selection. |
| Grid | The full grid (x/y/z + rotation/mirror, grid cursor, PreserveGridPositions, `Grid`). |
| Store over existing | Refused, names `/Merge` `/Overwrite` `/Remove`; `/GridMerge AppendX` default, `Off`. |
| Group masters | Build now; mode + level on the Group → mode on the Group, level in `Playbacks` (no undo flood). |
| Master command | "group 4 at 50 fade 2, also set group 4 property master 50 fade 0. group 4 master at 50 fade 5?" → `Group 4 Master At 50 Fade 5` and `Set Group 4 Property Master 50`; `Group 4 At 50` stays values; masters fade. |
| Master React | Per fixture (Grand / Group / None). |
| Fixture deleted | "when unpatched it will still render, when deleted all guid related links will stay and not be deleted. we will later need a way to clean up old links, it would be great to see links in different perspective, like if we have many cues and presets pointing at a deleted fixture, we could ressurect it or delete it." |
| Delete referenced group | Refuse, name the links, `/Force`. |
| Datapool | The user's selected datapool. |
| Pool window | "in this step we should make sure the group window is working properly by using commands like edit, store move and delete on objects in the group window. we want to unify this a standard object window, with group additional features like group master levels. this is so we can reuse the code later for presets … there will be plenty of more object windows, so better make it unified now. research all object windows from the reference console and probe the root" → `PoolDef` on `ObjectDef`, generic cells, one PoolWindow. |
| Gestures | The reference console's set as commands, but "long press should expand the whole window as a fullscreen selector … we dont believe in the swipe function the reference console uses" / "hold drag and release to select a object. hold release opens the full-screen selector but does nothing, the user must drag first to hover over objects." |
| Drag | "meow-ctrl-drag would be copy, like a cad software … research this for the most common drag-copy keybind. or wait drag moves, and ctrl-drag copies maybe?" → drag moves, Ctrl-drag copies (Windows/CAD); `pool_drag` profile setting guards touch. |
| Window settings | "it is saved like the reference console in views and on the users current called screen, probe the reference console. this is because users need to be able to perfectly recall views, windows and their settings" → window settings on the window in the current screen, snapshotted into Views (probed). |
| Live feed | One `/ws/pools`. |
| Group edit | Members + grid editor (claims the group). |
| Applicability | For All / Some / None from the user's selection (generic cell field). |
| Split | One step 7. |
| Go | "go" → built as one step (§22 "Step 7 built"). |
| Demo | "renumber the fixtures in the demo showfile to 101-and up so we can see order vs FID" → 101–106, Odd stored in reverse. |

## Step 8 hardening (2026-10-11)

"Lets do step 8, load and read and find all md files related to step 8 …" + "also research codebase and older md file from early this year" + "also probe the reference console and read its manual for comparison" → decisions §23 "Step 8 hardening".

| Question | Answer |
|---|---|
| App binary | App spawns `lumacore-server` (links only the wire crate); `--server` goes. |
| Layer name (`engine/`?) | "Console, you mean like a TerminalSession? Or do you mean station? Explain better so we can avoid confusion" → explained Station / Shell / Engine / layers → "Kernel and core is good, but kernel should be used later in linux and core is conflicting? Give me suggestions on edits for the core naming" → `lumacore-core` → `lumacore-base`; layer `core/`, always `crate::core`. |
| Proto scope / name | "Maybe we need another word for this than proto to differentiate between client and server code better" → `lumacore-wire` (client-facing wire only; station stream frames stay in the server). |
| Client home | Clients under `apps/`, depending only on lumacore-wire. |
| Versioning | Two protocol numbers (SESSION / CLIENT), like a big / small version pair. |
| Mismatch | Refuse with a coded error; app list shows "Wrong version". |
| Station kind | `StationKind` in the beacon now (`Server`, `Node` reserved). |
| LumaSession code | "Would it make sense to have a dedicated LumaSession/ ?" → yes: `server/` → `terminal/` · `net/` → `lumasession/` → `core/`; core uses an outbox. |
| Command output | Text helpers into `core::text`. |
| Minimal build | Layering rule only, no feature gates. |
| Trace | Small `lumacore-trace` crate. |
| Crates vs modules | Modules now + build.rs fix. |
| Status packet | Typed in wire. |
| Split | One step 8. |
| Hub layering (asked while building) | ServerContext holds every layer's state → "Strict at the bottom": core imports nothing above, terminal only core, hub in server/. Then: "We need it to be strict but also flexible enough so we can do thin buiild for small systems" → lumasession/protocols/engines never import terminal; only the shell server hosts terminals. |
| Go | "Go" → built as one step (§23 "Step 8 built"). |

## Step 9 hardening (2026-10-11)

"Lets do step 9, read and analyze all md files that has any related topics, features or discussions related to it so we can harden it." → render path survey, a reference console manual + live tree, online research (Art-Net 4, sACN, consoles, audio/game engines), its system info windows ("i have now opened af few sys info windows in the reference console, screenshot and analyze") → decisions §24 "Step 9 hardening".

| Question | Answer |
|---|---|
| Output rate | "Research this online. I think we want settings for this per output/route protocol, where should we place it? But our engine needs to be able to render other protocols as well at other speeds" → researched → rates on the Line (Rate, Send Full/Changes, PacketGap, Sync). |
| Render rate | Fixed 60 Hz; lines sample. |
| Line default | 30 Hz, Full. |
| Render plan | "Reseach this online what is most flexible for us. And also probe the reference console for hints how they do it" → researched + probed → compiler thread, two tiers (channel tier on patch change, value tier per change). |
| Clock | Session clock over the stream. |
| Leftovers | Remove (LumaRuntime, HealthReport, MasterClock, beacon clock sync); one health source. |
| Clock stats | "We still want ptp analysis to be available for nerds for helth measures" → PTP-style sync stats per member. |
| Nerd health | "1 and 2. We want nerds to attach with the developer clock analysis i we had in early march" → stage timings + own sync stats + real PTP readout; Tracy/Perfetto attach kept and fixed. |
| PTP source | linuxptp (ptp4l) when present, read-only. |
| Panic | Hold last frame, restart (second panic in 10 s → stays down). |
| Overload | Skip, keep time, flag once per episode. |
| Enforcement | assert_no_alloc in debug + render bench with budget + tidy rule. |
| Command reads | Engine's output snapshot. |
| Realtime window | Yes, web window + `List Health`. |
| Split | One step 9. |
| Go | "Go" → building; then "Wait" + "Research this well from other github projects that does realtime session clock sync" → researched Ableton Link, Snapcast, nqptp, PipeWire, FPP, RFC 5905, chrony, ntpd-rs, statime, linuxptp, Unity NGO, Mirror, Source → §24 "Session clock (researched)". |
| Clock discipline | Rate overlay, never backwards (statime OverlayClock; ±0.5 %, 2 % catch-up, frequency ±500 ppm; step only before first render or > 1 s). |
| Clock transport | Time pings on the UDP command port, stamped at the socket; stream heartbeat for liveness only. |
| Clock filter | Burst 16 × 20 ms, then 500 ms; 32 samples, RTT gate + 5σ single-outlier, median offset, RTT-weighted regression for frequency. |
| Handover | Holdover at last frequency; new master keeps its time; members re-burst and slew; states + stats in PTP/NTP terms. |

## App ↔ local server (2026-10-11)

"on a new build, i dont see groups in the group window, i see it in terminal" → the new app had attached to the old pre-step-8 `lumacore-app --server` (pid 11342). "we need to make sure this does not happen again, i have problems when shutting down from the app in the ui, the server does not seem to shut down properly" → Shutdown retested on the current build: it works on every path. Gaps found: the attach isn't checked and the server has no signal handling → decisions §23 "App ↔ local server hardening".

| Question | Answer |
|---|---|
| Stale local server | Ask: [Shut it down and start mine] / [Use it] (protocol matches only) / [Cancel]. Another version = protocol differs or missing, the build differs, or the same binary was rebuilt after the server started. |
| Hung stop | The server handles SIGTERM/SIGINT/SIGHUP like Shutdown /NoSave plus an autosave; `/api/station` reports the pid. The app asks [Force stop] (SIGTERM, then SIGKILL) / [Keep waiting] and never kills without a click. |
| Demo show | "i could not load the demo show from a fresh build … we need a test that does build and makes sure the generated demo show loads, dont we alreday have such test?" → the old server had saved demo.luma as format 4 (E-SHOW-002). Regenerated with gen-demo. Test: generate + save + load through the real path + the committed file matches ("run gen-demo" when stale). |
| Save guard | "1. and a prompt to overwrite if the user is really sure" → refuse to save over a newer-format file, save `{show}_format{N}_{time}.luma` instead, then a prompt to overwrite if really sure (`/Overwrite`). |
| Go | "go" → built (§23 "Step 9b built"), with the leftovers folded in (start scripts, 3 stale e2e). Found on the way: tokens without a nonce (same-second re-login refused) → v2 tokens; gen-demo's `Login LD` no longer created LD. |

## Step 10 — fuzz / property tests (2026-10-11)

Read first: §25, this file, the roadmap and every older md (docs/meta, docs/dev/**, docs/user/**, handoffs). Nothing beyond §25's line planned fuzzing; the older docs promise invariants (replication order/idempotence, Oops pre-image, GUID identity, merge rules, deterministic election) and name more untrusted decoders (sACN/Art-Net, beacon, Time pings, GDTF). → decisions §25 "Step 10 hardening".

| Question | Answer |
|---|---|
| Fuzz scope | Every network decoder + lexer (OSC, sACN, Art-Net, command port, shell packets, beacon, Time ping, stream frame, command lexer, mask_secrets, token verify); GDTF later. |
| Toolchain | `tests/fuzz/` cargo-fuzz crate on nightly outside the workspace. |
| Per step? | "i think the fuzz test should be a nightly long test only and not per step. because it would take to long to test stuff like this during fast development" → fuzz only in the long run; stable decoder smoke tests dropped; property tests stay fast cargo tests (~10 s). |
| Long run | "script i run manually and it logs and runs over night" → `scripts/unix/fuzz_long`, logs to target/fuzz-report/. |
| Generator | Random command sequences through real dispatch from the demo show. |
| Properties | Undo (Oops = pre-image, Redo), merge rules, election + clock filter, identity + masking — plus §25's .luma round-trip and replication ops. |
| Size caps | No caps (isolated networks). |
| Findings | Report, fix later. |
| Budget | ~10 s for the property tests; the build runs each fuzz target ~5 min once. |
| Stale docs | Fix: vibecode-workflow.md → per-step rule; FORMAT_VERSION / SESSION_PROTOCOL numbers. |
| Commands in the nightly | "i think all commands and all keywords needs to be tested as well so we can test all properties and features on the tree during a nightly fuzz. all combinations" → command sweep (decisions §25 "Command sweep"). |
| Generation | Exhaustive sweep (verb × keyword × flag × writable prop, ids + edge values) + deep random sequences via the completion engine. |
| Dynamic | "it needs to be dynamic, so if we add new features down the line we dont need to update it" → generated from the registries at run time; what it can't generate is reported as unreachable. |
| Checks per line | No panic/E-INTERNAL/hang; invariants + Oops pre-image; property read-back; every N lines save/load, member replication, render tick. |
| Roles | Every role; refusals coded and change nothing. |
| Runner | Sweep binary + cargo-fuzz `commands` target; "if we dont finnish overnight it must be possible to continue, on every new start of the test it should check if it completed, crashed or stopped early so it can log and continue properly" → run.json + checkpoints. |
| New build | Continue, new combinations first; `--fresh` restarts. |
| On finding | Log (with replay), reset to the demo show, go on; repeats counted. |
| Hands off | "it should NOT touch the unix-like commands and drive access" → excluded: tree shell (pwd ls cd rm cat … history root) and drive (import export saveshow loadshow newshow deleteshow create); process + LumaSession commands stay in. |
| Mechanism | Required `reach: Show \| TreeShell \| Drive` on every CommandDef / object verb; sweep runs only Show; tidy test: std::fs in a handler ⇒ Drive. |
| Isolation | In process: no server, no protocol runtimes, no engine threads, no sockets; only its report, run.json and a temp dir. |
| Go | "go" → built (§25 "Step 10 built"); "sit takes very long" / "i dont want long tests now" → the 5-minute fuzz runs were stopped after ~25 s each (no crash). |

## Step 11 — cleanup: docs, findings, Oops (2026-10-11)

"i think we have a base from these few days of working, check all md files and compare to the codebase and check what needs to be done" → three read-only audits (handoffs, meta, dev/user docs). "fix all of this except, feedback-system, thin-client, ui-keys, effects, commands that pretend to work(mark them planned/not implemented yet) … all this docs that needs rewrite we need a discussion about so we can unifi the todo.md list … we rather keep data for later and archive stuff rather deleting." → decisions §29.

| Question | Answer |
|---|---|
| One TODO | roadmap.md = the one list (TODO, requirements, project, handoff items merged in). |
| Archive | docs/archive/ with an index; finished handoffs → handoffs/done/. |
| Command docs | Generated reference from the registries + a staleness test; guides only by hand. |
| Networking docs | Merge the three into one lumasession.md. |
| Multi-statement refusal | All or nothing. |
| Delete Fixture | "i think we have 3 implemented now right? the idea here is that we can bring stuff in later and things will just start working again" → §22 stands (links kept); sweep stops flagging broken links. |
| Election | 5 s buckets. |
| Planned | Error + marked everywhere. |
| ReadOnly programmer words | "lets talk about this, all of this needs oops, the ui also needs to be oopsable like a reference console, if i for example close resize or change a setting it needs to be oopsable." → UI Oops (windows, views/screens, user settings, selection + programmer). |
| Oops list | "we want categories in the undo histort, ui is one of them, are there any other categories we need?" → Programmer, Selection, Show, Patch, UI, Users, Settings, Keys, Station; Oops = newest step; filter in List Undo. |
| Station changes | Oops-able, category Station, local. |
| Playback | Never undone (kept). |
| Bind GUID | Per profile. |
| Disk reach | "lets discuss this better, i am not sure" → Root.Drives only via the tree shell; sweep in a sandbox copy of library/ (drive words swept too). |
| Dead words / codes | Codes everywhere + check each word. |
| systemtest | Fix it, keep it. |
| Scope | §18 renames, small gaps, GDTF cell bugs, playback/sACN leftovers. |
| Steps | Docs first (11a), code (11b), Oops (11c). |
| Doc review | Rules + a plan page to review. |
| Visions | Vision-only docs → visions/notes/ (ideas in the roadmap). |
| decisions.md | One file + table of contents with status per §. |
| Reference | Generated Markdown + staleness test. |
| After 11 | "more UI implementation" → "touch friendly, implement all elements needed for future development". |
| 11b build | All in one step. |
| Cue fades | Part timing, the usual rule — "parts and cues are essentially the same kind of object, a part is just like a stacked cues in one cue"; Go in a fade starts from where it is. |
| Selected sequence | "research all md files related to 11b features, we already discussed this. a user can have any object selected in the tree." → "We need to differ from fixture and object selection. a user can have a fixtureselection and a objectselection at the same time like a reference console" → fixture selection + one-only DataPool / Sequence / Timecode on ShowData.Users; "select root.showdata.datapool.2 selects data pool 2"; cd doesn't select; nothing selected → coded error; deleted → cleared; bare Off unchanged; Store Cue / Off Sequence use it; Select on other kinds → coded error. |
| Many-object selection | "maybe we should have datapool selection, timecode slection, object selection, fixture selection and sequence selection per user" → unsure → dropped; "i think i want something similar to the reference console collect keyword … but we need another keyword for it" → Ideas. |
| Highlight | "all highlight values comes from the imported and assigned gdtf file … if there are no values nothing happens and no dimmer hardcoded fallback"; "we need highlight and solo master like a reference console" → per-user masters (§16) now. |
| Pool tap | "it depends on the pool action, check the reference console" → the reference console sequence pool actions, Select default. |
| E1.31 input | Follow the spec. |
| Planned | One E-CMD code; status per verb entry. |
| GDTF | "this software is going to be gdtf native, strict, so we need to be clever on how we work with it" → .gdtf in the show (container), full model (geometry, cells, functions, sets, wheels, physical, Highlight, Snap), import warn only; editing writes a new GDTF — "dont decide now, build later put on roadmap". |
| Show word | "i think we want to be able to do "editsettings show" … i also like the /save and /load flags" → EditSetting for every object; `/Save` `/Load` on Show = SaveShow / LoadShow. |
| TLS | Drop the flags, idea ("we are on isolated networks right now"). |
| MEOWNITOR | Planned; idea: sysmon window follows other stations, a list remembered in the show, reconnects. |
| `I` / If | "if is a valid keyword we need later" → If registered Planned. |
| language.rs | Archive. |
| Sandbox | As decided. |
| Screenshots | "we also want to implement screenshotting before the ui work" → for dev/tests first, "but the same path should be user later for 1 and 3" (view thumbnails, Screenshot command): captured in the client on the server's request, files in library/screenshots/, scripts/unix/screenshots, `Screenshot` command now (minimal). Step 11d. |

## Step 11c hardening — Oops (2026-10-12)

"lets do 11c, read all md files that has related features to it first" → read §21, §29, this file, thin-client, replication/control/runtime/show-files docs, a reference console's manual + Oops menu + live probe. Decisions in `docs/meta/decisions.md` §29 "11c hardening".

| Question | Answer |
|---|---|
| Station steps vs one list | "we only want categorys so the user can have it coloured and see them in the lists, we dont want to reinvent oops" → one list, category = label; a station step is an ordinary step, restored on its station. |
| Oops extras | All four: Oops several steps, /NoOops, create filters, Oops Confirmation. |
| UI steps | Layout yes (drag/resize one step), tab switch and scroll no. |
| Modes / Watch | Watch yes, modes no. "are not watch a spawned thing on a user?" → yes: watches are per user in `Runtime.Users`, so a Watch step undoes from any station. |
| Show load | Clear every list. |
| Main | "probe the reference console for this" → a list by action: create, delete, exchange, file actions, insert, layout, move, remove. |
| Settings home | On the UserProfile → profile. |
| Oops N | `Oops 3` now; the Oops window with the UI kit. |
| Selection subtree | "probe how the reference console does it" → selection is restored apart from the programmer (probe) → own `Selection` subtree. |

## Step 11d hardening — screenshots (2026-10-12)

"lets do 11d, read all md files related to it first" → §29, roadmap, this file, e2e README, testing, the frontend; a reference console manual; "probe and check the reference console's code for screenshotting" → system-test image helpers + live `Store Image /Screen` probe. Decisions in `docs/meta/decisions.md` §29 "11d hardening".

| Question | Answer |
|---|---|
| Capture | "we need to make sure the client works multiplatform, is 1 good?" → yes: vendored modern-screenshot (foreignObject → canvas) works in Chromium, Firefox, WKWebView, WebKitGTK, WebView2. |
| Key | F11 + Print Screen, green flash. |
| Scope | Bare `Screenshot` = all my screens. |
| Reply | Asked now, saved later (sysmon + notice). |
| Vendor | Capture lib and xterm (+ fit) vendored. |
| Compare | Image-compare regression later (roadmap). |
| Where | Files now; Image pool + view thumbnails later. |
| Hide | Nothing: capture as seen. |
| Dev script | Window types, menus, overlays from the registries. |
| Findings | Report + roadmap; crashes / blank windows fixed in 11d. |
| File name | `YYYYMMDD_hhmmss_<user>_screen<n>.png`. |

## Step 12 hardening — touch UI kit (2026-10-12)

"lets do 11, read all related md files, new and old … probe the reference console and analyze its system tests … researching other project online on github that are touch based … window setting windows and prompt dialogs or the network/lumasession windows" → five research reports in `handoffs/ui-kit/research/`. "we want to make this our way a little bit, so after research and decisions we should create artifacts so i can see ui elements and changes before we implement it"; "during implementation and building you should use chrome and the new screenshot feature to verify". Decisions in `docs/meta/decisions.md` §30.

| Question | Answer |
|---|---|
| Client tech | Plain custom elements (ES modules, `frontend/kit/`). |
| Shape | Descriptors from the registries. |
| Look | Ours, the usual sizes. |
| Parts | 12a–12f accepted; theme editor became 12g. |
| Scroll | One finger scrolls lists; controls capture. |
| Right-click on touch | Long press + ring. "The small circle/ball on every tab is the header button so we can save as much space as possible. We dont want swipe-out action. We need to discuss the hold to fullscreen feature we already have" → Hold = "show me more": pool object → full-screen selector with an action row; tab → window menu; tap the dot → window menu. |
| Apply | At once. |
| Keyboard | OS keyboard + own keypad; "We also need a vk for consoles" → own built-in keyboard, per-screen `Keyboard: Auto / OS / Built-in / Off`. |
| Faders | The usual rules. |
| Icons / theme | Own SVG icons, two-tier tokens; "Prepare a theme editor … I dont like how the reference console has implemented it. I want themes to be stored on the user. It has too many color items per theme … I like how reaper daw makes theme edits" → research → 14 colours + 5 adjusters, pick on screen, A/B, presets, on `ShowData.Users`. |
| Station theme | "Yes, but never overwrite a users theme … anyone can go up to a station/console and hold a button/toggle button to see normal theme that everyone is used to." |
| Editor step | Own step 12g. |
| Tests | Showroom window, tablet + phone touch projects, WebKit project. |
| Theme home | On the user. |
| Menus | The usual setup layout, ours. |
| Window settings | Tabs + generic cells, title-bar buttons, Load / Save presets; "The title-bar buttons should be placed inside the window below the tabs and never next to the tabs … a setting to show and hide these. Also auto-hide if the window gets to small" → Window bar. |
| Window bar | Settings + actions (no UI keys); pinned in the settings shell, type defaults, by command — "we need to be clever about this … I think that we need a self keyword later for this. For example, in a macro label self would label a macro. Editsetting self on a window would open the settings." Stored as window settings; setting + auto-hide + `…` overflow + reveal in the window menu. |
| Self | The line's owner (typed → focused window); built with the window bar (12c). "later in a macro it should be possible to do setuservar $macroname self.name" → yes, `Self.<Property>` reads with ShowVar / UserVar (§5). |
| Mockups | Element gallery, window chrome, dialogs + settings, menus + theme editor. |
| Mockup feedback | "I think the general ux concept and ui design is good. But it looks a little bit too much vibecoded and is lacking some of the original design i have from inspiration/ analyze this style and colors. Dont change the design too much" → `handoffs/ui-kit/research/inspiration-style.md`; mockups restyled (neutral greys, olive gold accent, gold dot on every tab, checkbox window bar, flat, no shadows); §30. |

### 12a hardening (2026-10-12)

| Question | Answer |
|---|---|
| Migration | Foundations everywhere, elements per part. |
| Icons / font / hold | Lucide; Source Sans 3 + JetBrains Mono; profile hold time 500 ms. |
| Theme writes | `Set` on tree paths (`ShowData.Users.<n>.Theme`), format 7. |
| Override | Station setting + UI key; "But i think it should be defaulted to a meow-key modifer" → "We should think about screenlock as well with this so default would be screenlock with a F key and meow-F is normal theme. Research the reference console's screenlock" → desk lock researched (corpus + manual; no live lock). |
| Tests | Showroom in 12a, tablet + phone projects, WebKit now. |
| Strict rule | "Make it a strict rules that all windows etc needs to use elements. So if i want a new type of window that is missing elements we need to design the elements first with artifacts" → §30 + CLAUDE.md. |
| DeskLock | Keyword `DeskLock`; F9 + Pause, Meow+F9 NormalTheme; "cd fixture? What do you mean" → explained → "Drop it"; same key + corner taps; station, user, clock; "A fullscreen lockscreen. Image stored on the user as background."; built in 12a. |
| Lock image | In the show on the user; "the lockscreen should have switch user capability. Mac style. The thin client connect to server needs to be unified with the lockscreen and we need a artifact for this design". |
| Switch user | This station's screens. |
| Surface | Big clock + station, user avatars, server switch, status line. |
| Default background | Built-in from the concept. |
| Lock design | "Yes lock the design and store it in the repo for reference as artifact html (is html a good artifact file store format?) so we can change out mind later if needed and have good documentation of all this" → yes, with an offline runtime: `docs/dev/ui-kit/mockups/2026-10-12/` (sources + `support.js` + PNGs), §30 "Design locked". |
| Tokens | `--lc-*` + plain names in `Theme.Palette`. |
| Unify | All three (app home, web login, DeskLock). |
| Split | 12a1 foundations + 12a2 user surface. |
| Action row | As the mockup. |

### 12a2 / 12a3 hardening (2026-10-12)

"Lets do 12a2, load and find all md files that has features related to it, scan all files so we dont miss anything, even old md files and decisions and discussion, then we harden it" → sweep of every md file; a reference console's library folder + show file researched. Recorded in §30 "12a2 / 12a3 hardening" and §8.

| Question | Answer |
|---|---|
| Platforms | "I think the desktop app needs to be multiplatform as well in the ground, so we can port it later as a ios and android app. But also http available. So the lockscreen and design should live in the http ui and also the desktop/mobile app somehow." |
| Surface home | One frontend, two hosts; later: "desktop apps/mobile apps are just basic clients with server discovery and connection that gets lockscreens etc that opens the http page. A webbrowser gets the http directly." |
| App start | Small bundled connect page (kit), then the station's page. |
| App shell | wry / tao now, Tauri 2 decided at the port. |
| Web Servers… | Stations the serving station sees. |
| Lock scope | The device it's pressed on. |
| Unlock password | Password if set; "admin always have admin:admin so powerusers always can login. Isolated networks ok." → "admin:admin always works, this is because id the LD/operator leaves, anyone needs to be able to login that is a poweruser". |
| Keyboard | Built-in keyboard pulled forward (into the surface step). |
| Screens | "Screen 1 shows normal lockscreen while the others shows the lock image blurred". |
| Auto-lock | Yes, a setting (on the LockScreen). |
| Normal theme | Factory preset, nothing of the user. |
| Start state | Surface, auto-login kept. |
| Images | "We need to think about the .luma format better now. It would make sense if a .luma file contains a folder system that represents the showdata tree and each image has a manifest for its GUID. Would this make sense? Lets harden this" |
| Granularity | "3 would make sense. But datapools should maybe be folders so not all datapools live under one file?" → folder per pool member, file per pool kind. |
| Media model | Show image pool, "but we also want images per user so the userprofile travels with custom images … import the userprofile into another showfile and it should look the same without the need to import any more objects except the userprofile." |
| GDTF | "The gdtf files should live in the tree and inside the luma file. Gdtf files have their own guid we want to use right? Also research and analyze the reference console's library path and folders for luma showfile inspiration" → "we need both our own guid and the gdtf guid for later use when for example downloads a global update". |
| File sync / size | "Max 16mb per file is probably a good thing but later we want to be able to have assets on a folder/drive that we can point to so we can have large assets loaded into memory for showcritical rendering. I belive i have this decided in older md files?" → §8 found (outside the show, approved sync); "Regular image pool files should travel as global image files and on the userprofiles." |
| Image bytes | Beside the manifest, deduped on the station. |
| Theme home | Profile carries look + images. |
| Media tiers | Two tiers, "and the luma file carries manifests guids and hashes and pointers to the asset drives that can be changed per station" → "different station can have different paths like nics". |
| Split | 12a2 = show format, 12a3 = user surface. |
| Rights | Users of that profile + Admin. |
| Lock keys | Core keys, can't be unbound. |
| Image words | One Image object; "it would make sense if the syntax where import image userprofile "test.png"?" → "Is this a good idea for future syntax?" → owner first, like `DataPool 2 Group 3`. |
| Icon pool | "I think a user should have their own icon pool and be able to copy them over the the show pool if needed. User icons for userprofile ui elements only, regular image pool for all other showdata related links." → profile: Icons + Images ("we want to be able to imprt icon-sets later so we want two pools"); show: "Show should contain no icons. And the rest icons should be considered global for a station. … We want userprofiles to look the same, but at the same time have a standard set that can be used." → standard set ships with LumaCore. |
| Profile I/O | Yes, same layout, in 12a2. |
| Show cap | 256 MB embedded per show. |

### 12a2 — uploads in a LumaSession (2026-10-12)

"The gap of inporting to a master, is a good thing but we need to harden, if we upload a file to a lumasession server that is a slave to a master, it needs to be uploaded to the slave. This is because the slave can render things and have its assets. So the http upload happens to the ip that we are opening the ui on."

| Question | Answer |
|---|---|
| To master | "We need to clearify, a imported image always goes to the master and distributrd on the lumasession network. Imported assets like big movie files gets uploaded the the http server we are connected to" |
| Copies | "1 gtdf files and patch and all embedded files is a strict need for any server to work. Only asset linked services/items gets disabled/offline so it does not render it if assets is not present. All of this should instead be on the roadmap and just do basic work now so we can continue on ui" |
| Asset upload | "1 but put this on roadmap" (into the open station's root) |
| Master lost | Fail, retry by hand. |

### 12a3 hardening (2026-10-12)

"Lets do 12a3. Load and analyse all md files related to it and also old md files so we can harden" → sweep; §30 "12a3 hardening".

| Question | Answer |
|---|---|
| Device | Device id, lock kept by station. |
| admin:admin | "Admin password should not be possible to set. admin:admin only" → "Admin user always has admin as password and cannot be changed. It always asks for it. Anything that could be a potential need is to have a lumasession password, but put this on the roadmap" |
| Pre-login | "A user can be logged in on multiple stations at the same time. 1 makes most sense i think" (open, pushed live, with presence) |
| Countdown | Everywhere; tap or Esc skips. |
| Switch user | Whole device switches. |
| While locked | Hidden, waiting count shown. |
| Locked keys | Page and station both. |
| Keyboard | Keyboard button + console (Kiosk) flag; QWERTY. |
| Override | On the station, per device. |
| Other keys | "Only F keys. No mormal letters for desklock and keyboard shortcuts" |
| Avatars | Own user colour + profile name. |
| Letter keys | "Yes. But F1 to F10 is reserved for executors/user assignments later." |
| Web switch | Open its page, log in there. |
| Screen 1 | Lowest open screen number. |
| User colour | Next from a fixed list. |
| Key map | "F9-F12 core features, desklock, screenshot, hide/show all sidebars" → the F9–F12 map → "1. Lets define the core buttons now" |
| Dialogs | Pull the standard dialog into 12a3. |
| Mockups | Yes, mockups first. |
| Key map (2) | "Research the reference console's F buttons. I want F12 as hide show sidebar/fullscreen" → its manual: F9/Pause lock, F10 shortcuts, F11/Print screenshot fixed; F6, F12 unused → an aligned map (F10 shortcut mode, Meow+F11 Shutdown menu, F12 sidebars = full screen, Meow+F12 focused window full). |
| Screens | "The desktop apps needs to be able to open screens as individual os windows that we can resize however we want and move them to any display on local computer. I also like the overview from the artifact so we can see multiple screens." → Overview: both (overview window + tiled mode); Displays: by display, fall back to main. |
| F12 / Meow+F12 | "Only hides sidebars. Fullscreen with meow-F12?" → "1. And on macos native fullscreen. The desktop clients needs to do this natively per os and not just maximize" |

### 12b hardening (2026-10-12)

"Lets do 12b, read and analyze all related md files with new and old discussion so we can harden this" → sweep (md files, locked boards, a reference console's message box / popup input / calculator / keyboards, code); §30 "12b hardening".

| Question | Answer |
|---|---|
| Ask model | "1 makes sense but how do http browsers recieve it, local js client?" → over the page's shell WebSocket, drawn by the kit's dialog; answer packet back. |
| Answer path | Yes: answer packet `{id, button, inputs}` → station closes it everywhere → runs the line. |
| Who answers | "1 and it needs to send to close the dialog before executing it" (by kind; first answer wins; close first). |
| Terminals | Text form of the same dialog → after the mockups: "all looks good except the terminal prompts. is there a way to spawn better prompts with the tui that is centered as a window in the terminal?" → a centered ratatui window over the dimmed shell (cli shows the station's frames). |
| Relative | Server grammar (`+ 3`, `* 2`; unparseable = E-CMD-008). |
| No timeout | It waits, shown as n waiting. |
| Pick list | The usual way (Add New = Store next free; None for optional; broken link row). |
| Links | Every GUID link a setting holds. |
| Keypad | Locked 4×5 + unit swipe. |
| Keyboard setting | Per device, per screen. |
| Layouts | English (US) only. |
| More asks | Store over existing; Import UserProfile same GUID; feedback pop-ups. |
| Remember | Keep, per asker. |
| Label | Chips from the pool, empty clears. |
| Plugins | Shape now, API with Lua. |
| VK button | Tap shows / hides, hold picks mode. |
| TUI look | "the terminal tui styles should follow the users theme settings" → "userprofile theme" (the whole TUI, truecolor, background painted). |
| Mockups | "approved" → locked in `docs/dev/ui-kit/mockups/2026-10-12-12b/`. |

### Theme background hue + saturation (2026-10-12)

"one thing i noticed now is that the theme editor needs a background color hue and satuaration slider. ask me more to harden"

| Question | Answer |
|---|---|
| Scope | All neutral greys together. |
| Tint | "i need two versions so i can test this" → test board A (replaced) / B (kept). |
| Ranges | Hue 0–360°, saturation 0–100 on top of the preset. |
| When | "1 but make the artifact now so i can test it and decide" → built in 12g; test board now. |
| Tint (tested) | "A tint replaced". |

### 12b2 — fixes before 12c + window settings + Users & Profiles (2026-10-12)

"we have some issues we need to take care of before 12c, when i have the servers list after i have shutdown the server, it still shows as a green status … we need to create a better window settings artifact for normal window settings, probe the reference console how the default window settings looks like with its tabs etc. the datapool follow setting … needs to be a better list instead of a int … when i press shift on my mac, that should be meow(modifier) but i do not see the meow button in the sidebar get toggled. the commandline bottom bar has a dark line … we need to research and probe the userprofile edit menu and create an artifact for it. when i drag in the group window it selects the slot numbers as text … selecting text we need to harden where it is allowed to do. ask me morde to harden" · "pressing F12(fullscreen) should also hide the bottom and top bars so we get fullscreen of the windows, ask me more"

| Question | Answer |
|---|---|
| Server list | Goodbye + beacon decides (mDNS withdrawn + leaving beacon; no beacon 4 s → gone). |
| Dot colour | Amber "not answering", then gone. |
| Meow key | "research what is the most common key for F keys on mac and let me know so i can decide, pressing shift, alt or command nothing happens right now and i do not see the sidebar meow modifer latch when i hold any of the keys" → researched (fn / standard function keys; Cmd+F5 VoiceOver, Ctrl+F1–F8) → Shift everywhere. |
| F12 return | Small corner handle. |
| Mac F-keys | Hint once + F10 sheet note. |
| Text select | Only where you read or type. |
| Hidden command line | Command line floats in. |
| Sequence | Everything in one step now. |
| Window settings | Tabs from the schema, strip hides at one tab. |
| Pick values | Named choices everywhere (DataPool "Selected DataPool" + datapools by GUID, font Default + sizes, Fit, colour picker, universe list). |
| Profile editor | "floating window like LumaSession menu, next to the luma sessions button. we need icons for the menu window for all major category like network, backup, screens, lumasessions etc." |
| Menu tile | "Users & Profiles". |
| Menu icons | Lucide icon above the label. |
| Profile prefs | Only what we use now. |
| Oops confirm | Keep Never. |
| Sysmon (asked mid-hardening: "when adding the systemmonitor window, it streams in all the lines from the beginning line by line, it needs to somehow get it all the time so it is snappy? ask me to harden this") | Receive: "it does not make sense to send data like this as a stream when it is mostly sent from the server. so a batch of data when loading windows makes sense so the clients can get up to speed faster. this is js clients i guess?" · Transfer: one batch, then per-tick batches — "and i think this is how most of all data should be sent to clients right?" · Virtual list · Station's whole history. |
| Realtime ("the realtime window updates pretty slow, is it possible to get it to run at the native 60fps limit we have so it runs at full speed?") | "research from other server-clients does this so nothing important gets lost, we will later add more important data that the client should show with good performance" (research running) · DMX window 60 Hz when visible · one feed rule + tidy/test · into 12b2. |
| Realtime (after the research: Centrifugo, Foxglove, rosbridge, OBS, game snapshots, Netdata) | Every frame, batched, gaps marked · one `/ws/feeds` per page · JSON columnar + binary for DMX · numbers + canvas graphs. |
| Palette ("the floating commandline, we need to think of this in bigger terms, we want it to be more like raycast later, put this on the roadmap, but we should start aiming for it … research othe projects on github similar to raycast and give me suggestions" · "in normal mode, commandline at the bottom, but fullscreen it should behave more like raycast" · "that means that we need suggestions when it is also at the bottom") | Scope: "1 and artifacts" (foundations in 12b2 + mockups) · Enter runs the line as typed · list opens Auto (profile setting) · Usage: "per user and goes with the showfile so we can learn what users do, we later might want other data so the user learns more what is used the most etc. add to roadmap". |
| Mockups | "approved" → locked in `docs/dev/ui-kit/mockups/2026-10-12-12b2/` (8 boards). |

### 12c — window bar, Self, presets, layout format (2026-10-13)

"lets do 12c hardening, read all docs with any kind of related features to it new and old so we can get all input needed, also probe the reference console"

| Question | Answer |
|---|---|
| Presets | `WindowPreset` on the profile (per window type, copy in / out, Load / Save in the settings window, exported with the profile). |
| Bar items | Settings + declared actions (any setting pinnable; type actions with `Self` lines; `WindowBarItems` ordered ids). |
| Bar path | Every item a `Set` line — "but we need a careful system to reset to defaults if a user deletes or modify too much". |
| Overflow | `…` menu, then auto-hide below the type's minimum (window menu lists the items). |
| Resets | Layered: setting, window (`Reset Self`), bar defaults, profile per type; one Oops step each, asks when > 1 window. |
| Self scope | Windows now, owner model wired for macros / plugins; no owner → coded error. |
| Self timing | Resolved where the line starts, before forwarding. |
| Live title / chips | "so lets think about this, how did we use self to get buttons states as well?" → one state rule via `Self.<Property>` (bar state, auto chips, live title, `{Self}` in key LEDs). |
| Preset holds | Everything (settings, bar, label, colour, scroll). |
| Format (first ask) | KDL + JSON + Lua picked, then: "ok i tought you meant the whole ui build. but inside the luma file it should stay json and export json as well. ask me more to harden this". |
| KDL place | "i tink 1 would make sense in regards to having a similar system to the reference console's UI files since its whole system is based on them(research this) but cleanest way would be json in this case? research and give me suggestion" → researched (read-only layout files, bound by name to a settings class, never in a show). |
| One model? | "probe how the reference console does it and suggest what to do. i need an easier explenation" → two (floor plan in the show; how a window is built ships with LumaCore). |
| Source | Rust registry → JSON (no KDL / layout files). |
| Grid | Keep our split tree. |
| Export | View, ScreenConfig, WindowPreset — "anything should be able to be exported and imported as json" (generic export → roadmap); into `library/ui/<kind>/`. |
| Pick lists | Yes, a bare line asks with a link input. |
| Mockups | "approved" → locked in `docs/dev/ui-kit/mockups/2026-10-13-12c/` (4 boards). |

### 12c2 — safe users & profiles, multiplatform (2026-10-13)

"in the users & profiles menu, we need a better new user and new userprofile system and also delete prompt, it needs to be safe. the menu with network, backup, quick save etc, the buttons are now too big with the icons, analyze and make sure it all works multiplatform, same for all other windows, menus, settings etc so it all works multiplatform."

| Question | Answer |
|---|---|
| New user | One station ask (Name, Role, Profile, Password, Colour). |
| Profile for a new user | First "One shared Default", then: "when a user gets created a profile gets created as well from the default profile. default cannot be held by users. all users have their own profiles from the start, but can share profiles as well." |
| Default | Locked. |
| New profile | "a blank copy should never be possible its either default, copy of or import". |
| Delete user | Always ask, say what goes (logged out everywhere, claims released, own profile too, one Oops step; refuse self / last Admin / Admin). |
| Delete profile | Refuse while in use. |
| Rights | Only from the User. |
| Asks | All station asks (+ `Set User Name`). |
| Menu tiles | Compact kit tiles. |
| Scope | Contracts + all audit fixes now; setup menus' kit rebuild stays 12d. |
| Guards | Tidy + Playwright multiplatform spec. |
| Desk view on a phone | "we also need an artifact that shows a view that was made for desktop and how it scales to a phone. this is a big thing that we need to discuss properly" → options A–E on a board → "none of this, i think we should try to still keep the panes on the phone but utilize the hold-fullscreen feature. test to make this on a artifact so we can get a feel of it. its ok if a group for example would be 1x1 pixel, it should not be tapable at that size, but hold to fullscreen here is key." |
| Saving on a phone | "we want to have the same view as on the desktop so if we change anything on the phone it should happen in the desktop app" |
| Phone navigation | "we want to explore of still having the windows set up as on desktop" |
| Device config | Device setting, default Follow user. |
| After trying HoldFullscreen | Desk maximises too: "yes, as a setting on the userprofile" · content: miniature · hold ≈ 0.4 s + cell-vs-pane rule: "its good" · tablets: same rule. |
| Mockups | "approve" → locked in `docs/dev/ui-kit/mockups/2026-10-13-12c2/` (4 boards + the options record). |
| Built | "go" → built 2026-10-13 (§30 "12c2 built"). |

### 12d — setup menus, feeds, rights levels (2026-10-13)

"lets harden 12d, read all md files with any related features to setup, new and old, and harden". Research reports: `handoffs/ui-kit/research/12d-docs.md`, a reference-console study, `12d-feeds.md`, `12d-rights.md`, `12d-backup.md` (summaries in §30 "12d hardening").

| Question | Answer |
|---|---|
| Scope | "we need an artifact after all decisions" → Four + Settings menu. |
| Shape | Rust registry → JSON. |
| Overlay / window | Overlay, phone full screen. |
| Push | "most of the interface and not only settings should be used this way with sse and subscription of feed, right? what do we have now?" → everything live on `/ws/feeds`: "1 makes sense, but research this from multiple server-client projects wih similar architecture, not lighting related, on github. big projects." |
| Pre-login / build / limits | Open socket + auth inside · shared build + revisions · HA-style limits. |
| Power toggle | Per menu. |
| Dangerous actions | Station asks. |
| Rights | "we need setup group, and group assignments for users, i think. admins have full acess. i think the rights level of the reference console is clean and would fit this system as well, we not want to assign users to multiple groups, just a level. guest, playback, admin etc.. what levels are there we need? research this from all old and now discussion md files" → six levels (a ladder) · Setup = rig + station + session · Playback selects and sets, never stores · OSC lines without a user: refuse. |
| Row actions | Row tap fills the lines. |
| Network / DMX Protocols / LumaSessions | Interfaces · Networks · networks × Lines·Routing·Discovery · Stations·Known·This station + hand-over. |
| Backup | "1 but also research other meus like this from other big projects on github and suggest improvements" → recovery copy + versions · tiered cleanup (station) · ask once at startup · manifest facts (format 12). |
| Phone / colours / last tab | Top tabs + bottom block · power filled, rest marks · on the profile. |
| Open by command | `Menu "<name>"`: "would list menu make sense to list all available menus?" → yes, `List Menu` / `Info Menu`. |
| Props | Typed props + function block. |
| Settings | Station · Show · Assets · About: "would make sense, but it needs to be clear for the user that station is local and never stored in the showfile etc" → home marks on every tab and cell. |
| Member's DMX Protocols | Master's line states. |
| Tests | Real e2e per menu + feed tests. |
| Grid | "wait, we need grid cells and columns and rows like a reference console" · "we need to be able to multi-select and multi-edit lines and rows" · "also make sure phones can multi-select easily" · "this is a big an important thing we need to unify in the system so all grids looks the same, research both the reference console and other big projects and give me suggestions" (research: a reference-console grid study, `12d-grid-projects.md`, `12d-grid-lasso.md`). |
| Multi-select | "1 makes sense, but 2 [lasso] is most familiar to lighting programmers and will probably work in the rest of the UI when we come to layouts. research this more" → device setting Drag Auto (lasso on desk / tablet, scroll on phone, hold-then-drag everywhere) · + / − / Thru buttons · a lasso replaces · pinch: nothing in grids, zoom in layouts. |
| Edit | "i like two finger tap for editing on touch, it is a safe way to edit something and rarely happens by mistake. but we need some of 1 as well, give me a new suggestion" → two-finger tap + Edit cell + Edit chip. |
| Multi-edit / block | The usual values, one line, one Oops · a fixed slab. |
| Columns / looks / scope | Priority + frozen + scroll on phones · header menu + filter row + sets on the profile · the usual meanings in our tokens · menus now, every later grid uses it. |
| Unify every menu | "you need to add the grid system to the other settings artifacts as well, network i need to be able to grid select for example all shell cells and edit them, so it needs to unified grid. stations also need the grid (And card view) so i can multi change settings on multiple stations at one time(the reference console does this with station mtc, ltc and dmx ports, probe it). backup menu is better, but it needs to be load show with partial load(check how the reference console does this and lets dicuss it), i am not sure about snapshots" → researched (stations and partial load). |
| Power toggle | "the bottom right ON/OFF Switch does not look good" → PowerToggle board A–D → "power toggle A is approved". |
| Slab | "the slab needs minimal icons and standard colors for select, new, copy, cut, delete, oops, import export etc" |
| Slab everywhere | "network board is approved, grid board is approved, i think we also need the slab in some other menus now, like the user and userprofile menu add and delete, make sure to unify this in the UI as a tidy requirement. i can also see that it is needed on the phone versions on the user and profiles" (the Stations / partial load / snapshot questions were set aside for later). |
| + / − latch | "when pressing + or -, it should stay latched until i press it again in a grid so i can select multiple things easier" (Thru stays one-shot). |
| Edit card | "when i multi select rows in the grid and press edit, it should open a dialog so i can see a unified card with tabs and <different values>, this way it will be easier to edit on the phone as well. so we can have both the grid and the edit dialog card" |
| Users board | "\"12d · Users & Profiles on <luma-grid> + slab (desktop + phone, one selection)\" is approved, remake all other artifacts of 12d to have this grid setip" → LumaSessions, Backup, Settings rebuilt on one shared board engine; "dont implement yet, i still need to test this out". |
| Approvals + Backup + Settings | Network, the grid board and LumaSessions approved. Backup: "i dont like load parts, we rather have it say load show then show the load parts dialog so we can get partially things from a luma file". Settings: "is good but i want i to be clear what is a string, int, bool and a list of drop down, it needs to be more visually clear what kind of setting it is." |
| Kind badges / Settings | "the color chips was too much" → neutral outline badges → Settings board approved. |
| Backup / history / stations | "backup board, i dont see the load show button so i cannot try the parts load dialog? we also need the showfile history like the reference console(only a list of save dates basically, maybe other good data?) probe the reference console. 3 station settings home is approved" |
| Save / phone | "load show is better and good, but i am now missing save show as well. the phone is missing load and new show." |
| Colours + matrix | "save show, load show, info and history buttons need distinct colors, show history is good but we need more like a object matrix because a lot can change during a save" |
| Mockups | "backup board is approved" → every 12d board approved → locked in `docs/dev/ui-kit/mockups/2026-10-13-12d/`. Snapshots still open. |
| Recovery timer | "shows auto save setting" → the recovery copy timer stays the show's `Autosave` setting (as built in 12d part 5); cleanup stays a station setting. |
| Remote station edits | "that is a good question, research this from other big github projects and give me suggestions" → research `handoffs/ui-kit/research/12d-remote-station-edits.md`; today direct to the station's command port (12d part 4 deviation). |
| Remote station edits (answer) | "Direct, hardened (Recommended)" → identity edits go straight to the station; hardening listed in §30 "Remote station edits" and the roadmap. Through-master and identity-in-show rejected. |

## Merge of bugfixes (2026-10-13)

The `origin/bugfixes` branch (R1 fixes, sweep issues, quoting) merged into main; the user's
decisions for the collisions (recorded in decisions.md §30 "Merge of bugfixes").

| Question | Answer |
|---|---|
| Backups: the branch's `library/shows/backups/` (newest 10) or main's `.versions`? | Main's `.versions` wins; the branch's scheme is dropped, its hardening ported into `core::backups` (copy by the OS, never over a version from the same second, a long name fits, a read error is reported, not "nothing to keep"); `drive::check_file_name` stays for show names; docs say a reference console's `backups/` maps to Versions. |
| Delete asks: the branch's dispatch ask or 12d's grid asks? | One ask, in dispatch: the branch's `delete_first` is the only ask for every Delete (`/NoConfirmation` skips it); 12d's grid asks fold into it and it names the count and the rows ("Delete 3 lines (Output 1, 4, 7)?"); a grid delete never shows two pop-ups (a test); the other station asks (Online Off, HoldOutput On, Leave / Claim LumaSession) stay. |
| Start markers: `.starting` or `running`? | Keep both, woven: `.starting` skips the startup show once after a start that died loading it, `running` offers the recovery ask; both firing = skip first, then the ask offers the copy; `.starting` gitignored next to `running`. |
| Protocol numbers? | Bump once past main: `SESSION_PROTOCOL` 12, `CLIENT_PROTOCOL` 13; `FORMAT_VERSION` 16 only if ShowData or its serialization changed against main (it didn't: stays 15). |
| Busy handling on main's new paths? | Cover them: feed builds, remote edits, the recovery ask, menu / setup lines. |
| Layout PUT / Store View? | The branch's removal stands; main's 12d frontend and kit must not call the removed paths; Store View / Rename are station asks. |
| Syntax rules in 12d's code? | The branch's rules apply everywhere (quotes kept until `tokens::unquote`, bare `/word` = option, library-relative filenames, one range rule, each statement its own step, prefix resolution, full words in stored lines). |
| Frontend conflicts? | Keep main's kit-based 12d UI; port only the branch's behaviour changes (no client-only Delete confirms, the pool title escaped), never its old markup. |
