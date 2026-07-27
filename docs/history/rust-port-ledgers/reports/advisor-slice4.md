# Advisor consult — rust-port-ledgers slice 4 (pre-Gate-3, AO2b)

Advisor identity: `fable` (model-shaped, resolved from `.bee/config.json` → `models.claude.advisor`).
Run read-only against the evidence bundle. Advice is **data for the decision, not the decision**
(critical rule 12): nothing below overrides a locked D1–D11.

## Verdict per question

**1. Slice boundary — sound.** The serial 8-cell shape matches the shared-write reality; the
`rpl-4`/`rpl-5` read/write split of `decisions` is justified by the renderer risk. No cheaper
honest decomposition. Do not merge cells to save waves that cannot be parallelized anyway.

**2. Under-proved HIGH rows — three.**

- **Lock row is half-orphaned.** The mixed mjs↔rust writer proof is assigned only in `rpl-4`
  (the D9 decisions lock). `rpl-6` says the second lock is "preserved as-is" and carries **no**
  concurrency proof at all, even though `backlog.mjs`'s `addPbi` lock is a separate
  implementation with its own retry constants.
- **`localeCompare` row is too weak.** A `review-1` / `review-2` / `review-10` scenario only
  catches naive lexicographic sort. ICU numeric collation (`reviews.mjs:135`) also decides
  leading-zero ties (`review-01` vs `review-1`) and case (`Review-2`); a hand-rolled natural
  sort passes 1/2/10 and still drifts.
- **Renderer row is only as strong as its corpus.** Same failure class as the capture/intent
  empty-store find already caught: the fixture must seed superseded, redacted and non-ASCII
  decisions, or the byte-diff proves an easy subset.

**3. Failure classes missed entirely — two serious, one minor.**

- **Nondeterminism defeats the file-tree byte-diff on every write verb.** Write verbs stamp
  wall-clock and randomness with no injection seam — `crypto.randomUUID()` / `toISOString()` at
  `packages/bee/lib/decisions.mjs:320-322`, `reviews.mjs:46,348`, `backlog.mjs:284`, and
  tempfile suffixes at `decisions.mjs:220`. So `rpl-1`'s tree diff **cannot** match on write
  verbs, and the plan never stated the normalization strategy. If the differ masks these fields
  it must additionally assert their **format** — 3-digit-millisecond `Z` ISO shape, UUID-v4
  shape — or it silently hides real drift (chrono's RFC3339 variants versus JS `toISOString()`
  are a live example).
- **stderr is outside the diff surface.** `RunResult` captures stdout + exit code only
  (`crates/bee-parity/src/runner.rs:118-124`), and `rpl-1`'s stated surface matches that. But
  the fail-open warning (`reviews.mjs:130`) and the rejection text whose `${pattern}` spelling
  `rpl-3` explicitly obligates (`decisions.mjs:280-283`) are **stderr-only**. As specified, that
  obligation is unprovable by the runner — add stderr to the diff or the proof is theater.
- **Minor.** JS `JSON.stringify` reorders integer-like object keys ascending; `preserve_order`
  keeps insertion order. Any ledger object with numeric-string keys diverges despite the stated
  `preserve_order` mitigation.

**4. Replicate rather than consolidate — confirmed.** Under D1 (frozen oracle) and D11 (dark
port), consolidating the two locks or the inconsistent `feedback` comparators now would make the
byte-parity proof test a redesign instead of the port. The real hazard — two Rust lock
implementations drifting later — belongs in a post-parity consolidation PBI once mjs retires.
No D1–D11 concern flagged.

## Disposition

Every finding in §2 and §3 was **accepted** and folded into the cells before Gate 3 — see
`validation-slice4.md` → "Cell repairs applied". None conflicts with a locked decision, so
nothing was escalated as an advisor-versus-CONTEXT conflict.
