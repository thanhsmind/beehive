---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: pi-1-6-upgrade

## Summary

The Pi belt stops its result-inbox drain at session shutdown, bee doctor
names the live pi package, and the recorded Pi range moves to 1.6.0.

Mode: `small` — 0 risk flags: none (one teardown call, one string, header and docs text)
Why this is the least workflow that protects the work: one code cell with an independent tester, one docs cell.

Playbook: `skills/bee-planning/playbooks/bugfix.md`, situation `skills/bee-planning/playbooks/bugfix/bug-fix.md`.

## Requirements (from CONTEXT.md)

- D1 (`d9fa372a`): session_shutdown stops the drain timer except on reload.
- D2 (`8344cfa9`): doctor install hint names @earendil-works/pi-coding-agent.
- D3 (`045dd245`): CEILING and audit page move to 1.6.0.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | shutdown stops only the broker timer | read | .pi/extensions/bee-guard/events.ts:867 | `stopBrokerTimer()` |
| 2 | a drain stop helper already exists | read | .pi/extensions/bee-guard/result-inbox.ts:86 | `export function stopDrainTimer(): void {` |
| 3 | the hint names the retired package | read | packages/bee-rs/crates/bee/src/doctor.rs:1009 | `("pi", "pi", "npm install -g @mariozechner/pi-coding-agent"),` |
| 4 | the header ceiling reads 1.4.0 | read | .pi/extensions/bee-guard/index.ts:82 | `// CEILING: Pi 1.4.0, proven live on 2026-10-27 in this repo: a real `pi` 1.4.0` |

## Cells (current slice)

```json
[
 {
  "id": "p16u-1",
  "feature": "pi-1-6-upgrade",
  "title": "Stop the Pi result-inbox drain at session shutdown and name the live pi package in doctor",
  "lane": "small",
  "role": "code",
  "deps": [],
  "decisions": [
   "d9fa372a-2fed-4980-826f-0c8e9c5341d4",
   "8344cfa9-7982-495b-b413-2c1114c43676",
   "045dd245-308f-49b0-a8e5-198592f5e115"
  ],
  "files": [
   ".pi/extensions/bee-guard/events.ts",
   ".pi/extensions/bee-guard/index.ts",
   "packages/bee-rs/crates/bee/src/doctor.rs"
  ],
  "read_first": [
   "docs/history/pi-1-6-upgrade/CONTEXT.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Implement CONTEXT.md D1, D2 and the header half of D3. (a) In .pi/extensions/bee-guard/events.ts, the session_shutdown handler: right after the existing stopBrokerTimer() call (which already sits after the reload early return), call stopDrainTimer() imported from ./result-inbox.ts. Do not touch the reload path. (b) In packages/bee-rs/crates/bee/src/doctor.rs runtime_checks_with_env, change the pi install hint to npm install -g @earendil-works/pi-coding-agent; update any test that pins the old string (rg mariozechner). (c) In .pi/extensions/bee-guard/index.ts, the existing CEILING header block: change it to Pi 1.6.0 proven live on 2026-11-24 in this repo (pi -p ok probe printed ok with empty stderr on openai-codex gpt-6-luna and zai glm-5.3-flash; a pre-gate write was refused by the worktree-first guard). Keep the block's shape and the probe sentence. The build embeds .pi/extensions/bee-guard into the binary, so rebuild before testing. Add no new code comments beyond rewording that existing header block.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
  "must_haves": {
   "why": "A drain timer outliving its Pi session prints stale-ctx advisories on every tick, and doctor sends users to a pi below the belt floor.",
   "out_of_scope": [
    "the audit page",
    "the reload path"
   ],
   "truths": [
    "after session_shutdown with a non-reload reason no drain interval remains armed",
    "after session_shutdown with reason reload the drain interval stays armed",
    "bee doctor's pi hint names @earendil-works/pi-coding-agent"
   ],
   "prohibitions": [
    "No new code comments",
    "No change to the reload early return"
   ]
  },
  "behavior_change": true
 },
 {
  "id": "p16u-2",
  "feature": "pi-1-6-upgrade",
  "title": "Test that Pi session shutdown disarms the drain and doctor names the live package",
  "lane": "small",
  "role": "test",
  "deps": [
   "p16u-1"
  ],
  "decisions": [
   "d9fa372a-2fed-4980-826f-0c8e9c5341d4",
   "8344cfa9-7982-495b-b413-2c1114c43676"
  ],
  "files": [
   "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
   "packages/bee-rs/crates/bee/src/doctor/tests.rs"
  ],
  "read_first": [
   "docs/history/pi-1-6-upgrade/CONTEXT.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Without reading the p16u-1 worker's notes, check the p16u-1 commit against CONTEXT.md D1 and D2 and add behavior tests. In tests/pi_plugin_contracts.rs, reuse the node harness of session_shutdown_closes_record_on_real_end_reasons_and_skips_reload: start a session with a bee store and a usable session id so startResultDrain arms its interval, then fire session_shutdown with reason quit and assert no drain interval remains armed (the belt keeps it in state.drainTimer and the globalThis drain slot), and fire it with reason reload and assert the interval stays armed. Watch the quit test fail against the pre-fix events.ts before trusting it. In src/doctor/tests.rs add a test that the pi runtime check's install hint names @earendil-works/pi-coding-agent and not @mariozechner. Fix any defect in p16u-1's code and name it in the cap. No code comments.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor",
  "must_haves": {
   "why": "A code cell never certifies itself.",
   "out_of_scope": [
    "the audit page"
   ],
   "truths": [
    "a test fails if session_shutdown stops calling stopDrainTimer",
    "a test fails if reload disarms the drain",
    "a test fails if the doctor hint names the old package"
   ],
   "prohibitions": [
    "No code comments"
   ]
  },
  "behavior_change": false,
  "tester_rule": true,
  "judge_obligation_ack": "Tester for p16u-1."
 },
 {
  "id": "p16u-3",
  "feature": "pi-1-6-upgrade",
  "title": "Record the Pi 1.4.0 to 1.6.0 capability audit",
  "lane": "small",
  "role": "docs",
  "deps": [
   "p16u-1"
  ],
  "decisions": [
   "045dd245-308f-49b0-a8e5-198592f5e115"
  ],
  "files": [
   "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md"
  ],
  "read_first": [
   "docs/history/pi-1-6-upgrade/CONTEXT.md",
   "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Implement CONTEXT.md D3 on the audit page. Update the title, description and the Data Dictionary 'Last live-evidence version' row to 1.6.0 (2026-11-24, ok probe on openai-codex gpt-6-luna and zai glm-5.3-flash, guarded write refused, pi_plugin_contracts green). Add a section 'The 1.4.0 to 1.6.0 audit' after the 1.0.0 to 1.4.0 one, same table shape, covering four releases 1.5.0 (2026-11-02), 1.5.1 (2026-11-09), 1.5.2 (2026-11-10), 1.6.0 (2026-11-20), with rows: 1.5.0 BREAKING pi-schema tool argument validation (keep: belt tool parameters are plain JSON Schema, tool-verdict.ts VERDICT_TOOL_PARAMETERS and tool-steer.ts BEE_STEER_TOOL_PARAMETERS); 1.5.1 BREAKING GrepOperations.readFile and OpenAI empty tools omitted (keep, no belt surface); 1.6.0 BREAKING component render() returns readonly string[] (keep: workers-widget.ts returns its own rows array and subclasses no Container); 1.5.0 blocked option for ctx.ui.custom, Usage.cost.totalFromProvider, pi.exec maxBuffer (keep, out of scope per CONTEXT.md); 1.6.0 fix that RPC switch/fork no longer runs session_start twice (keep, belt-relevant fix); the fifteen events (keep, all in the 1.6.0 binary by rg -a -c); built-in registry (keep, 1.6.0 docs/settings.md:46 same eight names); stale ctx after session_shutdown (adapt, p16u-1: the drain now stops at shutdown). Read the CHANGELOG under /home/thanhsmind/.local/share/mise/installs/pi/1.6.0/pi/ for line numbers. Cite CONTEXT.md D-numbers, never raw decision ids. Add the 1.6.0 tree to the Pointers 'Upstream compared' line. Do not use the words deferred, until or later. Write through bee-technical-writing.",
  "verify": ".bee/bin/bee knowledge check",
  "must_haves": {
   "why": "The audit page is the authority for which Pi versions the belt was checked against.",
   "out_of_scope": [
    "code"
   ],
   "truths": [
    "the page names 1.6.0 as the last live-evidence version",
    "every 1.5.0, 1.5.1 and 1.6.0 breaking entry has a row"
   ],
   "prohibitions": [
    "No raw decision ids from this feature"
   ]
  },
  "behavior_change": false
 }
]
```

## Role assignments

```json
{
 "schema_version": "1.0",
 "runtime": "claude",
 "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
 "stages": [
  {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "The release runs after merge as its own lane."},
  {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote this plan from CONTEXT.md."},
  {"stage": "implementation", "classification": "required", "role": "code", "reason": "p16u-1 writes the belt teardown, the doctor hint and the header."},
  {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "p16u-2 tests p16u-1 independently."},
  {"stage": "documentation-and-capture", "classification": "required", "role": "docs", "reason": "p16u-3 records the 1.4.0 to 1.6.0 audit."},
  {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The leader read the shutdown and drain sites; no gather needed."},
  {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No narrow fact lookup is needed."},
  {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Every job here has a specific role."},
  {"stage": "independent-review", "classification": "not-applicable", "role": "review", "reason": "Independent review stays user-invoked and was not requested."},
  {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Small lane, no consult owed."},
  {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "No unattended loop runs here."},
  {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No competing designs; live evidence fixed the shape."},
  {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No competing designs; live evidence fixed the shape."},
  {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No competing designs; live evidence fixed the shape."},
  {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."},
  {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "One obvious fix; no hat wave."},
  {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "One obvious fix; no hat wave."},
  {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "One obvious fix; no hat wave."},
  {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "One obvious fix; no hat wave."},
  {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "One obvious fix; no hat wave."},
  {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "The audit text rides p16u-3 under the docs role."}
 ]
}
```

## Proof

- p16u-2 behavior tests, green.
- Live re-probe after merge: `pi -p "Reply with the single word ok"` prints `ok` with empty stderr.
- `bee test` (full declared suite) once before `bee worktree merge`.

## Skips

- architect skipped: one teardown call and one string.
- hat wave skipped: the evidence fixed the shape; one obvious fix.
- `bee-how` gather skipped: the leader read the shutdown and drain sites directly.
