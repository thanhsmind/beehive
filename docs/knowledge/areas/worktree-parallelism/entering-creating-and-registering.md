---
type: bee.area
title: "Worktree Parallelism — entering: creating a feature worktree and registering it"
description: "The paved road that creates and grants a feature worktree in one move, the adoption command that registers a hand-made one, the fresh lifecycle state a bootstrap writes, a concurrency-aware refusal when the source checkout holds a shared nested checkout without a declared companion mount, and the typed zero-mutation refusals and best-effort rollback that guard all of it."
timestamp: 2026-08-05
bee:
  id: worktree-parallelism-entering-creating-and-registering
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/the-trust-model.md]
  decisions: ["worktree-session-routing D7 (worktree new is the paved road for STARTING a feature worktree, GH #21)", worktree-feature-parallelism (register/list/unregister and the bootstrap contract), I46 (issues-46-53 — immutable creation slug), worktree-concurrency-guard D1(a)/D3/D4/D6 (docs/history/worktree-concurrency-guard/CONTEXT.md; supersession 0ccc1cf3), worktree-reclaim D3 (a teardown either finishes or does not start — unregister drops the workspace record in the same call it drops the grant), worktree-reclaim D3a (the shared teardown helper takes directory removal as an explicit parameter; unregister never reaches it), "c47fa930 (pi-worktree-session-relocation D1 — active-session replacement, no cwd mutation)", "1f947417 (pi-worktree-session-relocation D2 — forkFrom and switchSession)", "85d85ede (pi-worktree-session-relocation D3 — CLI transition intent and Pi harness action)", "0b4ff55b (pi-worktree-session-relocation D4 — enter after creation, exit before merge, no exit deletion)", "538c723d (pi-worktree-session-relocation D5 — zero-mutation worktree enter command for existing grant)", "113e9f2c (pi-worktree-session-relocation D6 — Pi queues relocation until agent_settled)"]
  sources: [docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-registering-a-worktree-the-cli", "docs/specs/worktree-parallelism.md#S-entering-worktree-new-feature-slug-d7-gh-21", "issues-46-53 cell i-2 (GH #46 — the creation slug is recorded immutably because the feature name is not; the refusal names the drifted field instead of the branch; trace in .bee/cells/, 2026-07-23)", "worktree-concurrency-guard cell wcg-3 (capped trace and report, 2026-07-24 — worktree-new concurrency-aware refusal)", "worktree-concurrency-guard cell wcg-fix-1 (capped trace and report, 2026-07-26 — acting-session self-exclusion fix, review finding #1)", "docs/history/worktree-reclaim/CONTEXT.md and plan.md (D3, D3a, wr-1); commit 430ec952 (lift teardown into one helper; unregister drops its workspace record); packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs (run_unregister), merge.rs (teardown_worktree)", "docs/history/pi-worktree-session-relocation/CONTEXT.md and plan.md"]
  authoritative_for: "worktree-parallelism: creating, granting, bootstrapping, and entering a feature worktree"
---

# Worktree Parallelism — Entering: Creating, Registering, and Relocating

Three ways in: `register` adopts a worktree that already exists; `new` creates one, grants it,
and bootstraps it in the same move; `enter` provides the zero-mutation entry and recovery path
for an existing verified grant. Successful creation and entry both emit structured session-transition
intent to relocate the active session without mutating process cwd (c47fa930, 85d85ede, 538c723d).

## Registering a worktree (the CLI)

- `worktree register --feature <slug>` — run from inside a linked-valid worktree. Writes the
  grant into the main store's registry (keyed by the git-verified id) and **bootstraps** the
  worktree's own store: copies the main store's onboarding + config, writes a FRESH lifecycle
  state (the named feature, phase idle, all gates unapproved). An independent-feature worktree
  runs its OWN feature, so it inherits none of main's state/gates/log.
- `worktree list` also names which granted worktrees are already merged and only waiting for
  cleanup (worktree-keep-on-merge D1): an id whose root still has a pending `worktree-cleanup`
  entry in the queue ledger renders `(granted, merged — pending cleanup)` (JSON: a
  `merged_pending` map), a plain grant stays `(granted)` — so the list answers "what is still on
  disk only for cross-checking" without opening the ledger.
- `worktree list` / `worktree unregister [--id <id>]` — read the grants in the main store; unregister drops
  the grant AND the workspace record in the same call (worktree-reclaim D3), never one without the
  other, through the same shared teardown helper `worktree merge` cleanup and `worktree prune` call
  — see `returning-and-the-merge-gate.md` and `pruning-dead-worktrees.md`. `unregister` reaches only
  that registry half: it never removes the worktree's directory or its branch, because it cannot know
  whether the caller still wants those — a partial teardown that removed a grant but left the
  workspace record behind was exactly the bug this closes: 13 orphan records with no matching grant,
  invisible to a grant-driven scan.

## Entering: `worktree new --feature <slug>` (D7, GH #21)

The paved road for STARTING a feature worktree — create and register in one move, run from
the ordinary main checkout:

- Creates the sibling `../<repo-basename>--wt--<slug>` on branch `wt/<slug>` (optional
  `--base-ref`, resolved as a commit-ish via `git rev-parse --verify --end-of-options
  "<ref>^{commit}"` — accepts HEAD, HEAD~1, short shas, tag^{commit}; the RESOLVED sha is
  what the worktree is created from, and anything unresolvable is one typed
  `WORKTREE_BASE_NOT_FOUND` refusal, the old separate invalid-syntax code retired), then
  grants + bootstraps exactly as `register` does. The grant id is read back from the worktree's git metadata after creation,
  never assumed from the directory name. Output names the created path, emits structured
  `sessionTransition` metadata (operation `enter-worktree`), and emits the `@@BEE_SESSION_TRANSITION@@`
  marker on stderr when `PI_SESSION_ID` is present (85d85ede). In non-relocating environments,
  it instructs the human to open their next session there; in supported harnesses (Pi), the belt
  relocates the active session automatically (c47fa930, 113e9f2c).
- Slug allowlist `^[a-z0-9][a-z0-9-]*$`; every git call is an argv array (no shell), `--`
  before user-derived values.
- **A worktree carries no runtime of its own, and must not need one.** A linked worktree
  materialises TRACKED files only, and the vendored binary is deliberately untracked
  (machine-local, decision 1f4262ca) — so a worktree's `.bee/bin/` holds prompts and nothing
  else, while its `.claude/settings.json` (tracked) wires all eleven hooks at a path that is
  not there. Every hook then took its visible-fail-open arm and exited 0: no gate enforcement,
  no reservation check, no tier guard, no session preamble. Silent, and unreportable from the
  inside — the component that would have raised the alarm is the one that is missing. Worse,
  the worktree-first routing rule sends code-touching work *into* that state by design.
  The wiring therefore resolves the binary at HOOK time, not at onboard time, and falls back
  to the MAIN checkout via `git rev-parse --path-format=absolute --git-common-dir` — the one
  question git answers usefully from both sides (`--show-toplevel` returns the worktree
  itself). The probe runs only after the direct candidates miss, so an ordinary checkout
  spawns no extra process, and every worktree runs the same binary the main checkout does —
  never a stale copy. Found and closed 2026-08-03.
  Since 2026-08-21 the bootstrap also provisions the binary itself: creating a worktree puts
  `<worktree>/.bee/bin/bee` in place from the main store's own binary — a symlink where the
  platform allows one, so a rebuilt main binary is instantly live in every worktree and can
  never read stale, and a mode-preserving copy where it does not. A destination that already
  exists is left alone, checked at the link level so a stale-target link is never written
  *through* into the main store. A missing source or a refused link never fails the bootstrap;
  the outcome rides the report. So a worktree's `.bee/bin/` now holds prompts AND the binary,
  and the hook-time resolution above remains the belt to that suspenders — the fix closed the
  case an agent hits directly, invoking `.bee/bin/bee` by the path AGENTS.md names
  (store-reach-gaps D2, 2026-08-21).
- **The creation slug is recorded immutably, because the feature name is not.** Directory, branch
  and the worktree's own feature field all derive from one slug at creation — so at that instant
  they agree by construction. The feature field is then freely rewritten afterwards by ordinary
  state handlers that have no worktree awareness at all, and the paved road makes a rename
  near-certain: the worktree-first routing rule (docs/specs/worktree-first.md) tells the agent to
  create the worktree at **feature start**, often before exploring has settled what the feature is
  actually called. Bootstrapping therefore writes
  the creation slug once, write-if-absent, to a record the return path reads **in preference to**
  the mutable field.

  Two properties matter. The record lives on an already-ignored runtime path, deliberately: a
  tracked one would make every worktree read *dirty* to the return path's own uncommitted-work
  pre-check — the same trap that forces the companion-session record to be torn down first. And a
  worktree created before this record existed behaves exactly as it always did: absence degrades
  to reading the mutable field, never to a crash and never to a new refusal.

  Without the record, the return path derived its expected branch from the drifted field and
  refused by naming the **branch** — which is correct, fixed at creation, and the one thing the
  operator must not change. A refusal that names the only unchangeable thing is a dead end, so the
  refusal now names the field that actually drifted and says outright not to rename the branch.
- **A live concurrent session and an undeclared shared nested checkout together refuse the
  creation.** When another session's heartbeat is live for the source checkout, and that
  checkout holds a nested checkout another session could also reach, `worktree new` refuses
  — typed, zero-mutation, no override — unless the call declares its own companion mount for
  that nested checkout. The refusal names declaring a companion mount as the fix. With no other
  session live, or with no such nested checkout present in the source checkout, creation
  proceeds exactly as it always has; declaring a companion mount is likewise never refused by
  this check regardless of concurrency, since a declared mount is itself the fix.
  The concurrency check excludes the acting session's own heartbeat when deciding whether
  another session is live — a solo agent whose own session record is the only one present is
  never mistaken for its own concurrent peer.
- Every refusal is **typed and zero-mutation**: invalid slug/base-ref, caller not an ordinary
  checkout, target path / branch / grant already exists, the live-concurrent-and-undeclared-
  shared-checkout condition above, and git's own `worktree add` failure (the pre-checks are
  advisory; git's atomic failure is authoritative). A failure AFTER the worktree was created
  rolls back best-effort (worktree, branch, grant) and reports typed; if even rollback fails,
  the error names `worktree register` as the adoption path.
- `register` remains for adopting a hand-made worktree; `new` is the paved road.

## Entering an existing worktree: `worktree enter --id <id>` (538c723d)

The zero-mutation paved road to enter an existing granted worktree — run from the ordinary MAIN checkout:

- **Main-only and zero-mutation:** Requires cwd to be an ordinary main checkout; refuses if run inside any linked worktree (`WORKTREE_ENTER_FROM_WORKTREE`). Verifies the git worktree link and strict grant in the main store registry. Performs no git commits, writes no `.bee/` state, and modifies neither branch nor directory.
- **Emits enter transition intent:** Emits identical `sessionTransition` metadata (`operation: "enter-worktree"`, canonical `sourceCwd: main_root`, `targetCwd: worktreeRoot`, `worktreeId: <id>`, `feature: <slug>`, `piSessionId`, `continuation: null`) on stdout and the `@@BEE_SESSION_TRANSITION@@` stderr marker when `PI_SESSION_ID` is present (85d85ede).
- **Recovery and re-entry:** Serves as the canonical entry path when returning to an in-flight worktree or re-entering after a refused merge on main (in Pi, the post-exit merge notification explicitly names `/bee-worktree-enter --id <id>` as the re-entry recovery command; CLI merge refuses typed while keeping the worktree granted).

## Pi worktree navigation and active session replacement (c47fa930, 1f947417, 113e9f2c)

Pi binds tools, resources, trust, and persistence to session cwd. Mutating process cwd would corrupt these bindings (c47fa930). Relocation replaces the active session instead:

- **Supported harness replacement:** Uses Pi 0.85.1's supported `SessionManager.forkFrom(sourceSessionFile, targetCwd)` and `ctx.switchSession(forkFile, { withSession })` sequence (1f947417). Conversation history and parent linkage are preserved across project directories without touching process cwd.
- **Direct user commands:** `/bee-worktree-new --feature <slug>` and `/bee-worktree-enter --id <id>` run in `ExtensionCommandContext` when idle, invoke the bee CLI without a shell via `execFile`, validate the emitted `sessionTransition`, fork the session, and switch immediately.
- **Automatic agent CLI relocation:** When an agent runs `bee worktree new` or `bee worktree enter` through `bash` or `powershell`, the Pi belt intercepts `@@BEE_SESSION_TRANSITION@@` on stderr, strips the marker from visible tool results to keep the prompt clean, and holds the intent. Because event contexts (`ExtensionContext`) cannot switch sessions, the belt waits for `agent_settled` and then submits private command `/bee-worktree-relocate <token>` via `pi.sendUserMessage(..., { expandPromptTemplates: true })` (113e9f2c). This executes with `ExtensionCommandContext` without opening a new model prompt turn.
- **Failure boundary and rollback honesty:** Failure handling operates across three distinct phases: (1) before fork, pre-switch validation failures (idle check, session ID mismatch, schema check, operation check, non-canonical path, non-existent target directory, unpersisted session, or missing/empty source session file) fail closed before any fork exists, leaving the source session active with zero fork files created or deleted; (2) fork exists plus cancellation or pre-teardown switch failure, where `SessionManager.forkFrom` succeeded in creating a forked session file, but `ctx.switchSession` fails before teardown begins (`!transitionTeardownOccurred`) or the switch is cancelled, explicitly deleting the forked session file (`rmSync`) and cleanly preserving the source session without leaving orphaned session artifacts; (3) post-teardown failure, where once `ctx.switchSession` begins upstream teardown of the old runtime context (`transitionTeardownOccurred`), subsequent failures during target service reconstruction cannot promise rollback, and the extension logs the Pi error honestly without deleting the fork or claiming impossible rollback safety.
- **Compatibility:** Old Pi belt versions ignore the additive `sessionTransition` JSON payload; the new Pi belt warns of a version mismatch and refuses to fork or switch if `worktree new` or `enter` output lacks session transition metadata.
