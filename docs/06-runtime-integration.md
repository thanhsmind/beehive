# 06 — Runtime Integration: The Automation Skeleton

bee supports **two first-class runtimes**, and neither is a port of the other — they get the same two belts:

- **Hooks on both** (learned from claudekit). The 9 scripts in `hooks/` are rendered from one shared catalog and wired per runtime: `.codex/hooks.json` carries 8 lifecycle events for Codex, `hooks/claude-hooks.json` carries 7 for Claude Code. The workflow chain, gates, reservations, and state are refreshed *mechanically*, not by hoping the model remembers.
- **The helper floor underneath both** (learned from khuym). The same rules are enforced inside the vendored CLI (`bee.mjs`) — identically on either runtime — plus the AGENTS.md block and compact-prompt recovery instructions.
- **One caveat, stated honestly.** Whether an installed Codex CLI actually discovers and executes `.codex/hooks.json` is unverified (see the open question below). Shipping the file is not proof it runs, so on any runtime whose hook execution is unconfirmed the guardrails are self-honored, and the helper floor — never the hooks — is what parity rests on.

The principle that makes dual-runtime cheap: **enforcement lives in the shared helpers first; hooks are a second belt, not the only belt.** `bee.mjs cells cap` refusing to cap an unverified cell works identically on both runtimes. A hook that blocks an unreserved write is a Claude Code bonus on top of the same check the Codex worker runs through the helper.

## What claudekit teaches (and bee adopts)

Reading claudekit's installed skeleton (`.claude/settings.json` + 16 hooks + `lib/`), five patterns are load-bearing:

1. **Config-gated hooks.** Every hook begins with `isHookEnabled('<name>')` against one config file and exits 0 if disabled. The skeleton is one JSON edit away from silent, per-repo, per-hook.
2. **Fail-open crash wrappers.** Every hook wraps its whole body in try/catch, logs the crash to a file, and exits 0. A broken hook never breaks a session.
3. **Injection dedup.** Context-injecting hooks (claudekit's `dev-rules-reminder` on UserPromptSubmit) reserve an "injection scope" and skip when recently injected — the reminder costs tokens once, not on every prompt.
4. **Chain-nudging via SubagentStop matchers.** When a `Plan` agent finishes, `cook-after-plan-reminder` fires and tells the main agent the next stage. The workflow chain is advanced by the harness, not by memory. This is the heart of the "automation skeleton".
5. **State persistence via PostToolUse/Stop.** `session-state.cjs` fires after task-tool calls, on SubagentStop, and on Stop — state files stay fresh as a side effect of working, not as a discipline the model must maintain.

And one anti-lesson bee keeps from the earlier audit: claudekit injects context via env vars and ~16 scripts with overlapping concerns. bee caps the skeleton at **9 thin scripts**, puts shared logic in `lib/` modules (claudekit itself extracts `project-detector.cjs` etc. into `lib/` precisely so another runtime's plugin can reuse it — the exact pattern bee needs), and keeps subagent context inline in spawn prompts, not env magic.

## The bee hook skeleton (both runtimes)

Nine scripts, wired from one shared catalog across 8 Codex lifecycle events (`.codex/hooks.json`) and 7 Claude Code ones (`hooks/claude-hooks.json`). All ship inside the plugin (`hooks/` + `hooks.json`), so no user `settings.json` surgery is required. The six **core** hooks are tabled below; `bee-model-guard`, `bee-tools-logger` and `bee-codex-subagent-audit` were added later and are documented with the features that introduced them. Every script:

- exits 0 silently if the repo has no `.bee/onboarding.json` (plugin enabled ≠ repo onboarded),
- checks `.bee/config.json → hooks.<name>` and exits 0 if disabled — that is the six-toggle set below; the three later scripts are not individually toggleable,
- is wrapped fail-open with crash logging to `.bee/logs/hooks.jsonl`,
- imports its logic from `.bee/bin/lib/` — the same modules the CLI helpers use, so hook behavior and helper behavior cannot diverge.

| # | Hook | Event (matcher) | What it does |
|---|---|---|---|
| 1 | `bee-session-init` | SessionStart (`startup\|resume\|clear\|compact`) | Runs the `bee.mjs status` logic inline and injects: onboarding health, current phase + gate states, `HANDOFF.json` surfacing ("do not auto-resume — present and wait"), `critical-patterns.md` digest, top-3 recent active decisions. This is superpowers' session-start injection + gstack's preamble, done once by the harness. |
| 2 | `bee-prompt-context` | UserPromptSubmit | Injects a one-to-three-line reminder: `phase / mode / next_action / open gate`. **Deduped**: only when state changed since the last injection or after a compaction (claudekit's injection-scope reservation). Costs ~0 on quiet turns. |
| 3 | `bee-write-guard` | PreToolUse (`Edit\|Write\|MultiEdit\|Bash`) | Three checks in one script, first hit wins: **(a) Gate guard** — if `state.json` shows execution not yet approved (Gate 2's execution component — the old standalone Gate 3 folded into it, validation-diet D2) and the target is source code (paths outside `.bee/`, `docs/history/`, `docs/`, `.spikes/`), block with the reason and the gate to ask for. Mechanically enforces "no execution before its approval". **(b) Reservation guard** — during `swarming`, a write to a path not reserved by this agent identity is blocked with a pointer to `bee.mjs reservations` (direct descendant of khuym's `khuym_pre_tool_use.mjs`, which already parses Bash commands for broad write patterns like `sed -i`, `tee`, `rm`). **(c) Privacy/scout guard** — reads of secret globs (`.env*`, `*.pem`, key files) emit a structured `@@BEE_PRIVACY@@` JSON marker that the skill contract turns into an AskUserQuestion approval; reads of `node_modules/`, `dist/`, `.git/` internals are blocked outright (claudekit privacy-block + scout-block, merged). |
| 4 | `bee-state-sync` | PostToolUse (`TaskCreate\|TaskUpdate\|TodoWrite`) + SubagentStop + Stop | Persists a state snapshot: worker registry, cell status counts, last activity. State files stay fresh as a side effect of tool use (claudekit `session-state` pattern). |
| 5 | `bee-chain-nudge` | SubagentStop | When a registered bee worker/reviewer subagent stops, inject the contract's next step: "Worker for cell auth-3 returned — collect its `[STATUS]`, update the cell, release/verify reservations" or, when the last review agent stops, "All reviewers done — synthesize findings, then Gate 4." The chain advances mechanically (claudekit `cook-after-plan-reminder` pattern generalized to the bee chain). |
| 6 | `bee-session-close` | Stop | Warns when the session ends mid-phase with no `HANDOFF.json`, with active reservations, or with claimed-but-uncapped cells — the "you are about to leave the hive door open" check. Also nudges (deduped, warn-only): source files changed with no bee flow and no recent decision logged; and the newest decision more recent than every `docs/specs/*.md` update — something settled was never captured (decision 0003). |

Not hooks (deliberately): subagent context injection (inline in spawn prompts — claudekit's own protocol says "craft prompts explicitly", the env-var channel is its bloat), naming enforcement, kanban rendering, usage-quota caching, statusline. Any future addition must name which of the nine it replaces.

### Hook Response Protocol (skill-side contract)

Hooks can only block or inject text; the *skills* define how the agent responds (claudekit documents this in CLAUDE.md — bee does the same in the hive skill and the AGENTS.md block):

- `@@BEE_PRIVACY@@ … @@END@@` marker → the agent MUST route through AskUserQuestion; on approval, retry with the documented approval prefix. Never work around the block.
- Gate-guard block → the agent MUST NOT retry the write; it surfaces the gate question to the user (Gate 2's execution wording from the workflow doc — the old standalone Gate 3 folded into it).
- Reservation block → the worker returns `[BLOCKED]` with the conflict; the orchestrator fixes reservations or cell scope.

## Codex parity: the helper-enforced skeleton

Codex now loads its own project hooks from `.codex/hooks.json` (SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, SubagentStart, SubagentStop, PreCompact, Stop — 8 events, rendered from the same shared catalog as the Claude Code side), replacing the earlier claim that Codex lacked lifecycle hook support. Helper-level enforcement stays the floor on both runtimes either way — hooks are a second belt, not the only one:

| Automation | Claude Code (hooks) | Codex (helpers + AGENTS.md) |
|---|---|---|
| Session bootstrap & routing | `bee-session-init` injects it | `AGENTS.md` BEE block: "run `.bee/bin/bee status --json` first, re-read after compaction"; `compact_prompt` recovery instructions (khuym pattern) |
| HANDOFF surfacing, never auto-resume | Hook injects the handoff and the wait rule | `bee.mjs status` prints the handoff block first in its output; AGENTS.md rule |
| Phase/gate reminder per prompt | `bee-prompt-context` (deduped) | Skill preambles: every stage skill's first step is "run bee.mjs status, verify the expected gate state" |
| Gate 2 "no execution before its approval" (folds in the old standalone Gate 3) | `bee-write-guard` blocks source writes pre-approval | `bee.mjs cells claim` refuses while `approved_gates.execution: false`; workers only act on claimed cells; AGENTS.md red-flag rule |
| Reservation enforcement | `bee-write-guard` blocks unreserved writes | `bee.mjs reservations reserve` conflict → skill contract mandates `[BLOCKED]`; `BEE_AGENT_NAME` env prefix on write-heavy shell commands (khuym convention) |
| Cap requires verification | (same helper) | `bee.mjs cells cap` refuses without a recorded verify pass — **helper-level, identical on both runtimes** |
| Privacy / scout blocking | `bee-write-guard` check (c) | Guardrail text in AGENTS.md block + hive skill; no mechanical block (accepted gap, documented) |
| State freshness | `bee-state-sync` | Skills update `state.json` at their handoff step (khuym contract); `bee.mjs status` flags staleness (`state.json` phase vs cell reality) |
| Chain advancement after workers finish | `bee-chain-nudge` | The parent thread receives `[DONE]/[BLOCKED]/…` tokens directly (khuym same-session swarm); swarming skill's tend-loop is the nudge |
| End-of-session hygiene | `bee-session-close` | "Session Finish" section of the AGENTS.md block (close/update cells, leave state + HANDOFF consistent, name blockers) |

Codex's project hooks ship a PreToolUse write/privacy guard and a SubagentStop chain-nudge alongside the rest, so the privacy-block and chain-nudging gaps once listed here are mechanism-present (file-shipped) rather than truly absent; whether a given installed Codex actually discovers and trusts each event — as opposed to the file merely being present — is what the capability spike confirms, not something assumed from shipping. Everything gate- and integrity-critical remains helper-enforced first regardless, so behavior stays identical either way.

Codex's `approval_policy` (tool-call permission, `.codex/config.toml`) and bee's `gate_bypass` (workflow-gate auto-approval, `.bee/config.json`) are distinct layers, which is exactly why this table's helper-level enforcement column holds regardless of either setting: gate, reservation, and verification checks live in the shared helpers, not in Codex's permission mode. Codex hook trust is a third, independent layer underneath both — a changed `.codex/hooks.json` may be skipped pending a `/hooks` review no matter how `approval_policy` or `gate_bypass` are configured. See [INSTALL.md](../INSTALL.md) §2 for the recommended `bee-safe`/`bee-autopilot` profiles.

## Render model: one source tree, runtime-conditional blocks (D9)

The runtime-sensitive skills (`bee-hive` and `bee-swarming` — whose "Execute" section carries the old standalone worker skill — with `bee-reviewing` historically in the set) are **one human-edited source tree**, not a fork per runtime. A skill file may fence a runtime-only passage with a strict, full-line HTML-comment marker pair:

```text
<!-- bee:only claude -->
...content that only makes sense on Claude Code (subagent_type, .claude/agents, bee-model-guard)...
<!-- bee:end -->
<!-- bee:only codex -->
...content that only makes sense on Codex (wait_agent/list_agents native tending, read-budget tier enforcement)...
<!-- bee:end -->
```

`render(bytes, runtime)` drops the block not meant for the target runtime and strips every marker line; a file with no markers passes through byte-identical (BOM, CRLF, final-newline state, and arbitrary bytes all preserved — nothing is decoded-and-re-encoded unless a marker line is actually present). The attribution rule is **who must act, not who is mentioned**: a passage is tagged only when it names a mechanism the agent invokes differently per runtime (a spawn call, a tool name, a config path); a sentence that merely mentions "Claude" or "Codex" as a config example or a data fact stays shared, untagged prose. A malformed marker anywhere (unclosed, nested, stray, inside frontmatter, inside a fenced code block) refuses the **entire** render with zero writes — never a partial or best-effort render.

This produces **four rendered-tree roots**, all generated from canonical `skills/` and never hand-edited:

| Root | Runtime | Generated by |
|---|---|---|
| `.claude/skills/` | claude | the onboarding sync path (`onboard_bee.mjs`'s `applySyncSkill`), run via `onboard_bee.mjs --repo-root . --apply` |
| `.agents/skills/` | codex | the same onboarding sync path, codex target |
| `.claude-plugin/skills/` | claude | `scripts/render_plugin_skill_trees.mjs` |
| `.codex-plugin/skills/` | codex | `scripts/render_plugin_skill_trees.mjs` |

Each rendered root is stamped with a `.bee-render.json` provenance sidecar (`{schema:"bee-render/1", target_runtime}`). `source-identity.mjs` classifies any skills root carrying that sidecar as a **rendered projection** and refuses it as an onboarding source for **any** target, own-runtime included — a projection can never become someone else's (or its own) source of truth, closing the loop that would otherwise let a stripped copy silently re-seed itself. Canonical `skills/` is the only valid onboarding source.

Today only the 5 adapter-split skills carry any marker at all; the other 10 workflow-semantic skills and the always-loaded doctrine layer (`AGENTS.md`, `AGENTS.block.md`) stay deliberately unsplit — identical prose on both runtimes — because their content genuinely doesn't differ by runtime. When a cross-runtime contrast is useful for a human reading the docs but isn't itself part of either runtime's operating instructions, it belongs here, in this file, rather than duplicated (untagged) into both projections or shoehorned into a marker block that neither runtime actually needs to act on.

## Shared `lib/` — one brain, two belts

```
.bee/bin/
  bee.mjs            ← sole shipped CLI, all 9 command groups
  lib/
    state.mjs          ← read/write state.json, gate checks, staleness detection
    cells.mjs          ← cell schema, cap-requires-verify, lane tiers, ready-set
    reservations.mjs   ← reserve/release/conflict/sweep (khuym_reservations lineage)
    guards.mjs         ← secret globs, scout-block dirs, gate-guard path rules
    inject.mjs         ← context digests (status, patterns, decisions), injection dedup
plugin hooks/          ← 6 thin wrappers: parse stdin payload → call lib → print/exit
```

Hooks are wrappers around `lib/`; CLI helpers are wrappers around the same `lib/`. When a rule changes (say, a new secret glob), both runtimes pick it up from one file. This is claudekit's `lib/` extraction pattern applied deliberately instead of retroactively.

## Onboarding responsibilities (one script, both runtimes)

`onboard_bee.mjs` (with `--apply` after approval):

1. Installs/updates the `AGENTS.md` BEE block (BEE:START/END markers) — bootstraps Codex and any AGENTS.md-reading tool.
2. Vendors `.bee/bin/bee.mjs` + `lib/` into the repo, removes any retired `bee_*.mjs` shims found there (`RETIRED_HELPERS` pass, D2), writes `.bee/` runtime files and `config.json` (all six hooks default-on, each toggleable).
3. Claude Code hooks need **no repo install** — they ship with the plugin and self-arm when `.bee/onboarding.json` appears. `--repo-hooks` exists as a fallback that writes them into `.claude/settings.json` for environments that don't load plugin hooks.
4. Verifies drift on later runs: managed block version, helper versions, config keys (khuym's `onboarding.json` managed-versions pattern).

The session-start preamble content is generated from one source (`inject.mjs`) for all three consumers — the plugin hook, the AGENTS.md block text, and `bee.mjs status` output — so the two runtimes can never drift apart in what they tell the agent. (gstack's docs-from-code rule, applied to bee's own bootstrap.)

## Tier 3: the repo-native playbook (any agent, no plugin)

repository-harness proves a distribution model skills cannot match: because its knowledge lives *in the repo* (AGENTS.md, intake docs, durable records), **every** agent that enters the repo is governed — regardless of runtime, plugin installation, or whether any skill triggers. Skill suites are activation-dependent: no plugin, or a missed description match, and the agent is blind.

bee is already half repo-native (helpers enforce mechanically for any agent; the AGENTS block bootstraps). The gap is workflow knowledge: how to actually run the stages lives only in SKILL.md files on the plugin side. Close it with a third degradation tier:

1. `onboard_bee.mjs` additionally installs **`.bee/PLAYBOOK.md`** (~150 lines, hard cap): the compressed chain — per-stage minimum checklists, the three gates verbatim, the risk-flag mode gate, key report formats (reality gate, status tokens), and the helper command surface. Enough for a plugin-less agent (Cursor, Copilot, Gemini CLI…) to run the chain correctly at a basic level.
2. **Generated, not hand-written**: the playbook is produced from the SKILL.md sources at plugin build time (gstack's docs-from-code rule) — one source, two forms: full skills (lazy-loaded, persuasion-hardened) and compressed playbook (always-on, procedural only). Anti-rationalization content stays in skills; the playbook carries procedure, the helpers carry enforcement.
3. The AGENTS block gains one routing line: *"If bee skills are not available in this runtime, follow `.bee/PLAYBOOK.md`."*
4. `bee.mjs status`'s `recommended_next` points at the playbook section for the current phase — the repo navigates any agent, independent of skill triggering.

Degradation ladder, complete: **skills** (Claude Code/Codex with plugin) → **playbook** (any AGENTS.md-reading agent) → **helpers** (mechanical enforcement for everyone, including agents that read nothing). Scheduled with the phase-4 docs-from-code work in [05-roadmap.md](05-roadmap.md).

## Testing the skeleton

- Each hook gets a fixture test: feed a recorded stdin payload, assert block/inject/silence (khuym's `test_onboard_khuym.mjs` style, no framework).
- One parity test asserts that every rule in `guards.mjs`/`cells.mjs` is exercised by *both* a hook test and a helper test — the two-belt guarantee.
- Pressure scenarios for the skill-side contracts (e.g., agent tries to work around a privacy block) live with the hive skill per the Iron Law.
