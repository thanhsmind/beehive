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

## Proof

- prd-1: `cargo test --release -p bee --bin bee` filtered to the new verb's tests — green:unit.
- prd-2: the new contract case watched RED before the belt edit, then
  `cargo test --release -p bee --test pi_plugin_contracts` green:unit, plus
  `bee doctor --runtime pi` for `wiring_matches_binary` — green:live.
- prd-4: the seven row names present in the Pi table with explicit n/a where the
  mechanism does not exist, and the Claude and Codex blocks unchanged — green:static.
- Feature: a `.bee/verify/verify-app` run driving a Pi leader through a
  relocation with a job in flight, at close, if the sandbox is available.
