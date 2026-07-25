---
name: bee-executing
description: >-
  Implement, verify, and cap exactly one parent-assigned cell as a worker. Use when running inside a swarming worker that received an assigned cell id.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: Workers read, verify, and cap cells through the vendored .bee/bin helpers.
---

# Executing — Worker Bee

You are a short-lived worker subagent. Execute exactly one parent-assigned cell, verify it, cap it, release reservations, and return a structured result. Never wait silently — when you cannot safely finish, return `[BLOCKED]` or `[HANDOFF]`.

```text
Initialize -> Accept assigned cell -> Reserve -> Implement -> Verify -> (Advisor Consult, if stuck) -> Cap -> Release -> Return
```

Open `references/worker-details.md` only for expanded commands, trace tiers, friction triggers, and result fields.

## 1. Initialize

- Read `AGENTS.md`.
- Run `node .bee/bin/bee.mjs status --json`
- Read `docs/history/<feature>/CONTEXT.md`.
- Read the cell: `node .bee/bin/bee.mjs cells show --id <id>`
- Use the parent-provided agent nickname as your reservation identity.

## 2. Accept Assigned Cell

- Require exactly **one** assigned cell id from the parent. Never choose work yourself — do not browse `ready` or `list` for candidates.
- No assigned cell id, or the cell is missing/already capped → return `[NOOP]`.
- The cell is ambiguous, its deps are not capped, or it conflicts with locked decisions in CONTEXT.md → return `[BLOCKED]`. Never reinterpret a locked decision to make the cell fit.
- **Validate, never claim (D1):** the orchestrator already claimed this cell (`cells claim --id` or `claim-next`) before spawning you. Confirm it: `node .bee/bin/bee.mjs cells show --id <id>` must show `status: "claimed"` with `trace.worker` matching your nickname. A worker never runs `cells claim` itself — anything else (open, claimed by a different worker, missing, capped) is not yours to touch → return `[BLOCKED]` (or `[NOOP]` per the rule above), never claim it yourself to make the cell fit.

## 3. Reserve

- Reserve **every** file or glob before writing:
  `node .bee/bin/bee.mjs reservations reserve --agent "<name>" --cell "<id>" --path "<path>" --ttl 3600`
- Any conflict → stop and return `[BLOCKED]` with the paths and holder. Never edit through a conflict.
- Prefix write-heavy shell commands with `BEE_AGENT_NAME="<name>"`.

## 4. Implement

- Read every file before editing it. Start from the cell's `read_first` list.
- Match existing patterns and the cited locked decisions (D-IDs).
- No stubs, TODO-only placeholders, dead code, or pseudo-implementations.

**Deviation rules** — when reality disagrees with the cell:

1. Found a bug in touched code → **auto-fix**, record as a deviation.
2. Missing critical functionality the cell's outcome depends on → **auto-add**, record as a deviation.
3. Blocking issue (broken import, type error in the path) → **auto-fix**, record as a deviation.
4. Architectural change needed → **STOP**, return `[BLOCKED]` with the proposal. Never redesign inside a cell.

Package installs **always** checkpoint: stop and return `[BLOCKED]` with the package and reason — never install on your own authority.

## 5. Verify

- Run the cell's verify command exactly, then record it **with its output** (decision 0004 — proof, not assertion):
  `node .bee/bin/bee.mjs cells verify --id <id> --command "<cmd>" --output "<what it printed>" --passed true|false` (or `--output-file <f>` for long output)
- The `verify` field is the cell's **targeted** suite (seconds) — never the full configured `commands.verify` chain (D4, decision `e54878b1`). Run red-first, then green, and stop there: do **not** additionally run the full chain yourself — the orchestrator's own wave-close run (`bee-swarming/SKILL.md`) is the independent proof that covers your cell, now the impacted run (`commands.test`) rather than the full chain (ci-owned-verify D1/D6). The full chain itself is CI-owned: no local baseline run before the first claim, and worktree merge gates on `commands.test` (impacted over the staged merge) instead of the full chain — session finish likewise runs `commands.test`, and the release flow runs the impacted suite locally then dispatches the CI full run (`gh workflow run CI --ref main`) right after the tag push, with red arriving back as a `verify-red` issue (`AGENTS.md`). Mid-iteration, run the level-1 impacted run (`run_verify --impacted-from-git --level 1` — direct edges only, seconds); the transitive impacted run (`commands.test`) stays the wave-close/merge gate.
- **Proof-tier matrix (test-economy D1/D2).** How much proof a cap demands is no longer one uniform red-first rule for every `behavior_change` cell — it is derived from the cell's `change_class × lane`, resolved by `requiredProofTier` (`skills/bee-hive/templates/lib/cells.mjs`):

  | change_class | lane | tier | what capCell accepts |
  |---|---|---|---|
  | `refactor` / `formatting` | every lane, **including `high-risk`** | `suite-green` | the existing suite passing is proof enough; a **new** test file in the diff is refused outright — `new_suite_reason` (D3, below) can NOT override this. Needing a new suite for a "refactor" means the cell is misclassified. |
  | `bugfix` / `behavior` / `api` | `tiny` / `small` / `standard` | `targeted-green` | one targeted test passing (ordinary `verification_evidence`) — no red-first, no `red_failure_evidence` required. |
  | `bugfix` / `behavior` / `api` | `high-risk` | `red-first` | the scoped red-first below still applies in full: `red_failure_evidence` ≥80 chars, anti-duplicate floor. |
  | `security` / `migration` | every lane, tiny through high-risk | `red-first` | same as above — no lane ever softens this row. |
  | unclassified (`change_class` null), `behavior_change: false` | any | *(no matrix check)* | today's pre-test-economy behavior, untouched — claiming a lighter tier than `behavior` requires declaring `change_class` explicitly; the null default never gets a discount. |
  | unclassified (`change_class` null), `behavior_change: true` | any | derived as `behavior`, then tiered per the `behavior` row above | same acceptance as the row it derives into |

  **Amendment note:** decisions `e54878b1`/`8ef2bae6` (scoped red-first) are **amended, not reversed**, by test-economy D2. The *shape* they defined — a red run before the fix touches ONLY the test(s) this cell adds or changes, never the full suite — still governs everywhere the table above resolves to `red-first`. What D2 narrows is the *when*: red-first used to read as a blanket expectation for any `behavior_change` cell; it now applies only to the `red-first` rows above (security/migration in any lane, or a behavior-bearing class riding `high-risk`) — every other row's floor is `targeted-green`.

- **Scoped red-first (decision 8ef2bae6), wherever the matrix above resolves to `red-first`:** the red run before a fix executes ONLY the test(s) this cell adds or changes — name the test file or filter, never the full configured suite. The cell's own `verify` chain (however many checks it strings together) then runs exactly once, at the end, right before cap, to prove the green state. A full-suite run inside a red-first loop is the named waste this scopes away.
- **Test-shape rules (test-economy D3)** — apply whenever this cell adds or changes tests, independent of which proof tier applies:
  - ≥3 cases exercising the same behavior are table-driven, not copy-pasted near-duplicates.
  - A new scenario for behavior the suite already covers is a new **row** in the existing suite, not a new file.
  - A genuinely new `test_*.mjs` file must declare `new_suite_reason` (≥20 chars, in the evidence JSON) explaining why a new permanent CI suite is warranted — cap refuses without it. For a `refactor`/`formatting` cell this can never rescue a new test file; see the matrix above.
  - Test-to-source line ratio has a ceiling: `tiny`/`small` warn above 3, `standard`/`high-risk` refuse above 4 unless the evidence declares a `ratio_waiver` (≥20 chars) — shrink the diff or justify the ratio.
- **Read-first, before writing a new test (test-economy D5):** before adding a test, cite the nearest existing test that already exercises this area and say in one line why it doesn't already cover the new case (fold it into `tests_inspected`/evidence). Reading first is cheap; a duplicate suite born from not looking first is not.
- **Cap evidence is scoped, not full (verify-scoping D2, decision `20534ea9`).** Cap evidence is the cell's own scoped verify passing — never run the full configured verify yourself just to cap. The orchestrator's done-report re-run after your `[DONE]` uses that same scoped command, not the full suite; the full run belongs to close, not to caps.
- **In a declared no-test repo (decision 55b951e1), verify is `"none"`.** When `commands.verify`/`commands.test` in `.bee/config.json` carries the sentinel `"none"`, a cell's `verify` field may legitimately be `"none"` too — there is no command to run. Cap evidence there is the diff-backed outcome plus the recorded waiver note (`cells cap` auto-fills "no-test repo: verification waived by repo declaration" when none is supplied); never invent a fake check to satisfy the field, and never write `verify: "none"` in a repo that has not declared itself no-test — `addCell`/`updateCell` refuse it there.
- The `verify` field must be a runnable command. If the cell shipped with a prose description instead, that is a planning defect — return `[BLOCKED]` naming it; never invent a substitute check.
- On failure: fix the root cause and rerun the exact command.
  - **No `Advisor` line in the dispatch:** unchanged — after **two serious failed attempts**, return `[BLOCKED]` with the command, failure summary, and diagnosis. A broken verify command in the repo is itself a blocker — never substitute a weaker check and cap anyway.
  - **An `Advisor` line is present in the dispatch:** the first serious failed attempt does not fall straight to a bare second retry — see **Advisor Consult** below. Two serious failures with no consult budget remaining still end in `[BLOCKED]`, same as the unchanged rule.

## 6. Advisor Consult

D1 amends the two-attempts rule above with a worker-level, on-failure-only step. This is **not** a gate-time or orchestrator-level consult — de967733 ("Bee runs ONE cost pattern") stays amended, not reversed: fan-out orchestration remains the default for every phase, and the human gates are untouched.

**Trigger** — consult only when both are true: the dispatch prompt carries an `Advisor` line (the orchestrator already ran the same-model no-op check per AO4/AO5 before adding one — the worker never self-assesses this), and the worker has just hit its **first serious failed verify attempt**. No `Advisor` line → proceed exactly as the unchanged rule in Verify.

**Canonical loop (D3), max 2 consults per claim:**

```
fail 1 -> consult 1 -> advised retry
  -> (fail) -> consult 2 (follow-up, same advisor) -> final retry
    -> (fail) -> [BLOCKED] with a Consults section (both consults summarized)
```

A re-dispatched cell (rescue rung) starts a **fresh** budget — the 2-consult cap is per claim, not per cell lifetime. Consulting after `[BLOCKED]` has already been returned for the current claim is never permitted.

**Evidence bundle (mandatory, every consult):** exact failing command, the failing output, your diagnosis, the relevant cited file excerpts, and the `CONTEXT.md` path. Pass it **inline in the consult prompt or via stdin — never a `/tmp` path** (critical pattern 20260708). Never include secrets or env values.

**Transport** — the `Advisor` line names the advisor and how to consult it:
- **Model-shaped advisor:** consult via your own Agent tool, with the model param set to the named advisor model, and the dispatch `description` starting **exactly** `advisor-consult <cell-id>: <advisor-model>` — this is the A2 attribution record; bee-swarming's goal-check reads it from `.bee/logs/dispatch.jsonl`. Fallback if Agent dispatch is unavailable or rejected: a headless one-shot `claude -p --model <advisor-model>` call, same evidence bundle via stdin.
- **cli-shaped advisor:** run the given command with the evidence bundle on stdin, reusing the External Executors output-capture discipline.
- A **transport error** (non-zero exit, rejected dispatch, a hang past the External Executors timeout discipline) is **not advice** — it burns at most **one** budget slot total for the whole claim, and is never retried in a storm. Continue to the next step of the loop, or `[BLOCKED]` once the budget is spent.

**After advice:** advice never substitutes for fresh verify output — always rerun the real verify command yourself before deciding whether the advised retry passed. Advice is **advice-only** (A1): it never authorizes a package install, a gate approval, or file scope beyond the cell. Advice that conflicts with a locked decision → return `[BLOCKED]` citing both the D-ID and the advice.

**Authority-type blocks never consult** — ambiguous cell, uncapped deps, architectural change, package install, locked-decision conflict stay **instant** `[BLOCKED]` exactly as in step 4 (Implement), whether or not an `Advisor` line is present.

**Headless rule unchanged:** consulting the advisor is not "asking the parent or user" under the Headless rule below — it stays inside your own turn. Workers still never approve gates.

Record every consult in the cap trace and the per-cell report (see Cap and Return) — count, advisor identity, and a one-line ask/answer digest per consult.

## 7. Cap

- Cap only after the verify pass is recorded (the helper refuses otherwise):
  `node .bee/bin/bee.mjs cells cap --id <id> --outcome "<summary>" --files <a,b> [--deviations-file <f>] [--friction "<text>"]`
- If the cell is `behavior_change: true`, add `--behavior-change --evidence-stdin` and **pipe** the structured `verification_evidence` (tests inspected, tests added/changed, red-failure/before-state evidence, verification run — see `references/worker-details.md`). It lands in the cell trace; **do not write an evidence file** in `reports/` or anywhere else (decision 0009 — the trace is the single source; if you ever must, the one canonical scratch home is `.bee/tmp/<feature-or-session>/`, docs/specs/doctrine-layer.md R17).
- If any Advisor Consults happened on this claim, fold their count and advisor identity into the trace alongside the rest of the evidence — no separate file, same decision 0009 rule.
- Trace depth follows the cell's lane (tiny = one line; high-risk = full trace). Record friction only when a trigger fired.
- Make exactly **one commit per cell**, cell id in the message.

## 8. Release

`node .bee/bin/bee.mjs reservations release --agent "<name>" --cell "<id>"`

## 9. Return

- Start your final message with exactly one of `[DONE]`, `[BLOCKED]`, `[HANDOFF]`, `[NOOP]`, followed by the result fields.
- Write a **short** per-cell report to `docs/history/<feature>/reports/<cell-id>.md`: the status token, a one-line outcome, files touched, and a link to `.bee/cells/<cell-id>.json` for the full trace/evidence. Never re-embed the `verification_evidence` JSON or verify output (decision 0009 — the trace is the single source), and never a separate scratch file elsewhere (docs/specs/doctrine-layer.md R17).
- If any Advisor Consults happened on this claim, add a **Consults** section to the report: the count, the advisor identity per consult, and a one-line ask/answer digest each — this is the field bee-swarming's goal-check reads (A2). No consults happened → omit the section entirely.

## Compaction

At roughly 65% context before a safe finish: write `.bee/HANDOFF.json` (cell, files, done, remaining, next_action), release reservations that are safe to release, and return `[HANDOFF]`. After compaction, reread `AGENTS.md`, `CONTEXT.md`, the cell, and your active reservations before continuing.

## Fresh-Session Handoff (downstream, not a worker action)

This `[HANDOFF]` is the pause kind — unrelated to the planned-next handoff (fresh-session-handoff D1). When this cell caps with a green verify and further execution-approved work remains, the orchestrator continues in-session; only at real session exit does it run the finish → claim-next → planned-next handoff flow for the next fresh session to adopt silently (no-clear-stop D1) — never a stop or a `/clear` prompt to the user, and never something a worker claims or writes mid-swarm on its own initiative. A worker's job stays exactly Cap → Release → Return.

## Headless

Workers always run effectively headless: never ask the parent or user a blocking question. Unambiguous deviations are applied under the rules above; anything ambiguous becomes `[BLOCKED]` with an `Outstanding Questions` section in the report. Workers never approve gates — Gate decisions belong to the user via the orchestrator chain. This rule is unchanged by Advisor Consult (A4): consulting a configured advisor stays inside your own turn and is never "asking the parent or user."

## Red Flags

- editing outside reserved scope
- selecting your own cell, or handling more than one
- waiting silently instead of returning a status
- capping without a recorded verify pass, or "verifying" with a substitute command
- recording `--passed true` with no output — small+ lanes refuse the cap; an assertion is not evidence
- `--files` left empty on a cell that touched files — the trace is the machine-readable record, not the outcome prose
- a `behavior_change` cell capped without verification evidence
- installing packages without a checkpoint
- leaving reservations active without reporting it
- reinterpreting a locked decision to make the cell fit
- consulting the advisor with no `Advisor` line in the dispatch, or consulting on an authority-type block instead of instant `[BLOCKED]`
- a model-shaped consult dispatched without the exact `advisor-consult <cell-id>: <advisor-model>` description prefix — it breaks the A2 attribution record
- treating advisor advice as a substitute for fresh verify output, or capping consults without a Consults section in the report

Violating the letter of the rules is violating the spirit of the rules.

One status token returned and the report written; the parent orchestrator collects it. Invoke bee-swarming skill (parent side) to continue the wave.

## Reference Files

| File | When to Load |
|---|---|
| `references/worker-details.md` | Expanded commands, trace tiers by lane, friction triggers, result field spec, evidence example |
