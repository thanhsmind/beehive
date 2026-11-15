# Port notes — skill bee-show-me-your-work, cell ppb-16

Source: pstack skill `show-me-your-work` (`SKILL.md`,
`references/decision-log-template.tsv`, `scripts/log.sh`) at commit `b9ddc83`,
MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-show-me-your-work/LICENSE.pstack`, a verbatim copy of the pstack
`LICENSE`. This file is the only place a pstack-only tool is named; no file under
`skills/bee-show-me-your-work/` names one.

Kept as the base, with the swaps below: the format section and its six columns,
the plain-spoken example rows, "Logging a row", "Where it lives" (local by
default, committed only when a reviewer needs the trail), the three rules, the
audit of the log against the run, the cross-model review with its "Attention"
section, "Reviewing the trail" and "Composing this skill". The template header
row is a byte copy. `scripts/log.sh` is the source script with its comment lines
removed and the `#!` line kept; the logic is unchanged (stamp, header on first
use, strip tab and newline, quote a leading `=`, `+`, `-` or `@`).

Frontmatter follows `bee-how`: `name`, a description of 267 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee`
and a `bee-cli` dependency that degrades to a trail with no independent review.
The source's `disable-model-invocation: true` is dropped, as in `bee-architect`,
`bee-arena` and `bee-interrogate`: the description names the explicit trigger.

## Tool swaps

- "Spawn a subagent on a different model family" became one
  `bee dispatch prepare --runtime claude --kind reviewer --json` with no `--role`,
  the returned payload run exactly as returned, no model or subagent type named.
  The model is a setting of the review role in the bee team table. The source's
  hard requirement of a different model family cannot be guaranteed by a skill
  that never names a model, so the skill instead requires the reply to name the
  model the payload carried and to say when it is the model that did the work.
- The brief rule (the reviewer brief carries the log path and records, never the
  leader's verdict, leaning or expected flags) and the one re-dispatch for a
  return that breaks the flag format come from `bee-reviewing` and
  `bee-interrogate`; the pstack source has neither.
- The default log location `decisions.tsv` in the work dir, or `.audit/<task-slug>.tsv`,
  became `.bee/tmp/<feature>/decisions.tsv` or `.bee/tmp/<feature>/<task-slug>.tsv`
  when local, and `docs/history/<feature>/decisions.tsv` when committed. This is
  where bee keeps scratch and feature history.
- "Read this run's transcript under the active workspace's `agent-transcripts/`
  directory (the system prompt names the path)" and the warning not to glob the
  per-project transcript directories became "this run's own records: commits and
  cell traces, logged decisions, and the session transcript at the session
  record's `transcript_path` (`bee state session list --json`)", with the same
  rule to read only this run's transcript. The transcript path is the same
  evidence; the lookup is bee's session record.
- "The **unslop** skill" became `bee-unslop`. "The **encode-lessons-in-structure**
  principle skill" became `bee-principle-encode-lessons-in-structure`.
- Added, not in the source: the opening paragraph that the trail never replaces a
  record and a settled rule still goes through `bee decisions log`
  (`bee-principle-record-at-the-moment-of-settlement`); cell id and decision id as
  evidence pointers; one sentence from `bee-principle-verify-before-reporting` in
  the audit; the line that the reviewer read is not the review gate, which stays
  `bee-reviewing`'s; a `Handoff` section ("Invoke bee-hive skill"), as in
  `bee-interrogate`.

## Pointers kept as plain sentences

None. Every skill and principle the source names exists in bee.

## Dropped steps

- The rule that the reviewer must be of a different model family: replaced, not
  kept, for the reason in the first swap (no skill names a model).
- Cursor's workspace path for transcripts and the system-prompt hint about it:
  no bee counterpart as text; replaced by the session record lookup above.
- Graphite, the Cursor loop, bugbot, the poteto-agent, arena, architect, how and
  why are not cited in the source, so nothing was dropped for them.

## Trigger fixture

Six briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
four expect (one Vietnamese) and two near (expect `bee-capturing` and
`bee-reviewing`), none naming the slug.
