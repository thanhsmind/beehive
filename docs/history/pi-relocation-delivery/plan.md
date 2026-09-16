---
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Pi relocation delivery

## Summary

When a Pi session moves into a feature worktree (or back to main before a
merge), Pi replaces the session and mints a new session id. Today that move
drops two things: the results of background jobs started before the move, and
ownership of the cell the session claimed. The leader then waits for a result
that can never arrive, and the cap has to override ownership.

This plan carries both across the move. The new session keeps reading the old
session's inbox folder — it carries the token, it never moves the files — and it
asks bee to rebind the old session's claims to the new session. One contract
test drives a relocation with a job in flight and proves both. The Pi rows in
the swarming skills are completed in the same feature.

Mode: `standard` — 0 risk flags: none
Why this is the least workflow that protects the work: one new CLI verb, one
belt change, one contract test and two skill files; no gate-bearing behavior of
another runtime changes, and the behavior change is covered by a test that is
red before the fix.

## Requirements (from CONTEXT.md)

- D1 (`0833887d`): a relocated Pi session keeps receiving the results of
  detached herding jobs launched before the move.
- D2 (`52d1e3aa`): a cell claim survives its session's relocation; the cap after
  the move needs no `--force-ownership`.
- D3 (`b527603f`): one Pi contract test drives relocation and detached delivery
  together.
- D4 (`3cb3523c`): the Pi skill blocks name the worker-side advisor transport
  for a herding-shaped advisor and carry the spawn-mechanics rows Claude and
  Codex already have.

Constraints carried in: relocation stays session replacement
(`pi-worktree-session-relocation` D1/D2); delivery stays at-least-once with
`job_id` as the dedupe key (`pi-result-mailbox`); Pi worker dispatch stays
herding-only (`pi-stage-dispatch` D9).

## Load-bearing claims

Every row is load-bearing; no `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The drain arms on the current session id alone, so a pre-move marker is never seen again | read | `.pi/extensions/bee-guard.ts:841-844` | `const token = usableInboxToken(sessionId)` |
| 2 | The belt holds both ids at relocation: the old one is checked against the live session before the fork | read | `.pi/extensions/bee-guard.ts:1408-1409` | `typeof intent?.piSessionId !== "string" ||` … `intent.piSessionId !== currentSessionId` |
| 3 | The marker is written once, pre-spawn, under the MAIN checkout's `.bee` — never the worktree's | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2243` | `write_inbox_marker(&opts.main_root.join(".bee"), opts);` |
| 4 | The reader probes the current directory FIRST and the main root second, returning the first root that holds the token dir | read | `.pi/extensions/bee-guard.ts:652-658` | `for (const root of candidateRoots(directory)) {` … `if (isDirectory(dir)) return dir` |
| 5 | A claim owned by another session refuses the cap, naming `--force-ownership` as the only way past | read | `packages/bee-rs/crates/bee/src/verbs/cells/trace.rs:168-175` | `"cell \"{id}\" is claimed by session \"{owner_disp}\" ({}) — another session owns it. Pass --force-ownership to override (audited).",` |
| 6 | The cells module already owns an in-place claim rewriter that bumps the fence epoch under a gate, and no verb reaches it today | read | `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:334-339` | `/// Not called from a verb in THIS module: the one native caller today is` … `/// `state handoff adopt`, which lives in verbs/state_group.rs` |
| 7 | A bumped fence epoch refuses a later caller that presents an epoch behind the stored one | read | `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:234`, `:311` | `// typed CLAIM_FENCE_STALE when a caller presents an epoch BEHIND the stored` |
| 8 | Module state survives the switch, but `session_shutdown` fires mid-switch with `reason === "resume"` and clears the belt's relocation maps | read | `.pi/extensions/bee-guard.ts:1961-1962`, `:1990-1991` | `if (reason === "resume" && activeTransition) {` … `pendingRelocationTokens.clear()` |
| 9 | An injected marker stays on disk as `<job>.json.processing` until the turn settles, and orphan reclaim renames any such file back for re-injection | read | `.pi/extensions/bee-guard.ts:710-719`, `:1838-1839` | `if (name.endsWith(".json.processing")) requeueClaim(path.join(dir, name))` ; `for (const processing of inFlightClaims) rmSync(processing, { force: true })` |
| 10 | The contract harness hands the replaced context the OLD session id, so a relocation test cannot tell the two ids apart today | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:701` | `const replacedCtx = createCommandContext(newCwd, ctxSessionId, {` |
| 11 | The loss is real, twice, in one live run | ran | `/tmp/bee-verify/evidence/20260915-224413-891129/pi-leader-log.md` | markers stranded under `.bee/result-inbox/01a0a5be…/` and `…/01a0a5c9…/` while the session ran as `01a0a5c9…` then `01a0a5e1…` |
| 12 | The installed belt must match the bytes bee embeds, or Pi doctor fails closed | read | `packages/bee-rs/crates/bee/src/doctor.rs:312` | `.pi/extensions/bee-guard.ts differs from what this bee embeds` |

## Discovery

A dispatched read worker digested the relocation path, the drain, the marker
writer and the claim store (`job-1789511807050-1887671-1`). The plan-step hat
wave then critiqued the first draft; its synthesis is
`docs/history/pi-relocation-delivery/reports/hat-wave-synthesis.md`, and every
finding folded in below was re-checked against the bytes before it was accepted.
Baseline before any edit: `cargo test --release -p bee --test pi_plugin_contracts`
— `67 passed; 0 failed`.

## Approach

Recommended path — the new session CARRIES the old token; nothing on disk moves.

1. A new control-plane verb `bee cells rebind-session --from <old> --to <new>`
   rewrites every ACTIVE claim whose `session` equals `--from`, through the
   cells module's own rewriter (`verbs/cells/claims.rs:339`, claim 6), which
   bumps the fence epoch under its gate. Claims of any other session are
   untouched; no match is a clean no-op. This satisfies D2.
2. The belt carries the token across relocation. At transition time it records
   the pair (old token → new token) and the new session's drain reads its own
   inbox folder plus the carried one(s), keyed under the MAIN checkout's `.bee`
   — the same root the writer always uses (claims 3, 4). Nothing is created in
   the worktree's `.bee`, so a job dispatched AFTER the move is still found, and
   worktree cleanup cannot delete a pending marker.
   - The carry survives the mid-switch `session_shutdown` (claim 8): the carry
     slot is exempt from that handler's clears, and the same pair is written as
     one small pointer file under the main root so a host restart mid-relocation
     does not lose it.
   - A carried folder is read for unclaimed `<job-id>.json` markers only. Orphan
     reclaim never runs over a carried folder, so a result already injected into
     the old session (still on disk as `.processing`, claim 9) is not injected a
     second time — the forked transcript already carries it.
3. Order and failure handling inside the transition: the carry is recorded and
   `cells rebind-session` runs FIRST in the `withSession` closure, before
   `handlePostExitMerge` (which shells a merge that can refuse, and on success
   removes the worktree). If the switch afterwards reports cancelled or failed,
   the rebind is undone in the same symmetry (rebind back to the old id) so the
   still-running old session keeps its claims. A rebind that fails does not
   abort the transition; it warns in the UI naming the manual verb.
4. The relocation notice names what was carried when it is non-zero (claims
   rebound, jobs carried).
5. The embedded copy of the belt is regenerated so `wiring_matches_binary` stays
   green (claim 12).
6. One Pi contract test drives it: dispatch a job, relocate, and assert the new
   session drains the carried marker and the claim names the new session — plus
   a job dispatched AFTER the move still delivered. The harness must first be
   able to tell the ids apart (claim 10), so the replaced context reports the
   forked session's own id; the test is written and watched red before the belt
   change in the same cell.

SMALLER PATH: the first draft moved marker files between folders. Carrying the
token deletes four pieces of machinery it needed — the destination `mkdir`, the
same-name collision policy, the `.processing` requeue, and the root question —
and it is also safer under an aborted switch, where nothing on disk has changed.
The one thing moving files bought was durability across a host restart, and the
pointer file buys that for one small write instead of N.

Rejected alternatives:

- Move the marker files into the new token's folder — rejected above (SMALLER PATH).
- Drain the parent session's inbox by following `parentSession` — rejected: the
  chain grows with every move, and a stale parent folder would be drained by
  whichever session happens to be alive.
- Keep one stable token per conversation instead of Pi's session id — rejected:
  it re-opens `pi-result-mailbox`'s token contract and every shipped
  `--inbox-session "$PI_SESSION_ID"` instruction.
- Teach `cells finish` to accept a session whose ancestor held the claim —
  rejected: it weakens the ownership guard for every runtime to fix one runtime.
- Leave the claim and tell leaders to pass `--force-ownership` — rejected: it
  makes an audited override the normal path.

Risk map:

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| `cells rebind-session` verb | MEDIUM | It rewrites claim files other sessions may hold | prd-1 | Unit tests: only active claims whose `session` equals `--from` move; fence epoch bumps by one; no match is a clean no-op; the verb states what it did |
| Carried-token drain | MEDIUM | A wrong root or a reclaimed `.processing` file would swap one silent loss for a duplicate | prd-2 | Contract test: pre-move job delivered after the move; post-move job delivered; no second injection of an already-injected job |
| Transition order and abort | MEDIUM | A rebind that outlives a cancelled switch strands the claims | prd-2 | Contract test asserts rebind-then-merge order and the undo on a cancelled switch |
| Embedded belt bytes | LOW | A stale embed fails Pi doctor closed | prd-2 | `bee doctor --runtime pi` row `wiring_matches_binary` ok |
| Skill rows | LOW | Text only; must not change Claude or Codex blocks | prd-4 | All seven row names present in the Pi table, with an explicit n/a where the mechanism does not exist on Pi |

Waves: prd-1 and prd-4 run in parallel — disjoint files, no dependency. prd-2
follows prd-1, because its test asserts the claim rebind end to end.

## Shape

| Cell | Title | Role | Files | Depends on |
|---|---|---|---|---|
| prd-1 | Add `bee cells rebind-session --from --to` over the cells-module claim rewriter | code | `packages/bee-rs/crates/bee/src/verbs/cells/*`, `src/generated/registry_payload.json` | — |
| prd-2 | Carry the inbox token and the claims across a Pi relocation, red-first | code | `.pi/extensions/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`, generated embed | prd-1 |
| prd-4 | Complete the Pi runtime rows in the swarming skills | docs | `skills/bee-swarming/references/worker-details.md`, `skills/bee-swarming/references/swarming-reference.md` | — |

## Cells (current slice)

```json
[
  {
    "id": "prd-1",
    "feature": "pi-relocation-delivery",
    "title": "Add bee cells rebind-session --from --to",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["52d1e3aa-97d3-4a0d-a89a-67f0822ec695"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/util.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/pi-relocation-delivery/CONTEXT.md",
      "docs/history/pi-relocation-delivery/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/util.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add the control-plane verb `bee cells rebind-session --from <old-session> --to <new-session>` (both required, non-empty). It rewrites every ACTIVE claim in .bee/claims whose `session` field equals --from so that it names --to, through the cells module's own rewriter `adopt_claim` (packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:339) — never a second writer, and never the narrowed state_group twin. Claims of any other session, and expired claims, are untouched. No match is a clean no-op: exit 0 with an empty list. Human output names each cell rebound; --json returns {\"rebound\":[{\"cell\":\"<id>\",\"from\":\"<old>\",\"to\":\"<new>\",\"fence_epoch\":<n>}]}. Serve it in the cells verb table (util.rs try_mutating) AND declare it in src/generated/registry_payload.json in the same change — a served-and-undeclared verb reads as unknown to `bee --help --all`, and a declared-and-unserved one fails tests/registry_dispatch.rs. State in one line, in the verb's doc comment, what a bumped fence_epoch means for any other holder of that cell (see CLAIM_FENCE_STALE, claims.rs:234 and :311).",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee rebind",
    "must_haves": {
      "truths": [
        "bee cells rebind-session --from A --to B rewrites only active claims whose session is A",
        "a rebound claim's fence_epoch is exactly one higher than before",
        "no matching claim exits 0 and reports an empty list",
        "the verb is both served and declared in the registry"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs", "substantive": "the handler, with its own unit tests"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "the verb declaration with both flags required"}
      ],
      "key_links": [
        "the handler calls claims.rs adopt_claim rather than duplicating the claim rewrite",
        "util.rs try_mutating routes the verb name to the handler"
      ],
      "prohibitions": [
        "Do not change existing claim, cap, or handoff-adopt behavior",
        "Do not touch .pi/extensions/bee-guard.ts or the test file pi_plugin_contracts.rs"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "prd-2",
    "feature": "pi-relocation-delivery",
    "title": "Carry the inbox token and the claims across a Pi relocation",
    "lane": "standard",
    "role": "code",
    "deps": ["prd-1"],
    "decisions": [
      "0833887d-9005-46de-84f4-df265da53cbb",
      "52d1e3aa-97d3-4a0d-a89a-67f0822ec695",
      "b527603f-226d-4a19-96e1-7feaecf090d4",
      "a73c592a-d6c5-4e15-abe6-5ffab20e37c8"
    ],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pi-relocation-delivery/CONTEXT.md",
      "docs/history/pi-relocation-delivery/plan.md",
      "docs/history/pi-relocation-delivery/reports/hat-wave-synthesis.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST, in this order. (1) Make the contract harness able to tell the two session ids apart: the replaced context currently reports the OLD id (pi_plugin_contracts.rs:701 `createCommandContext(newCwd, ctxSessionId, {`), so have it report the forked session file's own id. (2) Add ONE contract case that dispatches a detached job with --inbox-session <old id>, relocates the session, and asserts: the new session drains that carried marker; a job dispatched AFTER the relocation is also delivered; an already-injected marker (`<job>.json.processing`) is NOT injected a second time; and the claim held by the old session names the new session with no --force-ownership. Run it and watch it FAIL for those reasons. (3) Then change the belt: the new session CARRIES the old token instead of moving files — the drain reads its own result-inbox folder plus the carried one(s), resolved under the MAIN checkout's .bee (the root `herding/run.rs:2243` always writes to), never the worktree's; the carry survives the mid-switch session_shutdown (bee-guard.ts:1961-1962, :1990-1991 clears the other relocation maps — the carry slot is exempt and is mirrored in one small pointer file under the main root); a carried folder is read for unclaimed `<job-id>.json` markers only, and orphan reclaim never runs over it. In performSessionTransition's withSession closure, record the carry and call `bee cells rebind-session --from <old> --to <new>` through execBeeCli FIRST, before handlePostExitMerge; if the switch afterwards reports cancelled or failed, rebind back to the old id. A failed rebind never aborts the transition — it warns in the UI naming the manual verb. When non-zero, the relocation notice names how many claims were rebound and how many jobs were carried. (4) Run the full regen inside this cell — `bee dev regen` (render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write, in that order) — so the embedded belt bytes and the release manifest match the edit.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check && .bee/bin/bee doctor --runtime pi --json",
    "must_haves": {
      "truths": [
        "a job dispatched before a relocation is delivered to the relocated session",
        "a job dispatched after the relocation is delivered too",
        "an already-injected marker is not injected a second time",
        "the claim survives the move: a cap after relocation needs no --force-ownership",
        "the carry and rebind run before handlePostExitMerge, and a cancelled switch undoes the rebind",
        "doctor --runtime pi reports wiring_matches_binary ok"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "carried-token drain, exempt carry slot, pointer file, ordered and compensating rebind, notice counts"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "harness reports the forked id, plus the new relocation-with-a-job-in-flight case"}
      ],
      "key_links": [
        "the drain resolves carried folders under the main checkout root, not the worktree",
        "the rebind call goes through the verb prd-1 added"
      ],
      "prohibitions": [
        "Do not move or delete marker files as the delivery mechanism",
        "Do not change how relocation itself works (session replacement stays)",
        "Do not touch Claude, Codex or OpenCode paths"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "prd-4",
    "feature": "pi-relocation-delivery",
    "title": "Complete the Pi runtime rows in the swarming skills",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["3cb3523c-7aef-4b22-aa90-17970371dae7"],
    "files": [
      "skills/bee-swarming/references/worker-details.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pi-relocation-delivery/CONTEXT.md",
      "skills/bee-swarming/references/worker-details.md",
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "affects_skills": ["skills/bee-swarming/SKILL.md"],
    "affects_specs": [],
    "action": "Two Pi gaps, text only. (1) worker-details.md around :272-281 offers a model-shaped advisor transport (claude block), a Codex-native one, and a cli-shaped one — but team.pi.advisor is herding-shaped, so a Pi worker handed an `Advisor:` line has no instruction it can run. Add a `bee:only pi` block naming the herding-shaped transport: run the prepared `bee herding run` command with the evidence bundle on stdin, keep the same `advisor-consult <cell-id>: <advisor-model>` attribution the goal-check reads from .bee/logs/dispatch.jsonl, and keep the existing one-slot transport-error rule. (2) swarming-reference.md :397-404 gives Pi one row (Result collection) where Claude (:375-385) and Codex (:386-396) carry seven. Fill the Pi table so all seven row names are present — Spawn, Model, Result collection, Follow-up/rescue, Harness assist, Isolation guarantee, Subagent type — with an explicit n/a and one-line reason where the mechanism does not exist on Pi (Subagent type has none: every Pi worker runs through `bee herding run`, never a native spawn). Model comes from config.team.pi.<role>; follow-up/rescue is the herding pane verbs; harness assist is the bee-guard belt's result drain. Do not touch the Claude or Codex blocks. Then run the full regen inside this cell — `bee dev regen` — so the rendered skill trees and the release manifest match the edit.",
    "verify": "rg -n 'bee:only pi' skills/bee-swarming/references/worker-details.md && rg -c 'Follow-up|Harness assist|Isolation guarantee|Subagent type' skills/bee-swarming/references/swarming-reference.md && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "a Pi worker reading worker-details.md finds a transport it can run for a herding-shaped advisor",
        "the Pi spawn-mechanics table carries all seven row names, with explicit n/a where the mechanism does not exist",
        "the Claude and Codex blocks are unchanged"
      ],
      "artifacts": [
        {"path": "skills/bee-swarming/references/worker-details.md", "substantive": "a bee:only pi advisor transport block"},
        {"path": "skills/bee-swarming/references/swarming-reference.md", "substantive": "the completed Pi table"}
      ],
      "key_links": [
        "the Pi advisor block keeps the same attribution string the goal-check reads"
      ],
      "prohibitions": [
        "Do not edit the claude or codex bee:only blocks",
        "Do not touch code or tests"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "prd-5",
    "feature": "pi-relocation-delivery",
    "title": "Run the relocation rebind from the main checkout, not the worktree",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["52d1e3aa-97d3-4a0d-a89a-67f0822ec695"],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pi-relocation-delivery/CONTEXT.md",
      "docs/history/pi-relocation-delivery/reports/uat-correction.md",
      ".pi/extensions/bee-guard.ts"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "A live acceptance run found the shipped relocation rebind refused every time. The belt calls `bee cells rebind-session` with a worktree cwd, and that verb is refused inside a granted feature worktree (reproduced: from the worktree it exits 1 with 'refused inside a granted feature worktree — ... FIX: run it from <main>'; from the main checkout the same call exits 0). Fix all three call sites in .pi/extensions/bee-guard.ts to pass the already-resolved `mainRoot` (computed at :1605) instead of a worktree path: the forward rebind at :1635 passes `intent.targetCwd`, and the two compensating rollbacks at :1688 and :1730 pass `intent.sourceCwd`, which is the worktree on the exit-before-merge direction. RED FIRST: the existing contract case missed this because the harness fakes the bee CLI and its stub records only argv and stdin, never the working directory — extend the stub prelude to append its own $PWD beside the argv it logs, add an assertion that the rebind invocation ran with the main checkout root, watch it fail against the current belt, then fix. Finally run `bee dev regen` so the embedded belt bytes and the release manifest match.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check && .bee/bin/bee doctor --runtime pi --json",
    "must_haves": {
      "truths": [
        "every bee cells rebind-session call the belt makes runs with the main checkout root as its cwd",
        "the stub bee records the working directory of each invocation",
        "the contract test asserts that cwd and fails against the pre-fix belt",
        "the compensating rollback on a cancelled or failed switch also runs from the main root",
        "doctor --runtime pi reports wiring_matches_binary ok after regen"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "all three rebind call sites pass mainRoot"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "the stub logs its cwd and the case asserts the rebind ran from the main root"}
      ],
      "key_links": [
        "mainRoot is already computed at bee-guard.ts:1605 and is in scope for all three sites"
      ],
      "prohibitions": [
        "Do not change the carry mechanism itself",
        "Do not touch Claude, Codex or OpenCode paths"
      ]
    },
    "behavior_change": true
  }
]
```

## Proof

- prd-1: `cargo test --release -p bee --bin bee` filtered to the new verb's tests — green:unit.
- prd-2: the new contract case watched RED before the belt edit, then
  `cargo test --release -p bee --test pi_plugin_contracts` green:unit, plus
  `bee doctor --runtime pi` for `wiring_matches_binary` — green:live.
- prd-4: the seven row names present in the Pi table with explicit n/a where the
  mechanism does not exist, and the Claude and Codex blocks unchanged — green:static.
- Feature: a `.bee/verify/verify-app` run driving a Pi leader through a
  relocation with a job in flight, at close, if the sandbox is available.
