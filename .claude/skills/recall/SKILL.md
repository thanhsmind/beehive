---
name: recall
description: "Rebuild recent working context from bee state, your own chat history and the shared record, then hand back a tight current-state brief. Fires when a session resumes a topic whose history is not already in bee state - 'catch me up', 'recall my work on X', 'where did I leave off' - and before starting work that continues something older than this session."
---

# Recall

> Ported from pstack (`cursor/plugins`, skill `recall`), adapted for bee.

**Before you start or resume work, you rebuild the user's recent working context and hand back a tight capsule of where things stand now and what to do next.**

Keep it tight and on-topic. Read only what the in-scope threads need, then stop.

Your context lives in two records. Your own chat history holds what you did and decided. The shared record holds everything that happened around the same code under other names: the symptoms users keep reporting, the fixes that shipped and got reverted, the errors still firing in prod. That second record is what the **why** skill searches, across code archaeology and the bee store. A feature with a long bug tail keeps most of its story there, so do not reconstruct it from your transcripts alone.

Transcripts live at `~/.claude/projects/<slug>/<uuid>.jsonl` (matching `~/.claude/projects/<slug>/*.jsonl`), where `<slug>` is the workspace path with every `/` turned into `-` (for example, `/home/thanhsmind/Projects/goglbe/beehive` becomes `-home-thanhsmind-Projects-goglbe-beehive`). Every line is one JSON record / chat message.

1. **Classify, then route.** One specific prior chat to resume is a session pickup task, not this. Turning habits into a durable skill is an automation task. A human-readable summary of your work is a different task. Recall loads working context across recent chats before you act. If the user already gave you a full state capsule (paths, branch, the change), use it and skip the mining.
2. **Lock the scope before searching.** Pin the window ("recent" is a real range, default the last 7 days), the topic if named, and the workspace (default the active one. Never read another project's transcripts without being asked). State the scope back. Never quietly turn "all" into "recent N".
3. **Read bee state first.** Bee's own state is the cheapest and freshest part of the record and must be read BEFORE any transcript mining:
   - Run `bee orient` for phase, blockers, and next action.
   - Run `bee decisions search --text "<topic>"` for what was decided.
   - Run `bee cells list` for open and capped work.
   - Read `docs/history/<feature>/CONTEXT.md` when the topic names a feature.
   A fact already in bee state never needs to be reconstructed from a transcript.
4. **Fan out across your chat history.** Dispatch parallel extraction workers through the ONE bee door:
   `.bee/bin/bee dispatch prepare --runtime claude --kind gather --role extraction --purpose "mine transcript slice for <topic>" --json`
   Then run exactly the tool and payload it returns; never hand-pick a model.
   Tell every dispatched gather worker:
   - Order candidates by real modification time with `ls -t`, never by UUID name.
   - Grep the topic first, then read only matching files and only their relevant matching regions.
   - Skip the current chat and noise (subagent, eval, and test chats).
   Each worker returns the same schema, one block per chat: topic, user goal, decisions, open threads, struggles and corrections, and artifacts (PRs, tickets, branches), citing the session UUID.
   For one or two chats, skip the fan-out and search directly. Raw transcripts stay in the workers; the main thread receives only their digested findings.
5. **Sweep the shared record.** Sweep the shared record whenever the topic names a feature, file, subsystem, area, or bug. This is the default, not a judgment call, and 'my work on X' does not exempt it. Hand it to the **why** skill's source investigators (whose two sources in this repo are code archaeology and the bee store), but steer their question from 'why was this built this way' to 'what is the current state, what has been tried and did not hold, and what are users still reporting'. Run the investigators in parallel with the chat-history mining, and inherit its posture: one investigator per source, null results are findings. Fold what comes back into the brief. Skip this step only for pure activity recall with no named target ("what did I do this week"), where your own history and live state are the entire answer.
6. **Verify against live state.** Take the PRs, branches, and tickets that the mining and the sweep surfaced and check them with `git` and `gh`. When the answer hinges on what an agent actually did (the tools it ran, files it read, errors it hit), read the full transcript, not just a trimmed local copy.
7. **Write the brief to the contract below.** Group by thread. Stay on the named topic.

## Output contract

Lead with the capsule, then the thread status, then the problems, then the next move. Deeper detail goes below or gets cut.

- **Capsule.** At most 5 bullets. What this work is and where it stands overall.
- **Threads.** One line each, prefixed with exactly one status tag: `[merged #N]`, `[open PR #N]`, `[in flight <branch>]`, `[verified, uncommitted]`, `[reverted #N]`, or `[planned, not started]`. A thread with no tag is not done yet, so tag it.
- **Problems.** At most 5, the recurring ones. Include the symptoms users keep reporting and any fix that shipped and was reverted, so the next attempt starts where the last one failed.
- **Next move.** The single most useful next action, concrete.

An adjacent feature or ticket stays out unless it blocks this one. When the capsule and thread lines outgrow a screen, cut detail before you cut threads. Write the brief through the **unslop** skill, cite chat findings by session UUID and shared-record findings by their source (commit SHA, PR #, decision id, `docs/` path), and sanitize private context before any public output.

**Reply:** the brief, to the contract above.
