---
artifact_contract: bee-plan/v1
mode: high-risk
approved_gate2: 2026-08-11
---

# Plan: OpenCode Support

Mode: `high-risk` — 3 risk flags: multi-domain, external-systems, public-contracts
Why this is the least workflow that protects the work: a third runtime crosses
render, onboarding, config, and the enforcement belt at once, against an
external API with no stability policy — proof must precede plumbing.

## Requirements (from CONTEXT.md)

- D1: OpenCode becomes the third first-class runtime at parity with Codex —
  hooks, rendered skills, `models.opencode`, onboarding; doctrine goes 2→3.
- D2: enforcement is real blocking via an OpenCode TypeScript plugin on
  `tool.execute.before`; config hooks are not an acceptable vehicle.
- D3: the beehive repo is the first consumer and test bed; host-project
  onboarding follows.
- D4: swarming must work under OpenCode. Upstream reality found in discovery
  (sequential-only dispatch) conflicts with "equivalent to bee-build/bee-gather"
  — that is a Gate 2 question for the user, not a planning call (see Gate note).

## Discovery

Local: OpenCode is not installed on this machine (`which opencode` empty) —
install is part of slice 1, version-pinned at install time (current stable was
1.18.16 on 2026-08-10 per changelog; S1 records the actually-installed pin).
Repo: every runtime enumeration is a closed claude/codex list, and the
fan-out is STRING-KEYED, not enum-keyed — `skill_trees.rs:42-45` picks the
target dir with `if runtime == "claude" { ".claude-plugin" } else {
".codex-plugin" }`, so a third runtime today silently renders into the codex
tree; same class at `skill_trees.rs:34` (`RENDER_RUNTIMES`),
`plugin_distribution.rs:1057` (`parse_runtime`), `install_support.rs:40-67`
(`--claude/--codex` flag pair), `onboard/skills.rs:616,930` (string literals).
Two distinct render pipelines produce skills roots
(`docs/06-runtime-integration.md:87-95`): `bee dev render-skill-trees` →
`.<runtime>-plugin/skills/`; the onboarding sync path (`applySyncSkill`,
`onboard/skills.rs`) → `.claude/skills/` and `.agents/skills/`. Marker render
refusal is tested at `skill_trees.rs:785-790` but asserts the literal
"expected claude or codex" (`skill_trees.rs:734`, mirrored
`onboard/render.rs:478`). The two-belt parity test named in
`docs/06-runtime-integration.md:143` DOES NOT EXIST in the Rust tree — it
died with the Node runtime (deleted in R6, commit 5c62cad0); bee's nine hooks
are enumerated in `packages/bee-rs/tests/hook_contracts.rs:37-47`.

Web (researcher digest, 2026-08-11, labeled Docs/Upstream):
- Plugin: `@opencode-ai/plugin` (v1.18.16 lockstep with CLI); project
  `.opencode/plugins/`, global `~/.config/opencode/plugins/`; export is an
  async function returning a `Hooks` object.
- Blocking: `tool.execute.before` has NO abort field — a thrown `Error` is the
  only documented block path. `tool.execute.after` observes; it cannot block.
  `permission.ask` exists but has two open bypass/not-firing issues
  (anomalyco/opencode #19927, #7006) — never a load-bearing gate.
- Skills: native SKILL.md support, discovery order `.opencode/skills/` →
  `~/.config/opencode/skills/` → `.claude/skills/` → `~/.claude/skills/` →
  `.agents/skills/` — an opencode projection in `.opencode/skills/` shadows
  bee's existing trees.
- Agents: `.opencode/agent/<name>.md` with `description`,
  `mode: primary|subagent|all`, `model`, `permission`, `prompt`. Dispatch via
  description-match or task tool; parallel dispatch is sequential today
  (upstream issue anomalyco/opencode #29638), background behind an
  experimental flag.
- Models: `provider/model` ids; per-agent `model:` frontmatter;
  `opencode run -m provider/model`.
- Stability: 1–2 day release cadence, no stated compat policy, one observed
  in-minor breaking behavior change → pin the version, verify empirically.

## Approach

Third hook-belt, same brain (cites D1, D2; helpers stay the FIRST belt on
every runtime — `docs/06-runtime-integration.md:7`): a thin TypeScript plugin
at `.opencode/plugins/` whose every guard decision is a call into
`.bee/bin/bee hook <name>`; a deny becomes a thrown `Error` in
`tool.execute.before`. No rule logic in TypeScript. After-events are advisory
only (logging/state-sync) — blocking rides `before`, per the digest.

Hook mapping (answers CONTEXT.md's expressibility question; S1 verifies
empirically, S3 pins as fixtures):

| bee hook | OpenCode surface | Fallback when inexpressible |
|---|---|---|
| write-guard | `tool.execute.before` (write/edit/bash), throw on deny | — (load-bearing; must prove in S1) |
| model-guard | `tool.execute.before` on task-tool args | structural: per-agent `model:` pins in `.opencode/agent/bee-*.md` make wrong-tier dispatch unrepresentable |
| session-init / prompt-context | `chat.message` hook injects the preamble digest | AGENTS.md auto-read (doctrine floor) |
| state-sync | `event: file.edited` / `session.idle` | helper-level staleness flag in `bee status` |
| tools-logger | `tool.execute.after` (observe-only) | omit — advisory |
| chain-nudge | `event: session.idle` | swarming tend-loop prose (same fallback Codex uses) |
| session-close | `event: session.idle` / `session.deleted` | Session-Finish prose in AGENTS.md |
| codex-subagent-audit | n/a — codex-specific | named exclusion (R1 named difference) |

Skills projection: the ONBOARDING SYNC PATH gains an `opencode` target
producing `.opencode/skills/` with a `.bee-render.json` sidecar —
`render-skill-trees`/`RENDER_RUNTIMES` deliberately gains NO opencode root
(no marketplace equivalent; a named exclusion, not an omission). Marker
grammar gains the `opencode` value in both render sites. S2 also replaces the
string-keyed dir picks with one exhaustive runtime→targets mapping so the
compiler (or a single table) owns the fan-out. `merge-plugin-state` gains a
third `--opencode` flag mirroring the existing pair — generalizing the public
contract to `--runtime <k>=<v>` was rejected: churn on a shipped surface for
zero present need. Worker tiers: `.opencode/agent/bee-*.md` with models from
`models.opencode.{extraction,generation,review}`.

Rejected alternatives:
- Ride OpenCode's native `.claude/skills/` reading — claude projection carries
  claude-only mechanics (subagent_type, bee-model-guard) that would misdirect.
- `permission.ask` as enforcement point — flaky upstream (see Discovery).
- Config `experimental.hook` — undocumented in current official docs, cannot block.
- `.opencode-plugin/` marketplace tree — no marketplace equivalent exists.

Risk map: TS plugin fail-open on crash / HIGH / fixture proves throw when the
bee binary is missing or crashes · string fan-out sends opencode output to a
codex tree / HIGH / S2 centralizes the mapping + wrong-target probe · plugin
API drift under 1–2 day releases / MEDIUM / version pin recorded, doctor
warns on drift (S5) · swarming parity gap / MEDIUM / one worker caps one real
cell in an OpenCode session (S4).

## Shape

Outcome: an OpenCode session on this repo runs the bee lifecycle with real
enforcement — proven by a blocked pre-gate write and a worker-capped cell.
Basis: two render pipelines, helpers-first doctrine, empirically verified
plugin API.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 Capability floor | Pinned install + minimal blocking plugin | No stability policy upstream; D2 hinges on throw-blocks | S1 | Live transcript: pre-gate write denied AND gate-true write allowed; skills discovered; preamble loads |
| E2 Runtime plumbing | `opencode` through both render pipelines via one central runtime→targets mapping; marker value; sidecar; refusal-message updates | D1 parity; R1 named exports; kills the silent-wrong-tree failure | S2 | `cargo test` green incl. wrong-target probe; `.opencode/skills/` rendered with provenance |
| E3 Guard parity | Full mapping table above implemented fail-closed; NEW three-belt parity test authored from zero (predecessor died in R6) | D2 | S3 | Fixtures incl. crash→deny per mapped hook; parity test: every guard rule hit at helper level and per-runtime hook level |
| E4 Workers + models | `.opencode/agent/bee-*.md`, `models.opencode` tiers, swarming prose marker block | D4 (as resolved at Gate 2) | S4 | Worker caps a real cell inside an OpenCode session |
| E5 Onboarding + doctrine | onboard target, `--opencode` flag, doctor version-pin drift check, docs 2→3 rewrite, fix stale `catalog.mjs` pointer in hook-runtime knowledge doc | D1, D3; CONTEXT.md:66 fix-in-passing | S5 | `bee onboard --apply` idempotent; manifest check green; doctor drift warning demonstrated |

Slice queue: S1 (no deps) → S2 (needs S1's verified layout names) → S3 (needs
S2 render + S1 payload shapes) → S4 (needs S3 guards live) → S5 (needs all).
Current slice to prepare: **S1**.

S1 — walking skeleton (in the feature worktree): install OpenCode, record the
installed pin; hand-write the minimal `.opencode/plugins/` guard calling
`bee hook write-guard`, throwing on deny; then in a live `opencode run`
session prove (a) a pre-gate source write is DENIED, (b) the same write with
the gate approved is ALLOWED, (c) bee skills are discovered, (d) the AGENTS.md
preamble loads. Findings land in `docs/history/opencode-support/discovery.md`;
verified layout names feed S2.

## Test matrix

High-risk probes per applicable dimension (each cell's writer judges existing
coverage first):

- Env (6): no `opencode` binary → onboard names the gap, never half-installs;
  plugin present but `.bee/bin/bee` missing → throw (deny), never silent allow.
- Error cascades (7): bee hook crashes or exits nonzero → plugin throws;
  probe per mapped hook (S3).
- State transitions (5): gate false → denied; gate true → allowed — live in
  S1 (proofs a+b), pinned as fixtures in S3.
- Integration (10): wrong-target probe — rendering `opencode` must never
  write into `.codex-plugin/` or `.claude-plugin/` (S2); version drift →
  doctor warns (S5); malformed-marker whole-render refusal exists at
  `skill_trees.rs:785-790` but asserts "expected claude or codex" — S2 edits
  that expectation, in both `skill_trees.rs:734` and `onboard/render.rs:478`.
- Business rules (12): three-belt parity test, authored new in S3 — every
  guard-catalog rule exercised at the helper level AND through each runtime's
  hook belt (claude fixture, opencode fixture; codex where file-shipped).
- Not applicable: user types, authz, compliance, money (1, 8, 11) — developer
  tool, no new PII surface.

## Out of scope

- Host-project onboarding polish and INSTALL walkthrough for OpenCode-only
  repos — follows the beehive proof (D3, deferred).
- Smoke-testing the existing `opencode-review` CLI-executor preset — adjacent debt.
- Parallel/background subagent dispatch mechanics — upstream-gated; what D4
  accepts in the interim is decided at Gate 2, never assumed here.
- Any OpenCode marketplace/plugin-registry distribution.
