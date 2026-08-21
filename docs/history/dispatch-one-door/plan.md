---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: dispatch-one-door

Mode: `standard` — 2 risk flags: covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the change edits refusal strings that
existing tests assert on, across Rust hook code, Rust status code, and two vendored skill
references — a plan plus per-file cells is the smallest shape that keeps those three domains
from being fixed to three different answers again.

## Requirements (from CONTEXT.md)

- **D1** — every subagent dispatch resolves its transport from `.bee/config.json` through one
  verb, `bee dispatch prepare`; prose never tells a reader to prefer a `subagent_type`.
  Consequences (a) Transport bullet, (b) retire `herding-executor D7`'s gather boundary,
  (c) retire "unrendered tier → `general-purpose`", (d) guard FIX messages name `prepare` and
  stop calling a herding slot "a cli executor or unconfigured".
- **D2** — direct `subagent_type: bee-*` stays legal where the slot is model-shaped; only the
  prose recommending it is withdrawn.

## Discovery

Seven drift sites, listed with file:line in `CONTEXT.md` ("Drift found"). Inspected
`model_guard.rs` (branches 3a/3b/4), `status_full/store.rs:892-952`, `onboard/agents.rs`
(`compute_agent_file_plan`), and both skill references. Evidence: with
`generation = {kind:"herding"}`, `bee dispatch prepare --runtime claude --kind gather --json`
returns a `herding-exec` Bash payload while `Agent(bee-gather)` is denied — the config was
already authoritative, only the readers were not.

`skills/bee-hive/references/routing-and-contracts.md` was checked and is clean.
`docs/knowledge/areas/hook-runtime/dispatch-guard.md` R6 ("a guard refuses only what it
cannot derive or resolve") already supports D1: the guard cannot rewrite Agent→Bash, so the
refusal is right and only its FIX line was wrong.

## Approach

Recommended path (D1, D2): change what the reader is told and what the refusal hands back —
no new mechanism. `bee dispatch prepare` already resolves every slot shape; nothing in the
dispatch pipeline needs new code. Two Rust edits are string/coverage repairs, two skill edits
are prose repairs.

Rejected alternatives:

- Teach the hook to rewrite an Agent call into a Bash call — a PreToolUse hook allows or
  denies; it has no rewrite channel for the tool itself.
- Add a `cli`/`herding` branch to the prose and keep `subagent_type` preferred (CONTEXT.md's
  rejected alternative) — leaves the config unread and the drift re-openable.
- Auto-delete stale `.claude/agents/bee-*.md` outside onboarding — a guard/status path that
  silently removes files the operator can see is a bigger surprise than an honest advisory.

Risk map:

| Component | Risk | Proof needed |
|---|---|---|
| `model_guard.rs` FIX strings | MEDIUM — existing tests assert on substrings | model_guard test module green, updated assertions cite the new wording |
| `status_full/store.rs` drift check | LOW — additive tier row + message wording | `status_full` tests green, one new case for a herding slot |
| `gates-and-delegation.md` | MEDIUM — the contract other skills cite | pointer/parity check; no orphaned cross-reference to D7's retired boundary |
| `swarming-reference.md` | LOW — three localized spots | pointer/parity check |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| **S1 — the refusal tells the truth** | `model_guard.rs`: `cli-tier-denied`, `herding-tier-denied` and `bare-denied` FIX lines name `bee dispatch prepare`; the bare FIX stops calling a herding slot "a cli executor or unconfigured" | This is the surface the reporting host actually hit — an agent that reads only the refusal now recovers without reading any skill | Run the guard against a herding config; the denial names `prepare` | An agent can recover from the denial with zero doc reads |
| **S1 — the prose points at the door** | `gates-and-delegation.md` Transport bullet + retire the D7 gather boundary; `swarming-reference.md` at 110-113, 300, 380-384 | Same slice: these are the three documents that disagreed; fixing one without the others re-creates the split | Grep: no surviving "prefer this shape", no "does not exist yet", no "unrendered → general-purpose" | One answer to "how do I dispatch" |
| **S2 — the drift is reported honestly** | `status_full/store.rs`: add `bee-build` to `AGENT_FILE_TIER`; a herding slot is named `herding`, not "cli-shaped or unconfigured"; the remedy line works for a host repo that cannot reach a bee checkout | Stale `bee-gather.md` (`model: sonnet`) sat unreported next to a herding config in the reporting host | `bee status` on a herding config lists the stale agent files with a remedy that runs | An operator who changes a slot sees the consequence |
| **S3 — knowledge sync** | `docs/knowledge/areas/hook-runtime/dispatch-guard.md` | Behavior changed; the state layer is read first by the next session | Knowledge check green | Next session starts from the new answer |

Slice queue: S1 (current) → S2 → S3. S2 depends on nothing in S1 but shares no files, so it
may run concurrently once S1's cells are claimed; S3 follows execution per the capture
discipline.

Current slice to prepare: **S1** — three cells, disjoint product files, parallel by default.

## Test matrix

Triad, at its smallest demonstrating size:

- **Happy path** — a model-shaped `generation` slot still allows `Agent(bee-gather)` and still
  repairs a `[bee-tier: extraction] + general-purpose` dispatch (D2: no behavior change).
- **Edge cases** — `generation = {kind:"herding"}` denies `bee-gather` with a FIX naming
  `bee dispatch prepare`; a bare dispatch under the same config denies with a herding-accurate
  FIX; `generation = {kind:"cli"}` denies with a FIX naming `prepare`.
- **Error paths** — `status_full` reports `agent-file-drift` for a rendered `bee-build.md`
  under a herding slot, and the message names the slot kind correctly.

Each cell's writer judges existing coverage first (`model_guard.rs` and
`status_full/tests.rs` both already pin the current strings) and authors only the gap.

## Out of scope

- Making `bee onboard` runnable in a host repo without a bee checkout (`blocked_no_engine`).
  Real, separately reported, not caused by this drift.
- Any change to `bee dispatch prepare` itself — it already resolves every slot shape.
- Any change to the `herding` runtime, the mailbox protocol, or `bee herding run`.
- Removing the guard's direct `subagent_type: bee-*` branch (D2 keeps it).
- Rewriting the reporting host's `.bee/config.json` — the operator wants herding on.
