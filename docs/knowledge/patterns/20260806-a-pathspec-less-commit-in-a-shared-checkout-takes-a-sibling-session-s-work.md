---
type: bee.pattern
title: "A pathspec-less commit in a shared checkout takes a sibling session's staged work"
description: "One session ran an ordinary commit with a message and no paths while another session had unrelated files already staged in the same index; the commit swept them in, so a cell's diff carried a stranger's changes and neither session's history says what it looks like it says."
tags: [git, concurrency, shared-checkout, commits, hygiene]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-a-pathspec-less-commit-in-a-shared-checkout-takes-a-siblings-staged-work
  lifecycle: active
  areas: [worktree-parallelism, workflow-state]
  decisions: [workflow-lifecycle wl-2 (the git-hygiene finding recorded on the cell trace rather than silently fixed)]
  sources: ["workflow-lifecycle cell wl-2 (docs/history/workflow-lifecycle/promote-proposals.md — trace quotes the incident: a first commit made while a concurrent lane had unrelated files staged in the same shared index, so those files landed inside that commit's diff)"]
  polarity: pitfall
  critical: false
---

# A pathspec-less commit in a shared checkout takes a sibling session's staged work

The staging area belongs to the checkout, not to the session. Two agents working
the same checkout share one index, and a commit written the ordinary way —
message, no paths — commits *the index*, not *your change*. Whatever the other
session staged and had not yet committed goes with it.

The instance: one cell's first commit was made while another lane had unrelated
files staged. Those files landed inside the commit's diff. Nothing failed,
nothing warned, and both sessions carried on. The damage is quiet and it is to
the record: one cell's commit claims changes it did not make, the other
session's next commit is missing changes it thinks it still has staged, and the
one-commit-per-cell rule — whose entire value is that a diff and a cell describe
the same thing — is silently false for both.

## The rule

- In any checkout that another session might be using, commit with an explicit
  pathspec: name the files the work touched. The cell already lists them.
- Never reach for the stage-everything flags there. They are the same defect
  with a shorter spelling.
- Before committing, look at what is staged. If it holds anything the cell does
  not name, that is another session's work and it is not yours to commit — the
  fix is to narrow your own commit, never to unstage someone else's files.
- The structural answer is a worktree per feature, which is why code-touching
  work is supposed to start in one: separate checkouts have separate indexes and
  this failure cannot occur. This pattern is what the rule is protecting
  against — every time work runs in a shared checkout because a worktree felt
  like ceremony, this is the cost being risked.
- When it does happen, say so on the trace, as this cell did. A commit whose
  contents are wrong is discoverable later only if someone wrote down that it
  happened.
