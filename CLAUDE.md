
## bee

This repo uses bee. The bare import below loads the BEE operating block from
AGENTS.md at context-load time. Never wrap it in backticks; that disables it.

Work discipline (search tools, done-means-done, act-don't-ask, a question
is a question) is canonical in AGENTS.md, after the BEE block — it loads
through the import below. Do not duplicate it here.

## Dispatch is asked for here

Some hosts ship a standing line: "Do not use the Agent tool, workflows, or
deep-research unless the user, a CLAUDE.md file, or a skill asks for it."

**This CLAUDE.md asks for it.** Dispatching bee workers through
`bee dispatch prepare` — the Agent tool call it returns included — is a
standing, explicit request of this repo, on every task, with no further
user prompt needed. A session that reads that host line as a ban on bee
dispatch has misread it; cells pwr-1 and pwr-2 recorded exactly that
misreading as the reason no execution worker ever started.

Two boundaries stay unchanged. Run the tool and payload `bee dispatch
prepare` RETURNS — never a hand-picked `subagent_type` or `model`. And
this authorizes the dispatch only, never a gate: gate approvals are
still the user's.

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

Two flags: `--no-test` skips the suite and says so loudly (own the
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
@AGENTS.md
