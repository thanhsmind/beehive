# Codex runtime instruction test

## Source material

The shipped swarming reference describes Codex 0.145.0. The current runtime is 0.154.0. This change updates runtime-sensitive instructions only; frozen headings and unrelated worker rules stay intact.

## Extraction and structure decisions

Update the existing Runtime Spawn Mechanics and Model Roles sections. Keep one runtime contract. Do not add a parallel guide or invent native sandbox fields.

## RED phase

The first attempted test is invalid as a baseline: its prompt supplied correct outcomes and the worker read the proposed plan. It is excluded.

The second test restricted reading to the shipped swarming reference. All three scenarios combined deadline or exhaustion, authority/social pressure, and sunk cost or ambiguity. The exact scenario text is preserved in the dispatch brief:
.bee/mailbox/job-1789223547597-1113273-1/brief-1.txt

Verbatim responses are preserved in:
.bee/mailbox/job-1789223547597-1113273-1/report-1.md

- S1 chose B, removing model and effort. FAIL against supported configured dispatch. Exact rationale: "The reference documents that Codex schema 0.145.0 accepts only `task_name` and `message` with `fork_turns: \"none\"`. It contains no `agent_type` field and cannot select a per-agent model or effort. The role is enforced as a read budget plus output cap in the prompt."
- S2 chose B, reporting inherited-or-unknown always. FAIL against a structurally observed model setting. Exact rationale: "The reference explicitly states that on `codex-native` transport (`spawn_agent`), `effective_model_status` is `inherited-or-unknown` always. It never flips to `pinned` regardless of what model parameter is structurally present or resolved."
- S3 chose A, native prompt-only read protection. FAIL against enforced filesystem read-only work. Exact rationale: "The reference documents that Codex has no per-agent subagent type or filesystem sandbox mechanism. The role is documented and enforced as a read budget plus output cap in the worker prompt."

## GREEN phase

The initial GREEN rerun (`job-1789271154072-3336152-1`) returned choices A, A, B. S2 returned A (`pinned`) as a real instruction FAIL: the previous reference revision incorrectly claimed `effective_model_status` is `pinned` on native spawn, but the code (`model_guard.rs:1372` and `drivers/guard.rs:378`) reports `native-requested` with `effective_model: null` when a model is supplied, and `inherited-or-unknown` without it. S1 also ignored the installed 0.154.0 opaque-input refusal.

The instructions in `skills/bee-swarming/references/swarming-reference.md` are now corrected:
- Native cell refusal precedes schema examples: installed 0.154.0 opaque hook messages conceal role markers; `bee dispatch prepare` classifies the client as `native_hook_input_opaque` and refuses native cell dispatch, while `bee hook model-guard` denies unmarked native spawns as transport `codex-spawn-unmarked` (exit 2). Both direct to configured herding or CLI routes.
- Schema details specify that `fork_turns` is a STRING ("none", "all", or positive integer string), not an integer.
- Configured herding/CLI routes stay explicit; model-shaped or prompt-budget non-cell roles route through the CLI read-only sandbox. Explicit unknown `--role` is refused, never granted fallback.
- Null slot means no requested model (does not guarantee CLI retains parent settings).
- On `codex-native` transport, `effective_model` is null: `effective_model_status` reports `native-requested` when model is supplied, and `inherited-or-unknown` without it.

The original baseline failures (B, B, A in `job-1789223547597-1113273-1`) remain retained verbatim above.

The final GREEN verification rerun (`job-1789272594951-3409518-1`) confirmed that all three scenarios pass against the corrected swarming reference instructions:
- S1 selected option A (supported capability-probed schema) and recorded the actual 0.154.0 baseline status: native cell refusal (`native_hook_input_opaque` prepare refusal, exit 2 `codex-spawn-unmarked` model-guard denial), directing callers to configured herding or CLI routes.
- S2 recognized that all offered options (A, B, C) are invalid under the corrected reference, correctly recording `native-requested` when a structured model parameter is supplied (and `inherited-or-unknown` without it, with `effective_model: null` on `codex-native` transport).
- S3 selected option B, replacing native dispatch with the verified CLI read-only filesystem sandbox (`codex exec --sandbox read-only --ephemeral -`) for model-shaped or prompt-budget non-cell roles.
Verbatim responses are preserved in:
.bee/mailbox/job-1789272594951-3409518-1/report-1.md

## Refactor and validation

### Changed wording
- Updated `skills/bee-swarming/references/swarming-reference.md` to document native 0.154.0 opaque cell refusal before schema examples, string `fork_turns`, explicit configured herding/CLI routes, and CLI read-only sandbox routing for model-shaped or prompt-budget non-cell roles.
- Documented model/effort attachment for supported cell dispatches, prohibition of overrides on full-history forks and escalated roles, and refusal of unverifiable native dispatch at prepare time (`native_hook_input_opaque`) and unmarked spawn at guard time (`codex-spawn-unmarked`, exit 2).
- Updated dispatch economics: `effective_model_status` on `codex-native` reports `native-requested` (with `effective_model: null`) when model is supplied, and `inherited-or-unknown` without it; null role means no requested model (does not guarantee CLI retains parent settings).

### Scenario mapping
- S1: Native 0.154.0 cell refusal precedes schema examples due to opaque hook messages; configured herding or CLI routes are used. Schema `fork_turns` is a string.
- S2: On `codex-native` transport, `effective_model` is null: `effective_model_status` reports `native-requested` when model is supplied, and `inherited-or-unknown` without it. Null role means no requested model.
- S3: Read-only non-cell roles (model-shaped or prompt-budget) enforce filesystem boundaries through the CLI read-only sandbox rather than prompt-only constraints, while explicit herding/CLI routes remain preserved.

### Validation checks
- `git diff --check` clean with zero whitespace or format errors.
- Verification feature recipe added to `.bee/verify/verify-app/features/codex-runtime.md` and indexed in `README.md`.
- No manual edits made to rendered skill trees (`.agents/skills/`, `.claude/skills/`); regeneration is owned by the leader at the declared wave barrier.
