# bee harness — state registers

The cross-stage shared state. Every stage in the [chain](index.md) reads and writes
these files, and they are the reason work is resumable and reviewable. Two
invariants govern all of them:

1. **Never hand-edit `.bee/*.json(l)`.** Every mutation goes through the CLI
   (`.bee/bin/bee <group> <verb>`). A mutation with no CLI verb is filed
   as friction (`bee backlog add`), then edited by hand — never silently.
2. **State is the truth the hook cannot see.** The write-guard hook is a net; the
   phase and gate fields below are what actually decide whether work may proceed.
   Its hard direct-edit deny list is *narrower* than the rule — `.bee/state.json`,
   `.bee/backlog.jsonl`, `docs/backlog.md`, the two `.bee/runtime/` ledgers, and
   the companion marker, each named with the verb that owns it. Everything else in
   `.bee/` is held by discipline, not by the guard. Silence is never permission.

Anchors below are linked from stage pages and [index.md](index.md).

## The control plane (`.bee/runtime/`)

Shared across worktrees. The **workflow record is the source of truth**; every
file marked *projection* below is derived from it.

| Path | Holds |
|------|-------|
| `runtime/workflows/<wf-id>/state.json` | the workflow record (truth): `id`, `feature`, `phase`, `mode`, `plan_rev`, `gates` (each `{approved, approved_for_plan_rev}`), `route`, `feature_verify`, `summary`, `next_action`, `status`, `created_at` |
| `runtime/leases/cells/<cell-id>.json` · `runtime/leases/paths/<prefix>/<hash>.json` | sharded per-resource leases (exclusive/advisory) — replace the monolithic reservations store |
| `runtime/handoffs/<wf-id>/<seq>.json` | per-workflow handoff mailbox — one workflow pausing never blocks another |
| `runtime/workspaces/` · `cross-worktree-holds.json` · `worktree-grants.json` | worktree registry, path-keyed cross-checkout holds, and the grants `bee worktree new/register` issue |

The data plane (cells in flight, logs, tmp) stays isolated per worktree. Every
state verb resolves the live workflow's record first.

## The registers

### `.bee/state.json`
**Read-only projection.** The legacy single runtime state file — now a projection
of the live workflow record, kept for compatibility. Same keys as before:

| Key | Holds |
|-----|-------|
| `schema_version` | state-file format version |
| `phase` | current chain phase — the closed nine: `idle` · `exploring` · `planning` · `swarming` · `reviewing` · `scribing` · `compounding` · `grooming` · `compounding-complete` |
| `feature` | active feature slug |
| `mode` | lane (`tiny` · `small` · `standard` · `high-risk` · `spike` · `docs`) |
| `approved_gates` | `{context, shape, execution, review}` booleans — four fields, three gates: `shape` and `execution` flip together as Gate 2 |
| `gate_revoked_at` | map of gate name → ISO timestamp (revocation audit) |
| `cells` | rollup counts `{open, claimed, capped, blocked}` |
| `route` | the recorded triage `{class, lane, flags[], product_files, rationale, updated_at}` — written by `bee route` |
| `summary`, `next_action` | human-readable resume hints |
| `last_scribing_run` | `{feature, date, at, areas_synced, next_action}` — the scribing-debt check also consults the durable ledger `.bee/logs/scribing-runs.jsonl` |
| `last_compounding_run` | `{feature, at, learnings, critical_promotions, decisions_logged, …}` |
| `advisor_ref` | high-risk advisor record `{consulted_at, feature, newest_decision_id, plan_sha256, advisor, digest_head}` |
| `last_activity` | heartbeat |

**Workers are derived, not stored**: "active workers" = live heartbeat sessions
joined with their workflow id and cell claims. The `workers` array is vestigial
and `state worker add/update/remove/clear/prune` are compatibility no-ops.

Written by: every stage, via `state set --owner <phase>`, `bee gate`,
`state scribing-run`, `state compounding-run`, `state start-feature`,
`bee route --set`, `state advisor-ref record` — each verb mutates the live
workflow record; this file is re-projected. `state rebuild-projections`
regenerates it from the records.

### `.bee/config.json`
Per-repo configuration.

| Key | Holds |
|-----|-------|
| `commands` | `{setup, start, test}` shell commands. **`test` is the single declaration of how the project is tested** (a string or an array run in order) — the one command the green base check, every cap, close, merge, and CI all run. A `"none"` sentinel means the gate is deliberately disabled |
| `hooks` | toggle map: `session-init`, `prompt-context`, `state-sync`, `chain-nudge`, `session-close`, `write-guard` — each default-on |
| `gate_bypass` | `off` · `normal` · `full` · `total` — the opt-in gate autopilot level |
| `models` | per-runtime tier→model map: `{claude:{extraction, generation, review, advisor}, codex:{…}}`. A tier may be an object `{kind:"cli", command, promptVia}` — an external gather-only executor |
| `lanes`, `capabilities` | per-repo overrides |

Read by hive (bypass level), planning (test scoping), swarming (model tiers),
`bee test` / `bee cells finish` / `bee close` (`commands.test`), and
`bee worktree merge` (`commands.test`).

**Config is the one hand-edited register.** The `config get/set/unset/validate`
verbs are [declared but not built](#declared-but-not-built), and `config.json` is
deliberately absent from the write guard's direct-edit deny list — so changing
`gate_bypass` or a model tier means editing the file, preserving every other
field, and logging a one-line audit decision in the same turn. This is a named
exception to invariant 1, not a licence that generalizes.

### `.bee/onboarding.json`
Onboarding state + managed-file version hashes (drift detection).

Keys: `schema_version`, `bee_version`, `managed` (sha256 per tracked file:
`agents_block`, `gitignore_block`, hook wiring, statusline, vendored binary,
skills, expertise), `agents_sync`, `created_at`, `updated_at`. Written by
`bee onboard --repo-root <root> --apply`. Read at session start to detect drift.

### `.bee/cells/<feature>-<n>.json`
One unit of executable work — the atom the swarm dispatches. One file per cell.

| Field | Holds |
|-------|-------|
| `id`, `feature`, `lane`, `title` | identity |
| `files` | paths the cell may write (reserved before write) |
| `read_first` | files the worker must read before editing |
| `action` | free-text instruction |
| `must_haves` | `{truths[], artifacts[], key_links[], prohibitions[]}` |
| `behavior_change` | bool — gates scribing + the semantic judge |
| `verify` | plan text describing how the change is proven. **MAIN owns running it, once, at feature close** — never the assigned worker (`cells show` states this as `verify_owner`) |
| `deps` | cell ids that must cap first |
| `tier` | dispatch tier (`generation`, …) |
| `status` | `open` · `claimed` · `capped` · `blocked` · `dropped` |
| `trace` | populated on finish: `{worker, outcome, files_changed[], deviations[], friction, behavior_change, capped_at, warnings[], tests, results, ran_at, attempts[], budget_resets[]}` |

`trace.tests` is `"green"` when the declared suite ran and passed — with
`trace.results` pointing at `.bee/logs/test-results.json` and `trace.ran_at`
stamping it — or `"undeclared"` in a repo with no `commands.test`. There is no
proof-tier field, no `red_failure_evidence`, and no evidence-tier ladder: those
were deleted with the proof-economy machinery. A red finish appends a
`tests-red` entry to `trace.attempts` *before* refusing the cap.

Created by planning (`bee cells add --stdin`, whole slice in one call), mutated by
swarming/executing (`cells claim` / `bee finish` / `cells block` / `cells reopen` / …).

### `.bee/claims/<cell-id>.json`
Atomic single-winner cell claims — TTL plus heartbeat on every claim path, typed
refusal codes, gate-protected adoption and reclaim. A capped cell's claim is
released as part of the cap. Written through the claim doors
(`cells claim`, `cells claim-next`, `bee dispatch prepare --claim`), never by hand.

### `.bee/lanes/<feature>.json`
Per-feature lane records — the per-feature slice of workflow state every reader
resolves through when several features are live in one checkout. Listed by
`state lanes`; `bee status --lanes-full` renders them all instead of the
active-lane summary.

### `.bee/intent/<key>.json`
The **shaping anchor**, written by `bee shape` (the flow spelling of
`intent set`): the user's VERBATIM request plus what "done" means, before any code
is touched. Shape: `{schema_version, key, written_at, request, acceptance,
next_action, feature, lane, cell, do_not_reverse[], stop_conditions[]}`. Read back
by `intent show`, advanced by `intent advance`, cleared by `intent clear`; the
prompt-context hook re-surfaces it so a long session cannot drift off the request
it started from.

### `.bee/sessions/<session-id>.json`
Self-derived session identity: `{id, started_at, last_heartbeat, transcript_path,
workspace_id}`. Heartbeats are throttled and self-renewing; this is the join key
that makes "active workers" derivable, and the input `bee status`'s recovery block
reads for crash candidates (the `recovery` verbs themselves are
[not built](#declared-but-not-built)). Managed by `state session bind/list/unbind`.

### `.bee/locks/`
The cross-process coordination-store lock: holder body, stale takeover, and the
serialization behind concurrent reservation and state writes. Never read or
written by an agent directly — it exists so two sessions cannot interleave a
read-modify-write on the same store.

### `.bee/reviews/<review-id>.json`
One frozen review session: baseline, head, included/excluded scope, findings,
UAT records, decision. Written via `reviews create/record`; listed by
`reviews list` / `reviews status`. The scope is immutable once created.

### `.bee/decisions.jsonl`
Append-only decision log — the source of truth for *why*. One JSON object per line:
`{id (uuid), type, date, decision, rationale, alternatives, scope, source,
confidence, tags}`. Written via `decisions log/supersede/tag`, never by hand;
write-time secret and injection rejection applies, and reads are datamarked.
Once `docs/decisions/taxonomy.json` exists a zero-tag event is refused. Archived
to `.bee/decisions-archive.jsonl`, which `decisions active` reads as a union.

### `.bee/reservations.json`
**Compat mirror.** File holds for same-checkout swarms. The **lease store**
(`.bee/runtime/leases/`, sharded per resource) is the source; this file mirrors it
so the legacy surface — and the cross-worktree coordination write that lives beside
it — keeps working. Shape:
`{reservations: [{agent, cell, path, kind, session, ttl_seconds, reserved_at, released_at}]}`
(`released_at` null while held). `kind` is `lease` (a worker's write-time hold — a
hard conflict) or `intent` (a broad planning-time scope the write guard only warns
about). Written via `reservations reserve/release/sweep`. A write to a held path is
refused with the holder named — do not write around it.

### `.bee/backlog.jsonl`
Event-sourced friction + PBI records. Event shapes include `proposal`
(`{ts, type, title, detail, predicted_impact, lane, source}`) and PBI lifecycle
events. Written via `bee backlog add` / `backlog propose` / `backlog pbi.*`.
Rendered to `docs/backlog.md` (generated — never hand-edited).

### `.bee/HANDOFF.json`
**Legacy projection.** The pause/resume artifact — a projection of the live workflow's own mailbox
(`runtime/handoffs/<wf-id>/<seq>.json`), rendered when no mailbox entry exists.
`state handoff show` resolves the mailbox first. Exists only while paused. Two
`kind`s:

- **`pause`** — `{…, kind:'pause', written_at}`. Surfaced and **waited on**; never
  auto-resumed. A missing/unknown kind reads as `pause` (fail-safe).
- **`planned-next`** — requires `writer_session`, `previous_cell` (capped, tests
  green), `next_cell` (claimed by the same session). Adopted automatically **only**
  at a fresh-session boundary (`/clear` or fresh start) via `state handoff adopt`.

Written via `state handoff write/adopt/show`.

### `.bee/capture-queue.jsonl`
Deferred capture stubs awaiting their spec merge.
Shape: `{kind:'stub', id, at, outcome, dids[], area, files[], lane}`. Written via
`bee capture add`, drained via `capture flush --id <id> --into <spec>`. High-risk
lane never queues — it merges now.

### `.bee/expertise/`
The vendored craft layer — 9 craft guides and 6 domain guides plus `INDEX.md`,
copied from the source `expertise/` by onboarding. **Read-only from the agent's
side**: skills reference `.bee/expertise/<guide>.md`, and a change belongs in the
source tree, not the vendored render.

### Logs & caches (read-mostly)
- `.bee/logs/test-results.json` — **the one test record**: `{ran_at, green,
  commands:[{command, exit, duration_ms, failure_excerpt}]}`. Written by `bee test`,
  read by `cells finish` and `bee close`. The runner is a program; an agent's word
  is never the record.
- `.bee/logs/hooks.jsonl` — hook audit/crash log `{ts, hook, event, tool_name, tool_input_keys[]}`
- `.bee/logs/timings.jsonl` — per-invocation `{ts, cmd, ms, ok}`
- `.bee/logs/dispatch.jsonl`, `tools.jsonl`, `contention.jsonl` — stage traces
- `.bee/logs/scribing-runs.jsonl` — durable scribing ledger; the scribing-debt
  threshold consults it alongside the workflow record's stamp
- `.bee/review-candidates.jsonl` — review queue for reviewing/compounding
- `.bee/feedback-digest.json` — feedback digest cache for evolving
- `.bee/cache/`, `.bee/tmp/`, `.inject-cache.json` — regenerable; `tmp sweep --feature` clears the feature scratch

## The CLI — how registers are mutated

One native binary (`.bee/bin/bee`, `bee.exe` on Windows) with **two surfaces**.

### Porcelain — the flow surface

`bee --help` (and `--help --json`) shows only these; the JSON manifest carries
`{schema_version, surface:"porcelain", total_commands, commands}`. Every one obeys
the **teach-at-point-of-contact contract**: its output and its refusals end with
the next action in plain language.

| Verb | Role in the flow |
|---|---|
| `bee orient` | Session-start context packet: where am I, what is locked, what is next |
| `bee status` | Full snapshot when routing work |
| `bee route` | Record the triage/lane classification |
| `bee shape` | Write the shaping anchor — verbatim request + what "done" means |
| `bee gate` | Record a gate approval (`--merge` for shape+execution together) |
| `bee cells add` · `cells ready` · `cells show` | Persist shaped work · what is claimable · one cell in full |
| `bee dispatch prepare` | Build a worker dispatch payload (`--claim` claims + reserves in the same verb) |
| `bee finish` | Worker completion: run the declared tests, cap on green, release reservations |
| `bee reservations reserve` | Claim write scope before editing |
| `bee decisions log` · `decisions active` | Record an agreement · what is in force |
| `bee capture add` · `bee backlog add` | Queue a learning stub · park future work |
| `bee test` | Run `commands.test`, write `.bee/logs/test-results.json` |
| `bee close` | Feature close driver: declared test run → what remains |

Four of them are **aliases**, not new behavior — argv is rewritten and the proven
verb runs, so there is one implementation and one test set per operation:

| Flow verb | Runs |
|---|---|
| `bee route` | `bee state route` |
| `bee shape` | `bee intent set` |
| `bee gate` | `bee state gate` |
| `bee finish` | `bee cells finish` |

### Plumbing — everything else

`bee internal <group> <verb>` is the plumbing namespace; it dispatches identically
after the prefix is stripped and **refuses a flow verb** ("call it as `bee gate`,
without `internal`"), so the boundary is real in both directions. The bare
top-level spellings still work. `bee --help --all [--json]` lists the full registry
(127 entries), each carrying its `surface` value; `bee internal --help` lists just
the plumbing.

| Group | Verbs |
|-------|-------|
| `cells` | list · ready · show · add · update · claim · claim-next · unclaim · finish · cap · block · drop · reopen · tier · judge · judge-record · reset-budget · schedule · archive · unarchive |
| `state` | set · gate · route · start-feature · lanes · scribing-run · compounding-run · plan-rev bump · session.* · handoff.* · workflows.* · rebuild-projections · advisor-ref.* · compact-* (worker.* = compat no-ops) |
| `reservations` | reserve · release · list · sweep |
| `decisions` | log · supersede · redact · active · search · archive · tag · render |
| `backlog` | add · propose · counts · rank · badges · render · findings · pbi.* |
| `capture` | add · list · flush · count |
| `reviews` | create · list · show · record · candidate.add · candidates · status |
| `feedback` | digest · count · collect · rank |
| `knowledge` | check · index · list · context · promote |
| `worktree` | new · merge · list · register · unregister |
| `intent` | set · show · advance · clear |
| *other* | `dispatch prepare` · `tmp sweep` · `recovery scan/window` · `config.*` · `perf.*` · `herding.*` · `doctor` |

### Maintenance surfaces (outside the registry)

These probe **before** the verb tree, so nothing in it can shadow them:

- `bee hook <name>` — the nine handlers the hook catalogs invoke: `session-init`,
  `prompt-context`, `write-guard`, `model-guard`, `state-sync`, `chain-nudge`,
  `session-close`, `tools-logger`, `codex-subagent-audit`. Each is fail-open: an
  undecidable payload allows the operation and says the guard did not run on it.
- `bee onboard [--repo-root R] [--apply] [--json]` — the installer. `changes_needed`
  → summarize, get approval, re-run with `--apply`; `blocked_*` → zero mutations.
- `bee dev render-skill-trees | render-prompt | statusline | release-manifest` — the
  maintainer surface.
- `bee herding classify-lane | interlock | command-template | …` — the cockpit's
  executable helpers.
- `bee rs-info` — diagnostic: runtime, version, and the ported-shape list.

### Refusals are typed

Nothing falls through silently. An argv no probe claims produces a named refusal
on stderr with a non-zero exit — and `--json` still yields `{error, kind, command}`
on stdout so a caller can branch instead of regex-matching prose. The five kinds:

| Headline | Means |
|---|---|
| `bee: unknown command` | nothing in the registry spells this — nearest spellings, or the group's verb list |
| `bee: not built into this binary` | the registry declares it and this build has no implementation |
| `bee: unexpected positional argument` | a real command with a stray positional |
| `bee: missing required argument` | names the flags and quotes the command's own example |
| `bee: unsupported argument shape` | required flags all present; an optional flag, a value, or a target is wrong |

### Declared but not built

23 registry entries have no implementation in the current binary — the R6 Node
deletion removed the only one they had. Each refuses by name, states that nothing
ran and nothing changed, and names its fallback:

| Unbuilt | Count |
|---|---|
| `doctor` · `doctor attest` | 2 |
| `config get/set/unset/validate` | 4 |
| `perf start/stop/section/log/render/report/sync` | 7 |
| `herding enable/disable/status` | 3 |
| `state advisor-ref record/show` | 2 |
| `state compact-capsule/check/log` | 3 |
| `recovery scan/window` | 2 |

Treat them as known gaps, not as verbs to route around: a Codex install cannot be
attested until `doctor` is ported, so that runtime reads as degraded; config
changes go through the hand-edit exception above; and `bee status --json` still
reports onboarding health.

Run `.bee/bin/bee --help --json` for the porcelain manifest, `--help --all --json`
for every command, or `<group> --help` before a group's first use in a session.
