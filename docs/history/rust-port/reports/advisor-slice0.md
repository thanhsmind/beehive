# rust-port Slice 0 — Advisor Consult (pre-Gate-3)

Advisor: fable (model-shaped, `models.claude.advisor`), read-only, 2026-07-26.
Evidence bundle: CONTEXT.md, plan.md, approach.md, validation-slice0.md, cells rust-port-1..6.

## Verdict

**PROCEED WITH NOTES** — packet well-ordered (deps acyclic, confirmed), constraints traceable to D-IDs, B1–B5 repairs genuinely reflected in cell text. Three proofs thinner than they read; none stop-ship for a proofs-only slice.

## Notes and disposition

1. Self-parity zero-diff could pass on two identical failures (inner exits unchecked) → **applied**: cell 4 self-check now requires both inner exits 0 + fixture sanity read.
2. Generator happy path unverified → **applied**: cell 2 `--self-test` now asserts refusal cases AND a happy-path generation meeting every pin + bee.mjs sanity read.
3. Two-temp-root diff will trip on absolute paths; escape hatch collided with the no-blanket-normalization prohibition → **applied**: root-path→`<ROOT>` rewrite added to the declared allowlist (pre-decided, no worker improvisation).
4. Cell 6 title overstated ("runs under run_verify/CI" is full-run true, impacted-run false until D6) → **applied**: title + action + truth now name the impacted-run blind spot. CONTEXT.md typo `impact-registry.mjs` → corrected to `impact_registry.mjs`.
5. Truncated-tail tolerance had no oracle → **applied**: cell 5 proves it by running the real mjs reader on the same corrupt bytes and diffing.
6. Re-rank: parity-fixture representativeness HIGH for Slice 0 (was MEDIUM) — mitigated by notes 1+2+7; lock interop well-covered; musl floor stays LOW.
7. Highest-value addition (fixture sanity truth in cell 4) → **applied** (covers notes 1, 2, half of 3).

Advice is data, not approval; no locked decision overridden.
