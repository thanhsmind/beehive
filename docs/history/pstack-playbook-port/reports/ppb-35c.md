# ppb-35c: check of the pstack technical-writing merge in ppb-35

Checked against commit `6ae9e959e` (ppb-35). The source is the pstack skill
`technical-writing` at `/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/technical-writing/SKILL.md`,
the only file in that skill. The bee side is `skills/bee-technical-writing/SKILL.md`.
No file other than this report was edited.

## Verdict

The merge holds. Every pstack technical-writing rule is present in
bee-technical-writing, or left out with a named bee home or a named bee
difference. None is missing. The port notes in `port-notes/merge-ppb-35.md` list
the same split.

- **pstack-only tool names:** none in `SKILL.md`. A scan of
  `skills/bee-technical-writing/` for `cursor`, `graphite`, `bugbot`, `poteto`,
  `/loop`, `subagent_type` and `model:` matches one line,
  `CREATION-LOG.md:18`, which names the source path `cursor/plugins` as
  provenance. That line predates ppb-35 and names no tool. ppb-35 removed the
  same `cursor/plugins` path from the `SKILL.md` header. The pstack skill
  reference `unslop` is swapped for `bee-unslop`, per D2.
- **bee rules removed:** none. The ppb-35 diff to `SKILL.md` has two removed
  lines. The header line comes back without the `cursor/plugins` path. The
  compass sentence comes back with two sentences appended. Every other change is
  an insertion. The project-standard precedence line, the "never rewrite quoted
  evidence" line, the PR and commit format, and the Headless section are
  unchanged.
- **Other checks:** the description is unchanged at 253 characters, under 300.
  `skills/bee-technical-writing/LICENSE.pstack` is byte-identical to the pstack
  `LICENSE`. `SKILL.md` and `LICENSE.pstack` match the source byte for byte in
  all five rendered trees (`.claude-plugin`, `.codex-plugin`, `.agents`,
  `.opencode`, `.claude`).
- **Whole-file diff:** after ppb-35, the pstack body and the bee body differ in
  six places. Four are bee additions (header, precedence line, PR format,
  Headless). Two are punctuation: a colon became a full stop in the reference
  mode and in the "don't churn" sentence. The other two differences are the
  unslop line and the PR line, both listed as L below.

## Rule by rule

P = present in bee-technical-writing. L = left out, with the bee home or the
bee difference. M = missing.

### Frontmatter and opening

| Source rule | Status | Note |
|---|---|---|
| `name`, `description` | L | bee routes by its own name and description (D5: the bee skill stays bee's) |
| `disable-model-invocation: true` | L | bee frontmatter has no such key; bee routes by description |
| Goal: a tired engineer understands on the first read; four layers, apply all four | P | |
| Cut every word that does no work | P | |
| Use the short, everyday word | P | |
| When a rule makes a sentence worse, fix it another way or leave it | P | |
| The codebase is the word list | P | |
| Don't invent jargon; a named pattern is fine when defined | P | |
| Add new offenders to `unslop`'s abstract-metaphor rule | L | bee differs: propose the addition with a diff in the reply and never edit `bee-unslop` |

### Vary the rhythm

| Source rule | Status | Note |
|---|---|---|
| Intro: a doc can obey the layers and still read machine-written | P | |
| Mix sentence lengths on purpose | P | |
| One thought per sentence is not one length per sentence | P | |
| Have a view where the mode allows it | P | |
| Be specific over sterile | P | |

### Pick the mode first (Diátaxis)

| Source rule | Status | Note |
|---|---|---|
| One document, one mode; two questions pick it | P | |
| The four quadrants | P | |
| Use the compass on a whole document or one sentence | P | |
| Reach for it when unsure; gut feel is often wrong | P | added by ppb-35 |
| Tutorial paragraph | P | |
| How-to paragraph | P | |
| Reference paragraph | P | colon became a full stop |
| Explanation paragraph | P | |
| Don't mix modes; split and link | P | |
| Source line (diataxis.fr) | P | |

### Write sentences to the reader (Google developer style)

| Source rule | Status | Note |
|---|---|---|
| "You", present tense; "will" only for later | P | |
| Say who does what | P | |
| Instructions as commands; never "should be done" | P | |
| Condition before the instruction | P | |
| Common case first | P | |
| Knowledgeable friend; no "please", "simply", "easy", "quickly" | P | |
| No pre-announcing; no repeated sentence openings | P | |
| Read the awkward sentence aloud | P | added by ppb-35 |
| Descriptive link text; never "click here" | P | |
| Headings carry the point; sentence case; one h1 | P | |
| Numbered lists for sequences; parallel items | P | |
| Code font, bold UI, serial commas, no "etc." | P | |
| Source line (developers.google.com/style) | P | |

### Make statements load one at a time (STE rules)

| Source rule | Status | Note |
|---|---|---|
| One instruction per sentence | P | |
| Split at about 20 and 25 words | P | |
| Warning or condition before the step | P | |
| Keep "the" and "a" | P | |
| One meaning and one job per word | P | |
| One word per action | P | |
| Procedures as direct commands | P | |
| Avoid "-ing" words | P | |
| Source line (asd-ste100.org) | P | |

### Leave no sentence open to two readings (Global English)

| Source rule | Status | Note |
|---|---|---|
| "Only" and "not" next to the word they change | P | |
| Break up long noun strings | P | |
| Every "it", "they", "this" points at one thing | P | |
| Don't drop verbs | P | |
| Keep the small words that show structure | P | |
| Repeat the article in a series | P | |
| Say which parts "and" or "or" joins | P | |
| Periods, not semicolons; no em dash | P | |
| Parentheses as full units; no "(s)" | P | |
| No slashes | P | |
| One name per thing; don't churn | P | colon became a full stop |
| No idioms, Latin abbreviations, metaphors | P | |
| Source line (Kohl) | P | |

### Voice and repo specifics

| Source rule | Status | Note |
|---|---|---|
| Apply the unslop skill to every doc | P | swapped to `bee-unslop` (D2) |
| PR descriptions and commit messages are writing; every layer except Diátaxis | P | bee keeps the sentence and adds its longer PR and commit format |
| Product UI strings are not documentation | P | |
| Tabs for code; real paths and symbols; true counts with the regenerating command | P | bee adds a project-standard precedence line that can override tabs |

### Worked example and checklist

| Source rule | Status | Note |
|---|---|---|
| Before and after example | P | |
| The fixes, by layer | P | added by ppb-35 |
| Review checklist items 1 to 8 | P | identical text |

## Counts

66 source rules checked. P: 63. L: 3. M: 0.

## Suspicions that failed verification

- `CREATION-LOG.md:18` names `cursor/plugins`. It reads as a pstack-only name
  but is the source path in a provenance line, not a tool, and the action
  scopes the tool check to tools.
- The upstream summary lists commit `823e5c604` beside ppb-35. That commit is
  the ppb-34c unslop check and does not touch `skills/bee-technical-writing/`;
  `git log -- skills/bee-technical-writing/` shows `6ae9e959e` as the only
  ppb-35 commit.
