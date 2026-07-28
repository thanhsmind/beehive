# Provenance — bee-executing body rules

The worker body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Validate, never claim — the orchestrator claims before spawning, the worker only confirms `status: "claimed"` + `trace.worker` | D1 (worker-claim isolation) | A worker that self-claims can race the orchestrator's schedule and pick up work nobody assigned it |
| `[DONE]` carries the diff and commit, never verify output | main-verifies D4 | Verify is MAIN's proof event at feature close, not per-cell worker evidence |
| Cap default is `--feature-verify-pending`; classic evidenced path stays sanctioned for spot use, other repos, transition | main-verifies D1/D4 | The pending path is the default; the classic per-cell verify path is preserved, not removed |
| `behavior_change: true` pipes `verification_evidence` via `--evidence-stdin`, never a written evidence file | decision 0009 | Evidence lives in one place — the cell trace — never duplicated on disk |
| Per-cell report links `.bee/cells/<cell-id>.json` instead of re-embedding evidence or verify output; never a separate scratch file | decision 0009; docs/specs/doctrine-layer.md R17 | The trace is the single source of evidence; `.bee/tmp/<feature-or-session>/` is the one canonical scratch home |
| Advisor consult required before the execution gate on high-risk/hard-gate cells; staleness is hash-and-decision-anchored, never a TTL | AO3/AO13 | Advice must reflect the current plan and decisions, not a stale snapshot |
| Consults section (count, advisor identity, ask/answer digest) — bee-swarming's goal-check reads this field | A2 | Attribution record for advisor spend, read back at goal-check time |
| Headless: consulting a configured advisor stays inside your own turn, never "asking the parent or user" | A4 | Advisor Consult does not change the Headless rule — it is not a blocking question to a human |
| Fresh-Session Handoff: planned-next adopts only at a real fresh-session boundary, worker never claims/writes it mid-swarm | fresh-session-handoff D1; no-clear-stop D1 | Adoption is safe only where no stale mid-swarm context can disagree; a worker's job stays Cap → Release → Return |
