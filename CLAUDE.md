use fable subagents when you need more intelligence

## bee

This repo uses bee. The bare import below loads the BEE operating block from
AGENTS.md at context-load time. Never wrap it in backticks; that disables it.

Work discipline (search tools, done-means-done, act-don't-ask, a question
is a question) is canonical in AGENTS.md, after the BEE block — it loads
through the import below. Do not duplicate it here.

## Speed (Opus 5 only)

When running as Opus 5: optimize for wall-clock speed. Finish tasks quickly.

- Parallelize aggressively. Independent tasks run at the same time, never one after another — batch tool calls, spawn subagents concurrently.
- Delegate by complexity: Sonnet 5 subagents for routine work (search, bulk edits, boilerplate, verification), Opus 5 subagents for hard reasoning that can run independently.
- Keep working in the main thread while subagents run — don't sit idle waiting on them.
- Don't over-deliberate. Enough info to act = act. No long option surveys for decisions with an obvious default.
- Speed never trades away quality: same rigor, same verification, same "done means done". If parallelizing risks a worse result, slow down.
- No conflicts from parallelism: never let two subagents touch the same files or overlapping scope. Split work by non-overlapping boundaries; merge and reconcile results in the main thread.

## Release

When the user asks for a release: bump the version in
`.claude-plugin/plugin.json`, make the release commit, then run
`scripts/release.sh` to the end. The release is done ONLY when the script
prints its final `OK` line — tag pushed, release-binaries CI green, GitHub
release carrying the binaries. A release commit without that OK is NOT a
release; never report a release as done without it.

## Short responses

It's been a long day and my brain is fried, talk to me like I'm 5.

Small words, short sentences, short paragraphs. If you have to use a big word, explain it right after. Only return what's actually necessary.

Just tell me what you did, did it work, what do I do now.

If I have to decide something: 2 options max, the context I need to pick fast, and which one you'd go with.

Keep paths and commands exact.

Always use ASD-STE100 Simplified Technical English when you talk to me.
@AGENTS.md
