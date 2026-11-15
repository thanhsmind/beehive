# peer-brief-craft — plan

Lane: small. Class: feature. Architect skipped: no new data shape — D1/D2
are prompt text, D3 reuses the existing arm-outcome shape over a second key.

## Throughput

- Blocking first steps: none; the gate is the only stop.
- Independent workstreams: the prompt texts (pbc-1) and the report (pbc-2)
  touch disjoint files and run in parallel.
- Shared mutable state: none between pbc-1 and pbc-2.
- Smallest safe decomposition: two implementer cells, each followed by its
  own tester cell (pbc-3 after pbc-1, pbc-4 after pbc-2).

## Cells

| Cell | Role | Does | Decisions |
|---|---|---|---|
| pbc-1 | code | peer-boot.md and worker-cell.md text, vendored copy, concept sync | D1, D2 |
| pbc-2 | code | principles-arm block in `bee knowledge report` | D3 |
| pbc-3 | test | proves the rendered briefs and the peer preamble | D1, D2 |
| pbc-4 | test | proves the principles-arm block on a fixture store | D3 |

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The peer preamble forbids AGENTS.md | read | packages/bee/prompts/peer-boot.md:8 | Do not read AGENTS.md, CLAUDE.md or bee process docs. |
| 2 | The worker brief lists AGENTS.md as an input | read | packages/bee/prompts/worker-cell.md | Inputs — read these; nothing else is provided: |
| 3 | Native and herding renders are exclusive | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1554 | let native_bookkeeping = if is_herding { "" } else { "1" }; |
| 4 | prepare records principles_arm on cell rows | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3525 | "principles_arm".into(), |
| 5 | The principles holdout defaults to 10 percent | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1193 | const KNOWLEDGE_HOLDOUT_DEFAULT_PERCENT: u64 = 10; |
| 6 | The outcome builder reads only knowledge_arm | read | packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs:347 | let arm = row.get("knowledge_arm").and_then(Value::as_str).unwrap_or_default(); |
| 7 | Every principle points Depth into expertise | ran | rg -l Depth.*\.bee/expertise skills/bee-principle-*/SKILL.md | 37 of 37 files |

## Proof

Each cell runs its filtered cargo tests. The full suite runs once with
`bee test` before `bee worktree merge`.
