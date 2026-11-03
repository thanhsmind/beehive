---
type: bee.pattern
title: Two stores both read as "the human inbox" — a not-found answer needs a knowledge search first
description: Supervisor mailbox/WakeReport (.bee/supervisor/) and human-mailbox letters (.bee/human-mailbox/) both read as "the inbox"; asked whether a summary was sent, a session checked only one store and answered "nothing was sent" while a filed letter existed
tags: [human-mailbox, herding-supervisor, naming, consultation]
timestamp: 2026-08-30
bee:
  id: pattern-20260830-two-inboxes-naming-collision
  lifecycle: active
  areas: [human-mailbox]
  sources: ["session observation, 2026-08-30 — 'was a summary sent to my inbox' answered from the supervisor store alone, missing a filed close letter in the human-mailbox store"]
  polarity: pitfall
  evidence: observed
---

# Two stores both read as "the human inbox"

Two independent stores both plausibly answer to "the human inbox": the
supervisor mailbox / WakeReport store (`.bee/supervisor/`) and the
human-mailbox letter store (`.bee/human-mailbox/`). Asked "was a summary
sent to my inbox," a session checked only the supervisor store, answered
"nothing was sent" — while a filed close letter existed in the other
store the whole time. It also did not run `bee knowledge search` on the
symptom before answering.

**The lesson.** An inbox-shaped question must check BOTH stores. More
generally: a not-found answer about a bee mechanism requires a knowledge
search before saying "it does not exist" — the two-store naming collision
is exactly the kind of thing a search would have surfaced.

## Status

Recorded as a naming/consultation lesson — no letters or skill content
changed; the fix is in how a session answers, not in the stores themselves.
