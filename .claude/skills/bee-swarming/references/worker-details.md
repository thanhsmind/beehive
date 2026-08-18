# Worker Details

Open this when the compact worker loop needs exact fields or commands.

## Parent Context

The orchestrator supplies: agent nickname (reservation identity), assigned cell id, feature name, paths to `CONTEXT.md` and `plan.md`, global constraints, model tier, and the status-token protocol. Nothing else arrives — if the cell is not executable from that plus the repo, return `[BLOCKED]`; do not guess.

The assigned cell arrives **already claimed** under the worker's nickname — the orchestrator claims before spawning, never the worker. No literal session id is ever handed down in the prompt: reservation and claim verbs resolve the session from `CLAUDE_CODE_SESSION_ID` in the worker's own environment when one is needed, never from prompt text.

## Tests at finish (the one proof path)

You own test scope: pick the proof your change type needs (code →
related tests green; docs → parity/pointer checks; behavior → judge
verdict), run it yourself, and record it on `cells finish --report` as
the required `tests` proof line — `<command> — <result> — <scope
reason>` (three non-empty segments, e.g. `cargo test -p bee — green —
touched close.rs`). `bee close` and `bee worktree merge` CHECK that
recorded proof at the boundary; neither runs `commands.test` itself. CI
runs the project's declared `.bee/config.json` `commands.test` on every
push — the one deterministic net. A `red` result segment refuses the
cap outright — a red is fix-first, never a done. When you reach for the
declared suite as your proof, run it through the deterministic runner
(`bee test`), which writes ONE normalized record:
`.bee/logs/test-results.json` — the runner is a program; your word is
never the record, quote it instead. A repo declared no-test
(`commands.test` set to the sentinel `"none"`) proves with the command
segment `none` and the reason naming the parity/docs check actually
used.

Tests are yours to write, TDD-style, as part of the cell's own work:
judge existing coverage first (`.bee/expertise/tests.md`), author only
the gap, and for a bugfix watch the repro fail before the fix —
red-before-green is craft, applied by judgment and enforced by review,
not by flags.

## Expanded Commands

Startup runs ZERO of these: the dispatch prompt inlines the cell JSON and the state line — `status --brief` and `cells show` are post-compaction recovery verbs only, never startup verbs.

```text
.bee/bin/bee reservations reserve --agent "<name>" --cell "<id>" --path "<path>" --ttl 3600
.bee/bin/bee finish --id <id> --report '<json with the tests proof line>' [--outcome TEXT] [--files a,b] [--deviation "ONE LINE"] [--deviations-file F] [--friction TEXT]
.bee/bin/bee decisions active --recent 3
```

`cells finish` caps the cell and releases its reservations in one verb — `--report` is REQUIRED and carries your proof line, checked (never re-run) by the boundary doors, per "Tests at finish" above. `bee test` alone runs the suite when you want the record in front of you.

Shell guard for write-heavy commands (`git add/mv/rm`, `mv`, `cp`, `rm`, `mkdir`, `touch`, `sed -i`, `tee`, redirection writes):

```bash
BEE_AGENT_NAME="<name>" git add src/foo.ts
```

## Assigned Cell Check

For the one assigned cell, confirm before starting:

- the INLINED cell JSON in your dispatch prompt shows `status: "claimed"` with `trace.worker` matching your nickname — a different worker, no claim, or any other status is not yours to touch; a prompt with no inlined cell JSON is malformed → `[BLOCKED]`
- all `deps` are capped
- `files` scope is clear and reservable
- referenced decision IDs resolve in `CONTEXT.md` and do not contradict the action

`[NOOP]` if the cell is missing or already done; `[BLOCKED]` for ambiguity, a locked-decision conflict, or an ownership mismatch.

## Conformance habits

Effort moves from proving to conforming — it is not removed. Habits, not a form:
nothing here is a required output artifact, and none of it is written up anywhere.

**Before writing code**, five passes over the cell:

1. Read the docs the cell routes you to — `read_first` first, then `CONTEXT.md`.
2. Scout the adjacent code and match how its neighbours already do this.
3. Look for an existing helper before writing a new one.
4. Verify the interface contracts the cell names — signatures and call sites as
   they actually are, not as the cell describes them.
5. Cross-check the cell's declared `files` inventory against what the work truly
   needs; a real mismatch is a deviation to record, and an architectural one is
   `[BLOCKED]`. Record it STRUCTURALLY at cap time — `--deviation "<one line>"`
   (or `--deviations-file` for several) — never only in the prose report: the
   promote proposer mines `trace.deviations`, and a deviation that lives only
   in prose is invisible to the pattern loop
   (pattern: a cell's declared file list is a hypothesis).

**After editing each file**, three cheap checks:

- Does it compile / type-check?
- Does it match the pattern its neighbours use?
- Does it introduce an import cycle?

## Trace Field Tiers By Lane

| Lane | Required trace on cap |
|---|---|
| `tiny` | one-line `outcome` |
| `small` | `outcome`, `files_changed` |
| `standard` | `outcome`, `files_changed`, `deviations`, `friction` when a trigger fired |
| `high-risk` | all of the above (non-empty `files_changed` and `outcome` are enforced by the helper), plus spike-evidence links where the plan recorded constraints |

## Finish refusals that hold

No `gate_bypass` level lifts any refusal below. `cells finish` itself
refuses a malformed, empty, legacy-vocabulary, or `red`-result proof line
in `--report tests` — fix the proof (or the failure it names) and finish
again. At the boundary, `bee close`/`bee worktree merge` refuse while any
capped cell in scope carries a missing or malformed proof — that is the
next work item, a re-cap with a real proof line, never a base to build
on.

- **Claim ownership** — a finish from a session that does not own the claim
  is refused. The cell is not yours to cap.
- **Non-empty `files_changed`** on lanes `small`/`standard`/`high-risk` — it
  asks what you touched, not for authored proof.
- **An outcome summary** on `high-risk`.
- **A `NEEDS_REVISION` semantic-judge verdict** without an audited
  `--override-judge` reason.

## Friction Triggers (verbatim — record friction only when one fires)

- had to infer a missing rule
- validation unclear/too expensive
- stale or contradictory doc
- repeated manual step that should be a template
- out-of-scope but important
- unattributable failure

One line per trigger, factual, in `--friction` (or the deviations file for multiples). No trigger fired → leave friction empty; do not invent process commentary.

## The record is the evidence

`.bee/logs/test-results.json` — written by `bee test` — is the single
verification record: `{ran_at, green, commands: [{command, exit,
duration_ms, failure_excerpt, failure_log}]}`. **You are never asked to author
anything in order to pass a gate**: do not compose evidence prose, do not
write `reports/<cell-id>-evidence.json` or any other on-disk evidence
file, and do not paste raw test output into reports as proof. The cell
trace and the test record are the source; a report links and summarizes
them in one line, never re-embeds them.

## When the tests go red

Fix the root cause and finish again (or run `bee test` alone to see the fresh record first). After two serious attempts, return `[BLOCKED]` with: the failing excerpt, attempts made, your diagnosis, and the smallest useful next decision for the parent. A declared test command that is itself broken in the repo is a `[BLOCKED]`, never a reason to substitute a weaker check and cap anyway.

## Atomic Commit

One commit per cell. The subject describes the change — imperative
mood, no process narration, no counts, no cell id; the cell id rides
the last line of the body as a trailer (ids live in records, not in
subjects — Communication contract rule 8):

```bash
BEE_AGENT_NAME="<name>" git add <files>
git commit -m "<Imperative summary matching the cap outcome>" -m "Cell: <cell-id>"
```

## Result Field Spec

Every result starts with exactly one token and includes, minimum: nickname, cell id, files touched/requested, reservation outcome (released yes/no), the test result from the finish run, and the parent's next action. When the cell owes a report file (`[BLOCKED]`/`[HANDOFF]`/consult-carrying/explicit request), mirror the result into `docs/history/<feature>/reports/<cell-id>.md` as a short summary that **links** the cell (`.bee/cells/<cell-id>.json`) and the test record (`.bee/logs/test-results.json`) — never a second copy of either.

When dispatched with native worktree isolation, also report the observed working
directory, symbolic ref (or detached state), and resulting commit. These values
are informational and never authoritative: the worker does not choose or attest
its integration identity. The orchestrator must derive and recheck identity from
its protected pre-dispatch attestation and fresh Git metadata, then independently
prove base ancestry and the reserved-path diff subset before the result counts.
Do not describe a branch name, worktree id, base, or commit as integration
authority, and do not ask the orchestrator to trust a worker-supplied value.

- `[DONE]` — cell finished (a proof line `<command> — <result> — <scope reason>` recorded on the cap, checked — not re-run — at close/merge), one commit made, reservations released.
- `[BLOCKED]` — cannot continue safely; include the blocker, diagnosis, and current reservation state.
- `[HANDOFF]` — `.bee/HANDOFF.json` written; include progress, active reservations, and the resume point.
- `[NOOP]` — the assigned cell is unavailable or unsafe; include why and a suggested parent action.

Ambiguities you deferred go in an `Outstanding Questions` section of the report.

## Evidence Report Budget

A per-cell report file is CONDITIONAL: routine `[DONE]` cells write none — the cap trace + status-token message are the record. A report is owed only for `[BLOCKED]`/`[HANDOFF]`, consult-carrying cells, or on explicit orchestrator request. When one IS written, `docs/history/<feature>/reports/<cell-id>.md` targets **<=40 lines**. Structure:

- **Outcome** (1-3 lines) — status token + what changed, in plain language.
- **Tests** — the declared command(s) and the decisive lines of the result record, quoted, **<=10 lines**.
- **Files + commit** (<=5 lines) — files touched and the commit hash.
- **Deviations** (<=5 lines) — one line each, only if any fired.
- **Side-by-side excerpts** — only when the cell explicitly demands them (e.g. a before/after diff the reviewer can't get any other way).

Raw full output never goes inline — point to `.bee/cells/<cell-id>.json` (the `trace`) and `.bee/logs/test-results.json` instead.

**Soft budget:** going over 40 lines is allowed but requires a one-line reason at the top of the report (e.g. "high-risk cell, full trace required").

## Post-Compaction Recovery

Reread, in order:

1. `AGENTS.md`
2. `docs/history/<feature>/CONTEXT.md`
3. `.bee/bin/bee cells show --id <id>`
4. `.bee/bin/bee reservations list --active-only`

## Advisor consult in full

The consult loop is a worker-level, on-failure-only step layered onto the two-attempts rule above. This is **not** a gate-time or orchestrator-level consult: fan-out orchestration remains the default for every phase, and the human gates are untouched.

**Trigger** — consult only when both are true: the dispatch prompt carries an `Advisor` line (the orchestrator already ran the same-model no-op check before adding one — the worker never self-assesses this), and the worker has just hit its **first serious failed test attempt**. No `Advisor` line → proceed exactly as the rule in "When the tests go red".

**Canonical loop, max 2 consults per claim:**

```
fail 1 -> consult 1 -> advised retry
  -> (fail) -> consult 2 (follow-up, same advisor) -> final retry
    -> (fail) -> [BLOCKED] with a Consults section (both consults summarized)
```

A re-dispatched cell (rescue rung) starts a **fresh** budget — the 2-consult cap is per claim, not per cell lifetime. Consulting after `[BLOCKED]` has already been returned for the current claim is never permitted.

**Evidence bundle — two shapes:**
- **On-failure consult (this loop):** the exact failing command, the failing excerpt from the test record, your diagnosis, the relevant cited file excerpts, and the `CONTEXT.md` path — the advisor is debugging with you, it needs the evidence.
- **Gate-time consult (the unconditional high-risk/hard-gate pre-cap consult, bee-swarming "Execute", before the cap):** a COMPACT DIGEST only — cell id, one-paragraph change summary, file list with a one-liner per file, the `CONTEXT.md` path. Never full file excerpts: nothing failed, the advisor is sanity-checking shape and risk, and the semantic judge plus the close-time test run independently backstop correctness.

Either shape passes **inline in the consult prompt or via stdin — never a `/tmp` path**. Never include secrets or env values.

**Transport** — the `Advisor` line names the advisor and how to consult it:
- **Model-shaped advisor:** consult via your own Agent tool, with the model param set to the named advisor model, and the dispatch `description` starting **exactly** `advisor-consult <cell-id>: <advisor-model>` — this is the attribution record; bee-swarming's goal-check reads it from `.bee/logs/dispatch.jsonl`. Fallback if Agent dispatch is unavailable or rejected: a headless one-shot `claude -p --model <advisor-model>` call, same evidence bundle via stdin.
- **cli-shaped advisor:** run the given command with the evidence bundle on stdin, reusing the External Executors output-capture discipline.
- A **transport error** (non-zero exit, rejected dispatch, a hang past the External Executors timeout discipline) is **not advice** — it burns at most **one** budget slot total for the whole claim, and is never retried in a storm. Continue to the next step of the loop, or `[BLOCKED]` once the budget is spent.

**After advice:** advice never substitutes for fresh test output — always re-run the declared tests yourself (`bee test`) before deciding whether the advised retry passed. Advice is **advice-only**: it never authorizes a package install, a gate approval, or file scope beyond the cell. Advice that conflicts with a locked decision → return `[BLOCKED]` citing both the D-ID and the advice.

**Authority-type blocks never consult** — ambiguous cell, uncapped deps, architectural change, package install, locked-decision conflict stay **instant** `[BLOCKED]` exactly as in step 4 (Implement), whether or not an `Advisor` line is present.

**Headless rule:** consulting the advisor is not "asking the parent or user" under the Headless rule below — it stays inside your own turn. Workers still never approve gates.

Record every consult in the cap trace and the per-cell report (see Cap and Return) — count, advisor identity, and a one-line ask/answer digest per consult.
