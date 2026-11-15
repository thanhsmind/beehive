# dcl-2 — tester report for decision-concept-links

Status: **DONE** (round 2). Round 1 was BLOCKED on `concept_link_other_feature_d_number_on_the_same_line_does_not_link`; dcl-1's fix (commit 2174dc8af) makes it pass. No product code was changed by this cell.

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
| `concept_link_other_feature_d_number_on_the_same_line_does_not_link` | leader case: `feat-x D1 and other-feat D5`; `feat-x D1` links, `feat-x D5` must not | pass (failed in round 1, before 2174dc8af) |

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- concept_link`, run at 2174dc8af:

```
running 14 tests
test verbs::decisions::tests::concept_link_missing_knowledge_dir_searches_without_links ... ok
test verbs::decisions::tests::concept_link_decisions_active_output_never_carries_concepts ... ok
test verbs::decisions::tests::concept_link_decision_without_d_number_links_any_feature_d_line ... ok
test verbs::decisions::tests::concept_link_feature_d_number_matches_lists_and_ranges_only ... ok
test verbs::decisions::tests::concept_link_four_citing_concepts_give_three_with_id_hits_first ... ok
test verbs::decisions::tests::concept_link_comma_list_links_d2_and_not_d3 ... ok
test verbs::decisions::tests::concept_link_root_without_knowledge_dir_returns_results_without_concepts_key ... ok
test verbs::decisions::tests::concept_link_short_id_in_a_concept_gets_concepts_and_knowledge_line ... ok
test verbs::decisions::tests::concept_link_caps_at_three_with_id_hits_first ... ok
test verbs::decisions::tests::concept_link_unlinked_result_keeps_bytes_without_concepts_key ... ok
test verbs::decisions::tests::concept_link_other_feature_d_number_on_the_same_line_does_not_link ... ok
test verbs::decisions::tests::concept_link_range_covers_d3_and_not_d4 ... ok
test verbs::decisions::tests::concept_link_slash_list_links_d5_and_not_d3 ... ok
test verbs::decisions::tests::concept_link_slug_that_is_only_a_word_prefix_does_not_link ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 5234 filtered out; finished in 0.00s
```

14 = 6 tests from dcl-1 + 8 added here. All 8 new names show up in the run output.

## Round 1 defect (fixed by dcl-1 in 2174dc8af)

`line_cites_feature` passed the whole rest of the line after the slug to `d_tokens`, so `other-feat D5` counted for `feat-x`. dcl-1 now reads only the D-token run right after the slug; the test above is green against that fix.

## Live run

The built binary (`packages/bee-rs/target/release/bee`, built from 2174dc8af; `.bee/bin/bee` was not rebuilt), at the repo root:

`bee decisions search --text "trust-layers D1" --json` → exit 0, 610 results, 348 of them with `concepts`, 15 of those naming `host-project-artifacts.md`. Decision `420c8c96` (`trust-layers D1: onboarding detects a host project's static checks…`):

```
{"id": "420c8c96-ef15-4522-bc96-b7109044af2b", "concepts": ["docs/knowledge/areas/onboarding/host-project-artifacts.md:326"]}
```

Text form prints `  knowledge: docs/knowledge/areas/onboarding/host-project-artifacts.md:326`. Line 326 holds `trust-layers D1/D5)`. **Yes**: a concepts entry names `docs/knowledge/areas/onboarding/host-project-artifacts.md`. Round 1 counted 365 results with `concepts` out of 609; the drop to 348 is consistent with the fix no longer linking D tokens that follow another slug on the same line (not traced decision by decision).

Missing-dir exit code, live (round 1 run, matcher change does not touch this path): a temp root with `.git`, `.bee/onboarding.json`, one decision and no `docs/` → `decisions search --text feat-x --json` prints the decision with no `concepts` key, `exit=0`.

## Departures

- Exit 0 with no `docs/knowledge` is shown by a live binary run, not a unit test: `run_active_or_search` resolves the root from the process cwd, so an in-process test would have to change the cwd. The unit test covers the output shape through `search_output`.
