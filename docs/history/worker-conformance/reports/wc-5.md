# wc-5 — Teach the worker to conform first and stop authoring evidence

**[DONE]** — worker: Carl · lane: high-risk · capped on `--feature-verify-pending`

> Over the 40-line budget by design: high-risk cell, full trace required.

## Outcome

The bee-executing doctrine now states what the cap machinery actually does after
wc-1/wc-4. The two retired refusals — the `behavior_change` evidence door and
decision 0004's "recorded pass with no output" door — are rewritten as a recorded
absence (`trace.proof: "unrecorded"`) that arms the feature close-door, with the
judgement kept intact: an assertion is still not evidence. The worker is told
plainly it is no longer asked to author anything to pass a gate. A new "What did
NOT change" section names every surviving refusal, split honestly into doors that
fire on **both** cap paths and doors the pending path **defers, never waives**.
The D8 conformance habits (five pre-code passes, three post-edit checks) replace
the deleted evidence work.

## Honest scope note (cell action, second job)

CONTEXT D2 reads "lane high-risk (all classes)", but `requiredProofTier`
(`packages/bee/lib/cells.mjs:163-186`) leaves `refactor`/`formatting` at
`suite-green` and `test` at `targeted-green` even at `high-risk`. The doctrine
states the code's narrower truth and flags the discrepancy rather than repeating
the broader phrase.

## Files + commit

- `skills/bee-executing/references/worker-details.md` — retired rules rewritten;
  "Absent proof is recorded, not forgiven"; "What did NOT change"; conformance habits
- `skills/bee-executing/SKILL.md` — cap step, implement step, red flags
- `skills/bee-executing/references/provenance.md` — D1/D2/D6/D8/D12/D14 rows

Commit: see `.bee/cells/wc-5.json` (`trace.files_changed`) for the full trace.

## Deviations

- None to the cell's action. Post-consult correction inside the cell: the first
  draft claimed every surviving refusal fires unconditionally; three of them
  (`cells.mjs:2014`, `:2047`, `:2150`) are gated on `!pendingFeatureVerify`. The
  section was restructured before commit.

## Consults

1 consult · advisor **fable** (model-shaped, `advisor-consult wc-5: fable`).
Ask: falsify the rewritten doctrine against live `cells.mjs`/`state.mjs`.
Answer: narrow red-first scope, the `unrecorded` predicate, and the
`featureVerifyDebt` union all confirmed correct; four corrections accepted — the
missing `noTestWaiver` exemption, the wrong stamp line cite, the unnamed
`guardClaimOwnership` throw, and the false "no path lifts these" header. No
softening of the judgement found. All four applied before commit.

## Outstanding Questions

- `SKILL.md:39` states "capping throws without a fresh `advisor_ref`". No advisor
  check exists in `capCell`; the AO3/AO13 precondition throws at gate approval
  (`command-registry.mjs:710`). Pre-existing text belonging to another feature's
  law — recorded as friction, deliberately not rewritten inside this cell.

Full trace and evidence: `.bee/cells/wc-5.json`.
