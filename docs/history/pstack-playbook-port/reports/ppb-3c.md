# ppb-3c — the bug-fix port checked against its sources

Cell ppb-3c checks the ppb-3 port (commits `25bccf3b6`, `a8be04ffd`) against
its two sources. It covers D2 (`063a21e4`), D3 (`5a1fc656`) and decision
`913115b3`.

Sources read:

- pstack `skills/poteto-mode/playbooks/bug-fix.md` (refs checkout)
- the old bee class playbook: `git show 25bccf3b6^:skills/bee-planning/playbooks/bugfix.md`

Files under check:

- `skills/bee-planning/playbooks/bugfix/bug-fix.md` (the situation)
- `skills/bee-planning/playbooks/bugfix.md` (the class index)
- `docs/history/pstack-playbook-port/port-notes.md`

## Verdict

The port is faithful. Every pstack step is either in the situation file or
swapped or dropped with a port note. Every enforced line from the old bee
playbook is still there. The index line parses under the plan's grammar.
There are two problems:

1. **The cell's verify command is red, and ppb-3 did not cause it.**
   `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/`
   matches `skills/bee-planning/references/design-sketch.md:11`
   (`Adapted from pstack (cursor/plugins, pstack/skills/architect).`). That
   line was added on 2026-09-11 in `2a4856520`, before this feature. It is a
   provenance credit, not a tool step. The same rg limited to
   `skills/bee-planning/playbooks/` finds nothing. Because of this line,
   port-notes.md's claim that "no file under `skills/` names one" is false
   word for word, though no playbook names a pstack tool.
2. **A stale line pointer was carried over.** Step 2 cites
   `bee-swarming/references/worker-details.md:33-35`, but the quoted
   sentence ("red-before-green is craft, …") sits on line 40, both at HEAD
   and at the feature base `fe5342caf`. The old bee playbook already had this
   drift, and the merge kept it.

## pstack steps — present or missing

| pstack source line | In `bugfix/bug-fix.md` | Status |
|---|---|---|
| "You own this task. Plan, review, verify." | opening line | present |
| Delegate investigation and the fix to subagents, stay in the lead | opening paragraph, via `bee dispatch prepare` | present (swapped, noted) |
| Be scientific: every shipped line traces to runtime evidence | paragraph 2 | present |
| Belt-and-suspenders is a hypothesis, not a fix; does not ship | paragraph 2 | present |
| When evidence refutes a hypothesis, revert what it motivated | paragraph 2 | present |
| Smallest change the evidence justifies ships | paragraph 2 | present |
| Same discipline for Perf, evidence is the trace | paragraph 2, `playbooks/perf.md` (file exists) | present (swapped, noted) |
| 1. Reproduce yourself on the matching surface via the control skill | step 1, the `verify-app` skill or the real interface | present (swapped, noted) |
| 1. Don't hand the repro to the user | step 1 | present |
| 1. A debug protocol that says ask the user does not override; you drive the instrumented runtime | step 1 | present |
| 1. Ask the user only with a stated, specific reason, after driving as far as it goes | step 1 | present |
| 1. Won't reproduce: synthesize the trigger, tighten conditions, instrument | step 1 | present |
| 1. A bug you can't reproduce, you can't prove fixed | step 1 | present |
| 2. Binary-search the cause | step 3 | present |
| 2. Form candidate hypotheses, rule them out until one survives | step 3 | present |
| 2. Seed with `how` and the `why` skill | step 3, `bee-how` / `bee-why` (both exist under `skills/`) | present (swapped, noted) |
| 2. Each pass, the split that cuts the most, runtime evidence | step 3 | present |
| 2. Add instrumentation or logging when state is unclear | step 3 | present |
| 2. Don't guess | step 3 | present |
| 2. Drive a long hunt with Cursor's `/loop` | — | missing (dropped, noted) |
| 2. Confirm the surviving mechanism before the architect/interrogate fan-out; unanimously wrong warning | step 3, "before you plan the fix" | present (fan-out dropped, noted) |
| 3. Plan the fix | step 4 | present |
| 3. If it crosses a function boundary, `architect` first | step 4, "write down the contract it keeps" | present in part (architect dropped, noted; contract rule kept) |
| 3. Delegate implementation with the configured bug-fix model, specific scope | step 4, `bee dispatch prepare --kind cell` | present (swapped, noted) |
| 3. Review the diff | step 4 | present |
| 4. Verify on the same surface; original repro passes | step 5 | present |
| 4. Inconclusive or wrong surface is not a pass; flag it | step 5 | present |
| 4. Unit tests show branch behavior, not bug absence | step 5 | present |
| 5. Stage commits so the failing repro lands before the fix | step 6 | present |
| 5. `tdd` skill for the failing-test-first cadence | — | missing (dropped, noted) |
| 5. Skip when the test is expensive, integration-heavy or unclear | step 6 | present |
| 5. `sequence-verifiable-units` principle skill | step 2 home pointer + step 6 | present (swapped, noted) |
| 6. Run Opening a PR | — | missing (dropped, noted) |
| Investigation fans out `how` + `why` as parallel subagents | step 3, "two parallel gathers" | present |
| Reply: what was broken, root cause, fix, how verified | Reply line | present |
| Reply: paste the failing-then-passing output verbatim | Proof line | present (moved to Proof line, noted) |

Every source step is accounted for. The four missing ones (`/loop`, the
architect/interrogate fan-out, the `tdd` reference, Opening a PR) each have a
dropped-step entry in port-notes.md.

## Old bee `bugfix.md` enforced lines — present or missing

| Old line | Where now | Status |
|---|---|---|
| 1. Reproduce on the real interface, because the user met it there, not in a unit | step 1 | present (merged with pstack step 1) |
| 2. Watch the reproduction FAIL; the home pointer to worker-details.md, not copied | step 2, word for word | present (stale line range, see Verdict 2) |
| 3. Find the mechanism, trace the bad value, `bee-principle-crash-site-versus-fault-site` | step 3 | present (skill exists) |
| 4. Fix the mechanism; a guard at the symptom leaves the cause free | step 4 | present |
| 5. Re-run the same repro on the same interface; inconclusive or other surface is not a pass | step 5 ("surface" for "interface") | present |
| List the candidate causes | step 3 ("Form the candidate hypotheses") | present (moved) |
| Each pass the split that removes most candidates; `.bee/expertise/tests.md` ("Instrument before guessing") | step 3 | present (anchor exists at tests.md:489) |
| Event-order symptom: `statechart` model of `feature.md` step 6 | "When the cause is not known" | present (feature.md step 6 is the statechart step) |
| Formal verify skill rerun (`feature.md` step 7) | "When the cause is not known" | present (feature.md step 7 is the model step) |
| Revert every edit a refuted hypothesis motivated | "When the cause is not known" | present |
| A one-line fix with known cause keeps the short path | "When the cause is not known" | present |
| Propose the cheap always-on signal after a deep dig | "When the cause is not known" | present |
| Proof line: the repro's output, red then green, verbatim | Proof line, word for word | present |
| Proof floor line (plan claim 12) | last line of the situation file and of the index, word for word | present |

No enforced line was lost.

## pstack-only tool names under `skills/`

- `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/` →
  exit 0, with one hit at `skills/bee-planning/references/design-sketch.md:11`.
  The hit predates the feature (see Verdict 1).
- `rg -n -i 'architect|interrogate|/loop|\btdd\b|sequence-verifiable|gpt-5|control skill|opening a pr' skills/bee-planning/playbooks/bugfix*`
  → exit 1. The new files name none of the swapped or dropped tools.
- A wider sweep (`/loop|arena|interrogate|poteto|pstack|graphite|bugbot`
  over `skills/`) finds only provenance credits and CREATION-LOG history in
  `bee-unslop`, `bee-teach`, `bee-why`, `bee-how`, `bee-technical-writing`
  and `bee-reviewing`. None of them is a step that calls a pstack tool.

## port-notes.md coverage of swaps and drops

All swaps are named: control skill → `verify-app`, how/why → `bee-how`/`bee-why`
gathers, subagent delegation and the named bug-fix model →
`bee dispatch prepare` and `--kind cell`, `sequence-verifiable-units` → the
red-before-green pointer plus step 6, and Perf → `playbooks/perf.md`.

All drops are named: `/loop`, the architect/interrogate fan-out, `tdd`,
Opening a PR, and the Reply's verbatim paste (which moved to the Proof line).

port-notes.md does not name these two things:

- Pstack's "Investigation fans out `how` + `why`" line was folded into step 3.
  It was merged, not dropped, so no note is needed.
- The swap of `sequence-verifiable-units` points to the worker-details
  pointer. bee already ships `skills/bee-principle-red-before-green`, which
  would be a closer one-to-one counterpart. This is an observation, not a
  defect.

## Index line against the plan grammar

The plan's ppb-1 action defines a `## Situations` heading followed by
`` - `<name>` — <triggers, English and Vietnamese> — delivers: <deliverable> ``,
where `<name>` matches `[a-z0-9-]+` and names `playbooks/<class>/<name>.md`.

- `bugfix.md` has a `## Situations` heading and one line:
  `` - `bug-fix` — bug, broken, … sửa lỗi, không chạy, chạy sai — delivers: the root cause confirmed … on the same surface ``.
  It matches `^- `([a-z0-9-]+)` — .+ — delivers: .+$`.
- The name `bug-fix` resolves to `skills/bee-planning/playbooks/bugfix/bug-fix.md`, which exists.
- The triggers include both English and Vietnamese.
- No further `## ` heading follows. The Proof floor line is not a `` - ` ``
  row, so `situations.rs::parse_line` skips it (its `strip_prefix` would fail).
- The rendered trees match the source:
  `diff -r skills/bee-planning/playbooks .claude-plugin/skills/bee-planning/playbooks`
  and the `.codex-plugin` twin both report no difference.

## Suggested follow-ups (not done here; this cell edits only this report)

- Either narrow the checker's rg to `skills/bee-planning/playbooks/`, or
  rewrite the design-sketch.md:11 credit so it does not say "cursor". Then
  fix the port-notes sentence so it matches what is true.
- Point step 2 at `worker-details.md:38-41` (or quote the sentence without a
  line range) in both the situation file and anywhere else that cites 33-35.
