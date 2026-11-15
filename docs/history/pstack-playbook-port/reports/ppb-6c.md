# ppb-6c — research port checked against its sources

Scope: what ppb-6 wrote in commit `0f4a55e06`. That is the class index
`skills/bee-planning/playbooks/research.md`, the situation files
`research/investigation.md`, `research/runtime-forensics.md`,
`research/trace-forensics.md` and `research/eval.md`, and the port notes
`docs/history/pstack-playbook-port/port-notes/research.md`. The files are
checked against the pstack sources `pstack/skills/poteto-mode/playbooks/`
`investigation.md`, `runtime-forensics.md`, `trace-forensics.md` and `eval.md`
(commit `b9ddc83`), and against the old bee `research.md` from
`git show 0f4a55e06^:skills/bee-planning/playbooks/research.md`.

Verdict: every source step is present or dropped with a port-notes entry. No
method step is missing. Every old enforced line is present, and the Proof
floor line is byte-identical in the index. The pstack-only tool fence is
clean. Every skill the files name exists in `skills/`, except `verify-app`,
which is named on the condition the cell action prescribes. All four index
lines match the contract grammar. There are three findings. One is a real
mismatch: investigation names `bee-how` modes that `bee-how` does not have.
The other two are small. None blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 counts as missing. **missing** means a method step is gone with no reason.

## Old enforced lines (old `research.md`)

| Old line | New home | Status |
|---|---|---|
| 1 Read-only: the outcome is an account, never a diff | `investigation.md` step 1 | present |
| 2 Trace the runtime path, not just the file list | `investigation.md` step 3, verbatim reason | present |
| 3 Name every source searched that came up EMPTY | `investigation.md` step 4, verbatim | present |
| 4 End with anchors a reader can open (`path:line` or the command) | `investigation.md` step 6 | present |
| 5 A choice between alternatives ends in a recommendation and a tradeoffs table | `investigation.md` step 5 | present |
| 5 An answer that leads to a change re-routes to `bugfix` or `feature` | `investigation.md`, paragraph after step 7 | present |
| ONE home: `bee-researching/references/trace-and-provenance.md`, § "Trace" and § "Provenance sweep" | `investigation.md` step 3 | present; both headings exist in that file (`:7`, `:67`) |
| Live runtime: capture a real profile or trace | `runtime-forensics.md` step 1 | present |
| Live runtime: reduce through `bee dispatch prepare --kind gather` | `runtime-forensics.md` step 2 | present |
| Live runtime: confirm with one instrumented run, map to `path:line` | `runtime-forensics.md` steps 3 and 4 | present |
| Live runtime: without a before-and-after pair, label the cause a hypothesis | `runtime-forensics.md` step 3, verbatim | present |
| Investigation route paragraph (D3, decision `f1ffa7bd`, backlog `p-69bee217`) | `investigation.md` | present, verbatim |
| Proof line: the account's anchors, each one opened or run | `investigation.md` | present, verbatim |
| Proof floor line | `research.md` line 12 | present; byte-identical (checked with `diff` of the `^Proof floor` line, old against new). See Finding 3 |

The research class has no routed-principle, statechart or formal-verify lines
in the old text, so D3 had nothing more to carry.

## investigation (pstack `investigation.md`)

| Source | Status |
|---|---|
| Lead line "You own the answer. Plan, route, write." | present |
| Trigger examples: how does X work, why was Y built, are we sure about Z, should we do X or Y; output is a cited explanation or recommendation, not a code change | present |
| 1 Route through `how` (Explain mode for narrow, Critique mode for "are we sure?") | present: swapped to `bee-how` as a gather (noted); see Finding 1 |
| 1 Motivation questions also route through `why` | present: `bee-why` (noted) |
| 2 Throughput checkpoint one line | dropped (noted): no bee counterpart |
| 3 `how`-shaped output (Overview, Key Concepts, How It Works, Where Things Live, Gotchas) | present (step 5); the five sections match the Trace answer shape in `trace-and-provenance.md:47` |
| 3 Or a recommendation with a tradeoffs table for a decision | present (step 5) |
| 4 Apply `unslop` to the reply | present: `bee-unslop` (step 7, noted) |
| No PR, no babysit | dropped (noted): read-only step 1 says it |
| No `architect` unless the investigation precedes a code change | dropped (noted, D5 slice 3) |
| If it precedes a code change, hand back to the user and re-route to Bug fix or Feature | present: re-route to `bugfix` or `feature` (noted) |
| Reply: the investigation output; real judgment with reasons for "are we sure?" | present |
| Push back if the premise is wrong (see Autonomy) | present; the Autonomy pointer is dropped (noted) and a reason is given |

## runtime-forensics (pstack `runtime-forensics.md`)

| Source | Status |
|---|---|
| Lead line "You own the diagnosis. Instrument the live process, don't theorize from source." | present, with a reason added |
| Triggers: leaking, spinning, slow at runtime, heap snapshots, idle-but-busy, intermittent glitches; deliverable is a cited diagnosis, not a fix | present |
| 1 Capture the live signal via the control skill | present: the project's `verify-app` or the real interface (noted) |
| 1 CPU profile for spinning, heap snapshot for a leak, CDP trace for a visual glitch | present; "CDP trace" became "a trace". See Finding 2 |
| 1 A real artifact, not a guess | present |
| 2 Reduce to the smoking gun: hot-path function, retainer chain to a GC root, loop firing without input | present |
| 2 Parse large artifacts in a subagent (`guard-the-context-window`); keep the reduced finding in the main thread | present: `bee dispatch prepare --kind gather` (noted); the principle citation is dropped (noted), its reason kept |
| 3 Prove the mechanism before believing it: inject instrumentation via CDP eval, or hotfix live code without reloading | present: "into the running process" (noted) |
| 3 A plausible but unconfirmed cause can be wrong while the real one sits one layer over | present, verbatim |
| 4 Map back to source: file, symbol, the line that allocates or schedules | present |
| 5 Throughput checkpoint one line | dropped (noted) |
| Reply: signal, reduced finding, how the mechanism was proved, source location, artifact paths | present |
| No fix unless asked; hand back to Bug fix or Perf once the cause is known | present: re-route to `bugfix` or `perf` (noted) |

## trace-forensics (pstack `trace-forensics.md`)

| Source | Status |
|---|---|
| Lead line "You own the diagnosis from the artifact. Load it, shape it, narrow to the cause, attribute to source." | present |
| Triggers: `.cpuprofile`, `Trace-*.json.gz`, `Spindump.txt`, `.heapsnapshot` with slow, unresponsive, leaking or crashing | present |
| Distinct from Runtime forensics; the artifact is a fixed dataset, read it, don't re-run it | present: pointer to `research/runtime-forensics.md`, which exists (noted) |
| Keep tooling generic: DevTools or trace parser, text editor, heap tooling | present |
| 1 Identify the format and load with the right tool | present |
| 1 Parse large artifacts in a subagent (`principle-guard-the-context-window`) | present: gather dispatch (noted); citation dropped (noted), reason kept |
| 2 Transform into a queryable form: sqlite, one row per sample, frame or node; reach the queryable shape before reading | present |
| 3 Narrow: frames with the most time, call tree to the hot path; retainer chain for a leak; stuck or blocked thread and wait reason for a spindump | present |
| 4 Attribute to source via the artifact's own symbols; an unmapped frame is not yet a diagnosis | present |
| 5 Confirm against a paired capture; without one, mark the strongest hypothesis | present |
| 6 Hand back a cited diagnosis, no fix unless asked; route to Bug fix or Perf issue | present: `bugfix` or `perf` (noted) |
| 6 Throughput checkpoint one line | dropped (noted) |
| Reply: artifact and format, reduced finding, source location, artifact paths, whether a paired capture confirmed it | present, verbatim |

## eval (pstack `eval.md`)

| Source | Status |
|---|---|
| Lead line "You own the experiment design. Plan, blind, run, synthesize." | present |
| Purpose (test a change before promoting it) and the observer effect, so candidates run blind | present |
| Blinding: no `eval`, `test`, `judge`, `experiment`, `rubric`, `score`, `compare`, `benchmark`, `candidate`, `arena` | present; the word `arena` is kept as a cue word, not a skill call (noted) |
| Blinding: organic prompt, goal not meta, the todo-cli example | present |
| Blinding: no chain-eliciting cues; grade from code shape, not self-report | present |
| Blinding: sanitize directory and slug names | present |
| Blinding: don't tell a candidate others exist | present |
| Blinding: judge sees sanitized labels, never model names | present |
| Blinding: one judge scores both sets in one pass on one scale | present |
| 1 Frame: variant, success, a 3 to 6 criterion rubric held back from candidates | present |
| 2 Sanitized per-candidate environments with the variant and planted context | present |
| 3 One organic prompt with no leakage | present |
| 4 Spawn N parallel candidates per `arena` Phase B, own dir, same prompt | present: `bee dispatch prepare` (noted); the `arena` reference is dropped (noted, slice 3) |
| 4 "on different models" | dropped (noted): the door fixes the model per role |
| 5 One blinded judge per `arena` Phase C | present: `bee dispatch prepare --kind reviewer` (noted); `reviewer` is a real kind (`DISPATCH_KINDS` in `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:152`) |
| 5 Judge on a different model family | partly kept (noted): the reply must say when the door returns the same model |
| 6 Verify the chain from transcripts; Cursor transcript path and the no-glob-across-projects warning | present: "own transcript under the active workspace and no other", privacy reason kept (noted) |
| 6 Files actually opened; citing is not reading, reading is not applying; grade from files read and code shape | present |
| 7 Read every output yourself, compare to the verdict; disagreement means bias or an ambiguous rubric; synthesize | present |
| Reply: variant, rubric, per-candidate notes, verdict, synthesis, promote recommendation | present |

## Tool fence and skill names

- `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/`
  finds nothing (exit 1).
- A wider search of `research.md` and `research/` for arena, architect,
  interrogate, swarm, tdd, figure-it-out, show-me-your-work, `/loop`, `gt`,
  CDP, subagent, guard-the-context-window, laziness, prove-it-works,
  throughput, `agent-transcripts` and `.cursor` finds only `eval.md:12`, the
  word `arena` in the blinding list. The port notes keep it on purpose, as a
  cue word a candidate must not see. I agree: it names no tool.
- Skills named: `bee-how`, `bee-why`, `bee-unslop` and `bee-researching` (with
  `references/trace-and-provenance.md`) all exist under `skills/`.
  `verify-app` is not under `skills/`. It lives in `.claude/skills/verify-app`
  and `.agents/skills/verify-app`. `runtime-forensics.md` names it only as "the
  project's `verify-app` skill when it has one, otherwise the real interface".
  That is the swap the cell action prescribes, and the ppb-5c and ppb-12c
  reports rule the same way.
- Dispatch kinds named (`gather`, `reviewer`) are both in `DISPATCH_KINDS`.

## Index grammar (contract `913115b3`)

All four lines under `## Situations` in `research.md` match
`` ^- `[a-z0-9-]+` — .+ — delivers: .+$ ``. Each named file exists:

- `` `investigation` `` exists as `research/investigation.md`.
- `` `runtime-forensics` `` exists as `research/runtime-forensics.md`.
- `` `trace-forensics` `` exists as `research/trace-forensics.md`.
- `` `eval` `` exists as `research/eval.md`.

Each line carries English and Vietnamese triggers and a deliverable. No `## `
heading follows the section, so it runs to the end of the file. The rest of
the section is blank lines and the Proof floor line, which does not start
with `- `, the same shape as `bugfix.md` and `perf.md`.

## Findings

1. **Investigation names `bee-how` modes that do not exist. Verified.**
   `investigation.md:14-16` says narrow questions take `bee-how`'s "Explain
   mode" and "are we sure?" takes its "Critique mode". `skills/bee-how/SKILL.md`
   has no modes. It is a door that runs the Trace procedure, and the only
   other mode it names is `mode:headless`. `rg -i 'explain mode|critique mode'
   skills/` finds only this playbook line. The modes belong to the pstack
   `how` skill. The port swapped the skill name and kept the modes, so a
   worker told to pick Critique mode finds nothing to pick, and the "are we
   sure?" challenge has no home. The port notes record the swap ("Explain
   mode, Critique mode") but not that the modes have no counterpart. Suggested
   fix: drop the mode names and say that an "are we sure?" question asks the
   lead to challenge the account, not tour it. Or name the modes as dropped
   in the port notes, to be re-added if `bee-how` gains them.
2. **One swap is not in the port notes. Verified, cosmetic.**
   pstack runtime-forensics step 1 says "a CDP trace for a visual glitch".
   The port writes "a trace" (`runtime-forensics.md:14`). The notes record
   only the step 3 "via CDP eval" swap. The reason given there covers this
   one too, but the note should name both.
3. **The Proof floor line is in the index only, not in any research
   situation file. Verified, low severity.** The ppb-6 truth says "every
   enforced line of the old class playbook survives in a situation file". The
   Proof floor line survives byte-identical in `research.md`, which is what
   the cell action asked for, but no file under `research/` carries it. The
   bugfix, perf, refactor, docs, content and release situation files each
   carry it. The spike and feature classes are index-only like research
   (`rg -c '^Proof floor' skills/bee-planning/playbooks/`), so the set is
   mixed. The cap reads its own table, so nothing is weaker in practice. If
   the leader wants one shape, add the line to each research situation file.

Checked and not filed: the old live-runtime step "confirm the reduction with
one instrumented run" looked dropped, but runtime-forensics step 3 carries it
as "inject instrumentation into the running process". Eval step 4 dispatches
candidates through `bee dispatch prepare` without naming a `--kind`. Candidates
are not cells, and I did not find which kind fits. This is a guess, so I am
not filing it. All cross-references (`research/runtime-forensics.md`, the
Trace and Provenance sweep headings) resolve.
