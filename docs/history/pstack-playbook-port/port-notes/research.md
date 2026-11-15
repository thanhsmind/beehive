# Port notes — research class

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). Cell ppb-6. This
file is the only place a pstack-only tool is named; no ported file names one. `skills/bee-planning/playbooks/research.md` is now the class index
with four situations.

## investigation

Source `playbooks/investigation.md`, merged with the bee `research` class
playbook (D3) into `skills/bee-planning/playbooks/research/investigation.md`.

Tool swaps:

- Routing through the `how` skill (Explain mode, Critique mode) became
  `bee-how` run as a gather (`bee dispatch prepare --kind gather`); the
  motivation route through the `why` skill became `bee-why`.
- The `unslop` skill became `bee-unslop`.
- The pointer for the hand-back after an investigation that precedes a code
  change (Bug fix or Feature) became a re-route to the `bugfix` or `feature`
  class.

Dropped steps:

- The one-line throughput checkpoint (`throughput checkpoint: n/a, read-only
  investigation`): bee has no throughput checkpoint concept, so the step has no
  counterpart.
- The "no `architect`" clause: the skill does not exist in bee yet (D5, slice
  3), and the situation has no step that would call it. Restored (ppb-24): the
  closing paragraph of `investigation` runs `bee-architect` only for the code
  change that follows, never inside the read-only account.
- The "no PR, no babysit" clause: it restated that a read-only class lands no
  work; the read-only step already says it.
- The reference to the Autonomy section of the router for pushing back on a
  false premise: bee has no such section. The Reply line keeps the rule and
  gives its reason.

Kept from the bee `research` playbook (D3): read-only outcome (step 1), the
runtime-path trace and its one home `bee-researching/references/trace-and-provenance.md`
(step 3), the empty-sources sweep (step 4), the anchors rule (step 6), the
recommendation-plus-tradeoffs rule (step 5), the re-route of an answer that
leads to a change, the paragraph naming the route as the existing `research`
class with decision `f1ffa7bd` and backlog `p-69bee217`, the Proof line and the
class Proof floor line (now in the index).

Moved out of this file: the "When the symptom is live at runtime" paragraph,
which became the body of `runtime-forensics`.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`).

## runtime-forensics

Source `playbooks/runtime-forensics.md`, merged with the live-runtime
paragraph of the bee `research` playbook (D3) into
`skills/bee-planning/playbooks/research/runtime-forensics.md`.

Tool swaps:

- The control skill (capture the live signal on the matching surface) became
  the project's `verify-app` skill, or the real interface when the project has
  none.
- Parsing large artifacts in a subagent, with the `guard-the-context-window`
  principle skill, became a gather dispatch (`bee dispatch prepare --kind
  gather`) that returns the reduced finding.
- The hand-back to Bug fix or Perf became a re-route to `bugfix` or `perf`.
- Injecting instrumentation "via CDP eval" became "into the running process":
  CDP is a Cursor and browser tool, and the instruction holds for any live
  process.

Dropped steps:

- The one-line throughput checkpoint (`throughput checkpoint: n/a, read-only
  forensics`): no bee counterpart.
- The `guard-the-context-window` principle citation: no bee principle of that
  name exists yet (D5, slice 3). The step keeps its reason: the raw artifact fills the lead's context.
  Restored (ppb-37): runtime-forensics step 2 names
  `bee-principle-guard-the-context-window`.

Moved in from the bee `research` playbook (D3): "Capture a real profile or
trace", "Reduce it through a gather dispatch", "Confirm the reduction with one
instrumented run, and map it to `path:line`" and "Without a before-and-after
pair, label the cause a hypothesis" are steps 1 to 4, with the hypothesis
label in step 3.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`), and a Proof line.

## trace-forensics

Source `playbooks/trace-forensics.md` into
`skills/bee-planning/playbooks/research/trace-forensics.md`. The bee `research`
playbook carries no overlapping text, so nothing merges.

Tool swaps:

- Parsing large artifacts in a subagent, with the `principle-guard-the-context-window`
  skill, became a gather dispatch (`bee dispatch prepare --kind gather`).
- The pointer to the Runtime forensics playbook became
  `research/runtime-forensics.md`.
- The hand-back to Bug fix or Perf issue became a re-route to `bugfix` or
  `perf`.

Dropped steps:

- The one-line throughput checkpoint: no bee counterpart.
- The `principle-guard-the-context-window` citation: no bee principle of that
  name exists yet (D5, slice 3). The step keeps its reason.
  Restored (ppb-37): trace-forensics step 1 names
  `bee-principle-guard-the-context-window`.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`), and a Proof line.

## eval

Source `playbooks/eval.md` into
`skills/bee-planning/playbooks/research/eval.md`. The bee `research` playbook
carries no overlapping text, so nothing merges. The class is `research`
because an eval delivers a verdict and a recommendation and edits nothing.

Tool swaps:

- Spawning N parallel candidates "per the arena skill's Phase B" became
  dispatch through `bee dispatch prepare`; the door returns each worker's model
  and the playbook says never to hand-pick one, replacing "on different
  models".
- Spawning one blinded judge "per the arena skill's Phase C" on a different
  model family became `bee dispatch prepare --kind reviewer`, which returns the
  review tier's model; the step tells the lead to say in the reply when that
  model equals the candidates' model.
- The Cursor-specific transcript path (the `agent-transcripts/` directory the
  system prompt names, and the warning not to glob across `~/.cursor/projects/`)
  became "the candidate's own transcript under the active workspace and no
  other", keeping the privacy reason.

Dropped steps:

- The `arena` skill reference in steps 4 and 5: the skill does not exist in
  bee yet (D5, slice 3 re-adds it). The steps keep the spawn and the judge
  through the dispatch door. Restored (ppb-24): both steps name `bee-arena`
  Phase B and Phase C.
- The model-per-candidate spread ("on different models"): the dispatch door
  fixes the model per role and bee has no per-candidate model flag, so the
  spread is dropped, not faked. Restored (ppb-24): `bee-arena` Phase B sends
  each candidate through a `lane-1` to `lane-3` role, whose model is a team
  setting, so step 4 names the roles and the spread comes back.

Kept as written: the word `arena` inside the blinding list. It names a cue word
a candidate must never see, not a skill the playbook calls, and removing it
would weaken the blinding rule.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`), and a Proof line.

## Steps re-added in ppb-24

Restored: `arena` (eval steps 4 and 5, now `bee-arena`, with the per-candidate
model spread through the `lane-1` to `lane-3` roles) and the `architect`
clause of `investigation` (now `bee-architect`, read-only text).

Still waiting, because no skill of that name exists in `skills/`:
`guard-the-context-window` (runtime-forensics step 2, trace-forensics step 1).
It keeps its method as a plain sentence.
