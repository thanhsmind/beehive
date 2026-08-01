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

#### Coverage debts R6 must close (a delegated path is fine until Node is deleted)

Tracked here because "the verb is ported" is not the same as "every repo shape runs native".
Each entry is a branch that currently returns to Node and therefore blocks deleting `bee.mjs`:

- **The lane/workflow world.** `state set/gate/scribing-run/plan-rev bump/handoff` are native only
  when the repo has no `--lane` selector, no lane-bound session, and zero records under
  `.bee/runtime/workflows/`. A repo using lanes or workflows still runs Node for those verbs —
  the projection write-through, workflow locks, and handoff mailboxes are unported.
- **Whole verbs still on Node:** `state start-feature|route|workflows.*|rebuild-projections|
  advisor-ref.*|compact-*`, `decisions supersede|render`, `knowledge promote`,
  `backlog rank|badges|render`, `feedback digest|collect|rank`, plus every `--stdin` shape
  (a probe must decide before consuming the pipe, so stdin can never be validated natively first).
- **Cross-cutting delegate classes:** corrupt-JSON reads whose warning embeds a V8 message;
  collation over free prose (`localeCompare` on titles); `session-init`'s preamble and
  `session-close`'s PreCompact branch.
- **Linked worktrees — classification done, routing deliberately NOT flipped.** `roots.rs` now
  carries both arms of `resolveRootsCore` (gitdir read, namespace shape, bidirectional
  back-pointer, the four `WorktreeLinkInvalidError` messages, grant lookup), pinned against a
  Node harness over real `git worktree add` fixtures. But `resolve_store_root` still answers
  `NeedsNode` for a linked worktree ON PURPOSE: every verb ported so far encodes the invariant
  "the native path only ever holds an ordinary classification" — `status_full.rs` hardcodes
  `worktree_notice: None` and ports only the ordinary half of `orientWorktreeContext`,
  `reservations.rs` treats `resolveMainRoot`/`resolveHoldTopology` as constants, and several
  ports assume `controlRoot == root`. Flipping the mapping was tried and measured: inside a
  granted worktree `orient --json` lost its whole `worktree` block, and inside an ungranted one
  `status --json` lost `worktree_notice`. That is a C2 break, not a coverage win. The flip is
  therefore per-verb: a verb opts in by calling `resolve_roots_core` once its own
  worktree-sensitive branches are ported. `verbs/worktree.rs` is the first such caller.

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
