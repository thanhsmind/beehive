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

A separate file that would repeat what `plan.md` already says is a section.
Fold, don't fan out.

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

## Load-bearing claims
<one line naming the labels and the match rule, then the table — spec
below. Every row is load-bearing; no `guessed` row survives the gate.>

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | <what the shape depends on being true> | read/ran/guessed | <path:line, or the command that was run> | <the bytes, verbatim> |

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

## Open Questions
<what is still unknown, one line each — or "(none)". This is where a claim
lands when it cannot be upgraded past `guessed`.>

## Out of scope
<explicitly not solved; deferred ideas stay deferred>
```

No `## Current slice` / `## Cells` sections are added post-approval: the
plan stays frozen and the current slice lives only in cells.

### The load-bearing claims table

**Mandatory, and the gate enforces it.** A `## Load-bearing claims` table
is required in every plan.md. A claim is load-bearing when the shape
changes if the claim turns out to be false — that is the whole membership
test. Five columns, in this order:

| Column | Carries |
|---|---|
| `#` | the row number, so a refusal and an audit can name the row |
| `Claim` | the one thing the shape depends on being true |
| `Label` | `read`, `ran`, or `guessed` — nothing else |
| `Anchor` | `path:line` or `path:line-line` for `read`; the exact command for `ran` |
| `Verbatim evidence` | the bytes found there, copied, never retyped |

**Label vocabulary.** `read` = the author opened that file at that line and
saw those bytes. `ran` = the author executed that command and holds its
output. `guessed` = inferred, not observed — legal while drafting, never at
the gate.

**Match rule (the audit's one rule).** The evidence column is a verbatim
byte substring of the anchored line(s); multi-line evidence joins the lines
with `" / "`. Reflowed, trimmed, prettified, or paraphrased text is a
MISMATCH, not a near miss — a quote that drops a prefix is the exact defect
this table exists to catch.

**Membership converse.** Every load-bearing claim must be a row. A
load-bearing claim living only in the plan's prose is a plan defect, caught
by the leader's pre-flight check and by the audit
(`.bee/expertise/review.md` ("Claims-table audit")) — the binary cannot
judge prose.

**The mechanical refusal.** `bee gate --name shape` and `bee gate --merge`
refuse an approval while the table is missing or malformed, while any row
lacks a label, anchor, or evidence, while a label sits outside the three
words, or while any row is still `guessed`. There is no waiver flag. Two
remedies, both self-serve: upgrade the label by doing the real read or the
real run, or move the claim to `## Open Questions` and reshape so nothing
load-bearing rests on it. The refusal fires at `shape` and at `--merge` —
where plan.md is still editable — and never at a plain
`--name execution`, which lands after the freeze where no edit could
answer it.

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

## Class playbooks

Each playbook binds one **route class** — the value `bee route --set --class`
records (`bee-hive/references/scout-and-ticks.md` ("Route record")). It does
NOT bind the cell-level `change_class` enum
(`packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:123`), which is a
different taxonomy that overlaps this one on `bugfix` and `refactor` only — a
cell's `change_class` never selects a playbook.

**How a plan uses one (per D1, decision `132551fb`).** The plan **cites** its
class's playbook by name and anchor — this file, `("Class playbooks")`, plus
the playbook name — and never transcribes the steps into `plan.md`. The steps
live here, in one home, and are read here; a copied list goes stale and can be
satisfied by transcription. A step that does not apply stays VISIBLE and
carries its recorded reason — "named deviation is the system working"
(`AGENTS.md` ("Judgment and deviation")). A skipped step is never a refusal:
nothing in this section blocks anything.

**Which principles a class routes.** `bee orient` names them in the session
preamble, read from the `## Principle homes` section of
`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`.
Those `classes:` lines are the one home of that mapping; a playbook never
restates them.

### perf

1. Capture a baseline with the real command or trace, and record the number.
2. State the hypothesis — what is slow, and why you believe it.
3. Change ONE thing.
4. Re-measure the same way, with the same command.
5. Keep the win, discard the loss, and record BOTH numbers.

"It feels faster" is not a result (per D2, decision `1593e365`).

### bugfix

1. Reproduce the symptom on the real interface.
2. Watch that reproduction FAIL before the fix. This step's rule already has a
   home — `bee-swarming/references/worker-details.md:33-35`
   ("red-before-green is craft, applied by judgment and enforced by review,
   not by flags"). Read it there; it is deliberately not copied here, so a
   cold execution worker never has to open a planning reference.
3. Find the mechanism, not the symptom.
4. Fix the mechanism.
5. Re-run the same reproduction, on the same interface.

### refactor

1. Record existing behavior FIRST — a characterization test, a snapshot, or an
   equivalence script.
2. Prove that record green on the UNCHANGED tree.
3. Change structure in small steps.
4. The record stays green at every step.
5. A behavior change is not a refactor — it is a separate cell.

### research

1. Read-only: the outcome is an account, never a diff.
2. Trace the runtime path, not just the file list.
3. Name every source searched that came up EMPTY.
4. End with anchors a reader can open — `path:line`, or the command that ran.

Both flows have ONE home: `bee-researching/references/trace-and-provenance.md`
— § "Trace" for step 2, § "Provenance sweep" for step 3.

This is the investigation route (per D3, decision `f1ffa7bd`): the existing
`research` class, no new route and no new lane. Nothing yet ENFORCES step 1 —
read-only is craft here, not a guard (backlog `p-69bee217`).

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
   **Carry the hits, not a line range.** A `read_first` entry or an action
   that names a span inside a large file — `tests.rs:2045-2549`, or a bare
   list of line numbers — is a read instruction, and the worker follows it
   literally. Paste the `rg -n` output into the cell instead: the matching
   lines verbatim, each with its number. Line numbers ride ALONGSIDE their
   anchor text, never alone — they drift the moment an earlier edit lands,
   and a worker that can only count lines cannot recover. Never instruct a worker to work by line
   number *instead of* searching: when several sites look identical, that
   is a reason to carry more anchor — the enclosing function, the
   occurrence index, the neighbouring line — not a reason to abandon the
   search. Observed both ways: a cell naming `2045-2549` in a 5516-line
   file stalled its worker outright; the re-dispatch carried the eleven
   `rg` hits and it ran.
3. **Testable exit.** The cell's outcome is provable by the proof its
   writer will run and record at cap time (related tests for code, a
   parity/pointer check for docs, a judge verdict for behavior) — plan the
   cell so that proof exists by cap time (rule: agents-proof-at-cap).
   "Manually check" is not an exit.
4. **must_haves are contracts:** `truths` (observable behavior),
   `artifacts` (path + substantive description — no stub counts),
   `key_links` (wired, not just existing), `prohibitions` (what must NOT
   change). Required for `standard`/`high-risk`; `tiny` may omit.
5. **behavior_change honesty.** Any cell changing observable behavior is
   `behavior_change: true`. The flag decides the capture debt and review
   scrutiny — never mislabel it.
6. **Deps are real.** `deps` lists cell ids whose output this cell needs.
   Ready = all deps capped.
7. **Current slice only.** If the cell belongs to a later slice, it does
   not exist yet.
8. **Evidence lives in the trace — never manufacture evidence artifacts.**
   `artifacts` are the product the cell builds (a source file, a spec, a
   migration), never a report that the cell ran; verification evidence
   belongs in the cell trace, its single source.
9. **Predicted affects_skills and affects_specs.** Every cell carries flat arrays
   `affects_skills` and `affects_specs` (repo-relative paths; `[]` when none are
   affected) required on every lane (per D3).
10. **Role is required; the name is guidance, not an enum.** Every cell carries
    `role`, the job this work is — the sole model selector, required exactly as
    `lane` is (`bee cells add` refuses without it). The recommended vocabulary —
    `code`, `read`, `test`, `docs`, `review`, `design` — is authoring guidance
    only; any non-empty name is legal, validation checks presence and shape,
    never membership. A role nothing in `models.<runtime>` configures still
    runs: it falls through to the next name the dispatch asks for and warns,
    it never fails. The one silent case is `code` or `read` on a runtime whose
    `models.<runtime>` configures NEITHER of them — the pre-roles window, where
    falling through is the intended no-op; configuring either key closes it.

## Example cell JSON

```json
{
  "id": "auth-3",
  "feature": "auth",
  "title": "Wire session middleware into API router",
  "lane": "standard",
  "role": "code",
  "status": "open",
  "deps": ["auth-1", "auth-2"],
  "decisions": ["D2", "D4"],
  "files": ["src/api/router.ts", "src/auth/middleware.ts"],
  "read_first": ["src/api/router.ts"],
  "affects_skills": [],
  "affects_specs": [],
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

## Pre-flight before cells add

Walk this checklist BEFORE drafting cells — it is the map, the validator
stays the source of truth:

1. **Ids.** Match `^[A-Za-z0-9][A-Za-z0-9._-]*$` and follow the
   `<feature-slug-abbrev>-<n>` convention (e.g. `auth-3`); collide with no
   existing cell id — list current ids first: `bee cells list`.
2. **Required fields.** `id`, `feature`, `title`, `action`, `verify`, `role`
   are all non-empty strings; `affects_skills` and `affects_specs` are
   required flat arrays (`[]` when none are affected); `verify: "none"` is
   legal only in a repo whose `commands.test` declares itself no-test (the
   `"none"` sentinel). `role` is any non-empty name — `code`, `read`, `test`,
   `docs`, `review`, `design` are the recommended vocabulary, guidance only;
   an unconfigured name still runs (fall-through, plus a warning — silent only
   for `code` or `read` on a runtime that configures neither), never a
   refusal.
3. **Lane.** One of `tiny`/`small`/`standard`/`high-risk`/`spike`;
   `standard`/`high-risk` cells carry non-empty `must_haves.truths`.
4. **Scope-derived obligations.** Any `files` path under a release-manifest
   or onboarding-ledger root obliges `verify` to carry `bee dev
   release-manifest --check`, `files` to carry the manifest record, and the
   cell to run the regen chain (`bee dev regen`) — or a reasoned
   `regen_obligation_ack` (recognized value `"wave-barrier"` defers to wave
   close). A guard-source path on a lane below `standard` obliges
   `judge_obligation_ack` or a raised lane.
5. **Deps and slice.** `deps` are acyclic; current slice only; the
   feature's execution gate is approved.

Then pipe the drafted batch through `bee cells add --stdin --dry-run` and
run the real add only after a clean dry-run — a dirty dry-run's problems
list is the fix list, applied before anything persists.

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

The agent owns test scope end to end (rule: agents-proof-at-cap). `commands.test` is the project's ONE declared test command, and it stays what CI runs on every push, the one deterministic net. `commands.verify` is retired. A host keeps CI fast by
pointing `commands.test` at a suite it is willing to run there. In a
repo that has declared itself no-test (`commands.test` set to the
sentinel `"none"`), cells prove with the command segment `none` and the
reason naming the parity/docs check actually used — never invent a fake
check to satisfy the runner. A scoped-green cap whose CI later goes red
is a fix-first cell PLUS a mandatory captured learning on why the chosen
scope missed — the learning loop is what keeps agent-owned scope safe
over time.

A cell's `verify` is authored at that same scope, and the worker runs it
as written — so write the NARROWEST command that proves this cell: a
test-name or module filter over the tests the cell's own change can
break, never a copy of the whole declared suite. Pasting the declared
command into every cell buys nothing the push already buys and makes each
worker pay a full build of everything; keep the release profile (or any
other slow flag the suite declares) only where the cell's proof actually
needs it. A cell whose change is docs or prose carries a parity/pointer
check as its `verify`, not a test run at all.

## Greenfield init lane

When the repo has no build and the init-lane offer was accepted at
onboarding, the first slice is **one init cell** — `must_haves`: setup
succeeds from scratch, one passing test exists, standard commands recorded
in `.bee/config.json`, clean first commit — before any feature cell. Its
proof is the recorded test command (`commands.test`) running green,
recorded on the cap and checked at the boundary (`bee close`/`bee
worktree merge`).

## Tiny/small merged gate

**Preview before persist.** Draft the cell(s) and run the SMALLER PATH
check first, then present one merged question in place of the shape and
execution gates: "Work shape + execution: I'm about to do [X] via [Y],
verified by [Z]. Approve?" The draft cells render as a preview **in the
gate message** — approval covers the exact previewed packet, and
`cells add` runs only after it. Execution approval is never granted before
the execution package exists. A SMALLER PATH FAIL is always surfaced to
the human first, whatever the gate-bypass level; the hat wave never opens
for these lanes — the cold-pickup criteria are self-checked
while writing the cells.

**Evidence folds INTO the one question — never a second artifact.** These
lanes write no plan.md, so the load-bearing claims ride the gate message
itself: `"…verified by [Z] (claim: <file:line> — "<verbatim quote>")"`.
Same match rule as the table — the quote is bytes copied from that
location, never a paraphrase. One claim stays inline in the sentence; at
two or more claims, list them under the question, one line each. The list
is the only permitted growth — a tiny lane never grows a plan.md to hold
its evidence.

**Zero load-bearing claims is a real answer.** When the change rests on
nothing that could be false — a typo fix, a rename the compiler proves —
say exactly that in the gate message ("no load-bearing claims: <why>").
Never manufacture a claim to fill the slot: a quote nobody needed is
noise, and it teaches the reader to skim the ones that matter.

## Plan check — the hat wave

Planning's plan check for standard/high-risk IS the hat wave. The
plan-step wave the leader opens to build the plan absorbed the old merged
plan-reviewer dispatch (proactive-leader-intake D4, decision `b34fdea9`).
**The procedure lives in one home** —
`bee-hive/references/gates-and-delegation.md` ("Hat wave"): firing point,
seats and instruments, budget, quorum, idempotence, headless, bypass, and
communication. This section is what PLANNING consumes, by pointer; it never
restates that procedure.

The wave opens the moment the shape is drafted, concurrent with the SMALLER
PATH check — the stage costs `max(wave, planning)`, never the sum. Findings
block nothing until the gate presentation, and the gate never happens while
the wave is outstanding.

**The dispatch kind changed.** No reviewer subagent on the review tier any
more: the check rides advisor-kind hat seats —
`bee dispatch prepare --kind advisor --role <hat-role>`, one seat per
dispatch — three by default (`hat-facts-gaps`, `hat-alternatives`,
`hat-user-impact`), all five on high-risk (D3, decision `423e1664`). The
leader synthesizes; synthesis never delegates.

**SMALLER PATH — one home.** The inline SMALLER PATH check at every lane
(`SKILL.md` ("Shape")) is the single home for that question. The
`hat-alternatives` seat runs it at plan altitude by CITING that mandate —
never by copying it into a seat prompt.

**Two mandates, two vocabularies — never merged.** MANDATE 1 **Structure**
(findings BLOCKER/WARNING) rides `hat-facts-gaps`' plan-step question.
MANDATE 2 **Cells, cold pickup** (findings CRITICAL/MINOR) stays with the
LEADER at cell drafting — the same self-check the `tiny`/`small` lanes
already run. The synthesis carries both sections, in their own words.

**Scaling.** A clear or tiny ask gets NO wave: the fast path stays, and
ceremony capture is the named failure (D2, decision `a52c854d`). `standard`
with ≤5 product files and zero hard-gate flags: no dispatch — the session
model runs both mandates inline, same vocabularies, same caps. A hard-gate
flag, a 6th product file, or doubt about self-review independence opens the
wave at its three default seats. `high-risk` always opens it, at all five
seats. The unit is once per FEATURE, never per message — the recorded
advisor-ref is that mark (procedure home, "Idempotence"). Dedupe findings,
then split into auto-fix (apply, record) and present-for-decision.

**One shot, then at most one blocker pass.** WARNINGs and mechanically
fixable findings are applied directly to the cells (legal — cells are
mutable before the gate). Only unresolved BLOCKERs earn a second, final,
blocker-scoped pass; a BLOCKER still open after it escalates to the user
with both positions. All CRITICAL cell flags are fixed before the gate;
MINOR ships with a recorded note. On slice 2+ the scope is new/changed
cells only — the plan is frozen and was checked on slice 1.

**MANDATE 1 — Structure.** The leader folds these criteria into the
`hat-facts-gaps` seat's prompt body, over `docs/history/<feature>/CONTEXT.md`,
approach.md, plan.md, and the drafted cells
(`.bee/bin/bee cells list --feature <feature>`). Assume the work is flawed
until proven so. Verify exactly 5 dimensions:

1. Requirement/decision coverage — every locked decision lands in at least one cell.
2. Cell completeness — each cell has files, read_first, directive action, must_haves
   (per lane tier), and a testable exit the declared suite can prove.
3. Dependency correctness — deps form a DAG; no cell depends on a future slice.
4. Key links — integration points named in plan.md are owned by a specific cell.
5. Scope sanity — no cell is doing hidden architecture work or exceeds its lane.

Every structural finding reports as BLOCKER (structurally unsound) or
WARNING (survivable, note it).

**MANDATE 2 — Cells, cold pickup.** The leader's own self-check at cell
drafting, never a seat's job. Read each cell with NO session history: could
a worker who has read only CONTEXT.md, plan.md, and this cell implement and
verify it without guessing?

- CRITICAL: assumed context, vague acceptance, scope overload, unproven
  feasibility, an exit the declared suite cannot prove.
- MINOR: missing rationale, implicit file assumption, fuzzy boundary, known
  tradeoff not recorded.

Findings quote file/cell evidence, are never softened, and never propose a
redesign. The synthesis carries both sections and never merges the two
vocabularies:

```text
PLAN CHECK
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
judgment, and red-before-green: `.bee/expertise/tests.md`. The writer
picks and runs the proof its cell needs and records it as the cap's
proof line — the full rule under "Test scoping" above.
