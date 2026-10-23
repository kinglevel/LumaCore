---
id: 0107
title: The tree shell is a fenced Unix sub-grammar
area: syntax
status: built
date: 2026-10-15
built_in: lum part 3a
supersedes: [archive §33 L7128–7129 "root rm whoami", archive §33 L7945–7946 "ls /r"]
superseded_by: —
confirmed_by: [E-SYN-002, E-SYN-003, E-SYN-009, `commands::typed::FenceCommand`, `CommandDef::fence`, lum.md's `fence` rule]
source: [archive §33 L7124–7133 (item 18), archive §33 L7205–7209 (item 30), archive §33 L7377–7387 (3a)]
---
# The tree shell is a fenced Unix sub-grammar

## Context
Browsing the tree wants Unix habits (`ls -r`, `tree -d 2`, `..`, `/` = root) that clash with Lum's `/option` and `-` operator. Instead of one mixed grammar the word decides. The old `root`, `rm` (= Delete), `whoami`, `env` and the `history` stub went; `ls /r` became `ls -r`.

## Decision
A statement that starts with a fence word parses by the fence's own small grammar (lum.md's `fence` rule; its `FENCE-WORD` list: `cd`, `pwd`, `ls`, `cat`, `tree`, `find`, `grep`, `head`, `tail`, `wc`, `du`, `stat`, `diff`, and the `mv` / `cp` / `rm` lum.md names since §35, which go through the Lum doors Move / Copy / Delete); every other statement is Lum. Each runs as `fenced::<C>` on a `lumacore_syntax::FenceLine`: positional paths and texts plus flags declared as Unix flags (`ls -r`, `find "Front" --type Group`). A path is dotted (see tree-paths-typed-node); `cd .` is refused (`TREE-PATH` has `..` and `/`, not `.`); a usage error is E-SYN-002, a slash path E-SYN-003.

Fence lines are allowed in anything a user writes — typed lines, macros, keys, cue commands — and never in LumaCore's own code: Rust handlers, registry lines, the kit's templates and menu definitions, generated lines and the demo builder emit pure Lum (E-SYN-009; a test checks every line the code emits). So the NAV keys' `cd ..` / `cd /` are profile key data from a shipped default-profile data file, not strings in Rust or JS.

## Consequences
A new browsing word is a fence word with declared flags, never a Lum word with Unix arguments. The default-profile data file for NAV keys and the cat aliases is decided; the archive does not confirm it built.
