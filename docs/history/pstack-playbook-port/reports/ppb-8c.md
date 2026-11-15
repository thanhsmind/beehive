# ppb-8c — spike port checked against its sources

Scope: what ppb-8 wrote in commit `c710cdc42`. That is the class index
`skills/bee-planning/playbooks/spike.md`, the situation file
`spike/prototype.md`, and the port notes
`docs/history/pstack-playbook-port/port-notes/spike.md`. The files are checked
against the pstack source `pstack/skills/poteto-mode/playbooks/prototype.md`
and against the old bee `spike.md` from
`git show c710cdc42^:skills/bee-planning/playbooks/spike.md`.

Verdict: every source step is present or dropped with a port-notes entry. No
method step is missing. Every old enforced line is present, and the Proof
floor line is byte-identical in the index and in the situation file. The
pstack-only tool fence is clean. Every skill the files name exists in
`skills/`, except `verify-app`, which is named on the condition the cell
action prescribes. The one index line matches the contract grammar. There is
one small finding. It does not block the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 counts as missing. **missing** means a method step is gone with no reason.

## Old enforced lines (old `spike.md`)

| Old line | New home | Status |
|---|---|---|
| 1 Write the question as a YES/NO before you start, so you know when it is answered | `prototype.md` step 1, verbatim reason | present |
| 2 Timebox it and say the box out loud; a spike with no end turns into the build | `prototype.md` step 2, verbatim reason | present |
| 3 Keep the throwaway under `.bee/spikes/<feature>/`; the ANSWER is the deliverable, not the code | `prototype.md` step 4, verbatim | present |
| 3 Pointer to `planning-reference.md` "Shape bodies by mode" | `prototype.md` step 4; the heading exists at `skills/bee-planning/references/planning-reference.md:179` | present |
| 4 Record the answer with `bee decisions log`, then DELETE the spike; a kept spike gets shipped | `prototype.md` step 7, verbatim reason | present |
| 5 Comparing variants: build them all side by side; the answer is seen, not argued | `prototype.md` step 5, verbatim reason | present |
| 5 Each variant under `.bee/spikes/<feature>/` behind one switcher, each labelled | `prototype.md` step 5 | present |
| 5 Observe each variant on the real surface with the project's verification skill, otherwise log or print the thing decided; the observation is the proof | `prototype.md` step 6 (`verify-app` or the real interface; logging or printing for a behavioral decision; "The observation is the test here, not an assertion") | present |
| 5 The recorded answer names the chosen variant | `prototype.md` step 7 ("naming the chosen variant and the evidence") | present |
| 5 When the design space is open, add one variant the ask did not name | `prototype.md` step 5 | present |
| "A spike that becomes the implementation was never a spike." | `prototype.md` intro and closing line | present, verbatim |
| Proof line: the decision id and the observation of each variant | `prototype.md` lines 65-66 | present, verbatim |
| Proof floor line | `spike.md` and `prototype.md` | present; byte-identical (old `^Proof floor` line compared with each new file's by string equality) |

## pstack `prototype.md`

| Source step | New home | Status |
|---|---|---|
| Intro: you own the design decision, not the code; the real build follows Feature | intro, Feature became `playbooks/feature.md` | present (swapped, noted) |
| Intro: trigger phrases and the empirical-fork use | intro, verbatim | present |
| Intro: the Laziness Protocol's "smallest change" and the verification bar invert | intro, without the principle name | present; the `Laziness Protocol` citation dropped (noted) |
| Intro: speed over polish, no planning, rigor in picking the design cheaply, be bold | intro, verbatim | present |
| 1 Scope the decision; no decision means no prototype, route to Feature | step 1, with the bee YES/NO line and a reason added | present |
| 2 Gather references when the design space is open; skip when set | step 3, reason added | present |
| 3 Build throwaway in an isolated scratch dir; lightest stack; smallest script; no framework, tests or abstractions | step 4; scratch dir became `.bee/spikes/<feature>/` | present (swapped, noted) |
| 4 Build alternatives behind one switcher, each labelled | step 5 | present |
| 4 "the exhaust-the-design-space principle skill made cheap" | step 5 keeps the rule in plain words, no principle name | dropped (noted) |
| 5 Verify on the matching surface; screenshot via the control skill, drive the interaction; log or print for a behavioral decision; observation is the test | step 6; control skill became `verify-app` or the real interface | present (swapped, noted) |
| 6 Present alternatives, tradeoffs and a recommendation | step 7 | present |
| 6 The output is the decision plus the throwaway artifact | step 7: "the decision, not shippable code"; the artifact is deleted per the bee line | present (swapped, noted) |
| 6 Hand the chosen direction to Feature | step 7, `playbooks/feature.md` | present (swapped, noted) |
| 6 "(or `architect` for the shape)" | gone | dropped (noted; slice 3) |
| Reply: variants, evidence, tradeoffs, recommendation | Reply paragraph | present |
| Reply: the scratch path | Reply names the decision id instead | present (swapped, noted) |
| Reply: say plainly that the prototype is throwaway | Reply: "throwaway and is deleted" | present |

## Fences

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
- pstack-only tool names under `skills/bee-planning/playbooks/`: none.
  `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/`
  exits 1 with no output.
- Skills named: `verify-app` and the `bee-planning` reference path. `bee-planning`
  exists in `skills/`. `verify-app` is not in `skills/`; it is a project skill
  (this repo has it at `.claude/skills/verify-app`), and `prototype.md` names it
  only as "the project's `verify-app` skill when the project has one, otherwise
  through the real interface itself", the form the cell action prescribes for
  the control skill. `architect`, `exhaust-the-design-space` and
  `laziness-protocol` are not named in the playbook files.
- Playbook paths named: `playbooks/feature.md` exists.
- Index grammar (contract `913115b3`): `spike.md` has a `## Situations` heading
  and one line, `` - `prototype` — <triggers, English and Vietnamese> — delivers: <deliverable> ``.
  The name matches `[a-z0-9-]+`, the triggers and deliverable are non-empty, and
  the file has a matching `spike/prototype.md`. The section runs to end of file
  because no later `## ` heading exists, so the Proof floor line sits inside it;
  the parser skips any line that does not start with `` - ` ``, and `bugfix.md`
  has the same shape. `cargo test --release --test class_playbook_parity`: 9
  passed, 0 failed.
<!-- /bee:not-a-deferral -->

## Findings

1. Port notes list `exhaust-the-design-space` twice: once under Tool swaps
   ("became the bee spike line itself") and once under Dropped steps. The
   playbook text matches both (the rule stays in plain words, the name is
   gone), so this is only a duplicate note. It is harmless. Slice 3 should
   treat it as one re-add, not two.

No missing steps. Nothing calls for an edit to the port.
