# 2026-08-12 — A port honors only the verdict its host has a place to put

Batch: opencode-support (15 cells, 5 slices) plus 7 queued promote proposals
(write-guard-precision, knowledge-link-check, finish-advisory, evidence-ladder,
evolving-watch, workflow-lessons, doctor-binary-freshness). Cells cmp-1, cmp-2,
cmp-3.

- Porting the guard belt to a third runtime, the belt was built against the
  verdict the new host could express — a thrown error — and every verdict that
  had nowhere to go was silently discarded. Bee's dominant enforcement is not
  the refusal exit; it is the exit-0 repair (`updatedInput`, `permissionDecision:
  "ask"`), so the belt passed its own live deny/allow proof while the model
  guard was inert on that runtime. Caught by an independent slice judge, not by
  the belt's own tests, which asserted the shapes the port already handled.
  The durable fix is a test, not prose: the parity rows are now per
  (rule, verdict-shape), derived from the guards' own emit paths, so a fourth
  runtime that handles one shape and drops three fails by name (cmp-2).
- A coverage gate that derives its ground truth from the *host's* registry —
  not from the port's own mapping table — caught two real bypasses in one
  feature: a patch-applying tool and a code-navigation tool that returns file
  content for an arbitrary path. Both would have read as complete coverage under
  a hand-maintained list, and the second was found by the gate after the first
  was found by a human reviewer. Deriving ground truth moved the discovery from
  review to CI within a single feature.
- An environment-gated proof that skips is worse than one that fails: the belt's
  fixture suite reported four green tests and zero enforcement coverage in any
  shell whose interpreter predated the required version, because the skip wrote
  to captured output. Fail by default; degrade to a named skip only behind an
  explicit opt-out.
- Of 8 promote proposals reviewed, 1 carried content the bundle did not already
  hold. Twenty of the newest proposal's 24 area bullets were already stated by
  the same-day scribe, its entire second-area column was a duplicate (seven of
  those bullets named no surface in that area at all), and 4 of 5 pattern
  candidates restated patterns the bundle already holds. Mining is confirmation
  of the scribe, not a substitute for it — and the mined delivery draft's own
  frontmatter was wrong in three ways (unresolvable trace paths after archival,
  an area tag its body says does not exist, "no deviations" for a cell that
  wrote outside its declared files). Filed as friction rather than fixed by
  hand.
- The one thing the batch could not verify itself: worker dispatch on the new
  runtime is functional but sequential, and its CLI silently runs a primary
  agent when asked for a helper. Both are upstream-shaped and recorded as named
  gaps, not worked around.
