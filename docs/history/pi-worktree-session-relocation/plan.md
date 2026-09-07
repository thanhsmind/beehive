# pi-worktree-session-relocation — plan (high-risk lane)

**Goal.** Let one live Pi conversation enter a feature worktree and later return
to main before merge. Preserve the conversation. Do not change process cwd.
Decisions: [CONTEXT.md](CONTEXT.md).

**Lane.** High risk: this changes a public CLI result, a Pi runtime boundary,
worktree identity handling, session persistence, and Windows/POSIX paths.

**Execution shape.** Three serial cells. The CLI contract must exist before the
Pi belt can consume it. The verification map must describe the final contract.
This real dependency is the reason not to run these cells in parallel.

## Smallest honest design

Pi exposes `switchSession` only on `ExtensionCommandContext`. Event handlers and
LLM-callable tools receive `ExtensionContext`, which cannot replace a session.
The belt therefore does not switch inside `tool_result`.

The complete route has two entry paths:

1. A user runs a Pi command: `/bee-worktree-new`, `/bee-worktree-enter`, or
   `/bee-worktree-merge`. Its command handler receives the supported command
   context and can switch directly.
2. An agent runs the normal bee CLI through `bash` or `powershell`. A successful
   CLI process emits a Pi-only structured marker on stderr. The belt removes the
   marker from the visible tool result and holds its intent. After
   `agent_settled`, the belt submits one private extension command through
   `pi.sendUserMessage(..., { expandPromptTemplates: true })`. Pi dispatches that
   command with `ExtensionCommandContext`; no model turn is started.

This second path closes the current workflow gap: the agent still owns every bee
CLI command, while Pi alone owns the session replacement.

### User surfaces

- `bee worktree new --feature <slug>` and `/bee-worktree-new ...` create the
  worktree. Enter intent exists only after create, grant, bootstrap, and optional
  companion setup succeed.
- `bee worktree enter --id <id>` and `/bee-worktree-enter ...` enter an existing
  granted worktree. This is the recovery path after a refused merge. It is
  main-only and changes no bee or Git state.
- `bee worktree merge [--id <id>]` and `/bee-worktree-merge ...`, when run inside
  the matching granted worktree, emit exit-before-merge intent. They do not join
  the queue, merge, stop a companion, clean up, or write bee state there. Inside
  a linked worktree, omitted `--id` defaults to the current verified id.
- After Pi switches to main, the belt reconstructs and runs the intended merge
  from typed continuation fields. Main-side merge behavior stays unchanged.
- If that merge refuses, the session stays on main and names the exact
  `/bee-worktree-enter --id <id>` recovery command.

No plain exit command is added. The locked user value is exit-before-merge. The
existing-worktree enter path provides recovery without adding an unrelated
navigation mode.

## Transition contract

Successful CLI results contain one additive `sessionTransition` object:

```json
{
  "schemaVersion": 1,
  "operation": "enter-worktree | exit-worktree-before-merge",
  "sourceCwd": "/canonical/source",
  "targetCwd": "/canonical/target",
  "worktreeId": "git-verified-id",
  "feature": "creation-slug-or-null",
  "piSessionId": "present only inside Pi",
  "continuation": null
}
```

Exit intent carries this typed continuation instead of executable text:

```json
{
  "operation": "merge-worktree",
  "noCleanup": false,
  "skipUat": false,
  "queueWaitMs": null
}
```

The CLI derives paths, identity, feature, and normalized merge options from
validated state and flags. It never accepts transition paths from the caller.
The belt reconstructs merge argv from the typed continuation. It never executes
an argv or executable supplied inside an intent.

When `PI_SESSION_ID` is present, successful transition-producing commands also
write one compact marker to stderr:

```text
@@BEE_SESSION_TRANSITION@@ {same object}
```

JSON stdout stays valid. Plain text outside Pi stays unchanged. The belt accepts
a marker only from a successful `bash` or `powershell` result. It requires the
marker session id to equal the active Pi session id, strips the marker from the
visible result, and consumes it once.

Before any fork, the belt requires schema version 1, an exact known operation,
an exact typed field set, canonical absolute paths, `sourceCwd === ctx.cwd`, a
persistent source session file, and an existing target directory. It rejects a
same-source target. It serializes transitions and refuses a second live one.

## Load-bearing claims

| Claim | Evidence | Planning consequence |
|---|---|---|
| `worktree new` has one isolated result builder after all creation and registration steps succeed. | `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs`, `run_new` and `new_result_and_text` | Add enter intent only to final success. Refusal and rollback paths cannot emit it. |
| `worktree merge` rejects linked worktrees before the integration queue and merge phases. | `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs`, `run_merge`; `packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs`, `Pre` | Replace only this early branch with verified zero-mutation exit intent. |
| Worktree identity and main root already come from bidirectionally verified Git links. | `packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs`, `prelude`, `Pre::main_root`, strict grant readers | Reuse these values. Never derive identity from a directory name or user path. |
| Pi event contexts do not expose session replacement; registered commands receive `ExtensionCommandContext` with `switchSession(path, { withSession })`. | Pi 0.85.1 `docs/extensions.md`; package `dist/core/extensions/types.d.ts` | Event handlers only capture intent. A registered command performs replacement. |
| `pi.sendUserMessage` with `expandPromptTemplates: true` dispatches an extension command before any model prompt, including while a turn exists. | Pi 0.85.1 package `dist/core/agent-session.js`, `prompt` | Defer submission until `agent_settled`; the private command then runs without another model turn. |
| Pi sets the agent inactive before it emits `agent_settled`. | Pi 0.85.1 package `dist/core/agent-session.js`, `_emitAgentSettled` | Schedule the private command after the handler returns. User commands also require `ctx.isIdle()`. |
| `SessionManager.forkFrom(sourcePath, targetCwd)` writes a target-cwd header and copies every non-header entry. | Pi 0.85.1 `docs/sessions.md`, `docs/session-format.md`, and package `dist/core/session-manager.js` | Fork the settled source session, then switch to the returned session file. Never edit source JSONL. |
| Pi opens and validates the target before old-runtime teardown, then rebuilds target services. | Pi 0.85.1 package `dist/core/agent-session-runtime.js`, `switchSession` | Validate extension-owned conditions first. Do not claim rollback safety after Pi invalidates the old runtime. |
| The contract suite imports the real TypeScript extension and drives a stub Pi API in one Node process. | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`, `HARNESS_JS` | Extend it for commands, deferred submission, fake SessionManager, and ordering assertions. |

## Alternatives rejected

- **Call `ctx.switchSession` from `tool_result`:** rejected because Pi does not
  expose it there. A type cast would depend on a method absent at runtime.
- **Call `process.chdir`:** rejected by D1 because tools, resources, trust, and
  persistence would keep the old session binding.
- **Restart Pi in the target directory:** rejected because it does not preserve
  the active conversation through Pi's supported replacement path.
- **Only provide user slash commands:** rejected because bee agents create and
  merge worktrees through CLI tools. The created worktree would otherwise exist
  while the active session remained blocked on main.
- **Reload after a self-hosting merge:** rejected as extra lifecycle risk. The
  session switch already loads main resources; a later session can load merged
  extension bytes normally.
- **Duplicate CLI flag validation in TypeScript:** rejected. The belt only
  tokenizes without a shell and rejects NUL or malformed quotes. Rust remains
  the single source for command flag validation.

## Cells

### pwsr-1 — Emit verified CLI transition intent (`behavior_change`)

**Owns**

- `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs`
- `packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs`
- `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs`
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json`

**Work**

1. Add a shared schema-version-1 intent builder and Pi stderr marker emitter.
2. Add enter intent to successful `worktree new`.
3. Add main-only, zero-mutation `worktree enter --id` using strict grant and Git
   link resolution.
4. In linked `worktree merge`, default an omitted id to the current id. Require
   any supplied id to match. Require its strict grant. Emit typed exit intent and
   normalized merge continuation. Do nothing else in this branch.
5. Keep ordinary-main merge behavior unchanged.
6. Update registry help and routing for `enter` and the changed merge behavior.
7. Test exact JSON, Pi marker, non-Pi text, exit status, wrong id, absent grant,
   malformed flags, and zero mutation.

**Proof**

- Targeted worktree unit and CLI tests.
- Registry contract and dispatch tests for changed registry bytes.

### pwsr-2 — Replace the active Pi session (`integration`)

**Depends on** pwsr-1.

**Owns**

- `.pi/extensions/bee-guard.ts`
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`

**Work**

1. Register `/bee-worktree-new`, `/bee-worktree-enter`,
   `/bee-worktree-merge`, and one private token command.
2. Resolve the trusted bee binary with the belt's existing project/main lookup.
   Invoke it with `execFile`, bounded output, `--json`, and no shell.
3. Tokenize public command arguments with bounded quote handling. Reject NUL and
   malformed quoting. Let Rust validate all bee flags.
4. Capture Pi markers from successful shell tool results. Strip them before the
   result reaches the model. Hold one validated, session-bound intent.
5. After `agent_settled`, submit the private command through
   `pi.sendUserMessage` with template expansion. Use a random in-memory token;
   never put target paths into the private command text.
6. Require idle state. Validate the intent again. Get the current session file.
   Call `SessionManager.forkFrom` through the real manager instance's constructor,
   then call `ctx.switchSession` with the fork file.
7. If forking fails, do not switch. If switching is cancelled before
   replacement, remove only the new fork. Never remove the source session.
8. Use `withSession` for post-switch work. Report cwd, branch, and preserved
   history. For exit, run the reconstructed merge only after replacement exists.
   Report exact merge output and the re-entry command on failure.
9. Keep existing guard and advisory behavior unchanged.

**Proof**

Extend the real-extension Node harness to prove:

- public and private commands register;
- direct user commands and deferred agent CLI results reach the same switch path;
- the marker is removed from visible tool content and consumed once;
- a stale session id, replay, malformed marker, failed tool, or unknown schema
  cannot queue a switch;
- source and target cwd checks reject spoofed or stale intent;
- history entries and parent linkage survive `forkFrom`;
- no call changes `process.cwd()`;
- enter switches once and performs no merge;
- exit order is `settled → fork → switch → replacement context → merge`;
- wrong id, missing source, malformed JSON, CLI failure, tokenizer failure, fork
  failure, and switch cancellation keep the original session active;
- cancellation removes only the new fork;
- merge refusal after exit stays on main, keeps the worktree, and names re-entry;
- paths with spaces and non-ASCII text work;
- a user command during an active turn refuses before CLI or fork;
- all existing Pi guard contracts remain green.

### pwsr-3 — Map and drive the user-visible flow (`verification_docs`)

**Depends on** pwsr-2.

**Owns**

- `.bee/verify/verify-app/features/worktree-and-close.md`
- user-facing Pi/worktree documentation selected during execution

**Work**

1. Add direct user commands, automatic agent transitions, status messages,
   merge-refusal recovery, and old-Pi/old-bee compatibility to the feature map.
2. In a throwaway onboarded Git repository, drive real `worktree new`, `enter`,
   and linked `merge`. Prove verified roots and zero mutation on enter/exit.
3. Drive the Pi command bridge with the closest real Pi 0.85.1 surface available.
   If interactive TUI automation is absent, record that environment fact and use
   the real-extension Node contract as the explicit fallback. Do not label the
   fallback `green:live`.
4. Run the full declared Rust command after targeted proof is green.

**Proof**

- Updated feature-map commands and expected results.
- Fresh sandbox output attached to the cell report.
- `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`.

## Edge and failure matrix

| Dimension | Case | Required result |
|---|---|---|
| Creation | `worktree new` refuses or rolls back | No intent or marker. Pi stays in source session. |
| Creation | success | Enter intent appears only after all setup succeeds. |
| Existing enter | valid granted id from main | Enter intent; no Git or bee-state write. |
| Existing enter | missing, dead, or ungranted id | Typed refusal; no intent. |
| Merge ground | main | Existing queue, gate, merge, and cleanup behavior remains. |
| Merge ground | matching granted worktree | Exit intent; no queue, companion action, merge, cleanup, or store write. |
| Merge identity | supplied id differs from current | Typed refusal; no intent. |
| Tool result | marker on successful shell result | Remove marker, hold one intent, wait for settled. |
| Tool result | failed tool, wrong session id, replay, or malformed marker | No pending transition. Visible failure remains. |
| Intent | unknown version/operation/type, relative path, same source/target | Belt refusal before fork. |
| Session | source is in-memory or missing | Belt refusal before fork. |
| Fork | invalid source JSONL or unwritable target session directory | Show error; original session remains active. |
| Switch | `session_before_switch` cancels | Remove new fork; original session remains active. |
| Switch | target disappears before Pi opens it | Remove new fork when old context remains usable. |
| Switch | runtime rebuild fails after old teardown | Report Pi error. The extension cannot promise rollback after old-context invalidation. |
| Continuation | merge refuses after exit | Stay on main; keep worktree; show exact refusal and re-entry command. |
| Continuation | merge succeeds with cleanup | Session is already on main before worktree removal. |
| Paths | spaces, Unicode, Windows separators/drive case | Use native canonicalization and argv execution. |
| Concurrency | two transitions overlap | Serialize; refuse the second while one is live. |
| Compatibility | new bee with old belt | Additive JSON is ignored; normal text remains useful. |
| Compatibility | new belt with old bee | Missing intent gives a named version mismatch; no fork or switch. |

## Failure boundary, migration, and rollback

All extension-owned failures before `ctx.switchSession` leave the original
session active. Cancellation removes only the newly created fork. Pi's runtime
implementation invalidates the old context before target service construction
finishes. A failure in that later upstream window is not honestly rollback-safe;
the plan reduces it through canonical target checks and fully bootstrapped
worktrees, then reports it without a false recovery claim.

No bee store migration is needed. JSON gains an additive field. The two intended
CLI changes are a new zero-mutation `worktree enter` command and linked-worktree
`merge` returning transition success instead of a main-only error.

Each successful relocation creates a normal Pi fork. The source session remains
in history. Rollback removes the Pi commands, marker support, and additive CLI
fields. Existing Pi session files need no conversion.

## Advisor synthesis

The user-impact review found two blockers in the first draft. A user-only slash
route did not move sessions after an agent-created worktree. A refused merge had
no path back. This revision adds deferred command dispatch after
`agent_settled` and `worktree enter` for existing grants.

Accepted warnings add idle checks and visible status. The value review removed
self-hosting reload and duplicate TypeScript flag validation. It suggested a
plain exit mode; this plan does not add one because exit-without-merge is outside
the locked value, while re-entry covers the failure path.

The facts-gaps, alternatives, and risks seats were dropped because their prepared
runtime required an Agent tool unavailable in this session. The two available
seats completed. The leader checked source anchors, alternatives, failure
atomicity, replay, path, and concurrency risks during synthesis.

## Gate-2 acceptance

Approve this plan only if:

- supported Pi command context is the only replacement trigger;
- ordinary agent CLI commands can cause relocation after their turn settles;
- CLI owns worktree identity, targets, and typed merge continuation;
- belt validates each intent before it writes a fork;
- exit completes before any merge starts;
- merge refusal has a verified re-entry path;
- no failure is called rollback-safe after Pi invalidates the old runtime;
- mapped behavior and the full Rust suite are close proof.
