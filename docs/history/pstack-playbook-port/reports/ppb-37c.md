# ppb-37c: check of the restored principle pointers and leftovers

Checked against commit `4c0f0b358` (ppb-37). The pstack source is
`/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/` at
commit `b9ddc83`. Playbook paths in bee are under
`skills/bee-planning/playbooks/`. This report is the only file this cell edited.

## Verdict

68 of the 69 principle pointers in the pstack sources now appear in the
matching bee file as a `bee-principle-<slug>`. The one gap is an example in
the architect rationale template. ppb-37 did not touch that file, so the gap
is older than ppb-37 (finding 1). Every leftover from ppb-24c and ppb-25c is
closed. The ppb-37 diff adds no pstack-only tool name and removes no bee
rule. Two cosmetic gaps remain in the port notes: three summary paragraphs
still say "Still waiting" (finding 2), and one multi-phase-plan line runs past
80 columns (finding 3). Neither blocks the cap.

## Method

A script greps every pstack playbook and every pstack skill that a D5 bee skill
ports (`architect`, `arena`, `no-comments`, `figure-it-out`, `interrogate`,
`show-me-your-work`, `swarm`, `tdd`, `reflect`, `recall`, `blast-radius`) for
each pstack principle name. It maps each file to its bee home and counts
`bee-principle-<slug>` there. Renamed principles map like this: `fix-root-causes`
becomes `crash-site-versus-fault-site`, and `migrate-callers-then-delete-legacy-apis`
becomes `migrate-callers-then-delete`. A second grep looked for principle names
written as prose (for example "Laziness Protocol" or "root cause"). It found
three more pointers: `prototype.md:5`, `feature.md:12` and `arena/SKILL.md:49`.
All three are restored. The other prose matches (`bug-fix.md:17` "root
cause", `interrogate/references/rubric.md:19` "Root Causes") are plain words,
not pointers.

## Every source pointer, by status

"present" means the bee file names the `bee-principle-<slug>`. Where pstack
cites the same principle twice in one file, a single bee pointer in the same
section covers both. Two places work this way:

- `architect/SKILL.md:77` (Phase E, step 2). The pointer sits once at the head
  of Phase E (`skills/bee-architect/SKILL.md:72`).
- `refactoring.md:7` and `:12`. Bee has two pointers, at steps 1 and 8.

| # | pstack source:line | Principle | bee file | Status |
|---|--------------------|-----------|----------|--------|
| 1 | `architect/references/rationale-template.md:15` | `boundary-discipline` | `skills/bee-architect/references/rationale-template.md` | missing (bare name, see finding 1) |
| 2 | `architect/references/runner-prompt.md:12` | `separate-before-serializing-shared-state` | `skills/bee-architect/references/runner-prompt.md` | present |
| 3 | `architect/references/runner-prompt.md:14` | `encode-lessons-in-structure` | `skills/bee-architect/references/runner-prompt.md` | present |
| 4 | `architect/references/runner-prompt.md:15` | `boundary-discipline` | `skills/bee-architect/references/runner-prompt.md` | present |
| 5 | `architect/references/runner-prompt.md:17` | `make-operations-idempotent` | `skills/bee-architect/references/runner-prompt.md` | present |
| 6 | `architect/references/runner-prompt.md:18` | `laziness-protocol` | `skills/bee-architect/references/runner-prompt.md` | present |
| 7 | `architect/references/runner-prompt.md:18` | `minimize-reader-load` | `skills/bee-architect/references/runner-prompt.md` | present |
| 8 | `architect/SKILL.md:35` | `exhaust-the-design-space` | `skills/bee-architect/SKILL.md` | present |
| 9 | `architect/SKILL.md:49` | `foundational-thinking` | `skills/bee-architect/SKILL.md` | present |
| 10 | `architect/SKILL.md:49` | `outcome-oriented-execution` | `skills/bee-architect/SKILL.md` | present |
| 11 | `architect/SKILL.md:61` | `fix-root-causes` | `skills/bee-architect/SKILL.md` | present |
| 12 | `architect/SKILL.md:61` | `redesign-from-first-principles` | `skills/bee-architect/SKILL.md` | present |
| 13 | `architect/SKILL.md:77` | `redesign-from-first-principles` | `skills/bee-architect/SKILL.md` | present |
| 14 | `architect/SKILL.md:78` | `subtract-before-you-add` | `skills/bee-architect/SKILL.md` | present |
| 15 | `arena/SKILL.md:29` | `separate-before-serializing-shared-state` | `skills/bee-arena/SKILL.md` | present |
| 16 | `arena/SKILL.md:57` | `redesign-from-first-principles` | `skills/bee-arena/SKILL.md` | present |
| 17 | `arena/SKILL.md:65` | `prove-it-works` | `skills/bee-arena/SKILL.md` | present |
| 18 | `figure-it-out/SKILL.md:21` | `prove-it-works` | `skills/bee-figure-it-out/SKILL.md` | present |
| 19 | `figure-it-out/SKILL.md:25` | `never-block-on-the-human` | `skills/bee-figure-it-out/SKILL.md` | present |
| 20 | `figure-it-out/SKILL.md:29` | `foundational-thinking` | `skills/bee-figure-it-out/SKILL.md` | present |
| 21 | `figure-it-out/SKILL.md:32` | `laziness-protocol` | `skills/bee-figure-it-out/SKILL.md` | present |
| 22 | `figure-it-out/SKILL.md:33` | `separate-before-serializing-shared-state` | `skills/bee-figure-it-out/SKILL.md` | present |
| 23 | `figure-it-out/SKILL.md:41` | `sequence-verifiable-units` | `skills/bee-figure-it-out/SKILL.md` | present |
| 24 | `figure-it-out/SKILL.md:53` | `encode-lessons-in-structure` | `skills/bee-figure-it-out/SKILL.md` | present |
| 25 | `no-comments/SKILL.md:22` | `fix-root-causes` | `skills/bee-comment-review/SKILL.md` | present |
| 26 | `no-comments/SKILL.md:22` | `redesign-from-first-principles` | `skills/bee-comment-review/SKILL.md` | present |
| 27 | `poteto-mode/playbooks/authoring-a-skill.md:10` | `encode-lessons-in-structure` | `docs/authoring-a-skill.md` | present |
| 28 | `poteto-mode/playbooks/autonomous-run.md:8` | `sequence-verifiable-units` | `session/autonomous-run.md` | present |
| 29 | `poteto-mode/playbooks/autopilot-full.md:6` | `prove-it-works` | `session/autopilot-full.md` | present |
| 30 | `poteto-mode/playbooks/bug-fix.md:12` | `sequence-verifiable-units` | `bugfix/bug-fix.md` | present |
| 31 | `poteto-mode/playbooks/feature.md:10` | `separate-before-serializing-shared-state` | `feature/feature.md` | present |
| 32 | `poteto-mode/playbooks/feature.md:12` | `model-the-domain` | `feature/feature.md` | present |
| 33 | `poteto-mode/playbooks/feature.md:15` | `sequence-verifiable-units` | `feature/feature.md` | present |
| 34 | `poteto-mode/playbooks/hillclimb.md:12` | `guard-the-context-window` | `perf/hillclimb.md` | present |
| 35 | `poteto-mode/playbooks/hillclimb.md:12` | `separate-before-serializing-shared-state` | `perf/hillclimb.md` | present |
| 36 | `poteto-mode/playbooks/hillclimb.md:16` | `sequence-verifiable-units` | `perf/hillclimb.md` | present |
| 37 | `poteto-mode/playbooks/hillclimb.md:17` | `laziness-protocol` | `perf/hillclimb.md` | present |
| 38 | `poteto-mode/playbooks/hillclimb.md:5` | `prove-it-works` | `perf/hillclimb.md` | present |
| 39 | `poteto-mode/playbooks/hillclimb.md:8` | `build-the-lever` | `perf/hillclimb.md` | present |
| 40 | `poteto-mode/playbooks/multi-phase-plan.md:10` | `encode-lessons-in-structure` | `feature/multi-phase-plan.md` | present |
| 41 | `poteto-mode/playbooks/multi-phase-plan.md:13` | `prove-it-works` | `feature/multi-phase-plan.md` | present |
| 42 | `poteto-mode/playbooks/multi-phase-plan.md:6` | `never-block-on-the-human` | `feature/multi-phase-plan.md` | present |
| 43 | `poteto-mode/playbooks/multi-phase-plan.md:7` | `guard-the-context-window` | `feature/multi-phase-plan.md` | present |
| 44 | `poteto-mode/playbooks/multi-phase-plan.md:8` | `sequence-verifiable-units` | `feature/multi-phase-plan.md` | present |
| 45 | `poteto-mode/playbooks/orchestrate.md:19` | `separate-before-serializing-shared-state` | `session/orchestrate.md` | present |
| 46 | `poteto-mode/playbooks/orchestrate.md:27` | `encode-lessons-in-structure` | `session/orchestrate.md` | present |
| 47 | `poteto-mode/playbooks/perf-issue.md:17` | `sequence-verifiable-units` | `perf/perf-issue.md` | present |
| 48 | `poteto-mode/playbooks/prototype.md:10` | `exhaust-the-design-space` | `spike/prototype.md` | present |
| 49 | `poteto-mode/playbooks/refactoring.md:10` | `laziness-protocol` | `refactor/refactoring.md` | present |
| 50 | `poteto-mode/playbooks/refactoring.md:10` | `subtract-before-you-add` | `refactor/refactoring.md` | present |
| 51 | `poteto-mode/playbooks/refactoring.md:11` | `migrate-callers-then-delete-legacy-apis` | `refactor/refactoring.md` | present |
| 52 | `poteto-mode/playbooks/refactoring.md:12` | `prove-it-works` | `refactor/refactoring.md` | present |
| 53 | `poteto-mode/playbooks/refactoring.md:13` | `minimize-reader-load` | `refactor/refactoring.md` | present |
| 54 | `poteto-mode/playbooks/refactoring.md:14` | `sequence-verifiable-units` | `refactor/refactoring.md` | present |
| 55 | `poteto-mode/playbooks/refactoring.md:7` | `prove-it-works` | `refactor/refactoring.md` | present |
| 56 | `poteto-mode/playbooks/refactoring.md:8` | `model-the-domain` | `refactor/refactoring.md` | present |
| 57 | `poteto-mode/playbooks/refactoring.md:9` | `foundational-thinking` | `refactor/refactoring.md` | present |
| 58 | `poteto-mode/playbooks/refactoring.md:9` | `redesign-from-first-principles` | `refactor/refactoring.md` | present |
| 59 | `poteto-mode/playbooks/runtime-forensics.md:6` | `guard-the-context-window` | `research/runtime-forensics.md` | present |
| 60 | `poteto-mode/playbooks/session-pickup.md:11` | `prove-it-works` | `session/session-pickup.md` | present |
| 61 | `poteto-mode/playbooks/session-pickup.md:7` | `guard-the-context-window` | `session/session-pickup.md` | present |
| 62 | `poteto-mode/playbooks/trace-forensics.md:7` | `guard-the-context-window` | `research/trace-forensics.md` | present |
| 63 | `poteto-mode/playbooks/visual-parity.md:7` | `separate-before-serializing-shared-state` | `refactor/visual-parity.md` | present |
| 64 | `poteto-mode/playbooks/worktree-cleanup.md:5` | `build-the-lever` | `session/worktree-cleanup.md` | present |
| 65 | `poteto-mode/playbooks/worktree-cleanup.md:5` | `encode-lessons-in-structure` | `session/worktree-cleanup.md` | present |
| 66 | `poteto-mode/playbooks/worktree-cleanup.md:6` | `prove-it-works` | `session/worktree-cleanup.md` | present |
| 67 | `poteto-mode/playbooks/worktree-cleanup.md:7` | `guard-the-context-window` | `session/worktree-cleanup.md` | present |
| 68 | `reflect/SKILL.md:53` | `encode-lessons-in-structure` | `skills/bee-reflect/SKILL.md` | present |
| 69 | `show-me-your-work/SKILL.md:52` | `encode-lessons-in-structure` | `skills/bee-show-me-your-work/SKILL.md` | present |

### Left out with a named bee home

These pointers are not in the bee text. `principle-pointers.md` § "Not
restored, named here" names each one:

- `principle-minimize-reader-load` links to `guard-the-context-window` in
  pstack (`principle-minimize-reader-load/SKILL.md:13`). The bee principle
  keeps the sentence without the link (`principles-ppb-27.md`). It stays open
  until a cell owns that principle, because the principle was outside ppb-37's
  files. This is a correct left-out.
- `principle-experience-first` says "prototype in throwaway markup" and points
  to a design skill. No `skills/bee-design` exists. Note that
  `.claude/skills/bee-design/SKILL.md` is tracked in git, but there is no
  source under `skills/`, so nothing renders it to the other trees. The
  left-out holds for vendored hosts.
- `deslop` has no bee counterpart and no decision brings one in.
- The `arena` pointer inside `bee-architect` is replaced by the blind lanes.

### Missing

1. **`architect/references/rationale-template.md:15`, `boundary-discipline`
   (consider).** The bee file still carries the pstack example citation
   verbatim: "Cite the principle behind each decision (e.g., `per
   boundary-discipline`)". The principle exists as
   `bee-principle-boundary-discipline`, but the example uses the bare pstack
   name. The file was last changed in `52777acd0` (the architect port), so
   the gap is older than ppb-37 and outside the port-note drop lists. ppb-37
   only restored pointers a note had dropped. The fix is one word: change the
   example to `per bee-principle-boundary-discipline`.

## Leftovers from ppb-24c and ppb-25c

| Leftover | Status | Evidence |
|----------|--------|----------|
| ppb-24c #25: `worktree-cleanup` names `bee-arena` | closed | `session/worktree-cleanup.md:22` "A herded worker spawns `bee-arena`" |
| ppb-25c A: multi-phase-plan owners rebase | closed | `feature/multi-phase-plan.md:112` and `:181` send drift to the merger as a cell. `rg -i rebase` over the playbooks finds no line telling an owner or worker to rebase. Every match is a ban, or a merger-side note in `shipping.md:30` and `orchestrate.md:227` |
| ppb-9c #2: gathers on the read role | closed | `feature/multi-phase-plan.md:16` "on the read role's configured model" |
| ppb-9c #3a: "Rebase into small, ordered commits" | noted | `port-notes/feature.md:192-199` § "Fixes in ppb-37" gives the swap and its reason (`b25486e3`) |
| ppb-9c #3b: "the app is small" rebuttal | restored | `feature/feature.md:58`. Noted in `port-notes/feature.md:200-204` |
| ppb-9c #4: `encode-lessons-in-structure` pointer | restored | present in `feature/multi-phase-plan.md` (table row for `multi-phase-plan.md:10`) |
| ppb-25c B: misplaced Restored note in `session-ppb-12.md` | closed | the paragraph now sits at line 102, after the last drop bullet (lines 89-100) |
| ppb-11c table-only rows not noted | closed | `port-notes/session-ppb-11.md:187-195`: the `gh` probes, "pushes its branch", the `units.tsv` PR column, "merge order" |
| ppb-24c "Still waiting" principles | closed in the playbooks | all 7 principles are restored (table above). See finding 2 for the notes |

Still open from ppb-25c, and outside the scope of both ppb-37 and this cell:
ppb-9c #1b (multi-phase-plan has no floor line, optional), ppb-11c #4
(session-pickup step 1 order, optional), and ppb-25c C (unwrapped lines,
cosmetic).

### Finding 2 (cosmetic): three stale "Still waiting" summaries

The per-entry notes carry "Restored (ppb-37)", but three end-of-file summary
paragraphs still say each principle "keeps its method as a plain sentence". The
playbooks no longer read that way:

- `port-notes/research.md:163` (`guard-the-context-window`)
- `port-notes/perf.md:104` (`prove-it-works`, `guard-the-context-window`,
  `sequence-verifiable-units`, `laziness-protocol`)
- `port-notes/spike.md:63` (`exhaust-the-design-space`, `laziness-protocol`)

`feature.md:187` and `refactor.md:105` replaced the same paragraph with
"Restored (ppb-37), all … now exist", so these three are inconsistent with
them. The fix is the same edit in each file.

### Finding 3 (cosmetic): a long line

`feature/multi-phase-plan.md:16` runs past 80 columns, as do lines 112 and 181.
The file's skeleton section already used long checklist lines. This does not
change what the text says.

## pstack-only tool names

`rg -i 'cursor|graphite|bugbot|poteto-agent|subagent_type|grok|/loop'` over
`skills/bee-planning/playbooks`, `skills/bee-architect`, `skills/bee-arena`,
`skills/bee-comment-review` and `skills/bee-figure-it-out` prints nothing. Every
`bee-principle-<slug>` that ppb-37 added names a directory that exists under
`skills/`.

## Bee rules removed

None. `git show 4c0f0b3 --word-diff=plain -- skills` shows only these
removals:

- punctuation moved so a pointer could join the sentence;
- prose principle names swapped for their slugs ("exhausting the design space",
  "foundational thinking", "A redesign-from-first-principles stance");
- the swaps the cell asked for: "Rebase onto" and "Rebased onto current trunk
  after the verdict" became merger cells, "explore" became "read", "arena"
  became `bee-arena`, and "press on" became "run `bee-interrogate` over … press
  on it".

No Proof floor, routed-principle, statechart or formal-verify line changed.
`diff -rq` of the five touched source directories against `.claude-plugin`,
`.codex-plugin`, `.agents` and `.opencode` reports no difference.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity`

```
running 5 tests
test every_row_carries_a_non_empty_spoken_line_before_its_applied_at ... ok
test every_principle_skill_has_exactly_one_row_and_every_row_a_skill ... ok
test every_row_routes_only_to_known_class_values ... ok
test every_principle_skill_ships_in_both_plugin_trees ... ok
test every_row_anchors_an_existing_guide_heading ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
