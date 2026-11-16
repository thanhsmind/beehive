# Token audit: PLANNING branch (job-1794840888458-1454246-1, round 1)

Read-only. I edited no files. Scope: skills/bee-planning/SKILL.md, references/*.md and agents/. Playbooks are out of scope.

Paths read: .bee/tmp/token-audit/criteria.md, .bee/expertise/review.md (up to "Where to look"), skills/bee-planning/SKILL.md, references/{planning-reference,design-sketch,edge-dimensions}.md, agents/openai.yaml, skills/bee-hive/references/gates-and-delegation.md:201-290, skills/bee-swarming/SKILL.md:263-280, packages/bee/LEADER.md:483-494.

## 1. Load map (`wc -c`)

| File | Bytes | When it loads |
|---|---|---|
| SKILL.md | 12,817 | Prepaid on every invoke |
| agents/openai.yaml | 387 | Codex interface metadata, not the body [unverified that Codex loads it per run] |
| references/planning-reference.md | 32,065 | On-branch on paper, but **prepaid in practice on every lane except docs** (see F2) |
| references/design-sketch.md | 8,013 | standard/high-risk with an open shape only (SKILL.md:113-115) |
| references/edge-dimensions.md | 3,850 | high-risk/hard-gate test matrix only (SKILL.md:211) |

**Does the body force the 32KB reference on every run? Yes, on every code-touching lane.** `rg -n planning-reference SKILL.md` gives 9 pointers: :61, :89, :103, :112, :148, :155-156, :163, :181, :209. Even the cheapest code path reaches it:
- SKILL.md:61, "Playbooks", on every routed lane.
- :148, "Tiny/small merged gate".
- :155-156, "Pre-flight before cells add" and "Cell quality rules", on every lane that adds cells.
- :181, "Test scoping".

Every pointer is a section anchor inside one 653-line file, and a Read with no range loads all of it. A tiny or small run needs about 13.5KB. These are the section sizes from an awk split: merged gate 1,556, cell rules 4,483, example JSON 1,188, pre-flight 2,541, test scoping 1,779, playbooks 1,987. Yet it pays 32KB.

## 2. Findings

| # | class | path:line | what it says | proposed semantic rewrite / move | est. bytes saved | risk |
|---|---|---|---|---|---|---|
| F1 | 7 (stale, contradicts the live rule) | planning-reference.md:640-653; also :104-105, edge-dimensions.md:7 and :11-13 | "There is no trailing test cell… The cell's writer owns its tests… author only what is not already pinned" | Delete the section. The live rule is SKILL.md:173-178 plus bee-swarming/SKILL.md:265-269: the implementer owns only the existing-test edits its change forces, and a separate `role: test` tester adds the new tests. Change "each cell's writer judges" (:104, edge-dimensions:11) to "the tester cell judges", and repoint edge-dimensions:7 to SKILL.md ("Prep"). The triad / 12-dimension scope rule already has its home at edge-dimensions:3-5. | ~840 | low. `rg -F 'Tests ride the cell' packages` and `rg -F 'no trailing test cell' packages` both find nothing |
| F2 | 4 (rare-branch detail prepaid) | planning-reference.md (whole file) | One file serves every lane | Split by branch. `cells.md`, every lane: cell quality rules, example JSON, pre-flight, test scoping (~10KB). `plan-artifact.md`, standard/high-risk only: fan-out, plan.md template, claims table, approach.md, phase/epic, consolidation cell, hat-wave consumption (~17KB). The tiny/small merged gate (1.5KB) goes into the cells file or into the SKILL body. Greenfield (535B) moves to onboarding. | ~17KB per tiny/small run | med. Inbound anchors must be repointed: packages/bee/LEADER.md:490, bee-hive/references/scout-and-ticks.md:44, set_gate.rs:759 (a comment), and these docs/knowledge files: cells-scheduling.md:14/118, cells-authoring-and-revision.md:14/424, dissent-and-the-verdict-duty.md:303, lane-and-working-discipline.md:51/84, router-triage…:212, skill-reference-pointer-integrity.md:130, and the patterns doc that cites `:162` |
| F3 | 1 (same fact, several homes) | planning-reference.md:542-584 | Says it "never restates that procedure", then restates the seats (:558-563), SMALLER PATH one-home (:565-568), the two mandates (:570-574) and the once-per-feature threshold (:576-583) | Owner: gates-and-delegation.md:201-264. In planning, keep only what is planning-only: the inline rule "standard ≤5 product files, no hard-gate flag" (:577-580; `rg '≤5 product files'` finds no other home), MANDATE 1's five dimensions (:594-608), MANDATE 2's criteria (:610-618) and the synthesis template (:624-638) | ~1,800 | low |
| F4 | 3 (history narration) | planning-reference.md:544-546, :558-559, :253-254, :284, :561-562, :577, :478 | "absorbed the old merged plan-reviewer dispatch (… `b34fdea9`)", "The dispatch kind changed. No reviewer subagent… any more", decision ids, "`commands.verify` is retired" | Move the ids and history to docs/decisions/skills/bee-planning-provenance.md. Cut the retirement note: when a config still carries the key, the runtime prints RETIRED_VERIFY_KEY_WARNING (onboard/templates.rs:383) | ~650 | low. `rg` finds no test pin on these strings ("is retired" matches only unrelated code) |
| F5 | 1 | SKILL.md:137 vs planning-reference.md:172-177 (and SKILL.md:144) | The `bee gate --preview` contract (fields, hash, freshness) appears twice | Keep SKILL.md:137. Delete ref:172-177; `bee gate --help` already describes `--preview` | ~480 | low |
| F6 | 1 | planning-reference.md:330-339 vs :438-442 | Role vocabulary, fall-through and the silent `code`/`read` case, written twice in the same file | Keep rule 10. Pre-flight item 2 says only "`role` non-empty (Cell quality rule 10)" | ~380 | low |
| F7 | 2 (runtime echo) | SKILL.md:45-49 | What `bee feature start` does internally | `bee feature start --help` already prints "create and grant its git worktree, create its lane record, and record its triage route. Prints the worktree path…". Shorten to: run `bee feature start …` from main, then open the session in the worktree it prints (rule: agents-worktree-first) | ~180, prepaid | low |
| F8 | 1 | planning-reference.md:375-377 vs SKILL.md:159-164 | Consolidation cell: when and why, written twice | Keep SKILL.md. The reference keeps only the JSON | ~250 | low |
| F9 | 3 | planning-reference.md:301-303 | Incident story "Observed both ways: … 2045-2549 in a 5516-line file stalled…" | The rationale already sits at :295-297 ("they drift… cannot recover"). Cut the anecdote | ~300 | med. It is evidence for the rule; the D6 why-clause carve-out could be argued |
| F10 | 4 | SKILL.md:168-171 | How the mint trap works and where it is blind | Keep the duty in the prepaid body ("cite the contract decision even where the trap cannot see it"). Move the mechanism to the dispatch.md concept the line already cites | ~200, prepaid | med. It names a real gap that agents hit |
| F11 | 7 (wrong gate) | design-sketch.md:33-35 | "Never write it as source before the merged shape+execution gate is approved" | The sketch loads only on standard/high-risk (design-sketch.md:3, SKILL.md:113), and those lanes use the separate shape gate (SKILL.md:137-142). The merged gate is tiny/small only (SKILL.md:144). Change it to "before the shape gate is approved" | 0 (fixes a wrong rule) | low |

Verbs and flags the bodies name all exist; I probed each and the CLI resolved it. Commands run: `bee feature start`, `bee knowledge pack`, `bee knowledge consolidate`, `bee backlog pbi add --help` (has `--brief-file`), `bee dispatch prepare`, `bee plan check`, `bee worktree new`, `bee gate --help` (has `--preview`). For the conflict check, `bee state --bogus` lists `plan-conflicts derive` and `plan-conflicts verdict`. The cross-file anchors also resolve: gates.md R138 is at :462, "Hat wave" and "Blind lanes and convergence" in gates-and-delegation.md at :201 and :158, "Route record" at scout-and-ticks.md:38, "Claims-table audit" at review.md:193, CHANGE_CLASSES at validate.rs:122-123, "Principle homes" at router-triage:321, and trace-and-provenance.md exists.

## 3. Top 3 by saved tokens × frequency

1. **F2:** split planning-reference.md by branch. Saves ~17KB on every tiny/small run and about 3-5KB on standard runs.
2. **F1:** delete the stale "Tests ride the cell" section and fix the "writer owns its tests" lines. It contradicts the tester-cell contract every planner follows, so it is a correctness defect as well as ~0.9KB.
3. **F3+F4:** shrink the hat-wave consumption section to the planning-only parts and move history out. Saves ~2.4KB on every run that loads the file.

## 4. Do not touch (class 8)

- SKILL.md:83-91, "One reality touch…", including "Existence is not evidence". It is a compact rule that comes with its why.
- SKILL.md:132-135, "a gate that refuses after the human already said yes spends their approval on your homework". This is a why-clause doing real work (agents-block-diet D6).
- SKILL.md:140, the gate wording "Work shape is ready. Approve before current-work preparation?". It is pinned verbatim in cells/handlers_write.rs:2037.
- planning-reference.md:149-153, the match rule, and the "load-bearing claims table" phrase. set_gate.rs:2160 asserts `msg.contains("load-bearing claims table")`, and plan_claims.rs:346 prints the phrase.
- planning-reference.md:291-297, "Carry the hits, not a line range". The rule and its drift rationale stay; only the anecdote is a candidate (F9).
- planning-reference.md:206-211, the one-fact rule for the risk map ("a writeup reads the same whether it is true or not") and :536-540 ("Zero load-bearing claims is a real answer"). Both are anti-manufacture rules with their rationale.
- design-sketch.md:11, "Adapted from pstack". This is attribution; playbooks/LICENSE.pstack exists [unverified whether the license requires the line].
- Greenfield, if it moves: notices.rs:437/812 pins "Greenfield init lane" as the onboarding notice's own prefix, not the skill heading, so moving the section is safe. Keep the notice text unchanged.

## Found outside scope

- docs/handbook/stages/planning.md:101 repeats the same stale "There is no trailing test cell per slice" model as F1.
- skills/bee-planning/playbooks/session/ exists, but there is no playbooks/session.md index. planning-reference.md:238-251 says "Every route class has one index file" and lists only 9 classes. I did not check whether `session` is a route class.
- SKILL.md:19/96/212 route to .bee/expertise/planning.md (22,883B) and tests.md (30,502B). I did not audit whether those load on every run; they are outside this branch.

## Proof

Command, exactly as the brief wrote it: `cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`, with the cargo bin directory on PATH. Output went to /tmp/planning-audit-proof.log. Exit status: `EXIT=0`.
- Clippy: `Finished dev profile … in 9.95s`, no errors.
- Tests: 5918 passed, 0 failed, 30 ignored, summed over 49 `test result:` lines. The unit suite alone printed `test result: ok. 5290 passed; 0 failed; 26 ignored`.
- The test build printed 11 rustc warnings in test-only code: unused `Result` ×4, unused variables `root`/`stored`/`win`, unneeded `mut` ×2, a duplicated attribute, and an unreachable `else`. Clippy's lint pass never builds the test target, so `-D warnings` does not catch them. They are unrelated to this audit.
- This is an audit only, so the proof shows the tree is green. It does not prove any of the findings above.
