---
type: bee.area
title: Hook Runtime — the request shapes the write guard can read
description: "How the write guard decides a batch file-change request target by target, how it shape-checks a workflow command against the published catalog, which command forms it still recognises, how it repairs a mechanically fixable question request instead of refusing it, and why an intercepted-but-unreadable request is denied rather than waved through."
timestamp: 2026-07-23
bee:
  id: hook-runtime-write-guard-request-shapes
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-runtime-parity D1, D2", "bbc6bcea (shim-retire D3: dual command-shape recognition, retired form transitional)", "ask-guard-autofix D1/D2 (fixable question violations repaired + announced, deny wins, 2026-07-23)", "d4182ff1 (blanket-staging-guard: git add -A/-u and git commit -a count as broad writes, 2026-07-26)"]
  sources: ["codex-runtime-parity repo-fallback capture 2026-07-12 — cells codex-parity-6a, 6b", "dispatcher-unify du-2 (2026-07-12, flushed capture stub 9e68432b)", "shim-retire D3 transition guard (cell shim-retire-3, 2026-07-14)", "ask-guard-autofix cell ag-1 (2026-07-23, commit 52dad26)", "blanket-staging-guard cell bsg-1 (2026-07-26, commit b240110)", "docs/specs/hook-runtime.md#B3", "docs/specs/hook-runtime.md#B3a", "docs/specs/hook-runtime.md#R3", "docs/specs/hook-runtime.md#R14a", "docs/specs/hook-runtime.md#E1", "docs/specs/hook-runtime.md#P6", "docs/specs/hook-runtime.md#P7"]
  authoritative_for: "hook-runtime: write-guard request-shape recognition and per-target decisions"
---

# Hook Runtime — the request shapes the write guard can read

Before the guard can decide whether a write is allowed, it has to understand what
was actually requested. Three request shapes reach it — a batch file-change
envelope, a shell invocation of a workflow verb, and the two command forms the
vendored surface has used over time — and the discipline is the same in all
three: a request the guard intercepted but cannot read is denied, while an event
it never saw at all fails open and says so.

**`R14a` is a disambiguated id.** This rule shipped as `R14` and shared that id
with the gate-bypass block-verdict rule in
[`advisories-and-turn-control.md`](advisories-and-turn-control.md); the collision made
one of the two permanently unmeasurable by the coverage gate. The two are
genuinely different rules, so neither was dropped: this one — the id no other
document ever cited — was renumbered `R14a` in the source before the migration
pin was captured, and the pointer stub's anchor map records both readings.

## Behaviors & Operations

**B3 — Batch file-change requests are guarded per target.** When the runtime
announces a batch file-change request (the patch-style tool), the write guard
parses every add/update/delete/move target and runs each one through the same
gate, direct-edit, and reservation decisions that govern single writes.
- All targets provable → each target decided on its own; one denied target
  denies the request with a corrective message.
- Request intercepted but targets NOT provable (no parsable change lines, a
  blank path, a target resolving outside the project) → **deny** with a
  corrective message. An intercepted-but-unreadable batch is never waved
  through.
- The outer event itself malformed (no batch envelope present at all) →
  fail-open, logged: the guard cannot know a write was intended.
- Containment recognizes the worktree-companion mount (PR #61, cell mp61-1,
  2026-07-24): a path under the recorded `commands.worktree_companion_mount`
  symlink — the mount `bee worktree new --with-companion` creates for a nested
  repo's own worktree — resolves to its companion-relative form instead of
  being denied as an out-of-worktree escape. Every other out-of-worktree
  target keeps today's denial.

**B3a — Workflow-command requests are shape-checked against the published
catalog.** When a shell request invokes a workflow verb, the guard resolves the
command against the catalog of record — including verbs whose full name is
three words deep (group, sub-group, action) — and validates the required
parameters and value shapes before the command runs. A malformed invocation is
denied with the command, the missing or wrong field, and the corrective shape;
a well-formed one proceeds untouched. Deep verbs previously escaped this check
unvalidated (a silent fail-open); they no longer do.

**B23 — Blanket staging reads as a broad write, not as zero targets.** When a
shell request's targets are extracted for the reservation guard, `git add
-A`/`--all`/`-u`/`--update` and `git commit -a`/`--all` (combined short
clusters like `-am` included) set the broad-write marker even though they name
no path — they stage or fold in *every* changed file, which on a shared
checkout can sweep another session's in-progress work into the commit. The
broad-write marker resolves to the `**` target, so the existing reservation
flow blocks exactly when another session holds a reservation and stays a no-op
for a single session. Explicit-path `git add`, plain `git commit`/`-m`, and
`--amend` (matched by exact token, never substring) are untouched (cell bsg-1,
2026-07-26).

**B22 — A malformed question-to-the-human request is repaired when the repair
is mechanical, refused when it is not.** When the runtime announces the
ask-the-human tool, the guard shape-checks the request before the platform's
own opaque validation can reject it. Trigger: any question request. What
happens: a violation whose repair is deterministic and meaning-preserving — a
chip-label heading over the 12-character limit — is FIXED, not refused: the
heading is rewritten (first 11 characters, right-trimmed, plus an ellipsis) on
a copy of the request, and the question proceeds with the rewritten input; the
platform is told, in the approval itself, exactly what was changed, and the
human sees a one-line note of the rewrite. A violation with no mechanical
repair — question count outside 1–4, option count outside 2–4, an option
missing its label or description — refuses with the specific correction, and a
refusal always wins over any repair collected in the same request: the mixed
case refuses. Odd shapes still fail open. What each actor observes: the asker's
question reaches the human instead of dying on a label-length technicality;
the original request object is never mutated — the rewrite rides a replacement
copy (ask-guard-autofix D1/D2, cell ag-1, 2026-07-23).

## Business Rules

- R3 — An intercepted batch change with unprovable targets is denied, not
  fail-opened (codex-runtime-parity D2, strengthening).

- R14a — The write guard's command-shape recognition accepts both the unified
  dispatcher form (group + verb) and the retired per-command helper form. The
  retired form is a transition affordance for hosts whose vendored tools predate
  the unified surface — it is slated for removal once hosts have upgraded (a
  debt item tracks it), and its recognition never revives the deleted scripts
  themselves (decision bbc6bcea, D3).

- R22 — A question-request violation is repaired only when the repair is
  deterministic and meaning-preserving; everything else refuses with the
  specific correction, and a refusal always beats a repair found in the same
  request. The repair is announced — to the platform in the approval, and to
  the human as a one-line note — never applied silently (ask-guard-autofix
  D1/D2, 2026-07-23).

## Edge Cases Settled

- A change line with a whitespace-only path counts as unprovable → deny (found
  and pinned during matrix construction).

## Pointers (implementation)

- Batch guard: `packages/bee/hooks/bee-write-guard.mjs` (`extractApplyPatchTargets`).

- CLI-shape guard incl. 3-token verb resolution: `packages/bee/hooks/bee-write-guard.mjs`
  against the `command-registry.mjs` catalog. Evidence: `.bee/cells/du-2.json`,
  `docs/history/dispatcher-unify/`.

- Question-schema guard + auto-fix: `checkAskUserQuestion` in
  `packages/bee/lib/guards.mjs` (fixed-verdict shape
  `{allow, fixed, notes}`); updatedInput emission in
  `packages/bee/hooks/bee-write-guard.mjs` (`fixedAskVerdict` branch — stdout JSON
  `hookSpecificOutput.updatedInput`, exit 0; deny path unchanged, exit 2 +
  stderr). Evidence: `.bee/cells/ag-1.json`, tests
  `packages/bee/tests/test_guards.mjs` and
  `hooks/test_write_guard.mjs`, commit 52dad26.
