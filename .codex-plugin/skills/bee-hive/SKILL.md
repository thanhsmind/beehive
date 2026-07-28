---
name: bee-hive
description: >-
  Bootstrap and route the bee workflow: gates, state, and the next skill. Use when starting or resuming any bee session, choosing the next bee skill, running go mode, checking onboarding state, or enforcing workflow gates.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: Onboarding and the vendored .bee/bin helpers run in Node.js 18+.
---

# hive

Load first in bee repos. Rules are stated bare — decision IDs:
`references/provenance.md`; quoted headings resolve in `references/routing-and-contracts.md`.

## Lanes — triage first (the mode gate)

Decide the lane from the request itself, before loading a second skill: count
risk flags and product files (`.bee/**`, docs, plans, generated renders never
count):

> auth · authorization · data model · audit/security · external systems ·
> public contracts · cross-platform · behavior an existing test asserts ·
> weaken/delete/replace existing proof · multi-domain (covered bugfix
> keeping tests green + adding one: 0 on the last two)

| Lane | Trigger (from the request alone) |
|---|---|
| `docs` | all touched files are knowledge, not runtime (docs, README, samples, plans) |
| `tiny` | 0–1 flags, ≤2 product files, no API/data change, one direct task |
| `spike` | one yes/no proof decides whether the plan is real |
| `small` | 0–1 flags, ≤3 product files, no gray areas |
| `standard` | 2–3 flags, story-sized behavior, or genuine row uncertainty |
| `high-risk` | 4+ flags or any hard-gate flag (auth, authorization, data loss, audit/security, external provider, validation removal) |

- docs/tiny/small: nothing more — merged shape+execution gate, one dispatched execution worker, no `bee-planning`; standard/high-risk: the normal chain with `bee-planning`.
- Uncertainty resolves downward, into loading more — never upward into skipping. One hard-gate flag is `high-risk` at one file; re-counting to land under a threshold = already `standard`.
- One re-lane checkpoint after the first evidence pass: measured demotion only, never twice, never with a hard-gate flag; promotion always open ("Re-lane checkpoint").
- Review is on demand — every lane closes `unreviewed` via scribing/compounding; one dispatched worker per lane, never in-session; `small` cells SERIAL; standard/high-risk: checklist judge per capped `behavior_change` cell ("Goal-check judge tier"); tiny/small fast path: preview cells in the merged Gate 2+3 question, persist after approval, orchestrator-authored done-report ("Lane ceremony in full").

## Onboarding

1. `node --version` below 18 or missing → stop (Node.js 18+ required).
2. From the bee source root: `node packages/bee/scripts/onboard_bee.mjs --repo-root <root> --json`.
3. `up_to_date` → continue; `changes_needed` → summarize, get approval, then `--apply` — never silent, never outside the BEE markers; `blocked_*` → zero mutations, surface `versions` to the user.

Incomplete onboarding → stop ("Onboarding Protocol"), ("Greenfield init lane").

## Session Scout

Preamble first — never re-fetch what it told you; `bee.mjs status --json` only
when routing work or the preamble is missing/stale.
HANDOFF (missing kind = pause): pause → present, **wait, never auto-resume**;
planned-next → adopt only at a fresh-session boundary; resumed/compacted
sessions wait like pause. One-line offers, never auto-run: capture-queue flush
· crash-recovery mining ("Crash recovery") · orphaned scribing debt · review
candidates (`high_risk_unreviewed` stated plainly). One hop away:
("Session Scout in full"), ("State layer reading order"),
("Worktree routing"), ("Delegation contract").

## Routing

Vague/new or in doubt → `bee-exploring` · clear scope or small fix →
`bee-planning` (tiny/small) · docs-only → docs lane ("Docs lane") · explicit
review request → `bee-reviewing` · merge/ship/release with unreviewed
candidates → report count + risk, ask ONE question — only an explicit yes
dispatches review · document an area / settled rule → `bee-scribing`
· backlog-triage pass (explicit-only) → `bee-qualifying` · `/go`
→ go mode (`references/go-mode.md`) · resume → surface HANDOFF, wait.
Briefing, grooming, compounding, skill-writing, evolving, and the
jump-to-planning offer: ("First-Skill Routing").

## The Four Gates

Never skipped, never batched, never self-approved — every mode, go and
headless included. Sole exception, the opt-in bypass level (`bee-bypass-gate`):
`normal` auto-approves Gates 1-3 for tiny/small/standard only
(high-risk/hard-gate, secrets, Gate 4 UAT still stop); `full` adds high-risk/hard-gate
Gates 1-3 (only secret reads and a review P1 stop); `total`: everything, zero
stops. At `full`/`total` the human lifted the high-risk floor — never re-erect a
removed stop ("Gate bypass mode"). Headless is not bypass — it stops at
every gate.

- Gate 1: "Decisions locked. Approve CONTEXT.md before planning?"
- Gate 2: "Work shape is ready. Approve before current-work preparation?"
- Gate 3: "Feasibility validated. Approve execution?"
- Gate 4: P1 > 0 → "P1 findings block merge. Fix before proceeding?"; P1 = 0 → "Review complete. Approve merge?"

Gate 4 exists only inside a user-invoked review session — never automatic
after execution, never after an unreviewed close; bypass never *creates* one;
`normal`/`full`: UAT items and any P1 always stop; `total` auto-proceeds.
`docs`: no gates; `tiny`/`small`: the merged question above; Gates 1-3
otherwise one at a time. Presentation: plain-language layer + fixed question;
report linked, never pasted; the user can restate it
("Gate Presentation Contract"); optional cross-model second opinion at Gates
2-4, never auto-resolved. CI status gate before the first `cells claim`, never a
local run: red CI / open `verify-red` → a fix-first tiny cell, **never build
on red**; impacted tests locally, full suite CI-owned ("CI status gate").

## Priority Rules (hive law)

Rules 2-4 and 13 appear in full in `AGENTS.md` (auto-loaded every session).

1. P1 review findings always block.
2. At ~65% context, write `.bee/HANDOFF.json` and pause.
3. `CONTEXT.md` is truth; locked decisions cited, never reinterpreted.
4. No source-editing execution before Gate 3.
5. Failed reality gate or NO spike → halt; back to planning.
6. Never skip validating; tiny = 2-minute reality check.
7. Critical patterns + recent decisions before planning/executing (Session Scout).
8. "done/passing/fixed" needs fresh command output in the same message.
9. Lanes scale ceremony, never memory: scribing sync per `behavior_change` cap; settlements captured as they settle; every close: a capture line or "nothing settled" ("Capture discipline").
10. The agent runs the machinery, never the user ("The agent runs the machinery").
11. Work language only; under bypass one outcome line per cap/slice/wave/re-lane ("Silent Bookkeeping"), ("Progress ticks").
12. No hand-edits to `.bee/*.json(l)`; CLI verbs only; `state set` needs `--owner`; no verb → file friction first.
13. Hooks are a safety net, never the authority — an unblocked write is not an approved write; never retry a blocked action (`AGENTS.md` Guardrails).
14. Headless: never ask; defer into `Outstanding Questions`; never self-approve a gate ("Headless mode").
15. Session-end nudge: ask for a durable decision/learning; log via `decisions log`.

## Red Flags

docs-only change through the full pipeline · a gate with no plain-language
layer · a gate the user cannot restate · a bee command handed to the user to
run. Violating the letter of the rules violates their spirit.

## Reference Map

| File | When to load |
|---|---|
| `references/routing-and-contracts.md` | Every exiled section — resolve any quoted heading here; plus skill catalog, first-skill routing, contracts, quick references |
| `references/go-mode.md` | `/go` runs: gate wording, slice loop, fallbacks, headless + bypass |
| `references/provenance.md` | Decision IDs + rationale for every body rule |

Session oriented. Invoke bee-<selected-skill> skill.
