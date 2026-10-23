---
title: Tree Shell
category: docs
---
# Tree shell

Everything in LumaCore lives in one object tree (`Root`). The **tree shell**
(TreeShell) is a set of Unix-style words to walk it and look at it, at the same
prompt as Lum. Luma climbs the tree.

Every tree-shell word is **read-only**: it changes nothing, needs only Guest
rights and runs in your own shell. The syntax of each word is in the reference,
category [Unix-tree Navigation](../reference/commands.md#unix-tree-navigation).

The tree itself is described in [object-tree.md](../object-tree/object-tree.md).

## Paths

- Dot paths: `ShowData.Devices.Dmx.Patch.Fixtures`, `Root.ShowData.UserProfiles.1`.
- Relative to where you are, or absolute from `Root`.
- `..` is one level up, `/` is the root. `pwd` prints the path with slashes
  (`/ShowData/Devices/Dmx/Patch`).
- Names match in any case; Tab completes paths.

## Moving around

| Word | Does |
|---|---|
| `pwd` | where you are |
| `cd <path>` | go there — no claim, no scope change |
| `cd ..` / `cd /` | up one level / to the root |
| `ls [<path>]` | the children here (or of a path); `ls -r` recurses |
| `tree [<path>] [-d N]` | a recursive listing with counts, N levels deep |

Alt+Left and Alt+Right walk back and forward through your `cd` history. In NAV
shortcut mode the arrow keys browse the tree (see
[cli-terminal.md](cli-terminal.md#shortcut-modes)).

`cd` vs `Enter`: `cd` only moves. `Enter` moves **and** claims a claimable node
(Patch, ShowData, …) and sets your scope; `Exit` releases it. Use `cd` to look,
`Enter` (or `Edit`) to work.

## Looking at nodes

| Word | Shows |
|---|---|
| `cat [<path>]` | a node as pretty JSON |
| `head [<path>] [-n N]` | the first N entries (default 10) |
| `tail [<path>] [-n N]` | the last N entries |
| `stat [<path>]` | metadata: type, ID, GUID, name, path, children, size |
| `wc [<path>]` | how many direct children |
| `du [<path>]` | how many objects of each type below a path |
| `diff <path1> <path2> [--all]` | two objects field by field |

## Searching

| Word | Finds |
|---|---|
| `find <pattern> [--type TYPE]` | nodes by name (substring or glob, any case) |
| `grep <pattern> [<path>]` | objects whose values contain the text (e.g. an IP) |

```text
find "Main*"
find Spot --type Fixture
grep "192.168"
grep "Admin" ShowData.UserProfiles
```

## About you

`Info User` shows your user, rights level and claims; `List Claims` what you hold
(the tree shell's `whoami`, `env`, `history`, `root` and `rm` went with decisions §33
decision 18 — `cd /` is the root, `Delete` removes).

## Root.Drives — the disk

`Root.Drives` is this machine's file system, from `/` (or `C:\` on Windows). The
tree-shell words can walk it: `ls Root.Drives.Users`, `stat` on a file. A file
shows its name, size, type and modified time — never its content. Nothing writes
through Drives; saving and loading shows go through their own words (`SaveShow`,
`LoadShow`, `Import`, …).

Walking the whole tree (`tree`, `du`, `find`) skips Drives so it never crawls the
disk.

Only tree-shell words enter `Root.Drives` (every word declares its reach: Show,
TreeShell or Drive); `List`, `Watch`, `Enter` and the Quicksheet never walk into it
unless the path names it.

## The Quicksheet

Tab on an empty line opens the Quicksheet: `ls` of where you are plus every
command word by category. `help <word>` gives the details.

## Background

The tree shell exists because LumaCore keeps nothing hidden behind the UI: show
data, runtime state, stations, health and the disk are all nodes you can look
at. "Luma roams in the dark where no fixture shines" — see
[lore.md](../identity/lore.md).
