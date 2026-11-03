---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Token thrift

## Summary

The text bee injects at session start gets smaller and always fits the host's
inline limit. Today the limit is passed on this repo, so Claude Code hides most
of the text behind a file link. Discovery maps with nothing to resume fold into
one line. The dispatch door lists roles per model on one line each. `bee
orient` names 3 promote proposals, not 74. A knowledge concept records the
rules.

Mode: `standard` — 2 risk flags: covered-contract-change (tests pin the
preamble lines, the dispatch door lines, and the orient blocker line),
multi-domain (session hooks and the orient verb).
Why this is the least workflow that protects the work: every change is a
text renderer with unit tests beside it; a wrong shape costs one more cell, and
no data is changed.

Playbook: `skills/bee-planning/playbooks/feature.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1: `PREAMBLE_BUDGET_BYTES` = 9500; a named constant for the 10,000-char Claude Code inline limit; a test asserts budget < limit.
- D2: over budget, drop whole sections in order Recent decisions, Critical patterns (digest), Command surface, Project map, until it fits; then one line before the trailer naming the dropped sections and `bee status --json` / `bee --help --names`. Never drop header, gates, HANDOFF, debt counts, dispatch door, knowledge context, discovery maps.
- D3: preamble discovery maps with 0 frontier tickets fold into one count line pointing to `bee discovery list`; a map with frontier tickets keeps its line.
- D4: dispatch door prints one line per distinct rendered target with all its roles, no 60-char cut; compaction reads the same function.
- D5: orient promote-proposal blocker names at most 3 features with clear commands, then `+N more`.
- D6: knowledge concept on the token economy of injected context; budgets on generated payloads only (decision 8f63adb4).

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The preamble budget is above the host inline limit | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:65 | pub(crate) const PREAMBLE_BUDGET_BYTES: usize = 14336; |
| 2 | Claude Code moved this session's preamble to a file | ran | SessionStart hook output of this session | Output too large (10.8KB). Full output saved to: |
| 3 | The preamble on this repo is over 10,000 characters | ran | bee hook session-init piped to wc -c, from main | 11161 |
| 4 | Each discovery map gets its own preamble line | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:458-461 | lines.push(format!( / "### Open discovery map(s): {} — {} frontier ticket(s)", |
| 5 | Dispatch door lines are cut at 60 chars per role | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:554 | let line = if line.chars().count() > 60 { |
| 6 | Compaction re-injects the same door lines | read | packages/bee-rs/crates/bee/src/hooks/compaction.rs:1763 | assert!(text.contains("- review → opus (native)"), "{text}"); |
| 7 | The orient promote blocker names every feature | read | packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:514 | "promote proposal unapplied: {} feature(s) — {named} — after you apply or decide to skip the proposals: {}", |
| 8 | The budget test builds a pathological store and asserts the cap | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs:854 | text.len() <= PREAMBLE_BUDGET_BYTES, |
| 9 | The per-turn reminder is already deduped by hash | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:1047 | /// inject.mjs shouldInject: hash changed, or >30 min since last injection. |
| 10 | The preamble tests pin the door line shape | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs:1461 | assert!(text.contains("- review → opus (native)"), "{text}"); |
| 11 | The budget test is green on the base | ran | cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee the_preamble_stays_inside_its_budget | completed (exit code 0) |

## Discovery

The leader rendered the real preamble from main and measured each section
(header 3082, command surface 2601, dispatch door 1090, decisions 803, nine
discovery-map lines about 600). The per-turn reminder, the inject cache, and
the budget test were read in place. Decision 8f63adb4 forbids budgets on
authored text, so D1 and D2 bound the generated preamble only.

## Approach

tt-1 changes the budget, adds the drop guard at the end of
`build_session_preamble_with_runtime`, and folds zero-frontier maps. tt-3
changes the orient blocker. They touch disjoint files and run in parallel.
tt-2 regroups the dispatch door; it edits `session_preamble/tests.rs`, which
tt-1 also edits, so it runs after tt-1. tt-4 documents all after the code.

Rejected: cutting the command surface or principle lines for good (decisions
csc-1 and preamble-surface-slim chose to keep them; the drop guard removes the
surface only when the store crowds it out); budgets on skills or AGENTS.md
(8f63adb4); trimming host history (the host owns the request).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Drop guard removes a section a test pins | MEDIUM | tt-1 | the pathological test still finds HANDOFF, debt counts, doc links |
| Door regroup breaks compaction text | LOW | tt-2 | compaction test updated and green |
| Orient blocker loses a clear command | LOW | tt-3 | test with 5 proposals shows 3 commands and `+2 more` |

Waves: wave 1 = tt-1 and tt-3 (disjoint files); wave 2 = tt-2 (shares
`session_preamble/tests.rs` with tt-1); wave 3 = tt-4 (documents all).
Leader live proof after wave 2: build the binary, render the preamble on this
repo, and show its size under 9500 bytes.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"tt-1 to tt-3 change Rust renderers and their tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each code cell; the live proof is the leader's."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"tt-4 writes the knowledge concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done; gathers only on demand."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard plan check."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Three seats on standard work."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Three seats on standard work."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard plan check."}
  ]
}
```

## Shape

Epic map. Outcome: what bee injects reaches the agent in full and costs fewer
tokens.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Fit | budget under the host limit, drop guard, map fold | the preamble is hidden behind a file today | tt-1 | budget tests + live size |
| Door | one line per target | 20 cut lines | tt-2 | model_guard, preamble, compaction tests |
| Orient | capped promote blocker | 74 features on one line | tt-3 | orient tests |
| Record | knowledge concept | one home for the rules | tt-4 | pointer check |

One slice; every cell is current.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| tt-1 | Fit the session preamble under the host inline limit | hooks/session_preamble/budget.rs, hooks/session_preamble/tests.rs, hooks/session_init.rs | — | the session preamble stays under 9500 bytes and names any section it dropped | cargo test session_preamble |
| tt-2 | Group dispatch door roles by target | hooks/model_guard.rs, hooks/compaction.rs, hooks/session_preamble/tests.rs | tt-1 | the dispatch door shows one line per model with every role in full | cargo test model_guard, compaction, session_preamble |
| tt-3 | Cap the promote-proposal blocker in orient | verbs/status_full/orient.rs, verbs/status_full/tests.rs | — | `bee orient` names 3 promote proposals and `+N more` | cargo test status_full |
| tt-4 | Document the token economy of injected context | docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md (new), docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md | tt-1, tt-2, tt-3 | one concept explains the limits | pointer check |

Paths under `hooks/` and `verbs/` are in `packages/bee-rs/crates/bee/src/`. The cell packet:

```json
[
  {
    "id": "tt-1",
    "feature": "token-thrift",
    "title": "Fit the session preamble under the host inline limit",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "D3"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs"
    ],
    "read_first": [
      "docs/history/token-thrift/CONTEXT.md",
      "docs/history/token-thrift/reports/hat-wave-synthesis.md",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"],
    "action": "Step 0, before any other change: in a scratch test, render the pathological fixture of `the_preamble_stays_inside_its_budget_however_big_the_store_gets` with Recent decisions, Critical patterns (digest), Command surface and Project map removed and print its byte length; if it is over 9500, stop and return [BLOCKED] with the measured size per section. Remove the scratch test after. In budget.rs: (1) D1: add `pub(crate) const HOST_INLINE_LIMIT_CHARS: usize = 10_000;` and set `PREAMBLE_BUDGET_BYTES` to 9500; update the doc lines above the constant to state the Claude Code inline limit (hook output over 10,000 characters is moved to a file with a 2 KB preview) and drop the old measured numbers that no longer hold. Doc lines are `///` on items only where the file already uses them; add no new `//` comments. Make PREAMBLE_BUDGET_BYTES used outside tests (remove its `cfg_attr(not(test), allow(dead_code))`) because the guard below reads it. (2) D3: change `open_maps_lines` so efforts with `frontier > 0` keep today's line `### Open discovery map(s): {name} — {frontier} frontier ticket(s)`, and all efforts with frontier 0 fold into ONE line `### Open discovery map(s): {n} with no frontier ticket — bee discovery list`; the unreadable lines stay as they are. (3) D2: in `build_session_preamble_with_runtime`, build the sections Recent decisions, Critical patterns (digest), Command surface and Project map as separately removable blocks (each block = its blank separator line plus its lines), keep today's order in the output, and after assembly, while the joined text (including the final trailer line) is longer than PREAMBLE_BUDGET_BYTES bytes, remove the next block in this order: Recent decisions, Critical patterns (digest), Command surface, Project map. When at least one block was removed, insert, right before the blank line that precedes the closing trailer, a blank line and the line `### Omitted to fit the host limit: ` followed by one entry per dropped section, joined by `; `, each naming where to fetch it: `Recent decisions → .bee/bin/bee status --json`, `Critical patterns → docs/knowledge/index.md`, `Command surface → .bee/bin/bee --help --names`, `Project map → docs/knowledge/index.md`. A dropped block takes its blank separator line with it. Never remove any other section (Doc links, knowledge context and discovery maps stay). When nothing is over budget the output must be byte-identical to today except for the D3 change. (4) The budget bounds the WHOLE hook stdout, not only the preamble: add `pub fn build_session_preamble_within(root, session_id, handoff_outcome, runtime, budget_bytes: usize)` that holds the logic, and make `build_session_preamble_with_runtime` call it with PREAMBLE_BUDGET_BYTES. In session_init.rs, compute the belt line (`belt_contract_line`) and the intent anchor (`intent_lead_block`) BEFORE the body, and call the preamble builder with `PREAMBLE_BUDGET_BYTES.saturating_sub(<belt bytes + anchor bytes + the \"\\n\\n\" separators actually used>)`; the output order (belt, anchor, body) and bytes stay the same as today when nothing is over budget. The compact capsule path (`build_compact_capsule_with_runtime`) is unchanged. In tests.rs: keep `the_preamble_stays_inside_its_budget_however_big_the_store_gets` and its content asserts; add asserts that the text contains `### Omitted to fit the host limit:` and that `PREAMBLE_BUDGET_BYTES < HOST_INLINE_LIMIT_CHARS`. Add tests: over-budget drops follow the order (with a budget that forces exactly one drop, only Recent decisions is gone and named; with a tighter one, Recent decisions and Critical patterns are gone and Command surface stays); a dropped section's heading is absent from the text; the composed session-init output for a `resume` start with an intent anchor of about 1500 characters on the pathological store is at most 9500 bytes; a quiet repo renders no Omitted line; the existing tests that assert Command surface, Critical patterns or Recent decisions are present (tests.rs near 340-345, 468-469, 1394) stay green; three discovery maps with 0 frontier and one with 2 render one fold line `### Open discovery map(s): 3 with no frontier ticket — bee discovery list` plus the one map's own line; the existing discovery-map tests near lines 1420-1430 stay green. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_preamble && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_init",
    "must_haves": {
      "truths": [
        "PREAMBLE_BUDGET_BYTES is 9500 and a test asserts it is below HOST_INLINE_LIMIT_CHARS",
        "the pathological store renders at most 9500 bytes and still names HANDOFF, both debt counts and doc links",
        "an over-budget render names every dropped section in one Omitted line before the trailer",
        "a quiet repo renders no Omitted line",
        "the whole session-init stdout on a resume with a long intent anchor stays at most 9500 bytes",
        "each Omitted entry names its own fetch path",
        "zero-frontier maps fold into one count line; a map with frontier tickets keeps its own line"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs", "substantive": "budget constants, drop guard, map fold"}
      ],
      "key_links": ["build_session_preamble_with_runtime applies the drop guard before it returns"],
      "prohibitions": [
        "No section other than the four named may be dropped",
        "No change to the closing trailer text",
        "No code comments"
      ]
    }
  },
  {
    "id": "tt-2",
    "feature": "token-thrift",
    "title": "Group dispatch door roles by target",
    "lane": "standard",
    "role": "code",
    "deps": ["tt-1"],
    "decisions": ["D4"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "packages/bee-rs/crates/bee/src/hooks/compaction.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs"
    ],
    "read_first": [
      "docs/history/token-thrift/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"],
    "action": "In model_guard.rs `dispatch_door_lines` (near line 542): keep the first line and the last `- open set:` line unchanged. Replace the per-role lines with one line per distinct rendered target from `role_slot_display`, in the order each target first appears, shaped `- {rendered}: {role1}, {role2}, ...` with roles in their existing order, and remove the 60-character truncation. `- none configured` stays when there are no roles. Update every test that pins the old shape: model_guard.rs `assert_eq!` pins near 3692-3695 and 3745 and the list near 3730; rewrite `dispatch_door_renders_all_18_roles_under_60_chars` (near 4122; it checks `- {expected} → ` at 4169 and the 60-char cap at 4175) so it asserts every one of the 18 roles appears in exactly one grouped line and no line carries `…`, and rename it to match; compaction.rs pins near 1762-1764; session_preamble/tests.rs pins near 1460 (the cut text `(herding: agy-flas…`), 1461, 1497-1509, 1524-1527, 1542-1545, 1564-1566, 1584, 1585, 1599 and 1632. Run `rg -n ' → ' packages/bee-rs/crates/bee/src/hooks` after the change and fix any remaining pin of the old door shape. Assert the new grouped line, for example `- opus (native): review` or a line that starts with `- opus (native): ` and contains `design`. Add one test where two roles share a target and one long herding target renders in full on one line with both role names and no `…`. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee model_guard && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee compaction && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_preamble",
    "must_haves": {
      "truths": [
        "the dispatch door prints one line per distinct target listing all its roles",
        "no dispatch door line is cut with an ellipsis",
        "the compaction re-inject prints the same grouped lines"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/model_guard.rs", "substantive": "grouped dispatch_door_lines and its tests"}
      ],
      "key_links": ["compaction and the session preamble both call dispatch_door_lines"],
      "prohibitions": ["No change to the prepare command line or the open set line", "No code comments"]
    }
  },
  {
    "id": "tt-3",
    "feature": "token-thrift",
    "title": "Cap the promote-proposal blocker in orient",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D5"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
    ],
    "read_first": [
      "docs/history/token-thrift/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"],
    "action": "In orient.rs, the promote-proposal blocker (near line 506): add `const PROMOTE_PROPOSALS_SHOWN: usize = 3;`. Name only the first PROMOTE_PROPOSALS_SHOWN proposals (same order as today) in the `{named}` part and in the clear-command part; when more exist, append `; +N more` to the named part (N = total - shown). The count `{} feature(s)` stays the total. With 3 or fewer proposals the line is byte-identical to today, so the existing test near tests.rs:1246 stays green. Add a test with 5 proposals: the line starts with `promote proposal unapplied: 5 feature(s) — `, names exactly 3 features, contains `+2 more`, and carries exactly 3 clear commands. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee status_full",
    "must_haves": {
      "truths": [
        "orient names at most 3 promote proposals and states +N more",
        "the feature count stays the total",
        "3 or fewer proposals render exactly as before"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs", "substantive": "capped promote blocker"}
      ],
      "key_links": ["the blocker reads unapplied_promote_proposals once"],
      "prohibitions": ["No change to other blocker lines", "No code comments"]
    }
  },
  {
    "id": "tt-4",
    "feature": "token-thrift",
    "title": "Document the token economy of injected context",
    "lane": "standard",
    "role": "docs",
    "deps": ["tt-1", "tt-2", "tt-3"],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D6"],
    "files": [
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md",
      "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"
    ],
    "read_first": [
      "docs/history/token-thrift/CONTEXT.md",
      "docs/history/token-thrift/reports/hat-wave-synthesis.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"],
    "action": "Write a new concept in the frontmatter shape and style of the other files in docs/knowledge/areas/hook-runtime/ (ASD-STE100, short sentences). Content: bee does not own the host request, so it saves tokens by what it injects and prints; the Claude Code inline limit (10,000 characters; larger hook output goes to a file with a 2 KB preview) and why PREAMBLE_BUDGET_BYTES is 9500; the drop order and the Omitted line; zero-frontier map fold; the grouped dispatch door; the capped orient promote blocker; what was already in place (per-turn reminder deduped by hash with a 30-minute re-inject in hooks/prompt_context.rs, append-only per-turn text keeps the cached prefix, each cell runs in a fresh dispatched worker); and the boundary: budgets bind generated payloads only, never authored instruction text (decision 8f63adb4-de70-424e-b082-1f4c5ec4b693). State that the budget binds the whole session-start hook stdout (belt line, intent anchor and preamble) on every runtime, because the builder is shared and the smallest host limit sets it; the compact capsule is not budgeted yet. Each Omitted entry names its own fetch path. In model-roles-and-escalation.md, replace the statement that door lines are at most 60 characters per line (near line 161) with the grouped shape: one line per target, roles in full. Out of scope: trimming host history, wait timing tuned to cache TTL. Pointers: hooks/session_preamble/budget.rs, hooks/model_guard.rs, hooks/prompt_context.rs, verbs/status_full/orient.rs, docs/history/token-thrift/CONTEXT.md. Run `.bee/bin/bee knowledge check` if it exists and fix what it reports for this file.",
    "verify": "rg -n 'HOST_INLINE_LIMIT_CHARS|9500|Omitted to fit|8f63adb4|should_inject' docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md",
    "must_haves": {
      "truths": [
        "the concept names the host limit, the budget, the drop order and the decision boundary",
        "Pointers list budget.rs, model_guard.rs, prompt_context.rs and orient.rs"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md", "substantive": "the concept"}
      ],
      "key_links": ["the concept cites decision 8f63adb4"],
      "prohibitions": ["No code comments added anywhere"]
    }
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Happy path | quiet repo | tt-1 | no Omitted line; output unchanged except the map fold |
| Edge | pathological store | tt-1 | at most 9500 bytes; HANDOFF and debt counts present; Omitted line present |
| Edge | maps with and without frontier | tt-1 | one fold line plus one own line |
| Edge | two roles share a long herding target | tt-2 | one line, both roles, no `…` |
| Edge | 5 promote proposals | tt-3 | 3 named, `+2 more`, count 5 |
| Regression | 1 promote proposal | tt-3 | line identical to today |
| Regression | each code cell's test filter | tt-1..tt-3 | the cell's verify command green |
| Live | preamble rendered on this repo after wave 2 | leader | size under 9500 bytes |

## Open Questions

(none)

## Out of scope

- Trimming or dropping host history.
- Wait timing tuned to the cache TTL.
- Byte budgets on skills or AGENTS.md (decision 8f63adb4).
- The compact capsule (`build_compact_capsule_with_runtime`), a separate path on `compact` starts; it keeps no budget in this feature.
- A different budget per runtime; one budget serves every runtime because the builder is shared.
- The `bee status` text renderer's discovery-map lines (`verbs/status_full/render.rs`); it is printed on request, not injected.
