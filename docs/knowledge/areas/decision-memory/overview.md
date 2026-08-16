---
type: bee.area
title: Decision Memory — what the system remembers about its own decisions
description: "How a decision event is classified, reversed and reconciled against its citing artifacts, recalled through a derived index, kept bounded by an explicit archive, and honored by a backlog row's own done-flip rule — all one topic at this source's size."
timestamp: 2026-08-16
bee:
  id: decision-memory-overview
  lifecycle: active
  areas: [decision-memory]
  decisions: ["decision-propagation GH #32/#33/#34 (2026-07-21)", D1 b9b9fee3 (backlog CoS-gated done-flip), D2 b9b9fee3 (reversal citation sweep), D3 b9b9fee3 (citation discipline), "D4c b9b9fee3 (bounded store, archive verb)", "D5 b9b9fee3 (no stored graph, no daemon)", D6 b9b9fee3 (reversals inherit place), D7 c81c6795 (write-time classification + retro-tag reclassification), D8 1cea7713 (derived index recall surface), "D11b (scribing-skill copy of the done-flip rule, since consolidated into bee-capturing)", "compounding-skill fallback (identical, never-looser; same consolidation)", "supersession-is-an-edge 252102b5 (2026-08-08, decision-supersede-hygiene dsh-1 d2c0a33e)"]
  sources: ["docs/specs/decision-memory.md#R1", "docs/specs/decision-memory.md#R2", "docs/specs/decision-memory.md#R3", "docs/specs/decision-memory.md#R4", "docs/specs/decision-memory.md#R5", "docs/specs/decision-memory.md#R6", "docs/specs/decision-memory.md#R7", "docs/specs/decision-memory.md#R8", "docs/specs/decision-memory.md#R9", docs/history/decision-propagation/reports/e2e-supersede.md, test_decisions_propagation.mjs (84 checks incl. worker-thread log-vs-archive race), "backfill: 406/406 legacy events classified via extraction batches; --untagged --all returns zero; 5-event recall spot check green", "judge-record-tags cells jrt-1, jrt-2 (five internal callers swept; the census derives its sites by scanning source, and was itself widened by measurement after its first scope hid a live instance; traces in `.bee/cells/`, 2026-07-23)", "dsh-1-supersedes-edge (capped, d2c0a33e; packages/bee-rs/crates/bee/src/verbs/decisions/read.rs, verbs_read.rs, tests.rs)"]
  authoritative_for: "decision-memory: what the system remembers about its own decisions"
---

# Decision Memory (what the system remembers about its own decisions)

What the system remembers about its own decisions, how a reversal propagates to
everything that cited the old truth, and how any session finds the decisions
that matter without re-deriving dead conclusions. At this source's size (nine
rules, no Behaviors/Edge Cases/Pointers sections) the area is one coherent
topic rather than several — every rule below governs the same store, the same
index, and the same reversal path, so splitting it further would shred it past
usefulness rather than clarify it.

## The problem this area solves

Three field failures (reported against a host repo, fixed generically):

1. A reversed decision lived only in the log; the artifacts sessions actually
   read (tickets, backlog rows, specs) still stated the old conclusion — every
   new session re-derived it (#33).
2. A backlog row flipped `done` when a feature merely *matched* it, never
   checking the row's Conditions of Satisfaction — partial delivery silently
   read as full (#34).
3. The decision store outgrew substring grep: no classification, no
   completeness guarantee on recall (#32).

## Business Rules

- **R1 — Every decision event is classified at write time** (D7, `c81c6795`). A
  canonical taxonomy (`docs/decisions/taxonomy.json`, entries `{name,
  description}`) governs tags. Once the taxonomy exists, an untagged
  decide/supersede is refused with a typed error; before it exists (bootstrap),
  untagged writes warn and proceed. An unknown tag is never refused — it is
  accepted onto the event and appended to `candidates[]` awaiting curation.
  `candidates[]` never holds an already-canonical tag.
- **R1a — The write-time refusal binds the system's OWN callers, and they are
  swept by a derived census.** R1 governs every writer, not only the ones a human
  invokes: the system logs decisions to itself when it resets a stale claim,
  overrides a judge verdict, resets a cell's budget, reopens a cell on a
  rework verdict, and records the audit line that is the price of the
  scribing-debt waiver. Each of those was written before R1 existed and carried
  no tags, so once a taxonomy was present each one hit R1's refusal. The failure
  modes differed and both are worth knowing: where the call sits inside the
  operation's own write path, the refusal unwinds the whole operation, so the
  work fails; where it sits inside a best-effort catch, the operation succeeds
  and the audit line vanishes **silently** — the more dangerous of the two,
  because nothing surfaces. A rework verdict was unrecordable for exactly this
  reason and had to be written by hand.

  The rule this leaves behind is not "remember to tag": a cross-cutting
  write-time refusal is not finished until the callers already in the tree are
  swept, and the sweep is held by a check that **derives** its call sites by
  scanning source rather than listing them. A hand-maintained list is what let
  the first sweep miss a caller — the census's own scope was set from assumption,
  passed green, and hid a live instance until it was widened by measurement.
  Two properties keep it honest: a caller that legitimately forwards
  a *user's* tags is not an offender and must never be flagged, or an author is
  pushed to fabricate a value that is not theirs to choose; and the check is
  proven by injection — a tagless call added on purpose must name its own file
  and line.
- **R2 — Reversal is not finished until citing artifacts are reconciled** (D2,
  `b9b9fee3`). A supersede computes a citation sweep over `docs/**` (full id +
  word-boundary short8) BEFORE its single append; the event carries the sweep
  result. Every hit is reconciled same-turn or explicitly waived with a
  recorded reason; each hit also becomes a capture stub so an unreconciled
  citation resurfaces at every flush. Historical records (reports) are
  reconciled by appended dated correction notes, never by rewriting history.
  `decisions log --relation touches:<id>[,...]` runs the same sweep at log
  time, once per touched id (doc-impact-synthesis D1, touches `c48a9b0d`):
  every hit under `docs/**` becomes a `source: "touches-sweep"` capture stub
  EXCEPT the generated `docs/decisions/index.md` (regenerated by `decisions
  render`, never hand-fixed) and, when the logging context has a bound
  feature, that feature's own live `docs/history/<feature>/` dir
  (self-citation of live work is not staleness) — a different feature's
  history is still a real citation and is never excluded.
- **R2a — Supersession is an edge, never prose, and every write now declares
  its relation to what's already active** (`252102b5`; knowledge-distill-trigger
  D3, cell kdt-3). Every `decisions log` call requires `--relation
  supersedes:<id>[,...]|touches:<id>[,...]|none`; a missing or malformed value
  refuses the write outright (`RELATION_REQUIRED_MESSAGE`), and the refusal
  names up to 3 conflict candidates from the same area/tags so the fix command
  is ready-made (dcc-1). `supersedes:` ids resolve against the currently
  ACTIVE decide/supersede set only (an already-superseded or already-redacted
  target cannot be named again); `supersedes:` and the dedicated `decisions
  supersede` verb (R2's own path) land the same shape — the retiring event
  carries a `supersedes` field (an array for `--relation supersedes:`, a bare
  string for `decisions supersede`) — and `active_decisions()` excludes every
  named target on ANY event type, not only `type=="supersede"`. `touches:` ids
  resolve the same way but persist onto their own `touches` array; a touched
  id stays active, unlike a superseded one. Decision text that reads as an
  inline supersession claim — the word stem "supersed(e/es/ed)", "replaces",
  "overrides", "no longer applies", "instead of the previous" — is refused
  unless `--relation supersedes:<id>` resolves it (`--relation none` or
  `touches:` never silences this guard): free text was previously the only
  way to hide a supersession from the active set, silently, and the store
  audit that triggered this rule found 70 decide events doing exactly that
  against 29 proper supersede events. Decision text that reads as
  postponement — the guard's own stem/phrase list (`matches_deferral_prose`)
  — is refused unless `--trigger <id>` names an already-registered trigger id
  (`bee triggers add --decision <id> --condition "..."` registers one
  first): no postponed condition may exist outside the trigger registry.
  The same list, unchanged, gates postponement-shaped prose written straight
  into a closing feature's own touched docs (doc-impact-synthesis D3) — the
  check that gave decision text a way in gives doc prose the identical one.
  `active_decisions()` —
  the projection behind R5's derived index — stays the single read surface
  for current truth; the append-only log itself is never re-read for "is
  this still true," only for "why."
- **R2a — Postponed conditions live in a two-tier trigger registry**
  (knowledge-distill-trigger D2, kdt-2). One record per trigger in the
  shared control-root store (`.bee/triggers/<slug>__<short8>.json`), each
  citing the decision that named it. Predicate-tier records
  (`path-exists:`/`path-missing:`) are re-evaluated on every registry
  read; a predicate that has come true flips the record `waiting → due`
  and the flip persists. Manual-tier records never auto-fire — they
  surface as awaiting confirmation until a human resolves them.
  `triggers resolve` writes the outcome onto the record only; any
  follow-up decision is logged separately under R2's rules. Orientation
  surfaces the registry as one line — due count plus
  awaiting-confirmation count — so a registered condition can wait but
  never sink; an unreadable record degrades to a visible
  delete-the-file remedy line, never a silent skip. A due trigger routes
  its work through the backlog; the registry never executes anything.
- **R3 — Reversals inherit their place** (D6, `b9b9fee3`). A supersede without
  explicit tags/scope inherits both from the (overlay-applied) decision it
  supersedes, so the reversal is discoverable exactly where the original
  lived.
- **R4 — Memory is re-classifiable without rewriting history** (D7, `c81c6795`).
  Retro-tag events (`decisions tag`, single or batch) are append-only; reads
  apply a latest-wins overlay (tags replace, scope only when carried). No
  stored line is ever edited.
- **R5 — The derived index is the recall surface** (D8, `1cea7713`).
  `docs/decisions/index.md` is regenerated, never hand-edited, grouped scope →
  first tag, superseded events excluded, byte-stable for the same store,
  complete by construction. Reading order per area: spec → decision index
  section → history. Search offers structured filters (`--tag`,
  `--scope`/`--area`, `--since`, `--untagged`, `--all`) and multi-term OR
  ranking; bare substring grep is fallback, never the recall path.
- **R6 — The store stays bounded** (D4c, `b9b9fee3`). An explicit archive verb
  moves superseded/redacted and aged-out events (explicit cutoff, never a
  default purge) to an archive file; union reads (`--all`) reach both and
  de-duplicate by id (active copy wins — which also self-heals a crash between
  the two archive writes). All store writers share one lock; append-only
  integrity is absolute.
- **R7 — A backlog row flips `done` only when every CoS clause has cited
  evidence** (D1, `b9b9fee3`; the skill-side rule text formerly lived in the
  scribing skill's D11b and the compounding skill's identical, never-looser
  fallback — both since consolidated into `bee-capturing`, which carries no
  separate copy, so this spec and the decision record are now the rule text's
  home). Partial delivery keeps
  the row `in-flight` with a `Delivered:`/`Remaining:` annotation; splitting
  the remainder into a new row is allowed when the delivered subset ships
  alone; silent full-flip never.
- **R8 — Citation discipline** (D3, `b9b9fee3`). An artifact that encodes a
  decision cites its short8 id — that is what makes R2's sweep able to reach
  it. Uncited embodiments are the accepted residual risk.
- **R9 — No stored graph, no daemon** (D5, `b9b9fee3`). All consistency is
  derived at read/mutation time; a second source of truth is exactly the
  failure mode this area exists to kill.

## Data dictionary

- **Decision event** — append-only record: `decide` (id, date, decision,
  rationale, alternatives, scope, source, confidence, tags[], a required
  `--relation` declared as either an optional `supersedes` array from
  `decisions log --relation supersedes:<id>[,...]`, an optional `touches`
  array from `--relation touches:<id>[,...]`, or neither for `--relation
  none`; plus an optional `trigger` id from `--trigger <id>` on a
  deferral-shaped decision), `supersede` (adds `supersedes` as a single
  string, `sweep`), `redact`, `tag` (target, tags[], scope?). A `supersedes`
  field excludes its targets from `active_decisions()` on any event type,
  string or array alike (R2a); a `touches` field never excludes its targets.
- **Feature** — an optional `feature` field on a newly logged `decide` event
  (doc-impact-synthesis D1), stamped from the calling context's active
  feature — a session-bound lane, else the default `.bee/state.json` record,
  the same resolution `state route` uses — only when one resolves; absent
  otherwise, and legacy (pre-D1) lines and every reader tolerate that
  absence.
- **Taxonomy** — `docs/decisions/taxonomy.json`: `tags[] {name, description}`
  (canonical, human-curated) + `candidates[]` (strings awaiting promotion;
  CLI-appended).
- **Scope** — the area dimension (spec-area slug; legacy default `repo`).
- **Sweep** — `{scanned_at, hit_count, files[]}` recorded on the supersede
  event.
- **Index** — `docs/decisions/index.md`, provenance-headed, timestamp-free
  body; `--check` mode exits non-zero on drift.
- **Delivered subset** — the evidenced portion of a row's CoS at a refused
  flip (R7 annotation).

## Proven behavior (evidence anchors)

- Live e2e (2026-07-21): supersede of `d20f4c96` → `257ab1e5` — sweep 2 hits, 1
  reconciled, 1 waived with reason, stubs created/flushed, index self-corrected.
  `docs/history/decision-propagation/reports/e2e-supersede.md`.
- Backfill: 406/406 legacy events classified via extraction batches;
  `--untagged --all` returns zero; 5-event recall spot check green.
- Store/CLI behavior: suite `test_decisions_propagation.mjs` (84 checks incl.
  worker-thread log-vs-archive race) + full verify.

## Actors & Access

- **A session** — writes decision events (decide/supersede/redact/tag), reads
  the derived index before re-deriving a conclusion, and reconciles any
  citation hit a supersede's sweep surfaces.
- **The taxonomy** — `docs/decisions/taxonomy.json` — governs which tags are
  canonical; an unknown tag is accepted onto an event and appended to
  `candidates[]` awaiting curation, never refused.
- **bee-capturing** — successor of the scribing and compounding skills, which
  held the backlog done-flip rule text (R7) as their own, identical,
  never-looser fallback; the consolidated skill carries no separate copy, and
  R7 above (with decision D1 `b9b9fee3`) is the rule text's home.
- **The archive** — receives superseded/redacted and aged-out events at an
  explicit cutoff; union reads (`--all`) reach both the active store and the
  archive and de-duplicate by id.
