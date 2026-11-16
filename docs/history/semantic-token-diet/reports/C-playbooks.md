# Token audit: route playbooks and principles

Read-only. No head check (brief says skip). Criteria: `.bee/tmp/token-audit/criteria.md`.

**The brief's premise is wrong.** It says `bee orient` names 3 principles per class. It names every principle whose index row lists the class, with no cap (`packages/bee-rs/crates/bee/src/principles.rs:128-139`, filter at :131, source index at :22-27). Measured with `bee orient --class X | rg -c bee-principle-` and `| wc -c`:

| class | principles | orient bytes |
|---|---|---|
| feature | 31 | 8,321 |
| refactor | 29 | 7,813 |
| bugfix | 12 | 3,350 |
| docs | 8 | 2,643 |
| perf | 6 | 2,374 |
| research | 3 | 2,295 |
| release | 5 | 1,781 |
| spike | 4 | 1,691 |
| content | 0 | 387 |

## 1. Load map

- **Paid on every route:** the `bee orient --class X` output. The class `## Situations` rows are compiled into the binary (`situations.rs:13-21` `include_str!`, parse at :45) and echoed by orient. The principle list is printed with the header "Load each one with skillstation skill show <slug>" (`principles.rs:139`).
- **Paid on every principle load:** `skills/bee-principle-*/SKILL.md`, 37 files, 1,137–2,925 B, 58,064 B in total (`wc -c`). If an agent obeys "load each one" on a feature route, it reads about 46 KB [estimated from the sizes].
- **Paid on branch, one situation per route:**
  - bugfix: bug-fix 4,718
  - feature: feature 7,345, multi-phase-plan 12,950
  - perf: perf-issue 5,712, hillclimb 5,969
  - refactor: refactoring 5,539, visual-parity 2,378
  - research: investigation 3,149, eval 4,553, runtime-forensics 2,259, trace-forensics 2,975
  - spike: prototype 4,496
  - docs: docs 919, authoring-a-skill 1,843
  - content 1,183; release 819
  - session: orchestrate 16,583, babysit 8,418, autopilot-full 6,244, autopilot-stack 5,738, opening-a-pr 4,586, shipping 4,550, worktree-cleanup 3,272, autonomous-run 2,840, session-pickup 2,668, pause-safely 2,384, index 3,392
- **Rarely opened:** the class files `X.md` (573–1,961 B). After orient prints them, the "next:" line sends the agent straight to the situation file.
- **Reads that repeat:** session situations chain into each other. For example, autopilot-full:23-27 and :55 point at babysit, orchestrate § Merge safety and shipping step 3. autopilot-full:69-70 and autopilot-stack:25-26 re-read the whole playbook from main on every audit tick. multi-phase-plan:88-92 re-reads 4 or more files on every 30-minute tick.

## 2. Findings

| # | class | path:line | what it says | proposed rewrite / move | est. bytes saved | risk |
|---|---|---|---|---|---|---|
| 1 | 4/6 | principles.rs:131; orient output | Prints every class-tagged principle (31 for feature, 29 for refactor) and tells the agent to load each one | Rank them and print the 3–5 core principles per class. List the rest by slug only, loaded on demand. Or tag fewer classes per row in the index doc (`principles.rs:23`) | ~6 KB per feature/refactor route; up to ~40 KB if loads are obeyed | med: needs a ranking decision; the principle-arm cap check (`cells finish --help`, `--principle-used`) must keep working |
| 2 | 7/2 | 21 files, e.g. feature.md:113, bug-fix.md:75, every class X.md:11 | "Proof floor: … the table is the cap's own (`bee cells finish --help`)" | The pointer is dead: `bee cells finish --help \| rg -c -i floor` gives 0. The floor lives in code (`finish_support.rs:111` `class_proof_floor`), and the cap names it when it refuses. Delete the line from the 14 situation files. Keep one corrected line in each class X.md, or drop it | ~165 B per file, ~3.4 KB in total, ~165 B per route | low: rg finds no test pinning it (only a fixture `Proof floor: x` at class_playbook_parity.rs:323) |
| 3 | 1 | autonomous-run:36-39, autopilot-full:50-62, autopilot-stack:66-73, babysit:107-112, shipping:36-38,60-62, opening-a-pr:50-53, orchestrate:162-165; plus perf-issue:77-80, hillclimb:79-82, refactoring:77-80, visual-parity:30-33, authoring-a-skill:16-19 | The landing paragraph repeated: "`bee worktree merge` … without asking, as `uat_stop = "close"` already does … the `uat` door of `bee close` stays the owner's: uat is never auto-approved at any bypass level" | opening-a-pr step 6 owns it (feature.md:92 already points there). Elsewhere use one line: "Land per `session/opening-a-pr.md` step 6." Keep each file's own why-clause, e.g. visual-parity's "the owner is the one who looks at the screen" | ~200 B per file, ~2.5 KB in total | low–med: session files are read standalone, and the uat-door rule also lives in bee-hive SKILL.md:63 |
| 4 | 7 | autopilot-full:50, autopilot-stack:67, shipping:37 | `bee worktree merge --id <unit> --cleanup` | Drop `--cleanup`. `bee worktree merge --help` says: "Legacy flag … now a NO-OP: cleanup runs by default" | ~10 B each | low |
| 5 | 7 | bug-fix.md:23-27 | Cites "home — `bee-swarming/references/worker-details.md:33-35`" and explains why the rule is not copied | The line anchor is stale: the quote is at worker-details.md:40 (`rg -n "red-before-green is craft"`). Rewrite as "2. Watch that reproduction FAIL before the fix (`bee-principle-red-before-green`)." orient already names that principle for bugfix | ~230 B per bugfix route | low |
| 6 | 1 | autopilot-stack:13-39 vs autopilot-full:13-31,63-84 | Stack restates full's owner loop, audit tick (near-verbatim, ~700 B) and stop and state-then-wait rules, although stack:8-10 says "the owner loop and the verification gate are the same" | Merge the two into one autopilot file with a "terminal: per-unit merge \| one chain" switch. The choice rule (stack:78-81) becomes its first step | ~4–5 KB on disk; per run only when both are read (multi-phase-plan:32-34 sends planners to both) | med: per-tick re-read paths (full:70, stack:26, multi-phase-plan:89) change |
| 7 | 1 | autonomous-run:18-20, feature:84-86, perf-issue:66-68, hillclimb:60-62, refactoring:74-77, bug-fix:56-58, multi-phase-plan:30-31 | Rewords `bee-principle-sequence-verifiable-units` ("verify each before the next, so a failure points at one unit") right next to the slug | Keep the slug and drop the rewording. The principle body says it (principle SKILL.md:12-22) and orient names it for these classes | ~120 B per route | med: steps become less self-contained when the todo list is built |
| 8 | 1 | runtime-forensics:18-21, trace-forensics:18-21 (identical sentence), session-pickup:18-21, worktree-cleanup:24-25, hillclimb:47-48 | Rewords `guard-the-context-window` | Slug only | ~90 B per route | low |
| 9 | 1/2 | feature:62-64, opening-a-pr:32-35, autopilot-full:24-25, autopilot-stack:17-18, multi-phase-plan:110 | "Write no comment … because bee writes no comment in code" | Always-loaded AGENTS.md BEE block already has "Write no comment" and so does the worker preamble. Keep the `bee-comment-review` step and drop the restated rule | ~100 B per file | low |
| 10 | 3 | content:13-15, perf-issue:82, hillclimb:84, investigation:43-45 | "per D6, decision `8fd45bbb`", "per D2, decision `1593e365`", "per D3 … `f1ffa7bd` … backlog `p-69bee217`" | Move the ids to `skills/bee-planning/references/provenance.md` (it does not exist yet; `ls` of references/). Keep the rules: "It feels faster is not a result" and "read-only is craft, not a guard" | ~60–150 B each | low |
| 11 | 3 | orchestrate:57-58; babysit:22-23, 44-46; shipping:33-35 | "hard-coded worker trees were tried and parked"; "Each step below exists because that failure cost a night"; "severed a long chain of PRs and cost a day"; "Twenty-one verdicts went stale" | Cut the incident stories and keep the because-clauses that state a mechanism (shipping:31-32 "A rebase rewrites every SHA …" stays) | ~450 B in total | low–med: D6 protects why-clauses, and these are anecdotes, not mechanisms |
| 12 | 4 | feature.md:25-32, 77-83 | The statechart and formal-verify steps (~1.4 KB) are paid on every feature route, though both are host-optional | Shrink each to one line: "When the host has `statechart` (event-order flow) / `create-formal-verification-skill`, run it; else say skipped." The detail lives in those skills | ~900 B per feature route | med: the step-8 why is real in this repo (formal-cell-lifecycle exists) |
| 13 | 7 [partial] | autopilot-full:66-67, autopilot-stack:23-24, multi-phase-plan:93-94, orchestrate:182 | Arm the "audit tick" under `bee herding control-loop` with a custom tick prompt | The verb takes only `--role dispatch\|merge` and reads its prompt from skills/bee-herding/references (`bee herding control-loop --help`). It cannot carry the "use this tick prompt, verbatim" text. Re-point to whatever really arms a custom tick, or say "a recurring wake" | n/a (correctness) | med [unverified whether another door takes a prompt] |
| 14 | 6 | multi-phase-plan:153-162 | Ten literal `Lane n` rows | One row plus "repeat to Lane 10" | ~550 B per run of that situation (rare) | low–med: templates with ten rows reliably produce ten |
| 15 | 1 | 13 files (`rg -c verify-app`) | "through the project's `verify-app` skill when it has one, otherwise through the real interface itself" | Coin one term ("the real surface") and define it once in each class X.md. Saves little per route because one situation is read per route; this is mainly about drift | ~70 B per route | low |

## 3. Top 3 by saved tokens × frequency

1. **Cap the principle list in orient** (#1). It is the largest per-route cost on the most common classes, with a ~40 KB tail if "load each one" is obeyed.
2. **Delete the dead "Proof floor" pointer** (#2). It is on every route, and its target help text carries no floor table.
3. **Trim the feature.md optional-skill steps (#12) and collapse the landing paragraph (#3).** The first lands on the most frequent situation file; the second appears in 13 files, all on the session branch.

## 4. Do not touch (class 8)

- The `## Situations` row format, `` - `name` — triggers — delivers: … ``. It is parsed at compile time (`situations.rs:3,45`) and fenced by `class_playbook_parity.rs`.
- feature.md:56-61, "Delegation is mandatory … 'The app is small' is no exemption". It names an exemption agents reach for.
- pause-safely:5-9, "a context mark never stops work"; session-pickup:12-14, adopt only in a fresh session.
- babysit step 1 (declare the mode), step 2 (frontier only) and step 7 ("never retry the failed jobs … reuses the original ref snapshot"). These are mechanism why-clauses.
- worktree-cleanup steps 2 and 4 and :45-46. Deletion is irreversible.
- The eval blinding list (eval:9-31). Each bullet is a distinct behavior; collapsing them loses some.
- investigation step 1 (read-only). investigation:44-45 says no guard enforces it, so the prose is the only guard. Move only the ids (#10).
- multi-phase-plan's repeated verification sentence (:50-52, 80, 147, 151, 164). Step 7 requires it in the output.
- reproduce-first:23-27, which separates reproduce-first from red-before-green. That is disambiguation, not duplication.
- Each situation's own `Proof line:`. They differ per situation.

## Pins checked

- `rg -n "Proof floor|cap refuses a result weaker"` over the cells sources finds no hit. The only match is the parity fixture at class_playbook_parity.rs:323.
- `rg -l` for situation-file paths in Rust finds only status_full tests, which pin the `playbook:` path format and not the prose.
- No test pins any sentence I proposed to change. The one pinned format is the Situations row grammar.
