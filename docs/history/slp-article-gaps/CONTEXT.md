# SLP Article Gaps — Context

Source: review of https://vhlam.com/article/agent-orchestration-multi-agent-slp
against bee (2026-11-04). The four gaps are backlog rows in commit 06514a686.

## Feature Boundary

In scope:

- The worker brief names three parts apart: goal, hard constraints, open
  choices. A cell carries its open choices in `must_haves.open_choices`.
- A read-only verb `bee report outcomes` that shows, per layer, how often
  the layer changed a result.

Out of scope:

- Herding-worker dissent: already closed on 2026-08-29 (decision 7db30738,
  PBI p-05d2a4f4 done); a herding worker dissents as data on its mailbox result.
- Removing or adding any review layer. The report gives data; the human
  makes that call.

## Locked Decisions

- **D1 (50349b36).** A cell's `must_haves` may carry `open_choices`: the
  plan's current design choices that a worker may question with evidence.
  The worker brief names three parts apart: goal (`must_haves.truths`),
  hard constraints (`must_haves.prohibitions` plus locked D-IDs), open
  choices (`must_haves.open_choices`).
- **D2 (b79877b1).** No worker prompt, skill, or template text orders a
  worker to look for faults, doubt the lead, or dissent a set number of
  times. Dissent stays a right used on evidence, never a duty.
- **D3 (e4711613).** `bee report outcomes [--feature <slug>]` reads active
  and archived cells and prints, per layer, how often the layer changed a
  result: worker reaction counts (absent counted apart), dissents and
  their verdicts, semantic judge PASS vs NEEDS_REVISION, leader check
  verdicts, cells reopened for rework, role reroutes. A layer with no
  record prints `not recorded`, never zero. It only reports; it never
  recommends adding or removing a layer.
- **D4 (34ad6dae).** The finding "workers almost never challenge the
  brief" closes through D1 plus D3, with no separate fix.

### Agent's Discretion

- The text and JSON layout of `bee report outcomes`, inside D3.
- The exact wording that tells a worker what an open choice is, inside D2.

## Terms

- **Open choice** — a design choice the plan made that the user did not
  ask for. A worker may question it with evidence through `bee cells
  dissent`. It is not a truth the goal-check reads.
- **Layer** — one check that can change a result: worker reaction,
  dissent, semantic judge, leader check, rework reopen, role reroute.

## Specific Ideas And References

- Article: "Brief nên tách ba thứ: mục tiêu, ràng buộc thực sự bắt buộc,
  và lựa chọn thiết kế đang dùng."
- Article: "Đừng biến quyền phản biện thành nghĩa vụ phản biện."
- Article: a reviewer that rarely changes results must prove its keep.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/report_usage.rs` — the shape of a
  read-only `bee report <sub>` verb wired through `verbs/mod.rs`.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs`
  (`cells_archive_dir`) — archived cells live in
  `.bee/cells/archive/<feature>/*.json`.

### Established Patterns

- Cell trace fields seen today (parsed over 1198 cells): `reaction`
  (8 confirm, 24 null, 1166 absent), `dissent` (4, all accepted),
  `semantic_judge` (464 verdicts, 67 NEEDS_REVISION), `leader_check`
  (266, all ok), `reopened_for_rework` (52), top-level `role_reroutes` (15).
- `must_haves` keys are not validated by name; only `truths` is required
  for some lanes (`verbs/cells/validate.rs`). `open_choices` needs no
  schema change.

### Integration Points

- The dispatch prompt inlines the full cell JSON, so `open_choices`
  reaches the worker with no code change.
- Worker prompt template and Execute loop:
  `skills/bee-swarming/references/swarming-reference.md`,
  `skills/bee-swarming/SKILL.md`.
- How a plan writes `must_haves`: `skills/bee-planning/`.

## Canonical References

- `docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md`
- `docs/discovery/slp-supervisor-lead-peer/MAP.md`

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

None.

## Deferred Ideas

- Correction to backlog row "Workers almost never challenge the brief":
  its numbers came from a raw line count. The parsed numbers are the ones
  under Established Patterns.

## Handoff Note

Two cells, disjoint files: one Rust verb, one skill-text change.
