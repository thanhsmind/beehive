# bee harness — state registers

The cross-stage shared state. Every stage in the [chain](index.md) reads and writes
these files, and they are the reason work is resumable and reviewable. Two
invariants govern all of them:

1. **Never hand-edit `.bee/*.json(l)`.** Every mutation goes through the CLI
   (`.bee/bin/bee <group> <verb>`). A mutation with no CLI verb is filed
   as friction (`backlog add`), then edited by hand — never silently.
2. **State is the truth the hook cannot see.** The write-guard hook is a net; the
   phase and gate fields below are what actually decide whether work may proceed.

Anchors below are linked from stage pages and [index.md](index.md).

## The registers

### `.bee/runtime/workflows/<wf-id>/state.json`
The **workflow record — source of truth** since multisession-native (v1.17.0).
One record per workflow, holding `id`, `feature`, `phase`, `mode`, `plan_rev`,
`gates`, `summary`, `next_action`, `status`. Every state verb resolves the live
workflow's record first; the files below marked *projection* are derived from it.

Part of the **control plane** (`.bee/runtime/`), shared across worktrees:

| Path | Holds |
|------|-------|
| `runtime/workflows/<wf-id>/state.json` | the workflow record (truth) |
| `runtime/leases/cells/<cell-id>.json` · `runtime/leases/paths/<prefix>/<hash>.json` | sharded per-resource leases (exclusive/advisory) — replace the monolithic reservations store |
| `runtime/handoffs/<wf-id>/<seq>.json` | per-workflow handoff mailbox — one workflow pausing never blocks another |
| `runtime/workspaces/` · `cross-worktree-holds.json` · `worktree-grants.json` | worktree registry, holds, and grants for `bee worktree new/merge` |

The data plane (cells in flight, logs, tmp) stays isolated per worktree.

### `.bee/state.json` *(read-only projection)*
The legacy single runtime state file — now a projection of the live workflow
record, kept for compatibility. Same keys as before:

| Key | Holds |
|-----|-------|
| `schema_version` | state-file format version |
| `phase` | current chain phase (`idle` · `exploring` · `planning` · `swarming` · `reviewing` · `scribing` · `compounding` · `grooming` · `compounding-complete`) |
| `feature` | active feature slug |
| `mode` | lane (`tiny` · `small` · `standard` · `high-risk` · `docs`) |
| `approved_gates` | `{context, shape, execution, review}` booleans — the four gates |
| `gate_revoked_at` | map of gate name → ISO timestamp (revocation audit) |
| `cells` | rollup counts `{open, claimed, capped, blocked}` |
| `summary`, `next_action` | human-readable resume hints |
| `last_scribing_run` | `{feature, date, at, areas_synced, next_action}` — the scribing-debt check also consults the durable ledger `.bee/logs/scribing-runs.jsonl` as fallback (sss-1) |
| `last_compounding_run` | `{feature, at, learnings, critical_promotions, decisions_logged, …}` |
| `advisor_ref` | high-risk advisor record `{consulted_at, feature, newest_decision_id, plan_sha256, advisor, digest_head}` |
| `last_activity` | heartbeat |

**Workers are derived, not stored** (multisession D6): "active workers" = live
heartbeat sessions joined with their workflow id and cell claims. The old
`workers` array is gone and `state worker add/update/remove/clear/prune` are
compatibility no-ops.

Written by: every stage, via `state set --owner <phase>`, `state gate`,
`state scribing-run`, `state start-feature`, `state advisor-ref record` — each
verb mutates the live workflow record; this file is re-projected.

### `.bee/config.json`
Per-repo configuration.

| Key | Holds |
|-----|-------|
| `commands` | `{setup, start, test, verify}` shell commands (a `"none"` sentinel means the gate is deliberately disabled) |
| `hooks` | toggle map: `session-init`, `prompt-context`, `state-sync`, `chain-nudge`, `session-close`, `write-guard`, … |
| `gate_bypass` | `off` · `normal` · `full` · `total` — the opt-in gate autopilot level |
| `models` | per-runtime tier→model map: `{claude:{extraction, generation, review, advisor}, codex:{…}}` |
| `lanes`, `capabilities` | per-repo overrides |

Mutated via `config get/set/unset/validate`. Read by hive (CI/verify gate,
bypass level), swarming (model tiers), executing (`commands.verify`).

### `.bee/onboarding.json`
Onboarding state + managed-file version hashes (drift detection).

Keys: `schema_version`, `bee_version`, `managed` (sha256 per tracked file:
`agents_block`, `gitignore_block`, `helpers.bee.mjs`, `lib.*.mjs`, `repo_hooks.*`,
`statusline.*`), `agents_sync`, `created_at`, `updated_at`. Written by
`packages/bee/scripts/onboard_bee.mjs --apply` (hive onboarding). Read at session
start to detect drift.

### `.bee/cells/<feature>-<n>.json`
One unit of executable work — the atom the swarm dispatches. One file per cell.

| Field | Holds |
|-------|-------|
| `id`, `feature`, `lane`, `title` | identity |
| `files` | paths the cell may write (reserved before write) |
| `read_first` | files the worker must read before editing |
| `action` | free-text instruction |
| `must_haves` | `{truths[], artifacts[], key_links[], prohibitions[]}` |
| `behavior_change` | bool — gates scribing + goal-check judge |
| `verify` | the runnable verify command (an assertion is not evidence) |
| `deps` | cell ids that must cap first |
| `tier` | dispatch tier (`generation`, …) |
| `status` | `open` · `claimed` · `capped` · `blocked` · `dropped` |
| `trace` | populated on cap: `{worker, outcome, files_changed[], verification_evidence, verify_output, verify_passed, verified_at, attempts[]…}` |

Created by planning (`cells add`), mutated by swarming/executing
(`cells claim/verify/cap/…`).

### `.bee/decisions.jsonl`
Append-only decision log — the source of truth for *why*. One JSON object per line:
`{id (uuid), type, date, decision, rationale, alternatives, scope, source, confidence}`.
Written via `decisions log/supersede`, never by hand. Archived to
`.bee/decisions-archive.jsonl`.

### `.bee/reservations.json` *(compat mirror)*
File holds for same-checkout swarms. The **lease store**
(`.bee/runtime/leases/`, sharded per resource) is the source since v1.17.0; this
file mirrors it so the legacy surface — and the cross-worktree coordination write
that lives beside it — keeps working. Shape:
`{reservations: [{agent, cell, path, ttl_seconds, reserved_at, released_at}]}`
(`released_at` null while held). Written via `reservations reserve/release/sweep`
(CLI surface unchanged). A write to a held path is refused with the holder named —
do not write around it.

### `.bee/backlog.jsonl`
Event-sourced friction + PBI records. Event shapes include `proposal`
(`{ts, type, title, detail, predicted_impact, lane, source}`) and PBI lifecycle
events. Written via `backlog add/propose/pbi.*`. Rendered to `docs/backlog.md`
(generated — never hand-edited).

### `.bee/HANDOFF.json` *(legacy projection)*
The pause/resume artifact — a projection of the live workflow's own mailbox
(`runtime/handoffs/<wf-id>/<seq>.json`), rendered when no mailbox entry exists.
`state handoff show` resolves the mailbox first. Exists only while paused. Two
`kind`s:

- **`pause`** — `{…, kind:'pause', written_at}`. Surfaced and **waited on**; never
  auto-resumed. A missing/unknown kind reads as `pause` (fail-safe).
- **`planned-next`** — requires `writer_session`, `previous_cell` (capped, verify
  green), `next_cell` (claimed by the same session). Adopted automatically **only**
  at a fresh-session boundary (`/clear` or fresh start) via `state handoff adopt`.

Written via `state handoff write/adopt/show`.

### `.bee/capture-queue.jsonl`
Deferred capture stubs awaiting their spec merge.
Shape: `{kind:'stub', id, at, outcome, dids[], area, files[], lane}`. Written via
`capture add`, drained via `capture flush`.

### Logs & caches (read-mostly)
- `.bee/logs/hooks.jsonl` — hook audit/crash log `{ts, hook, event, tool_name, tool_input_keys[]}`
- `.bee/logs/timings.jsonl` — per-invocation `{ts, cmd, ms, ok}`
- `.bee/logs/dispatch.jsonl`, `tools.jsonl` — stage traces
- `.bee/logs/scribing-runs.jsonl` — **durable scribing ledger**; the
  scribing-debt threshold consults it alongside the workflow record's stamp (sss-1)
- `.bee/review-candidates.jsonl` — review queue for reviewing/compounding
- `.bee/feedback-digest.json` — feedback digest cache for evolving
- `.inject-cache.json` — session-preamble cache

## The CLI — how registers are mutated

Every register above is read/written through `.bee/bin/bee <group> <verb>`.
The primary nine groups (per `AGENTS.md`) plus the utility groups the dispatcher
also exposes:

| Group | Verbs |
|-------|-------|
| `status` | *(single verb)* |
| `cells` | list · ready · show · add · update · claim · verify · cap · block · drop · unclaim · reopen · tier · judge · claim-next · reset-budget · judge-record · schedule · archive · unarchive |
| `reservations` | reserve · release · list · sweep |
| `decisions` | log · supersede · redact · active · search · archive · tag · render |
| `state` | set · gate · scribing-run · start-feature · lanes · session.* · handoff.* · advisor-ref.* · compact-* (worker.* = compat no-ops, workers are derived) |
| `backlog` | counts · rank · badges · add · propose · pbi.* · render · findings |
| `capture` | add · list · flush · count |
| `reviews` | create · list · show · record · candidate.add · candidates · status |
| `feedback` | digest · count · collect · rank |
| *utility* | `intent` · `knowledge` (check·index·list·context·promote) · `perf` · `worktree` (new·merge·…) · `herding` · `config` · `tmp` · `dispatch` · `recovery` |

Run `.bee/bin/bee --help --json` for the full tool-schema-shaped manifest,
or `<group> --help --json` before a group's first use in a session.
