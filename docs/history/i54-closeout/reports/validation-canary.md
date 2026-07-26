# validation-canary — live codex 0.145.0 evidence (i54-closeout)

Collected 2026-07-24 during validating, on this machine (`codex-cli 0.145.0`,
`which codex` → `~/.local/bin/codex`). Three independent probes; provenance
labeled per claim. Feeds cells i54-closeout-1 (schema alignment) and
i54-closeout-8 (capability pin) per CONTEXT D1/D8.

## 1. Full canary run (`node scripts/canary_codex.mjs`) — P5 red, real bug

- P1 SessionStart (real codex exec): **PASS**
- P2 unmarked spawn deny (installed hook): **PASS** (exit 2)
- P3 marked spawn allow (installed hook): **PASS** (exit 0)
- P4 update_plan state-sync (installed hook): **PASS**
- P5 pre-Gate-3 write block: **FAIL** — `bee-write-guard.mjs` exit 1 (expected
  2): `ERR_MODULE_NOT_FOUND: .../.bee/bin/hooks/tokenize-command.mjs`.

Root cause (verified in source): `hooks/bee-write-guard.mjs:38` imports
`./tokenize-command.mjs`, but `HOOK_FILENAMES` in
`skills/bee-hive/scripts/onboard_bee.mjs:177-194` does not list it, so a fresh
repo-hooks install never vendors it and the write guard CRASHES (fail-open) on
every fresh install. Introduced by the internals-reach guard work
(state-query-surface, sqs-a). → cell i54-closeout-9; cell 8's "full canary
green" precondition is blocked on it.

## 2. Native-override probe (`canary_codex.mjs --probe`) — envelope rejected

Machine record: `.bee/native-transport-probe.json`
(`config_scope_hash af38f955…`, at 2026-07-24T08:49:35Z):

- `multi_agent: true`, `multi_agent_v2: true`
- `override_spawn_accepted: false` — live turn errored HTTP 400:
  *"Invalid Value: 'tools'. Function 'collaboration.spawn_agent' is reserved
  for use by this model and must match the configured schema."*
- `v3_hook_invoked: false`, `v3_envelope_count: 0`
- `classification: native_budget_only`

Meaning: on 0.145.0 the spawn_agent tool schema is server-governed and cannot
be redefined; per-agent model override envelopes are NOT accepted
(`native_budget_only`). R18 consequence: no capability row may claim override
support on 0.145.0.

## 3. Tool-schema self-inspection (live `codex exec --ephemeral`, spike dir)

Prompt asked the model to print its spawn/collaboration tool parameter schema
verbatim (no tool call made). Reply, verbatim:

```json
{"type":"object","properties":{
  "fork_turns":{"type":"string"},
  "message":{"type":"string"},
  "model":{"type":"string"},
  "reasoning_effort":{"type":"string"},
  "task_name":{"type":"string"}},
 "required":["message","task_name"]}
```

Provenance: model self-report — treated as evidence only because it is
corroborated by (a) the canonical doc `swarming-reference.md:134`
(`{task_name, message, fork_turns}`), (b) issue #54's advisor reading the same
schema from an independent environment, and (c) probe 2's 400 error proving
the schema is server-configured, not client-declared.

## Verdict for cell i54-closeout-1 (D1)

- **`task_name` is REQUIRED; `agent_type` does not exist in the 0.145.0
  schema.** The helper's everyday payload `{agent_type:'worker', message}`
  (dispatch-prepare.mjs:389-397) is invalid on this version: missing required
  `task_name`.
- Target emitted shape: `{task_name: "<stable-name>", message: "[bee-tier: …]\n…", fork_turns: "none"}`
  — exactly the doc-canonical form. `model`/`reasoning_effort` fields exist in
  the schema but probe 2 proves the override path is rejected end-to-end
  (`native_budget_only`) — the ordinary emit path must NOT attach them.
- Guard: evaluate every `spawn_agent` payload by TOOL NAME (marker check on
  `message`), covering both the new `{task_name,…}` shape and the legacy
  `{agent_type,…}` shape; unmarked deny stays for both. No verdict may key on
  `agent_type` presence alone.

## Verdict for cell i54-closeout-8 (D8)

- Pin target: `PROBED_CODEX_VERSION = '0.145.0'` (exact `codex --version`
  string: `codex-cli 0.145.0`).
- Observed on 0.145.0: multi_agent + multi_agent_v2 config accepted; override
  spawn REJECTED (`native_budget_only`); P1-P4 hook chain green; P5 blocked
  only by the vendoring bug (cell 9), not by codex behavior.
- Precondition: cell 9 fixed and full canary green before the bump commits.

## 4. Post-fix full canary rerun (2026-07-24, after cells 1 and 9 capped)

`node scripts/canary_codex.mjs` → **all probes green**: P1 PASS (real codex
exec, session record written), P2 PASS (unmarked spawn denied exit 2 — the
deny message now cites i54-closeout D1, proving cell 1's widened guard is the
one running in the installed chain), P3 PASS (marked spawn allowed), P4 PASS
(state-sync), P5 PASS (pre-Gate-3 write blocked exit 2 — cell 9's vendoring
fix confirmed live in a fresh onboarded fixture). Cell 8's "full canary
green" precondition is satisfied.
