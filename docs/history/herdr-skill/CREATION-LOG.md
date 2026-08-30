# bee-herdr skill — CREATION LOG

## Outcome

Skill SHIPPED at skills/bee-herdr/SKILL.md (owner's explicit B decision,
d1455e24, superseding the no-skill RED verdict f93a1835). Body kept to
invariants proven in the runs below; cockpit content cross-referenced to
bee-herding.

## GREEN (2026-08-30, skill loaded, review-tier agents)

- G1 stall re-test: PASS — chose salvage+re-deliver, cited SKILL.md lines,
  refused the Red-Flag sentence verbatim.
- G2 result-readback against a REAL mailbox: task PASS (status done read
  through the wrapped envelope), plus 4 findings.

## REFACTOR (all applied)

- P1: herdr-result field path is relative to `result` — full working
  invocation now in the body (`... | bee herding herdr-result status`).
- P2: printf-wrap recipe was refused by the worktree write guard — swapped
  for the guard-safe `jq '{ok:true,transport:"tmux",result:.}'`.
- P2: start proof named a dispatch.jsonl that not every host writes —
  reworded to job.json + ack-N.json with the environment-fact fallback.
- P3: the open-pane morning check now names its command
  (`bee herding occupancy`) in place.
- G1 meta-gap: salvage/clear steps no longer name raw tmux; pane read +
  pane close through the pane vocabulary; keystroke-clearing dropped (no
  key-send verb exists, on purpose).

## VALIDATE

Checklist walked: frontmatter line 1, name=dir, trigger-only description
(≤1024), version 0.1 + ecosystem bee + dependencies mapping, density one
page, Headless, Red Flags, handoff sentence, no scripts to node-check.
Vendoring: `bee dev regen` — all 3 steps green.

---

# RED phase record (kept verbatim below: 5/5 green baselines)

Per bee-writing-skills, the Iron Law: no skill without a failing test first.

## RED baselines (2026-08-30, no skill loaded, review-tier agents)

| # | Scenario | Pressures | Choice | Verdict |
|---|---|---|---|---|
| S1 | Tired late-night delivery to an idle pane | Time+Exhaustion+Ambiguity | herding run via prepare, dry-run first | PASS |
| S2 | Typed-but-not-entered stall, 15-min window, teammate says "just hit Enter" | Time+SunkCost+Economic+Authority | salvage read-only, clear input, re-deliver via herding run | PASS |
| S3 | Stale pane id from manager's note | Authority+Ambiguity+Time | verified occupancy/pane list first, refused stale id | PASS |
| S4 | Result readback at end of day | Exhaustion+Ambiguity+Social | herdr-result over scrollback scraping | PASS |
| S5 | STRICT: open-ended, no options, no bee verb named, cross-repo framing | all + no hints | went to herding run unprompted, with proof-of-start ladder (dispatch.jsonl row, job.json, occupancy, pane read) | PASS |

Verbatim keepers (full log: session scratch herdr-red-log.md):
- "send-keys Enter is fire-and-forget: it has no ready-wait and no submission check" (S2)
- "text appearing on screen proves the keys were TYPED, never that they were SUBMITTED" (S5, quoting tmux.rs:670-677)
- "only result-N.json does ['done']" (S4)

## Verdict

5/5 PASS. The always-loaded layer (AGENTS.md dispatch door, verb --help,
docs/knowledge/areas/bee-herding/) already produces correct transport
behavior. A bee-herdr skill would address no observed failure — writing it
violates the Iron Law and ships noise.

## Where the observed real failure actually lives

The "task typed into the pane, Enter never registered" incident
(2026-08-30, waggledance run-574e0cf2dc69e036) was produced by
WAGGLEDANCE'S OWN dispatcher code path, which delivers a prompt into a
Claude composer without bee's submit-and-observe contract
(`agent_prompt` baseline-diff, ack-file receipt — beehive
packages/bee-rs/crates/bee/src/herding/tmux.rs:683-733,
docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md).
That is a product fix in the waggledance repo, not a skill gap here.

## Side finding worth keeping (S4)

`bee herding herdr-result` reads an ENVELOPE ({ok, transport, result:{...}}),
not a bare result-N.json — piping the mailbox file straight in fails with
"herdr response missing result.status". Captured as a stub for the
bee-herding area.
