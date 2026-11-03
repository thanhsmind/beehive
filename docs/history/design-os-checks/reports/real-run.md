# Real run (D11) — design-os-checks

Date: 2026-10-30. Head binary: built from `wt/design-os-checks` at the regen commit. Main binary:
`.bee/bin/bee` in the main checkout (gate pinned at ease-design 0.6.1). The tool came from npx
(`ease-design@0.6.2` on head, `@0.6.1` on main). No file in anphabe-goglbe or design-os was written.

## anphabe-goglbe copy (51 pages, baseline recorded at 0.6.1, no artifacts)

| Binary | Tool | Result |
|---|---|---|
| main | 0.6.1 | `design gate: no regressions on 51 pages`, exit 0 |
| main | 0.6.2 (override) | errors rose on most pages, up to 0 -> 330, all in `ds-usage`, exit 1 |
| head | 0.6.2 | `design gate: new families ds-usage — not counted until a person re-records with bee design gate --write` then `design gate: no regressions on 51 pages`, exit 0; last-run `{'status': 'ran', 'pages': 51, 'regressions': 0}` |

## design-os examples (9 charts, 19 diagrams, 1 flow model from `tests/fixtures/diagram/product-flow-real.json`)

| Step | Result |
|---|---|
| main `--write` (pre-feature baseline, no artifacts key) | `baseline written for 28 pages` |
| head gate on that baseline | `design gate: no regressions on 28 pages and 29 artifacts`, exit 0 |
| head `--write` | `baseline written for 28 pages and 29 artifacts`; families a11y, autofix, content, layout, taste, tell; artifacts chart 9, diagram 19, flow 1 |

## Not run live

- A planted artifact regression: the write guard refused edits to the scratch copy (it holds its
  own `.bee`). The rise and new-artifact paths are proven by the doc-1 unit tests on canned tool
  output; the real `ui chart lint` error shape was seen on `ads-report.html` (plan claim 7).
- A motion (`scrub`) page: no sample upstream; covered by the doc-1 unit tests only.
