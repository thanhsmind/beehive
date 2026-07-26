# 02 — Architecture

## Repository layout (the plugin itself)

```
bee/
  README.md
  docs/                          ← these design docs
  .claude-plugin/plugin.json     ← Claude Code plugin manifest
  .codex-plugin/plugin.json      ← Codex plugin manifest
  packages/bee/                  ← vendored payload: CLI + lib + hooks (single source onboarding copies from)
    hooks/
      hooks.json                   ← shared hook catalog (8 lifecycle events, 9 scripts)
      bee-session-init.mjs         ← SessionStart: status + gates + handoff + patterns + decisions
      bee-prompt-context.mjs       ← UserPromptSubmit: phase/gate reminder (injection-deduped)
      bee-write-guard.mjs          ← PreToolUse: gate guard + reservation guard + privacy/scout + CLI-shape validation
      bee-state-sync.mjs           ← PostToolUse/SubagentStop/Stop: state snapshot persistence
      bee-chain-nudge.mjs          ← SubagentStop: advance the chain after workers/reviewers
      bee-session-close.mjs        ← Stop: warn on mid-phase exit without HANDOFF + decision/capture nudges
    bee.mjs                       ← sole shipped CLI: bee.mjs <group> <verb> over all 9 command groups (D1, shim-retire, decision bbc6bcea; supersedes the 9-shim compat net from harness-integration-adopt/decision 30606de4)
    lib/command-registry.mjs      ← single source of truth for every subcommand's JSON-Schema parameters
    lib/validate-args.mjs         ← validates parsed CLI args against a registry entry's schema; shared by bee.mjs and bee-write-guard.mjs
    scripts/
      onboard_bee.mjs             ← installer: AGENTS.md block, .bee/, bee.mjs + lib
      test_onboard_bee.mjs
  AGENTS.template.md             ← Codex bootstrap block (installed into repo AGENTS.md, BEE:START/END markers)
  skills/
    hive/                        ← bootstrap + routing meta-skill (instructions only; the onboarding engine lives at packages/bee/scripts/ above)
      SKILL.md
      references/routing-and-contracts.md
      references/go-mode.md
    exploring/     SKILL.md + references/{gray-area-probes.md, context-template.md}
    planning/      SKILL.md + references/{planning-reference.md, edge-dimensions.md}
    validating/    SKILL.md + references/validation-reference.md
    swarming/      SKILL.md + references/swarming-reference.md
    executing/     SKILL.md + references/worker-details.md
    reviewing/     SKILL.md + references/reviewing-reference.md
    scribing/      SKILL.md + references/scribing-reference.md
    compounding/   SKILL.md + references/compounding-reference.md
    grooming/      SKILL.md + references/grooming-reference.md
    bee-writing-skills/  SKILL.md + references/{pressure-test-template.md, creation-log-template.md}
```

Eleven skills; additions are decision-gated (a decision record naming the uncovered workflow gap — decision 0002), never casual. Every SKILL.md stays lean (< ~200 lines); depth lives in one `references/` file per skill, never nested deeper than one level (khuym/superpowers rule).

## Target-repo layout (what onboarding installs)

```
<repo>/
  AGENTS.md                      ← contains the BEE:START..BEE:END block (Codex + any AGENTS.md-reading tool)
  .bee/
    onboarding.json              ← onboarding status + managed file versions
    state.json                   ← single runtime state file (phase, gates, active feature, workers)
    HANDOFF.json                 ← pause/resume artifact (exists only while paused)
    reservations.json            ← local file reservations for same-session swarms
    decisions.jsonl              ← event-sourced decisions (decide/supersede/redact)
    backlog.jsonl                ← machine backlog: friction + grooming + review findings (predicted → actual) — NOT the product backlog (that is docs/backlog.md)
    tools.json                   ← capability registry (present/absent, fallback notes)
    config.json                  ← per-repo config: hooks.<name> toggles + commands (setup/start/test/verify — the host project's standard paths, docs/09 item 1)
    logs/hooks.jsonl             ← fail-open hook crash/audit log
    cells/                       ← one JSON file per cell: <feature>-<n>.json
    bin/                         ← bee.mjs, the sole shipped CLI (bee.mjs <group> <verb> over all 9 command groups; D1, shim-retire) — the 9 legacy bee_*.mjs shims are retired
    bin/lib/                     ← shared modules (state, cells, reservations, guards, inject, backlog, commands_detect) used by BOTH helpers and hooks
  docs/
    backlog.md                   ← product backlog: prioritized PBI rows (proposed/in-flight/done), scribing-owned (docs/10) — distinct from .bee/backlog.jsonl
    history/
      <feature>/
        CONTEXT.md               ← locked decisions (source of truth)
        discovery.md             ← research findings
        approach.md              ← chosen path, risks, proof needs
        plan.md | epic-map.md    ← unified plan artifact, enriched in place (see below)
        reports/                 ← worker results, reviewer reports, spike reports (file-based comms)
      learnings/
        critical-patterns.md     ← mandatory pre-planning/pre-execution context
        YYYYMMDD-<slug>.md       ← dated learnings
    specs/
      <area>.md                  ← BA-grade functional spec per long-lived area (state layer)
      system-overview.md         ← cross-area glue: area map, shared entities, global roles, cross-area flows (decision 0003)
      visuals/<area>/            ← one settled snapshot per screen of a UI area, referenced from its spec (decision 0003)
      reading-map.md             ← one line per location: what lives where
    decisions/NNNN-<slug>.md     ← long-form decision records (linked from decisions.jsonl)
  .spikes/<feature>/             ← disposable feasibility proofs
```

**Policy vs operations** (repository-harness): markdown under `docs/` (including `docs/history/`) is human-readable policy and narrative; JSON/JSONL under `.bee/` is the queryable operational record. Helpers keep them consistent; agents never hand-edit JSONL except through helpers.

**Unified plan artifact** (compound-engineering): `docs/history/<feature>/plan.md` is one document across planning's two passes, with frontmatter:

```yaml
artifact_contract: bee-plan/v1
artifact_readiness: requirements-only | implementation-ready
mode: tiny | small | standard | high-risk | spike
```

The shape pass writes it as `requirements-only` and stops at Gate 2; the post-approval prep pass enriches the *same file* to `implementation-ready` and creates the current-slice cells. Downstream skills (validating, swarming, reviewing, compounding) all receive one canonical plan path — no doc-discovery ambiguity, and the readiness field is machine-checkable (`bee.mjs status` reports it).

## The state layer: area specs + reading map (decisions 0001, 0002)

Everything else under `docs/` is **history-shaped** (append-only, dated, feature-sliced) and answers *"how did we get here"*. `docs/specs/` is **state-shaped** and answers *"where are we now"* — the opposite write discipline, on purpose:

| | Log artifacts (`decisions.jsonl`, `docs/history/`, learnings) | State artifacts (`docs/specs/`) |
|---|---|---|
| Answers | How we got here | Where we are |
| Write discipline | Append-only, supersede, never edit | **Overwritten/merged** to match reality |
| Organized by | Feature / date | **Area** (a form, a module — outlives features) |

- **`docs/specs/<area>.md`** — a **BA-grade, technology-agnostic functional spec** of one long-lived area (domain-general: a screen/form, an API, a background job, an integration, a pipeline, a business process), written in the present tense: purpose, entry points & triggers (which link opens which screen; which schedule/event/call runs what), data dictionary (every field/input/output's meaning, every enum value's business meaning, display order for UI, chosen config values with their deciding D-ID), behaviors & operations per user action or system run (what blocks or triggers it, what changes, side effects, what each actor or consumer observes afterwards, failure behavior for operations), an actors & access matrix (human roles and consuming systems), numbered business rules citing active D-IDs, settled edge cases, honest open gaps — and a quarantined `Pointers (implementation)` section as the *only* technology-bound content. It never narrates history ("was", "changed from" are banned — history lives in git and `docs/history/`). Acceptance test is the **rebuild bar** (decision 0002): an agent given only the spec, minus Pointers, can rebuild the same observable behavior on a different stack; a human reads it and understands the area without the code. Template in `bee-scribing`'s reference.
- **`docs/specs/reading-map.md`** — one line per location: `path — what lives here`, optionally pointing at the area's spec. This is the navigation knowledge that otherwise gets re-derived every session.
- **`docs/specs/system-overview.md`** (decision 0003) — the cross-area glue no per-area spec owns: the area map (what areas exist, where each spec lives), shared business entities and their meanings, the global actor/role model stated once, and cross-area flows. Synced by scribing whenever a feature adds/removes an area or changes shared entities, roles, or a cross-area flow.
- **`docs/specs/visuals/<area>/`** (decision 0003) — UI areas only: one settled snapshot per screen, referenced from the spec's `Visuals` section, refreshed at sync when the screen visibly changed. The vibe loop's final artifact is often *seen*; a missing snapshot is an Open Gap, never silent.

The loop that keeps the layer honest:

1. **Write:** `bee-scribing` owns the layer (decision 0002). In the chain it runs directly after execution — a feature may be scribed and closed while unreviewed; independent review is a separate, user-invoked session (decision 565e68d0): capped cells with `behavior_change: true` (plus their `verification_evidence`) are the ready-made delta list — sync means merging those deltas into the touched areas' specs and refreshing reading-map lines, not rewriting docs. On demand it also **captures** settled outcomes of the discuss → build → test → adjust loop — rules agreed, behaviors confirmed, values tuned, whatever the domain (logged as decisions, merged immediately) — and **harvests** first specs for areas built before/outside bee. An explicit user settlement signal ("chốt", "final", "ok ship it") is a **mandatory same-turn capture trigger** (decision 0003); the session-close hook nudges when the newest decision is more recent than every spec update. `bee-compounding` guards the handoff: it verifies scribing ran, and invokes it if not.
2. **Read:** `bee-hive`'s scout contract reads the touched area's spec *before* the area's code, in every lane; the session preamble mentions the state layer when `docs/specs/` exists. Fresh-session reading order: **system overview → touched area's spec (what is) → decisions (why) → history (only for archaeology)**.
3. **Guard:** `bee-grooming`'s entropy score carries a `stale specs` term — an area with `behavior_change` cells capped after its spec's `updated` date (or with such cells and no spec at all) is measured debt, not a hope. The term also reads **git** (decision 0003): files under an area's Pointers / reading-map locations changed after `updated` count as stale even with no cell — vibe edits outside the chain are debt too. The audit additionally reports spec coverage (informational, unscored).

Area naming is kebab-case, chosen at first spec write, stable thereafter; the scribing orchestrator maps cells to areas by the files they touched.

## Skill invocation modes

Every bee skill supports two invocation modes (compound-engineering):

- **Interactive (default):** ask at decision points, using the standard question format.
- **Headless (`mode:headless`):** never block on a question. Apply only unambiguous actions, classify ambiguous cases as deferred, and end with a structured report containing an `Outstanding Questions` section. Terminal output is JSON or structured markdown so an orchestrator (go mode, a pipeline, another skill) can consume it deterministically.

Hard limit: headless mode defers *within-stage* ambiguity only and never self-approves a gate. The one mode that self-approves gates is the opt-in gate-bypass switch (`.bee/config.json` `gate_bypass`, toggled by `bee-bypass-gate`, decision 0010): it auto-approves Gates 1-3 for `tiny`/`small`/`standard` work, never for high-risk/hard-gate work, and never Gate 4 UAT/P1 or secret reads. Off by default; surfaced loudly in the preamble and `bee_status` when on.

## The cell (task unit)

One JSON file per cell in `.bee/cells/`, one schema across planning → execution → trace:

```json
{
  "id": "auth-3",
  "feature": "auth",
  "title": "Wire session middleware into API router",
  "lane": "standard",
  "status": "open | claimed | capped | blocked | dropped",
  "deps": ["auth-1", "auth-2"],
  "decisions": ["D2", "D4"],
  "files": ["src/api/router.ts", "src/auth/middleware.ts"],
  "read_first": ["src/api/router.ts"],
  "action": "Directive prose. Cites decisions (per D2). No code blocks.",
  "must_haves": {
    "truths": ["Unauthenticated /api/* requests return 401"],
    "artifacts": [{"path": "src/auth/middleware.ts", "substantive": "exports authGuard, no TODO stubs"}],
    "key_links": ["router.ts imports and mounts authGuard"],
    "prohibitions": ["No change to public response envelope"]
  },
  "verify": "npm test -- auth",
  "trace": {
    "worker": null, "outcome": null, "files_changed": [],
    "deviations": [], "friction": null, "capped_at": null,
    "behavior_change": false, "verification_evidence": null
  }
}
```

Rules:

- **Capping requires verification — with proof.** `bee.mjs cells cap <id>` refuses unless a passing verify result is recorded; for `small`/`standard`/`high-risk` lanes it additionally refuses without recorded verify *output* (or `verification_evidence`) and a non-empty `files_changed` list (decision 0004 — dogfood showed assertion-capping: `verify_passed: true` with no output and an empty file list). A `behavior_change` cell additionally refuses without a **"before" characterization** in the evidence — `red_failure_evidence` (the prior behavior this change alters: a `git show` of the old state, or a pre-change check that failed), or a `deliberate_exceptions` note for a genuinely new surface (decision 0009 — dogfood showed a `behavior_change` cell capped with empty `red_failure_evidence`, forcing a whole evidence-backfill cell later in review). An assertion is not evidence. Evidence lives in the cell trace, the single source; per-cell reports link it, never re-embed it. One commit per cell, cell id in the commit message.
- **Lane scales strictness.** `tiny` cells may omit `must_haves` and record a one-line trace; `high-risk` cells require full `must_haves`, spike evidence links, and a detailed trace (fields checked mechanically, harness-style tiers).
- **Ready = all deps capped.** `bee.mjs cells ready` lists claimable cells; only the orchestrator assigns them (workers never self-select).
- Optional adapter: when the beads CLI (`br`) is present and the user opts in, cells mirror into beads for graph tooling. Nothing in the chain depends on it.

## The CLI (`bee.mjs`, sole shipped surface)

One Node script (Node 18+, zero npm deps), vendored to `.bee/bin/bee.mjs` by onboarding, mirroring khuym's `.codex/*.mjs` pattern but as a single dispatcher rather than one file per group: `bee.mjs status [--json]`, `bee.mjs cells <verb> ...`, `bee.mjs reservations <verb> ...`, `bee.mjs decisions <verb> ...`, and five more groups below.

| Group | Operations |
|---|---|
| `status` | Read-only scout: onboarding health, state, handoff, gates, active cells, standard commands (warns when unrecorded), entropy quick-read, recommended next reads. `--json` |
| `cells` | `list / ready / show / add / claim / cap / block / drop`; enforces cap-requires-verify and lane field tiers |
| `reservations` | `reserve / release / list / sweep` with agent, cell id, path glob, TTL; conflict → caller must return `[BLOCKED]` |
| `decisions` | `log / supersede / redact / search --recent / active`; write-time secret & injection rejection, datamark on read |
| `state` | `set / gate / worker add|update|remove|clear|prune / scribing-run / start-feature / handoff` |
| `backlog` | `add / counts / rank / badges` — friction + grooming queue |
| `capture` | `add / list / flush / count` — queued scribing-capture stubs |
| `reviews` | `create / list / show / record / candidate add / candidates / status` |
| `feedback` | `digest / count / collect / rank` — dogfood-repo aggregation |

Everything a skill tells an agent to run is `bee.mjs <group> <verb>`, `git`, or the project's own build/test commands.

### History: from 4 shims to the sole CLI (decision 30606de4 → D1, shim-retire/decision bbc6bcea)

`bee.mjs` began (harness-integration-adopt, decision 30606de4, `docs/decisions/0024`, adopted from vantt's PR #1) as an *additive* dispatcher living alongside 4 legacy per-group shim scripts (status, cells, reservations, decisions), then later extended to cover all 9 groups while the shims stayed a compatibility net (DA6 scope-freeze applied only to that original 4). The shim-retire feature (D1, decision bbc6bcea, owner-directed) superseded that compat-net clause: the 9 shims are deleted from `packages/bee/` and, via an onboarding `RETIRED_HELPERS` removal pass (D2), from every host's `.bee/bin/` too. `bee.mjs` is now the sole canonical *and* sole shipped CLI — no skill instruction names a legacy shim directly any more. Durable pieces of the original design still apply:

- **Manifest shape.** `node .bee/bin/bee.mjs --help --json` emits `{schema_version, commands:[{name, invoke, description, parameters, examples, deprecated}]}`, sourced from `command-registry.mjs` — the single source of truth for every subcommand, one entry per command, `parameters` expressed as JSON-Schema (`{type:"object", properties, required}`) in the exact shape Claude Code's own tool/subagent definitions use. The registry's old `helper` field (which shim used to implement a command) was removed together with the shims (D5) — it never appears in the public manifest.
- **Manifest drift tracking (DA4).** A sha256 of `{schema_version, COMMAND_REGISTRY}` is persisted to `.bee/manifest-hash.json` (`{hash, checked_at}`, gitignored — it is rewritten on every `bee.mjs` invocation, including read-only ones). When the current hash differs from the last-persisted one, a `manifest_changed: true` hint is written to **stderr only** — stdout's JSON/text shape never changes, so a machine consumer parsing a command's steady-state output never has to special-case a drift call.
- **CLI-shape enforcement.** `packages/bee/hooks/bee-write-guard.mjs` parses and validates a Bash call shaped like a `bee.mjs` invocation against `command-registry.mjs`'s schema via `validate-args.mjs` before the shell executes it — malformed calls are denied with a structured correction; unrecognized shapes fail open (that classification is the dispatcher's own job, via its Levenshtein nearest-match suggestion, not the guard's). `LEGACY_HELPER_RE` (D3) keeps resolving old `bee_*.mjs` invocation shapes too, as a transition guard for hosts mid-upgrade whose sessions still invoke shim names — its removal is filed as future grooming debt.
- **Drift enforcement (DA5, re-pointed at runtime).** A standing test derives the live verb list from `bee.mjs <group>`'s own "Unknown command … Use: …" contract line (never from grepping source — pinned syntax can be the bug) and asserts a bijection with the registry's `group.*` entries.
- **Deferred.** An MCP server wrapper and a mandatory every-session `--help --json` discovery call are out of scope (foundation-add without demonstrated need) — revisit only if dogfood shows real need.

## Dual-runtime support (Claude Code + Codex)

The workflow contract is runtime-neutral; only two seams differ:

### Seam 1 — Bootstrap (how bee-hive gets loaded)

| Runtime | Mechanism |
|---|---|
| Claude Code | `packages/bee/hooks/bee-session-init.mjs` (SessionStart on startup/resume/clear/compact) injects the routing preamble plus live state: status, gates, HANDOFF surfacing, standard commands + baseline gate, critical-patterns digest, recent decisions (superpowers pattern + claudekit session-init) |
| Codex | The `AGENTS.template.md` block installed into the repo's `AGENTS.md` carries the same instructions (khuym pattern); `bee.mjs status --json` is the first commanded step. Re-read after any compaction. |

Both vectors point at the same skill (`bee-hive`); the preamble content is generated from one shared module (`bin/lib/inject.mjs`) for the hook, the AGENTS.md block, and `bee_status` output, so the runtimes can never drift. The preamble carries, in order: standard commands (host project paths), a Project map section (pointers to `docs/specs/` maps and a specced-area count, or a bootstrap warning when absent) with a PBI counts line when `docs/backlog.md` exists, the critical-patterns digest, and recent decisions.

### Seam 2 — Subagent spawn (how swarming launches workers)

- **Claude Code:** Agent tool, one worker per cell, `run_in_background` for parallel waves; worker results returned as tool results *and* written to `docs/history/<feature>/reports/` (file-based record survives either way).
- **Codex:** Codex subagents (khuym's same-session path) with the parent thread collecting `[DONE]/[BLOCKED]/[HANDOFF]/[NOOP]` messages, plus the same report files.

The spawn *contract* is identical on both: assigned cell id, CONTEXT.md path, global constraints, reservation identity, status-token protocol. `references/swarming-reference.md` holds both mechanics side by side; the SKILL.md body is runtime-agnostic.

### Everything else is shared

Skills, artifacts, cells, gates, helpers, templates: one copy of prose/logic in `skills/`. Each plugin manifest routes to its own **committed rendered tree** instead — `.claude-plugin/plugin.json` → `.claude-plugin/skills/` = `render(skills/, "claude")`, `.codex-plugin/plugin.json` → `.codex-plugin/skills/` = `render(skills/, "codex")` — generated only through the D9 renderer (`packages/bee/scripts/onboard_bee.mjs::renderSkillBytes`, regenerated via `scripts/render_plugin_skill_trees.mjs`); with zero runtime markers today both rendered trees stay byte-identical to `skills/` (cnr2-12).

## Hooks: one automation skeleton, both runtimes + the helper floor underneath

bee ships a coherent **9-script automation skeleton** — session-init, prompt-context (deduped), write-guard (gate + reservation + privacy + CLI-shape in one), state-sync, chain-nudge, session-close, model-guard, tools-logger, codex-subagent-audit — rendered from one shared catalog and wired per runtime: `.codex/hooks.json` carries 8 lifecycle events for Codex, `packages/bee/hooks/claude-hooks.json` carries 7 for Claude Code. It is learned from claudekit's tightly-wired hook system but capped and disciplined: the six core hooks are config-gated in `.bee/config.json` (six toggles, each default-on), every script is fail-open with crash logging, silent on non-onboarded repos, and a thin wrapper over the same `.bee/bin/lib/` modules the CLI helpers use.

The dual-runtime rule: **enforcement lives in the shared helpers first** (cap-requires-verify, reservation conflicts, gate-locked claiming work identically under Codex); the hooks are a second, mechanical belt on both runtimes. Whether an installed Codex CLI actually executes its hooks is unverified, so the helper floor — not the hooks — is what parity rests on. Full design, the Codex parity matrix, and the hook response protocol: [06-runtime-integration.md](06-runtime-integration.md).

Any proposed tenth hook must name which of the nine it replaces — claudekit's 16-hook sprawl remains the documented anti-goal.

## State model

`.bee/state.json` is the single runtime state file:

```json
{
  "schema_version": "1.0",
  "phase": "idle | exploring | planning | validating | swarming | reviewing | scribing | compounding | grooming",
  "feature": "<slug> | null",
  "mode": "tiny | small | standard | high-risk | spike | null",
  "approved_gates": { "context": false, "shape": false, "execution": false, "review": false },
  "workers": [],
  "summary": "one plain-language sentence",
  "next_action": "Invoke bee-<skill>."
}
```

- Every skill updates `phase`, `summary`, `next_action` on completion — the handoff is machine-checkable.
- At ~65% context usage, the active skill writes `.bee/HANDOFF.json` (phase, feature, cells in flight, done/remaining, next action) and pauses. Resume never auto-continues: `bee-hive` surfaces the handoff and waits for the user.
- Gate approvals are recorded here; `bee.mjs status` refuses to report "ready to swarm" unless `execution: true`.

## Security posture (carried from upstreams)

- Decision/learning writes reject secrets and instruction-like content at write time; reads are datamarked so resurfaced text cannot act as instructions (gstack).
- Artifact/transcript content mined by compounding or grooming is treated as untrusted data, never as runtime instructions (khuym dream policy).
- Package installs during execution always checkpoint to the human (gsd slopsquat rule).
