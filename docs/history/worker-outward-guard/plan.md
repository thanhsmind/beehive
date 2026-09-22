---
artifact_contract: bee-plan/v2
mode: high-risk
---

# worker-outward-guard — plan

Route: class `feature` · lane `high-risk` · flags `audit-security` · product files 4.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

## Summary

A helper working inside a feature worktree can today push to the remote, open or merge a pull request with `gh`, or start another agent from its shell, and nothing stops it. A push from here publishes: CI runs on every push, the public site rebuilds on a push to main, and release assets are built from a pushed tag. After this change the guard refuses those three things from any shell command whose working directory is a linked git worktree, in every phase, and tells the caller the sanctioned path for its own case: a cell worker stops and reports blocked; a human lands through `bee worktree merge` from main; releases go through `scripts/release.sh` from main; helpers start through `bee dispatch prepare`, and the command that door returns is never refused. `bee` itself is never judged, so `bee herding run` still opens a pane agent. Work in the main checkout is untouched, byte for byte. One config key in the main checkout turns the refusals off.

Mode: `high-risk` — 1 risk flag: audit-security (a new refusal in the write guard).
Why this is the least workflow that protects the work: the change is one arm in an existing check plus tests and doc rows, but it edits the guard every runtime's belt calls and ships default-on to every host, so the five-seat check and a byte-identical main proof are owed.

## Requirements (from CONTEXT.md)

- D1: scope is the location, not the caller — every shell call the write guard judges whose cwd resolves to a linked-valid worktree, in every phase; main-checkout calls unchanged.
- D2: refuse `git push` in every spelling the guard already resolves a git invocation from.
- D3: refuse every `gh` form except the read-only allowlist.
- D4: refuse a nested agent launch — command word `claude`, `codex`, `pi`, `opencode`, directly or via wrappers; `bee` is never judged.
- D5: every refusal names the remedy: `bee worktree merge` from main, `scripts/release.sh` from main, `bee dispatch prepare`, and the opt-out key.
- D6: one config key `guards.worker_outward`, default on, `false` turns the three refusals off, read like `guards.idle_gate` from the control root.
- D7: deny-more only — main-checkout verdicts byte-identical, proved by a test.
- D8: red before green from an executing-phase linked-worktree fixture.
- D9: the codex, pi and opencode belts carry the same refusals — satisfied by construction: each belt hands its shell tool to this one Rust check as `tool_name: "Bash"` or `exec` (claims 18–19); no belt edit; the codex-shape test proves it.
- D10 (decision 293cdfe5): (a) a launch is exempt when the command starts with a cli command configured under `models.<runtime>.<name>.command` in the main checkout's config, or is `codex exec` carrying `--sandbox read-only` / `-s read-only`; (b) the gh allowlist is `pr view|list|status|checks|diff`, `run list|view|watch|download`, `issue view|list`, `release view|list|download`, `repo view`, `workflow list|view`, `search <any>`, `cache list`, `label list`, `status`, `auth status`, `api` with GET explicit (glued or `=`-joined) even with `-f/-F/--field/--raw-field`, `api graphql` when no token is `mutation`; `--input` always refused; a glued `-XPOST`/`--method=POST` is a method; leading gh global flags (`-R/--repo <v>`, `--hostname <v>`) are skipped; (c) the command head is found after skipping `env`, `NAME=value`, `npx`, `bunx`, `sudo`, `nohup`, `command`, `exec`, `timeout <n>`, any token starting with `-`, and `(`, `{`, `!`; (d) the scope is any linked-valid worktree, granted or not, and the docs say so.
- D11 (decision 1b8955e5): on tokenizer truncation, scan the flat fenced tokens and refuse on a push git invocation or a `gh`/agent basename, else record the gap and allow; the opt-out is read from the MAIN checkout's `.bee/config.json` and every refusal says the worktree's copy is not read; each refusal names the current worktree id and leads with the remedy for its own form; at idle inside a worktree the outward text takes precedence.

## Load-bearing claims

Labels: `read` = opened at that line, `ran` = executed and output held. Match rule: the evidence column is a verbatim byte substring of the anchored line(s); multi-line evidence joins with ` / `.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The existing push refusal runs only in a terminal phase; every other phase returns before it | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:830-831` | `if !is_terminal_phase(phase) {` / `return Ok(None);` |
| 2 | Terminal means idle or compounding-complete, so swarming is not terminal | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:570` | `matches!(phase, Value::String(s) if s == "idle" \|\| s == "compounding-complete")` |
| 3 | The push arm keys on the git subcommand token | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:856` | `if subcommand == Some("push") {` |
| 4 | The guard treats Claude `Bash` and codex `exec` as the same shell request | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:205` | `let is_shell = matches!(tool_name.as_str(), "Bash" \| "exec");` |
| 5 | A valid linked worktree resolves to the literal `linked-valid` | read | `packages/bee-rs/crates/bee/src/hooks/adapter.rs:190` | `worktree_resolution: "linked-valid",` |
| 6 | The guard already branches on that resolution value | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:209` | `if write_capable && ctx.worktree_resolution == "linked-invalid" {` |
| 7 | The config opt-out pattern to copy for D6 | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:837` | `Some(g) if truthy(g) && g.get("idle_gate") == Some(&Value::Bool(false))` |
| 8 | Config is read through one helper | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/store.rs:86` | `pub(crate) fn read_config(root: &Path) -> R<Map<String, Value>> {` |
| 9 | The git check already fences, deep-tokenizes and finds git invocations once per shell call | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:714-715,721-722` | `let fenced = fence_heredocs(command);` / `let deep = tokenize_deep(&fenced);` / `let invocations = find_git_invocations(&deep.tokens);` / `if invocations.is_empty() {` |
| 10 | The git finder already skips global flag values such as `-C <path>` | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:4357` | `fn find_git_invocations_still_skips_global_flag_values() {` |
| 11 | The shell checks are called from one site after other denials | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:838` | `if let Some(WV::Deny(reason)) = check_git_bash_command(` |
| 12 | A linked-worktree fixture at phase swarming already exists for D8 | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:1540,1565` | `fn build_linked(valid: bool) -> Linked {` / `"phase": "swarming", "mode": "high-risk", "feature": "worktree-isolation",` |
| 13 | Today's push tests cover the idle fixture only | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:2648` | `let fx = build_git_fixture("idle");` |
| 14 | No guard judges a `gh` or agent-CLI command head (six hits, none a command-head check; the first is this comment) | ran | `rg -n '"gh"\|\bgh\b\|claude -p\|"codex"\|"claude"' packages/bee-rs/crates/bee/src/hooks/write_guard/*.rs` | `gh-1 (E1): the harness allowlist roots ride in as a parameter so tests can` |
| 15 | A safe-form table already exists in the shape the gh allowlist copies | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:968` | `fn idle_gate_safe_form(sub: &str, rest: &[String]) -> bool {` |
| 16 | The docs home of the `guards` key | read | `docs/config-reference.md:375` | `` `guards` | `idle_gate` (`false` disables the idle intake gate) `` |
| 17 | The contracts doc names the idle gate under checkWrite | read | `docs/07-contracts.md:98` | `disable per repo via ``config.guards.idle_gate: false`` |
| 18 | The pi belt hands its shell tool to this guard as Bash | read | `.pi/extensions/bee-guard.ts:414` | `tool_name: "Bash",` |
| 19 | The opencode belt does the same | read | `.opencode/plugins/bee-guard.ts:282` | `return { hook: "write-guard", tool_name: "Bash", tool_input: { command: args?.command } }` |
| 20 | The control root is main for a linked worktree, so the opt-out lives in main's config | read | `packages/bee-rs/crates/bee/src/hooks/adapter.rs:344` | `let control_root = roots.main_root.clone().or_else(\|\| root.clone());` |
| 21 | A delegated verdict is exit 0 with the whole call unguarded (why D11 does not delegate) | read | `packages/bee-rs/crates/bee/src/hooks/mod.rs:68-69` | `"bee: hook {name} could not decide this payload — allowing the operation (fail-open).` |
| 22 | The dispatch door returns tool Bash with the configured cli command verbatim (why D10a exists) | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2462-2463` | `tool = "Bash".into();` / `payload.insert("command".into(), Value::String(command.clone()));` |
| 23 | The current worktree id is derivable inside the guard for the refusal text | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:541,567` | `let current = derive_current_worktree(root)?;` / `` back from main via `bee worktree merge --id {id}`." `` |
| 24 | Two more docs enumerate the guards keys | read | `docs/handbook/register.md:119`, `docs/product-description/foundations/guards.md:98` | `` | `guards` | write-guard tuning: `idle_gate`, `max_read_lines` | `` / `` `guards.idle_gate`, `guards.auto_isolate` `` |

## Discovery

Read `checks.rs:705-900`, `main.rs:195-215` and `828-870`, `adapter.rs:90-200,344`, `tests.rs:54-76`, `1531-1573`, `2567-2660`, `4255-4270`, `hooks/mod.rs:62-71`, `prepare.rs:2455-2470`, the two belt lines, and the four docs anchors. Ran the `rg` in claim 14 (six hits, none a head check). Findings: the push refusal is phase-gated by design; the tokenizer, the git finder, the safe-form table shape, the fixtures and the worktree-id helper all exist; the dispatch door returns a `codex exec` command for cli slots, so an unamended D4 would have refused bee's own sanctioned dispatch (fixed by D10a). The five-seat wave ran on the first draft; its synthesis is recorded through `bee state advisor-ref record`.

## Approach

Recommended path (D1–D11): one new arm inside `check_git_bash_command` in `checks.rs`, inserted after `find_git_invocations` and BEFORE the `invocations.is_empty()` early return, so the existing fence + deep-tokenize pass serves both checks. The arm returns at once unless `worktree_resolution == "linked-valid"` (passed in from `main.rs` as one new argument). It splits `deep.tokens` on `is_separator`, finds each segment's head per D10(c), and classifies: a git invocation with subcommand `push` → push form; head basename `gh` → allowed only by a `gh_read_only_form(sub, rest) -> bool` table written in the shape of `idle_gate_safe_form`, after skipping global flags (D10b); head basename in {claude, codex, pi, opencode} → launch form unless exempt per D10(a). Only when a form is about to deny does it read `guards.worker_outward` through `read_config(control_root)` (a bad config never turns every shell call into a delegate). On `deep.truncated` it applies D11's flat scan instead of `Err(Nd)`. Refusals come from `outward_fix_line(form, worktree_id)` in `paths.rs`: the form's own remedy first, the shared "Last resort, repo-level opt-out … in the MAIN checkout's .bee/config.json … the copy inside this worktree is not read" clause last; the gh text lists the allowed reads from the table's one home. Tests red-first on `build_linked(true)` with cwd = work root.

Rejected alternatives:
- A second entry point with its own fence/tokenize pass: doubles the walk and the `main.rs` diff for nothing (alternatives seat).
- Extending the idle-gate push arm: it sits behind `is_terminal_phase` by design and would refuse main-checkout pushes mid-release.
- A Claude settings deny list (as the source does): host-specific, reaches neither codex nor pi, cannot see the worktree.
- A dispatch-ledger match for D10a: `.bee/logs/dispatch.jsonl` carries no command bytes.
- Caller identity through child attribution: a herding pane is a top-level session with no parent.

Risk map:

| component | risk | lands in | proof needed |
|---|---|---|---|
| main-checkout verdicts drift | HIGH | wog-1 | code 0 + empty stderr on `build_fixture("swarming", true)` for `git push`, `gh pr create`, `claude -p`; idle-main `git push` keeps the `never exempted` text |
| the dispatch door's own return refused | HIGH | wog-1 | allow tests for a configured cli command prefix and for `codex exec --sandbox read-only --ephemeral --cd /x -` |
| a false refusal on a read-only `gh` | MEDIUM | wog-1 | allow tests for `gh -R o/r pr list`, `gh api -X GET search/issues -f q=x`, `gh api graphql -f query=...`, `gh workflow view x` |
| a bypass by wrapper/flag spelling | MEDIUM | wog-1 | deny tests for `npx -y claude`, `sudo claude`, `gh api --method=POST`, `gh api -XPOST`, `--raw-field`, nested `sh -c` truncation |
| `bee` or a script caught | LOW | wog-1 | allow tests for `bee dispatch prepare …`, `bee herding run …`, `bash scripts/release.sh 1.0.0` |
| docs drift on the `guards` key | LOW | wog-2 | the four docs carry `worker_outward`; release-manifest check green |
| wog-2's recipe drafted before wog-1 lands | LOW | wog-2 | the recipe asserts only the D5 substring `bee worktree merge`; accepted, parallel |

Waves: wog-1 and wog-2 run in parallel — no shared file.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"wog-1 writes the arm, the gh table, the fix line and the tests red-first in one cell."},
    {"stage":"documentation","classification":"required","role":"docs","reason":"wog-2 adds the config section, the contracts bullet, two enumerations and the verify feature file, then the regen chain."},
    {"stage":"test-authoring","classification":"not-applicable","role":"test","reason":"The code cell owns its tests red-first; no separate test cell."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"This page is the plan; the leader wrote it."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery was targeted reads by the leader."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left open."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked and was not requested."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The high-risk consult ran as the five-seat hat wave."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk lane: all five seats critiqued this plan before the gate."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk lane: all five seats critiqued this plan before the gate."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk lane: all five seats critiqued this plan before the gate."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk lane: all five seats critiqued this plan before the gate."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk lane: all five seats critiqued this plan before the gate."}
  ]
}
```

## Shape

Epic map (risk-shaped). Outcome: a worker inside a linked worktree cannot push, write to GitHub, or start an unsanctioned agent from its shell. Repo-reality basis: claims 1–24.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | outward guard in the write guard | the idle gate stops nothing mid-swarm; every belt shares this guard; a push publishes | S1: arm + tests + docs (this slice) | write-guard module green red-first; main byte-identical; release-manifest check green |

Slice queue: S1 only. Current slice to prepare: S1.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| wog-1 | Refuse push, GitHub writes and unsanctioned agent launches from a linked worktree | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs`, `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs`, `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs`, `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` | — | inside a worktree, `git push`, `gh pr create` and `claude -p` stop with a message that leads with the remedy for that form and names the worktree id; `gh pr view`, the dispatch door's own `codex exec` line and every main-checkout command behave exactly as before | write-guard test module green, red-first on the new arm |
| wog-2 | Name the worker-outward key, its allowlist and its refusal in the docs and the verify map | `docs/config-reference.md`, `docs/07-contracts.md`, `docs/handbook/register.md`, `docs/product-description/foundations/guards.md`, `.bee/verify/verify-app/features/worker-outward-guard.md`, `.bee/verify/verify-app/features/README.md`, `docs/history/codex-harness-hardening/release-manifest.json` | — | the config reference has a `guards.worker_outward` section listing the allowed reads and the main-checkout opt-out; a driver can prove the refusal from the feature map | `bee dev release-manifest --check` green and the four docs name `worker_outward` |

```json
[
  {
    "id": "wog-1",
    "feature": "worker-outward-guard",
    "title": "Refuse push, GitHub writes and unsanctioned agent launches from a linked worktree",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["e4167310-5453-41e0-a089-a99ef7dc4f8c", "277edc8f-0875-4dbd-841f-593ed237486d", "58925048-fe36-4a5f-8342-09f1f6afb9cb", "293cdfe5-33ef-41c0-a164-41f13bd0d12a", "1b8955e5-d9e3-4865-ac10-ca10b92bf3c7", "54598ac5-c324-4179-bdc8-b3debda9de09"],
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
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Steps in order; per D1-D11 in docs/history/worker-outward-guard/CONTEXT.md.\n\n1. RED FIRST — tests. In packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs, beside the linked-worktree matrix (the fixture `fn build_linked(valid: bool) -> Linked {`, which writes `\"phase\": \"swarming\", \"mode\": \"high-risk\", \"feature\": \"worktree-isolation\",` with execution approved), add a test group `worker_outward`. Helper: run a Bash payload `{\"tool_name\":\"Bash\",\"tool_input\":{\"command\":<cmd>}}` with cwd = lx.work_root (see `run_payload`). DENY (exit 2, stderr contains `bee worker-outward guard denied`): `git push origin main`; `git push --dry-run`; `git -C /tmp/x push`; `git status && git push`; `sh -c \"git push\"`; `echo git push` (the git finder scans every token — pin it); `gh pr create --fill`; `gh pr merge 12`; `gh pr checkout 12`; `gh api -X POST repos/o/r/issues`; `gh api -XPOST repos/o/r/issues`; `gh api --method=POST repos/o/r/issues`; `gh api repos/o/r/issues -f title=x` (no explicit GET); `gh api graphql -f query='mutation { x }'`; `gh api --input body.json repos/o/r`; `gh auth token`; `gh v` (an alias token); `claude -p \"hi\"`; `codex exec x`; `pi --mode rpc`; `opencode run`; `npx claude -p x`; `npx -y claude -p x`; `env FOO=1 claude -p x`; `sudo claude`; `timeout 60 claude -p x`; `( gh pr create )`; and one payload `{\"tool_name\":\"exec\",\"tool_input\":{\"cmd\":\"git push\"}}` (codex shape). ALLOW (exit 0): `gh pr view 12`; `gh -R o/r pr list`; `gh pr list --search \"x\"`; `gh run list`; `gh run download 5`; `gh workflow view ci.yml`; `gh search prs x`; `gh api repos/o/r`; `gh api -X GET search/issues -f q=x`; `gh api --method GET search/issues -F q=x`; `gh api graphql -f query='query { x }'`; `gh auth status`; `bee dispatch prepare --runtime claude --kind cell --json`; `bee herding run --role x`; `bash scripts/release.sh 1.0.0`; `git status`; `pip install x`; `python pi.py`; `echo gh pr create`; `cat <<EOF\\ngit push\\nEOF` (heredoc body is fenced — pin it); `codex exec --sandbox read-only --ephemeral --cd /x -`; `codex exec -s read-only -`; and, with the MAIN root's .bee/config.json carrying `{\"models\":{\"claude\":{\"cli-x\":{\"kind\":\"cli\",\"command\":\"opencode run --quiet\"}}}}`, `opencode run --quiet extra args` (D10a prefix). OPT-OUT: with the MAIN root's .bee/config.json carrying `{\"guards\":{\"worker_outward\":false}}`, `git push origin main`, `gh pr create --fill` and `claude -p x` exit 0; with the same key written only into the WORKTREE's .bee/config.json they still exit 2 (D11). TRUNCATION (D11): a command nesting `sh -c` past the tokenizer depth bound that ends in `git push` exits 2; the same nest ending in `ls` exits 0 with the existing gap line. MAIN BYTE-IDENTICAL (D7): on `build_fixture(\"swarming\", true)` with cwd = fx.root, `git push origin main`, `gh pr create --fill`, `claude -p x` exit 0 with empty stderr; on `build_git_fixture(\"idle\")`, `git push origin main` still exits 2 with `never exempted`. PRECEDENCE: on build_linked(true) with phase rewritten to idle in the main store, `git push` exits 2 with the outward text. LINKED-INVALID: `build_linked(false)`, `git push` → the existing WORKTREE_LINK_INVALID text, unchanged. Run `cargo test --release … hooks::write_guard` and watch the new deny tests fail (exit 0 today) before writing any guard code.\n\n2. main.rs — at the shell call site `if let Some(WV::Deny(reason)) = check_git_bash_command(`, pass one new argument `ctx.worktree_resolution` (a `&str`); nothing else in main.rs changes.\n\n3. checks.rs — extend `pub(crate) fn check_git_bash_command(` with the `worktree_resolution: &str` parameter and, directly after `let invocations = find_git_invocations(&deep.tokens);` and BEFORE `if invocations.is_empty() {`, insert `if let Some(v) = outward_arm(root, control_root_override, worktree_resolution, &fenced, &deep, &invocations, emit)? { return Ok(Some(v)); }`. Write `fn outward_arm(...) -> R<Option<WV>>` in checks.rs: return Ok(None) unless worktree_resolution == \"linked-valid\". Resolve the control root as the git check does (control_root_override, else the main root). If deep.truncated: D11 — over deep.tokens, deny when any git invocation's subcommand is push, or any token's basename (after the D10c skip set) is gh, claude, codex, pi or opencode; otherwise push_gap and return Ok(None). Else split deep.tokens on `is_separator` into segments; for each segment find the head by skipping tokens equal to env, npx, bunx, sudo, nohup, command, exec, `(`, `{`, `!`, any token matching `^[A-Za-z_][A-Za-z0-9_]*=`, any token starting with `-`, and `timeout` plus its following numeric token (D10c). Classify, in order: (a) push — any invocation in `invocations` with subcommand == Some(\"push\") (per D2); (b) gh — head basename `gh`: skip leading global flags (`-R`/`--repo` and its value, `--hostname` and its value, any other `-`-token) then take (sub, rest) and allow when `gh_read_only_form(sub, rest)` is true, else deny (per D3, D10b); (c) launch — head basename in {claude, codex, pi, opencode}: allow when the segment joined with single spaces starts with any `models.<rt>.<name>.command` string found in the control root's config (kind cli), or when head is codex, rest[0] is exec and rest contains `--sandbox` followed by `read-only`, or `-s` followed by `read-only` (per D10a); else deny (per D4). Only when a form is about to deny, read the config through `read_config(control_root)` and return Ok(None) when `guards.worker_outward == Some(&Value::Bool(false))` (copy the idle_gate read: `Some(g) if truthy(g) && g.get(\"idle_gate\") == Some(&Value::Bool(false))`). Write `fn gh_read_only_form(sub: &str, rest: &[String]) -> bool` beside `fn idle_gate_safe_form(sub: &str, rest: &[String]) -> bool {`, one arm per verb: pr → view|list|status|checks|diff; run → list|view|watch|download; issue → view|list; release → view|list|download; repo → view; workflow → list|view; search → any; cache → list; label → list; status → true; auth → status; api → false when any token is `--input`; when rest[0] == \"graphql\": true unless any token equals `mutation` or a token's value contains the word `mutation`; otherwise true only when a method token is present and reads GET case-insensitively (`-X GET`, `-XGET`, `--method GET`, `--method=GET`) or no `-f`/`-F`/`--field`/`--raw-field` token is present and no method token is present (a bare GET); false when a method other than GET is present. Deny text per form, built by `outward_fix_line(form, worktree_id)` (step 4): push → `bee worker-outward guard denied <tool>: \\`git push\\` is outward-facing and is never exempted inside a linked worktree, regardless of what it would push.`; gh → `bee worker-outward guard denied <tool>: \\`gh <sub> <verb>\\` writes to GitHub or moves the checkout, and a linked worktree never does either. Reads still run here: <the allowlist rendered from one const>.`; launch → `bee worker-outward guard denied <tool>: \\`<head>\\` starts another agent, and a worker runs exactly the one cell it was handed — starting helpers is the leader's.` where <tool> is the payload's tool name (Bash or exec).\n\n4. paths.rs — beside `pub(crate) fn intake_fix_line() -> String {` add `pub(crate) fn outward_fix_line(form: OutwardForm, worktree_id: Option<&str>) -> String` (define `pub(crate) enum OutwardForm { Push, Gh, Launch }` beside it). Push: `FIX: if you are executing a cell, stop here and report the push as blocked — your leader lands the work. To land it yourself, run \\`bee worktree merge --id <id>\\` from the MAIN checkout; a release goes through \\`scripts/release.sh <version>\\` from main.`. Gh: `FIX: open the pull request or make the GitHub write from the MAIN checkout, or land the branch with \\`bee worktree merge --id <id>\\` from main; releases and their GitHub writes go through \\`scripts/release.sh\\` from main.`. Launch: `FIX: run \\`bee dispatch prepare --runtime <rt> --kind cell|gather|reviewer|advisor --json\\` and then run exactly the tool and payload it returns; \\`bee\\` itself is never judged by this guard.`. Every form ends with the shared clause: ` Last resort, repo-level opt-out: set guards.worker_outward to false in the MAIN checkout's .bee/config.json (plain JSON; delete the key to re-enable) — the copy inside this worktree is not read.`. `<id>` is the real id from `derive_current_worktree(root)` (hook_local.rs) when it resolves, else the literal `<worktree-id>`.\n\n5. Run `cargo test --release … hooks::write_guard`; every test in step 1 green, every pre-existing test green. NO COMMENTS of any form in the code you add (no_code_comments is on; the ratchet reds the suite if the count rises). One commit, subject in imperative mood, trailer `cell: wog-1`.",
    "must_haves": {
      "truths": [
        "from a linked-valid worktree at phase swarming, every DENY command in step 1 exits 2 with a text that names the form's own remedy, the worktree id, guards.worker_outward and the MAIN checkout's .bee/config.json",
        "from the same worktree every ALLOW command in step 1 exits 0, including the dispatch door's codex exec read-only line and a configured cli command prefix",
        "guards.worker_outward=false in the MAIN root's .bee/config.json restores exit 0 for all three forms; the same key in the worktree's copy changes nothing",
        "on tokenizer truncation a push or a gh/agent basename exits 2 and a harmless nest exits 0 with the gap line",
        "from the main checkout at phase swarming, git push, gh pr create and claude -p exit 0 with empty stderr, and the idle-main git push refusal text is unchanged",
        "a codex-shaped exec payload with cmd is judged exactly like Bash with command",
        "the linked-invalid denial text is unchanged",
        "the write-guard test module is green under cargo test --release"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs", "substantive": "outward_arm inside check_git_bash_command, gh_read_only_form beside idle_gate_safe_form"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs", "substantive": "OutwardForm and outward_fix_line with the per-form remedy and the shared main-checkout opt-out clause"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "the worker_outward test group: deny, allow, opt-out, truncation, main byte-identical, precedence, codex-shape, linked-invalid"}
      ],
      "key_links": [
        "main.rs passes ctx.worktree_resolution into check_git_bash_command at the existing shell call site",
        "docs/history/worker-outward-guard/CONTEXT.md D1-D11 are the locked rules"
      ],
      "prohibitions": [
        "No change to the idle-gate arm, the gc-2 arm, the staging arm, or any main-checkout verdict",
        "No second fence or tokenize pass",
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
    "title": "Name the worker-outward key, its allowlist and its refusal in the docs and the verify map",
    "lane": "high-risk",
    "role": "docs",
    "deps": [],
    "decisions": ["58925048-fe36-4a5f-8342-09f1f6afb9cb", "293cdfe5-33ef-41c0-a164-41f13bd0d12a", "1b8955e5-d9e3-4865-ac10-ca10b92bf3c7"],
    "files": [
      "docs/config-reference.md",
      "docs/07-contracts.md",
      "docs/handbook/register.md",
      "docs/product-description/foundations/guards.md",
      ".bee/verify/verify-app/features/worker-outward-guard.md",
      ".bee/verify/verify-app/features/README.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/worker-outward-guard/CONTEXT.md",
      "docs/config-reference.md",
      "docs/07-contracts.md",
      ".bee/verify/verify-app/features/semantic-role-routing.md",
      ".bee/verify/verify-app/features/README.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Five edits, then the regen chain. Write every sentence through bee-technical-writing. (1) docs/config-reference.md — in the `guards` row (the line starting `| \\`guards\\` | \\`idle_gate\\` (\\`false\\` disables the idle intake gate)`), add ` · \\`worker_outward\\` (\\`false\\` disables the worker-outward guard — see the section below)` after the idle_gate clause and ` · worker-outward guard on` in the defaults cell; then, beside the existing `### guards.memory_root (GH #71)` section, add `### guards.worker_outward` (the ONE prose home of the allowlist) saying: inside any linked git worktree — bee-granted or not — the write guard refuses, in every phase, `git push` in every spelling, every `gh` command except these reads: `pr view|list|status|checks|diff`, `run list|view|watch|download`, `issue view|list`, `release view|list|download`, `repo view`, `workflow list|view`, `search …`, `cache list`, `label list`, `status`, `auth status`, `api` with an explicit GET (`-f`/`-F` fields allowed then) and `api graphql` with no `mutation`; and a nested `claude`/`codex`/`pi`/`opencode` launch unless it is the command `bee dispatch prepare` returned (a configured `models.*.command` prefix, or `codex exec` with a read-only sandbox). Say that this refuses a human in their own worktree too; that the key is read from the MAIN checkout's `.bee/config.json` and the worktree's tracked copy is not read; that `bee` is never judged; that a `gh` alias is refused; and that the guard judges command lines, not script contents. (2) docs/07-contracts.md — do NOT put it under checkWrite; add a sibling bullet group `checkGitBashCommand(root, state, command, cwd, …)` → judges shell requests (Bash / codex exec), not file writes, with one bullet: `outward (worker-outward-guard D1-D11): cwd resolves to a linked-valid worktree → deny \\`git push\\`, any \\`gh\\` form outside the read-only list, and an unsanctioned nested agent launch, in every phase; the refusal leads with the remedy for its form. Default-on; disable per repo via \\`config.guards.worker_outward: false\\` in the MAIN checkout. The list and the gotchas live in config-reference.md § guards.worker_outward.` (3) docs/handbook/register.md — the `guards` row (`| \\`guards\\` | write-guard tuning: \\`idle_gate\\`, \\`max_read_lines\\` |`) adds `, \\`worker_outward\\``. (4) docs/product-description/foundations/guards.md — the Configuration sentence naming `guards.idle_gate` adds `\\`guards.worker_outward\\`` to its list; no other change. (5) .bee/verify/verify-app/features/worker-outward-guard.md — a new feature file of about 60 lines in the four-section shape of semantic-role-routing.md: what it is (the three refused forms, the read list by pointer to config-reference, the main-checkout opt-out, the door exemption); how a user reaches it (a Bash tool call from a session whose cwd is a `bee worktree new` worktree — a human in their own worktree included); how to drive it with control-bee (launch a sandbox, `control-bee cli -- worktree new --feature demo`, then craft a PreToolUse write-guard payload whose `cwd` is the worktree path per the README's `VERIFY_CWD` convention and `command` is `git push origin main`, expect exit 2 and text containing `bee worktree merge`; `gh pr view 1` → exit 0; `claude -p x` → exit 2 and text containing `bee dispatch prepare`; write `{\"guards\":{\"worker_outward\":false}}` into the SANDBOX MAIN root's .bee/config.json, rerun `git push origin main`, expect exit 0); gotchas (`git push --dry-run` is refused; heredoc bodies are not judged; `echo git push` is refused; the worktree's own config copy is not read; a `gh` alias is refused; `bee herding run` still opens a pane). Add one index line for it in .bee/verify/verify-app/features/README.md in the existing list style. Then run `.bee/bin/bee dev regen` so the rendered copies of the verify tree under .claude/, .agents/ and .opencode/ match (regen output is not a hand edit), and commit the refreshed copies plus docs/history/codex-harness-hardening/release-manifest.json with the rest. Prove: `.bee/bin/bee dev release-manifest --check` green and `rg -l worker_outward docs/config-reference.md docs/07-contracts.md docs/handbook/register.md docs/product-description/foundations/guards.md` printing four paths. One commit, subject in imperative mood, trailer `cell: wog-2`.",
    "must_haves": {
      "truths": [
        "docs/config-reference.md has a guards.worker_outward section that lists every allowed gh read, the door exemption, the main-checkout opt-out and the gotchas, and its guards row points there",
        "docs/07-contracts.md carries the outward rule under a shell-request bullet, not under checkWrite, and points at the config section for the list",
        "docs/handbook/register.md and docs/product-description/foundations/guards.md name worker_outward beside idle_gate",
        "the verify map has a worker-outward-guard feature file of about 60 lines with a drivable refusal recipe that places the opt-out in the sandbox main root, plus an index line",
        "bee dev release-manifest --check is green and rg finds worker_outward in the four docs"
      ],
      "artifacts": [
        {"path": "docs/config-reference.md", "substantive": "the guards row clause and the ### guards.worker_outward section"},
        {"path": ".bee/verify/verify-app/features/worker-outward-guard.md", "substantive": "the four-section feature file with the drivable recipe"}
      ],
      "key_links": [
        "docs/history/worker-outward-guard/CONTEXT.md D5, D6, D10 and D11 are the words the docs must match",
        "wog-1 is the behavior the feature file drives; the recipe asserts only substrings D5 locks"
      ],
      "prohibitions": [
        "No hand edit under .claude/, .agents/ or .opencode/ — those come from bee dev regen",
        "No edit to AGENTS.md or any skill",
        "No second copy of the allowlist outside config-reference.md",
        "No new claim about behavior wog-1 does not implement"
      ]
    },
    "verify": ".bee/bin/bee dev release-manifest --check && rg -l worker_outward docs/config-reference.md docs/07-contracts.md docs/handbook/register.md docs/product-description/foundations/guards.md"
  }
]
```

## Test matrix

High-risk: the twelve dimensions of `edge-dimensions.md`, applicable ones probed; each row ends with its pass condition.

| dimension | probe | pass when |
|---|---|---|
| 1 user types | orchestrator in main vs worker in worktree vs human in their own worktree, same command | main exit 0 + empty stderr; both worktree callers exit 2 |
| 2 input extremes | empty command; a 2 KB compound line ending in `&& git push`; a wrapper nest past the depth bound | empty → 0; compound → 2; nest with push → 2, nest with `ls` → 0 + gap line |
| 3 timing | phase idle vs swarming in the worktree | both exit 2; idle text is the outward text |
| 5 state transitions | `guards.worker_outward` false in main, then removed; false only in the worktree copy | main false → 0; removed → 2; worktree-only → 2 |
| 6 environment | codex `exec` payload with `cmd`; `env FOO=1 claude`; `sudo claude`; `npx -y claude` | all exit 2 |
| 7 error cascades | unreadable main config when a form is about to deny | the read's own error path, never a delegate on a harmless command |
| 8 authorization | `gh api` GET with `-f` vs `-X POST`/`-XPOST`/`--method=POST`; `gh pr view` vs `gh pr merge` vs `gh pr checkout`; `gh auth status` vs `gh auth token` | reads 0; writes, checkout and token 2 |
| 9 data integrity | `linked-invalid` worktree | existing WORKTREE_LINK_INVALID text, unchanged |
| 10 integration | `bee dispatch prepare …`, `bee herding run …`, `bash scripts/release.sh 1.0.0`, `codex exec --sandbox read-only … -`, a configured cli prefix | all exit 0 |
| 12 business logic | `git -C <main> push` from the worktree; `echo git push`; heredoc-body push | first two exit 2 (location is the cwd; the finder scans tokens); heredoc → 0 |
| 4 scale, 11 compliance | not applicable — one request per call, no regulated data | — |
| regression | `git push` at idle in main on main and on head | exit 2 with the existing `never exempted` text on both |

## Open Questions

- (none blocking) A host whose main is a submodule or uses `--separate-git-dir` resolves `ordinary` and gets no refusal; recorded, fails open to today's behavior.

## Out of scope

- Refusing raw `git push` from the main checkout in execution phases (CONTEXT Deferred Ideas).
- Reading a script's contents for pushes it contains.
- A doctor row reporting the key's state (the read-side signal for a worktree-flipped config that lands in main at merge — the merge diff is the signal today; Deferred Ideas).
- A per-call escape hatch for a human in their own worktree (would be a new decision).
