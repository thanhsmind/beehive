---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: design-os chart, diagram and motion checks

## Summary

bee-design moves to ease-design 0.6.2. Every pack that runs the design tool
(charts, motion, web, figma) gets its own list of `ui` commands, made from the tool itself, so a worker is no
longer told to run a command the core forbids. `bee design gate` also checks
chart, diagram, flow and motion files, and a host's old baseline keeps
passing: the new ds-usage checks of 0.6.2 are reported, not failed, until a
person records a new baseline.

Mode: `standard` — 4 risk flags: external systems, public contracts, changes behavior an existing test asserts, multi-domain
Why this is the least workflow that protects the work: the gate output and baseline file are a contract every host reads, so it needs a plan and red-first tests, but no hard-gate flag is present.

Playbook: `skills/bee-planning/playbooks/feature.md` (route class `feature`), cited, not copied.

## Requirements (from CONTEXT.md)

- D1: pin ease-design 0.6.2 in the gate, the core skill and UPSTREAM.json; absorbs sup-20261030-76ac.
- D2: sync taste from design-os v0.6.2 (369b041) with `scripts/sync-design-os.sh`.
- D3: one generated `references/contract.md` per pack that calls `ui`, list homed in the sync script.
- D4: a "Not for host work" section names `knowledge check`, `knowledge effect-matrix`, `gflow i2v` with reasons.
- D5: core rule = core contract plus loaded pack contracts; a suite test fails on an uncovered pack command.
- D6: artifacts found by marker: `data-chart-owned="true"`, `data-diagram-owned="true"`, class `scrub`, `flow.json` / `*.flow.json`.
- D7: artifacts ratchet in a baseline `artifacts` map; new artifacts need 0 errors; no artifacts, no output change.
- D8: `scrub-lint` and `scroll-story` stay out of the gate, in the motion contract.
- D9: baseline records its families; compare only those; no list means the 0.6.1 set; extra families get one advisory line.
- D10: cap rule default UI globs add `flow.json` and `*.flow.json`.
- D11: real run on an anphabe-goglbe copy and design-os examples before done.
- D12: a baseline with no `artifacts` key reports artifact errors in the advisory line, never fails on them.

## Load-bearing claims

Labels: `read` (opened at the anchor), `ran` (command run, output held). Evidence is a verbatim substring of the anchor.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The gate pins 0.6.1 in one constant | read | packages/bee-rs/crates/bee/src/verbs/design_group.rs:13 | const PIN: &str = "ease-design@0.6.1"; |
| 2 | The gate reads only the total errorCount today | read | packages/bee-rs/crates/bee/src/verbs/design_group.rs:134 | let errors = data.get("errorCount").and_then(Value::as_u64).ok_or_else(malformed)?; |
| 3 | The cap rule's default UI globs have no flow file | read | packages/bee-rs/crates/bee/src/verbs/cells/design_rule.rs:10 | const DEFAULT_UI_GLOBS: [&str; 7] = ["*.html", "*.css", "*.scss", "*.jsx", "*.tsx", "*.vue", "*.svelte"]; |
| 4 | The core skill forbids commands outside its contract | read | packages/bee/design/bee-design/SKILL.md:50 | call nothing that is not there. |
| 5 | The sync takes the upstream commit from the checkout HEAD | read | scripts/sync-design-os.sh:13 | commit="$(git -C "$src" rev-parse --short=7 HEAD)" |
| 6 | The design-os checkout HEAD is the 0.6.2 release on origin | ran | git -C /mnt/data/Projects/goglbe/design-os log --oneline -1 origin/main | 369b041 release: ease-design 0.6.2 — the design-system usage release (#376) |
| 7 | chart lint fails an ordinary dashboard page, so artifacts need a marker | ran | npx --yes ease-design@0.6.2 chart lint /mnt/data/Projects/anphabe/anphabe-goglbe/ads-report.html --json | Expected exactly one owned SVG; found 0. Mark one root data-chart-owned=\"true\". |
| 8 | 0.6.2 gate reports per-family error counts, and ds-usage is new | ran | node .bee/workers/ed062/node_modules/ease-design/dist/cli.js gate settings.html --dir . --json (in the anphabe copy) | ds-usage {'errorCount': 330, 'warningCount': 1119, 'advisoryCount': 0} |
| 9 | 0.6.1 on the same copy shows no regressions | ran | BEE_DESIGN_OS_CLI=<0.6.1 cli> bee design gate (in the anphabe copy) | design gate: no regressions on 51 pages |
| 10 | 0.6.2 ships the artifact linters with these signatures | ran | npx --yes ease-design@0.6.2 schema --json | chart lint <file.html> :: Lint self-contained chart HTML deterministically |
| 11 | Upstream examples carry the chart and diagram markers | ran | rg -l 'data-diagram-owned|data-chart-owned' /mnt/data/Projects/goglbe/design-os/examples | wc -l | 28 |

## Discovery

Read the gate, the cap rule, the sync script and the core contract; ran
`ui schema --json`, `ui chart lint`, and `bee design gate` with 0.6.1 and
0.6.2 on a copy of anphabe-goglbe (51 pages, no write to that repo). Pack
command inventory (rg over `packages/bee/design/bee-design-*`): charts calls
chart lint, diagram lint, flow lint, ds-usage-lint; motion calls tenant-lint,
tenant-scaffold, scrub-lint, scrub-scaffold, ds-usage-lint, knowledge check,
knowledge effect-matrix, gflow i2v; web and figma call design-system and
Figma commands (all present in 0.6.2 except `designmd export`, which upstream
names as future work); every pack also calls core commands.

## Approach

Three disjoint cells run in one wave. The gate cell (D1 pin, D6-D9) changes
`design_group.rs` only. The cap-rule cell (D10) changes `design_rule.rs` and
its test. The contracts cell (D2-D5, D1 skill lines) adds a Node renderer,
extends the sync, runs it, edits the skill text, and adds the parity test.
The real run (D11) is the leader's, after the wave, on the built binary.

Leader steps after the wave: (a) the D11 real run on the anphabe-goglbe copy
(main vs head) and on design-os `examples/` with a baseline recorded by main's
binary; (b) set PBI sup-20261030-76ac to done at close (D1); (c) file the
core-contract gap (core references naming `ui` commands outside the core
contract) to the backlog; (d) capture syncs
`docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md`.

Shared build unit: the three cells write disjoint files but build one crate.
Each worker keeps the crate compiling at every save; a compile error in a file
outside its cell is another worker's edit in flight — wait and re-run, never
touch it.

Rejected: widening the core contract (it is upstream's file and the packs are
optional); config globs for artifacts (no host config needed when upstream's
own markers exist); failing on new families (breaks every host until a
re-record); `ui gate --skip ds-usage` in place of the family sum (never
measures ds-usage, hard-codes one family); reusing upstream's
`contract-render.mjs` (its command list is a hard-coded constant); folding
doc-2 into doc-1 (saves one dispatch but serializes D10 behind the largest
cell); the schema from the checkout's `dist/cli.js` (needs a build that
matches HEAD; npx of the pinned version is what bee runs).

Plan check: `docs/history/design-os-checks/reports/hat-wave.md`.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Baseline compatibility | MEDIUM | doc-1 | Old baseline (no `families`) on a 0.6.2 run with ds-usage errors passes; ran fact: claim 8 and 9 |
| Artifact detection | LOW | doc-1 | Fake-tool tests per marker |
| Contract drift | LOW | doc-3 | Parity test over shipped pack text |

Waves: wave 1 = doc-1, doc-2, doc-3 (no shared files). Leader real run after.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage": "plan-hat-wave", "classification": "required", "role": "advisor", "reason": "Standard lane with more than five product files opens the wave at its three default seats."},
    {"stage": "hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "Default seat; carries MANDATE 1."},
    {"stage": "hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "Default seat."},
    {"stage": "hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "Default seat."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Not high-risk."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Not high-risk."},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "Every claim was read or run by the leader."},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "A narrow fact is needed from a known file during execution.", "reason": "Cheap narrow lookups."},
    {"stage": "gather", "classification": "conditional", "role": "read", "condition": "A multi-file hunt is needed that the leader does not hold.", "reason": "Mechanical reads delegate down-tier."},
    {"stage": "cell-execution", "classification": "required", "role": "test", "reason": "All three cells are red-first changes; the code role's herding worker went idle twice on the prior design feature (decision db4d5464)."},
    {"stage": "code", "classification": "not-applicable", "role": "code", "reason": "See cell-execution."},
    {"stage": "design", "classification": "not-applicable", "role": "design", "reason": "No cell builds rendered UI; the cells change the checker, not a page."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "Skill text rides doc-3; the area spec rides capture."},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "The slice judge runs.", "reason": "Goal-check judge for behavior_change cells."},
    {"stage": "plan", "classification": "not-applicable", "role": "plan", "reason": "One slice."},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "Every writing stage resolves to a named role above."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No open design shape."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "See blind-lane-1."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "See blind-lane-1."}
  ]
}
```

## Shape

| Epic | Capability / risk area | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| Gate checks artifacts at 0.6.2 | gate, baseline | D1, D6-D9 | 1 | design_group unit tests; real run (D11) |
| Flow files are UI files | cap rule | D10 | 1 | design rule test |
| Packs carry their own contract | sync, skill text, parity | D1-D5 | 1 | parity test over shipped text |

**Current slice to prepare: slice 1 (only).**

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| doc-1 | Check chart, diagram, flow and motion artifacts in the design gate at 0.6.2 | packages/bee-rs/crates/bee/src/verbs/design_group.rs | — | `bee design gate` names a chart page whose errors rose; an old baseline still passes with one advisory note | design_group tests green |
| doc-2 | Count flow models as UI files in the design cap rule | packages/bee-rs/crates/bee/src/verbs/cells/design_rule.rs, packages/bee-rs/crates/bee/src/verbs/cells/tests.rs | — | a cell that changes only `flow.json` must run the gate to cap | design_rule tests green |
| doc-3 | Give every design pack that calls ui its own command contract at 0.6.2 | scripts/sync-design-os.sh, scripts/design-os-contract.mjs (new), packages/bee/design/*, packages/bee-rs/crates/bee/src/onboard/skills.rs | — | charts, motion, web and figma each have a contract.md; a stray pack command fails the suite | parity test green plus pin and link checks |

```json
[
  {
    "id": "doc-1",
    "feature": "design-os-checks",
    "title": "Check chart, diagram, flow and motion artifacts in the design gate at 0.6.2",
    "lane": "standard",
    "role": "test",
    "deps": [],
    "decisions": ["D1", "D6", "D7", "D8", "D9", "D12"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/design_group.rs"],
    "read_first": ["docs/history/design-os-checks/CONTEXT.md", "packages/bee-rs/crates/bee/src/verbs/design_group.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md"],
    "action": "RED FIRST: add the failing tests to the existing tests module of design_group.rs (fake node and fake cli, canned JSON per page) and watch each fail for its own reason, then change the code. (1) Per D1 set PIN to ease-design@0.6.2. (2) Per D9 read data.families from ui gate output: for each page, the compared error count is the sum of families.<name>.errorCount over the baseline's family set; the baseline written by --write gains a top-level families array (sorted names the run returned); a baseline with no families key counts the 0.6.1 set a11y, autofix, content, layout, taste, tell; when the run returned any family outside the baseline's set, print one advisory line naming those families and saying they are not counted until a person re-records with bee design gate --write. A ui gate output with no families object falls back to data.errorCount. Keep recording warnings and coverage as today. (3) Per D6 detect artifacts: among the HTML pages the gate already scores, a page whose text contains data-chart-owned=\"true\" also runs ui chart lint <page> --json, data-diagram-owned=\"true\" runs ui diagram lint, an element whose class attribute has the token scrub runs ui tenant-lint; from the same walk (same skipped dirs, ignoring design.pages globs) a file named flow.json or ending in .flow.json runs ui flow lint. Read data.errorCount (required) and data.warningCount (missing counts as 0: tenant-lint returns none). A linter that does not start or returns other JSON is a failure line naming the kind and path, like the page case. (4) Per D7 the baseline gains an artifacts object keyed by kind (chart, diagram, motion, flow) then path, each {errors, warnings}; a known artifact may not gain errors; one with no entry must have 0 errors; regression lines read design gate: <kind> <path> — errors rose a -> b or design gate: <kind> <path> — new <kind> has n errors, must have 0. Per D12, when the baseline file exists but has no artifacts key, artifact errors are not failures: the advisory line also names the artifacts with errors. The advisory line prints at most once per run, before the summary, also under --write; it never counts as a failure, never sets exit 1, never enters last-run.json regressions, and never blocks --write. --json output gains an artifacts object only when an artifact was found. When no artifact is found, every output line, the JSON object and the baseline pages object stay byte-identical to today; with artifacts the summary reads no regressions on N pages and M artifacts. (5) Per D8 never run scrub-lint or scroll-story. Shared build unit: a compile error in a file outside this cell is another worker's edit in flight — wait and re-run, never touch it. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml design_group",
    "must_haves": {
      "truths": [
        "An old baseline with no families key and a run whose only new errors are in ds-usage exits 0 and prints one line naming ds-usage and bee design gate --write",
        "A page with data-chart-owned=\"true\" whose chart lint errors rose against the baseline exits 1 naming the page and the counts",
        "A flow.json with errors and no baseline entry exits 1",
        "A project with no artifacts prints exactly the same lines as before",
        "The advisory line never counts as a regression in last-run.json and never blocks --write",
        "A baseline with no artifacts key and a marked chart page with chart lint errors exits 0 and the advisory line names that page"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/design_group.rs", "substantive": "PIN 0.6.2, family-scoped compare, artifact lint and ratchet, tests for each truth"}],
      "key_links": ["bee design gate --write records families and artifacts in .bee/design/gate-baseline.json"],
      "prohibitions": ["No new config key", "No change to the --write refusal while a cell is claimed", "No change to the not-run line"]
    }
  },
  {
    "id": "doc-2",
    "feature": "design-os-checks",
    "title": "Count flow models as UI files in the design cap rule",
    "lane": "standard",
    "role": "test",
    "deps": [],
    "decisions": ["D10"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/cells/design_rule.rs", "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"],
    "read_first": ["docs/history/design-os-checks/CONTEXT.md", "packages/bee-rs/crates/bee/src/verbs/cells/design_rule.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md"],
    "action": "RED FIRST: in packages/bee-rs/crates/bee/src/verbs/cells/tests.rs, next to the existing design cap tests (search for the string FIX: add `bee design gate` to the cell's verify), add a test whose name starts with design_rule_ (so the verify filter runs it) that a cell whose files list only flows/signup.flow.json, and one whose files list only flow.json, in a host whose config has design.packs [\"charts\"], is refused at cap when its verify lacks bee design gate; watch it fail. Then per D10 add flow.json and *.flow.json to DEFAULT_UI_GLOBS in design_rule.rs. A host's design.ui_globs still replaces the default. Shared build unit: a compile error in a file outside this cell is another worker's edit in flight — wait and re-run, never touch it. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml design_rule",
    "must_haves": {
      "truths": ["A cell that changes only a flow.json or *.flow.json file in a host that picked design packs is refused at cap unless its verify runs bee design gate"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/design_rule.rs", "substantive": "DEFAULT_UI_GLOBS includes flow.json and *.flow.json"}],
      "key_links": ["design_cap_refusal uses DEFAULT_UI_GLOBS when design.ui_globs is absent"],
      "prohibitions": ["No change to behavior in a host with no design pick"]
    }
  },
  {
    "id": "doc-3",
    "feature": "design-os-checks",
    "title": "Give every design pack that calls ui its own command contract at 0.6.2",
    "lane": "standard",
    "role": "test",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "D8"],
    "files": ["scripts/sync-design-os.sh", "scripts/design-os-contract.mjs", "packages/bee/design/*", "packages/bee-rs/crates/bee/src/onboard/skills.rs"],
    "read_first": ["docs/history/design-os-checks/CONTEXT.md", "scripts/sync-design-os.sh", "packages/bee/design/bee-design/SKILL.md", "packages/bee/design/bee-design/references/contract.md", "/mnt/data/Projects/goglbe/design-os/skills/design-os-designer/contract-render.mjs"],
    "affects_skills": ["packages/bee/design/bee-design/SKILL.md", "packages/bee/design/bee-design-charts/SKILL.md", "packages/bee/design/bee-design-motion/SKILL.md"],
    "affects_specs": ["docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md"],
    "action": "RED FIRST: in the tests module of packages/bee-rs/crates/bee/src/onboard/skills.rs, next to the_shipped_design_root_holds_the_core_and_one_dir_per_pack, add a test whose name contains shipped_design (so the verify filter runs it) that, for every pack dir under packages/bee/design whose name starts with bee-design- (all packs, not the core), scans the pack's .md files for backticked `ui <word>[ <word>]` mentions (words are [a-z][a-z0-9-]*) and fails naming pack, file and command when the command is in neither the core contract (## ui headings of bee-design/references/contract.md), the pack's references/contract.md (its ## ui headings), nor that contract's Not for host work bullets; a pack with no contract counts as an empty one, so any uncovered mention fails; a first word that is a group (some listed entry starts with it plus a space) needs its second word to complete a listed entry, unless the mention has no second word; a first word that is a listed leaf command passes whatever follows it. Watch it fail (no pack contract yet). Then: (1) per D3 write scripts/design-os-contract.mjs (Node, no comments, no dependencies): reads ui schema --json on stdin, takes the command refs as arguments (name or name:sub), and writes the same format as upstream contract-render.mjs (sha256 header line over the body, # Generated command contract, one ## ui section per ref with summary, Usage, flag table, Error codes), with the heading text naming the pack, plus, per D4, a final ## Not for host work section whose bullets (one `ui <ref>` plus its reason each) come from arguments of the form --not-for-host 'ref|reason'; a not-for-host ref needs no schema entry, every listed ref does (the renderer exits non-zero naming a missing one). (2) Extend scripts/sync-design-os.sh: a per-pack contract list beside the entries lists, the single home per D3 — charts: chart:lint diagram:lint flow:lint ds-usage-lint; motion: ds-usage-lint tenant-lint tenant-scaffold scrub-lint scrub-scaffold scroll-story (D8) with not-for-host knowledge:check and knowledge:effect-matrix (they lint upstream's own knowledge folder) and gflow:i2v (drives a paid video service on the owner's account); web: designmd:extract-tokens ds:change-token ds:kit pattern:lint registry:register schema with not-for-host designmd:export (upstream names it as future work; 0.6.2 has no such command); figma: color:contrast critique-coverage doctor ds:a11y ds:init ds:preview ds:specimen ds:status ingest-figma-ds memory:record schema synthesize-conventions with not-for-host init (it writes design-os's own adapter skills and commands, which bee's packs replace); apple calls no ui command and gets no contract; after copying references, render each listed pack's references/contract.md from npx --yes ease-design@<version> schema --json where version is the one read from the checkout's package.json. (3) Per D2 run scripts/sync-design-os.sh /mnt/data/Projects/goglbe/design-os (its HEAD is 369b041, tag v0.6.2) and keep its whole output, UPSTREAM.json included. (4) Per D1 and D5 edit packages/bee/design/bee-design/SKILL.md: taste version line and npx line say 0.6.2; the contract rule says call only commands in references/contract.md and in references/contract.md of each pack you loaded. In the Check step, add that the gate also lints chart, diagram, motion and flow artifacts it finds by marker (data-chart-owned, data-diagram-owned, class scrub, flow.json or *.flow.json) and that families a baseline predates are reported, not failed, until a person re-records. Edit the SKILL.md of bee-design-charts, bee-design-motion, bee-design-web and bee-design-figma to point at their contract and say the Not for host work commands are not run in a host. In bee-design-charts/SKILL.md also say: mark the root of a chart with data-chart-owned=\"true\" and of a diagram with data-diagram-owned=\"true\", and name a flow model flow.json or <name>.flow.json, or the design gate never checks it. Shared build unit: a compile error in a file outside this cell is another worker's edit in flight — wait and re-run, never touch it. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml shipped_design && rg -q '\"version\": \"0.6.2\"' packages/bee/design/UPSTREAM.json && rg -q 'ease-design@0.6.2' packages/bee/design/bee-design/SKILL.md && ! rg -q a7ad5f1 packages/bee/design",
    "must_haves": {
      "truths": [
        "bee-design-charts, bee-design-motion, bee-design-web and bee-design-figma each ship references/contract.md generated from ease-design 0.6.2 ui schema --json",
        "The parity test fails when a pack names a ui command in no contract, and passes on the shipped text",
        "packages/bee/design/UPSTREAM.json records version 0.6.2 and commit 369b041",
        "The core skill names ease-design 0.6.2 and allows core plus loaded pack contracts"
      ],
      "artifacts": [
        {"path": "scripts/design-os-contract.mjs", "substantive": "renders a pack contract with a Not for host work section"},
        {"path": "packages/bee/design/bee-design-charts/references/contract.md", "substantive": "chart lint, diagram lint, flow lint, ds-usage-lint with 0.6.2 flags"},
        {"path": "packages/bee/design/bee-design-motion/references/contract.md", "substantive": "motion commands plus Not for host work"}
      ],
      "key_links": ["sync-design-os.sh calls design-os-contract.mjs with the per-pack lists"],
      "prohibitions": ["No hand edit of synced reference files", "No change to the core contract.md beyond what the sync writes"]
    }
  }
]
```

## Test matrix

- Happy: no artifacts, baseline at parity — output identical to today. Pass when lines equal `design gate: no regressions on N pages`.
- Happy: chart page at parity with its baseline entry. Pass when exit 0 and summary names 1 artifact.
- Edge: old baseline (no families), ds-usage errors only. Pass when exit 0 and one line names ds-usage.
- Edge: baseline with families including ds-usage, ds-usage errors rose. Pass when exit 1 naming the page.
- Error: new flow.json with errors. Pass when exit 1 with `new flow`.
- Error: pack text names an uncovered command. Pass when the parity test fails naming it.
- Same scenario main vs head: anphabe copy with its 0.6.1 baseline. Pass when head prints no regressions plus the ds-usage note.

## Open Questions

(none)

## Out of scope

- Core references naming commands outside the core contract — backlog (leader step c).
- anphabe-goglbe re-install, its `scripts/design-gate.mjs` bump and its baseline re-record — owner's acts there.
