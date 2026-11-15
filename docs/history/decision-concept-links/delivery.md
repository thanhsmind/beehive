# decision-concept-links — delivery

## What changed

- `packages/bee-rs/crates/bee/src/verbs/decisions/concept_links.rs` (new): one scan reads every `*.md` under `docs/knowledge/` (a missing directory gives none); one function returns up to three `path:line` strings per event. Id hits (8-character short id) rank first, then feature hits, then path order, first matching line per concept (D1).
- `verbs_read.rs`: `search_output` adds the `concepts` array and the `  knowledge: <path:line>[, <path:line>]` line to a linked result. `run_active_or_search` calls it only on the search path; `decisions active` is untouched (D2).
- `generated/registry_payload.json`: the `decisions search` description names the `concepts` key and the knowledge line.
- `docs/knowledge/areas/decision-memory/overview.md`: R5 gains one sentence.

## Proof

- Tests named `concept_link` in `decisions/tests.rs`: short id link and text line, feature D-number with list and range matching, unlinked result byte-identical, missing knowledge directory, three-link cap with id hits first, and `decisions active` never carrying concepts.
- Red first: with a stub that returned no links, the three linking tests failed and the three no-link tests passed. After the real function, all six pass.
- Full suite: `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`, no failures.

## Known limit

A feature line counts when a D token follows the slug on the same line; a line citing several features is matched per slug occurrence, to the end of the line.
