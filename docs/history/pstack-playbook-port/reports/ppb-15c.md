# ppb-15c — bee-interrogate checked against its pstack source

Scope: what ppb-15 wrote in commit `261af76a7`. That is `skills/bee-interrogate/`
(`SKILL.md`, four reference files, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-interrogate.md`. Each is
checked against the pstack skill `pstack/skills/interrogate/` (`SKILL.md` and
`references/code-quality-review.md`, `lead-judgment.md`, `reviewer-prompt.md`,
`rubric.md`). The reference files were compared with `diff`, line by line.

Verdict: no source step is missing. All five steps, the verdict shape and every
rule in the four references are present, in bee's terms where a tool was swapped.
Every removed text (the Task tool spawn, `subagent_type`, `readonly`, the Cursor
rules-file list, the four model slugs, the slug-rejection fallback, the alias
note, `disable-model-invocation`) has a port-notes entry. The pstack-only tool
fence is clean. No model and no subagent type is picked by hand. The description
is 276 characters and names what the skill is not for. `rubric.md` is unchanged
and `LICENSE.pstack` is byte-identical to the pstack `LICENSE`. There are three
findings. None blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 (`063a21e4`) counts as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule (source line) | New home | Status |
|---|---|---|
| Frontmatter `name: interrogate` (2) | `name: bee-interrogate` | present |
| Frontmatter description (3) | line 4, rewritten with a "Not for" clause | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Intro: one reviewer per configured model, same prompt and rubric (9) | line 18, "several reviewers", same prompt and rubric | present (swap noted, bullets 1-2) |
| Intro: signal from model diversity, not personas (9) | line 18, "independence, not assigned personas" | present (swap noted, bullet 3) |
| Intro: cross-model agreement high signal, lone finding lower (9) | line 18, "agreement across reviewers on different models" | present |
| Deliverable is a synthesized verdict; do not auto-apply (11) | line 20, plus "advice before a gate" | present (addition noted) |
| Step 1: user-named files or diff (17) | line 26 | present |
| Step 1: feature branch, `git diff main...HEAD` (18) | line 27 | present |
| Step 1: recent work, gather files (19) | line 28 | present |
| Step 1: package diff plus context files (21) | line 30 | present |
| Step 2: state intent before spawning; four sources (25-30) | lines 34-39, "pull request description" | present (wording swap noted) |
| Step 2: one paragraph; challenge execution not intent; ask when unsure (32) | line 41 | present |
| Step 3: launch all in one message with the Task tool (36) | line 45, one `bee dispatch prepare --kind reviewer` per reviewer, same turn, background | present (swap noted, bullet 1) |
| Step 3: `interrogate reviewers` list from the Cursor rules file (36) | line 45, review role of the bee team table | present (swap noted, bullet 2) |
| Step 3: extend or shrink labels A-D to entry count (36) | line 47, three by default, two for a small change, generic labels | present (swap noted, bullet 2) |
| Step 3: four-model default table (38-43) | removed | dropped (noted: bullet 2) |
| Step 3: `subagent_type: generalPurpose` (46) | removed; payload used as returned | dropped (noted: bullet 1) |
| Step 3: `model:` configured entry or table default (47) | line 45, "never pick a model by hand" | dropped (noted: bullet 1) |
| Step 3: `readonly: true` (48) | removed from SKILL.md; reviewer kind is read-only | dropped (noted: bullet 1) |
| Step 3: slug-rejection fallback and separate PR (50) | removed | dropped (noted: Dropped steps) |
| Step 3: `inherit-parent` / `auto` alias note (50) | removed | dropped (noted: Dropped steps) |
| Step 3: fill reviewer template with intent, diff, rubric, lens (52-56) | lines 49-53 | present |
| Step 3: same filled template to all, every model applies lens (58) | line 55, plus the no-verdict brief rule | present (addition noted, bullet 6) |
| Step 3: structured findings per the template (60) | line 59, with P1-P3 severity | present (swap noted, bullet 4) |
| Step 4: parse, consensus, lone, dedupe, disagreements (66-70) | lines 71-75, "reviewers" for "models" | present |
| Step 5: lead is a pragmatic senior engineer, not aggregator (74) | line 79 | present |
| Step 5: read lead-judgment; use full context aggressively (76) | line 81, plus reproduce-or-trace rule | present (addition noted, bullet 7) |
| Step 5: four buckets Act on / Consider / Noted / Dismissed (80-83) | lines 85-88, "real merge", P1 under Act On, Dismissed names what was run | present (swaps noted, bullets 4, 7, wording) |
| Step 5: per finding, who raised it, category, rationale (85-88) | lines 90-93, plus severity | present (swap noted, bullet 4) |
| Output: Intent, Reviewers, Act On, Consider, Noted, Dismissed, Agreement Map (94-113) | lines 99-118 | present |
| Output: Reviewers bullet names the model (98) | line 103, "model name the payload named, or not named" | present (swap noted, bullet 3) |

Additions with no source counterpart: the `metadata` block with the `bee-cli`
dependency (noted, frontmatter paragraph), the same-model warning on line 57
(noted, bullet 3), the P1-P3 scale on lines 59-63 (noted, bullet 4), the
re-dispatch rule on line 65 (noted, bullet 6, but see finding 2) and the Handoff
section (noted).

## `references/rubric.md`

`diff` prints nothing. Correctness, Root Causes vs. Symptoms, Structural
Integrity, Verification, Complexity Budget and Security are all present,
unchanged.

## `references/code-quality-review.md`

| Source section | Status |
|---|---|
| Core Prompt (7) | present, unchanged |
| Dimensions 0, 2-7 (21, 25-35) | present, unchanged |
| Dimension 1, file past 1k lines (23) | present, "a PR" became "a change" (noted, wording) |
| Output Expectations (37) | present, unchanged |
| Approval Bar (41-43) | present, "the PR" became "the change", plus one P2 sentence (noted, bullet 5) |
| Review Tone (45) | present, unchanged |

## `references/lead-judgment.md`

| Source section | Status |
|---|---|
| Opening: "configured reviewers" (3) | present, "reviewers" |
| Why This Step Matters, incl. "next PR in the stack" (5-14) | present, "next planned cell or change" (noted, wording) |
| Nitpick Gravity (18) | present, unchanged |
| Hypothetical vs. Actual (22) | present, plus the reproduce-or-trace paragraph (noted, bullet 7) |
| Premature Abstraction Warnings (26) | present, unchanged |
| "I Would Have Done It Differently" (30) | present, unchanged |
| Missing Context Signals (34) | present, unchanged |
| When Reviewers Are Right, consensus across models (43-52) | present, "different models" condition added (noted, bullet 3); "single model" became "single reviewer" |
| Verdict Calibration (54+) | present, unchanged |
| new: A P1 Blocks | addition (noted, bullet 4) |

## `references/reviewer-prompt.md`

| Source section | Status |
|---|---|
| Template intro (3) | present, plus byte-identical brief rule (noted, bullet 6) |
| Role paragraph (7) | present, plus "You are read-only: never edit a file" (not noted, finding 1) |
| Intent, Code Under Review, Review Rubric, Code Quality Lens slots (9-27) | present, unchanged |
| Instructions (29-33) | present, unchanged |
| Severity `critical / warning / nit` (35-38) | present as `P1 / P2 / P3`, source meanings folded in (noted, bullet 4) |
| Finding (39) | present, unchanged |
| Evidence (40) | present, plus command/output/file:line rule (noted, bullet 4) |
| Suggestion (41) | present, unchanged |
| What Makes a Good Finding, What to Avoid (43-56) | present, unchanged |
| Output and Findings format (57-72) | present, `[Severity]` became `[P1|P2|P3]` |

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-interrogate/` prints
nothing and exits 1. A wider search under `skills/bee-interrogate/` for
`subagent_type`, `generalPurpose`, `Task tool`, `model:`, `readonly`,
`pstack-models` and the model families `claude-`, `gpt-`, `grok`, `opus`,
`sonnet`, `haiku`, `fable` also prints nothing. The only pstack names left are in
the port notes, as the notes say.

## Hand-picked model or subagent type

None. Step 3 sends every reviewer through `bee dispatch prepare --kind reviewer`
with no `--role`, uses each payload as returned, and says "Never pick a model or
a subagent type by hand". The `--runtime claude` flag picks a runtime, not a
model or subagent type; it matches how `bee-architect` and `bee-arena` dispatch.

## Description

276 characters (counted from the folded `description:` value). It says what the
skill does, when to use it (`/bee-interrogate`, interrogate, stress-test, tear
apart) and when not to: "Not for the review gate (bee-reviewing)". It fits
the 300-character limit.

## Findings

1. **P3, not noted.** `reviewer-prompt.md` line 7 adds "You are read-only: never
   edit a file." The port notes say the `readonly` flag is gone because
   read-only is the reviewer kind's own property, and do not mention this
   sentence. It is harmless belt-and-braces, but D2 asks every change to be
   named. One line in bullet 1 of the port notes would close it.
2. **P3, partly noted.** SKILL.md line 65 re-dispatches a return that "cites a
   path that does not resolve" as well as one that breaks the schema. The port
   notes (bullet 6) name only the schema case. The path case comes from the same
   `bee-reviewing` rule, so this is a wording gap in the notes, not a new method.
3. **P3, observation.** The port notes say "Lead Judgment says agreement on one
   model is weaker corroboration". That sentence lives in
   `references/lead-judgment.md` (When Reviewers Are Right), not in the SKILL.md
   Step 5 section. The claim is true; only the location is loose.

## Verify

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-interrogate/ ; test $? -eq 1`
— no output, `test` exits 0.
