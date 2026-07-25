---
type: bee.area
title: Hook Runtime — post-compaction orientation and the compact capsule
description: "The narrowed orientation a session sees when it resumes from a compaction instead of the full startup preamble: what it carries, in what fixed order, why its bytes never depend on whether an intent anchor exists, and how the integrity sweep and the handoff-adoption explanation reach it."
timestamp: 2026-07-23
bee:
  id: hook-runtime-post-compaction-orientation-and-the-compact-capsule
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/the-intent-anchor-and-compaction-survival.md, areas/hook-runtime/advisories-and-turn-control.md]
  decisions: [089905ba (compaction-hardening D19 — the hook keeps owning the anchor; the capsule is the preamble replacement only), "compaction-hardening D6 (the capsule's fixed twelve-item order)", compaction-hardening D8 (every other start keeps the full startup preamble byte-identical), compaction-hardening D9 (the compaction-survival advisory), "compaction-hardening D12, D13 (the integrity sweep reports and never mutates)", 2056a7ca (compaction-hardening D27 — the adoption-refused explanation is a call-site contract)]
  sources: ["compaction-hardening cells cz-3, cz-4, cz-5, cz-6 (the compaction module, its verbs, the capsule builder and its golden, the SessionStart wiring; traces in `.bee/cells/`, 2026-07-23)", "docs/history/compaction-hardening/CONTEXT.md D6, D8, D9, D12, D13, D19, D26, D27"]
  authoritative_for: "hook-runtime: post-compaction orientation and the compact capsule"
---

# Hook Runtime — Post-Compaction Orientation and the Compact Capsule

## Purpose

A session that resumes right after a compaction is asking one narrow question —
which unit of work is this, and what exactly is the next step — not the broad
question a brand-new session asks. Handing it the full startup orientation answers
questions it did not ask, at the exact moment its budget for reading anything is
smallest. This concept owns the narrower orientation built for that one moment: what
it carries, in what order, and the guarantees that keep it trustworthy even when the
state it describes is not.

## Entry Points & Triggers

- A session starts with its source marked as a resume from a compaction → the
  narrowed orientation renders in place of the full preamble.
- Every other start — a fresh session, a manual clear, an ordinary resume — keeps
  the full startup orientation, byte-identical to what a session with no compaction
  history has always seen.

## Data Dictionary

| Element | Meaning |
|---|---|
| the narrowed orientation | The block a compacting session sees instead of the full startup preamble. Carries twelve items, always in the same order. |
| item 1 — the intent anchor | Rendered by the anchor's own owner, not by this orientation (see the intent-anchor concept). The narrowed orientation's own content begins at item 2. |
| item 2 — state mismatch | One line naming every check the integrity sweep found wrong, immediately after the anchor. |
| item 3 — onboarding-missing notice | Present only when the project's guardrails are not current. |
| item 4 — the handoff block | Present only when a paused or planned handoff exists; carries its wait-and-never-auto-resume instruction verbatim, and, when adoption of a planned handoff was refused, the reason it was refused. |
| item 5 — the bypass banner | Present only when a gate-bypass level is active; a mandatory loud banner wherever any orientation renders, narrowed or full. |
| item 6 — phase/mode/feature/lane | The same short status line the full preamble has always carried. |
| item 7 — the claimed unit | The cell this session holds, its verification command, and whether its dependencies are still capped. |
| item 8 — the first open gate | The next approval the human owes, if any. |
| item 9 — next action | The single next step recorded for this session. |
| item 10 — recorded commands | The project's setup/start/test/verify commands. |
| item 11 — compaction survival count | How many times the claimed unit has now been compacted, and, once that count is two or more, an advisory that the unit may be oversized. |
| item 12 — critical-patterns pointer | One line naming where the durable patterns live, not their content. |

## Behaviors & Operations

### Rendering the narrowed orientation

- **Runs when:** a session starts with its source marked as a compaction resume.
- **Blocked when:** nothing — this path always renders; it is a shape decision, not
  a gate.
- **What changes:** the full startup preamble is replaced by the twelve-item
  orientation above; a compaction-log record for the resume is appended in the same
  pass.
- **Side effects:** none beyond the log append.
- **Afterwards:** the session sees exactly what it needs to resume the unit it was
  on, nothing more; every other start this concept does not touch renders its full
  orientation exactly as before.

### The anchor stays with its owner

The narrowed orientation never renders the intent anchor itself — the anchor's own
owner renders it first, unchanged, and the orientation begins after it (see
[`the-intent-anchor-and-compaction-survival.md`](the-intent-anchor-and-compaction-survival.md)).
**No byte of the narrowed orientation may vary with whether an anchor exists.** The
integrity sweep's own anchor-presence check is therefore muted before the state-
mismatch line renders — an anchor missing is real information the sweep still
collects, it is simply never allowed to change this orientation's bytes. This was
measured two ways: paired renders came out byte-equal both on a clean sweep and on
a failing one, and — the sharper version — a paired render where the anchor was
written *before* the compaction log, so the underlying log bytes themselves
differed between the two renders, still came out byte-equal.

### The adoption-refused explanation is a call-site contract, not just a renderer one

When a planned handoff exists but this start does not qualify to adopt it, the
handoff block explains why in one line. That explanation only reaches a reader if
the code that calls the orientation builder passes the refusal reason through
explicitly — fixing only the renderer that would otherwise print the line is not
enough, because the renderer never sees a value nobody handed it. This was found
because every pre-existing check exercising that path matched only the wait
heading and never the reason line itself, so a version of the caller that dropped
the parameter passed every test in the feature and the full verification chain
while a compacted session silently lost the explanation of why its handoff was not
adopted.

### The compaction survival advisory

- **Runs when:** the record just appended for a compaction shows the claimed unit
  has now been compacted twice or more.
- **What changes:** nothing is blocked — the advisory is informational only, on
  both the pre-compaction notice and the narrowed orientation that follows it.
- **Side effects:** none.
- **Afterwards:** the session sees a suggestion that the unit may be oversized and
  worth capping at the next green verification rather than carrying forward again.

### The integrity sweep

- **Runs when:** a compaction resume renders the narrowed orientation (and on
  demand, by command, at any other time).
- **What it checks:** the session record it is resuming still matches; the lane it
  believes it is in still resolves; every unit it holds is still held by it; the
  approval that let it write is still in force whenever a unit is held; that unit's
  dependencies are still capped; its file holds are still its own; an anchor exists
  (reported, but muted from the orientation's bytes per above).
- **What it never does:** repair, release, or block anything. A mismatch is
  reported as one line naming each failed check, with the rule that disk state
  overrides whatever the session remembers — the session's own recollection is
  exactly the thing least trustworthy right after a compaction, so nothing here
  acts on it automatically.

## Actors & Access

- **A compacting session** — receives the narrowed orientation in place of the full
  preamble; observes exactly the twelve items above, in order.
- **A non-compacting session start** (fresh, cleared, or an ordinary resume) —
  unaffected; receives the full startup orientation, unchanged.
- **The integrity sweep** — a read-only reporting surface, reachable by command
  independent of whether any lifecycle checkpoint actually fires, so it stays
  usable even on a runtime whose checkpoint execution is unconfirmed.
- **The human owner** — sees the mismatch line, the handoff block and its adoption
  explanation, the bypass banner, and the survival advisory exactly as the session
  sees them; approves whatever they name.

## Business Rules

- **R1** — The narrowed orientation replaces the full startup preamble only on a
  compaction-resume start; every other start stays byte-identical to what it has
  always rendered (compaction-hardening D8).
- **R2** — The narrowed orientation renders exactly the twelve items above, in that
  fixed order; the anchor is item 1 but is rendered by its own owner, not by this
  orientation (compaction-hardening D6, D19).
- **R3** — No byte of the narrowed orientation may vary with whether an intent
  anchor exists; the integrity sweep's anchor-presence check is muted before the
  state-mismatch line renders, and this was measured to hold even when the
  underlying log bytes genuinely differ between the two renders being compared
  (compaction-hardening D19).
- **R4** — The handoff-adoption-refused explanation reaches the orientation only
  because the code that calls the orientation builder passes the refusal reason
  through explicitly; a fix confined to the renderer is not sufficient
  (compaction-hardening D27, `2056a7ca`).
- **R5** — The integrity sweep reports and never mutates: no mismatch it finds is
  auto-repaired, released, or blocked — only surfaced with the rule that disk state
  overrides conversational recollection (compaction-hardening D12, D13).
- **R6** — A unit compacted twice or more produces a non-blocking advisory that it
  may be oversized, shown on both the pre-compaction notice and the narrowed
  orientation (compaction-hardening D9).

## Edge Cases Settled

- **A design where the narrowed orientation also carried the anchor was rejected.**
  It would render the anchor twice, and neither the check that the anchor leads nor
  a test built to assert a straightforward replacement would have caught the
  duplicate — both pass equally whether the anchor appears once or twice. Keeping
  the anchor's existing owner in sole control made an entire planned change to an
  existing proof unnecessary: the feature that added this orientation closed with
  zero edits to that proof.
- **Proving byte-identity against the previous version of the code goes circular
  the moment the change lands** — the durable form of "unchanged" is a committed
  golden fixture compared byte-for-byte, not a reconstruction from version control
  that decays into asserting the new code equals itself.

## Open Gaps

- Proven so far by direct construction and by fixture-driven tests, not by a live
  session carried through a real compaction boundary — the same open gap the
  intent-anchor concept records; one end-to-end run would close it for both.
- The non-compacting sources (a fresh session, a manual clear) have no
  through-the-hook byte-for-byte comparison row in the capsule-specific test suite
  — their coverage today is the relative anchor-vs-control rows plus the committed
  golden fixture, not a dedicated equality check against this orientation's own
  suite.

## Pointers (implementation)

- Builder: `buildCompactCapsule(root, {sessionId, handoffOutcome})` in
  `skills/bee-hive/templates/lib/compaction.mjs`, alongside
  `appendCompactionRecord`, `readCompactionCounts`, `survivalWarning`,
  `anchorMissing`, `compactCheck`, and `CAPSULE_MUTED_CHECKS` (the anchor-mute
  list). Mirrored to `.bee/bin/lib/`.
- CLI: `state compact-log`, `state compact-check`, and `state compact-capsule` in
  `command-registry.mjs` + `bee.mjs` — each reachable independent of whether any
  lifecycle checkpoint fires.
- Wiring: `hooks/bee-session-init.mjs` (the compaction-resume branch; computes and
  passes `handoffOutcome` through to the builder), `hooks/bee-session-close.mjs`
  (the pre-compaction log append and survival notice).
- Golden fixture: `scripts/fixtures/preamble-golden.txt`.
- Proof: `scripts/tests/test_compact_capsule.mjs`, `scripts/tests/test_compaction_module.mjs`,
  `scripts/tests/test_compaction_advisories.mjs`, and the compaction-resume rows in
  `hooks/test_hook_contracts.mjs`.
