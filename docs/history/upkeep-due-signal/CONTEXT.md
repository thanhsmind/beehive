# Upkeep due signal — Context

**Feature slug:** upkeep-due-signal
**Date:** 2026-11-14
**Scope:** Standard
**Domain types:** RUN

## What was asked

The owner asked whether knowledge upkeep runs on its own. It does not: both
upkeep skills carry `disable-model-invocation: true`, and no hook, timer,
trigger or bee verb says a run is due. The owner then asked that knowledge
upkeep and verify upkeep give a signal, so the owner can set up a periodic
waggledance check that starts the upkeep run.

## What will be done

bee gets one read-only verb, `bee upkeep due`, that says per kind (knowledge,
verify) whether a run is due and why, as JSON and as an exit code. Each upkeep
run records itself in the audit log, so a clean run clears the signal. The two
upkeep skills name the verb and the record step, and the okf-profile concept
tells how a scheduler starts the run through waggledance.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | New read-only verb `bee upkeep due`. `--json` prints schema `bee-upkeep-due/v1`: `any_due`, plus one item per kind (`knowledge`, `verify`) with `skill`, `invoke` (`/bee-knowledge-upkeep` or `/bee-verify-upkeep`), `present`, `due`, `reasons` [{code, detail}], `last_run` {at, outcome, note} or null. `--kind knowledge\|verify` narrows to one item; `--root` reads another store as `bee knowledge report --root` does. It writes nothing. | e05b2de3 |
| D2 | Exit 0 when nothing is due, 10 when any reported item is due; any other non-zero exit is an error. JSON prints in both cases. | a69de382 |
| D3 | Each upkeep run records its last run as one audit row: `bee audit log --kind knowledge-upkeep` or `--kind verify-upkeep`, note starting with the outcome word (`clean`, `changed`, `blocked`). The closed audit kind set gains these two kinds. The newest row of each kind is `last_run`. Both skills gain this as their final step on every outcome. | d4fc0607 |
| D4 | Knowledge reasons: `never_run`, `release_since_last_run` (newest `v*` tag by creator date newer than `last_run.at`), `reported_wrong` (a concept reported wrong by a cell capped after `last_run.at`; any when never run), `orphan` (freshness orphan count above 0), `suspect_loaded` (a suspect concept loaded 5 or more times). Loaded-never-used goes under `info` and never makes the run due alone. No bundle: `present` false, `due` false. | 5ced4e9d |
| D5 | Verify reasons: `never_run` (`.bee/verify/verify-app/SKILL.md` present, no `verify-upkeep` row) and `release_since_last_run`. No SKILL.md: `present` false, `due` false. | d489fc4b |
| D6 | Both skills keep `disable-model-invocation: true`. A scheduled waggledance task sends the `invoke` slash command as the task text to the project leader. The how-to lives in `docs/knowledge/areas/okf-profile/overview.md`. | f88a51a3 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/project/overview.md | confirmed | where verbs, the router and the release script live |
| docs/knowledge/areas/okf-profile/overview.md | suspect | names the upkeep loop; home of the scheduled-upkeep how-to (D6) |
| docs/knowledge/areas/okf-profile/freshness-and-document-homes.md | confirmed | the freshness grades (orphan, suspect) the knowledge reasons read (D4) |
| docs/knowledge/areas/decision-memory/audit-log.md | suspect | the audit log and its closed kind set that gains two kinds (D3) |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/audit.rs` | the closed audit kind set |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs` | `knowledge_wrong` and `knowledge_used.loaded`, and `--root` reading |
| `packages/bee-rs/crates/bee/src/verbs/tmp_group.rs` | the small verb-group shape the new verb mirrors |
| `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | the verb registry the help and the CLI-shape guard read |

## Out of scope

The scheduler itself. waggledance has no schedule verb (`waggledance --help`
lists none); the owner sets up the periodic check. bee only gives the signal.
