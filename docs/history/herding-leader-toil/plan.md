---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: herding-leader-toil

## Summary

The leader spends less time on paperwork around workers, and Paseo workers run
cheaper:

- After checking a herded worker's cell, the leader caps it with one command that
  reads the worker's result instead of a hand-written report.
- Control commands work from inside a feature worktree; they write to the main
  checkout's store instead of refusing.
- A judge's answer is recorded in one command for every cell it judged.
- bee waits on Paseo state changes instead of asking every 3 seconds.
- A Pi worker can run with its own small Pi folder: the repo guard, no personal
  skills or extensions.
- `bee doctor` catches the desktop AppImage `paseo` and a broken worker folder.
- Workers show in the Paseo app by cell and agent.

Mode: `high-risk` — 4 risk flags: external-systems, covered-contract-change, multi-domain, cross-platform
Why this is the least workflow that protects the work: the worktree change alters a refusal several verbs and tests rely on, and the wait loop change touches every Paseo run, so every cell is red-first and the slice gets a live proof.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1: cells finish --from-job with the leader's --proof-result builds the cap report from the job.
- D2: control verbs from a granted worktree serve main's store; claim-like verbs still refuse; history reads stay in the worktree.
- D3: cells judge-record --from-text records each `json <cell-id>` block.
- D4: paseo agent wait on a background thread replaces the 3 s inspect.
- D5: opt-in isolated_config gives a pi worker its own Pi folder through PI_CODING_AGENT_DIR.
- D6: doctor fails on the AppImage CLI and a broken isolated folder.
- D7: worker title `<cell-id> <agent>` and bee_agent/bee_cell labels.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command executed, output kept). Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | One function builds the granted-worktree refusal text | read | packages/bee-rs/crates/bee/src/verbs/mod.rs:180 | refused inside a granted feature worktree |
| 2 | dispatch prepare already re-roots a granted worktree to main | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:34 | pub(crate) fn resolve_root_serving_granted(cwd: &Path) -> (Roots, Option<PathBuf>) { |
| 3 | The shared narrow-door prelude lives in reservations/emit.rs | read | packages/bee-rs/crates/bee/src/verbs/reservations/emit.rs:38 | pub(crate) fn prelude(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Pre> { |
| 4 | The cap report has exactly five keys | read | packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:57 | pub(crate) const REPORT_KEYS: [&str; 5] = ["outcome", "commit", "files", "tests", "deviations"]; |
| 5 | The proof command must equal the cell verify | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:266 | does not match approved cell verify command |
| 6 | herding run reports an uncapped success with a pointer to cells finish | read | packages/bee-rs/crates/bee/src/herding/run.rs:5155 | without capping it — settle with `bee cells finish` |
| 7 | Workers are titled by job id today | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:110 | argv.push("--title".to_string()); |
| 8 | The wait loop inspects on a 3 s cadence today | read | packages/bee-rs/crates/bee/src/herding/run.rs:3951 | let should_inspect = match last_inspect_ms { |
| 9 | Doctor checks only the version number of the paseo CLI | read | packages/bee-rs/crates/bee/src/doctor.rs:777 | if !crate::herding::paseo::version_at_least(&out, (0, 10, 3)) { |
| 10 | paseo agent wait exists with --timeout and --json in 0.10.3 | ran | /home/thanhsmind/.local/share/mise/installs/node/24.19.0/bin/paseo agent wait --help | --timeout |
| 11 | Pi reads PI_CODING_AGENT_DIR as its agent folder | ran | rg -n PI_CODING_AGENT_DIR /home/thanhsmind/.local/share/mise/installs/pi/0.87.1/pi/docs/environment-variables.md | PI_CODING_AGENT_DIR |

## Discovery

Two advisor digests (2026-10-06) mapped the refusal door, the cap and judge
parsers, `paseo agent wait`, Pi's agent folder and trust rules, the AppImage
output, and Paseo titles, labels and parent inheritance; CONTEXT.md carries the
environment facts. The leader opened claims 1-9. The plan-step hat wave (five seats) found blockers in hlt-1 (private mailbox module, missing mistakes answer), hlt-2 (a widened shared door, no close.rs, a non-existent release verb), hlt-3 (zero blocks exited 0) and hlt-4 (an unstoppable wait thread); D1-D4 and D6 were revised and the cells carry the fixes (reports/hat-wave.md).

## Approach

Recommended: reuse what exists — the prepare re-root for D2, the cap path for D1,
the judge validator for D3 — and add one seam (WaitSource) for D4. D5 is opt-in.

Rejected:
- Auto-cap on the worker's result — the leader must check first (user, 2026-10-06).
- Widening resolve_store_root itself — its refusal is asserted by tests and other callers rely on it.
- A `--parent` flag — Paseo has none and inherits the caller.

Risk map:

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| worktree verbs write main | HIGH | a wrong root writes the wrong store | hlt-2 | tests on a granted worktree, main and ungranted |
| event wait spins | MEDIUM | wait returns at once on idle or permission | hlt-4 | fake WaitSource tests for re-arm gaps |
| from-job proof honesty | MEDIUM | a synthesized green line | hlt-1 | --proof-result required; refusal tests |
| isolated folder drops bee-guard | MEDIUM | trust must stay always | hlt-5, hlt-6 | settings test and doctor row |
| auth link replaced by a refresh | LOW | Pi may rewrite auth.json | hlt-5, hlt-6 | doctor fails on a non-link; live check |
| orphan wait child | MEDIUM | a wait outliving the round | hlt-4 | cancel on every exit, fake-source test |

Waves: wave 1 runs hlt-1, hlt-2, hlt-4 and hlt-6 (disjoint files). Wave 2 runs hlt-3 (shares catalog.rs and the registry with hlt-1) and hlt-5 (shares herding.rs with hlt-1 and run.rs and paseo.rs with hlt-4). Wave 3 runs hlt-7.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"hlt-1 to hlt-6 change Rust verbs, the run loop and doctor."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first; the leader drives the live proof."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"hlt-7 updates concepts, the swarming reference and the config reference."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"required","role":"review","reason":"The slice judge for the behavior cells."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk plan check."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk plan check."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk plan check."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk plan check."}
  ]
}
```

## Shape

Epic map. Outcome: the leader's per-cell work is checking, not typing, and Paseo
workers cost less.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Leader toil | D1, D2, D3 | hand-built reports, refused verbs, copied verdicts | 1 | unit tests on granted worktrees, caps and judges |
| Paseo cost | D4, D5, D7 | polling, large worker context, unreadable workers | 1 | fake WaitSource and folder tests, live run |
| Setup check | D6 | AppImage passes doctor | 1 | doctor tests |
| Docs | all | concepts must match | 1 | knowledge check |

Current slice: all seven cells.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| hlt-1 | Cap a herded cell from its job with the leader's proof verdict | packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs; packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs; packages/bee-rs/crates/bee/src/verbs/cells/tests.rs; packages/bee-rs/crates/bee/src/catalog.rs; packages/bee-rs/crates/bee/src/generated/registry_payload.json; packages/bee-rs/crates/bee/src/herding.rs | — | bee cells finish --from-job with --proof-result and a mistakes answer caps a cell whose job result is done, with tests built from the cell verify, the leader's proof result and the worker proof text | related tests green |
| hlt-2 | Serve the main control plane for allow-listed verbs inside a granted worktree | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs; packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs; packages/bee-rs/crates/bee/src/verbs/cells/util.rs; packages/bee-rs/crates/bee/src/verbs/cells/mod.rs; packages/bee-rs/crates/bee/src/verbs/drivers/close.rs; packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs | — | state verbs, gate, route, close and cells add/list/show/ready/update/schedule/escalate/reroute/judge/judge-record/dissent/dissent-verdict/leader-check run from a granted worktree against the main store and name the main root | related tests green |
| hlt-3 | Record a judge's fenced verdicts straight from its answer | packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs; packages/bee-rs/crates/bee/src/verbs/cells/judge.rs; packages/bee-rs/crates/bee/src/catalog.rs; packages/bee-rs/crates/bee/src/generated/registry_payload.json; skills/bee-hive/references/gates-and-delegation.md | hlt-1 | cells judge-record --from-text records every valid json <cell-id> fenced verdict on its cell | related tests green |
| hlt-4 | Wait on Paseo events and name workers by cell and agent | packages/bee-rs/crates/bee/src/herding/run.rs; packages/bee-rs/crates/bee/src/herding/paseo.rs | — | the Paseo wait loop learns idle, permission, error and timeout from a paseo agent wait child it polls on the 200 ms tick and kills on every round exit | related tests green |
| hlt-5 | Give an opted-in Pi worker its own Pi config folder | packages/bee-rs/crates/bee/src/herding/pi_agent_dir.rs; packages/bee-rs/crates/bee/src/herding.rs; packages/bee-rs/crates/bee/src/herding/paseo.rs; packages/bee-rs/crates/bee/src/herding/run.rs | hlt-1, hlt-4 | a pi-provider Paseo agent with isolated_config true gets .bee/runtime/pi-agent/<agent>/ with auth, optional model and npm links, trust-always settings and empty skills and extensions | related tests green |
| hlt-6 | Catch the AppImage paseo CLI and a broken isolated Pi folder in doctor | packages/bee-rs/crates/bee/src/doctor.rs; packages/bee-rs/crates/bee/src/doctor/tests.rs | — | paseo_ready fails with the npm FIX when --version's first line is not a bare version or the command is an AppImage wrapper script | related tests green |
| hlt-7 | Document from-job caps, worktree control verbs, judge text, Paseo wait and isolated Pi folders | docs/knowledge/areas/bee-herding/the-paseo-channel.md; docs/knowledge/areas/worktree-parallelism/control-plane-topology.md; skills/bee-swarming/references/swarming-reference.md; docs/config-reference.md | hlt-1, hlt-2, hlt-3, hlt-4, hlt-5, hlt-6 | the Paseo channel concept describes the event wait, isolated Pi folders, the doctor checks and worker titles with their decision ids | knowledge check green |

```json
[
  {
    "id": "hlt-1",
    "feature": "herding-leader-toil",
    "lane": "high-risk",
    "role": "code",
    "title": "Cap a herded cell from its job with the leader's proof verdict",
    "deps": [],
    "decisions": [
      "D1"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "read_first": [
      "docs/history/herding-leader-toil/CONTEXT.md",
      "docs/history/herding-leader-toil/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs",
      "packages/bee-rs/crates/bee/src/herding.rs"
    ],
    "action": "Red first. Add three flags to `bee cells finish` (not `cells cap`): --from-job <job-id>, --proof-result <green:unit|green:static|green:live>, --proof-reason <text> (per D1). In finish_support.rs add a pure builder report_from_job(job: &Value, result: &MailboxResult-like fields, git: Option<GitFacts>, cell_verify: &str, proof_result: &str, proof_reason: Option<&str>) -> Result<String, String> that returns the five-key report JSON string: outcome = result summary; files = result files_changed, or the git changed paths when that list is empty; commit = git HEAD sha; tests = '<cell_verify> — <proof_result> — <proof_reason or the result proof text>' joined with the PROOF_SEPARATOR the parser uses; deviations = [dissent claim] when the result carries dissent, else []. The caller in handlers_close.rs (search for run_finish and cap_flags_from) reads the job from <main>/.bee/mailbox/<job>/job.json and the latest result through the herding mailbox helpers: in herding.rs change `mod mailbox;` to `pub(crate) mod mailbox;` and use select_latest_round, result_path and parse_result_text (do not touch herding/run.rs, which another cell owns; read_result and git_block are private there); run git in job.json cwd with a small local helper in finish_support.rs (rev-parse HEAD, diff --name-only <merge-base with the main branch>..HEAD, status --porcelain), then sets the report flag and continues through the existing cap path with no second validator. With --from-job the usual mistakes answer is required (--no-mistakes or --mistake with --fix-at), so bee close never finds the cell unanswered. The cap output (text and JSON) names the files source: worker files_changed or git fallback. Refuse, typed and naming the fix: --from-job with --report; --from-job without --proof-result; --from-job without a mistakes answer; an empty worker proof text with no --proof-reason (FIX: pass --proof-reason); a proof-result outside the three green words; no result file; result status not done; job.json cell_id present and different from --id; job cwd missing (FIX: pass --report). Declare the three flags for cells.finish in catalog.rs and in generated/registry_payload.json in the same shape as the existing cells.finish flags, and keep the registry and catalog tests green. Tests in verbs/cells/tests.rs with a temp repo, a job mailbox and a git worktree dir: a done result caps with the built report and the verify-match passes; empty files_changed falls back to git paths; each refusal; the cell still needs the commit trailer as today. Cite decision d248942f (contract:cells-finish-from-job). No code comments.",
    "must_haves": {
      "truths": [
        "bee cells finish --from-job with --proof-result and a mistakes answer caps a cell whose job result is done, with tests built from the cell verify, the leader's proof result and the worker proof text",
        "empty files_changed falls back to the git changed paths of the job's working directory, and the cap output names the source",
        "--from-job refuses with --report, without --proof-result, without a mistakes answer, with a non-green proof result, with no result, with a non-done result, with another cell's job, with a missing working directory, and with empty proof text and no --proof-reason",
        "the existing cells finish and cells cap paths and their tests are unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
          "substantive": "report_from_job builder"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
          "substantive": "--from-job wiring into the cap path"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
          "substantive": "from-job tests"
        }
      ],
      "key_links": [
        "run_finish builds the report from the job before parse_report_flag runs"
      ],
      "prohibitions": [
        "No code comments",
        "No cap without --proof-result",
        "No change to cells cap"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee catalog",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "hlt-2",
    "feature": "herding-leader-toil",
    "lane": "high-risk",
    "role": "code",
    "title": "Serve the main control plane for allow-listed verbs inside a granted worktree",
    "deps": [],
    "decisions": [
      "D2"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/util.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "read_first": [
      "docs/history/herding-leader-toil/CONTEXT.md",
      "docs/history/herding-leader-toil/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/advisor_ref.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/roots.rs",
      "docs/knowledge/areas/worktree-parallelism/control-plane-topology.md"
    ],
    "action": "Red first. Per D2 (revised): do NOT change verbs/reservations/emit.rs prelude or resolve_store_root. Use the existing serve-granted resolvers in verbs/drivers/prepare.rs (resolve_root_serving_granted and ctx_serving_granted, already used by state_group/advisor_ref.rs) as an opt-in: (1) state_group's entry (verbs/state_group/set_gate.rs, search `go` and its prelude call) serves granted worktrees for every state verb, and the gate and route aliases with it; (2) verbs/drivers/close.rs: both root resolutions (search resolve_store_root and prelude) serve granted worktrees, and its reads of docs/history/<feature>/ and docs/knowledge go through the worktree found for the feature (reuse find_granted_worktree_for_feature or the advisor_plan_path approach) so a worktree close judges the worktree's docs; (3) cells: add a dispatch_serving_granted twin next to dispatch in verbs/cells/handlers_write.rs and route ONLY add, update, schedule, escalate, reroute, judge, judge-record, dissent, dissent-verdict and leader-check through it (find the routing in verbs/cells/util.rs try_mutating and the handlers that call dispatch); every other cells verb keeps the existing refusal untouched; list, ready and show in verbs/cells/mod.rs serve granted worktrees too. cells finish stays on its full door. dispatch prepare --claim, dispatch wave and dispatch authorize keep refusing. Gate and plan reads already resolve the worktree plan through advisor_plan_path: add a test, no second resolver. A served call's text output gains one line `control plane: <main root>`; a JSON object output gains control_root; JSON array outputs (cells list, ready) are unchanged. Tests (inline #[cfg(test)] modules in the touched files, not verbs/cells/tests.rs): from a granted worktree, state route --set, state set, gate --preview (reads the worktree plan.md), cells add, cells list and close --dry-run (reads the worktree CONTEXT.md) succeed against main's store; cells claim, claim-next, unclaim, reopen, cap, block, drop and rebind-session still refuse with the GrantedWorktree text; cells finish still caps from the worktree; the main checkout and an ungranted worktree behave as before. Cite decision e32f3a66 (contract:control-plane-from-worktree). No code comments.",
    "must_haves": {
      "truths": [
        "state verbs, gate, route, close and cells add/list/show/ready/update/schedule/escalate/reroute/judge/judge-record/dissent/dissent-verdict/leader-check run from a granted worktree against the main store and name the main root",
        "close and gate run from a granted worktree read the worktree's docs/history files",
        "every other cells verb, dispatch prepare --claim, dispatch wave and dispatch authorize still refuse inside a granted worktree with the existing text, and cells finish still caps there",
        "JSON array outputs are unchanged; JSON object outputs gain control_root when served",
        "the main checkout and ungranted worktrees behave exactly as before; the shared prelude and resolve_store_root are unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
          "substantive": "dispatch_serving_granted for the allow-listed cells verbs"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
          "substantive": "close served with worktree docs reads"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
          "substantive": "state entry served"
        }
      ],
      "key_links": [
        "only allow-listed verbs call the serve-granted resolver"
      ],
      "prohibitions": [
        "No code comments",
        "No change to verbs/reservations/emit.rs prelude or resolve_store_root",
        "No per-worktree cells store",
        "No change to cells finish"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/worktree-parallelism/control-plane-topology.md"
    ]
  },
  {
    "id": "hlt-3",
    "feature": "herding-leader-toil",
    "lane": "high-risk",
    "role": "code",
    "title": "Record a judge's fenced verdicts straight from its answer",
    "deps": [
      "hlt-1"
    ],
    "decisions": [
      "D3"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/judge.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "skills/bee-hive/references/gates-and-delegation.md"
    ],
    "read_first": [
      "docs/history/herding-leader-toil/CONTEXT.md",
      "docs/history/herding-leader-toil/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/judge.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs"
    ],
    "action": "Red first. Per D3: in verbs/cells/judge.rs add a pure extract_fenced_verdicts(text) -> Result<Vec<(String, Value)>, String> that finds every fenced block whose opening line is three backticks then `json <cell-id>` (exactly one id token), takes the body to the closing fence, and parses it with the same JSON parser run_judge_record uses; a duplicate id is an error naming it; zero blocks is an error ('no json <cell-id> verdict block found'). In handlers_meta.rs factor the body of run_judge_record after parsing into record_one(...) and add --from-text <path|-> (stdin for -), mutually exclusive with --file; with --from-text, --id becomes optional and, when given, every block id must equal it. Record each valid block on its cell through record_one (claim guards and archive checks per cell, as today); output a JSON array of {id, recorded, errors} (text: one line per id); exit non-zero when any block failed validation or recording. Declare --from-text for cells.judge-record in catalog.rs and generated/registry_payload.json like the existing flags and keep those tests green. In skills/bee-hive/references/gates-and-delegation.md, in the Judge tier paragraph that says the judge returns the judge-verdict/1 schema recorded via cells judge-record, add one sentence: the judge returns one fenced block per cell whose info string is `json <cell-id>`, and the leader records them all with `bee cells judge-record --from-text`. Zero blocks refuse before recording anything. Tests (inline in judge.rs and handlers_meta.rs or the existing judge tests): two valid blocks record two verdicts; an invalid block is reported and the exit is non-zero while the valid one records; duplicate ids refuse; --id mismatch refuses; --file with --from-text refuses; an answer with no blocks refuses. Cite decision e7509db5 (contract:judge-record-from-text). No code comments.",
    "must_haves": {
      "truths": [
        "cells judge-record --from-text records every valid json <cell-id> fenced verdict on its cell",
        "an invalid block is reported by id with the validator errors and the verb exits non-zero",
        "zero blocks, duplicate ids, an --id mismatch, and --file with --from-text refuse by name",
        "the judge tier paragraph tells the judge to return one json <cell-id> block per cell",
        "cells judge-record --file behaves as before"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/judge.rs",
          "substantive": "extract_fenced_verdicts"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs",
          "substantive": "--from-text and record_one"
        }
      ],
      "key_links": [
        "run_judge_record and --from-text share record_one"
      ],
      "prohibitions": [
        "No code comments",
        "No change to the judge-verdict/1 schema"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee catalog",
    "affects_skills": [
      "skills/bee-hive/references/gates-and-delegation.md"
    ],
    "affects_specs": []
  },
  {
    "id": "hlt-4",
    "feature": "herding-leader-toil",
    "lane": "high-risk",
    "role": "code",
    "title": "Wait on Paseo events and name workers by cell and agent",
    "deps": [],
    "decisions": [
      "D1",
      "D4",
      "D7"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "read_first": [
      "docs/history/herding-leader-toil/CONTEXT.md",
      "docs/history/herding-leader-toil/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "Red first. paseo.rs: add wait_argv(id, timeout_secs) = [agent, wait, <id>, --timeout, <n>, --json] and parse_wait(stdout) -> Option<WaitStatus> (Idle|Permission|Error|Timeout) reading the first JSON object's status, skipping leading non-JSON lines like parse_run_agent_id does. run_argv (D7): the title becomes `<cell-id> <agent>` when the run has a cell id, else `<agent> <job-id>`; add --label bee_agent=<agent> and, when a cell id is set, --label bee_cell=<cell-id>, keeping --label bee_job=<job-id>; thread the cell id and agent name from the call sites (search run_argv). run.rs (D4): introduce a WaitSource trait with arm(timeout), poll() -> Option<WaitStatus> and cancel() for wait_for_round_paseo_driven: the real one spawns paseo agent wait as a std::process::Child with piped stdout (never through the 15 s CLI and never FailFastPaseoCli), poll uses try_wait on the existing 200 ms tick and parses stdout when the child exits, cancel kills and reaps the child; cancel runs on every round exit (result, mark, timeout, died) through a drop guard and before every re-arm; no thread, no channel. The tick maps Idle -> PaseoState::Idle, Permission -> Blocked, Error -> Dead, Timeout -> Working (heartbeat fresh). Arm the first wait after the one post-spawn inspect (which still records the model, paseo-pi-hardening D4); re-arm only after a Timeout or after Idle handling, never sooner than 3 s after the previous wait returned; while Blocked, and after an Error until an inspect reports a state, do not arm a wait and fall back to one inspect every 3 s (so the existing three-read died debounce sees real reads). Use a wait timeout of 60 s, or the remaining idle budget when smaller. Keep the silent-idle nudge and second-idle end, the mailbox checks, the mark checks and the idle and ceiling timeouts exactly as they are. Tests inject a fake WaitSource and the driven clock: Idle after Working with no result still nudges once and ends on the second idle; a Timeout keeps the run alive; Permission then Idle; an Error switches to the inspect fallback and three Dead inspects reach the died path; no wait is armed while Blocked; cancel kills the child on every round exit; the post-spawn inspect records the model; title and labels for a cell run and a plain run. Also change the `worker reported success for cell ... without capping it` line to name `bee cells finish --id <cell> --from-job <job-id> --proof-result <green:...>` (D1). Cite decision a0c36961 (contract:paseo-agent-wait). No code comments.",
    "must_haves": {
      "truths": [
        "the Paseo wait loop learns idle, permission, error and timeout from a paseo agent wait child it polls on the 200 ms tick and kills on every round exit",
        "a wait is re-armed only after a timeout or idle handling and never sooner than 3 s; while blocked and after an error the loop uses one inspect every 3 s instead",
        "the post-spawn inspect still records the observed model, and the silent-idle nudge, mailbox checks and timeouts are unchanged",
        "a worker's Paseo title is <cell-id> <agent> or <agent> <job-id> with bee_agent and bee_cell labels beside bee_job",
        "the uncapped-success line names bee cells finish --from-job",
        "herdr and tmux runs are unchanged and the existing herding tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/run.rs",
          "substantive": "WaitSource-driven Paseo wait loop"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/herding/paseo.rs",
          "substantive": "wait_argv, parse_wait, title and labels"
        }
      ],
      "key_links": [
        "wait_for_round_paseo_driven drains the WaitSource channel each tick"
      ],
      "prohibitions": [
        "No code comments",
        "No thread or channel for the wait",
        "No paseo agent wait through FailFastPaseoCli or the 15 s CLI",
        "No change to herdr or tmux paths"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "hlt-5",
    "feature": "herding-leader-toil",
    "lane": "high-risk",
    "role": "code",
    "title": "Give an opted-in Pi worker its own Pi config folder",
    "deps": [
      "hlt-1",
      "hlt-4"
    ],
    "decisions": [
      "D5"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/pi_agent_dir.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/herding-leader-toil/CONTEXT.md",
      "docs/history/herding-leader-toil/plan.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "action": "Red first. Per D5: paseo.rs PaseoSpec gains isolated_config: bool from the paseo block (default false; ignored unless provider is pi). New module herding/pi_agent_dir.rs (declare it in herding.rs): ensure_pi_agent_dir(main_root, agent, home) -> Result<(PathBuf, Vec<String>), String> creates <main_root>/.bee/runtime/pi-agent/<agent>/ idempotently: symlink auth.json to <home>/.pi/agent/auth.json (refuse with FIX 'log in with pi once' when that file is missing), and symlink models.json, models-store.json and npm when their sources exist; write settings.json {\"defaultProjectTrust\":\"always\",\"quietStartup\":true} when absent or different; create empty skills/ and extensions/; never replace an existing real file or directory where a link belongs — return a note for it instead. run.rs: in the Paseo spawn path (search build_child_env and run_argv), when the spec has isolated_config, call ensure_pi_agent_dir and add PI_CODING_AGENT_DIR=<dir> to the child env (so it becomes a --env flag); a refusal ends the run as SpawnFailed with the FIX line; notes go to stderr. Tests with a temp home and root: links and settings created; idempotent second call; a real file in place of a link is kept and reported; missing auth refuses; the argv carries --env PI_CODING_AGENT_DIR only when isolated_config is true and provider is pi. In the live proof the leader checks that auth.json stays a link after a worker runs. Cite decision db780128 (contract:pi-isolated-config). No code comments.",
    "must_haves": {
      "truths": [
        "a pi-provider Paseo agent with isolated_config true gets .bee/runtime/pi-agent/<agent>/ with auth, optional model and npm links, trust-always settings and empty skills and extensions",
        "the worker receives PI_CODING_AGENT_DIR pointing at that folder; agents without the flag are unchanged",
        "an existing real file or directory where a link belongs is never replaced and is reported",
        "a missing ~/.pi/agent/auth.json refuses the spawn with a FIX line",
        "the existing herding tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/pi_agent_dir.rs",
          "substantive": "ensure_pi_agent_dir"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/herding/run.rs",
          "substantive": "PI_CODING_AGENT_DIR injection"
        }
      ],
      "key_links": [
        "the Paseo spawn path calls ensure_pi_agent_dir for isolated_config agents"
      ],
      "prohibitions": [
        "No code comments",
        "No change for agents without isolated_config",
        "No copy of auth.json"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "hlt-6",
    "feature": "herding-leader-toil",
    "lane": "high-risk",
    "role": "code",
    "title": "Catch the AppImage paseo CLI and a broken isolated Pi folder in doctor",
    "deps": [],
    "decisions": [
      "D6"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs"
    ],
    "read_first": [
      "docs/history/herding-leader-toil/CONTEXT.md",
      "docs/history/herding-leader-toil/plan.md",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "action": "Red first. Per D6, in doctor.rs paseo_ready_row_with_env_and_cli (search paseo_ready): after the --version call, fail with FIX 'set herding.paseo.command to the npm @getpaseo/cli paseo' when the first non-blank stdout line is not a bare x.y.z version, or when the command resolved on PATH (or given as a path) is a text script containing 'AppImage'. For each herding agent whose raw config paseo block has provider pi and isolated_config true, when <root>/.bee/runtime/pi-agent/<agent>/ exists, check its settings.json has defaultProjectTrust always and its auth.json is a symlink that resolves (a real file or a broken link fails); when the folder does not exist yet, only check that ~/.pi/agent/auth.json exists; fail with a FIX naming the agent otherwise (read the raw config keys; do not depend on new structs). Tests in doctor/tests.rs with the fake CLI and temp files: AppImage-style version output fails; an AppImage wrapper script fails; the npm style passes; a broken isolated folder fails; a replaced (non-link) auth.json fails; a missing folder with the home auth present passes; a good one passes. Cite decision a79e3edc (contract:doctor-paseo-cli). No code comments.",
    "must_haves": {
      "truths": [
        "paseo_ready fails with the npm FIX when --version's first line is not a bare version or the command is an AppImage wrapper script",
        "paseo_ready fails when an existing isolated_config folder lacks trust-always settings or an auth.json that is a working link, and only checks the home auth when the folder does not exist yet",
        "the npm CLI with good folders still passes and the existing doctor tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/doctor.rs",
          "substantive": "AppImage and isolated-folder checks"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/doctor/tests.rs",
          "substantive": "tests for both"
        }
      ],
      "key_links": [
        "paseo_ready runs the new checks"
      ],
      "prohibitions": [
        "No code comments",
        "No state-changing call from doctor"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor",
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ]
  },
  {
    "id": "hlt-7",
    "feature": "herding-leader-toil",
    "lane": "high-risk",
    "role": "docs",
    "title": "Document from-job caps, worktree control verbs, judge text, Paseo wait and isolated Pi folders",
    "deps": [
      "hlt-1",
      "hlt-2",
      "hlt-3",
      "hlt-4",
      "hlt-5",
      "hlt-6"
    ],
    "decisions": [
      "D1",
      "D2",
      "D3",
      "D4",
      "D5",
      "D6",
      "D7"
    ],
    "files": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/worktree-parallelism/control-plane-topology.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/config-reference.md"
    ],
    "read_first": [
      "docs/history/herding-leader-toil/CONTEXT.md",
      "docs/history/herding-leader-toil/plan.md"
    ],
    "action": "Update docs to match the shipped code, citing herding-leader-toil D1-D7 with store ids from bee decisions search --text herding-leader-toil. the-paseo-channel.md: the event wait (D4), the per-agent Pi folder and isolated_config (D5), the doctor AppImage and folder checks (D6), worker titles and labels (D7), and that a herded run's uncapped-success line names cells finish --from-job (D1). control-plane-topology.md: which verbs now serve main from a granted worktree and which still refuse (D2, narrowing d7b83394). skills/bee-swarming/references/swarming-reference.md: in the tending and cap steps for herded workers, the leader checks the work, then caps with bee cells finish --id <cell> --from-job <job> --proof-result <green:...>; the slice judge's answer is recorded with bee cells judge-record --from-text (D1, D3). docs/config-reference.md: herding.agents.<name>.paseo.isolated_config. Plain technical English, no deferral words.",
    "must_haves": {
      "truths": [
        "the Paseo channel concept describes the event wait, isolated Pi folders, the doctor checks and worker titles with their decision ids",
        "control-plane-topology.md names the verbs served from a granted worktree and those that still refuse",
        "the swarming reference tells the leader to cap herded cells with --from-job after checking and to record judges with --from-text",
        "the config reference names isolated_config",
        "bee knowledge check passes"
      ],
      "artifacts": [
        {
          "path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
          "substantive": "event wait, isolated folders, doctor, titles"
        },
        {
          "path": "docs/knowledge/areas/worktree-parallelism/control-plane-topology.md",
          "substantive": "served verbs"
        },
        {
          "path": "skills/bee-swarming/references/swarming-reference.md",
          "substantive": "from-job and from-text steps"
        },
        {
          "path": "docs/config-reference.md",
          "substantive": "isolated_config"
        }
      ],
      "key_links": [
        "each doc cites herding-leader-toil decision ids"
      ],
      "prohibitions": [
        "No change outside these docs"
      ]
    },
    "verify": ".bee/bin/bee knowledge check --json",
    "affects_skills": [
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
      "docs/knowledge/areas/worktree-parallelism/control-plane-topology.md"
    ]
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Happy | done result, leader passes green:unit | hlt-1 | cell capped; tests line = verify — green:unit — worker proof |
| Error | no --proof-result; other cell's job; no result | hlt-1 | typed refusal, cell stays claimed |
| Topology | route --set and cells add from a granted worktree | hlt-2 | exit 0; main store changed; output names main |
| Topology | cells claim from a granted worktree | hlt-2 | GrantedWorktree refusal as before |
| Artifact | gate --preview from a granted worktree | hlt-2 | reads the worktree plan.md |
| Parsing | two good blocks, one bad block | hlt-3 | two recorded, one reported, exit non-zero |
| Event | Timeout, Idle, Permission, Error from the fake source | hlt-4 | Working, nudge path, Blocked with inspect fallback, died debounce |
| Spin | Idle returned at once twice | hlt-4 | no re-arm sooner than 3 s |
| Files | real file where a link belongs | hlt-5 | kept and reported |
| Setup | AppImage version output; wrapper script | hlt-6 | paseo_ready fails with the npm FIX |
| Live | after merge: one Paseo Pi worker with isolated_config in a sandbox, then cells finish --from-job | leader | worker runs with PI_CODING_AGENT_DIR, Paseo title is cell + agent, cap succeeds |

## Open Questions

(none)

## Out of scope

- Turning isolated_config on in this repo's own config: the config guard keeps that a user edit.
