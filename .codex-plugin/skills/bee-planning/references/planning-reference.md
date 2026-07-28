# Planning Reference

Use when `bee-planning` needs artifact templates, cell quality rules, or shape guidance.

## Artifact fan-out — separate files are earned, not default (decision 0009)

`plan.md` is the truth artifact for **standard/high-risk** lanes (and small only when opt-in). Tiny drops it entirely and small skips it by default (D3/D4) — the cell(s) and the logged scoping synthesis carry the shape. Where `plan.md` exists, discovery and approach content default to **sections inside it**; they graduate to their own files only when real complexity makes a standalone file worth reading on its own. The dogfood lesson: a small/standard feature that spawned `discovery.md` + `approach.md` + `plan.md` + `implement-plan.md` restated the same "current state" four times.

| Artifact | Separate file when | Otherwise |
|---|---|---|
| `plan.md` | standard/high-risk (frozen at Gate 2), or small when a durable multi-slice/product-decision doc is genuinely needed | **tiny: none** — request + one cell (D3); **small: opt-in** — a logged scoping synthesis + 1–3 cells is the default (D4) |
| `discovery.md` | discovery ran at **L2/L3** (a real multi-candidate comparison worth preserving) | a `## Discovery` note inside `plan.md` (L0/L1 findings, cited) |
| `approach.md` | **high-risk** lane, or discovery **L2+** (rejected alternatives + a risk map substantial enough to stand alone) | an `## Approach` section inside `plan.md` |
| `implement-plan.md` (via `bee-briefing`) | **high-risk** (mandatory) | **standard**: on-demand — `plan.md` + the Gate 2 chat layer are the review record; render only if the user asks or the slice spans multiple domains. `small`: optional mini-brief on request. `tiny`/`spike`: none |

Rule of thumb: if the separate file would just repeat what `plan.md` already says, it should have been a section. Fold, don't fan out.

## Artifact: approach.md (only when the fan-out table calls for a separate file)

```markdown
# Approach: <Feature>

## Recommended path
<2–5 sentences: what we build and in what order. Cites D-IDs.>

## Rejected alternatives
- <alternative> — <why rejected, one line>

## Risk map
| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| <area> | LOW/MEDIUM/HIGH | <why> | <command, inspection, or spike question> |

## Files and order
<bounded list, likely touch order>

## Relevant learnings
- <docs/history/learnings file or decision id> — <what it changes here>

## Questions for validating
- <assumption that could invalidate the path>
```

## Artifact: plan.md (standard/high-risk; small only when opt-in)

Frozen at Gate 2 (D1): once `approved_gates.shape` is set the content sections are immutable — the only permitted post-approval write is the approval stamp in the frontmatter. No requirements-only→implementation-ready mutation, no in-place enrichment.

```markdown
---
artifact_contract: bee-plan/v1
mode: standard | high-risk | spike | small (opt-in)
# approved_gate2: <unset until Gate 2; then a date stamp — the only permitted post-approval write>
---

# Plan: <Feature>

Mode: `<mode>` — <k> risk flags: <list, or "none">
Why this is the least workflow that protects the work: <one sentence>

## Requirements (from CONTEXT.md)
- D1: <locked decision restated> ...

## Discovery
<Only when discovery ran at L0/L1 and there is no separate discovery.md.
2–4 lines: what was inspected, the finding, the evidence command. At L2/L3 this
becomes its own discovery.md and this section is a one-line pointer to it.>

## Approach
<Only when the fan-out table keeps approach in plan.md (not high-risk, not L2+).
Recommended path (cites D-IDs) · rejected alternatives (one line each) · a compact
risk map (component / LOW-MEDIUM-HIGH / proof needed). When approach.md exists as
its own file, drop this section and point to it instead.>

## Shape
<one of the bodies below, by mode>

## Test matrix
<edge dimensions that apply (see edge-dimensions.md), at lane depth:
tiny/small = the 2–3 dimensions that bite; standard = one pass over all 12;
high-risk = probes written out per dimension>

## Out of scope
<explicitly not solved; deferred ideas stay deferred>
```

No `## Current slice` / `## Cells` sections are added post-approval: the plan is frozen (D1) and the current slice lives **only in cells** (D2). Prep creates the cells; the plan is never re-opened to index them.

**Shape bodies by mode:**

- `tiny` — no plan.md (D3): the cell `action` is the micro-plan (current work outcome, proof command, out of scope).
- `small` — no plan.md by default (D4): a logged scoping synthesis + 1–3 cells carry the shape; write a plan.md only when opt-in (durable multi-slice/product-decision doc).
- `spike` — the one yes/no question, what proves YES, what NO implies, `.bee/spikes/<feature>/` location.
- `standard` (milestone-shaped) — **phase plan**: `Phase | What Changes | Why Now | Demo | Unlocks` table. First phase obvious; later phases build on it; no technical buckets ("backend", "frontend" are not phases).
- `standard` / `high-risk` (capability/risk-shaped) — **epic map**: feature outcome, repo-reality basis, `Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed` table, slice queue with deps and feasibility status, current slice to prepare.

## Phase plan vs epic map

Use **phases** only when the work has observable milestones a user could demo in order. Use an **epic map** when capability or risk areas explain the work more honestly than a timeline — typical for `high-risk` (it defaults to epic map + mandatory feasibility proof). Never force 2–4 phases onto work that is really one slice, and never use phases as architecture layers.

## Cell quality rules

A cell is an executable prompt a cold worker can pick up with zero session history.

1. **Directive action, no code blocks.** Prose that says what to do and cites decisions (`per D2`). Code belongs in the repo, written by the worker.
2. **Bounded files.** `files` lists everything the worker may write; `read_first` what it must read. A worker touching other paths returns `[BLOCKED]`. Cross-cell file overlap is legal, not a scoping error — it only costs a wave (auto-serialized per D2); prefer explicit paths or trailing-`*` patterns in `files`, since overlap detection uses `pathsOverlap` semantics (mid-path globs are treated as literals, not wildcards).
3. **Testable exit.** `verify` is a real command that runs in this repo today. "Manually check" is not a verify.
4. **must_haves are contracts:** `truths` (observable behavior), `artifacts` (path + substantive description — no stub counts), `key_links` (wired, not just existing), `prohibitions` (what must NOT change). Required for `standard` and `high-risk` lanes; `tiny` may omit.
5. **behavior_change honesty.** Any cell changing observable behavior is `behavior_change: true` — the cap helper refuses these without both verification evidence and a "before" characterization (`red_failure_evidence`), and reviewing double-checks it; mislabeling is a P1 waiting to happen.
6. **Deps are real.** `deps` lists cell ids whose output this cell needs. Ready = all deps capped.
7. **Current slice only.** If you can write the cell without knowing the previous slice's outcome, fine; if it belongs to a later slice, it does not exist yet.
8. **Evidence lives in the trace — do not manufacture evidence artifacts (decision 0009).** Never add a `reports/execution-*-evidence.md` or `reports/<cell>-evidence.json` to a cell's `must_haves.artifacts`. Verification evidence belongs in the cell trace (`verification_evidence` + `verify_output`), which is the single source; requiring a parallel evidence file duplicates it and inflates the doc set. `artifacts` are the *product* the cell builds (a source file, a spec, a migration) — not a record of the cell having run.

## Example cell JSON (schema per docs/02-architecture.md)

```json
{
  "id": "auth-3",
  "feature": "auth",
  "title": "Wire session middleware into API router",
  "lane": "standard",
  "status": "open",
  "deps": ["auth-1", "auth-2"],
  "decisions": ["D2", "D4"],
  "files": ["src/api/router.ts", "src/auth/middleware.ts"],
  "read_first": ["src/api/router.ts"],
  "action": "Mount the session middleware from auth-2 onto all /api/* routes (per D2). Preserve the existing public response envelope (per D4). Follow the error-handler registration pattern already used in router.ts.",
  "must_haves": {
    "truths": ["Unauthenticated /api/* requests return 401"],
    "artifacts": [{"path": "src/auth/middleware.ts", "substantive": "exports authGuard, no TODO stubs"}],
    "key_links": ["router.ts imports and mounts authGuard"],
    "prohibitions": ["No change to public response envelope"]
  },
  "verify": "npm test -- auth",
  "trace": {
    "worker": null, "outcome": null, "files_changed": [],
    "deviations": [], "friction": null, "capped_at": null,
    "behavior_change": true, "verification_evidence": null
  }
}
```

Create with one batched stdin call for the whole slice (a JSON array; a single object works for a one-cell slice — do not write per-cell scratchpad files; the one canonical scratch home, if one is ever genuinely needed, is `.bee/tmp/<feature-or-session>/`, docs/specs/doctrine-layer.md R17):

```bash
node .bee/bin/bee.mjs cells add --stdin <<'EOF'
[ { ...cell 1... }, { ...cell 2... } ]
EOF
```

A batch is all-or-nothing: every cell is validated (including duplicate ids within the batch) before any is written. The helper validates id, feature, title, lane, action, verify — and non-empty `must_haves.truths` for `standard`/`high-risk`. Fix rejects; never downgrade the lane to dodge validation.

## Trace of shapes

```text
mode -> shape (plan.md, frozen at Gate 2) -> [GATE 2] -> cells
```

(Tiny/small have no plan.md in this trace: `mode -> draft-cell preview + reality check -> [merged gate] -> cells`, D3/D4/D5.)

## Verify scoping

**Verify is scoped, not full (verify-scoping D2, decision `20534ea9`).** A cell's `verify` command is the narrowest honest check covering its change — the specific test file(s) or pattern for the touched area, never the full configured test/verify command by default. In this repo that means `node scripts/run_verify.mjs --only <suite>` or the direct test file, not the full `commands.verify` chain. The full configured verify is never owed locally and never authored as a per-cell `verify` (ci-owned-verify D1/D6, superseding the three-moments rule above): CI runs it on the project's own CI cadence (push, nightly, or scheduled — the host workflow decides) and auto-files a `verify-red` issue when red. The dev loop's own broader check is `commands.test` (`node scripts/run_verify.mjs --impacted` / `--impacted-from-git`), selected from the impact registry (`impact_registry.mjs --query`) — narrower than the full chain, wider than one `--only` suite, and still never authored as a per-cell `verify`. Mid-iteration, run the level-1 impacted run (`run_verify --impacted-from-git --level 1` — direct edges only, seconds); the transitive impacted run (`commands.test`) stays the wave-close/merge gate. In a repo that has declared itself no-test (`commands.verify`/`commands.test` set to the sentinel `"none"`, decision 55b951e1), a cell's own `verify` may itself be `"none"` — author it only there, never invent a fake check to satisfy the field, and never carry the sentinel into a repo that has real commands.

## Greenfield init lane

**Greenfield init lane (P1, docs/09 item 6):** when the repo has no build and the init-lane offer was accepted at onboarding, the first slice is **one init cell** — `must_haves`: setup succeeds from scratch, one passing test exists, standard commands recorded in `.bee/config.json`, clean first commit — before any feature cell. Infrastructure first; the init cell's verify command is the recorded `test` command itself.

## Lane-scaled bootstrap in full

Bootstrap scaled to the lane the mode gate just picked — never a full context sweep before the lane is known:

- **tiny:** the targeted reads only (the ≤2 from intake), plus the mandatory critical-patterns digest already in the preamble. With a `bee.work-item` concept in `docs/knowledge/`, run the knowledge-context read too, scaled to a tiny read.
- **small:** bounded bootstrap — `CONTEXT.md` if one exists + recent decisions (`node .bee/bin/bee.mjs decisions active --recent 3`). Same knowledge-context read as standard/high-risk when the feature has a work item.
- **standard / high-risk:** full bootstrap, in order:
  1. **Area truth first — the reading order is `bundle → decisions → history` (G4).** When the repo has a knowledge bundle (`bundleMode`: `docs/knowledge/` holds at least one concept that parses), read `docs/knowledge/areas/<area>/` for every area the work touches — `index.md` names the concepts, each concept states the subject it is authoritative for — and `bee knowledge context --work <feature>` when the feature has a work item. `docs/specs/` is named here for one job only: the read-only compatibility surface that resolves a legacy `docs/specs/<area>.md#R7` citation through its pointer stub to the concept that owns that anchor now; it is never the place to read current truth, and never the place to write it (`scripts/okf_specs_fence.mjs`, G2). **When there is no bundle, today's guidance stands verbatim:** read `docs/specs/<area>.md` before its code, with `docs/specs/reading-map.md` for "where does X live" before any broad grep.
  2. `docs/history/<feature>/CONTEXT.md` (or the hive scoping synthesis for surface-scope-earlier work).
  3. The critical patterns — with a bundle, `docs/knowledge/index.md`'s `## Critical patterns` section; with no bundle, `docs/history/learnings/critical-patterns.md` — already digested from the preamble; re-read for the feature's area as needed.
  4. Recent decisions: `node .bee/bin/bee.mjs decisions active --recent 3`, then recall for this feature's area through the structured filters and the derived index (decision-propagation D7/D8) — `node .bee/bin/bee.mjs decisions search --tag <tag>` / `--scope <area>` (multi-term `--text` is OR-ranked; `--all` reaches the archive), and the area's section of `docs/decisions/index.md` as the complete-by-construction recall surface.
  5. Tag-matched precedent in `docs/history/learnings/` (grep for the feature's domain keywords). Inject hits as "we've solved X before: <file>"; precedent beats research.
  6. Session scout: `node .bee/bin/bee.mjs status --json`.
  7. **Re-lane checkpoint — only when exploring was skipped** (it spends the one checkpoint otherwise): measured evidence may demote `standard` to `small` once — files within threshold, zero hard-gate flags, zero open gray areas, all three. Never `tiny`, never twice. Log it, tick it. Rule: `bee-hive/references/routing-and-contracts.md` ("Re-lane checkpoint").

## Discovery in full

Pick the lowest level that removes real uncertainty:

- **L0 — skip:** pattern already exists in repo or learnings; cite it.
- **L1 — quick verify:** confirm one API/version/behavior with a command or doc check.
- **L2 — standard:** compare 2–3 candidate approaches; note trade-offs.
- **L3 — deep dive:** unfamiliar territory, external systems, or hard-gate flags.

At L2+, invoke `bee-xia` in-chain: local truth → local reuse → upstream patterns → version-aware docs, evidence labels on every claim, and the anti-reinvention ladder (reuse → built-in → adapt upstream → build) for the recommendation; its findings merge into the approach, never a standalone research file. §2 Lane-scaled bootstrap (area truth, CONTEXT, critical-patterns, decisions, learnings grep, status) delegates as an extraction-tier I/O worker per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`); other ad-hoc research dispatches during discovery (including bee-xia) default to the generation slot model; ceiling requires the [bee-tier: ceiling] marker plus a one-line justification. Frame candidates through **three layers of knowledge**: tried-and-true (what the repo/ecosystem already trusts), new-and-popular (current mainstream, verify version claims), first-principles (what the problem actually requires). Recommend from evidence, not novelty.

**Artifact fan-out (decision 0009).** Only **L2/L3** discovery earns a separate `docs/history/<feature>/discovery.md` (a real multi-candidate comparison worth reading alone). At **L0/L1**, record the finding in `plan.md`'s `## Discovery` note and cite it — do not spawn a discovery file that just restates the current state `plan.md` already carries.

## Gate 2 bypass mechanics

**Gate-bypass check FIRST** (routing-and-contracts.md §Gate bypass, decisions 0010/dcf01d7b). Read the active level (`node .bee/bin/bee.mjs status --json` → `gate_bypass_level`). If it bypasses Gate 2 for this lane — `normal` covers `tiny`/`small`/`standard` non-hard-gate; `full`/`total` cover **every** lane incl. high-risk/hard-gate — then **DO NOT ask.** Take the shaped plan as approved (the recommended path), set `approved_gates.shape` yourself (`bee.mjs state gate --name shape --approved true`), stamp the plan frontmatter with the approval date (the only permitted post-approval write, per D1), log a one-line audit decision, post `⚡ auto-approved Gate 2 (bypass) — preparing cells`, and continue straight to §6 Prep. Only present the question below when the level does NOT cover this gate. Present **Gate 2** per the Gate Presentation Contract (bee-hive routing reference): plain-language layer in chat — what I plan to build / why this size / cost if the shape is wrong / what you are deciding — in the user's language, the review document linked not pasted; then verbatim: "Work shape is ready. Approve before current-work preparation?" — then **stop**. No pseudo-cells in markdown, no prep, no cells.

## Tiny/small merged gate

**Preview before persist (D5).** For `tiny` and `small`, the ordering is inverted so the approval covers the exact work packet: **draft the cell(s) and run the validating reality check FIRST**, before the merged shape+execution question. The draft cell(s) are rendered as a **preview in the gate message** (never persisted first); the reality check — MODE FIT / REPO FIT / ASSUMPTIONS / SMALLER PATH / PROOF SURFACE, each one line of file/command evidence, 2 minutes not a report — runs inline. Then present **one merged question** in place of Gates 2 and 3: "Work shape + execution: I'm about to do [X] via [Y], verified by [Z]. Approve?" The approval covers the **exact previewed work packet**; `cells add` runs only **after** approval and the cells are claimed only then — **never persist-then-preview**. Execution approval is never granted before the execution package exists. Approval records **both** `approved_gates.shape` and `approved_gates.execution`. **Under any active bypass level** (tiny/small are always covered — even `normal`), do NOT ask the merged question: the reality check still runs (bypass changes only whether the question is asked, never whether the check runs), the draft-cell preview goes into the auto-approval audit line, and if the reality check PASSES, set both `approved_gates.shape` and `approved_gates.execution` yourself, log one audit decision, post `⚡ auto-approved shape+execution (bypass)`, then persist the cells and continue to bee-swarming. Only a reality-check FAIL is surfaced to the human regardless of bypass, and it is presented before asking, never buried. `bee-validating` is not separately invoked for these lanes; its subagents (plan-checker, cell reviewer) do not run — the cell(s) are what a stranger picks up with zero session history, and the cold-pickup criteria are self-checked when writing them.

## Slice-tail test batching in full

**One trailing test cell per slice (slice-tail-test-batching P2, spec #80/#85).** Whenever the slice holds ≥1 `change_class: 'behavior'`/`'api'` cell **that touches code** — instruction/knowledge text (`skills/`, `docs/`, plans, `.md`) is not code and owes no test — emit **exactly one** `change_class: 'test'` cell, last, with `deps` naming **every** implementation cell of the slice. Its `action`: the slice's **net behavior** — happy path, edge cases, error paths — over the declared surfaces, never per-cell internals; `verify` is the targeted suite over the slice's scope. Implementation cells no longer author tests (they cap on existing-green), so a code-touching slice with no test cell is a planning defect. Test economy relocates here unchanged: D5 read-first applies **at this cell**, D3's `new_suite_reason` still governs new files, and D3's ratio ceilings compute against the slice's **aggregate** source delta, not per cell. `bugfix` and `high-risk` cells keep per-cell red-first/repro-first and are not batched.
