# 0006 — The Agent Runs the Machinery, Not the User

- **Status:** active — owner-approved 2026-07-08 (in-session settlement)
- **Date:** 2026-07-08
- **Source:** owner feedback (dogfood) — "tôi thấy thường tôi phải chạy script, nhưng không thấy tự quyết và chạy, tôi nghĩ nên tự làm chuyện đó mới đúng" (I keep having to run the scripts myself; the agent should decide and run them itself)
- **Confidence:** 0.9 (pure UX rule; no mechanism change)

## Decision

Every bee command — `bee_status`, `bee_cells`, `bee_reservations`, `bee_decisions`, onboarding, cell verify commands — is run **by the agent itself**, immediately, the moment the workflow calls for it. An agent must never print a bee command for the user to execute, and never end a turn on "run this and tell me the output".

The only human actions in the bee workflow are: **gate approvals, decision answers, and privacy approvals**. Everything mechanical is the agent's job. Users *may* run the helpers manually to inspect state — that is their option, never a delegated step.

## Rationale

- The vendored helpers were designed so *agents* have a mechanical interface; instructional text ("Run `node .bee/bin/bee_status.mjs` …") was read by some agents as user-facing guidance, producing sessions where the human relays command output by hand — the opposite of the automation bee exists for.
- The gates already isolate exactly where human judgment is required. Any other human step is accidental ceremony.

## Scope

- `packages/bee/AGENTS.block.md` — Critical rule 9.
- `skills/bee-hive/SKILL.md` — Priority rule 9 + red flag ("a bee command handed to the user to run").
- `packages/bee/lib/inject.mjs` — session preamble closing line now says "yourself … (agent-run — never hand bee commands to the user)".
- `BEE_VERSION` bumped to 0.1.4 (AGENTS block + vendored lib drift; repos refresh via re-onboard).

## Alternatives considered

- **Hook-enforce it** (detect a bee command in the assistant text and warn). Rejected: hooks can't reliably parse intent from prose; a norm in the three bootstrap surfaces (AGENTS block, SKILL, preamble) is where every agent actually looks.
- **Docs-only note.** Rejected: README isn't loaded into sessions; the rule must live in the loaded surfaces.

## Consequences

- Onboarded repos show drift until re-onboarded (`onboard_bee.mjs --apply`).
- Skills copies per runtime need a re-copy (`~/.claude/skills`, `~/.codex/skills`).
