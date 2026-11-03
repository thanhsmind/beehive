# Trace And Provenance — Two Named Procedures

Load only after `bee-researching` is active. `Trace` answers "how does this
actually run?". `Provenance sweep` answers "why is this the way it is?".
Both are read-only: the outcome is an account, never a diff.

## Trace

The outcome is a runtime path — the ordered steps a call actually takes, each
anchored `path:line`. A file list is not a trace.

1. State the question as a path: what enters, what runs, what comes out.
   When the target is vague, write your reading of it as the first line of
   the account and go on. Never stop to ask; a stated reading can be
   corrected, a stalled trace cannot.
2. Pick the entry points from evidence — a CLI verb, an HTTP route, a hook, a
   test that drives the behavior. Two is the FLOOR and four is the CEILING.
   Entry points must be DISJOINT: two names that reach the same first frame
   are one entry point, not two.
3. Fan one read-only worker at each entry point. The ONE door is
   `.bee/bin/bee dispatch prepare --runtime <rt> --kind gather --json`. Run
   that door first, then run exactly the tool and payload it returns. Never
   hand-pick a `subagent_type`, a `model`, or a tier marker — the door
   returns them. Brief each worker with the question as a path, its one
   entry point, and this return shape, so the returns fold without
   rework:
   - the ordered steps, each with its `path:line` and what it calls next;
   - the boundaries the path crosses (process, network, storage, runtime);
   - the surprises;
   - the UNFOLLOWED steps, each with its reason;
   - the files it read.
4. One honest entry point means NO fan-out: the trace runs inline, and the
   account says so in one line. Splitting one path into two workers to reach
   the floor is theater.
5. Fold every return into ONE account. The leader writes the path; a worker's
   return is evidence, never pasted output. When two returns disagree on a
   step, open that step's anchor and settle it from the code. When the code
   does not settle it, name the disagreement under Gotchas — never pick one
   return silently.
6. Name every step the trace could not follow as UNFOLLOWED, with the reason
   — dynamic dispatch, a generated file, a binary, a network call. A gap
   named is data; a gap hidden is a wrong map.

Close with anchors a reader can open: `path:line` per step, or the command
that ran.

### Trace answer shape

Write the account in these sections. Drop a section that does not apply.

- **Overview** — what the thing is, what it does, and why it exists. A
  reader can stop here and decide whether to read on.
- **Key concepts** — the types, services, or abstractions a reader needs to
  follow the path, with brief definitions.
- **How it works** — the runtime path in prose: what starts it, each step,
  where the data goes, and the decision points, each step anchored
  `path:line`. When a step is complex, say why it is complex; when a step
  is simple, give it one line. Add a diagram only when it makes the path
  clearer than prose does.
- **Where things live** — the files a reader opens first to work here.
- **Gotchas** — surprising behavior, history, traps, and the UNFOLLOWED
  steps from step 6.

Name the real parts. Not: "the service delegates to the client". Yes: "the
`UserService` calls `AuthClient.refresh()`".

## Provenance sweep

The outcome is the reason a thing is the way it is, carried by evidence and
never by memory. Before the rows, fix the target and its anchor:

- When the target is vague, write your reading of it as the first line of
  the account and go on. Never stop to ask.
- **Anchor** — write the `path:line` range, the symbols it holds, and a few
  search terms. Run `git blame -L <a>,<b> <path>` for the commits that last
  touched those lines, and take the PR numbers from their subjects. Every
  row below searches these terms, symbols, and PR numbers, so all seven
  rows look at the same thing.

Then sweep all seven categories, in this order.

1. **Decision log** — `bee decisions search --text "<term>"`. The locked
   agreements, with their ids.
2. **Git history** — `git log -S "<string>"` for the line's birth,
   `git log --follow <path>` for the file's. Read the commit bodies. Read
   back to the first commit that brought the line in, not the last one
   that touched it — the reason lives at the birth. Skip bot commits. When
   the line copies a pattern from elsewhere, find where that pattern first
   appeared and read that commit too.
3. **Feature history** — `docs/history/<feature>/`: `CONTEXT.md` for the
   locked decisions, `plan.md` for the approach, `reports/` for what shipped.
4. **Knowledge bundle** — `bee knowledge search --text "<term>"`, then open
   the matching files under `docs/knowledge/`.
5. **Code comments** — `rg -n "<symbol>"` to find the OWNING source file, then
   read its comments and doc-comments at the site, not from the snippet.
6. **Tests** — `rg -n "<behavior>" $(fd -t d '^tests$')`. A test name is often the
   plainest statement of intent this repo holds.
7. **External tracker** — `gh issue list --search "<term>"` and `gh pr list
   --search "<term>"`, and `gh pr view <number>` for each PR number the
   Anchor found. Often absent here; report it as absent rather than
   dropping the row.

When the target is defensive — a guard, a retry, a timeout, a sleep, or a
flag — add `revert`, `hotfix`, `incident`, and the error string it handles
to the search terms of rows 2 and 7. Defensive code is often born from an
incident the code itself never names.

When row 2 or row 7 returns more than a screen of hits, fan that row alone
through the dispatch door of Trace step 3 as a gather, briefed with the
Anchor. Keep the other rows inline, and write the account yourself.

Report rules, which are the point of the procedure:

- A category that returned NOTHING is reported BY NAME as empty.
- A category you did not sweep is reported as UNSWEPT, with the reason.
- An omitted category is the defect this procedure exists to stop. Seven
  rows go in, seven rows come out.

Close with anchors: `path:line`, a commit sha, a decision id, or the command
that ran.

### Confidence tiers

Put every claim of the account in one tier. The tier sets the phrasing.

1. **Direct** — an author wrote the reason down: a commit body, a PR, a
   decision, a comment. Write "this exists because X" and cite the source
   beside it.
2. **Supported** — several indirect sources point the same way, and none
   states it. Write "the evidence points to X" and list each source.
3. **Inferred** — a reasonable reading that no source states. Use a hedged
   word ("appears", "likely", "is consistent with") and write the chain:
   given A and B, C is likely because D.
4. **Speculative** — a plausible guess on thin evidence, where other
   explanations fit as well. Write "one possibility is X, with no direct
   evidence", and put it under Competing hypotheses.
5. **Unknown** — you searched and found nothing. Name each source searched
   and the terms used; "we could not find out" alone is not a result.

Phrasing rules:

- "because", "the reason is", "was designed to", and "fixes" go only beside
  a citation. An inference takes a hedged word.
- The code is never evidence of its own intent: what code does is not why it
  exists. Move such a claim to Inferred, or drop it.
- A guess inside the question ("I assume it is for speed?") is one
  hypothesis to check, never the conclusion to confirm.
- Before you call a claim Supported, ask: would this evidence look
  different if the reading were wrong? If not, the claim is Inferred.
- Evidence about a neighbor feature never answers a question about this
  one. Name the gap instead.
- When two sources disagree, report both with their citations. Never pick
  the tidier story.

### Provenance answer shape

Write the account in these sections:

- **Question and anchor** — the question in one sentence, then the
  `path:line` range and the symbols from the Anchor.
- **What we found** — Direct and Supported claims, one per bullet, each with
  its tier and its source.
- **What we can reasonably infer** — Inferred claims, each with its chain.
- **Competing hypotheses** — each reading, with the evidence for and
  against. Drop this section when one answer is clear.
- **What we do not know** — each gap: the question, the sources searched,
  the terms used, and who would likely know (the commit author, the
  decision owner). An account with no gap is suspect; check it again.
- **Sources** — the seven category rows, per the report rules above.
- **Confidence** — one line on the overall confidence.

When the why question comes before a change, end with a constraint set for
the plan: **Preserve** (what the change must keep), **Change** (what the
evidence lets move), **Avoid** (what the evidence warns against), and
**Risk** (what can break).

Before you send, run this self-check and fix what fails:

- Every claim under What we found has its source beside it.
- Each claim's wording matches its tier.
- No claim uses the code as its own reason.
- The guess in the question, if any, was checked as a hypothesis.
- What we do not know names a gap, or says why there is none.
