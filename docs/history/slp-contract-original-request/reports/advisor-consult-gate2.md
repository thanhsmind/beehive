# Advisor consult — high-risk execution gate, slice 1

- Date: 2026-08-29 · Advisor: fable (configured `models.claude.advisor`)
- Evidence bundle: `docs/history/slp-contract-original-request/plan.md` (rev 2),
  `docs/history/slp-contract-original-request/CONTEXT.md`, base `37f6ae3a`.
- Verdict: **GO — approve execution for slice 1 only.**

## 1. Faithfulness of D1-D6

**D1 — FAITHFUL.** Plan: "A pure read, no store, no registry" (plan.md
Approach/S3) and Out of scope "Any new store, registry, or interface
enumeration (D1)". CONTEXT.md D1: "a DERIVED view over the decision log; bee
keeps no hand-maintained contract registry."

**D2 — FAITHFUL, and the widening is the right call.** CONTEXT.md D2 locks the
tag as `contract:<name>`; the Agent's Discretion clause frees only "its
spelling in prose", not the tag. Verified in tree: `tag_pattern_test`
(`packages/bee-rs/crates/bee/src/verbs/decisions/scanners.rs:476-483`) refuses
a colon today, so the only two moves are widen the pattern or respell the tag
— and respelling is the one CONTEXT.md forbids. Widening is
backward-compatible (every char class in the current predicate survives), and
the plan carries the `TAG_PATTERN_DISPLAY` side effect (`scanners.rs:474`).
The claim that no path or filename is built from tags held up on spot-check.

**D3 — FAITHFUL.** The plan puts the tripwire at BOTH doors; CONTEXT.md
explicitly delegates the door choice to planning, and both is a superset. The
"store citation" definition — a local D-ID is a pointer into a CONTEXT.md
table, not a store citation, and is passed over silently — is a definition the
locked text does not give, but Discovery 5's measurement (87% of citing cells
use local D-IDs) makes the literal reading unusable, and the definition is
recorded, not silent. Residual hole: a cell citing ONLY local D-IDs reads as
citing nothing, so the tripwire never inspects it — coherent with D4, which
then treats it as uncited.

**D4 — LEGITIMATE ROLLOUT, not a quiet narrowing — but it is a judgment on a
locked letter and is logged as one.** Two honest observations:

- The tension is real: D4's rationale targets exactly the never-logged-contract
  state, and the ramp warns in exactly that state. A strict reading refuses
  every test-writing cell on day one with a remedy only the user can perform.
  The ramp is the only operable faithful-in-the-limit reading.
- The cited precedent is weaker than claimed: `NO_ROUTE_RECORD`
  (`verbs/cells/handlers_write.rs`, "warn once per session, then refuse") is a
  BOUNDED per-session ramp; this ramp is UNBOUNDED — a store that never adopts
  `contract:*` tags warns forever and D4 never binds. Acceptable only because
  the plan says so in the gate text the user approved.
- **Condition attached and met:** the ramp is logged as a decision with
  `--relation touches:9c0104e0`, so the flip condition lives in the log, not
  only in a plan file.

**D5 — FAITHFUL via D6.** CONTEXT.md D6 itself resolves D5 to the
dispatch-door read. The plan's rejection of a per-cell field cites the locked
rationale correctly.

**D6 — FAITHFUL.** The plan names the `request` field, the `PRECOMPACT_HEADER`
framing (verified verbatim at `verbs/intent_group.rs:250-252`), all four
templates, and both var slices. Templates are `include_str!`-embedded from
`packages/bee/prompts/*.md` (`verbs/drivers/prompt.rs:33`), one renderer, no
live Node twin — so "four templates + rebuild" is the true full surface. Pass
2 of `render` never re-scans a substituted value, so a request containing `{{`
cannot inject or die at the door; the test-matrix probe pins an already-safe
property.

## 2. Slice 1 and the no-fallback choice

Strongest argument against: D5/D6 say "every dispatch" and "every template" —
a featureless gather or advisor dispatch renders nothing, so the anti-drift
guarantee silently does not hold precisely for the cross-feature dispatches
where drift is most likely, and no one is told.

It does not hold. Verified live: `.bee/intent/default.json` carries a
2026-08-25 request for a shipped decision-attribution fix whose own
`next_action` reads "Nothing owed — this request SHIPPED." Rendering that
under a DO-NOT-PARAPHRASE banner is meaning-REPLACEMENT — the exact violation
D5 forbids — while absence is a visible gap. `read_anchor_at`
(`intent_group.rs:216`) has no staleness check, so no guard can distinguish a
fresh default from a dead one.
<!-- bee:not-a-deferral: reports a deferral CONTEXT.md already made and planning already answered; the answer is in plan.md, Approach Half B -->
CONTEXT.md explicitly deferred "which anchor
does a featureless dispatch read" to planning, so "none" is granted
discretion, not narrowing.
<!-- /bee:not-a-deferral -->
S1 is the right skeleton: end-to-end,
dependency-free, and its riskiest property (byte-identical absence) is exactly
what the existing pattern already pins twice.

## 3. Most likely way it ships and is still wrong

The mint trap ships green and never fires on this repo's dominant
test-writing shape. Per the plan's own Discovery 7: a `role: code` cell adding
a `#[cfg(test)]` module to a source file it already touches is the majority
shape, the armed arm (`path_looks_like_test`,
`verbs/cells/finish_support.rs:693`) cannot see it, and the advisory arm only
warns — and this feature exists for the SLP swarm lanes, where warnings scroll
past unattended workers under `gate_bypass` with nobody reading them. Every
proof in the matrix goes green, the named-hole test passes by design, and D4's
purpose remains unserved for roughly 60% of real test-writing cells. The plan
is honest about this; honesty does not make the trap effective. Expect this
exact gap to be the first post-ship finding.

## 4. Go / no-go

**GO — approve execution for slice 1 only.** Condition: S1's cap proof must
include the risk-map row as written — a featureless dispatch renders
byte-identically to today WHILE a non-empty `.bee/intent/default.json` exists
— since that single test is what separates this design from the rejected
stale-fallback, and it is the one behavior nothing else in the suite would
catch.
