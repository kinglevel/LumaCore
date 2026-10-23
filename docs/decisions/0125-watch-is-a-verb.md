---
id: 0125
title: Watching is the Watch verb only
area: commands
status: decided
date: 2026-10-15
built_in: —
supersedes: [archive §33 L7200–7204 "global /Watch option"]
superseded_by: —
confirmed_by: none
source: [archive §33 L7200–7204, archive §33 L7924–7942]
---
# Watching is the Watch verb only

## Context
A watch could be started three ways: the global `/Watch` option on any line (`Clock /Watch`, `List … /Watch`), the object form `Watch Clock`, and `Watch` as an object with its own `Call`. The Lum syntax round decided an option never says what a line does (see quoting-values-options), so an action word has to carry the watching.

## Decision
Watching is one verb: `Watch "List Fixture" /Every 500` — the watched line quoted as a value — and `Unwatch 2` ends it. The global `/Watch` option goes, and so does the object form `Watch Clock` (it becomes `Watch "Info Clock"`). The tree word's depth stays inside the Unix fence (`tree -d 2`); the global `/Debug` keeps its meaning everywhere else.

## Consequences
Not built. Today `Watch` is still an object (`List Watch`, on `lumacore-cmd-station`) whose default verb is `Call` (it starts a watch), and the station's `Hook::Intercept` still takes `<line> /Watch`; both go when this record is built. The open question to settle first, at 3e: a verb `Watch` beside an object `Watch` breaks one name per word (see one-registry-one-namespace) — either the object is renamed or the verb takes another name. Until then no new line, key or doc introduces a `/Watch` option.
