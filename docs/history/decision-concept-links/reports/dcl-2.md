# dcl-2 — tester report for decision-concept-links

Status: **BLOCKED**. One contract test fails and exposes a defect in dcl-1's matcher. No product code was changed.

## Tests added (`packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs`)

| Test | Case | Result |
|---|---|---|
| `concept_link_slash_list_links_d5_and_not_d3` | `feat-x D1/D5`: D5 links, D3 does not | pass |
| `concept_link_comma_list_links_d2_and_not_d3` | `feat-x D1, D2`: D2 links, D3 does not | pass |
| `concept_link_range_covers_d3_and_not_d4` | `feat-x D1-D3`: D3 links, D4 does not | pass |
| `concept_link_slug_that_is_only_a_word_prefix_does_not_link` | `feat-xy D2` never links `feat-x D2`, does link `feat-xy D2` | pass |
| `concept_link_decision_without_d_number_links_any_feature_d_line` | no D-number: skips `feat-x alone`, links `feat-x D7` | pass |
| `concept_link_four_citing_concepts_give_three_with_id_hits_first` | 2 id hits + 2 feature hits give 3, id hits first, text line matches | pass |
| `concept_link_root_without_knowledge_dir_returns_results_without_concepts_key` | no `docs/knowledge`: JSON equals the input events, no `knowledge:` line | pass |
| `concept_link_other_feature_d_number_on_the_same_line_does_not_link` | leader case: `feat-x D1 and other-feat D5`; `feat-x D1` links, `feat-x D5` must not | **FAIL** |

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- concept_link`

```
running 14 tests
...
test verbs::decisions::tests::concept_link_other_feature_d_number_on_the_same_line_does_not_link ... FAILED
...
---- verbs::decisions::tests::concept_link_other_feature_d_number_on_the_same_line_does_not_link stdout ----
panicked at crates/bee/src/verbs/decisions/tests.rs:3024:9:
assertion `left == right` failed
  left: Array [String("docs/knowledge/a.md:1")]
 right: Null

test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 5234 filtered out
```

14 = 6 tests from dcl-1 + 8 added here. All 8 new names show up in the run output.

## Defect

`line_cites_feature` in `concept_links.rs` passes the whole rest of the line after the slug to `d_tokens`. So a D token that belongs to a later slug (`other-feat D5`) counts for `feat-x`. CONTEXT D1 says a feature line counts when "one of its D tokens equals it or a D range covers it" and names list forms only. The leader's ruling says D5 here belongs to other-feat. Possible fix (for the owner of dcl-1, not done here): stop the token scan at the first word that is not a D token, list separator (`/`, `,`) or range dash, so only the contiguous D list right after the slug counts.

## Live run

The built binary (`packages/bee-rs/target/release/bee`, `.bee/bin/bee` was not rebuilt), at the repo root:

`bee decisions search --text "trust-layers D1" --json` → exit 0, 609 results, 365 of them with `concepts`. Decision `420c8c96` (`trust-layers D1: onboarding detects a host project's static checks…`):

```
{"id": "420c8c96-ef15-4522-bc96-b7109044af2b", "concepts": ["docs/knowledge/areas/onboarding/host-project-artifacts.md:326"]}
```

Text form prints `  knowledge: docs/knowledge/areas/onboarding/host-project-artifacts.md:326`. Line 326 holds `trust-layers D1/D5)`. **Yes**: a concepts entry names `docs/knowledge/areas/onboarding/host-project-artifacts.md`.

Missing-dir exit code, live: a temp root with `.git`, `.bee/onboarding.json`, one decision and no `docs/` → `decisions search --text feat-x --json` prints the decision with no `concepts` key, `exit=0`.

## Departures

- Exit 0 with no `docs/knowledge` is shown by a live binary run, not a unit test: `run_active_or_search` resolves the root from the process cwd, so an in-process test would have to change the cwd. The unit test covers the output shape through `search_output`.
