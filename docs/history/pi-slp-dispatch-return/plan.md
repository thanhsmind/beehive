---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: pi-slp-dispatch-return

## Summary
A worker that bee starts in a separate pane gets two orders today: "cap the
cell with bee" and "never run bee". A small model obeys one of them at random.
After this change the worker gets one order: do the work, commit, report. The
leader caps.

On Pi, three more gaps close. The dispatch tool can send the stage and the
feature that bee needs. The worker's result tool refuses to guess which job it
belongs to and refuses an empty proof. The leader's result message says that
the cell still needs a cap.

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: two disjoint cells, one per language, each with its own focused test.

## Requirements (from CONTEXT.md)
- D1: a herding cell dispatch renders a task with no bee bookkeeping; the leader caps; native dispatch is byte-identical.
- D2: `bee_dispatch` passes stage, feature, claim, expertise; `bee_advisor` passes stage, feature; prepare validates.
- D3: the verdict tool refuses with no job id, refuses a done status with a blank proof, refuses a second result for the same round; each refusal names one fix.
- D4: an injected done result with a `cell_id` adds one fixed line outside the fence saying the leader must check and cap.

## Load-bearing claims
Labels: `read` (opened at the anchor) or `ran`. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The herding wrapper forbids every bee command | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:367 | Never run any `bee` command. Never claim, cap, or write workflow state under .bee/ |
| 2 | The cell prompt orders the worker to cap | read | packages/bee/prompts/worker-cell.md:68 | Finish with: .bee/bin/bee cells finish --id {{cell_id}} |
| 3 | A cell pane prompt prepends the bee-build body | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2359 | Some(body) => format!("{body}\n\n{prompt_body}"), |
| 4 | herding run tells the leader to cap an uncapped done cell | read | packages/bee-rs/crates/bee/src/herding/run.rs:4195 | without capping it — settle with `bee cells finish` |
| 5 | Prepare already reads stage and feature flags | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3426 | let stage_flag = flags.truthy_str("stage") |
| 6 | The Pi tool sends only five flags | read | .pi/extensions/bee-guard/tool-dispatch.ts:91 | runBeeDispatch(flagArgs(params, ["kind", "role", "cell", "worker", "purpose"]), ctx), |
| 7 | herding run always exports the job id to the worker | read | packages/bee-rs/crates/bee/src/herding/run.rs:2564 | pane_env.insert("BEE_HERDING_JOB_ID".to_string(), opts.job_id.clone()); |
| 8 | The verdict tool falls back to the newest mailbox | read | .pi/extensions/bee-guard/tool-verdict.ts:120 | dirs.sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs) |
| 9 | The verdict tool checks only that proof is a string | read | .pi/extensions/bee-guard/tool-verdict.ts:92 | if (typeof params.proof !== "string") { |
| 10 | The injection already carries the cell id | read | .pi/extensions/bee-guard/result-inbox.ts:215 | push("cell_id", marker.cell_id) |
| 11 | Templates support `{{#if}}` blocks only | read | packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:149 | blocks are consumed WITH the newline |

## Discovery
Read the herding wrapper, the cell prompt, the pane-prompt assembly, the
postflight cap check, the Pi dispatch, verdict and result-inbox tools, and the
herding env export. The research report's isolated run showed the verdict
tool accepting an empty proof and writing into an unrelated mailbox
(`docs/history/research/seatworks-slp-pi-small-models.md`, "Evidence").

## Approach
Playbook: `skills/bee-planning/playbooks/bugfix.md` (class bugfix) — red test first in each cell.

Recommended path: in prepare, when the resolved transport is herding and the
kind is cell, skip the bee-build body and render worker-cell with a herding
flag that swaps the bee bookkeeping blocks for a herding block (D1, claims
1-3, 11). In the Pi extension, widen the two tool schemas (D2, claims 5-6),
remove the verdict fallback and add the two refusals (D3, claims 7-9), and add
the one fixed line to the injection (D4, claim 10).

Rejected: a new prompt file for herding cells — a second copy of the cell
fields drifts from the first. Moving the cap into `herding run` — that makes
bee accept a worker's word as completion, which the leader check forbids.

Risk map:
- worker-cell template / MEDIUM / psd-1 / native render byte-identical test plus herding render test
- verdict tool / LOW / psd-2 / node-driven contract tests for each refusal
- dispatch schema / LOW / psd-2 / contract test that stage and feature reach the prepare argv

Waves: psd-1 and psd-2 in parallel — disjoint files, no shared output.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"psd-1 and psd-2 change Rust and TypeScript with their tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own red-first tests."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader captures at close."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Standard lane; the hat wave is the plan check."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Plan-step hat wave."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Plan-step hat wave."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"Plan-step hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Plan-step hat wave."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Plan-step hat wave."}
  ]
}
```

## Shape
Phase plan:

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | A herded cell worker gets one order; Pi tools carry the prepare inputs and bind results to their job | A small model on a herding pane cannot follow two opposite orders | `bee dispatch prepare --kind cell` on a herding role prints a stdin with no `cells finish`; the verdict tool refuses with no job id | Slice 2: per-role operation packet |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| psd-1 | Give a herded cell worker one order | prepare.rs, worker-cell.md, drivers/tests.rs | — | A herding cell brief has no bee commands; a native brief is unchanged | cargo test --release -p bee --bin bee verbs::drivers |
| psd-2 | Bind Pi dispatch and verdict calls to their job | tool-dispatch.ts, tool-verdict.ts, result-inbox.ts, pi_plugin_contracts.rs | — | Pi sends stage and feature; the verdict tool refuses a guess or a blank proof; the leader is told to cap | cargo test --release -p bee --test pi_plugin_contracts |

```json
[
  {
    "id": "psd-1",
    "feature": "pi-slp-dispatch-return",
    "lane": "standard",
    "role": "code",
    "change_class": "bugfix",
    "title": "Give a herded cell worker one order",
    "deps": [],
    "decisions": ["D1", "4029e89b-67d7-4599-812d-6bef59a39dac"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee/prompts/worker-cell.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-dispatch-return/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1. In prepare_dispatch, the pane prompt is built at `let pane_prompt = if is_deployment {` (prepare.rs:2351); its else arm is `Some(body) => format!(\"{body}\\n\\n{prompt_body}\"),` (prepare.rs:2359). When the kind is cell and the resolved transport is `Resolved::Herding { .. }`, do not prepend the bee-build body, and render the worker-cell template with a herding flag. prompt_body_for (prepare.rs:1071, `pub(crate) fn prompt_body_for(`) renders worker-cell at `let Some(template) = load_prompt(\"worker-cell\")` (prepare.rs:1125); thread the flag in there. Templates support only `{{#if NAME}}` blocks with no else and no nesting (prompt.rs:149). In packages/bee/prompts/worker-cell.md, wrap the bee bookkeeping parts — the `Contract:` list, the `Result form:` paragraph with its json block, and the `Finish with:` paragraph — in a block that renders only for native dispatch, and add a block that renders only for herding. The herding block says: implement only this cell inside the listed files; run the narrowest proof; commit once with an imperative subject and the literal last body line `cell: {{cell_id}}`; put a three-part proof line `<command> — <green:unit|green:live|green:static> — <scope reason>` in the result file's proof field and the commit sha in its summary; the leader checks the work and caps the cell. The herding block must not contain the word sequence `bee cells`, `.bee/bin/bee`, `reservations reserve` or `mailbox reflect`. Every other rendered byte of a native cell prompt stays identical. Write the tests red first in drivers/tests.rs: a herding cell dispatch stdin contains no `cells finish` and no `You are a bee execution worker`, and does contain the `cell: <id>` trailer instruction; a native cell dispatch prompt equals what it rendered before (it still contains `Finish with: .bee/bin/bee cells finish`). Look at existing herding cell tests in drivers/tests.rs (`rg -n \"herding\" packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs`) for the fixture that resolves a herding role. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::drivers",
    "must_haves": {
      "truths": [
        "A cell dispatched on a herding role carries a stdin with no bee command and no bee-build agent body",
        "The herding cell stdin tells the worker to commit with the cell trailer and return a three-part proof line",
        "A native cell dispatch prompt is byte-identical to before"
      ],
      "artifacts": [
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "native-only and herding-only blocks; herding block has no bee command"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "herding cell path skips the bee-build body and sets the herding flag"}
      ],
      "key_links": ["prepare_dispatch passes the herding flag into prompt_body_for for kind cell on Resolved::Herding"],
      "prohibitions": ["No change to the herding wrapper in mailbox.rs", "No change to native or Agent-tool cell prompts", "No new prompt file"]
    }
  },
  {
    "id": "psd-2",
    "feature": "pi-slp-dispatch-return",
    "lane": "standard",
    "role": "code",
    "change_class": "bugfix",
    "title": "Bind Pi dispatch and verdict calls to their job",
    "deps": [],
    "decisions": ["D2", "D3", "D4", "ee22fb67-deec-4013-807f-c2c7628c78c8", "180c5b3c-8963-4790-8286-389c8890f54f", "26c6bd37-c291-4d6f-a601-41643e343d38"],
    "files": [
      ".pi/extensions/bee-guard/tool-dispatch.ts",
      ".pi/extensions/bee-guard/tool-verdict.ts",
      ".pi/extensions/bee-guard/result-inbox.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-dispatch-return/CONTEXT.md",
      ".pi/extensions/bee-guard/tool-dispatch.ts",
      ".pi/extensions/bee-guard/tool-verdict.ts"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D2: in tool-dispatch.ts add `stage`, `feature`, `expertise` (strings) and `claim` (boolean) to the bee_dispatch parameters, and `stage`, `feature` to bee_advisor. Pass them through flagArgs (line 91: `runBeeDispatch(flagArgs(params, [\"kind\", \"role\", \"cell\", \"worker\", \"purpose\"]), ctx),`); `claim: true` becomes a bare `--claim`. No validation of values in TypeScript. Per D3: in tool-verdict.ts executeVerdictTool, delete the newest-mailbox fallback (the else branch with `dirs.sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)`, line 120); with no BEE_HERDING_JOB_ID throw an error that says the verdict tool works only inside a bee herding job and the fix is to finish with a normal final message. Throw when status is done and proof.trim() is empty, naming the fix: run the proof and pass `<command> — <result> — <scope reason>`. Throw when result-<round>.json already exists in the job mailbox, naming the fix: the result for this round is already recorded, end the turn. Each refusal writes nothing. Per D4: in result-inbox.ts renderResultInjection, when marker.cell_id is a non-empty string and result.status is done, append after the closing fence one fixed line: the cell <id> stays claimed until the leader checks the artifacts against the cell and caps it with `.bee/bin/bee cells finish --id <id>`. Use headerValue for the id. Other results render byte-identically. Tests red first in pi_plugin_contracts.rs (it drives the TypeScript with node; follow the node_or_skip! pattern and the job_mailbox / write_marker / write_result helpers near `fn job_mailbox(`): verdict with no job id refuses and writes no file even when an unrelated mailbox exists; done with blank proof refuses; a second call for a round with an existing result refuses; a valid call still writes result-<round>.json; bee_dispatch with stage and feature puts `--stage` and `--feature` in the prepare argv; the done-cell injection carries the cap line and a gather injection does not. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "verdict with no BEE_HERDING_JOB_ID refuses and writes no file",
        "verdict with status done and a blank proof refuses",
        "verdict refuses a second result for the same round",
        "bee_dispatch forwards stage, feature, claim and expertise to dispatch prepare",
        "a done cell result injection tells the leader to check and cap the cell"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard/tool-verdict.ts", "substantive": "no newest-mailbox fallback; three refusals each naming a fix"},
        {"path": ".pi/extensions/bee-guard/tool-dispatch.ts", "substantive": "schemas carry the new inputs and pass them through"}
      ],
      "key_links": ["bee_dispatch execute passes the new keys to runBeeDispatch", "renderResultInjection adds the cap line only for a done result with a cell_id"],
      "prohibitions": ["No workflow rule in the Pi extension beyond these refusals", "No change to the inbox claim, requeue or dedupe logic", "Injection of non-cell results byte-identical"]
    }
  }
]
```

## Test matrix
- Happy: a herding cell brief renders one order; a valid verdict writes its result; stage and feature reach prepare. Pass when the new tests in `verbs::drivers` and `pi_plugin_contracts` are green.
- Edge: a native cell prompt on main and on head renders the same bytes. Pass when the byte-identical native test is green.
- Error: verdict with no job id, a blank done proof, or a repeated round refuses and writes nothing. Pass when each refusal test sees the error text and no new file.

## Open Questions
(none)

## Out of scope
Plan-revision binding on results, the per-role operation packet, recovery and
the small-model measurement — slices 2 to 4 of the research report.
