# sv-2 — Slice test: ship_visibility config surfacing

[DONE] New hermetic suite `scripts/tests/test_ship_visibility.mjs` proves all
4 sv-1 behaviors (absent -> off/no line/no warn; draft-pr -> surfaced +
preamble line; junk -> off + stderr warning; explicit off -> silent like
absent) against disposable temp-dir fixtures — live `.bee/config.json` never
touched. `scripts/impact-registry.json` regenerated (`--write`) after
`--check` flagged drift from the new suite; diff was purely additive.

Files touched:
- `scripts/tests/test_ship_visibility.mjs` (new)
- `scripts/impact-registry.json`

Commit: `f0460a67`

Full trace/evidence: `.bee/cells/sv-2.json`
