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
| The `behavior_change: true` evidence door and decision 0004's "recorded pass with no output" door no longer refuse a cap; the worker is never asked to author anything to pass a gate | worker-conformance D1 | The doors that made a worker overshoot are the ones asking it to *write* evidence to pass a gate, and authoring work drifts |
| Absent proof is stamped `trace.proof: "unrecorded"` and arms the feature-boundary close-door instead of blocking the cap | worker-conformance D12 (supersedes D10's mechanism, keeps its intent); main-verifies D3 | A separate inert field, never a reuse of `feature_verify: "pending"` — the pending flag short-circuits six refusal sites, so reusing it would have voided the red-first tier and the volume brakes |
| "unrecorded" requires BOTH channels empty — a cell holding real `verification_evidence` is never marked, even with empty `verify_output` | worker-conformance D14 | The predicate defined against `verify_output` alone would arm the close-door for cells holding the strongest proof in the system |
| Red-first still refuses on the classic path (deferred, never waived, by `--feature-verify-pending`), at the tier `requiredProofTier` resolves — `security`/`migration` every lane, `bugfix`/`behavior`/`api` at `high-risk`, while `refactor`/`formatting` stay suite-green and `test` stays targeted-green even at `high-risk` | worker-conformance D2; test-economy D1/D2; slice-tail-test-batching P1/P2/P3 | Red evidence is emitted by a real red run, not authored prose, so it does not cause the drift D1 removes; the lane alone never buys red-first |
| The new-test-file rules and the ratio ceiling still refuse, unchanged and with no bypass level lifting them | worker-conformance D6; test-economy D3 | The triad is the shape guide and the ratio ceiling is the volume brake; a second numeric cap on the same axis would contradict |
| Conformance habits — the pre-code five and the three post-edit checks per file | worker-conformance D8 | Cheap conformance work replaces the deleted evidence work: move worker effort from proving to conforming, not remove it |
