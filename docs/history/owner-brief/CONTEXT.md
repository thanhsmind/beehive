# Owner brief — Context

**Feature slug:** owner-brief
**Date:** 2026-10-28
**Shaping session:** complete (Qualify of a foreign-origin spec drop, sup-20261028-c9d4 from waggledance@09d94dd)
**Scope:** Standard
**Domain types:** CALL | READ | ORGANIZE

## Feature Boundary

Every new backlog item and every shaped feature is born with a decision-ready owner brief at `docs/briefs/<id>.md`; bee validates the brief at the door that creates it, lists the owner view with in-flight items folded into their feature, and flags every live item or active feature whose brief is missing or malformed. It ends there: no backfill of existing items, no LLM call on read, no staleness digest.

## Locked Decisions

<!-- bee:not-a-deferral: locked decisions about how briefs record deferrals; the one moved-out item is backlog p-0b45c28d -->
| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The brief contract is byte-compatible with the spec: path `docs/briefs/<id>.md` (`<id>` = PBI id or feature slug); flat `key: value` frontmatter between `---` lines with `title` (≤80 characters, owner language, states the outcome), `audience` (`owner`\|`internal`), `size` (`S`\|`M`\|`L`), `ask` (`build`\|`park`\|`drop`\|`answer`\|`test`); a body of exactly six paragraphs, each opening with its bold label, 120–220 words in total. | waggledance `/api work-items` and agent-works `mirror.mjs` already parse this. |
| D2 | The body carries no ids, file paths or function names. The validator refuses a PBI-id token (`p-` + 8 hex), a token holding `/` or ending in a file extension (`.rs`, `.md`, `.json`, `.js`, `.mjs`, `.ts`, `.py`, `.sh`, `.toml`, `.yml`, `.yaml`), backticked code, and an `identifier(` call shape. | Consumers add the technical footer themselves. |
| D3 | Owner language is configured by `brief.language` in `.bee/config.json`: `vi` (the default when absent) or `en`. `vi` labels are exactly `Là gì:`, `Vì sao:`, `Nguồn gốc:`, `Chi phí và rủi ro:`, `Anh cần làm:`, `Chuẩn xong:`. `en` labels are `What:`, `Why:`, `Origin:`, `Cost and risk:`, `Your call:`, `Done when:`. Frontmatter keys and enum values stay English in both. An unknown language value validates with the `vi` set and the check names the unknown value. | `vi` is what consumers parse today. |
| D4 | `bee backlog pbi add` and `bee backlog propose` take `--brief-file <path>`. The file is validated against D1–D3; a valid brief is copied to `docs/briefs/<id>.md` with the new id; an invalid brief refuses the add, naming every defect, and writes nothing (no PBI event, no brief). With no `--brief-file`, the PBI is still added, and the result carries `brief: "missing"` (JSON) and a warning line naming `docs/briefs/<id>.md` and `bee backlog briefs` — recorded and flagged, never silent. | The agent that adds the item holds the context; the id exists only after the add, so the door copies the draft in. |
| D5 | A live item is a PBI with status `proposed` or `in-flight`. bee has no `ready` status; `parked`, `done` and `declined` need no brief. | The spec named `ready`; bee's status set does not carry it. |
| D6 | An active feature is a feature whose pipeline record — the default `.bee/state.json` or a `.bee/lanes/<feature>.json` lane — has a phase other than `idle` and `compounding-complete`. | — |
| D7 | Folding: a PBI with status `in-flight` whose `feature` names an active feature is shown inside that feature, never as its own item, and needs no brief of its own — the feature's brief covers it. | The owner sees one card per piece of work. |
| D8 | `bee backlog briefs [--id <id>] [--json]` is the owner view and the check: it lists active features (with their folded PBIs nested) and unfolded live PBIs, each with brief status `ok`, `missing` or `malformed` plus the defect list; it exits 1 when any listed brief is missing or malformed. `--id <id>` checks the one brief at `docs/briefs/<id>.md` and exits 1 when it is missing or malformed. | One verb serves Lock, deferral and the owner listing. |
| D9 | `bee doctor` gains an advisory `owner_briefs` note row (never a voting row): the count of live items and active features without a valid brief, pointing at `bee backlog briefs`. | Doctor's verdict is about runtime wiring; a brief gap must not block it. |
| D10 | `docs/backlog.md` (`bee backlog render`) stays the internal ledger, unchanged. The owner view bee lists is `bee backlog briefs`; the folding rule is documented in the knowledge concept so consumers apply it the same way. | Changing the rendered table would break its byte-determinism check for no owner gain. |
| D11 | bee-shaping's Lock writes `docs/briefs/<feature>.md` from the Feature Boundary and the Locked Decisions and checks it with `bee backlog briefs --id <feature>`. When shaping or planning defers scope into a backlog item, it adds the PBI with `--brief-file`, and `Chi phí và rủi ro` carries the reason for the deferral and its cost. | This is the gap behind agent-works p-bea1cb2a. |
| D12 | Skills (bee-shaping, bee-planning, bee-capturing deferral line) and one knowledge concept teach the contract with one good example and one bad example; the bad example is the one-sentence CoS of agent-works p-bea1cb2a. | Cold agents copy examples. |
| D13 | The staleness digest (`sources_digest`) is out of scope; it is deferred as a proposed PBI that carries its own brief. No other repo is backfilled; this repo's own backfill of existing items is left to its leader after the release. | Spec marked the digest optional and forbade backfill. |
| D14 | Brief files — the draft read, the stored brief and every brief status read — resolve against the checkout the command runs in (the feature worktree when inside one); the PBI store and lane records stay at the control root. Added during execution after a live run from the worktree reported an existing brief as missing. | Lock writes the feature brief inside the worktree and it merges with the branch. |
<!-- /bee:not-a-deferral -->

### Agent's Discretion

The spec delegated the flag shape, the label configuration and the check placement; D3, D4, D8 and D9 record those choices.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| brief | The file `docs/briefs/<id>.md` in the D1 contract. |
| live item | A PBI in `proposed` or `in-flight` (D5). |
| active feature | A feature whose pipeline phase is not `idle` or `compounding-complete` (D6). |
| folded | An in-flight PBI shown inside its active feature (D7). |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/backlog.rs` — `add_pbi`, `run_pbi_add`, `run_propose`, `fold_pbis`, `try_native` verb dispatch.
- `packages/bee-rs/crates/bee/src/state.rs` — `read_config_raw` reads `.bee/config.json`.
- `packages/bee-rs/crates/bee/src/doctor.rs` — `doctor_report`; `team_config_advisory` is the advisory-row pattern.
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — hand-maintained command tree; `tests/registry_dispatch.rs` runs each entry's first example.

### Integration Points

- `skills/bee-shaping/SKILL.md` (Lock, scope-creep deferral), `skills/bee-planning/` deferral, `skills/bee-capturing/SKILL.md` deferral line; rendered by `bee dev render-skill-trees`.
- `docs/knowledge/areas/decision-memory/backlog-store.md` — the backlog store concept to extend.

## Canonical References

- `docs/history/owner-brief/spec.md` — the spec drop as received; `docs/history/owner-brief/brief-draft.md` — this feature's own brief, placed at `docs/briefs/owner-brief.md` by cell work (the write guard holds `docs/briefs/` until the execution gate).

## Outstanding Questions

None block planning.

<!-- bee:not-a-deferral: locked decisions about how briefs record deferrals; the one moved-out item is backlog p-0b45c28d -->
## Deferred Ideas
<!-- /bee:not-a-deferral -->

- Brief staleness digest (D13) — proposed PBI with its own brief.
