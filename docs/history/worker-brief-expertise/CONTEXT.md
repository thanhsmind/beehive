# Worker Brief Expertise Section — Context

**Feature slug:** worker-brief-expertise
**Date:** 2026-08-20
**Shaping session:** complete
**Scope:** Quick
**Domain types:** SEE | CALL

## Feature Boundary

The dispatcher can hand a worker an Expertise section in its brief — path,
purpose, and a "read this to do X correctly" line per entry, pointing at bee's
own skill/knowledge files — without pulling the worker into the bee workflow.
Ends at the brief rendering plus the wording rescope; no automatic entry
derivation, no new knowledge machinery.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The worker brief gains an optional Expertise section; each entry names an absolute file path, its purpose, and one "read this to do X correctly" line. Zero entries renders no section. | Leader-style briefing: the dispatcher names the capabilities the job needs, not just the task. |
| D2 | Entries point at bee's own expertise content — skill reference files under `skills/` and bee knowledge files — chosen per task by the dispatching agent's judgment (Main orchestrator or herding dispatch role). No automatic derivation machinery. | The user wants leader judgment, not a pipeline; keeps scope to rendering. |
| D3 | The standalone-executor clause is rescoped to workflow-only: the worker still never runs bee commands and never writes workflow state, but reading the listed expertise files is explicitly allowed and encouraged. Brief wording must not contradict the Expertise section. | Today's "IGNORE any bee instructions" would tell the worker to skip the very files D1 lists. Workflow-ignorance ≠ expertise-denial (touches herding-worker-standalone D1-D3, not superseding). |
| D4 | The same entry shape (path + purpose + read-to line) appears in the bee-swarming dispatched-worker brief, where entries may also name skills directly. | Task-tool workers live inside the workflow; they may be told to load skills, not only read files. |

### Agent's Discretion

How the entries travel into `bee herding run` (flag shape, job.json field) and
the exact section wording — constrained by D1's three-part entry shape and D3's
no-contradiction rule.

## Existing Code Context

### Integration Points

- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:118` — `render_brief` renders the standalone-executor brief; the Expertise section and the D3 wording rescope land here.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — builds `BriefSpec` and `job.json`; entry plumbing (CLI flag → spec → brief) lands here.
- `skills/bee-swarming/SKILL.md` (worker contract, "Execute") — D4's prose change for Task-tool worker briefs.
- `docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md` — the brief/mailbox contract this extends; sync at capture.

## Outstanding Questions

### Deferred To Planning

- [ ] Flag shape for `bee herding run` (repeated `--expertise "path|purpose|read-to"` vs a JSON file) — pick whichever the existing run flags already pattern.
- [ ] Whether the herding dispatch role's prompt (`skills/bee-herding/references/dispatch-prompt.md`) needs a line telling the dispatcher to compose entries — check the file, add only if it composes the run command.
