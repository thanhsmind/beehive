---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: design-os in bee — Slice 3

## Summary

Slices 1 and 2 shipped: bee carries the design-os taste, a `design` member of
the team runs design cells, and `bee onboard --design-packs` puts only the
picked packs into a project (commits `c004f6eb5` to `334e4c147`).

This slice adds the check. A new command, `bee design gate`, runs design-os's
own `ui gate` on each page through `npx`, and compares the result with the
project's saved baseline. The error count may not rise, and the share of the
page that uses the project's own design tokens may not fall. Saving a new
baseline is a person's act: the command refuses it while a cell is in flight.

In a project that picked design packs, a cell that changes UI files must use
this command as its proof, or bee refuses to close the cell and says how to fix
it. On a machine without Node 20, the command says plainly that it did not run;
the cell may still close, and the gap stays on the record. Projects with no pick
are not touched.

Mode: `high-risk` — 4 flags: external-systems, public-contracts, multi-domain,
covered-contract-change.
Why this is the least workflow that protects the work: the cap rule changes what
a UI cell must prove in every host that picked packs.

## Requirements (from CONTEXT.md and the owner's slice-3 answers)

- **D5** (`d5399608`) — checks run `ui gate` through `npx ease-design` at a
  pinned version (0.6.1), Node 20+.
- **D6** (`587bcc85`) — a UI cell carries `ui gate` proof under a baseline
  ratchet; the cap refuses a UI cell with no such proof.
- **Owner answer 1** (2026-10-24, logged) — in a host with a pick and no
  Node 20+, a UI cell may cap with a proof that records the gate as not run;
  the gap stays visible on the record.
- **Owner answer 2** (2026-10-24, logged) — the cap rule binds only where
  `design.packs` names at least one pack.
- Hat-wave findings carried here: `--write` is a human act and a cell that
  changes the baseline file is refused; the refusal names its fix; cell
  creation warns early.

## Load-bearing claims

Labels: `read` = the file was opened at the named line; `ran` = the command was
executed and its output kept; `guessed` = neither, and no such row may survive
the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The cap already binds the proof command to the cell's approved verify, before any write — the place the UI rule joins | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:308` | `if js_trim(&proof_command) != cell_verify {` |
| 2 | One helper reads the host's merged config | read | `packages/bee-rs/crates/bee/src/state.rs:161` | `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` |
| 3 | External programs are reached through an env override tests can point at a fake | read | `packages/bee-rs/crates/bee/src/verbs/report.rs:96` | `gh: std::env::var_os("BEE_GH_BIN").unwrap_or_else(` |
| 4 | A small native verb group plugs into one dispatch chain | read | `packages/bee-rs/crates/bee/src/verbs/mod.rs:122` | `if let Some(code) = tmp_group::try_native(args, t0) {` |
| 5 | The router lists every native verb | read | `packages/bee-rs/crates/bee/src/router.rs:90` | `"tmp sweep",` |
| 6 | A cell's claimed state is read from its record's status | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs:304` | `if !matches!(record.get("status"), Some(Value::String(s)) if s == "claimed") {` |
| 7 | The field ratchet reads `errorCount` from `ui gate --json` | read | `/mnt/data/Projects/anphabe/anphabe-goglbe/scripts/design-gate.mjs:28` | `errors: data.errorCount,` |
| 8 | The field ratchet runs the pinned package through npx, or a local CLI via an env override | read | `/mnt/data/Projects/anphabe/anphabe-goglbe/scripts/design-gate.mjs:11` | ``const [bin, prefix] = local ? [process.execPath, [local]] : ['npx', ['--yes', `ease-design@${VERSION}`]];`` |
| 9 | The core design skill still tells workers to call npx directly — slice 3 points it at the new command | ran | `rg -n 'npx --yes ease-design@0.6.1 gate' packages/bee/design/bee-design/SKILL.md` in the feature worktree | `77:npx --yes ease-design@0.6.1 gate <page.html> --dir . --json` |
| 10 | The design-cell preamble is one constant | ran | `rg -n 'DESIGN_CELL_PREAMBLE: &str' packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs` in the feature worktree | `45:pub(crate) const DESIGN_CELL_PREAMBLE: &str = "Design cell:` |

## Discovery

Read the cap path, config reader, process override pattern, verb wiring and
claim status (claims 1–6) and the field ratchet (claims 7–8). The slice-3
gather worker went idle; the leader read these directly. The registry payload
(`src/generated/registry_payload.json`) has no other source in the tree, so a
new verb adds its entry there by hand, as `tmp.sweep` sits there.

## Approach

**Recommended path.**

- **`bee design gate` (D5).** A new native verb group `design` with one verb,
  `gate`, wired like `tmp sweep` (claims 4–5) plus its registry entry. Pages:
  host config `design.pages` (glob list), default every `*.html` under the repo
  outside hidden dirs, `node_modules`, `target`, `dist`. For each page it runs
  `npx --yes ease-design@0.6.1 gate <page> --dir . --json`, or `node <cli>` when
  `BEE_DESIGN_OS_CLI` names a local CLI (claim 8, claim 3 pattern), and reads
  `data.errorCount`, `data.warningCount` and
  `data.tokenCoverage.overall.coverage` (claim 7). Baseline file
  `.bee/design/gate-baseline.json` `{version, pages: {page: {errors, warnings,
  coverage}}}`. Rules: a page with no baseline must have 0 errors; errors may
  not rise; coverage may not fall. Exit 0 with one summary line when clean;
  exit 1 naming each page that regressed. `--write` records the baseline and
  refuses while any cell in this repo is claimed (claim 6). `--json` prints the
  result as one object.
- **No Node (owner answer 1).** When `node --version` is missing or below 20
  (override `BEE_DESIGN_NODE_BIN`), the gate prints exactly
  `design gate: not run — Node 20 or newer is missing`, writes
  `.bee/design/last-run.json` `{status: "not-run", reason}`, and exits 0. A run
  that did run writes `{status: "ran", ...}`.
- **Cap rule (D6, owner answer 2).** In `handlers_close.rs` beside claim 1, only
  when the host config `design.packs` is a list with at least one pack: if any
  path in the cell's `files` matches `design.ui_globs` (default `*.html`,
  `*.css`, `*.scss`, `*.jsx`, `*.tsx`, `*.vue`, `*.svelte`), the cell's verify
  must contain `bee design gate`, else refuse naming the fix (add
  `bee design gate` to the cell's verify); if `.bee/design/last-run.json` says
  `not-run`, the proof reason must contain `design gate not run`, else refuse
  naming that fix; and any cell whose `files` include
  `.bee/design/gate-baseline.json` is refused (the baseline is a person's act).
  `bee cells add` warns on stderr, never refuses, when a UI cell's verify lacks
  `bee design gate` in such a host.
- **Point the taste at the command.** The core skill's Check step, the
  `bee-design` agent template and `DESIGN_CELL_PREAMBLE` (claims 9–10) name
  `bee design gate` and the not-run line instead of the raw npx call.

**Rejected alternatives.**

- Porting the linters to Rust — rejected by D5.
- Refusing the cap on a Node-less host — rejected by owner answer 1.
- Checking the gate's output at cap by running it — doors never run tests.

**SMALLER PATH check.** The verb is D5's one door; the cap rule is D6; the
not-run record is owner answer 1; the baseline guard is the hat-risks blocker.
Pointing the skill at the verb is three text edits. PASS.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Cap refusal on UI cells | HIGH — changes what every UI cell in a picked host must prove | dob-6 | a UI cell without `bee design gate` in verify is refused with the fix named; a non-UI cell and a no-pick host are untouched |
| Baseline raised by the cell it guards | HIGH | dob-5, dob-6 | `--write` refuses while a cell is claimed; a cell with the baseline in `files` is refused |
| External tool output shape | MEDIUM — a changed JSON shape breaks parsing | dob-5 | a fake CLI via `BEE_DESIGN_OS_CLI` drives clean, regressed, new-page and malformed-output cases |
| Node absent | MEDIUM — must not jam work | dob-5, dob-6 | fake node below 20 prints the exact not-run line, exits 0, and the cap accepts a reason carrying `design gate not run` |

Waves: dob-5 and dob-6 run in parallel. Shared contract fixed here: the
`last-run.json` shape and the exact not-run line. Files are disjoint.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "plan-hat-wave", "classification": "not-applicable", "role": "advisor", "reason": "The wave is once per feature and ran at slice 1 (docs/history/design-os-in-bee/reports/hat-wave.md); its slice-3 findings (baseline write, refusal fix text, early warning) are in this plan. Re-stamped, not re-run."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "See plan-hat-wave."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "See plan-hat-wave."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "See plan-hat-wave."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "See plan-hat-wave."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "See plan-hat-wave."},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "Every claim was read directly by the leader."},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "A narrow fact is needed from a known file during execution.", "reason": "Cheap narrow lookups."},
    {"stage": "gather", "classification": "conditional", "role": "read", "condition": "A multi-file hunt is needed that the leader does not hold.", "reason": "Mechanical reads delegate down-tier."},
    {"stage": "cell-execution", "classification": "required", "role": "test", "reason": "Both cells are red-first Rust changes; the code role's herding worker went idle twice on this feature (decision db4d5464)."},
    {"stage": "code", "classification": "not-applicable", "role": "code", "reason": "See cell-execution."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "The skill text edits ride dob-5; capture runs after the feature."},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "The user invokes an independent review, or the slice judge runs.", "reason": "Independent review is user-invoked only."},
    {"stage": "plan", "classification": "not-applicable", "role": "plan", "reason": "This is the last slice."},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "Every writing stage resolves to a named role above."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No open design shape; the field ratchet fixes the mechanism."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "See blind-lane-1."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "See blind-lane-1."}
  ]
}
```

## Shape

| Epic | Capability / risk area | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| The taste ships with bee | Content, license | D1, D2 | 1 — **shipped** | done |
| A design agent works from it | Dispatch | D3 | 1 — **shipped**; 2 read-only fix **shipped** | done |
| Only picked packs reach a host | Onboarding | D3, D4 | 2 — **shipped** `334e4c147` | done |
| UI cells prove themselves | Cap rule, external tool | D5, D6, owner answers | 3 (this plan) | fake-CLI gate runs; cap refusal tests |

**Current slice to prepare: slice 3 (last).**

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| dob-5 | Add bee design gate, a baseline ratchet over ui gate | new design verb, verbs/mod, router, registry payload, skill text, agent template, preamble, manifest | — | `bee design gate` prints one clean line, or names each page that regressed; with no Node it prints the not-run line | release-manifest check, then the declared test command, red first |
| dob-6 | Refuse a UI cell's cap without the design gate in a host that picked packs | cells cap path, cells add warning, tests | — | capping a UI cell whose verify lacks `bee design gate` is refused with the fix named | the declared test command, red first |

```json
[
  {
    "id": "dob-5",
    "feature": "design-os-in-bee",
    "title": "Add bee design gate, a baseline ratchet over ui gate",
    "lane": "high-risk",
    "role": "test",
    "deps": [],
    "decisions": ["d5399608-c7db-4184-968f-1f870805a522", "587bcc85-ac38-42a1-87e5-9fae09a85db9", "b99dde04-7232-4139-b09b-a409034e0464"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/design_group.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/router.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
      "packages/bee-rs/crates/bee/tests/registry_contracts.rs",
      "packages/bee/design/bee-design/SKILL.md",
      "packages/bee/agents/bee-design.md.tmpl",
      ".claude/agents/bee-design.md",
      ".opencode/agent/bee-design.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/design-os-in-bee/CONTEXT.md",
      "docs/history/design-os-in-bee/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/tmp_group.rs",
      "packages/bee-rs/crates/bee/src/verbs/report.rs",
      "/mnt/data/Projects/anphabe/anphabe-goglbe/scripts/design-gate.mjs",
      "packages/bee/design/bee-design/SKILL.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST: write the failing tests before the code and watch each fail for its own reason. Drive the external tool through fakes only: a fake CLI script named by BEE_DESIGN_OS_CLI that prints a canned `ui gate --json` object, and a fake node named by BEE_DESIGN_NODE_BIN that prints a version.\n\n(1) VERB (per D5). Add verbs/design_group.rs with `bee design gate [--write] [--json]`, wired like tmp sweep: `if let Some(code) = tmp_group::try_native(args, t0) {` in verbs/mod.rs gains a design_group sibling; router.rs gains `\"design gate [--write] [--json]\",` beside `\"tmp sweep\",`; add a `design.gate` entry to src/generated/registry_payload.json in the same shape as `tmp.sweep` (name, invoke, surface, description, parameters, examples, deprecated) and keep tests/registry_contracts.rs and the cli-shape guard (hooks/cli_shape.rs) green. Port the field ratchet /mnt/data/Projects/anphabe/anphabe-goglbe/scripts/design-gate.mjs: pages = host config design.pages (a list of globs, read with crate::state::read_config_raw), default every *.html under the repo root skipping hidden dirs, node_modules, target, dist; per page run `npx --yes ease-design@0.6.1 gate <page> --dir . --json` with cwd = repo root, or `<node> <BEE_DESIGN_OS_CLI> gate <page> --dir . --json` when that env var is set (the BEE_GH_BIN pattern in verbs/report.rs); a non-zero exit with JSON on stdout is still a result (the field script does the same). Read data.errorCount, data.warningCount and data.tokenCoverage.overall.coverage (absent coverage = 0). Baseline file .bee/design/gate-baseline.json {version, pages: {<page>: {errors, warnings, coverage}}}, coverage rounded to 4 places. Rules: a page with no baseline entry must have 0 errors; errors may not rise; coverage may not fall. Clean -> one summary line, exit 0. Any regression -> one line per page naming it and the numbers, exit 1. Malformed tool output -> exit 1 naming the page and that the output was not the expected JSON.\n\n(2) WRITE IS A HUMAN ACT. `--write` records the baseline for every page and refuses, writing nothing, while any cell in this repo's store has status claimed (a record with status \"claimed\", as handlers_select.rs reads it), naming the claimed cell ids.\n\n(3) NO NODE (owner answer 1). Before any page, check `node --version` (or BEE_DESIGN_NODE_BIN). Missing, unparsable, or major below 20 -> print exactly `design gate: not run — Node 20 or newer is missing`, write .bee/design/last-run.json {\"status\": \"not-run\", \"reason\": \"node-missing\"}, exit 0. Every run that does run writes .bee/design/last-run.json {\"status\": \"ran\", \"pages\": <n>, \"regressions\": <n>}. This file shape and the exact line are a contract dob-6 reads; do not change them.\n\n(4) SKILL TEXT. In packages/bee/design/bee-design/SKILL.md replace the raw npx call in the Check step (`npx --yes ease-design@0.6.1 gate <page.html> --dir . --json` at :77, and the `ui <command>` note at :41 where it concerns the gate) with `bee design gate`, saying the not-run line is what a Node-less machine prints and that the proof reason must then carry `design gate not run`. Make the same change in packages/bee/agents/bee-design.md.tmpl and in DESIGN_CELL_PREAMBLE (prompt.rs `pub(crate) const DESIGN_CELL_PREAMBLE: &str = \"Design cell:`). Then rewrite the rendered agents with `bee dev regen` using the freshly built binary, and the release manifest with `bee dev release-manifest --write`. Revert any regen change outside this cell's files (.gitattributes, .bee/onboarding.json).\n\nTests: clean pages -> exit 0, one line; error rise -> exit 1 naming the page; coverage fall -> exit 1; new page with errors -> exit 1; new page with 0 errors -> exit 0; malformed output -> exit 1; --write with no claimed cell writes the baseline; --write with a claimed cell refuses and writes nothing; node 18 -> exact not-run line, exit 0, last-run.json not-run; a ran run writes last-run.json ran; design.pages globs limit the pages.",
    "verify": ".bee/bin/bee dev release-manifest --check && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "each new test fails first for its own reason",
        "a clean run exits 0 with one summary line",
        "an error rise, a coverage fall, or a new page with errors exits 1 naming the page",
        "--write refuses and writes nothing while any cell is claimed",
        "a Node-less or Node-below-20 run prints the exact not-run line, writes last-run.json not-run and exits 0",
        "the core skill, the agent template and the preamble name bee design gate, not the raw npx call",
        "the registry and cli-shape tests stay green with the new verb"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/design_group.rs", "substantive": "the gate verb: pages, tool run, baseline compare, write guard, Node check, last-run record"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "design.gate entry"}
      ],
      "key_links": [
        "verbs/mod.rs routes design to design_group",
        "last-run.json shape and the not-run line match the contract dob-6 reads"
      ],
      "prohibitions": [
        "Do not call the real npx or network in tests",
        "Do not change the cap path in this cell",
        "Do not add design skills to any plugin tree"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "dob-6",
    "feature": "design-os-in-bee",
    "title": "Refuse a UI cell's cap without the design gate in a host that picked packs",
    "lane": "high-risk",
    "role": "test",
    "deps": [],
    "decisions": ["587bcc85-ac38-42a1-87e5-9fae09a85db9", "d5399608-c7db-4184-968f-1f870805a522", "572bc11e-0249-4ef8-97ef-a2dc996d4a9b"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/design_rule.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "docs/history/design-os-in-bee/CONTEXT.md",
      "docs/history/design-os-in-bee/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/state.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST: write the failing tests before the code and watch each fail for its own reason.\n\n(1) WHEN THE RULE BINDS (owner answer 2). Read the host config with `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` (state.rs:161). The rule binds only when design.packs is an array with at least one entry. design.packs absent, \"none\", or an empty array -> the cap behaves exactly as today.\n\n(2) UI CELL (per D6). A cell is a UI cell when any path in its `files` matches design.ui_globs (config list of globs; default *.html, *.css, *.scss, *.jsx, *.tsx, *.vue, *.svelte, matched against the file name at any depth). Put the binding test and the glob match in a small new module verbs/cells/design_rule.rs (declared in verbs/cells/mod.rs) so the cap and the add warning share it.\n\n(3) CAP REFUSALS, in handlers_close.rs with the cheap pre-checks beside `if js_trim(&proof_command) != cell_verify {` (:308), before any write, each naming its fix in the house refusal shape (`capCell: cell \"<id>\" refused — <what>. FIX: <remedy>`): (a) a UI cell whose verify does not contain `bee design gate` -> refuse, FIX: add `bee design gate` to the cell's verify (bee cells update while open) and run it; (b) a UI cell where .bee/design/last-run.json has status \"not-run\" and the proof reason segment does not contain `design gate not run` -> refuse, FIX: say in the proof reason that the design gate did not run because Node 20 or newer is missing; when it does contain it, the cap is accepted (owner answer 1); (c) any cell, UI or not, whose `files` include .bee/design/gate-baseline.json -> refuse, FIX: the baseline is recorded by a person with `bee design gate --write` when no cell is claimed. last-run.json shape (contract with dob-5): {\"status\": \"not-run\", \"reason\": \"node-missing\"} or {\"status\": \"ran\", \"pages\": n, \"regressions\": n}; a missing or unreadable file means no not-run requirement.\n\n(4) EARLY WARNING. In bee cells add (handlers_write.rs), when the rule binds and a UI cell's verify lacks `bee design gate`, print one stderr warning naming the cell and the fix; never refuse, never change the cell.\n\nTests: no-pick host, UI cell without the gate -> cap accepted (unchanged); picked host, UI cell without the gate -> refused with the FIX text; picked host, UI cell with the gate in verify -> accepted; picked host, non-UI cell -> accepted; last-run not-run and reason lacking `design gate not run` -> refused; same with the phrase -> accepted; a cell listing .bee/design/gate-baseline.json -> refused in a picked and a no-pick host; design.packs \"none\" -> unchanged; cells add warns on stderr and still adds the cell.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "each new test fails first for its own reason",
        "in a host with no pick the cap behaves exactly as before",
        "in a picked host a UI cell whose verify lacks bee design gate is refused with the fix named",
        "a not-run record requires the proof reason to say design gate not run, and then the cap is accepted",
        "a cell listing the baseline file is refused",
        "cells add warns on stderr and still adds the cell"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/design_rule.rs", "substantive": "binding test, UI glob match and last-run read shared by cap and add"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "substantive": "the three cap refusals before any write"}
      ],
      "key_links": [
        "cap and add both call design_rule",
        "the last-run.json shape matches the contract dob-5 writes"
      ],
      "prohibitions": [
        "Do not change the cap for hosts with no design pick",
        "Do not run any external program at cap",
        "Do not touch the design gate verb files in this cell"
      ]
    },
    "behavior_change": true
  }
]
```

## Verify

dob-5: `bee dev release-manifest --check`, then the declared `commands.test` —
it edits `packages/bee`, an inventory root. dob-6: the declared `commands.test`.

## Test matrix

| Case | Kind | Pass when |
|---|---|---|
| Clean gate | happy | exit 0, one summary line |
| Error rise | error | exit 1, the page named with old and new counts |
| New page with errors | error | exit 1 |
| No Node | edge | exact not-run line, exit 0, `last-run.json` not-run |
| `--write` mid-cell | error | refused, baseline bytes unchanged |
| UI cell without gate, picked host | error | cap refused, FIX names `bee design gate` |
| Same, no-pick host | regression | cap accepted, same as main |
| Not-run proof | edge | accepted only when the reason says `design gate not run` |
| Baseline in cell files | error | cap refused |

## Open Questions

- Upstream `THIRD_PARTY_LICENSES.md` says MIT while `LICENSE` is Shield.
- What a "page" is in a React or Vue host with no static HTML: this slice gates
  `design.pages` (default `*.html`); a build step that emits HTML is the host's
  to wire, named in the capture.

## Out of scope

- Changing design-os itself.
- Running a build to produce pages.
