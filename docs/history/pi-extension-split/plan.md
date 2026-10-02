---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: pi-extension-split

## Summary

The Pi guard is one 3767-line file today. After this work it is a flat
folder, `.pi/extensions/bee-guard/`, with a short `index.ts` and one small
file per job: finding bee, running hooks, mapping tools, shared state, the
result inbox, the worker widget, session moves, the status line, the bee
tools, the event handlers and the slash commands. Pi sees the same guard and
the same behavior.

`bee onboard` installs the folder in a host repo and removes the old single
file first, so Pi never loads the guard twice. `bee doctor --runtime pi`
checks every file in the folder and tells the user to run
`bee onboard --apply` when something is off.

Mode: `high-risk` — 4 risk flags: external-systems, public-contracts,
covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the installed path
changes in every host repo and several Rust surfaces read the old path, so a
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
- D6: flat folder; all `pi.on` handlers in one `events.ts` in original
  order; `register*` functions take `pi`; shipped set = every `.ts` file in
  the folder sorted by name; legacy remove planned before copies and only
  when the source ships files; prune only unshipped `.ts`, no symlink
  follow; remove arm scoped to `.pi/extensions/bee-guard/*.ts` or the exact
  legacy path; doctor text names `bee onboard --apply`.

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
| 15 | The agent_settled slice test ends its cut at the session_before_compact handler, so those handlers must stay in one file in order | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:380 | .find("pi.on(\"session_before_compact\"") |
| 16 | `.pi/extensions` is a release-manifest inventory root, so deleting the file owes a manifest regen | read | packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:83 | ".pi/extensions", |
| 17 | The manifest lists the old file today | read | docs/history/codex-harness-hardening/release-manifest.json:1776 | "path": ".pi/extensions/bee-guard.ts", |
| 18 | Onboard already has a vendor-and-prune pattern to copy | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:804 | plan.push(plan_item("remove_expertise", &format!(".bee/expertise/{name}"))); |
| 19 | The Pi copy arm copies by base name, so a folder needs a relative-path copy | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:489 | let name = posix_basename(rel); |

## Discovery

Read the full declaration map of `.pi/extensions/bee-guard.ts` with
`rg -n '^(export )?(async )?(function|const|let|type|interface|class) '`:
sections fall on clean seams (locate 100-166, hooks 168-323, tool map
325-567, session helpers 568-646, result inbox 647-990 and 1137-1195, worker
widget 991-1136, bee cli and session transition 1196-2037, model usage
2038-2282, verdict tool 2283-2467, dispatch tools 2468-2572, steer tools
2573-2769, the belt 2770-3744, exports 3745-3767). Pi 1.0 docs confirm the
`index.ts` folder shape. Baseline: both contract suites ran green on the
unchanged tree (`cargo test --release --no-fail-fast -p bee --test
pi_plugin_contracts --test opencode_plugin_contracts`: 104 passed and 8
passed). The plan-step hat wave report is
`docs/history/pi-extension-split/reports/hat-wave.md`.

## Approach

Playbook: `skills/bee-planning/playbooks/refactor.md`. Step 1-2 (record
first): the Pi and OpenCode contract suites are the record, green on the
unchanged tree (see Discovery). Step 6 (subtract first): no dead code was
found in the map; named deviation. Step 7 (API move): pes-1 migrates every
reader of the old path in the same slice; pes-4 sweeps the live docs.
Step 8: no assertion is edited; only the source-loading helper changes (D5).

Module layout (D1, D6), flat under `.pi/extensions/bee-guard/`:
`index.ts` (header comment lines 1-94, default export, export list),
`locate.ts`, `hooks.ts`, `tool-map.ts`, `state.ts`, `session.ts`,
`result-inbox.ts`, `workers-widget.ts`, `bee-cli.ts`, `transition.ts`,
`model-usage.ts`, `tool-verdict.ts`, `tool-dispatch.ts`, `tool-steer.ts`,
`events.ts` (every `pi.on` handler, original order), `commands.ts`. The
default export keeps its per-invocation locals (`fullToolSet`, `lastStage`,
`toolsReopened`) in one object made inside the factory and passes it to
`registerEvents(pi, belt)` and `registerCommands(pi, belt)`, so a Pi reload
still starts them fresh.

Rejected: keep one file with regions (does not meet the ask); bundle a
build step to one file (adds a build, D1); subfolders `tools/` and
`events/` (nesting not proven safe on Pi 1.0, and the agent_settled slice
cut needs the handlers in one file, D6); a hand-kept module list in doctor
or a static Rust array (drifts, D4); the `include_dir` crate (adds a
dependency, D4 locks build.rs); per-domain state with accessors (D2 locked;
read-only ES live bindings make it riskier); a startup duplicate check in
`index.ts` (a behavior change, out of scope; onboard ordering removes the
case); a `.bak` copy of a drifted legacy file (onboard already overwrites
drifted vendored files without a backup; same posture); a module-map
comment in `index.ts` (no new code comments; the map lives in the knowledge
concept); debug-mode cell proof (CI runs `--release`).

Rollback: a code revert leaves a host that already ran the new onboard with
the folder. An older bee's onboard then copies `bee-guard.ts` back beside
it and Pi loads two guards. Manual step on downgrade:
`rm -r .pi/extensions/bee-guard/`. pes-1 to pes-3 land in one merge; no
release happens between them.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Cross-module writes to `let` state | HIGH | pes-1 | Pi contract suite green, node harness runs `index.ts` |
| Test derivations losing a marker | MEDIUM | pes-1 | both contract suites green, no assertion edited |
| Doctor false OK on a partial folder | MEDIUM | pes-2 | doctor unit test: one changed module file fails the row |
| Host with legacy file loads guard twice | HIGH | pes-3 | onboard test: legacy remove is planned before every copy, and removed |
| Prune deletes a user file | HIGH | pes-3 | onboard test: a non-`.ts` file and an out-of-scope path survive |
| Stale docs naming the old path | LOW | pes-4 | `! rg` over live docs finds nothing |

Waves: pes-1 alone. Then pes-2 and pes-4 in parallel (disjoint files). Then
pes-3 — serial after pes-2 because both compile the same crate in one
worktree, and a half-written cell breaks the other's build.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pes-1 to pes-3 move the TypeScript and change doctor and the installer."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"pes-4 updates the live docs that name the old path."},
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

Epic map. Outcome: the Pi guard is a flat folder of small modules, installed
and checked as a folder, with every existing proof still green.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Split | TypeScript module layout + test readers + entry-file doctor | The ask | 1 | contract suites + doctor tests + manifest check green |
| Check | Doctor compares the whole folder | A partial folder must not read as OK | 1 | doctor tests green |
| Install | Onboard vendors the folder, removes the legacy file first | Hosts must not load two guards | 1 | onboard tests green |
| Docs | Live docs name the folder | Readers find the code | 1 | no live doc names the old file |

Slice queue: slice 1 holds all four cells. Current slice: slice 1.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pes-1 | Split the Pi guard into a folder of small modules | `.pi/extensions/bee-guard/*.ts` (new), `.pi/extensions/bee-guard.ts` (deleted), doctor, contract tests, manifest | — | Pi loads `bee-guard/index.ts`; contract suites stay green | contract suites + doctor tests + manifest check green |
| pes-2 | Check every guard module file in bee doctor | build.rs, doctor.rs, doctor/tests.rs | pes-1 | `bee doctor --runtime pi` names each drifted module and says to run `bee onboard --apply` | doctor tests green |
| pes-3 | Install the guard folder and remove the old single file on onboard | onboard plan.rs, apply.rs, tests.rs | pes-2 | `bee onboard --apply` deletes `.pi/extensions/bee-guard.ts` first, then writes the folder | onboard tests green |
| pes-4 | Point the live docs at the guard folder | docs/knowledge areas, docs/*.md, verify-app feature files | pes-1 | docs name `.pi/extensions/bee-guard/` and carry a module map | no live doc names the old file |

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
      "D5",
      "D6",
      "19afb54e-dadc-48af-b293-fee2398e47bf",
      "75e947e9-554d-41cd-89cb-ecfe99ea7ace",
      "b8635d0e-5609-4bda-b58a-b948505e1cbb",
      "8cdd7725-c059-4b92-aaef-b548c427ae36"
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
      ".pi/extensions/bee-guard/tool-verdict.ts",
      ".pi/extensions/bee-guard/tool-dispatch.ts",
      ".pi/extensions/bee-guard/tool-steer.ts",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/commands.ts",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs",
      "packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs",
      "packages/bee-rs/crates/bee/src/devtools/mod.rs",
      "packages/bee-rs/crates/bee/src/devtools/release_manifest.rs",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pi-extension-split/CONTEXT.md",
      "docs/history/pi-extension-split/plan.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "action": "Move .pi/extensions/bee-guard.ts into the flat folder .pi/extensions/bee-guard/ with exactly the module layout in plan.md Approach, per D1 and D6. Move code verbatim: no logic change, every comment moves with its code, the file header comment (lines 1-94, with the NAMED EXCLUSION lines) goes to the top of index.ts. Relative imports use ./name.ts specifiers. Every module ends with a newline. Per D2, every module-level let (cachedPreamble, preambleInjected, sessionInitRun, drainTimer, drainToken, drainDirectory, activeDrainCtx, drainInFlight, turnStartPending, selfBusy, activeTransition, transitionTeardownOccurred, cachedLimitsText, lastLimitsFetchTime, lastLimitsModelKey, isFetchingLimits, dispatchCounter, workerSteerDrainInFlight) becomes a field of one exported state object in state.ts and every read and write uses state.<name>. Per D6, every pi.on handler moves into events.ts in its original order inside registerEvents(pi: ExtensionAPI, belt), every pi.registerCommand into commands.ts inside registerCommands(pi: ExtensionAPI, belt), and the default export in index.ts makes the belt object { fullToolSet, lastStage, toolsReopened } inside the factory. index.ts re-exports every name in the old export list. Delete the old file. In doctor.rs, embed .pi/extensions/bee-guard/index.ts for now, set hooks_rel for Pi to .pi/extensions/bee-guard/index.ts, and fail the wiring row when .pi/extensions/bee-guard.ts exists with text that names bee onboard --apply; update doctor/tests.rs to match. Per D5, in both contract test files replace the include_str! constant with one helper that reads index.ts, then every other .ts file in the folder sorted by name, joined with a newline, and point the node harness path at index.ts. Never edit an assertion. Update the old-path mentions in stage_tools.rs, hook_manifests.rs, devtools/mod.rs and release_manifest.rs doc comments to the folder without adding comment lines. Then run .bee/bin/bee dev release-manifest --write and commit the regenerated manifest. No new code comments.",
    "must_haves": {
      "truths": [
        "the Pi contract suite is green with no assertion changed",
        "the OpenCode contract suite is green with no assertion changed",
        "the release manifest check is green",
        "no module assigns a let binding imported from another module",
        "index.ts re-exports every name the old file exported",
        "the header comment with the NAMED EXCLUSION lines opens index.ts",
        "doctor fails the wiring row naming bee onboard --apply when the legacy single file exists"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/index.ts",
          "substantive": "default export registers every handler, tool and command"
        },
        {
          "path": ".pi/extensions/bee-guard/state.ts",
          "substantive": "one exported state object holding the former module-level lets"
        },
        {
          "path": ".pi/extensions/bee-guard/events.ts",
          "substantive": "every pi.on handler in its original order"
        }
      ],
      "key_links": [
        "pi_plugin_contracts.rs runs .pi/extensions/bee-guard/index.ts",
        "doctor.rs embeds .pi/extensions/bee-guard/index.ts"
      ],
      "prohibitions": [
        "No guard behavior change",
        "No assertion edited to pass",
        "No new code comments",
        "No subfolder under .pi/extensions/bee-guard/"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts --test opencode_plugin_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor:: && .bee/bin/bee dev release-manifest --check",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pes-2",
    "feature": "pi-extension-split",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Check every guard module file in bee doctor",
    "deps": ["pes-1"],
    "decisions": [
      "D4",
      "D6",
      "8822545c-f8df-4dce-b277-95af37922795",
      "8cdd7725-c059-4b92-aaef-b548c427ae36"
    ],
    "files": [
      "packages/bee-rs/crates/bee/build.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-extension-split/CONTEXT.md",
      "packages/bee-rs/crates/bee/build.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs"
    ],
    "action": "Red first in doctor/tests.rs, then per D4: build.rs lists every .ts file directly in .pi/extensions/bee-guard/ sorted by name, writes a generated Rust file into OUT_DIR holding (file name, include_str! of the absolute path) pairs, and prints rerun-if-changed for the folder. doctor.rs includes that generated file and replaces the single index.ts embed: the wiring row compares each embedded file byte for byte against .pi/extensions/bee-guard/<name> in the host, and also flags host .ts files in that folder that bee does not ship. On failure the detail names the count of mismatched files, the first few names, and bee onboard --apply. The legacy-file failure from pes-1 stays. No hand-kept file list. No new code comments.",
    "must_haves": {
      "truths": [
        "doctor passes the wiring row when every module file matches",
        "doctor fails the wiring row naming the file when one module file differs",
        "doctor fails the wiring row when one module file is missing",
        "doctor failure text names bee onboard --apply and the mismatch count",
        "adding a .ts file to the folder needs no Rust edit to be checked"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/build.rs",
          "substantive": "generates the bee-guard file list from the folder"
        }
      ],
      "key_links": [
        "doctor.rs includes the build.rs generated list"
      ],
      "prohibitions": [
        "No hand-kept module list in Rust",
        "No new code comments"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor::",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pes-3",
    "feature": "pi-extension-split",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Install the guard folder and remove the old single file on onboard",
    "deps": ["pes-2"],
    "decisions": [
      "D3",
      "D6",
      "0c06dba7-d2e6-4674-8843-72383467e385",
      "8cdd7725-c059-4b92-aaef-b548c427ae36"
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
    "action": "Red first in onboard/tests.rs, then per D3 and D6, copying the shape of the expertise vendor-and-prune code (plan.rs list_source_expertise, walk_md_tree, the remove_expertise item near plan.rs:804, and the remove_expertise arm in apply.rs): list_pi_extension_files returns every .ts file directly in the source .pi/extensions/bee-guard/ as bee-guard/<name>, sorted, never following symlinks. In the plan step, when that list is not empty: first plan remove_pi_extension for .pi/extensions/bee-guard.ts if it exists, then create .pi/extensions/bee-guard if missing, then plan copy_pi_extension for each missing or drifted file at .pi/extensions/bee-guard/<name>, then plan remove_pi_extension for each .ts file directly in the host .pi/extensions/bee-guard/ that the source does not ship. When the list is empty, plan no Pi removal. apply.rs copies copy_pi_extension by its path relative to .pi/extensions (not the base name). The new remove_pi_extension arm accepts only .pi/extensions/bee-guard.ts or .pi/extensions/bee-guard/<name>.ts with no .. and no further slash, refuses anything else, and reports a failed remove instead of ignoring it. Update the existing Pi onboard tests to the folder shape. No new code comments.",
    "must_haves": {
      "truths": [
        "onboard plans one copy_pi_extension item per drifted module file",
        "onboard apply writes every module file under .pi/extensions/bee-guard/",
        "the legacy remove_pi_extension item comes before every copy_pi_extension item",
        "onboard removes a legacy .pi/extensions/bee-guard.ts on apply",
        "onboard plans no Pi removal when the source folder ships no files",
        "onboard removes a host .ts file in bee-guard/ that bee does not ship",
        "a non-.ts file in the host bee-guard/ folder survives onboard",
        "the remove_pi_extension arm refuses a path outside its scope",
        "a second onboard on an up-to-date host plans no Pi item"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/plan.rs",
          "substantive": "folder listing, ordered legacy removal and prune items"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/apply.rs",
          "substantive": "relative-path copy and a scoped remove_pi_extension arm"
        }
      ],
      "key_links": [
        "apply.rs copies copy_pi_extension by relative path"
      ],
      "prohibitions": [
        "No change to the OpenCode plugin vendoring",
        "No removal outside .pi/extensions/bee-guard.ts and .pi/extensions/bee-guard/*.ts",
        "No new code comments"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee onboard::",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "pes-4",
    "feature": "pi-extension-split",
    "lane": "high-risk",
    "role": "docs",
    "change_class": "docs",
    "title": "Point the live docs at the guard folder",
    "deps": ["pes-1"],
    "decisions": [
      "D1",
      "D6",
      "19afb54e-dadc-48af-b293-fee2398e47bf",
      "8cdd7725-c059-4b92-aaef-b548c427ae36"
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
      ".bee/verify/verify-app/features/semantic-role-routing.md",
      ".claude/skills/verify-app/features/pi-runtime.md",
      ".claude/skills/verify-app/features/semantic-role-routing.md",
      ".agents/skills/verify-app/features/pi-runtime.md",
      ".agents/skills/verify-app/features/semantic-role-routing.md",
      ".opencode/skills/verify-app/features/pi-runtime.md",
      ".opencode/skills/verify-app/features/semantic-role-routing.md",
      ".bee/config-sample.json"
    ],
    "read_first": [
      "docs/history/pi-extension-split/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md"
    ],
    "action": "Per D1 and D6, replace every live mention of the single file .pi/extensions/bee-guard.ts in the listed files with the folder .pi/extensions/bee-guard/ (entry index.ts), and name the module that holds a fact when a doc points at one function. A doc that describes onboard upgrade names the legacy file as the one onboard removes. Add a short module map (one line per file in the folder) to docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md, citing pi-extension-split D1-D6, plus the downgrade step rm -r .pi/extensions/bee-guard/. Edit the .bee/verify feature file first, then make the .claude, .agents and .opencode copies byte-identical to it. Leave docs/history/**, docs/knowledge/work/** (delivery records) and .bee/*.jsonl untouched.",
    "must_haves": {
      "truths": [
        "no listed file names .pi/extensions/bee-guard.ts except as the legacy file onboard removes",
        "one knowledge concept carries the module map and cites pi-extension-split D1-D6",
        "the four verify-app feature copies are byte-identical"
      ],
      "artifacts": [
        {
          "path": "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md",
          "substantive": "names the bee-guard folder and carries the module map"
        }
      ],
      "key_links": [
        "the module map names each file in .pi/extensions/bee-guard/"
      ],
      "prohibitions": [
        "No edit under docs/history",
        "No edit under docs/knowledge/work",
        "No edit to .bee jsonl stores"
      ]
    },
    "verify": "! rg -n 'extensions/bee-guard\\.ts' docs/knowledge/areas docs/config-reference.md docs/01-distillation.md docs/02-architecture.md docs/06-runtime-integration.md docs/product-description .bee/verify .claude/skills/verify-app .agents/skills/verify-app .opencode/skills/verify-app .bee/config-sample.json | rg -v -i 'legacy'",
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
| Slice cut | agent_settled slice test cuts inside `events.ts` | that test passes unchanged |
| State sharing | inbox drain and turn handlers share `state` across modules (existing drain tests) | existing drain tests pass unchanged |
| Reload | factory locals made per call | `rg -n '^let ' .pi/extensions/bee-guard` finds lines only in `state.ts`, or none |
| Exports | node imports `derivePostExitTimeoutMs` from `index.ts` (existing test) | that test prints `OK` |
| Manifest | release manifest after the delete | `bee dev release-manifest --check` exits 0 |
| Doctor drift | one module file changed in a host | row `wiring_matches_binary` fails, naming that file and `bee onboard --apply` |
| Doctor missing | one module file absent | row fails, naming that file |
| Doctor legacy | legacy single file present | row fails, naming `.pi/extensions/bee-guard.ts` and `bee onboard --apply` |
| Doctor clean | folder byte-identical | row is ok |
| Onboard upgrade order | host with legacy file | the legacy `remove_pi_extension` index is lower than every `copy_pi_extension` index; after apply the file is gone |
| Onboard empty source | source folder ships no files | no `remove_pi_extension` item |
| Onboard prune | host has `bee-guard/old.ts` and `bee-guard/notes.md` | `old.ts` removed, `notes.md` kept |
| Onboard scope | remove arm given `.pi/extensions/../x.ts` | refused, nothing deleted |
| Idempotent | second onboard | no Pi plan item |

## Open Questions

(none)

## Out of scope

- The OpenCode plugin keeps its one-file shape.
- No guard behavior change.
- `docs/history/**` and `docs/knowledge/work/**` keep their historical paths.
