# Lane Model Diversity — Context

**Feature slug:** lane-model-diversity
**Date:** 2026-08-29
**Shaping session:** complete
**Scope:** Standard
**Domain types:** CALL | ORGANIZE

## Feature Boundary

Blind-lane and hat-wave advisor dispatches stop sharing the single `advisor`
model slot: each seat gets its own role name in the existing
`models.<runtime>` table so each parallel brainstorm agent can run a
different model. The feature ends at role resolution + config validation +
the two procedure docs; it does not touch the blind-lane/hat-wave
procedures themselves, the dossier format, or `bee blind check`.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Store provenance: decision chain `9a9a03b1` → `c27ec06e` →
**`23de5362`** (final, 2026-08-29), touching `4a6e38be` (pi model routing).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Seat roles join the one `models.<runtime>` open table: lanes `lane-1`, `lane-2`, `lane-3`; hats `hat-facts-gaps`, `hat-risks`, `hat-value`, `hat-alternatives`, `hat-user-impact`. No separate config file, no new config home. | Store `23de5362`; keeps the one-parser rule (`cd72ec97`). |
| D2 | An unconfigured seat role falls through to `advisor`. For `--kind advisor` dispatches, an explicit `--role <seat>` therefore resolves the ordered list `[<seat>, advisor]` — this supersedes T012a's refuse-by-name FOR ADVISOR-KIND DISPATCHES ONLY; every other kind keeps T012a's refusal unchanged. | User approved fall-through explicitly ("ghế chưa khai thì rơi về advisor"). Ordered-list-with-tail is existing law (`561e1bda`); advisor's own one-name walk (`4faf1de9`) stays for the tail itself. |
| D3 | Each `hat-*` slot is REQUIRED to carry a `description` stating the hat's purpose — the config must read self-documenting. `bee config validate` enforces it; `lane-*` slots need no description (interchangeable seats, no per-seat purpose). | Store `23de5362`/`c27ec06e` rationale. |
| D4 | No dispatch-time model flag, ever: the model comes only from the table. The dispatch door's marker/guard machinery treats seat roles like any other configured role — the `[bee-tier: …]` marker names the RESOLVED role (seat when configured, `advisor` when fallen through). | Model-guard law unchanged (`3c9d6262` resolved-name marker). |
| D5 | The two procedure homes name the seat roles: `gates-and-delegation.md` §"Blind lanes and convergence" (fan-out move names `--role lane-N` per lane) and §"Hat wave" (each hat row names its role). One-line pointers elsewhere stay pointers. | Single-home rule for both procedures. |

### Agent's Discretion

Naming/wiring internals: where the seat-role list constant lives, test file
placement, and whether `default_models` seeds the seat names (it must NOT
seed models for them — absent seat = fall-through to advisor, per D2 an
absent seat may not resolve a built-in default).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| seat | One parallel advisor position in a blind-lane run (lane-1..3) or hat wave (5 hats) |
| seat role | The `models.<runtime>` role name a seat resolves its model through |
| fall-through | An unconfigured seat role resolving the `advisor` slot instead of refusing |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:544` — `resolve_role_named`: the ordered-list walk with warn-and-fall-through; D2's `[<seat>, advisor]` is one more caller of this exact walk.
- `models.rs:672` — `resolve_advisor`: the one-name advisor walk (decision `4faf1de9`); stays as the tail resolution.
- `models.rs:646` — `tier_role_list`: the single home for tier-shaped ordered lists (`[review, generation]` precedent) — the seat list belongs beside it.

### Established Patterns

- Open role set + refuse/fall-through semantics — model-role-split `06e49368`/`561e1bda`/`3c9d6262`.
- Config validation rules — `bee config validate` (D3's enforcement point).

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1149` — T012a `--role` refusal arm: D2 branches it by kind (advisor-kind walks `[seat, advisor]`).
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:273` — comment declaring advisor "deliberately NOT here"; updates with D2.
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs` — marker classification against `known_roles`; must accept configured seat names (D4).
- `skills/bee-hive/references/gates-and-delegation.md:152` — both procedure sections (D5).

## Canonical References

- Decision store ids: `23de5362` (final table), `4a6e38be` (one-config-home law), `4faf1de9` (advisor one-name walk), `561e1bda` (ordered tail), `3c9d6262` (resolved-name marker), T012a (`--role` refusal).
- docs/history/research/pi-harness-support.md — companion research; `models.pi` will carry the same seat roles when pi-support lands.

## Outstanding Questions

<!-- bee:not-a-deferral: both questions were answered during planning (plan.md Discovery: no --role/--brief-file interaction per reviewer P3-1; config validate is unported, D3 venue moved to doctor by decision) — this section records the shaping→planning handoff, it promises no future work -->
### Deferred To Planning

- [x] Whether `--role` + `--brief-file` interact anywhere in prepare's argument validation beyond the resolution walk — ANSWERED: none, `resolve_brief_file` never reads `role` (plan.md Discovery).
- [x] Where `bee config validate` hangs slot-level rules today — ANSWERED: nowhere, the config verbs are unported; D3's venue moved to `bee doctor` (venue decision).
<!-- /bee:not-a-deferral -->

<!-- bee:not-a-deferral: this section's one idea is routed to the pi-support feature and already recorded there in the decision store (7f9c8518, 4a6e38be) — the pointer documents the routing, it is not an untracked promise -->
## Deferred Ideas

- `models.pi` runtime block with these same seat roles — belongs to the `pi-support` feature (decisions `7f9c8518`, `4a6e38be`, model table settled 2026-08-29).
<!-- /bee:not-a-deferral -->

## Handoff Note

<!-- bee:not-a-deferral: template boilerplate describing how planning consumes this record — machinery description, not a promise to act later -->
CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and reviewing
use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
