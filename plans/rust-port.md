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

#### Cutover status (2026-08-01, after the debt-closing waves)

Every hard blocker below is CLOSED: the seven missing contracts exist and are pinned against a
live Node oracle; write-guard check (d) and `listWorkflows` tolerance are native; the two
vacuous laws now fail on an empty scan set; `worktree merge` and `state start-feature` — the
last delegated verbs — are native, with all of their recorded blockers dissolved rather than
waived. 846 Rust tests; CLI harness 19/19; hook harness 60/60.

**What still reaches Node.** These are narrow arms, not verbs. Each must be either ported or
consciously accepted as a behavior change before `bee.mjs` is deleted — a delegation that has
nowhere to delegate is not a fallback, it is a crash:

- **Companion worktrees** — `worktree new --with-companion` and `merge` on a companion
  worktree. Both tear down a mount before staging and cannot fall back once it is gone.
- **`--queue-wait-ms`** on `worktree merge` (only the default 180 000 ms bound is native).
- **A second live session** during `worktree new` — wcg-3's shared-nested-checkout scan has no
  Rust counterpart, and its helper is private to the write guard; re-deriving it forks the
  guard, which is the drift C5 exists to prevent.
- **`WorktreeLinkInvalidError`** anywhere — Node's throw escapes its own timing wrapper, so the
  shared wrapper cannot reproduce it.
- **Every `--stdin` shape** — a probe must decide before consuming the pipe.
- **Corrupt-JSON reads** whose warning embeds a V8 message, across every verb.
- **Two deliberate divergences scheduled FOR the cutover, not before it**: the session
  preamble's own `node .bee/bin/bee.mjs status --json` string (emitted by `lib/inject.mjs`,
  byte-pinned by a golden file) and the `encodeProjectDir` drive-colon bug. Both mean diverging
  from Node, which is only legal once Node is gone.
- **`test_agents_budget.mjs`'s meaning guards** — the one instruction-law invariant whose
  subject is another Node suite; it needs a new home, not a migration.

#### The three wiring surfaces that still named Node — REWIRED 2026-08-01

The last surfaces that would have broken the moment the `.mjs` tree is deleted. All three now
prefer the binary and keep Node only as a fallback arm, so a host that has not built the binary
yet is byte-unchanged:

- **Plugin hook manifests** (`packages/bee/hooks/claude-hooks.json`, `hooks/hooks.json`) —
  rendered from `hooks/catalog.mjs` (the only authority; `test_hook_contracts.mjs` drift-checks
  all three files byte-for-byte). Each command is now
  `for b in "$CLAUDE_PROJECT_DIR/.bee/bin/bee" … "${CLAUDE_PLUGIN_ROOT}/.bee/bin/bee.exe"; do
  [ -x "$b" ] && exec "$b" hook <name>; done; exec node "${CLAUDE_PLUGIN_ROOT}/…/bee-<name>.mjs"`.
  Detection is at HOOK TIME, not render time: these files are rendered once and shipped, and the
  binary is machine-local (decision 1f4262ca) so **a plugin root never carries one** — the
  deployment-true location is the host repo's own `.bee/bin/`.
- **`.codex/hooks.json`** — same catalog, `TARGETS.REPO`. The POSIX leg keeps its git-root
  resolve (that is what LOCATES the binary from a nested cwd, and what the visible fail-open
  diagnostic is keyed on) and then prefers `"$r"/.bee/bin/bee[.exe] hook <name> --source=repo`.
  The `commandWindows` leg keeps its `node -e` bootstrap but now launches the binary, falling
  back to the wrapper: **this is the one surface Node cannot leave.** R8a's ban on `$`, `%` and
  backtick (what makes one string parse identically under cmd.exe and PowerShell) is also a ban
  on command substitution, so the command text cannot ask git for the root, and the binary can
  only resolve its own root once something has launched it from a root-dependent path. Deleting
  the `.mjs` tree does not break this leg; removing Node from such a host does. Closing it needs
  either a guarantee that Codex's Windows cwd IS the repo root, or `bee` on the host's PATH.
- **`.bee/config.json` `commands.test` / `commands.verify`** — both are now
  `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path
  packages/bee-rs/Cargo.toml`. The PATH prefix is deliberate: an agent session started before
  rustup otherwise fails the cap door with `cargo: command not found` rather than a real red.

**What the deletion step still owns here:**

1. Drop the `exec node …` fallback arm from all three renderers in `hooks/catalog.mjs`
   (`repoCommand`, `pluginCommand`, `windowsBootstrap`) and re-render; update the matching pins
   in `hooks/test_hook_contracts.mjs` (`launchesSourceWrappers`, `NODE_WINDOWS_BOOTSTRAP`,
   `expectedPluginAuditCommand`) and the fixture in its `codex-commandWindows-nested-cwd-execution`
   row. There is no render script — the three files are regenerated by calling
   `renderProjectionText` and the drift check is the gate.
2. **The HOST Codex projection is a FOURTH surface and is still Node-only.** `onboard`'s
   `renderCodexHookEntries` (both twins: `packages/bee/scripts/onboard_bee.mjs` and the live one,
   `packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs`) renders every host's
   `.codex/hooks.json` as `exec node "$r"/.bee/bin/hooks/bee-*.mjs`, with no feature detection —
   unlike `repoHookCommand`/`repo_hook_command`, which already detect a vendored binary. Both
   twins must change together or the two-renderers-one-file trap fires. Left alone here only
   because `packages/bee-rs/` was under concurrent edit.
3. `scripts/tests/test_verify_manifest.mjs` hard-fails on `commands.verify` not containing
   `run_verify.mjs` — false by design as of this change. Retire that clause (the SUITES/floor law
   above it is still live) or re-point it; `.github/workflows/ci.yml` runs it via `verify_all.mjs`.
4. `.bee/onboarding.json`'s `managed.repo_hooks[".codex/hooks.json"]` records
   `sha256(JSON.stringify(renderCodexHookEntries()))` — the HOST projection — while `doctor`'s
   `capability_baseline_match` row compares it to the sha256 of the LIVE file bytes. Those two
   values can never be equal in bee's own repo; the row was already warning before this change
   (recorded `882cb314…` vs the HEAD file's `e615e937…`). Fix the comparison or scope the row out
   of a catalog-owning repo.

#### Hard blockers for deleting the Node runtime (found by R5, 2026-08-01) — ALL CLOSED

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

## R6 cutover — what landed, and the one step that did NOT (2026-08-01)

**Landed and green (969 Rust tests, `cargo test --release -- --test-threads=1`).**

1. **The fourth wiring surface is closed.** `renderCodexHookEntries` — the HOST
   Codex projection — launches the binary on both legs, in BOTH twins
   (`packages/bee/scripts/onboard_bee.mjs` and
   `crates/bee/src/onboard/hooks_wiring.rs`), proven byte-identical by running
   each against its own fixture repo and diffing `.codex/hooks.json`. The
   recognizer (`isBeeCodexHookEntry` / `is_bee_codex_hook_entry`) was widened to
   the binary spelling in both, so a re-render REPLACES the old node entry
   instead of stacking a double-firing twin. Idempotence and stale-entry
   migration proven on a real fixture.
   The Claude repo projection (`repoHookCommand`) lost its node arm too: a host
   with no binary now gets the same runtime-detecting `for b in …` loop the
   plugin projection uses, ending in a VISIBLE `bee: hook binary missing`.

2. **The delegate is gone.** `js_fallback.rs` is deleted; `main.rs` ends at
   `router::emit_unsupported_shape`, which names the attempted argv, adds the
   owning group's real sub-verb list read out of `REGISTRY_PAYLOAD`, and exits
   1 (`{"error": …}` on stdout when `--json` was asked for). `state show` and
   `backlog list` — the two shapes found in live use — now answer with `bee
   state takes: set, gate, …`. The hook delegate became `emit_undecidable`:
   exit 0 (a hook may never fail closed on infrastructure) with a loud stderr
   line saying the guard did NOT run on that payload.

   `roots.rs` grew a third door and lost `NeedsNode` entirely:
   * FULL (`resolve_store_root_worktree`) — worktree/status/orient/reservations.
   * WIDE (`resolve_store_root_any`, new) — help, status --brief, capture,
     backlog, feedback, knowledge, intent, reviews, tmp, test. Audited to read
     nothing but the store root, so both grant states are served.
   * NARROW (`resolve_store_root`) — state, close, cells, dispatch prepare,
     decisions. Now SERVES an ungranted worktree (where `storeRoot == mainRoot`,
     making it identical to an ordinary checkout at mainRoot — a proof, not a
     widening) and REFUSES a granted one by name, printing the main checkout to
     run from. `bee worktree new` produces an ungranted checkout, so the refusal
     only fires after an explicit `bee worktree register`.
   * `LinkInvalid` is EMITTED everywhere now (Node's own message, including the
     timings.jsonl append Node skipped) instead of delegated. `Exotic` retired.

3. **Distribution.** INSTALL.md, LLM.md and README require a Rust toolchain and
   a `cargo build --release`; the Node 18+ preflight is gone from both
   installers, replaced by a cargo preflight, a real `cargo build --release`,
   and a copy of the built binary into `<target>/.bee/bin/`. `.github/workflows`
   run cargo (canary.yml deleted with the `.mjs` probe it existed to run;
   windows.yml now runs the WHOLE suite, unexcluded). The statusline scripts
   call `bee dev statusline`.

4. **catalog.mjs — DECIDED: ported to Rust**, not kept and not hand-maintained.
   `crates/bee/src/devtools/hook_manifests.rs` owns the catalog; `bee dev
   render-hook-manifests --write|--check` regenerates the three projections and
   `hook_manifests_match_disk` is the drift gate inside `cargo test`. All three
   files were re-rendered with no `exec node …` arm. The Codex REPO projection
   now CALLS onboard's renderer instead of duplicating it, so the
   "two-renderers-one-file" trap that once broke a release cannot fire.

**DONE (2026-08-02) — the physical deletion of the `.mjs` trees.** 236 files /
~177k LOC removed. `cargo test --release -- --test-threads=1`: **964 passed, 3
ignored** (from 969; the delta is explained below). All eleven blockers were
worked; ten are closed, one is deliberately NOT closed.

**The three owner decisions, as implemented.**

1. **`BEE_VERSION` lives in `.claude-plugin/plugin.json`.** It is resolved AT
   COMPILE TIME (`crates/bee/src/version.rs` `include_str!`s the manifest and
   extracts the value in a `const fn`), so the binary cannot disagree with the
   checkout it was built from and a manifest that loses the key is a build
   error. This also retired three hand-maintained copies of the literal in
   `hooks/compaction.rs`, `hooks/session_preamble.rs` and
   `verbs/status_full.rs` that nothing had ever pinned to each other. The
   runtime tuple in `read_source_release_identity` went from three members to
   two (`.claude-plugin/plugin.json` + `.codex-plugin/plugin.json`) — still two,
   deliberately, because the property that mattered was that the manifests
   cannot drift apart unnoticed.

2. **The impact registry is RETIRED**, not re-pointed: `devtools/impact_registry.rs`,
   the `bee dev impact-registry` route, `scripts/impact-registry.json`,
   `scripts/verify-cache-inputs.json`, the CI freshness step and the cap-time E1
   cross-check in `verbs/cells.rs` are all gone. Rationale: its subject was the
   `.mjs` suite graph, and the cargo suite that replaced it runs whole in ~20s
   (measured: `bee test` = 22.5s), so impact-based filtering buys nothing.

3. **`release_manifest`'s scope is THE SHIPPED FRAME** — what a host actually
   receives. `.bee/bin/lib/*.mjs` (38 records) and the two
   `scripts/tests/test_*.mjs` (2 records) left it; `.bee/bin/bee` joined it.
   The binary is GITIGNORED and per-host, so it is represented as a
   PRESENCE-ONLY entry in a new top-level `unhashedArtifacts` block rather than
   hashed (a hash would fail on every machine but the last one to `--write`) or
   omitted (the frame would not mention the only executable it ships).
   `schemaVersion` is 2 and a v1 stored manifest is now its own one-line
   refusal instead of 40 spurious mismatches. The committed manifest was
   regenerated: 326 -> 184 records, 0 `.mjs`.

**Two traps found and closed that the table did not list.** Both are the same
shape as the install-probe trap that stopped the first attempt — a guard that
would have gone quietly inert rather than loudly red:

* **`.bee/bin/bee.mjs` had no removal path.** Blocker 4 named the missing
  `remove_repo_hook`; the vendored HELPER had the same hole. `remove_helper`
  only ever fired for names hard-coded in `RETIRED_HELPERS`, so deleting the
  Node CLI entrypoint from source would have left it on every host forever — a
  dispatcher whose entire `lib/` closure `remove_lib` deletes out from under it,
  sitting at the path AGENTS.md tells agents to invoke. Both removals are now
  derived from the ledger diff the way `remove_lib` always was, and the applier
  refuses any `remove_helper` target that is not a flat `*.mjs` under
  `.bee/bin/` — because `.bee/bin/` also holds the running binary.
* **`instruction_laws.rs`'s JSON scan root over `scripts/`** went to zero files
  the moment `scripts/impact-registry.json` was deleted. The vacuity guard
  caught it loudly (`IMPLAUSIBLY SMALL`), exactly as designed; the law is
  re-pointed at `packages/bee/**/*.json`.

**Test-count delta: 969 -> 964 (-5).** Net of: -16 removed with the impact
registry (its own unit tests, the `jspath` helpers only it used, and the
cap-time E1 test), +11 added — the two ledger-derived removal actions end to
end, the `remove_helper` containment proof, the `unhashedArtifact` presence
check, the two `INVENTORY_ROOTS` pins, the ledger-group classification pin, and
three `version.rs` pins. No test was deleted without either its subject being
gone or its assertion being re-homed.

**Two pins were replaced rather than dropped**, because both were INDIRECT pins
whose authority was the deleted `.mjs`:

* `devtools/prompts.rs` and `verbs/drivers.rs` are two Rust ports of one prompt
  grammar. They used to be pinned only by each byte-embedding the same
  `prompt-renderer.mjs`. The corpus now runs through BOTH ports and compares
  their answers directly (proven to bite: mutating `drivers.rs::render` fails
  the corpus, naming the case).
* `verbs/cells.rs`'s `REGEN_GUARDS` derived its covered roots by PARSING two
  `.mjs` files, and `derive_regen_guards` `continue`d — silently deactivating
  the guard — when a script was missing. Deleting the scripts would have hit
  exactly that arm and switched both obligations off with no output. The guards
  now read the same constants their authorities use
  (`release_manifest::INVENTORY_ROOTS`, `plan::LEDGER_GROUPS`), each pinned to
  its builder in both directions, and the missing-file arm no longer exists.

**Ledger parity was NOT lost with `scripts/ledger_parity.mjs`.** Its check —
recompute every managed hash and compare — already exists natively in
`verbs/status_full.rs`'s `compute_runtime_drift`, which runs on every
`bee status` rather than only when someone remembers to invoke a script.

**Blocker 11 is NOT CLOSED — deliberately, and this is the one thing left.**
`packages/bee/scripts/plugin_distribution.mjs` (464 lines) survives, and both
installers still execute it. It is not a shim: it proves an installed plugin
package against the release manifest, strips bee entries from host hook configs
(with the codex-hybrid exemption that stops it deleting the enforcement it just
installed), gates user-global skill-root cleanup on an exact ownership ledger,
and snapshot/revalidates every target to close a TOCTOU window. Porting it is
its own piece of work with its own contract, not a mechanical step, so it was
kept rather than deleted through. **`node` therefore remains required to INSTALL
bee; it is not required to RUN it.** The inline `node -e` JSON helpers in the
installers are a symptom of the same gap and go when the helper is ported.

Its test, `test_plugin_distribution.mjs`, WAS deleted: two of its three imports
(`onboard_bee.mjs`, `scripts/lib/env-capabilities.mjs`) are gone, so it could no
longer run. Keeping an unrunnable suite on disk reads as coverage while
asserting nothing. **The helper is therefore currently unported AND untested —
that is the open item.**

What was fixed in the installers along the way (both were already broken, or
about to be): the source-detection probe keyed on the deleted
`onboard_bee.mjs` — left alone it would have silently RE-CLONED from GitHub
instead of installing the local checkout (`install.sh`), and hard-failed every
clone-path Windows install with a message blaming the user's git version
(`install.ps1`); both now key on `packages/bee-rs/Cargo.toml`. And
`install.sh`'s up_to_date recheck ran `node "$ONBOARD"` where `$ONBOARD` was
never assigned anywhere in the file — it now runs the installed binary.

<details>
<summary>The original blocker table (2026-08-01), kept for the record</summary>

| # | Blocker | Why it blocks |
|---|---|---|
| 1 | `devtools/prompts.rs:53` `include_str!(lib/prompt-renderer.mjs)` | build error; its pin test goes with it |
| 2 | `devtools::bee_source_root()` markers = `packages/bee/lib/state.mjs` + `scripts/run_verify.mjs` | every `bee dev …` verb silently becomes "unsupported command shape" |
| 3 | `onboard::source::locate()` marker = `packages/bee/scripts/onboard_bee.mjs`; `read_source_release_identity` reads `BEE_VERSION` out of `lib/state.mjs`; `onboard/skills.rs` uses the same file as its identity anchor | `bee onboard` becomes unreachable. **Needs an owner decision: where does `BEE_VERSION` live now?** (`.claude-plugin/plugin.json` is the obvious candidate — the installer already reads it.) |
| 4 | onboard's vendoring plan (`plan.rs` `list_template_helpers` / `list_template_lib_modules` / `list_plugin_hooks`) | the lists go empty. `remove_lib` then DELETES every host's `.bee/bin/lib/*.mjs` (correct, and wanted) — but there is no `remove_repo_hook` action at all, so stale `.bee/bin/hooks/*.mjs` linger on hosts forever |
| 5 | `devtools/release_manifest.rs` inventory roots: `.bee/bin/lib/*.mjs` (REQUIRED) and `scripts/tests/test_verify_manifest.mjs` + `test_release_tuple.mjs` (REQUIRED) | hard refuse. Needs its scope redrawn and `docs/history/codex-harness-hardening/release-manifest.json` (248 `.mjs` records) regenerated |
| 6 | `devtools/impact_registry.rs` derives its suite list by PARSING `scripts/run_verify.mjs` | the impact registry has no subject once the `.mjs` suites are gone. **Decision needed: retire the tool + `scripts/impact-registry.json` + `verify-cache-inputs.json`, or re-point it at the cargo suite** |
| 7 | `verbs/cells.rs` `REGEN_GUARDS` parses `scripts/release_manifest.mjs` and `scripts/ledger_parity.mjs` source to derive covered roots | the regen obligation on `cells cap` silently stops firing |
| 8 | `devtools/skill_trees.rs` walks `skills/**` for all file types | deleting `skills/bee-herding/scripts/*.mjs` fails `render_matches_the_committed_trees` unless the four rendered projections are regenerated in the same change |
| 9 | `tests/instruction_laws.rs` `SCRIPT_TREE` (`min_files: 10` over `scripts/**.mjs`) | `scripts/` survives (install.sh/.ps1/JSON), so the floor fires `IMPLAUSIBLY SMALL` rather than skipping — two laws go red, and a third (`SELF_REFERENTIAL`) goes half-vacuous |
| 10 | `tests/hook_contracts.rs` `copy_vendored_lib` panics on the real `.bee/bin/lib/*.mjs` | the whole hook-contract matrix dies |
| 11 | `scripts/install.sh` / `install.ps1` still call `plugin_distribution.mjs` (unported) and use inline `node -e` for JSON | the installers cannot be Node-free until that helper is ported |

Blockers 3, 5 and 6 carry owner-facing decisions; the rest are mechanical.

</details>
