# bee harness — state registers

The cross-stage shared state. Every stage in the [chain](index.md) reads and writes
these files, and they are the reason work is resumable and reviewable. Two
invariants govern all of them:

1. **Never hand-edit `.bee/*.json(l)`.** Every mutation goes through the CLI
   (`.bee/bin/bee <group> <verb>`). A mutation with no CLI verb is filed
   as friction (`bee backlog add`), then edited by hand — never silently.
2. **State is the truth the hook cannot see.** The write-guard hook is a net; the
   phase and gate fields below are what actually decide whether work may proceed.
   Its hard direct-edit deny list is *narrower* than the rule — any `.json` under
   `.bee/cells/` or `.bee/lanes/`, plus `.bee/state.json`, `.bee/onboarding.json`,
   `.bee/backlog.jsonl`, `docs/backlog.md`, the two `.bee/runtime/` ledgers, and
   the companion marker, each named with the verb that owns it. Everything else in
   `.bee/` is held by discipline, not by the guard. Silence is never permission.
   The net itself is phase-split, not one list: at a **gated** phase (`exploring`
   or `planning` with execution unapproved) only `.bee/`, `docs/history/`,
   `plans/`, and `AGENTS.md` pass — note the *narrower* `docs/history/`, not
   blanket `docs/` — while the idle/terminal **intake** gate still allows blanket
   `docs/` (`write_guard/guards.rs`'s `GATE_ALLOWED_PREFIXES_GATED` vs.
   `GATE_ALLOWED_PREFIXES_INTAKE`). The consequence: a `docs/` write outside
   `docs/history/` now refuses at a gated phase, where it used to pass.

Anchors below are linked from stage pages and [index.md](index.md).

## The control plane (`.bee/runtime/`)

Shared across worktrees. The **workflow record is the source of truth**; every
file marked *projection* below is derived from it.

| Path | Holds |
|------|-------|
| `runtime/workflows/<wf-id>/state.json` | the workflow record (truth): `id`, `feature`, `phase`, `mode`, `plan_rev`, `gates` (each entry — see sub-table below), `run_state`, `waiting_on`, `route`, `summary`, `next_action`, `status`, `created_at` (older records also carry a `feature_verify` object — legacy residue of the retired `commands.verify`; no code reads or writes it) |
| `runtime/leases/cells/<cell-id>.json` · `runtime/leases/paths/<prefix>/<hash>.json` | sharded per-resource leases (exclusive/advisory) — replace the monolithic reservations store |
| `runtime/handoffs/<wf-id>/<seq>.json` | per-workflow handoff mailbox — one workflow pausing never blocks another |
| `runtime/workspaces/` · `cross-worktree-holds.json` · `worktree-grants.json` | worktree registry, path-keyed cross-checkout holds, and the grants `bee worktree new/register` issue |
| `runtime/integration/queue/<seq>.json` | the durable merge queue `bee worktree merge` drains, one file per queued worktree, plus the processor lease that keeps two mergers from draining it at once |

Each `gates` entry (one per name in `GATE_NAMES` — `context`, `shape`,
`execution`, `review`, `uat`) carries more than the boolean flag. `uat` is
the acceptance stop for the finished work: user-only approval — an
`--actor auto` call is refused outright for `uat` at any `gate_bypass`
level, including `total` (uat-gate-before-merge D1) — and it is only
enforced for `standard`/`high-risk` features (a missing or unrecognized
lane fails closed as standard); `tiny`/`small`/`docs`/`spike` are exempt.
Config `uat_stop` (below) picks WHERE it sits: under `"merge"` (default,
absent means this) `bee worktree merge` refuses `WORKTREE_MERGE_UAT_PENDING`
until the gate is approved, unless `--skip-uat` is passed for that one
merge or the door is turned off repo-wide; under `"close"` the merge lands
first and `bee close` carries the door instead; under `"off"` neither door
exists anywhere.

| Field | Holds |
|-------|-------|
| `approved` | boolean — the pre-existing flag most call sites still read |
| `approved_for_plan_rev` | the plan revision this approval was granted for, or `null` |
| `state` | `pending` \| `approved` \| `rejected` — the gate's own persisted approval state |
| `actor` | `user` \| `auto` \| `null` — who moved the gate; `auto` is a `gate_bypass` auto-approval |
| `at` | ISO timestamp of the last state change, or `null` |
| `reason` | free text, required when `actor` is `auto`, else `null` |
| `bypass_level` | `off` \| `normal` \| `full` \| `total` \| `null` — the level an `auto` approval recorded as its reason for not halting |

`bee gate` writes this shape; `.bee/state.json`'s `approved_gates`
([below](#beestatejson)) is a booleans-only projection of it — see that entry
for the cross-reference.

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
| `approved_gates` | `{context, shape, execution, review, uat}` booleans — five fields: `shape` and `execution` flip together as Gate 2, and `uat` is the acceptance stop before merge, never the same call as Gates 1-2. This is a flattened projection: the workflow record's own `gates` entry carries `state`/`actor`/`at`/`reason`/`bypass_level` too ([above](#the-control-plane-beeruntime)) — read the record, not this boolean, when the richer shape matters |
| `gate_revoked_at` | map of gate name → ISO timestamp (revocation audit) |
| `run_state` | the run's own closed-vocabulary lifecycle name — `shaping` · `awaiting-approval` · `running` · `blocked` · `done` (or `null` pre-migration). Projected from the workflow record, never computed at read time; exposed by `bee status --json` |
| `waiting_on` | the persisted wait mark, or `null` when nothing is waited on: `{kind, subject, asked_at, session}` — `kind` is `gate` (a formal approval) or `question` (something the agent asked and has not been answered). Written by `bee state waiting-on set`, cleared by `bee state waiting-on clear` |
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

> **Open gap (as of 2.1.9).** One door disagrees: the `small`+ cap guard reads the
> stored `workers[]` array to decide whether an execution worker was registered
> ([below](#the-doors-that-refuse)), while multisession-native D6 says that array is
> display-only and no gate treats it as truth. The door ships as written; re-anchoring
> it onto the derived view is filed work. Recorded as a disagreement on purpose — the
> spec is not rewritten to match whatever shipped.

Written by: every stage, via `state set --owner <phase>`, `bee gate`,
`state scribing-run`, `state compounding-run`, `state start-feature`,
`bee route --set`, `state advisor-ref record` — each verb mutates the live
workflow record; this file is re-projected. `state rebuild-projections`
regenerates it from the records.

### `.bee/config.json`
Per-repo configuration.

| Key | Holds |
|-----|-------|
| `commands` | `{setup, start, test}` shell commands. **`test` is the single declaration of how the project is tested** (a string or an array run in order) — the one command CI runs on every push, and `bee test` runs on demand. Each cap records its own proof line `<command> — <result> — <scope reason>`; `bee close` and `bee worktree merge` check that recorded proof and run nothing themselves. A `"none"` sentinel means the gate is deliberately disabled |
| `hooks` | toggle map over nine of the ten handlers: `session-init`, `prompt-context`, `state-sync`, `chain-nudge`, `session-close`, `write-guard`, `model-guard`, `tools-logger`, `codex-subagent-audit` — each default-on. The tenth, `activity` (the herding heartbeat), has no kill switch here |
| `guards` | write-guard tuning: `idle_gate`, `max_read_lines` |
| `gate_bypass` | `off` · `normal` · `full` · `total` — the opt-in gate autopilot level |
| `models` | per-runtime **role→model** map: `{claude:{code, read, extraction, generation, review, advisor}, codex:{…}}`. The key is the job a dispatch asks for; a fresh config seeds `code`/`read` beside the historical `extraction`/`generation` tail, and any role name you add is legal — bee holds no fixed list, and asks "is this name configured", never "is it one of four words". Named seats exist too: the blind-lane and hat-wave seat roles (`lane-1..3`, `hat-*`) fall through to `advisor` when unconfigured, and `supervisor` names the herding observer's model. A role value may be an object `{kind:"cli", command, promptVia}` — an external gather-only executor — or `{kind:"herding", agent, fallback}`; an object slot may carry a `description` (clipped to 60 chars) that `bee models show` and the dispatch-door roles line display. `models.pi` is a herding-only preview runtime — any non-herding dispatch shape on it refuses `pi_requires_herding`. `retry.fallbackChains` is a separate key: an explicit-only, role- or model-keyed chain bee PUBLISHES on the payload and never walks itself |
| `product_root` | subdirectory the product lives in, when the repo root is not it — the product-file count that picks the lane is measured against it |
| `worktree_first` | whether a code-touching route must open its feature worktree before execution |
| `cells_archive_on_close` | default true — a capped cell is relocated to `.bee/cells/archive/<feature>/` at close |
| `ship_visibility` | how much of the ship line a session prints |
| `dogfood_repos` | the foreign repos `bee feedback` collects a digest from |
| `worktree_cleanup_on_merge` | boolean, absent means KEEP (worktree-keep-on-merge D1) — `worktree merge` leaves the merged worktree in place unless this is explicitly `true` or the one-merge `--cleanup` flag is passed; `--no-cleanup` wins over both and always keeps |
| `uat_stop` | `"merge"` (default, absent means this) \| `"close"` \| `"off"` — where the `uat` acceptance stop sits: at `bee worktree merge` (today's behavior), moved to `bee close` (merge lands first so the product is testable on main; that merge SETS the lane's `waiting_on` gate mark instead of clearing it, and holds the worktree — `--cleanup`/`worktree_cleanup_on_merge: true` are ignored, reported as `WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING`, while the gate is pending), or off everywhere. A value outside the three refuses `WORKTREE_MERGE_UAT_CONFIG_INVALID` rather than guessing (uat-stop-placement D1) |
| `uat_before_merge` | back-compat alias for `uat_stop`, read only when `uat_stop` itself is absent — boolean, `true` reads as `"merge"`, `false` reads as `"off"`; a non-boolean value refuses `WORKTREE_MERGE_UAT_CONFIG_INVALID` rather than guessing (uat-gate-before-merge D1, superseded as the primary key by uat-stop-placement D1) |
| `staging_before_merge` | boolean, absent means ON — whether the repo uses the staging mixing ground at all; explicit `false` makes `bee staging add`/`bee staging rebuild` refuse `STAGING_DISABLED`, so the repo runs feature worktree -> `uat` gate -> main with no staging step; a non-boolean value refuses `STAGING_CONFIG_INVALID` rather than guessing. Independent of `uat_stop` — the `uat` gate itself is unaffected |
| `doc_viewer` | `{base_url, project}` — an opt-in URL prefix. When set, the session preamble and the compaction capsule give doc links as this URL plus the repo-relative path, instead of the bare path |

Read by hive (bypass level), planning (test scoping), swarming (model roles),
and `bee test` (`commands.test`, its own runner). `bee close` and `bee
worktree merge` no longer run `commands.test` — each checks the cap's own
recorded proof line instead; `bee cells finish` is commit-only proof and
writes that line.
`.bee/config-sample.json` is the annotated
copy of the whole schema — its `_doc` block is the per-key contract.

**Config is the one hand-edited register.** The `config get/set/unset/validate`
verbs are [declared but not built](#declared-but-not-built), and `config.json` is
deliberately absent from the write guard's direct-edit deny list — so changing
`gate_bypass` or a model role means editing the file, preserving every other
field, and logging a one-line audit decision in the same turn. This is a named
exception to invariant 1, not a licence that generalizes.

### `.bee/runtime/staging.json`
The disposable mixing-ground record (staging-lane D0/D0a): staging = main plus every
feature still awaiting uat. Written only by `bee staging add`/`bee staging rebuild`.

| Field | Holds |
|-------|-------|
| `branch` | always `"staging"` |
| `worktree_root` | the staging worktree's path — a sibling `<repo>--wt--staging`, lazily created from main's CURRENT HEAD the first time `staging add` runs, never from a feature branch |
| `created_at` | ISO timestamp of that lazy create |
| `base_sha` | the exact commit staging was cut from |
| `staged` | `[{feature, branch, last_merged_sha, at}]` — one entry per feature currently merged in |

Commands: `bee staging add --feature <slug>` (trigger 1/2 — lazy-create-from-main if
needed, merge the feature's branch in, record it, run the build; re-running after a fix
re-merges the same branch); `bee staging rebuild [--without <slug,...>]` (trigger 3 —
`reset --hard main`, re-merge every feature still staged and awaiting uat minus the
exclusions, then build — a feature whose uat gate is approved, or whose branch already
merged to main, drops out of the staged set on its own); `bee staging status` (the
staged set, each feature's uat gate state, staging's base sha vs main). Build hook:
config `commands.staging_build` (optional) — absent skips the step with a visible note,
never an error.

Refusals: a merge conflict while staging a feature (add or rebuild) aborts that one
merge and reports `STAGING_MERGE_CONFLICT`, naming the feature and files — on rebuild
the remaining features still build, so one broken feature never blocks testing the
others. **No escape into main**: `bee worktree merge` refuses
`WORKTREE_MERGE_STAGING_FORBIDDEN`, zero mutation, whenever the worktree/branch being
merged IS staging itself — no bypass flag exists ([doors table](#the-doors-that-refuse));
the only legitimate path to main is `bee worktree merge` on the FEATURE's own branch,
after its uat gate. A direct `git commit` inside the staging worktree is separately
refused by the write guard unless `BEE_STAGING_MACHINERY=1` (the env marker bee's own
staging commands set around their own merge commits) — remedy: fix on the feature
branch, `bee staging add` again.

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
| `role` | **required** — the job this work is (`code`, `read`, `test`, `docs`, `review`, `design`, or any name you configure). The cell's sole model selector: the dispatch asks for this name first and falls through when nothing configures it. Any non-empty name is legal; bee holds no list |
| `escalate` | bool — run this cell on the session model and charge the 40% escalation ration. Set through `bee cells escalate` (the door that checks the ration), never a role name |
| `tier` | **retired** as the model selector (model-role-split D4). `role` selects the model; `escalate` is the session-model flag. Records written before the retirement still carry the field, and it is read for exactly one thing: `tier: "ceiling"` still counts as escalated until `bee cells backfill-roles` has run |
| `status` | `open` · `claimed` · `capped` · `blocked` · `dropped` |
| `trace` | populated on finish: `{worker, outcome, files_changed[], deviations[], friction, behavior_change, capped_at, warnings[], tests, results, ran_at, attempts[], budget_resets[], claim_session, claimed_at, verify_passed, verify_output, verification_evidence, report}` — `report` is the worker's structured Result, written by `cells finish --report`. `escalation_reason` joins it when an over-ration escalation was allowed on a named `--reason` (spelled `tier_reason` on records written before model-role-split D4; `bee cells backfill-roles` renames it) |

`trace.tests` is a proof string `<command> — <result> — <scope reason>` on a
cap (decision `1f534837`): the writer picks the proof its change type needs,
runs it, and a red result refuses the cap; `bee close`/`bee worktree merge`
check the recorded proof and run nothing themselves. `"boundary"` and
`"undeclared"` are historical values from the boundary-run era
(test-cadence-boundary D1, decision `13ce1858`), and `"green"` is a
historical value from before that:
older cells capped while `bee finish` still ran the suite and recorded
`trace.results`/`trace.ran_at`; those fields are read for historical cells but
no longer written at cap. There is no proof-tier field, no
`red_failure_evidence`, and no evidence-tier ladder: those were deleted with
the proof-economy machinery. A historical `tests-red` entry in `trace.attempts`
marks a cell capped under the old per-cap run; since decision `1f534837` a
red proof refuses the cap itself, and `bee close`/`bee worktree merge` only
check the recorded proof.

Created by planning (`bee cells add --stdin`, whole slice in one call), mutated by
swarming/executing (`cells claim` / `bee finish` / `cells block` / `cells reopen` / …).
At close, `cells_archive_on_close` (default true) moves capped cells to
`.bee/cells/archive/<feature>/`, so an old cell id resolves there, not here.

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
reads for crash candidates. `bee recovery scan` is real and implemented — it
releases every crashed-session claim on invocation; only `bee recovery window`
is [not built](#declared-but-not-built). Managed by `state session bind/list/unbind`.

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

<!-- bee:not-a-deferral: this section names and describes the deferred-queue mechanism itself — the add/claim/release/complete event shapes and the work kinds it carries — so the word "deferred" is the subject under description here, not an open promise this doc is making -->
### `.bee/deferred-queue.jsonl`
Event-sourced, last-event-wins fold (add/claim/release/complete), holding
deferred `capture` / `scribe` / `review` / `promote` work that a session
absent when it was queued can pick up later — the ONE claimable queue for
that work. One item: `{id, kind, feature, cells[], areas[], files[], reason,
queued_at, claim, completed, completed_at}`; `claim` is `{owner, claimed_at,
ttl_seconds}` or `null`. `claim` follows the same exclusive-append,
lease-plus-heartbeat reclaim pattern `.bee/claims/` uses, so a parallel agent
takes exactly one item. Written via `deferred-queue add / list / claim /
release / complete`.
<!-- /bee:not-a-deferral -->

### `.bee/HANDOFF.json`
**Legacy projection.** The pause/resume artifact — a projection of the live workflow's own mailbox
(`runtime/handoffs/<wf-id>/<seq>.json`), rendered when no mailbox entry exists.
`state handoff show` resolves the mailbox first. Exists only while paused. Two
`kind`s:

- **`pause`** — `{…, kind:'pause', written_at}`. Surfaced and **waited on**; never
  auto-resumed. A missing/unknown kind reads as `pause` (fail-safe).
- **`planned-next`** — requires `writer_session`, `previous_cell` (capped),
  `next_cell` (claimed by the same session). Adopted automatically **only**
  at a fresh-session boundary (`/clear` or fresh start) via `state handoff adopt`.

Written via `state handoff write/adopt/show`.

<!-- bee:not-a-deferral: this section names and describes the capture-queue's own stub shape and lifecycle — deferred capture stubs awaiting their spec merge — so "deferred" here is the mechanism being documented, not a deferred obligation of this register entry -->
### `.bee/capture-queue.jsonl`
Deferred capture stubs awaiting their spec merge.
Shape: `{kind:'stub', id, at, outcome, dids[], area, files[], lane}`. Written via
`bee capture add`, drained via `capture flush --id <id> --into <spec>`. High-risk
lane never queues — it merges now.
<!-- /bee:not-a-deferral -->

### `.bee/expertise/`
The vendored craft layer — 10 craft guides and 6 domain guides plus `INDEX.md`,
copied from the source `expertise/` by onboarding. **Read-only from the agent's
side**: skills reference `.bee/expertise/<guide>.md`, and a change belongs in the
source tree, not the vendored render.

### The memory surfaces (for the human)

Four stores exist so the human can catch up without replaying transcripts.
All four are written by the machine at fixed moments, never narrated by the
agent:

- `.bee/human-mailbox/` — the **letter store**: `entries/<run-slug>.jsonl`
  (clean-stop entry appends), `<UTC-timestamp>-<short-run-slug>.md` letters
  (typed YAML frontmatter — `subject`, `run`, `project`, `filed_at`,
  `status: read|unread`, `items[]`, `needs_you[]` — over a short human body),
  and `digest-<period>.md` folds. A letter files at an armed run's end, at
  `bee close` (immediately), and — for a run that went silent — at the next
  session start. `bee mailbox mark --id <file> --status read|unread` is the one
  consuming verb. Digest folding runs at session start for every ended UTC day
  (`digest-YYYY-MM-DD.md`) and ISO week (`digest-YYYY-Www.md`), transcribing
  from close letters and `.bee/usage/` verbatim — no computed sums.
- `.bee/usage/<feature>.json` — token usage recorded by a green `bee close`
  (`bee-usage/v1`: `feature`, `closed_at`, `sessions`, `skipped`, `totals`),
  committed inside the close bookkeeping commit. The close prints one `usage:`
  summary line, and stays silent when no session transcript was readable —
  never a false zero.
- `.bee/supervisor/` — the herding supervisor's stores: `observations.jsonl`
  (append-only, one record per cold tick) and `interventions.jsonl`
  (`intervention` / `escalation` / `urgent` / `advisor-nudge` records).
  Verbs: `supervisor away/back/presence` (presence marks; `back` renders one
  WakeReport per away window), `record` (frequency-capped per
  `(target_session, point_key)`; `--kind urgent` bypasses the cap and fires a
  best-effort desktop notification), `report`, `pending`, `mark-delivered`,
  `consent-sweep`, `metrics` (two-sided health bands; `not-measurable` is
  first-class). An unanswered `advisor-nudge` arms the response debt the cap,
  close, and merge doors check.
- `.bee/triggers/` — registered revisit conditions, one JSON file per trigger
  (`bee triggers add --decision <id> --condition "..."`, `list`, `resolve`).
  The doc-deferral door and the contract guards resolve citations against it.

Herding's transport keeps two stores of its own: `.bee/mailbox/<job-id>/` —
one file-mailbox per dispatched job (`brief-N.txt`, `ack-N.json`,
`result-N.json`, `report-N.md`, `job.json`), written through `bee herding run`
and read natively, zero LLM tokens — and `.bee/wave-ledger.jsonl`, one row per
real dispatch.

### Logs & caches (read-mostly)
- `.bee/logs/test-results.json` — **the one test record**: `{ran_at, green,
  commands:[{command, exit, duration_ms, failure_excerpt, failure_log}]}`.
  `bee test` is the only writer. The only runtime reader is the D2 red-base
  check `cells claim` runs before granting a claim (`classify_red_base`,
  `verbs/cells/handlers_write.rs`) — `cells finish` and `bee close` no longer
  run or read it; they check the cap's own recorded proof line instead. The
  runner is a program; an agent's word is never the record. `failure_log`
  names the path of that command's complete, untrimmed output (below), or
  `null` when the command passed or the log write failed.
- `.bee/logs/test-failure-<runner>-<index>.log` — the complete output of a
  failing declared command, one file per `(runner, index)`; `runner` is
  always `test` — `bee test` is the only process that writes this file, and
  `cells finish`/`bee close` no longer run the declared command themselves.
  Written on a red, removed on the next green at the same index (no
  accumulation); the excerpt in `test-results.json` stays bounded at
  `FAILURE_EXCERPT_MAX_CHARS` while this file carries the rest.
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
| `bee gate` | Record a gate approval (`--merge` for shape+execution together). `--actor <user\|auto>`, `--bypass-level <off\|normal\|full\|total>`, `--reason "<text>"` — an `--actor auto` call refuses without both a bypass level and a reason |
| `bee cells add` · `cells ready` · `cells show` | Persist shaped work · what is claimable · one cell in full |
| `bee dispatch prepare` | Build a worker dispatch payload (`--claim` claims + reserves in the same verb) |
| `bee dispatch wave` | Claim, reserve, and build payloads for a whole ready wave in one call — the normal batch verb; `dispatch prepare` is the single-cell fallback |
| `bee finish` | Worker completion: commit-only proof, cap and release reservations (records the cap's own proof line) |
| `bee reservations reserve` | Claim write scope before editing |
| `bee decisions log` · `decisions active` | Record an agreement · what is in force |
| `bee capture add` · `bee backlog add` | Queue a learning stub · park future work |
| `bee test` | Run `commands.test`, write `.bee/logs/test-results.json` |
| `bee close` | Feature close driver: recorded-proof check (unconditionally, whether or not the feature has a worktree) → what remains |
| `bee doctor` | Install health: verdict ladder over the wiring, the vendored binary, and the runtime (`doctor attest` is the plumbing-surface attestation) |

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
(189 entries, 35 of them porcelain), each carrying its `surface` value;
`bee internal --help` lists just the plumbing.

| Group | Verbs |
|-------|-------|
| `cells` | list · ready · show · add · update · claim · claim-next · unclaim · finish · cap · block · drop · reopen · escalate · dissent · dissent-verdict · judge · judge-record · reset-budget · schedule · archive · unarchive · backfill-roles |
| `state` | set · gate · route · start-feature · lanes · scribing-run · compounding-run · plan-rev bump · session.* · handoff.* · workflows.* · rebuild-projections · advisor-ref.* · compact-* (worker.* = compat no-ops) |
| `reservations` | reserve · release · list · sweep |
| `decisions` | log · supersede · redact · active · search · archive · tag · reattribute · render |
| `backlog` | add · propose · counts · rank · badges · render · findings · pbi.* |
| `capture` | add · list · flush · count |
| `reviews` | create · list · show · record · candidate.add · candidates · status |
| `feedback` | digest · count · collect · rank |
| `knowledge` | check · index · list · context · promote · search · bootstrap · report |
| `worktree` | new · merge · list · register · unregister · prune |
| `intent` | set · show · advance · clear |
| `supervisor` | away · back · presence · record · report · pending · mark-delivered · consent-sweep · metrics · list |
| `staging` | add · rebuild · status |
| *other* | `dispatch prepare` · `dispatch wave` · `blind check` · `mailbox mark` · `models show` · `triggers add/list/resolve` · `discovery stub/list` · `work set/show` · `timings report` · `tmp sweep` · `recovery scan` · `recovery window` · `config.*` · `perf.*` · `herding.*` · `doctor attest` |

### Maintenance surfaces (outside the registry)

These probe **before** the verb tree, so nothing in it can shadow them:

- `bee hook <name>` — the ten handlers the hook catalogs invoke: `session-init`,
  `prompt-context`, `write-guard`, `model-guard`, `state-sync`, `chain-nudge`,
  `session-close`, `tools-logger`, `codex-subagent-audit`, `activity` (the
  herding heartbeat — the one name that still runs under the cockpit's marker
  and the one absent from `config.hooks`' nine kill switches). Each is
  fail-open: an undecidable payload allows the operation and says the guard did
  not run on it.
- `bee onboard [--repo-root R] [--apply] [--json]` — the installer. `changes_needed`
  → summarize, get approval, re-run with `--apply`; `blocked_*` → zero mutations.
- `bee dev render-skill-trees | render-prompt | render-hook-manifests | statusline |
  release-manifest | plugin-distribution | install-support` — the maintainer surface:
  the first three regenerate payload assets, the last two build what ships.
- `bee dev regen` — chains the three-step regen in order (render-skill-trees →
  `onboard --repo-root . --apply` → `release-manifest --write`), stopping at the
  first red with that step named. Run it after any doctrine or skill edit instead
  of sequencing the three steps by hand.
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

### The doors that refuse

The five kinds above are *shape* refusals — they answer "is this a command". A
second class refuses a well-formed command because the **state says the work may
not proceed**. These are the harness's teeth: each names its own remedy, and each
either has one explicit escape hatch or none at all. An escape hatch is a recorded
reason, never a silent skip — it is written onto the record it excuses.

| Door | Refuses when | Escape hatch |
|---|---|---|
| `cells add` | the target feature is in `exploring`/`planning` and its execution gate is not approved — no cells before the gate. One gated cell fails the whole batch | none — approve Gate 2 (`bee gate --merge`). A `docs` lane is exempt |
| `bee gate --name shape` / `--merge` | `plan.md`'s load-bearing claims table is missing, malformed, carries an invalid label, or still holds a `guessed` row — a shape approval over a guessing plan is the defect the table exists to catch | none — do the reality touch, relabel the row `read`/`ran` with its anchor and verbatim evidence |
| `cells claim` (contract guards) | `CONTRACT_UNCITED`: a test-writing cell (test-shaped path in `files`, or `role: test`) cites no store decision tagged `contract:<name>` · `CONTRACT_RETIRED` / `CONTRACT_UNSETTLED`: the cited contract decision is superseded/archived, or still carries an open trigger | none — cite the live contract decision, or settle/supersede it first. A local `D1`-style id passes over silently |
| `cells claim` | `.bee/logs/test-results.json` records the last run as red — never claim onto a red base | `--fix-first "<reason>"`, stored on `trace.fix_first` |
| `cells claim` | the feature has no route record **and** this session already spent its one-time warning on an earlier claim (warn once, then refuse) | `bee route --set …`. A racing loser sees the typed `CLAIMED` refusal first, so a claim conflict never reads as a routing problem |
| `cells finish` (cap) | lane is `small`/`standard`/`high-risk` and `trace.worker` names no registered execution worker — a lane that must dispatch cannot cap as if it had | `--inline-reason "<why>"`, stored on `trace.inline_reason`. `tiny` never reaches this branch |
| `cells finish` (cap) | the cell changed files and no commit in the last 50 commits carries the trailer `cell: <id>` — one commit per cell, checked, not asserted | `--commit-pending "<reason>"`, stored on `trace.commit_pending` |
| `cells escalate` | escalating a cell (setting its `escalate` flag) would put more than 40% of the feature's cells on the session model (exactly 40% passes) | `--reason "<text>"`, stored on `trace.escalation_reason` |
| `bee close` | the feature has `behavior_change` cells capped since the last scribing stamp and nothing captured them | run `bee-capturing`, or log a `capture-deferral` decision naming the feature |
| `bee close` (judge-debt, standard/high-risk lanes only) | a `behavior_change` cell capped since the judge-debt door shipped carries no `cells judge-record` verdict | run `bee cells judge` then `bee cells judge-record` for each named cell, or log a `judge-deferral` decision naming the feature |
| `bee close` (doc-deferral door) | deferral-shaped prose (matching `matches_deferral_prose`) appears outside a fence with no same-line registered trigger citation | register the condition (`bee triggers add --decision <id> --condition "..."`) and cite it inline (backtick `` `<id>` `` or `[[trigger:<id>]]`), log a `doc-deferral` decision naming the feature, or — only for prose that *documents* deferral machinery rather than prose that itself defers — wrap it in a reasoned `<!-- bee:not-a-deferral: <reason> --> ... <!-- /bee:not-a-deferral -->` block; the reason is required, an empty or missing one exempts nothing |
| `bee route --set` | a re-lane would demote a `high-risk` feature, demote while a hard-gate flag is present, or demote a second time (`demoted_at` already stamped) | **none** — all three are absolute |
| `state start-feature` | the current phase is neither `idle` nor the terminal alias `compounding-complete` — a prior feature is still in flight | none — finish it, or drop its remaining cells |
| `state handoff adopt` | the session started from `resume` or `compact`, not a fresh-session boundary; or the record's `kind` is `pause` | **none** — present the handoff and wait for the user. A session with no recorded start source warns and proceeds |
| `reviews record` (`approved`) | a `P1` finding is not named in the decision's `p1_resolutions[]` with a fixing cell | none — land the fix cell, then record with `p1_resolutions` |
| write-guard, `docs/history/<feature>/plan.md` | that feature's `approved_gates.shape` is true — plan.md freezes once shape is locked | `bee state plan-rev bump --lane <feature>`, or unapprove shape to redraft |
| model-guard, `Agent`/`Task` | the dispatch declares no role and names no pinned subagent type, **or** its `[bee-tier: <name>]` marker names a role nothing configures (`role-not-configured`). A pinned `bee-gather`/`bee-build`/`bee-extract`/`bee-review` *derives* its role from the agent file instead of refusing | declare `[bee-tier: <role>]` (the marker keeps its historical spelling and carries a role name) or a `model` param; for an unconfigured name, add it to `models.<runtime>` or open with one that is configured. A derived `cli` role still refuses — an external process is not dispatchable as an agent |
| `worktree merge`, dirty main (`WORKTREE_MERGE_MAIN_DIRTY`) | before this row's own refusal can fire, dirt confined to the bookkeeping roots — `.bee/`, `docs/decisions/`, touched `docs/knowledge/`, and `docs/history/<the-merging-feature>/` — is auto-committed first, warn-never-block. A remaining **tracked** modification outside those roots refuses, named by path; an **untracked** file refuses only when it collides with a file the branch itself changed (measured from the merge base) — a bystander untracked file neither blocks nor is touched | `worktree_merge_commit_bookkeeping: false` in config turns the auto-commit off — then a dirty main refuses unconditionally. Mirrors `bee close`'s own bookkeeping auto-commit |
| `bee close` (every lane) · `worktree merge` (`WORKTREE_MERGE_DISSENT_DEBT`) | a recorded dissent on any of the feature's cells has no `cells dissent-verdict` — the worker's question is data the orchestrator owes an answer | none — record the verdict (`accept` / `reject` / `escalate`, with `--reason`; it lands in the decision log) |
| `cells finish` (cap) · `bee close` · `worktree merge` (advisor-nudge debt) | the feature has an unanswered supervisor `advisor-nudge` intervention (`feature_advisor_nudge_debt` > 0) | log a decision tagged `advisor-nudge` whose text names the nudge row's id — an answer, not a dismissal |
| `dispatch prepare --kind advisor --brief-file` (LaneBrief guard) | the brief leans — a verdict stem or an invalid section shape betrays a preferred answer in what must be a neutral brief for blind lanes | none — rewrite the brief neutral; the check is lexical, and a quoted rival proposal rides a tagged fence past it |
| `dispatch prepare`, runtime `pi` (`pi_requires_herding`) | any non-herding dispatch shape on the Pi runtime — Pi has no subagent tool surface, so it is a herding-only preview transport | none — route the dispatch through `bee herding run`, or use another runtime |
| `worktree merge` (`WORKTREE_MERGE_UAT_PENDING`) | under `uat_stop: "merge"` (default), the feature's lane is standard/high-risk (missing/unrecognized lane fails closed as standard) and its `uat` gate is not approved | approve it (`bee gate --name uat --approved true`), or skip uat for JUST this merge (`bee worktree merge --id <id> --skip-uat`), or turn the door off repo-wide (`uat_stop: "off"`). Never auto-approved — `uat` is user-only at every `gate_bypass` level (uat-gate-before-merge D1) |
| `bee close` (`uat` door, headline "Uat gate pending for") | under `uat_stop: "close"`, standard/high-risk lane (same fail-closed rule), the merged feature's `uat` gate is not yet approved | approve it (`bee gate --name uat --approved true`), or log a `uat-deferral` decision naming the feature (uat-stop-placement D2) |
| `worktree merge` (`WORKTREE_MERGE_STAGING_FORBIDDEN`) | the worktree/branch being merged IS the staging branch — staging is disposable, never a source main merges from | none — the catastrophic direction has no hatch; the only exit is removing the staging config/record by hand (staging-lane D0) |

Three notes on doors that are not refusals:

- **`bee orient`'s capture queue escalates.** Below 10 pending stubs *and* under 7
  days old, it is an offer line; at or past either threshold it moves into
  `work.blockers[]`. Nothing is blocked mechanically — the blocker is the report.
- **`cells finish` is the one worktree exemption.** Every other mutating cells verb
  refuses to run from a granted linked worktree and names the main checkout. `finish`
  resolves its cell and claim at the main store, commit-only proof recorded from
  the calling worktree's own directory, so a worker caps where it worked and
  records its own proof line there; `bee close`/`bee worktree merge` check
  that recorded proof and run nothing themselves.
- **A `NEEDS_REVISION` verdict reopens its cell.** `cells judge-record` recording
  `NEEDS_REVISION` after a cap does not just log a finding — it moves the cell
  capped → open, clears its claim and verify evidence, and sends it back for rework.
  A reopened cell cannot re-cap until a fresh independent verdict clears it.

### Declared but not built

15 registry entries have no implementation in the current binary — the R6 Node
deletion removed the only one they had (`state advisor-ref record/show` and
`herding enable/disable/status`, once in this list, have since been ported and
are built). Each refuses by name, states that nothing ran and nothing changed,
and names its fallback:

| Unbuilt | Count |
|---|---|
| `config get/set/unset/validate` | 4 |
| `perf start/stop/section/log/render/report/sync` | 7 |
| `state compact-capsule/check/log` | 3 |
| `recovery window` | 1 |

Treat them as known gaps, not as verbs to route around: config changes go through
the hand-edit exception above, and `bee doctor` (now ported) plus
`bee status --json` cover install and onboarding health.

Run `.bee/bin/bee --help --json` for the porcelain manifest, `--help --all --json`
for every command, or `<group> --help` before a group's first use in a session.
