---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: verify-map-upkeep-3

## Summary

Ship the proven corrections from the 2026-11-25 `bee-verify-upkeep` pass to the
`verify-app` source under `.bee/verify/verify-app/`: a global-config isolation
fix and a `drive --fast|--full` entry point in `control-bee`, a tester packet
in the fixtures, and the doc drift in the feature files. Then re-render the
runtime copies with `bee onboard --verify-only --apply`.

Live evidence (main checkout, read only):
`~/.local/state/bee-verify/evidence/20261125-225437-3171768/`,
`~/.local/state/bee-verify/evidence/vu-*/`.
Source-wave reports: `.bee/mailbox/<job>/report-1.md` in the main checkout,
one per feature; index in `/mnt/data/Projects/goglbe/beehive/.bee/tmp/vu/out/<feature>.json`
(`report_path`).
Working drive scripts to copy recipes from (main checkout, read only):
`/mnt/data/Projects/goglbe/beehive/.bee/tmp/vu/{cells,wt,guards,cg,rest,iso}.sh`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | control-bee has no drive subcommand | read | .bee/verify/verify-app/control-bee:449 | *) die "unknown subcommand '$1' — run 'control-bee --help'" ;; |
| 2 | the close tests door needs a capped tester on the test role for a stamped cell | read | packages/bee-rs/crates/bee/src/verbs/cells/proof.rs:64 | stamped cell(s) have no capped dependent tester or checker on the test role |

## Proven corrections

### Harness (`control-bee`, cell vmu3-1)

- H1 Global-config isolation (CONTEXT D1). In `launch`, `record_in`, `host`
  and `smoke`, when the caller has not exported `BEEHIVE_CONFIG_DIR`, export
  it as `<run dir>/global-config` (created empty). A caller that exports it
  keeps its value (config-layers recipe).
- H2 `drive --fast` and `drive --full` (CONTEXT D2-D4). One function per
  mapped feature, each on a private run id `drive-<feature>-<time>-<pid>`
  that never writes `current-run` (use the same `SMOKE_RUN` guard idea as
  `smoke`), launch + doctor, then the feature's checks, asserting on the
  recorded `.exit` files and JSON keys. Print `  ok    <feature>` or
  `  FAIL  <feature>  (<first failed check>, evidence <dir>, sandbox kept at <dir>)`.
  Run every feature even after a failure; exit 0 when all pass, 1 when any
  fails. `drive` with no argument, or any argument other than `--fast` /
  `--full`, prints usage to stderr and exits 2. `--fast` runs
  orient-and-status, feature-gates, cells-and-proof, onboard,
  comment-guard, worker-outward-guard. `--full` runs all thirteen.
  Per-feature minimum checks (all proven live in this pass):
  - orient-and-status: `version --json` version equals plugin.json; idle
    `orient --json` where.phase idle, next.skill bee-hive; `status --json`
    onboarding.installed true, drift false; after start-feature, orient
    where.phase exploring and next.skill bee-shaping.
  - feature-gates: start-feature; second start exits non-zero; route --set
    `--files a.rs` exits non-zero; merge gate refuses (no CONTEXT), put
    CONTEXT fixture, refuses (no plan.md), put plan, preview, append a line to
    plan.md, refuses stale preview, re-preview, merge approves
    shape+execution; `gate --name uat` true then false; `--merge --name`
    refused; `--actor auto` without bypass level refused.
  - cells-and-proof: on top of feature-gates' approved state: dry-run add
    reports ok false; mismatched cell refused; add demo-note-1; claim;
    worker add; commit NOTE.md; red, bare green, mismatched command and
    locator-less green:live caps all exit 1 and the cell stays claimed;
    green:live with the evidence dir caps it (`--no-mistakes`).
  - onboard: newrepo target; plan status changes_needed with a
    `design_offer` key; apply status applied, recheck up_to_date, no plan
    key; re-run up_to_date with empty plan.
  - comment-guard: `config set --key no_code_comments --value true`,
    start-feature, `config set --key worktree_first --value off`,
    `gate --name execution --approved true`, put
    `packages/bee-rs/crates/bee/src/x.rs`, crafted Edit payload to
    `hook write-guard` exits 2 with `bee comment guard` on stderr; key false
    exits 0; moved comment exits 0.
  - worker-outward-guard: start-feature, worktree new; payloads with cwd the
    worktree: `git push origin main` exits 2, `gh pr view 1` exits 0,
    `claude -p x` exits 2; `config set --key guards.worker_outward --value false`
    then push exits 0.
  - worktree-and-close: worktree new / list grant / exit transition / linked
    merge emits `exit-worktree-before-merge` / merge from main merged true and
    WORK.md lands; then the demo-note flow with BOTH fixture cells capped
    (demo-note-2 is the tester), leader-check recorded, capture deferral
    audited, and `close --feature demo-note --dry-run --json` lists no door
    with blocking true.
  - config-layers: global dir under the run holding
    `{"from_global_only":"g"}`; `config explain --key from_global_only`
    source global, effective g.
  - semantic-role-routing: v1 preview has role_plan null; role-plan-v2
    preview refuses `role_plan_roster_stale`.
  - codex-runtime: model-guard probe exits 2 with
    `every Codex spawn_agent needs an explicit role` (line says
    `cli surface; live canary not driven`).
  - pi-runtime: doctor --runtime pi ready with HERDR env; drifted index.ts
    blocked; attest refused `not_applicable`.
  - pi-harness-dispatch: `hook stage-tools` exits 0 and lists `bee_dispatch`
    and `bee_steer` (line says `live Pi leader not driven`).
  - pi-hat-wave: `dispatch prepare --runtime pi --kind advisor --role
    hat-facts-gaps` command carries `--seat "hat-facts-gaps"`, `--ceiling 600`
    and `--no-pane` (line says `live Pi leader not driven`).
- H3 Add the tester packet `demo-note-2` (role `test`, deps `demo-note-1`,
  files `NOTE.md`, verify `grep -q note NOTE.md`) to
  `fixtures/demo-note-plan.md` and as `fixtures/demo-note-2.json`.
- H4 SKILL.md: a `## Drive entry points` section (commands, exit codes, which
  features each runs, D4 limits) and a Launch line on global-config isolation;
  add `drive` to the Helpers subcommand list and the `--help` header.
- H5 Add no comment line to `control-bee`; the comment guard covers
  `.bee/verify/`.

### Feature map (cell vmu3-2)

- M1 README: Fixtures section names `demo-note-2`; the Upkeep log gains the
  2026-11-25 line (run `20261125-225437-3171768`, shipped in
  `verify-map-upkeep-3`); Baseline preconditions mention `drive --fast|--full`
  and the default global-config isolation.
- M2 orient-and-status: `bee-wayfinding` replaces `bee-hive` only when the
  pipeline is idle (no handoff, no open/claimed/parked cells, phase idle or
  compounding-complete); `orient --class <c>` prints no timing line.
- M3 feature-gates: `gate --preview` writes `gate_preview` (feature,
  plan_sha256, previewed_at, cells); `approved_cell_packet` is copied at
  approval. Refusal order: missing CONTEXT, then `plan.md does not exist`,
  then `missing gate preview`, then `stale gate preview`. Approval
  preconditions are six (plan claims, CONTEXT read set, preview, owner-first,
  advisor, lane conflict), not four. Start payload `approved_gates` holds
  context, shape, execution, review; uat is reported by `gate` and `orient`.
- M4 cells-and-proof: dry-run problems sit at `cells[].problems`; the
  mismatched-cell example is refused with `(files mismatch)`; the result set
  has four values incl. `green:red-first`; the bare-green refusal names four;
  optional report keys add `principles_used` and `knowledge_wrong`; a test
  claim refuses `CONTRACT_UNCITED` once any contract decision exists; a
  `tiny` cap skips the registered-worker check only while at most one counted
  file changed; the self-cap recipe path is `$VERIFY_HOME/run/<run-id>/…`,
  not `/tmp/bee-verify`; close needs the tester cell demo-note-2 capped.
- M5 worktree-and-close: the `tests` door blocks with `stamped cell(s) have
  no capped dependent tester or checker on the test role` until demo-note-2
  is capped; the green-close recipe adds, claims and caps it.
- M6 comment-guard: use `config set` (it exists); the guard is reached only
  past the intake and execution gates: start a feature, set
  `worktree_first` off, approve the execution gate; the denial names
  `<path>:<line>`, not an edit index; an apply_patch license header is not
  exempt.
- M7 onboard: the design-packs offer is the top-level `design_offer` key, not
  a notice; drop the stale `file:line` cites (keep file names); mention
  `--verify-only` (refuses `blocked_not_onboarded` on a virgin repo).
- M8 worker-outward-guard: turn the key off with
  `config set --key guards.worker_outward --value false` (a `put` replaces the
  whole config); the key is read from the merged config layers; a leader
  holding the worktree grant is also exempt from the launch refusal; a herded
  pane has a separate worker guard that ignores the opt-out.
- M9 codex-runtime: `doctor attest` writes `.bee/doctor-attest.json` (not a
  read-only inspection); the read-only sandbox applies to every non-cell,
  non-deployment Codex dispatch.
- M10 pi-runtime: the drift detail is count-based
  (`1 file in .pi/extensions/bee-guard/ differs from what this bee embeds (index.ts) — run \`bee onboard --apply\` to refresh`);
  a `paseo_ready` row appears when `team.pi` names paseo agents; a
  `workspace-protocol` row exists; an extra `*.ts` file also fails wiring.
- M11 pi-harness-dispatch: the leader tool set includes `bee_steer`; the
  probe evidence lists it.
- M12 pi-hat-wave: `--no-pane` only for Pi-binary agents; the ceiling is
  min(configured, 600); a paseo agent's readiness is the paseo CLI on PATH;
  a detached launch also prints `inbox_session`.
- M13 config-layers: the precondition now reads that `control-bee` isolates
  the global dir by default, and exporting `BEEHIVE_CONFIG_DIR` overrides it.

## Cells, current slice preview

```json
[
  {
    "id": "vmu3-1",
    "feature": "verify-map-upkeep-3",
    "title": "Isolate global config and add drive entry points to control-bee",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [],
    "files": [
      ".bee/verify/verify-app/control-bee",
      ".bee/verify/verify-app/SKILL.md",
      ".bee/verify/verify-app/fixtures/demo-note-plan.md",
      ".bee/verify/verify-app/fixtures/demo-note-2.json"
    ],
    "read_first": [
      "docs/history/verify-map-upkeep-3/plan.md",
      "docs/history/verify-map-upkeep-3/CONTEXT.md",
      ".bee/verify/verify-app/SKILL.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Apply H1-H5 from docs/history/verify-map-upkeep-3/plan.md to control-bee, SKILL.md and the fixtures",
    "verify": "bash .bee/verify/verify-app/control-bee drive --full",
    "must_haves": {
      "truths": [
        "drive --full prints one ok line per mapped feature and exits 0",
        "drive with an unknown argument exits 2",
        "a sandbox bee process sees an empty global config unless the caller exported one"
      ],
      "artifacts": [
        {
          "path": ".bee/verify/verify-app/control-bee",
          "substantive": "drive subcommand and global-config isolation"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no comment lines added to control-bee",
        "no product code changes"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "vmu3-2",
    "feature": "verify-map-upkeep-3",
    "title": "Correct the verify feature map drift",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": [],
    "files": [
      ".bee/verify/verify-app/features/README.md",
      ".bee/verify/verify-app/features/orient-and-status.md",
      ".bee/verify/verify-app/features/feature-gates.md",
      ".bee/verify/verify-app/features/cells-and-proof.md",
      ".bee/verify/verify-app/features/worktree-and-close.md",
      ".bee/verify/verify-app/features/comment-guard.md",
      ".bee/verify/verify-app/features/onboard.md",
      ".bee/verify/verify-app/features/worker-outward-guard.md",
      ".bee/verify/verify-app/features/codex-runtime.md",
      ".bee/verify/verify-app/features/pi-runtime.md",
      ".bee/verify/verify-app/features/pi-harness-dispatch.md",
      ".bee/verify/verify-app/features/pi-hat-wave.md",
      ".bee/verify/verify-app/features/config-layers.md"
    ],
    "read_first": [
      "docs/history/verify-map-upkeep-3/plan.md",
      ".bee/verify/verify-app/features/README.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Apply M1-M13 from docs/history/verify-map-upkeep-3/plan.md to the feature files",
    "verify": "rg -c green:red-first .bee/verify/verify-app/features/cells-and-proof.md",
    "must_haves": {
      "truths": [
        "every M item is reflected in its feature file"
      ],
      "artifacts": [
        {
          "path": ".bee/verify/verify-app/features/README.md",
          "substantive": "2026-11-25 upkeep log line"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "feature entry contract keeps its four H2 sections"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "vmu3-3",
    "feature": "verify-map-upkeep-3",
    "title": "Test the drive entry points",
    "lane": "small",
    "role": "test",
    "deps": [
      "vmu3-1"
    ],
    "decisions": [],
    "files": [
      ".bee/verify/verify-app/control-bee"
    ],
    "read_first": [
      "docs/history/verify-map-upkeep-3/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run drive --fast, drive --full and drive --bogus; check per-feature lines and exit codes 0, 0, 2; check a sandbox launched without BEEHIVE_CONFIG_DIR does not read the user-global config",
    "verify": "bash .bee/verify/verify-app/control-bee drive --fast",
    "must_haves": {
      "truths": [
        "drive exit codes are 0 on green and 2 on an unknown entry point"
      ],
      "artifacts": [
        {
          "path": ".bee/verify/verify-app/control-bee",
          "substantive": "verified, not edited unless a defect is found"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "no product code changes"
      ]
    },
    "behavior_change": false
  }
]
```
