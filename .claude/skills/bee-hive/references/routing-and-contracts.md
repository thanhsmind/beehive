# Routing And Contracts Reference

Open this when the compact bootstrap in `SKILL.md` is not enough.

## Skill Catalog

| # | Skill | One-line description | Load when... |
|---|-------|----------------------|--------------|
| 1 | `bee-hive` | Routing, go mode, gates, red flags. | Starting any session |
| 2 | `bee-exploring` | Identify gray areas, lock decisions into `CONTEXT.md`. | Feature request is vague or new |
| 3 | `bee-planning` | Research, mode gate, approach, unified plan, current-slice cells. | Decisions are locked, or scope is already clear |
| 4 | `bee-validating` | Reality gate, feasibility matrix, spikes, plan-checker, cell review. | Work shape is approved |
| 5 | `bee-swarming` | Launch and tend bounded workers with reservations. | Gate 3 approved |
| 6 | `bee-executing` | Bounded worker loop for one cell. | Spawned by swarming |
| 7 | `bee-reviewing` | Parallel review gate with P1/P2/P3 findings, user-invoked over a scope the user chooses. | User explicitly requests review (decision 565e68d0) — never automatic after a final slice or feature close |
| 8 | `bee-scribing` | BA-grade tech-agnostic area specs: sync, capture, harvest. | Review approved; documenting any area (UI/API/job); a settled outcome must be kept |
| 9 | `bee-compounding` | Capture durable learnings and decisions. | Scribing done or work abandoned |
| 10 | `bee-grooming` | Entropy audit, debt hunt, approved kills. | Cleanup/audit requested; hive idle |
| 11 | `bee-writing-skills` | TDD-for-skills, pressure testing. | Authoring or editing a bee skill's `SKILL.md` content |
| 12 | `bee-evolving` | Run bee's gated self-improvement loop over its own collected feedback digest (cluster → rank → Gate A → Iron Law hand-off → suites green → Gate B → push). Bee repo only, human-invoked, never auto-runs, never pushes on its own. | Human asks bee to evolve/improve itself from its own dogfood friction, in the bee repository |
| 13 | `bee-briefing` | Render the one human-readable implement plan per feature, and the post-Gate-4 walkthrough (consolidator, not planner). | Planning shaped `small`+ work; a feature's implement plan needs (re)generating; a `standard`/`high-risk` feature passed Gate 4 |
| 14 | `bee-bypass-gate` | Toggle opt-in gate-bypass autopilot (`on`/`off`/`status`): auto-approve Gates 1-3 for normal-lane work; high-risk/hard-gate, secrets, UAT always stop. | User wants to run without approving every gate, or to check/turn off bypass |

## First-Skill Routing

| Request type | First skill | Notes |
|---|---|---|
| Vague/new feature | `bee-exploring` | Always start here if gray areas exist |
| Research a topic/library/approach (no feature underway) | `bee-xia` | Standalone brief; suggests exploring or planning as next step |
| (Re)generate or read a feature's implement plan or walkthrough | `bee-briefing` | Consolidates the truth artifacts into `docs/history/<feature>/implement-plan.md`, any phase; writes `walkthrough.md` post-Gate-4 for `standard`/`high-risk`; renders nothing for `tiny`/`spike` |
| Research inside a scoped feature | `bee-planning` | Discovery L2/L3 invokes `bee-xia` in-chain |
| "Just fix this" / small change | `bee-planning` | Route in tiny or small mode |
| Review code | `bee-reviewing` | Load directly — only on an explicit review request (decision 565e68d0); never automatic after execution completes |
| Document a screen/API/job/area; keep a settled outcome (rule agreed, behavior confirmed, value tuned); spec a legacy area | `bee-scribing` | Load directly, any phase — capture never waits for feature close |
| Clean up / tech debt / audit | `bee-grooming` | Load directly |
| Capture learnings | `bee-compounding` | Load directly |
| Author or edit a bee skill (`SKILL.md` content) | `bee-writing-skills` | Load directly |
| Evolve bee from its own dogfood feedback (rank friction, ship a self-improvement) | `bee-evolving` | Load directly; bee repo only (D3), never auto-runs, never pushes without Gate B (D5) |
| `/go` / full pipeline | Go mode | See `go-mode.md` |
| Turn gate-bypass on/off, or check it | `bee-bypass-gate` | Load directly, any phase; toggles `.bee/config.json` `gate_bypass` |
| Resume session | Resume logic | Check `HANDOFF.json` first — kind-aware: pause waits, planned-next adopts only at a fresh-session boundary |

**Surface-scope-earlier check** (runs before routing to exploring): the request contains concrete acceptance criteria AND references to existing patterns → offer "Found clear requirements. Jump straight to planning, or explore alternatives first?" On approval, planning receives a one-paragraph scoping synthesis whose decisions still carry D-IDs.

## Onboarding Protocol

`SKILL.md`'s Onboarding section carries the three steps a session actually runs; this is the full
status contract behind them (moved here by router-cost rc-4 — the router keeps the steps, this file
keeps the detail). Run from the bee source root (the checkout or installed plugin package):

```bash
node packages/bee/scripts/onboard_bee.mjs --repo-root <repo-root> --json
```

Then inspect the result:

- `status: "up_to_date"` → continue.
- `status: "changes_needed"` → summarize the plan to the user, ask for approval, and only then re-run with `--apply`. Never apply silently. Never replace an existing compact prompt or AGENTS.md content outside the BEE markers without explicit consent. Every `--apply` also syncs the bee skill set into the host repo's two managed roots (`<repo>/.claude/skills/bee-*` for Claude Code, `<repo>/.agents/skills/bee-*` for Codex) in the same run — one command keeps vendored helpers and installed skills at the same version. The trees are committed to the host repo, never gitignored. `--global-skills` additionally syncs the legacy global `~/.claude/skills/bee-*` root; without the flag the global root is never read, written, or deleted. The payload's `skills.targets` carries one entry per target root: `{kind: "repo-claude" | "repo-agents" | "global", target_root, mode, blocked, versions, items}`. When the repo being onboarded contains the running script's own skill tree (bee's own repo), the per-project targets sync through the ordinary `applySyncSkill` path (mode `sync`/`fresh`/`noop`) like every other managed target; only the exact source-equals-target root is a `noop`. Each managed root is rendered per runtime (Claude vs Codex) and stamped with a render provenance marker, so a rendered projection is never accepted back as an onboarding source. Global sync there is unchanged.
- `status: "blocked_downgrade"` → the source tree is older than the repo's vendored helpers or a target's installed skills (or a version could not be read — reported as `unknown`, refused the same way). The three-version preflight runs per target; ANY blocked target blocks the whole run (blocked-first), zero mutations happen anywhere, and the top-level `reason`/`versions` surface the blocked target(s). Surface the reported `versions` to the user; only pass `--force-downgrade` on explicit user instruction, and only when every blocked target resolved all three versions numeric — an `unknown` version is never forceable.
- `status: "blocked_no_source"` → no authoritative skill source resolved for this run (identity check failed, or source/target/repo roots overlap). Fail-closed, zero mutations, never forceable with `--force-downgrade` — surface it to the user and resolve the source location before retrying. `versions` is still reported on every blocked return (identity/overlap included), with `unknown` for each of the three (resolution was never attempted) — never `null`.
- **Forced-apply transparency (D2):** whenever a blocked result is forceable, both the plain `--json` dry-run and a refused `--apply` (no `--force-downgrade` yet) carry every target's computed `items` inside `skills.targets` — the full per-target list of `sync_skill`/`remove_skill`/`blocked_*` items a `--force-downgrade` would apply. Show this list to the user BEFORE they authorize the force — it is exactly which skills get overwritten or DELETED, per target; a forced apply then executes precisely that reviewed set.
- Every skill-stage item (`sync_skill`, `remove_skill`, `blocked_symlink`, `blocked_alias`) carries `target` (the target kind above) and `scope: "installed" | "source"`: `installed` means `path` is relative to that target's `target_root`, `source` means `path` is relative to the running script's own skill tree. Legacy plan items (AGENTS.md, `.bee/` runtime files, vendored helpers, etc.) carry no `scope` or `target` at all — they are always repo-relative. Never resolve a skill-stage `path` against `repo_root`.
- A `blocked_symlink` item inside `plan` means one skill directory is a symlink and was skipped (not synced, not deleted) — surface it to the user; it does not block the rest of the apply.
- **Recheck honesty (D5):** after `--apply`, the response's `recheck` field applies blocked-first precedence aggregated across ALL targets — if the skill-sync stage is still blocked post-apply on ANY target (e.g. a residual per-skill symlink/alias block left one skill's version marker un-synced after a forced downgrade), `recheck` reports that blocked status and can never read `"up_to_date"`, even when the rest of the plan is empty. `recheck_skills` carries `{blocked, reason, versions, targets}` whenever this fires.
- `--repo-hooks` only when the user asks for repo-local hook wiring.
- `--claude-md` only when plugin hooks are unavailable and the user wants the CLAUDE.md `@AGENTS.md` import fallback.

If onboarding is not complete, do not continue into the rest of the bee workflow.

## State Bootstrap

On every session start:

1. Confirm onboarding is current via `.bee/onboarding.json` (see Onboarding Protocol above).
2. Run `node .bee/bin/bee.mjs status --json`.
3. If `.bee/HANDOFF.json` exists, check its kind: a pause handoff (or any kindless record) is presented and waited on — do not auto-resume. A planned-next handoff is adopted only at this fresh-session boundary (see Resume Logic below).
4. **Critical patterns (bundleMode, D1):** with a bundle, read `docs/knowledge/index.md`'s `## Critical patterns` section — the live equivalent, generated from the bundle. With no bundle — today's guidance stands, unchanged: read `docs/history/learnings/critical-patterns.md` when present.
5. Surface recent active decisions: `node .bee/bin/bee.mjs decisions active --recent 3`.
6. Check active reservations when workers may be in flight: `node .bee/bin/bee.mjs reservations list --active-only`.

Default `.bee/state.json` shape:

```json
{
  "schema_version": "1.0",
  "phase": "idle",
  "feature": null,
  "mode": null,
  "approved_gates": { "context": false, "shape": false, "execution": false, "review": false },
  "workers": [],
  "summary": "",
  "next_action": "Invoke bee-hive."
}
```

## Resume Logic

If `.bee/HANDOFF.json` exists, read its `kind` (`bee state handoff show --json`; a missing/unknown kind normalizes to `pause`, fail-safe) and branch:

**Pause** (or any kindless record) — unchanged, the original rule:

1. Read `HANDOFF.json` and `.bee/state.json`.
2. Extract phase, feature, mode, cells in flight, done/remaining, and next action.
3. Present the pause point to the user in plain language.
4. Continue only after explicit confirmation. If the user's first message is an unrelated request, still surface the handoff first, then ask which to pursue.

Do not auto-resume. Ever.

**Planned-next** — the previous cell was capped with a green verify and the next cell was already claimed for this handoff. Adoption fires ONLY at a fresh-session boundary (a cleared or newly started session — never a resumed or memory-compacted one, which follows the pause path above):

1. `bee state handoff adopt` transfers the carried claim to this session and clears the handoff record.
2. On success, present the adopted cell, its verify command, and its lane as a start-now instruction — no wait, no confirmation prompt (fresh-session-handoff D1).
3. On a failed adoption (claim lost the race, handoff already cleared), fall back to the pause presentation above — never fabricate a start-now instruction.

## Scout Contract (just-enough reading)

Retrieval triggers, not reading lists. Token budgets by lane:

| Lane | Harness-context budget | Always read | Trigger-based reads |
|---|---|---|---|
| tiny / small | ≈ 2K tokens | bee_status, critical-patterns digest, touched area's state-layer doc — with a bundle: `docs/knowledge/areas/<area>/index.md`; with no bundle: `docs/specs/<area>.md` when present, unchanged | touched-file neighborhood only |
| standard | ≈ 5K tokens | + recent active decisions, CONTEXT.md | touching schema → schema decisions first; touching auth → auth decisions |
| high-risk | ≈ 10K tokens | + full decision search on tags, plan history | + high-risk template, prior spikes in `.bee/spikes/`, related learnings files |

**Reading order per area (state layer, bundleMode; recall surface per decision-propagation D8):**

- **With a bundle** — read `docs/knowledge/areas/<area>/` FIRST: its `index.md` names the area's concepts. Then decision index (the area's section of `docs/decisions/index.md`, complete by construction; drill into events via `decisions search --tag/--scope`) → history. `docs/specs/<area>.md` is named for exactly one job — the read-only compatibility surface: a legacy citation resolves through its pointer stub to the concept that owns the anchor now; never read there for current truth.
- **With no bundle** — today's guidance stands, unchanged: **spec → decision index (the area's section of `docs/decisions/index.md`, complete by construction; drill into events via `decisions search --tag/--scope`) → history**. `docs/specs/reading-map.md` answers "where does X live" before any broad grep.

Do not read `node_modules/`, `dist/`, `build/`, `.git/` internals, `vendor/`, `coverage/` — the scout guard blocks them anyway.

**Orphaned scribing debt (scribing-integrity si-1/si-2/si-3):** when `bee.mjs status --json` reports a non-zero `scribing_debt.orphaned` count (the preamble already prints one loud line for it), surface it and offer it as fix-first knowledge work with the same one-line offer discipline as the capture-queue flush — e.g. "N cell(s) across M feature(s) never got their scribing sync — close the gap now, or after the current task?" One line, user chooses; orphaned scribing debt is never silently ignored. The repair verb is `bee.mjs state scribing-run --feature <feature> --areas "<a,b>" --next-action "<n>"`, which can stamp a non-active feature directly — no need to reactivate it first.

## Chaining Contract

| Skill | Reads | Writes |
|-------|-------|--------|
| hive | onboarding, state, HANDOFF, critical-patterns, decisions | state routing updates only |
| exploring | user conversation, critical-patterns, quick scout | `docs/history/<feature>/CONTEXT.md`, state update |
| planning | CONTEXT.md, critical-patterns, active decisions, bee_status | `approach.md`, `plan.md` (frozen at Gate 2 — approval stamp only after approval; none for `tiny`, opt-in for `small`, D1/D3/D4), current-slice cells via `bee.mjs cells add` |
| briefing | CONTEXT.md, approach.md, frozen plan.md + cells (drift re-render triggers on cell changes only, since the plan can no longer drift after approval — D9), validating reports, state gates (render/refresh); capped cell traces, review findings, UAT (walkthrough) | `docs/history/<feature>/implement-plan.md` (projection; `high-risk` always, `standard` on-demand, `small` optional on request per D4); `docs/history/<feature>/walkthrough.md` (post-Gate-4; `standard`/`high-risk`) |
| validating | CONTEXT.md, discovery, approach, approved shape, cells | reality-gate report, feasibility matrix, spike results in `.bee/spikes/`, repaired cells |
| swarming | validated cells, state, reservations | worker registry in state, HANDOFF at ~65%, wave results |
| executing | assigned cell, CONTEXT.md, reservations | implementation commits (one per cell, cell id in message), verify record, cap, report in `docs/history/<feature>/reports/` |
| reviewing | user-selected immutable scope (a `bee_reviews` session — never triggered by phase or cell completion) | session findings (P1/P2/P3) and the Gate 4 decision recorded on that session, backlog items, `residual-findings.md` fallback |
| scribing | `behavior_change` cells + verification evidence, CONTEXT.md, active decisions, UAT/worker reports, code + user interview (harvest) | with a bundle: `docs/knowledge/areas/<area>/` concepts (BA-grade merge); with no bundle, unchanged: `docs/specs/<area>.md` (BA-grade merge), `docs/specs/reading-map.md`; either way: capture-mode decision log entries, state record |
| compounding | feature history, traces, findings, commits, scribing state record | `docs/history/learnings/YYYYMMDD-<slug>.md`, critical-patterns promotions, decision log, backlog friction, state-layer guard verdict |
| grooming | entropy inputs, backlog, traces, diffs | kill proposals, tiny/small cells, outcome records |

**Recommended-next after execution (SPEC §11.5, decision 565e68d0):** once a feature's execution work is done, the chain hands off to `bee-scribing` then `bee-compounding` directly — `bee_status`'s `recommended_next` and the session preamble report the review-candidate count instead of proposing `bee-reviewing`. The feature closes truthfully `unreviewed`; independent review remains available on request at any later point, over any scope the user names.

Every skill ends with an explicit handoff: `[Outcome]. Invoke bee-<next-skill> skill.`

## Direction of Truth — Projection Rule (D12)

The repo artifacts are the single source of truth for what work exists and its state: **cells** (`.bee/cells/`) for in-flight execution and the **PBI rows** in `docs/backlog.md` for product intent. A session's todo list — `TaskCreate`, `TodoWrite`, and any equivalent scratch checklist — is an **ephemeral projection** of those durable records, never the reverse.

The mapping is one-way: cells and PBI rows generate the session todo list, and no edit to that list ever writes back to a cell or a backlog row. When the two disagree, the repo artifact wins and the session list is regenerated from it. A todo item with no cell or PBI behind it is a projection bug, not a new unit of work — file the cell or the backlog row first, then let the list re-derive. This keeps the durable layer authoritative and the chat/session state disposable.

## Communication Contract

Plain language first:

- practical first, abstract second; scenario-first, not jargon-first
- explain what happens in real life before naming technical properties
- translate decision IDs, invariants, and architecture terms on first use
- prefer "here is what the code does today" over "here is the category of bug"

For plans, findings, blockers, and handoffs, answer in this order:

1. Plain-language summary
2. Current behavior or state
3. Why it matters
4. Concrete scenario
5. Next step

Avoid "violates D5" or "non-monotonic" without immediate explanation.

### The agent runs the machinery, not the user (hive law 10)

Every bee command (`bee.mjs status`, `cells`, `reservations`, `decisions`, onboarding, cell verify
commands) is run by the agent itself the moment the workflow calls for it — never printed for the
user to execute, never "run this and tell me the output". The only human actions in bee are gate
approvals, decision answers, and privacy approvals. `AGENTS.md` critical rule 9 states the same law
in one line and defers here for the full form; `SKILL.md`'s hive law 10 is the router-side pointer.

### Silent Bookkeeping — work language only (decision 1689af1b)

Bee is bookkeeping, not the deliverable. Every mechanical workflow act — claiming or capping cells, status and `state.json` changes, reservations, phase transitions, decision logging, capture stubs — is done silently: run it, never narrate it. Chat speaks the user's work language only: "fixing the login redirect", "done — tests pass", never "capped cell auth-3" or "phase is now swarming".

Bee vocabulary may enter chat in exactly two cases:

1. the user asks about bee itself (state, cells, workflow) — answer plainly, in their language;
2. a gate genuinely needs their decision — and the Gate Presentation Contract already requires that question in work terms, not bee terms.

Litmus: strip every bee term out of a chat message; if nothing the user needs is lost, those terms should not have been there.

### Purpose-first narration (decision 4439bd7e, work-visibility D1)

Silence about mechanics is never silence about purpose. Every perceivable work unit — a phase of real work starting, a worker sent out, a long-running step, a change of direction — opens with one work-language sentence naming what is being done and for what outcome. This is a positive duty, not an exception carved out of Silent Bookkeeping: the bee terms still stay out of chat; what changes is that the work itself is no longer left unnarrated either. Twin litmus: strip the message entirely — if the user loses the thread of what is happening and why, the sentence was owed.

## Gate Presentation Contract

A gate message has two layers, and **only the human layer goes into chat**:

1. **Human layer (the chat message)** — written in the language the user is conversing in, jargon-free, answering four questions in order:
   - **What I'm about to do** — one sentence in the user's terms: what changes *for them*, not the mechanism.
   - **Why it's trustworthy** — the single strongest piece of evidence in plain words ("a dry run rebuilt all 3 pages byte-for-byte identical"), never a checklist.
   - **If it goes wrong** — what breaks for the user and how it would be noticed (loud failure, rollback path).
   - **What you are deciding** — the exact commitment being approved and its boundary ("current slice only").

   Then the fixed gate question verbatim, with the standard options, and a link to the full report.

2. **Machine layer (the linked report)** — the full mechanical material (reality-gate tables, feasibility matrices, plan-checker findings, cell lists) is written to `docs/history/<feature>/reports/` and **linked** from the gate message. It is never pasted into the gate message. It exists for the agent, the audit trail, and grooming — not for the human's eyes at decision time.

Litmus test: **the user must be able to restate what they are approving in their own words.** A gate the user cannot restate is a dead gate — worse than no gate, because it manufactures false confidence. A technical term (BLOCKER count, spike id) may appear in the human layer only with an immediate plain-language gloss.

This contract applies to all four gates, in every mode, including go mode.

### AskUserQuestion — honor the tool's schema (a valid call, every time)

Gates, decisions, and confirm-before-doing prompts are presented with the `AskUserQuestion` tool. If the call violates the tool's schema the harness rejects the **whole** call with **"Invalid tool parameters"** — a recurring, silent waste (the model then retries a valid one). Build the call inside these limits:

- **`header` ≤ 12 characters** — it is a short chip label, NOT the question. Vietnamese/English descriptive headers ("Xử lý external", "Cách hiển thị") overflow instantly — use "Approach", "Scope", "External". **This is the #1 cause of the error.**
- **2–4 options per question** — never 1, never 5+. An "Other" free-text choice is added automatically, so fold overflow there or into a follow-up question.
- **1–4 questions per call** — batch independent questions (up to 4), serialize dependent ones.
- Every option needs both a **`label`** and a **`description`**; put the recommended option first with "(Recommended)" in its label.

A question that "needs" a long header or >4 options is a signal to reshape it — split it, or push detail into the option descriptions — never to exceed the schema.

### Gate bypass mode (opt-in autopilot, decisions 0010 / dcf01d7b)

Off by default. Turned on with the `bee-bypass-gate` skill, which sets `.bee/config.json` `gate_bypass` (persistent per-repo). When on at any level, the agent does **not** stop at a bypassed gate — it takes the RECOMMENDATION option itself and continues. This is the one deliberate exception to "gates are never self-approved"; **headless mode is not** — headless still stops at every gate.

**`gate_bypass` is a level.** `bypassLevel()` (lib/state.mjs) normalizes the config value; the level decides how far bypass reaches. The whole point of the levels above `normal` is that the human said, in advance and explicitly, "when you have a recommended option I will always approve it — do not stop me; the result is what I care about." Honor that literally: at the chosen level, the recommended option IS the approval.

| Level | Config value | Auto-approves | Still stops for the human |
|---|---|---|---|
| `off` | `false` / absent | nothing — every gate stops | every gate (default) |
| `normal` | `true` / `"on"` / `"normal"` | Gates 1-3 for `tiny`/`small`/`standard` non-hard-gate work | high-risk/hard-gate Gates 1-3 · secret reads · Gate 4 UAT/P1 |
| `full` | `"full"` | **all** Gates 1-3 at every lane, high-risk/hard-gate included | secret-file reads · a review P1 finding |
| `total` | `"total"` | **everything** — all Gates 1-3 any lane, secret-file reads, Gate 4 UAT, review P1 findings | **nothing — zero stops** |

Legacy `true` maps to `normal`, so existing repos are unchanged. At **Gate 1, 2, or 3** when the level bypasses that gate:

1. **Safety floor is level-scoped, not absolute.** Under `normal` the floor is exactly as before: a `high-risk` lane or any hard-gate flag (auth · authorization · data loss · audit/security · external provider · validation removal · database migration/schema change) is **NOT** bypassed — present it to the human normally. Under `full` and `total` the high-risk/hard-gate floor is **lifted** — the human lifted it by choosing the level — so those gates auto-approve too.
2. Do not ask. Instead: select the option the RECOMMENDATION favors; set `approved_gates.<gate>` in `.bee/state.json` (same write the human's "yes" would trigger); still write the machine-layer report to `docs/history/<feature>/reports/`; log a one-line audit entry — `node .bee/bin/bee.mjs decisions log --decision "auto-approved Gate N (bypass): <choice>" --rationale "<the recommendation's why>"` — so the approval is never silent; then post a **short chat line** (not a question) — `⚡ auto-approved Gate N (bypass): <what/why in one plain sentence>` — and continue. The human sees what happened and can still interrupt.

**Bypass suppresses approvals, never genuine information-gathering (decision a93994d3).** The point of the levels is to stop the agent asking merely to be *approved* — not to gag a real question. So distinguish two kinds of "question": an **approval** (the agent already has a confident best answer; the human would only rubber-stamp it) is suppressed under `full`/`total` — the agent takes its own answer and continues. An **information** question (the answer turns on a preference or knowledge only the human holds, and the agent cannot resolve it from evidence with a confident default) is still asked, even under `total`. This is where `bee-exploring`'s Socratic step still stops when it must (§4 materiality test + the information-vs-approval refinement): the human asked to keep being consulted for real information, only never for a rubber stamp. Litmus: *"do I already have a confident best answer?"* — yes → proceed; no, and only the human can supply it → ask.

**Gate 4 and secret reads follow the level.** Under `normal` and `full`, Gate 4 is never fully bypassed and bypass never creates a review session (SPEC R8, decision 565e68d0): a review only exists once the user invoked `bee-reviewing`, its UAT items are always presented, and any P1 always stops. Under `total`, a review the user started runs to completion without stopping — UAT items and P1 findings auto-proceed on the recommended resolution. **Secret-file reads** stop for the human under `off`/`normal`/`full`; only `total` auto-proceeds on them (the human accepted that credential contents may enter context/logs unprompted). Bypass still never *creates* a review session on its own at any level.

The mechanical guards do not change: `claimCell` and the write-guard still require `approved_gates.execution: true` — bypass simply means the agent records that approval itself for eligible work instead of waiting for the human. Bypass state is surfaced every session (the preamble and `bee_status` both print a loud level-specific `GATE BYPASS` banner — `NORMAL` / `FULL AUTOPILOT` / `TOTAL AUTOPILOT — ZERO STOPS`) so the active level is never silently in effect.

**The bypass is now mechanized at runtime, not prose-only (GitHub #18, hook-runtime B15/R14).** The rule above is still the assistant's to follow, but it is no longer the *only* thing honoring it: the session-stop checkpoint (`hooks/bee-session-close.mjs` `maybeBypassBlock`) emits a turn-control block that forces continuation when the assistant tries to stop mid-planning/validating at a gate the active level covers and is still pending. It is loop-guarded (blocks once per `sessionId:phase:gate:level`, then degrades to advisory) and excludes exploring/Gate 1 (genuine information questions still stop even under `total`). This closes the "invariant left in prose WILL be bypassed" gap (crit-pattern 20260714): the doctrine test mechanized the prose, this mechanizes the runtime.

### Headless mode (never ask; defer into Outstanding Questions)

With `mode:headless`: never ask blocking questions. Perform onboarding checks and routing only when
unambiguous; defer every ambiguity (stale onboarding needing `--apply`, HANDOFF present, unclear
route) into an `Outstanding Questions` section of a structured terminal report. The four gates are
NEVER self-approved in headless mode — the only mechanism that self-approves gates is the explicit
opt-in gate-bypass switch above, and how far it reaches is its level (`normal` = normal-lane only;
`full` = also high-risk/hard-gate; `total` = everything incl. UAT/secrets). Headless and bypass are
independent: headless without bypass still stops at every gate. Go mode's own headless behaviour is
in `references/go-mode.md` ("Headless Go Mode").

### Delegation contract (fan-out: decide-altitude vs gather-altitude)

The one orchestration pattern bee runs: the session model (the owner's best model) stays the orchestrator in every phase, and mechanical gather/render/mine steps dispatch down-tier as I/O workers that return digests (D1 — replaces the advisor pattern in full, decisions 0013/0015 reversed).

- **Decide-altitude stays on the session model**: gates, Socratic questions, the mode gate, synthesis of findings, accept/reject of worker results, state writes, human conversation.
- **D2 rubric** — a mechanical step delegates down-tier when it needs reading >3 files OR content the main model only needs as a digest, not verbatim; the orchestrator may override either way at dispatch, same spirit as tier-judging (decision 0016). Prose-ruled — no new hook enforces the threshold.
- **D3 lane rule** — the rubric applies in every lane and every phase, tiny/small included. Lane scaling v2's (d02a6bc6) "0 subagents" for tiny/small means zero *ceremony* subagents (reviewers/checkers/panels); I/O workers are exempt. A 1-file tiny fix never crosses the rubric, so it stays inline naturally.
- **Digest contract** — an I/O worker returns paths read, the facts extracted (with file:line anchors), and verbatim quotes only where asked; the orchestrator never re-reads what a digest already answers.
- **Transport unchanged** — anchored `[bee-tier: <tier>]` marker or `model` param (decision 0023), one work-language intent sentence of what the worker will find/build/check plus the model name in the Agent description (decision 4439bd7e, work-visibility D2 — a description that is only a model name or a codename is a red flag), background dispatch where the runtime supports it (decision 0017), P22 dispatch log as the audit trail. I/O workers do **not** register in `bee.mjs state worker add` — the registry stays swarm-cell-scoped (reservations/status are execution concerns); the dispatch log is the audit surface for gathers.
- **Execution worker (AO14, second named class)** — the Delegation contract's other dispatch shape, distinguished from the I/O-offload worker by **authority and state effects**, not by task size. Unlike an I/O worker, an execution worker **does** register in the swarm registry (`bee.mjs state worker add`) and **does** take reservations under its own nickname; it implements exactly one assigned cell (claim → read `read_first` → implement within `files` → verify → cap → release) and returns exactly one status token (`[DONE]`/`[BLOCKED]`/`[HANDOFF]`/`[NOOP]`) — it is authority-bearing, never a digest-only gather. Every `bee-swarming` worker dispatch belongs to this class: full waves in `standard`/`high-risk`, and, since AO14, the single dispatched worker that now carries out `tiny`/`small` cell implementation too (`bee-swarming/SKILL.md`'s Single execution worker section) — never zero of them, even for the lightest lane. **Small-lane serial doctrine (hardening-7):** a `small` lane's 1-3 cells never fan out — one live execution worker at a time, cells processed serially (worker returns and its done-report lands before the next cell is claimed/dispatched); never 2+ live small-lane execution workers for one feature — that shape belongs to `standard`/`high-risk` waves, not this lane. **Parallel criterion:** serial stays the default; cells may run in parallel ONLY when every cell's file set — including regen targets (release manifest, onboarding ledger, plugin mirrors) — is provably disjoint; any shared generated artifact forces serial; in doubt, serial. An independent reviewer or checker (plan-checker, cell reviewer, panel member) is **neither** class: it is a review-class dispatch — read-only, no registry entry, no reservations, no cell of its own — and is never called an "execution worker."
- **cli gather branch (plan 2A-ii, decision 34398e69)** — when `resolveTier(root, 'generation', runtime, {for:'gather'}).type === 'cli'`, a gather dispatch runs the configured command **verbatim** via the shell — nothing appended, ever (W7); the prompt goes in on **stdin**; every path handed to the worker is **absolute** (W9); the run is **read-only** by contract. **Stdout IS the digest**, framed by a delimiter contract: the worker prompt instructs the CLI to emit its digest between `<<<BEE_DIGEST` and `BEE_DIGEST>>>` lines, and the orchestrator extracts only what sits between them — missing delimiters or an empty digest is a **failed run**, surfaced loudly, never accepted as a silent green (fail-open-masking pattern, critical-patterns 20260716-class). No `result.json`, no cell, no reservation, no `bee.mjs state worker add` registration for a gather, same as any other I/O worker. **Known measurement gap, named not solved here:** a Bash-launched gather emits zero `dispatch.jsonl` rows (W-d) — closing that gap is Slice 3's job, not this branch's.

### Goal-check judge tier (D4/D5, self-correcting-loop) — verification, not review

The swarming goal-check (P12, decision 0018) gains a **semantic** judge tier by lane, layered on the existing frozen judge (`bee cells judge`, undeclared-file check) — this is verification of a capped cell, never the user-invoked review session (decision 565e68d0 stands; Gate 4 and the candidates ledger are untouched by every row below).

| Lane | Judge | Model | Verdict handling |
|---|---|---|---|
| `tiny` / `small` | mechanical only (frozen judge) — unchanged | — | — |
| `standard` | ONE checklist judge — a pinned `bee-review` dispatch, review tier, read-only — per capped `behavior_change` cell; checks `must_haves` truths vs the diff, CONTEXT decision citations, task-to-diff alignment | review-tier config | `PASS` → counts; `NEEDS_REVISION` + `automatic` → cell NOT done, re-dispatch with the exact failing checks + a ledger entry; `NEEDS_REVISION` + `authority` → escalate to the user |
| `high-risk` | same checklist judge as `standard` | independence preferred — model differs from the builder's resolved model; if equal, record `model_independence: "same-model"` honestly and the judge still runs | same verdict handling as `standard` |

The judge returns the D5 schema (`judge-verdict/1`), recorded via `bee cells judge-record`; free-prose output is a failed judge run, re-dispatched once, then recorded `unverified`. This table is the single home for the judge-tier rule (D4+Δ6) — every other 565e68d0-adjacent surface (bee-swarming SKILL + reference, bee-hive SKILL both sites, go-mode, AGENTS.md + its template, bee-scribing SKILL) carries only a one-line pointer back here, never a repeated table.

### Verify Ladder (D4, `e54878b1`, superseded by ci-owned-verify D6)

A cell's `verify` field is always its **targeted** suite (seconds), never the full configured `commands.verify` chain. The four-milestone ladder above is retired: no local full run is ever a workflow obligation. The dev loop's own broader check is `commands.test` (the impacted run, `run_verify.mjs --impacted` / `--impacted-from-git`), authored from the impact registry (`impact_registry.mjs --query`) — wave close (once, by the orchestrator, per `bee-swarming/SKILL.md`) and worktree merge both run it instead of the full chain. The full `commands.verify` chain is CI-owned: it runs on the project's own CI cadence (push, nightly, or scheduled — the host workflow decides) and auto-files a `verify-red` issue when red. Session finish also runs `commands.test` (impacted), not the full chain (`AGENTS.md`); the release flow runs the impacted suite locally and then dispatches the CI full run (`gh workflow run CI --ref main`) right after the tag push — a red result there arrives back as the same `verify-red` issue, not a local gate. Judges and reviewers verify against the diff and `must_haves`, never by running the full chain as part of a verdict.


## Question Format

Used at all gates and Socratic steps:

```text
CONTEXT: <one or two sentences of relevant state, plain language>
QUESTION: <one outcome-framed question>
RECOMMENDATION: <the option the evidence favors, and why in one line>
  (a) <option> — <expected outcome>
  (b) <option> — <expected outcome>
  (c) <option> — <expected outcome>
```

One question per message. Never bundle. Never answer your own question.

## File Quick Reference

```text
.bee/
  onboarding.json  state.json  config.json  HANDOFF.json
  reservations.json  decisions.jsonl  backlog.jsonl
  capture-queue.jsonl                                 ← settlement stubs awaiting their flush (0017)
  cells/<id>.json  logs/hooks.jsonl  .inject-cache.json
  bin/  bin/lib/

docs/history/<feature>/
  CONTEXT.md  reports/                                ← always
  plan.md                                              ← frozen at Gate 2 (D1): standard/high-risk
                                                        always; small opt-in (D4); tiny/spike none (D3)
  discovery.md  approach.md  implement-plan.md        ← conditional (decision 0009): separate
                                                        files only for L2+ discovery / high-risk;
                                                        else folded into plan.md sections
  walkthrough.md                                      ← standard/high-risk, post-Gate-4

docs/history/learnings/
  critical-patterns.md  YYYYMMDD-<slug>.md

docs/knowledge/                                       ← state layer when a bundle exists (bundleMode)
  areas/<area>/  index.md  <subject-slug>.md   patterns/  work/<id>/

docs/specs/
  <area>.md  reading-map.md                            ← read-only compat surface when a bundle exists;
                                                          state layer itself when no bundle (unchanged)

.bee/spikes/<feature>/
```

## Helper CLI Quick Reference

`node .bee/bin/bee.mjs <group> <verb>` is the sole canonical and sole shipped
form for all 9 groups (`status`, `cells`, `reservations`, `decisions`, `state`,
`backlog`, `capture`, `reviews`, `feedback`) — one dispatcher, one registry.
The original `bee_*.mjs` shims (one per group — `status`, `cells`,
`reservations`, `decisions`, `state`, `backlog`, `capture`, `reviews`,
`feedback`) are retired (decision bbc6bcea, D1) and no longer ship in
templates or host `.bee/bin` — `LEGACY_HELPER_RE` in the write-guard stays
only as a transition guard for hosts mid-upgrade (D3).

```text
node .bee/bin/bee.mjs status [--json]
node .bee/bin/bee.mjs cells list [--feature F] [--status S] | ready [--feature F] | show --id ID
node .bee/bin/bee.mjs cells add --stdin   # one cell object or a whole-slice JSON array (all-or-nothing); --file cell.json also accepted
node .bee/bin/bee.mjs reservations list [--active-only] | sweep
node .bee/bin/bee.mjs decisions active [--recent N] | search --text T
node .bee/bin/bee.mjs state set --owner <selected pre-mutation phase> | gate | worker add/update/remove/clear/prune | scribing-run | start-feature
node .bee/bin/bee.mjs backlog add | counts | rank | badges
node .bee/bin/bee.mjs capture add | list | flush | count
node .bee/bin/bee.mjs reviews create | list | show | record | candidate add | candidates | status
node .bee/bin/bee.mjs feedback digest | count | collect | rank
```
