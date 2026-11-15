# ppb-32c: check of the pstack why merge in ppb-32

Checked against commit `8db3596ee` (ppb-32). The source is the pstack skill
`why` at `/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/why/`:
`SKILL.md`, `references/epistemics.md`, `investigator-prompt.md`,
`synthesizer-prompt.md`, `source-playbook.md` and the eight files under
`references/sources/` (`code-archaeology`, `incident-postmortem`, `linear`,
`notion`, `slack`, `datadog`, `sentry`, `databricks`). The bee side is
`skills/bee-why/SKILL.md` and `skills/bee-researching/references/trace-and-provenance.md`
("Provenance sweep", "Confidence tiers", "Provenance answer shape"). No file
other than this report was edited.

## Verdict

The merge holds. Almost every source rule is present in the Provenance sweep or
in the bee-why door. The rules that are left out each name a bee home or a D2
drop, and the port notes list them correctly. Seven small points are missing.
None changes the shape of the answer. Most of them are reading cautions for the
extra rows a session may expose (chat, observability, error tracking,
analytics).

- **pstack-only tool names:** none. A case-insensitive scan of both bee files
  for `cursor`, `graphite`, `bugbot`, `poteto`, `mcp`, `subagent_type`,
  `generalPurpose`, `grok`, `readonly`, `linear`, `notion`, `slack`, `datadog`,
  `sentry`, `databricks`, `seer`, `dbt`, `fable`, `sonnet`, `opus` and `haiku`
  matches one line only: `trace-and-provenance.md:28` "Never hand-pick a
  `subagent_type`". That line is an older bee rule in Trace that forbids the
  pstack pattern, and it is not in the ppb-32 diff. Tool names appear only in
  `port-notes/merge-ppb-32.md`, as the notes say.
- **bee rules removed:** none. Every `-` line in the diff to the two files comes
  back in a `+` line with its original words kept and new words added: the
  description (adds "threshold"), the Scope pin (adds the kind of question), the
  Anchor (adds ticket ids), the git history row, the code comments row, the
  external tracker row (adds `--json` fields), the defensive paragraph, the
  Sources line and the first self-check item.
- **Other prohibitions:** the bee-why description is 289 characters, under 300.
  `skills/bee-why/LICENSE.pstack` is byte-identical to the pstack `LICENSE`. The
  source `trace-and-provenance.md`, the bee-why body and `LICENSE.pstack` match
  in all five rendered trees (`.claude-plugin`, `.codex-plugin`, `.agents`,
  `.opencode`, `.claude`).

## SKILL.md

| Source rule | Status | Where in bee |
|---|---|---|
| Investigate motivation: why built so, edge cases, constraints, rejected alternatives | present | sweep "Kind of question"; door description |
| Companion split: how answers what code does, why answers what forces shaped it | present | bee-why description and Scope ("The how part becomes ... `/bee-how`") |
| Seven evidence categories, enumerated at run time, queried all, default is coverage | left out, bee home | bee's own seven rows; extra tools become extra rows ("When this session exposes tools ...") |
| Null results are first-class evidence, reported beside positive findings | present | Report rules: EMPTY by name, seven rows in, seven out |
| Posture: evidence before narrative | present | phrasing rule "Collect the evidence first" |
| Posture: precision over polish, exact quote and citation | present | worker brief "Quote verbatim ... location, author, date"; tier 1 cites beside |
| Posture: consider what you haven't seen (counterfactual) | present | phrasing rule "ask what you would expect to see if another reading were true" |
| Posture: name the gaps | present | What we do not know; self-check last item |
| Posture: hedge on purpose | present | Confidence tiers; phrasing rule on hedged words |
| Posture: no shortcut by code-reading | present | phrasing rule "The code is never evidence of its own intent" |
| Epistemics: cite everything, uncited is inference | present | tier 1–3 phrasing; self-check first item |
| Prefer "appears to" over "because" | present | phrasing rule on "because" beside a citation |
| Surface contradictions | present | phrasing rule "When two sources disagree, report both" |
| Acknowledge gaps; honest "could not find out" | present | tier 5 Unknown |
| Multiple hypotheses are valid | present | Competing hypotheses section |
| Beware rationalization | present | phrasing rule "Code that makes sense today ..." |
| Step 1 question kinds (rationale, tradeoff, edge case, forcing function, dead code, history) | present | "Kind of question" bullet |
| Step 1 vague target: best guess from context, state it, proceed | present | "When the target is vague, write your reading ... Never stop to ask." |
| Step 2 code anchor: paths, lines, symbols, blame, commits, PR numbers | present | Anchor bullet |
| Step 2 ticket ids from commit and PR bodies | present | Anchor bullet (added by ppb-32) |
| Step 2 `git log --follow -p`, `gh pr view --json ...` | present | rows 2 and 7 |
| Step 2 pass seed context so investigators don't rediscover | present | "fan that row alone ... briefed with the Anchor" |
| Step 3 MCP discovery from Cursor `mcps/` and category mapping | left out, D2 | "other record sources" paragraph; tool named only in port notes |
| Step 3 one investigator per category, all in one message, never collapsed | left out, bee home | bee leader sweeps inline and fans only a row over a screen (port notes) |
| Step 3 subagent type, model, readonly flags | left out, bee home | Trace step 3 door `bee dispatch prepare --kind gather` |
| Step 3 append incident-postmortem when target is defensive | present | defensive paragraph |
| Roster: seven categories and what each surfaces | left out, bee home / D2 | bee's seven rows; tool rows via "other record sources" |
| Skip a category only with written reason: no tool (gap) or provably irrelevant | present | "A tool that needs a login you lack ... UNSWEPT"; "Skipping a row as irrelevant needs proof" |
| Trivial single-commit target may answer inline after confirming | left out, bee home | moot: bee sweeps every row (port notes) |
| Step 4 synthesizer subagent with findings, anchor, epistemics | left out, bee home | leader writes the account ("write the account yourself") |
| Step 5 present, never rewrite confidence language | present | phrasing rule "Never drop a hedge ... when you pass a worker's return on" |
| Output format: Question, Code, Found, Infer, Hypotheses, Don't Know, Sources | present | Provenance answer shape |
| Sources line form: searched, found / no results / skipped with reason | present | Sources bullet (extended by ppb-32) |
| Preserve / Change / Avoid / Risk when the why precedes a change | present | constraint set; door Scope |
| Failure modes: confident storytelling, code as own intent, recency bias, sycophancy, skipped gaps, skipping by anticipation, collapsing investigators | present (last: left out, bee home) | tiers, phrasing rules, row 2 "read back to the first commit", Report rules; collapsing is bee's inline sweep |

## references/epistemics.md

| Source rule | Status | Where in bee |
|---|---|---|
| Five tiers Direct, Supported, Inferred, Speculative, Unknown with phrasing | present | Confidence tiers |
| Unknown names what was searched and the terms | present | tier 5; What we do not know |
| Confidence words only beside a citation | present | phrasing rule |
| Hedge words for inferences | present | tier 3 |
| Words to avoid: obviously, clearly, of course, just, I think | present | phrasing rule (added by ppb-32) |
| Avoid rationalization: right-thing retrofit, copy-paste pattern, absence of evidence | present | phrasing rule (added by ppb-32) |
| Sycophancy trap | present | phrasing rule on the guess in the question; door Scope; self-check |
| When evidence contradicts, show both | present | phrasing rule |
| Missing evidence: name the person to ask | present | What we do not know "who would likely know" |
| Calibration check: citation, tier phrasing, code-as-intent, gaps present | present | self-check |

## references/investigator-prompt.md

| Source rule | Status | Where in bee |
|---|---|---|
| Gather evidence, never answer the question | present | worker brief first line |
| Quote, don't paraphrase where wording matters | present | worker brief |
| Go wide before deep | present | worker brief |
| Track what you searched, record queries | present | worker brief |
| Resist the story; the contradiction is the finding | present | phrasing rules; return "contradictions" |
| Consider the counterfactual | present | phrasing rule |
| Never invent; label a partial finding partial | present | tiers; "never write a finding from a row you could not read" |
| Read the whole PR, ticket, doc or thread | present | worker brief "read each PR or thread whole" |
| Follow links inside the assigned source (PR to PR, ticket to parent or sibling, doc to doc) | **missing** | brief covers cross-row leads but says nothing about chasing in-row links |
| Cross-source reference is a lead, never chased | present | worker brief |
| Note absences | present | worker brief; Report rules |
| Mechanics vs motivation; no intent from code style | present | phrasing rule on code as own intent |
| No silent substitutions | present | phrasing rule "Evidence about a neighbor feature" |
| Return shape: searched, Direct, indirect with chain and alternatives, contradictions, gaps, leads | present | worker brief Return |

## references/synthesizer-prompt.md

| Source rule | Status | Where in bee |
|---|---|---|
| Six key rules (tiers, citations, hedges, code-as-intent, gaps, embedded hypothesis) | present | Confidence tiers and phrasing rules |
| Reconcile overlapping findings: one PR cited by several rows becomes one reference | **missing** | no rule; leader-written account makes it likely but unstated |
| Spot-check citations before passing them on | present | self-check "you opened that source once" |
| Don't overreach; leave an open question open | present | tier 5; What we do not know |
| Tagged bullets `[Direct]` / `[Supported]` / `[Inferred]` | present | "each with its tier and its source" |
| Confidence summary | present | Confidence line |
| Quality check incl. contradictions surfaced and overall tone calibrated | present | phrasing rules; self-check |
| Final note: honesty over authority | present | phrasing rule "It reports the record; it is not an opinion" |

## references/sources/

| Source rule | Status | Where in bee |
|---|---|---|
| code-archaeology: `git log -S`, `-G`, `--follow`, blame, `git show` | present | row 2; Anchor |
| code-archaeology: `gh pr view --json` reviews and comments | present | row 7 |
| code-archaeology: ADRs | present | rows 1 and 3 (decision log, feature history) |
| code-archaeology: TODO, FIXME, HACK, XXX, NOTE near target | present (XXX and NOTE not named) | row 5 |
| code-archaeology: tests encode edge cases | present | row 6 |
| code-archaeology: co-change files, CHANGELOG | present | row 2 (added by ppb-32) |
| code-archaeology: squash-merge, misleading message, cargo-cult, bot commits | present | row 2 |
| incident-postmortem: hunt incidents when defensive; revert then re-apply; read postmortem whole with action items; corroboration is strong | present | defensive paragraph |
| incident-postmortem: defensive list incl. rate limit, egress guard, OOM handler | present (partial list) | defensive paragraph names guard, retry, timeout, sleep, flag |
| incident-postmortem: per-tool incident queries (`#sev-*`, `incident` labels) | left out, D2 | port notes "Not ported" |
| linear / notion / slack / datadog / sentry / databricks query recipes | left out, D2 | port notes "Not ported" |
| linear: scope drift (closed, reopened), closed-as-duplicate chains to the canonical ticket | **missing** | row 7 has no "follow the duplicate and reopen history" line |
| linear / notion: boilerplate "why"; stale or outdated doc, check dates; doc vs reality | present | "Reading a source" |
| linear / notion / slack: private or restricted or unauthenticated = gap | present | UNSWEPT with reason |
| notion: multiple drafts, take the finalized or latest one | **missing** | none |
| notion: unlinked docs, broad keyword search | present | "Cast a wide net first" |
| slack: retention cliff; jokes are not decisions; fetch whole threads | present | "Reading a source" |
| slack: DMs are unsearchable, a known limit | present | "could not look" is UNSWEPT |
| datadog: correlation not causation; overfitting a chart; instrumented ≠ caused; vanished telemetry | present | "Reading a source" |
| datadog: noise at scale, narrow and aggregate | left out, D2 | tool-specific (`analyze_datadog_logs`); fan-out rule covers volume |
| sentry: release correlation noisy; resolved ≠ fixed | present | "Reading a source" |
| sentry: silent fix, the error stopped because upstream changed | present (partial) | "Other changes may sit in the same window" covers repo changes, not upstream ones |
| sentry: grouping drift; Seer explanations | left out, D2 | port notes name both as dropped tool specifics |
| sentry: sampling, a low count may be heavy sampling | **missing** | none |
| databricks: instrumented ≠ caused; retention cliff; notebooks not queryable = gap | present | "Reading a source"; UNSWEPT |
| databricks: silent instrumentation change, a step in volume may be new logging | **missing** | none |
| databricks: never report from a table you never confirmed exists | **missing** (general form) | port notes drop `SHOW TABLES` as tool specific; the tool-free rule did not come in |
| databricks: schema drift, dbt refresh lag | left out, D2 | port notes "Not ported" |

## Missing, in one list

1. Follow links inside a row (PR to PR, ticket to parent or sibling), the
   in-row counterpart of "a cross-row reference is a lead".
2. Reconcile one record cited by several rows into one reference.
3. Ticket scope drift and closed-as-duplicate chains back to the canonical
   ticket (row 7).
4. Several drafts of one doc: use the finalized or latest one.
5. Sampling: a low event count may reflect sampling, not rarity.
6. A step in event volume may be new instrumentation, not new behavior.
7. Never report a result from a source whose existence you did not confirm.

Each is one line in the sweep's worker brief or under "Reading a source". None
is a pstack-only tool name, and none breaks a bee rule.

## Found outside the check

Commit `8db3596ee` replaced `.claude/skills/bee-why/SKILL.md`, a Skillstation hub
alias that `e8d3a2ab1` put there on purpose, with the full rendered skill, and
re-added `.claude/skills/bee-why/CREATION-LOG.md` that `e8d3a2ab1` deleted.
ppb-31 did the same to `.claude/skills/bee-how/`. The port notes say other
`.claude/skills/` re-renders were left uncommitted, so committing this one looks
unintended. Whether `.claude/skills/` should hold the hub alias or the render is
a call for the owner.
