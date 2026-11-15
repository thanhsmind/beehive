---
name: bee-comment-review
description: >-
  Send a fresh reviewer over a change's comments, delete the ones that fail, fix the workaround behind each flag and offer encodings for constraint comments. Use when the user types /bee-comment-review or asks to strip or audit comments. Not for the review gate (bee-reviewing).
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The comment reviewer goes out through the vendored bee binary's dispatch door. Absent, the leader reads the scoped comments itself with the same reference and the account says so.
---

# Comment review

Send a comment reviewer over the scope. Act on accepted findings.

Authoring agents defend comments. Defer to the reviewer's fresh perspective.

The leash is the project's own no-comments rule, never a list restated here: the marked rule `agents-no-code-comments` in `.bee/LEADER.md`, switched by the `no_code_comments` config key, and the comment baseline that only ever goes down. `references/comment-reviewer.md` says how the reviewer reads it.

## Scope

Use the caller's files or diff. Otherwise use the current diff against the base branch, default `main`, including the working tree.

## Steps

1. Dispatch the reviewer. Issue `bee dispatch prepare --runtime claude --kind reviewer --json` with no `--role`, and run the returned payload exactly as it returns. Never pick a model or a subagent type by hand: which model the reviewer runs on is a setting of the review role in the bee team table. The brief is `references/comment-reviewer.md` plus the scope, and does not restate its rules or carry your own verdict. The reviewer is read-only: it returns a report and edits nothing.
2. Inspect its report against the files. Reject application-code edits, scope escapes, deletions of text the rule protects, misstated `MUST KILL` reasons, and flags that treat kept intentional code as guilty. Reshape flags on our-code surprises stay actionable. Do not keep those comments. A keep survives only with proof it is about something we cannot change. Audit the scoped lint and type-checker suppressions the reviewer missed. Correctness or safety suppressions stay actionable `MUST KILL`s. Apply the accepted comment deletions yourself. Refuse a deletion only with the exact exception that protects the text and scoped proof that it holds. Before accepting thin `IMPORTANT` or `do not remove` kills or keeps, run `bee-how` or `bee-why` on their symbol. If a kill is ambiguous, do not apply it. If a keep is refuted or still ambiguous, delete it. Re-dispatch one rejected report with the failure named. Reject a second, report it open, and fail `bee-comment-review`.
3. Fix trivial accepted flags directly by deleting a dead path, dropping a parameter, or using the real API. If any fix needs a shape, run `bee-architect` once for the accepted set and surrounding code. Stop at the sketch. `bee-architect` shapes. Step 4 implements.
4. Implement the smallest root-cause fix in scope. Remove every named workaround. If the root cause is out of scope, land the smallest in-scope fix and report the rest open. `bee-principle-crash-site-versus-fault-site` and `bee-principle-subtract-before-you-add` guide intent: fix the origin, delete before you add, never bolt on a symptom guard. A redesign-from-first-principles stance applies too: build the fix as if the requirement had always existed. None of them widens the fence or fixes instances outside it.
5. Constraint comments say `do not remove`, `do not change wording`, or `talk to X before changing`. Leave keeps about things we cannot change, where the project's rule allows a keep. Offer the cheapest in-scope type, runtime, test, or CI lint. Wait for interactive approval. An unattended run requires the caller's pre-approval. If approved, encode then delete. Otherwise delete, report the constraint open, and sketch out-of-scope work. Where the rule is on, the why a deleted comment carried goes to a `docs/knowledge` concept whose Pointers name the file, or to `bee decisions log`, never to a new comment.
6. Report the deletion count, comments not applied, re-dispatches, architect sketch, fixes, encoding offers, encodings, unenforced constraints, and other open work. For a long or unattended run, keep the decision trail with `bee-show-me-your-work`.

## Reply

The step 6 report. Edits are limited to comment lines and the in-scope fixes of steps 3 to 5.

## Handoff

Report delivered. Invoke bee-hive skill.
