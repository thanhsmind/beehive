# Provenance — bee-validating body rules

The validating body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| `plan.md` frozen at Gate 2 — content byte-identical to what the human approved | D1 (plan freeze) | The artifact validating reads must be the same artifact the human signed off on |
| Discovery/approach content may be `plan.md` sections instead of standalone files | decision 0009 | A small/standard feature spawning extra restated-state files is the anti-pattern this closes |
| The current slice lives only in cells; no separate slice document | D2 (slice-in-cells) | One durable record per concept — cells already carry the current-work boundary |
| Orient read delegates to an extraction-tier I/O worker per the D2 rubric, launched inside the review wave | Delegation contract D2/D3 | Mechanical multi-file reads dispatch down-tier as I/O; judgment stays on the session model |
| Sync point: findings block nothing until the Gate 3 presentation, wave-wide | decision 0017 | No wave member's outstanding work may be silently skipped at the approval moment |
| Review-wave dispatch resolves the `review` slot (default opus on Claude, generation fallback) | decision 0021 | The model that reviews should not be the model that implemented |
| Codex has no per-agent subagent type; the review tier is a read budget + output cap only | AO11 | Codex's runtime cannot select per-agent models the way Claude Code can |
| A cli-shaped review slot resolves via the purpose-scoped 4-arg form; a bare 3-arg resolve refuses | AO12/B1 | A read-only gather must be dispatched through the Delegation contract's cli branch, never a bare resolve |
| WARNING-level/mechanically-fixable findings apply directly to cells — legal because cells are mutable before Gate 3 | D2 (cells mutable pre-Gate-3) | Fixing a typo does not need a second reviewer pass |
| Advisor consult required before Gate 3 for high-risk/hard-gate slices, at every bypass level | AO2b/AO3/AO4 | A bypass level lifts the human checkpoint, never the mechanical advisor precondition |
| `advisor_ref` staleness anchors (feature mismatch, newer decision, changed plan hash, gate revocation) — never a time-based TTL | AO13 | A TTL was invented and wrong once already; hash-and-decision anchors are the only law |
| Bypass covers Gates 1-3 by level; the level table lives in bee-hive's routing reference | decisions 0010/dcf01d7b | One shared bypass-level table, not a duplicate copy per skill |
| `validated` is never written as a phase; the approved execution gate is what records that | chain-integrity D6 | An agent that hits the refusal and invents a phase value is exactly how the chain broke before |
| bee-briefing refresh is presentation, not evidence — deferred lazily when bypass covers Gate 3 | spec #77 P6 | The machine report is the evidence; the human-facing brief only needs to be current when a human is about to read it |
