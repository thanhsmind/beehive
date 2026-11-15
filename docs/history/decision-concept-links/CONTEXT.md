# decision-concept-links — context

Request (verbatim): "Sửa bee decisions search: để nó chỉ thẳng tới trang
knowledge thì phải sửa code. Phần này vẫn mở trong PBI p-eb824507, và mình
đã sửa PBI để nó chỉ còn phần này."

cite-knowledge-first D1 says a decision hit leads to the concept that cites
its D-number before docs/history is opened. Today `bee decisions search`
prints no concept, so an agent that finds a decision goes straight to
docs/history. This feature makes the search print the concept lines that
cite each result. Backlog item: p-eb824507.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/decision-memory/overview.md | suspect | R5: search is the recall surface, and the reading order per area is spec, then decision index, then history. |
| docs/knowledge/areas/decision-memory/attribution.md | suspect | The `feature` field on a decision event and where it comes from. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs` | `run_active_or_search`: builds the result list, the JSON object and the text. |
| `packages/bee-rs/crates/bee/src/verbs/decisions/scanners.rs` | `format_decision`: the text lines of one result. |
| `packages/bee-rs/crates/bee/src/verbs/decisions/read.rs` | `active_decisions` and `filter_decision_events`. |
| `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | The `decisions search` help text. |

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | `bee decisions search` links each result to the `docs/knowledge` concepts that cite it. A concept line cites a decision when it holds the decision's 8-character short id, or the decision's feature slug as a whole word followed in the same line by a D-number token. The D-number comes from the text prefix `<slug> D<n>`. When the D-number is known, a feature line counts only when one of its D tokens equals it or a D range covers it (`D1-D3`, `D1/D5`, `D1, D2`). The feature slug comes from the event's `feature` field, else from the text prefix. At most three concepts per decision, one `path:line` each (the first matching line). Id hits rank before feature hits, then path order. | Measured on this store: 555 of 3443 decisions have their short id in a concept, 1838 have their feature slug in one. A stored index was rejected: one more derived file to keep fresh, for a scan that costs milliseconds. |
| D2 | Contract `decision-concept-links`. In `--json`, a search result with a link gains a `concepts` array of repo-relative `path:line` strings under `docs/knowledge/`. A result with no link has no `concepts` key, so its JSON is byte-identical to today. Text output adds one line `  knowledge: <path:line>[, <path:line>]` after the why and alternatives lines, only when a link exists. `decisions active` does not change. A missing `docs/knowledge` directory gives no links and never fails the search. Proving tests carry `concept_link` in their names. | Additive: today's consumers keep their bytes. Search is the recall path the doctrine names. |

## Out of scope

- `decisions active`, `decisions render` and the decision index.
- A stored or cached concept index.
- A release. The owner did not ask for one.
