# 0007 — Settlement Detection Is the Scribe's Duty, Unprompted

- **Status:** active — owner-approved 2026-07-08 (in-session settlement; refines the intent behind 0006's session)
- **Date:** 2026-07-08
- **Source:** owner feedback (dogfood) — "bee-scribing đây là chuyện ghi lại cho tôi kiến thức, nhưng thường để tôi phải manual gọi, trong khi nên thấy là ghi lại" (scribing exists to record knowledge for me, but I usually have to invoke it manually — it should *notice* and record)
- **Confidence:** 0.85

## Decision

bee-scribing's capture mode is **self-triggering**: the agent watches every turn for settlement — a rule agreed, a behavior confirmed working, a value chosen, an option picked and moved past — and runs the capture **without being asked**. Explicit signals ("chốt", "final", decision 0003) are the loud case; most settlements are silent and detecting them is the agent's job. The protocol is announce-then-do, one line ("chốt: X — ghi vào `docs/specs/<area>.md` + decision log"), same turn — never "should I document this?". A user having to say "ghi lại" means detection already failed once.

Capture writes only `docs/` and `.bee/` — allowed in every phase, no gate, no permission needed.

## Rationale

- Capture mode already existed (0002, 0003) but its trigger text lived *inside* the scribing skill — a surface agents read only after the skill is invoked. Nothing in the always-loaded surfaces (AGENTS block, bee-hive) assigned the *watching* duty, so in practice capture fired only on explicit user request — the exact inversion of why the skill exists.
- The knowledge loop is bee's core promise; a capture step gated on the human remembering to ask is a silent leak of exactly the knowledge bee promises to keep.

## Scope

- `packages/bee/AGENTS.block.md` — rule 8 extended: detection duty, announce-then-do, no-gate note.
- `skills/bee-hive/SKILL.md` — priority rule 8 extended: "user asks to document" routing row is the fallback, not the norm.
- `skills/bee-scribing/SKILL.md` — description marked SELF-TRIGGERING; capture mode gains the detection-duty paragraph; two new red flags (user-prompted capture of a silent settlement; asking permission to document).

## Alternatives considered

- **Hook-detect settlements** (scan turns for signal words). Rejected: settlement is semantic, not lexical; a keyword hook would both miss silent settlements and false-fire.
- **Keep explicit-signal-only (0003 as-is).** Rejected by owner feedback: the manual step is the complaint itself.

## Consequences

- AGENTS block drift → repos need re-onboard; skills copies need refresh.
- Risk: over-capture (noting trivia as decisions). Mitigation: the litmus stays "would this outcome exist anywhere but the chat, and does losing it cost anything?" — trivia fails that test. Watch in dogfood.
