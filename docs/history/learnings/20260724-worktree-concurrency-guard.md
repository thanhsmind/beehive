---
date: 2026-07-24
feature: worktree-concurrency-guard
categories: [pattern, decision, failure]
severity: [P2, P2, P1]
tags: [worktree, concurrency, write-guard, scheduling, regen-obligation, cell-authoring]
---

# Learnings — worktree-concurrency-guard

## What Happened (1): a locked decision proved too narrow by its own validating-stage spike

D2 was originally locked as symlink-escape-only detection, matching the
backlog brief's literal wording. A validating-stage spike
(`.bee/spikes/worktree-concurrency-guard/probe-nested-checkout-baseline.mjs`,
three real-fixture probes against the unmodified write-guard) found the
write-guard already denies an unrecognized symlink escape today, regardless
of concurrency — so D2's original scope covered a gap that barely existed.
Meanwhile a plain nested checkout physically inside the tree — the actual
incident shape the feature exists to prevent — was completely unguarded, and
behaved structurally identically to a legitimately registered git submodule.

**Root Cause:** the original decision was locked from the brief's prose
before any fixture evidence existed. Prose framing ("companion checkout")
implicitly anchored the mental model to the symlink-mount shape already
partly built (PR #61's containment recognition), while the actual reported
incident (STR65) was a plain nested repo inside a shared checkout with no
symlink involved at all.

**Recommendation:** When a locked decision's scope is phrased in terms of an
existing partial implementation ("the companion mount shape"), treat that as
a hypothesis, not a settled fact, until a validating-stage spike proves what
the current code actually does for the shape the decision is supposed to
close. A spike that only confirms "yes this trigger fires" is not enough — it
must also check whether the *adjacent, unguarded* shape is real, since that
is usually the shape self-discipline actually failed on.

## What Happened (2): the same undeclared-regen-side-effect race recurred three times in one feature

Cells wcg-1, wcg-2, and wcg-3 each independently discovered that editing a
hashed lib/hook/template file triggers a regen chain (canonical template
propagation, manifest/ledger rewrite) that touches more files than the cell's
own primary edit — and, for wcg-2/wcg-3 specifically, an adversarially
dispatched plan-checker caught that this made the two cells unsafe to
schedule in parallel: both silently rewrite the same manifest/ledger via
their regen step, but only one of them had declared those paths in its own
`files`, so the scheduler (which serializes purely on declared file overlap)
saw no conflict and would have run them concurrently.

**Root Cause:** the cell scheduler has no independent knowledge of what a
regen command actually writes — it trusts the cell author's declared `files`
completely. A regen obligation is a real, mandatory side effect of touching
a hashed root in this repo, but nothing forces it into the scheduling
model's view.

**Recommendation:** Any cell whose `files` include a hashed root (lib, hook,
or mirrored template path) must also declare every file its own regen step
will touch (the release manifest, the onboarding ledger) — even when another
cell in the same batch already declares them. Do not rely on "another cell
already covers it" reasoning; declare it per cell, since the scheduler's
conflict detection is purely file-list-based. Promoted as a critical pattern:
`docs/knowledge/patterns/20260724-scheduler-blind-to-regen-side-effects.md`.

## What Happened (3): a fresh worktree inherited an unrelated feature's stale claimed cell, twice

Both times a new worktree was created for this feature (the first was later
destroyed by concurrent activity in main and recreated under a different
name), `state start-feature` refused immediately, citing a claimed-but-
uncapped cell (`rel1150-1`) belonging to a completely different, already-
released feature.

**Root Cause:** `.bee/cells/` is git-tracked, so `git worktree add` checks out
whatever cell files existed in the source commit, including any other
feature's stale in-flight cell. Worktree bootstrap only copy-if-absents
`onboarding.json`/`config.json`/`state.json` — it never scopes `cells/`.

**Recommendation:** Until the underlying scaffolding bug is fixed (tracked as
backlog `p-9c48a67c`, feature `worktree-scaffolding-cell-leak`), expect this
exact refusal on the first `state start-feature` call in any newly created
worktree, and resolve it with `cells drop --reason "..."` naming why the cell
is stale (already resolved/committed in the base commit), never by editing
the cell file by hand. Promoted as a critical pattern:
`docs/knowledge/patterns/20260724-worktree-inherits-stale-cells.md`.

## What Happened (4): an advisor's independent re-verification, not the orchestrator's own reasoning, caught both real defects in this feature

The configured advisor (fable) was the first to flag that D2's literal text
didn't cover the shape the cell actually implemented (finding #1 above); an
independently dispatched plan-checker was the first to catch the parallel-
wave regen race (finding #2 above). Neither was found by the orchestrator's
own planning or validating pass before dispatching a second, independent
reviewer.

**Root Cause:** a single agent authoring a plan and then validating its own
plan shares the same blind spots in both passes — narrowing scope to match a
convenient mental model, or failing to notice an implicit side effect it
itself introduced, are exactly the kind of self-consistent errors a second
independent read (not the same reasoning re-run) is positioned to catch.

**Recommendation:** For high-risk work, always dispatch the plan-checker and
advisor as genuinely independent passes (fresh context, re-running the actual
commands rather than trusting the plan's prose) rather than treating them as
a formality to satisfy before Gate 3 — both caught real, cited, evidence-
backed defects in this feature, not stylistic nits.

## Backlog and Friction Filed

- `p-9c48a67c` (feature `worktree-scaffolding-cell-leak`, proposed): the
  stale-cell-copy bug from finding (3), not fixed in this feature.
- Deferred from wcg-2's worker: `apply_patch` write targets are not covered
  by the new write-guard check (the cell scoped explicitly to Bash and
  Edit/Write) — a same-threat follow-up if `apply_patch` coverage is ever
  wanted for this repo's Codex-runtime path. Not filed as a backlog row since
  no host project currently exercises `apply_patch` against this guard in a
  way that makes the gap material; noted here so it is not silently lost.
