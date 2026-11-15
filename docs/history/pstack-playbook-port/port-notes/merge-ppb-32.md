# Port notes — pstack why merged into bee-why and the Provenance sweep, cell ppb-32

Source: pstack skill `why` (`SKILL.md` and its `references/`: epistemics,
investigator prompt, synthesizer prompt, source playbook index, the code
archaeology, issue tracker, long-form docs, team chat, observability, error
tracking and analytics playbooks, and the incident-postmortem angle), MIT,
copyright 2026 Lauren Tan. `skills/bee-why/` got `LICENSE.pstack`, a verbatim
copy of the pstack `LICENSE`. This file is the only place a pstack-only tool is
named.

Rule applied (D5): the bee skill stays bee's. The Provenance sweep stays
single-homed in `skills/bee-researching/references/trace-and-provenance.md`;
`bee-why` stays a door and restates no step. Every bee rule is kept. The
bee-why description is 289 characters, under the 300 budget.

## What came in

bee-why (the door):
- Description names a threshold as a kind of thing the skill answers for.
- Scope: the kind of question is pinned with the target, "as the sweep lists the
  kinds".

Provenance sweep (`trace-and-provenance.md`):
- Kind of question: design rationale, tradeoff, edge case, outside forcing
  function, survival (dead code, where no reason may survive) and history. The
  kind sets what a Direct source looks like and never removes a row.
- Anchor also takes ticket ids from commit and PR bodies, and every row searches
  them.
- Git history row: `git log -G` for a string that changed shape, read the diff as
  well as the message, the squash-merge fallback to the PR body and comments,
  CHANGELOG and release-note entries, files changed in the same commits.
- Code comments row: TODO, FIXME and HACK notes near the target.
- External tracker row: the `gh pr view --json` fields that hold the review
  threads and linked issues.
- Defensive code: a revert then re-apply and an incident-named commit as strong
  signals; read a postmortem whole with its action items; a postmortem naming the
  target is Direct, converging records that do not name it are Supported.
- Other record sources: when the session exposes tools for a chat history, an
  observability or error-tracking tool or an analytics store, each is an extra
  row under the same report rules; a tool needing a login you lack is UNSWEPT
  with the reason and yields no finding; skipping a row as irrelevant needs
  proof, not a hunch.
- Worker brief for a gathered row: gather evidence and never answer, wide net
  then narrow, read each PR or thread whole, quote verbatim with location,
  author and date, record the queries as run, record a cross-row reference as a
  lead and never chase it, and a return shape (searched, Direct, indirect with
  chain and other readings, contradictions, gaps, leads).
- Phrasing rules: evidence first and story second, the counterfactual question
  before concluding, the retrofit and copy-paste and silence warnings, never drop
  a hedge when passing a return on, and the words "obviously", "clearly", "of
  course", "just" and "I think" kept out.
- Reading a source: a signal near the ship date is suggestive and needs the
  neighboring commits checked, an instrumented or "resolved" mark is not proof
  of cause, check dates against the ship date, boilerplate "why" text and passing
  chat remarks are not reasons, and a record that is gone is a gap, EMPTY versus
  UNSWEPT.
- Answer shape: each Sources row reads as what was searched, then found, EMPTY or
  UNSWEPT with its reason.
- Self-check: each cited source was opened once and says what the claim says.

## Left out because bee already has it

- Evidence before narrative, no shortcut by code-reading, cite everything,
  prefer "appears to" over "because", surface contradictions, name the gaps,
  multiple hypotheses: the confidence tiers, the phrasing rules, Competing
  hypotheses and What we do not know.
- Direct, Supported, Inferred, Speculative and Unknown with their phrasing guide:
  the same five tiers in "Confidence tiers".
- The sycophancy trap and the guess inside the question: the phrasing rule on a
  guess as one hypothesis.
- Recency bias, trace back to the birth: row 2 reads back to the first commit.
- Cargo-culted patterns and bot commits: row 2 follows a copied pattern to its
  first appearance and skips bot commits.
- Evidence about a neighbor feature is not evidence about this one (the investigator
  "no silent substitutions"): the phrasing rule of the same words.
- Where to ask a human when a gap is open: What we do not know names who would
  likely know.
- Preserve, Change, Avoid and Risk after a why that precedes a change: the
  constraint set.
- Restating the question and the code in question: the "Question and anchor"
  section.
- The calibration check before finalizing: the self-check.
- Defensive code search terms (revert, hotfix, incident, the error string): kept
  as bee wrote them.
- Parallel investigators, one per category, always spawned in one message:
  bee's leader sweeps inline and fans only a row that returns more than a screen;
  that bee rule stays. The skip-inline exception for a trivial single-commit
  target is moot, since bee sweeps every row.
- The synthesizer subagent and "present it lightly edited": bee's leader writes
  the account, and a worker return is evidence, never pasted output.
- Subagent type, model and read-only flags and the configured why-investigator
  and why-synthesizer model names: bee's `bee dispatch prepare --kind gather`
  door returns them.
- The seven pstack MCP categories as the fixed roster, and MCP discovery from a
  Cursor `mcps/` directory: bee's seven rows differ (decision log, git history,
  feature history, knowledge bundle, code comments, tests, external tracker);
  discovery became the "other record sources" paragraph.

## Tool swaps

- "Spawn parallel investigators" with a hand-set subagent type and model becomes
  the existing gather door of Trace step 3, written as
  `bee dispatch prepare --runtime <rt> --kind gather --json` in that step; no
  model or subagent type is named.
- The source control investigator's `git log --follow -p` and `gh pr view` stay
  as plain git and `gh` commands, which a bee row already uses.

## Added, not in the source

- EMPTY versus UNSWEPT for a record that is gone, tying the pstack "retention
  cliff, gap not null result" pitfall to bee's two report words.
- A postmortem naming the target is Direct and converging records are Supported,
  mapping the pstack "especially strong when multiple sources corroborate" onto
  bee's tiers.

## Not ported

- The per-tool playbooks (issue tracker, long-form docs, team chat,
  observability, error tracking, analytics): no bee counterpart tool exists, so
  per D2 the tool-specific query recipes are dropped. Their tool-independent
  pitfalls came in under "Reading a source" and "Other record sources". Examples
  of dropped tool specifics: dbt refresh lag and `SHOW TABLES` probing, Sentry
  grouping drift and the vendor's own explainer, Datadog log aggregation calls.
- No bee skill or principle was missing for this merge.

## Regen note

`bee dev regen` also re-renders unrelated `.claude/skills/` entries in this
checkout, as in ppb-30 and ppb-31; those stay uncommitted. Only the `bee-why`
directory in each rendered tree, the `bee-researching` `trace-and-provenance.md`
copy, the render manifests, the onboarding hash and the release manifest are
committed.
