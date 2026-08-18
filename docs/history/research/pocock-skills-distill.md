---
artifact_contract: bee-research/v1
topic: pocock-skills-distill
depth: deep
date: 2026-08-18
---

## Bottom Line

- Recommendation (ladder rung): reuse + adapt — bee already owns or
  outclasses ~85% of the pack (two files are near-1:1 ancestors of bee's
  own); distill ~6 one-paragraph nuggets into `.bee/expertise/` files
  and one synthesis-rule question for bee-reviewing. No new skill.
- Why lightest: every structural idea (wayfinder map, spike craft,
  design-it-twice, debugging loop ladder, deletion test) is already in
  bee, usually with more machinery (D-IDs, gates, evidence labels).
  The residue is sentences, not systems.
- Why next-best rung lost: porting any whole skill would duplicate an
  existing bee skill and re-introduce pocock's weakest property — no
  decision persistence beyond ADRs, no gates, no proof discipline.
- Confidence: 80 (two bee files unread: `.bee/expertise/architecture.md`,
  `.bee/expertise/review.md` — could already contain nuggets 4 and 6).
- Suggested next step: discussion; then one small docs feature if the
  nuggets are wanted.

## Source Manifest (xia)

| Field | Value |
|---|---|
| Repo or path | /home/thanhsmind/projects/AI/mattpocock-skill (github.com/mattpocock/skills) |
| Ref | HEAD |
| Resolved commit SHA | 84fdeff |
| Narrowed scope | skills/engineering/ — 18 skills, 36 md files |

## Dependency Matrix (per skill pair)

| Pocock skill | Bee equivalent | Verdict | Label |
|---|---|---|---|
| wayfinder | bee-wayfinding | EXISTS — bee is a port of it, plus D-ID log, frontier rounds, agent-suspected fog markers; pocock adds tracker-backed maps (GitHub/GitLab), bee chose files deliberately | Local |
| grill-with-docs + domain-modeling | bee-shaping (interview craft, pinned terms, gray-area probes) | EXISTS; nugget: `_Avoid_` rejected-terms list in glossaries (n7) | Local |
| to-spec | bee-shaping Lock / bee-capturing area specs | EXISTS — bee's Pointers-section rule is the stronger form of pocock's "no file paths in specs" | Local |
| to-tickets | bee-planning cells | EXISTS (walking skeleton = tracer bullet; deps = blocking edges); nuggets: expand–contract (n1), prefactor mantra (n9) | Local |
| implement + tdd | bee-swarming worker + `.bee/expertise/tests.md` | EXISTS — tests.md is deeper (independent-oracle, determinism leaks, fakes>stubs>mocks); nugget: pre-agreed-seams ritual (n5) | Local |
| code-review | bee-reviewing | EXISTS with stronger severity machinery; nuggets: two-axis Standards/Spec separation (n4), Fowler 12-smell named vocab (n4b) | Local |
| triage | bee route Qualify + backlog + decisions search | EXISTS (evidence-first, repro-before-verdict); pocock's `.out-of-scope/<concept>.md` KB is a sharper dedup surface for OSS triage — bee covers via decisions search | Local |
| diagnosing-bugs | `.bee/expertise/debugging.md` | EXISTS ~1:1 (loop ladder, 4 bars, DEBUG-tags, bisect axes all present); nugget: 3-5 ranked hypotheses before testing any (n2) | Local |
| research | bee-researching | EXISTS — bee far deeper (labels, ladder, xia); pocock's "follow every claim to the source that owns it" is spirit-equivalent of the Docs label | Local |
| prototype | planning spike craft + wayfinding spike rules | EXISTS ~1:1 (already ported); UI `?variant=` switcher recipe is web-app-specific, low value here | Local |
| improve-codebase-architecture | bee-grooming | EXISTS (deletion test, propose-approve, decline-with-reason); git-log hot-spot weighting worth checking against grooming-reference (n6b) | Local |
| ask-matt PHASE-BOUNDARIES | bee 65%-handoff + compact machinery | EXISTS; pocock's 5-option first-yes-wins tree and "summary flattened a decision" framing are sharper prose (n8, optional) | Local |
| wizard | — none | NEW capability class (human-only-steps bash wizard); not a skill-quality item — backlog idea at most | Local |
| resolving-merge-conflicts | — none (worktree merge assumes clean) | NEW but tiny; git craft, backlog idea at most | Local |

Cross-cutting sweep: the pack's wiring runs through
`docs/agents/issue-tracker.md` + setup skill (tracker abstraction) and a
shared vocabulary layer (domain-modeling, codebase-design) that other
skills import by reference. Bee's equivalents: onboarding/config and
`.bee/expertise/` + knowledge bundle. No hidden middleware. [Local]

## Distill — nuggets worth taking (ranked)

1. **Expand–contract for wide mechanical refactors** — "first expand:
   add the new form beside the old; migrate call sites in batches sized
   by blast radius; contract: delete the old form once no caller
   remains — green is promised only there." Missing from
   `.bee/expertise/planning.md` (build-order section). [Upstream]
2. **Multi-hypothesis before testing any** — generate 3–5 ranked
   falsifiable hypotheses; single-hypothesis generation anchors on the
   first plausible idea. `debugging.md` has the falsifiable-hypothesis
   shape but only singular. [Upstream]
3. **Cause-aligned options only after diagnosis** (also ak-brainstorm's
   nugget) — no fix menus from symptoms. Same `debugging.md` home.
   [Upstream]
4. **Two-axis review: Standards vs Spec, reported side by side, never
   merged or reranked** — bee's synthesis promotes/demotes into one
   ranked list; the axis separation prevents spec-conformance findings
   drowning under style findings. Needs the review owner's judgment +
   a read of `.bee/expertise/review.md` first. [Upstream]
   4b. Fowler 12-smell named vocabulary with "repo overrides; every
   smell is a judgement call, never a hard violation". Same check.
5. **Pre-agreed seams ritual** — before TDD: "what's the public
   interface, and which seams should we test?" — one confirming
   question, then tests only at those seams. `tests.md` picks the
   cheapest level but never asks the seam question up front. [Upstream]
6. **"One adapter is a hypothetical seam; two adapters is a real one"**
   + the deletion test phrasing — check `.bee/expertise/architecture.md`
   before merging; grooming already cites the deletion test. [Upstream]
   6b. Git-log hot-spot weighting for grooming scans — check
   `grooming-reference.md`.
7. **`_Avoid_` rejected-terms convention** — a pinned term lists the
   losing synonyms explicitly; scribing's Data Dictionary and shaping's
   pinned terms could carry it. One line each. [Upstream]
8. *(optional)* Phase-boundary five-option tree, first-yes-wins, with
   the primary/secondary-source framing. Bee's machinery covers the
   mechanics; only the prose is sharper. [Upstream]
9. **Prefactor mantra** — "make the change easy, then make the easy
   change" — one line beside expand–contract. [Upstream]

## Weaknesses (dở — what bee should NOT import)

- No decision persistence beyond optional ADRs; contract state lives in
  conversation and tracker labels — bee's decision log/D-IDs are the
  cure, not the disease. [Inference]
- No gates: "explicit autonomous execution may continue" — weaker human
  control than bee's approval model. [Local]
- No proof discipline: implement says "run the full suite once at the
  end"; bee's proof-line-per-cap + CI is stronger. [Local]
- Tracker-coupled (GitHub/GitLab/local trio) — surface bee deliberately
  keeps out of its core. [Local]

## Risks, Unknowns, Follow-Ups

- `.bee/expertise/architecture.md` and `.bee/expertise/review.md`
  unread — nuggets 4/4b/6 may already exist there; check before any
  merge cell.
- Open question (user): which nuggets to merge, and is the two-axis
  review change wanted at all?

## Source Pack

- Local: bee skills read this session (shaping, wayfinding, planning,
  swarming, researching, reviewing, grooming SKILL.md) + expertise
  digests (debugging, tests, planning, thinking) + gray-area-probes,
  shaping-reference.
- Upstream: mattpocock/skills @ 84fdeff — all 36 md files via three
  gather digests.
- Docs: none (not applicable).
