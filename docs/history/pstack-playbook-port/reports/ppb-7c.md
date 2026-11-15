# ppb-7c — refactor port checked against its sources

Scope: what ppb-7 wrote in commit `1f29c38fb`. That is the class index
`skills/bee-planning/playbooks/refactor.md`, the situation files
`refactor/refactoring.md` and `refactor/visual-parity.md`, and the port notes
`docs/history/pstack-playbook-port/port-notes/refactor.md`. The files are
checked against the pstack sources `pstack/skills/poteto-mode/playbooks/`
`refactoring.md` and `visual-parity.md`, and against the old bee `refactor.md`
from `git show 1f29c38fb^:skills/bee-planning/playbooks/refactor.md`.

Verdict: every source step is present or dropped with a port-notes entry. No
method step is missing. Every old enforced line is present, and the Proof
floor line is byte-identical in the index and in both situation files. The
pstack-only tool fence is clean. Every skill the files name exists in
`skills/`, except `verify-app`, which is named on the condition the cell
action prescribes. Both index lines match the contract grammar. There are two
small findings. Neither blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 counts as missing. **missing** means a method step is gone with no reason.

## Old enforced lines (old `refactor.md`)

| Old line | New home | Status |
|---|---|---|
| 1 Record existing behavior FIRST: characterization test, snapshot or equivalence script | `refactoring.md` step 1 | present |
| 1 A type check or a lint is not a record | `refactoring.md` step 1, verbatim | present |
| 2 Prove the record green on the UNCHANGED tree; a red base hides new reds | `refactoring.md` step 2, verbatim reason | present |
| 3 Change structure in small steps, so a red points at one step | `refactoring.md` step 6 | present |
| 4 The record stays green at every step; a red found later has many suspects | step 6 ("each keeping the record green") and the opening paragraph (the reason) | present; the reason moved to the paragraph |
| 5 A behavior change is not a refactor; it is a separate cell | opening paragraph, verbatim | present |
| 6 Subtract first; an early cell deletes dead code and one-caller wrappers | `refactoring.md` step 5 | present |
| 7 An API move migrates every caller and deletes the old API in one slice; two live APIs drift apart | `refactoring.md` step 6 | present |
| 7 Then search strings and docs for the old name | `refactoring.md` step 6 ("strings, prose and back-references") | present; "docs" became "prose" |
| 8 Never edit the record, the harness or the baseline to make a step green | `refactoring.md` step 7, verbatim reason | present |
| 9 Keep the diff only if it lowers what a reader must hold in mind | `refactoring.md` step 9, verbatim | present |
| Proof line: the record, green before the change and green after it | `refactoring.md` | present, verbatim |
| Proof floor line | `refactor.md` line 10, `refactoring.md`, `visual-parity.md` | present; byte-identical (checked with `diff` of the `^Proof floor` line, old against each new file) |

The refactor class has no routed-principle, statechart or formal-verify lines
in the old text, so D3 had nothing more to carry.

## refactoring (pstack `refactoring.md`)

| Source | Status |
|---|---|
| Lead line "You own the contract. The structure changes; the behavior does not." | present |
| Trigger words: refactor, rename, extract, inline, dedupe, restructure, move this module, tidy up this area | present |
| Distinct from Feature (adds behavior) and Bug fix (corrects it) | present ("a feature", "a bug fix") |
| A refactor that smuggles a behavior change loses its safety net | present |
| A revealed feature or bug is split out; the structural change ships first against the pinned contract | present |
| A redesign is allowed, but name it and route to Feature | present: "route it to the feature class" (noted) |
| Large or cross-cutting work belongs to the figure-it-out skill | present as "anything larger gets a plan of its own"; figure-it-out pointer dropped (noted) |
| 1 Pin the contract: run `how` over the subsystem | present: `bee-how` as a gather through `bee dispatch prepare --kind gather` (noted) |
| 1 Characterization test, snapshot or equivalence harness before any structure moves | present |
| 1 The harness makes "refactor" a checkable claim (principle-prove-it-works) | present; the citation dropped (noted), the reason kept |
| 1 No coverage: write the pin before touching structure | present, verbatim |
| 1 Type check and lint are not a pin | present (merged with the old bee line) |
| 2 Name the missing structure (principle-model-the-domain): state machine, table or registry, typed model, reducer | present as step 3; the citation dropped (noted) |
| 2 Boring code stays when the shape is clear and local; the reshape must delete branches or invalid states, not add indirection | present (step 3) |
| 3 Name the target shape: layout, types, call graph as if built today (foundational-thinking, redesign-from-first-principles) | present as step 4; both citations dropped (noted) |
| 3 A target crossing a function boundary runs `architect` first | dropped (noted); step 4 keeps "write down the contract it keeps before dispatching" |
| 4 Subtract before you add: dead weight, one-caller wrappers, redundant validators, orphan references | present as step 5; `bee-principle-subtract-before-you-add` (noted) |
| 4 The smallest change that reaches the target ships (principle-laziness-protocol) | present (step 5); the citation dropped (noted) |
| 4 A speculative cleanup that "might help" gets reverted | present (step 5) |
| 5 Small behavior-preserving steps, each keeping the pin green | present (step 6) |
| 5 API reshapes migrate every caller and delete the old API in the same wave (principle-migrate-callers-then-delete-legacy-apis) | present (step 6), "wave" became "slice"; `bee-principle-migrate-callers-then-delete` (noted) |
| 5 No compatibility shims, no parallel old-and-new paths | present (step 6) |
| 5 Spot-check every rename; renames miss usages in strings, prose and back-references | present (step 6) |
| 5 Delegate the mechanical edits to a subagent on the configured refactoring model (default named model), with a specific scope; review the diff yourself | present: `bee dispatch prepare --kind cell` with the scope and the diff review kept; the named model dropped (noted) |
| 6 Prove behavior unchanged on the real artifact, not "it compiles" (principle-prove-it-works) | present as step 8; the citation dropped (noted) |
| 6 Equivalence check: diff of old and new outputs, recorded baseline replay, smoke run via the control skill | present (step 8): the project's `verify-app` or the real interface (noted) |
| 6 Own the verification; do not trust a delegate's "looks good" | present (step 8) |
| 7 Confirm the change earns its place: reduced reader load (principle-minimize-reader-load) | present as step 9, merged with the old bee line; the citation dropped (noted) |
| 7 Fewer layers, less hidden state, fewer indirections without a second consumer; revert if no lower reader load | present (step 9) |
| 8 Rebase into small ordered commits: subtraction, reshape, follow-on cleanup; one revert undoes one slice | present as step 10 ("Order the commits"; the word "rebase" is gone, the order is kept) |
| 8 Shape them with sequence-verifiable-units; each slice stays green before the next | present (step 10); the citation dropped (noted) |
| 8 Run Opening a PR | present: the bee close and `bee worktree merge` path, `uat` the owner's (noted) |
| Reply: structure changed, pin, equivalence proof, reader-load delta, shipped and reverted; no new behavior | present, verbatim |

## visual-parity (pstack `visual-parity.md`)

| Source | Status |
|---|---|
| Lead line "You own pixel-exact equivalence. The baseline is the spec; you do not touch it." | present |
| Triggers: make X match Y exactly, styling-system migrations, porting a UI across frameworks | present |
| Equivalence by image diff, not by eye | present, with an added reason |
| 1 Baseline first: harness screenshots the current component across states, plus the target when matching two implementations | present |
| 1 No baseline, no parity claim; a blocking prerequisite, not a follow-up | present |
| 2 Anti-shortcut clauses: no harness modifications, no baseline tampering, no restructuring to make a diff pass | present |
| 2 If the baseline looks wrong, stop and ask; don't edit it | present |
| 3 Migrate one component at a time; each is an independent artifact | present |
| 3 Parallelize across worktrees, one owner per component (separate-before-serializing-shared-state) | present: one cell per component through `bee dispatch prepare --kind cell`, `bee-principle-separate-before-serializing-shared-state` (noted) |
| 3 Shared primitives migrate first as a blocking phase | present |
| 4 Verify each component by image diff on the matching surface via the control skill | present: the project's `verify-app` or the real interface (noted) |
| 4 A nonzero diff is a fail; investigate the pixel delta, don't wave it through | present |
| 4 `/loop` per component until the diff is zero | dropped (noted); the step keeps "rerun the diff per component until it is zero" |
| 5 Run Opening a PR per component or per safe batch | present: the bee close and `bee worktree merge` path per component or batch (noted) |
| Reply: components migrated, diff result for each, baseline harness location, what's left | present |

Added and noted: a Proof line, because the pstack text had only a Reply.

## pstack-only tool fence

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/`
exits 1, so no line matches. A wider sweep of the refactor files for `arena`,
`architect`, `figure-it-out`, `interrogate`, `show-me-your-work`, `swarm`,
`tdd`, `grok` and `/loop` also exits 1.

## Skills named

The refactor files name `bee-how`, `bee-principle-subtract-before-you-add`,
`bee-principle-migrate-callers-then-delete`,
`bee-principle-separate-before-serializing-shared-state` and `verify-app`. The
first four are directories under `skills/`. `verify-app` is not under
`skills/`. It lives in `.claude/skills/verify-app`, `.agents/skills/verify-app`
and `.opencode/skills/verify-app`, and both files name it only as "the
project's `verify-app` skill when it has one, otherwise the real interface".
That is the wording the cell action prescribes for the control skill, so it is
not a broken reference. The `bee dispatch prepare --kind gather|cell` forms
match the kinds the dispatch hook names
(`packages/bee-rs/crates/bee/src/hooks/model_guard.rs:545`).

Every dropped principle citation is real: `ls skills` has no directory for
prove-it-works, model-the-domain, foundational-thinking,
redesign-from-first-principles, laziness-protocol, minimize-reader-load,
sequence-verifiable-units, architect or figure-it-out.

## Index grammar (contract 913115b3)

`refactor.md` has one `## Situations` heading and two lines under it:

- `refactoring` — matches `^- \`[a-z0-9-]+\` — .+ — delivers: .+$`; triggers
  in English and Vietnamese.
- `visual-parity` — matches; triggers in English and Vietnamese.

The section has no following `## ` heading, so it runs to the end of the file
and holds the Proof floor line. `parse_situations` and `parse_line` in
`packages/bee-rs/crates/bee/src/situations.rs:24-54` skip a line that does not
start with ``- ` ``, so the Proof floor line yields no situation. Each named
situation has its file under `refactor/`, and no file there lacks a line.

## Findings

1. **"Search strings and docs" became "strings, prose and back-references".
   Verified, low severity.** The old bee step 7 asked for a search of docs for
   the old name. The merged step 6 takes the pstack wording, and "prose"
   covers docs only by reading. Nothing is lost in practice. If the leader
   wants the old enforced line word for word, add "docs" to the list.
2. **The pstack commit order lost the word "rebase". Verified, low
   severity.** Step 10 says "Order the commits so they tell the story". The
   pstack step said "Rebase into small ordered commits". The order and the
   one-revert-one-slice reason are kept, so the method holds. The port notes
   do not mention the wording change. This is a wording note, not a dropped
   step.

Checked and not filed: refactoring step 10 describes landing through
`bee worktree merge`, which could read as the D4 merge-on-green rule. It only
names the path and leaves `uat` with the owner, so it does not claim D4 for a
class D4 does not list. The old line "a red found later has many suspects"
looked dropped from the steps, but the opening paragraph carries it.
