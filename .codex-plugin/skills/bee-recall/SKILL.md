---
name: bee-recall
description: >-
  Rebuild recent working context from bee's records, session transcripts and the shared record, then hand back a current-state brief. Use when the user types /bee-recall or asks to catch up or where they left off. Not for resuming one session (session-pickup) or a why (bee-why).
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The transcript miners and the shared-record sweep go out through the vendored bee binary's dispatch door, and the first sources are read through its verbs. Absent, the leader reads them inline and the brief says so.
---

# Recall

**Before you start or resume work, you rebuild the user's recent working context and hand back a tight capsule of where things stand now and what to do next.** Use it for "recall my work on X", "catch me up", "what have I been working on", or "where did I leave off".

Keep it tight and on-topic. Read only what the in-scope threads need, then stop. The heavy reading fans out to parallel workers. The main thread keeps only their findings and the final brief.

Your context lives in three records. Bee's own records hold the state: the phase, the cells, the handoff and the locked decisions. Your own session transcripts hold what you did and decided. The shared record holds everything that happened around the same code under other names: the symptoms users keep reporting, the fixes that shipped and got reverted, the errors still firing in prod. That third record is what `bee-why` sweeps, across the decision log, source control, the feature history, the knowledge bundle and the issue tracker. A feature with a long bug tail keeps most of its story there, so don't reconstruct it from your transcripts alone.

## Process

1. Classify, then route. One specific prior session to resume is the `session-pickup` playbook (`skills/bee-planning/playbooks/session/session-pickup.md`), not this. A rule that just settled goes through `bee-capturing`. A human-readable summary of your work is a different task. Recall loads working context across recent sessions before you act. If the user already gave you a full state capsule (paths, branch, the change), use it and skip the mining.

2. Lock the scope before searching. Pin the window ("recent" is a real range, default the last 7 days), the topic if named, and the workspace (default the active one; never read another project's transcripts without being asked). State the scope back. Never quietly turn "all" into "recent N".

3. Read bee's own records first. They are cheap and they are the current truth about the work:
   - `bee orient` for the phase, the active feature and the next skill.
   - `bee status` for the cells, the claims and what is open.
   - The handoff record, `bee state handoff show --json`. A `pause` record is the owner's word to confirm before anything continues.
   - `bee decisions search --text "<topic>"` for the locked decisions on the topic, with their ids.

   The transcripts fill what these records leave out.

4. Fan out across your session transcripts. The session records name each transcript (`bee state session list --json`, field `transcript_path`). Issue one `bee dispatch prepare --runtime claude --kind gather --role extraction --json` per slice of the corpus, all in the same turn, and run every returned payload in the background, exactly as it returns. Never pick a model or a subagent type by hand: which model a role runs on is a setting of the bee team table. Searching transcripts is grunt work, which is why the extraction role is the seat. Tell every worker to order candidates by real modification time (`ls -t`) and never by file name, grep the topic first and then read only the matching sessions and only their relevant regions, and skip the current session plus obvious noise (worker, eval and test sessions). Each returns the same schema, one block per session: topic, the user's goal, decisions, open threads, struggles and corrections, and artifacts (PRs, tickets, branches, cell ids), each citing the session id. For one or two sessions, skip the fan-out and search directly. The raw transcripts stay in the workers. The main thread gets only their findings.

5. Sweep the shared record whenever the topic names a feature, file, subsystem, area, or bug. This is the default, not a judgment call, and "my work on X" does not exempt it. A named target carries history you never see in your own transcripts, and that history is the point of the sweep. Run the Provenance sweep of `bee-why` (`bee-researching/references/provenance.md`), but steer its question from "why was this built this way" to "what's the current state, what's been tried and didn't hold, and what are users still reporting". Reuse its per-source rows so you don't reinvent each query vocabulary, run them in parallel with the transcript mining, and inherit its posture: one worker per row that needs one, null results are findings, a source that is unavailable is reported UNSWEPT with the reason. Fold what comes back into the brief. Skip this step only for pure activity recall with no named target ("what did I do this week"), where bee's records and your own sessions are the entire answer.

6. Verify against live state. A transcript or a stale ticket is history, not current truth, so take the PRs, branches, cells and tickets that the mining and the sweep surfaced and check them with `bee status`, `git` and `gh`, per `bee-principle-verify-before-reporting`. When the answer hinges on what an agent actually did (the tools it ran, files it read, errors it hit), read the full transcript, not just a trimmed copy.

7. Write the brief to the contract below. Group by thread. Stay on the named topic.

## Output contract

Lead with the capsule, then the thread status, then the problems, then the next move. Deeper detail goes below or gets cut.

- **Capsule.** At most 5 bullets. What this work is and where it stands overall.
- **Threads.** One line each, prefixed with exactly one status tag: `[merged #N]`, `[open PR #N]`, `[in flight <branch>]`, `[verified, uncommitted]`, `[reverted #N]`, or `[planned, not started]`. A thread with no tag is not done yet, so tag it.
- **Problems.** At most 5, the recurring ones. Include the symptoms users keep reporting and any fix that shipped and was reverted, so the next attempt starts where the last one failed.
- **Next move.** The single most useful next action, concrete.

An adjacent feature or ticket stays out unless it blocks this one. When the capsule and thread lines outgrow a screen, cut detail before you cut threads. Write the brief through `bee-unslop`, cite session findings by session id and shared-record findings by their source (PR #, ticket ID, decision id, commit), and sanitize private context before any public output.

## Reply

The brief, to the contract above. Read-only: write no brief file, change no state.

## Handoff

Brief delivered. Invoke bee-hive skill.
