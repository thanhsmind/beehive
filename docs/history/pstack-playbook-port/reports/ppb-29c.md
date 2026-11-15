# ppb-29c: check of the principles ported in ppb-29

Checked against commit `331557906` (ppb-29). The sources are the pstack
`principle-guard-the-context-window`, `principle-never-block-on-the-human`,
`principle-exhaust-the-design-space` and `principle-experience-first`
`SKILL.md` files (each directory holds only that one file). No file other than
this report was edited.

## Verdict

Every source rule of the four principles is present in bee. None is left out
and none is missing. No pstack-only tool name appears in the four new skills or
in their expertise sections, and no bee rule was removed: all four skills are
new in ppb-29, and the commit's only deleted lines are hashes and a timestamp in
`.bee/onboarding.json` and the release manifest, replaced by the regenerated
values.

"Present" means the rule is in the bee skill body, its depth section, or both.
Depth sections: `expertise/thinking.md` § Exhaust the Design Space (line 275),
§ Experience First (line 304), § Guard the Context Window (line 331), and
`expertise/decisions.md` § Never block on the human (line 129). The vendored
copies under `.bee/expertise/` match these files byte for byte, the four
`SKILL.md` and `LICENSE.pstack` files are identical in all five rendered trees
(`.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`, `.opencode`), and each
`LICENSE.pstack` matches the pstack `LICENSE` byte for byte.

The only source line not carried is the frontmatter
`disable-model-invocation: true`, a pstack loader flag rather than a rule;
the port notes (`port-notes/principles-ppb-29.md` line 15) record the drop.

## guard-the-context-window

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | The context window is finite and non-renewable within a session | present | skill body opening; depth opening |
| 2 | Every token that enters should earn its place | present | skill body opening; depth opening |
| 3 | Why: overflow degrades reasoning, creates compression artifacts and halts progress | present | skill Why line; depth opening |
| 4 | Why: unlike compute or time, spent context cannot be reclaimed | present | skill Why line ("cannot be reclaimed"); depth opening keeps the compute/time contrast |
| 5 | Isolate large payloads: verbose outputs, screenshots, large documents go to subagents; main context gets summaries | present | skill body; depth bullet. "Subagents" swapped for `bee dispatch prepare --runtime claude --kind gather --json` (D2). "Screenshots" is in the depth bullet only |
| 6 | Don't read what you won't use; read selectively by relevance | present | skill body; depth bullet |
| 7 | Keep frequently used content inline in the skill file, not in separate files costing a read | present | skill body; depth bullet |
| 8 | Size phases and cap scope: limit files per phase | present | skill body ("cap the files each one touches"); depth bullet, pointing at What makes a good unit of work in `planning.md` |
| 9 | Set turn budgets | present | depth bullet only |
| 10 | Account for mechanism costs | present | skill body (fan-out example); depth bullet, naming `bee-fanout` |

Bee additions, not from the source: a Not-when line (exact bytes decide the
answer; one or two files cost less to read than to delegate), pointing at
Prove it works (`tests.md` line 352).

## never-block-on-the-human

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | The human supervises asynchronously; make reasonable decisions, proceed, let the human course-correct after | present | skill body opening; depth opening |
| 2 | Code is cheap, waiting is expensive | present | skill body opening; depth opening |
| 3 | Why: every permission pause stalls the pipeline and makes the human the bottleneck | present | skill Why line; depth opening |
| 4 | Why: code changes are reversible and reviewable, so a wrong decision usually costs less than blocking | present | skill Why line ("a reversible mistake is cheap to find in review") |
| 5 | Proceed, then present: do X, explain why; don't ask "should I do X?" | present | skill body and Check line; depth bullet |
| 6 | Reserve questions for genuine ambiguity, when intent truly cannot be inferred | present | skill body; depth bullet; bee adds "one short yes/no question, never a menu" |
| 7 | Make the system self-healing: log a problem and fix it in the next round | present | depth bullet only |
| 8 | Supervision is async: the human reviews plans, diffs and changes on their own schedule; design for review after the fact | present | skill body opening (async supervision); depth bullet "Design for review after the fact" |
| 9 | Code is cheap, attention is scarce: a wrong implementation costs minutes, a blocked agent costs attention | present | skill body opening; depth opening |
| 10 | Boundary: irreversible actions (force-push, delete production data, send external messages) need confirmation | present | skill body and Not-when line; depth boundaries paragraph |
| 11 | Boundary: reversible actions (write code, edit notes, split tasks) proceed without blocking | present | skill body and Check line; depth boundaries paragraph lists all three examples |
| 12 | Boundary: product direction comes from the human; execution does not block | present | skill body and Not-when line; depth boundaries paragraph |

Bee additions, not from the source: name the assumption where the review will
read it (with Record at the moment of settlement, `decisions.md` line 65); a
locked decision is never reinterpreted to get unblocked (Reinterpretation is
quiet narrowing, `decisions.md` line 158); the uat acceptance at `bee close`
stays the owner's, which matches D4.

## exhaust-the-design-space

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | For a novel interaction or architectural decision with no precedent, explore several concrete alternatives before implementation | present | skill description and body; depth opening |
| 2 | Building the wrong thing costs more than exploring three options | present | skill Why line; depth opening |
| 3 | When the right answer is not obvious, build 2-3 competing prototypes or sketches | present | skill body; depth opening |
| 4 | Compare them side by side; only then commit | present | skill body; depth opening |
| 5 | Design it twice is this rule by another name | present | skill body; depth second paragraph, pointing at Design it twice in `planning.md` (line 80) |
| 6 | A second flavor of the first shape does not count | present | skill body and Check line; depth second paragraph |
| 7 | Applies: novel UI interactions with no prior art | present | skill body; depth third paragraph |
| 8 | Applies: architectural choices with multiple viable approaches | present | skill body; depth third paragraph |
| 9 | Applies: product design decisions that depend on feel, not logic | present | skill body; depth third paragraph |
| 10 | Not: mechanical implementation where the pattern is established | present | skill Not-when line; depth third paragraph |
| 11 | Not: bug fixes or refactors with a clear target state | present | skill Not-when line; depth third paragraph |
| 12 | Not: changes where constraints dictate a single viable approach | present | skill Not-when line; depth third paragraph |

Bee additions, not from the source: the blind-lane mechanism
(`bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`,
`--kind reviewer` judge, `bee blind check`), the same spelling
`skills/bee-architect/SKILL.md` line 45 already uses; `bee blind check` is
the `check` arm in `packages/bee-rs/crates/bee/src/verbs/blind/mod.rs`
line 1144. The depth also names `bee-architect` and `bee-arena`, both present in
`skills/`, and ends with "a verdict and a reason, never a menu".

## experience-first

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | The product is the experience; every technical decision helps or hurts it | present | skill body opening; depth opening |
| 2 | When implementation convenience conflicts with user delight, choose delight | present | skill body opening and Check line; depth opening |
| 3 | Say no to 1,000 things: every feature, control and option must earn its place | present | skill body; depth bullet |
| 4 | Ship less, ship better: three polished features beat ten rough ones | present | skill body; depth bullet |
| 5 | Prototype before committing: decisions are cheaper in throwaway HTML than production code | present | skill body; depth bullet ("throwaway markup", a wording change, not a tool swap) |
| 6 | Sweat the details: transitions, alignment, spacing, feedback, error states | present | skill body; depth bullet |
| 7 | Tighten the core loop: every feature serves the central workflow or gets out of the way | present | skill body; depth bullet |
| 8 | The user is whoever consumes the work: end user for a UI, the importing colleague for a library or internal API, the next maintainer | present | skill body; depth paragraph |
| 9 | Weigh their experience the same way and explain impact from their seat | present | skill body; depth paragraph |
| 10 | Foundations serve the experience, not the other way around | present | skill body ("Foundations serve the experience"); depth closing paragraph |
| 11 | Foundational thinking governs the sequence of work; this principle governs the target | present | skill body; depth closing paragraph, pointing at Foundational Thinking in `architecture.md` (line 330) |

Bee additions, not from the source: a Not-when line (a locked decision or a
stated constraint already settled the tradeoff, or a throwaway surface) and
"polish nobody asked for is scope", pointing at The smallest honest shape in
`planning.md` (line 31).

## pstack-only tool names

`rg -i 'cursor|/loop|graphite|bugbot|poteto|subagent|disable-model-invocation'`
over the four `SKILL.md` files and over the lines ppb-29 added to
`expertise/` returns nothing. The one source tool reference, "subagents" in
guard-the-context-window, is swapped for the gather spelling (D2) and recorded
in the port notes.

## Cross-references checked

Every section and skill the new text points at exists: `planning.md` §§ Design
it twice, What makes a good unit of work, The smallest honest shape;
`tests.md` § Prove it works; `architecture.md` § Foundational Thinking: Data
Structures First; `decisions.md` §§ Record at the moment of settlement,
Reinterpretation is quiet narrowing; `skills/bee-fanout`, `skills/bee-architect`,
`skills/bee-arena`. The dispatch kinds `gather`, `advisor` and `reviewer` are
known to `packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs` lines 72-74.

## Index rows

The four rows are in
`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
under Principle homes, each with its depth section and an `applied_at` entry.
`principle_index_parity` is the verify command for this cell; its output is in
the cell result.
