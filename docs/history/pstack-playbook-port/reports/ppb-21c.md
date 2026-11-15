# ppb-21c — bee-blast-radius checked against its pstack source

Scope: what ppb-21 wrote in commit `3cf7e9719`: `skills/bee-blast-radius/`
(`SKILL.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-blast-radius.md`. They were
checked against the pstack skill `pstack/skills/blast-radius/SKILL.md` at pstack
commit `b9ddc83`. The source has no reference files. `SKILL.md` was compared line
by line, and the licence with `diff`. The five rendered copies (`.claude-plugin`,
`.codex-plugin`, `.claude`, `.agents`, `.opencode`) were compared with `diff -rq`,
and all of them match `skills/bee-blast-radius/`.

Verdict: no source step is missing. Every source step and rule is present: the
opening promise, the "grep is not the job" line, "Don't trust your own writeup",
the five-rung certainty ladder, all six steps, and the five-part hand-back. Each
swapped item has a port-notes entry. Nothing in the source was dropped. The only
removal is the frontmatter key `disable-model-invocation`, and the port notes name
it. `LICENSE.pstack` is identical to the pstack `LICENSE`. The pstack-only tool
check is clean. No model and no subagent type is picked by hand. The description
is 294 characters and says when not to use the skill. There are no findings.

Labels: **present** means the step is in the new file. **swapped (noted)** means
it is present with a bee tool in place of the pstack tool, and the port notes name
the swap. **dropped (noted)** means it is gone and the port notes name it.
**dropped (not noted)** means it is gone and the notes say nothing; D2
(`063a21e4`) counts that as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule (source line) | New home (bee-blast-radius line) | Status |
|---|---|---|
| Frontmatter `name: blast-radius` (2) | `name: bee-blast-radius` (2) | present |
| Frontmatter description (3) | 4, rewritten: what it does, when, when not | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Opening promise and trigger phrases (9) | 18 | present |
| Companion to `how` and `why` (11) | 20, `bee-how` and `bee-why` | swapped (noted: "Tool swaps", first bullet) |
| Listing callers is not the job; the job is what grep won't show (13) | 22 | present |
| Don't trust your own writeup; prove the one or two facts by running code (17) | 26, plus `bee-principle-verify-before-reporting` | present (noted: "Tool swaps", fifth bullet) |
| Certainty ladder: get each fact as far down as is cheap, say where it stopped (21) | 30 | present |
| Rung 1, you said so (23) | 32 | present |
| Rung 2, you pointed at the line (24) | 33 | present |
| Rung 3, you showed the bad case can't happen (25) | 34 | present |
| Rung 4, you ran it (26) | 35 | present |
| Rung 5, reproduced in the running app (27) | 36 | present |
| A fact short of rung 4 is said out loud; rung 4 is usually one small script (29) | 38 | present |
| Step 1: read the change, including what the diff doesn't spell out (33) | 42 | present |
| Step 1: use `why` step 2 to pull the PR and commits (33) | 42, `bee-why`'s Provenance sweep (`bee-researching/references/trace-and-provenance.md`) | swapped (noted: "Tool swaps", second bullet) |
| Step 2: find the one fact it's safe because of; spend the time there (34) | 43 | present |
| Step 3: read the library source, its pinned version and local patch (35) | 44 | present |
| Step 3: work out when things run; "Solid versus React" (35) | 44, "one runtime versus another" | swapped (noted: "Tool swaps", last bullet) |
| Step 3: follow what a symbol search misses (JSON, DB column, wire format, another language, feature flag, three hops downstream) (35) | 44 | present |
| Step 4: real chance and real cost per risk; confirmed and cleared kept apart (36) | 45 | present |
| Step 4: same rules as `why`; real `file:line`, an empty search is an answer, never invent a caller or API (36) | 45, `bee-why` | swapped (noted: "Tool swaps", first bullet) |
| Step 5: prove the one fact with a script that runs real code; paste the result; else unproven, don't round up (37) | 46 | present |
| Step 6: a big or wide change runs as an `arena`; several models, merge the answers (38) | 47, `bee-arena` with runners from `bee dispatch prepare --runtime claude --kind advisor --role lane-1..3 --json` | swapped (noted: "Tool swaps", third bullet) |
| Hand-back: what it does (42) | 51 | present |
| Hand-back: the one fact, its rung, the proof or unproven (43) | 52 | present |
| Hand-back: risks, each with how it breaks, `file:line`, likelihood, cost, check (44) | 53 | present |
| Hand-back: cleared (45) | 54 | present |
| Hand-back: before you merge, the cheapest test or repro (46) | 55 | present |
| Write through `unslop`, cite real code, strip private context (48) | 57, `bee-unslop` | swapped (noted: "Tool swaps", fourth bullet) |
| Reply: the writeup, the safety fact proven or unproven (50) | 59-61, a `Reply` section with a read-only line | present (noted: "Added" third bullet) |

Additions with no source counterpart, each named in the port notes ("Added, not in
the source"): the `metadata` block with the `bee-cli` dependency (frontmatter
paragraph); the knowledge-bundle search before the code hunt in step 1 (line 42);
the vendored-files-and-persisted-records sentence in step 3 (line 44); the
read-only line in Reply (line 61); the Handoff section (lines 63-65).

## Cross-references checked

Each name the new file cites exists in this worktree: `skills/bee-how`,
`skills/bee-why`, `skills/bee-arena`, `skills/bee-unslop`, `skills/bee-reviewing`,
`skills/bee-hive`, `skills/bee-principle-verify-before-reporting`, and
`skills/bee-researching/references/trace-and-provenance.md` with its
`## Provenance sweep` heading (line 67). The lane-runner command in step 6 matches
`skills/bee-arena/SKILL.md:37` word for word in its flags.

## pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-blast-radius/` finds
nothing (exit 1). The source names none of these either. Its pstack skill names
(`how`, `why`, `arena`, `unslop`) are all swapped for bee skills that exist.

## Hand-picked model or subagent type

None. `rg -i 'model|subagent_type|opus|sonnet|haiku|fable|gpt'` over the new
`SKILL.md` matches only line 47. That line forbids picking a model or subagent type
by hand and routes each runner through `bee dispatch prepare`.

## Description

294 characters, which fits in 300. It ends with "Not for how code runs (bee-how)
or the review gate (bee-reviewing)", so it says when not to use the skill.

## Licence

`diff pstack/LICENSE skills/bee-blast-radius/LICENSE.pstack` prints nothing, so the
two files are identical.

## Verify

```
$ rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-blast-radius/ ; test $? -eq 1; echo $?
0
```
