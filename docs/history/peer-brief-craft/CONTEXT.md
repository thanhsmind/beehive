# peer-brief-craft — CONTEXT

## Ask

The owner asked whether the current playbooks and skills help a peer
worker write better code, then chose option A: fix the brief
contradiction, give workers the depth layer, and measure the principle
lines — after checking whether `.bee/expertise` still earns its place
beside the skills.

## Findings that shaped the decisions

- `packages/bee/prompts/peer-boot.md` says "Do not read AGENTS.md", while
  `packages/bee/prompts/worker-cell.md` lists AGENTS.md as the first input
  for every cell. A thin read peer dispatched for this feature used `grep`
  against this repo's AGENTS.md rule (use `rg`/`fd`) and blocked.
- All 37 `skills/bee-principle-*/SKILL.md` end with a `**Depth:**` line into
  `.bee/expertise/`. Principles restate about 3 of 22 sections of
  `tests.md` and 3 of 16 of `debugging.md`; the domain guides (data, apis,
  security, frontend, performance, operations) have no skill twin.
  Expertise is the depth layer, not a duplicate.
- A cell brief gets expertise only when the leader passes `--expertise`;
  reviewers get `review.md` by default. A herded thin peer never loads
  `bee-swarming`, so it never sees the `.bee/expertise/INDEX.md` route.
- `prepare` records `principles_arm` (`principles` | `holdout`, 10 percent
  default holdout) on every cell dispatch, but no report compares the arms.

## Locked Decisions

- **D1** A herded peer reads AGENTS.md first. `peer-boot.md` names
  AGENTS.md as the project's hard-rule file and forbids only CLAUDE.md,
  `.bee/LEADER.md` and bee process docs. Touches thin-worker-boot D2
  (`48021ff8`): AGENTS.md is that one thin shared file.
- **D2** `.bee/expertise` stays as the depth layer and is never injected
  whole into a cell brief. `worker-cell.md` gains one on-demand line, in
  both the native and the herding render: follow a principle's Depth line,
  or route `.bee/expertise/INDEX.md` to the one guide section the cell
  needs. Rejected: default tests.md + debugging.md in every code cell
  (about 950 lines per brief, repeats the principle lines); deleting
  expertise (37 principles lose their depth).
- **D3** `bee knowledge report` gains a principles-arm outcome block shaped
  like the knowledge-arm block (`outcomes`), read from the `principles_arm`
  key of prepare cell rows in `.bee/logs/dispatch.jsonl`, arms
  `principles` and `holdout`, same signals and same minimum per arm.
  Touches 4cfda42e (feature_outcomes).
- **D4** The herding worker brief asks a peer which principles it applied: when
  a Principles block appears, the result file carries a `principles_used` array,
  one `<slug> — <the decision it changed>` entry per principle, `[]` when none
  changed a decision (decision `de367f58`). `herding/mailbox.rs` already parses
  it and `cells finish --from-job` already carries it; no Rust product change.
