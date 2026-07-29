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

Decide the lane from the request, before loading a second skill: count
risk flags. Lane caps count product files only — `.bee/**`, docs, plans,
generated renders never count:

> auth · authorization · data model · audit/security · external systems ·
> public contracts · cross-platform · changes behavior an existing test asserts ·
> weakening, deleting, or replacing existing proof · multi-domain (covered
> bugfix keeping tests green + adding one: 0 on the last two)

| Lane | Trigger (from the request alone) |
|---|---|
| `docs` | all touched files are knowledge, not runtime (docs, README, samples, plans) |
| `tiny` | 0–1 flags, ≤2 product files, no API/data change, one direct task — cell is the micro-plan |
| `spike` | one yes/no proof decides whether the plan is real — opt-in by change class, never a phase step |
| `small` | 0–1 flags, ≤3 product files, no gray areas — logged scoping synthesis; plan.md is opt-in |
| `standard` | 2–3 flags, story-sized behavior, or genuine row uncertainty |
| `high-risk` | 4+ flags or any hard-gate flag (auth, authorization, data loss, audit/security, external provider, validation removal) |

- Record same turn: `state route --set` — `Route: class=<c> | lane=<l> | flags=<n> [<names>] | files=<n>`; re-lane updates in place ("Route record").
- docs/tiny/small: nothing more — merged shape+execution gate, one dispatched execution worker, no `bee-planning`; standard/high-risk: the full chain.
- Uncertainty resolves downward, never up into skipping. One hard-gate flag = `high-risk` at one file; re-counting to dodge a threshold = already `standard`.
- One re-lane checkpoint after first evidence: measured demotion only, never twice, never with a hard-gate flag; promotion always open ("Re-lane checkpoint").
- Review is on demand — every lane closes `unreviewed`; one dispatched worker per lane; `small` SERIAL; standard/high-risk: goal-check judge per capped `behavior_change` cell ("Goal-check judge tier"); tiny/small: preview-then-persist, orchestrator-authored done-report ("Lane ceremony in full").

## Onboarding

1. `node --version` below 18 or missing → stop.
2. From the bee source root: `node packages/bee/scripts/onboard_bee.mjs --repo-root <root> --json`.
3. `up_to_date` → continue; `changes_needed` → summarize, get approval, then `--apply` — never silent/outside BEE markers; `blocked_*` → zero mutations, surface `versions` to the user.

Incomplete onboarding → stop ("Onboarding Protocol"), ("Greenfield init lane").

## Session Scout

Preamble first — never re-fetch what it told you; `bee.mjs status --json` only
when routing work or the preamble is missing/stale.
HANDOFF (missing kind = pause): pause → present, **wait, never auto-resume**;
planned-next → adopt only at a fresh-session boundary; resumed/compacted
wait like pause. One-line offers, never auto-run: capture-queue flush
· crash-recovery mining ("Crash recovery") · orphaned scribing debt · review
candidates (`high_risk_unreviewed`). More:
("Session Scout in full"), ("State layer reading order"),
("Worktree routing"), ("Delegation contract").

## Routing

Vague/new or in doubt → `bee-exploring` · clear scope or small fix →
`bee-planning` (tiny/small) · docs-only → docs lane ("Docs lane") · explicit
review request → `bee-reviewing` · merge/ship/release with unreviewed
candidates → report count + risk, ask ONE question — only an explicit yes
dispatches review · document an area / settled rule → `bee-scribing`
· backlog-triage pass (explicit-only) → `bee-qualifying` · `/go`
→ go mode (`references/go-mode.md`) · resume → surface HANDOFF, wait
· busy + disjoint paths → lane not wait; own checkout → worktree
("Concurrency law in full").
Briefing, grooming, compounding, skill-writing, evolving,
jump-to-planning: ("First-Skill Routing").

## The Gates

Never skipped, batched, or self-approved — any mode, headless
included. Sole exception, the opt-in bypass level (`bee-bypass-gate`):
`normal` auto-approves Gates 1-2 for tiny/small/standard
(high-risk/hard-gate, secrets, Gate 4 UAT still stop); `full` extends to
high-risk too (secret reads, review P1 still stop); `total`: everything,
zero stops. `full`/`total` lift the high-risk floor — never re-erect it
("Gate bypass mode"); headless stops regardless, not bypass.

- Gate 1: "Decisions locked. Approve CONTEXT.md before planning?"
- Gate 2: "Work shape is ready. Approve before current-work preparation?" — approves `shape` AND `execution` together (`--merge`, D2); Gate 3 is retired.
- Gate 4: P1 > 0 → "P1 findings block merge. Fix before proceeding?"; P1 = 0 → "Review complete. Approve merge?"

Gate 4 exists only inside a user-invoked review session — never automatic
after execution, never after an unreviewed close; bypass never *creates* one;
`normal`/`full`: UAT items and any P1 always stop; `total` auto-proceeds.
`docs`: no gates; `tiny`/`small`: the merged question above; Gates 1-2
otherwise one at a time. Presentation: plain-language layer + fixed question;
report linked, never pasted; the user can restate it
("Gate Presentation Contract"); optional cross-model second opinion at Gates
2/4, never auto-resolved. CI status gate before the first `cells claim`: red
CI / open `verify-red` → a fix-first tiny cell, **never build on red**;
impacted tests locally, full suite CI-owned ("CI status gate").

## Priority Rules (hive law)

Rules 2-4, 12 are in `AGENTS.md` (auto-loaded).

1. P1 review findings always block.
2. At ~65% context, write `.bee/HANDOFF.json` and pause.
3. `CONTEXT.md` is truth; locked decisions cited, never reinterpreted.
4. No source-editing execution before Gate 2 approves execution.
5. Failed SMALLER PATH check or a NO spike → halt; redraft before the gate.
6. Critical patterns + recent decisions before planning/executing (Session Scout).
7. "done/passing/fixed" needs fresh command output in the same message.
8. Lanes scale ceremony, never memory: scribing sync per `behavior_change` cap; capture on settle; every close: a capture line or "nothing settled" ("Capture discipline").
9. The agent runs the machinery, never the user ("The agent runs the machinery").
10. Work language only; every perceivable step emits one tick line, on by default (rule: `AGENTS.md` 17; "Silent Bookkeeping", "Progress ticks").
11. No hand-edits to `.bee/*.json(l)`; CLI verbs only; `state set` needs `--owner`; no verb → file friction first.
12. Hooks are a safety net, never the authority; never retry a blocked action (`AGENTS.md` Guardrails).
13. Headless: never ask; defer into `Outstanding Questions`; never self-approve a gate ("Headless mode").
14. Session-end nudge: ask for a durable decision/learning; log via `decisions log`.

## Red Flags

docs-only change through the full pipeline · a gate with no plain-language
layer · a gate the user cannot restate · a bee command handed to the user to
run. Violating the letter of the rules violates their spirit.

## Reference Map

| File | When to load |
|---|---|
| `references/routing-and-contracts.md` | Every exiled section — resolve quoted headings here; skill catalog, first-skill routing, contracts, quick references |
| `references/go-mode.md` | `/go` runs: gate wording, slice loop, fallbacks, headless + bypass |
| `references/provenance.md` | Decision IDs + rationale for every body rule |

Session oriented. Invoke bee-<selected-skill> skill.
