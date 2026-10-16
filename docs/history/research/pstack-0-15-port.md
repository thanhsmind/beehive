---
artifact_contract: bee-research/v1
topic: pstack-0-15-port
depth: standard
date: 2026-10-16
---

## Bottom Line

- Recommendation (ladder rung): **adapt-upstream** — five of the six candidate
  ideas port as one Rust change plus short doctrine lines; the sixth
  (effort-matched spawns) is deferred as a red flag against bee's door law.
- Why this is the lightest credible path: bee already owns the machinery each
  idea needs (the cap commit crosses the merge door, reflections are clustered
  by fix-at layer, the rule index has a `spoken:` line, interview craft has a
  single home). Every port is a field added to a struct that already exists or
  a sentence added to a section that already exists.
- Why the next-best rung lost: **reuse** cannot remove the `proof_stale`
  false positive after a rebase, cannot name a narrowed re-verify, and has no
  goal-playback step. **build** is unjustified: no new verb, no new store.
- Confidence: **85%**. Both sides read from source at pinned commits; the
  mapping is judgment against the anchors below.
- Suggested next step: **bee-shaping** — D-candidates listed under Challenge.

## Repo Snapshot

- This repo (`Local`): bee — Rust CLI (`packages/bee-rs`), plugin-packaged for
  Claude and Codex; skills under `skills/`, rule index under
  `docs/knowledge/areas/doctrine-layer/`, CLI-only state under `.bee/`.
- Source (`Upstream`): `cursor/plugins`, `pstack/` at commit
  `99743e9c151c8537cbbf1f98bb38f34ef3368453` (v0.15.25, 2026-10-15), local
  checkout `/mnt/data/Projects/refs/plugins/pstack`. Prior pins:
  `b9ddc83` (v0.14.5) in `docs/history/research/pstack-xia.md` and
  `pstack-playbooks-gap-check.md`. Range studied: `b9ddc83..99743e9`, 54
  pstack commits.
- Constraint that shapes the answer: bee's door law — the dispatch door picks
  the model, never the leader (`AGENTS.md`, "Work in parallel"); proof is
  checked, never re-run, at `bee close` and `bee worktree merge`.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `/mnt/data/Projects/refs/plugins` (`https://github.com/cursor/plugins.git`) |
| Ref | `origin/HEAD` (main) |
| Resolved commit SHA | `99743e9c151c8537cbbf1f98bb38f34ef3368453` |
| Narrowed scope | `pstack/skills/{correct,sharpen,in-your-own-words,principle-fix-root-causes,principle-test-behavior-not-implementation,principle-attack-the-premise,principle-make-operations-idempotent}/SKILL.md`, `pstack/skills/poteto-mode/SKILL.md` § Subagents, `pstack/skills/poteto-mode/playbooks/{shipping,autopilot-full,multi-phase-plan,bug-fix}.md` |

## Question & Assumptions

- What was asked: bring the pstack 0.15.x updates into bee "in the most
  sensible way".
- What success appears to mean: each idea lands where bee already keeps that
  kind of rule, with a check where a check is possible, and nothing that
  collides with the gate or door law.
- Assumption still needing confirmation: the user accepts a deferral for the
  effort-matching idea (F) rather than a door-law exception.

## Findings

### Dependency matrix

| # | pstack component (`Upstream`) | bee today (`Local`) | Status |
|---|---|---|---|
| A | Rules state their reason ("because …") — `ce649c0`, `e5136d8` | 20 of 27 `spoken:` lines in the rule index already carry a reason after an em dash (`router-triage-and-the-agents-md-duplication-boundary.md:185-385`); 7 do not. No `why:` field; parity test pins only `id`, `spoken:`, `applied_at:` (`tests/rule_index_parity.rs:95-218`). Class playbooks at `skills/bee-planning/playbooks/*.md` are step prose. | EXISTS (partial) |
| A2 | Four principles restated as reasoning, mechanism bans dropped — `e5136d8` | bee has 14 `bee-principle-*` skills; `red-before-green`, `test-behavior-not-structure` overlap two of the four. Whether they prescribe a mechanism is unchecked here. | EXISTS (unchecked) |
| B | Verdict bound to `git patch-id`; rebase keeps the verdict — `shipping.md:7` | Cap stores `report.commit`; merge computes `merge-base --is-ancestor` per commit and emits `proof_stale` advisory (`proof.rs:47-82`, `worktree/phases.rs:74-112, :908`). No patch-id anywhere (`rg` → 0). A rebase with the same change set reads as stale today — a false positive. | NEW (one field, one compare) |
| C | Full swarm once at code-ready head; rebase/minor fix gets one verifier — `autopilot-full.md:14` | Hat wave is once per feature (`gates-and-delegation.md:225`); judge tier per lane (`:406-410`); leader completeness check per cap. No rule says re-verify after a rebase is narrower (`rg 'rebase\|re-verify'` → 0 relevant). | NEW (one doctrine row) |
| D | `/correct`: mistake classes (twice = a class), fix at highest layer, prove each new check red on a past mistake, rule table pairs rule with its enforcer — `correct/SKILL.md:13-30` | `bee mailbox reflect --fix-at architecture\|check\|doctrine\|none` (`mailbox.rs:628`), digest clusters by `(fix_at, what)` (`mailbox_digest.rs:755-816`), bee-evolving ranks and gates (`SKILL.md:54-95`). No rule that a repeated reflection is a fix-first candidate; no red-first proof for a new guard; `check` folds types/lint/test. | EXISTS (partial) |
| D2 | "If the deciding fact took a deep dig, propose the cheap always-on signal" — `principle-fix-root-causes/SKILL.md:21` | Not in `skills/bee-planning/playbooks/bugfix.md` nor in `bee-principle-crash-site-versus-fault-site`. | NEW (one sentence) |
| E | `/sharpen` + `/in-your-own-words`: play back the goal, name over/under-specification, leave the user's choices open — `sharpen/SKILL.md:7` | Interview craft has six moves (`shaping-reference.md:6-20`); restate-in-own-words exists only as the gate litmus (`gates-and-delegation.md`). No intake playback, no specification-level move. | NEW (one craft bullet) |
| F | Per-spawn effort ±1 step, fast mode only when urgent — `poteto-mode/SKILL.md:97` | `{model, effort}` per role in `team.<runtime>` (`models.rs:896`); effort stamped into the payload (`prepare.rs:700`). No per-dispatch override; the door law forbids the leader hand-picking. | CONFLICT |

### Cross-cutting sweep

- `tests/rule_index_parity.rs` reads every rule row; a new field passes it
  unread (`Local`). A reason clause inside `spoken:` needs no test change.
- `verbs/worktree/tests.rs:925` pins that `proof_stale` never blocks
  (`Local`). The patch-id compare keeps that posture: it only turns a stale
  verdict back into a carried one.
- `FIX_AT_VALUES` is a closed vocabulary read by `cells finish`, `mailbox
  reflect`, the digest and `bee close`'s silence check (`Local`). Widening it
  is a public-contract change; D ports without widening it.
- Rendered skill copies under `.claude/skills/`, `.codex-plugin/` come from
  `bee dev regen`; every doctrine edit lands in `skills/` and regenerates.
- `.bee/verify/verify-app/features/` has no feature file for the merge
  advisory or the interview craft (checked by name); B needs a unit test, not a
  driven feature.

### Upstream

- `shipping.md:7` names the noise rule: a lane's result stands when the two
  builds differ only by what also differs between two builds of the same SHA.
  bee's proof is a recorded line, never re-run at the door, so only the
  patch-id half ports; the build-compare half does not apply.
- `e5136d8` thesis: "Restate each as the idea an agent needs to derive the
  call in cases the text never names." The before/after pairs are in
  `.bee/mailbox/job-1792161216958-2731699-1/report-1.md` § 6.

### Docs

Not applicable — both sides read from source. No web research performed.

### Inference

- B is the one item with a measurable defect today: a clean rebase makes the
  merge door say `proof_stale` for proof that still describes the code.
  `git patch-id --stable` of the recorded commit, compared against the
  patch-ids of the commits between the merge base and the branch head, turns
  that into `proof_carried`. Advisory stays advisory.
- C ports as one row in the judge-tier table: after a rebase, a conflict
  resolution, or a fix under N lines with no new behavior, re-verify is the
  recorded proof command re-run by the worker, not a fresh judge. The judge
  returns for new behavior, a redesign, or a large diff.
- D ports as doctrine on top of the existing store: a reflection key seen
  twice is a fix-first candidate at the highest layer that works; a new guard
  or test must be shown red on the past mistake before it lands (the
  `red-before-green` principle, extended to guards); `check` stays one word —
  the sub-kind (type, lint, test) rides in `--better`.
- A ports as: every `spoken:` line carries its reason (7 to fill), and the
  nine class playbooks gain a "because" clause where a step has none. A2 is a
  bounded read of the 14 principle skills for mechanism bans; expected small.
- E ports as one bullet in Interview craft, written for bee's short-reply
  contract: one line of playback, then the question.
- F is deferred: a per-dispatch effort knob is the hand-pick the model-guard
  hook refuses. The honest shape is a `team.<runtime>.<role>.effort` default
  per role (already supported) plus a backlog item for a bounded ±1 step the
  door itself applies from the cell lane.

## Challenge

| Question | Source answer | Local answer | Risk if wrong | Flag |
|---|---|---|---|---|
| Does patch-id make a stale proof block the merge? | A stale verdict is re-verified, not blocked | Advisory never blocks (`tests.rs:925`); patch-id only removes a false positive | Low — posture unchanged | green |
| Does "swarm once" weaken bee's per-cap check? | One full swarm per code-ready head | Completeness check still runs per cap; only the post-rebase re-verify narrows | Medium — a "minor fix" must be defined by diff, never by claim | green with a size bound |
| Does /correct need a new verb? | A skill reads commits and reverts | Digest already clusters by fix-at; the rule rides bee-evolving and the cap | Low | green |
| Does `/sharpen` collide with "A question is a question" or short replies? | Playback is silent framing, "don't hand back a rewritten prompt" | One-line playback before the first question; no rewrite returned | Low | green |
| Does per-spawn effort collide with the door law? | Leader adjusts effort ±1 per spawn | Leader never hand-picks; the hook rewrites | High — breaks the one-door invariant | **red → defer** |
| Does "rules with reasons" bloat the always-loaded contract? | Reasons live in playbooks loaded at the step | Reasons go in `spoken:` lines and playbooks, never in `AGENTS.block.md` | Low | green |

## Risks, Unknowns, Follow-Ups

- Unverified: `git patch-id` cost on long branches — one `git log -p` per
  merge; acceptable, but measure once.
- Open question for shaping: the "minor fix" bound for C (proposed: no new
  behavior and under 30 changed lines).
- Open question for shaping: whether A2 (principle restatement) is in scope
  now or a follow-up read.
- Deferred: F as backlog `effort-by-lane-at-the-door`.

## Source Pack

- Local files read: `docs/history/research/pstack-xia.md`,
  `pstack-playbooks-gap-check.md`, `docs/history/pstack-adoption/CONTEXT.md`,
  `docs/knowledge/work/pstack-gaps/delivery.md`, the two gather reports under
  `.bee/mailbox/job-1792161216958-2731699-1/` and `job-1792161255412-2734872-1/`.
- Upstream: `cursor/plugins` `pstack/` at `99743e9`, commits `ce649c0`,
  `e5136d8`, `36cecbf`, `79afd75`, `7d8d909`, `9511e60`.
- Docs pages checked: none.
