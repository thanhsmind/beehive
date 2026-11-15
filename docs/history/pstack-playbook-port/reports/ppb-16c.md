# ppb-16c — bee-show-me-your-work checked against its pstack source

Scope: what ppb-16 wrote in commit `6886cfdbf`. That is `skills/bee-show-me-your-work/`
(`SKILL.md`, `references/decision-log-template.tsv`, `scripts/log.sh`,
`LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-show-me-your-work.md`. Each
is checked against the pstack skill `pstack/skills/show-me-your-work/` (`SKILL.md`,
`references/decision-log-template.tsv`, `scripts/log.sh`). The template, the
script and the licence were compared with `cmp` and `diff`.

Verdict: no source step is missing. Every section and rule of the source
`SKILL.md` is present, in bee's terms where a tool was swapped. Every removed text
(`disable-model-invocation`, the different-model-family requirement, the Cursor
transcript path and its glob warning, the `decisions.tsv` / `.audit/` locations)
has a port-notes entry. The template is byte-identical. `log.sh` matches the
source line for line once the comment lines are removed, and stays executable.
`LICENSE.pstack` is byte-identical to the pstack `LICENSE`. The pstack-only tool
check is clean. No model and no subagent type is picked by hand. The description
is 267 characters and says what the skill is not for. There are three findings.
None blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 (`063a21e4`) counts as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule (source line) | New home | Status |
|---|---|---|
| Frontmatter `name: show-me-your-work` (2) | `name: bee-show-me-your-work` | present |
| Frontmatter description (3) | line 4, rewritten with a "Not for" clause | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Intro: trail lets a reviewer reconstruct what, why, evidence; one canonical log (9) | line 18, unchanged | present |
| Format: one TSV, why TSV, single-line cells, evidence is a pointer (13) | line 24, unchanged | present |
| Format: copy the template header to start a log (15) | line 26, unchanged | present |
| Column `ts` (17) | line 28 | present |
| Column `phase` (18) | line 29 | present |
| Column `decision` (19) | line 30 | present |
| Column `why`, principle said plainly (20) | line 31 | present |
| Column `evidence`, pointer never a paragraph (21) | line 32, plus cell id and decision id | present (addition noted) |
| Column `result` (22) | line 33 | present |
| Example rows, illustration only (24-32) | lines 35-43, unchanged | present |
| Logging: plain words, the unslop skill applies (36) | line 47, `bee-unslop` | present (swap noted) |
| Logging: use `scripts/log.sh`; what it does; bare `printf` caveat (38) | line 49, plus "run from this skill's directory" | present (addition not noted, finding 1) |
| Logging: log decision points, one row per loop iteration, skip the trivial (40) | line 51, unchanged | present |
| Where: local by default at `decisions.tsv` or `.audit/<task-slug>.tsv`, out of git (44) | line 55, `.bee/tmp/<feature>/…` | present (swap noted, bullet 3) |
| Where: commit only when a reviewer needs the trail (46) | line 57, at `docs/history/<feature>/decisions.tsv` | present (swap noted, bullet 3) |
| Rule: one row is one decision (50) | line 61 | present |
| Rule: append-only, supersede never edit (51) | line 62 | present |
| Rule: prefer evidence from committed scripts (52) | line 63, `bee-principle-encode-lessons-in-structure` | present (swap noted, bullet 5) |
| Audit: at run end check the log told the truth (56) | line 67 | present |
| Audit: read the transcript under `agent-transcripts/` (56) | line 67, session record's `transcript_path` via `bee state session list --json`, plus commits, cell traces, decisions | present (swap noted, bullet 4; see finding 3) |
| Audit: never glob `~/.cursor/projects/*/` (56) | line 67, "Read only this run's transcript, never another session's" | dropped as text, rule kept (noted: Dropped steps, bullet 2) |
| Audit: every row maps to a real action (58) | line 69 | present |
| Audit: evidence resolves (59) | line 70 | present |
| Audit: unlogged fork or pivot is a gap (60) | line 71 | present |
| Audit: drop padding (61) | line 72 | present |
| Audit: fix the log, not the story (63) | line 74, plus a `bee-principle-verify-before-reporting` sentence | present (addition noted) |
| Review: must spawn a subagent on a different model family (67) | line 78, one `bee dispatch prepare --runtime claude --kind reviewer --json`, no `--role`, payload as returned | present (swap noted, bullet 1) |
| Review: different model family as a hard requirement (67) | replaced by naming the payload's model and flagging a same-model review (line 87) | dropped (noted: Dropped steps, bullet 1) |
| Review: self-review is not a substitute (67) | line 78 | present |
| Review: reads trail and transcript, flags what to watch, not a redo (67) | line 80, "the run's records" | present |
| Review flag: weak or absent evidence (69) | line 82 | present |
| Review flag: verification skipped or unproven (70) | line 83, "in the records" | present |
| Review flag: risky choices in hindsight (71) | line 84 | present |
| Review flag: gaps a skim would miss (72) | line 85 | present |
| Reply ends with "Attention", `reviewed by <model>` first, flags point at rows; "No flags" valid, model name is not (74) | line 89, `<model the payload named, or "not named">` | present (swap noted, bullet 1) |
| Reviewing the trail: read, follow pointers, `column` render (78) | line 95, unchanged | present |
| Composing: other skills route their trail here (82) | line 99, unchanged | present |

Additions with no source counterpart: the `metadata` block with the `bee-cli`
dependency (noted, frontmatter paragraph); the line 20 paragraph that the trail
never replaces a record (noted, bullet 6); the brief rule on line 78 and the
re-dispatch rule on line 87 (noted, bullet 2, but see finding 2); the
not-the-review-gate line 91 (noted, bullet 6); the Handoff section (noted,
bullet 6).

## `references/decision-log-template.tsv`

`cmp` reports no difference. The header row `ts phase decision why evidence
result` is present, unchanged.

## `scripts/log.sh`

`diff` of both files with every `#` line removed prints nothing. The new file
keeps the `#!` line and drops the usage comment and the five-line comment above
`clean()`, as the port notes say. Every step is present: the six-argument check
with the usage message, `mkdir -p` of the log directory, the header on first
use, the UTC `ts` stamp, the tab/newline/CR strip, the single-quote prefix for a
leading `=`, `+`, `-` or `@`, and the append. Both files are mode `755`.

The dropped comment carried the reason for the quote prefix (spreadsheet formula
execution from attacker-controlled cells). That reason survives in `SKILL.md`
line 49, so nothing is lost.

## `LICENSE.pstack`

`cmp` against the pstack `LICENSE` reports no difference.

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-show-me-your-work/`
prints nothing and exits 1. A wider search under `skills/bee-show-me-your-work/`
for `subagent_type`, `generalPurpose`, `Task tool`, `model:`, `readonly`,
`agent-transcripts`, `.audit/` and the model families `claude-`, `gpt-`, `grok`,
`opus`, `sonnet`, `haiku`, `fable` also prints nothing. The only pstack names
left are in the port notes, as the notes say.

Every bee name the skill cites resolves: `bee-unslop`, `bee-reviewing`,
`bee-principle-encode-lessons-in-structure`,
`bee-principle-record-at-the-moment-of-settlement` and
`bee-principle-verify-before-reporting` are directories under `skills/`;
`bee state session list` is routed (`router.rs:85`) and its JSON returns the full
session records, which carry `transcript_path` from session creation
(`hooks/session_init.rs:481`).

## Hand-picked model or subagent type

None. Line 78 sends the reviewer through `bee dispatch prepare --kind reviewer`
with no `--role`, runs the payload as returned, and says "never pick a model or
a subagent type by hand". The `--runtime claude` flag picks a runtime, not a
model or subagent type; it is the same call `bee-interrogate` line 45 uses.

## Description

267 characters (counted from the folded `description:` value), matching the
port notes. It says what the skill does, when to use it (`/bee-show-me-your-work`,
a human reviewing the run later) and when not to: "Not for recording a settled
rule (bee decisions log)". It fits the 300-character limit. The source's
"autonomous or multi-phase runs" trigger and its "local by default" sentence are
not in the description; the first is covered by "long-running or unattended
work", the second lives in the body (line 55).

## Findings

1. **P3, not noted.** `SKILL.md` line 49 adds "run from this skill's directory"
   to the `scripts/log.sh` instruction. The port notes do not list it among the
   additions. It is a usage clarification, not a new method, but D2 asks every
   change to be named. One clause in bullet 6 of the port notes would close it.
2. **P3, partly noted.** Line 87 re-dispatches a return that "cites a row or path
   that does not resolve" as well as one that breaks the flag format. Port-notes
   bullet 2 names only the flag-format case. Same gap ppb-15c found in
   bee-interrogate; the path case comes from the same `bee-reviewing` rule.
3. **P3, observation.** Line 67 tells the agent to find "this session's
   transcript" through `bee state session list --json`, which lists every
   session record. It does not say how the agent knows which record is its own.
   The source leaned on the system prompt naming the path. Not a missing step,
   since the rule to read only this run's transcript is kept, but a reader may
   not know which row to pick.

## Verify

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-show-me-your-work/ ; test $? -eq 1`
— no output, `test` exits 0.
