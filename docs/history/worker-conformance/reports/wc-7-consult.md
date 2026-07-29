# wc-7 — Advisor consult (fable)

advisor-consult wc-7: fable — 1 consult, read-only, cell wc-7 (worker-conformance, high-risk).

**ASK:** check the ten doctrine corrections in `skills/bee-executing/references/worker-details.md`,
`skills/bee-planning/SKILL.md` and `skills/bee-planning/references/planning-reference.md` against live
`cells.mjs` / `state.mjs` for statements still false, statements made more PERMISSIVE than the code
(forbidden for this cell), and surviving overstatements I missed.

**ANSWER (all seven asserted claims CONFIRMED against the cited lines; four corrections accepted and applied):**

1. `behaviorEvidenceWarning` (`cells.mjs:1969`) non-blocking, no lane gate, `!pendingFeatureVerify` — CONFIRMED.
2. `recordedProofWarning` lane-gated at `:2182` — CONFIRMED, **plus two leaks I missed**: `spike` is a lane
   (`cells.mjs:91`) and sits outside the gate exactly like `tiny`; and whitespace-only evidence reads present
   at `:1969` but empty at `:2229-2230`, so the marker can land silently on a `behavior_change` cell after all.
   → text now names `tiny` **and** `spike` and hedges "never in silence" to the non-degenerate case.
3. The five `!pendingFeatureVerify`-gated doors (`:1913`, `:2014`, ratio `:2047/:2053`, `:2150`, and `:2118`
   unreachable-by-construction) — CONFIRMED.
4. The five ungated doors (`:1874`, `:1926`, `:1990`, `:2191`, `:2197`) — CONFIRMED, **but** `:1990` is gated on
   `diff_stats` being supplied (`:1992`, fail-open), so calling it "ungated" claimed more enforcement than exists.
   → text now says it fires on both cap paths and names the `diff_stats` fail-open as the only skip.
5. The exclusivity checks (`:1891`, `:1896`) live inside `if (pendingFeatureVerify)` — my narrower reading is
   right and the cell action's "ungated, both paths" was wrong. CONFIRMED.
6. `requiredProofTier` class × lane matrix (`:163-186`) — CONFIRMED verbatim.
7. `testCellDebt(root, feature)`, the two non-shared predicates, the three-arm offender set — CONFIRMED, **with a
   permissiveness caveat**: a pending-capped `test` cell that recorded a FAILING verify is still an offender at
   `state.mjs:2605-2606`, so "clears this door" was unconditional where the code is not.
   → both bee-planning statements now carry "provided no failing verify was recorded on it".

**Permissiveness risk flagged and removed:** `planning-reference.md` rule 5 read as "capping with no proof at all
still caps", which the machine denies — `:1913` still refuses a cap without a recorded passing verify on the
classic path. The sentence now says what actually became non-blocking: the pass asserted with nothing recorded.

Advice was advisory only; no gate approved, no locked decision overridden. Every accepted correction was
re-verified against the cited line before editing.
