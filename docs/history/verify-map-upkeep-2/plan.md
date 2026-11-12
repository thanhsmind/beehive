---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: verify-map-upkeep-2

## Summary

Ship the proven corrections from the 2026-11-12 `bee-verify-upkeep` pass to
the `verify-app` source under `.bee/verify/verify-app/`: two harness defects
in `control-bee`, runnable fixtures for every gate-dependent recipe, and the
doc drift in the README, `SKILL.md` and the twelve feature files. Then
re-render the runtime copies with `bee onboard --apply`.

Live run: `~/.local/state/bee-verify/evidence/20261112-123209-529324/`.
Source-wave reports: `.bee/mailbox/<job>/report-1.md`, one per feature.
Scratch fixtures to copy from (main checkout, read only):
`/mnt/data/Projects/goglbe/beehive/.bee/tmp/upkeep/`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | control-bee picks the evidence index with a check-then-create loop | read | .bee/verify/verify-app/control-bee:225 | while [ -e "$stem.cmd" ]; do |
| 2 | bee registers a paseo workspace whenever PASEO_AGENT_ID or PASEO_CLI is set | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:66 | ["PASEO_AGENT_ID", "PASEO_CLI"] |

## Proven corrections

### Harness (`control-bee`) — re-drive each one live before the cap

- H1 Strip every `PASEO_*` variable from the environment of `cli`, `sh`,
  `host` and the onboard inside `launch` (`record_in` builds the env). Proof:
  a driving session inside paseo made `worktree new` in the sandbox register
  `wks_077afcfe9baf8dff` and `wks_d682b05eac8a6189` in the user's real paseo
  daemon (`▸ paseo: registering workspace…` on stderr).
- H2 Make the evidence index atomic. Five parallel `control-bee cli` calls
  wrote two `.out` files (221, 222), each holding two JSON lines. Claim the
  stem with a no-clobber create (`set -C`) and step on collision.
- H3 `launch` creates the sandbox on branch `main` (`git init -q -b main`).
  `dispatch prepare --stage deployment` refuses `main_branch_required` on
  `master`.
- H4 `launch` links `.bee/bin/bee` to the built binary after onboard.
  Onboard from the source checkout leaves no `.bee/bin/bee`, so
  `doctor --runtime pi` reports `hook_handler not_ok` and Pi recipes cannot run.
- H5 Add no comment line to `control-bee`; the comment guard covers
  `.bee/verify/`.

### Fixtures — new folder `.bee/verify/verify-app/fixtures/`

- `demo-note-plan.md` = `.bee/tmp/upkeep/plan-demo-note.md` (one packet,
  cell `demo-note-1`, verify `test -f NOTE.md`).
- `demo-note-1.json` = `.bee/tmp/upkeep/cell-demo-note-1.json` (the exact
  packet `cells add` must receive).
- `context-no-read-set.md` = one line
  `<!-- bee:no-read-set: verify fixture -->`.
- `role-plan-v2.md` = `.bee/tmp/upkeep/plan-v2.md` with its
  `roster_sha256` set to `REPLACE_FROM_ROSTER_STALE_ERROR`.
- README gains a `## Fixtures` section: put the context and plan with
  `control-bee put docs/history/<f>/{CONTEXT,plan}.md < fixtures/...`, then
  `gate --preview`, then `gate --merge`; for v2, preview once, copy the
  expected digest out of the `role_plan_roster_stale` error, put again.

### Doc drift (proven live unless marked source)

- SKILL.md: drop the `--no-mistakes` gotcha (bare `cells cap --no-mistakes`
  caps and writes `trace.no_mistakes: true`); add the paseo strip, the `main`
  branch and the linked binary to Launch; say parallel drives are safe; point
  at `fixtures/`.
- feature-gates: the merged gate needs `docs/history/<f>/CONTEXT.md` (read
  set or the no-read-set marker) and a previewed `plan.md`; refusal order is
  missing CONTEXT, `missing gate preview`, then `stale gate preview` after an
  edit. After merge `approved_gates` carries `uat: false` (drop the "NO uat
  key" claim). The conflict-review refusal fires only for a lane target
  (`start-feature --as-lane`, `gate --merge --lane <f>`; remedy
  `state plan-conflicts derive --lane <f>`). A second start refuses naming the
  current phase and offering `--as-lane`, not an unclean workspace; drop the
  `feature-start-guard` clean-tree sub-feature. `gate_records` are read from
  `status --json`, not the `state.json` snapshot. `gate --preview` writes
  `gate_preview.plan_sha256` (source). Replace stale line anchors with
  function names.
- cells-and-proof: `cells add` must match the approved packet verbatim
  (`differs from approved preview packet (action mismatch)`); use the fixture
  cell. The dry-run `problems[]` also names the packet mismatch. `--report`
  optional keys are mistakes, knowledge_used, baseline, reaction, skipped,
  unknowns, found, stopped (source). `cells show` works inside a granted
  worktree; `cells cap` refuses there (`refused inside a granted feature
  worktree`). A `green:live` cap whose reason names the absolute evidence dir
  caps. Claiming a `test`-role cell warns about a missing `contract:<name>`
  decision.
- comment-guard: with a feature active on main the worktree-first guard
  answers before the comment guard; drive it before starting a feature or
  with `worktree_first: "off"`. The baseline-rise step writes real source:
  the idle intake gate refuses it on main, so run it inside a worktree with
  an active feature. `--write` refuses the whole write when any file rises
  (source). `SAFETY:` is exempt anywhere, not only on unsafe blocks (source).
  Drop the `html.rs` string-literal example (source).
- worktree-and-close: no `same_worktree` kind exists; the refusal is
  `{"error":"cannot enter worktree \"<id>\": target directory is identical to
  source directory"}`, exit 1. Drop the "enter is main-only" gotcha. A
  `small`-lane dry run lists 13 doors including `leader-check`;
  `scribing-debt` and `leader-check` block for a `behavior_change` cell,
  cleared by `bee cells leader-check` / `bee audit log --kind
  leader-check-deferral` and bee-capturing / `bee audit log --kind
  capture-deferral`; add both to "Settle it and close green". With `--json`
  close prints no "Filed for" line; assert `fix_at_rows`. The merge payload
  adds `cleanup_suggested_command`. The merge-debt list is four codes plus
  `WORKTREE_MERGE_UAT_PENDING` (source). `worktree new` registers a paseo
  workspace when a paseo variable is set.
- semantic-role-routing: the v1 preview payload has no `ok` key. Errors are
  sentences that carry the code (`gate preview: role_plan_required: …`).
  Every stage role must be configured for the runtime, even
  `not-applicable` ones (`role_plan_unconfigured_role`); the default sandbox
  has no `team.pi.deploy`. `dispatch prepare --kind cell` on an open cell
  refuses `claim_ownership` first: pass `--claim`, and `cells unclaim` after
  a refused probe (backlog row: the refusal still claims). `decisions log`
  takes `--tags`, and the reroute decision needs `--feature <f>` or reroute
  refuses `role_reroute_decision_feature_mismatch`. Reroute success returns
  the cell; refusals are `role_unchanged`, `role_reroute_status` (claimed),
  `role_reroute_unconfigured_role`. The sandbox has no `scripts/release.sh`
  (exit 127): mark that step unreachable there. Deploy permit needs
  `team.pi.deploy`, branch `main`, a lane feature with its conflict check
  derived, and the issuer session registered through `hook session-init`
  with `session_id: verify-<run-id>`; then `authorized: true`, and a replay
  refuses `deploy_authorization_consumed`.
- codex-runtime: the live canary needs an authenticated private
  `CODEX_HOME` (an empty one ends `401 Unauthorized`), placed where the
  agent's write guard allows. The model-guard refusal prints `every Codex
  spawn_agent needs an explicit role` with exit 2 and does not print
  `codex-spawn-unmarked`.
- pi-runtime: rows are keyed `row` and include `design_skill`,
  `owner_briefs` and `runtime_*` advisories. With default `team.pi` roles on
  `--no-pane`, a missing `HERDR_ENV`/`HERDR_PANE_ID` and `herding.transport:
  "tmux"` without TMUX all stay `ready`; drop those red outcomes. The
  sandbox has no `.claude-plugin/plugin.json`: the manifest and
  stale-version cases are unreachable there. Attest refuses with `kind:
  "not_applicable"`, exit 1, no file. Under `BEE_HERDING_WORKER`, `activity`,
  `worker-guard` and `write-guard` still run (source). The no-pane advisor
  run works with the sandbox agent `pi`.
- pi-harness-dispatch: hook probes need the `session_id` of a registered
  session in the payload, or the stage reads `idle` and the intake gate
  answers; register with `hook session-init`. The leader set also carries
  `bee_steer`. Steer refusal codes `job_not_found`, `cell_capped`,
  `result_already_written`, `mailbox_read_failed` (source).
- pi-hat-wave: prepare always appends `--no-pane`; drop the pane-layout
  checks. A feature with an approved v2 role plan refuses `stage_required`,
  so use a feature without one. `transport_ready` is `false` without
  `HERDR_ENV` (backlog row). The `Seat: <seat>.` line is present, not first.
  A five-seat parallel wave ended five `done` jobs with reports; the ceiling-1
  seat ends `timed_out_ceiling`, `retryable: false`, exit 1. Add the
  foreground fallback for an empty `$PI_SESSION_ID` (source).
- onboard: the skillstation list is `skills.skipped`, not `skipped_skills`.
  Apply also lands `.gitattributes`, `.bee/LEADER.md`, `.bee/playbooks/`,
  `.claude/agents/`. Notices add the Claude settings baseline and the
  design-packs offer. `blocked_codex_hook_write` does not exist (source).
- orient-and-status: `orient` runs lease and claim expiry sweeps, so it is
  not a pure read (source). Idle `next.skill` may be `bee-wayfinding` when a
  discovery map has a frontier; `exploring` gives `bee-shaping` (source).
- README: record this pass (date, run id) under a `## Upkeep log` section.

## Cells, current slice preview

```json
[
  {
    "id": "vmu2-1",
    "feature": "verify-map-upkeep-2",
    "title": "Ship the proven verify-app corrections from the 2026-11-12 upkeep pass",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": [],
    "files": [
      ".bee/verify/verify-app/control-bee",
      ".bee/verify/verify-app/SKILL.md",
      ".bee/verify/verify-app/features/README.md",
      ".bee/verify/verify-app/features/cells-and-proof.md",
      ".bee/verify/verify-app/features/codex-runtime.md",
      ".bee/verify/verify-app/features/comment-guard.md",
      ".bee/verify/verify-app/features/feature-gates.md",
      ".bee/verify/verify-app/features/onboard.md",
      ".bee/verify/verify-app/features/orient-and-status.md",
      ".bee/verify/verify-app/features/pi-harness-dispatch.md",
      ".bee/verify/verify-app/features/pi-hat-wave.md",
      ".bee/verify/verify-app/features/pi-runtime.md",
      ".bee/verify/verify-app/features/semantic-role-routing.md",
      ".bee/verify/verify-app/features/worker-outward-guard.md",
      ".bee/verify/verify-app/features/worktree-and-close.md",
      ".bee/verify/verify-app/fixtures/"
    ],
    "read_first": [
      "docs/history/verify-map-upkeep-2/plan.md",
      "skills/bee-verify-upkeep/SKILL.md",
      ".bee/verify/verify-app/features/README.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Apply every item under Proven corrections in docs/history/verify-map-upkeep-2/plan.md to the verify-app source under .bee/verify/verify-app/ only: harness fixes H1-H5 in control-bee, the fixtures folder, and the doc drift in SKILL.md, the README and the twelve feature files. Add nothing the plan does not prove. Then run bee onboard --apply so the rendered verify-app copies match the source.",
    "verify": "bash .bee/verify/verify-app/control-bee launch && bash .bee/verify/verify-app/control-bee doctor",
    "must_haves": {
      "truths": [
        "control-bee strips PASEO_* from every sandbox command",
        "parallel control-bee cli calls get distinct evidence stems",
        "a launched sandbox is on branch main and has .bee/bin/bee",
        "no feature file still claims a behavior the 2026-11-12 run disproved"
      ],
      "artifacts": [
        {"path": ".bee/verify/verify-app/control-bee", "substantive": "H1-H4 harness fixes, no comment lines added"},
        {"path": ".bee/verify/verify-app/fixtures/demo-note-plan.md", "substantive": "plan whose packet is the cells-and-proof cell"}
      ],
      "key_links": [],
      "prohibitions": [
        "no product code edits",
        "no edits to commands.test",
        "no hand edits to rendered copies under .claude/skills, .agents/skills, .opencode/skills"
      ]
    },
    "behavior_change": true
  }
]
```
