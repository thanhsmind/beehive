# Token audit — branch CLOSE / CAPTURE / REVIEW

Scope: skills/bee-capturing, skills/bee-verifying, skills/bee-reviewing, skills/bee-knowledge-upkeep (SKILL.md + references). Read-only; nothing edited. Criteria: .bee/tmp/token-audit/criteria.md. Lens calibration: .bee/expertise/review.md.

## Paths read

skills/bee-capturing/SKILL.md, references/{area-spec,citations,concept-authoring,promotion}.md · skills/bee-reviewing/SKILL.md, references/reviewing-reference.md · skills/bee-verifying/SKILL.md · skills/bee-knowledge-upkeep/SKILL.md · packages/bee/prompts/reviewer.md · packages/bee/LEADER.md:340-370 · docs/knowledge/areas/okf-profile/freshness-and-document-homes.md (rg hits) · .bee/backlog.jsonl:1756 · docs/history/research/pstack-xia.md:108-112 · `bee upkeep due --help`, `bee knowledge consolidate --feature cli-before-mcp`.

## 1. Load map (`wc -c`)

| When | File | Bytes |
|---|---|---|
| Prepaid, highest frequency (model-invocable, "SELF-TRIGGERING", description in every skill listing) | bee-capturing/SKILL.md | 11,462 |
| Prepaid, per explicit user review | bee-reviewing/SKILL.md | 10,879 |
| Prepaid in practice: every review wave (SKILL.md:44-47 routes there for lens cards + dispatch prompt shape) | bee-reviewing/references/reviewing-reference.md | 9,187 |
| Per reviewer dispatch ×4-6 (dispatch `Expertise:` line) — outside branch files, driven by it | .bee/expertise/review.md | 25,886 |
| Prepaid, rare (`disable-model-invocation`, once per repo) | bee-verifying/SKILL.md | 10,587 |
| Prepaid, rare (`disable-model-invocation`, periodic) | bee-knowledge-upkeep/SKILL.md | 6,324 |
| On branch: any spec write/merge | capturing/references/area-spec.md | 20,065 |
| On branch: Compound step only | capturing/references/promotion.md | 9,557 |
| On branch: concept write | capturing/references/concept-authoring.md | 8,398 |
| On branch: citing a decision | capturing/references/citations.md | 1,127 |
| On branch: verifying step 3 | verifying/references/feature-map-example/* | 9,005 |
| Never loaded by an agent | verifying/LICENSE.pstack; */agents/openai.yaml (runtime metadata) | 1,067; 521; 596 |

Stale-name sweep (class 7): every verb named in the four skills answers `--help` (ran a loop over 19 verbs: all `ok`); `--skill-answer`, `--verify-only`, `--brief-file` exist in help; every cited docs/knowledge and .bee/expertise path exists (`[ -e ]` loop: all `ok`); skills bee-verify-upkeep, bee-knowledge-setup exist. One stale instruction found (#16).

Test pins: `rg -F` over packages/ (excluding rendered skill copies) for 40 candidate phrases. The only pinned ones are REVIEW_HEAD_MISMATCH / "re-freeze the review head" / "The brief never carries" — pinned in packages/bee/prompts/reviewer.md via drivers/tests.rs:8238 (the prompt, not the skill); `green:live` (many); `verify-only --apply` (onboard tests); rule ids agents-capture-line-at-close / agents-feedback-changes-framework (LEADER.md); workflow-state-capture-skill-answer (capture.rs). No skill sentence proposed below is pinned. Constraint: packages/bee-rs/crates/bee/tests/pointer_integrity.rs checks that a quoted heading after `references/x.md ("Heading")` resolves — any move must keep or re-point cited headings.

## 2. Findings

| # | class | path:line | what it says | proposed semantic rewrite or move | est. bytes saved | risk |
|---|---|---|---|---|---|---|
| 1 | 2+1 | capturing/SKILL.md:96-102 | Restates the Read-when / title-description rule and the three `knowledge check` warnings | Same rule is printed by `bee knowledge consolidate` ("Knowledge rules:" line 13 of its output) and stated again at concept-authoring.md:93-100. Body keeps one line: "`bee knowledge new` names the path and frontmatter; the write loop is `references/concept-authoring.md`." | ~330 | low — two other homes remain, one runtime-printed |
| 2 | 4+1 | capturing/SKILL.md:115-121 | Template sections open with a question + deletion line; optional `status/proves` tail; cut template | Only acts while writing a spec, when area-spec.md is loaded and already shows it (area-spec.md:85-203, :158-160); third home is freshness-and-document-homes.md:115-133. Delete the paragraph. | ~530 | low |
| 3 | 3+4 | capturing/SKILL.md:136-141 | "Close no longer writes `promote-proposals.md`…" then a drain-once procedure for old files | History narration in a prepaid body; the drain is live (405 legacy files: `fd -t f promote-proposals.md docs/history \| wc -l` → 405) but a rare branch. Move the drain to promotion.md as "Legacy promote-proposals drain"; body step 2 becomes one line pointing there. How many of the 405 are already drained: [unverified] | ~330 | low |
| 4 | 1 | capturing/SKILL.md:159-163 | "Prefer an executable check over prose… escalate to a durable owner… or record the one-line reason" | Verbatim content of promotion.md:123-148 (Decision Tree steps 2-4), which the same sentence routes to. Keep the three bars (154-158) and the unique "promote a wording or placement change, never a second copy" (157-159); cut the rest to the pointer. | ~450 | low |
| 5 | 1 | capturing/SKILL.md:33-36 | Bee-repo owner feedback changes the framework, never private memory | Always-loaded home: packages/bee/LEADER.md:363-368 (rule agents-feedback-changes-framework). Delete from the skill. | ~270 | low — rule stays in every leader session |
| 6 | 4 | capturing/SKILL.md:105-107 | "A bundle with no anchors is seeded once with `bee knowledge seed-anchors`" | One-time branch in a prepaid body; move to concept-authoring.md "Which verb, when" table. Keep the `confirm` sentence. | ~120 | low |
| 7 | 1 | capturing/SKILL.md:202-203 | References rows for .bee/expertise/knowledge.md, documentation.md, decisions.md | Same pointers already sit in the body at :23-25, :65-66, :91-92. Delete the two rows. | ~290 | low |
| 8 | 1 | capturing/references/area-spec.md:281-287 | Merge Rules: deltas never from plan.md; contradicting line replaces; present tense | Same rule at area-spec.md:41-44 and capturing/SKILL.md:78-82. Keep it once in Merge Rules; :41-44 keeps only "Never invent". | ~200 | low |
| 9 | 4 | capturing/references/area-spec.md:246-270 | "Cut area template" for PBI p-3a756890 | PBI is `"status":"proposed"` (.bee/backlog.jsonl:1756); the text itself says hosts keep the nine-section template. Paid on every spec write for a template nobody uses yet. Move to the PBI's brief/history; restore when the cut ships. | ~950 | med — freshness-and-document-homes.md:167 points here for it; re-point in the same change |
| 10 | 1+3 | reviewing/SKILL.md:113-133 | Dismissed section rules + "Two independent studies of pstack…" (:119-122) | Three homes: this, reviewing-reference.md:165-180 ("not a finding… never enters the summary line's counts", "Omit the whole section when nothing was dismissed"), review.md "Verify before reporting". Body keeps: "Directly above the last line, list every dropped suspicion as a Dismissed section (shape: reference "Dismissal shape"); no drop → no section; dismissals never enter the counts." Provenance (:119-122) → references/provenance.md. | ~650 | low — reference is read every run (load map), so nothing goes unseen |
| 11 | 1 | reviewing/SKILL.md:65-67, :96-98 | Axis definitions; "ONE report, grouped by axis, spec-axis group first" | Restated at reviewing-reference.md:123-127. Body keeps "every finding carries an axis label (`spec`/`standards`)"; definition and grouping live in the reference's Finding schema. | ~350 | low |
| 12 | 2 | reviewing/SKILL.md:52-55 | Reviewer checks HEAD vs frozen head, reports REVIEW_HEAD_MISMATCH + fix line | Reviewer-side instruction already delivered in every dispatch by packages/bee/prompts/reviewer.md:3 (the pin at drivers/tests.rs:8238 is on the prompt). Orchestrator only needs: "A `REVIEW_HEAD_MISMATCH` return means re-freeze the head or clean the stray paths, then re-dispatch." Keep :51-52 "The brief never carries…" (orchestrator-side). | ~150 | low |
| 13 | 1 | reviewing/SKILL.md:150-152 | UAT fail → P1 fix cell; skip needs reason; intermittent is Fail | Same three bullets at reviewing-reference.md:192-194, which the sentence already routes to. Keep the route + `bee reviews record --kind uat`. | ~150 | low |
| 14 | 5 | reviewing-reference.md:3-4; promotion.md:3-4; area-spec.md:3-5; concept-authoring.md:3-7 | "Judgment lives in SKILL.md; templates live here" openers | Inert for a reader already routed here. Reduce each to its first "Load when" clause or delete. | ~350 total | low |
| 15 | 3 | knowledge-upkeep/SKILL.md:92-94 | "limit is 5 because a measured result… (ReHarness, arXiv 2610.12668)" | Keep "At most 5 concepts" plus a 6-word why ("measured gain is flat past 5"); citation → references/provenance.md (new; convention exists in bee-evolving and bee-writing-skills). | ~200 | med — it is a why on a threshold; keep the clause, move only the citation |
| 16 | 7+2 | knowledge-upkeep/SKILL.md:49-68 | Signal table, "numbers are starting defaults… change them here when they prove wrong", loaded-never-used caveat | `bee upkeep due --help` prints the same reasons (reported_wrong, orphan, suspect_loaded "5 or more", loaded_never_used "never make the run due"); the thresholds live in the binary, so "change them here" no longer changes anything: stale. Rewrite: "Run `bee upkeep due --kind knowledge --json`; exit 10 = due; each reason names its signal. Thresholds live in the binary." Keep the step-2 rank order. | ~800 | low-med — removes the human-readable table; the help text replaces it |
| 17 | 6+1 | knowledge-upkeep/SKILL.md:70-78 | Table "When knowledge is read, written and revised" | 4 of 5 rows are cap-time behavior owned by capturing/SKILL.md:110-113 and worker briefs; none is an upkeep-run action. Only the last row describes this skill, and the Pass section says it better. Delete the table. | ~460 | low |
| 18 | 3 | capturing/references/promotion.md:116, :136-140 | "(mistake-fix-at D5)"; parenthetical on why prose has no mechanical owner | Decision id → provenance. The parenthetical explains why the route is judged from the run's record; shorten to one clause: "(no hook records a skill open, so judge it from the run's record)". | ~250 | med — part of it is a working why |
| 19 | 1 | verifying/SKILL.md:139-140 | Re-states `bee onboard --verify-only --apply` + "needs no bee checkout" | Already the standing rule at :41-42. Shorten to "Then re-render (rule above)." Once-per-repo skill, so the saving is near zero. | ~70 | low |

## 3. Top 3 by saved bytes × frequency

1. bee-capturing/SKILL.md, #1+#2+#3+#4+#5+#7 ≈ 2.3 KB of 11.5 KB, in the branch's only self-triggering skill.
2. bee-reviewing SKILL.md + reviewing-reference.md (both paid every review), #10-#13 ≈ 1.3 KB. Give each rule one home in the reference the body already routes to on every run.
3. bee-knowledge-upkeep/SKILL.md, #16+#17 ≈ 1.26 KB of 6.3 KB. This also removes the stale "change them here" instruction.

## 4. Do-not-touch (class 8)

- verifying/SKILL.md:105-114: why skill helpers are invoked as `bash <path>`. The text itself says it is there so nobody "fixes" it back. Removing it would bring back the `Permission denied` failure on host onboarding.
- verifying/SKILL.md:154-158 and :175-179: the exit-code contract, plus why a drive is not the test command. Both are why-clauses on hard rules (agents-block-diet D6).
- reviewing/SKILL.md:18-22: the explicit-request trigger rule. :83-90: "A behavior finding survives only on a run". :103-111: the required last line and the verbatim zero-findings line. :159-162: the verbatim merge question. :51-52: "The brief never carries the orchestrator's own verdict…", which is the orchestrator-side half of the reviewer.md pin.
- capturing/SKILL.md:56-64: the capture-line format and the verbatim `nothing settled.`. LEADER.md:344-348 holds the rule but not the format. :43-47: `--skill-answer`, which the binary refuses without (capture.rs). Removing it causes a refusal loop. :177-186: Hard rules.
- promotion.md:170-172: why critical-patterns must stay selective (it is injected into every preamble).
- knowledge-upkeep/SKILL.md:41-45: edit scope (never edit product code).

## Out of scope, noted for follow-up

- .bee/expertise/review.md (25,886 B) is loaded per reviewer dispatch, ×4-6 per wave. Its author-side sections are inert for a dispatched reviewer: "Asking for a review", "Receiving a finding", "Reviewing your own work". The same goes for the CRUD and claims-table sections on a non-plan, non-lifecycle diff. A split into reviewer-side and author-side homes is the largest per-dispatch cost this branch drives [unverified: exact dispatch payload size not measured].
- skills/bee-verifying/LICENSE.pstack (1,067 B) is shipped and rendered but never loaded by an agent. Licensing is required, so no saving is possible; noted only so it is not counted.

## Dismissed

- capturing/SKILL.md:56-64 duplicates LEADER.md:344-348 — `capturing/SKILL.md:56` — dismissed: LEADER states the rule but not the `captured: X → Y` format or the verbatim `nothing settled.`, so the skill copy carries unique behavior.
- reviewing-reference.md:8-11 restates SKILL.md:42-43 (no `--role`) — `reviewing-reference.md:8` — dismissed: the reference adds the why (a lens name under `--role` refuses), so cutting either side drops something; saving ~150 B is not worth it.
- verifying/SKILL.md overall — dismissed as a target: it is `disable-model-invocation` and runs once per repo, so frequency × bytes is near zero, and most of its prose is generation spec the output depends on.

## Proof

I ran the brief's proof command read-only. It writes only to the gitignored target/ directory. Final line of /tmp/job-1794841419147-proof.log: `EXIT 0`. The clippy `-D warnings` step passed. `cargo test --release`: 49 `test result:` lines, 5918 passed, 0 failed, 30 ignored. The test-target build printed 11 compiler warnings (unused variables, unused `Result`). They come from `bee` (bin "bee" test), which clippy does not cover here. They are not mine and are not part of this audit. `git status --short` after the run shows only the paths that were already modified before the job, so the working tree is unchanged by me.
