---
title: Lum
category: docs
---
# Lum

`Lum` is LumaCore's command language. **This page is its binding spec** (decisions §33): the
parser (`lumacore-syntax`) implements exactly the `grammar` block below, a test fails when the
two differ, and every ` ```lum ` example on this page is a parser test.

- **Changing Lum is a hard discussion of its own** — the change, the alternatives, what it
  breaks and which edge case it adds — decided first, then this page, then the code. Never on
  the side of another task.
- The words themselves (verbs, objects, their slots, options, rights, examples) are in the
  generated reference, built from the registries:
  [commands](../reference/commands.md) · [objects](../reference/objects.md). This page says how
  words combine; the reference says which words exist. They never disagree: both come from the
  same signatures.

## Two parts, one prompt

- **Lum** — the operator language: select, set values, store, assign, play back, run the show.
- **The tree shell** — a small Unix-style fence for browsing the object tree: `cd`, `pwd`,
  `ls`, `cat`, `tree`, `find`, `grep`, `head`, `tail`, `wc`, `du`, `stat`, `diff`
  ([tree shell](../terminal/tree-shell.md)). A statement that starts with one of these words is
  read with the fence's rules; every other statement is Lum.

Names: `LumaCore` is the system, `Luma` the cat and the cursor that moves through the tree,
`Lum` the language.

## Grammar

````grammar
(* A line *)
line          = statement { ";" statement } ;
statement     = alias | fence | hint | lum ;
hint          = WORD "?" ;                     (* the word's quick hint *)

(* Lum *)
lum           = [ select-op ] [ verb [ phase ] ] [ if-early ] [ target ] { slot } [ if ] { option } ;
select-op     = "+" | "-" ;                    (* only with no verb: add to / take out of the selection *)
phase         = "On" | "Off" ;                 (* hold verbs only: Top, Flash, Temp, Swap *)
target        = item { set-op item } ;
item          = path | ROOT-PATH | "Self" | ids ;   (* bare ids: the verb's default object, else yours *)
path          = segment { segment } ;          (* owner first; each segment a declared child *)
segment       = OBJECT [ id-kind ] [ ids ] ;
id-kind       = WORD ;                         (* a kind its object declares for its ids, ids after it:
                                                  Fixture Houselights 4 — MVR's ID type + CID *)
ids           = range { set-op range } ;
range         = id [ "Thru" [ id ] ] | "Thru" [ id ] ;
id            = NUMBER | QUOTED ;
set-op        = "+" | "-" ;
slot          = HELPER [ value ] | value ;     (* which slots, in which order: the signature *)
value         = NUMBER | relative | spread | WORD | QUOTED | LINE | SELF-READ | target ;
relative      = ( "+" | "-" | "*" | "/" ) NUMBER ;
spread        = NUMBER "Thru" NUMBER ;
if-early      = "If" target "EndIf" ;          (* printed as `if` at the end *)
if            = "If" target ;
option        = "/" NAME [ value ] ;           (* whether it takes a value: declared on the option *)

(* Aliases: typed lines only *)
alias         = "!" NAME { any-token } ;       (* the typed words are appended to the alias line *)

(* The tree-shell fence *)
fence         = FENCE-WORD { fence-arg } ;
fence-arg     = TREE-PATH | "-" LETTER [ value ] | "--" NAME [ value ] | QUOTED | WORD | NUMBER ;

(* Tokens *)
NUMBER        = [ "+" | "-" ] DIGITS { "." DIGITS } ;   (* a glued sign belongs to the number *)
QUOTED        = '"' { any char but '"' } '"' ;          (* text, never a keyword *)
LINE          = '`' { any char but '`' } '`' ;          (* a command line as a value, never text *)
WORD          = LETTER { LETTER | DIGIT | "_" } [ "+" | "-" ] ;  (* case-insensitive; a glued
                                                  trailing sign only before a space, ";" or the end: Go- *)
ROOT-PATH     = "Root" "." PART { "." PART } ;          (* the tree form; with SELF-READ the only place a dot nests *)
PART          = NAME | DIGITS | QUOTED ;                  (* a node name as stored, a number, quoted text *)
SELF-READ     = "Self" "." NAME { "." NAME } ;
TREE-PATH     = ".." | "/" | "~" [ NAME ] | ROOT-PATH | PART { "." PART } ;   (* ~ = the local library, ~usb1 = a drive's *)
OBJECT, HELPER, NAME, FENCE-WORD
              = WORD from the vocabulary (the registries) ;
operators     = "+" "-" "*" "/" standing alone (space on both sides) ;
````

## The rules

How the grammar is read. Each rule is one sentence; the decision it comes from is in §33.

**Words**

- **One name per word.** Every verb and object has exactly one name — no short forms, no
  prefixes. A new word never changes what an old line means.
- **Case:** words ignore case (`store group 3` = `Store Group 3`); quoted text keeps it. A
  declared value (a choice, a protocol) ignores case too and is written out in its declared form:
  `Set Group 3 "Mode" alwayson` runs, is journalled and shown as `Set Group 3 "Mode" AlwaysOn`;
  any other spelling is refused with a hint (`always_on` → did you mean `AlwaysOn`?). Lowercase
  always works.
- **Quoted is text, never a keyword.** Whether a quoted word is a name, a value or a property
  name comes from the verb's signature, never from a guess.
- **Property names are always quoted:** `Set Fixture 3 "Name" "Spot SR"`.
- **A property is never nested inside its quotes.** A group of settings is a child object, owner
  first: `Set UserProfile Theme "AccentHue" -30`, `Set UserProfile Theme Paint "Accent" "#9a7a08"`
  — there is no `"Theme.AccentHue"`.

**Targets**

- **Owner first.** A nested object follows the tree: `DataPool 2 Group 3`, `Sequence 4 Cue 2`.
  A child alone uses your selected owner (`Cue 3` = a cue of your selected sequence).
- **An id is a number or quoted text.** Numbers may have dots when the object takes parts
  (`Cue 1.5`, `Master 2.1`, `DMX 1.001`, `Window 1.4`); everything else is quoted
  (`Group "Front"`, `Interface "en1"`, `Master "Grand"`).
- **Every object declares how it reads a dotted id** (decided 2026-10-17, decisions §12 "Tags
  hardened"; not built yet): *Decimal* (`Cue 1.5`, so `1.10` = `1.1`), *Parts(n)*, a fixed
  number of parts (`Master 2.1`, `DMX 1.001`, `Window 1.4`), or *Path*, one part per tree level
  at any depth (`Tag 1.2.1`, so `1.10` ≠ `1.1`). An object that nests itself defaults to Path.
  `DMX` alone also reads one part as the universe: `DMX 2` is universe 2's slots, `DMX 2.001` one
  address (no absolute form: `DMX 513` is universe 513; `DMX 1 Thru 2.010` mixes levels, E-SYN).
  The one resolver reads ids only through the declaration, never as a float. A dotted id on an
  object that declares none is E-SYN. Changing a declaration is a visible registry change (the
  reference and the golden record show it). `Fixture` declares none: a fixture is a fixture
  (decisions §40 3). On a Path, `Thru` walks the children of one parent
  (`Tag 1.2 Thru` = 1.2, 1.3, 1.4, …); ends under different parents are refused (join them with
  `+`).
- **Two ways to name a nested object.** The operator form is keywords with spaces, owner first
  (`DataPool 2 Group 3`); the tree form is node names with dots from `Root.`
  (`Root.ShowData.DataPools.2.Groups.3`). A dot nests only in a `Root.` path or a `Self.` read;
  anywhere else it joins the parts of a number. `DataPool.2.Group.3` is not a form.
- **Every node has a number.** A collection's children are numbered by their own number
  (`DataPools.2`); every other node by its place in the tree's declaration (`Root 3` = Resources).
  `Root 3`, `Root.3` and `Root.Resources` name one node; `Root` is an object like any other —
  alone it becomes your default object (`Root>`), its default verb is List (`Root> 3` =
  `List Root 3`). A line keeps the numbers or names you typed (the echo confirms what ran); a
  number in a stored line may point elsewhere after an update that adds a node.
- **Ranges:** `3`, `1 Thru 4`, `1 Thru` (to the last), `Thru 4`, joined by `+` / `-`; numbers
  that don't exist are skipped; a range keeps its typed order (`5 Thru 2` = 5, 4, 3, 2).
- **One list of things per statement.** Every extra — a file, a source, a description, a
  network universe — carries a `/label`.

**Operators**

- **A sign glued to digits is part of the number; a sign standing alone is an operator.**
  `At -120` is absolute, `At - 10` relative. In a list the operator must be spaced: `1+3` is
  refused.
- What an operator means comes from its slot: a set operator in an id list, relative in a
  value, add / take out at the start of a statement with no verb.

**Verbs and defaults**

- **A statement starts with its verb.** A statement with no verb is an object with ids (its
  type's default verb), a bare number or name (your default object), `At …` (on your
  selection) or `+` / `-` (the selection).
- **Each object type declares one default verb** (`Group 3` = `SelectFixtures Group 3`,
  `Sequence 3` = `Select Sequence 3`, `Blind` = `Toggle Blind`). The list is in the object
  reference.
- **A counted object typed alone sets your default object** (`Group` → the prompt shows
  `Group>`, then `3 At 50` = `Group 3 At 50`). An object there is only one of (`Blind`,
  `Clock`) typed alone runs its default verb.
- **A verb's own default object comes first:** `Store 2` = `Store Cue 2`.
- **`<target> At <value>` is the `At` verb on that target:** fixtures, groups and presets are
  selected, then set; an attribute is set on your selection; a master gets its level.
- **`At` is a value or a destination — the verb's signature says which.**
- **`Set` only sets properties; an action is a verb.** A level is set with `At`
  (`Master Group 4 At 50`), a one-shot change is its own verb (`Sort`, `Split`, `Call`), and
  putting one object into another is `Assign`.
- **The property's kind decides its value slot.** A choice takes its declared words — in
  `Set Fixture 1 "MasterReact" Group` the word `Group` is the choice, not a group; a link takes
  an object of its type, by any target form (`Set Sacn 1 "Network" Network 2`, `"Rig"`, a
  `Root.` path), kept by its GUID.
- **Trailing slots bind from the right:** the value is last, the quoted property before it,
  the ids take what is left (`Set UserProfile "LD" "ClearBuffer" On`).
- **Hold verbs** (`Top`, `Flash`, `Temp`, `Swap`) say `On` (press) or `Off` (release) right
  after the verb.

**Lines**

- **Typing is fuzzy, lines are exact.** While you type, each word is matched fuzzily and the
  full line shows as ghost text; Enter runs the line shown, written out in full. Stored and
  machine lines (keys, macros, cue commands, OSC, HTTP, the UDP port) are exact.
- **What runs is always written out:** a typed line is expanded on your station — aliases,
  fuzzy words, bare numbers, `If … EndIf` — and the full line is what runs, is forwarded,
  journalled and kept in history. Every declared word is written in its declared form —
  verbs, objects, options, property names (inside their quotes) and declared values:
  `set universe 1 "mergemode" htp /noconfirmation` runs as
  `Set Universe 1 "MergeMode" HTP /NoConfirmation`. Free text and tree-path node names keep
  what was typed. Lowercase always works; a copied line typed back runs as shown.
- **A line given as a value is in backticks:** ``Bind "F5" `Set Self "AutoScroll" Toggle` `` —
  so its quotes stay plain. `"…"` is always text, never a line; a backtick can't stand inside a
  line (there are no escapes). The verb's signature says which slots take a line.
- **A stored line can't use your personal things:** a bare number, an alias or a fuzzy word in
  a key, macro or cue command is refused when it is saved.

## Sentence shapes

```lum
Store Group 1
Info Fixture 1
Delete Group 2
Copy Group 1 At 2
Assign Universe 1 At Sacn 1 /Universe 1
Fixture 1 Thru 4
Fixture 1 Thru 4 At 50
Group 3
At 50
Full
```

- `<Verb> <Target>` — the verb acts on the target: `Store Group 1`.
- `<Verb> <Target> At <Destination>` — copy, move, assign: `Copy Group 1 At 2`.
- `<Target>` — the object's default verb: `Group 3` selects its fixtures.
- `<Target> At <Value>` — select and set: `Fixture 1 Thru 4 At 50`.
- `At <Value>`, `Full`, `Zero` — on your selection.

## Objects and targets

An object is the thing a sentence is about. Object nouns are singular and stable, and each
points at a real place in the [object tree](../object-tree/object-tree.md):

- `Fixture` (its `DMX` addresses and `Multipatch` copies), `FixtureType`, `Universe` (the
  object: name, merge mode, lines), `DMX` (its slots: what goes out — `DMX 1`, with `/Input` what
  comes in —, park, where a fixture goes),
  `AttributeMap`, `AttributeRule` — the patch (an area is claimed by its tree path, never a word)
- `Group`, `Preset`, `Macro`, `Sequence`, `Cue` — show content in a datapool
- `Executor`, `Master` — playback
- `Network`, `Sacn`, `ArtNet`, `Osc` — data networks and protocol lines (each protocol of the build is
  an object; a numbered one is a line, one DMX protocols row: `Sacn 1`)
- `User`, `UserProfile`, `Programmer`, `Selection` — who works and with what; a profile's
  `Theme` (its `Paint`) and `LockScreen`
- `Attribute`, `FeatureGroup`, `AttributeTemplate` — how attributes are grouped
- `LumaSession`, `Station`, `Client`, `Shell`, `Interface`, `Drive` — stations, their connections
  and their drives (decisions §38)
- `Tag` — a tree of tags (decided, not built: decisions §12 "Tags hardened"), see below

What `Object` means in Lum is in [objects.md](objects.md). "Session" alone is not a noun:
server-to-server sessions are `LumaSession`, a user's terminal is a `Shell`.

```lum
DataPool 2 Group 3
Store Sequence 4 Cue 2
Goto Sequence 2 Cue 3
List Sequence 4 Cue
Clear User 3 Programmer
Master Group 3 At 50
Park Fixture 1 Attribute "Pan" At 30
Group 1 + 3 - 2
Group 1 Thru 4 + Fixture 7
Fixture 5 Thru 2
Fixture 3 Thru
Cue 1.5
Master 2.1 At 80
Master "Grand" At 100
DMX 1.001 At 255
DMX 1
DMX 1 /Input
Select Root.ShowData.DataPools.2
List Root
List Root 3
Root 1.5
List Root.3
List Root "ShowData"
```

A quoted id may hold `*` wildcards anywhere (decisions §35): it names every row whose name
fits, ignoring case — `Group "Mac*"`, `Fixture "backt*blue"`. Only in a target: a value
(`Label Group 3 "A*"`) is never a pattern, a name may not contain `*`, and a pattern that fits
nothing is not found (never a new name). There is no bare `*` id: `Group Thru` is every group.

```lum
Group "Mac*"
Fixture "backt*blue"
Delete Group "Front*"
Store Cue "BO*" /Merge
```

**Tags** (decisions §12 "Tags hardened"; built except `If Tag`, which comes with `If`). A
tag is a node in one tree, numbered per parent (`Tag 1.2.1` = All › Lx1 › Right), and any show
object carries tags as GUID links. The Tag object's own verbs act on **the tags**: Store, Label,
Delete, Copy, Move, Assign / Unassign, EditSetting, List, Info. Every other verb on a tag
**expands to the tag's members** and runs on each kind that takes it, skipping the rest with a
summary. It reaches the tag's own objects (the current level) unless `/R` adds every tag under it.
`Assign Tag … At <object>` always adds a tag, a tag included (it tags that tag); nesting a tag
under another is `Move`, asked first (the one nesting rule, under Move below).

```lum
Assign Tag "Right" At Fixture 1 Thru 12
Unassign Tag "Right" At Fixture 3
Assign Tag 2 At Tag 1
Move Tag 2 At Tag 1
Move Tag 2 At Tag 1 /NoConfirmation
Store Tag 1.1.3 "Centre"
Delete Tag 1.2
Tag 1.1
Tag "Lx1" Tag "Right"
Off Tag "All" /R
Go Tag 1.2 Thru
Fixture 1 Thru 12 If Tag "Lx1" /R
```

**The MVR tree** (decisions §34 "34g4 part 3c hardened"): the patch's own tree, MVR's layers and
nested groups, is `MvrLayer` › `MvrGroup` › `MvrGroup`, numbered pools whose place is a parent
link; `MvrClass` and `MvrPosition` come with an import (no `Store` makes one). The `Mvr` prefix
names MVR's own concept (a `Group` is a selection). A bare one selects every fixture in it and
under it. A fixture has one place: `Assign` puts it in (it leaves its old group), `Unassign` puts
it in No MVR group. A fixture that holds fixtures — a folder, `Fixture Media 1`, or any fixture
(§40 3) — is a node of the tree too: `Assign … At Fixture Media 1` puts fixtures in it, `Move`
of a fixture **At a node nests** it (asked first) and **At a bare number renumbers** it; a folder
has no number to move. `Delete` of a node asks — all of it goes, or `/KeepChildren` lifts what
is under it to the node's parent. It has one class the same way: `Assign … At MvrClass n`,
`Unassign … At MvrClass`. Tags and the MVR tree share only what every object has — an MvrGroup
carries tags.

```lum
Store MvrLayer "Rig"
Store MvrGroup "Truss 1"
Assign Fixture 101 Thru 104 At MvrGroup 4
Assign Fixture 101 Thru 104 At Fixture Media 1
Unassign Fixture 101 At MvrGroup 4
Move MvrGroup 3 At MvrGroup 2
Move MvrGroup 3 At 7
Move Fixture Media 7003 At Fixture Media 2
Move Fixture 849 At Fixture Media 2
Move Fixture 849 At 850
Delete Fixture Media 1 /KeepChildren
MvrGroup 3
MvrLayer 1
MvrClass 2
Assign Fixture 301 Thru 306 At MvrClass 2
Unassign Fixture 301 At MvrClass
Assign Tag "FOH" At MvrGroup 3
```

A target can be another object, a destination slot or a path:

```lum
Assign View 3 At ViewButton 5
Copy Group 1 At 2
Move Fixture 1 Thru 4 At 10
Assign Universe 1 Thru 6 At Sacn 1 /Universe 101
Assign Universe 2 At ArtNet 3 /Universe 0.1.2
Assign Fixture 201 Thru 206 At DMX 2.001
Unassign Fixture 3 At DMX
```

### Patching

A fixture is created with `Store`, given its address with `Assign`, and loses it with `Unassign`
(decisions §33 48, §34 round 3 19; there are no `Patch` / `Unpatch` verbs — the patch window's
*Patch to…* and *Unpatch* run these lines).

- **`/FixtureType`** takes the show's type by number (its slot), a loaded type by name, or a
  `.gdtf` file from `library/gdtf/`, which is imported first in the same step. `/Mode` and
  `/Revision` pick inside it (the newest revision unless named).
- **Where it goes is `At DMX`.** `At DMX 2.001` is an exact place; `At DMX 2` the first free run in
  universe 2; a bare `At DMX` the first free run anywhere. The whole selection fits in one universe
  or the line is refused.
- **An exact place is exact, a run skips.** One fixture at a taken address overlaps (both turn red
  and are parked until it is fixed); several fixtures skip taken slots, and the reply names them.
- **Addresses are packed.** `/Every n` starts each fixture n slots after the one before (never
  less than its footprint); `/KeepSpacing` keeps the gaps the fixtures already have — the block
  keeps its shape and moves past what is taken. `/Every` is the same option as on `Watch`: every
  n of the line's unit.
- **A fixture's addresses are its `DMX`, owner first:** `Fixture 3 DMX` (every break), `Fixture 3
  DMX 2` (break 2). Two selections trade addresses pair by pair with `/Swap` (uneven counts stop
  at the shorter list).
- **A fixture has three ids** (decisions §40, 2026-10-20): a fixture number (`"FID"`,
  may be none), an id type (`"IDType"`: Fixture · Channel · Generic · Houselights · NonDim · Media
  · Fog · Effect · Pyro · Marker — MVR's list with its reserved slot 3, Generic, the default) and
  a number in that type (`"CID"`). The number is `Fixture 4`; the second id is typed inside the
  fixture's id: `Fixture Media 7003`, `Fixture Generic 12 Thru 20`. FIDs are unique in the show,
  CIDs within their type. The pool's key is nobody's business: a fixture with no FID (a folder)
  is reached by its second id or its name. `"MoreIds"` is gone.
- **A fixture may hold fixtures** (MVR's `ChildList`; decisions §40 3): a folder is a fixture of a
  channel-less type with a position of its own. `Fixture Media 1` is the folder; a value at it
  (`Fixture Media 1 At 50`) goes down into every fixture below, one step; Move and Delete carry the
  children (the one nesting rule). A fixture has no sub-fixtures here — a fixture is a fixture;
  parts and elements are a later page.
- **A multipatch** mirrors its fixture at another address: `Fixture 201 Multipatch 1`. It has its
  own address, name and place; a value set on it goes to its fixture, so every copy follows.
  Copies are numbered per fixture, the next number only; a bare `Multipatch` in `Store` is each
  fixture's next copy (201 holding one gets its 2, 202 its 1). Elsewhere a copy is named by its
  number.

```lum
Store Fixture 201 Thru 206 At DMX 2.001 /FixtureType 3 /Mode "Standard"
Store Fixture 201 /FixtureType "LED Par" /Mode 2
Store Fixture 301 Thru 312 At DMX /FixtureType "Robe_Spiider.gdtf" /Mode "Mode 1"
Store Fixture 401 Thru 404 At DMX 3 /FixtureType 3 /Mode 1 /Every 20
Assign Fixture 201 Thru 206 At DMX 2.001
Assign Fixture 201 Thru 206 At DMX 1 /Every 20
Assign Fixture 201 Thru 203 At DMX 3.001 /KeepSpacing
Assign Fixture 201 Thru 204 At Fixture 301 Thru 304 DMX /Swap
Unassign Fixture 201 Thru 206 At DMX
Unassign Fixture 201 At DMX 2.001
Unassign Fixture 201 At Fixture 201 DMX 2
Info Fixture 3 DMX
Park Fixture 3 DMX
Park DMX 2
Fixture Houselights 4
Fixture Houselights 1 Thru 4 + Fixture 101
Fixture Media 7003
Fixture Generic 12 Thru 20
Fixture Media 1 At 50
Set Fixture Media 7003 "IDType" Generic
Set Fixture 849 "CID" 12
Store Fixture 201 Multipatch 1 Thru 2 At DMX 5.001
Store Fixture 201 Thru 206 Multipatch At DMX 5.001
Assign Fixture 201 Multipatch 1 At DMX 5.020
Fixture 201 Multipatch 1 At 50
Delete Fixture 201 Multipatch 1
Set Fixture 101 Thru 106 "FixtureType" "Two-wheel spot" /Mode "Mode 1" /Map 3 /Remember
```

### Identify

`Identify` makes fixtures or addresses show themselves on the rig, to find a unit or check a patch
(decisions §34 round 3, "Identify, Probe and Find hardened"). It is the station's own output:
never stored, never an Oops step, never replicated as show data; Setup and up.

- **One word, an object owning what it flashes** (a word means one thing): `Identify Fixture 104`
  runs its default verb, On; `Off Identify Fixture 104` ends part of it.
- **A target, or your selection.** `Identify Fixture 104` flashes the fixture (every break and
  every multipatch copy); a bare `Identify` takes your selection; `Identify DMX 1.150` probes an
  address (one slot, or a run with `Thru`) whether anything is patched there or not.
- **What it sends:** the fixture's GDTF default values, its dimmer in a sine from 15 % to 100 %
  every 2 s and its colour round the hue circle every 6 s (through its mixing; a wheel steps one
  slot a second). A probed address sends 38 → 255 in the same 2 s sine; `/As Fixture n` sends
  the run the way fixture n would (Find).
- **Above everything but Park:** programmers, playback, masters (a blackout too), Solo and DMX
  input don't touch an identified channel; a parked channel stays parked. A fixture parked by a
  conflict flashes on its own slots.
- **Each user has one.** A new `Identify` replaces yours; other users' go on. `Off Identify` ends
  yours, `Off Identify Fixture 104` ends part of it. Leaving the patch dialog (✕, Unclaim, the
  dialog closing) runs `Off Identify Fixture Thru; Off Identify DMX Thru`; logging out, a lost
  holder and 5 minutes end it too, and the station checks that none of yours is left on.
- **Find** is Identify stepping: the patch dialog probes the current fixture's footprint at its
  address `/As` that fixture, ← / → move the probe one footprint along, and *Patch here* writes the
  one `Assign … At DMX` line.

```lum
Identify Fixture 101 Thru 106
Identify
Identify DMX 1.150
Identify DMX 1.043 Thru 1.056 /As Fixture 104
Off Identify Fixture 104
Off Identify
Off Identify Fixture Thru; Off Identify DMX Thru
```

### Clone

`Clone` puts one fixture's data onto another, line by line: its values in cues, presets and the
programmer, and its places in groups and tags — never the patch. Everything is cloned unless the
line narrows it: attributes owner first under the source (`FeatureGroup`, `Attribute`), objects
with `If`. Source and destination are the same count.

- When the destination already has values, Clone asks once; `/Keep` keeps theirs where both
  have one, `/Merge` lets the source's win, `/Overwrite` removes theirs.
- Values keep their meaning (a colour stays the colour on the new type); `/KeepOutput` keeps the
  DMX output instead. `/NoDependencies` leaves the presets the values point at alone.
- Between different types an `AttributeMap` says which attribute feeds which: `/Map 3` names one,
  `/Remember` keeps it as that pair of types' map. Without one, the pair's remembered map is used,
  else the show's `AttributeRule`s, which may point a name pattern at an attribute (show-wide) or
  at a feature group.
- A type change (`Set Fixture … "FixtureType"`) takes the same options.

```lum
Clone Fixture 101 Thru 106 At Fixture 201 Thru 206
Clone Fixture 101 Thru 106 FeatureGroup "Position" + "Color" At Fixture 201 Thru 206
Clone Fixture 101 Attribute "Pan" + "Tilt" At Fixture 201
Clone Fixture 101 Thru 106 At Fixture 201 Thru 206 If Sequence 1 + 2 + 7
Clone Fixture 101 At Fixture 201 /Merge /NoDependencies
Clone Fixture 101 At Fixture 201 /Keep /KeepOutput /Map 3
List AttributeMap
Info AttributeMap 3
Store AttributeRule "Dim*2" Attribute "Dimmer"
Store AttributeRule "Pixel*" FeatureGroup "FX"
Move AttributeRule 15 At 3
```

**Reserved words.** `CanvasLayer` (a layer of a pixelmapper canvas, owner first: `Fixture 900
CanvasLayer 2`) comes with the pixelmapper; `Part` stays a cue part. A bare `Layer` is no word.

### Drives

A station's drives (decisions §38; 38a built 2026-10-19 — the object and its settings, `/Drive` on
LoadShow and SaveShow, `~` in the fence; 38b: `/Drive` on DeleteShow, `Delete Show`, `Import Show`,
Import FixtureType, `Store Fixture … /FixtureType "file.gdtf" /Drive`, Import / Export UserProfile,
Import Image; a bare import word asks with the files element): `local-library` is its own
library, a volume it finds is `usb1`, `ssda`, …, a folder you add `folder1`; network drives
(`smb1`, `ftp1`, `sftp1`) are LumaCore's own connections, planned. A drive is a whole volume; its
library is `LumaCore/library/` on it, created on the first write. Every file word takes `/Drive`
(absent = the local library); the file name stays relative to the word's library folder, as
always. `Drive` is station-local: a keyed row like `Interface`, its settings declared properties.

```lum
List Drive
Info Drive "usb1"
Set Drive "usb1" "Label" "Kingston blue"
Set Drive "ssdb" "Kind" "USB"
Set Drive "usb1" "Library" Yes
Store Drive "folder1" /Path "/srv/shows"
Delete Drive "usb1"
LoadShow "rig" /Drive "usb1"
SaveShow "rig" /Drive "usb1" /Overwrite
Import Mvr "Rig 2026 v3.mvr" /Drive "usb1"
Export UserProfile "Bob" /Drive "usb1"
```

`Eject Drive "usb1"` is planned (the one new verb of §38). In the fence `~` is the local library
and `~usb1` that drive's library folder; `Root.Drives` lists the drives and `Root.Drives.usb1` is
the whole volume (a hyphenated id is quoted in a dot path: `Root.Drives."local-library"`, or `~`).

### MVR import

An MVR file is imported with one line (decisions §34 "Import MVR hardened" 2, decided
2026-10-20; built but for the grid's review mode and the dialog): `Import Mvr "file"` reads the file against your Devices draft and opens
the review in the patch grid; nothing is written until the ticked changes are applied. The
settings — the match key, lost fixtures, rotation, the offsets (the rig offset in millimetres,
MVR's unit) and what the file may change per field group (None · Take · Offer) — are your
profile's **MVR import** preferences, declared
properties set like any other; the line's options override them for this import. `/Scope` takes
a target (an MVR layer, group, class or a tag); without it the whole patch is in scope.

```lum
Import Mvr "Rig 2026 v3.mvr"
Import Mvr "Rig 2026 v3.mvr" /Drive "usb1" /Scope MvrGroup 3
Import Mvr "Rig 2026 v3.mvr" /Scope Tag "Truss" /Match FixtureName /Lost Delete
Import Mvr "Rig 2026 v3.mvr" /FidOffset 100 /UniverseOffset 10 /X -4200 /Z 2100 /Rotate 90
Import Mvr "Rig 2026 v3.mvr" /Addresses Take /Names None /NewFixtures Offer /IgnoreRotation
Set UserProfile "MvrMatch" Auto
Set UserProfile "MvrAddresses" Offer
Set UserProfile "MvrLost" Mark
```

The review is the draft's (decision 3): its rows are numbered like the fixtures they match or
make, the grid's selection is the ticks. The rows are the target, as on every line.

```lum
List Mvr
Info Mvr
Import Mvr 101 + 102 + 201 /Apply
Import Mvr /Apply
Import Mvr 102 /Take "Address"
Import Mvr 102 /Offer "Address"
Import Mvr /Cancel
```

A new fixture whose MVR group the show no longer has asks at Apply, one ask per missing group
(decision 4); the answers are lines (`/Gone` — `/Missing` is List Asset's word), and the profile's
`MvrMissing` (Ask · Recreate · Parent · None) can settle it without a question.

```lum
Import Mvr 201 Thru 224 /Apply /Gone Recreate
Import Mvr 201 Thru 224 /Apply /Gone Parent /Group "Upstage truss"
Set UserProfile "MvrMissing" Ask
```

A folder in the file — a fixture holding fixtures — comes in as a fixture holding fixtures, in
the file's order (decisions §40 3, 6). A CustomId the file never typed (a console's export writes none)
gets the profile's `MvrIdType` (Generic by default; §40 9, 10); `/IdType` says it for one
import; a folder with no id at all gets the next free second id of that type; a fixture with no
number and no CustomId still takes `/FidOffset`.

```lum
Import Mvr "The International 2026.mvr" /IdType Media
Set UserProfile "MvrIdType" Generic
```

### Pool XML import

A console's pool export as XML (groups with their items and grid cells) is read by
`Import PoolXml` (decisions §40 13), from `library/poolxml/`: `Groups` makes one group per group
of the export in your selected DataPool (`/DataPool` picks another), members by fixture
id — a second-id item by its type —, their grid cells kept; a group of sub-fixture items only is
reported and skipped. Presets, sequences and macros follow when their objects exist.

```lum
Import PoolXml "ti2026-groups.xml" Groups
Import PoolXml "default.xml" Groups /Drive "usb1" /DataPool 2
```

## Values

A value is what is applied, stored or compared. Values are typed:

- **numbers** — `At 50`, `At -120`, `Cue 1.5`
- **relative** — `At + 10`, `At - 10`, `At * 2`, `At / 2` (the sign stands alone)
- **spread** — `21 Thru 30` over several rows
- **on / off** — `On`, `Off`, `Yes`, `No`, and `Toggle` where the property declares it
- **words** — a declared value of the property (`HTP`, `Summary`, `Full`, `Zero`, `Ask`), typed
  bare in any case and written out as declared; quoted is text, never a choice (`"high"` where a
  choice goes is E-SYN-014: type it without quotes)
- **text** — quoted (`"Front"`)
- **reads** — `Self.Name`, `Self.DataPool.Name` (resolved where the line starts)

```lum
Fixture 1 Thru 4 At Full
Attribute "Pan" At -120
At - 10
Set Sacn 1 Thru 3 + 5 "Rate" 44
Set Sacn 1 Thru 3 "Rate" 21 Thru 30
Set Sacn 1 "Rate" * 2
Set Sacn 1 "Sync" On
Set Universe 3 "MergeMode" HTP
Set Group 3 "Name" Self.Name
```

Fixture values are one `Value` type everywhere — normalized or physical; DMX bytes exist only
inside the render engine. Validation is specific: a property says what it expects, an enum
lists its values, a read-only property says why `Set` can't change it.

## Verbs

The verbs, by family (every one in the [command reference](../reference/commands.md)):

- **Selection** — `SelectFixtures` (fixtures: `Fixture 1`, `Group 5`, `+ Group 3`),
  `Select` (your selected DataPool, Sequence, Timecode, Window, Attribute …)
- **Values** — `At`, `Full`, `Zero`, `Release`, `Park`, `Unpark`
- **Assignment** — `Assign`, `Unassign` (one object into another)
- **Store / edit** — `Store`, `Update`, `Label`, `Edit`, `Copy`, `Cut`, `Paste`, `Move`, `Insert`,
  `Compact`, `Delete`, `Sort`, `Split`, `Clone` (one fixture's data onto another)
- **Recall** — `Call` brings a stored thing back here: `Call View 3`, `Call WindowPreset 2`
- **Settings** — `Set`, `Reset` (back to the declared defaults: an object, or one quoted
  property), `EditSetting`
- **Inspection** — `Info`, `List`, `Dump`, `Watch`
- **Playback** — `Go`, `Go-`, `Goto`, `Pause`, `Off`, `Top`, `Flash`, `Temp`, `Swap`
- **Modes** — `On`, `Off`, `Toggle` (`On Blind`)
- **Users and stations** — `Login`, `Consent`, `Logout`, `Join`, `Leave`, `Invite`, `Dismiss`
- **Ownership** — `Claim`, `Unclaim`
- **Runtime** — `SaveShow`, `LoadShow`, `Shutdown`, `Restart`

### Set and Info

`Set` changes a typed property and always names its target; a bare owner keyword means your
own one (your profile, this station). `Info` is its read side.

```lum
Set Fixture 3 "Name" "Spot SR"
Set UserProfile "ClearBuffer" On
Set UserProfile "MeowKey" Alt
Set UserProfile "LD" "ClearBuffer" On
Set Station "Name" "Luma01"
Set Station "Tower" "Priority" High
Set Station "Tower" + "FOH" Network "Rig" "Interface" "en1"
Set Root.ShowData.Stations."Tower".Networks."Rig" "Interface" "en1"
Set Network "Rig" "Interface" "en1"
Set Interface "en1" "HTTP" No
Set Root.ShowData.Protocols.DMXProtocols.Sacn.1 "Priority" 120
Set Self "Universe" + 1
Set Self "Maximized" Toggle
Set User "LD" "Password" Ask
Set Fixture 1 "MasterReact" Group
Set Sacn 1 "Network" Network 2
Set Group 1 "Paint" "#3c8f3c"
Set UserProfile Theme "Base" Night
Set UserProfile Theme "AccentHue" -30
Set UserProfile "LD" LockScreen "AutoLock" 10
Set UserProfile Theme Paint "Accent" "#9a7a08"
Set Sequence 2 Cue 3 "Fade" 2
Set Sequence 2 Cue 3 FeatureGroup "Position" "Fade" 4
Reset UserProfile Theme
Reset UserProfile Theme Paint "Accent"
Reset Sacn 1 Thru 4 "Rate"
Reset Sequence 2 Cue 3 FeatureGroup "Position"
Theme
Info Fixture 3
```

A settings group typed alone opens its settings (`Theme` = `EditSetting UserProfile Theme`).
`Default` is a fixture value (`Fixture 1 At Default`), never a property value: `Reset` puts a
property back.

A tree property is writable only when its object declares it; everything else is read-only
and `Set` says why. The writable properties per object are in the
[object reference](../reference/objects.md).

### Assign

`Assign` puts one object into another — the thing first, where it goes after `At`, so the order
is the data flow: universes onto a line send (`Mode Output`), a line onto universes receives
(`Mode Input`); the protocol universe the run starts at is `/Universe`:

```lum
Assign Universe 1 At Sacn 1 /Universe 1
Assign Sacn 2 At Universe 5 /Universe 7
Unassign Universe 1 At Sacn 1
Unassign Sacn 2
Assign View 3 At ViewButton 5
Assign Attribute "Gobo1Pos" At FeatureGroup "Gobo"
Assign Attribute "Gobo1Spin" + "ColorWheelSpin" At FeatureGroup "FX" /WithSubs
Assign UserProfile Attribute "Gobo1" At UserProfile FeatureGroup "Beam"
Assign FeatureGroup "Strobe" At FeatureGroup "Shutter"
Assign Fixture 101 Thru 104 At MvrGroup 4
Unassign Fixture 101 At MvrGroup 4
Unassign Fixture 101 At MvrGroup
Unassign Tag "Right" At Fixture 3
```

**Unassign always names the kind it takes away.** A bare kind after `At` is whichever of that kind
the object is assigned to: `Unassign Fixture 101 At MvrGroup` leaves its group, `… At DMX` loses
every address. A fixture alone (`Unassign Fixture 101`) is refused and names the forms.

One-shot changes are verbs of their own:

```lum
Sort FeatureGroup "Beam" /By Family
Split FeatureGroup "Effects" /By Family
Call AttributeTemplate "LumaCore FX" At ShowData
Master Group 4 At 50 Fade 5
```

### Store

`Store` captures or creates show content; Cue is its default object (`Store 2` =
`Store Cue 2`, `Store` alone = the next cue of your selected sequence).

```lum
Store Group 1 "Front"
Store Sequence 4 Cue 2
Store 2
Store
Store Group 4 /Merge
Store Window 5 /Type Terminal /Overwrite
Store Fixture 201 Thru 206 At DMX 2.001 /FixtureType 3 /Mode "Standard"
```

An object that has a place takes it with `At` when it is stored (a fixture its address, see
[Patching](#patching)); another object's `Store … At` is refused.

### Presets

A DataPool has one preset pool, numbered like any pool (`Preset 12`). `Store Preset` keeps every
active value of your programmer, whatever its feature group; one value for every fixture is
Universal, else it is Selective (per fixture). `At Preset` puts the values on your selection and
the values remember the preset; `Call Preset` is the same; a bare `Preset` selects its fixtures.
Tags sort presets, and the Presets window shows the ones that touch the feature group you are on.

```lum
Store Preset 12 "Deep red"
Store Preset
Store Preset 12 /Merge
At Preset 12
Fixture 101 Thru 104 At Preset 12
Call Preset 12
Preset 12
Assign Tag "Song 3" At Preset 12
```

### Copy, Cut, Paste, Move, Insert, Compact

Every pool works the same way (decisions §35): one door does the work, so the rules don't
change from object to object.

- **Copy** makes new objects (new GUIDs; links inside are kept). `Copy … At …` copies at once; a
  bare `Copy Group 1 Thru 5` puts them on your clipboard.
- **Cut** puts objects on your clipboard to be moved: `Cut Group 1`.
- **Paste** finishes either: the target is where they go (`Paste Group 10`, `Paste DataPool 2
  Group 10`), the spacing kept. `Paste 10` takes your default object (`Group>`). A bare `Paste` of
  a copy goes to the next free numbers; of a cut it asks where. The clipboard holds the objects
  themselves, not a snapshot: Paste takes them as they are then, a deleted one is skipped.
- **Move** keeps each object's GUID. Onto a taken number the two **swap** (`Move Group 3 At 7`:
  3 ↔ 7; a range slot by slot). `/Skip` jumps over taken numbers instead and moves nothing else
  (`Move Fixture 101 Thru 103 At 107 /Skip` with 108 taken: 107, 109, 110). The new numbers follow
  the typed order (`Move Fixture 105 + 101 + 103 At 107`: 105 → 107, …) or a column's with `/By`
  (`/By Address`, `/By Name`: consecutive from the destination, in that order). Between owners of the same kind it is still the same object;
  another kind is refused.
- **One nesting rule.** Only tree kinds nest — Tag, and MvrLayer › MvrGroup › MvrGroup; every
  other pool stays flat and numbered (sort anything privately with tags). `Move <node> At <node>`
  makes it a child of the destination and **always asks** first ("MvrGroup 3 "Truss 2" becomes a
  child of MvrGroup 2 "Truss 1" (2 groups, 88 fixtures)?" Nest / Cancel); `/NoConfirmation`
  answers it. `Move <node> At <number>` still renumbers. A node never goes under itself or its own
  children, and a layer is the top of its tree. Deleting a node that holds something asks once —
  keep its children (they move to its parent) or delete them too; `/KeepChildren` /
  `/WithChildren` answer it (`Delete MvrGroup 3 /KeepChildren`, `Delete Tag 3 /WithChildren`).
- **Insert** is a move with shift: `Insert Group 5 At 9` puts 5 on 9 and moves the old 9, 10 …
  up to the first free number.
- **Compact** renumbers a pool, holes closed, GUIDs kept: the whole pool, some rows, or in an
  order (`/By Name`, `/By Address`). `Sort` orders an object's contents instead.
- Onto an object that exists, Store, Copy and Paste ask once — Merge (where the object can merge)
  / Overwrite / Remove (Store) / Cancel; `/Merge`, `/Overwrite`, `/Remove` answer it; a stored key
  line has to. Overwrite keeps the existing object's GUID.
- The next free number is the first unused one; cues append (after the last cue).
- Delete asks once and names what uses the object; links left behind are kept and listed.

```lum
Copy Group 1 At 5
Copy Group 1 Thru 5
Cut Group 1
Paste Group 10
Paste DataPool 2 Group 10
Paste
Move Group 3 At 7
Move Fixture 101 Thru 103 At 107 /Skip
Move Fixture 101 Thru 140 At 201 /By Address
Move DataPool 1 Group 3 At DataPool 2 Group 8
Insert Group 5 At 9
Compact Group
Compact Group 10 Thru 50
Compact Fixture /By Address
Copy Group 1 At 5 /Overwrite
Move MvrGroup 3 At MvrGroup 2 /NoConfirmation
Delete MvrGroup 3 /KeepChildren
Delete MvrLayer 2 /WithChildren
```

### Hold verbs

```lum
Top On Sequence 3
Top Off Sequence 3
Flash On Sequence 1
Flash Off Sequence 1
```

### Playback and the Master Section

You always have one selected sequence or none: `Sequence 3`, or `Executor 201` (its assigned
sequence). Your Master Section — Go+, Go−, Pause, the master faders — acts on it; an empty
executor leaves it empty, and a bare `Go` then says so.

```lum
Sequence 3
Go
Go Sequence 2
Go-
Go- Sequence 2
Goto Sequence 2 Cue 3
Executor 201
Off Sequence 1
```

### Login and the Meow License

Every login asks the user to accept the Meow License before it is finished (decisions 0290):
the station's dialog runs `Consent` ("I consent") or `Logout` ("Not me"). Until then a line
other than `Consent`, `Login` or `Logout` is refused (E-USER-010). `Consent` takes nothing.

```lum
Login "LD"
Consent
Logout
```

### If

`If` is planned (it replies "planned" until built); it will be a condition on the line:

```lum
If Group 5
Group 3 If Group 5
Delete Cue 3 If Fixture 4
Store If Group 5 EndIf Preset 1
```

## Options

A `/Name` word changes *how* a line runs, never *what* it does. A value follows as the next
word or quoted text; whether an option takes one is declared on it.

```lum
LoadShow "Festival 26" /NoSave /NoConfirmation
Store Group 4 /Merge
List Fixture /Verbose
Menu "Backup" /Tab "History"
Import Patch + Users /From "Festival"
Bind "F5" `List Fixture` /Description "Lists all fixtures"
Watch `List Fixture` /Every 500
LoadShow "Festival 26" /Drive "usb1"
```

`/Verbose` and `/Debug` override your `Verbosity` for one line; `/NoConfirmation` answers a
question up front; `/NoOops` files no undo step.

`/R` (Recursive) works with every object and command that reaches into a tree: without it a line
reaches the current level, with it everything below too (`List … /r`, `Off Tag "All" /R`).

## Context

**Your default object.** A counted object typed alone becomes your default object; the prompt
shows it, and a bare number or name names one of it:

```lum
Group
3 At 50
Fixture
1 Thru 4 At 50
```

**Your selection.** A later value works on the current selection; a plain select adds until a
value is set, then the next plain select replaces it; `+` always adds, `-` takes out.

```lum
Fixture 1 Thru 4
At 50
+ Group 3
- Fixture 2
```

**Your tool.** What a tap does in every window is your tool: `Tool 1 "Run"` (a tap runs the
object, a hold goes full screen) or `Tool 2 "Edit"` (a tap picks, a drag moves, a drag on empty
space lassos, a hold opens the menu). It is yours on every screen and window; the two tools are
built in, so they can't be stored, deleted or renamed. The name is quoted (a bare `Edit` is the
verb). `Toggle Tool` flips between them (Insert, by default); Esc goes back one layer at a time
until you are in your default tool:

```lum
Tool "Edit"
Tool 1
Toggle Tool
Info Tool
Set UserProfile "DefaultTool" Tool "Edit"
```

**Statements.** `;` separates statements; each runs in order and is its own Oops step:

```lum
Fixture 1 Thru 10 ; At 50
Store Group 3 ; Oops
```

Context is always visible: the prompt shows your default object and where you are, the path
bar your claims and selection count.

## Typing help

- **Fuzzy, Raycast-like:** type `str gro 3` and the ghost shows `Store Group 3`; Enter runs the
  line shown. An ambiguous word (`cu 2`: Cue 2 or CurrentUser 2) shows the choices and Enter
  waits for one. Your profile's `Suggestions Off` means exact input.
- **Completion, hints, help:** Tab completes from the same grammar the parser uses; `Help
  <word>` and the generated reference come from the same signatures.

## Aliases

An alias is yours: a word for a line, kept on your profile, typed with `!`.

```lum
Store Alias "St" `Store Cue 1`
Store Alias "sg" `Store Group`
Store Alias "au" `Set Self "AutoScroll" Toggle`
List Alias
Delete Alias "sg"
```

`!St` runs `Store Cue 1`; `!sg 3 "Front"` runs `Store Group 3 "Front"` (the words you type are
appended); `!St /Merge` runs `Store Cue 1 /Merge`. Aliases work only in a line you type on a
command line — never in keys, macros, cue commands or machine lines — and what runs is the full
line. New profiles start with `!hiss` and `!unmeow` (Oops) and `!turnoff` (Shutdown).

## The tree-shell fence

The fence's words take Unix-style arguments and tree paths:

A path is relative to where you are; `Root.` makes it absolute from anywhere. A part is the
node's own name (`DataPools`, `Groups` — plural collections), a number or quoted text. Typing is
fuzzy here too: inside ShowData, `cd datapool.1.group.1` shows the ghost `cd DataPools.1.Groups.1`
and Enter runs that; a stored line must name the nodes exactly.

```lum
cd ShowData
cd DataPools.1.Groups.1
cd ..
cd 3
cd /
cd Root.ShowData.DataPools.2
ls -r
cat Root.ShowData.Show
find "Front" --type Group
tree -d 2
head -n 5
diff Root.ShowData.DataPools.1 Root.ShowData.DataPools.2 --all
mv 3 7
cp DataPools.1.Groups.3 DataPools.2.Groups.8
rm 3
cd ~usb1
ls ~
cp "Rig 2026 v4 draft.mvr" ~usb1.mvr
```

`mv`, `cp` and `rm` go through the same doors as the Lum words (decisions §35): in the show
tree they are Move (swap), Copy and Delete with their asks and Oops; under `Root.Drives` they
work on files — a taken name asks Overwrite / Keep both / Cancel, `rm` moves the file to that
drive's `.trash` (30 days) and the reply says how to put it back; disk changes aren't Oops
steps. Between `Root.Drives` and the show tree they are refused: that is `Import` / `Export`.

Fence lines are allowed in anything a user writes — typed lines, macros, keys, cue commands —
and never in LumaCore's own code (its templates, registry lines and generated lines are pure
Lum; a test checks every line the code emits).

## Errors

Every error has a code after its message; `Info Error "E-SYN-001"` explains one, `List Error`
lists them. The parser refuses a line before it runs, with the word marked:

```text
Store Grou 3
      ^^^^ unknown word 'Grou' — did you mean Group? [E-SYN-001]
```

| Code | The parser refuses |
|---|---|
| E-SYN-001 | an unknown word (with "did you mean") |
| E-SYN-002 | a missing slot (`Copy Group 1 At`) |
| E-SYN-003 | a word that is neither a declared child nor a slot of the verb, or a dotted keyword path (`DataPool.2.Group.3`) |
| E-SYN-004 | a child before its owner (`Goto Cue 3 Sequence 2`, `Group 1 DataPool 1`) |
| E-SYN-005 | a bare property name (`Set Fixture 3 Name "x"`) |
| E-SYN-006 | an unknown option, or a value on an option that takes none |
| E-SYN-007 | an operator glued inside a list (`Group 1+3`) |
| E-SYN-008 | a bare number, alias or fuzzy word in a stored line |
| E-SYN-009 | a fence word in a line LumaCore's code emits |
| E-SYN-010 | an ambiguous word (fuzzy typing found several) |
| E-SYN-011 | an unterminated quote |
| E-SYN-012 | a wrong number of id parts (`Fixture 1.2.3`) |
| E-SYN-013 | a hold verb without `On` / `Off` |
| E-SYN-014 | a value the slot doesn't take (`"Sync" 1`; quoted text where a line in backticks goes) |

Errors that need the show stay with their area (`E-CMD-007` no such object, `E-CMD-008` value
out of range, `E-RIGHTS-…`).

```lum-error E-SYN-001
Store Grou 3
```

```lum-error E-SYN-004
Group 1 DataPool 1
```

```lum-error E-SYN-004
Goto Cue 3 Sequence 2
```

```lum-error E-SYN-005
Set Fixture 3 Name "Spot SR"
```

```lum-error E-SYN-007
Group 1+3
```

```lum-error E-SYN-012
Fixture 1.2.3
```

```lum-error E-SYN-003
DataPool.2.Group.3
```

```lum-error E-SYN-013
Flash Sequence 1
```

```lum-error E-SYN-014
Set Sacn 1 "Sync" 1
```

```lum-error E-SYN-014
Bind "F5" "List Fixture"
```

## LumaSession language

Stations work together in a LumaSession (decisions §10, §11, §18):

```lum
Join LumaSession "Festival"
Join LumaSession "10.0.0.5:47653"
Leave LumaSession
List LumaSession
Invite Station "FOH-2"
Dismiss Station "FOH-2"
Set Station "Priority" 100
List Client
List Shell
```

One station is the master at a time and runs every show change; station priority helps elect
a master, it never means two masters.

## Ownership language

Ownership is part of normal multi-user work: `Claim` and `Unclaim` on any object with a GUID
and on the areas `Root.ShowData.Devices`, `Root.ShowData`, `Root.ShowData.DataPools`,
`Root.ShowData.Protocols`, … — an area is named by its tree path (the areas are not words).

```lum
Claim
Claim Root.ShowData.Devices
Claim Fixture 3
Claim Group 1 + Fixture 3
Unclaim Root.ShowData.Devices /Keep
List Claims
```

- scope is where commands apply, not ownership; `Claim` alone claims the current scoped object
- claims block changes, not `List`, `Info` or `Watch`; ownership follows the user
- the `Programmer` is always its user's; selecting claims nothing; `Claim Fixture 3` locks it
- claiming `Root.ShowData.Devices` (or `enter patch`) edits a draft of every device kind; its
  `Unclaim` asks Keep / Discard / Cancel; meanwhile others edit the live columns (a live patch)
- `Edit Group 1` claims the group until you leave the editor; `cd` never claims
- claiming another user's object asks them (10 s); an Admin can `/Force`

## Variables

Three variable tiers are decided (§5): `GlobalVar` (built: `List GlobalVar`), `ShowVar` and
`UserVar` (planned). `$name` is reserved for them in command lines.

## Background

- **Common console conventions** are loose inspiration, not the rule (§33): Lum keeps their statement shape, `Thru`,
  `+` / `-`, owner-first nesting, `At`, `/Options`, quoted property names, hold verbs and `If`,
  and drops its irregular parts — a different default per object written nowhere, prefix
  expansion of names, glued operators read silently, a dot as both decimal and hierarchy,
  `Thru` reusing the last range. Where a rule follows a convention, §33 says why.
- **Why one page.** This file is the one place the language is specified; the reference is
  generated so words never drift from the code.
