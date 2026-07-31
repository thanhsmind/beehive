// command-registry.mjs — the single source of truth for every subcommand
// bee.mjs's dispatcher accepts, across all 9 groups (status, cells,
// reservations, decisions, state, backlog, capture, reviews, feedback).
//
// D3 (harness-integration CONTEXT.md): each entry's `parameters` field is
// JSON-Schema in the exact shape Claude Code's own tool definitions use —
// {type:"object", properties, required} — never a bespoke shape. This is what
// makes the `bee --help --json` manifest zero-translation for any
// Claude-based agent.
//
// The 9 bee_*.mjs shims that used to sit in front of this registry are
// retired (shim-retire D1/D5) — `bee.mjs <group> <verb>` is now the sole
// canonical and sole shipped CLI, so entries no longer carry an informational
// `helper` field naming a shim; only {name, invoke, description, parameters,
// examples, deprecated} make up an entry, plus an optional `surface` field:
// 'porcelain' marks the small flow-verb set the default `bee --help` shows
// (docs/specs/porcelain.md); absent reads as 'plumbing' — still shipped,
// still invocable, listed under `bee --help --all`. Presentation only:
// manifest drift detection keeps hashing the FULL registry.
//
// `examples[]` are literal, runnable `bee <group> <verb> ...` argument
// strings — the manifest-as-tested-contract discipline (every example is
// executed by tests/test_bee_cli.mjs and asserted not to error) holds
// against the unified dispatcher and, via the shims, against every
// bee_*.mjs entrypoint too.

import { MODEL_TIERS, KNOWN_PHASES, GATE_NAMES, HANDOFF_KINDS } from './state.mjs';
import { REVIEW_MODES } from './reviews.mjs';
import { COMPACT_EVENTS, ANCHOR_NUDGE_COMMAND } from './compaction.mjs';

export const SCHEMA_VERSION = '1.0';

// Mirrors the status enum cells.mjs's addCell/claimCell/capCell/blockCell/
// dropCell transition between (open -> claimed -> capped, or -> blocked /
// dropped at any point). Not re-exported by cells.mjs today, so restated here
// deliberately narrow — this is the one place a future status rename would
// need to update alongside cells.mjs itself.
const CELL_STATUSES = ['open', 'claimed', 'capped', 'blocked', 'dropped'];

// cells.cap and cells.finish share this one schema by construction: finish IS
// the cap door (same proof rules, same refusals) plus reservation release —
// never a weakened or extended restatement.
const CELL_CAP_PARAMETERS = {
  type: 'object',
  properties: {
    id: { type: 'string', description: 'Cell id, e.g. auth-3.' },
    outcome: { type: 'string', description: 'One-line outcome summary.' },
    files: { type: 'string', description: 'Comma-separated list of files the worker changed.' },
    'feature-verify-pending': { type: 'boolean', description: 'Cap through the feature-verify-pending path: no per-cell verify evidence demanded, trace.feature_verify stamped "pending". Refused when combined with per-cell evidence claims. The feature-level verify record (state feature-verify record --result green) is what later satisfies the close door.' },
    'behavior-change': { type: 'boolean', description: 'Force behavior_change true (a cell-declared true cannot be unset by omitting this flag).' },
    'evidence-stdin': { type: 'boolean', description: 'Read verification_evidence JSON from stdin (preferred — no evidence file is persisted).' },
    'evidence-file': { type: 'string', description: 'Path to a verification_evidence JSON file (back-compat; prefer --evidence-stdin).' },
    'deviations-file': { type: 'string', description: 'Path to a deviations list (JSON array or newline-delimited text).' },
    friction: { type: 'string', description: 'One-line friction note, only when a friction trigger fired.' },
    'override-judge': { type: 'string', description: 'Audited override reason — required to cap a cell whose latest semantic-judge verdict is NEEDS_REVISION (refused otherwise with JUDGE_REWORK_REQUIRED); recorded to trace.judge_overrides and logged as a decision.' },
    'session-id': { type: 'string', description: 'Acting session identity, for the claim-ownership guard. Optional — resolves from CLAUDE_CODE_SESSION_ID when omitted.' },
    'force-ownership': { type: 'boolean', description: 'Override a live claim owned by a different session (audited).' },
    json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
  },
  required: ['id'],
};

export const COMMAND_REGISTRY = [
  // ─── status (bee_status.mjs — no subcommand, flags only) ─────────────────
  {
    name: 'status',
    invoke: 'bee status',
    surface: 'porcelain',
    description:
      'Read-only snapshot: onboarding health, phase, gates, handoff, cell counts, reservations, active workers, decisions, staleness warnings, recommended next step. `lanes` is summarized by default: the ACTIVE lane (the one this session is bound to) in full, plus counts-by-phase and bare ids for every other lane record — pass --lanes-full for the full per-lane array. `workers` is a derived view — live-heartbeat sessions joined with their current cell claim — never state.json\'s hand-mutated `workers` array.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of the text report.' },
        'lanes-full': {
          type: 'boolean',
          description:
            'Restore the `lanes` field to its full per-lane array (every lane record in full, including bound_sessions) instead of the default summary ({active, counts, ids}). Payload-size only — every other top-level field (phase/mode/feature/gates/cells/recommended_next/...) is unaffected either way.',
        },
        brief: {
          type: 'boolean',
          description:
            'Fast orientation path for worker startup. Reads ONLY the state layer (state.json + config-derived gate_bypass_level/ship_visibility) — no cell scan, no review/handoff resolution, no models/tier_mix. Emits exactly {phase, feature, mode, gates, gate_bypass_level, ship_visibility, route} (route null when absent); ignored together with --lanes-full (mutually exclusive fast/full paths — --brief wins). Full `status` (this flag omitted) is unchanged.',
        },
      },
      required: [],
    },
    examples: ['bee status --json', 'bee status --lanes-full --json', 'bee status --brief --json'],
    deprecated: null,
  },

  // ─── orient (porcelain) — the session-start context packet ────────────────
  {
    name: 'orient',
    invoke: 'bee orient',
    surface: 'porcelain',
    description:
      'Read-only session-start context packet: where the work is ({phase, feature, mode, gates, gate_bypass_level}), the decisions in force (active count, up to 3 recent one-liners, the feature\'s CONTEXT.md path when one exists on disk), the work state (open/claimed/capped counts, up to 5 ready cell ids, blockers), and exactly one recommended next step ({action, skill, command}) — the skill to load for the current phase, and a runnable command when the action names one (null otherwise). Reshapes the same snapshot `bee status` builds; never a second state computation.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit the machine-readable packet ({where, decisions, work, next}) instead of the six-line text summary.' },
      },
      required: [],
    },
    examples: ['bee orient --json'],
    deprecated: null,
  },

  // ─── cells (bee_cells.mjs) ────────────────────────────────────────────────
  {
    name: 'cells.list',
    invoke: 'bee cells list',
    description: 'List cells, optionally filtered by feature and/or status.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Restrict to one feature slug.' },
        status: { type: 'string', description: 'Restrict to one status.', enum: CELL_STATUSES },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-cell summary.' },
      },
      required: [],
    },
    examples: ['bee cells list --json'],
    deprecated: null,
  },
  {
    name: 'cells.ready',
    invoke: 'bee cells ready',
    surface: 'porcelain',
    description: 'List open cells whose deps are all capped — claimable right now.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Restrict to one feature slug.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-cell summary.' },
      },
      required: [],
    },
    examples: ['bee cells ready --json'],
    deprecated: null,
  },
  {
    name: 'cells.show',
    invoke: 'bee cells show',
    surface: 'porcelain',
    description:
      'Show one cell by id, including its full trace. The output carries a `verify_owner` field (JSON and text alike) stating that the `verify` command is MAIN\'s — run once at feature close, never by the assigned worker.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id, e.g. auth-3.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of pretty-printed JSON (show always prints JSON; flag kept for surface consistency).' },
      },
      required: ['id'],
    },
    examples: ['bee cells show --id demo-1 --json'],
    deprecated: null,
  },
  {
    name: 'cells.add',
    invoke: 'bee cells add',
    surface: 'porcelain',
    description:
      'Add a cell (or a whole-slice JSON array) from stdin, or from a JSON file. Prefer --stdin: pipe one cell object or an array for the whole slice in one call — no per-cell scratchpad files. Exactly one of --stdin / --file is required at call time (both satisfy the schema; the handler itself enforces the choice). A batch is validated WHOLE: schema, regen obligations, duplicate ids, and the dependency-cycle check are checked for every cell before any is written, so one call names every problem instead of stopping at the first. --dry-run runs that same whole-batch validation and reports {dry_run:true, ok, cells:[{id, ok, problems}]} without writing anything, clean or dirty — exit 0 when every cell is clean, non-zero otherwise.',
    parameters: {
      type: 'object',
      properties: {
        file: { type: 'string', description: 'Path to a cell JSON file. Required unless --stdin is set.' },
        stdin: { type: 'boolean', description: 'Read the cell JSON from stdin instead of --file.' },
        'dry-run': {
          type: 'boolean',
          description:
            'Report the whole-batch validation verdict (schema, regen obligations, duplicate ids, dependency cycle) without writing anything.',
        },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee cells add --file cell-demo-1.json --json', 'bee cells add --stdin --dry-run --json'],
    deprecated: null,
  },
  {
    name: 'cells.update',
    invoke: 'bee cells update',
    description:
      'Door-validated in-place revision for validation-repair loops: only open|blocked cells are updatable. Plan fields only (title/action/verify/files/read_first/deps/decisions/must_haves/behavior_change/lane/pbi) — verify is plan text the worker edits but never runs; MAIN owns running it at feature close. Frozen keys (id/feature/status/trace/tier) and any unknown key refuse the whole patch untouched. Exactly one of --file / --stdin is required at call time (both satisfy the schema; the handler itself enforces the choice).',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id to update.' },
        file: { type: 'string', description: 'Path to a patch JSON file. Required unless --stdin is set.' },
        stdin: { type: 'boolean', description: 'Read the patch JSON from stdin instead of --file.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id'],
    },
    examples: ['bee cells update --id demo-1 --file cell-demo-1-update.json --json'],
    deprecated: null,
  },
  {
    name: 'cells.claim',
    invoke: 'bee cells claim',
    description:
      'Claim an open, dep-free cell for a worker. Refuses while Gate 3 (execution) is unapproved or deps are uncapped. Backed by the same O_EXCL claim file claim-next uses (claims.mjs claimCellFile, acquired before the cell JSON flips) — a losing concurrent claimant gets a typed CLAIMED refusal naming the owner + expiry instead of silently double-owning the cell. --session-id is optional — resolves from CLAUDE_CODE_SESSION_ID when omitted, and falls back to a legal sessionless claim when neither is present (single-session use is unaffected).',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id to claim.' },
        worker: { type: 'string', description: 'Reservation identity of the claiming worker.' },
        'session-id': { type: 'string', description: 'Claiming session identity (claims.mjs). Optional — resolves from CLAUDE_CODE_SESSION_ID, then falls back to a sessionless claim.' },
        ttl: { type: 'number', description: 'Claim TTL in seconds (default 3600).' },
        isolate: { type: 'boolean', description: 'Route this claim through the write-policy isolate-create path (a fresh worktree) instead of claiming directly in this checkout. Only meaningful when the write-policy mode actually enforces isolation.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'worker'],
    },
    examples: ['bee cells claim --id demo-1 --worker worker-a --json'],
    deprecated: null,
  },
  {
    name: 'cells.verify',
    invoke: 'bee cells verify',
    description: "Record a verify run's command, output, and pass/fail for a cell — the proof `cap` later requires.",
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id.' },
        command: { type: 'string', description: 'The exact verify command that was run.' },
        passed: { type: 'boolean', description: 'Whether the verify run passed ("true" or "false").' },
        output: { type: 'string', description: 'What the verify command printed (inline). Mutually exclusive with --output-file.' },
        'output-file': { type: 'string', description: 'Path to a file holding the verify command\'s output, for long output.' },
        signature: { type: 'string', description: 'Explicit failure_signature for the revision ledger, overriding the mechanical normalizer (ignored when --passed true).' },
        'session-id': { type: 'string', description: 'Acting session identity, for the claim-ownership guard. Optional — resolves from CLAUDE_CODE_SESSION_ID when omitted.' },
        'force-ownership': { type: 'boolean', description: 'Override a live claim owned by a different session (audited).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'command', 'passed'],
    },
    examples: ['bee cells verify --id demo-1 --command "manual check" --output "0 failing" --passed true --json'],
    deprecated: null,
  },
  {
    name: 'cells.cap',
    invoke: 'bee cells cap',
    description: 'Cap a cell — refuses without a recorded passing verify (and, for small+ lanes, recorded output/evidence plus non-empty files_changed). Exception: --feature-verify-pending caps with NO per-cell verify evidence, stamping trace.feature_verify: "pending" — the proof is deliberately relocated to ONE feature-level verify (`bee state feature-verify record`), and leaving phase "swarming" is refused until a fresh green record covers every pending cap (no bypass level lifts it). Combining the flag with per-cell evidence claims (--evidence-file/--evidence-stdin, or an already-recorded passing verify) is refused — the two proof paths are exclusive.',
    parameters: CELL_CAP_PARAMETERS,
    examples: ['bee cells cap --id demo-1 --outcome "demo cell capped" --files cell-demo-1.json --json'],
    deprecated: null,
  },
  {
    name: 'cells.finish',
    invoke: 'bee cells finish',
    surface: 'porcelain',
    description:
      'The worker\'s single completion verb: cap the cell through exactly the `cells cap` door — same parameters, same proof rules, refusals pass through unchanged — then, on a successful cap, release every reservation the cell\'s claiming agent holds for that cell. The result is cap\'s result plus `released` (the freed paths); the text ends by telling the worker what to return. A release failure never rolls the cap back — it is reported with the exact `bee reservations release` command to run manually. `cells cap` and `reservations release` stay available for stepwise completion.',
    parameters: CELL_CAP_PARAMETERS,
    examples: ['bee cells finish --id demo-fin-1 --outcome "demo cell finished" --files cell-demo-fin-1.json --json'],
    deprecated: null,
  },
  {
    name: 'cells.block',
    invoke: 'bee cells block',
    description: 'Mark a cell blocked with a reason.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id.' },
        reason: { type: 'string', description: 'Why the cell is blocked.' },
        'session-id': { type: 'string', description: 'Acting session identity, for the claim-ownership guard. Optional — resolves from CLAUDE_CODE_SESSION_ID when omitted.' },
        'force-ownership': { type: 'boolean', description: 'Override a live claim owned by a different session (audited).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'reason'],
    },
    examples: ['bee cells block --id demo-1 --reason "test block" --json'],
    deprecated: null,
  },
  {
    name: 'cells.drop',
    invoke: 'bee cells drop',
    description: 'Mark a cell dropped with a reason.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id.' },
        reason: { type: 'string', description: 'Why the cell was dropped.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'reason'],
    },
    examples: ['bee cells drop --id demo-1 --reason "test drop" --json'],
    deprecated: null,
  },
  {
    name: 'cells.unclaim',
    invoke: 'bee cells unclaim',
    description: 'Release a claimed cell back to open (the inverse of claim) so another worker can pick it up. Refuses on any non-claimed status.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id to unclaim.' },
        'session-id': { type: 'string', description: 'Acting session identity, for the claim-ownership guard. Optional — resolves from CLAUDE_CODE_SESSION_ID when omitted.' },
        'force-ownership': { type: 'boolean', description: 'Override a live claim owned by a different session (audited).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id'],
    },
    examples: ['bee cells unclaim --id demo-1 --json'],
    deprecated: null,
  },
  {
    name: 'cells.reopen',
    invoke: 'bee cells reopen',
    description: 'Return a capped, blocked, or dropped cell to open for rework, with a reason. Clears the recorded verify so the reopened cell must re-verify before it can cap again. Use unclaim (not reopen) for a claimed cell.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id to reopen.' },
        reason: { type: 'string', description: 'Why the cell is being reopened for rework.' },
        'session-id': { type: 'string', description: 'Acting session identity, for the claim-ownership guard. Optional — resolves from CLAUDE_CODE_SESSION_ID when omitted.' },
        'force-ownership': { type: 'boolean', description: 'Override a live claim owned by a different session (audited).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'reason'],
    },
    examples: ['bee cells reopen --id demo-1 --reason "needs rework" --json'],
    deprecated: null,
  },
  {
    name: 'cells.tier',
    invoke: 'bee cells tier',
    description: "Record the orchestrator's dispatch-time model-tier judgment for a cell.",
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id.' },
        tier: { type: 'string', description: 'Model tier chosen at dispatch.', enum: [...MODEL_TIERS] },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'tier'],
    },
    examples: ['bee cells tier --id demo-1 --tier generation --json'],
    deprecated: null,
  },
  {
    name: 'cells.judge',
    invoke: 'bee cells judge',
    description: "Frozen-judge check: flags test/CI/lockfile files changed outside the cell's declared file scope.",
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line verdict.' },
      },
      required: ['id'],
    },
    examples: ['bee cells judge --id demo-1 --json'],
    deprecated: null,
  },
  {
    name: 'cells.claim-next',
    invoke: 'bee cells claim-next',
    description:
      "Cross-session selection + claim: sweeps stale claims (TTL expired AND heartbeat stale) in-pass first — this IS sweepExpiredClaims's production trigger — then picks the next open cell to claim: the acting session's own bound lane (or the default pipeline when unbound) first, ONLY when its execution gate is approved; empty or unapproved falls back to every OTHER pipeline whose OWN execution gate is approved (an unapproved lane is never touched), ordered by backlog rank then lane created_at. Cells whose files intersect another session's active reservation hold are skipped (the acting session's own holds never exclude a cell). Claims via the two-store sequence (claims.mjs claimCellFile then cells.mjs claimCell, unwound with a claim-file release on any claimCell throw). Refuses (non-zero exit) when nothing is claimable (NO_APPROVED_WORK), the claims-store race is lost (CLAIMED), or the session's lane binding is broken (LANE_INVALID/LANE_MISSING/LANE_CORRUPT). --session-id is not required at the schema level — omit it and it resolves from CLAUDE_CODE_SESSION_ID instead; a session id is still functionally required (claim-next resolves the acting session's own lane from it), so a call with neither still refuses, just from the handler rather than arg validation.",
    parameters: {
      type: 'object',
      properties: {
        worker: { type: 'string', description: 'Reservation identity of the claiming worker.' },
        'session-id': { type: 'string', description: "Acting session's cross-session identity (claims.mjs) — resolves its bound lane, if any. Optional — falls back to CLAUDE_CODE_SESSION_ID; a call with neither is refused by the handler." },
        ttl: { type: 'number', description: 'Claim TTL in seconds (default 3600).' },
        isolate: { type: 'boolean', description: 'Route this claim through the write-policy isolate-create path (a fresh worktree) instead of claiming directly in this checkout. Only meaningful when the write-policy mode actually enforces isolation.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['worker'],
    },
    examples: ['bee cells claim-next --worker worker-a --session-id sess-claim-next --json'],
    deprecated: null,
  },
  {
    name: 'cells.reset-budget',
    invoke: 'bee cells reset-budget',
    description:
      'The ONLY door that reopens a cell whose claim door is closed by CELL_BUDGET_EXHAUSTED or REPEATED_FAILURE. Refuses (typed RESET_NOT_NEEDED) unless the cell is actually budget-blocked, and refuses without an actor (--operator, or the BEE_AGENT_NAME env fallback). Requires --reason (audited), logs a decision BEFORE writing the cell (the audit survives even if the write itself fails), and appends {reset_at, reason, by_session, by_actor} to the append-only trace.budget_resets — never rewrites or drops any trace.attempts ledger entry. gate_bypass never substitutes for this: the budget check itself never reads bypass config, so this verb is the only reopening path at any bypass level.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id whose claim-lifetime budget door is closed.' },
        reason: { type: 'string', description: 'Why a retry is warranted — required, logged to the decision log.' },
        operator: { type: 'string', description: 'Acting operator/agent name, recorded as by_actor in the audit trail and the decision text. Optional — falls back to the BEE_AGENT_NAME environment variable when omitted; refused when neither is present.' },
        'session-id': { type: 'string', description: 'Resetting session identity, recorded as by_session. Optional — resolves from CLAUDE_CODE_SESSION_ID when omitted.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'reason'],
    },
    examples: ['bee cells reset-budget --id demo-1 --reason "manager approved a genuine retry after a real fix" --operator "manager-session" --json'],
    deprecated: null,
  },
  {
    name: 'cells.judge-record',
    invoke: 'bee cells judge-record',
    description:
      'Validates a judge-verdict/1 payload (--file) and appends it, stamped with model_independence, to the append-only trace.semantic_judge. Refuses (typed, non-zero exit) on free prose, an unknown verdict/status/fixability/confidence value, or a FAIL check missing failure_signature — never a silent pass. --builder-model/--judge-model (optional) mark that side PINNED for independence derivation: both present AND differing -> "confirmed"; both present AND equal -> "same-model" (honest — the judge still runs); either absent -> "unverified". Never reads .bee/logs/dispatch.jsonl (corroboration only) — the models are caller-supplied from the orchestrator\'s own pinned dispatch params.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Cell id the verdict is being recorded against.' },
        file: { type: 'string', description: 'Path to a judge-verdict/1 JSON payload: {schema, verdict, checks[], failure_signature?, fixability, confidence}.' },
        'builder-model': { type: 'string', description: 'The resolved model name of the cell\'s builder dispatch, from the orchestrator\'s own pinned dispatch param. Omit when not pinned.' },
        'judge-model': { type: 'string', description: 'The resolved model name of this judge dispatch, from the orchestrator\'s own pinned dispatch param. Omit when not pinned.' },
        'session-id': { type: 'string', description: 'Recording session identity, for the claim-ownership guard. Optional — resolves from CLAUDE_CODE_SESSION_ID when omitted.' },
        'force-ownership': { type: 'boolean', description: 'Override a live claim owned by a different session (audited).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'file'],
    },
    examples: ['bee cells judge-record --id demo-1 --file verdict-demo-1.json --builder-model sonnet --judge-model opus --json'],
    deprecated: null,
  },
  {
    name: 'cells.schedule',
    invoke: 'bee cells schedule',
    description:
      'Compute the wave schedule for a feature: dep layering + file-overlap serialization, with cycle/unsatisfiable-dep/empty-files diagnostics.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Restrict to one feature slug. Omit to schedule every cell.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON (waves + diagnostics) instead of a human-readable report.' },
      },
      required: [],
    },
    examples: ['bee cells schedule --json'],
    deprecated: null,
  },
  {
    name: 'cells.archive',
    invoke: 'bee cells archive',
    description:
      'Move a fully-terminal feature\'s cells (every cell capped or dropped — refuses naming any open/claimed cell) out of the hot .bee/cells/ scan path into .bee/cells/archive/<feature>/, and record its capped/dropped counts in the archive summary ledger so `bee status` reports an honest archived total without scanning the archive tree. Refuses when --feature is the active state.feature (archiving in-flight work is never legal) or when the feature has zero cells.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug to archive — must be fully terminal (all cells capped/dropped) and NOT the active state.feature.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['feature'],
    },
    examples: ['bee cells archive --feature demo-archive --json'],
    deprecated: null,
  },
  {
    name: 'cells.unarchive',
    invoke: 'bee cells unarchive',
    description:
      'Reverse of cells.archive: moves a feature\'s cells back from .bee/cells/archive/<feature>/ into the active .bee/cells/ dir and drops that feature\'s entry from the archive summary ledger. Refuses when the feature has nothing archived.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug to restore from the archive.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['feature'],
    },
    examples: ['bee cells unarchive --feature demo-archive --json'],
    deprecated: null,
  },

  // ─── reservations (bee_reservations.mjs) ─────────────────────────────────
  {
    name: 'reservations.reserve',
    invoke: 'bee reservations reserve',
    surface: 'porcelain',
    description: "Reserve a file or glob path for a cell. A conflicting active reservation held by another agent returns ok:false with the holder(s). Optional --session stamps the reservation as owned by that cross-session identity, so the write guard's hold check (checkWrite) can deny another live session's write into the same path — a reservation made without --session keeps today's exact intra-swarm-only semantics. Optional --kind is 'intent' or 'lease' (default 'lease'): 'lease' is a worker's own write-time reservation and stays a hard conflict; 'intent' declares a broad/glob planning-time scope that the write guard only warns about (never hard-blocks) unless it collapses onto the exact write target.",
    parameters: {
      type: 'object',
      properties: {
        agent: { type: 'string', description: 'Reservation identity making the request.' },
        cell: { type: 'string', description: 'Cell id the reservation is for.' },
        path: { type: 'string', description: 'File or directory path to reserve.' },
        ttl: { type: 'number', description: 'Time-to-live in seconds (default 3600).' },
        session: { type: 'string', description: 'Owning cross-session identity. Omit to keep an intra-swarm-only reservation with no cross-session hold effect.' },
        kind: { type: 'string', description: "'intent' or 'lease' (default 'lease'). 'intent' is advisory-only (warn, never hard-block) on overlap; 'lease' is a hard conflict, unchanged from before this flag existed." },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['agent', 'cell', 'path'],
    },
    examples: [
      'bee reservations reserve --agent worker-a --cell demo-1 --path src/example.ts --json',
      'bee reservations reserve --agent worker-a --cell demo-1 --path src/example-session.ts --session sess-fsh7 --json',
      'bee reservations reserve --agent planner --cell demo-1 --path "src/api/*" --kind intent --json',
    ],
    deprecated: null,
  },
  {
    name: 'reservations.release',
    invoke: 'bee reservations release',
    description: "Release an agent's reservations, optionally scoped to one cell.",
    parameters: {
      type: 'object',
      properties: {
        agent: { type: 'string', description: 'Reservation identity releasing its holds.' },
        cell: { type: 'string', description: 'Restrict release to reservations for this cell id.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['agent'],
    },
    examples: ['bee reservations release --agent worker-a --cell demo-1 --json'],
    deprecated: null,
  },
  {
    name: 'reservations.list',
    invoke: 'bee reservations list',
    description: 'List reservations, optionally active-only.',
    parameters: {
      type: 'object',
      properties: {
        'active-only': { type: 'boolean', description: 'Only list reservations not released and not TTL-expired.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-reservation summary.' },
      },
      required: [],
    },
    examples: ['bee reservations list --active-only --json'],
    deprecated: null,
  },
  {
    name: 'reservations.sweep',
    invoke: 'bee reservations sweep',
    description: 'Release every TTL-expired reservation that was never explicitly released.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee reservations sweep --json'],
    deprecated: null,
  },

  // ─── decisions (bee_decisions.mjs) ───────────────────────────────────────
  {
    name: 'decisions.log',
    invoke: 'bee decisions log',
    surface: 'porcelain',
    description:
      'Append a decision event to the append-only decision log. Rejects secret-shaped or instruction-like content. Once docs/decisions/taxonomy.json exists, a zero-tag event is refused (typed, names --tags); without that file it warns and proceeds. An unknown tag is always accepted and appended to the taxonomy\'s candidates[] in the same call — never refused, never a second call.',
    parameters: {
      type: 'object',
      properties: {
        decision: { type: 'string', description: 'The decision text.' },
        rationale: { type: 'string', description: 'Why this decision was made.' },
        alternatives: { type: 'string', description: 'Alternatives considered, if any.' },
        scope: { type: 'string', description: 'Decision scope (default "repo").' },
        source: { type: 'string', description: 'Who/what decided (default "user").' },
        confidence: { type: 'number', description: 'Confidence, 0-100.' },
        tags: { type: 'array', description: 'Comma-separated lowercase slugs (e.g. "billing,nightly-job"), stored on the event for later --tag recall.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['decision', 'rationale'],
    },
    examples: [
      'bee decisions log --decision "Use in-repo registry for CLI commands" --rationale "Avoid duplicated validation logic across dispatcher and hook" --tags cli,registry --json',
    ],
    deprecated: null,
  },
  {
    name: 'decisions.supersede',
    invoke: 'bee decisions supersede',
    description:
      'Replace an earlier decision with a new one; the earlier decision drops out of the active set. Runs a propagation sweep of docs/** for citations of the superseded id and queues a capture stub per hit. Tag/scope inheritance consults the OVERLAY-APPLIED target, so a legacy target classified only via a retro-tag event still counts as tagged. Once docs/decisions/taxonomy.json exists, the final (explicit-or-inherited) tag set follows the same zero-tag refusal / unknown-tag-accepted rule as `decisions log`.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Id of the decision being superseded.' },
        decision: { type: 'string', description: 'The replacement decision text.' },
        rationale: { type: 'string', description: 'Why the replacement supersedes the original.' },
        tags: { type: 'array', description: 'Comma-separated lowercase slugs. Omit to inherit the superseded target\'s tags.' },
        scope: { type: 'string', description: 'Decision scope. Omit to inherit the superseded target\'s scope, falling back to "repo" for a metadata-less target.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'decision', 'rationale'],
    },
    examples: [
      'bee decisions supersede --id 00000000-0000-0000-0000-000000000000 --decision "Superseding decision" --rationale "Updated approach" --json',
    ],
    deprecated: null,
  },
  {
    name: 'decisions.redact',
    invoke: 'bee decisions redact',
    description: 'Redact a decision from the active set with a reason (the event stays in the log; only its active status changes).',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Id of the decision being redacted.' },
        reason: { type: 'string', description: 'Why the decision was redacted.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'reason'],
    },
    examples: ['bee decisions redact --id 00000000-0000-0000-0000-000000000000 --reason "test redaction" --json'],
    deprecated: null,
  },
  {
    name: 'decisions.active',
    invoke: 'bee decisions active',
    surface: 'porcelain',
    description: 'List active (non-superseded, non-redacted) decisions, newest first. Optional structured filters narrow the list; --recent applies after filtering.',
    parameters: {
      type: 'object',
      properties: {
        recent: { type: 'number', description: 'Return only the N most recent (post-filter) active decisions.' },
        tag: { type: 'string', description: 'Exact tag match, case-insensitive.' },
        scope: { type: 'string', description: 'Exact scope match, case-insensitive (scope is the spec-area dimension).' },
        area: { type: 'string', description: 'Alias for --scope.' },
        since: { type: 'string', description: 'ISO date; only events on/after this date (inclusive).' },
        all: { type: 'boolean', description: 'Also reach events archived by `decisions archive` — a union read of the active store and .bee/decisions-archive.jsonl, de-duplicated by id. Omit for the default active-store-only read.' },
        untagged: { type: 'boolean', description: 'List only events with no tags AFTER the retro-tag overlay is applied. Composable with every other filter, including --all.' },
        cell: { type: 'string', description: 'Word-boundary text match for a cell id (e.g. "si-1") over decision/rationale/alternatives — a decide event has no structural cell field, so this is a token match, not substring: "si-1" excludes "si-10".' },
        feature: { type: 'string', description: 'Word-boundary text match for a feature slug over decision/rationale/alternatives — same token-match discipline as --cell, not substring.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a formatted list.' },
      },
      required: [],
    },
    examples: ['bee decisions active --recent 5 --json', 'bee decisions active --tag billing --json', 'bee decisions active --all --json', 'bee decisions active --untagged --json', 'bee decisions active --cell si-1 --json'],
    deprecated: null,
  },
  {
    name: 'decisions.search',
    invoke: 'bee decisions search',
    description:
      'Search active decisions by multi-term text match and/or structured filters. --text is required only when no structured filter (--tag/--scope/--area/--since/--untagged) is given. --text is whitespace-split into terms, case-insensitive, OR across terms, matched over decision/rationale/alternatives AND (overlay-applied) tags — results are ranked by deterministic term-hit count descending, then date descending; a single term matches everything the old substring search matched, and more.',
    parameters: {
      type: 'object',
      properties: {
        text: { type: 'string', description: 'Whitespace-separated search terms (case-insensitive, OR-matched, ranked by hit count). Optional when a structured filter is present.' },
        tag: { type: 'string', description: 'Exact tag match, case-insensitive.' },
        scope: { type: 'string', description: 'Exact scope match, case-insensitive (scope is the spec-area dimension).' },
        area: { type: 'string', description: 'Alias for --scope.' },
        since: { type: 'string', description: 'ISO date; only events on/after this date (inclusive).' },
        all: { type: 'boolean', description: 'Also reach events archived by `decisions archive` — a union read of the active store and .bee/decisions-archive.jsonl, de-duplicated by id. Omit for the default active-store-only read.' },
        untagged: { type: 'boolean', description: 'List only events with no tags AFTER the retro-tag overlay is applied — the classification-completeness check (should reach zero once a backfill is done). Composable with every other filter, including --all; satisfies the "at least one filter" requirement on its own.' },
        cell: { type: 'string', description: 'Word-boundary text match for a cell id (e.g. "si-1") over decision/rationale/alternatives — a decide event has no structural cell field, so this is a token match, not substring: "si-1" excludes "si-10". Satisfies the "at least one filter" requirement on its own.' },
        feature: { type: 'string', description: 'Word-boundary text match for a feature slug over decision/rationale/alternatives — same token-match discipline as --cell, not substring. Satisfies the "at least one filter" requirement on its own.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a formatted list.' },
      },
      required: [],
    },
    examples: ['bee decisions search --text "registry" --json', 'bee decisions search --tag billing --since 2026-07-01 --json', 'bee decisions search --tag billing --all --json', 'bee decisions search --untagged --json', 'bee decisions search --cell si-1 --json'],
    deprecated: null,
  },
  {
    name: 'decisions.archive',
    invoke: 'bee decisions archive',
    description: 'Move superseded/redacted decision events (always, regardless of age) plus decide events strictly older than --before from .bee/decisions.jsonl to .bee/decisions-archive.jsonl. --before is always required — there is no default age window. Refuses (typed) when zero events qualify. Use `decisions active --all` / `decisions search --all` to keep reaching archived events afterward.',
    parameters: {
      type: 'object',
      properties: {
        before: { type: 'string', description: 'ISO date. Plain decide events dated strictly before this are archived; superseded/redacted events are archived regardless of this cutoff. Required.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['before'],
    },
    // Far-future sentinel (not a real deadline) so the manifest-as-tested-
    // contract example (test_bee_cli.mjs) always has something to archive
    // regardless of when it actually runs — mirrors this repo's other
    // far-future-ceiling idioms (e.g. lock.mjs's HARD_STALE_MS comment).
    examples: ['bee decisions archive --before 2099-01-01 --json'],
    deprecated: null,
  },
  {
    name: 'decisions.tag',
    invoke: 'bee decisions tag',
    description:
      'Append a retro-tag event that overlays tags/scope onto an existing decide/supersede event WITHOUT rewriting its jsonl line — visible via `decisions active`/`decisions search` (including --all) at read time. --target accepts a full id or a unique short8 prefix. --stdin accepts a JSON array of {target, tags, scope?} for a batch: every entry is validated before any write, and one unresolvable target refuses the WHOLE batch (nothing appended). The latest tag event wins when several target the same decision; overlay REPLACES the whole tags array, and scope only when the tag event carries one.',
    parameters: {
      type: 'object',
      properties: {
        target: { type: 'string', description: 'Full id or short8 prefix of the decide/supersede event to tag. Required unless --stdin is set.' },
        tags: { type: 'array', description: 'Comma-separated lowercase slugs (e.g. "billing,nightly-job"). Required (unless --stdin) — replaces the target\'s effective tags entirely.' },
        scope: { type: 'string', description: 'Optional scope to overlay onto the target. Omit to leave the target\'s existing scope untouched.' },
        stdin: { type: 'boolean', description: 'Read a JSON array of {target, tags, scope?} from stdin for a batch retro-tag (all-or-nothing).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee decisions tag --target 00000000-0000-0000-0000-000000000000 --tags billing,recall --scope billing --json',
    ],
    deprecated: null,
  },
  {
    name: 'decisions.render',
    invoke: 'bee decisions render',
    description:
      'Render docs/decisions/index.md from the active decision store (overlay-aware): grouped by scope then tag (untagged last), newest-first inside each group, one line per decision "short8 · YYYY-MM-DD · first line of decision text". Superseded/redacted events are always excluded; a supersede event renders under its inherited scope/tags. Consumes the SAME overlay-applied read path as `decisions search`/`active`, so a retro-tagged legacy event renders under its overlaid scope/tags, never under "untagged". The file carries a provenance header and is deterministic (byte-identical for the same store) — it is regenerated only, never hand-edited.',
    parameters: {
      type: 'object',
      properties: {
        all: { type: 'boolean', description: 'Also reach events archived by `decisions archive` — same union-read flag as `decisions search`/`active`. Omit to render the active store only.' },
        check: { type: 'boolean', description: 'Read-only: compute the index and compare it byte-for-byte against docs/decisions/index.md on disk, without writing. Exits non-zero (and never writes) when the on-disk file is missing or has drifted (e.g. hand-edited) from what the store would render.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee decisions render --json'],
    deprecated: null,
  },

  // ─── state (bee_state.mjs — .bee/state.json mutation verbs) ───────────────
  // Nested worker verbs use a 3-segment name (state.worker.add) resolved by
  // the dispatcher's longest-prefix match; every other verb is 2-segment.
  //
  // `required: []` on every state entry is deliberate (DB3): the generic
  // validate() layer emits a structured error on STDOUT, but the legacy
  // bee_state.mjs contract (pinned by test_lib.mjs) emits missing-flag / bad-
  // value errors on STDERR. So each state handler owns its own required-flag
  // and enum checks (requireFlag / requireFlags / MODEL_TIERS / GATE_NAMES),
  // throwing the legacy message text — which the dispatcher routes to STDERR —
  // rather than letting validate() preempt it onto STDOUT. Types stay 'string'
  // for the same reason (a bad --approved must reach the handler, not validate).
  {
    name: 'state.set',
    invoke: 'bee state set',
    description:
      'Set one or more top-level routing fields; only the flags given are written and every other field is preserved. Every call requires explicit --owner equal to the selected default/lane record\'s valid pre-mutation phase; missing/mismatched ownership or a corrupt phase refuses before write, and a successful phase change rolls ownership forward without persisting an owner field. --phase is validated against the known-phase enum AND against the tail guard: "compounding" is never settable directly (only `state scribing-run` produces it), and "compounding-complete" is legal only from "compounding", only with zero scribing debt, and only with a FRESH recorded compounding run (`state compounding-run`, matching feature, at >= last_scribing_run.at) — --waive-compounding permits the close without one, logging a decision. Every other transition, including all backward moves and --phase idle, stays permissive. Target resolution (symmetric with the read path): explicit --lane <feature> always wins > the calling session\'s bound lane when --lane is omitted (identity self-resolved at operation moment) > the default state.json for an unbound session. --no-lane forces the default record from a bound session. A missing or corrupt lane — explicit or session-bound — refuses loudly with zero writes, never a silent fall-back to the default. --feature is rejected whenever the selected target is a lane record.',
    parameters: {
      type: 'object',
      properties: {
        phase: { type: 'string', description: 'Workflow phase to set.', enum: [...KNOWN_PHASES] },
        mode: { type: 'string', description: 'Mode to set.' },
        feature: { type: 'string', description: 'Feature slug to set. Rejected together with --lane.' },
        'next-action': { type: 'string', description: 'Top-level next_action string.' },
        summary: { type: 'string', description: 'Session summary string.' },
        owner: { type: 'string', description: 'Selected record\'s exact pre-mutation phase. Required for every state set mutation; never persisted.' },
        lane: { type: 'string', description: 'Route the mutation to this lane record instead of the default state.json. Refuses if the lane is missing or corrupt. Omitted: the calling session\'s bound lane is targeted automatically; unbound sessions target the default record.' },
        'no-lane': { type: 'boolean', description: 'Force the default state.json even when the calling session is bound to a lane. Cannot be combined with --lane.' },
        'waive-scribing-debt': { type: 'boolean', description: 'Permit --phase compounding-complete while capped behavior_change cells are still unsynced to docs/specs/. Never silent: it logs a decision naming every waived cell.' },
        'waive-compounding': { type: 'boolean', description: 'Permit --phase compounding-complete without a fresh recorded `state compounding-run` (matching the last scribing run, same feature). Never silent: it logs a decision naming the feature.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state set --owner exploring --phase planning --json',
      'bee state set --lane demo-lane --owner exploring --phase planning --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.gate',
    invoke: 'bee state gate',
    surface: 'porcelain',
    description: 'Approve or unapprove a named gate, OR pass --merge instead of --name to approve/unapprove the MERGED shape+execution gate in ONE call (sets approved_gates.shape AND approved_gates.execution together, since bee now asks a single question at the end of briefing in place of separate Gate 2/Gate 3 questions). --merge and --name are mutually exclusive. This dedicated command does not accept routing --owner. Idempotent: the same call run twice yields an identical file. Target resolution (symmetric with the read path): explicit --lane <feature> always wins > the calling session\'s bound lane when --lane is omitted > the default state.json for an unbound session. --no-lane forces the default record. A missing or corrupt lane — explicit or session-bound — refuses loudly with zero writes. A --merge approval inherits the SAME high-risk advisor-consult precondition that already guards a --name execution approval — refuses when mode is high-risk and advisor_ref is missing or stale, before any write. Under --merge, both gates carry the approved_for_plan_rev stamp (a plain --name execution approval still stamps execution alone), so a later `state plan-rev bump` revokes both instead of leaving the merged gate half-revoked.',
    parameters: {
      type: 'object',
      properties: {
        name: { type: 'string', description: 'Gate name. Required unless --merge is set; refused together with --merge.', enum: [...GATE_NAMES] },
        merge: { type: 'boolean', description: 'Approve/unapprove shape AND execution together in this one call, instead of a single named gate. Mutually exclusive with --name.' },
        approved: { type: 'string', description: 'Whether the gate(s) are approved ("true" or "false").' },
        lane: { type: 'string', description: 'Route the mutation to this lane record instead of the default state.json. Refuses if the lane is missing or corrupt. Omitted: the calling session\'s bound lane is targeted automatically; unbound sessions target the default record.' },
        'no-lane': { type: 'boolean', description: 'Force the default state.json even when the calling session is bound to a lane. Cannot be combined with --lane.' },
        owner: { type: 'string', description: 'NOT accepted by this dedicated command — declared here only so the dispatcher\'s central unknown-flag check still lets it through to this handler\'s own specific refusal ("--owner is not accepted..."); routing ownership protects generic `state set` fields only.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state gate --name execution --approved true --json',
      'bee state gate --lane demo-lane --name execution --approved true --json',
      'bee state gate --merge --approved true --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.plan-rev.bump',
    invoke: 'bee state plan-rev bump',
    description: "Bump a workflow's plan_rev by 1. plan_rev lives ONLY on the workflow record, so this verb targets a lane exactly like gate/set/scribing-run (explicit --lane > the calling session's bound lane) but REFUSES when resolution would land on the default (non-lane) record — that record's workflow is not yet kept in sync. The bump immediately rebuilds the lane's projection: any gate stamped with the PRE-bump plan_rev (the execution gate always; the shape gate too when it was approved via `state gate --merge`) now projects as unapproved in .bee/lanes/<feature>.json, so a subsequent `cells claim` against that lane's cells refuses right away. Never touches any other workflow's record — context/review on THIS workflow are also untouched (they are never rev-stamped, so they stay effective across a bump).",
    parameters: {
      type: 'object',
      properties: {
        lane: { type: 'string', description: 'Route the bump to this lane\'s workflow record. Refuses if the lane is missing/corrupt, or names no live workflow record. Omitted: the calling session\'s bound lane is targeted automatically.' },
        'no-lane': { type: 'boolean', description: 'Force default-record resolution — always refused (plan_rev is not yet default-path-synced). Named so the refusal is explicit rather than a silent fallback.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state plan-rev bump --lane demo-lane --json'],
    deprecated: null,
  },
  {
    name: 'state.worker.add',
    invoke: 'bee state worker add',
    description: 'Append a worker entry (nickname + cell, optional tier/status) to state.workers.',
    parameters: {
      type: 'object',
      properties: {
        nickname: { type: 'string', description: 'Worker nickname.' },
        cell: { type: 'string', description: 'Cell id the worker is assigned.' },
        tier: { type: 'string', description: 'Model tier chosen at dispatch.', enum: [...MODEL_TIERS] },
        status: { type: 'string', description: 'Worker status.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state worker add --nickname w1 --cell c1 --json'],
    deprecated: null,
  },
  {
    name: 'state.worker.update',
    invoke: 'bee state worker update',
    description: 'Merge the given fields onto an existing worker entry found by nickname.',
    parameters: {
      type: 'object',
      properties: {
        nickname: { type: 'string', description: 'Worker nickname to update.' },
        cell: { type: 'string', description: 'New cell id.' },
        tier: { type: 'string', description: 'New model tier.', enum: [...MODEL_TIERS] },
        status: { type: 'string', description: 'New worker status.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state worker update --nickname w1 --status done --json'],
    deprecated: null,
  },
  {
    name: 'state.worker.remove',
    invoke: 'bee state worker remove',
    description: 'Drop the worker entry matching the given nickname.',
    parameters: {
      type: 'object',
      properties: {
        nickname: { type: 'string', description: 'Worker nickname to remove.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state worker remove --nickname w1 --json'],
    deprecated: null,
  },
  {
    name: 'state.worker.clear',
    invoke: 'bee state worker clear',
    description: 'Empty the whole state.workers array.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state worker clear --json'],
    deprecated: null,
  },
  {
    name: 'state.worker.prune',
    invoke: 'bee state worker prune',
    description: 'Delete stale dispatch transients from .bee/workers/ (keeps active-worker and non-capped-cell files). Reads state via readStateStrict and never writes state.json.',
    parameters: {
      type: 'object',
      properties: {
        'dry-run': { type: 'boolean', description: 'Report the candidate set without deleting anything.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state worker prune --json'],
    deprecated: null,
  },
  {
    name: 'state.scribing-run',
    invoke: 'bee state scribing-run',
    description: 'Stamp last_scribing_run (date + ISO-precise at), mirror --next-action to the top-level next_action, and advance phase to compounding. Target resolution (symmetric with the read path): explicit --lane <feature> always wins > the calling session\'s bound lane when --lane is omitted > the default state.json for an unbound session. --no-lane forces the default record. A missing or corrupt lane — explicit or session-bound — refuses loudly with zero writes. --show is a READ-ONLY query mode: it never appends to the scribing ledger and never advances phase, returning the most-recent recorded stamp overall, or for --feature <slug> if given; it needs neither --areas nor --next-action.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug the scribing run covers (write mode), or the feature to filter by (read-only --show mode).' },
        areas: { type: 'string', description: 'Comma-separated areas synced.' },
        'next-action': { type: 'string', description: 'Next action after scribing.' },
        lane: { type: 'string', description: 'Route the mutation to this lane record instead of the default state.json. Refuses if the lane is missing or corrupt. Omitted: the calling session\'s bound lane is targeted automatically; unbound sessions target the default record.' },
        'no-lane': { type: 'boolean', description: 'Force the default state.json even when the calling session is bound to a lane. Cannot be combined with --lane.' },
        show: { type: 'boolean', description: 'Read-only query mode: return the most-recent scribing stamp (overall, or for --feature <slug>) without appending to the ledger or advancing phase.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state scribing-run --feature newf --areas auth --next-action bee-capturing --json',
      'bee state scribing-run --lane demo-lane --feature demo-lane --areas auth --next-action bee-capturing --json',
      'bee state scribing-run --show --json',
      'bee state scribing-run --show --feature newf --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.compounding-run',
    invoke: 'bee state compounding-run',
    description: 'Stamp last_compounding_run (feature, date, ISO-precise at, learnings path, optional next-action) on the selected record. Legal ONLY from phase "compounding" (checkCompoundingRunPhase) — refused from any other phase. Unlike scribing-run, this verb does NOT advance phase: bee-capturing runs entirely inside phase "compounding", produced earlier by `state scribing-run`. What it DOES do is satisfy a precondition `state set --phase compounding-complete` now enforces: that transition refuses unless last_compounding_run exists, names the SAME feature as last_scribing_run, and was stamped at or after it. Target resolution (symmetric with scribing-run): explicit --lane <feature> always wins > the calling session\'s bound lane when --lane is omitted > the default state.json for an unbound session. --no-lane forces the default record. A missing or corrupt lane — explicit or session-bound — refuses loudly with zero writes.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug the compounding run covers.' },
        learnings: { type: 'string', description: 'Path to the durable learnings this run captured.' },
        'next-action': { type: 'string', description: 'Optional next action after compounding; mirrored to the top-level next_action when given.' },
        lane: { type: 'string', description: 'Route the mutation to this lane record instead of the default state.json. Refuses if the lane is missing or corrupt. Omitted: the calling session\'s bound lane is targeted automatically; unbound sessions target the default record.' },
        'no-lane': { type: 'boolean', description: 'Force the default state.json even when the calling session is bound to a lane. Cannot be combined with --lane.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state compounding-run --feature newf --learnings docs/history/newf/reports/learnings.md --json',
      'bee state compounding-run --lane demo-lane --feature demo-lane --learnings docs/history/demo-lane/reports/learnings.md --next-action bee-hive --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.route',
    invoke: 'bee state route',
    surface: 'porcelain',
    description:
      'A validated route record — {class, lane, flags[], product_files, rationale, updated_at} — persisted on the ACTIVE feature\'s tracked record (session-bound lane when the calling session is bound, else the default record; refuses when no active feature). This verb never accepts a --lane TARGETING flag (unlike state set/gate/scribing-run) — --lane here is the route\'s OWN lane-classification value, not a feature router; targeting a non-active feature is not yet supported. --set requires --class (enum: feature/bugfix/docs/refactor/research/release/spike), --lane (enum: docs/tiny/small/spike/standard/high-risk), --flags (comma-separated; every entry must be one of the canonical triage flag names auth/authorization/data-model/audit-security/external-systems/public-contracts/cross-platform/covered-contract-change/proof-weakening/multi-domain; the empty string means zero flags, not a missing flag), --files (a non-negative integer); optional --rationale. Any enum/shape violation is a typed refusal naming the bad value and the legal set — free prose is never accepted, and nothing is written on refusal. --show is read-only: prints the currently recorded route (null when absent) without requiring any --set flag. `cells claim` warns on stderr, never refuses, when the claimed cell\'s feature has no route yet.',
    parameters: {
      type: 'object',
      properties: {
        set: { type: 'boolean', description: 'Write mode: record a new route (requires --class/--lane/--flags/--files).' },
        show: { type: 'boolean', description: 'Read-only mode: print the currently recorded route (null when absent).' },
        class: { type: 'string', description: 'Route class.' },
        lane: { type: 'string', description: 'Route lane (the triage lane classification — never a feature-routing flag).' },
        flags: { type: 'string', description: 'Comma-separated triage flag names; the empty string means zero flags.' },
        files: { type: 'string', description: "Non-negative integer: the triage product-file count." },
        rationale: { type: 'string', description: 'Optional free-text rationale for the route.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state route --set --class feature --lane standard --flags multi-domain --files 7 --json',
      'bee state route --show --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.feature-verify.record',
    invoke: 'bee state feature-verify record',
    description:
      'Stamp the feature-level verify record — {feature, command, output_sha256, result, at} — on the ACTIVE feature\'s tracked record (session-bound lane else default, the same resolution as state route; no --lane targeting flag) AND its underlying workflow-store record. output_sha256 is computed by the verb from --output-file, never caller-supplied. --result is a closed green|red enum: red is storable (it documents the failure for the fix-cells-then-re-verify loop) but NEVER satisfies the close door — leaving phase "swarming" (state set out, or state scribing-run) stays refused (guardFeatureVerifyDebt — no gate_bypass level lifts it) while any cell capped via `cells cap --feature-verify-pending` lacks a green record stamped strictly newer than the newest pending cap. Refuses when no feature is active. --show is the read-only query mode (also available as `bee state feature-verify show`): prints the currently recorded feature_verify (null when absent) without writing.',
    parameters: {
      type: 'object',
      properties: {
        command: { type: 'string', description: 'The feature-verify command that was run (the impacted suite over the feature\'s whole diff).' },
        'output-file': { type: 'string', description: 'Path to the captured verify output; the verb computes and stores its sha256.' },
        result: { type: 'string', description: 'Verify outcome: green or red. Red is storable but never satisfies the close door.', enum: ['green', 'red'] },
        show: { type: 'boolean', description: 'Read-only query mode: print the currently recorded feature_verify (null when absent) without writing; needs none of the write flags.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state feature-verify record --command "node packages/bee/tests/run_verify.mjs --impacted" --output-file feature-verify-output.txt --result green --json',
      'bee state feature-verify record --show --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.feature-verify.show',
    invoke: 'bee state feature-verify show',
    description:
      'Read-only: print the ACTIVE feature\'s recorded feature-verify record — {feature, command, output_sha256, result, at} — or null when none is recorded. Same record `state feature-verify record --show` prints; never writes.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee state feature-verify show --json'],
    deprecated: null,
  },
  {
    name: 'state.workflows.list',
    invoke: 'bee state workflows list',
    description:
      "Read-only listing of every workflow record under .bee/runtime/workflows/ — {id, feature, status, phase, created_at, ...the full record} per entry, sorted newest-created first. Never writes. `status` is one of active/paused/closed (workflow-store.mjs STATUS_VALUES); MULTIPLE records can be status \"active\" at once (one per concurrently-running lane) — this verb is a plain inventory, not itself the pruning tool (see `state workflows close`).",
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON (the full record array) instead of a one-line-per-record summary.' },
      },
      required: [],
    },
    examples: ['bee state workflows list --json'],
    deprecated: null,
  },
  {
    name: 'state.workflows.close',
    invoke: 'bee state workflows close',
    description:
      'Closes zombie/stale workflow records without hand-editing .bee/runtime/workflows/*/state.json. Requires EXACTLY ONE of three mutually exclusive modes: --feature <f> closes every live (non-closed) record whose feature equals <f>; --id <id> closes the single record with that id; --all-but-active closes every live record whose feature differs from the CALLING context\'s own active feature (the same session-bound-lane-else-default resolution `state route`/`state feature-verify` use), keeping that one. The currently active feature\'s record is NEVER closed by --feature or --all-but-active — only --id may name it explicitly, bypassing the protection on purpose. Refuses with a typed, zero-mutation error when a mode selects zero live records (already closed, unknown id, unknown feature, or nothing left besides the active feature). Prints the closed {id, feature} pairs on success.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Close every live workflow record for this feature. Refused if this is the currently active feature (use --id instead).' },
        id: { type: 'string', description: 'Close the single live workflow record with this id, even if it is the currently active feature\'s record.' },
        'all-but-active': { type: 'boolean', description: 'Close every live workflow record except the currently active feature\'s.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state workflows close --feature stale-feature --json',
      'bee state workflows close --id wf-abc123 --json',
      'bee state workflows close --all-but-active --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.start-feature',
    invoke: 'bee state start-feature',
    description: 'Guarded atomic feature start: fails closed with zero mutations unless the workspace is clean (idle/terminal phase, no handoff/active workers/reservations/claimed or nonterminal prior cells); on success sets feature/mode/phase and resets all four gates. "Active workers" is a derived view — live-heartbeat sessions joined with their current cell claim, never a hand-maintained list — so --session-id names the calling session on EITHER path (default or --as-lane) so its own heartbeat never counts against itself; without --session-id every live session, including the caller\'s own if it has one, counts. Optional --as-lane starts the feature as a per-feature lane record (.bee/lanes/<feature>.json) beside the default pipeline instead of mutating state.json; --paths is a comma-separated list of intended file paths checked against other sessions\' active claims/reservations before the lane starts (--as-lane only).',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'New feature slug.' },
        mode: { type: 'string', description: 'Mode for the new feature.' },
        phase: { type: 'string', description: 'Entry phase (defaults to exploring).', enum: [...KNOWN_PHASES] },
        'as-lane': { type: 'boolean', description: 'Start this feature as a per-feature lane record instead of the default state.json.' },
        'session-id': { type: 'string', description: 'Calling session id: excludes its own live heartbeat from the derived active-workers precondition on either path, and — only meaningful with --as-lane and --paths — excludes its own active holds from the declared-paths conflict check.' },
        paths: { type: 'string', description: 'Comma-separated declared file paths checked against other sessions\' active claims/reservations before the lane starts (only meaningful with --as-lane).' },
        isolate: { type: 'boolean', description: 'Route this feature start through the write-policy isolate-create path (a fresh worktree) instead of starting directly in this checkout. Only meaningful when the write-policy mode actually enforces isolation.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: [
      'bee state start-feature --feature newf --json',
      'bee state start-feature --feature demo-lane --as-lane --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.lanes',
    invoke: 'bee state lanes',
    description: 'List every per-feature lane record with its phase, gates, and which sessions are currently bound to it.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-lane summary.' },
      },
      required: [],
    },
    examples: ['bee state lanes --json'],
    deprecated: null,
  },
  {
    name: 'state.rebuild-projections',
    invoke: 'bee state rebuild-projections',
    description: 'Recovery verb: rebuilds .bee/lanes/<feature>.json for every active workflow, entirely from .bee/runtime/workflows/ records — every lane mutation in this cell keeps its workflow record in sync, so this is always safe and lossless for lanes. Also rebuilds .bee/state.json\'s workflow-owned fields (phase/mode/feature/approved_gates/summary/next_action) from the newest ACTIVE workflow record, but ONLY while state.json is itself idle (no live default feature) — a live default record\'s own workflow record is not yet kept in sync by default-path writes, so it is never overwritten. A no-op (authoritative:false in the JSON result) when zero workflow records exist anywhere yet, state.json has a live non-idle default feature, or a lane names no live workflow record — the legacy file is left exactly as it already was in every no-op case. Also unconditionally rebuilds .bee/reservations.json from lease-store.mjs\'s current active path leases — never gated on workflow records, always overwrites. Safe to run any time.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee state rebuild-projections --json'],
    deprecated: null,
  },
  {
    name: 'state.session.list',
    invoke: 'bee state session list',
    description: 'List every session record (id, started_at, last_heartbeat, bound lane if any) — the cross-session identities lane claims key off.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-session summary.' },
      },
      required: [],
    },
    examples: ['bee state session list --json'],
    deprecated: null,
  },
  {
    name: 'state.session.bind',
    invoke: 'bee state session bind',
    description: "Bind a session to a lane (session→lane binding) so pipeline reads/writes for that session resolve to the named lane instead of the default state.json (resolvePipeline). Does not verify the lane record exists — a binding to a missing/invalid lane is a typed refusal at resolution time, not at bind time.",
    parameters: {
      type: 'object',
      properties: {
        'session-id': { type: 'string', description: 'Session id to bind.' },
        lane: { type: 'string', description: 'Lane feature name to bind the session to.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state session bind --session-id sess-demo --lane demo-lane --json'],
    deprecated: null,
  },
  {
    name: 'state.session.unbind',
    invoke: 'bee state session unbind',
    description: "Remove a session's lane binding (omits the key entirely, restoring the unbound shape) so the session resolves back to the default pipeline.",
    parameters: {
      type: 'object',
      properties: {
        'session-id': { type: 'string', description: 'Session id to unbind.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee state session unbind --session-id sess-demo --json'],
    deprecated: null,
  },
  {
    name: 'state.handoff.write',
    invoke: 'bee state handoff write',
    description: "Write a handoff through the guarded writer. --kind is required and never guessed: 'pause' writes today's free-form fields (--cell/--files/--done/--remaining/--next-action/--feature/--phase/--mode, whichever apply) plus the kind — no new precondition, the same surface-and-wait record as always. 'planned-next' REFUSES (typed, zero mutation) unless --previous-cell is capped with a passing verify AND --next-cell already has a claim owned by --writer-session (the carried claim) — on success the record stores writer_session/previous_cell/next_cell alongside kind. When a workflow resolves (--lane names it, or the calling session/default record is bound to one), the record is written to that workflow's OWN mailbox (.bee/runtime/handoffs/<workflow-id>/<seq>.json, scoped by --target-role) instead of the single legacy .bee/HANDOFF.json — a repo with no workflow records keeps writing the legacy file, byte-identical to before.",
    parameters: {
      type: 'object',
      properties: {
        kind: { type: 'string', description: 'Handoff kind — required, never guessed.', enum: [...HANDOFF_KINDS] },
        'writer-session': { type: 'string', description: 'planned-next only: the writing session id, which must already own the claim on --next-cell.' },
        'previous-cell': { type: 'string', description: 'planned-next only: the just-capped cell id (must be capped with trace.verify_passed true).' },
        'next-cell': { type: 'string', description: "planned-next only: the next cell id, whose claim must already be owned by --writer-session." },
        cell: { type: 'string', description: 'pause only: the cell mid-flight when the pause was written.' },
        files: { type: 'string', description: 'pause only: comma-separated files touched so far.' },
        done: { type: 'string', description: 'pause only: comma-separated completed steps.' },
        remaining: { type: 'string', description: 'pause only: comma-separated remaining steps.' },
        feature: { type: 'string', description: 'Feature slug to record on the handoff.' },
        phase: { type: 'string', description: 'Phase to record on the handoff.' },
        mode: { type: 'string', description: 'Mode to record on the handoff.' },
        'next-action': { type: 'string', description: 'Saved next-action text.' },
        lane: { type: 'string', description: 'Name the target workflow by its lane feature explicitly, instead of resolving it from the calling session or the default record.' },
        'target-role': { type: 'string', description: 'Scope this mailbox record to a role (e.g. "reviewer") so it never clobbers another role\'s open handoff for the same workflow. Omitted = the default/unscoped slot.' },
        'session-id': { type: 'string', description: 'The writing session id used ONLY to resolve which workflow this call targets when --lane is omitted (never required — BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID resolve it the usual way otherwise).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['kind'],
    },
    examples: [
      'bee state handoff write --kind pause --cell wip-1 --next-action "resume wip-1" --json',
      'bee state handoff write --kind planned-next --writer-session sess-handoff-writer --previous-cell handoff-prev --next-cell handoff-next --json',
    ],
    deprecated: null,
  },
  {
    name: 'state.handoff.adopt',
    invoke: 'bee state handoff adopt',
    description: "Adopt a planned-next handoff's carried claim into --session-id: transfers ownership of the handoff's next_cell claim to the adopting session, then clears the handoff — clear-after-adopt with idempotent recovery, not a single cross-file transaction (a crash between the two steps self-heals on the next call via a benign self-adopt). Refuses (typed, non-zero exit) when there is no handoff, the handoff is not kind planned-next (pause handoffs are never adopted — surface and wait instead), or the underlying claim adopt fails — every refusal leaves both the claim and the handoff untouched. When a workflow resolves (--lane, or --session-id's own bound lane/default record), this adopts from that workflow's mailbox instead of the single legacy .bee/HANDOFF.json.",
    parameters: {
      type: 'object',
      properties: {
        'session-id': { type: 'string', description: 'Adopting session id — also used to resolve which workflow to adopt from when --lane is omitted.' },
        lane: { type: 'string', description: 'Name the target workflow by its lane feature explicitly.' },
        'target-role': { type: 'string', description: 'Adopt from this role\'s mailbox slot instead of the default/unscoped one.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['session-id'],
    },
    examples: ['bee state handoff adopt --session-id sess-handoff-adopter --json'],
    deprecated: null,
  },
  {
    name: 'state.handoff.show',
    invoke: 'bee state handoff show',
    description: 'Show the current handoff, if any, with kind normalized for display (a missing/unknown kind reads as "pause" — fail-safe). Reports "no handoff" when none exists. When a workflow resolves (--lane, or the calling --session-id/default record\'s own bound lane), shows that workflow\'s own mailbox instead of the single legacy .bee/HANDOFF.json.',
    parameters: {
      type: 'object',
      properties: {
        lane: { type: 'string', description: 'Name the target workflow by its lane feature explicitly.' },
        'target-role': { type: 'string', description: 'Show this role\'s mailbox slot instead of the default/unscoped one.' },
        'session-id': { type: 'string', description: 'Resolve which workflow to show via this session\'s bound lane, when --lane is omitted.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee state handoff show --json'],
    deprecated: null,
  },
  {
    name: 'state.advisor-ref.record',
    invoke: 'bee state advisor-ref record',
    description: "Record an advisor consult onto the selected record's advisor_ref — the Gate 3 high-risk precondition needs a state field AND a verb. The verb stamps the staleness anchors ITSELF — current feature, newest active decision id, and sha256 of that feature's plan.md — so anchors are never caller-supplied; the caller passes only --advisor (identity) and --digest-file (its first 500 chars are stored as digest_head for audit). Refuses when no feature is active (phase idle/compounding-complete or no feature). Target resolution (symmetric with the read path): explicit --lane <feature> always wins > the calling session's bound lane when --lane is omitted > the default state.json for an unbound session; --no-lane forces the default record. A lane target — explicit or session-bound — records with anchors bound to the lane's own feature and plan.md, leaving the default state.json untouched; a missing or corrupt lane refuses loudly with zero writes. There is no clear verb — staleness makes an old ref inert.",
    parameters: {
      type: 'object',
      properties: {
        advisor: { type: 'string', description: 'Advisor identity that was consulted (e.g. the configured advisor model or cli command label).' },
        'digest-file': { type: 'string', description: 'Path to the captured advisor consult digest; its first 500 chars are stored as digest_head for audit.' },
        lane: { type: 'string', description: 'Route the record to this lane instead of the default state.json. Refuses if the lane is missing or corrupt. Omitted: the calling session\'s bound lane is targeted automatically; unbound sessions target the default record.' },
        'no-lane': { type: 'boolean', description: 'Force the default state.json even when the calling session is bound to a lane. Cannot be combined with --lane.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['advisor', 'digest-file'],
    },
    examples: ['bee state advisor-ref record --advisor gpt-5.6-sol --digest-file consult.txt --json'],
    deprecated: null,
  },
  {
    name: 'state.advisor-ref.show',
    invoke: 'bee state advisor-ref show',
    description: 'Show the selected record\'s advisor_ref, if any, with its live staleness verdict (stale + reason list) computed against the current feature, newest active decision id, plan.md sha256, and the last execution-gate revocation. A missing or malformed advisor_ref reads as "no advisor_ref recorded", never a crash. Optional --lane <feature> shows the lane record instead of the default state.json.',
    parameters: {
      type: 'object',
      properties: {
        lane: { type: 'string', description: 'Show this lane record instead of the default state.json.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee state advisor-ref show --json'],
    deprecated: null,
  },

  // ─── state compact-*: compaction-hardening's helper floor (D3) — three thin
  // CLI wrappers over lib/compaction.mjs, the ONE module every compaction
  // surface (hooks and verbs alike) calls. All three of D3's locked verbs are
  // registered here now that cz-5 has landed buildCompactCapsule; the earlier
  // note that `state compact-capsule` was deliberately withheld described a
  // builder that did not yet exist, and the coverage check below (every
  // registry entry must have an executed example) is unconditional over the
  // whole registry, so each one carries its own example.
  {
    name: 'state.compact-log',
    invoke: 'bee state compact-log',
    description: "Append one compaction telemetry record via lib/compaction.mjs's appendCompactionRecord — the durable write every compaction surface (the PreCompact hook, the SessionStart resume hook) and this verb both call, so the log stays reachable by command on any runtime whose hook execution is unconfirmed. --event is 'precompact' or 'resume'; compact_index/cell_compact_count follow the counting rule — only precompact records are counted, and only a precompact record counts itself. A write failure never changes the exit code (fail-open, logged locally).",
    parameters: {
      type: 'object',
      properties: {
        event: { type: 'string', description: 'Compaction event being recorded.', enum: [...COMPACT_EVENTS] },
        'session-id': { type: 'string', description: 'Session id the record is scoped to.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['event', 'session-id'],
    },
    examples: ['bee state compact-log --event precompact --session-id sess-demo --json'],
    deprecated: null,
  },
  {
    name: 'state.compact-check',
    invoke: 'bee state compact-check',
    description: "Read-only integrity sweep for a session (lib/compaction.mjs's compactCheck): session record, lane binding, claimed-cell ownership, execution gate, dependency capping, reservation holds, and intent-anchor presence. REPORTS ONLY — it never repairs, releases, or blocks, and it exits non-zero ONLY on a usage error, never on a detected mismatch (a mismatch is reported data). The JSON output's checks[] always carries an `anchor_missing` entry naming the exact command the anchor nudge would name (ANCHOR_NUDGE_COMMAND), so the nudge stays reachable by command on a runtime that never executes a hook.",
    parameters: {
      type: 'object',
      properties: {
        'session-id': { type: 'string', description: 'Session id to sweep.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: ['session-id'],
    },
    examples: ['bee state compact-check --session-id sess-demo --json'],
    deprecated: null,
  },
  {
    name: 'state.compact-capsule',
    invoke: 'bee state compact-capsule',
    description: "Render the compact capsule for a session (lib/compaction.mjs's buildCompactCapsule) — the narrow orientation body a SessionStart with source=compact emits INSTEAD of the full startup preamble: the state-mismatch block, the onboarding-MISSING line, the HANDOFF block with its adoption-refusal reason, the gate-bypass banner, phase/mode/feature/lane, the claimed cell with its verify and dependency status, the first open gate, next_action, the recorded commands, the compaction survival count with its advisory, and a POINTER to the critical patterns, in that order. It NEVER renders the intent anchor — the hook owns that — and no byte of it varies with anchor presence. Read-only, and reachable by command on any runtime whose hook execution is unconfirmed.",
    parameters: {
      type: 'object',
      properties: {
        'session-id': { type: 'string', description: 'Session id the capsule is rendered for.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of the rendered capsule text.' },
      },
      required: ['session-id'],
    },
    examples: ['bee state compact-capsule --session-id sess-demo'],
    deprecated: null,
  },

  // ─── backlog (bee_backlog.mjs — docs/backlog.md mechanical passes + the
  // .bee/backlog.jsonl `add` verb). `required: []` on `backlog.add` is
  // deliberate (DB3, same discipline as the state.* entries above): the
  // generic validate() layer would emit its structured error on STDOUT, but
  // the legacy bee_backlog.mjs `add` contract (pinned by test_lib.mjs) emits
  // its validation refusals on STDERR. So the handler owns every required-
  // flag / enum / length check itself, throwing the legacy message text —
  // which the dispatcher routes to STDERR. ─────────────────────────────────
  {
    name: 'backlog.counts',
    invoke: 'bee backlog counts',
    description: 'Render PBI backlog counts (done/in-flight/proposed/total) parsed from docs/backlog.md.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee backlog counts --json'],
    deprecated: null,
  },
  {
    name: 'backlog.rank',
    invoke: 'bee backlog rank',
    description:
      'P2 mechanical pass (dry-run reporting only): reports docs/backlog.md rows reordered by status group (in-flight, proposed, done). RETIRED: --write — "bee backlog render --write" now owns the generated view; passing --write refuses, naming render.',
    parameters: {
      type: 'object',
      properties: {
        write: { type: 'boolean', description: 'RETIRED — refuses, naming "bee backlog render --write" instead.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee backlog rank --json'],
    deprecated: null,
  },
  {
    name: 'backlog.badges',
    invoke: 'bee backlog badges',
    description: "P3 mechanical pass: refresh README.md's backlog badges from docs/backlog.md counts. --write persists, otherwise nothing is changed.",
    parameters: {
      type: 'object',
      properties: {
        write: { type: 'boolean', description: 'Persist the refreshed badges to README.md.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee backlog badges --json'],
    deprecated: null,
  },
  {
    name: 'backlog.add',
    invoke: 'bee backlog add',
    surface: 'porcelain',
    description:
      'Validate then append one row to .bee/backlog.jsonl (the feedback-digest source lib/feedback.mjs\'s collectFeedback reads) — agents never hand-edit .bee state. --type must be a KIND_ALIASES key or an already-normalized NORMALIZED_KINDS value (lib/feedback.mjs), --severity is P1|P2|P3, --layer is a free non-empty string <=40 chars (no allowlist), --title is required and <=200 chars. Any rejection leaves the file untouched. The row is always appended regardless of --queue-submit; only the auto-commit is gated: --queue-submit defaults false and must be passed explicitly for a human queue-submission (the caller is always the agent process, but the flag marks intent — a new item for the processing queue vs. the agent logging its own friction/debt/finding about its own session). With --queue-submit, a merge in progress is detected up front and the commit is skipped with commit_skipped_reason:"merge_in_progress" plus a visible warning suffix in the text output; every other commit failure stays a silent committed:false as before.',
    parameters: {
      type: 'object',
      properties: {
        type: { type: 'string', description: 'Backlog row type (a KIND_ALIASES key or an already-normalized NORMALIZED_KINDS value).' },
        title: { type: 'string', description: 'Row title, <=200 chars.' },
        severity: { type: 'string', description: 'Row severity.', enum: ['P1', 'P2', 'P3'] },
        layer: { type: 'string', description: 'Free non-empty layer string, <=40 chars.' },
        detail: { type: 'string', description: 'Optional detail text.' },
        feature: { type: 'string', description: 'Optional feature slug.' },
        'queue-submit': {
          type: 'boolean',
          description:
            'Human queue-submission for this row (default false). Only when true does the scoped auto-commit run at all — an agent logging its own friction/debt/finding about its own session leaves this unset so appendJsonl still writes the row but no commit is attempted.',
        },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee backlog add --type friction --title "example backlog row" --severity P2 --layer state --queue-submit --json'],
    deprecated: null,
  },
  {
    name: 'backlog.propose',
    invoke: 'bee backlog propose',
    description:
      'Submit a new product-backlog item (PBI) on demand — the human-facing front door onto "backlog pbi add", taking a story plus acceptance criteria with no id and no separate title. It appends one kind:\'pbi\' add event to the .bee/backlog.jsonl PBI fold with an auto-generated `p-<8hex>` id and Status=proposed; docs/backlog.md is the GENERATED view of that fold, so run "bee backlog render --write" to refresh the table. The command stops at the proposal — it never auto-starts bee-shaping for the new item. --story is required, <=200 chars (stored as the PBI title); --cos (acceptance criteria) is required, <=2000 chars; --feature is optional and reports as "—" when omitted. Any validation rejection leaves the log untouched.',
    parameters: {
      type: 'object',
      properties: {
        story: { type: 'string', description: 'PBI story text, required, <=200 chars — stored as the PBI title.' },
        cos: { type: 'string', description: 'Acceptance criteria (conditions of satisfaction), required, <=2000 chars.' },
        feature: { type: 'string', description: 'Optional feature slug; reports as "—" when omitted.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['story', 'cos'],
    },
    examples: [
      'bee backlog propose --story "A human can submit a backlog item on demand" --cos "bee backlog propose appends a proposed PBI with an auto-assigned id" --feature backlog-submit-command --json',
    ],
    deprecated: null,
  },

  // ─── backlog pbi / backlog render (backlog-unification D1-D5): the
  // event-sourced product-backlog layer. `backlog add` above is a DISTINCT
  // verb family (friction/proposal rows for the feedback digest); these PBI
  // verbs append kind:'pbi' records to the SAME .bee/backlog.jsonl stream,
  // which lib/backlog.mjs's foldPbis derives current state from
  // (last-event-wins) — never a hand-edit, never a second store. `render`
  // is the generated docs/backlog.md view that replaces the retired
  // `backlog rank --write`. ────────────────────────────────────────────────
  {
    name: 'backlog.pbi.add',
    invoke: 'bee backlog pbi add',
    description:
      'Append a PBI "add" event to .bee/backlog.jsonl and print the created id. --id generates a collision-free `p-<8hex>` id via crypto randomness (no lock, no read-then-increment); pass --id yourself ONLY to preserve a legacy id during migration — a given or generated id that already names an item refuses (duplicate add). --status defaults to "proposed".',
    parameters: {
      type: 'object',
      properties: {
        title: { type: 'string', description: 'PBI title (Story). Required, non-empty.' },
        cos: { type: 'string', description: 'Optional Condition of Satisfaction text.' },
        status: { type: 'string', description: 'Initial status (default "proposed").', enum: ['proposed', 'in-flight', 'parked', 'done', 'declined'] },
        feature: { type: 'string', description: 'Optional feature slug.' },
        id: { type: 'string', description: 'Migration-only override: preserve an existing id (e.g. a legacy P<n>) instead of generating one.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['title'],
    },
    examples: ['bee backlog pbi add --title "Unify the backlog" --cos "one store, no hand-edits" --json'],
    deprecated: null,
  },
  {
    name: 'backlog.pbi.status',
    invoke: 'bee backlog pbi status',
    description:
      'Append a PBI "status" event, flipping --id to --to. --feature optionally stamps the feature slug in the same move. Refuses an unknown --id or an out-of-enum --to.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'The PBI id to flip.' },
        to: { type: 'string', description: 'The new status.', enum: ['proposed', 'in-flight', 'parked', 'done', 'declined'] },
        feature: { type: 'string', description: 'Optional feature slug to stamp in the same event.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id', 'to'],
    },
    examples: ['bee backlog pbi status --id p-a1b2c3d4 --to in-flight --feature backlog-unification --json'],
    deprecated: null,
  },
  {
    name: 'backlog.pbi.amend',
    invoke: 'bee backlog pbi amend',
    description: 'Append a PBI "amend" event updating --title and/or --cos (at least one required). Status/feature never move through amend — that is "pbi status"\'s job. Refuses an unknown --id.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'The PBI id to amend.' },
        title: { type: 'string', description: 'New title text.' },
        cos: { type: 'string', description: 'New Condition of Satisfaction text.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['id'],
    },
    examples: ['bee backlog pbi amend --id p-a1b2c3d4 --cos "revised CoS text" --json'],
    deprecated: null,
  },
  {
    name: 'backlog.pbi.list',
    invoke: 'bee backlog pbi list',
    description: 'List current PBIs from the fold over .bee/backlog.jsonl (the token-cheap query — no docs/backlog.md read). --status filters to one enum value; id-sorted.',
    parameters: {
      type: 'object',
      properties: {
        status: { type: 'string', description: 'Filter to one status.', enum: ['proposed', 'in-flight', 'parked', 'done', 'declined'] },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-PBI summary.' },
      },
      required: [],
    },
    examples: ['bee backlog pbi list --status in-flight --json'],
    deprecated: null,
  },
  {
    name: 'backlog.render',
    invoke: 'bee backlog render',
    description:
      'Render the generated docs/backlog.md view from the .bee/backlog.jsonl PBI fold (proposed/in-flight/parked as full rows; done/declined collapse to one-line links, so the view stays short forever). Deterministic — no generation timestamp. --write persists; --check compares against what is on disk and refuses (non-zero exit) on drift; with neither, reports what would change.',
    parameters: {
      type: 'object',
      properties: {
        write: { type: 'boolean', description: 'Persist the generated content to docs/backlog.md.' },
        check: { type: 'boolean', description: 'Refuse (non-zero exit) if the generated content differs from what is on disk.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee backlog render --check', 'bee backlog render --write'],
    deprecated: null,
  },
  {
    name: 'backlog.findings',
    invoke: 'bee backlog findings',
    description:
      'List friction/finding rows from .bee/backlog.jsonl for one --feature (required). Two row schemas coexist on that stream and BOTH are read: legacy rows carry `kind: "friction"|"finding"`, current rows (from "backlog add") carry `type: "friction"|"finding"` — a row is returned if either field matches. --feature is a word-boundary/exact match, NOT substring ("auth" does not match "authz"); kind:\'pbi\' rows never surface. Optional --text narrows further by substring over title+detail (any whitespace-split term hitting either field is enough).',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug to match (word-boundary/exact over the row\'s feature field, not substring).' },
        text: { type: 'string', description: 'Optional substring filter over title+detail (whitespace-split terms, any-term match).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-row summary.' },
      },
      required: ['feature'],
    },
    examples: ['bee backlog findings --feature state-query-surface --json', 'bee backlog findings --feature state-query-surface --text grep --json'],
    deprecated: null,
  },

  // ─── capture (bee_capture.mjs — the capture-queue CLI, decision 0017) ─────
  {
    name: 'capture.add',
    invoke: 'bee capture add',
    surface: 'porcelain',
    description: 'Append a capture-queue stub for a same-turn settlement; the full BA-grade spec merge happens later at flush. High-risk lane never queues.',
    parameters: {
      type: 'object',
      properties: {
        outcome: { type: 'string', description: 'Outcome text for the stub.' },
        did: { type: 'string', description: 'Comma-separated decision ids the settlement relates to.' },
        area: { type: 'string', description: 'Spec area the stub belongs to.' },
        files: { type: 'string', description: 'Comma-separated list of files touched.' },
        lane: { type: 'string', description: 'Lane the settlement ran at (high-risk never queues).' },
        source: { type: 'string', description: 'Optional provenance tag (e.g. "mined" for a transcript-recovery candidate settlement); a mined stub sitting unflushed in the pending queue is the mined-unconfirmed state, and the normal flush is its confirmation.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee capture add --outcome "example capture stub outcome" --json'],
    deprecated: null,
  },
  {
    name: 'capture.list',
    invoke: 'bee capture list',
    description: 'List pending (not yet flushed) capture stubs, oldest first.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a formatted list.' },
      },
      required: [],
    },
    examples: ['bee capture list --json'],
    deprecated: null,
  },
  {
    name: 'capture.flush',
    invoke: 'bee capture flush',
    description: 'Mark a pending capture stub flushed (its content merged into a spec by bee-capturing). Refuses when the id names no pending stub.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Stub id to flush.' },
        into: { type: 'string', description: 'Where the stub content landed, e.g. docs/specs/<area>.md.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee capture flush --id 00000000-0000-0000-0000-000000000000 --json'],
    deprecated: null,
  },
  {
    name: 'capture.count',
    invoke: 'bee capture count',
    description: 'Report the pending capture-stub count.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee capture count --json'],
    deprecated: null,
  },

  // ─── intent (lib/intent.mjs — the intent anchor, feature intent-anchor
  // D1/D2). The ONE durable record of what the user asked for: `request` is
  // stored verbatim and, with `acceptance`, is immutable once set — only
  // `next_action` advances. Keyed by the active feature when one exists, else
  // the session id, else a shared default, so work that never enters a
  // feature (a direct question, a tiny fix) is still anchorable (D2). ───────
  {
    name: 'intent.set',
    invoke: 'bee intent set',
    description:
      "Write the intent anchor: the user's VERBATIM request plus what \"done\" means. Stored at .bee/intent/<key>.json, keyed by the active feature when one exists, else --session, else a shared default (work with no feature must still be anchorable). request/acceptance are immutable once set: re-setting the same request is idempotent, changing it refuses unless --force. Never paraphrase --request — verbatim IS the mechanism.",
    parameters: {
      type: 'object',
      properties: {
        request: { type: 'string', description: "The user's request, VERBATIM — their own words, never a paraphrase or a summary." },
        acceptance: { type: 'string', description: 'What "done" means — the drift detector the anchor is checked against.' },
        'next-action': { type: 'string', description: 'The single next step; the only field `intent advance` may move.' },
        feature: { type: 'string', description: 'Override the anchor key/feature instead of resolving the active feature from state.' },
        lane: { type: 'string', description: 'Lane the work is running at (context only).' },
        cell: { type: 'string', description: 'Cell id currently in flight (context only).' },
        session: { type: 'string', description: 'Session id to key on when there is no active feature.' },
        'do-not-reverse': { type: 'string', description: 'Comma-separated decisions that must not be reversed.' },
        'stop-conditions': { type: 'string', description: 'Comma-separated conditions under which the agent must stop and ask.' },
        force: { type: 'boolean', description: 'Pass "true" to replace a live anchor whose request/acceptance differ — a deliberate new objective, never a drift. Takes an explicit true/false value, same as cells.verify --passed.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['request', 'acceptance'],
    },
    examples: [
      'bee intent set --request "example verbatim request" --acceptance "example acceptance criteria" --next-action "example next step" --json',
    ],
    deprecated: null,
  },
  {
    name: 'intent.show',
    invoke: 'bee intent show',
    description:
      'Show the current intent anchor, or the rendered PreCompact/resume block for it. Emits nothing but a null/empty result when no anchor exists (a repo that never writes one behaves exactly as it did before).',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Look the anchor up under this key instead of resolving the active feature.' },
        session: { type: 'string', description: 'Session id to include in key resolution.' },
        render: { type: 'string', description: 'Render a block instead of the record: "precompact" (the labelled compaction re-assertion) or "resume" (the compact/resume lead block).', enum: ['precompact', 'resume'] },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of the text rendering.' },
      },
      required: [],
    },
    examples: ['bee intent show --json'],
    deprecated: null,
  },
  {
    name: 'intent.advance',
    invoke: 'bee intent advance',
    description:
      'Move the anchor\'s next_action to the next step. Structurally cannot touch request or acceptance — the through-line is what survives, only the step advances. Refuses when no anchor exists.',
    parameters: {
      type: 'object',
      properties: {
        'next-action': { type: 'string', description: 'The new single next step.' },
        feature: { type: 'string', description: 'Look the anchor up under this key instead of resolving the active feature.' },
        session: { type: 'string', description: 'Session id to include in key resolution.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['next-action'],
    },
    examples: ['bee intent advance --next-action "example advanced next step" --json'],
    deprecated: null,
  },
  {
    name: 'intent.clear',
    invoke: 'bee intent clear',
    description: 'Remove the intent anchor — the objective is finished or abandoned. Idempotent: clearing when none exists reports cleared:false and never errors.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Clear the anchor under this key instead of resolving the active feature.' },
        session: { type: 'string', description: 'Session id to include in key resolution.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee intent clear --json'],
    deprecated: null,
  },

  // ─── reviews (bee_reviews.mjs — review-session store + candidates ledger,
  // dispatcher-unify du-3). `reviews.candidate.add` is a NESTED 3-segment
  // name resolved by the dispatcher's longest-prefix match (du-1), sitting
  // alongside the separate FLAT `reviews.candidates` verb (bee_reviews.mjs
  // :186-207/199-207) — two distinct verbs, both pinned. `required: []` on
  // every reviews entry is deliberate (DB3, same discipline as state.*/
  // backlog.*): the generic validate() layer would emit its structured error
  // on STDOUT, but the legacy bee_reviews.mjs contract (pinned by
  // test_lib.mjs) emits its validation refusals on STDERR. So each handler
  // owns its own required-flag / enum checks, throwing the legacy message
  // text — which the dispatcher routes to STDERR. ─────────────────────────
  {
    name: 'reviews.create',
    invoke: 'bee reviews create',
    description:
      'Freeze a review scope into .bee/reviews/<id>.json. Runs the verification-evidence preflight and in-progress auto-exclusion BEFORE any write; fails closed with zero files written on missing evidence or an id that already exists (ids are never reused). Exactly one of --file / --stdin is required at call time (both satisfy the schema; the handler itself enforces the choice, same discipline as cells.add).',
    parameters: {
      type: 'object',
      properties: {
        file: { type: 'string', description: 'Path to a scope JSON file (id, requested_by, scope_description, included, excluded?, baseline, head). Required unless --stdin is set.' },
        stdin: { type: 'boolean', description: 'Read the scope JSON from stdin instead of --file.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee reviews create --file scope.json --json'],
    deprecated: null,
  },
  {
    name: 'reviews.list',
    invoke: 'bee reviews list',
    description: 'List every review session, one line per session (id, decision status, scope description).',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-session summary.' },
      },
      required: [],
    },
    examples: ['bee reviews list --json'],
    deprecated: null,
  },
  {
    name: 'reviews.show',
    invoke: 'bee reviews show',
    description: 'Show one review session by id, full contents.',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Review session id.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of pretty-printed JSON (show always prints JSON; flag kept for surface consistency).' },
      },
      required: [],
    },
    examples: ['bee reviews show --id rev-example --json'],
    deprecated: null,
  },
  {
    name: 'reviews.record',
    invoke: 'bee reviews record',
    description:
      'Set or append a sub-record on an existing session: manifest/preflight/decision SET the field, finding/uat APPEND one entry per call. Refuses any payload touching baseline/head/included/excluded — those are frozen at create. Exactly one of --file / --stdin is required at call time (both satisfy the schema; the handler itself enforces the choice).',
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Review session id.' },
        kind: { type: 'string', description: 'Sub-record kind.', enum: ['manifest', 'preflight', 'finding', 'uat', 'decision'] },
        file: { type: 'string', description: 'Path to the payload JSON file. Required unless --stdin is set.' },
        stdin: { type: 'boolean', description: 'Read the payload JSON from stdin instead of --file.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee reviews record --id rev-example --kind finding --file finding.json --json'],
    deprecated: null,
  },
  {
    name: 'reviews.candidate.add',
    invoke: 'bee reviews candidate add',
    description:
      "Append one entry to .bee/review-candidates.jsonl for a closing feature. --mode is required and must be the closing feature's lane. When --cells is omitted, it auto-fills from the feature's capped cells so review coverage can match the candidate.",
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Closing feature slug.' },
        head: { type: 'string', description: 'Head commit sha.' },
        mode: { type: 'string', description: "The closing feature's lane.", enum: [...REVIEW_MODES] },
        baseline: { type: 'string', description: 'Optional baseline commit sha.' },
        cells: { type: 'string', description: 'Optional comma-separated cell ids covered by this candidate.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee reviews candidate add --feature demo3 --head sha1 --mode standard --json'],
    deprecated: null,
  },
  {
    name: 'reviews.candidates',
    invoke: 'bee reviews candidates',
    description: 'List every review-candidate ledger entry (append-only, one per feature close), oldest first.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-entry summary.' },
      },
      required: [],
    },
    examples: ['bee reviews candidates --json'],
    deprecated: null,
  },
  {
    name: 'reviews.status',
    invoke: 'bee reviews status',
    description:
      'Derived coverage summary (status is never stored): verified count plus the four coverage labels unreviewed/in review/reviewed/review stale, one line per candidate. A candidate reviewed by an unchanged approved session reports "reviewed (covered by <review-id>)".',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Restrict to one feature slug.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a formatted summary.' },
      },
      required: [],
    },
    examples: ['bee reviews status --json'],
    deprecated: null,
  },

  // ─── feedback (bee_feedback.mjs — the dogfood feedback digest CLI, P18,
  // dispatcher-unify du-3). NO collection, redaction, or pain logic lives in
  // the dispatcher — that all lives in lib/feedback.mjs. ───────────────────
  {
    name: 'feedback.digest',
    invoke: 'bee feedback digest',
    description: 'Build the allowlist feedback digest and write it to disk (default .bee/feedback-digest.json).',
    parameters: {
      type: 'object',
      properties: {
        out: { type: 'string', description: 'Output path, relative to repo root (default .bee/feedback-digest.json).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee feedback digest --json'],
    deprecated: null,
  },
  {
    name: 'feedback.count',
    invoke: 'bee feedback count',
    description: 'Report the local feedback digest counts without writing anything to disk.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee feedback count --json'],
    deprecated: null,
  },
  {
    name: 'feedback.collect',
    invoke: 'bee feedback collect',
    description:
      "Merge the local digest with every configured dogfood repo's already-written digest (the consumer revalidates every foreign entry). With no dogfood_repos configured, returns the local digest only.",
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee feedback collect --json'],
    deprecated: null,
  },
  {
    name: 'feedback.rank',
    invoke: 'bee feedback rank',
    description: 'Cluster the merged digest view by normalized title and rank clusters by pain x frequency x corroboration, descending.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee feedback rank --json'],
    deprecated: null,
  },

  // ─── knowledge (okf-foundation S1/S3, lib/knowledge.mjs) — OKF v0.1 bundle
  // verbs over docs/knowledge/ (D17/D23). check is the two-level D4 validator;
  // index (D21) and list (D15) landed in S3; context (D27) in S5. knowledge
  // never writes .bee/*.json(l) stores (D2) — index writes ONLY generated
  // index.md files inside the bundle. ───────────────────────────────────────
  {
    name: 'knowledge.check',
    invoke: 'bee knowledge check',
    description:
      'Validate the docs/knowledge/ OKF v0.1 bundle. The walk never leaves docs/knowledge/; a missing or empty bundle is OK. Two levels — OKF errors: missing/unparseable frontmatter on a non-reserved .md, empty/absent type, frontmatter in a non-root index.md, a root index.md carrying any key but okf_version, a log.md date heading that is not ISO 8601. Profile warnings: type outside the nine profile types, missing profile-required field, dangling required_context/supersedes target, duplicate bee.id, duplicate bee.authoritative_for, and a parse→re-emit byte mismatch (not_canonical — the round-trip guard against silent misparse). Exits non-zero only on OKF errors, or on any finding under --strict.',
    parameters: {
      type: 'object',
      properties: {
        strict: { type: 'boolean', description: 'Promote profile warnings to errors: any finding at all exits non-zero.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON ({okf:{errors},profile:{warnings},counts}) instead of one line per finding.' },
      },
      required: [],
    },
    examples: ['bee knowledge check --json', 'bee knowledge check --strict --json'],
    deprecated: null,
  },
  {
    name: 'knowledge.index',
    invoke: 'bee knowledge index',
    description:
      'Generate the docs/knowledge/ index.md set: one index per directory level whose subtree contains at least one concept, plus the root index.md — the sole carrier of okf_version frontmatter (OKF §9); every generated index opens with an HTML-comment provenance header, and the root additionally carries a "## Critical patterns" section over every bee.critical: true concept. Byte-identical for identical bundle contents: path-sorted entries, LF endings, never a timestamp or any other wall-clock value. With --check, re-renders in memory and byte-compares against disk without writing: any drift (a stale or missing index) exits non-zero naming the file — the same freshness idiom as `bee decisions render --check`.',
    parameters: {
      type: 'object',
      properties: {
        check: { type: 'boolean', description: 'Read-only freshness check: exit non-zero naming each stale index instead of writing.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON ({written,count} or, with --check, {checked,stale,drift}) instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee knowledge index --json', 'bee knowledge index --check --json'],
    deprecated: null,
  },
  {
    name: 'knowledge.list',
    invoke: 'bee knowledge list',
    description:
      'List the bundle\'s concepts: one row per concept — path, bee.id, type, bee.lifecycle, title — path-sorted, never file content. Filters are exact matches: --type on the concept type, --lifecycle on bee.lifecycle, --area on membership in bee.areas. A concept with missing or unparseable frontmatter still rows (null fields) — hiding files is not this verb\'s job; grading them is `knowledge check`\'s.',
    parameters: {
      type: 'object',
      properties: {
        type: { type: 'string', description: 'Keep only concepts whose type equals this value (e.g. bee.pattern).' },
        lifecycle: { type: 'string', description: 'Keep only concepts whose bee.lifecycle equals this value (draft|active|superseded|archived).' },
        area: { type: 'string', description: 'Keep only concepts whose bee.areas array contains this value.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON ({concepts,count}) instead of one row per line.' },
      },
      required: [],
    },
    examples: ['bee knowledge list --json', 'bee knowledge list --type bee.pattern --lifecycle active --json'],
    deprecated: null,
  },
  {
    name: 'knowledge.context',
    invoke: 'bee knowledge context',
    description:
      'Assemble the budget-aware context manifest for a work item. Resolves --work <id> to the bee.work-item concept whose bee.id matches, then ranks: (1) the work item, (2) its bee.plan sibling in the same work/<id>/ directory, (3) bee.required_context walked TRANSITIVELY in BFS depth order — an already selected path is skipped silently, so a cycle is deduped and never an error, and a dangling or out-of-bundle link is tolerated (OKF §5; grading it is `knowledge check`\'s job), (4) the bee.critical: true concepts RANKED BY RELEVANCE to the work item and cut to the pinned keep, (5) every bee.decision concept whose bee.areas overlaps the work item\'s. The ranked list is then cut at --budget <tokens>: the first entry that would overshoot ends the manifest, and it plus every lower-ranked entry is named in `truncated`. Relevance is the IDF-weighted fraction of a concept\'s own distinctive vocabulary that the work item\'s text covers, over title/description/tags and body, plus a small tag/area bonus — chosen by measurement over the live bundle (AUC 0.805 against hand labels; tag overlap alone scored 0.550 and left 48 of 49 patterns tied at zero, so it is disqualified as the signal). The top few criticals are a FLOOR whose cost is reserved out of the budget left after rank 1, so a universal lesson is never evicted by a long required_context chain and the work item is never displaced by its own floor; --budget remains a hard ceiling. Nothing is dropped silently: every critical is accounted for exactly once across `entries` (whose reason names its score and rank), `truncated`, and `excluded` ({path, score, reason}), with `critical_total` stating the population. `zero_signal_count` is always reported, and a real population that is mostly zero FAILS with a typed zero_signal error — a ranking where most items tie at zero is a path sort wearing a relevance label. Ties break by path, so two runs are byte-identical. Sizes are estimated as bytes/4 and the output NAMES that estimator — bee vendors no tokenizer. The output is an ordered MANIFEST — per entry: repo-relative path, bytes, est_tokens, and a one-line inclusion reason — and NEVER file content; the agent decides what to read. `decisions` in the header is informational: the work item\'s own bee.decisions list, read from its frontmatter, never from a .bee/ store. An unresolvable --work id exits 1 with a typed unknown_work error.',
    parameters: {
      type: 'object',
      properties: {
        work: { type: 'string', description: 'The work item to assemble context for — matched against bee.id on a bee.work-item concept (the id is identity).' },
        budget: { type: 'number', description: 'Context budget in estimated tokens; sizes are bytes/4 (no tokenizer dependency). Explicit --budget always wins over --lane when both are given.' },
        lane: { type: 'string', enum: ['tiny', 'small', 'standard', 'high-risk'], description: 'Shorthand mapping to a lane-scaled budget preset (tiny 8000 / small 12000 / standard 20000 / high-risk 30000), resolved into --budget before this call reaches validate() — only when --budget is omitted. Omitting both --budget and --lane behaves exactly as before this option existed (required, missing).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON ({work,decisions,budget,estimator,total_est,entries,truncated,excluded,floor,critical_total,zero_signal_count}) instead of the human table.' },
      },
      required: ['work', 'budget'],
    },
    examples: [
      'bee knowledge context --work okf-foundation --budget 20000 --json',
      'bee knowledge context --work okf-foundation --lane standard --json',
    ],
    deprecated: null,
  },
  {
    name: 'knowledge.promote',
    invoke: 'bee knowledge promote',
    description:
      'Propose the knowledge a finished work item earned — and never write it. Resolves --work <id> to its bee.work-item concept, then READS the capped cell traces of that feature from .bee/cells/*.json (a read of the runtime store, which the bundle may perform; it is never a write path into one) and returns three PROPOSALS: (a) a DELIVERY DRAFT — a complete bee.delivery concept in canonical emitter form, ready to be saved as the work item\'s delivery.md sibling, carrying what shipped (each cell\'s recorded outcome), how it was verified (each cell\'s recorded verify command and evidence) and every recorded deviation; (b) AREA UPDATES — for each area named in the work item\'s bee.areas, the capped behavior_change cells whose files_changed touch that area\'s subject, as candidate spec-sync bullets each citing its cell id; (c) PATTERN CANDIDATES — every capped cell whose trace carries a deviation or a failure signature, shaped as a candidate bee.pattern concept with bee.polarity pitfall and bee.lifecycle draft, quoting the trace verbatim. Every proposed line traces to a cell trace or to the work item — nothing is invented. promote proposes and never writes — not into docs/knowledge/, not into .bee/*.json(l), not anywhere: `writes` in the --json payload is always []; applying a proposal is a human or agent decision. An unresolvable --work id exits 1 with a typed unknown_work error.',
    parameters: {
      type: 'object',
      properties: {
        work: { type: 'string', description: 'The work item to propose knowledge for — matched against bee.id on a bee.work-item concept (the id is identity).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON ({work,work_item,cells,delivery,area_updates,pattern_candidates,writes}) instead of the human proposal document.' },
      },
      required: ['work'],
    },
    examples: ['bee knowledge promote --work okf-foundation --json'],
    deprecated: null,
  },

  // ─── perf (lib/perf.mjs) — global cross-project performance log ───────────
  // Each section summarizes a piece of work: models used, per-model tokens
  // (new/cached/total), whether parallel, and running time (active, not idle).
  // The store is ~/.config/beehive/performance.jsonl (XDG-aware; BEEHIVE_PERF_DIR
  // override), shared across every project. Metrics are recovered from the
  // Claude Code session transcript on disk.
  {
    name: 'perf.start',
    invoke: 'bee perf start',
    description:
      'Open a named performance section: record the resolved session transcript + start time in .bee/cache/perf-open.json so `perf stop` measures the same window.',
    parameters: {
      type: 'object',
      properties: {
        label: { type: 'string', description: 'A short name for the piece of work this section covers.' },
        session: { type: 'string', description: 'Explicit Claude Code session id; default is the newest-mtime transcript for this project.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee perf start --label demo --json'],
    deprecated: null,
  },
  {
    name: 'perf.stop',
    invoke: 'bee perf stop',
    description:
      'Close the open section: slice the recorded transcript window, aggregate per-model tokens + parallelism + running time, append the section to the global log, and clear the marker.',
    parameters: {
      type: 'object',
      properties: {
        note: { type: 'string', description: 'An optional summary note stored on the section.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee perf stop --json'],
    deprecated: null,
  },
  {
    name: 'perf.section',
    invoke: 'bee perf section',
    description:
      'One-shot section over a trailing window (no prior start): compute + append a section covering everything since --since (e.g. 30m, 2h, 1d, or an ISO time).',
    parameters: {
      type: 'object',
      properties: {
        since: { type: 'string', description: 'Window start: a duration like 30m/2h/1d, or an ISO timestamp.' },
        label: { type: 'string', description: 'A short name for this section.' },
        note: { type: 'string', description: 'An optional summary note.' },
        session: { type: 'string', description: 'Explicit session id; default newest-mtime transcript for this project.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee perf section --since 1h --label demo --json'],
    deprecated: null,
  },
  {
    name: 'perf.log',
    invoke: 'bee perf log',
    description: 'Read recent sections from the global performance log (one line each, most recent last).',
    parameters: {
      type: 'object',
      properties: {
        limit: { type: 'string', description: 'Show only the most recent N sections.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of one-line summaries.' },
      },
      required: [],
    },
    examples: ['bee perf log --json'],
    deprecated: null,
  },
  {
    name: 'perf.render',
    invoke: 'bee perf render',
    description: 'Render the global performance log as a Markdown report.',
    parameters: {
      type: 'object',
      properties: {
        limit: { type: 'string', description: 'Render only the most recent N sections.' },
        json: { type: 'boolean', description: 'Emit the underlying sections as JSON instead of Markdown.' },
      },
      required: [],
    },
    examples: ['bee perf render'],
    deprecated: null,
  },
  {
    name: 'perf.report',
    invoke: 'bee perf report',
    description:
      'Scan every project\'s session activity and build the full per-project performance matrix. With --html (or --out) writes a self-contained HTML report; otherwise prints a per-project summary. Uses a per-transcript cache so repeat runs are fast.',
    parameters: {
      type: 'object',
      properties: {
        html: { type: 'boolean', description: 'Write a self-contained HTML matrix report instead of text output.' },
        out: { type: 'string', description: 'Path for the HTML report (implies --html); default ~/.config/beehive/performance.html.' },
        since: { type: 'string', description: 'Only include sessions active since this moment (duration like 7d/48h, or an ISO timestamp).' },
        json: { type: 'boolean', description: 'Emit the raw scan as JSON.' },
      },
      required: [],
    },
    examples: ['bee perf report --json'],
    deprecated: null,
  },
  {
    name: 'perf.sync',
    invoke: 'bee perf sync',
    description:
      'Scan every project\'s session activity and write one rolled-up row per session into the performance log (performance.jsonl). Backfills history; the report then reads the log. Cache-backed, so repeat syncs are fast.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee perf sync --json'],
    deprecated: null,
  },

  // ─── worktree (worktree-feature-parallelism Slice A) — register/list/
  // unregister a linked git worktree's opt-in per-worktree `.bee` store, and
  // bootstrap the granted worktree's own store so bee actually works there.
  // The resolver has honored `<main>/.bee/runtime/worktree-grants.json`
  // since the wire-in slice; this group is what lets a human populate it. ──
  {
    name: 'worktree.register',
    invoke: 'bee worktree register',
    description:
      "Grant the CURRENT linked git worktree its own local .bee store (writes its id into the MAIN checkout's runtime/worktree-grants.json) and bootstrap that worktree's .bee/: copies onboarding.json/config.json from the main store if present, and writes a FRESH state.json (phase idle, every gate unapproved, feature set) so the worktree runs its own independent lifecycle. Must be run from inside a linked worktree created with `git worktree add`; fails with a typed error from an ordinary checkout or an invalid/broken worktree link. Idempotent: re-running never overwrites an existing worktree state.json.",
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug the worktree will run (stamped into its bootstrapped state.json).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a short confirmation report.' },
      },
      required: ['feature'],
    },
    examples: ['bee worktree register --feature demo-feature --json'],
    deprecated: null,
  },
  {
    name: 'worktree.new',
    invoke: 'bee worktree new',
    description:
      "Create AND register a fresh linked git worktree for an independent feature in ONE move: runs `git worktree add ../<repo-basename>--wt--<feature> -b wt/<feature> [resolved baseRef sha]`, then grants and bootstraps it exactly as `worktree register` does (copies onboarding.json/config.json from the main store if present, writes a FRESH state.json — phase idle, every gate unapproved, feature set). Must be run from the MAIN checkout (an ordinary, non-worktree directory), never from inside another linked worktree. Typed, zero-mutation refusal when the feature slug is invalid, --base-ref does not resolve to a commit, the target sibling path or branch already exists, or a grant already exists for the derived id; `git worktree add` failing at runtime is caught and re-surfaced typed too, and a failure AFTER the worktree was created rolls back best-effort. With `--with-companion`, also runs the project-configured `commands.worktree_companion_start` and symlinks its result into the new worktree at `commands.worktree_companion_mount` — for a nested repo (its own `.git`, gitignored by this one) a bare worktree can't otherwise isolate; bee never assumes what the companion tool is, only that its start command prints JSON `{worktreePath, sessionId?}`. A companion-start failure rolls the whole worktree back, same as any other post-creation failure.",
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug for the new worktree (must match ^[a-z0-9][a-z0-9-]*$); becomes branch `wt/<feature>` and directory `../<repo-basename>--wt--<feature>`, and is stamped into the bootstrapped state.json.' },
        'base-ref': { type: 'string', description: 'Git commit-ish to base the new branch on — branch, tag, HEAD~N, short sha, `<tag>^{commit}`, etc. Resolved to a concrete commit sha via `git rev-parse --verify --end-of-options` (git >= 2.24), and that RESOLVED sha (not the ref string) is what the new branch is actually based on. Defaults to the main checkout\'s current HEAD when omitted.' },
        'with-companion': { type: 'boolean', description: 'Also run commands.worktree_companion_start and mount its result at commands.worktree_companion_mount inside the new worktree. Requires both to be set in .bee/config.json.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a short confirmation report.' },
      },
      required: ['feature'],
    },
    examples: ['bee worktree new --feature demo-feature --json', 'bee worktree new --feature demo-feature --with-companion --json'],
    deprecated: null,
  },
  {
    name: 'worktree.merge',
    invoke: 'bee worktree merge',
    description:
      "Merge a granted worktree's branch back into the MAIN checkout — `git merge --no-ff <branch>` run from MAIN, then the host project's configured commands.verify (if recorded) run against the merged tree. A textually-clean merge whose verify goes RED is the semantic-conflict alarm: behavior broke even though git found no conflict; the merge commit is NEVER rolled back. Must be run from the MAIN checkout (an ordinary, non-worktree directory) — running it from inside ANY linked worktree, including the one being merged, is refused (a worktree cannot merge itself). Typed, zero-mutation refusal when the id is unknown/ungranted, the MAIN or WORKTREE tree is dirty (a bootstrapped gitignored .bee store alone does not count as dirty), or the worktree is on a detached HEAD or a branch other than its expected `wt/<slug>`-style branch. With `--cleanup` and a green verify, cleanup runs unconditionally: `git worktree remove --force` (safe only because freshness was re-checked immediately before), `git branch -d` (never -D), then removeGrant — so a merged-and-cleaned-up id never lingers in `bee worktree list`. A repo with no commands.verify recorded (verify:'skipped') is ALSO cleanup-eligible, but the result always carries a loud warning that nothing was semantically gated. Without `--cleanup` the result only suggests the cleanup command; cleanup never runs when the merge itself came back MERGE_CONFLICT or MERGE_VERIFY_RED, even with --cleanup passed. No flag needed for companion teardown: if the worktree was created `--with-companion`, its marker alone is enough — commands.worktree_companion_end runs and the mounted symlink is removed BEFORE the dirty-tree check, regardless of --cleanup or how the merge itself resolves (an untracked companion symlink would otherwise read as a dirty worktree on every attempt). Queue-aware: every invocation enqueues through the shared integration queue (`controlRoot/.bee/runtime/integration/queue/`) and becomes the processor the instant the lease is free — an empty queue resolves immediately with the same output as an uncontended merge. A second concurrent merge against the same MAIN checkout waits its turn (bounded by --queue-wait-ms, default 180000) instead of racing the old dirty-tree refusal; on timeout the result is `ok:false, code:'INTEGRATION_QUEUE_TIMEOUT'` and the text unambiguously says the merge did NOT run, with the request's queue position — never a shape readable as success.",
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: "The granted worktree's git-verified id (see `bee worktree list`)." },
        cleanup: { type: 'boolean', description: "After a successful merge with a green (or skipped, loudly-warned) verify, remove the worktree, delete its branch, and drop its grant, unconditionally. Never runs after a conflict or a red verify." },
        'queue-wait-ms': { type: 'number', description: 'Override the integration queue\'s bounded wait (default 180000ms / a few minutes) for this invocation before giving up and returning a queue position instead of running the merge.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a short confirmation report.' },
      },
      required: ['id'],
    },
    examples: ['bee worktree merge --id demo-feature-missing --json', 'bee worktree merge --id demo-feature --queue-wait-ms 5000 --json'],
    deprecated: null,
  },
  {
    name: 'worktree.list',
    invoke: 'bee worktree list',
    description: "List the MAIN store's worktree grant registry (which worktree ids currently have their own local .bee store).",
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-grant summary.' },
      },
      required: [],
    },
    examples: ['bee worktree list --json'],
    deprecated: null,
  },
  {
    name: 'worktree.unregister',
    invoke: 'bee worktree unregister',
    description: "Remove a worktree's grant from the MAIN store's registry, so the resolver falls back to the main store for that id. Defaults to the CURRENT linked worktree's own id when --id is omitted.",
    parameters: {
      type: 'object',
      properties: {
        id: { type: 'string', description: 'Worktree id to revoke. Omit to use the current directory\'s own linked-worktree id.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee worktree unregister --id abc123 --json'],
    deprecated: null,
  },

  // ─── herding (herding-dispatch-lock-toggle, decisions D1-D5) — the human
  // owner's terminal-only convenience for the bee-herding dispatch loop's
  // owner enable marker. Byte-for-byte the same filesystem operation as
  // today's manual `touch`/`rm` of `<main-root>/.bee/tmp/bee-herding.enable`;
  // `.claude/skills/bee-herding/scripts/dispatch-interlock.mjs` stays the
  // sole reader and is never modified or called from here (D4). No runtime
  // guard is added (D5, explicit user decision) — no TTY/interactivity
  // check, and this group is NOT hidden from `bee.mjs --help --json`.
  // TRACKING (multisession-native-17): the `git rev-parse --git-common-dir`
  // mention in herding.enable's description below is documentation only —
  // the actual resolution lives in lib/herding.mjs's resolveHerdingMainRoot,
  // which this file never duplicates (no fs/child_process code in this
  // module at all). See herding.mjs's own tracking comment for why that
  // resolver is deliberately left independent of state.mjs's resolveContext
  // (the canonical resolver everywhere else). ──────────────────────────────
  {
    name: 'herding.enable',
    invoke: 'bee herding enable',
    description:
      "Create the bee-herding dispatch loop's owner enable marker (<main-root>/.bee/tmp/bee-herding.enable) — the SAME marker `.claude/skills/bee-herding/scripts/dispatch-interlock.mjs` reads before dispatch may build any dispatchable set. The main checkout root is resolved via `git rev-parse --git-common-dir`, identically to dispatch-interlock.mjs. Idempotent: running this on an already-enabled marker is not an error. Human-only convenience — never call this from dispatch-interlock.mjs, bootstrap, dispatch, merge, or any other bee automation/skill/agent code.",
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee herding enable --json'],
    deprecated: null,
  },
  {
    name: 'herding.disable',
    invoke: 'bee herding disable',
    description:
      "Remove the bee-herding dispatch loop's owner enable marker (<main-root>/.bee/tmp/bee-herding.enable), so `.claude/skills/bee-herding/scripts/dispatch-interlock.mjs` refuses to let dispatch build a dispatchable set again. Idempotent: running this on an already-absent marker is not an error. Human-only convenience — never call this from dispatch-interlock.mjs, bootstrap, dispatch, merge, or any other bee automation/skill/agent code.",
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee herding disable --json'],
    deprecated: null,
  },
  {
    name: 'herding.status',
    invoke: 'bee herding status',
    description:
      "Report whether the bee-herding dispatch loop's owner enable marker currently exists — {enabled, marker, main_root}, the same shape dispatch-interlock.mjs itself emits on stdout.",
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line summary.' },
      },
      required: [],
    },
    examples: ['bee herding status --json'],
    deprecated: null,
  },

  // ─── tmp (tree-hygiene th-4, docs/history/tree-hygiene/CONTEXT.md D1/D2) —
  // the ONE canonical scratch home (.bee/tmp/<feature-or-session>/,
  // .bee/spikes/<feature>/) and its broom. lib/scratch.mjs owns every safety
  // check (containment re-proved immediately before each removal, symlink
  // escapes refused rather than followed — reusing the write-guard's own
  // canonicalRelPath/isUnderRoot idiom); this entry is presentation only. ──
  {
    name: 'tmp.sweep',
    invoke: 'bee tmp sweep',
    description:
      "Remove scratch from .bee/tmp/ and .bee/spikes/ — the canonical home for every ephemeral file bee writes (judge payloads, evidence files, batch data, digests, verify logs, probe/debug scripts). An entry is any top-level directory OR loose file under those roots, since agents write helper scripts and evidence dumps straight into the root. ROOT-RESTRICTED: a candidate may only ever be removed once it is canonically resolved and re-checked contained inside one of those two roots; an escaping or symlinked candidate is refused, never followed. Refuses (typed, zero mutation) with NO flags at all — no default purge, same discipline as `decisions archive`'s mandatory --before. Default target set (neither --feature nor --all given): scratch whose feature/lane record is at a terminal phase (closed) is swept unconditionally; scratch with no record anywhere (absent) is swept only once older than --before. A LIVE feature's scratch survives the default sweep unless named explicitly via --feature; --all clears every entry, live or not. --dry-run reports exactly what would be removed (bytes/files) without deleting anything.",
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Sweep this one feature explicitly — the only way to sweep a LIVE feature\'s scratch. Matches the exact name plus its `<feature>-<n>` per-cell dirs and loose `<feature>-*` root files (bee\'s own cell-id convention); a prefix match never removes a sibling that is itself live.' },
        before: { type: 'string', description: 'ISO date age cutoff. In the default (non-all, non-feature) sweep, gates removal of scratch with no feature/lane record anywhere (a closed-record dir is swept regardless of this cutoff).' },
        all: { type: 'boolean', description: 'Clear every scratch entry under .bee/tmp/ and .bee/spikes/ — directories and loose root files alike, live or closed or absent alike.' },
        'dry-run': { type: 'boolean', description: 'Report exactly what would be removed (paths, bytes, files) without deleting anything.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: [],
    },
    examples: ['bee tmp sweep --all --dry-run --json'],
    deprecated: null,
  },

  // ─── config (ao-2ai-1) — loud refusal of malformed/prompt-less/unsafe
  // cli-tier config, where today it silently reverts to the seeded default
  // (normalizeTierValue -> undefined -> normalizeModels never overwrites). ──
  {
    name: 'config.get',
    invoke: 'bee config get',
    description: 'Read a .bee/config.json value by key (dot-notation for nested keys, e.g. guards.idle_gate). Exits 0 whether or not the key is set.',
    parameters: {
      type: 'object',
      properties: {
        key: { type: 'string', description: 'Config key, dot-notation for nested (e.g. product_root, guards.idle_gate).' },
        local: { type: 'boolean', description: 'Read the machine-local overlay file (.bee/config.local.json) only, instead of the effective (overlay-over-tracked) value guards.*/hooks.* keys resolve to by default.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['key'],
    },
    examples: ['bee config get --key gate_bypass --json'],
    deprecated: null,
  },
  {
    name: 'config.set',
    invoke: 'bee config set',
    description: 'Set a .bee/config.json value by key (dot-notation for nested), validating on write, instead of hand-editing the JSON. The value is parsed as JSON when it parses (true/false/numbers/objects) else kept as a string; --string forces a string. Refuses to write if it would introduce an invalid models/cli-tier config, or if the existing file is unparseable.',
    parameters: {
      type: 'object',
      properties: {
        key: { type: 'string', description: 'Config key, dot-notation for nested (e.g. product_root, guards.idle_gate).' },
        value: { type: 'string', description: 'The value; parsed as JSON when possible (false -> boolean), else a string.' },
        string: { type: 'boolean', description: 'Force the value to be stored as a string (skip JSON coercion).' },
        local: { type: 'boolean', description: 'Write to the machine-local overlay file (.bee/config.local.json) instead of the tracked config.json. guards.*/hooks.* keys are ALWAYS forced local regardless of this flag.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['key', 'value'],
    },
    examples: ['bee config set --key gate_bypass --value false --json'],
    deprecated: null,
  },
  {
    name: 'config.unset',
    invoke: 'bee config unset',
    description: 'Remove a .bee/config.json key (dot-notation for nested). A no-op (exit 0) when the key is absent. Refuses if the existing file is unparseable.',
    parameters: {
      type: 'object',
      properties: {
        key: { type: 'string', description: 'Config key to remove, dot-notation for nested.' },
        local: { type: 'boolean', description: 'Remove from the machine-local overlay file (.bee/config.local.json) instead of the tracked config.json. guards.*/hooks.* keys are ALWAYS forced local regardless of this flag (same as config set).' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['key'],
    },
    examples: ['bee config unset --key gate_bypass --json'],
    deprecated: null,
  },
  {
    name: 'config.validate',
    invoke: 'bee config validate',
    description:
      'Validate .bee/config.json models config: flags a cli-tier value missing kind:"cli"/a non-empty command (silently reverts to the seeded default today), a cli value with no declared prompt transport, and a cli command containing a known unsafe auto-approve/sandbox-bypass flag. Never throws on malformed/null config — reports it as a problem row instead. Exits non-zero when any problem is found.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-problem report.' },
      },
      required: [],
    },
    examples: ['bee config validate --json'],
    deprecated: null,
  },

  // ─── recovery (transcript-recovery D1-D6, docs/history/transcript-recovery/
  // CONTEXT.md) — crash-candidate detection + bounded mining-window math.
  // Schemas only: this file must NOT import lib/recovery.mjs (the perf-import
  // discipline extended to the recovery module — see recovery.mjs's own file
  // header). Mining itself never runs inside the CLI (D4): `recovery window`
  // only emits the down-tier worker's prompt; the orchestrator dispatches it.
  // `recovery scan` never auto-triggers mining (D2). ─────────────────────────
  {
    name: 'recovery.scan',
    invoke: 'bee recovery scan',
    description:
      'Detect recoverable crash candidates: sessions whose heartbeat is stale, whose transcript exists and lacks the clean-end trio, and that show a work signal (bound lane in a non-terminal phase, an active claimed cell, or transcript activity newer than the last durable settlement). Cheap and side-effect-free — never triggers mining.',
    parameters: {
      type: 'object',
      properties: {
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line-per-candidate summary.' },
      },
      required: [],
    },
    examples: ['bee recovery scan --json'],
    deprecated: null,
  },
  {
    name: 'recovery.window',
    invoke: 'bee recovery window',
    description:
      "For one crash-candidate session id, re-derive the bounded mining window and the down-tier miner prompt: resolves the session's transcript, computes the window start from the last durable settlement (lane-scoped, global fallback, or the session's own started_at when nothing settled), applies the hard event cap, and returns {transcript, since_ts, event_count, window_truncated, prompt}. Never calls an LLM itself — the orchestrator dispatches the returned prompt to a down-tier worker.",
    parameters: {
      type: 'object',
      properties: {
        session: { type: 'string', description: 'The crash-candidate session id, from `recovery scan`.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of printing the prompt text.' },
      },
      required: ['session'],
    },
    examples: ['bee recovery window --session sess-recovery-demo --json'],
    deprecated: null,
  },

  // ─── doctor (codex-native-runtime-v2 D11) — fail-closed runtime health
  // report. Reuses the onboarding-recorded hash baseline for drift (never a
  // second hash implementation) and cites the capability matrix
  // (docs/history/codex-native-runtime-v2/reports/capability-matrix.md) for
  // every structurally-unknown codex row. Performs zero writes, including
  // bypassing the dispatcher's own pre-routing manifest-hash cache write. ──
  // ─── dispatch (g22-1, GH #22 P0-3) — one source of truth for bee-owned
  // dispatch payloads: resolves the tier/advisor slot for the given
  // (runtime, kind), builds the exact Agent/spawn_agent/Bash-shaped envelope
  // dispatch-guard.mjs's evaluateDispatch will judge, and records a
  // prepare-time economics line to .bee/logs/dispatch.jsonl. ────────────────
  {
    name: 'dispatch.prepare',
    invoke: 'bee dispatch prepare',
    surface: 'porcelain',
    description:
      'Build a bee-owned dispatch payload (Agent tool / spawn_agent / an external cli executor) for the given runtime and purpose, plus an economics record (logical_tier, requested_model, channel, enforcement). kind "cell" resolves the generation tier for cell execution and requires --cell (loaded for prompt context) and --worker (checked against the cell\'s own status/claim owner); kinds "gather"/"reviewer" resolve read-only gather-shaped tiers (generation/review respectively); kind "advisor" resolves the configured advisor slot, never a bare tier. A cli-shaped resolution for kind "cell" is returned as a typed refusal ({ok:false, reason:"cli_tier_gather_only", ...}) — prepare never routes around it. For kind "cell", an unclaimed or foreign-claimed cell is refused as {ok:false, type:"refused", reason:"claim_ownership", code, status, owner, fix} naming the actual status/owner — --force-ownership overrides it and appends an audited ownership_override entry to the prepare-time dispatch record. A cli-shaped resolution for gather/reviewer/advisor builds an external-executor Bash payload instead of an Agent/spawn_agent one.',
    parameters: {
      type: 'object',
      properties: {
        runtime: { type: 'string', description: 'Target runtime the payload is shaped for.', enum: ['codex', 'claude'] },
        kind: { type: 'string', description: 'Dispatch purpose.', enum: ['cell', 'gather', 'reviewer', 'advisor'] },
        cell: { type: 'string', description: 'Cell id — required when --kind cell; loaded for prompt context.' },
        worker: { type: 'string', description: 'Requesting worker identity — required when --kind cell; checked against the cell\'s status/trace.worker (claim-ownership guard).' },
        'force-ownership': { type: 'boolean', description: 'Override a claim-ownership refusal for --kind cell (audited into the prepare-time dispatch record). Ignored for every other kind.' },
        claim: { type: 'boolean', description: 'One verb from "cell chosen" to "worker prompt in hand": claim the cell for --worker first (the same door as `cells claim` — refusals pass through unchanged), then reserve every path in the cell\'s files for that worker at the default TTL, then build the payload exactly as without the flag. On a reservation conflict the claim is unwound and the refusal names the conflicting paths and holder — state is left as it was found. The result gains {claimed: true, reserved: [paths]}. Only valid with --kind cell.' },
        'session-id': { type: 'string', description: 'Session id the --claim claim is taken under (same optional/env-resolved convention as `cells claim`). Ignored without --claim.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of pretty-printed JSON (prepare always prints JSON; flag kept for surface consistency).' },
      },
      required: ['runtime', 'kind'],
    },
    examples: ['bee dispatch prepare --runtime claude --kind gather --json', 'bee dispatch prepare --runtime claude --kind cell --cell demo-1 --worker exec-demo-1 --json', 'bee dispatch prepare --runtime claude --kind cell --cell claim-demo-1 --worker exec-claim-demo --claim --json'],
    deprecated: null,
  },

  {
    name: 'doctor',
    invoke: 'bee doctor',
    surface: 'porcelain',
    description:
      'Fail-closed runtime health report, a THREE-state verdict: overall_status is "blocked" when any mechanical row (hooks-file presence, capability-baseline byte match, hook-handler resolvability, skills-installed) is not ok; "degraded" when every mechanical row is ok but codex\'s hook-discovery/trust/project-trust/pending-review rows are still structurally unknown (per the capability matrix) and no valid attestation covers them; "ready" only with mechanical rows all ok AND, on codex, a currently-valid attestation (see "doctor attest") — claude has no trust-unknown rows, so mechanical green alone reaches ready there, no attestation required. Trust-row wording is version-scoped: a live codex --version other than the probed one reports "unprobed_version" instead of asserting the probed conclusions. Never "ready" from file presence alone. Performs zero writes anywhere, including the dispatcher\'s own pre-routing manifest-hash cache.',
    parameters: {
      type: 'object',
      properties: {
        runtime: { type: 'string', description: 'Which runtime to report on.', enum: ['codex', 'claude'] },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a human-readable report.' },
      },
      required: ['runtime'],
    },
    examples: ['bee doctor --runtime codex --json', 'bee doctor --runtime claude --json'],
    deprecated: null,
  },
  {
    name: 'doctor.attest',
    invoke: 'bee doctor attest',
    description:
      'Record a static attestation that codex trust state was reviewed via the interactive /hooks TUI for THIS exact pairing of .codex/hooks.json content, codex --version, and repo identity. Written to the gitignored .bee/doctor-attest.json (never tracked state). "bee doctor --runtime codex" treats the attestation as valid only while all three legs still match the live state; any single drifted leg makes it inert and doctor reports "degraded" naming the stale reason (hash_changed / version_changed / identity_changed / no_attestation). --runtime codex only — claude has no trust-unknown rows to attest. No liveness leg exists: codex exposes no hook-fire event surface to observe, so this is a static record, not a health probe.',
    parameters: {
      type: 'object',
      properties: {
        runtime: { type: 'string', description: 'Must be "codex" — claude has no attestation model.', enum: ['codex'] },
        session: { type: 'string', description: 'Optional session id to record alongside the attestation; defaults to $CODEX_SESSION_ID or $CLAUDE_SESSION_ID when unset.' },
        json: { type: 'boolean', description: 'Emit machine-readable JSON instead of a one-line confirmation.' },
      },
      required: ['runtime'],
    },
    examples: ['bee doctor attest --runtime codex --json'],
    deprecated: null,
  },

  // ─── close (porcelain, docs/specs/porcelain.md) — the feature close driver.
  // One verb answering "what stands between this feature and done, and can we
  // pay it now". The door predicates are the EXACT ones the existing close
  // path enforces (FEATURE_DEBT_KINDS in lib/state.mjs, plus scribingDebt /
  // captureQueue) — never a second implementation of any debt rule, and close
  // never waives a door. ────────────────────────────────────────────────────
  {
    name: 'close',
    invoke: 'bee close',
    surface: 'porcelain',
    description:
      'Feature close driver: reports every close door — the pending feature-level verify, consolidated test coverage, scribing/capture debt — each with the exact command that settles it. --dry-run is a read-only report ({feature, doors:[{door, blocking, detail, command}]}). Without --dry-run, when the ONLY outstanding door is the feature verify and a verify command is recorded for the feature, close runs that command, records the pass/fail through `state feature-verify record`, and reports: on green the text names what remains (the capture checklist) and the next skill; on red the failing output tail is surfaced and nothing is recorded as passed. When any other door blocks, close reports it and runs nothing. Doors are never waived — debts close cannot pay are reported with their commands.',
    parameters: {
      type: 'object',
      properties: {
        feature: { type: 'string', description: 'Feature slug to drive toward close.' },
        'dry-run': { type: 'boolean', description: 'Read-only: report the close doors and what settles each, without running anything.' },
        json: { type: 'boolean', description: 'Emit the machine-readable door report instead of the one-line-per-door text.' },
      },
      required: ['feature'],
    },
    examples: ['bee close --feature demo --dry-run --json'],
    deprecated: null,
  },
];
