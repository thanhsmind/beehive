# Bee-Hive Provenance

Decision IDs and supersession history removed from live `bee-hive` text.

## Gates and UAT
- **Gate renumbering (`validation-diet` D2):** bee previously had four gates; `validation-diet` D2 merged shape and execution into one `bee gate --merge` call (today's Gate 2). Old Gate 4 (review) is today's Gate 3; the old standalone execution gate no longer exists.
- **`uat` placement & no auto-approve (`defaults-and-agent-env` D1, `uat-gate-before-merge` D1):** `uat_stop` defaults to `"close"`, and `bee gate --name uat --actor auto` is refused outright.
- **Gate bypass record scope (`traceable-runs` D2):** `gate_bypass` controls whether the run stops, never whether the brief or approval record is written.

## Wait Mark
- **Waiting-on mark (`awaiting-human` D1/D2/D4, `auto-wait-mark` D3):** The agent marks `gate` or `question` on `bee state waiting-on set`; the Stop hook sets `turn-end`; the mark works with or without an active feature and expires with the session heartbeat.

## Delegation, Herding, and Deployment
- **Herding execution branch (`herding-executor` D1/D3/D4/D7, `herding-tier` D1-D6, `herding-review-slots` D1):** `herding-tier` D1-D6 (widened by `herding-review-slots` D1) routes every purpose against a `{kind:"herding"}` slot through `bee herding run`, superseding `herding-executor` D7's cell-execution-only boundary.
- **Deployment mutation exception (decision `38edea83`):** Authorizes mutation only for an approved `--stage deployment` dispatch with role `deploy`.
- **Legacy `subagent_type` guard spelling (D2):** Kept as a spelling the model-guard hook still accepts alongside `prepare` payloads.

## Blind Lanes and Hat Wave
- **Blind lanes (`slp-blind-lanes` D1, D2(e), D5, D7; decision `79b5437b`):** Unconverged dossiers hand off unchanged without coin flips (`D1`, `D2(e)`); pushback requires naming the missing context (`D5`); lanes are not hats (`D7`); citation checks prove provenance, not faithfulness (`79b5437b`).
- **Lane & hat seat roles (`lane-model-diversity` D1, D2, D3, D4):** `lane-1..3` and `hat-*` sit in `team.<runtime>` (`D1`), unconfigured seats fall through to `advisor` while recording `requested_role` (`D2`, `D4`), and configured hats carry a `description` field (`D3`).
- **Hat wave (`proactive-leader-intake` D1–D7; decisions `07328333`, `98ac20a1`, `a52c854d` superseding `8fb1e0da`, `423e1664`, `b34fdea9`, `f73d6c49`):** `gates-and-delegation.md` is the single home (`07328333`); plan-step wave and pre-Lock spec-critique window coexist (`D5`, `98ac20a1`); plan-step wave opens at PLAN (`D1`, `a52c854d` superseding `8fb1e0da`), once per feature (`D2`, `a52c854d`), 3 seats default / 5 on high-risk (`D3`, `423e1664`), absorbs plan-checker & high-risk advisor consult (`D4`, `b34fdea9`), headless records Open Questions without simulating (`D6`, `f73d6c49`), and filters output in one leader voice (`D7`).
