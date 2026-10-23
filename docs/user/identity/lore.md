---
title: Lore
category: docs
---
# Lore

Luma is the cat of LumaCore.
Luma is a short-haired black cat.

```text
        .          *  .                    *       .
    *                  _.._
              *      .' .-'`
   .               /  /
                  |  |        *
    *   .         \  \
                   '._'._ `-'

      /\_/\
     ( o.o )
      > ^ <
     /|   |\
    (_|   |_)
```

Luma is curious, quick, and likes to climb.
Luma likes to roam around in the dark where no lighting fixture shines.
Luma prefers sunbathing alone in the moonlight under the stars.
Luma's favorite colors are the colors of the banner:

- moon-purple `#af5fff`
- deep console blue `#5f87d7`
- star-cyan `#87d7ff`
- quiet night grey `#5f5f87`

That is why the shell world of LumaCore is called `TreeShell`:

- Luma climbs the tree
- Luma moves through branches and objects
- Luma explores structure instead of hiding it

The technical ideas borrow from that identity:

- `Luma`
  - the internal cursor that moves through the system
- `Lum`
  - the language spoken in the shell
- `TreeShell`
  - the tree world Luma climbs through

So the short version is:

Luma is the cat.
Luma likes to climb the TreeShell.

## Lore to meaning

`Luma climbs the TreeShell.`

The system is a tree of objects that can be navigated, inspected and worked
through directly ([tree-shell.md](../terminal/tree-shell.md)). The tree stays
aligned with the object tree, and operator nouns stay aligned with `Lum`.

`Luma speaks Lum.`

Users act on the system through `Lum`, the object-based command language
([lum.md](../syntax/lum.md)).

`Luma acts where it stands.`

Context matters. Selection, current location and current scope all affect how a
sentence behaves.

`Luma roams in the dark where no fixture shines.`

LumaCore is not only about visible fixture output. It is also about exploration,
inspection, structure, runtime state and the places behind the light — datapool
objects, runtime state, health, the disk — all visible instead of hidden behind
UI-only ideas.

## Cat words

The cat shows up in the vocabulary:

- `Meow` — the execute key and the modifier key
- `Oops` — undo, with the aliases `hiss` and `unmeow`
- `HissMe` — a deliberate crash report, internal only
- **Meownitor** — the System Monitor in a terminal (`lumacore-cli --meownitor`)
- **Pawtern** — the planned selection-pattern system

## The banner

The banner is the one canonical cat mark of LumaCore. The `Banner` command and
the server's startup print it (source: `crates/commands/objects/lumacore-cmd-station/src/splash.rs`):

```text
        .          *  .                    *       .
    *                  _.._
              *      .' .-'`
   .                 /  /
                     |  |        *
    *     .           \  \
                      '._'._ `-'
      /\_/\
     ( o.o )     _                          ___
      > ^ <     | |   _   _ _ __ ___   __ / __|___  _ __ ___
     /|   |\    | |  | | | | '_ ` _ \ / _` | |/ _ \| '__/ _ \
    (_|   |_)   | |__| |_| | | | | | | (_| | | (_) | | |  __/
                |_____|\__,_|_| |_| |_|\__,_|\_\___/|_|  \___|
   ───────────────────────────────────────────────────────────
    v0.1.0                    terminal-first DMX console
```

In color: the cat in moon-purple, the moon and wordmark in console blue, the
stars in star-cyan, the dots and rule in night grey.

The cat mark alone, for anywhere a small Luma is needed:

```text
  /\_/\
 ( o.o )
  > ^ <
 /|   |\
(_|   |_)
```

Any file that needs the banner or the cat refers here. The old separate banner
page (`docs/archive/dev/firmware/banner.md`) is archived.
