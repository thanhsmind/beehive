# ppb-17c — bee-fanout checked against its pstack source

Scope: what ppb-17 wrote in commit `ec92d65bc`. That is `skills/bee-fanout/`
(`SKILL.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-fanout.md`, checked
against the pstack skill `pstack/skills/swarm/SKILL.md` (no reference files).
The licence was compared with `diff`. The five rendered copies
(`.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`, `.opencode`) of
`SKILL.md` were compared with `diff -q` and match the source tree.

Verdict: no source step is missing. Every phase, step and rule of the source is
present, in bee's terms where a tool was swapped. Every removed text (the cloud
environment choice, `cloud_base_branch`, the concurrency-limit remark, the default
model and the `pstack-models.mdc` file, the worktree-or-branch output option,
`disable-model-invocation`) has a port-notes entry. `LICENSE.pstack` is
identical to the pstack `LICENSE`. The pstack-only tool check is clean. No model
and no subagent type is picked by hand. The description is 234 characters and
says what the skill is not for. There are two findings. Neither blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 (`063a21e4`) counts as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule (source line) | New home | Status |
|---|---|---|
| Frontmatter `name: swarm` (2) | `name: bee-fanout` (decision `6ca3cd62`) | present |
| Frontmatter description (3) | line 4, rewritten with a "Not bee-swarming" clause | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Intro: N parallel cloud workers; slices, race or mix; parent waits, aggregates, one report (9) | line 18, "cloud" removed | present (cloud drop noted) |
| Start: open a todolist with one entry per phase before launching (13) | line 24, "task list" | present (swap noted) |
| Start: the four phases Frame, Fan out, Aggregate, Report (15-18) | lines 26-29 | present |
| A1: state the done predicate and the artifact or report (22) | line 33 | present |
| A2: choose the shape: slices, identical-brief race, or mix (23) | line 34 | present |
| A2: declare `first pass`, `rank all` or `best-of` before spawning (23) | line 34, "before dispatching" | present |
| A3: N from the user or derived from the shape; N is total workers (24) | line 35 | present |
| A3: "not the cloud concurrency limit" (24) | removed | dropped (noted: Dropped steps, bullet 3) |
| A4: worker model from `pstack-models.mdc`, else a named default (25) | line 36, seat via `bee dispatch prepare --kind gather` or `--kind advisor --role lane-N`, model from the bee team table | present (swap noted, bullets 2-3); default model dropped (noted) |
| A4: for a model race, name each arm's model up front (25) | line 36, name each arm's role up front and the model each payload returns | present (swap noted, bullet 3) |
| A5: each writing worker gets its own writable output: worktree, branch or `/tmp/swarm-<slug>/worker-<n>/` (26) | line 37, leader saves each answer at `/tmp/fanout-<slug>/worker-<n>/`; writing work is a cell for `bee-swarming` | present (swap noted, bullet 4); worktree/branch option dropped (noted, bullet 4) |
| B: spawn all N in one message, `subagent_type: generalPurpose`, background, configured model (30) | line 41, all N `bee dispatch prepare` calls in the same turn, each payload run in the background | present (swap noted, bullet 1) |
| B: `environment: "cloud"`; `"local"` only when the user's computer is needed (30) | removed | dropped (noted: Dropped steps, bullet 1) |
| B: `cloud_base_branch` for a non-default pushed branch (32) | removed | dropped (noted: Dropped steps, bullet 2) |
| B: every brief stands alone: goal, scope, slice or arm, verify, report (34) | line 43 | present |
| B: reports use `PASS`, `ISSUES` or `BLOCKED` with evidence (34) | line 43 | present |
| B: a dropout means proceed with N-1 and note it (36) | line 45 | present |
| C: read the terminal results (40) | line 51 | present |
| C: coverage needs a result for every required slice (40) | line 51 | present |
| C: a race applies the declared rule: first pass, rank all, best-of (40) | line 51, plus a read-only `--kind reviewer` judge | present (judge addition noted, bullet 5) |
| C: no raw worker dumps (40) | line 51 | present |
| C: compact result table, one-line evidenced issues, explicit gaps or dropouts (42) | line 53 | present |
| D: one consolidated in-chat report: table, issues, gaps or dropouts, race rule when used (46) | line 59 | present |

Additions with no source counterpart, each named in the port notes: the
`metadata` block with the `bee-cli` dependency (frontmatter paragraph); the
"this is not `bee-swarming`" paragraph at line 20 (bullet 8); the race-brief
rule at line 43 that arms get the same bytes and no leader leaning (bullet 8);
the blind-lane redirect at line 47 (bullet 7); the judge at line 51 (bullet 5);
the `PASS`-needs-seen-evidence line at line 53 (bullet 8); the same-model
caveat at line 55 (bullet 8); the Handoff section (bullet 8).

## `LICENSE.pstack`

`diff` against the pstack `LICENSE` prints nothing.

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-fanout/` prints
nothing and exits 1. A wider search under `skills/bee-fanout/` for
`subagent_type`, `generalPurpose`, `grok`, `sonnet`, `opus`, `haiku`, `fable`,
`--model`, `environment:`, `cloud`, `todolist` and `swarm ` also prints nothing.
The only pstack names left are in the port notes, as the notes say.

Every bee name the skill cites resolves: `bee-swarming`, `bee-arena`,
`bee-hive`, `bee-principle-separate-before-serializing-shared-state` and
`bee-principle-verify-before-reporting` are directories under `skills/`;
`bee-hive/references/gates-and-delegation.md` has the section "Blind lanes and
convergence" (line 158); `gather`, `advisor` and `reviewer` are dispatch kinds
(`verbs/drivers/prepare.rs:152`); `bee blind check` is routed
(`verbs/blind/mod.rs:1144`). The `--kind advisor --role lane-1..3` seat, with a
fourth worker reusing a lane role, is the same call `bee-arena` line 37 uses.

## Hand-picked model or subagent type

None. Line 36 sends every worker through `bee dispatch prepare`, says which
model a role runs on is a team-table setting, and says "never pick a model or a
subagent type by hand". Lines 41 and 51 run each payload as returned. The
`--runtime claude` flag picks a runtime, not a model or subagent type; it is the
same call `bee-arena` and `bee-interrogate` use.

## Description

234 characters (counted from the folded `description:` value). It says what the
skill does, when to use it (`/bee-fanout`, parallel coverage, a race or a
gauntlet) and when not to: "Not bee-swarming, which runs approved cells to
done." It fits the 300-character limit. The source's "exploration" trigger is
not in the description; the body keeps it (line 36, "coverage, exploration, a
gauntlet of checks").

## Findings

1. **P3, wrong number.** The port notes say the description is 237 characters.
   It is 234 (`wc -m` on the folded value). The limit holds either way; one
   number in the frontmatter paragraph would correct it.
2. **P3, trace.** The ppb-17 handback names commit `2e4ecf2a0` as the port. That
   commit is ppb-16c's report (`Check bee-show-me-your-work against its pstack
   source`). The port is `ec92d65bc` (`Port the pstack swarm skill as
   bee-fanout`), the only commit touching `skills/bee-fanout/`.

## Verify

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-fanout/ ; test $? -eq 1`
— no output, `test` exits 0.
