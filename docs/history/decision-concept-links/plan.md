---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: decision-concept-links

## Summary

`bee decisions search` now shows, under each decision it finds, the
knowledge concept lines that cite that decision. An agent that finds a
decision can open the current home of the rule first, and open
docs/history only for the reason. In `--json` a linked result gains a
`concepts` array. An unlinked result keeps today's bytes.

Mode: `small`. Class feature, 0 flags. Product files: `verbs_read.rs`,
`scanners.rs` (or one new sibling module), the registry help text.
Why this is the least workflow that protects the work: one additive read
path with a pinned contract, an implementer and a separate tester.

Playbook: `skills/bee-planning/playbooks/feature/feature.md` (class `feature`, situation `feature`).

## Requirements (from CONTEXT.md)

- D1: the link rule (short id, or feature slug plus a matching D token), at most three concepts, ranking.
- D2: contract `decision-concept-links`: the `concepts` key only when linked, one `knowledge:` text line, `decisions active` unchanged, no failure without `docs/knowledge`.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | One function serves both search and active | read | `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs:82` | `pub(crate) fn run_active_or_search(` |
| 2 | The JSON object holds the result list under one key | read | `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs:192` | `Ok(Out::Emit(json!({ "decisions": decisions }), text_out, 0))` |
| 3 | One function renders the text of a result | read | `packages/bee-rs/crates/bee/src/verbs/decisions/scanners.rs:454` | `pub(crate) fn format_decision(event: &Value) -> String {` |
| 4 | The search help text lives in the generated registry | ran | `rg -l "Search active decisions by multi-term"` | `packages/bee-rs/crates/bee/src/generated/registry_payload.json` |

## Discovery

3443 active decide or supersede events; 1921 carry a `feature` field and
653 start with `<slug> D<n>`. Concepts cite decisions in two shapes: the
8-character short id (`af14e1d6`) and `<feature> D<n>` with lists and
ranges (`trust-layers D1/D5`, `skillstation-more-skills D1-D3`,
`knowledge-first-lean-decisions D1, D2`).

## Approach

Recommended path. dcl-1 adds a concept scan that reads every `*.md` under
`docs/knowledge/` once per search call, links each result by D1, adds the
`concepts` key and the `knowledge:` line only on the search path (D2),
updates the help text and the decision-memory concept, with tests named
`concept_link`. dcl-2, a separate tester, adds the edge-case tests of the
contract and runs the proof.

Rejected: a stored concept index (D1); linking in `decisions active`
(out of scope).

Risk map:
- an unlinked result changes bytes / MEDIUM / Lands in dcl-1 / a test compares an unlinked result's JSON with no `concepts` key.
- a D range or list is misread and links the wrong decision / MEDIUM / Lands in dcl-2 / tests for `D1-D3`, `D1/D5`, `D1, D2` and a non-covering number.
- a repo with no `docs/knowledge` fails / LOW / Lands in dcl-2 / a test on an empty root.

Waves: dcl-1, then dcl-2 (dcl-2 tests dcl-1's code).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"dcl-1 writes the concept scan, the output change and its first tests."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"dcl-2 is the separate tester: edge-case tests of the contract and a live run."},
    {"stage":"documentation","classification":"not-applicable","role":"docs","reason":"dcl-1 updates the concept and the help text with the code."},
    {"stage":"knowledge-consolidation","classification":"not-applicable","role":"writer","reason":"dcl-1 owns the concept edit and delivery.md."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release unless the owner asks."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Plan check runs inline at this size."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Inline check at this size."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Inline check."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Inline check."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Inline check."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Inline check."},
    {"stage":"design","classification":"not-applicable","role":"design","reason":"No rendered UI changes."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | search results name the concept lines that cite them | Owner request 2026-11-15, p-eb824507 | `bee decisions search --text "trust-layers D1"` prints a `knowledge:` line naming `docs/knowledge/areas/onboarding/host-project-artifacts.md` | the source order of cite-knowledge-first D1 has a tool behind it |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| dcl-1 | Link decision search results to the concepts that cite them | `verbs_read.rs`, `scanners.rs`, `concept_links.rs` (new), `mod.rs`, `tests.rs`, registry payload, concept, delivery | — | a search for a trust-layers decision prints a `knowledge:` line | `cargo test … --bin bee -- concept_link` |
| dcl-2 | Test the decision-concept-links contract edges | `tests.rs`, report | dcl-1 | tests for ranges, lists, a non-covering number, no knowledge dir, the three-concept cap | `cargo test … --bin bee -- concept_link` |

```json
[
  {
    "id": "dcl-1",
    "feature": "decision-concept-links",
    "title": "Link decision search results to the concepts that cite them",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "d9374293-0ef8-4b54-8770-66cf13d555bc", "c1124936-188a-48fd-9d48-8421d93681e9"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/decision-memory/overview.md"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/scanners.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/concept_links.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "docs/knowledge/areas/decision-memory/overview.md",
      "docs/history/decision-concept-links/delivery.md"
    ],
    "read_first": [
      "docs/history/decision-concept-links/CONTEXT.md",
      "docs/knowledge/areas/decision-memory/overview.md",
      "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/scanners.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs"
    ],
    "action": "RED FIRST in packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs, beside the existing search tests. Build a temp repo with a decisions store and docs/knowledge/areas/x/a.md and b.md. Tests named with concept_link: (a) a decision whose short id appears in a.md gets concepts [\"docs/knowledge/areas/x/a.md:<line>\"] in --json and a text line '  knowledge: docs/knowledge/areas/x/a.md:<line>' after why/alternatives; (b) a decision with feature field 'feat-x' and text 'feat-x D2: ...' links to a line 'feat-x D1-D3' and not to a line 'feat-x D4'; (c) a decision with no link has no concepts key and its text has no knowledge line; (d) decisions active output never carries concepts. Run them and record the red output. FIX: add concept_links.rs in the decisions module: one function reads every *.md under <root>/docs/knowledge/ once (missing dir gives empty), and one function returns up to three 'path:line' strings for an event per D1 (short id = first 8 chars of id; feature from the feature field else the '<slug> D<n>' text prefix; D tokens on a line parsed after the slug: single Dn, lists separated by '/' or ', ', ranges 'Da-Db' or 'Da-b'; with a known n, require a token equal to n or a range covering it; without n, any D token; first matching line per concept; id hits first, then path order). In run_active_or_search, only when is_search, insert the concepts array into each linked result before emit, and append the knowledge line to that result's text. Keep decisions active untouched. Update the decisions search description in generated/registry_payload.json with one sentence naming the concepts key and the knowledge line (keep the file's JSON form; if a registry test pins the text, update it in the same commit). Add to docs/knowledge/areas/decision-memory/overview.md R5 one sentence: search results name the concept lines that cite them (decision-concept-links D1, D2). Write docs/history/decision-concept-links/delivery.md. No code comments. One commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- concept_link",
    "must_haves": {
      "truths": [
        "a search result whose short id or feature D-number is cited in a concept carries concepts and a knowledge line",
        "a search result with no link keeps today's JSON and text bytes",
        "decisions active output is unchanged",
        "a repo without docs/knowledge searches without error"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/decisions/concept_links.rs", "substantive": "the concept scan and the link rule"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs", "substantive": "concepts key and knowledge line on the search path only"}
      ],
      "key_links": [
        "the link rule follows decision-concept-links D1",
        "the output shape follows contract decision-concept-links"
      ],
      "prohibitions": [
        "No code comments",
        "No stored concept index",
        "No change to decisions active"
      ],
      "open_choices": [
        "Helper names",
        "How the D-token parser is written"
      ]
    }
  },
  {
    "id": "dcl-2",
    "feature": "decision-concept-links",
    "title": "Test the decision-concept-links contract edges",
    "lane": "small",
    "role": "test",
    "deps": ["dcl-1"],
    "decisions": ["D1", "D2", "c1124936-188a-48fd-9d48-8421d93681e9"],
    "affects_skills": [],
    "affects_specs": [],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs",
      "docs/history/decision-concept-links/reports/dcl-2.md"
    ],
    "read_first": [
      "docs/history/decision-concept-links/CONTEXT.md",
      "docs/history/decision-concept-links/delivery.md",
      "packages/bee-rs/crates/bee/src/verbs/decisions/concept_links.rs"
    ],
    "action": "Separate tester for dcl-1: do not change product code. Add tests named with concept_link in packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs for: the list forms 'feat-x D1/D5' and 'feat-x D1, D2' (D5 and D2 link, D3 does not); a range 'feat-x D1-D3' covers D3 and not D4; a slug that is only a prefix of another word does not link ('feat-x' vs 'feat-xy D2'); a decision with no D-number links to any 'feat-x D<n>' line; four citing concepts give exactly three, id hits first; a root with no docs/knowledge returns results with no concepts key and exit 0. If a test exposes a defect, return BLOCKED with the failing output; never fix product code. Run the verify, then a live run: .bee/bin/bee is not rebuilt, so run the built binary from the cargo target dir with 'decisions search --text \"trust-layers D1\" --json' at the repo root and record whether a concepts entry names docs/knowledge/areas/onboarding/host-project-artifacts.md. Write docs/history/decision-concept-links/reports/dcl-2.md with each test name, the verify output count and the live run output. One commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- concept_link",
    "must_haves": {
      "truths": [
        "list, range, prefix, no-D-number, cap and missing-dir cases each have a passing concept_link test",
        "the report quotes the verify pass count and the live run output"
      ],
      "artifacts": [
        {"path": "docs/history/decision-concept-links/reports/dcl-2.md", "substantive": "tester report"}
      ],
      "key_links": ["dcl-2 tests dcl-1"],
      "prohibitions": ["No product code edits", "No code comments"],
      "open_choices": ["Fixture layout"]
    }
  }
]
```
