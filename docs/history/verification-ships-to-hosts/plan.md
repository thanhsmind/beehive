---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Verification Ships To Hosts

Mode: `high-risk` — 5 risk flags: public-contracts, cross-platform,
covered-contract-change, multi-domain, audit-security.

Why this is the least workflow that protects the work: the drafted shape would
have moved the bound on what `bee onboard --apply` may DELETE inside a host
repository. The hat wave killed that shape. What remains is smaller than what
was drafted, and the wave is the reason — so the wave earned its place.

## Requirements (from CONTEXT.md)

- **D1** — two skills by lifecycle: `bee-verifying` generates once,
  `bee-verify-upkeep` audits periodically.
- **D2** — the generated drive command COMPOSES into `commands.test`.
- **D3** — a generated `verify-<app>` is SOURCE under `.bee/verify/<name>/`,
  rendered into every runtime skill home.
- **D4** — no proof tier, no `commands.verify` revival, no new door.
- **D5** — `bee onboard` OFFERS, it never generates.

## Load-bearing claims

Labels are `read` (the author opened that file at that line and saw those
bytes) or `ran` (executed, output held). The evidence column is a verbatim byte
substring of the anchored line; multi-line evidence joins lines with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Only `bee-`-prefixed entries are walked as skill sources, so iteration domain and deletion domain are the same literal | read | `packages/bee-rs/crates/bee/src/onboard/render.rs:379` | `.filter(\|e\| e.name.starts_with("bee-")).collect()` |
| 2 | The prefix is the DELETION bound — the guard stopping onboard removing a host repo's own skills | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:882-883` | `if !name.starts_with("bee-") {` / `return Some(format!("refusing to remove {name}: outside the bee-* namespace"));` |
| 3 | Removing a prefixed dir absent from source is INTENDED behavior, pinned by a test — so widening the prefix to a word bee does not own would delete host-authored skills | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:954` | `fn foreign_bee_skill_in_target_is_removed_but_non_bee_is_untouchable() {` |
| 4 | The removal is unrecoverable — no trash, no journal, the error is discarded | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:907` | `remove_dir_all` |
| 5 | The renderer STRIPS the executable bit, so a rendered drive script cannot be executed directly | ran | `ls -l skills/bee-herding/scripts/bootstrap-cockpit.sh .claude/skills/bee-herding/scripts/bootstrap-cockpit.sh` | `-rwxr-xr-x skills/bee-herding/scripts/bootstrap-cockpit.sh` / `-rw-r--r-- .claude/skills/bee-herding/scripts/bootstrap-cockpit.sh` |
| 6 | There are THREE runtime skill homes, not the two D3 names | read | `packages/bee-rs/crates/bee/src/onboard/templates.rs:317-321` | `("repo-claude", &[".claude", "skills"]),` / `("repo-opencode", &[".opencode", "skills"]),` |
| 7 | D5's stated trigger does NOT fire for the repos it targets: the no-test notice is nested inside a legacy-key branch, so a repo that never had `commands.verify` gets no notice at all | read | `packages/bee-rs/crates/bee/src/onboard/notices.rs:203` | `if commands.contains_key("verify") {` |
| 8 | A copy-only ship path to a host tree already exists and has no removal half in the same shape — the precedent for render-without-prune | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:502-509` | `plan.push(plan_item("copy_expertise", &format!(".bee/expertise/{name}")));` |
| 9 | Ledger-diff removal (previous recorded set minus current) is the only prune shape bee already uses that deletes exclusively its own past writes | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:494-500` | `plan.push(plan_item("remove_lib", &format!(".bee/bin/lib/{name}")));` |
| 10 | `commands.test` is the only declared test slot; there is no verify slot to revive | read | `packages/bee-rs/crates/bee/src/onboard/templates.rs:284` | `pub const COMMAND_KEYS: &[&str] = &["setup", "start", "test"];` |
| 11 | The owner retired the proof-tier matrix for weight; D4 exists to stay inside that decision | read | `.bee/decisions.jsonl:1763` | `Owner 2026-07-31: bee's evidence machinery grew too heavy` |
| 12 | This repo's `verify-bee` is tracked, so it is recoverable here — the unrecoverable case belongs to prior adopters and to the generate→prove→commit window | ran | `git ls-files .claude/skills/verify-bee \| wc -l` | `8` |

## Discovery

Five perspectives were dispatched at the plan step. Three findings changed the
shape; two corrected the spec.

The decisive one: `bee-*` prune is sound because of an **ownership axiom**, not
a mechanism — bee mints those names and nobody else does, and the source ships
inside a versioned engine behind an identity anchor. `verify-` is a generic
English word with no anchor, no version stamp, and a source living in the
mutable host repo. Extending the mechanism without the axiom extends only the
deletions (claims 1-4).

The trap underneath it: writing the two-prefix guard and its test together
would mirror claim 3's test as `foreign_verify_skill_in_target_is_removed` —
and that green test would **certify the hazard as intended behavior**. The
fixture bytes are identical whether the directory is a stale bee render or a
user's hand-written skill. A guard and its tests are one model.

Two spec corrections, both verified: the renderer strips the executable bit
(claim 5), and D5's trigger does not fire for its target population (claim 7).

## Approach

**Render, never prune.** bee gains a copy-only path from `.bee/verify/` into
the three runtime skill homes. It creates and it updates. It never removes.
The deletion domain and its guard (claims 1-2) are not touched, and claim 3's
test is not modified.

This stays inside all five locked decisions: D3 requires the generated skill be
*rendered* into every runtime home; the prune half is mechanism inheritance,
never a locked decision.

Staleness is already owned — by `bee-verify-upkeep` (D1), run by an agent in a
git working tree, where a removal is visible, reviewable and committable.

The cost asymmetry is what decides it, and it matches bee's own locked
decision on irreversibility (`decisions.jsonl:1338`): render-only's worst case
is a stale skill lingering until the next audit, which self-heals; prune's
worst case is a silent, unrecoverable deletion of user-authored work in someone
else's repository during a routine upgrade.

Rejected, one line each:

- **Widen the prefix to `verify-`** — deletes host-authored `verify-*` skills
  (claims 1-4). The shape the wave was convened to test, and killed.
- **Name the generated skill `bee-verify-<app>`** so it falls inside the owned
  namespace — still routes a host-authored, unversioned source through an
  identity anchor and a three-version preflight that only have meaning for an
  engine-side source.
- **Ledger-derived prune** (claim 9) — the only safe prune shape, and correct;
  deferred, because render-only needs it only if staleness proves to be a real
  problem, and D1 already assigns staleness to a skill.
- **Write straight into the runtime homes at generation time** — three
  hand-maintained duplicates with no anchored source, and no drift repair on a
  teammate's fresh clone.

Risk map: the copy path is LOW (creates and updates only, no removal reachable);
the `commands.test` mutation is MEDIUM (host config, user-visible cost, proof
by before/after diff); the two new skills are LOW.

## Shape

Four slices. Slice 1 is a walking skeleton: real behavior end to end, no stubs.

1. **`bee-verifying` + `bee-verify-upkeep` land in `skills/`**, renamed from
   the two proven sources, keeping `disable-model-invocation` so neither joins
   the always-loaded routing surface. `bee dev regen`. No Rust yet. Proof:
   both render into all three committed trees and the parity check is green.
2. **The copy-only render path** — `.bee/verify/` → the three runtime homes,
   create and update only. The negative test is the point of this slice: a
   `verify-*` directory bee did not create is provably untouched, authored as a
   host-repo fixture, not from the guard's point of view.
3. **The offer** — a NEW notice for "no `commands.test` declared at all",
   independent of the retired-verify branch (claim 7), pointing at
   `bee-verifying`.
4. **The drive command reaches `commands.test`** — composed as
   `bash <path> …` because the rendered copy is not executable (claim 5), and
   behind its own second consent moment carrying the measured cost.

## Test matrix

High-risk, so the 12 edge dimensions apply. Load-bearing probes:

| Dimension | Probe |
|---|---|
| Foreign occupant | a host's own `verify-payments/` in a runtime home, absent from `.bee/verify/` → untouched by `--apply`. Authored as a host fixture. |
| Absence | `.bee/verify/` missing, empty, and unreadable → three tests, zero write items each, and never a `block_all` of bee's own skill sync |
| Blast containment | a malformed host-authored `.bee/verify/**/SKILL.md` → `bee-hive` still syncs; only the verify item is reported |
| Root preflight | `.bee/verify` symlinked, or resolving onto a target root → refused |
| Exec bit | the rendered drive is mode 0644 and the composed command still runs (`bash <path>`) |
| Cross-runtime | one generation visible under all three homes of claim 6 |
| Idempotence | `--apply` twice yields an identical tree and an identical plan |
| Dry-run honesty | an `--apply`-less run mutates nothing, proven by before/after diff, never by the flag's name |
| Global isolation | the legacy `~/.claude/skills` refresh never copies or removes a `verify-*` |
| Migration | this repo's own state as fixture: `.claude/skills/verify-bee` present, `.bee/verify/` absent → untouched |

## Open Questions

- Does Codex's runtime load a non-`bee-*` skill from `.agents/skills` the same
  way it loads a `bee-*` one? Unverified; D3 is not done until it is. Same
  question for `.opencode/skills`.
- What does the fast drive scope to in a repo the generator has not seen?
  `bee-verifying` must state the contract the generated skill has to satisfy.
- D2's composition semantics when `commands.test` is an ARRAY, not a string
  (`notices.rs:206` reads both) — and whether a second generation appends a
  second `&& drive`.

## Out of scope

- A progress test for `bee herding control-loop`.
- Narrowing the two cockpit panes' CLI wildcard.
- `.bee/expertise/changes.md`.
- Ledger-derived prune for `.bee/verify/` — recorded above as the correct
  future shape if staleness proves real.
