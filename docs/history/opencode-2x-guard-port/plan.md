---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: the bee guard runs on opencode 2.x

## Summary

opencode 2.x rejects the 1.x shape of `.opencode/plugins/bee-guard.ts`, so no
bee guard runs there. Port the belt to the 2.x `{id, setup}` API from one rule
core, prove it with the node harness, add a CI job on opencode 2.x, and close
the named gaps in the knowledge.

Mode: `standard` — one runtime belt, its contract tests, one CI job, one
concept.

## Requirements (from CONTEXT.md)

- D1: 2.x entry beside the 1.x entry, one rule core.
- D2: the derived 2.x tool-id registry is the map; every write- or read-capable 2.x tool is mapped or a named gap.
- D3: one CI job on opencode 2.x; the 1.x job stays.
- C1: contract opencode-2x-belt (decision 42af3b4f).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The 1.x belt default export is a function of directory and client | read | .opencode/plugins/bee-guard.ts:392 | `export default (async ({ directory, client }) =>` |
| 2 | The 1.x belt maps tool calls in one function | read | .opencode/plugins/bee-guard.ts:269 | `mapToolCall` |
| 3 | The 2.x loader accepts only default id plus effect or setup | ran | rg -a in the opencode 2.2.5 binary | `Plugin must export a default definition with an id and an effect or setup function.` |
| 4 | A 2.x execute.before hook that throws becomes a tool error | ran | rg -a in the opencode 2.2.5 binary | `"execute.before":(i)=>he({try:i,catch:Q})` |
| 5 | The 1.x proof runs the plugin under node with a stub bee | read | packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs:1006 | `every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary` |
| 6 | CI pins the 1.x opencode | read | .github/workflows/ci.yml:79 | `opencode-ai@1.18.16` |

## Approach

Situation: `skills/bee-planning/playbooks/feature/feature.md`.

oc2-1 writes the 2.x entry and the shared core, maps the 2.x ids (`shell`,
`patch`, `subagent`, `write`, `edit`, `read`, `grep`, `glob`, `question`) with
the `path` field, and adds the harness tests of contract opencode-2x-belt.
oc2-2 adds the CI job. oc2-3 is the independent tester: it loads the belt on
real opencode 1.18.16 and 2.2.5 binaries and drives the harness red then
green. oc2-4 syncs the concept and the discovery gap lines.

Rejected: a function default export carrying `id` and `setup` properties
(the 2.x schema decodes a struct, so a function is not a record).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "oc2-1 and oc2-2 write the belt and the CI job."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "oc2-3 proves the belt on real opencode binaries."},
    {"stage": "docs", "classification": "required", "role": "docs", "reason": "oc2-4 syncs the concept and discovery gap lines."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Facts came from the binary read."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "The loader leaves one shape."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Users gain a guard; nothing is removed."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "oc2-3 live-loads both runtimes."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Owner-approved backlog item."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "Gather already ran."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Every cell has a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "No prose deliverable beyond docs."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "No consult needed."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind lanes."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind lanes."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind lanes."},
    {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."}
  ]
}
```

## Risk map

| Risk | Guard |
|---|---|
| A 2.x entry in `.opencode/plugins/` breaks opencode 1.x startup | oc2-3 starts opencode 1.18.16 with both entries present and checks no plugin load error |
| The 1.x file breaks opencode 2.x startup | oc2-3 starts opencode 2.2.5 with both entries and checks the bee guard loaded |
| Two copies of the rules drift | oc2-1 keeps one rule core; the harness drives both entries over the same rows |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| oc2-1 | Port the bee guard belt to the opencode 2.x plugin API | .opencode/plugins, contracts test | — | 2.x harness rows pass | opencode_plugin_contracts |
| oc2-2 | Run the opencode contracts on opencode 2.x in CI | ci.yml | — | a 2.x CI job | static check |
| oc2-3 | Prove the 2.x belt on real opencode binaries | report | oc2-1 | both runtimes load the guard | opencode_plugin_contracts |
| oc2-4 | Record that opencode 2.x runs the bee guard | concept, discovery | oc2-1 | gap lines closed | knowledge check |

```json
[
{"id": "oc2-1", "feature": "opencode-2x-guard-port", "title": "Port the bee guard belt to the opencode 2.x plugin API", "lane": "standard", "role": "code", "deps": [], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts && .bee/bin/bee dev release-manifest --check", "must_haves": {"truths": ["a file under .opencode/plugins exports default {id, setup} whose setup subscribes api.tool.hook with execute.before", "the 2.x hook maps tool ids shell, patch, subagent, write, edit, read, grep, glob and question and reads the file field path", "the 1.x entry and the 2.x entry call one shared rule core with no copied mapping table", "a harness test drives every blocking mapped row through the 2.x entry: deny throws with the reason, allow passes, crash throws, missing binary throws", "the verify command passes"], "why": "opencode 2.x users run with no bee guard today.", "out_of_scope": ["advisory hooks (chat.message, event, execute.after, shell.env) on 2.x", "experimental 2.x tools artifact_create, artifact_update, schedule_create"]}, "decisions": ["9e9f6433-8823-42e7-9113-58ee71681dc6", "42af3b4f-becd-4aa6-98ac-391b4b310081"], "files": [".opencode/plugins/bee-guard.ts", ".opencode/plugins/bee-guard-2x.ts", "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs", "packages/bee-rs/crates/bee/src/onboard/plan.rs", "packages/bee-rs/crates/bee/src/onboard/tests.rs", "docs/history/codex-harness-hardening/release-manifest.json"], "read_first": ["docs/history/opencode-2x-guard-port/CONTEXT.md", "docs/history/opencode-support/discovery.md"], "affects_skills": [], "affects_specs": [], "action": "Implement CONTEXT.md D1 and D2 and contract opencode-2x-belt (decision 42af3b4f). Facts from the opencode 2.2.5 binary (static read 2026-11-19): the loader accepts only default {id, effect} or {id, setup}; setup(api) may return a cleanup function; a setup plugin subscribes with api.tool.hook('execute.before', fn) and fn receives {tool, sessionID, agent, messageID, id, input}; the arguments are in input and a hook may assign input; a throw inside the hook is caught and becomes the tool error; write, edit and read take path, patch keeps patchText, shell keeps command; the 1.x ids bash, task, apply_patch became shell, subagent, patch. 2.x still scans .opencode/plugins. Before writing, start opencode 1.18.16 (npx -y opencode-ai@1.18.16, run under .bee/tmp/oc11816) and the installed 2.2.5 with a probe file to settle how each runtime treats a default export of the other shape; pick one file or two files so that neither runtime fails to start and each loads exactly its own guard; record the probe result in the cap report. Keep one rule core (binary resolution, runBlockingHook, mapToolCall) that both entries use; if two files, the 2.x entry imports the core from the 1.x file or a shared module that opencode does not load as a plugin, and onboard (packages/bee-rs/crates/bee/src/onboard/plan.rs list_opencode_plugin_files) installs every file the belt needs. Add harness tests in opencode_plugin_contracts.rs per the contract, reusing run_harness and write_stub_bee. Update the registry gate so shell, patch and subagent count as mapped on 2.x. Write no code comment. Run only the verify command, never the full suite. After the change run bee dev regen so the release manifest is current, and commit it. Commit once with the cell trailer."},
{"id": "oc2-2", "feature": "opencode-2x-guard-port", "title": "Run the opencode contracts on opencode 2.x in CI", "lane": "standard", "role": "code", "deps": [], "verify": "rg -q '@opencode/cli@2.2.5' .github/workflows/ci.yml && rg -q 'opencode-ai@1.18.16' .github/workflows/ci.yml", "must_haves": {"truths": [".github/workflows/ci.yml keeps the job that installs opencode-ai@1.18.16", ".github/workflows/ci.yml has a job that installs @opencode/cli@2.2.5 and runs cargo test --test opencode_plugin_contracts", "the verify command passes"], "why": "CI pins opencode 1.18.16, so it never sees a 2.x regression.", "out_of_scope": ["moving the 1.x pin"]}, "decisions": ["9e9f6433-8823-42e7-9113-58ee71681dc6"], "files": [".github/workflows/ci.yml"], "read_first": ["docs/history/opencode-2x-guard-port/CONTEXT.md"], "affects_skills": [], "affects_specs": [], "action": "Implement CONTEXT.md D3. In .github/workflows/ci.yml add one job opencode-2x on ubuntu that sets up Rust and node the same way the existing job does, runs npm i -g @opencode/cli@2.2.5, and runs PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts. Leave the 1.18.16 install in the existing job. Write no YAML comment. Commit once with the cell trailer."},
{"id": "oc2-3", "feature": "opencode-2x-guard-port", "title": "Prove the 2.x belt on real opencode binaries", "lane": "standard", "role": "test", "deps": ["oc2-1"], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts", "must_haves": {"truths": ["opencode 2.2.5 started in a scratch project with the belt files shows no plugin load error and lists the bee guard as loaded", "opencode 1.18.16 started in a scratch project with the belt files shows no plugin load error", "with the throw removed from the 2.x hook the 2.x harness test fails, and restored it passes", "the report under docs/history/opencode-2x-guard-port/reports/oc2-3.md quotes each run's command and result line"], "why": "A code cell never certifies itself; the harness is not the real runtime.", "out_of_scope": ["editing belt source"]}, "decisions": ["42af3b4f-becd-4aa6-98ac-391b4b310081"], "files": ["docs/history/opencode-2x-guard-port/reports/oc2-3.md"], "read_first": ["docs/history/opencode-2x-guard-port/CONTEXT.md"], "affects_skills": [], "affects_specs": [], "action": "Check contract opencode-2x-belt on the oc2-1 commit without reading its cap report first. Live load: make a scratch project under .bee/tmp/oc2-live with a copy of the belt files in .opencode/plugins and a stub .bee/bin/bee; start the installed opencode 2.2.5 and opencode-ai@1.18.16 (npx -y, installed under .bee/tmp/oc11816) in it with a non-interactive command that loads plugins (find one: a debug or serve subcommand with logs), and capture the plugin load lines. Red-green: remove the throw in the 2.x hook, run the verify, expect a failing 2.x harness test, restore with a checkout of HEAD, run the verify, expect pass. Add a test if a contract truth has no test. Write docs/history/opencode-2x-guard-port/reports/oc2-3.md with each command and its result line verbatim. Return BLOCKED with evidence if a runtime fails to load the guard. Commit only the report and any added test, with the cell trailer."},
{"id": "oc2-4", "feature": "opencode-2x-guard-port", "title": "Record that opencode 2.x runs the bee guard", "lane": "standard", "role": "docs", "deps": ["oc2-1"], "verify": ".bee/bin/bee knowledge check", "must_haves": {"truths": ["docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md Open Gaps no longer says opencode 2.x runs with no bee guard", "R42 in that concept states which 2.x ids the belt maps and cites opencode-2x-guard-port D1", "docs/history/opencode-support/discovery.md oc2x-1 gap lines for shell, patch and subagent say they are mapped by opencode-2x-guard-port", "the verify command passes"], "why": "The concept still states the 2.x gap as open.", "out_of_scope": ["code changes"]}, "decisions": ["9e9f6433-8823-42e7-9113-58ee71681dc6"], "files": ["docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md", "docs/history/opencode-support/discovery.md"], "read_first": ["docs/history/opencode-2x-guard-port/CONTEXT.md"], "affects_skills": [], "affects_specs": [], "action": "Sync the concept to the shipped belt from the oc2-1 commit: rewrite the Open Gaps opencode 2.x line and R42, citing 'opencode-2x-guard-port D1' (never raw decision ids). In discovery.md oc2x-1, mark shell, patch and subagent as mapped by opencode-2x-guard-port; keep the other gap lines that still hold. Avoid the words deferred, until and later. Commit once with the cell trailer."}
]
```
