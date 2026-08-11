# Planning Reference

Use when `bee-planning` needs artifact templates, cell quality rules, or the
full gate/review protocols.

## Artifact fan-out — separate files are earned, not default

`plan.md` is the truth artifact for **standard/high-risk** (and small only on
request). Tiny drops it entirely; small's default is the logged scoping
synthesis + cells. Where `plan.md` exists, discovery and approach content
default to **sections inside it**; they graduate to their own files only when
real complexity makes a standalone file worth reading on its own.

| Artifact | Separate file when | Otherwise |
|---|---|---|
| `plan.md` | standard/high-risk (frozen at the shape gate), or small when a durable multi-slice/product-decision doc is genuinely needed | tiny: none; small: opt-in |
| `discovery.md` | a real multi-candidate comparison worth preserving alone | a `## Discovery` note inside `plan.md`, findings cited |
| `approach.md` | high-risk, or rejected alternatives + a risk map substantial enough to stand alone | an `## Approach` section inside `plan.md` |

Rule of thumb: if the separate file would just repeat what `plan.md` already
says, it should have been a section. Fold, don't fan out.

## Artifact: plan.md

Frozen once the shape gate is approved: the only permitted post-approval
write is the approval stamp in the frontmatter. No in-place enrichment.

```markdown
---
artifact_contract: bee-plan/v1
mode: standard | high-risk | spike | small (opt-in)
# approved_gate2: <unset until approval; then a date stamp — the only permitted post-approval write>
---

# Plan: <Feature>

Mode: `<mode>` — <k> risk flags: <list, or "none">
Why this is the least workflow that protects the work: <one sentence>

## Requirements (from CONTEXT.md)
- D1: <locked decision restated> ...

## Discovery
<2–4 lines: what was inspected, the finding, the evidence command — or a
one-line pointer to discovery.md when one exists.>

## Approach
<Recommended path (cites decision ids) · rejected alternatives (one line
each) · compact risk map (component / LOW-MEDIUM-HIGH / proof needed). When
approach.md exists, drop this section and point to it.>

## Shape
<one of the bodies below, by mode>

## Test matrix
<standard and below: the triad — happy path, edge cases, error paths — at
its smallest demonstrating size. high-risk/hard-gate: the 12 dimensions of
edge-dimensions.md, probes written per applicable dimension. Either way each
cell's writer judges existing coverage first and authors only what is not
already pinned (`.bee/expertise/tests.md`).>

## Out of scope
<explicitly not solved; deferred ideas stay deferred>
```

No `## Current slice` / `## Cells` sections are added post-approval: the
plan stays frozen and the current slice lives only in cells.

**Shape bodies by mode:**

- `spike` — the one yes/no question, what proves YES, what NO implies,
  `.bee/spikes/<feature>/` location.
- `standard` (milestone-shaped) — **phase plan**: `Phase | What Changes |
  Why Now | Demo | Unlocks` table.
- `standard`/`high-risk` (capability/risk-shaped) — **epic map**: feature
  outcome, repo-reality basis, `Epic | Capability/Risk Area | Why It
  Exists | Slices | Proof Needed` table, slice queue with deps, current
  slice to prepare.

## Artifact: approach.md (only when the fan-out table calls for it)

```markdown
# Approach: <Feature>

## Recommended path
<2–5 sentences: what we build and in what order. Cites decision ids.>

## Rejected alternatives
- <alternative> — <why rejected, one line>

## Risk map
| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| <area> | LOW/MEDIUM/HIGH | <why> | <command, inspection, or spike question> |

## Files and order
<bounded list, likely touch order>

## Questions still open
- <assumption that could invalidate the path>
```

## Phase plan vs epic map

Use **phases** only when the work has observable milestones a user could
demo in order — first phase obvious, later phases building on it, never
technical buckets ("backend" is not a phase). Use an **epic map** when
capability or risk areas explain the work more honestly than a timeline —
the high-risk default, with feasibility proof named per epic. Never force
2–4 phases onto work that is really one slice.

## Cell quality rules

A cell is an executable prompt a cold worker can pick up with zero session
history.

1. **Directive action, no code blocks.** Prose that says what to do and
   cites decisions (`per D2`). Code belongs in the repo, written by the
   worker.
2. **Bounded files.** `files` lists everything the worker may write;
   `read_first` what it must read. Cross-cell file overlap is legal, not a
   scoping error — it only costs a wave (auto-serialized); prefer explicit
   paths or trailing-`*` patterns, since overlap detection treats mid-path
   globs as literals.
3. **Testable exit.** The cell's outcome is provable by the declared
   suite (`commands.test`) at finish — plan the cell so its tests exist
   by cap time. "Manually check" is not an exit.
4. **must_haves are contracts:** `truths` (observable behavior),
   `artifacts` (path + substantive description — no stub counts),
   `key_links` (wired, not just existing), `prohibitions` (what must NOT
   change). Required for `standard`/`high-risk`; `tiny` may omit.
5. **behavior_change honesty.** Any cell changing observable behavior is
   `behavior_change: true`. The flag decides the capture debt and review
   scrutiny — mislabeling is a production bug waiting to happen.
6. **Deps are real.** `deps` lists cell ids whose output this cell needs.
   Ready = all deps capped.
7. **Current slice only.** If the cell belongs to a later slice, it does
   not exist yet.
8. **Evidence lives in the trace — never manufacture evidence artifacts.**
   `artifacts` are the product the cell builds (a source file, a spec, a
   migration), never a report that the cell ran; verification evidence
   belongs in the cell trace, its single source.

## Example cell JSON

```json
{
  "id": "auth-3",
  "feature": "auth",
  "title": "Wire session middleware into API router",
  "lane": "standard",
  "status": "open",
  "deps": ["auth-1", "auth-2"],
  "decisions": ["D2", "D4"],
  "files": ["src/api/router.ts", "src/auth/middleware.ts"],
  "read_first": ["src/api/router.ts"],
  "action": "Mount the session middleware from auth-2 onto all /api/* routes (per D2). Preserve the existing public response envelope (per D4). Follow the error-handler registration pattern already used in router.ts.",
  "must_haves": {
    "truths": ["Unauthenticated /api/* requests return 401"],
    "artifacts": [{"path": "src/auth/middleware.ts", "substantive": "exports authGuard, no TODO stubs"}],
    "key_links": ["router.ts imports and mounts authGuard"],
    "prohibitions": ["No change to public response envelope"]
  },
  "trace": {
    "worker": null, "outcome": null, "files_changed": [],
    "deviations": [], "friction": null, "capped_at": null,
    "behavior_change": true
  }
}
```

Create the whole slice with one batched stdin call (a JSON array; a single
object works for a one-cell slice — no per-cell scratchpad files):

```bash
.bee/bin/bee cells add --stdin <<'EOF'
[ { ...cell 1... }, { ...cell 2... } ]
EOF
```

The batch is all-or-nothing and validated before any write; fix rejects,
never downgrade the lane to dodge validation.

## Test scoping

Cells run `commands.test` — the project's ONE declared test command — at
finish; close, `bee worktree merge`, and CI re-run that same command.
`commands.verify` is retired. A host keeps every door fast by pointing
`commands.test` at a suite it is willing to run on every cap. In a repo that has declared itself no-test (`commands.test` set to
the sentinel `"none"`), cells cap with `tests: undeclared` — never invent
a fake check to satisfy the runner.

## Greenfield init lane

When the repo has no build and the init-lane offer was accepted at
onboarding, the first slice is **one init cell** — `must_haves`: setup
succeeds from scratch, one passing test exists, standard commands recorded
in `.bee/config.json`, clean first commit — before any feature cell. Its
proof is the recorded test command (`commands.test`) running green at
finish.

## Tiny/small merged gate

**Preview before persist.** Draft the cell(s) and run the SMALLER PATH
check first, then present one merged question in place of the shape and
execution gates: "Work shape + execution: I'm about to do [X] via [Y],
verified by [Z]. Approve?" The draft cells render as a preview **in the
gate message** — approval covers the exact previewed packet, and
`cells add` runs only after it. Execution approval is never granted before
the execution package exists. A SMALLER PATH FAIL is always surfaced to
the human first, whatever the gate-bypass level; the review wave never
dispatches for these lanes — the cold-pickup criteria are self-checked
while writing the cells.

## Review wave

Runs for standard/high-risk, dispatched the moment the shape is drafted,
concurrent with the SMALLER PATH check — the stage costs
`max(reviewer, planning)`, never the sum. Findings block nothing until the
gate presentation, and the gate never happens while the wave is
outstanding.

**One dispatch, two mandates, two vocabularies.** One reviewer subagent on
the review tier returns one report with two sections: **Structure**
(findings BLOCKER/WARNING) and **Cells, cold pickup** (findings
CRITICAL/MINOR). Never merge the vocabularies.

**Scaling.** `standard` with ≤5 product files and zero hard-gate flags:
no dispatch — the session model runs both mandates inline, same
vocabularies, same caps. A hard-gate flag, a 6th product file, or doubt
about self-review independence restores the dispatch. `high-risk` always
dispatches, scaled to a small panel: **coherence** and **feasibility**
personas always; add **security**, **product**, or **scope-guardian** by
the diff of concerns. Dedupe findings, then split into auto-fix (apply,
record) and present-for-decision.

**One shot, then at most one blocker pass.** WARNINGs and mechanically
fixable findings are applied directly to the cells (legal — cells are
mutable before the gate). Only unresolved BLOCKERs earn a second, final,
blocker-scoped pass; a BLOCKER still open after it escalates to the user
with both positions. All CRITICAL cell flags are fixed before the gate;
MINOR ships with a recorded note. On slice 2+ the scope is new/changed
cells only — the plan is frozen and was checked on slice 1.

Reviewer prompt:

```text
You are a merged plan reviewer. Two mandates. Assume the work is flawed until proven so.
Inputs: docs/history/<feature>/CONTEXT.md, approach.md, plan.md, and the current-work
cells (.bee/bin/bee cells list --feature <feature>).

MANDATE 1 — STRUCTURE. Verify exactly 5 dimensions:
1. Requirement/decision coverage — every locked decision lands in at least one cell.
2. Cell completeness — each cell has files, read_first, directive action, must_haves
   (per lane tier), and a testable exit the declared suite can prove.
3. Dependency correctness — deps form a DAG; no cell depends on a future slice.
4. Key links — integration points named in plan.md are owned by a specific cell.
5. Scope sanity — no cell is doing hidden architecture work or exceeds its lane.
Report every structural finding as BLOCKER (structurally unsound) or WARNING
(survivable, note it).

MANDATE 2 — CELLS, COLD PICKUP. You have NO session history. For each cell, answer:
could a worker who has read only CONTEXT.md, plan.md, and this cell implement and
verify it without guessing?
Flag CRITICAL: assumed context, vague acceptance, scope overload, unproven feasibility,
an exit the declared suite cannot prove.
Flag MINOR: missing rationale, implicit file assumption, fuzzy boundary, known tradeoff
not recorded.

Return ONE report with both sections. Never merge the two vocabularies.
Do not propose redesigns. Do not soften findings. Quote file/cell evidence per finding.
```

```text
REVIEW REPORT
Work: <current slice / direct task>

STRUCTURE
BLOCKERS: <dimension> problem / evidence / fix
WARNINGS: <dimension> problem / evidence / note

CELLS  (reviewed: <N>)
CRITICAL FLAGS: <cell-id> problem / evidence / fix
MINOR FLAGS: <cell-id> problem / evidence / suggestion
CLEAN CELLS: <cell-id>, <cell-id>

SUMMARY: <2-3 sentences>
```

## Tests ride the cell

There is no trailing test cell and no per-slice test mandate. The cell's
writer owns its tests, TDD-style, as part of the cell's own work —
coverage judgment first: cite the nearest existing tests by file and
case — read them, never guess at them — and author only what is not
already pinned. Authoring nothing on a "covered" verdict is a legitimate,
cheap outcome; duplicated rows are the waste. Shape at `standard` and
below is the triad — happy path, edge cases, error paths — at its
smallest demonstrating size; `edge-dimensions.md`'s twelve dimensions
apply only at `high-risk`/hard-gate. Case selection, duplication
judgment, and red-before-green: `.bee/expertise/tests.md`. `bee cells
finish` runs the declared suite (`commands.test`) at every cap, and
`bee close` re-runs it for the feature.
