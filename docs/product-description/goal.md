# Goal: complete the bee product description

You are working in `docs/product-description/` inside the beehive repo. Read `README.md`, `glossary.md`, `foundations/invocation.md`, and `memory/capture.md` first. The README defines the purpose, the document template, the method, the structure, and the coverage table. The other three are the exemplars: match their depth, tone, and structure exactly. Your job is to write every document in the README's structure until the coverage table has no `not started` rows, then run a consistency pass.

## Source of truth

The beehive repo is checked out at `/home/thanhsmind/Projects/goglbe/beehive` (this repo — the description set lives inside it, but treat everything outside `docs/product-description/` as read-only reference). Describe the experience in a freshly onboarded host repo with the seeded default configuration (`gate_bypass: false`, all hooks on), under Claude Code. `bee dev *`, `bee rs-info`, the fleet crate's internals, and the OpenCode plugin are out of scope.

For each document, read in this order before writing:

1. The command's registry entry in `packages/bee-rs/crates/bee/src/generated/registry_payload.json` (name, parameters, examples, availability), then its verb module under `packages/bee-rs/crates/bee/src/verbs/`.
2. The hooks where the feature is guarded or narrated: `packages/bee-rs/crates/bee/src/hooks/` (write_guard/checks.rs and guards.rs, model_guard.rs, session_preamble/, session_close, state_sync, prompt_context).
3. The tests in `packages/bee-rs/crates/bee/tests/`. They are close to executable specifications of edge cases. Key files: `front_door.rs`, `registry_dispatch.rs`, `concurrency.rs`, `hook_contracts.rs`, `workflow_verbs.rs`, `proof_gate.rs`, `session_release.rs`.
4. Recorded intent: the matching area under `docs/knowledge/areas/` (sixteen areas; `workflow-state`, `hook-runtime`, `worktree-parallelism`, `rust-runtime` carry the most).
5. Defaults and thresholds: `packages/bee-rs/crates/bee/src/state.rs`, `src/onboard/templates.rs`, and the constants the "Things already established" list below anchors.

Do not describe code. Describe what the agent sees and does. Technical detail goes only in `> Technical note:` block quotes, and only when the mechanism changes what the agent would expect.

## Writing rules

- Follow the eight-section template in the README for every command and flow document. Foundations and cross-cutting documents may drop sections that do not apply, but must still cover cancel/interrupt behavior wherever an interaction exists.
- Modifiers and cancel/interrupt go in tables, split by phase (before the first side effect / after it) as in `memory/capture.md`. The interrupt rows and the order of cross-cutting concerns are fixed in the README; do not add, drop, or reorder them in a single document.
- Use the glossary's words. If you need a term the glossary lacks, add it to `glossary.md` in the right section with a one-paragraph definition, then use it. Do not coin a synonym for an existing term. In particular: the agent (not "the user") runs commands; the human approves; a hook *denies*, the binary *refuses*.
- Sentence case for all headings. Direct, concrete language. No hedging, no marketing.
- State surprising behavior plainly and say why if the reason is in the code or a comment. If it looks like a bug, say so in "Open questions" rather than smoothing it over.
- Cross-reference other documents with relative links rather than repeating their content. `foundations/invocation.md` owns exit codes, refusal wording, `--json`, and the timing line; `foundations/store.md` owns the lock and TTLs; `foundations/gates.md` owns phases and bypass levels; `foundations/guards.md` owns the deny catalog. Do not restate them; link.
- Every document ends with "## Open questions and verification" listing what was read from code but not confirmed by hand, followed by `Verified against beehive commit \`<sha>\`` using the current `git rev-parse --short HEAD` of the beehive repo.
- Mermaid `stateDiagram-v2` for each interaction's states. Keep it to the states the agent passes through; omit internal bookkeeping states.

## Things already established (do not re-derive, do not contradict)

- Exit codes: 0 success, 1 failure; `doctor` exits 1 when blocked; `herding` also uses 3; a write-guard deny exits 2; a hook that cannot decide fails open with exit 0 and a stderr line.
- Refusal wording is a contract (five fixed phrases, `router.rs:320-326`); every argv shape gets served, refused, or unknown — never silence.
- `--json` puts the payload (errors included, as `{"error": msg}`) on stdout; without it, success text prints on stdout and error text on stderr. Every direct run prints `[bee] <cmd> <N>ms` to stderr and appends to `.bee/logs/timings.jsonl`.
- The command tree is data: `generated/registry_payload.json`, hand-maintained; `registry_dispatch.rs` runs every entry's first example against the binary.
- Five gates — `context`, `shape`, `execution`, `review`, `uat` — all default false (`state.rs:22-35`). Bypass mapping: `"total"`→total, `"full"`→full, `true`/`"on"`/`"normal"`→normal, else off (`state.rs:201-209`). Bypass lets `bee state gate` self-approve with `--actor auto --bypass-level <level>`; `--actor auto` without `--bypass-level` refuses. The UAT gate is never bypassed at any level.
- Write-guard allow-lists: gated phase — `.bee/`, `docs/history/`, `plans/`, `AGENTS.md`; idle/terminal intake — `.bee/`, `docs/`, `plans/`, `AGENTS.md`. Outside-the-worktree writes are refused except the agent's memory root and scratchpad. An unknown phase refuses rather than allows.
- Config merge law: `config.local.json` overlays `config.json`; objects merge recursively, arrays replace, overlay wins (`state.rs:134-157`). Per-hook toggles are `hooks.<name>: false`.
- Corrupt JSON in the store warns and falls back to defaults (total reads) — except coordination state (holds, reservations), where corrupt means deny (fail-closed).
- TTLs and thresholds: claim/reservation lease 3600 s; heartbeat stale 900 s; heartbeat touch throttle 60 s; store lock retry 50 ms × 100 (~5 s), stale 30 s, hard-stale 1 h; worktree prune liveness 6 h, default age 7 days; stale handoff 7 days; knowledge-context budgets by lane — tiny 8000, small 12000, standard 20000, high-risk 30000.
- Seeded host config: six hooks all on, `gate_bypass: false`, `models.claude` = code sonnet / read haiku / extraction haiku / generation sonnet, `models.codex` all null (`onboard/templates.rs:194-222`).
- Hook events (Claude): SessionStart→session-init; UserPromptSubmit→prompt-context; PreToolUse on Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion→write-guard; PreToolUse on Agent|Task→model-guard; PostToolUse on plan/task tools→state-sync; Stop→state-sync, session-close (sets the `turn-end` waiting mark); PreCompact and SessionEnd→session-close. Codex: `spawn_agent` matcher, no SessionEnd, advisory events never block.
- The model guard refuses a dispatch naming an unconfigured role or a bare tier, and repairs a pinned-type mismatch instead of refusing; silent exit 0 when it has no opinion.
- Rendered agents: bee-gather (read-only, opus), bee-extract (read-only, sonnet), bee-build (writes, opus, returns one of [DONE]/[BLOCKED]/[HANDOFF]/[NOOP]), bee-review (read-only plus read-only commands, opus).
- No-root error text: `No bee repo root found (no .bee/onboarding.json or .git up the tree). Run bee-hive onboarding.`
- Cell states: `open`, `claimed`, `capped`, `blocked`, `dropped`; archive is a file move, not a status. Budgets default 3/4/2 (claims/failed attempts/same signature), hard max 9/12/6; escalation ration 40% of the feature's cells.
- The proof line is `<command> — <result> — <scope reason>` with a literal ` — ` (em dash) separator; a `red` result refuses the cap. The cap runs no tests; close runs no tests (its tests door reads recorded proof).
- Route vocabularies: class `feature|bugfix|docs|refactor|research|release|spike|perf`; lane `docs|tiny|small|spike|standard|high-risk`. Lane transitions: promotion always allowed, high-risk never demotes, hard-gate flags block demotion, one demotion per feature ever.
- Close's doors in order: tests, scribing-debt, capture-queue (report-only), judge-debt (standard/high-risk), uat (only under `uat_stop: "close"`), pattern-check, knowledge-freshness, impact, routing, doc-deferral. Blocking doors record `blocked_by` on the merge-ready projection.
- `bee shape` is a pure alias of `bee intent set`; `bee finish` of `bee cells finish`; `bee route` of `bee state route`; `bee gate` of `bee state gate`. The intent anchor's request/acceptance are immutable once set (`--force` overrides); there is no slice record — the current slice is the set of cells that exist.
- Phases: `idle, exploring, planning, swarming, reviewing, scribing, compounding, grooming, compounding-complete`; gated = exploring/planning; terminal = idle/compounding-complete.

## Order of work

1. `memory/capture.md` (the pilot), then `foundations/` in this order: `invocation.md`, `store.md`, `session.md`, `gates.md`, `guards.md`, `worktrees.md`. Everything else links to them.
2. `lifecycle/` next, all six documents. Read the workflow record and gate code before starting any of them (`verbs/state_group/`, `verbs/workflow_store/`, `verbs/cells/`), because the states hand off to each other and the documents must agree on where one ends and the next begins. Ownership: `orient.md` owns the read-only footing; `shaping.md` owns everything up to the shape gate; `planning.md` owns lanes and the execution gate; `cells.md` owns the cell store's states; `execution.md` owns the worker's arc from claim to cap, including the proof line; `close.md` owns terminal states, UAT, and merge.
3. The remaining areas — `delegation/`, `memory/`, `discovery/`, `coordination/`, `observability/`, `maintenance/`, `reviews/`, `cross-cutting/` — are independent and can be drafted in parallel with subagents once the foundations and lifecycle documents exist to link to. If you parallelize, give each subagent this file, the exemplars, and the specific document to write; then review every result yourself for consistency with the glossary and the established facts above before accepting it.
4. Consistency pass over the whole set: same term for the same thing everywhere, no two documents describing the same behavior differently, every relative link resolves, every document has a verification footer, every glossary term used is defined.
5. Update the coverage table in `README.md` as you go: `drafted` when written, never `verified` (verification is a separate pass).

## Working rules

- Commit after each document or coherent group with `docs: add docs/product-description/<path>` or `docs: revise docs/product-description/<path>`, staging only `docs/product-description/` paths. End commit messages with the repo's Co-Authored-By trailer.
- Do not modify anything outside `docs/product-description/`. The rest of beehive is read-only reference material.
- Do not add files outside the README's structure without updating the structure and coverage table to match.
- When a behavior cannot be determined from code and tests, write down what you could determine, put the rest in "Open questions", and move on. Do not guess and do not block.
- Depth bar: `memory/capture.md` is roughly 150–200 lines for a small command. The lifecycle documents will be longer (250–300); observability documents will often be shorter. Completeness matters more than length. Every state, every modifier, every cancel/interrupt row must be accounted for, even if the answer is "no effect".
- If you find that the README's structure is wrong for something you discover (a document that should be split, two that should merge), make the change, update the structure and coverage table, and note why in the commit message.

You are done when the coverage table has no `not started` rows, the consistency pass is complete, and everything is committed.
