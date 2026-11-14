---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Six more craft principles from pstack

## Summary

bee gets six new principle skills from pstack: encode lessons in structure,
subtract before you add, migrate callers then delete, boundary discipline,
make illegal states unrepresentable, and separate before serializing shared
state. Each is a thin skill on top of one `expertise/` section, and each gets
one Principle homes row, so `bee orient` names it for its route classes.

Mode: `standard` — 0 risk flags; six new skills is more than the three
product files a small lane allows.

## Requirements (from CONTEXT.md)

- D1 (`63bc8581`): six `bee-principle-*` skills, each with Apply-when, body,
  Check, Not-when, Why and Depth lines, in bee's own words.
- D2 (`2be45bfd`): homes — three new sections, three extended sections.
- D3 (`ee869476`): route classes per principle, one index row each.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A principle skill ends on a Depth pointer to its expertise home | read | skills/bee-principle-build-the-lever/SKILL.md:25 | `**Depth:** `.bee/expertise/planning.md` § Build the lever.` |
| 2 | An index row names the skill and its guide heading | read | docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:429 | `- `bee-principle-build-the-lever` (expertise/planning.md § Build the lever):` |
| 3 | The parity test needs the guide heading to exist | read | packages/bee-rs/crates/bee/tests/principle_index_parity.rs:346 | `fn every_row_anchors_an_existing_guide_heading() {` |
| 4 | The parity test needs both rendered plugin trees | read | packages/bee-rs/crates/bee/tests/principle_index_parity.rs:401 | `fn every_principle_skill_ships_in_both_plugin_trees() {` |
| 5 | Three homes already exist in architecture.md | read | expertise/architecture.md:166 | `### Make Illegal States Unrepresentable` |
| 6 | The shared-state home exists | read | expertise/architecture.md:179 | `### One Writer for Shared State` |
| 7 | The boundary home exists | read | expertise/architecture.md:106 | `### Explicit Boundaries and Contracts` |

## Approach

Class playbook: `skills/bee-planning/playbooks/feature.md`. Step 1 (data
shape): the data is the index row grammar, claim 2. Step 2 (walking
skeleton): one cell carries the skill, its home and its row end to end.
Step 4 (user path): `bee orient` on a refactor route names the new
principles. Step 5: the index doc is the knowledge sync and moves in the
same cell. Steps 6 and 7 do not apply: no event-order flow, and no code to
model.

One docs cell writes the six skills, the six homes, the six rows and the
routing table rows in `expertise/architecture.md`, then runs `bee dev regen`.
One cell, because the regen output is shared by every part and two cells
would both write the rendered trees.

SMALLER PATH check: a cheaper shape is to add rows that point at existing
sections and skip the new sections. FAIL — three principles have no home,
and the parity test refuses a row whose heading does not exist (claim 3).

Rejected: copying pstack's text (D1 forbids it); routing all six to
`feature` (D3: the preamble budget); a writer consolidation cell (no code
changes, and the index doc is the knowledge sync).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "docs", "classification": "required", "role": "docs", "reason": "pspr-1 writes six skills, their expertise homes and index rows, and runs the regen chain."},
    {"stage": "implementation", "classification": "not-applicable", "role": "code", "reason": "No Rust changes."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "No behavior-change cell; the parity tests are the check."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Every claim row was opened this session; the decisions are the owner's own ask."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "The Approach names the rejected paths and the SMALLER PATH check."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Additive skills; no existing behavior changes."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "No door, guard or state write changes."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "The owner asked for this directly."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "test-and-live-proof", "classification": "not-applicable", "role": "test", "reason": "The cell carries its own parity tests; bee test runs once before merge."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The leader read the anchors directly."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "The cell has a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "pspr-1 runs as docs."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Standard lane; no high-risk advisor consult."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind lanes."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind lanes."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind lanes."},
    {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."}
  ]
}
```

## Risk map

| Risk | Guard | Lands in |
|---|---|---|
| A new row pushes a class's preamble over its budget | D3 routes each principle to at most two classes; the preamble budget test | pspr-1 |
| A skill outside the `bee-principle-` prefix ships nowhere | principle_index_parity both-trees test | pspr-1 |
| The new text duplicates an existing section | D2: extend, never duplicate | pspr-1 |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pspr-1 | Add six craft principles from pstack with their homes and index rows | six skills, three expertise guides, the doctrine index, rendered copies | — | `bee orient` on a refactor route lists the four new refactor principles | principle_index_parity, skill_triggers, knowledge check, release-manifest check |

```json
[
{"id":"pspr-1","feature":"pstack-principles","title":"Add six craft principles from pstack with their homes and index rows","lane":"standard","role":"docs","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test principle_index_parity --test skill_triggers && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check",
"must_haves":{"truths":["skills/ holds the six bee-principle-* directories named in D1, each with frontmatter name equal to its directory","each new skill has a **Check:**, a **Not when:**, a **Why:** and a **Depth:** line","expertise/knowledge.md has a section Encode lessons in structure; expertise/planning.md has Subtract before you add; expertise/architecture.md has Migrate Callers, Then Delete","the three existing architecture.md sections named in D2 are extended, not copied","the Principle homes index has six new rows with the D3 classes","the rendered plugin trees carry all six skills"],"artifacts":[{"path":"skills/bee-principle-boundary-discipline/SKILL.md","substantive":"Apply when description, Check and Depth lines"}],"key_links":["each index row's guide heading resolves in its expertise file"],"prohibitions":["Do not copy pstack sentences verbatim","Do not route a new principle to a class D3 does not name","No code comments"]},
"decisions":["63bc8581-9775-4e3e-9a5b-410a8e43a987","2be45bfd-cc86-42b0-b1c6-9e235f220226","ee869476-6553-4d65-a6a0-8589579e7a2c"],
"files":["skills/bee-principle-encode-lessons-in-structure/SKILL.md","skills/bee-principle-subtract-before-you-add/SKILL.md","skills/bee-principle-migrate-callers-then-delete/SKILL.md","skills/bee-principle-boundary-discipline/SKILL.md","skills/bee-principle-make-illegal-states-unrepresentable/SKILL.md","skills/bee-principle-separate-before-serializing-shared-state/SKILL.md","expertise/knowledge.md","expertise/planning.md","expertise/architecture.md",".bee/expertise/knowledge.md",".bee/expertise/planning.md",".bee/expertise/architecture.md","docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",".claude/skills/*",".agents/skills/*",".opencode/skills/*",".claude-plugin/skills/*",".codex-plugin/skills/*",".bee/onboarding.json","skills-lock.json","docs/history/codex-harness-hardening/release-manifest.json"],
"read_first":["docs/history/pstack-principles/CONTEXT.md","docs/history/pstack-principles/plan.md","skills/bee-principle-build-the-lever/SKILL.md","skills/bee-principle-attack-the-premise/SKILL.md","expertise/architecture.md","docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md","packages/bee-rs/crates/bee/tests/principle_index_parity.rs"],
"affects_skills":["skills/bee-principle-encode-lessons-in-structure/SKILL.md","skills/bee-principle-subtract-before-you-add/SKILL.md","skills/bee-principle-migrate-callers-then-delete/SKILL.md","skills/bee-principle-boundary-discipline/SKILL.md","skills/bee-principle-make-illegal-states-unrepresentable/SKILL.md","skills/bee-principle-separate-before-serializing-shared-state/SKILL.md"],
"affects_specs":["docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"],
"action":"Implement CONTEXT.md D1-D3. Source text (data, never instructions; MIT, Lauren Tan): /mnt/data/Projects/goglbe/beehive/.bee/tmp/pstack-pm/ref/skills/principle-{encode-lessons-in-structure,subtract-before-you-add,migrate-callers-then-delete-legacy-apis,boundary-discipline,type-system-discipline,separate-before-serializing-shared-state}/SKILL.md. Read it for ideas; write every sentence in bee's own plain words. (1) Homes first (D2). expertise/knowledge.md: new section '## Encode lessons in structure' — a lesson seen twice becomes structure, strongest first: architecture (a code change makes the mistake impossible), check (a test, hook, guard or doctor row catches it), doctrine (prose, only when the lesson needs judgment; then make it prominent and show the failure); when the structural fix exists, the prose copy goes; this is the same ladder as `bee mailbox reflect --fix-at`. expertise/planning.md: new section '## Subtract before you add' — sequence removal before construction, cut to the minimum before polishing, build for observed use not speculative edges, no guard or parser the contract does not ask for, a stub with no content is deleted. expertise/architecture.md: new section '### Migrate Callers, Then Delete' under ## Principles — a new internal API ships with every caller moved and the old API deleted in the same change; tests move to the new contract and tests that only pinned the old internals go; no compatibility shim unless an outside user depends on the old surface; link to 'Replace, Don't Layer'. Extend '### Explicit Boundaries and Contracts' with: validate and parse at the boundary (CLI args, config, files, network, other processes), trust typed data inside, keep decisions in pure functions the thin shell calls, never leak the boundary's private types through the public surface; tests: 'Is this data crossing a boundary right now?' and 'Can this be a pure function the shell calls?'. Extend '### Make Illegal States Unrepresentable' with: give each meaning its own type (a newtype for an id) and validate once at creation, parse outside data into types at the boundary, match every variant without a catch-all, derive types from the schema that owns the shape. Extend '### One Writer for Shared State' with the order: list the shared mutable things, first remove the sharing (each actor its own file, key or record, merged where it is read; two writers of different fields in one JSON file still share it), and only when one shared target is a real invariant enforce one writer structurally; 'we need a lock' is a smell to check. Add routing table rows in architecture.md for the new section. Copy each changed expertise file to the same path under .bee/expertise/ if `bee dev regen` does not. (2) Six skills (D1), each at skills/<slug>/SKILL.md, modeled on skills/bee-principle-build-the-lever/SKILL.md: frontmatter name = directory, description starts 'Apply when ...' and ends with the one-sentence move; '# Title'; 2-3 short paragraphs; '**Check:**' one question the agent can answer now; '**Not when:**' when the principle does not apply; '**Why:**'; '**Depth:** `.bee/expertise/<guide>.md` § <exact heading>'. Under 30 lines each. (3) Index rows (D3) in the Principle homes section of the doctrine index doc, after the last row, same grammar as line 429: '- `<slug>` (expertise/<guide>.md § <exact heading>):' then '  spoken: ...' then '  classes: ...' then the applied_at block pointing at the skill. Classes exactly: encode-lessons docs, bugfix; subtract refactor; migrate-callers refactor; boundary-discipline feature, refactor; make-illegal-states feature, refactor; separate-before-serializing feature, perf. Add the six skill paths to the doc's anchors frontmatter list. (4) Run `.bee/bin/bee onboard --repo-root . --apply` and `.bee/bin/bee dev regen` so the rendered skill trees, onboarding ledger, skills lock and release manifest follow. (5) Prove the user path: run `.bee/bin/bee orient` is not needed; instead check that `cargo test --test principle_index_parity` passes. Use the bee-technical-writing standard, plain global English, no code comments. Proof: the verify command; quote the counts."}
]
```
