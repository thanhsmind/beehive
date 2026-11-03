---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Claude read-only Paseo workers

## Summary

Read-only workers (gathers, advisors, hats, blind lanes) can now run on Claude
Opus 5.5 and Sonnet 5.5 through Paseo, and stay read-only for real: bee's
worker guard runs as a Claude hook inside them, and bee refuses to start one
when that guard is not in place. Before this, such a run was refused because
only pi workers had the guard.

Mode: `high-risk` — 2 risk flags: audit/security (a guard is wired into a new
host), public contracts (the Claude hook manifest every onboarded repo gets).
Why this is the least workflow that protects the work: one code cell, but it
changes the hooks every Claude session in every onboarded repo runs.

Playbook: `skills/bee-planning/playbooks/feature.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1: read-only allowed on Paseo provider `claude` (env set like pi, mode unchanged); other non-pi providers still refuse, FIX names pi and claude.
- D2: Claude-only PreToolUse group running `bee hook worker-guard` (script name `bee-worker-guard.mjs`), in both renderers; a no-op outside a herded worker.
- D3: inside a Paseo claude worker the hook applies mode A; a Claude supervisor with `BEE_SUPERVISOR_ALLOWED` gets mode B. Intended.
- D4 (store f84f697c): a read-only claude run fails closed before spawn unless the main-root binary lists `worker-guard` and the main-root `.claude/settings.json` wires it.
- D5 (store 8ea1fa19): matcher `Edit|Write|MultiEdit|NotebookEdit|Bash|Monitor|EnterWorktree|ExitWorktree|mcp__.*`, no status message; read-only judges `Monitor` like Bash, denies `EnterWorktree`, `ExitWorktree`, `mcp__*`; `cd` allowed.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | run.rs refuses every read-only Paseo provider except pi | read | packages/bee-rs/crates/bee/src/herding/run.rs:3390 | if opts.read_only && !provider_is_pi { |
| 2 | worker-guard returns at once when no marker env is set | read | packages/bee-rs/crates/bee/src/hooks/worker_guard.rs:25 | if !mode_a && !mode_b && read_only_job.is_none() { |
| 3 | The hook catalog has one shared PreToolUse write-guard group to sit beside | read | packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs:149 | Some("Edit\|Write\|MultiEdit\|Bash\|Read\|Glob\|Grep\|AskUserQuestion\|apply_patch\|exec"), |
| 4 | Onboard writes its own PreToolUse table for Claude | read | packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs:101 | { "matcher": "Agent\|Task", "hooks": [repo_entry("bee-model-guard.mjs", r)] }, |
| 5 | bee Claude hooks run inside a Paseo claude worker with the herding env | ran | jq -r .summary /mnt/data/Projects/goglbe/beehive/.bee/mailbox/spike-claude-1/result-1.json | BEE_HERDING_WORKER=1 |
| 6 | The rendered hook command allows when no binary resolves, so D4 must check before spawn | read | packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs:67 | echo \"{CODEX_BINARY_MISSING_DIAGNOSTIC}\" >&2; exit 0" |
| 7 | worker-guard ignores non-Bash tools outside read-only, so D5 needs a worker_guard.rs change | read | packages/bee-rs/crates/bee/src/hooks/worker_guard.rs:55-56 | if tool_name != "Bash" { / return Outcome::Done(ExitCode::SUCCESS); |
| 8 | Under the herding marker every hook but activity and worker-guard short-circuits, so folding into write-guard is not cheaper | read | packages/bee-rs/crates/bee/src/hooks/mod.rs:103-104 | fn marker_short_circuits(name: &str) -> bool { / name != "activity" && name != "worker-guard" |

## Discovery

One spike (`spike-claude-1`, Sonnet 5.5 on Paseo) printed `PASEO_AGENT_ID`,
`BEE_HERDING_WORKER=1`, `BEE_HERDING_JOB_ID=spike-claude-1`, and the activity
hook wrote `activity.json` from inside the worker. Five hat seats checked the
first draft; their findings are folded in (`hat-wave-synthesis.md`).

## Approach

Wire, do not rebuild: add the Claude-only group to both hook tables, render
the manifests and refresh this repo's `.claude/settings.json` through onboard,
relax the provider gate to pi or claude behind the D4 pre-spawn check, and
extend read-only judging to the extra tools (D5). SMALLER PATH check: no
cheaper shape — Claude plan mode blocks the result file (paseo-leverage hat
wave); a prompt-only rule enforces nothing; folding into write-guard is not
cheaper because the herding marker short-circuits write-guard (claim 8) and
write-guard is a shared Claude+Codex group, so the guard would leak into Codex.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Guard fails open on a stale or missing binary | HIGH | cro-1 | unit rows for the D4 refusal (binary without worker-guard; settings without the entry) |
| Write paths outside the matcher | HIGH | cro-1 | guard rows for Monitor, EnterWorktree, mcp__ tools in read-only |
| Hook runs in every Claude session | MEDIUM | cro-1 | manifest tests; no-env run exits success |
| Write-capable claude workers now get mode A | MEDIUM | cro-1 | a unit row: mode A denies git push for a claude Paseo worker payload |

Waves: one cell. Leader live proof after merge: a read-only Sonnet worker on
Paseo is blocked on a repo write and on `git push`, runs `cd X && rg`, and
lands `result-1.json`.

Known limit (accepted, recorded): a repo onboarded with the new manifest but
still carrying a pre-`worker-guard` binary sees a non-blocking hook error on
Edit, Write and Bash until it refreshes `.bee/bin/bee`; `bee onboard --apply`
installs both together.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"cro-1 changes the hook tables, the guard and the run gate."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"The code cell writes its own tests; the leader drives the live proof."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader updates the concept line at capture."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk plan check."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk plan check."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk plan check."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk plan check."}
  ]
}
```

## Shape

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Guard | read-only Claude workers on Paseo | the codex hat, advisor and lane roles move to Claude | cro-1 | hook and guard tests + live read-only run |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| cro-1 | Enforce read-only on Paseo Claude workers through a Claude hook | devtools/hook_manifests.rs, onboard/hooks_wiring.rs, hooks/worker_guard.rs, herding/run.rs, packages/bee/hooks/claude-hooks.json, .claude/settings.json | — | a read-only Opus or Sonnet worker on Paseo runs, hands in its result, and cannot write repo files; bee refuses to start one where the guard is missing | cargo test hook_manifests; hooks_wiring; worker_guard; execute_paseo_read_only |

Paths under `devtools/`, `onboard/`, `hooks/` and `herding/` are in `packages/bee-rs/crates/bee/src/`. The cell packet:

```json
[
  {
    "id": "cro-1",
    "feature": "claude-read-only",
    "title": "Enforce read-only on Paseo Claude workers through a Claude hook",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "D4", "D5", "906e0b1f", "63bd6725", "f84f697c", "8ea1fa19-fc2a-49c0-8e84-0c1ee1b3bc0f"],
    "files": ["packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs", "packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs", "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs", "packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee/hooks/claude-hooks.json", ".claude/settings.json"],
    "read_first": ["docs/history/claude-read-only/CONTEXT.md", "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs", "packages/bee-rs/crates/bee/src/hooks/mod.rs", "packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs", "packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md", "docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md"],
    "action": "Per D1-D5 (store 906e0b1f, 63bd6725, f84f697c, 8ea1fa19). (1) devtools/hook_manifests.rs: in the PreToolUse event add a CLAUDE_ONLY group, matcher `Edit|Write|MultiEdit|NotebookEdit|Bash|Monitor|EnterWorktree|ExitWorktree|mcp__.*`, script `bee-worker-guard.mjs` (the name-to-verb mapping strips `bee-` and `.mjs`, giving `hook worker-guard`), with no status message, beside the shared write-guard group at `Some(\"Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion|apply_patch|exec\")`. (2) onboard/hooks_wiring.rs: add the same entry with the same matcher to its Claude PreToolUse table beside `{ \"matcher\": \"Agent|Task\", \"hooks\": [repo_entry(\"bee-model-guard.mjs\", r)] }`. (3) Run `bee dev render-hook-manifests` (claude-hooks.json gains the group; hooks.json for Codex must not change) and `bee onboard --repo-root . --apply` so this repo's .claude/settings.json carries the entry; commit only the files listed. (4) hooks/worker_guard.rs, read-only path only: judge `Monitor` by its `command` input exactly as Bash; deny `EnterWorktree`, `ExitWorktree` and any tool name starting `mcp__` by name with the read-only denial text; add `cd` to READ_ONLY_ALLOWED. Mode A and mode B results for other inputs do not change. (5) herding/run.rs `execute_paseo`: replace the pi-only gate `if opts.read_only && !provider_is_pi {` with pi-or-claude; other providers refuse with a FIX line naming pi and claude. For claude + read_only, before spawn: run `<main root>/.bee/bin/bee hook --help` and require `worker-guard` in its output, and read `<main root>/.claude/settings.json` and require a PreToolUse hook command containing `hook worker-guard`; on either miss refuse with a FIX line (`bee onboard --apply` / refresh .bee/bin/bee). Keep `--mode` as configured. Tests: manifest test that the Claude projection has the group (matcher, script, no status message) and Codex has none; onboard table test for the entry; guard rows for Monitor (allowed rg, denied redirect), EnterWorktree, an mcp__ write tool, `cd x && rg y` allowed, and mode A denying `git push` for a claude Paseo worker payload; run.rs rows: claude read-only spawns with BEE_HERDING_READ_ONLY=1 and the configured mode when both checks pass, refuses when the binary help lacks worker-guard, refuses when settings lack the entry, and codex still refuses naming pi and claude.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee hook_manifests && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee hooks_wiring && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee worker_guard && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee execute_paseo_read_only",
    "must_haves": {
      "truths": ["The Claude hook projection has a PreToolUse group with the D5 matcher running hook worker-guard and no status message", "The Codex projection has no worker-guard group", "Onboard writes the same Claude entry and this repo's .claude/settings.json carries it", "A read-only claude Paseo spawn sets BEE_HERDING_READ_ONLY=1 and keeps the configured mode when the binary and settings checks pass", "A read-only claude spawn refuses before spawn when the binary help lacks worker-guard or settings lack the entry", "A read-only spawn on a provider other than pi or claude refuses naming both", "In read-only mode Monitor is judged like Bash, EnterWorktree, ExitWorktree and mcp__ tools are denied, and cd is allowed", "Mode A denies git push for a claude Paseo worker"],
      "artifacts": [{"path": "packages/bee/hooks/claude-hooks.json", "substantive": "rendered manifest with the worker-guard group"}],
      "key_links": ["hook_manifests.rs and hooks_wiring.rs carry the same Claude entry", "run.rs pre-spawn check reads the same settings entry the renderers write"],
      "prohibitions": ["No worker-guard group in the Codex manifest", "No change to mode A or mode B results outside the listed tools", "No status message on the worker-guard group"]
    }
  }
]
```

## Test matrix

| Dimension | Probe | Pass when |
|---|---|---|
| Wiring | render Claude projection | group with D5 matcher, worker-guard, no status message |
| Host isolation | render Codex projection | no worker-guard group |
| Fail closed | read-only claude with a binary lacking worker-guard; with settings lacking the entry | refused before spawn with a FIX line |
| Provider gate | read-only on claude / codex | claude spawns with env and configured mode; codex refuses naming pi and claude |
| Extra tools | read-only Monitor `echo x > f`; EnterWorktree; mcp__ write tool | each denied |
| Allowed forms | `cd x && rg y` under read-only | allowed |
| Write workers | mode A `git push` from a claude Paseo worker payload | denied |
| No-op outside workers | worker-guard with no marker env | exit success, no output |
| Live (leader, after merge) | read-only Sonnet worker: repo write, git push, cd && rg, result file | write and push blocked; cd && rg runs; result-1.json lands |

## Open Questions

(none)

## Out of scope

- Usage and cost for claude workers.
- The team role mapping edit (user-owned).
- Guarding Codex hosts with worker-guard.
