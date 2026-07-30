# Worker Details

Open this when the compact worker loop needs exact fields or commands.

## Parent Context

The orchestrator supplies: agent nickname (reservation identity), assigned cell id, feature name, paths to `CONTEXT.md` and `plan.md`, global constraints, model tier, and the status-token protocol. Nothing else arrives — if the cell is not executable from that plus the repo, return `[BLOCKED]`; do not guess.

The assigned cell arrives **already claimed** under the worker's nickname — the orchestrator claims before spawning (D1), never the worker. No literal session id is ever handed down in the prompt (D3): reservation and claim verbs resolve the session from `CLAUDE_CODE_SESSION_ID` in the worker's own environment when one is needed, never from prompt text.

## Feature-verify pending path (default, main-verifies D1/D4)

The sanctioned default: implement, commit, cap with `--feature-verify-pending` — no per-cell verify run, no `verification_evidence`. `cells cap` refuses combining this flag with a recorded passing verify or with per-cell evidence (the two proof paths are exclusive). The full feature-verify protocol — when the ONE feature verify runs, what proves it, how it's recorded, and the red path — is stated once, in full, at `bee-swarming/references/swarming-reference.md` ("Feature verify at close, in full"); this is the orchestrator's job, never the worker's. Bugfix cells: the repro red already exists (MAIN-produced pre-dispatch, cited in the cell) — fix it, never re-run it as "proof". The classic evidenced path below stays sanctioned for spot use, other repos, or transition.

## Expanded Commands

Startup runs ZERO of these (exec-speed D7): the dispatch prompt inlines the cell JSON and the state line — `status --brief` and `cells show` are post-compaction recovery verbs only, never startup verbs.

```text
node .bee/bin/bee.mjs reservations reserve --agent "<name>" --cell "<id>" --path "<path>" --ttl 3600
node .bee/bin/bee.mjs cells cap --id <id> --feature-verify-pending [--outcome TEXT] [--files a,b] [--deviations-file F] [--friction TEXT]
node .bee/bin/bee.mjs reservations release --agent "<name>" --cell "<id>"
node .bee/bin/bee.mjs decisions active --recent 3
```

Classic path only — record a per-cell verify before cap (never combine with `--feature-verify-pending`):

```text
node .bee/bin/bee.mjs cells verify --id <id> --command "<cmd>" --passed true|false [--output-file <f>]
node .bee/bin/bee.mjs cells cap --id <id> [--outcome TEXT] [--files a,b] [--behavior-change] [--evidence-stdin] [--deviations-file F] [--friction TEXT]
```

Shell guard for write-heavy commands (`git add/mv/rm`, `mv`, `cp`, `rm`, `mkdir`, `touch`, `sed -i`, `tee`, redirection writes):

```bash
BEE_AGENT_NAME="<name>" git add src/foo.ts
```

## Assigned Cell Check

For the one assigned cell, confirm before starting (D1 — the orchestrator claims before spawning; the worker only validates, never claims):

- the INLINED cell JSON in your dispatch prompt (exec-speed D7) shows `status: "claimed"` with `trace.worker` matching your nickname — a different worker, no claim, or any other status is not yours to touch; a prompt with no inlined cell JSON is malformed → `[BLOCKED]`
- all `deps` are capped
- `files` scope is clear and reservable
- the `verify` command is concrete and runnable in this repo
- referenced decision IDs resolve in `CONTEXT.md` and do not contradict the action

`[NOOP]` if the cell is missing or already done; `[BLOCKED]` for ambiguity, a locked-decision conflict, or an ownership mismatch.

## Conformance habits (worker-conformance D8)

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
   `[BLOCKED]`.

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
| any cell whose proof tier resolves to `red-first`, capping on the classic path | `verification_evidence` carrying `red_failure_evidence` is mandatory — `cap` refuses without it; pipe it via `--evidence-stdin` (no file written). On `--feature-verify-pending` the door is deferred to the feature verify, never waived |
| `behavior_change: true` below the `red-first` tier | evidence is accepted, not demanded — `cap` no longer refuses without it. Capping with neither verify output nor evidence records `trace.proof: "unrecorded"` and warns; see "Absent proof is recorded, not forgiven" below |

## Absent proof is recorded, not forgiven (worker-conformance D1/D12/D14)

Two cap refusals were retired: the `behavior_change: true` door that demanded
`verification_evidence`, and decision 0004's small+ door that refused a passing
verify recorded with no output. Both caps now succeed. **You are no longer asked
to author anything in order to pass a gate** — evidence is what the build emits
(a red run, a verify run, `git show`), never prose you compose at cap time.

The judgement did not soften. An assertion is still not evidence; it simply stops
being your refusal and becomes a recorded absence:

- Cap with **neither** real verify output **nor** `verification_evidence`, and the
  helper stamps `trace.proof: "unrecorded"` on the cell (predicate
  `cells.mjs:2231-2235`, stamped at `:2273`). Two different stderr warnings can
  announce it, and only one is lane-gated: the `behavior_change` warning
  (`:1969-1974`) has **no lane gate at all** and prints on `tiny` exactly as it
  does on `high-risk`, so a cell declaring `behavior_change` is essentially never
  marked in silence; decision 0004's warning (`:2188-2190`) is the lane-gated one
  (`small`/`standard`/`high-risk`), which leaves `tiny` **and `spike`** (both
  outside that gate, `LANES` at `:91`) taking the marker with nothing printed
  when the cell declares no `behavior_change`. One degenerate case slips both:
  whitespace-only `verification_evidence` reads as present at `:1969` (skipping
  the warning) but empty at `:2229-2230` (still marking). Silent or not, the door
  still knows.
- That marker arms the feature-boundary close-door alongside
  `trace.feature_verify: "pending"` — `featureVerifyDebt` refuses to leave
  `swarming` or run `scribing-run` until a green feature-verify record newer than
  the newest owed cap exists (`state.mjs:2526-2548`). **No `gate_bypass` level
  lifts it.**
- D14: "unrecorded" needs **both** channels empty. A cell holding genuine
  evidence — a `red_failure_evidence` that already passed the red-first door —
  is never marked, even with empty `verify_output`.
- Exempt: the explicit `--feature-verify-pending` path (it carries its own
  marker) and a repo declaring `commands.verify: "none"`.

So a silent cap is not a free cap. It moves the cost from your cell to the
feature's door, where main pays it with one real green run.

## What did NOT change (read this before assuming everything loosened)

No `gate_bypass` level lifts any refusal below. Which path you cap on does
matter, though, and the doctrine is honest about it: the doors marked
**both paths** fire whatever you do, and the ones marked **classic path** are
deferred — never waived — by `--feature-verify-pending`, because that flag hands
their proof to the feature-level verify and stamps a marker the close-door reads.

**Both paths:**

- **Claim ownership** (`cells.mjs:1874`, `guardClaimOwnership`) — a cap from a
  session that does not own the claim is refused. The cell is not yours to cap.
- **The two proof paths stay exclusive** — `--feature-verify-pending` combined
  with `verification_evidence` or with a recorded passing verify is refused
  (`:1892`, `:1897`). Read precisely: the whole check lives *inside*
  `if (pendingFeatureVerify)` (`:1887-1900`), so it is the pending path's own
  rule and can only fire when you pass the flag — nothing you pass alongside the
  flag escapes it, and the classic path has nothing here to trip over.
- **Non-empty `files_changed`** on lanes `small`/`standard`/`high-risk`
  (`:2191-2194`) — it asks what you touched, not for authored proof.
- **An outcome summary** on `high-risk` (`:2197-2200`).
- **A new test file on a `refactor`/`formatting` cell** — refused outright
  (`:1990-1998`), and `new_suite_reason` can never rescue it. Needing a new
  suite for a "refactor" means the cell is misclassified.
- **A `NEEDS_REVISION` semantic-judge verdict** (`:1926-1931`) without an audited
  `--override-judge` reason.

**Classic path (deferred, not waived, by `--feature-verify-pending`):**

Every door in this list is gated on `!pendingFeatureVerify`, and the pending
path is the **default** — so on the sanctioned path none of them fire at your
cell at all. Their proof moves to the feature verify; it does not evaporate.

- **A recorded passing verify** (`:1913-1917`) — cap without
  `trace.verify_passed === true` is refused **on the classic path only**; the
  condition literally opens with `!pendingFeatureVerify`, and a further
  exception is a `verify: "none"` cell in a declared no-test repo
  (`:1909-1911`). What was retired is the demand for *output alongside* the
  pass, never the pass itself.
- **Red-first** (`:2118-2139`, `:2150-2170`): `red_failure_evidence` ≥80 chars,
  non-duplicate, or an explicit `deliberate_exceptions` door. It fires wherever
  `requiredProofTier` (`cells.mjs:163-186`) resolves to `red-first`:
  `security`/`migration` in **every** lane, and `bugfix`/`behavior`/`api` at
  lane `high-risk` (an undeclared `change_class` on a `behavior_change: true`
  cell derives to `behavior` and lands here too). **Narrower than "high-risk,
  all classes":** at `high-risk`, `refactor` and `formatting` still resolve to
  `suite-green` and `test` still resolves to `targeted-green` — the lane alone
  does not buy red-first. Both doors sit out the default path: `:2150` is gated
  on `!pendingFeatureVerify` outright, and `:2118` can only fire when
  `verification_evidence` was supplied, which the pending path refuses by
  construction (`:1891-1895`).
- **`new_suite_reason` ≥20 chars** for a genuinely new suite file
  (`:2013-2028`) — gated on `!pendingFeatureVerify`, because its justification
  channel *is* per-cell evidence.
- **Ratio ceiling** (`:2044-2060`): tiny/small warn above 3, standard/high-risk
  refuse above 4 without a `ratio_waiver` ≥20 chars. On the pending path the
  same overage is recorded as a visible warning instead — the waiver channel is
  per-cell evidence, which that path refuses by construction.

Deferral is not a discount: a pending cap carries `trace.feature_verify:
"pending"`, and the feature cannot close until one real green feature verify
answers for it.

## Friction Triggers (verbatim — record friction only when one fires)

- had to infer a missing rule
- validation unclear/too expensive
- stale or contradictory doc
- repeated manual step that should be a template
- out-of-scope but important
- unattributable failure

One line per trigger, factual, in `--friction` (or the deviations file for multiples). No trigger fired → leave friction empty; do not invent process commentary.

## verification_evidence Example

Piped via `--evidence-stdin` on cap wherever evidence is supplied — **mandatory at the `red-first` tier, accepted but never demanded below it** (the evidence goes straight into `trace.verification_evidence` — **no file is written**):

```bash
node .bee/bin/bee.mjs cells cap --id <id> --files a,b --behavior-change --evidence-stdin <<'JSON'
{ ...the evidence object below... }
JSON
```

The evidence object:

```json
{
  "tests_inspected": ["tests/auth/middleware.test.ts"],
  "tests_added_or_changed": ["tests/auth/session-timeout.test.ts (new, 3 cases)"],
  "red_failure_evidence": "session-timeout.test.ts failed before the change: expected 401, received 200",
  "verification_run": "npm test -- auth -> 42 passed, 0 failed",
  "deliberate_exceptions": []
}
```

Every field is honest or explicitly empty with a reason in `deliberate_exceptions`. Vague evidence here becomes a P1 finding in bee-reviewing — the work comes back.

**`red_failure_evidence` is captured at cap time, not backfilled later (decision 0009).** Wherever `requiredProofTier` resolves to `red-first` — the door is keyed on the tier, not on the `behavior_change` flag (`cells.mjs:2118`, `:2150`) — the helper *refuses to cap*, **on the classic path**, unless the evidence names a "before": the prior behavior this change alters — a `git show <pre-change-commit>:<file>` extract, or a pre-change check that failed. If the surface is genuinely new (no prior behavior to characterize), say so in `deliberate_exceptions`. This is why the characterization is cheap to record now — the old state is one `git show` away while you hold the diff in context; recovering it after review means a whole extra evidence-only cell.

**Both red-first doors sit out the default `--feature-verify-pending` path**: `:2150` is gated on `!pendingFeatureVerify`, and `:2118` only fires when `verification_evidence` was supplied at all, which the pending path refuses by construction (`:1891-1895`). Deferred to the feature verify, never waived — dispatched on the default path you are not refused for its absence, and capping through the classic path (spot use, another repo, transition) is where it is owed at the cell.

**Scoped red-first (decision 8ef2bae6):** the red run that produces this "before" executes ONLY the test(s) this cell adds or changes — name the test file or filter in `tests_added_or_changed`, never the full configured suite. The full cell verify chain runs exactly once, at the end, right before cap; a full-suite run inside the red-first loop is the named waste this scopes away.

## Evidence lives in one place (decision 0009)

The cell **trace** is the single source of verification evidence: `trace.verification_evidence` (the JSON above) plus `trace.verify_output` (the recorded verify run). **Pipe evidence with `--evidence-stdin` so no evidence file is ever written — this stays the preferred path.** Do NOT create `reports/<cell-id>-evidence.json`, `reports/execution-*-evidence.md`, or any other on-disk evidence file — that is the exact duplication decision 0009 removed. (`--evidence-file` still exists only for back-compat; if you must use it, write to `.bee/tmp/<feature-or-session>/` — the one canonical scratch home, docs/specs/doctrine-layer.md R17 — pass it to cap, and delete it; never leave it in `reports/` or any other tracked path.) The per-cell report (below) *links and summarizes* the trace in one line; it never re-embeds it.

## Verification Failure

Fix the root cause and rerun the exact failing command. After two serious attempts, return `[BLOCKED]` with: the command, the failure summary, attempts made, your diagnosis, and the smallest useful next decision for the parent. A verify command that is itself broken in the repo is a `[BLOCKED]`, never a reason to cap with a substitute check.

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

Every result starts with exactly one token and includes, minimum: nickname, cell id, files touched/requested, reservation outcome (released yes/no), verification result, and the parent's next action. When the cell owes a report file (`[BLOCKED]`/`[HANDOFF]`/consult-carrying/explicit request — exec-speed D10), mirror the result into `docs/history/<feature>/reports/<cell-id>.md` as a short summary that **links** the cell (`.bee/cells/<cell-id>.json`) for the full trace and evidence — never a second copy of the `verification_evidence` JSON or the verify output (decision 0009: the trace is the single source).

When dispatched with native worktree isolation, also report the observed working
directory, symbolic ref (or detached state), and resulting commit. These values
are informational and never authoritative: the worker does not choose or attest
its integration identity. The orchestrator must derive and recheck identity from
its protected pre-dispatch attestation and fresh Git metadata, then independently
prove base ancestry and the reserved-path diff subset before the result counts.
Do not describe a branch name, worktree id, base, or commit as integration
authority, and do not ask the orchestrator to trust a worker-supplied value.

- `[DONE]` — cell capped (pending path: no per-cell verify; classic path: verification recorded as passed), one commit made, reservations released.
- `[BLOCKED]` — cannot continue safely; include the blocker, diagnosis, and current reservation state.
- `[HANDOFF]` — `.bee/HANDOFF.json` written; include progress, active reservations, and the resume point.
- `[NOOP]` — the assigned cell is unavailable or unsafe; include why and a suggested parent action.

Ambiguities you deferred go in an `Outstanding Questions` section of the report.

## Evidence Report Budget

A per-cell report file is CONDITIONAL (exec-speed D10): routine `[DONE]` cells write none — the cap trace + status-token message are the record. A report is owed only for `[BLOCKED]`/`[HANDOFF]`, consult-carrying cells, or on explicit orchestrator request. When one IS written, `docs/history/<feature>/reports/<cell-id>.md` targets **<=40 lines**. Structure:

- **Outcome** (1-3 lines) — status token + what changed, in plain language.
- **Verify** — the exact command, plus its decisive output lines, quoted, **<=10 lines**.
- **Files + commit** (<=5 lines) — files touched and the commit hash.
- **Deviations** (<=5 lines) — one line each, only if any fired.
- **Side-by-side excerpts** — only when the cell explicitly demands them (e.g. a before/after diff the reviewer can't get any other way).

Raw full output (verify logs, evidence JSON) never goes inline — point to `.bee/cells/<cell-id>.json` (the `trace`) instead, per decision 0009.

**Soft budget:** going over 40 lines is allowed but requires a one-line reason at the top of the report (e.g. "high-risk cell, full trace required").

## Post-Compaction Recovery

Reread, in order:

1. `AGENTS.md`
2. `docs/history/<feature>/CONTEXT.md`
3. `node .bee/bin/bee.mjs cells show --id <id>`
4. `node .bee/bin/bee.mjs reservations list --active-only`

## Verify in full

Classic/spot-use path (main-verifies D1) — the sanctioned default is the feature-verify pending path above; a cell only reaches this section when it is explicitly cited for spot use, another repo, or transition.

- Run the cell's verify command exactly, then record it **with its output** (decision 0004 — proof, not assertion):
  `node .bee/bin/bee.mjs cells verify --id <id> --command "<cmd>" --output "<what it printed>" --passed true|false` (or `--output-file <f>` for long output)
  - **The recorded pass is still required; the output is no longer enforced** (worker-conformance D1). A cap on the classic path with no passing verify is refused outright (`cells.mjs:1913`), but a pass recorded with empty output no longer refuses on small+ lanes — it caps, warns, and marks `trace.proof: "unrecorded"`, which arms the feature close-door. An assertion is still not evidence; skipping the output does not save you the run, it just relocates the bill. See "Absent proof is recorded, not forgiven".
- The `verify` field is the cell's **targeted** suite (seconds) — never the full configured `commands.verify` chain (D4, decision `e54878b1`). Run red-first, then green, and stop there: do **not** additionally run the full chain yourself. The orchestrator no longer re-runs it routinely either (main-verifies D4 retired the wave-close impacted run) — a fresh re-run happens only on a smell, and the feature's ONE proof event is the feature verify at final-slice close (`bee-swarming/references/swarming-reference.md`, "Feature verify at close, in full"). The full chain itself is CI-owned: no local baseline run before the first claim, and worktree merge gates on `commands.test` (impacted over the staged merge) instead of the full chain — session finish likewise runs `commands.test`, and the release flow runs the impacted suite locally then dispatches the CI full run (`gh workflow run CI --ref main`) right after the tag push, with red arriving back as a `verify-red` issue (`AGENTS.md`). Mid-iteration, run the level-1 impacted run (`run_verify --impacted-from-git --level 1` — direct edges only, seconds) if you need one; it is never owed by default.
- **Proof-tier matrix (test-economy D1/D2).** How much proof a cap demands is no longer one uniform red-first rule for every `behavior_change` cell — it is derived from the cell's `change_class × lane`, resolved by `requiredProofTier` (`packages/bee/lib/cells.mjs`):

  | change_class | lane | tier | what capCell accepts |
  |---|---|---|---|
  | `refactor` / `formatting` | every lane, **including `high-risk`** | `suite-green` | the existing suite passing is proof enough; a **new** test file in the diff is refused outright — `new_suite_reason` (D3, below) can NOT override this. Needing a new suite for a "refactor" means the cell is misclassified. |
  | `behavior` / `api` | `tiny` / `small` / `standard` | `existing-targeted-green` | the cell's targeted scope of the **existing** suite passing — **author no new test here** (slice-tail-test-batching P1, spec #80/#85): that moves to the slice's one trailing `test` cell. `verify` is still a runnable command recorded with output, so the cap still proves you didn't break what exists. At `tiny`/`small`, when the behavior is NOT a public contract and carries no hard-gate flag, a **verified transcript** is equally legal proof (proof-economy, decision 2bb2cb27): run the real command that demonstrates the behavior and record it with its decisive output via `cells verify` — no new test authored; the trailing cell's coverage judgment still runs at slice close. |
  | `bugfix` | `tiny` / `small` / `standard` | `targeted-green` | **unchanged — repro-first stays**: write the failing repro **before** the fix. It is diagnosis evidence, not coverage ceremony; P1 amended `behavior`/`api` only. |
  | `test` (the slice's consolidated cell) | every lane | `targeted-green` | its own new suite passing over the slice's **net behavior** — happy path, edge cases, error paths — for the surfaces the slice's cells declared. Not per-cell internals. |
  | `bugfix` / `behavior` / `api` | `high-risk` | `red-first` | the scoped red-first below still applies in full: `red_failure_evidence` ≥80 chars, anti-duplicate floor. |
  | `security` / `migration` | every lane, tiny through high-risk | `red-first` | same as above — no lane ever softens this row. |
  | unclassified (`change_class` null), `behavior_change: false` | any | *(no matrix check)* | today's pre-test-economy behavior, untouched — claiming a lighter tier than `behavior` requires declaring `change_class` explicitly; the null default never gets a discount. |
  | unclassified (`change_class` null), `behavior_change: true` | any | derived as `behavior`, then tiered per the `behavior` row above | same acceptance as the row it derives into |

  **Amendment note:** decisions `e54878b1`/`8ef2bae6` (scoped red-first) are **amended, not reversed**, by test-economy D2 and again by slice-tail-test-batching P1. The *shape* they defined — a red run before the fix touches ONLY the test(s) this cell adds or changes — still governs every `red-first` row, and repro-first still governs `bugfix`. What P1 moves is *when new coverage is authored*, never *whether the question gets asked*: a `behavior`/`api` cell outside `high-risk` caps on existing-green, and the trailing `test` cell owes the **coverage judgment** — cite the nearest existing tests by `file:line` and state whether they already cover the net behavior. **A test cell that authors no test is not a defect**; a "covered" verdict caps by running the cited tests green and recording "already covered, no new rows". The door is keyed on the **feature**, not the slice (`testCellDebt(root, feature)`, `state.mjs:2570`), and it refuses the close in two distinct ways: no `test` cell anywhere in the feature while capped code-touching `behavior`/`api` cells exist (`:2653-2654`), or a `test` cell that is still open/claimed/blocked, capped with `verify_passed: false`, or capped with `trace.proof: "unrecorded"` (`:2604-2615`, `:2656-2657`) — the second kind fires on the offending cell alone. A CLI throw no bypass level lifts (P4), owed only when the feature touched CODE: instruction/knowledge text is not code and owes no test (user law 2026-07-27). Regression catching comes from *running* the existing suite at every cap; that is unchanged.

- **Scoped red-first (decision 8ef2bae6), wherever the matrix above resolves to `red-first`:** the red run before a fix executes ONLY the test(s) this cell adds or changes — name the test file or filter, never the full configured suite. The cell's own `verify` chain (however many checks it strings together) then runs exactly once, at the end, right before cap, to prove the green state. A full-suite run inside a red-first loop is the named waste this scopes away.
- **Test-shape rules (test-economy D3)** — apply whenever this cell adds or changes tests, independent of which proof tier applies:
  - ≥3 cases exercising the same behavior are table-driven, not copy-pasted near-duplicates.
  - A new scenario for behavior the suite already covers is a new **row** in the existing suite, not a new file.
  - A genuinely new `test_*.mjs` file must declare `new_suite_reason` (≥20 chars, in the evidence JSON) explaining why a new permanent CI suite is warranted — cap refuses without it **on the classic path only** (`cells.mjs:2013-2028` opens with `!pendingFeatureVerify`): the justification channel is per-cell evidence, which the default pending path refuses by construction, so there the scrutiny moves to the feature verify and the trailing test cell's own review. For a `refactor`/`formatting` cell nothing rescues a new test file — that refusal (`:1990-1998`) fires on **both** cap paths, `new_suite_reason` included; the one thing that skips it is an absent `diff_stats` (`:1992`, fail-open by design, the same way `:2017` is), never a flag you pass. See the matrix above.
  - Test-to-source line ratio has a ceiling: `tiny`/`small` warn above 3, `standard`/`high-risk` refuse above 4 unless the evidence declares a `ratio_waiver` (≥20 chars) — shrink the diff or justify the ratio. **That refusal is classic-path only**: on the default `--feature-verify-pending` path the same overage is recorded as a visible warning in `trace.warnings` instead (`cells.mjs:2047-2052`), because the waiver channel is per-cell evidence, which that path refuses by construction. At the slice `test` cell, measure against the **slice's aggregate** source delta (the sum of its implementation cells), never that cell's own near-zero delta.
- **Read-first, before writing a new test (test-economy D5):** before adding a test, cite the nearest existing test that already exercises this area and say in one line why it doesn't already cover the new case (fold it into `tests_inspected`/evidence). Reading first is cheap; a duplicate suite born from not looking first is not.
  - **Falsifiability proof is scoped (test-runs-lean D2):** proving a new suite load-bearing by mutation (neuter the guard, watch the red, restore) is owed only when the suite guards `high-risk`/hard-gate behavior — and then at most ONE mutation cycle. Everywhere else it is optional, and skipping it is never a cap defect: the scarce resource is suite runs, not assurance theater.
- **Cap evidence is scoped, not full (verify-scoping D2, decision `20534ea9`).** Cap evidence is the cell's own scoped verify passing — never run the full configured verify yourself just to cap. **Verify-once (test-runs-lean D1, user feedback 2026-07-27):** in a serial `tiny`/`small` dispatch the worker's recorded verify output IS the cap evidence — the orchestrator does not repeat the same command by default; it re-runs only when the report smells (missing or garbled output, a pass claimed without the command's tail), when workers ran in parallel, or when the cell is `high-risk`/hard-gate. Proof stays proof — output recorded, never asserted — it just is not paid for twice. The full run belongs to close, not to caps.
- **In a declared no-test repo (decision 55b951e1), verify is `"none"`.** When `commands.verify`/`commands.test` in `.bee/config.json` carries the sentinel `"none"`, a cell's `verify` field may legitimately be `"none"` too — there is no command to run. Cap evidence there is the diff-backed outcome plus the recorded waiver note (`cells cap` auto-fills "no-test repo: verification waived by repo declaration" when none is supplied); never invent a fake check to satisfy the field, and never write `verify: "none"` in a repo that has not declared itself no-test — `addCell`/`updateCell` refuse it there.
- The `verify` field must be a runnable command, and a **targeted** one. A prose description instead of a command is a planning defect — return `[BLOCKED]` naming it; never invent a substitute check. So is a `verify` carrying the impacted or full chain (`run_verify`, `commands.test`, `commands.verify`): return `[BLOCKED]` the same way — the impacted run belongs to the slice close, never inside a cell (test-runs-lean D1).
- On failure: fix the root cause and rerun the exact command.
  - **No `Advisor` line in the dispatch:** unchanged — after **two serious failed attempts**, return `[BLOCKED]` with the command, failure summary, and diagnosis. A broken verify command in the repo is itself a blocker — never substitute a weaker check and cap anyway.
  - **An `Advisor` line is present in the dispatch:** the first serious failed attempt does not fall straight to a bare second retry — see **Advisor Consult** below. Two serious failures with no consult budget remaining still end in `[BLOCKED]`, same as the unchanged rule.

## Advisor consult in full

D1 amends the two-attempts rule above with a worker-level, on-failure-only step. This is **not** a gate-time or orchestrator-level consult — de967733 ("Bee runs ONE cost pattern") stays amended, not reversed: fan-out orchestration remains the default for every phase, and the human gates are untouched.

**Trigger** — consult only when both are true: the dispatch prompt carries an `Advisor` line (the orchestrator already ran the same-model no-op check per AO4/AO5 before adding one — the worker never self-assesses this), and the worker has just hit its **first serious failed verify attempt**. No `Advisor` line → proceed exactly as the unchanged rule in Verify.

**Canonical loop (D3), max 2 consults per claim:**

```
fail 1 -> consult 1 -> advised retry
  -> (fail) -> consult 2 (follow-up, same advisor) -> final retry
    -> (fail) -> [BLOCKED] with a Consults section (both consults summarized)
```

A re-dispatched cell (rescue rung) starts a **fresh** budget — the 2-consult cap is per claim, not per cell lifetime. Consulting after `[BLOCKED]` has already been returned for the current claim is never permitted.

**Evidence bundle — two shapes (exec-speed D11):**
- **On-failure consult (this loop):** exact failing command, the failing output, your diagnosis, the relevant cited file excerpts, and the `CONTEXT.md` path — the advisor is debugging with you, it needs the evidence.
- **Gate-time consult (the unconditional high-risk/hard-gate pre-cap consult, bee-executing step 6):** a COMPACT DIGEST only — cell id, one-paragraph change summary, file list with a one-liner per file, the `CONTEXT.md` path. Never full file excerpts: nothing failed, the advisor is sanity-checking shape and risk, and the semantic judge plus the feature verify independently backstop correctness.

Either shape passes **inline in the consult prompt or via stdin — never a `/tmp` path** (critical pattern 20260708). Never include secrets or env values.

**Transport** — the `Advisor` line names the advisor and how to consult it:
<!-- bee:only claude -->
- **Model-shaped advisor:** consult via your own Agent tool, with the model param set to the named advisor model, and the dispatch `description` starting **exactly** `advisor-consult <cell-id>: <advisor-model>` — this is the A2 attribution record; bee-swarming's goal-check reads it from `.bee/logs/dispatch.jsonl`. Fallback if Agent dispatch is unavailable or rejected: a headless one-shot `claude -p --model <advisor-model>` call, same evidence bundle via stdin.
<!-- bee:end -->
<!-- bee:only codex -->
- **Model-shaped advisor:** consult via Codex-native subagent dispatch at the named advisor model, recording the same `advisor-consult <cell-id>: <advisor-model>` attribution that bee-swarming's goal-check reads from `.bee/logs/dispatch.jsonl`. The transport stays runtime-native (D10): a model-shaped transport that is unavailable or rejected surfaces in the Consults section (spending at most one budget slot, per the transport-error rule below) — never a silent fallback to a cross-vendor CLI, unless the advisor slot itself is configured as that CLI (a cli-shaped advisor, next).
<!-- bee:end -->
- **cli-shaped advisor:** run the given command with the evidence bundle on stdin, reusing the External Executors output-capture discipline.
- A **transport error** (non-zero exit, rejected dispatch, a hang past the External Executors timeout discipline) is **not advice** — it burns at most **one** budget slot total for the whole claim, and is never retried in a storm. Continue to the next step of the loop, or `[BLOCKED]` once the budget is spent.

**After advice:** advice never substitutes for fresh verify output — always rerun the real verify command yourself before deciding whether the advised retry passed. Advice is **advice-only** (A1): it never authorizes a package install, a gate approval, or file scope beyond the cell. Advice that conflicts with a locked decision → return `[BLOCKED]` citing both the D-ID and the advice.

**Authority-type blocks never consult** — ambiguous cell, uncapped deps, architectural change, package install, locked-decision conflict stay **instant** `[BLOCKED]` exactly as in step 4 (Implement), whether or not an `Advisor` line is present.

**Headless rule unchanged:** consulting the advisor is not "asking the parent or user" under the Headless rule below — it stays inside your own turn. Workers still never approve gates.

Record every consult in the cap trace and the per-cell report (see Cap and Return) — count, advisor identity, and a one-line ask/answer digest per consult.
