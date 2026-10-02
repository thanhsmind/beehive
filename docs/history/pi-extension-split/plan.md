---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: pi-extension-split

## Summary

The Pi guard is one 3767-line file today. After this work it is a folder,
`.pi/extensions/bee-guard/`, with a short `index.ts` and one small file per
job: finding bee, running hooks, mapping tools, the result inbox, the worker
widget, session moves, the status line, the bee tools, the event handlers and
the slash commands. Pi sees the same guard and the same behavior.

`bee onboard` installs the folder in a host repo and removes the old single
file, so Pi never loads the guard twice. `bee doctor --runtime pi` checks
every file in the folder.

Mode: `high-risk` — 4 risk flags: external-systems, public-contracts,
covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the installed path
changes in every host repo and three Rust surfaces read the old path, so a
silent miss leaves a host with no guard or with two guards.

## Requirements (from CONTEXT.md)

- D1: directory extension `.pi/extensions/bee-guard/index.ts`, one module
  per concern, `.ts` import specifiers, no behavior change, every old
  export re-exported from `index.ts`.
- D2: module-level mutable state in one `state.ts` object; Maps and Sets
  stay exported consts.
- D3: onboard vendors the whole folder, deletes the legacy single file,
  prunes files bee does not ship inside `bee-guard/`.
- D4: doctor compares every embedded module file; `build.rs` generates the
  list from the folder; doctor fails when the legacy file is present.
- D5: contract tests derive from the concatenated module sources and run
  `index.ts`; comments move verbatim.

## Load-bearing claims

Labels: `read` (opened at that line), `ran` (command output held). Evidence
is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Pi 1.0 loads a subdirectory with an `index.ts` entry, and also loads top-level `.ts` files | ran | `sed -n 46p /home/thanhsmind/.local/share/mise/installs/pi/1.0.0/pi/docs/extensions.md` | Pi loads direct TypeScript or JavaScript files and subdirectories containing an `index.ts` or `index.js` entry point. |
| 2 | Onboard today vendors only top-level `.ts` files of `.pi/extensions/` | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:128 | .filter(\|e\| e.is_file && e.name.ends_with(".ts")) |
| 3 | Doctor embeds the single file at compile time | read | packages/bee-rs/crates/bee/src/doctor.rs:47 | const PI_EXTENSION_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts"); |
| 4 | Doctor names the single file as the Pi wiring file | read | packages/bee-rs/crates/bee/src/doctor.rs:77 | Runtime::Pi => ".pi/extensions/bee-guard.ts", |
| 5 | The Pi contract suite embeds the single file | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:107 | const PI_PLUGIN_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts"); |
| 6 | The OpenCode contract suite also embeds the Pi file | read | packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs:138 | const PI_PLUGIN_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts"); |
| 7 | A test reads a comment line of the Pi file as a contract marker | read | packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs:520 | PI_PLUGIN_SOURCE.lines().any(\|line\| line.contains(rule) && line.contains("NAMED EXCLUSION")) |
| 8 | The comment carrying that marker sits in the file header | read | .pi/extensions/bee-guard.ts:52 | // model-guard is a NAMED EXCLUSION on this belt — n/a — Pi has NO native |
| 9 | The node harness loads the extension by path | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1102 | cmd.arg(harness).arg(&extension); |
| 10 | The default export keeps per-invocation locals shared by handlers and commands | read | .pi/extensions/bee-guard.ts:2771 | let fullToolSet: string[] \| null = null |
| 11 | Module-level `let` state exists that several sections write | read | .pi/extensions/bee-guard.ts:568 | let cachedPreamble: string \| null = null |
| 12 | The release manifest walks `.pi/extensions` recursively | read | packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:298 | walk(root, top, &abs, role, exclude_top, out)?; |
| 13 | Node 24 runs `.ts` modules that import each other with `.ts` specifiers and shares object state | ran | `node /tmp/claude-1000/-home-thanhsmind-Projects-goglbe-beehive/86eae0ba-56e7-4a36-a190-3a13f12344f0/scratchpad/probe/index.ts` | 2 |
| 14 | The crate already has a build.rs that declares rerun inputs | read | packages/bee-rs/crates/bee/build.rs:21 | println!("cargo:rerun-if-changed=../../../../.claude-plugin/plugin.json"); |

## Discovery

Read the full declaration map of `.pi/extensions/bee-guard.ts` with
`rg -n '^(export )?(async )?(function|const|let|type|interface|class) '`:
sections fall on clean seams (locate 100-166, hooks 168-323, tool map
325-567, session helpers 568-646, result inbox 647-990 and 1137-1195, worker
widget 991-1136, bee cli and session transition 1196-2037, model usage
2038-2282, verdict tool 2283-2467, dispatch tools 2468-2572, steer tools
2573-2769, the belt 2770-3744, exports 3745-3767). Pi 1.0 docs confirm the
`index.ts` folder shape. Every Rust reader of the old path is listed in the
claims table.

## Approach

Playbook: `skills/bee-planning/playbooks/refactor.md`. Step 1-2 (record
first): the existing Pi and OpenCode contract suites are the record; the
leader runs them green on the unchanged tree before pes-1 starts. Step 6
(subtract first): nothing to subtract — no dead code was found in the map;
named deviation. Step 7 (API move): pes-1 migrates every reader of the old
path in the same slice; pes-3 sweeps the live docs.

Module layout (D1), all under `.pi/extensions/bee-guard/`:
`index.ts` (header comment, default export, export list), `locate.ts`,
`hooks.ts`, `tool-map.ts`, `state.ts`, `session.ts`, `result-inbox.ts`,
`workers-widget.ts`, `bee-cli.ts`, `transition.ts`, `model-usage.ts`,
`tools/verdict.ts`, `tools/dispatch.ts`, `tools/steer.ts`,
`events/guard.ts`, `events/session.ts`, `events/prompt.ts`,
`events/turn.ts`, `commands.ts`. The default export keeps its per-invocation
locals (`fullToolSet`, `lastStage`, `toolsReopened`) in one object made
inside the factory and passes it to each `register*` function, so a Pi
reload still starts them fresh.

Rejected: keep one file and add regions (does not meet the ask); bundle a
build step to one file (adds a build, D1 forbids); a hand-kept module list
in doctor (drifts, D4).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Cross-module writes to `let` state | HIGH | pes-1 | Pi contract suite green, node harness runs `index.ts` |
| Test derivations losing a marker | MEDIUM | pes-1 | both contract suites green, no assertion edited |
| Host with legacy file loads guard twice | HIGH | pes-2 | onboard unit test: legacy file planned for removal and removed |
| Doctor false OK on a partial folder | MEDIUM | pes-1 | doctor unit test: one changed module file fails the row |
| Stale docs naming the old path | LOW | pes-3 | `rg` finds no live doc naming the old file |

Waves: pes-1 alone (pes-2 and pes-3 build on the new folder); then pes-2 and
pes-3 in parallel (disjoint files).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pes-1 and pes-2 move the TypeScript and change the Rust installer and doctor."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"pes-3 updates the live docs that name the old path."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each code cell."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"required","role":"advisor","reason":"High-risk gate: the hat-wave synthesis is recorded as the advisor ref."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Plan-step hat wave."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Plan-step hat wave."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"Plan-step hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Plan-step hat wave."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Plan-step hat wave."}
  ]
}
```

## Shape

Epic map. Outcome: the Pi guard is a folder of small modules, installed and
checked as a folder, with every existing proof still green.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Split | TypeScript module layout + test and doctor readers | The ask | 1 | contract suites + doctor tests green |
| Install | Onboard vendors the folder, removes the legacy file | Hosts must not load two guards | 1 | onboard tests green |
| Docs | Live docs name the folder | Readers find the code | 1 | no live doc names the old file |

Slice queue: slice 1 holds all three cells. Current slice: slice 1.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pes-1 | Split the Pi guard into a folder of small modules | `.pi/extensions/bee-guard/**` (new), `.pi/extensions/bee-guard.ts` (deleted), build.rs, doctor.rs, doctor/tests.rs, both contract test files | — | Pi loads `bee-guard/index.ts`; doctor checks every module file | contract suites + doctor tests green |
| pes-2 | Install the guard folder and remove the old single file on onboard | onboard plan.rs, apply.rs, tests.rs | pes-1 | `bee onboard --apply` writes the folder and deletes `.pi/extensions/bee-guard.ts` | onboard tests green |
| pes-3 | Point the live docs at the guard folder | docs/knowledge, docs/*.md, verify-app feature files | pes-1 | docs name `.pi/extensions/bee-guard/` | rg finds no live doc naming the old file |

```json
[
  {
    "id": "pes-1",
    "feature": "pi-extension-split",
    "lane": "high-risk",
    "role": "code",
    "change_class": "refactor",
    "title": "Split the Pi guard into a folder of small modules",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D4",
      "D5",
      "19afb54e-dadc-48af-b293-fee2398e47bf",
      "75e947e9-554d-41cd-89cb-ecfe99ea7ace",
      "8822545c-f8df-4dce-b277-95af37922795",
      "b8635d0e-5609-4bda-b58a-b948505e1cbb"
    ],
    "files": [
      ".pi/extensions/bee-guard.ts",
      ".pi/extensions/bee-guard/index.ts",
      ".pi/extensions/bee-guard/locate.ts",
      ".pi/extensions/bee-guard/hooks.ts",
      ".pi/extensions/bee-guard/tool-map.ts",
      ".pi/extensions/bee-guard/state.ts",
      ".pi/extensions/bee-guard/session.ts",
      ".pi/extensions/bee-guard/result-inbox.ts",
      ".pi/extensions/bee-guard/workers-widget.ts",
      ".pi/extensions/bee-guard/bee-cli.ts",
      ".pi/extensions/bee-guard/transition.ts",
      ".pi/extensions/bee-guard/model-usage.ts",
      ".pi/extensions/bee-guard/tools/verdict.ts",
      ".pi/extensions/bee-guard/tools/dispatch.ts",
      ".pi/extensions/bee-guard/tools/steer.ts",
      ".pi/extensions/bee-guard/events/guard.ts",
      ".pi/extensions/bee-guard/events/session.ts",
      ".pi/extensions/bee-guard/events/prompt.ts",
      ".pi/extensions/bee-guard/events/turn.ts",
      ".pi/extensions/bee-guard/commands.ts",
      "packages/bee-rs/crates/bee/build.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/pi-extension-split/CONTEXT.md",
      "docs/history/pi-extension-split/plan.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/build.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "action": "Move .pi/extensions/bee-guard.ts into the folder .pi/extensions/bee-guard/ with the module layout named in plan.md Approach, per D1. Move code verbatim: no logic change, every comment moves with its code, the file header comment (lines 1-94, with the NAMED EXCLUSION lines) goes to index.ts. Relative imports use .ts specifiers. Per D2, every module-level let (cachedPreamble, preambleInjected, sessionInitRun, drainTimer, drainToken, drainDirectory, activeDrainCtx, drainInFlight, turnStartPending, selfBusy, activeTransition, transitionTeardownOccurred, cachedLimitsText, lastLimitsFetchTime, lastLimitsModelKey, isFetchingLimits, dispatchCounter, workerSteerDrainInFlight) becomes a field of one exported state object in state.ts and every read and write uses it. The default export makes one object holding fullToolSet, lastStage and toolsReopened inside the factory and passes it to register functions in events/*.ts and commands.ts. index.ts re-exports every name in the old export list. Delete the old file. Per D4, build.rs walks .pi/extensions/bee-guard/ and writes a generated Rust file listing (relative path, include_str!) pairs with rerun-if-changed on the folder; doctor.rs uses that list, hooks_rel for Pi becomes .pi/extensions/bee-guard/index.ts, the wiring row compares every file byte for byte and fails naming the first differing or missing file, and it fails when .pi/extensions/bee-guard.ts exists. Update doctor/tests.rs to write the folder. Per D5, both contract test files replace the include_str! constant with one helper that reads and concatenates index.ts then every other .ts file under the folder in sorted path order, and the node harness path points at index.ts. Never edit an assertion to make it pass. No new code comments.",
    "must_haves": {
      "truths": [
        "the Pi contract suite is green with no assertion changed",
        "the OpenCode contract suite is green with no assertion changed",
        "doctor fails the wiring row when one module file differs",
        "doctor fails the wiring row when the legacy single file exists",
        "no module assigns a let binding imported from another module",
        "index.ts re-exports every name the old file exported"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/index.ts",
          "substantive": "default export registers every handler, tool and command"
        },
        {
          "path": ".pi/extensions/bee-guard/state.ts",
          "substantive": "one exported state object holding the former module-level lets"
        }
      ],
      "key_links": [
        "doctor.rs reads the build.rs generated file list",
        "pi_plugin_contracts.rs runs .pi/extensions/bee-guard/index.ts"
      ],
      "prohibitions": [
        "No guard behavior change",
        "No assertion edited to pass",
        "No new code comments",
        "No hand-kept module list in Rust"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts --test opencode_plugin_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor::",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pes-2",
    "feature": "pi-extension-split",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Install the guard folder and remove the old single file on onboard",
    "deps": ["pes-1"],
    "decisions": [
      "D3",
      "0c06dba7-d2e6-4674-8843-72383467e385"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-extension-split/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "action": "Red first in onboard/tests.rs, then per D3: list_pi_extension_files returns every file under the source .pi/extensions/bee-guard/ as a relative path (bee-guard/index.ts, bee-guard/tools/verdict.ts, ...) in sorted order. The plan step creates the missing directories, plans copy_pi_extension for each file missing or drifted at .pi/extensions/<rel>, plans a remove item for .pi/extensions/bee-guard.ts when it exists, and plans a remove item for each file under the host .pi/extensions/bee-guard/ that the source does not ship. apply.rs copies by relative path (not basename) and performs the removes. Update the existing Pi onboard tests to the folder shape. No new code comments.",
    "must_haves": {
      "truths": [
        "onboard plans one copy_pi_extension item per drifted module file",
        "onboard apply writes every module file under .pi/extensions/bee-guard/",
        "onboard plans and performs removal of a legacy .pi/extensions/bee-guard.ts",
        "onboard removes a host file under bee-guard/ that bee does not ship",
        "a second onboard on an up-to-date host plans no Pi item"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/plan.rs",
          "substantive": "folder listing, legacy removal and prune items"
        }
      ],
      "key_links": [
        "apply.rs copies copy_pi_extension by relative path"
      ],
      "prohibitions": [
        "No change to the OpenCode plugin vendoring",
        "No new code comments"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee onboard::",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pes-3",
    "feature": "pi-extension-split",
    "lane": "high-risk",
    "role": "docs",
    "change_class": "docs",
    "title": "Point the live docs at the guard folder",
    "deps": ["pes-1"],
    "decisions": [
      "D1",
      "19afb54e-dadc-48af-b293-fee2398e47bf"
    ],
    "files": [
      "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md",
      "docs/knowledge/areas/hook-runtime/agent-activity-record.md",
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md",
      "docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md",
      "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md",
      "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md",
      "docs/knowledge/areas/onboarding/status-display-vendoring.md",
      "docs/knowledge/areas/onboarding/repo-local-guardrails.md",
      "docs/config-reference.md",
      "docs/01-distillation.md",
      "docs/02-architecture.md",
      "docs/06-runtime-integration.md",
      "docs/product-description/maintenance/onboarding.md",
      "docs/product-description/observability/status.md",
      ".bee/verify/verify-app/features/pi-runtime.md",
      ".claude/skills/verify-app/features/pi-runtime.md",
      ".agents/skills/verify-app/features/pi-runtime.md",
      ".claude/skills/verify-app/features/semantic-role-routing.md",
      ".agents/skills/verify-app/features/semantic-role-routing.md",
      ".bee/config-sample.json"
    ],
    "read_first": [
      "docs/history/pi-extension-split/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md"
    ],
    "action": "Per D1, replace every live mention of the single file .pi/extensions/bee-guard.ts with the folder .pi/extensions/bee-guard/ (entry index.ts), and name the module that holds a fact when a doc points at one function. Add a short module map to docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md or the hook-runtime concept that owns the Pi belt, citing pi-extension-split D1-D5. Leave docs/history/** and .bee/*.jsonl untouched.",
    "must_haves": {
      "truths": [
        "no file outside docs/history and .bee jsonl stores names .pi/extensions/bee-guard.ts except as the legacy file onboard removes",
        "one knowledge concept carries the module map and cites pi-extension-split D1-D5"
      ],
      "artifacts": [
        {
          "path": "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md",
          "substantive": "names the bee-guard folder"
        }
      ],
      "key_links": [
        "the module map names each file under .pi/extensions/bee-guard/"
      ],
      "prohibitions": [
        "No edit under docs/history",
        "No edit to .bee jsonl stores"
      ]
    },
    "verify": "rg -n 'extensions/bee-guard\\.ts' docs/knowledge docs/config-reference.md docs/01-distillation.md docs/02-architecture.md docs/06-runtime-integration.md docs/product-description .bee/verify .claude/skills/verify-app .agents/skills/verify-app .bee/config-sample.json",
    "affects_skills": [],
    "affects_specs": []
  }
]
```

## Test matrix

High-risk: the 12 edge dimensions, applied where they bite.

| Dimension | Probe | Pass when |
|---|---|---|
| Happy path | Pi contract suite runs `index.ts` against the stub | `test result: ok` for pi_plugin_contracts |
| Contract derivation | OpenCode suite derives Pi tool/hook pairs from the concatenated source | `test result: ok` for opencode_plugin_contracts |
| State sharing | inbox drain and turn handlers share `state` across modules (existing drain tests) | existing drain tests pass unchanged |
| Reload | factory locals made per call | `rg -n 'let (fullToolSet\|lastStage\|toolsReopened)' .pi/extensions/bee-guard` finds nothing at module level |
| Exports | node imports `derivePostExitTimeoutMs` from `index.ts` (existing test) | that test prints `OK` |
| Doctor drift | one module file changed in a host | doctor row `wiring_matches_binary` is fail, naming that file |
| Doctor legacy | legacy single file present | doctor row fails, naming `.pi/extensions/bee-guard.ts` |
| Doctor clean | folder byte-identical | doctor row is ok |
| Onboard fresh | empty host | apply writes every module file |
| Onboard upgrade | host with legacy file | plan holds a remove item; after apply the file is gone |
| Onboard prune | host has `bee-guard/old.ts` | plan removes it |
| Idempotent | second onboard | no Pi plan item |

## Open Questions

(none)

## Out of scope

- The OpenCode plugin keeps its one-file shape.
- No guard behavior change.
- `docs/history/**` keeps its historical paths.
