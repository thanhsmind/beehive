---
type: bee.area
title: Hook Runtime — the opt-in doc-viewer prefix in agent-facing briefings
description: "The opt-in configuration that turns every doc reference an assistant writes into a clickable viewer URL: the two briefing surfaces that carry the prefix, why the second one exists, what an unset key changes (nothing, silently) and what a half-set one does (warns, loudly), and why the prefix stops at agent prose instead of rewriting command output."
timestamp: 2026-08-05
bee:
  id: hook-runtime-doc-viewer-links-in-agent-briefings
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["4205835b (doc-viewer links: an opt-in viewer prefix carried into the startup briefing and the post-compaction capsule, joined never encoded, half-set warns — cells dvl-1/dvl-2, 2026-08-05)"]
  sources: ["doc-viewer-links cell dvl-1 (one shared reader + both briefing injectors; trace .bee/cells/archive/doc-viewer-links/dvl-1.json, capped 2026-08-05)", "doc-viewer-links cell dvl-2 (the contract rule and the configuration reference; trace .bee/cells/archive/doc-viewer-links/dvl-2.json, capped 2026-08-05)", docs/config-reference.md (the doc_viewer key and its half-set warning)]
  authoritative_for: "hook-runtime: the doc-viewer link prefix carried into agent-facing briefings"
---

# Hook Runtime — the opt-in doc-viewer prefix in agent-facing briefings

A record the assistant names as a bare path is a dead end for the human reading
the answer: they must go find it themselves. A project that runs a document
viewer can say so once in its configuration, and every doc reference the
assistant writes for the rest of the session becomes a URL the human clicks.
The behavior is opt-in, silent when unconfigured, and deliberately confined to
what the assistant writes in prose.

## Behaviors & Operations

**B24 — One configured viewer, two briefing surfaces.** The viewer is declared
once, as a pair: the viewer's own base address and the project name it serves.
Both must be present and non-empty for a prefix to exist. The prefix is built by
one shared reader, and both agent-facing briefings render from that reader —
never from the raw configuration:
- The **session-start briefing** gains one section, placed immediately after the
  declared project commands: the prefix, plus the instruction to append the
  repo-relative path of whatever doc is being named, never the bare path.
- The **post-compaction capsule** re-states the same fact in one line. This is
  not redundancy: the capsule is what survives a long session's compaction, and
  without it the assistant silently reverts to bare paths at exactly the moment
  the session has forgotten why it stopped writing them.

**B25 — bee joins, it never encodes.** Building the prefix normalizes only what
would otherwise collide at the join: exactly one trailing separator comes off
the base address, and every leading and trailing separator comes off the project
name. Nothing else is rewritten — a doubled separator the author typed inside
the base address stays as typed, and the repo-relative path appended afterwards
is never percent-encoded. A path containing a space is the link author's problem
to escape, not the configuration's to guess at.

**B26 — Unset is silent; half-set is loud.** An absent key is today's behavior
with no output at all: the briefings render byte-identical to a project that
never heard of the viewer, and every doc reference stays a bare path. A key that
is present but unusable — not a pair, a missing or non-string half, or a half
that is empty once its separators are trimmed — emits exactly one warning line
naming both required halves and stating that doc links are disabled, then
proceeds with no prefix. The failure is announced because a half-set key is
someone's unfinished intent, while an unset key is a decision.

## Business Rules

- R24 — The prefix reaches agent prose only. The routing and status commands,
  and every other command surface, keep printing bare paths whether or not a
  viewer is configured: a human reading command output is already in the
  terminal that produced it, and rewriting machine-readable output would break
  the readers that parse it.

- R25 — Where the prefix exists, the assistant's contract requires it: a record
  is linked as its viewer URL, never pasted and never named as a bare path. The
  briefing carries the prefix; the contract carries the obligation.

## Edge Cases Settled

- A base address that is nothing but a separator trims to empty and counts as
  half-set → warn, no prefix (never a URL made of separators).

- A local configuration overlay may supply or override the key like any other
  setting, so one developer can run a viewer without committing it for everyone.

## Pointers (implementation)

- Shared reader: `doc_viewer_prefix` in `packages/bee-rs/crates/bee/src/state.rs`
  (with `warn_half_set_doc_viewer` beside it). Injectors:
  `hooks/session_preamble/budget.rs` (the "Doc links" section) and
  `hooks/compaction.rs` (capsule item 10b). Contract text: `AGENTS.md`,
  `packages/bee/AGENTS.block.md`, and the Communication contract in
  `skills/bee-hive/references/routing-and-contracts.md` (plus their vendored
  copies). Key documentation: `docs/config-reference.md` (`doc_viewer`).
  Provenance: `.bee/cells/archive/doc-viewer-links/dvl-1.json`, `dvl-2.json`.
