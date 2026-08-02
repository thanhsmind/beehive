# 05 — Build Roadmap

Build bee with bee's own philosophy: smallest honest slices, validate before building, prove risky assumptions first, compound learnings between phases. Each phase ends with something usable.

## Phase 0 — Spikes (prove the risky assumptions first)

The V3-synthesis lesson: prove risky ideas early instead of discovering blockers halfway through.

| Spike | Yes/no question | Evidence |
|---|---|---|
| S1 dual-manifest | Does one `skills/` dir load correctly from both `.claude-plugin/plugin.json` and `.codex-plugin/plugin.json`? (khuym proves the Codex side; verify the pair) | Both runtimes list the same skills |
| S2 hook skeleton | Do plugin-shipped hooks (SessionStart, UserPromptSubmit, PreToolUse block, SubagentStop) fire reliably in Claude Code on Windows, including the self-arm check (silent when `.bee/onboarding.json` absent) and a PreToolUse deny? | Each hook demonstrably fires/blocks; crash log works |
| S3 cell helper | Can a zero-dependency Node script enforce cap-requires-verify and lane field tiers with atomic writes (`.tmp` + rename)? | a cells-helper prototype (later unified into `bee.mjs cells`, D1) + its test script |
| S4 Codex subagent results | Do Codex subagents reliably return `[DONE]/[BLOCKED]` tokens to the parent thread the way khuym's swarming assumes? | One toy swarm run |

A NO on any spike changes the architecture doc before any skill is written.

## Phase 1 — The spine (hive + tiny lane end-to-end)

Goal: a tiny fix can flow bootstrap → plan(tiny) → validate(light) → one worker → self-checks, entirely under bee.

1. Shared `bin/lib/` modules (state, cells, reservations, guards, inject) + vendored per-group helper scripts (status, cells, reservations; + test scripts, following khuym's `test_onboard_khuym.mjs` pattern) — later unified into and retired in favor of `bee.mjs <group> <verb>` (D1, shim-retire).
2. `onboard_bee.mjs`: installs AGENTS.md block (BEE:START/END), `.bee/` runtime files, helpers + lib; `--apply` gated on approval.
3. First half of the hook skeleton, as thin wrappers over `lib/`: `bee-session-init`, `bee-write-guard` (gate + privacy checks), `bee-session-close` — with the AGENTS.md preamble generated from the same `inject.mjs` source so the runtimes cannot drift.
4. Skills (each RED → GREEN → REFACTOR with pressure tests, in this order): **hive**, **planning** (mode gate + tiny/small paths first), **validating** (reality gate only; later removed and folded into **planning**, validation-diet D1), **executing**, **reviewing** (lightweight path).
5. Dogfood: run three real tiny fixes in a real repo. Capture friction verbatim — it is Phase 2's input.

Exit: the three fixes shipped through bee with zero hand-editing of `.bee/` files.

## Phase 2 — The full chain (standard lane)

1. **exploring** (Socratic locking, CONTEXT.md template, gray-area probes).
2. **planning** full: research levels, shapes (phase plan / epic map), cell prep with `must_haves`.
3. **validating** full: feasibility matrix, spikes, adversarial plan-checker, cell review. (Skill later deleted — feasibility matrix retired with no replacement, spikes and cell review folded into **planning**, validation-diet D1/D5/D6.)
4. **swarming**: wave analysis, spawn contracts for both runtimes, tend loop, HANDOFF.
5. **reviewing** full: specialist roster, EXISTS/SUBSTANTIVE/WIRED, UAT, Gate 3.
6. Second half of the hook skeleton: `bee-prompt-context` (injection dedup), `bee-state-sync`, `bee-chain-nudge` (reservation guard added to `bee-write-guard`) — plus the two-belt parity test (every guards/cells rule exercised by both a hook test and a helper test).
7. Go mode in hive; HANDOFF/resume tested across a real pause.
8. Dogfood: one standard feature end-to-end on each runtime (Claude Code and Codex).

Exit: Gate discipline holds under pressure tests; a feature pauses at 65% and resumes cleanly next session.

## Phase 3 — Memory and the clean hive

1. A decisions helper (event-sourced log, write-time redaction, datamark on read — later unified into `bee.mjs decisions`, D1) + decision surfacing in hive bootstrap.
2. **scribing** (decision 0002): the BA-grade state layer — area-spec template with the rebuild bar, sync mode in the chain after execution (review-on-demand, decision 565e68d0), capture mode for discussion-agreed rules, harvest mode for legacy areas.
3. **compounding**: analysts, learnings template, critical promotion, friction → backlog, and the state-layer **guard** (verify scribing ran; invoke it if not — decisions 0001/0002).
4. **grooming**: entropy audit (including the `stale specs` term), hunt checklists, propose/approve/execute/close-the-loop.
5. **The skill-writing discipline** (adapted from khuym; today the maintainer guide [handbook/writing-skills.md](handbook/writing-skills.md); needed before Phase 3 skills are edited further — consider pulling it into Phase 1 if skill churn is high).
6. Backlog outcome loop live: every grooming item closes with predicted-vs-actual.

Exit: after two features, `critical-patterns.md` and active decisions demonstrably change agent behavior (fewer repeated mistakes in dogfood notes); entropy score reported and trending; for at least one area touched twice, a fresh session answers "what does this area do now?" from its `docs/specs/` entry without re-exploring the code; and the **rebuild test** has run once (decision 0003) — one small area rebuilt on a different stack from its spec alone (Pointers removed), observable behavior compared against the original.

## Phase 4 — Polish (only if earned)

- Cross-model second-opinion step at Gates 2–3.
- Capability registry (`tools.json`) with graceful fallbacks (gkg, beads adapter, browser testing).
- Docs-from-code generation for helper command references, and the **repo-native playbook** (`.bee/PLAYBOOK.md` generated from SKILL.md sources at build time, installed by onboarding — the tier-3 degradation path so any plugin-less agent in the repo can still run the chain; see 06-runtime-integration.md).
- High-risk lane hardening: detailed trace tiers, mandatory spike evidence links, plan-review persona panel (coherence + feasibility + conditional lenses).
- Repo-profile cache: question-agnostic project profile (stack, deps, conventions) derived once per repo+HEAD and shared by all grounding skills (compound-engineering).
- Feedback sweep: deterministic ingestion of external feedback into grooming's backlog via a state machine — GitHub Issues only at first (compound-engineering's ce-sweep, radically simplified).

Deliberately deferred until real usage demands them — each is upstream-proven but none is spine.

## Working agreements while building

- The Iron Law applies to bee's own skills from the first line: no SKILL.md without a failing pressure scenario, and a CREATION-LOG.md for each.
- Dogfood friction is captured with the harness triggers and becomes the next phase's backlog — bee is its own first grooming target.
- Skill additions are decision-gated (decision 0002): a new skill needs a decision record naming the workflow gap no existing skill covers. Keep the hook cap; any proposed hook must name which of the nine it replaces.
- Every enforcement rule ships in `bin/lib/` first (works on both runtimes), then optionally gets a hook belt — never hook-only enforcement.
