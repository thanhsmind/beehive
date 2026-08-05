---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Knowledge Loop

Mode: `standard` — 3 risk flags: public-contracts, multi-domain, covered-contract-change
Why this is the least workflow that protects the work: four public CLI/preamble surfaces
change behavior that existing tests pin, across four modules, but no data, auth, or
external system is touched — a phase plan with per-phase proof is enough.

Revision 2 — redrafted after the pre-gate review wave returned 9 findings (4 HIGH).
Three of them became locked decisions D6, D7, D8 rather than plan prose, because they
change what the feature promises, not how it is built.

> Route record deviation (named, per AGENTS.md § Judgment): `bee state route --set`
> writes only to the ACTIVE feature's tracked record and accepts no lane-targeting flag,
> and this session is not bound to the `knowledge-loop` lane. The classification above IS
> the route record for this feature; the default record was restored to doc-viewer-links'
> own route.

## Requirements (from CONTEXT.md and the shaping decision log)

- **D1**: `knowledge context --work <id>` and `knowledge promote --work <id>` accept a
  feature slug with no `bee.work-item` concept, falling back to a `docs/history/<slug>/`
  anchor. Output NAMES the anchor used. No work-item file is auto-created. `unknown_work`
  survives only when neither a concept nor any history anchor exists.
- **D2**: `bee close` runs `promote` on the green path after the scribing-debt door,
  prints a headline, writes `docs/history/<slug>/promote-proposals.md`. SOFT door — never
  refuses close, never writes into `docs/knowledge/` (D38 preserved).
- **D3**: the session-preamble critical-pattern digest ranks by relevance to the bound
  feature via the `context.rs` IDF ranker against the D1 anchor; falls back to recency
  with a header that says so.
- **D4**: P68 (`knowledge stale` / `knowledge links`, lifecycle retirement) is out.
- **D5**: no `docs/knowledge/work/` file is created, moved, or deleted; an existing work
  item always wins over the fallback.
- **D6**: the history anchor is `CONTEXT.md` OR `plan.md` — whichever exists, both when
  both do. Measured: of 150 `docs/history/` dirs, 86 have `CONTEXT.md`, 36 have only
  `plan.md`, 28 have neither and keep refusing `unknown_work`.
- **D7**: under a history anchor, `context.rs` REPORTS `zero_signal` instead of throwing
  it. The typed error stays live for a work-item anchor.
- **D8**: the fallback lands on FOUR surfaces — `context.rs`, `promote.rs`, the port copy
  `drivers/kctx.rs`, and `drivers/prepare.rs` (which already passes a feature slug as
  `work` and swallows the refusal).

## Discovery

- `--work` resolution is duplicated THREE times, not two: `context.rs:229-237` (error
  tagged D27), `promote.rs:367-374` (tagged D38), and a verbatim port copy at
  `drivers/kctx.rs:705-729` (same predicate, same D27 message), wired at
  `drivers/mod.rs:217` and consumed at `drivers/prepare.rs:275`. A shared resolver is a
  deduplication, not a new layer.
- The byte-golden `learned_context_agrees_with_the_knowledge_verb_port`
  (`drivers/tests.rs:881-905`) calls `kctx::build_context_manifest`, NOT `context.rs`'s.
  Changing one copy and not the other drifts them with every test green — the exact
  invariant that golden exists to hold. This is why D8 puts kctx in the same cell.
- `drivers/prepare.rs:266-279` already passes the cell's `feature` slug as `work` and maps
  `ManifestOut::Thrown(_) => None`, so today every dispatched worker prompt for the ~39
  orphan features ships with no learned-context manifest, silently.
- The scorer's input is NOT just two strings. `score_critical_relevance`
  (`context.rs:117-201`) also reads the work concept's `tags` (`:146`) and `bee.areas`
  (`:154`) with `TAG_WEIGHT = AREA_WEIGHT = 0.05` (`:32-34`). Under a history anchor both
  sets are empty, more criticals score exactly 0.0, and `context.rs:302-310` would throw
  `zero_signal` — hence D7.
- `select` (`context.rs:243-250`) returns early for any path not in `by_path`, which is
  built from bundle concepts only (`:241-242`). A `docs/history/...` path is therefore
  DROPPED, not resized, and `rank_one_cost` (`:403`) silently becomes the top critical
  instead of the anchor. Sizing goes through `metadata(join_rel(dir, rel))` (`:395`) with
  `dir = docs/knowledge`, so a history path measures 0 bytes. Phase 1 has to size and rank
  the anchor explicitly.
- `promote.rs:386-391` derives the proposed delivery save path from
  `dir_of(&work_concept.path)`, and `promote.rs:676` writes `work_item` into the payload
  from the same path. Both need a defined value under the history arm.
- `build_promotion` is `pub(crate) fn build_promotion(root: &Path, dir: &Path, work: &str)
  -> Option<Promo>` (`promote.rs:359`), re-exported crate-wide (`knowledge/mod.rs:106`) —
  callable in-process from close.rs, as `build_close_report_doors` (`close.rs:541`) and
  `archive_feature_for_close` (`close.rs:774`) already are. Its `None` arm means
  "delegate to Node", and `rg -l buildContextManifest --glob '*.mjs'` returns nothing —
  there is no Node left. Phase 3 maps `None` and `Thrown` to the same warning line.
- No text-write helper exists: `fsutil.rs` exposes `write_json_atomic` and `append_jsonl`
  only, and `close.rs:7` imports just those. Phase 3 adds `write_text_atomic` beside them.
- Nothing forbids the D2 write: `docs_history_code_deny`
  (`hooks/write_guard/guards.rs:66-82`) denies 22 code extensions, not `.md`;
  `plan_freeze_feature` (`hooks/write_guard/checks.rs:159-166`) matches only
  `docs/history/<feature>/plan.md`; and the write guard is a PreToolUse hook on the
  agent's tools, never on bee's own in-process writes.
- Preamble cost, counted on this bundle: today the digest reads **1 file, 22,692 bytes**
  (`docs/knowledge/index.md`, `budget.rs:128`). A naive ranker call would read **264 files,
  ~1,407,407 bytes** per session start — `collect_concepts` (`walk.rs:291-308`) parses all
  186 bundle concepts (1,241,918 bytes) and then `concept_body` re-reads all 78 criticals
  (165,489 bytes). There is NO time budget anywhere under `hooks/session_preamble/`; the
  only guard is the 5120-byte payload cap (`budget.rs:43`). `build_session_preamble` has
  one caller, `hooks/session_init.rs:196`, so the cost is per session start.

Evidence commands already run:
- `bee knowledge context --work okf-foundation --budget 20000 --json`
  → entries 26, truncated 0, excluded 55, critical_total 78, zero_signal 0.
- `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
  → 1232 passed, 3 ignored (13 suites, 11.57s) — the green base this feature starts from.

## Approach

**Recommended path.** One shared resolver; four consumers, phased.

A new `verbs/knowledge/anchor.rs` exposes:

```
pub(crate) enum Anchor<'a> {
    WorkItem(&'a Concept),
    History { work: String, paths: Vec<String>, meta: String, body: String, bytes: u64 },
}
pub(crate) fn resolve_anchor<'a>(root, dir, concepts, work) -> Option<Anchor<'a>>
```

Resolution order is D5 then D1/D6: a `bee.work-item` whose `bee.id` matches always wins;
otherwise `docs/history/<work>/CONTEXT.md` and `plan.md`, whichever exist; otherwise `None`,
which each caller renders as today's `unknown_work` message verbatim. `meta` under the
history arm is the slug plus each file's first heading — read from disk, never composed, so
D10 holds. `bytes` is summed from the real files so the anchor can be ranked and sized.

Consumers:
- `context.rs` — anchor becomes rank 1 with `reason: "anchor (history fallback)"` and its
  real byte size; `rank_one_cost` (`:403`) reads that size, so the floor reservation keeps
  today's meaning. `zero_signal` is reported, not thrown, on the history arm (D7). The
  `bee.plan` sibling, `required_context`, and `decisions` stay empty — all three already
  degrade without panicking.
- `drivers/kctx.rs` — same edit, same cell, so `drivers/tests.rs:881-905` keeps meaning
  what it says (D8).
- `promote.rs` — under the history arm the proposed delivery save path is the canonical
  `docs/knowledge/work/<slug>/delivery.md` (a PROPOSAL; nothing is written, so D5 holds),
  and `work_item` in the payload carries the anchor paths. Empty `bee.areas` keeps its
  existing "None: the work item declares no bee.areas…" render (`promote.rs:714-720`).
- `drivers/prepare.rs` — no code change beyond inheriting kctx's fix; the proof is a test
  that a dispatched cell for a slug with only `docs/history/<slug>/CONTEXT.md` now carries
  a manifest instead of `None`.

Phase 4 avoids the 1.41 MB walk: the digest already reads `index.md`, whose
`## Critical patterns` rows carry each concept's path in the link target. Parse those rows
to paths, read ONLY those 78 bodies (165 KB), and rank. That keeps the row→concept join —
which the digest needs anyway to render a row per ranked concept — and skips
`collect_concepts` entirely. A row whose link target resolves to no file is dropped with
the count named in the header, so a stale `index.md` degrades visibly.

**Rejected alternatives.**
- Auto-writing a work-item stub per feature — rejected in shaping; fabricates prose (D10).
- Fixing `context.rs` alone — leaves the kctx golden meaningless and leaves dispatched
  workers without a manifest (D8).
- Ranking the digest over `index.md` gloss text only — cheaper, but the measured ranker
  (AUC 0.805) is body-based; changing its input silently changes what it was validated on.
- Calling `collect_concepts` from the preamble — 1.24 MB of parsing per session start for
  data the digest does not use.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| `anchor.rs` resolution order | LOW | a test proving an existing work item beats a present CONTEXT.md (D5) |
| `context.rs` + `kctx.rs` parity | HIGH | `learned_context_agrees_with_the_knowledge_verb_port` stays green AND a new test fails if only one copy is edited |
| anchor as rank 1 (size, `rank_one_cost`) | MEDIUM | a test pinning `total_est`, `floor`, and the anchor entry's non-zero `bytes` under the history arm |
| `zero_signal` under a history anchor | MEDIUM | a test that a short CONTEXT.md REPORTS `zero_signal` and exits 0, while a work-item anchor still throws |
| `promote.rs` proposed paths | LOW | a test pinning `work_item` and the delivery save path under the history arm |
| `close.rs` soft door | MEDIUM | tests that a promote `Thrown` AND a promote `None` each leave close's exit code unchanged and touch nothing under `docs/knowledge/` |
| `budget.rs` digest cost | HIGH | a measured file-count and byte-count assertion, plus the 5120-byte cap test staying green |
| stale `index.md` rows | MEDIUM | a test that an unresolvable row is dropped and counted in the header |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | `anchor.rs`; `context.rs` AND `drivers/kctx.rs` consume it in the same cell; `anchor` in the manifest header and text output; `zero_signal` reported on the history arm | The resolver is the feature's spine, and the two manifest copies must never move apart | `bee knowledge context --work exec-speed --budget 20000` returns a ranked manifest naming its CONTEXT.md anchor; the kctx golden still passes | Phases 2-4 |
| 2 | `promote.rs` consumes the resolver; `anchor` and defined `work_item` / delivery path in the payload; `prepare.rs` manifest proof | promote is what close needs; prepare's proof rides here because it consumes phase 1's kctx | `bee knowledge promote --work exec-speed` proposes off cell traces; a dispatched cell for an anchor-only feature carries a manifest | Phase 3 |
| 3 | `write_text_atomic` in `fsutil.rs`; close soft door: run promote, print headline, write `docs/history/<slug>/promote-proposals.md`; `None` and `Thrown` both degrade to one warning line | The forcing function P67 asked for; it can only exist once promote resolves any slug | `bee close --feature <f>` on a green feature emits the headline and the file, exit 0 either way | The knowledge write loop closes |
| 4 | `budget.rs` digest ranks by relevance off index rows + 78 critical bodies, with a named fallback header | Independent of 2-3; last because its cost evidence may send it back to recency | A session preamble whose three digest lines change when the bound feature changes | — |

Slice queue: 1 → 2 → 3 strictly ordered. 4 depends only on phase 1.
Current slice to prepare: **phase 1**.

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges existing coverage
first and authors only the gap. Existing coverage, corrected against the source:
`unknown_work` is pinned at `verbs/knowledge/tests.rs:312` (context) and `:505` (promote) —
NOT by the six manifest tests; those are `context_manifest_orders_and_cuts` (`:235`),
`promotion_mines_capped_traces_and_proposes_without_writing` (`:402`),
`context_conserves_the_critical_set_at_every_budget` (`:1120`),
`context_floor_keeps_the_top_criticals…` (`:1173`),
`context_zero_signal_fails_above_the_population_floor…` (`:1272`),
`context_relevance_ties_break_deterministically_by_path` (`:1335`).
The kctx golden is `drivers/tests.rs:881-905`; the preamble cap test is
`hooks/session_preamble/tests.rs:280`, and the current "3 most recent" digest is asserted
at `tests.rs:434` and `:440-441`.

- **Happy path** — a slug with no work-item concept but a history anchor resolves, ranks,
  and reports `anchor.kind == "history"` with a non-zero anchor byte size; a slug WITH a
  work item still reports `anchor.kind == "work-item"` and today's byte-identical manifest.
- **Edge cases** — `CONTEXT.md` only; `plan.md` only (D6); both present; a work item whose
  directory has no `bee.plan` sibling; a history anchor short enough to drive most criticals
  to 0.0 (must report, not throw — D7); the digest with no bound feature; an `index.md`
  critical row whose target file is gone.
- **Error paths** — neither a work-item concept nor any history file: the existing
  `unknown_work` message and exit code unchanged in BOTH verbs and in kctx; a promote
  `Thrown` and a promote `None` each leave close's exit code and every `docs/knowledge/`
  file untouched.

`commands.test` (`cargo test --release --manifest-path packages/bee-rs/Cargo.toml`) runs at
every cell cap, at close, at merge, and in CI.

## Out of scope

- PBI P68: `bee knowledge stale`, `bee knowledge links`, and any `bee.lifecycle`
  transition (D4).
- Re-grading the 78 `critical: true` patterns (D3 makes it non-urgent, not solved).
- Creating, moving, or backfilling any `docs/knowledge/work/` file (D5).
- The 28 `docs/history/` dirs with neither `CONTEXT.md` nor `plan.md` — they keep refusing
  `unknown_work` by D6's letter.
- The 20 orphaned scribing-debt cells across 5 features — this feature gives them a
  reachable promote path; running it for them is separate work.
