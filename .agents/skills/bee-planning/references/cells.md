# Cells — Authoring, Pre-flight, and Verification

Use when `bee-planning` drafts cells, runs pre-flight, scopes `verify`,
selects a playbook, or presents a tiny/small merged gate.

## Playbooks

Each playbook binds one **route class** — the value `bee route --set --class`
records (`bee-hive/references/scout-and-ticks.md` ("Route record")). It does
NOT bind the cell-level `change_class` enum
(`packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:123`), which is a
different taxonomy that overlaps this one on `bugfix` and `refactor` only — a
cell's `change_class` never selects a playbook.

Every route class has one index file, at
`skills/bee-planning/playbooks/<class>.md`; a class with situations lists
them there, each in its own file at
`skills/bee-planning/playbooks/<class>/<situation>.md`:

- [perf](../playbooks/perf.md)
- [bugfix](../playbooks/bugfix.md)
- [refactor](../playbooks/refactor.md)
- [research](../playbooks/research.md)
- [feature](../playbooks/feature.md)
- [docs](../playbooks/docs.md)
- [release](../playbooks/release.md)
- [spike](../playbooks/spike.md)
- [content](../playbooks/content.md)

**How a plan uses one.** The plan **cites** the situation file by path —
`skills/bee-planning/playbooks/<class>/<situation>.md` — and the class index
(`skills/bee-planning/playbooks/<class>.md`) only for a class with no
situations; it never transcribes the steps into `plan.md`. The steps live in
that file, in one home, and are read there; a copied list goes stale and can
be satisfied by transcription. A step that does not apply stays VISIBLE and
carries its recorded reason — "named deviation is the system working"
(`AGENTS.md` ("Judgment and deviation")). A skipped step is never a refusal:
nothing in a playbook blocks anything.

**Which principles a class routes.** `bee orient` names them in the session
preamble, read from the `## Principle homes` section of
`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`.
Those `classes:` lines are the one home of that mapping; a playbook never
restates them.

## Cell quality rules

A cell is an executable prompt a cold worker can pick up with zero session
history.

1. **Directive action, no code blocks — the contract, not the
   implementation.** Prose that says what to do and cites decisions
   (`per D2`). Code belongs in the repo, written by the worker. The action
   names the outcome and the contract the cell must keep — inputs,
   outputs, persisted state, error shape, the tests that prove it — and
   leaves internal structure (helper names, call graph, variable
   placement, line-level edits) to the worker unless that choice crosses
   a shared contract. Read anchors that say where to look stay. A detail
   the plan never proved turns into a constraint the moment the action
   prescribes it.
2. **Bounded files.** `files` lists everything the worker may write;
   `read_first` what it must read, citing paths from CONTEXT.md's
   `## Read set` first. Cross-cell file overlap is legal, not a
   scoping error — it only costs a wave (auto-serialized); prefer explicit
   paths or trailing-`*` patterns, since overlap detection treats mid-path
   globs as literals.
   **Carry the hits, not a line range.** A `read_first` entry or an action
   that names a span inside a large file — `tests.rs:2045-2549`, or a bare
   list of line numbers — is a read instruction, and the worker follows it
   literally. Paste the `rg -n` output into the cell instead: the matching
   lines verbatim, each with its number. Line numbers ride ALONGSIDE their
   anchor text, never alone — they drift the moment an earlier edit lands,
   and a worker that can only count lines cannot recover. Never instruct a
   worker to work by line number *instead of* searching: when several sites
   look identical, that is a reason to carry more anchor — the enclosing
   function, the occurrence index, the neighbouring line — not a reason to
   abandon the search.
3. **Testable exit.** The cell's outcome is provable by the proof its
   writer will run and record at cap time (related tests for code, a
   parity/pointer check for docs, a judge verdict for behavior) — plan the
   cell so that proof exists by cap time (rule: agents-proof-at-cap).
   "Manually check" is not an exit.
4. **must_haves are contracts:** `why` (non-empty string: the reason the
   work matters), `out_of_scope` (array of strings, `[]` when none: what
   the worker must leave alone), `truths` (observable behavior),
   `artifacts` (path + substantive description — no stub counts),
   `key_links` (wired, not just existing), `prohibitions` (what must NOT
   change). Required for `standard`/`high-risk`; `tiny` may omit.
   Optional `open_choices` lists the plan's current design choices the
   user did not ask for — a worker may question one with evidence; the
   goal-check never reads it as a truth.
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
   affected) required on every lane.
10. **Role is required; the name is guidance, not an enum.** Every cell carries
    `role`, the job this work is — the sole model selector, required exactly as
    `lane` is (`bee cells add` refuses without it). The recommended vocabulary —
    `code`, `read`, `test`, `docs`, `review`, `design` — is authoring guidance
    only; any non-empty name is legal, validation checks presence and shape,
    never membership. A role nothing in `team.<runtime>` configures still
    runs: it falls through to the next name the dispatch asks for and warns,
    it never fails. The one silent case is `code` or `read` on a runtime whose
    `team.<runtime>` configures NEITHER of them — the pre-roles window, where
    falling through is the intended no-op; configuring either key closes it.
11. **Optional `fixes_feature: <slug>` for fix-first cells.** A fix-first cell
    opened under the known-red-base rule (`rule: agents-never-build-on-red`,
    `bee-hive/references/gates-and-delegation.md`) sets the optional top-level
    `fixes_feature` field to the slug of the feature whose defect it fixes, so
    `bee report outcomes --by-feature` can count it (`outcome-scorecard` D4).

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
    "why": "Unauthenticated requests currently reach /api/* handlers without a session check.",
    "out_of_scope": ["Public /healthz routes", "Token refresh flow"],
    "truths": ["Unauthenticated /api/* requests return 401"],
    "artifacts": [{"path": "src/auth/middleware.ts", "substantive": "exports authGuard, no TODO stubs"}],
    "key_links": ["router.ts imports and mounts authGuard"],
    "prohibitions": ["No change to public response envelope"],
    "open_choices": ["Mount the middleware per route group instead of once at the app root"]
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
   `"none"` sentinel). `role` non-empty (Cell quality rule 10).
3. **Lane.** One of `tiny`/`small`/`standard`/`high-risk`/`spike`;
   `standard`/`high-risk` cells carry non-empty `must_haves.truths`, plus
   non-empty `must_haves.why` and an `out_of_scope` string array (empty
   allowed) checked at the plan-packet preview (`bee gate --preview`).
4. **Scope-derived obligations.** Any `files` path under a release-manifest
   or onboarding-ledger root obliges `verify` to carry `bee dev
   release-manifest --check`, `files` to carry the manifest record, and the
   cell to run the regen chain (`bee dev regen`) — or a reasoned
   `regen_obligation_ack` (recognized value `"wave-barrier"` defers to wave
   close). A guard-source path on a lane below `standard` obliges
   `judge_obligation_ack` or a raised lane.
5. **Deps and slice.** `deps` are acyclic; current slice only; the
   feature's execution gate is approved.
6. **Approved preview binding.** Incoming cells must match the approved
   preview packet recorded at gate approval. Cell IDs must match, and execution
   fields (`action`, `verify`, `files`, `read_first`, `must_haves`, `title`,
   `lane`, `role`) must match identically. Extra or missing cells refuse the
   batch.

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

The agent owns test scope end to end (rule: agents-proof-at-cap).
`commands.test` is the project's ONE declared test command, and it stays what
CI runs on every push, the one deterministic net. A host keeps CI fast by
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
needs it. `bee cells add` and `bee cells update` refuse a verify that is
the bare declared suite (or a bare `bee test`) and print a scoped form to
use; the full suite runs once per feature, as `bee test` before the merge
(rule: agents-proof-at-cap). A cross-cutting change whose proof really is
the whole suite sets a non-empty `full_suite_reason` on the cell, which
allows it and stays on the cell. A cell whose change is docs or prose
carries a parity/pointer check as its `verify`, not a test run at all.

## Greenfield init lane

When the repo has no build and the init-lane offer was accepted at
onboarding, the first slice is **one init cell** — `must_haves`: setup
succeeds from scratch, one passing test exists, standard commands recorded
in `.bee/config.json`, clean first commit — before any feature cell. Its
proof is the recorded test command (`commands.test`) running green,
recorded on the cap and checked at the boundary (`bee close`/`bee
worktree merge`); the suite is one test then, so the cell carries a
`full_suite_reason`.

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
