---
artifact_contract: bee-research/v1
topic: pi-harness-support
depth: standard
date: 2026-08-29
---

## Bottom Line

- Recommendation (ladder rung): adapt-upstream — write a Pi **extension** (one TypeScript file), NOT Claude-Code-style JSON hooks. Pi has no JSON hook config at all; its only integration surface is the extension API. bee already owns the exact pattern: `.opencode/plugins/bee-guard.ts` is a TS plugin that execs `bee hook <rule>` and translates verdicts — the Pi belt is that same file re-targeted at Pi's event names.
- Why this is the lightest credible path: bee's hook brain (helpers `bee hook <rule>`) stays untouched — the architecture is explicitly "helpers stay the FIRST belt on every runtime" (`packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs:1-80`). Only a thin translation file plus catalog/onboard wiring is new. Pi auto-discovers repo-local extensions at `<repo>/.pi/extensions/`, so bee can ship the file per-project with zero global install and zero user config.
- Why the next-best rung lost: build-from-scratch (a shim binary or wrapper CLI) loses because Pi's extension API already gives in-process blocking (`tool_call` → `{ block: true, reason }`) and system-prompt injection — a wrapper would re-implement what the host hands out. Plain reuse loses because no Pi belt exists in bee today (`rg` for a pi runtime in `packages/bee-rs` finds nothing).
- Confidence (0–100%): 85% — every needed event is confirmed in Pi 0.84.3's own docs and a working local extension; the open 15% is bee-side wiring detail (catalog rows, onboard copy step, parity-test extension).
- Suggested next step: bee-shaping for a `pi-support` feature (new runtime belt = same class as opencode-support, which ran as its own feature).
- Model routing (user direction, 2026-08-29, decision touches 7f9c8518): unified in the EXISTING homes — a `models.pi` runtime block with the same open role set as `models.claude`, every slot `kind: herding` → a `herding.agents` entry whose argv carries the pi model + thinking (`pi -a --model <provider>/<model>:<thinking>`; entries `pi-agy-flash-3.7`, `pi-opencode-free` already show the shape). Triad mapping: lead = the orchestrating session itself (cockpit `control_command`, never a dispatch slot); supervisor = the existing `supervisor` role; peers = execution roles (code/generation/test/docs); gathers = read/extraction; plus review/advisor. Unconfigured roles keep refusing by name — no silent fallback (paseo-pi-team's model-routing doc verified three silent-fallback failure modes; strict routing is the survivable shape). Today `dispatch prepare --runtime` accepts only `codex, claude` (guard-verified) — adding the `pi` runtime entry is part of the feature.
- Settled table (user, 2026-08-29): heavy roles (code, test, docs, review) stay on **Claude Opus**, advisor on **Fable** — as herding slots whose agents run the claude CLI (`claude --model opus` / `--model fable`, same shape as the existing `claude-sonnet` entry); cheap roles (read, extraction, generation, supervisor) default to the `agy-flash` herding agent. A pi-runtime dispatch landing on a claude-CLI pane is consistent: "every slot is herding" constrains the transport, not the model vendor.
- Locked constraint (user, 2026-08-29): Pi has **no native subagent surface** — no Task/Agent-style tool. On a Pi session, every worker dispatch routes through the herding transport (`bee herding run` / herdr panes); `dispatch prepare --runtime pi` must never emit an Agent-tool payload. Consequence: herding transport quality (result payloads, digest return) is a hard dependency of pi-support, not an optional cockpit — the digest-loss friction below is in scope.

  Executed since (feature pi-support, 2026-08-29): the dispatch door on
  runtime pi refuses every slot resolution that isn't `kind: herding` —
  plain-string, Native, Cli, Budget, and escalated/ceiling roles all refuse
  by name, no silent fallback (**8650ca7b**). The digest-loss friction
  recorded below got its fix scoped: the worker-result mailbox transport
  splits into its own follow-up feature, pi-result-mailbox — this belt
  ships first on the existing `bee herding run` result contract, friction
  standing until the mailbox lands (**4d7438ec**). Per **29dc2003**
  (impact door for pi-support's 8 flagged citations, this brief included):
  pi-support touched 7f9c8518 and 4a6e38be by extending them, retired
  neither, so every citation on this page stays true as written.

## Repo Snapshot

- Repo type / primary languages / runtimes: bee — Rust CLI (`packages/bee-rs`), plus TS plugin shims per harness.
- Harness belts today (Local): **claude** — generated JSON hooks in `.claude/settings.json` (SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, Stop → `bee hook <rule>`); **codex** — file-shipped projection whose PreToolUse execs `bee hook <rule>`; **opencode** — `.opencode/plugins/bee-guard.ts` TS plugin. Catalog of record: `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs`, rendered manifests `packages/bee/hooks/claude-hooks.json` + `hooks.json`; parity test `three_belt_parity_every_blocking_rule_hits_helper_claude_codex_and_opencode`.
- Failure policy (Local, opencode_plugin_contracts.rs): BLOCKING surfaces (write-guard, model-guard) fail CLOSED (deny/crash/missing binary → throw); ADVISORY surfaces fail OPEN. The Pi belt must keep this split.
- Target harness: Pi 0.84.3, installed via mise (`~/.local/share/mise/installs/pi/0.84.3`), extensions live at `~/.pi/agent/extensions/` (global) and `<cwd>/.pi/extensions/` (project — `CONFIG_DIR_NAME = ".pi"`, `localExtDir = path.join(resolvedCwd, CONFIG_DIR_NAME, "extensions")` in the pi binary).

## Question & Assumptions

- What was asked: make bee run under the Pi harness (Paseo + Pi, as paseo-pi-team does) as well as it runs under Claude Code; does that need Claude-Code-style hook config, or another mechanism?
- What success appears to mean: bee's five hook surfaces (session-init, prompt-context, write-guard, model-guard, state-sync/activity, turn-end waiting mark) all fire in a Pi session with the same verdicts and failure policies as the Claude belt.
- Assumptions still needing confirmation: whether `bee onboard` / `bee dev install-support` should also install prompt-context injection or leave the preamble to `before_agent_start` only; whether Paseo-spawned Pi agents inherit the repo cwd so project-local discovery fires (Inference: yes — Paseo workspaces set cwd to the repo).

## Findings

### Local

- bee's three-belt architecture is designed for exactly this addition: guard rules are derived from one catalog (`hook_manifests.rs`), each belt is a thin translation, and the parity test derives its row set from the catalog, "never a hand-authored list" (`opencode_plugin_contracts.rs`, citing `docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-truth-it-never-compares-two-hand-lists.md`).
- Plugin files are shipped from the checkout's own tree, not a template: onboard step `copy_opencode_plugin` copies from `Engine::opencode_plugin_dir` (`packages/bee-rs/crates/bee/src/onboard/apply.rs:362-369`). A Pi belt adds the analogous `copy_pi_extension` step and a checked-in `.pi/extensions/bee-guard.ts`.
- The Claude belt's full surface map (`.claude/settings.json`, Local): SessionStart→`hook session-init`, UserPromptSubmit→`hook prompt-context`+`hook activity`, PreToolUse(Edit|Write|Bash|…)→`hook write-guard`, PreToolUse(Agent|Task)→`hook model-guard`, PostToolUse(TodoWrite…)→`hook state-sync`, Stop→turn-end waiting mark.

### Upstream

- **paseo-pi-team** (`/home/thanhsmind/Projects/refs/slp/paseo-pi-team`, commit `94ead115`, 2026-08-22): answers the user's question directly — they use **no Claude-Code-style hooks anywhere**. The whole harness integration is one Pi extension, `extensions/paseo-team-policy.ts`, installed by `scripts/install.sh:62` (`cp` into `~/.pi/agent/extensions/`), activated by an env var (`PASEO_PI_ROLE`), passive when unset.
- Pattern worth modeling from it (`extensions/paseo-team-policy.ts`):
  - `pi.on("before_agent_start")` returns `{ systemPrompt: event.systemPrompt + roleprompt }` — prompt injection per turn (lines 1229-1246); it also reads `event.prompt` to re-derive per-turn authority, never sticky across turns.
  - `pi.on("tool_call")` returns `{ block: true, reason }` — the deny surface, fail-closed on unclassifiable input (lines 1248-1328). Direct analog of bee's PreToolUse write-guard/model-guard.
  - `pi.registerTool` for custom typed tools that exec a support script via `node` subprocess (lines 132-166) — the same exec-a-helper shape as bee-guard.ts execing `bee hook`.
  - Env-var activation + "safe to install globally, passive when unset" — the right shape for a global install; bee's repo-local discovery makes even that unnecessary.
- Pi extension API (`@earendil-works/pi-coding-agent`), confirmed against a second working extension on this machine (`~/.pi/agent/extensions/herdr-agent-state.ts`): `session_start`, `agent_start`, `agent_settled`, ctx with `sessionManager`, `mode`.

### Docs

- Pi 0.84.3's own shipped docs (`~/.local/share/mise/installs/pi/0.84.3/pi/docs/extensions.md`, version-matched) confirm every surface bee needs, and the mapping is total:

| bee hook rule | Claude Code surface | Pi surface |
|---|---|---|
| session-init | SessionStart | `session_start` (has `reason: new/resume/fork`) + `before_agent_start` systemPrompt chain |
| prompt-context | UserPromptSubmit | `before_agent_start` (`event.prompt` readable, systemPrompt appendable) or `input` |
| write-guard / model-guard (BLOCKING) | PreToolUse | `tool_call` → `{ block: true, reason?, terminate? }` (docs line 774); handlers chain, later handlers see earlier mutations |
| state-sync / activity | PostToolUse | `tool_result` (can mutate results), `tool_execution_start/end` |
| turn-end waiting mark | Stop | `agent_settled` — docs explicitly: "Use `agent_settled` for status integrations that need to know Pi will not continue running automatically" (line 569); `turn_end` also exists |

- Extension locations (docs "Extension Locations" + binary strings): global `~/.pi/agent/extensions/`, project `<cwd>/.pi/extensions/`, plus `pi install <source>` and `-e <path>`. `/reload` hot-reloads.
- Node 24+ runs erasable TS natively — same zero-build property bee already relies on for `.opencode/plugins/bee-guard.ts`.

### Inference

- Paseo needs no bee-specific work: Paseo spawns Pi as a provider (`pi-lead/<provider>/<model>` in paseo-pi-team) and Pi loads project extensions from the workspace cwd. bee integrates with **Pi**, and Paseo inherits it. A Paseo *plugin* (React Native panels/RPCs, per the local paseo-plugin skill) is the wrong layer for guards.
- The blocking-belt failure policy ports cleanly: in `tool_call`, exec `.bee/bin/bee bee hook write-guard`; exit 2 → `{block: true, reason}`; crash/missing binary → block (fail closed); advisory rules swallow errors (fail open). Same policy table as `bee-guard.ts:14-32`.

## Risks, Unknowns, Follow-Ups

- The parity test derives rows from the catalog; adding a fourth belt means extending `hook_manifests.rs` rows and the three-belt test to four — mechanical but it is the proof obligation for the feature.
- `tool_call` blocking semantics under Pi's parallel tool execution: sibling preflights are sequential (docs line 766), so guard ordering is safe, but this deserves a fixture test like the opencode stub suite.
- Whether session-init's full preamble (the big status block) belongs in Pi's systemPrompt every turn or once per `session_start` — token-cost question for shaping.
- Friction, recorded: two `bee herding run` gather dispatches from a non-herdr session returned only the job-summary JSON; the workers' digests were lost with their panes. Content came from direct reads instead.

## Source Pack

- Local files read: `.claude/settings.json`; `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs`; `packages/bee-rs/crates/bee/src/onboard/apply.rs`; `~/.pi/agent/extensions/herdr-agent-state.ts`; `~/.pi/settings.json`.
- Upstream repos checked: `/home/thanhsmind/Projects/refs/slp/paseo-pi-team` @ `94ead115960df493409d281cecbbbf02b6ce8bf0` (README.md, extensions/paseo-team-policy.ts, scripts/install.sh).
- Docs pages checked: Pi 0.84.3 shipped `docs/extensions.md` (version-matched); pi binary strings for extension discovery paths; local paseo-plugin skill (`~/.claude/skills/paseo-plugin/SKILL.md`).
