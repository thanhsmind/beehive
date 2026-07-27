---
name: bee-planning
description: >-
  Research the work, pick the smallest honest mode, and shape an executable plan. Use when exploring has locked CONTEXT.md, or a clear-scope task needs a mode decision and work shape before validation.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies: []
---

# planning

If `.bee/onboarding.json` is missing or stale, stop and invoke `bee-hive`.

Planning is the waggle dance: it turns locked `CONTEXT.md` decisions into the smallest believable path to execution — mode, approach, the right work shape for the lane (a frozen `plan.md` for standard/high-risk; cells alone for tiny/small), and (only after approval) current-slice cells.

Load `references/planning-reference.md` for artifact templates and cell quality rules; `references/edge-dimensions.md` for the test matrix. Discovery at L2/L3 runs through `bee-xia`.

## Hard Gates

- `CONTEXT.md` is the source of truth. Locked decisions are cited (`per D2`), never reinterpreted, never scope-reduced.
- **Stop at Gate 2.** No cell creation, no prep artifacts before the shape is approved.
- **Once approved, `plan.md` is frozen (D1)** — the only permitted post-approval write is an approval stamp, never a content edit.
- Cells for the **current slice only**. Future-slice cells are prohibited.
- Handoff only to `bee-validating` (standard/high-risk) or `bee-swarming` (tiny/small, merged gate).

## 1. Intake & Mode Gate (mechanical — first, per D8)

Classify before you bootstrap. From the **request text plus at most 2 targeted file reads**, run a cheap **intake classification** — the Mode Gate below — to pick the lane. Only then do the lane-scaled bootstrap (§2). Tiny work must not pay full context reads before it knows it is tiny; this deliberately supersedes the old unconditional full-bootstrap-first ordering (D8). AGENTS.md startup step 6 (read the critical patterns before planning) still applies — and since f4-5 it is itself bundle-branched: `docs/knowledge/index.md`'s `## Critical patterns` section with a bundle, `docs/history/learnings/critical-patterns.md` without.

The **critical-patterns digest stays mandatory in every lane, tiny included** — it is already surfaced in the session preamble, so honoring it costs no extra read. D8 rescopes only the *additional* bootstrap reads, never the patterns digest.

Count risk flags — do not vibe it:

> auth · authorization · data model · audit/security · external systems · public contracts · cross-platform · changes behavior an existing test asserts (a covered contract must change) · the change requires weakening, deleting, or replacing existing proof · multi-domain

The last two flags are narrowed (D7): "changes behavior an existing test asserts (a covered contract must change)" replaces the old "existing covered behavior", and "the change requires weakening, deleting, or replacing existing proof" replaces "weak proof around the area". A covered bugfix that keeps existing tests green and adds a new one scores **0** on both. All other flags and the thresholds below are unchanged.

**Lane file caps count product files only (D6)** — production source, tests, and runtime config the behavior change itself must touch. **Never counted:** `.bee/**`, `docs/**` (history, specs, backlog), plans/briefs/reports, and generated projections/manifests (plugin renders, release manifest). Bee's own artifacts never inflate a fix past its lane.

- **Every touched file is knowledge, not runtime** (docs/, specs, README, sample/example configs, plans) → `docs` lane: exit planning — announce one line, write it, format-check, capture per bee-hive. No plan.md, no cells, no gates.
- **0–1 flags** → `tiny` (≤2 product files, one direct task) or `small` (≤3 product files, no gray areas)
- **2–3 flags** or story-sized behavior → `standard`
- **4+ flags or any hard-gate flag** (auth, authorization, data loss, audit/security, external provider, validation removal) → `high-risk`
- One yes/no proof decides whether the plan is real → `spike` (regardless of flags)

The mode gate **re-runs upward** the moment evidence demands escalation; de-escalation requires cited evidence.

Where the mode-gate record lands (flag count + product-file count + lane choice): **tiny** records it in the cell itself (its `action`/notes) — there is no plan.md (D3); **small** records it in the logged scoping-synthesis decision (D4); **standard/high-risk** record it in `plan.md`. Above `small`, state why smaller modes are insufficient. Use the least workflow that honestly protects the work.

**Greenfield init lane (P1, docs/09 item 6):** when the repo has no build and the init-lane offer was accepted at onboarding, the first slice is **one init cell** — `must_haves`: setup succeeds from scratch, one passing test exists, standard commands recorded in `.bee/config.json`, clean first commit — before any feature cell. Infrastructure first; the init cell's verify command is the recorded `test` command itself.

## 2. Lane-scaled bootstrap (per D8)

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

## 3. Discovery (research levels)

Pick the lowest level that removes real uncertainty:

- **L0 — skip:** pattern already exists in repo or learnings; cite it.
- **L1 — quick verify:** confirm one API/version/behavior with a command or doc check.
- **L2 — standard:** compare 2–3 candidate approaches; note trade-offs.
- **L3 — deep dive:** unfamiliar territory, external systems, or hard-gate flags.

At L2+, invoke `bee-xia` in-chain: local truth → local reuse → upstream patterns → version-aware docs, evidence labels on every claim, and the anti-reinvention ladder (reuse → built-in → adapt upstream → build) for the recommendation; its findings merge into the approach (see §4), never a standalone research file. §2 Lane-scaled bootstrap (area truth, CONTEXT, critical-patterns, decisions, learnings grep, status) delegates as an extraction-tier I/O worker per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`); other ad-hoc research dispatches during discovery (including bee-xia) default to the generation slot model; ceiling requires the [bee-tier: ceiling] marker plus a one-line justification. Frame candidates through **three layers of knowledge**: tried-and-true (what the repo/ecosystem already trusts), new-and-popular (current mainstream, verify version claims), first-principles (what the problem actually requires). Recommend from evidence, not novelty.

**Artifact fan-out (decision 0009).** Only **L2/L3** discovery earns a separate `docs/history/<feature>/discovery.md` (a real multi-candidate comparison worth reading alone). At **L0/L1**, record the finding in `plan.md`'s `## Discovery` note and cite it — do not spawn a discovery file that just restates the current state `plan.md` already carries. The full fan-out table (which artifacts become separate files, when) is in `references/planning-reference.md`.

## 4. Synthesis — approach (section by default, file when earned)

Produce the approach: chosen path and rejected alternatives, risk map (component / LOW–MEDIUM–HIGH / proof needed), likely files and order, relevant learnings, and open questions for validating. MEDIUM/HIGH unknowns need a validating proof or a spike before execution cells exist.

Write it as an `## Approach` section **inside `plan.md`** by default (standard/high-risk). Graduate it to a standalone `docs/history/<feature>/approach.md` only for **high-risk** lanes or **L2+** discovery, where the rejected alternatives and risk map are substantial enough to read on their own (decision 0009 / fan-out table in the reference). Do not spawn `approach.md` for a fix whose approach is a paragraph — that just restates `plan.md`. For **tiny/small** there is no plan.md by default (D3/D4); the approach is the cell `action` (tiny) or the logged scoping synthesis (small).

## 5. Shape — the right artifact for the lane (STOP at Gate 2)

The work shape is lane-scaled. There is no single mandatory `plan.md`:

- **tiny** drops `plan.md` entirely (D3): the complete shape is **request + one cell** — the cell *is* the micro-plan. The mode-gate record lives in the cell's `action`/notes. No plan document.
- **small** (D4): the default complete shape is a short **scoping synthesis** logged through the decisions CLI (with D-IDs) + **1–3 cells**. `plan.md is opt-in` for small — written only when a durable multi-slice strategy or product-decision document is genuinely needed, never by default.
- **standard / high-risk:** write **one** `docs/history/<feature>/plan.md` with frontmatter:

  ```yaml
  artifact_contract: bee-plan/v1
  mode: standard | high-risk | spike | small (opt-in)
  # approved_gate2: <unset until Gate 2; then a date stamp — the only permitted post-approval write>
  ```

  Body scaled to mode: spike question, phase plan, or epic map (templates in `references/planning-reference.md`). Sketch the test matrix against the 12 edge dimensions at a depth matching the lane.

**Plan freeze (D1).** `plan.md` is **frozen at Gate 2**: once `approved_gates.shape` is set, its content sections are immutable. The only permitted post-approval write is an **approval stamp** (status + timestamp in the frontmatter) — never a content edit. There is no "enrich the same plan.md in place to implementation-ready" step and no `artifact_readiness` requirements-only→implementation-ready mutation; the artifact the human approved stays byte-equal to the artifact that ships. Prep (§6) creates the current slice's cells; it does not rewrite the plan.

Render `docs/history/<feature>/implement-plan.md` via `bee-briefing` only where the fan-out table calls for it (decision 0009): **high-risk** always; **standard** on-demand (default: `plan.md` + the Gate 2 chat layer are the review record — render the brief only when the user asks or the slice spans multiple domains); **small** optional mini-brief on request; **tiny**/**spike** none. When a brief is rendered, the Gate 2 message links it as the review document; when not, the Gate 2 message links `plan.md` directly.

**Gate 2 (standard/high-risk; small only when a plan.md was written).** **Gate-bypass check FIRST** (routing-and-contracts.md §Gate bypass, decisions 0010/dcf01d7b). Read the active level (`node .bee/bin/bee.mjs status --json` → `gate_bypass_level`). If it bypasses Gate 2 for this lane — `normal` covers `tiny`/`small`/`standard` non-hard-gate; `full`/`total` cover **every** lane incl. high-risk/hard-gate — then **DO NOT ask.** Take the shaped plan as approved (the recommended path), set `approved_gates.shape` yourself (`bee.mjs state gate --name shape --approved true`), stamp the plan frontmatter with the approval date (the only permitted post-approval write, per D1), log a one-line audit decision, post `⚡ auto-approved Gate 2 (bypass) — preparing cells`, and continue straight to §6 Prep. Only present the question below when the level does NOT cover this gate. Present **Gate 2** per the Gate Presentation Contract (bee-hive routing reference): plain-language layer in chat — what I plan to build / why this size / cost if the shape is wrong / what you are deciding — in the user's language, the review document linked not pasted; then verbatim: "Work shape is ready. Approve before current-work preparation?" — then **stop**. No pseudo-cells in markdown, no prep, no cells.

**Tiny/small merged gate (fast path) — preview before persist (D5).** For `tiny` and `small`, the ordering is inverted so the approval covers the exact work packet: **draft the cell(s) and run the validating reality check FIRST**, before the merged shape+execution question. The draft cell(s) are rendered as a **preview in the gate message** (never persisted first); the reality check — MODE FIT / REPO FIT / ASSUMPTIONS / SMALLER PATH / PROOF SURFACE, each one line of file/command evidence, 2 minutes not a report — runs inline. Then present **one merged question** in place of Gates 2 and 3: "Work shape + execution: I'm about to do [X] via [Y], verified by [Z]. Approve?" The approval covers the **exact previewed work packet**; `cells add` runs only **after** approval and the cells are claimed only then — **never persist-then-preview**. Execution approval is never granted before the execution package exists. Approval records **both** `approved_gates.shape` and `approved_gates.execution`. **Under any active bypass level** (tiny/small are always covered — even `normal`), do NOT ask the merged question: the reality check still runs (bypass changes only whether the question is asked, never whether the check runs), the draft-cell preview goes into the auto-approval audit line, and if the reality check PASSES, set both `approved_gates.shape` and `approved_gates.execution` yourself, log one audit decision, post `⚡ auto-approved shape+execution (bypass)`, then persist the cells and continue to bee-swarming. Only a reality-check FAIL is surfaced to the human regardless of bypass, and it is presented before asking, never buried. `bee-validating` is not separately invoked for these lanes; its subagents (plan-checker, cell reviewer) do not run — the cell(s) are what a stranger picks up with zero session history, and the cold-pickup criteria are self-checked when writing them.

## 6. Prep (after Gate 2 approval only)

1. **Do not rewrite the plan.** It is frozen (D1); the only post-approval write is the approval stamp (status + timestamp). Prep creates cells — it never enriches a plan in place.
2. Create cells for the **current slice only** (D2) — the whole slice in **one** call, a JSON array piped straight to stdin (never one scratchpad file + one `add` per cell):
   ```bash
   node .bee/bin/bee.mjs cells add --stdin <<'EOF'
   [ { ...cell 1... }, { ...cell 2... } ]
   EOF
   ```
   The batch is all-or-nothing: every cell is validated before any is written. A single object (no array) still works for a one-cell slice; `--file` remains for pre-existing files. For **tiny/small** under the merged gate, these are the previewed cells — persisted **here, after approval, never before** (D5); the current slice lives only in cells, not in a plan section (D2).
   **One trailing test cell per slice (slice-tail-test-batching P2, spec #80/#85).** Whenever the slice holds ≥1 `change_class: 'behavior'`/`'api'` cell, emit **exactly one** `change_class: 'test'` cell, last, with `deps` naming **every** implementation cell of the slice. Its `action` is the slice's **net behavior** — happy path, edge cases, error paths — over the surfaces those cells declared, never per-cell internals; its `verify` is the targeted suite over the slice's touched scope. Implementation cells no longer author tests (they cap on existing-green), so a slice with behavior work and no test cell is a planning defect. Test economy relocates here unchanged: D5 read-first applies **at this cell**, D3's `new_suite_reason` still governs new files, and D3's ratio ceilings compute against the slice's **aggregate** source delta, not per cell. `bugfix` and `high-risk` cells keep per-cell red-first/repro-first and are not batched.

   Every cell is an executable prompt: `files`, `read_first`, directive `action` citing D-IDs, `must_haves` (truths / artifacts / key_links / prohibitions), a runnable `verify` command, and `behavior_change: true` whenever the cell changes observable behavior. You may leave the model `tier` unset — the orchestrator judges each cell's difficulty and assigns the tier when it dispatches (decision 0016); set `tier` only as a hint when a cell is obviously mechanical (`extraction`) or obviously a hard integration/architecture call (`ceiling`), and even then swarming may override it. Cell quality rules and a schema example live in `references/planning-reference.md`.

   **Verify is scoped, not full (verify-scoping D2, decision `20534ea9`).** A cell's `verify` command is the narrowest honest check covering its change — the specific test file(s) or pattern for the touched area, never the full configured test/verify command by default. In this repo that means `node scripts/run_verify.mjs --only <suite>` or the direct test file, not the full `commands.verify` chain. The full configured verify is never owed locally and never authored as a per-cell `verify` (ci-owned-verify D1/D6, superseding the three-moments rule above): CI runs it on the project's own CI cadence (push, nightly, or scheduled — the host workflow decides) and auto-files a `verify-red` issue when red. The dev loop's own broader check is `commands.test` (`node scripts/run_verify.mjs --impacted` / `--impacted-from-git`), selected from the impact registry (`impact_registry.mjs --query`) — narrower than the full chain, wider than one `--only` suite, and still never authored as a per-cell `verify`. Mid-iteration, run the level-1 impacted run (`run_verify --impacted-from-git --level 1` — direct edges only, seconds); the transitive impacted run (`commands.test`) stays the wave-close/merge gate. In a repo that has declared itself no-test (`commands.verify`/`commands.test` set to the sentinel `"none"`, decision 55b951e1), a cell's own `verify` may itself be `"none"` — author it only there, never invent a fake check to satisfy the field, and never carry the sentinel into a repo that has real commands.
3. If an implement plan was rendered at §5 (high-risk, or a standard/small feature where one was produced on request), invoke `bee-briefing` in refresh mode so its Affected Files and Implementation Steps re-project from the created cells. If no brief exists, skip — there is nothing to refresh.
4. Update state and hand off by lane: `tiny`/`small` (merged gate already approved) → `node .bee/bin/bee.mjs state set --owner planning --phase swarming --next-action "Invoke bee-swarming (single execution worker)."`; every other lane → `node .bee/bin/bee.mjs state set --owner planning --phase validating --next-action "Invoke bee-validating."` **The phase must be a real member of the enum** (`idle, exploring, planning, validating, swarming, reviewing, scribing, compounding, grooming, compounding-complete`) — invented names like `planning-complete` are refused by `state set`, and an agent that hits that refusal starts improvising the state machine, which is exactly how the chain broke (chain-integrity D6). Completion is carried by the approved gate, never by a phase name. `--owner` always names the selected record's pre-mutation phase; it is not persisted.

## Scope-Reduction Prohibition

If the shape cannot fit the budget or context, **never** quietly shrink a locked decision or drop a must-have. Answer `SPLIT RECOMMENDED`: propose slice boundaries, each slice honoring every locked decision it touches, and let the user choose. Cheaper alternatives found in research are *noted* alongside the honored decision — swapping them in requires the user superseding the D-ID.

## Headless

With `mode:headless`: run intake classification, lane-scaled bootstrap, discovery, and synthesis without questions. For **standard/high-risk**, write `plan.md` and stop — Gate 2 is never self-approved. For **tiny/small**, produce the draft-cell preview + inline reality check in the structured terminal report and stop before persisting cells — the merged gate is never self-approved. Ambiguities (mode borderline, conflicting decisions, missing CONTEXT.md sections) go to an `Outstanding Questions` section of the report.

## Red Flags

- skipping the mandatory critical-patterns digest, active decisions, or `CONTEXT.md`
- full-bootstrapping before the mode gate has picked the lane (D8 inverts this)
- skipping the mode gate, or choosing a mode without counting flags
- counting `.bee/**`, `docs/**`, or generated projections as product files against a lane cap (D6)
- defaulting to phases without proving the work needs them
- editing `plan.md` content after Gate 2 (frozen — approval stamp only, per D1)
- writing a `plan.md` for tiny, or by default for small (D3/D4)
- persisting cells before the merged gate approval (persist-then-preview, D5)
- cells or prep artifacts before Gate 2 approval
- future-slice cells · pseudo-cells in markdown
- vague exit states, missing deps, or a `verify` that cannot run
- silently swapping a locked decision for a "better" research finding
- shrinking scope instead of answering SPLIT RECOMMENDED

Violating the letter of the rules is violating the spirit of the rules.

Plan shaped and current-slice cells prepared. `tiny`/`small`: invoke bee-swarming skill (single execution worker — the merged gate already covers execution approval). All other lanes: invoke bee-validating skill.
