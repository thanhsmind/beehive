# Rust port — bee platform, Node → Rust (fluent-class single binary)

Status: APPROVED 2026-08-01 — strangler strategy, hooks-first ordering, R6 distribution =
host-side `cargo build` (no prebuilt binaries in repo). Campaign in progress from R0.
Reference: fluent (D:\projects\tools\AI\harness\fluent) — structural reference only: single
binary, clap CLI, serde state, assert_cmd black-box tests, hooks as subcommands. No text copied.

## Goal

One native `bee` binary replaces the entire Node runtime surface:

- `bee.mjs` unified dispatcher + 9 thin shims → clap subcommand tree (same 16-verb porcelain,
  docs/specs/porcelain.md unchanged).
- 10 runtime hooks (`node .bee/hooks/*.mjs`) → `bee hook <name>` subcommands; claude-hooks.json
  rewired to call the binary. Hook stdin/stdout contract unchanged.
- statusline → `bee statusline`.
- scripts (onboard, render trees, prompts renderer, verify_all) → `bee` maintenance subcommands
  or a second dev-only binary in the same cargo workspace.

Hosts no longer need Node at all. Vendored frame ships `.bee/bin/bee(.exe)` instead of mjs shims.

## Why (measured, not vibes)

1. **Hook latency.** PreToolUse hooks fire on Edit|Write|Bash|Read|Glob|Grep|AskUserQuestion —
   i.e. nearly every tool call. Each firing pays Node cold-start (~50–120 ms on Windows).
   A Rust binary starts in ~1–5 ms. This is the single largest daily UX win.
2. **Distribution fit.** Decision 1f4262ca chose source-checkout per-project install, no
   registry. A single static binary is the cleanest possible form of that decision: no Node
   version drift on hosts, no ESM path bugs (the d:-drive ESM bug class disappears).
3. **Known win-32 defect class dies.** The two filed chips (onboard entryIdentity Number-precision
   inode collision → u64/u128 native; encodeProjectDir drive-colon) are Node-shaped bugs; the
   port fixes both structurally.
4. **Fluent parity.** Same architecture language across both harness tools; expertise/ guidance
   transfers.

## Inventory (measured 2026-08-01)

| Area | LOC | Notes |
|---|---|---|
| lib/ (30 modules) | 29,803 | kernel: state 3.3k, cells 3.0k, command-registry 2.4k, guards 2.1k, worktree-store 2.1k, knowledge 2.0k |
| scripts/ | 10,426 | onboard, render_plugin_skill_trees, verify_all, prompts renderer |
| bee.mjs + shims + hooks + statusline + agents | ~5k | |
| **Product total** | **~45–50k** | |
| tests/ (37 files) | 37,403 | 34/37 import lib directly (white-box); 12 spawn subprocess |

Runtime dependencies: **zero npm packages.** Node builtins only:
path(33) fs(32) crypto(14) child_process(6) url(3) os(2)
→ Rust: std::{fs,path,process}, sha2, serde/serde_json, clap, anyhow. No async runtime needed.

## Strategy: strangler, not big-bang (recommended)

A big-bang rewrite of 50k LOC parks the product for weeks with nothing shippable. Instead:
the Rust binary grows verb-by-verb behind a dispatcher that falls back to `node bee.mjs`
for unported verbs. Node and Rust interleave against the same `.bee/` state throughout.
Precedent: DB2 already proved byte-identical-output migration works for this codebase.

### Compatibility contracts (hold for the entire campaign)

- **C1** `.bee/` state files stay format-identical — either runtime can read/write mid-campaign.
- **C2** `--json` output byte-identical to Node (diff-harness enforced); human output identical
  unless an improvement is deliberate and logged.
- **C3** hooks.json is the only wiring change; hook payload contract untouched.
- **C4** prompts renderer byte-identity pin (packages/bee/prompts/*.md) survives the port.
- **C5** write-guard / worktree-first semantics (guards.mjs, docs/specs/worktree-first.md)
  ported with paired tests before the Node guard is retired — never a window with no guard.

### Progress log (kept current; commits are the authority)

- **R0 DONE** `9932774b` — workspace, front door, diff harness 19/19.
- **R1 DONE** `7736d7ed` `793888c1` `72c5399e` — fsutil/jsjson/roots/state/registry/path_identity/
  lock; first native verb (`status --brief`). Design call: `.bee/` files are read as
  order-preserving `serde_json::Value`s with JS-spread-parity merging, NOT rigid structs —
  structs would reorder or drop unknown keys and break C2.
- **R2 DONE** `8939b151` `9361a7de` `f58b9304` — all nine hooks native under `bee hook <name>`
  (hook diff harness 60/60); onboard's repo-hook renderer feature-detects a vendored
  `.bee/bin/bee[.exe]` per target repo, so hosts without the binary keep node wiring
  byte-identical. write-guard embeds the 26-file vendored-lib closure and byte-compares it at
  runtime: lib skew ⇒ delegate, so a guard decision can never flip on a skewed host.
- **R3 IN PROGRESS** `8fd3e21e` `bd0372d6` (wave 1: 31 surfaces) + wave 2 landing —
  status(full)/orient, cells read+mutating, reservations, decisions, capture, backlog, feedback,
  intent/reviews/knowledge/tmp, state group, help surfaces, `bee test`.

Two campaign rules that emerged from practice and now govern every port:

1. **Conservative argv routing.** A verb serves ONLY argv shapes proven equivalent; everything
   else returns before any output and Node handles it. This means the dispatcher's
   error/validate/nearest-match machinery never has to be reproduced.
2. **Refusals delegate, unless their bytes are deterministic.** A refusal whose text embeds a V8
   message goes back to Node; a typed refusal (lock-busy, gate, dep-uncapped) is reproduced
   natively — with the one exception that a refusal reached AFTER a lock attempt must be native,
   or delegation would double the contention telemetry.

### Phases

- **R0 — scaffold + diff harness.** Cargo workspace `packages/bee-rs/` (binary name `bee`).
  clap skeleton mirroring the porcelain tree. `xtask diff` harness: run Node and Rust on the
  same fixture repo, byte-diff stdout/stderr/state. CI: build + test on win32 (primary) and
  linux/mac (fluent hosts).
- **R1 — state kernel.** path-identity, lock, atomic-write, state read model, serde types for
  every `.bee/` file. Fix the two filed chips here (bigint inode → u64/u128; drive-colon encode).
  This is the foundation everything imports.
- **R2 — hooks (payoff first).** Port all 10 hooks to `bee hook <name>`; rewire
  claude-hooks.json. Hooks are small, latency-critical, and already have contract tests
  (test_hook_contracts, test_write_guard, test_model_guard) to port as the first Rust test suite.
  After R2 the daily latency win is fully banked even though most verbs are still Node.
- **R3 — porcelain verbs, dependency order.** status/orient → cells → reservations/claims →
  dispatch (--claim prompt assembly incl. Learned-context block) → decisions/capture →
  reviews/feedback/knowledge → finish/cap/close + `bee test` runner (keeps
  .bee/logs/test-results.json contract and POSIX-sh execution). Each verb: port, diff-harness
  green, flip dispatcher routing, port its white-box tests to Rust unit tests.
- **R4 — dev surface.** onboard (vendoring, render), render_plugin_skill_trees, prompts
  renderer (C4 pin re-verified byte-for-byte), verify_all → `cargo test` + thin driver.
- **R5 — test completion.** Remaining white-box tests ported per-module (they land alongside
  R3 flips, this phase is the sweep); subprocess-style tests become assert_cmd black-box tests.
  Target: green suite with Node deleted, including the 6 formerly env-limited suites — Rust
  removes the ESM/PATH/symlink excuses (EPERM atomic-write on win32 stays capability-skipped).
- **R6 — cutover.** Delete Node runtime + shims; `.bee/bin/` ships the binary; INSTALL.md gains
  the per-platform build step (`cargo build --release`, matching source-checkout distribution);
  onboard re-render; docs sweep; memory updated.
- **R6a — skills and expertise move onto the Rust system** (owner requirement, 2026-08-01). The
  port is not finished while the instruction layer still teaches the Node runtime, and it has
  two halves:
  - **Content.** Every agent-facing command spelling must name the binary, not a `.mjs` script:
    `skills/*/SKILL.md` and their `references/`, `expertise/*.md` + `INDEX`, `AGENTS.md`,
    `CLAUDE.md`, and the live docs. Measured at the start of R6a: 49 `node …*.mjs`
    invocations across skills/expertise/AGENTS.md, concentrated in
    `bee-hive/references/routing-and-contracts.md` (16), `bee-swarming/references/
    swarming-reference.md` (15), and the `bee-herding` reference set. Note `skills/bee-herding/
    scripts/*.mjs` and `control-loop.sh` are executable helpers, not prose — they are ported or
    rewritten, not merely reworded.
  - **Machinery.** The skill-tree render, the `.claude/skills` projection, and the
    `.bee/expertise` vendoring must be produced by the Rust binary (the onboard and render
    ports), never by a Node script. After the flip, re-render every projection and prove the
    vendored trees byte-identical to the intended output.

  Sequencing: R6a runs AFTER the CLI surface settles, because every spelling it writes must
  match the final binary invocation.

#### The two filed win32 defects — one fixed, one still live

- **Inode precision — FIXED in the port.** `entryIdentity` keyed a map on
  `` `${st.dev}:${st.ino}` `` built from JS Numbers. Measured on this NTFS volume over the
  beehive tree: **29 of 3544 entries (0.8%) have their file index silently rounded**, and two
  indices actually present on the volume collide once rounded (…919 and …920 both become …920).
  A collision makes `detectAliasCollisions` block two unrelated skills as a case-insensitive
  alias that does not exist — or hide a real one. The Rust port keeps the volume serial as
  `u64` and the file index as `u128`. No output byte changes: the value was only ever a map key.
- **`encodeProjectDir` drive colon — STILL LIVE, fix at cutover.** It is not in onboard; it
  lives in `packages/bee/lib/perf.mjs` and is faithfully replicated in `verbs/status_full.rs`
  and `hooks/session_close.rs` (C2 demanded that). Confirmed live: Node encodes
  `D:-projects-…`, and `mkdir` of that transcript directory fails EINVAL — the layout it names
  cannot exist on NTFS — while the correct `D--projects-…` (what Claude Code itself writes)
  succeeds. So every transcript-dependent path (recovery scan, perf rollup) is unreachable on
  win32 for BOTH runtimes. Fixing it means diverging from Node, so it belongs at R6 when there
  is no Node left to match: change all three sites together, and unpin the tests that currently
  assert the broken spelling — including `test_bee_cli.mjs`'s `RECOVERY_LAYOUT_UNREPRESENTABLE`
  skip, which exists solely because of this bug.

#### Hard blockers for deleting the Node runtime (found by R5, 2026-08-01)

The test migration turned up gaps that are missing CODE, not missing tests. Deleting `bee.mjs`
before these are closed does not "lose coverage" — it loses behavior:

- **Seven contracts have no Rust implementation at all**: `createWorkflow`, `renewLease` /
  `LEASE_MISSING`, `LEASE_FENCE_STALE`, `CLAIM_FENCE_STALE`, `adoptClaim`, multi-resource batch
  acquire, and homoglyph folding of an authority subject. Each is named at its site in the Rust
  tree rather than faked green.
- **Two behaviors were pinned as deliberate delegations, so they had no native owner once
  `bee.mjs` is gone**: write-guard's CLI-shape check (d), and `listWorkflows`' skip tolerance.
  These were the sharpest blockers — the delegation WAS the implementation. **Both are now
  native (2026-08-01):**
  - **Check (d)** lives in `crates/bee/src/hooks/cli_shape.rs`: the argv tokenizer feeds
    `splitCliSegments`, longest-prefix command resolution runs against the embedded
    `registry::REGISTRY_PAYLOAD` (the bytes the freshness test already pins), and
    `validate-args.mjs`'s `isValidParameterSchema`/`typeMatches`/`isPresent`/`validate` are
    ported whole, including the ce-1 batched-`problems` rendering. Never bails: every arm is
    deterministic. Proven by a 34-row byte-diff (stdout/stderr/exit) against the `.mjs` hook
    with `BEE_HOOK_NO_DELEGATE=1`, all native, zero mismatches.
    **One deliberate widening:** the guard now also recognizes the R6a BINARY spelling
    (`.bee/bin/bee <verb>`, `bee <verb>` in command position, `bee.exe`), which no `.mjs`
    regex could see — so a malformed binary-spelled call is denied with the same bytes the
    `.mjs` spelling always produced. The divergence runs in one direction only (Node-allow →
    Rust-deny); a 14-row harness proves no denial ever becomes an allow, and
    `cli_shape.rs::documented_invocations` fails the build if any SHIPPED command spelling in
    `skills/`, `expertise/`, `docs/` or the root instruction files would be refused.
  - **`listWorkflows` skip tolerance** is reproduced in `verbs/workflow_store.rs`: the three
    ordinary skips (missing record, not-a-JSON-object, id mismatch) warn with Node's exact
    `listWorkflows: skipping unreadable workflow "<id>" — <reason>` bytes, and the repeat
    count needs no modelling because the Rust call graph calls `list_workflows` from the same
    places `bee.mjs` does. Measured node-vs-rust warn counts over corrupt fixtures: 1/1, 3/3,
    5/5, 15/15, 25/25. **Residue — two arms only:** a reason embedding a V8 `JSON.parse`
    message, and the non-ENOENT read failure whose reason embeds a libuv errno string. Both
    are decided in a PRE-PASS, so a delegating run still emits zero bytes first (verified:
    a delegating run's stderr carries only the tripwire line).
- **Two laws go vacuously green at deletion.** `test_instruction_size_law` and
  `test_scan_set_hygiene` scan `scripts/**` and `packages/bee/**`; once those trees are gone the
  laws pass because their subject vanished, not because the law holds. Re-point or retire them
  deliberately — a green check over an empty set is worse than no check.
- **`scribingTarget` (~35 contracts) has no plan entry anywhere.** Confirm whether it is live
  surface before porting it; if it is dead, say so and delete it rather than porting by inertia.

#### Coverage debts R6 must close (a delegated path is fine until Node is deleted)

Tracked here because "the verb is ported" is not the same as "every repo shape runs native".
Each entry is a branch that currently returns to Node and therefore blocks deleting `bee.mjs`:

- **The lane/workflow world.** `state set/gate/scribing-run/plan-rev bump/handoff` are native only
  when the repo has no `--lane` selector, no lane-bound session, and zero records under
  `.bee/runtime/workflows/`. A repo using lanes or workflows still runs Node for those verbs —
  the projection write-through, workflow locks, and handoff mailboxes are unported.
- **Whole verbs still on Node** (list rewritten 2026-08-01 after the debt-closing wave — the
  earlier entries for claim-next, rebuild-projections, route, decisions supersede/render,
  backlog rank/badges/render, feedback digest/collect/rank and knowledge promote are all now
  native):
  - `state start-feature` — its default path calls `applyWritePolicy` with
    `enforceIsolation: true`, and `registerWorkspace`/`attachWorkspace` **write** into the
    unported `workspace-store.mjs` before the decision is made; the consented branch then runs
    a real `git worktree add` to produce `redirect`. Nothing after that first write can fall
    back, so the residue is exactly the write-policy half.
  - `state advisor-ref.*`, `state compact-*`.
  - `worktree new|merge` — not a store-shape problem: every observable byte on the happy path
    comes from a child process (`git worktree add`, `git merge --no-ff`, surfaced verbatim in
    both text and `--json`), the failure arms embed Node's `spawnSync` error shape and are
    reached *after* the worktree or merge commit exists, `createFeatureWorktree`'s rollback
    ladder is order-sensitive (a different unwind order leaves a different tree — a C1 breach),
    and `merge` holds a processor lease over the integration queue while running the host
    verify, so lease + queue + verify + teardown must land in one piece.
  - `dispatch prepare --claim` — its claim and reserve doors are private to cells.rs and
    reservations.rs; the assembled prompt (the product) is already covered by the non-claim
    shape's twin diff.
  - Every `--stdin` shape: a probe must decide before consuming the pipe, so stdin can never be
    validated natively first.
- **Cross-cutting delegate classes:** corrupt-JSON reads whose warning embeds a V8 message;
  collation over free prose (`localeCompare` on titles); `session-init`'s preamble and
  `session-close`'s PreCompact branch.
- **Linked worktrees — classification done, routing flipped PER VERB.** `roots.rs` carries both
  arms of `resolveRootsCore` (gitdir read, namespace shape, bidirectional back-pointer, the four
  `WorktreeLinkInvalidError` messages, grant lookup), pinned against a Node harness over real
  `git worktree add` fixtures. The flip is per-verb, never blanket: an early attempt to widen
  `resolve_store_root` wholesale was measured and cost `orient --json` its `worktree` block
  inside a granted worktree and `status --json` its `worktree_notice` inside an ungranted one —
  a C2 break, not a coverage win. A verb opts in via `resolve_store_root_worktree` only once its
  own worktree-sensitive branches are ported.
  - **Worktree-native now:** `worktree list|register|unregister`; `status` / `status
    --lanes-full` / `orient` (ungrantedWorktreeNotice, BOTH halves of `orientWorktreeContext`
    incl. `readWorktreeBranch`, and the real `controlRootFor` so sessions/claims/workers/lanes
    resolve onto mainRoot); `reservations list|reserve|release|sweep` (the real
    `resolveMainRoot`/`resolveHoldTopology` — the ledger is addressed at mainRoot, the holder is
    the git-verified worktree id when granted, and the cross-worktree section is skipped
    entirely when ungranted). Proven by twin-fixture byte-diff from inside a granted AND an
    ungranted worktree with `BEE_JS_ENTRY` sabotaged, stdout/stderr/exit AND the resulting
    `.bee/` trees, including a cross-worktree `FOREIGN_HOLD` refusal.
  - **Still delegated inside a worktree:** `status --brief` (separate module, no
    worktree-sensitive branch but undiffed there), `cells *`, `decisions *`, `dispatch prepare`,
    `close`, `capture *`, `backlog *`, `feedback *`, `knowledge *`, `intent *`, `reviews *`,
    `state *`, `tmp sweep`, `test`, `--help`. cells.mjs re-roots claims through `controlRootFor`
    and that branch is unported; the rest read the control plane as if it were `root`.
  - **Delegated for everyone:** a BROKEN link (`WorktreeLinkInvalidError`) — Node's throw
    escapes main()'s `recordTiming` try-block, so reproducing it means bypassing the shared
    timing wrapper — and the `Exotic` V8-worded ENOENT.

### Sizing (honest)

Fluent is 78k LOC Rust for a comparable tool. Expect the finished port in the same class.
This is a multi-week, multi-session campaign — R0+R1 ≈ 2–3 sessions, R2 ≈ 2, R3 is the long
middle (≈ 8–12, one session per verb-cluster), R4–R6 ≈ 4–6. The strangler shape means every
session ends shippable.

## Risks

- **Test debt is the real cost center** (37k LOC white-box). Mitigation: port tests with their
  module (R3 rule), never as a detached backlog.
- **guards.mjs subtlety** (2.1k lines of path/worktree edge cases). Mitigation: C5 — paired
  tests first, longest diff-harness soak of any module.
- **Two-runtime window confusion.** Mitigation: dispatcher owns routing; `bee --version` prints
  which runtime served the verb; diff harness runs in CI until R6.
- **Windows Git Bash dependency for `bee test`** (POSIX sh runner) — unchanged by the port;
  explicitly out of scope. NOTE (found during the R3 port): a bare `bash` spawn on Windows
  resolves through System32 first and launches **WSL bash (Linux)**, not Git Bash. The Rust
  runner passes an explicit PATH to the child so resolution is PATH-first like libuv's.

## Open decisions for owner

1. Approve strangler strategy (vs big-bang)?
2. R2-before-R3 ordering (hooks first for latency payoff) — confirm.
3. Binary distribution at R6: commit prebuilt binaries per platform into the frame, or
   require `cargo` on hosts (fits source-checkout decision 1f4262ca)? Recommendation: require
   cargo; prebuilt binaries in-repo bloat history.
