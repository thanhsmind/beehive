---
artifact_contract: bee-plan/v2
mode: high-risk
---

# worker-outward-guard — plan

Route: class `feature` · lane `high-risk` · flags `audit-security` · product files 3.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

## Summary

A helper that works inside a feature worktree can today push to the remote, open or merge a pull request with `gh`, or start another agent from its shell, and nothing stops it. After this change the guard refuses those three things from any shell command run inside a feature worktree, in every phase, and tells the caller the sanctioned path instead: land through `bee worktree merge` from main, release through `scripts/release.sh` from main, start helpers through `bee dispatch prepare`. Work in the main checkout is untouched, byte for byte. One config key turns the refusals off.

Mode: `high-risk` — 1 risk flag: audit-security (a new refusal in the write guard).
Why this is the least workflow that protects the work: the change is one new check plus tests and two doc rows, but it edits the guard every runtime's belt calls, so the five-seat check and a byte-identical main proof are owed.

## Requirements (from CONTEXT.md)

- D1: scope is the location, not the caller — every shell call the write guard judges whose cwd resolves inside a linked feature worktree, in every phase; main-checkout calls unchanged.
- D2: refuse `git push` in every spelling the guard already resolves a git invocation from.
- D3: refuse every `gh` form except the read-only allowlist (`pr view|list|status|checks|diff`, `run list|view|watch`, `issue view|list`, `release view|list`, `repo view`, `auth status`, `api` with GET only and no `-f`/`-F`/`--input`).
- D4: refuse a nested agent launch — command word `claude`, `codex`, `pi`, `opencode`, directly or via `npx`/`bunx`/`env`; `bee` is never judged.
- D5: every refusal names the remedy: `bee worktree merge` from main, `scripts/release.sh` from main, `bee dispatch prepare`, and the opt-out key.
- D6: one config key `guards.worker_outward`, default on, `false` turns the three refusals off, read like `guards.idle_gate`.
- D7: deny-more only — main-checkout verdicts byte-identical, proved by a test.
- D8: red before green from an executing-phase linked-worktree fixture.
- D9: the codex and pi belts carry the same refusals under the existing parity gate.

## Load-bearing claims

Labels: `read` = opened at that line, `ran` = executed and output held. Match rule: the evidence column is a verbatim byte substring of the anchored line(s); multi-line evidence joins with ` / `.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The existing push refusal runs only in a terminal phase; every other phase returns before it | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:830-831` | `if !is_terminal_phase(phase) {` / `return Ok(None);` |
| 2 | Terminal means idle or compounding-complete, so swarming is not terminal | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:570` | `matches!(phase, Value::String(s) if s == "idle" \|\| s == "compounding-complete")` |
| 3 | The push arm keys on the git subcommand token | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:855` | `if subcommand == Some("push") {` |
| 4 | The guard treats Claude `Bash` and codex `exec` as the same shell request | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:205` | `let is_shell = matches!(tool_name.as_str(), "Bash" \| "exec");` |
| 5 | A valid linked worktree resolves to the literal `linked-valid` | read | `packages/bee-rs/crates/bee/src/hooks/adapter.rs:190` | `worktree_resolution: "linked-valid",` |
| 6 | The guard already branches on that resolution value | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:209` | `if write_capable && ctx.worktree_resolution == "linked-invalid" {` |
| 7 | The config opt-out pattern to copy for D6 | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:837` | `Some(g) if truthy(g) && g.get("idle_gate") == Some(&Value::Bool(false))` |
| 8 | Config is read through one helper | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/store.rs:86` | `pub(crate) fn read_config(root: &Path) -> R<Map<String, Value>> {` |
| 9 | Heredoc fencing, deep tokenizing and separator detection exist for the new check | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:389,459,313` | `pub(crate) fn tokenize_deep(command: &str) -> DeepTokens {` / `pub(crate) fn fence_heredocs(command: &str) -> String {` / `pub(crate) fn is_separator(t: &str) -> bool {` |
| 10 | The git finder already skips global flag values such as `-C <path>` | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:4357` | `fn find_git_invocations_still_skips_global_flag_values() {` |
| 11 | The shell checks are called from one site after other denials | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:838` | `if let Some(WV::Deny(reason)) = check_git_bash_command(` |
| 12 | A linked-worktree fixture at phase swarming already exists for D8 | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:1540,1566` | `fn build_linked(valid: bool) -> Linked {` / `"phase": "swarming", "mode": "high-risk", "feature": "worktree-isolation",` |
| 13 | Today's push tests cover the idle fixture only | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:2648` | `let fx = build_git_fixture("idle");` |
| 14 | No guard judges a `gh` or agent-CLI command head | ran | `rg -n '"gh"\|\bgh\b\|claude -p\|"codex"\|"claude"' packages/bee-rs/crates/bee/src/hooks/write_guard/*.rs` | `gh-1 (E1): the harness allowlist roots ride in as a parameter so tests can` |
| 15 | The pi and opencode belts call the same guard helper, so one Rust check reaches every runtime | read | `docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md:70` | `one project file translating that runtime's own events into the same helper calls every projection makes` |
| 16 | The docs home of the `guards` key | read | `docs/config-reference.md:375` | `` `guards` | `idle_gate` (`false` disables the idle intake gate) `` |
| 17 | The contracts doc names the idle gate under checkWrite | read | `docs/07-contracts.md:98` | `disable per repo via ``config.guards.idle_gate: false`` |

## Discovery

Read `checks.rs:705-900` (the git bash check: heredoc fence, deep tokens, git finder, gc-2 and staging arms, the phase-gated idle gate with its push arm), `main.rs:195-215` and `828-870` (tool classification and the shell call site), `adapter.rs:90-200` (resolution values), `tests.rs:54-76`, `1531-1573`, `2567-2660`, `4255-4270` (fixtures, idle-gate tests, compound-line tests). Ran the `rg` in claim 14: three hits, all comments. Finding: the push refusal is phase-gated by design (idle intake), so a worker mid-swarm is unguarded; the tokenizer and fixtures the new check needs already exist.

## Approach

Recommended path (D1–D9): one new phase-independent function `check_outward_bash_command` in `checks.rs`, called from `main.rs` at the shell call site before `check_git_bash_command`, only when `ctx.worktree_resolution == "linked-valid"` and `guards.worker_outward` is not `false`. It fences heredocs, deep-tokenizes, splits on separators, and judges each segment's command head after skipping `env`, `VAR=value`, `npx`, `bunx` prefixes: a git invocation whose subcommand is `push` (via `find_git_invocations`) → deny; head basename `gh` → allow only the D3 list, deny the rest; head basename in {`claude`, `codex`, `pi`, `opencode`} → deny. Every deny text carries the four D5 remedies. Tests red-first in `tests.rs` on `build_linked(true)` with cwd = work root.

Rejected alternatives:
- Extend the idle-gate push arm: it sits behind `is_terminal_phase` by design (intake gate); making it phase-independent would also refuse main-checkout pushes mid-release.
- A Claude settings deny list (as the source does): host-specific, reaches neither codex nor pi, and cannot see the worktree.
- Caller identity through child attribution: a herding pane is a top-level session with no parent; location is the fact both share.

Risk map:

| component | risk | lands in | proof needed |
|---|---|---|---|
| main-checkout verdicts drift | HIGH | wog-1 | byte-identical stderr test on `build_fixture("swarming", true)` for `git push`, `gh pr create`, `claude -p` |
| a false refusal on a read-only `gh` | MEDIUM | wog-1 | allow tests for `gh pr view`, `gh run list`, `gh api repos/o/r` |
| `bee` shell commands caught by the agent-CLI rule | MEDIUM | wog-1 | allow test for `bee dispatch prepare --runtime claude --kind cell --json` |
| codex `exec` payload shape missed | MEDIUM | wog-1 | one test with `tool_name: exec`, field `cmd` |
| docs drift on the `guards` key | LOW | wog-2 | the two rows present, release-manifest check green |

Waves: wog-1 and wog-2 run in parallel — no shared file.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"wog-1 writes the check, its call site, the fix line and the tests red-first in one cell."},
    {"stage":"documentation","classification":"required","role":"docs","reason":"wog-2 adds the config row, the contracts bullet and the verify feature file, then the regen chain."},
    {"stage":"test-authoring","classification":"not-applicable","role":"test","reason":"The code cell owns its tests red-first; no separate test cell."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"This page is the plan; the leader wrote it."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery was targeted reads by the leader."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left open."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked and was not requested."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The high-risk consult runs as the five-seat hat wave."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk lane: all five seats critique this plan before the gate."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk lane: all five seats critique this plan before the gate."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk lane: all five seats critique this plan before the gate."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk lane: all five seats critique this plan before the gate."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk lane: all five seats critique this plan before the gate."}
  ]
}
```

## Shape

Epic map (risk-shaped). Outcome: a worker inside a feature worktree cannot push, write to GitHub, or start another agent from its shell. Repo-reality basis: claims 1–15.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | outward guard in the write guard | the idle gate stops nothing mid-swarm; belts share this guard | S1: check + tests + docs (this slice) | write-guard suite green red-first; main byte-identical; release-manifest check green |

Slice queue: S1 only. Current slice to prepare: S1.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| wog-1 | Refuse push, GitHub writes and nested agent launches from a feature worktree | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs`, `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs`, `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs`, `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` | — | inside a worktree, `git push`, `gh pr create` and `claude -p` stop with a message naming `bee worktree merge`, `scripts/release.sh` and `bee dispatch prepare`; `gh pr view` and every main-checkout command behave exactly as before | write-guard test module green, red-first on the new arm |
| wog-2 | Name the worker-outward key and its refusal in the config reference, the contracts doc and the verify map | `docs/config-reference.md`, `docs/07-contracts.md`, `.bee/verify/verify-app/features/worker-outward-guard.md`, `.bee/verify/verify-app/features/README.md`, `docs/history/codex-harness-hardening/release-manifest.json` | — | the config table lists `worker_outward` beside `idle_gate`; a driver can prove the refusal from the feature map | `bee dev release-manifest --check` green |

```json
[
  {
    "id": "wog-1",
    "feature": "worker-outward-guard",
    "title": "Refuse push, GitHub writes and nested agent launches from a feature worktree",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["e4167310-5453-41e0-a089-a99ef7dc4f8c", "277edc8f-0875-4dbd-841f-593ed237486d", "58925048-fe36-4a5f-8342-09f1f6afb9cb", "54598ac5-c324-4179-bdc8-b3debda9de09"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/worker-outward-guard/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST. In packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs, beside the linked-worktree matrix (the `fn build_linked(valid: bool) -> Linked {` fixture, which writes `\"phase\": \"swarming\", \"mode\": \"high-risk\", \"feature\": \"worktree-isolation\",` with execution approved), add tests that run Bash payloads with cwd = lx.work_root and assert exit 2 with the new text: `git push origin main`; `git -C /tmp/x push`; `git status && git push`; `gh pr create --fill`; `gh pr merge 12`; `gh api -X POST repos/o/r/issues`; `claude -p \"hi\"`; `codex exec x`; `pi --mode rpc`; `opencode run`; `npx claude -p x`; `env FOO=1 claude -p x`; and one payload with `tool_name: \"exec\"` and field `cmd: \"git push\"` (codex shape). Assert exit 0 for `gh pr view 12`, `gh run list`, `gh api repos/o/r`, `gh auth status`, `bee dispatch prepare --runtime claude --kind cell --json`, `git status`, and for `git push` when .bee/config.json in the MAIN root carries `{\"guards\":{\"worker_outward\":false}}`. Assert byte-identical behavior on main: for `build_fixture(\"swarming\", true)` run `git push origin main`, `gh pr create --fill`, `claude -p x` and assert code 0 and stderr empty (per D7). Assert the `linked-invalid` denial text is unchanged (`build_linked(false)`, `git push` → the existing WORKTREE_LINK_INVALID text). Run the module and watch the new tests fail because the commands exit 0 today. THEN in checks.rs add `pub(crate) fn check_outward_bash_command(root: &str, control_root: &str, worktree_resolution: &str, command: &str) -> R<Option<WV>>` beside `pub(crate) fn check_git_bash_command(`: return Ok(None) unless worktree_resolution == \"linked-valid\"; read config through `read_config` from the control root and return Ok(None) when `guards.worker_outward == false` (copy the idle_gate read at `Some(g) if truthy(g) && g.get(\"idle_gate\") == Some(&Value::Bool(false))`); `fence_heredocs` then `tokenize_deep` (truncated → Err(Nd), as the git check does); split the tokens into segments on `is_separator`; for each segment find the command head by skipping tokens that are `env`, `npx`, `bunx`, or match `NAME=value`; (a) run `find_git_invocations` over the whole token list and deny when any invocation's subcommand is `push` (per D2); (b) head basename `gh`: allow exactly `pr view|list|status|checks|diff`, `run list|view|watch`, `issue view|list`, `release view|list`, `repo view`, `auth status`, and `api` when no `-X`/`--method` token other than GET follows and no `-f`, `-F` or `--input` token is present; deny every other gh form (per D3); (c) head basename in {claude, codex, pi, opencode}: deny (per D4); `bee` is never judged. The deny text, one per form, starts `bee worker-outward guard denied Bash: <form> is outward-facing and never runs from a feature worktree.` and ends with the fix line from a new `pub(crate) fn outward_fix_line() -> String` in paths.rs beside `intake_fix_line`, reading: `FIX: land through bee worktree merge --id <worktree-id> from the main checkout; releases and GitHub writes go through scripts/release.sh from main; to start a helper run bee dispatch prepare and use the tool it returns. Repo-level opt-out: set guards.worker_outward to false in .bee/config.json (plain JSON; delete the key to re-enable).` (per D5, D6). In main.rs, at the shell call site `if let Some(WV::Deny(reason)) = check_git_bash_command(`, add the outward check immediately BEFORE it inside the same `if denial.is_none() && is_shell` block, passing `ctx.worktree_resolution` and the control root already in scope. Nothing else in main.rs changes; the idle-gate arm in checks.rs is untouched (per D7). NO COMMENTS of any form in the code you add (no_code_comments is on). One commit, subject in imperative mood, trailer `cell: wog-1`.",
    "must_haves": {
      "truths": [
        "from a linked-valid worktree at phase swarming, git push (plain, -C path, compound), gh pr create, gh pr merge, gh api -X POST, claude -p, codex, pi, opencode, npx claude and env-prefixed claude exit 2 and the text names bee worktree merge, scripts/release.sh, bee dispatch prepare and guards.worker_outward",
        "from the same worktree gh pr view, gh run list, gh api GET, gh auth status, bee dispatch prepare and git status exit 0",
        "guards.worker_outward=false in the main root's .bee/config.json restores exit 0 for all three forms",
        "from the main checkout at phase swarming, git push, gh pr create and claude -p exit 0 with empty stderr, byte-identical to before",
        "a codex-shaped exec payload with cmd is judged exactly like Bash with command",
        "the linked-invalid denial text is unchanged",
        "the write-guard test module is green under cargo test --release"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs", "substantive": "check_outward_bash_command with the push, gh-allowlist and agent-launch arms"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs", "substantive": "outward_fix_line naming the four remedies"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "the red-first deny tests, the allow tests, the opt-out test, the main byte-identical test, the codex-shape test"}
      ],
      "key_links": [
        "main.rs shell call site calls check_outward_bash_command before check_git_bash_command",
        "docs/history/worker-outward-guard/CONTEXT.md D1-D8 are the locked rules"
      ],
      "prohibitions": [
        "No change to the idle-gate arm or any main-checkout verdict",
        "No edit outside the four listed files",
        "No code comment of any form",
        "No judgment of the bee command word"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks::write_guard"
  },
  {
    "id": "wog-2",
    "feature": "worker-outward-guard",
    "title": "Name the worker-outward key and its refusal in the config reference, the contracts doc and the verify map",
    "lane": "high-risk",
    "role": "docs",
    "deps": [],
    "decisions": ["58925048-fe36-4a5f-8342-09f1f6afb9cb"],
    "files": [
      "docs/config-reference.md",
      "docs/07-contracts.md",
      ".bee/verify/verify-app/features/worker-outward-guard.md",
      ".bee/verify/verify-app/features/README.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/worker-outward-guard/CONTEXT.md",
      "docs/config-reference.md",
      "docs/07-contracts.md",
      ".bee/verify/verify-app/features/comment-guard.md",
      ".bee/verify/verify-app/features/README.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Three edits, then the regen chain. (1) docs/config-reference.md — in the `guards` row (the line starting `| \\`guards\\` | \\`idle_gate\\` (\\`false\\` disables the idle intake gate)`), add ` · \\`worker_outward\\` (\\`false\\` disables the worker-outward guard: inside a linked feature worktree the write guard refuses \\`git push\\`, every writing \\`gh\\` form, and a nested \\`claude\\`/\\`codex\\`/\\`pi\\`/\\`opencode\\` launch, in every phase)` after the idle_gate clause, and extend the defaults cell with ` · worker-outward guard on`. (2) docs/07-contracts.md — under the checkWrite bullet that names `config.guards.idle_gate: false`, add one sibling bullet: `outward (worker-outward-guard D1-D6): when the shell request's cwd resolves to a linked-valid worktree, deny \\`git push\\`, any \\`gh\\` form outside the read-only allowlist, and a nested agent launch, in every phase; the refusal names \\`bee worktree merge\\`, \\`scripts/release.sh\\` and \\`bee dispatch prepare\\`. Default-on; disable per repo via \\`config.guards.worker_outward: false\\`. Main-checkout verdicts are unchanged.` (3) .bee/verify/verify-app/features/worker-outward-guard.md — a new feature file with the same four-section contract as comment-guard.md: what it is (the three refused forms and the allowlist), how a user reaches it (a Bash tool call from a session whose cwd is a `bee worktree new` worktree), how to drive it with control-bee (launch a sandbox, `control-bee cli -- worktree new --feature demo`, then run the hook binary with a PreToolUse payload whose cwd is the worktree and command `git push origin main`, expect exit 2 and text containing `bee worktree merge`; run `gh pr view 1`, expect exit 0; set `guards.worker_outward` false and expect exit 0), and gotchas (the guard judges command lines, not script contents; `bee` is never judged; main is untouched). Add one index line for it to .bee/verify/verify-app/features/README.md in the existing list style. Write every sentence through bee-technical-writing. Then run `.bee/bin/bee dev regen` so the rendered copies of the verify tree match, and commit the refreshed copies plus docs/history/codex-harness-hardening/release-manifest.json with the rest. Prove: `.bee/bin/bee dev release-manifest --check` green. One commit, subject in imperative mood, trailer `cell: wog-2`.",
    "must_haves": {
      "truths": [
        "docs/config-reference.md's guards row names worker_outward, its default and what false does",
        "docs/07-contracts.md has one outward bullet under checkWrite naming the three forms, the allowlist, the remedies and the opt-out",
        "the verify map has a worker-outward-guard feature file and an index line, and bee dev release-manifest --check is green"
      ],
      "artifacts": [
        {"path": ".bee/verify/verify-app/features/worker-outward-guard.md", "substantive": "the four-section feature file with a drivable refusal recipe"},
        {"path": "docs/config-reference.md", "substantive": "the worker_outward clause in the guards row"}
      ],
      "key_links": [
        "docs/history/worker-outward-guard/CONTEXT.md D5 and D6 are the words the docs must match",
        "wog-1 is the behavior the feature file drives"
      ],
      "prohibitions": [
        "No hand edit under .claude/, .agents/ or .opencode/ — those come from bee dev regen",
        "No edit to AGENTS.md or any skill",
        "No new claim about behavior wog-1 does not implement"
      ]
    },
    "verify": ".bee/bin/bee dev release-manifest --check"
  }
]
```

## Test matrix

High-risk: the twelve dimensions of `edge-dimensions.md`, applicable ones probed; each row ends with its pass condition.

| dimension | probe | pass when |
|---|---|---|
| 1 user types | orchestrator in main vs worker in worktree, same command | main exit 0 + empty stderr; worktree exit 2 |
| 2 input extremes | empty command; a 2 KB compound line ending in `&& git push` | empty → exit 0; compound → exit 2 |
| 3 timing | phase idle vs swarming in the worktree | both exit 2 (phase-independent) |
| 5 state transitions | `guards.worker_outward` flipped false then removed | false → exit 0; removed → exit 2 |
| 6 environment | codex `exec` payload with `cmd`; `env FOO=1 claude -p x` | both exit 2 |
| 7 error cascades | tokenizer truncation (wrapper nested past depth) | Err(Nd) → fail open, as the git check does |
| 8 authorization | `gh api` GET vs `-X POST`; `gh pr view` vs `gh pr merge` | GET/view exit 0; POST/merge exit 2 |
| 9 data integrity | `linked-invalid` worktree | existing WORKTREE_LINK_INVALID text, unchanged |
| 10 integration | `bee dispatch prepare …` and `bee herding run …` from the worktree | exit 0 (bee never judged) |
| 12 business logic | `git -C <main> push` from the worktree | exit 2 — location is the cwd, not the target |
| 4 scale, 11 compliance | not applicable — one request per call, no regulated data | — |
| regression | same scenario on main and head for `git push` at idle in main | exit 2 with the existing `never exempted` text on both |

## Open Questions

- Whether `ctx.worktree_resolution` reads `linked-valid` for a worktree created by `bee worktree new` in a nested-repo host — the adapter answers it from the `.git` file and the `gitdir` back-link (claim 5); a host with a bare or detached layout is judged `ordinary` and gets no refusal. Recorded, not blocking: the guard fails open to today's behavior.

## Out of scope

- Refusing raw `git push` from the main checkout in execution phases (CONTEXT Deferred Ideas).
- Reading a script's contents for pushes it contains.
- A doctor row reporting the key (agent's discretion, not taken in this slice).
