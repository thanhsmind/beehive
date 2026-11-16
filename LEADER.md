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
- Keep working in the main thread while subagents run — don't sit idle waiting on them.
- Don't over-deliberate. Enough info to act = act. No long option surveys for decisions with an obvious default.
- Speed never trades away quality: same rigor, same verification, same "done means done". If parallelizing risks a worse result, slow down.

## Release

When the user asks for a release, run `scripts/release.sh <VERSION>`.
That is the whole release: it bumps both plugin manifests, runs
`bee dev regen`, runs the declared test suite BEFORE anything is
tagged, runs the live smoke of the built binary
(`bash .bee/verify/verify-app/control-bee smoke`) between the suite and
the tag, makes the release commit, then tags, pushes, waits for
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

Every task that arrives from waggledance (a `t-<hex>` correlation id) closes with a completion report in the `--kind result` mail body.
Read `docs/handbook/waggledance-completion-report.md` before composing it.

## ASD-STE100 Simplified Technical English

Always respond using ASD-STE100 Simplified Technical English.

Key rules:
- **Use approved words only.** The standard gives a word list. Each word has one meaning.
- **Use one word for one idea.** Do not use two words for the same thing.
- **Write short sentences.** Use 20 words or less for instructions.
- **Use active voice.** Write "Turn the switch", not "The switch must be turned".
- **Write short paragraphs.** Keep one topic in each paragraph.
- If a specific technical term is not in standard STE, define it clearly as a Technical Name or Technical Verb before using it.
