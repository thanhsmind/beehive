# handoff-self-relaunch — plan

Lane: small. Two disjoint cells, one wave. Locked context: `CONTEXT.md` D1, D2.

## hsr-1 — doctrine text (role: docs)

Outcome: every doctrine home of rule `agents-context-handoff-65` says D1.
Files: `packages/bee/AGENTS.block.md` (§ Care for the session), `AGENTS.md`
(through `bee dev regen` only), `skills/bee-hive/references/go-mode.md`
(Fallback Paths line), `skills/bee-swarming/SKILL.md` (handoff paragraph),
`skills/bee-swarming/references/swarming-reference.md` ("Fresh-session
handoff in full"), `docs/handbook/stages/swarming.md`,
`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
(spoken line), `docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md`.
Contract: no other rule text changes; regen output is committed; the
doctrine parity tests stay green.
Proof: `bee dev regen` clean, then the cargo tests that pin AGENTS.md and
skill text.

## hsr-2 — hook line (role: code)

Outcome: the prompt-context line past the mark tells the leader to write
the handoff and hand the rest to waggledance (`waggledance_dispatch`,
`leader: true`), and never tells it to wait for the user.
Files: `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` (the line
and its test at the `expected_line` assertion).
Contract: the line still fires at the same threshold and only then; one
line; red-before-green on the test.
Proof: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml prompt_context`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The doctrine tells the leader to pause at the mark | read | packages/bee/AGENTS.block.md:292 | `first, write .bee/HANDOFF.json and pause cleanly.` |
| 2 | Go mode tells the leader to send the user to a new session | read | skills/bee-hive/references/go-mode.md:124 | `present "[X] cells capped, [Y] in flight. Resume in a new session."` |
| 3 | The hook line says to continue in a fresh session, with no door | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:367 | `write a planned-next handoff (bee state handoff write --kind planned-next), and continue in a fresh session.` |
| 4 | A test pins that hook line word for word | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:3267 | `let expected_line = "context 160k tokens is past the handoff mark (150k): ...` |

## Out of scope

`bee state handoff dispatch` verb — backlog (D2).
