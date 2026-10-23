# lumacore-cmd-treeshell

The tree shell's words: the fenced Unix commands on the object tree and the drives, and Enter /
Exit.

## Purpose

The tree shell (decisions §33 decision 18) is the Unix-like side of the command line: `cd`, `ls`,
`pwd`, `cat`, `head`, `tail`, `stat`, `tree`, `find`, `grep`, `du`, `wc`, `diff`, `cp`, `mv`, `rm`
are fence words — each line is read as its own Unix-like sentence (`ls -r Root.ShowData`), never
as Lum — and `Enter` / `Exit` move into a node and claim it, or release it. On the show tree `mv`
/ `cp` / `rm` are the Lum line they mean (`mv 3 7` in DataPool 1's Groups is `Move DataPool 1
Group 3 At 7`), written out by its one `Hook::Rewrite` before rights, forwarding, the journal and
Oops; under `Root.Drives` they work on this station's files through the drive door, `rm` moving a
file to the drive's `.trash` (§35 19). Walking the tree — the vfs, navigation, Enter's claim, the
`ls` listing the help's quicksheet shows too — stays in the kernel (§34 25.1); this crate holds
the words and what they print (§34 25.7).

## Owns / Doesn't own

- **Owns:** the rows (`rows`: `ROWS`, `HOOKS`), the fence words and their Unix flags (`words`),
  `cat` / `head` / `tail` / `stat` (`read`), `tree` (`tree`), `find` / `grep` (`search`), `du` /
  `wc` (`count`), `diff` (`diff`), the printing helpers (`print`), `mv` / `cp` / `rm` and the drive
  trash (`door`), Exit.
- **Doesn't own:** the tree's walk (`lumacore-objects`: `vfs`, `navigation`,
  `commands::navigate` — Enter, cd, ls), the object door's verbs (the kernel's `door`), the disk
  (`lumacore-luma` `drive`).

## Public surface

- `register` — the tree shell's rows and hook, after the kernel's.
- `ROWS`, `OBJECTS` (none), `HOOKS`.

## Rules

- A tree-shell line is a fence line, never Lum. [test a_fence_words_path_is_not_a_flag]
- `mv` / `cp` / `rm` on the show tree run as the Lum line they mean. [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-syntax, lumacore-wire, lumacore-luma, lumacore-objects.
- **Used by:** lumacore-terminal (the shells register it), lumacore-sweep, lumacore-tidy;
  lumacore-suggest's tests.

## Decisions

- [§33 18](../../../../docs/archive/decisions-2026-10.md) — the tree shell, a fenced Unix sub-grammar
- [§35 19](../../../../docs/archive/decisions-2026-10.md) — `mv`, `cp`, `rm`
- [§34 25.7](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-cmd-treeshell` — the words line in / reply out, mv / cp / rm. The golden
  record of every path is `tests/catalog`'s (it walks every object's tree).
