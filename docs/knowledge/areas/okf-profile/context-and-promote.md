---
type: bee.area
title: "Bee OKF Profile — the context consumer, the promote proposer, and the session preamble"
description: "The budget-aware manifest a work item's curated context is returned as, the measured relevance ranking that cuts critical patterns without losing one, the propose-never-write loop closer, the shared anchor resolver that lets both verbs work from a docs/history/ fallback when no work-item concept exists, the symptom-text search verb that pulls matching patterns and area concepts mid-flow, and the preamble that makes the bundle load-bearing — every section of which resolves through the one bundle predicate, including the critical-patterns digest's own relevance ranking."
timestamp: 2026-08-06
bee:
  id: okf-profile-context-and-promote
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md]
  decisions: [D2, D10, D12, D13, D27, D38, "G5/G11 (okf-switchover-f3 — critical patterns ranked, cut, floored and conserved)", F4-D1, F4-D2, F4-D3, i54-closeout D3, "knowledge-loop D1/D5/D6/D7/D8 (one shared anchor resolver; a history CONTEXT.md/plan.md fallback when no work-item concept matches; zero_signal reported not thrown under it; the fallback reaches context, promote, kctx and dispatch prepare)", "07dce495 (capture-reads-proposals: review-then-merge a promote proposal, never apply it as written; the area tag and the code it describes are both verified first, and the scribing stamp is owed even when nothing is kept)", "knowledge-search D1-D7 (symptom pull verb `bee knowledge search`: decisions-search grammar, patterns+areas corpus only, read-only at any phase from any checkout, skill wiring at the two debug moments — docs/history/knowledge-search/CONTEXT.md)"]
  sources: ["knowledge-search cells ks-1 (`bee knowledge search` verb; commit 448287b7, 2026-08-10) and ks-2 (skill wiring; commit f693f20f, 2026-08-10)", CONTEXT.md `docs/history/knowledge-search/CONTEXT.md`, "okf-foundation cell okf-9 (`bee knowledge promote` — the propose-never-write loop closer, B5; trace in `.bee/cells/`, 2026-07-22)", "okf-foundation cell okf-6 (critical-patterns.md -> patterns/ migration, work/okf-foundation/ work item + plan concepts, Templates section; trace in `.bee/cells/`, 2026-07-22)", CONTEXT.md `docs/history/okf-foundation/CONTEXT.md`, "docs/specs/okf-profile.md#B5", "docs/specs/okf-profile.md#B6", "docs/specs/okf-profile.md#B6b", "docs/specs/okf-profile.md#B7", "docs/specs/okf-profile.md#P2", "docs/specs/okf-profile.md#P3", "okf-integration-close-f4 cell f4-3 (the preamble stops printing the retired model — bundle-routed digest and project map, both no-bundle branches proven byte-identical; trace in `.bee/cells/`, 2026-07-22)", CONTEXT.md `docs/history/okf-integration-close-f4/CONTEXT.md`, red evidence `docs/history/okf-integration-close-f4/reports/red-preamble-before.md`, "i54-closeout cell i54-closeout-3 (knowledge context --lane budget presets; trace in .bee/cells/, 2026-07-24)", "knowledge-loop cell kl-1 (anchor.rs + context.rs/kctx.rs fallback, D5/D6/D7/D8; commit 1b2a8253, 2026-08-05)", "knowledge-loop cell kl-2 (promote.rs consumes the resolver; dispatch-prepare manifest proof; commit e6f99a7a, 2026-08-05)", CONTEXT.md `docs/history/knowledge-loop/CONTEXT.md`, "capture-reads-proposals cell crp-1 (the capture doctrine names the proposal as the first input of a compounding run and the scribing stamp as its receipt; skills/bee-capturing/SKILL.md, 2026-08-06)"]
  authoritative_for: "okf-profile: the context consumer, the promote proposer, and the session preamble"
---

# Bee OKF Profile — The Context Consumer, the Promote Proposer, and the Session Preamble

This concept owns the two verbs that read the bundle on a work item's behalf — `context`, which
returns an ordered manifest inside a budget, and `promote`, which proposes the knowledge finished
work earned and never writes it — plus the session preamble that makes the bundle load-bearing
rather than optional.

## Behaviors & Operations

**B9 — `context` and `promote` resolve their anchor through one shared resolver, and a work-item concept always wins (D1, D5, D6, D8).** Both verbs used to duplicate their own `--work` resolution — separately tagged `D27` in `context` and `D38` in `promote`, already drifted from each other — plus a third verbatim port copy in `drivers/kctx.rs`. All three now call one function, `resolve_anchor` in `verbs/knowledge/anchor.rs`:

- A `bee.work-item` concept whose `bee.id` matches the given work argument ALWAYS wins (D5). An existing work item is never displaced by the fallback below, and the two work items that already resolve this way today keep resolving exactly as they did before this feature.
- When no work-item concept matches, the anchor is the feature's own history directory: `docs/history/<id>/CONTEXT.md` and `docs/history/<id>/plan.md`, whichever of the two exist on disk — both together when both do (D1, D6). The resolver only reads; no work-item file is ever auto-created, and nothing under `docs/knowledge/work/` is created, moved, or deleted by either verb (D5).
- When no work-item concept matches and the feature has no history directory either, the anchor is the feature's most recent entry in the append-only ledger `.bee/logs/scribing-runs.jsonl`, with its meta and body built from the slug, the stamped area names, and the recorded `next_action` — all read from disk (decision 34ccf18d). This arm exists because lanes `small` and `tiny` deliberately produce no `docs/history/<slug>/` artifacts: a small-lane feature logs its scoping synthesis as a decision instead. Without it every small and tiny feature stays permanently unreachable. Everywhere past resolution the ledger arm behaves exactly like the history arm — `zero_signal` is reported rather than thrown, the anchor is the rank-1 manifest entry, and `areas_source` names the same stamp.
- When none of the three exists, both verbs keep the refusal they always had: a typed `unknown_work` error, byte for byte, still carrying its historical `(D27)` tag in `context` and `(D38)` tag in `promote` — the resolver changed how the anchor is found, never the shape of "found nothing."

Both verbs name what they resolved. The `--json` payload for each carries an `anchor` object (`kind`: `"work-item"`, `"history"`, or `"ledger"`, `paths`: the file or files behind it), and the plain-text output states the same in one line, so a caller can always tell which anchor answered the call and not only that one did.

`bee dispatch prepare` consumes the same resolver through the port copy in `drivers/kctx.rs` (D8): a dispatched worker prompt for a feature whose only anchor is its history directory now carries a learned-context manifest, where before the swallowed `unknown_work` refusal left it with none.

**B10 — `search` pulls bundle knowledge by symptom text, mid-flow, read-only (knowledge-search D1-D7).**
`bee knowledge search --text "<symptom>"` is the pull move for a session that hits an error text,
odd behavior, or unfamiliar mechanism mid-work — no work item, no anchor, no budget involved. Its
grammar deliberately mirrors `decisions search` (D1): `--text` required, whitespace-split terms,
case-insensitive OR; `--limit` optional, default 5 (D4). The corpus is the active bundle's
patterns and area concepts ONLY — generated `index.md`/`log.md` files are skipped, and
`docs/specs/` and the decisions corpus are never searched (decisions belong to `decisions search`,
D2). Matching runs over title, body, and the frontmatter `sources`/`decisions` entries; ranking is
deterministic — term-hit count descending, then file recency descending (D3) — and every result
row names the path, the title, and a why-matched line stating which terms hit in which field (D4).
Zero hits is an empty result with a one-line note, exit 0, never a typed refusal (D5). The verb is
read-only with `writes: []`, callable at any phase from any checkout (D7). The two debug moments
that invoke it are wired into the instruction layer (D6): the bee-swarming Execute worker contract
and bee-hive's scout — the wiring targets bee-swarming, not the retired bee-executing (site
erratum, D6). The session preamble's project map additionally carries one always-loaded pull line
naming the verb for mid-flow symptoms ("Hit a symptom mid-flow? Pull it: `bee knowledge search
--text …`"), so the retrieval move is taught before the symptom ever appears (knowledge-usable U1,
cell ku-1, 2026-08-10). The registry entry lives in the hand-maintained registry payload; no regen chain
exists for it — the gap is recorded in the ks-1 trace, with `tests/registry_contracts.rs` as the
drift net.

**B5 — `promote` proposes; it never writes (D38).** `bee knowledge promote --work <id>` resolves
the work item by `bee.id` (the same resolution `context` performs — an unresolvable id exits 1 with
a typed `unknown_work` error), then mines the **capped** cells of that feature from `.bee/cells/`
and returns exactly three sections:

| Section | What it proposes | Where every line comes from |
|---|---|---|
| **(a) Delivery draft** | A complete `bee.delivery` concept **in canonical emitter form**, ready to be saved as the work item's `delivery.md` sibling: what shipped, how it was verified, every recorded deviation. Because it is emitted through `emitFrontmatter`, saving it produces zero `not_canonical` findings. | Each cell's `trace.outcome`, `verify` command and `trace.verification_evidence`; the work item's own title, tags, `bee.decisions`, `bee.areas`, `bee.lane`. |
| **(b) Area updates** | For each area named in the work item's `bee.areas`, the capped **`behavior_change`** cells whose `files_changed` touch that area's subject — its concepts' own paths and their `bee.sources` — as candidate spec-sync bullets, each citing its cell id. | `trace.files_changed` matched against bundle concept paths and `bee.sources`. |
| **(c) Pattern candidates** | Every capped cell whose trace carries a **deviation** or a **failure signature**, shaped as a candidate `bee.pattern` concept with `bee.polarity: pitfall` and `bee.lifecycle: draft`, quoting the trace verbatim. A clean cell yields nothing. | `trace.deviations`, `trace.attempts[].failure_signature`, `trace.semantic_judge[].failure_signature`. |

The `--json` payload is `{work, work_item, cells, delivery, area_updates, pattern_candidates,
writes}`, and **`writes` is always `[]`** — the machine-readable form of the contract. There is no
`--apply` flag and no write path of any kind: `promote` never touches `docs/knowledge/`, never
touches `.bee/*.json(l)`, and never touches anything else. Deciding to save a proposal — and
editing it into curated prose first — is a human or agent decision.

Under a history anchor (B9), the proposed delivery save path in section (a) is still the canonical
`docs/knowledge/work/<id>/delivery.md` — the draft names where a human or agent would save it, never
where `promote` itself writes. `writes` stays `[]` on every anchor kind, always: D38 is unchanged by
the fallback. An empty `bee.areas` list (no work-item concept means none to read) keeps the existing
"None: the work item declares no bee.areas…" render for section (b).

**B6 — `context` returns a manifest, never content (D27).** `bee knowledge context --work <id>
--budget <tokens>` resolves the work item by `bee.id`, walks its `bee.required_context`
**transitively** with a cycle guard that dedupes silently (a cycle is never an error), adds every
concept with `bee.critical: true` and the bundle's `bee.decision` concepts whose `bee.areas`
overlap the work item's, ranks them, and cuts at the budget. The order is fixed: the work item, its
`bee.plan` sibling, `required_context` in BFS depth order, critical patterns, then area decisions.
Each entry carries `path`, `bytes`, `est_tokens` and a one-line `reason` naming *why* it was
selected (and, for a required_context hit, *through which parent*) — and **nothing else**: the
manifest never contains file bodies, because its whole purpose is to spend a few dozen tokens
instead of thousands. The budget cut is a **prefix cut** with one named exception (B6b): the first
overshooting entry ends the manifest, and it plus every lower-ranked entry is named in `truncated`,
so the output always means "the highest-ranked context that fits". The estimator is `bytes/4` and
the output **names itself as an estimate** — bee vendors no tokenizer (D12), so the number is never
dressed up as a token count. An unresolvable id exits 1 with a typed `unknown_work` error.

Under a history anchor (B9), the anchor itself takes the work item's rank-1 slot, sized from the
real byte count of the history files behind it — `bee.plan` sibling and `required_context` steps are
simply empty and skipped, never panicking on a path the bundle does not own.

**B6b — critical patterns are ranked by relevance, cut, floored and conserved (G5/G11).** D27's
original "include every critical pattern" rule was written when three patterns existed. At 49 it
inverted: on the first real run, 40 of 45 manifest entries were critical patterns consuming 13,000
of 19,726 tokens, most of them unrelated, with 7 more truncated for lack of room — so an irrelevant
pattern could evict a relevant one, and the consumer built to stop context waste had become its
largest source.

The replacement ranks the critical concepts against the work item and cuts them to
`CRITICAL_RELEVANCE.KEEP`. **The relevance signal was chosen by measurement, not intuition.** Tag
overlap — the obvious candidate — is disqualified: measured against the live bundle it left 48 of
49 patterns tied at zero (AUC 0.550 against hand labels; `bee.areas` overlap 0.500, i.e. a coin
flip). The shipped signal is the **IDF-weighted fraction of a concept's own distinctive vocabulary
that the work item's text covers**, scored over two fields (title/description/tags, and body), plus
a small tag and area bonus: AUC 0.805, no ties, no zeros. IDF is computed over the ranked population
itself, so no word list ships. Widening the query with the `required_context` bodies was measured
and **rejected** — it dilutes the work item's own vocabulary (AUC 0.751 → 0.615).

Three properties make the cut safe to trust:

- **Floor.** The top `CRITICAL_RELEVANCE.FLOOR` criticals have their cost reserved out of the budget
  remaining after rank 1, so a genuinely universal lesson is never evicted by a long
  `required_context` chain — while the work item itself is never displaced by its own floor. The
  budget stays a hard ceiling: `total_est` never exceeds it, and a zero budget still includes
  nothing.
- **Conservation.** Every `bee.critical` concept is accounted for exactly once — in `entries` (whose
  `reason` names its score and rank), in `truncated`, or in `excluded` as `{path, score, reason}`.
  `critical_total` states the population and the assembler *throws* rather than lose one. A silent
  exclusion is worse than the noise it replaces: the failure being fixed was loud, and it must not
  be traded for a quiet one where a pattern that would have prevented a bug is simply absent.
- **Zero-signal guard.** `zero_signal_count` is always reported. When the population is at least
  `ZERO_SIGNAL_MIN_POPULATION` and more than `ZERO_SIGNAL_MAX_RATIO` of it scores zero, the run
  **fails** with a typed `zero_signal` error. A ranking where most items tie at zero is a path sort
  wearing a relevance label, and shipping it green is the defect — the guard exists so a future
  signal cannot rot into one silently. Below that population the count is reported but not enforced:
  a two-concept bundle is not a ranking problem. Under a history anchor (B9, D7), this guard
  **reports instead of failing**: `score_critical_relevance` also weights the work concept's own
  `tags` and `bee.areas` (`TAG_WEIGHT`/`AREA_WEIGHT`), fields a history anchor has none of, so more
  criticals land at exactly 0.0 for a reason that is an artifact of the anchor kind rather than a
  real relevance failure. The population-size floor above still governs whether the count is
  enforced at all. A work-item anchor still throws exactly as it always has.

Ties break by path, so the order is total and two runs over the same bundle are byte-identical.

**`context` accepts a lane shorthand that resolves to a budget preset before the
explicit flag is validated (i54-closeout D3).** `--lane tiny|small|standard|
high-risk` maps to a fixed budget preset (8000 / 12000 / 20000 / 30000 tokens
respectively) resolved before the generic `--budget` flag validator runs, so an
explicit `--budget` always wins when both are given, and a bare call with
neither flag refuses exactly as it did before the shorthand existed — nothing
about the default (20000, unmapped mode falls back to it) changes. The session
preamble's own recommended `context` command picks its `--budget` from the
active work item's mode through this same shared preset table, rather than
hardcoding one number for every lane. The budget-cut semantics themselves — the
prefix cut, the critical-patterns floor and conservation exception (B6b) — are
unaffected; the shorthand only changes how the number arrives, never what
happens once it does.

**B7 — The session preamble makes the bundle load-bearing.** A tool nobody calls is a directory
rename. When `.bee/state.json`'s active feature has a matching `bee.work-item` concept, the session
preamble emits a three-line block naming the exact runnable `context` command and instructing the
session to read the manifest's files before touching code. Three rules keep it honest: the preamble
carries the **pointer, never the manifest** (embedding it would defeat the purpose); a feature with
no matching work item produces **silence, not a nag**; and a terminal phase (`idle`,
`compounding-complete`) produces nothing even when a stale `feature` string outlives the closed
feature — the phase, not the feature name, decides.

**B8 — Every section of the preamble that describes the state layer resolves through the same
bundle predicate.** The preamble is what every agent reads before doing anything, so a section of it
that names the retired model teaches that model to every session, silently and forever, with no
check ever going red. Three sections therefore branch on the one predicate, resolved once per build
and handed to each:

- **The project map** names the bundle as the thing to read before the code and states what the
  bundle holds — the number of areas and concepts, derived from the single inventory walk, never a
  second directory scan and never a hand-maintained list. The compatibility surface is named for
  what it is, never as "specced areas" to read before the code.
- **The critical-patterns digest ranks by relevance to the bound feature, not by recency
  (knowledge-loop D3, cell kl-4).** The "a preamble has no work item to rank against" limit above no
  longer holds: the digest resolves the bound feature's own anchor through the same shared resolver
  (B9) and scores the bundle's critical rows with `context.rs`'s own IDF ranker (B6b) — reading
  **only** the concept bodies the index's `## Critical patterns` rows name, never `collect_concepts`,
  which would put roughly 1.41 MB of parsing (264 files) on every session start against today's one
  file, 22,692 bytes. A row whose link target resolves to no file on disk is dropped, and the dropped
  count is named in the header, so a stale `index.md` degrades visibly instead of silently. The
  header always states which mode produced the rows below it: ranked by relevance to the named
  feature, or a recency fallback with its own reason — no feature bound, no anchor resolved, or
  nothing left to score — so a reader never mistakes one for the other.
- **The scribing-debt nudge** names the resolved target, bundle or compatibility surface, rather
  than hardcoding one.

Two rules keep this honest. **The no-bundle branch of all three is byte-identical** to the behavior
of a repo that never migrated — proven both by a permanent bundle-less fixture pinning each line as
an exact literal and by a whole-preamble byte comparison, because a host that never migrated must
not be able to tell the migration happened. And **degradation is silence, never a fallback**: in
bundle mode with no generated index, or an index carrying no critical section, the digest emits
nothing. Falling back to the retired file would re-print a forwarding address as if it were
lessons — which is the exact defect this behavior exists to remove. Orientation never fails a
session either: an unreadable predicate resolves to the legacy branch rather than throwing.

## Business Rules

- **`promote` proposes; it never writes (D38).** Finished work is allowed to *suggest* knowledge —
  a delivery draft, area bullets, pitfall candidates — and is never allowed to *commit* it. No
  section `promote` emits is written to disk by `promote`; accepting one is a human or agent
  decision, and `writes: []` in the payload states that in machine-readable form. The reason is the
  same one behind D10's never-invent rule: a proposal that writes itself into the bundle arrives
  reading as curated truth and is then trusted, without anyone having judged it.
- `promote` invents nothing: every proposed line is copied from a capped cell trace or from the
  work item concept (D10). A cell that was never capped, and a cell belonging to another feature,
  are never mined.
- **A work-item concept always outranks the history fallback, and the fallback never authors a
  file (D1, D5, D6).** `resolve_anchor` (B9) picks a matching `bee.work-item` concept first, every
  time; only its absence opens the `docs/history/<id>/CONTEXT.md` / `plan.md` fallback, and reading
  those files never creates, moves, or deletes anything under `docs/knowledge/work/`. `unknown_work`
  survives, byte for byte, when neither anchor exists.

- **`promote` reaches retired cells, and takes its areas from the scribing ledger when the work
  item names none (decision 86d96c9f).** Mining reads `.bee/cells/*.json` AND
  `.bee/cells/archive/<feature>/*.json`, deduped by cell id with the live copy winning, so a
  feature whose cells already retired still proposes — before this, every closed feature mined
  zero cells and therefore proposed nothing. When the resolved work item declares no `bee.areas`
  — which is every feature reached through the history fallback — the area list comes from that
  feature's most recent entry in the append-only ledger `.bee/logs/scribing-runs.jsonl`. The
  payload and the text render both NAME the source as `areas_source`: `{kind: work_item}` or
  `{kind: scribing_ledger, ts}`. When neither source yields an area, the existing no-areas render
  stands byte for byte. The area list can come from nowhere else: `bee.authoritative_for` is
  prose, only 5 of 95 area concepts carry a code path in `bee.sources`, and only 10 of 95 mention
  `packages/` at all — the bundle is deliberately technology-agnostic, so no file-to-area mapping
  can be derived from it. The scribing stamp is the one place that mapping is already asserted.

- **The session preamble invites retrieval for every anchorable feature, and names an unapplied
  proposal as open work (D1, D2, D3 of knowledge-in-flow).** The preamble's knowledge block used
  to open only when a `bee.work-item` concept existed, so 162 of the 164 features the resolver can
  now anchor were never told the retrieval command existed, and it printed advice to author the
  very file D5 made unnecessary. It now resolves through the same three-arm resolver, prints the
  runnable `bee knowledge context` command whenever ANY arm answers, names which arm answered, and
  prints nothing at all when none does. Both that block and the critical-pattern digest anchor on
  the SESSION'S ACTIVE feature — the bound lane when there is one — never the default record
  unconditionally; a live preamble was measured ranking against a feature closed hours earlier.
  Separately, `bee orient` and the preamble both name every `docs/history/<slug>/promote-proposals.md`
  that no compounding run has yet answered, with its feature and counts. That surfacing is a
  REPORT: it refuses nothing, blocks nothing, and changes no exit code.

- **The step that answers a proposal is doctrine now, not folklore (capture-reads-proposals,
  cell crp-1, 2026-08-06).** The loop had three of its four parts built and nothing telling a
  worker the third one existed: a proposal is mined automatically at close, and a scribing stamp
  at or after the proposal's own timestamp is what retires the reminder — but no instruction text
  anywhere named the file, so a compounding run mined the raw traces again from scratch. The
  capture doctrine now names it as the FIRST input of a compounding run, with two guards that come
  from answering fifteen open proposals by hand: the area a proposal names comes from the work
  item and is routinely over-broad or simply wrong (one named an area that does not exist in the
  bundle), and a proposal can faithfully describe code that a later port has since retired. So the
  rule is review-then-merge, never apply-as-written, and the stamp is owed either way — including
  when the honest answer is that nothing in the proposal was worth keeping.

## Pointers (implementation)

- Proposal builder (B5): `buildPromotion` in the same module, with `readCappedCellTraces` as its
  read-only view of `.bee/cells/` and `.bee/cells/archive/<feature>/`. Neither function writes; the CLI handler
  (`handleKnowledgePromote` in `.bee/bin/bee`) only prints what they return.
- CLI wiring: `.bee/bin/lib/command-registry.mjs` (the `knowledge` group) +
  `.bee/bin/bee` dispatch (`HANDLERS`).
- Shared anchor resolver (B9): `resolve_anchor` in
  `packages/bee-rs/crates/bee/src/verbs/knowledge/anchor.rs`; consumed by `context.rs`, `promote.rs`,
  and the port copy in `drivers/kctx.rs` (reached by `bee dispatch prepare` via `drivers/prepare.rs`).
  Evidence: trace `.bee/cells/kl-1.json`, commit `1b2a8253`; trace `.bee/cells/kl-2.json`, commit
  `e6f99a7a`.
- Digest relevance ranking (B8, D3): `bundle_critical_patterns_digest` in
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs`, reusing
  `score_critical_relevance` from `verbs/knowledge/context.rs` against the B9 anchor. Evidence:
  trace `.bee/cells/kl-4.json`, commit `d74ca11c`.
