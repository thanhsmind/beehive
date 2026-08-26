---
type: bee.pattern
title: A truncated identifier is not an identifier
description: A truncated identifier is not an identifier
tags: [failure, stores, identifiers, data-loss]
timestamp: 2026-08-26
bee:
  id: pattern-20260826-a-truncated-identifier-is-not-an-identifier
  lifecycle: active
  areas: [human-mailbox, workflow-state]
  sources: ["human-mailbox cell hm-1 (the slug collision found while building the letter store, 2026-08-25)", "docs/history/human-mailbox/CONTEXT.md D11 (one letter maps to one run)"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "wherever an id is shortened to make a filename or a key, ask what happens when two ids share the surviving prefix — if the answer is 'they collide', the shortening needs a hash tail, not a longer prefix"
---

# A truncated identifier is not an identifier

A store keyed one record per run needed a readable filename, so it built one
from a timestamp and a shortened run id. Readable, sortable, obvious.

Two run ids sharing their first twenty characters produce the **same
filename**. The second run overwrites the first record and both runs' append
logs merge into one file. Nothing errors. The result is a record that looks
complete and describes two different runs as though they were one — while the
rule the store was built around says one record maps to one run, never two.

The shortening was correct about what it was for. It was wrong about what it
had become: the moment a truncated id decides a *path*, it is no longer a label
for humans, it is a **key**, and a key that can collide is not a key.

**The rule:** wherever an identifier is shortened to build a filename, a
directory name, or a map key, ask what happens when two identifiers share the
surviving prefix. If the answer is "they collide", the fix is a hash tail on
the truncated form — not a longer prefix, which only moves the collision
further out.

**Why a longer prefix is not the fix.** It converts a certainty into a
probability and hides the failure behind rarity. The collision then arrives on
the one night two long ids happen to agree, in production, silently, and
looking exactly like data that was never written.

**The tell:** any expression of the form `&id[..n]`, `slug[..20]`, "first eight
characters of the hash", or a `_capped(x, n)` helper, on a value that then
becomes part of a path. Shortening for a log line is fine — nothing downstream
resolves it. Shortening for a *name* is the moment to add the tail.

**A related trap in the same family:** two different truncations of the same id
are not interchangeable. In this store, entry files and letter files each
shortened the run id their own way, so neither name could be turned back into
the other. Code that needs to relate them must slug **forward** from the full
id, which is exact, rather than attempt to invert a slug, which is a guess.
