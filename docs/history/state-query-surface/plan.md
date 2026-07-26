---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-24 (auto-approved, gate_bypass=total)
---

# Plan — state-query-surface

**One line:** đóng khung các truy vấn state của bee thành CLI có hợp đồng, để agent
thôi text-munging `.bee/*.jsonl` bằng grep/python và thôi reach vào internals bằng
`node -e import(bin/lib/*)`.

## Mode gate (mechanical)

Flags counted: **public contracts** (CLI command surface is a public contract) +
**multi-domain** (decisions / backlog / state-ledger / hook-guard) = **2 flags** →
`standard`. No hard-gate flag: the guard is *additive* safety, it weakens no existing
proof; the query verbs are additive reads. Not `small` because it crosses >3 product
files across 4 domains. Not `high-risk` (no auth/data-loss/security/provider/validation-removal).

Product files (canonical source): `skills/bee-hive/templates/bee.mjs`,
`skills/bee-hive/templates/lib/command-registry.mjs`,
`skills/bee-hive/templates/lib/guards.mjs`, hook wiring in `bee-write-guard.mjs`,
reusing `lib/decisions.mjs`, `lib/feedback.mjs`/`lib/backlog.mjs`, `lib/cells.mjs`.
Each change propagates to the byte-identical mirror `.bee/bin/**` and the 4 rendered
plugin skill trees via `scripts/render_plugin_skill_trees.mjs` — mirror/parity is a
**shared-file** fact that forces serial execution (see Risk).

## Discovery (L1 — repo verified, gather digest)

Feasibility gather (report: `reports/feasibility-gather.md` — digest below) settled the
one real gray area and confirmed every insertion point:

- **B1 gray area RESOLVED — no structural field.** A `decide` event is
  `{id, type, date, decision, rationale, alternatives, scope, source, confidence, tags?}`
  (`lib/decisions.mjs:300-338`). There is **no** `cell`/`feature` field; cell ids like
  `si-1` appear only inside free-text `decision`/`rationale`. So `--cell/--feature` cannot
  be a structural join — the honest form is a **word-boundary token match** over the text.
  The repo already owns that discipline: `sweepDecisionCitations` uses `\b…\b`
  (`lib/decisions.mjs:382-383`); `--text` does **not** (`bee.mjs:1688-1701`, plain
  `.includes()`), which is exactly the `si-1`≈`si-10` collision.
- **B2:** friction/finding rows carry `{ts, (kind|type), feature, title, detail, severity, layer}`.
  **Two schemas coexist** — legacy `kind:` + current `type:` — a reader must accept both.
  Read/fold path: `lib/feedback.mjs:470-509` (skips `kind:'pbi'`). `backlog add` at
  `bee.mjs:2800-2850`, registry `command-registry.mjs:1082-1107`.
- **B3:** ledger rows `{ts, feature, areas[]}` (`.bee/logs/scribing-runs.jsonl`).
  Reader `readScribingLedger` + `scribingLedgerPath` (`lib/cells.mjs:2081-2087`), per-feature
  max via `bestStampMs` inside `globalScribingDebt` (`cells.mjs:2128+`). `--show/--last`
  reuses these — pure read, no write, no phase advance.
- **A (guard):** `bee-write-guard.mjs` already inspects Bash command content
  (`checkGitBashCommand` shape, guards.mjs; wired ~`bee-write-guard.mjs:834-848`). Add
  `checkBinLibImportBashCommand(command)` in `lib/guards.mjs`, wire into the Bash branch,
  reuse the ERROR/WHY/FIX denial convention. Test bed: `hooks/test_hook_contracts.mjs`.
- **A already-public fact:** `globalScribingDebt` is already surfaced in `status --json`
  as `scribing_debt.orphaned` (4986 bytes) — the `node -e` reach fetches public data by the
  worst possible path. The guard's FIX message names `bee status --json` and
  `bee <group> --help --json` as the paved read.

## Approach

Four additive slices, **executed serially** (shared regen targets — Risk §1). Every code
cell is `behavior_change: true` and closes with a scribing sync (knowledge: "read the CLI,
don't grep the jsonl / import internals").

- **Cell `sqs-A` — internals-reach guard + paved-read FIX.** New pure fn
  `checkBinLibImportBashCommand` in `lib/guards.mjs`: deny a Bash command that is an inline
  eval (`node -e` / `--eval` / `-p`) whose script text `import`s/`require`s a `bin/lib/` or
  `templates/lib/` module. **Prohibition:** never block file-based `node <path>.mjs` runs
  (tests import lib legitimately) — only inline-eval reaches. FIX message points to
  `bee status --json` / `bee <group> --help --json`. Wire into `bee-write-guard.mjs` Bash
  branch; propagate to mirrors.
- **Cell `sqs-B1` — `decisions active/search --cell / --feature` (word-boundary).** Add both
  flags; match as whole tokens (`\b…\b`, reusing the `sweepDecisionCitations` discipline) over
  `decision`+`rationale`+`alternatives`. Register params in `command-registry.mjs` so
  `--help --json` advertises them. **Non-collision is the point:** `--cell si-1` must exclude
  `si-10`. (Follow-on note, not this cell: same `\b` fix could harden `--text` — out of scope,
  logged as friction.)
- **Cell `sqs-B2` — `backlog findings --feature <slug> [--text …]`.** New read verb listing
  friction/finding rows filtered by feature (word-boundary, not substring) + optional text,
  reusing the `lib/feedback.mjs` read/skip-pbi loop, **accepting both `kind:` and `type:`**
  rows. Register in `command-registry.mjs`.
- **Cell `sqs-B3` — `state scribing-run --show [--feature <slug>]`.** `--show` switches the
  verb to **read-only** (no ledger append, no phase advance): returns the most-recent stamp
  overall, or for `--feature`. Reuse `readScribingLedger`/`bestStampMs`. Register the flag.

### Rejected alternatives
- **B1 as a structural `cell`/`feature` field** — rejected: no such field exists; adding one
  is a data-model change (a flag → high-risk) and does nothing for the existing store the pain
  is about. Word-boundary text match matches the observed grep usage exactly.
- **A as doc-only (no guard)** — rejected: knowledge alone did not stop the reach (the reach is
  the worst offender). Guard + instructive FIX both stops it and teaches the paved read. Guard
  scoped to inline-eval keeps false-positive risk near zero.
- **A as a hard block on all bin/lib imports** — rejected: would trap legitimate test files.

## Risk map

| Component | Risk | Proof needed (validating) |
|---|---|---|
| Shared regen targets (`bee.mjs`, `command-registry.mjs`, 5 mirrors) | **MEDIUM** — parallel edits collide (cli-ergonomics precedent: a merge wiped a worker's edits) | Serialize the 4 cells; reserve `bee.mjs`+`command-registry.mjs`; run render + `test_lib_mirror` between cells |
| A guard false-positive | **MEDIUM** — blocking a legitimate `node <file>.mjs` | Negative-control test: file-based lib import PASSES, inline-eval reach is DENIED |
| B1 word-boundary correctness | LOW | Positive+negative test: `si-1` matches `si-1`, excludes `si-10` |
| B2 dual schema (`kind`/`type`) | LOW-MEDIUM | Test covers a legacy `kind:` row and a current `type:` row |
| B3 read/write mode split | LOW | `--show` asserts no ledger append and no phase change |

## Test matrix (sketch, 12 dimensions)
- **Happy path:** each verb returns the right rows/stamp.
- **Boundary/collision:** `si-1`↛`si-10` (B1); feature substring not matched (B2).
- **Negative control:** guard PASSES file-import, DENIES inline-eval-import (A).
- **Schema variance:** B2 across `kind:`+`type:` rows.
- **Idempotence/no-side-effect:** B3 `--show` leaves ledger + phase untouched.
- **Contract surface:** each new flag/verb appears in `--help --json`.

Verify per cell is **scoped** (the area's test file), never the full chain:
- `sqs-A` → `node hooks/test_hook_contracts.mjs`
- `sqs-B1` → `node skills/bee-hive/templates/tests/test_decisions_propagation.mjs`
- `sqs-B2` → `node skills/bee-hive/templates/tests/test_backlog_capture.mjs`
- `sqs-B3` → `node skills/bee-hive/templates/tests/test_cli_state.mjs`

Mirror/render parity (`test_lib_mirror.mjs`, `test_render_race.mjs`) rides the wave-close
impacted run, not each scoped verify.

## Open questions for validating
1. Exact propagation step: edit `templates/` + `.bee/bin/` both, or edit one and render? Pin
   the command sequence so cells are byte-parity-clean.
2. `state scribing-run --show` as a flag on the write verb vs a distinct `show` subcommand —
   confirm the flag form doesn't trip the verb's write-path validation.
3. Confirm each named test file is the one the impact registry maps the touched source to.
