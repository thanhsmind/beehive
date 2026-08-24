# Model role split — discovery map

## Destination

A locked decision on how many model roles bee configures, which ones a
dispatch can actually reach, and what a role entry may carry (fallback
chain, effort). Then one shaped feature that implements it.

Spawned: (not yet)

## Origin

Owner observation, 2026-08-24, from another tool's model-roles screen
(DEFAULT / SMOL / SLOW / VISION / PLAN / DESIGNER / COMMIT / TINY / TASK
/ ADVISOR, each with an ordered fallback list and its own effort level):
bee's `generation` and `extraction` are broad by comparison, and a finer
split would make configuration more dynamic and more efficient.

## Notes — reality at map time (2026-08-24, verified in code)

- **Four configurable slots, not two.** `CONFIGURABLE_SLOTS =
  ["extraction", "generation", "review"]` —
  `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:37`;
  `MODEL_NORMALIZE_SLOTS` adds `advisor` — `models.rs:40`. `ceiling` is a
  fifth pseudo-tier that means "the session model" and is deliberately
  never configurable (decision 0015) — `models.rs:324-326`.
- **Effort already exists.** `EFFORT_LEVELS = ["low","medium","high",
  "xhigh","max"]` — `models.rs:27`, carried on the `{model, effort}` leaf
  (`models.rs:167-181`) and the `{kind:'native', …}` leaf
  (`models.rs:86-97`). One scalar per entry.
- **Fallback already exists, but never as a chain.** Two single-step
  mechanisms only: the explicit-only composite
  `{primary, fallback_policy, fallback}` (`models.rs:134-166`, decision
  3ceba8f5 D2) and the herding slot's `fallback: "default"` flag
  (`models.rs:112-133`, decision 267192c1). No list-of-models anywhere.
- **The `extraction` slot is unreachable through the dispatch door.**
  `DISPATCH_KINDS = ["cell", "gather", "reviewer", "advisor"]` —
  `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:31`; and
  `slot_for_kind` maps `"cell" | "gather" => "generation"` —
  `prepare.rs:34-40`. The string `extraction` does not appear in
  `prepare.rs` at all. So a configured `extraction` model is dead
  config: the `bee-extract` agent's own tier has no `--kind` that
  selects it, and every gather runs on `generation`.
- **The guard's tier lists are asymmetric.** `CLAUDE_TIERS` has 4
  entries and omits `advisor`; `CODEX_TIERS` has 5 and includes it —
  `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:192-193`. Two
  hand-maintained lists, not one shared constant.
- **The config shape has two independent parsers.** `resolve_tier` in
  `models.rs:318-383` (over `Map<String, Value>`) and a second
  `resolve_tier` in `model_guard.rs:442-467` (over the guard's own
  structs). Every new role or entry shape has to land in both.

## Decisions so far

(none — nothing locked yet)

## Open shape

The observation that started this map points at *more roles*. The code
says the first defect is *a role nothing can select*. Adding roles on
top of the current door multiplies dead config rather than reducing it,
so ticket 001 comes before any decision on role count.

## Not yet specified

- Role count and names (ticket 002).
- Whether a role entry gains an ordered fallback chain, or the two
  existing single-step mechanisms stay as they are (ticket 003).
- Whether the two parsers and the two guard tier lists collapse into one
  source before the surface grows (ticket 004).

## Out of scope

- `ceiling` becoming configurable — settled by decision 0015 and not
  reopened here.
- Per-provider model catalogues or auto-selection. bee configures a
  model per role; it does not rank or discover models.

## Recorded deviation

The bee CLI is absent from this checkout (`.bee/bin/bee` holds only
`bee.pre-expertise.bak`), so this map was written directly rather than
through `bee` verbs, and no decision-log line was cut. Re-run the
wayfinding verbs against this file once the binary is restored.
