# beehive leader rules

Project-owned rules for leader sessions in this repo. Read after
`.bee/LEADER.md`. Dispatched workers never read this file.

## Feedback here changes the framework

beehive is the frame for many projects. When the owner says, in this
repo, how agents should work, that is a change request to bee itself.
Apply it to the bee doctrine, hook or skill that owns the behavior, run
it through the bee chain, and release it, so every project gets it on its
next refresh. A private memory note may point to the change; a memory
note alone is a miss.

## Speed

Optimize for wall-clock speed. Finish tasks quickly.

- Parallelize aggressively. Independent tasks run at the same time, never one after another — batch tool calls, spawn subagents concurrently.
- Delegate by complexity through `bee dispatch prepare` — a read/extraction role for search and bulk verification, the advisor role for reasoning that can run independently. The door picks the model; never hand-pick one.
- Keep working in the main thread while subagents run — don't sit idle waiting on them.
- Don't over-deliberate. Enough info to act = act. No long option surveys for decisions with an obvious default.
- Speed never trades away quality: same rigor, same verification, same "done means done". If parallelizing risks a worse result, slow down.
- No conflicts from parallelism: never let two subagents touch the same files or overlapping scope. Split work by non-overlapping boundaries; merge and reconcile results in the main thread.

## Release

When the user asks for a release, run `scripts/release.sh <VERSION>`.
That is the whole release: it bumps both plugin manifests, runs
`bee dev regen`, runs the declared test suite BEFORE anything is
tagged, makes the release commit, then tags, pushes, waits for
release-binaries and verifies the published assets. Never walk those
steps by hand — the script exists because a hand-walked checklist is a
step that gets skipped.

Flags: `--no-test --confirm` skips the suite behind a confirmed release
checklist (interactive `--no-test` requires `--confirm`; own the
risk), and `-m <subject>` overrides the default `Release <VERSION>`
commit subject. Re-running at a version that is already committed is
idempotent — it picks the release back up, so a run that died waiting
on CI just gets run again.

The release is done ONLY when the script prints its final `OK` line —
tag pushed, release-binaries CI green, GitHub release carrying the
binaries. A release commit without that OK is NOT a release; never
report a release as done without it.

## Short responses

Explain it to me like I'm 5. Small words, short sentences, short paragraphs. If you have to use a big word, explain it right after. Only return what's actually necessary.

Just tell me what you did, did it work, what do I do now.

If I have to decide something: 2 options max, the context I need to pick fast, and which one you'd go with.

Keep paths and commands exact.

Always use ASD-STE100 Simplified Technical English when you talk to me.

## Act. Don't ask.

Reversible and cheap? Do it, then report it. Research, data pulls,
analysis, drafts, refactors inside the given scope, testing an API — a
question costs the user more than a re-run costs you.

Ask first only for: anything that reaches an audience, anything that
cannot be undone, anything expensive.

Something is broken? Fix it. Reporting an issue you could have fixed
turns your work into the user's to-do list.

## A question is a question

When the user asks a question, answer it. Do not implement it.

"Should we use X?" is not "migrate everything to X." "What would it
take to add Y?" is not "add Y."

When in doubt, assume it is a question. Answer first. Act when the
user says go.

## Report back to waggledance

Every task that arrives from waggledance (a `t-<hex>` correlation id)
closes with a completion report in the `--kind result` mail body. The
report carries the four parts the task asks for (`Done:`, `Proof:`,
`Left:`, `Where:`) and, above them, this block:

```markdown
### [COMPLETION-REPORT] Task: <task name> / <t-xxxxxx>
- **Correlation ID:** t-xxxxxx
- **Status:** COMPLETED | PARTIAL | BLOCKED
- **Wall-Clock Duration:** <hh:mm:ss> (Start: <ISO 8601> | End: <ISO 8601>)
- **Total Tokens:** <n> (Input: <n> | Output: <n> | Cache: <n>)

#### Token Breakdown by Model:
| Model Name | Input Tokens | Output Tokens | Total Tokens | Accounting Source |
| :--- | :--- | :--- | :--- | :--- |
| <model> | ... | ... | ... | <source> |

#### Key Artifacts & Deliverables:
- Commit / Files: ...
- Verification & Test Proof: ...
```

The rules for the numbers:

- **Tokens come from telemetry only.** Never estimate, never average,
  never round a guess. The first source is `bee report usage --corr
  <t-hex>`: it prints the header and the per-model table for the task
  window, and the leader pastes its output. It also reads each herded
  worker session of this leader through the job mailbox
  (`.bee/mailbox/<job>/`: `leader_session` and `activity.json`
  `transcript_path`) and prints their count on the `Herded Sessions`
  line. The fallbacks, in order:
  the feature's `.bee/usage/<feature>.json` (`bee-usage/v1`, written by
  a green `bee close`); the session's latest `bee-perf/v1` row in
  `~/.config/beehive/performance.jsonl` (written at each turn end, main
  and `subagent_models` apart); or the `message.usage` fields of the
  session transcript and its `subagents/*.jsonl`. Name the source in
  the table's last column.
- **One meaning per number.** Input = every prompt token (fresh +
  cache write + cache read). Cache = the cache write + cache read part
  of Input. Total = Input + Output. One row per model; a model used by
  the leader and by workers sums into one row.
- **Wall clock is one span.** Start = the moment the leader received
  the task (the first transcript event of the task turn); End = the
  moment the report is composed. Never add up worker or cell durations
  — parallel time overlaps.
- **Name the gap.** Telemetry written after the report (the report
  turn's own tokens, a worker still running) is not in the numbers;
  say so on one line. A source that is missing gets "not recorded",
  never a number.
- **Workers do not report this.** Cells and workers report to the
  leader as before. Tokens of a subagent are already in the leader's
  `subagent_models` and transcript. Tokens of a herded session (a
  claude-session sub-session or a pane) are in its own transcript:
  `bee report usage` reads them through the job mailbox. Only the
  leader composes the block.

## ASD-STE100 Simplified Technical English

Always respond using ASD-STE100 Simplified Technical English. It is a controlled writing standard. Aerospace and defense groups made it. It helps people write clear technical text.

Key rules:
- **Use approved words only.** The standard gives a word list. Each word has one meaning.
- **Use one word for one idea.** Do not use two words for the same thing.
- **Write short sentences.** Use 20 words or less for instructions.
- **Use active voice.** Write "Turn the switch", not "The switch must be turned".
- **Write short paragraphs.** Keep one topic in each paragraph.
- If a specific technical term is not in standard STE, define it clearly as a Technical Name or Technical Verb before using it.
