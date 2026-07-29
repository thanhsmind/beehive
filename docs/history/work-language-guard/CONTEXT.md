# Work-Language Guard — Context

**Feature slug:** work-language-guard
**Date:** 2026-07-29
**Exploring session:** complete
**Scope:** Standard
**Domain types:** SEE (what the person being served reads in chat), RUN (an end-of-turn hook)

## Feature Boundary

Make the communication contract bite instead of drift: rewrite the per-step worked-example
catalog so its examples obey the law they illustrate, and add an end-of-turn guard that reads
the turn's own assistant output and blocks once when internal terms leaked into user-facing
prose or a turn that performed perceivable steps emitted no per-step line. It ends at
observation of the assistant's own text; it never inspects user text, never rewrites a message
itself, and never blocks more than once for the same turn.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | On a violation the guard blocks the stop **at most once per turn**, feeding the reason back to the model so it rewrites; it never blocks until clean, and it is never warn-only. | User-answered, the one question asked. Warn-only reproduces today's failure (advisory text the agent skims past); block-until-clean risks a live session looping on a false positive. Once-per-turn is the only shape that both bites and terminates. Implemented with `lib/inject.mjs` `shouldInject`/`markInjected`, the same dedup `bee-session-close.mjs:413-421` already uses for its bypass net. |
| D2 | The leaking worked-example catalog is **rewritten into work language, not deleted**. Every row in `skills/bee-hive/references/routing-and-contracts.md` "Progress ticks" (lines 259-285) becomes a line the person being served could actually read; the catalog itself survives. | The catalog is useful — it teaches the fixed shape. Its defect is that it teaches the shape with forbidden content (`✓ cell capped — vt-1`, `✓ cells created: 3 cells — 1 wave, disjoint files`, `✓ slice <n> closed`). An example is what gets copied; a rule that its own examples contradict loses to the examples. Deleting would leave the rule with no worked form at all. |
| D3 | Detection reads the **session transcript**, never `.bee/logs/tools.jsonl`. The guard resolves the transcript via `lib/perf.mjs` `resolveTranscript`, reads it with `lib/fsutil.mjs` `readJsonl`, and works from the trailing `type === "assistant"` entries. | `tools.jsonl` records `{ts, tool_name, agent_id, agent_type}` only — no session id and no turn boundary (decision f1ca79b9), so "which steps ran this turn" could only be guessed from a timestamp window. The transcript carries both the assistant's text blocks and that same turn's `tool_use` blocks, making both halves of the check exact instead of inferred. |
| D4 | Two violation classes, both required: **(a) internal-term leak** — a term naming bee's machinery appears in user-facing prose; **(b) silent step** — the turn's assistant entries contain `tool_use` blocks for perceivable steps yet the turn's text carries zero glyph lines. | These are the two failures the person actually reported. (b) is only checkable because of D3: the perceivable steps and the emitted lines are read from the same turn's entries. |
| D5 | False positives are prevented by **construction, not tuning**: the guard matches an explicit curated term list plus the cell-id shape, and only inside user-facing prose — fenced code blocks, inline code spans, file paths, command strings, and quoted tool output are excluded before matching. No fuzzy or heuristic matching, no similarity scoring. | The words are ordinary English elsewhere ("spreadsheet cell", "swim lane", "gate"). A guard that blocks a live turn must be wrong approximately never, and an explicit list is auditable where a heuristic is not. Quoting bee's own machinery inside a code block — as this very document does — must stay legal. |
| D6 | The guard is the **first code that reads `quiet`**. Under `quiet: true` the silent-step half (D4b) is skipped; the internal-term half (D4a) is never skipped by any setting, at any bypass level. | `quiet` exists today only as prose in AGENTS.md rule 17 and `routing-and-contracts.md:242` — no `.mjs` reads it. A guard demanding per-step lines from someone who asked for quiet would punish an honoured preference. Asking for less noise is not asking to be addressed in machine terms, so the term half stands regardless. |
| D7 | It **ships to every host repo**, registered in `catalog.mjs` with the three projections regenerated (`hooks/hooks.json`, `hooks/claude-hooks.json`, `.codex/hooks.json`), and enabled by default. Blocking is emitted **only** when `ctx.event === "Stop"`. Codex transcript parsing is out of scope this feature: on a runtime whose transcript cannot be resolved the guard is silent. | The problem is experienced wherever bee speaks, not only in its own checkout. `adapter.mjs:404-409` restricts `encodeBlock` to Stop; blocking elsewhere is documented misuse. Codex's transcript shape is unverified, and a guard that blocks on a shape it guessed is worse than one that stays quiet. |
| D8 | The guard **fails open, always**. A missing, unreadable, or unparseable transcript, a `hooks.<name>: false` toggle, or any internal error produces silence — never a block, never a crash. | Every existing bee hook is fail-open (`adapter.mjs` `logCrash`, exit code always 0). A communication guard is not worth a session that cannot end its turn. |

### Agent's Discretion

The exact contents of the curated term list (D5), the regex for the cell-id shape, the wording
of the block reason fed back to the model, and the dedup hash composition (D1) are the agent's,
within the constraint that the list is explicit and enumerated in one named place rather than
scattered across call sites.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Internal term | A word whose referent is bee's own machinery rather than the work the person asked for — the unit of assigned work, the group of them run together, the parallel track, the approval checkpoint, the act of closing a unit, and the identifiers of any of these. Named in prose to the person being served, it is a violation; named inside a code block, a path, or quoted output, it is not (D5). |
| Perceivable step | A step of a run the person could notice happening — evidenced in this feature by the turn's own `tool_use` blocks for state mutations, worker dispatch, and verification runs. Not every tool call: a read is not perceivable, a state write is. |
| User-facing prose | The assistant's text blocks in the turn, minus fenced code, inline code spans, paths, command strings, and quoted tool output (D5). |

## Specific Ideas And References

- The person quoted their own failure case verbatim: *"Ba cell đã ghi. tci-1 và tci-2 độc lập — chạy song song."* This line is the acceptance example — the guard must catch it, and no rewritten catalog row may resemble it.
- They also reported the per-step lines being "not remembered and understood, so usually missed" — which is why D4 carries the silent-step half rather than shipping term-matching alone.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee/hooks/adapter.mjs:306` — `readHookContext(hookName, {argv})`, the sole hook entry point; returns `{payload, cwd, root, storeRoot, controlRoot, worktreeResolution, source, event, gaps}` and never throws.
- `packages/bee/hooks/adapter.mjs:410-412` — `encodeBlock(reason)` → `{decision:"block", reason}`; the exact mechanism D1 needs, already written.
- `packages/bee/lib/inject.mjs` — `shouldInject(root, key, hash)` / `markInjected(root, key, hash)`, the per-session dedup cache that makes "once" mean once.
- `packages/bee/lib/perf.mjs:55` — `resolveTranscript(projectsRoot, projectPath, {sessionId, transcriptPath})`.
- `packages/bee/lib/fsutil.mjs:112` — `readJsonl(file)`, skips corrupt lines silently (fail-open by construction).
- `packages/bee/hooks/bee-session-init.mjs:154-164` — already persists the real hook payload's `transcript_path` onto the session record, so the transcript is resolvable without guessing.

### Established Patterns

- Loop-guarded Stop block — `packages/bee/hooks/bee-session-close.mjs:385-447`, gated on `ctx.event === "Stop"`, deduped by a `session:phase:gate:level` hash, falling through to the advisory path on a repeat. D1 follows this shape exactly.
- Assistant-entry filtering — `packages/bee/lib/perf.mjs:137,209`, `if (!ev || ev.type !== 'assistant') continue;` then `ev.message.content` as a block array.
- Hook toggle — `stateLib.hookEnabled(root, HOOK_NAME)` called inside every wrapper's `main()`.

### Integration Points

- `packages/bee/hooks/catalog.mjs:257-267` — the `Stop` group the new wrapper joins, alongside `bee-state-sync.mjs` and `bee-session-close.mjs`.
- `packages/bee/hooks/hooks.json`, `packages/bee/hooks/claude-hooks.json`, `.codex/hooks.json` — checked-in projections that must byte-match `renderProjection()`; the drift rows in `test_hook_contracts.mjs:853-932` fail otherwise.
- `packages/bee/hooks/test_hook_contracts.mjs:225-234` — the hand-maintained `WRAPPERS` array for adversarial-input coverage; nothing forces a new hook into it, so it is added deliberately.
- `skills/bee-hive/references/routing-and-contracts.md:259-285` — the catalog D2 rewrites.
- `docs/knowledge/areas/doctrine-layer/the-communication-contract.md:160-168` — the "Open Gaps" section this feature closes; it must be updated, and its warning that reachability is not obedience must survive in amended form.

## Canonical References

- `docs/knowledge/areas/doctrine-layer/the-communication-contract.md` — the 15 business rules this guard enforces; rules 11-15 are the per-step contract.
- `packages/bee/AGENTS.block.md` critical rules 10 and 17 — the operative always-loaded text, rendered into every host's `AGENTS.md`.
- `scripts/tests/test_always_loaded_rules.mjs:11-13` — the existing suite's explicit disclaimer that it proves reachability and nothing about obedience; the guard is the missing half, and that disclaimer stays true.

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

- [ ] The exact schema of an assistant text block in a real transcript (`{type:'text', text:...}` is implied by the `tool_use` sibling at `perf.mjs:211` but unconfirmed) — answered by reading one real transcript file before the extractor is written.
- [ ] Which `tool_use` names count as perceivable steps under D4b — answered by listing the mutating bee CLI invocations and the worker-dispatch tool names actually seen in transcripts.
- [ ] Whether `quiet` (D6) belongs in `.bee/config.json` or `.bee/config.local.json` given the machine-local preference at `lib/state.mjs:1962-1993` — answered by how the other reader-preference keys are stored.

## Deferred Ideas

- Codex transcript coverage for the guard — deferred by D7; the transcript shape is unverified and a guess that blocks is worse than silence.
- Enforcing the remaining communication rules (concrete estimates, runnable win, cause+fix+actor on errors, one-question-at-a-time) — deferred: they need judgement, not matching, and the two reported failures are matchable today.
- A check that the worked-example catalog itself stays clean over time, so D2's rewrite cannot silently rot back — deferred; worth filing once the term list of D5 exists and can be reused as the checker.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
