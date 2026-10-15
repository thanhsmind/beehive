# SLP role-interaction gaps — Context

**Feature slug:** slp-role-gaps
**Date:** 2026-10-15
**Shaping session:** complete (gate bypass full; the user asked for all nine gaps: "làm toàn bộ")
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

A study of the five reference sets in `/mnt/data/Projects/refs/slp`
(paseo-room, piggery v0.10.2, paseo-doctrine-downstream, paseo-pi-team,
pi-peer) against bee found nine places where the roles talk to each other
worse than the references do. This feature closes all nine. It ends at the
contracts, prompts, doors and docs named below; it adds no new role.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | A worker result carries four optional fields beside its status: `reaction` (`confirm` \| `partial` \| `challenge` \| `blocked`, its stance toward the brief), `skipped` (checks it did not run, each with a reason), `unknowns` (assumptions it could not prove), `found` (incidental discoveries outside the cell). Herding results read them leniently (absent or wrong-typed reads as absent, never `Malformed`) and the run envelope re-emits them only when present; the native Result form names them too. Store `15f6804c`. | paseo-room, doctrine and piggery handbacks all carry stance, skips and unknowns. |
| D2 | A worker that finds a false premise in part of its cell finishes the part that does not depend on it, returns reaction `partial`, and names the stopped part with its evidence in a `stopped` field. The leader turns the stopped part into a new cell or a dissent verdict. `BLOCKED` stays for a cell with no safe part. Store `6e81d8db`. | paseo-room and paseo-pi-team stop only the dependent part. |
| D3 | Dissent once per point: after a verdict on a cell's dissent, a further `bee cells dissent` on that cell is refused unless it carries `--new-evidence <text>`. The worker doctrine says to follow the ruling. Every dissent still appends; nothing is overwritten. The herding transcription path passes the worker's new evidence through when the result carries it. Store `b2baf5e0`. | piggery: question the premise once, with evidence, then follow. |
| D4 | An advisor, hat or blind-lane answer states a recommendation, a confidence 0-100, and a falsifier: one concrete check that would prove the recommendation wrong, marked not run. The advisor prompt and the hat-wave and blind-lane briefs require all three. Store `2fb9b028`. | piggery advisor contract. |
| D5 | Synthesis over blind lanes or a hat wave reads the proposals with lane and model names hidden (lane A, B, C, in a shuffled order) until each is scored against the rubric; names are revealed only after scoring. Store `121947c8`. | paseo-room anonymized council packet; piggery reverse-order read. |
| D6 | A review brief never carries the orchestrator's own verdict, leaning or expected findings. A reviewer first checks that `HEAD` equals the frozen review head and that the in-scope paths are clean, and refuses with a typed reason on a mismatch. Store `1fa910ba`. | paseo-room and paseo-pi-team review the exact commit, given doubts, never verdicts. |
| D7 | A write-mode herding worker whose worktree fingerprint (HEAD plus diff hash) has not changed for `herding.stall_after_secs` (default 900) while it is still working shows `stalled` with reason `no_diff_progress`, and the leader gets ONE notice per stalled stretch in its result inbox. Observe-only: nothing is stopped or retried. Read-only jobs never get this signal. Store `8a826f2f`. | paseo-room: a frozen diff is the stall detector that works; piggery notifies `reports_to` once. |
| D8 | A final report for work that ships something runnable states each layer separately: source (committed), artifact (built), installed (copied or onboarded), live (the running bytes observed), each with its own verdict or `not checked`. The proof-line tag `green:live\|unit\|static` is unchanged. Store `e11f3e7a`. | paseo-room closing report. |
| D9 | New config `herding.paseo.on_model_mismatch`: `warn` (absent means this; paseo-pi-hardening D4 unchanged) or `refuse`. With `refuse`, a confirmed model or thinking mismatch stops the worker and the run returns a typed `model_mismatch` refusal naming configured and observed values. In both modes the run envelope carries `model_mismatch {configured, observed}`. Store `0ccaa387`. | paseo-room forbids silent route substitution; every study gather today ran `thinking off` against a configured `thinking medium`, seen only on stderr. |

### Agent's Discretion

Field spellings inside the JSON shapes, the fingerprint method in D7, and
where each doctrine line lands, as long as each fact keeps one home.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| reaction | The worker's stance toward the brief it was handed, not the run status. |
| falsifier | One concrete check that would show a recommendation is wrong; reported, not run. |
| fingerprint | HEAD sha plus a hash of `git diff HEAD` in the worker's worktree. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:516` — `MailboxResult`; `options`/`leaning`/`dissent` are the lenient-optional pattern D1 and D2 copy.
- `packages/bee-rs/crates/bee/src/herding/run.rs:5164` — `result_envelope`, re-emits optional fields only when present.
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:1038` — `transition_status`, write-on-transition dedupe D7 reuses for its one notice.
- `packages/bee-rs/crates/bee/src/herding.rs:825-860` — `compute_job_status`, where `stalled` is derived today.
- `packages/bee-rs/crates/bee/src/herding/run.rs:4058-4099` — `record_observed_model`, the D9 hook point.
- `packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs:780` — the append path D3 guards.
- `packages/bee/prompts/advisor.md` — the advisor digest contract D4 extends.
- `skills/bee-hive/references/gates-and-delegation.md:173-177` — blind lanes and convergence, D4 and D5.
- `skills/bee-reviewing/SKILL.md:47` — review inputs, D6.
- `skills/bee-swarming/references/swarming-reference.md:607-664` — the Result form and status tokens, D1 to D3 and D8.

### Established Patterns

- Optional result fields are lenient at parse and present-only in the envelope (slp-followup-gaps D3).
- One writer for the dissent record: `record_dissent` (slp-followup-gaps D4).
- Config absent means today's behavior, byte-identical (piggery-borrow D3).

## Canonical References

- `/mnt/data/Projects/refs/slp/paseo-room/protocol/handback.md`, `roles/lead.md`, `protocol/council.md`, `SUPERVISOR_NOTEBOOK.md:1062-1100`
- `/mnt/data/Projects/refs/slp/piggery/prompts/advisor/advisor.md`, `prompts/council/chair.md`, `internal/core/watch.go`
- `/mnt/data/Projects/refs/slp/paseo-doctrine-downstream/foundation/dist/docs/ASSIGNMENT_AND_HANDBACK.md`
- `/mnt/data/Projects/refs/slp/paseo-pi-team/prompts/peer.md:188-215`

## Outstanding Questions

### Deferred To Planning

- [ ] D7: where the run loop can read the worker's worktree path for a fingerprint, and the cheapest diff hash.
- [ ] D6: whether the head check is a reviewer prompt step or a `bee reviews` verb check.

## Deferred Ideas

- Per-role stall windows (piggery `silent_for` per role) — one value first.
- Fingerprint liveness of the installed binary (doctrine K) — backlog.
