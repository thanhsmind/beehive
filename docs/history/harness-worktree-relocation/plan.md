---
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Harness worktree relocation

## Summary

Today only Pi moves its session into a bee worktree and back. Claude Code,
Codex and OpenCode get no move help, and even Pi gets stuck when an agent runs
`cd <main> && bee worktree merge --id …`: the merge lands, and the session stays
in the worktree.

After this plan, bee knows which harness session called it and where that
session sits. `bee worktree new`, `enter`, the new `exit`, and `merge` each print
one instruction for that harness: Pi moves by itself (now also on a plain exit),
Claude Code calls `EnterWorktree` / `ExitWorktree`, and Codex and OpenCode ask
the user to type `/cd <path>` or `/move <path>` and then send a message. A merge
started from main while the session still sits in the worktree moves the session
first. It does not merge behind the session's back.

Mode: `standard` — 2 risk flags: public-contracts (a new verb, a new merge flag,
new result fields), cross-platform (four harness runtimes)
Why this is the least workflow that protects the work: one new verb, one merge
arm, one Pi belt operation, a small OpenCode env change and a docs sync; the
transition payload, merge transaction, cleanup and worktree paths do not change.

## Requirements (from CONTEXT.md)

- D1 (`016ae1ef`): `bee worktree exit` moves the session from a worktree back to
  main with no merge and keeps the worktree. Exit-before-merge stays.
- D2 (`86337de7`): `bee worktree merge` run from main while the calling session
  still sits in that worktree emits `exit-worktree-before-merge` and does not
  merge in place; `--detached` merges directly for sessionless callers.
- D3 (`6c860e87`): on Codex, bee marks a waiting-on question and tells the user
  to type `/cd <target path>`.
- D4 (`c6023a6e`): worktree paths stay siblings; Claude Code moves through
  `EnterWorktree {path}` / `ExitWorktree {action: keep}`.

## Load-bearing claims

<!-- bee:not-a-deferral: the rows below are evidence about what the code does today, not promises of future work -->

Labels: `read` (file opened at that line) or `ran` (command run, output kept).
Evidence is a verbatim substring of the anchored bytes.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Merge picks its path from the process cwd only, so `cd <main> && bee worktree merge` always takes the direct-merge arm | read | `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:764`, `:797` | `if ctx.kind == "linked-valid" {` / `if ctx.kind != "ordinary" {` |
| 2 | The stderr marker fires only when a Pi session id is present; this plan keeps that | read | `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:112` | `if transition.get("piSessionId").and_then(|v| v.as_str()).is_some() {` |
| 3 | The Pi belt refuses any transition whose keys differ from the exact v1 set, so the transition object must keep its keys | read | `.pi/extensions/bee-guard.ts:1111`, `:1152` | `const EXPECTED_TRANSITION_KEYS = [` / `if (!hasExactKeys(parsed, EXPECTED_TRANSITION_KEYS)) {` |
| 4 | The Pi belt knows only two operations today | read | `.pi/extensions/bee-guard.ts:1158` | `if (parsed.operation !== "enter-worktree" && parsed.operation !== "exit-worktree-before-merge") {` |
| 5 | The Pi belt re-runs bee to verify each transition, and has a command arm only for enter and merge, so a new operation needs its own verify arm against a real verb | read | `.pi/extensions/bee-guard.ts:1346-1349` | `  if (intent.operation === "enter-worktree") {` / `    if (!intent.worktreeId) return null` / `    args = ["worktree", "enter", "--id", intent.worktreeId, "--json"]` / `  } else if (intent.operation === "exit-worktree-before-merge") {` |
| 6 | The Pi belt compares `sourceCwd` to its own cwd byte for byte | read | `.pi/extensions/bee-guard.ts:1221` | `if (parsed.sourceCwd !== canonSource || parsed.targetCwd !== canonTarget) {` |
| 7 | Hook events stamp the session record with the harness-reported cwd | read | `packages/bee-rs/crates/bee/src/hooks/activity.rs:307` | `activity.insert("cwd".into(), Value::String(ctx.cwd.to_string_lossy().into_owned()));` |
| 8 | Claude Code runs the activity stamp before every tool call, so the record's cwd is current when a Bash call runs bee | ran | `python3` walk of `.claude/settings.json` hooks | `.claude/settings.json PreToolUse None activity` |
| 9 | Claude Code's session record id equals `CLAUDE_CODE_SESSION_ID`, and its cwd follows `EnterWorktree` | ran | `echo "$CLAUDE_CODE_SESSION_ID"; rg -n '"cwd"' /home/thanhsmind/Projects/goglbe/beehive/.bee/sessions/7033d8f8-2786-4936-b02c-73b18858ea0e.json` | `"cwd": "/home/thanhsmind/Projects/goglbe/beehive--wt--harness-worktree-relocation",` |
| 10 | Codex exports its thread id to shell children, and it equals the id bee's session record uses (the rollout id) | ran | `codex exec --skip-git-repo-check "Run exactly: echo T=\$CODEX_THREAD_ID S=\$CODEX_SESSION_ID ; print the output verbatim"` then `ls -t ~/.codex/sessions/2026/09/16` | `T=01a0a9de-6a1c-7663-9c9b-f6fb1018d111 S=01a0a9de-6a1c-7663-9c9b-f6fb1018d111` |
| 11 | A Codex started from a Claude Code pane inherits `CLAUDE_CODE_SESSION_ID`, so the innermost harness variable must win | ran | `codex exec --skip-git-repo-check "Run exactly: env | grep -c '^CLAUDE_CODE_SESSION_ID=' ; print output verbatim"` | `1` |
| 12 | The shared session-id lookup knows no Codex variable | read | `packages/bee-rs/crates/bee/src/session_identity.rs:18` | `"PI_SESSION_ID",` |
| 13 | Herding passes `BEE_SESSION_ID` into worker panes, so a bare `BEE_SESSION_ID` can name the leader, not the caller | read | `packages/bee-rs/crates/bee/src/herding/run.rs:1287` | `"BEE_SESSION_ID",` |
| 14 | The OpenCode plugin reports the plugin's fixed directory as cwd, never the session's, so bee cannot tell where an OpenCode session sits | read | `.opencode/plugins/bee-guard.ts:541` | `cwd: directory,` |
| 15 | A session record reader exists in the state group | read | `packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:329` | `pub(crate) fn read_session(root: &Path, session_id: &str) -> Ex<Option<Map<String, Value>>> {` |
| 16 | The waiting-on writer takes the session id as an argument, so bee can pass the Codex id it located | read | `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs:455-460` | `pub(crate) fn set_workflow_waiting_on(` / `    root: &Path,` / `    id: &str,` / `    kind: &str,` / `    subject: &str,` / `    session: &str,` |
| 17 | Claude Code moves a session launched in main into a sibling bee worktree with `EnterWorktree {path}` | ran | `EnterWorktree {path: "/home/thanhsmind/Projects/goglbe/beehive--wt--harness-worktree-relocation"}` (this session) | `Entered worktree at /home/thanhsmind/Projects/goglbe/beehive--wt--harness-worktree-relocation on branch wt/harness-worktree-relocation.` |
| 18 | The earlier Pi design ruled out a plain exit verb (superseded by D1) | read | `docs/history/pi-worktree-session-relocation/plan.md:54` | `No plain exit command is added. The locked user value is exit-before-merge.` |
| 19 | The herding merge role merges from main | read | `skills/bee-herding/references/role-merge.md:163` | `bee worktree merge --id <grant-key> --cleanup` |
| 20 | The Pi belt bytes are compiled into bee, so the belt edit ships with the binary | read | `packages/bee-rs/crates/bee/src/doctor.rs:47` | `const PI_EXTENSION_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts");` |
| 21 | The Pi contract tests pin the registered command list | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:4371` | `for cmd in ["bee-worktree-new", "bee-worktree-enter", "bee-worktree-merge", "bee-worktree-relocate"] {` |
| 22 | The shared agent instructions tell a session to move into a worktree but never how to leave | read | `packages/bee/AGENTS.block.md:189` | `Move the session into the worktree` |

<!-- /bee:not-a-deferral -->

## Discovery

An advisor-tier research pass (2026-09-16) inventoried the four harnesses
(Claude Code 2.1.273 EnterWorktree/ExitWorktree; Codex 0.154.0 `/cd`, user-typed
only; OpenCode 1.18.30 `/move` and an experimental move-session route, unprobed;
Pi 0.85.1 done). The incident was confirmed in the memorypad Pi session log: the
merge call was `cd "$main"` then
`.bee/bin/bee worktree merge --id memorypad--wt--memorypad-omarchy --no-cleanup`.
The plan-step hat wave critiqued the first draft; its synthesis is in
`## Approach` ("Hat wave synthesis"). Claims 8–11 and 17 were run in this session.

## Approach

Recommended path — bee locates its caller, then prints that harness's move.

1. **Caller locator** (hwr-1). A new function beside the shared lookup in
   `session_identity.rs` (its existing order does not change, claim 12) returns
   the caller as `{id, runtime}` from the FIRST variable present, innermost
   harness first (claim 11): `PI_SESSION_ID` → pi, `CODEX_THREAD_ID` → codex,
   `BEE_SESSION_ID` only when `BEE_RUNTIME` is also set → that runtime (claim 13),
   `CLAUDE_CODE_SESSION_ID` → claude. A second function reads that id's record
   through `read_session` (claim 15) and returns its `activity.cwd`. It reads the
   caller's OWN record only; no 90-second liveness window applies, because the
   env id already proves the caller is that session. Both take an injected env
   lookup so tests never read the real process env.
2. **Instruction beside the transition** (hwr-1). `sessionTransition` keeps its
   exact v1 keys (claim 3) and the Pi-only marker (claim 2). The result gains
   top-level `sessionRuntime` and `instruction`, in JSON and as one text line,
   on every transition path: pi — the session moves when this turn settles;
   claude — `Call EnterWorktree with path=<target>` (enter) or
   `Call ExitWorktree with action=keep` (exit), plus
   `then run bee worktree merge --id <id>` for exit-before-merge (D4);
   codex — `Ask the user to type /cd <target>, then send any message to continue`
   (D3); opencode — the same with `/move <target>`; unknown —
   `Open a session at <target>`.
3. **Exit verb and merge detection** (hwr-2). `bee worktree exit [--id]` inside a
   linked worktree is zero-mutation and emits operation `exit-worktree` (source
   the worktree root, target main, continuation null). From main it needs
   `--id`, and emits the same when the caller's record cwd sits inside that
   worktree's root; otherwise it refuses. `bee worktree merge --id X` from main,
   without `--detached`, emits `exit-worktree-before-merge` and merges nothing
   when the caller's record cwd sits inside worktree X (D2). In both, `sourceCwd`
   is the caller's canonical cwd, so Pi's byte-for-byte check holds (claim 6).
   For codex and opencode callers, bee also writes a waiting-on question mark
   with the located id passed in (claim 16, D3).
4. **Pi learns plain exit** (hwr-3), after hwr-2 so its verify arm (claim 5)
   runs against the real verb: accept `exit-worktree` (claim 4), verify with
   `bee worktree exit --id <id> --json`, relocate to main with no merge, and add
   a `/bee-worktree-exit` command (claim 21).
5. **OpenCode names its session** (hwr-4): a `shell.env` hook exports
   `BEE_SESSION_ID` and `BEE_RUNTIME=opencode`, so bee prints the `/move`
   instruction. OpenCode reports no session cwd (claim 14), so merge detection
   and exit-from-main cannot fire there; the docs say so.
6. **Docs** (hwr-5): AGENTS block and swarming skill name the per-harness move
   and `bee worktree exit`; the herding merge role passes `--detached`; the
   mapped feature file and the worktree knowledge area carry the new verb, the
   detection, the locator order, and the limits (OpenCode no detection; Claude
   Code `cd` that persisted from an earlier Bash call is not seen).

SMALLER PATH: the first draft added payload v2, a new locator module, an
OpenCode move adapter, and split locator from verb. The alternatives hat showed
a cheaper shape honoring D1–D4: keep the payload at v1 with instruction fields
beside it, reuse `read_session`, and give OpenCode the instruction floor that
CONTEXT.md's discretion clause already allows. Adopted. The Rust work stays two
cells (locator + instruction, then verb + detection) because large single Rust
cells stall the configured code worker; the split costs one serial edge on
already-serial files.

Hat wave synthesis (accepted findings, each re-checked against bytes):

- Pi re-verifies every transition by re-running bee (claim 5) → Pi cell runs
  after the real exit verb exists and adds a verify arm.
- `sourceCwd` must equal the caller's exact cwd (claim 6) → set from the record.
- A slow permission prompt can age the 90 s window → no liveness window; the
  caller's own record is used.
- Herded panes carry the leader's `BEE_SESSION_ID` (claim 13) → used only with
  `BEE_RUNTIME`.
- Existing tests pin v1 payload and a "no marker" case → payload stays v1;
  tests use injected env; `registry_contracts` joins hwr-2's verify.
- Codex `/cd` alone does not resume the turn → instruction says send a message.
- OpenCode writes no session cwd (claim 14) → instruction floor, limit documented.
- Rejected: a "Claude exit loop" worry — the PreToolUse activity stamp (claim 8)
  refreshes cwd before the Bash call that runs bee.

Rejected alternatives:

- Payload v2 inside `sessionTransition` — rejected: breaks the Pi exact-key check
  for no locked decision.
- Change the shared `session_identity.rs` order — rejected: every
  session-owned command would change owner resolution.
- Refuse the merge instead of emitting exit — rejected by the user (D2).
- OpenCode move-session adapter now — rejected for this slice: the route is
  experimental and unprobed; filed as backlog follow-up.

Risk map:

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| Merge detection | MEDIUM | A false positive stops a merge; a false negative repeats the incident | hwr-2 | Unit tests: caller cwd in worktree → exit intent, main HEAD unchanged; caller cwd main → merges; no caller env → merges; `--detached` → merges |
| Caller locator | MEDIUM | Wrong pick gives the wrong instruction | hwr-1 | Unit tests: Codex under Claude env → codex; bare `BEE_SESSION_ID` skipped; empty env → none |
| Pi exit | MEDIUM | Verify arm must match real bee output | hwr-3 | Contract test with the real verb's JSON shape; existing Pi tests stay green |
| Existing v1 tests | LOW | Env-dependent tests could turn red under Claude/Codex | hwr-1 | Worktree test module green inside this Claude session |
| Docs | LOW | Text drift from the CLI | hwr-5 | `bee dev release-manifest --check`; rg pins |

Waves: {hwr-1, hwr-4} in parallel (disjoint files). hwr-2 follows hwr-1 (same
files, needs the locator). hwr-3 follows hwr-2 (verify arm needs the real verb).
hwr-5 follows hwr-3 and hwr-4 (text must match shipped behavior).

## Shape

| Cell | Title | Role | Files | Depends on |
|---|---|---|---|---|
| hwr-1 | Locate the caller session and print a per-harness move instruction | code | `packages/bee-rs/crates/bee/src/session_identity.rs`, `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs`, `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs` | — |
| hwr-2 | Add `bee worktree exit` and move the session before a merge started from main | code | `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs`, `.../worktree/mod.rs`, `.../worktree/tests.rs`, `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | hwr-1 |
| hwr-3 | Teach the Pi belt plain exit | code | `.pi/extensions/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` | hwr-2 |
| hwr-4 | Export the OpenCode session id to shell children | code | `.opencode/plugins/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs` | — |
| hwr-5 | Document enter and exit per harness | docs | AGENTS block + regen, swarming skill, herding merge role, feature map, worktree knowledge area | hwr-3, hwr-4 |

## Cells (current slice)

```json
[
  {
    "id": "hwr-1",
    "feature": "harness-worktree-relocation",
    "title": "Locate the caller session and print a per-harness move instruction",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["c6023a6e-b1f2-4eb7-bf31-ea542d0cb975", "6c860e87-9aff-42f4-8fed-8e628a217e21"],
    "files": [
      "packages/bee-rs/crates/bee/src/session_identity.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/harness-worktree-relocation/CONTEXT.md",
      "docs/history/harness-worktree-relocation/plan.md",
      "packages/bee-rs/crates/bee/src/session_identity.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "(1) In session_identity.rs, add (do NOT change `SESSION_ENV_VARS` or `resolve_env_session_id_from`) a caller locator over an injected lookup `F: Fn(&str) -> Option<String>`: return the FIRST present non-blank candidate as {id, runtime} in this order — `PI_SESSION_ID` → \"pi\"; `CODEX_THREAD_ID` → \"codex\"; `BEE_SESSION_ID` only when `BEE_RUNTIME` is also non-blank → runtime = BEE_RUNTIME's value; `CLAUDE_CODE_SESSION_ID` → \"claude\"; else none. Add a second function that, given the main checkout root and a located id, reads that session's record with `read_session` from verbs/state_group/store.rs (`pub(crate) fn read_session(root: &Path, session_id: &str) -> Ex<Option<Map<String, Value>>> {`) and returns `activity.cwd` when present. No liveness window. (2) In verbs/worktree/handlers.rs, leave `sessionTransition` and `build_session_transition_with_session_id` byte-identical in keys and schemaVersion, and leave `format_pi_transition_marker` Pi-only. On every path that returns a sessionTransition (worktree new, worktree enter, merge from inside a linked worktree — find them by `\"enter-worktree\",` and `\"exit-worktree-before-merge\",`), add top-level result fields `sessionRuntime` (runtime string or null) and `instruction` (string), and print the instruction as one text line. Build the instruction in one function keyed by runtime and operation: pi → `Pi moves this session to <target> when this turn settles.`; claude → `Call EnterWorktree with path=<target>.` for enter-worktree, `Call ExitWorktree with action=keep.` for exit-worktree, and `Call ExitWorktree with action=keep, then run bee worktree merge --id <id> from <target>.` for exit-worktree-before-merge; codex → `Ask the user to type /cd <target>, then send any message to continue.`; opencode → `Ask the user to type /move <target>, then send any message to continue.`; none/other → `Open a session at <target>.` Handlers read the real env through the locator; the builder functions take the located caller as a parameter so tests inject it. (3) Tests: locator — Codex id with a Claude id also present returns codex; bare BEE_SESSION_ID without BEE_RUNTIME is skipped; BEE_SESSION_ID + BEE_RUNTIME=opencode returns opencode; empty env returns none; record cwd read from a fixture record. Instruction — one case per runtime and operation. Existing tests in verbs/worktree/tests.rs that assert schemaVersion 1 or the no-marker case (search `schemaVersion` and `no marker formatted`) must stay green with CLAUDE_CODE_SESSION_ID or CODEX_THREAD_ID set in the process env — run the module inside this environment to prove it.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee worktree && PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_identity",
    "must_haves": {
      "truths": [
        "the locator returns the innermost harness: Codex wins over an inherited Claude id",
        "a bare BEE_SESSION_ID without BEE_RUNTIME is not treated as the caller",
        "worktree new, enter and linked merge results carry sessionRuntime and a per-harness instruction in JSON and text",
        "sessionTransition keys, schemaVersion and the Pi-only marker are unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/session_identity.rs", "substantive": "caller locator and record-cwd reader with unit tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs", "substantive": "instruction builder and the two new result fields on every transition path"}
      ],
      "key_links": [
        "handlers.rs gets runtime from the new locator, not from PI_SESSION_ID directly",
        "the record-cwd reader goes through state_group read_session, not a new file reader"
      ],
      "prohibitions": [
        "Do not change SESSION_ENV_VARS or resolve_env_session_id_from",
        "Do not change sessionTransition keys, schemaVersion, or marker emission",
        "Do not change merge, queue, cleanup, or grant behavior",
        "Do not touch .pi or .opencode plugin files"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "hwr-2",
    "feature": "harness-worktree-relocation",
    "title": "Add bee worktree exit and move the session before a merge started from main",
    "lane": "standard",
    "role": "code",
    "deps": ["hwr-1"],
    "decisions": ["016ae1ef-c7d3-4949-81c2-28c6503906e1", "86337de7-429c-4317-b2e0-ed4fc7ff152a", "6c860e87-9aff-42f4-8fed-8e628a217e21"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/harness-worktree-relocation/CONTEXT.md",
      "docs/history/harness-worktree-relocation/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/session_identity.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "(1) New verb `bee worktree exit [--id <id>]` (per D1), routed where `worktree enter` is routed (`pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {` in handlers.rs, and mod.rs if it routes there) AND declared in src/generated/registry_payload.json beside `worktree enter` in the same change (hand-edit; no generator exists). Inside a linked worktree: verify the grant and Git links as `linked_worktree_merge_core` does, mutate nothing, and return a sessionTransition with operation `exit-worktree`, sourceCwd the canonical worktree root, targetCwd main, continuation null, plus hwr-1's `sessionRuntime` and `instruction`; an --id that differs from the current worktree refuses. From the main checkout: require --id; when hwr-1's locator finds a caller whose record cwd sits inside that worktree's canonical root, return the same transition with sourceCwd = the caller's canonical cwd; otherwise refuse with a typed error saying no calling session sits in that worktree. Emit the Pi marker exactly as the other paths do. (2) In `run_merge`, in the ordinary arm after `if ctx.kind != \"ordinary\" {` and the --id check, BEFORE `resolve_cleanup_on_merge` and every lock or precondition: unless the new boolean flag `--detached` is true, locate the caller; when its record cwd sits inside the --id worktree's canonical root, return `ok: true` with a sessionTransition of operation `exit-worktree-before-merge` (sourceCwd = caller's canonical cwd, targetCwd = main, continuation from `build_merge_continuation` with the given flags), hwr-1's fields, and a text line saying the merge did NOT run yet; merge nothing (per D2). Validate `--detached` fail-closed with `bool_flag_ok` and add it to the `keys_known` list; declare it on `worktree merge` in registry_payload.json. (3) When the located runtime is codex or opencode, both paths also record a waiting-on question for that session through `set_workflow_waiting_on` in verbs/workflow_store/record.rs, passing the located id as `session` explicitly, subject naming the target path (per D3); a failure to write the mark warns and does not fail the command. (4) Tests: exit inside a worktree emits exit-worktree and changes nothing on disk; exit from main with the caller's record cwd inside the worktree emits it with sourceCwd = that cwd; exit from main with no such caller refuses; merge from main with the caller's record cwd inside the worktree emits exit-worktree-before-merge and main's HEAD sha is unchanged; the same with --detached merges; caller cwd in main merges; no caller env merges; a codex caller gets the waiting-on mark. Inject the caller; never depend on the real process env.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee worktree && PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_dispatch --test registry_contracts",
    "must_haves": {
      "truths": [
        "bee worktree exit inside a worktree emits an exit-worktree transition and mutates nothing",
        "bee worktree merge --id X from main does not merge while the caller's session record cwd sits inside worktree X, and emits exit-worktree-before-merge",
        "bee worktree merge --id X --detached merges directly",
        "a codex or opencode caller gets a waiting-on question mark"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs", "substantive": "exit handler and the merge detection arm with --detached"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "worktree exit declared; --detached declared on worktree merge"}
      ],
      "key_links": [
        "the worktree verb table routes `worktree exit` to its handler",
        "merge detection uses hwr-1's locator and record-cwd reader"
      ],
      "prohibitions": [
        "Do not change the merge transaction, queue, proof, uat, or cleanup behavior once the direct-merge path is taken",
        "Do not change sessionTransition keys or schemaVersion",
        "Do not touch plugin files or docs"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "hwr-3",
    "feature": "harness-worktree-relocation",
    "title": "Teach the Pi belt plain exit",
    "lane": "standard",
    "role": "code",
    "deps": ["hwr-2"],
    "decisions": ["016ae1ef-c7d3-4949-81c2-28c6503906e1"],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/harness-worktree-relocation/CONTEXT.md",
      "docs/history/harness-worktree-relocation/plan.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Teach the Pi belt the `exit-worktree` operation (per D1). Sites in .pi/extensions/bee-guard.ts: the parser check `if (parsed.operation !== \"enter-worktree\" && parsed.operation !== \"exit-worktree-before-merge\") {` (~1158) and its continuation branches just below; the verify arms `if (intent.operation === \"enter-worktree\") {` / `} else if (intent.operation === \"exit-worktree-before-merge\") {` (~1346-1349) — add an arm running `worktree exit --id <worktreeId> --json` (real verb added by hwr-2); the comparison `if (validatedVerified.operation !== intent.operation) return null` (~1386); the post-switch branch `if (intent.operation === \"exit-worktree-before-merge\") {` (~1415 and ~1673); and the relocation guard `intent.operation !== \"enter-worktree\" && intent.operation !== \"exit-worktree-before-merge\"` (~1499). `exit-worktree` requires `continuation === null`, relocates to targetCwd through the same forkFrom + switchSession path, runs NO merge, and its notice says the worktree was kept and names `/bee-worktree-enter --id <id>` to return. Add a `/bee-worktree-exit [--id <id>]` command modeled on `/bee-worktree-enter`, and add it to the pinned list `for cmd in [\"bee-worktree-new\", \"bee-worktree-enter\", \"bee-worktree-merge\", \"bee-worktree-relocate\"] {` in pi_plugin_contracts.rs. Contract tests, written red first: an exit-worktree marker relocates and no merge command runs; the fake bee's `worktree exit --json` output uses the exact JSON shape hwr-2's handler returns (copy from hwr-2's unit test or run the built binary); `/bee-worktree-exit` relocates; existing enter and exit-before-merge tests stay green.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "an exit-worktree marker relocates the Pi session to main and runs no merge",
        "the belt verifies exit-worktree by running bee worktree exit --json",
        "/bee-worktree-exit is a registered Pi command",
        "existing enter and exit-before-merge relocation still work"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "exit-worktree parse, verify arm, relocation branch, /bee-worktree-exit command"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "exit relocation and command tests"}
      ],
      "key_links": [
        "exit-worktree goes through the same fork and switchSession path as exit-worktree-before-merge"
      ],
      "prohibitions": [
        "Do not change EXPECTED_TRANSITION_KEYS or the schemaVersion check",
        "Do not touch Rust source under packages/bee-rs/crates/bee/src"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "hwr-4",
    "feature": "harness-worktree-relocation",
    "title": "Export the OpenCode session id to shell children",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["6c860e87-9aff-42f4-8fed-8e628a217e21"],
    "files": [
      ".opencode/plugins/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/harness-worktree-relocation/plan.md",
      ".opencode/plugins/bee-guard.ts",
      ".opencode/node_modules/@opencode-ai/plugin/dist/index.d.ts",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add a `shell.env` hook to the OpenCode plugin (signature in .opencode/node_modules/@opencode-ai/plugin/dist/index.d.ts) that sets `BEE_SESSION_ID` to the hook's sessionID and `BEE_RUNTIME` to `opencode` for every shell child, so bee's caller locator prints the `/move` instruction for OpenCode. Keep existing env entries; set nothing when sessionID is absent. The hook never throws. Contract test in opencode_plugin_contracts.rs: the plugin exposes `shell.env`; calling it with a sessionID yields both variables; calling it without one adds neither.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test opencode_plugin_contracts",
    "must_haves": {
      "truths": [
        "every OpenCode shell child sees BEE_SESSION_ID and BEE_RUNTIME=opencode when a session id exists"
      ],
      "artifacts": [
        {"path": ".opencode/plugins/bee-guard.ts", "substantive": "shell.env hook"},
        {"path": "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs", "substantive": "shell.env contract test"}
      ],
      "key_links": [
        "the hook is registered in the plugin's returned hooks object beside tool.execute.after"
      ],
      "prohibitions": [
        "Do not change the plugin's existing advisory hooks or tools-logger behavior",
        "Do not touch .pi or Rust source under packages/bee-rs/crates/bee/src"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "hwr-5",
    "feature": "harness-worktree-relocation",
    "title": "Document enter and exit per harness",
    "lane": "standard",
    "role": "docs",
    "deps": ["hwr-3", "hwr-4"],
    "decisions": ["016ae1ef-c7d3-4949-81c2-28c6503906e1", "86337de7-429c-4317-b2e0-ed4fc7ff152a", "6c860e87-9aff-42f4-8fed-8e628a217e21", "c6023a6e-b1f2-4eb7-bf31-ea542d0cb975"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "AGENTS.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-herding/references/role-merge.md",
      ".bee/verify/verify-app/features/worktree-and-close.md",
      "docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md",
      "docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"
    ],
    "read_first": [
      "docs/history/harness-worktree-relocation/CONTEXT.md",
      "docs/history/harness-worktree-relocation/plan.md",
      "packages/bee/AGENTS.block.md",
      "skills/bee-herding/references/role-merge.md",
      ".bee/verify/verify-app/features/worktree-and-close.md"
    ],
    "affects_skills": ["skills/bee-swarming/SKILL.md", "skills/bee-herding/references/role-merge.md"],
    "affects_specs": ["docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md", "docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],
    "action": "Sync the words with the shipped behavior. (1) packages/bee/AGENTS.block.md, where it says `Move the session into the worktree`: add one short rule — follow the `instruction` line bee prints on `worktree new`, `enter`, `exit` and `merge`; leave with `bee worktree exit`; never `cd` to main inside a shell call to merge. Regenerate AGENTS.md with `.bee/bin/bee dev regen`, never by hand. (2) skills/bee-swarming/SKILL.md near `Enter the worktree first (EnterWorktree, or a session/pane`: name `bee worktree exit` as the way back. (3) skills/bee-herding/references/role-merge.md: every `bee worktree merge --id <grant-key> --cleanup` gains `--detached`, with one sentence why (per D2). (4) .bee/verify/verify-app/features/worktree-and-close.md: add rows and drive steps for `worktree exit`, merge-from-main detection, `--detached`, and the `instruction`/`sessionRuntime` fields. (5) Both knowledge files: add the exit verb, the caller locator order, the detection rule, the two limits (OpenCode has no session cwd so detection never fires there; a Claude Code `cd` that persisted from an earlier Bash call is not seen), cite decisions 016ae1ef, 86337de7, 6c860e87, c6023a6e, and mark the old no-exit statement superseded. Short plain sentences.",
    "verify": ".bee/bin/bee dev release-manifest --check && rg -n 'worktree exit' packages/bee/AGENTS.block.md AGENTS.md skills/bee-swarming/SKILL.md && rg -c -- '--detached' skills/bee-herding/references/role-merge.md",
    "must_haves": {
      "truths": [
        "the AGENTS block and swarming skill name bee worktree exit and the instruction line",
        "the herding merge role passes --detached",
        "the feature map and knowledge area describe exit, detection, locator order and limits"
      ],
      "artifacts": [
        {"path": "packages/bee/AGENTS.block.md", "substantive": "per-harness move rule"},
        {"path": ".bee/verify/verify-app/features/worktree-and-close.md", "substantive": "exit and detection drive steps"}
      ],
      "key_links": [
        "AGENTS.md is regenerated from AGENTS.block.md, not hand-edited"
      ],
      "prohibitions": [
        "Do not edit Rust or plugin source",
        "Do not rewrite unrelated sections of the knowledge files"
      ]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Case | Kind | Pass when |
|---|---|---|
| Merge from main, caller cwd in worktree (the incident) | happy | Pass when the JSON result has `sessionTransition.operation` = `exit-worktree-before-merge` and main's HEAD sha is unchanged |
| Same scenario on main vs head | behavior change | Pass when main (before) merges and head emits the transition |
| Merge with `--detached` | edge | Pass when the result is the normal merged payload |
| No caller env (human terminal, cockpit) | edge | Pass when the merge runs as today |
| Codex id with inherited Claude id | edge | Pass when `sessionRuntime` is `codex` and the instruction names `/cd` |
| `worktree exit` inside a worktree | happy | Pass when the transition operation is `exit-worktree` and `git status` is unchanged |
| `worktree exit --id X` from main, no caller in X | error | Pass when the command exits non-zero with the typed "no calling session" error |
| Pi exit-worktree marker | happy | Pass when the Pi session switches to main and no merge command is recorded |
| Existing Pi and worktree tests | regression | Pass when both suites stay `ok` inside a Claude Code session |

## Open Questions

(none)

## Out of scope

- An OpenCode move adapter using the experimental move-session route (backlog follow-up after a probe).
- Moving bee worktree paths under `.claude/worktrees/` (rejected by D4).
- Codex programmatic relocation (no public surface today, D3).
- Detecting a Claude Code `cd` that persisted from an earlier Bash call.
