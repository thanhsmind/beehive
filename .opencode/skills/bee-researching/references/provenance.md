# Provenance sweep

Load only after `bee-researching` is active. `Provenance sweep` answers "why is
this the way it is?". It is read-only: the outcome is an account, never a diff.

The outcome is the reason a thing is the way it is, carried by evidence and
never by memory. Before the rows, fix the target and its anchor:

- When the target is vague, write your reading of it as the first line of
  the account and go on. Never stop to ask.
- **Kind of question** — name which one it is, because the kind sets what a
  Direct source looks like, never which rows run: a design rationale ("why
  built this way"), a tradeoff ("why X and not Y"), an edge case ("what
  motivated this guard"), an outside forcing function (a product, business
  or operations constraint), survival ("why does this still exist" — dead
  code, where the honest answer may be that no reason survives), or history
  ("what is the story of X", a wide sweep of the file's life).
- **Anchor** — write the `path:line` range, the symbols it holds, and a few
  search terms. Run `git blame -L <a>,<b> <path>` for the commits that last
  touched those lines, and take the PR numbers and any ticket ids from their
  subjects and bodies. Every row below searches these terms, symbols, PR
  numbers and ticket ids, so all seven rows look at the same thing.

Then sweep all seven categories, in this order.

1. **Decision log** — `bee decisions search --text "<term>"`. The locked
   agreements, with their ids.
2. **Git history** — `git log -S "<string>"` for the line's birth,
   `git log --follow <path>` for the file's. Read the commit bodies. Read
   back to the first commit that brought the line in, not the last one
   that touched it — the reason lives at the birth. Use `git log -G
   "<regex>"` when the exact string changed shape. Read the diff as well as
   the message: a "small refactor" can hide a behavior change. Skip bot
   commits. When the line copies a pattern from elsewhere, find where that
   pattern first appeared and read that commit too. When the repo squashes
   its PRs the branch commits are gone, so the PR body and its comments carry
   the reason. A CHANGELOG or release-note entry that names the change
   counts here, and so do the files that changed in the same commits.
3. **Feature history** — `docs/history/<feature>/`: `CONTEXT.md` for the
   locked decisions, `plan.md` for the approach, `reports/` for what shipped.
4. **Knowledge bundle** — `bee knowledge search --text "<term>"`, then open
   the matching files under `docs/knowledge/`.
5. **Code comments** — `rg -n "<symbol>"` to find the OWNING source file, then
   read its comments and doc-comments at the site, not from the snippet.
   Read the TODO, FIXME and HACK notes near the target too.
6. **Tests** — `rg -n "<behavior>" $(fd -t d '^tests$')`. A test name is often the
   plainest statement of intent this repo holds.
7. **External tracker** — `gh issue list --search "<term>"` and `gh pr list
   --search "<term>"`, and `gh pr view <number> --json
   title,body,comments,reviews,closingIssuesReferences` for each PR number
   the Anchor found. The review threads and the linked issues are where the
   debated alternatives sit. A ticket closed as a duplicate leads to the
   canonical ticket, and a reopen or a change of scope is part of the
   story, so follow both. Often absent here; report it as absent rather
   than dropping the row.

When the target is defensive — a guard, a retry, a timeout, a sleep, or a
flag — add `revert`, `hotfix`, `incident`, and the error string it handles
to the search terms of rows 2 and 7. Defensive code is often born from an
incident the code itself never names. A revert followed by a re-apply, or a
commit named for an incident, is a strong signal. When a postmortem or an
incident record turns up, read it whole, action items included, since those
tie to code changes. A postmortem that names the target is Direct; several
records that converge on one incident, none naming the target, are
Supported.

When this session exposes tools for records beyond the seven — a team chat
history, an observability or error-tracking tool, a product-analytics store
— each is one more row under the same report rules, swept in the same
pass. A tool that needs a login you lack ends its row as UNSWEPT with the
reason; never write a finding from a row you could not read. Skipping a row
as irrelevant needs proof, not a hunch: a build-time script has no runtime
error trail, but "probably not in there" is a row to run.

When row 2 or row 7 returns more than a screen of hits, fan that row alone
through the dispatch door of `trace.md` step 3 as a gather, briefed with the
Anchor. Keep the other rows inline, and write the account yourself.

Brief that worker to gather evidence and never to answer the why, and to
walk its row this way:

- Cast a wide net first, then narrow; read each PR or thread whole, since
  the reason often sits in a late comment.
- Quote verbatim where the exact wording matters, with its location
  (commit sha, PR number, `path:line`), its author and its date.
- Record the queries as run, so an empty result says what was looked for.
- A link inside your own row (a PR to another PR, a ticket to its parent or
  sibling, a doc to the doc it replaces) is followed to its end. A
  reference that points at another row is recorded as a lead, never
  chased; that row holds it.
- Return: what it searched; Direct evidence, each with its quote and one
  line on how it bears; indirect evidence, each with what it suggests, the
  chain and the other readings it allows; contradictions, both sides cited;
  gaps; leads.

Report rules, which are the point of the procedure:

- A category that returned NOTHING is reported BY NAME as empty.
- A category you did not sweep is reported as UNSWEPT, with the reason.
- An omitted category is the defect this procedure exists to stop. Seven
  rows go in, seven rows come out.
- One record that several rows cite is one reference in the account, with
  every row that found it named beside it.

Close with anchors: `path:line`, a commit sha, a decision id, or the command
that ran.

## Confidence tiers

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
- Collect the evidence first, then see which story it supports. Never pick
  a story and recruit the evidence that fits it.
- Before you conclude, ask what you would expect to see if another reading
  were true, and whether a row looked for it.
- Code that makes sense today may have been written for reasons that no
  longer apply, or for none. Never retrofit a clean reason onto messy
  history. A pattern repeated across files may be copy-paste, and a silence
  ("nobody raised security") is not evidence that the concern was absent.
- Never drop a hedge to sound more authoritative, in the account or when
  you pass a worker's return on.
- "obviously", "clearly", "of course", "just" (as in "it is just for
  speed") and "I think" stay out of the account. It reports the record;
  it is not an opinion.
- Before you call a claim Supported, ask: would this evidence look
  different if the reading were wrong? If not, the claim is Inferred.
- Evidence about a neighbor feature never answers a question about this
  one. Name the gap instead.
- When two sources disagree, report both with their citations. Never pick
  the tidier story.

Reading a source:

- A signal near the ship date — a spike, a monitor, a resolved issue — is
  suggestive. Other changes may sit in the same window, so check the
  neighboring commits. That something was measured shows someone cared, not
  why this line exists, and a "resolved" mark is a human's word, not proof
  the code fixed it.
- Check dates. A ticket or doc written before the code may describe a plan
  that then changed; set it beside what shipped.
- A "why" field filled with boilerplate ("improve user experience") is not
  a reason, and a passing remark in chat is not a decision. Look for
  considered discussion and open the whole thread.
- Several drafts of one doc: take the finalized or latest one, and say
  which.
- A low event count may be heavy sampling, not rarity, so check how the
  source samples before you read a count as rare.
- A step in event volume may come from new logging, not new behavior, so
  check whether the instrumentation changed that day.
- Never report a result from a source you never confirmed exists. Open it
  first; a name you only guessed is UNSWEPT.
- A record that is gone — retention, a deleted thread, a page you cannot
  open, a renamed metric — is a gap, not an empty result. "Found nothing"
  is EMPTY; "could not look" is UNSWEPT.

## Provenance answer shape

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
- **Sources** — the seven category rows, per the report rules above, each
  as what was searched and then what was found, EMPTY, or UNSWEPT with its
  reason.
- **Confidence** — one line on the overall confidence.

When the why question comes before a change, end with a constraint set for
the plan: **Preserve** (what the change must keep), **Change** (what the
evidence lets move), **Avoid** (what the evidence warns against), and
**Risk** (what can break).

Before you send, run this self-check and fix what fails:

- Every claim under What we found has its source beside it, and you opened
  that source once to see it says what the claim says.
- Each claim's wording matches its tier.
- No claim uses the code as its own reason.
- The guess in the question, if any, was checked as a hypothesis.
- What we do not know names a gap, or says why there is none.
