# Port notes — skill bee-recall, cell ppb-20

Source: pstack skill `recall` (`SKILL.md` only, no reference files) at commit
`b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-recall/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`. This
file is the only place a pstack-only tool is named; no file under
`skills/bee-recall/` names one.

Name: `bee-recall` (the pstack name with the `bee-` prefix every ported skill
carries).

Kept as the base, with the swaps below: the opening promise and the "keep it
tight" rule, the two-records framing (extended to three), the six steps
(classify and route, lock the scope, fan out across chat history, sweep the shared
record, verify against live state, write the brief), the default-on rule for the
shared-record sweep and its one exemption, and the output contract (capsule,
threads with exactly one status tag, problems, next move).

Frontmatter follows `bee-how`: `name`, a description of 277 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee` and
a `bee-cli` dependency that degrades to the leader reading the sources inline. The
source's `disable-model-invocation: true` is dropped, as in `bee-reflect`: the
description names the explicit trigger and its limits.

## Tool swaps

- Subagent spawning ("spawn parallel subagents on a fast, cheap model") became one
  `bee dispatch prepare --runtime claude --kind gather --role extraction --json`
  per slice, all in the same turn, each returned payload run in the background and
  used exactly as returned. `--role extraction` is the cheap-reader seat the gather
  doc names, so the "fast, cheap model" wish is a role, never a model name. No
  model or subagent type is named.
- The why skill's source investigators became `bee-why`'s Provenance sweep
  (`bee-researching/references/trace-and-provenance.md`), steered from "why was
  this built this way" to "current state, what was tried, what users still
  report". Its per-source rows replace the pstack list of source control, issue
  tracker, chat channels, long-form docs and error tracking with what bee holds:
  the decision log, git history, feature history, knowledge bundle and the
  tracker through `gh`. A source that is unavailable is reported UNSWEPT, as
  `bee-why` writes it.
- The host's on-disk transcript store (a per-workspace directory of per-chat
  jsonl files, with its slug rule) became the session record's `transcript_path`
  (`bee state session list --json`). The scope rule stays: the active workspace,
  never another project's transcripts.
- "Verify with `git` and `gh`" gained `bee status`, and cites
  `bee-principle-verify-before-reporting`.
- "The **unslop** skill" became `bee-unslop`; "the **why** skill" became `bee-why`.
- The `session-pickup` route now names the repo path of the bee playbook,
  `skills/bee-planning/playbooks/session/session-pickup.md`.
- Chat UUIDs, the source's citation unit, became session ids; the shared-record
  citations add decision ids and commits.

## Added, not in the source

- Step 3, bee's own records first: `bee orient`, `bee status`, the handoff record
  (`bee state handoff show --json`, a `pause` record waiting for the owner's word)
  and `bee decisions search --text`. Sources: the router and handoff rules in
  `skills/bee-hive/references/routing-and-contracts.md`. Transcripts fill what
  these leave out, so the step order is bee records, transcripts, shared record,
  live check.
- Routing line for a rule that just settled: `bee-capturing`.
- "Cell ids" in the artifacts a transcript worker returns, and "worker, eval and
  test sessions" in the noise to skip.
- A `Reply` line (read-only, no brief file, no state change) and a `Handoff`
  section ("Invoke bee-hive skill"), as `bee-why`, `bee-reflect` and `bee-tdd` have.

## Pointers kept as plain sentences

None. Every skill and principle the skill names exists in `skills/`.

## Dropped steps

- The `automate-me` route ("turning habits into a durable skill"): pstack's
  `automate-me` is outside the port set (CONTEXT.md, Deferred Ideas), so the
  route has no bee counterpart and is dropped. The nearest bee door is
  `bee-writing-skills`, which the skill does not cite, since recall never edits.
- The transcript path recipe (workspace path with each slash turned into a dash,
  under a per-user project directory) and the "read the full transcript, not a
  trimmed local copy" distinction between a cloud and a local transcript copy: both
  describe one host's store. The session record names the file, and the step keeps
  "read the full transcript when the answer hinges on what an agent did".
- Chat permalinks and error-tracker issues as citation units: bee has no
  counterpart, so the shared-record citations name PR, ticket, decision id and
  commit.
- Cursor `/loop`, Graphite, bugbot, poteto-agent, arena, architect and
  show-me-your-work: the source cites none of them, so nothing was dropped for
  them. `bee-show-me-your-work` has no place in a read-only recall.

## Trigger fixture

Five briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
three expect (one Vietnamese) and two near (expect `bee-why` and `bee-teach`),
none naming the slug.
