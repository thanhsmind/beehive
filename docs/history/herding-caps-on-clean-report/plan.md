---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: One runnable finish instruction

## Summary

A dispatched worker does its job, commits, runs its tests — and then does not
record the result. Five in a row did exactly that. The plan-step review found
why, and it is not the worker: the task text hands it two different finish
instructions, and the one it meets first does not run.

This gives the worker one instruction, runnable as written, at the end of the
prompt where the last thing asked is the thing that finishes the job. Then it
drives a real worker to see whether the cell comes back capped on its own.

Mode: `high-risk` — inherited, and it does not demote. The lane no longer matches
the work's size (one flag, two template files), but a high-risk route never steps
down, so the ceremony stands.
Why the ceremony still earns its place: this is the text every execution worker on
every runtime reads, and this repo already carries a pattern about instruction
text being an untested code path.

## Requirements (from CONTEXT.md)

CONTEXT.md's D1-D7 described the runner auto-cap and are superseded by decision
`7152ebab`. The live requirements are that decision's:

- **R1** A worker is handed exactly ONE finish instruction.
- **R2** That instruction is runnable as written — it carries `--id` and the `.bee/bin/` prefix.
- **R3** It is the last thing the prompt asks of the worker.
- **R4** The runner does NOT cap on the worker's behalf.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The agent contract hands the worker an unrunnable finish command. | read | `packages/bee/agents/bee-build.md.tmpl:15` | the bullet ends `then` + the finish verb carrying only a report flag — no `--id` and no `.bee/bin/` prefix. The exact spelling is deliberately not reproduced here: it is a command shape the CLI-shape guard refuses, and shipping it in a doc trips the guard's own scan |
| 2 | The command in claim 1 is refused before it runs, because `--id` is required. | ran | bee's CLI-shape guard, on a `cells finish` call carrying no `--id` | `does not match cells.finish's schema — required, missing (--id) (field: id).` |
| 3 | The correct instruction exists in the other prompt, ~200 words later and mid-bullet. | read | `packages/bee/prompts/worker-cell.md:49` | `Finish with: .bee/bin/bee cells finish --id {{cell_id}} --outcome "<one line>" --files <a,b> --report '<json>'` |
| 4 | Both reach one worker: the worker-cell prompt is the task text. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1125` | `let Some(template) = load_prompt("worker-cell") else { return Err(Delegate) };` |
| 5 | The template renders to the agent file a Task-shaped worker reads. | ran | `fd -t f 'bee-build.md' .claude` | `.claude/agents/bee-build.md` |
| 6 | The runner cannot legally cap for the worker, so R4 is a constraint and not a preference. | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:466` | `MailboxResult` carries `round, status, summary, files_changed, proof, options, leaning` — no `commit`, no `deviations`, no `mistakes` |
| 7 | A legal cap needs keys the envelope does not carry. | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:57` | the required report keys are `outcome, commit, files, tests, deviations`, with `outcome` and `commit` non-empty |

## Discovery

The wave's full record is `docs/history/herding-caps-on-clean-report/hat-synthesis.md`.
It re-shaped this feature from a runner change to an instruction fix, and it
recorded the four blockers that stop the runner route, so a later attempt starts
from them rather than rediscovering them.

One correction the wave forced on the leader's own record: the backlog row filed
at the previous feature's close reads *"workers commit and test but never cap"*,
which blames the worker for following an instruction that does not run.

## Approach

**Recommended path.** One slice, two cells. `wci-1` makes the instruction single
and runnable and moves it last, then runs the regen chain the rendered trees need.
`wci-2` dispatches a real worker for a throwaway cell and asserts the cell comes
back capped with nobody touching it.

Rejected alternatives:
- Build the runner auto-cap — rejected by `7152ebab` and by claims 6-7; it cannot be built without the runner authoring fields about a worker it only observed.
- Fix only the template and leave the prompt ordering — rejected by R3; the correct command already existed and was still missed, so making it single is necessary but not obviously sufficient.

**SMALLER PATH check.** Could this ship as one cell, the edit alone? No. The whole
claim is that a worker will now do the step, and the only thing that shows it is a
worker doing it. An edit with no drive is the same shape as the inert tool gate
this session already shipped once. FAIL. Kept at two cells.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Prompt text every worker reads | MEDIUM | `wci-1` | exactly one finish instruction in the composed text, and it is the runnable one |
| Rendered trees drifting from source | MEDIUM | `wci-1` | regen run; release-manifest check clean |
| The change not actually working | HIGH | `wci-2` | a real dispatched worker caps its own cell, `green:live` |

Waves: `wci-1` then `wci-2`, serial.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "planning", "classification": "required", "role": "plan", "reason": "High-risk lane, inherited and non-demotable."},
    {"stage": "implementation-code", "classification": "conditional", "role": "code", "condition": "wci-2 shows a worker still skipping a runnable, single instruction, so the fix has to move back into code", "reason": "This feature deliberately changes prompt text rather than code. The code role stays configured because the measurement in wci-2 can send the work back there."},
    {"stage": "implementation", "classification": "required", "role": "docs", "reason": "The change is prompt text in two template files, not code."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "The claim is that a worker now takes the step; only a live dispatch shows it."},
    {"stage": "documentation-and-capture", "classification": "conditional", "role": "docs", "condition": "a verify-app feature file needs the drive recipe", "reason": "Only if a verify-app feature file needs the recipe."},
    {"stage": "read-only-gather", "classification": "conditional", "role": "read", "condition": "execution finds prompt text beyond the two files already anchored", "reason": "The two files are already read and anchored."},
    {"stage": "fact-extraction", "classification": "conditional", "role": "extraction", "condition": "a single already-located fact is needed", "reason": "Cheap tier for narrow lookups."},
    {"stage": "generation-fallback", "classification": "conditional", "role": "generation", "condition": "a role with no configured slot is requested", "reason": "Fallback only."},
    {"stage": "independent-review", "classification": "conditional", "role": "review", "condition": "the user invokes a review", "reason": "Review is user-invoked."},
    {"stage": "generic-advisor", "classification": "required", "role": "advisor", "reason": "High-risk owes a consult; the hat-wave synthesis serves it and is what re-shaped this plan."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Single-leader feature."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No convergence lane."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No convergence lane."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No convergence lane."},
    {"stage": "hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "Ran; audited the claims table and found the cap door's unnamed preconditions."},
    {"stage": "hat-risks", "classification": "required", "role": "hat-risks", "reason": "Ran; found that an auto-cap moves the manual step to close rather than removing it."},
    {"stage": "hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "Ran; found the duplicated instruction that re-shaped the feature."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Recorded deviation: the owner chose the behaviour against a stated alternative with the measured failure rate in hand."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Recorded deviation: no user-facing surface changes; the audience for this text is the dispatched worker."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release is cut."}
  ]
}
```

## Shape

One slice: `wci-1` (the text and the regen), then `wci-2` (the live drive).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| `wci-1` | Give a worker one runnable finish instruction, last | `bee-build.md.tmpl`, `worker-cell.md`, rendered trees, release manifest | — | A dispatched worker is told once, in a command that runs, how to record its result | the composed text carries exactly one finish instruction; regen clean; `release-manifest --check` clean |
| `wci-2` | Dispatch a real worker and watch it cap itself | `.bee/verify/verify-app/features/cells-and-proof.md` | `wci-1` | A worker finishes and the cell is already capped — nobody touches it | `green:live` — a real dispatch, cell status read afterwards |

```json
[
  {
    "id": "wci-1",
    "feature": "herding-caps-on-clean-report",
    "title": "Give a worker one runnable finish instruction, last",
    "lane": "high-risk",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": ["7152ebab-5933-4af4-ae9e-2480656c49e7"],
    "files": [
      "packages/bee/agents/bee-build.md.tmpl",
      "packages/bee/prompts/worker-cell.md",
      ".claude/agents/bee-build.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "packages/bee/agents/bee-build.md.tmpl",
      "packages/bee/prompts/worker-cell.md",
      "docs/history/herding-caps-on-clean-report/hat-synthesis.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "A dispatched worker is currently handed TWO finish instructions and meets the broken one first. packages/bee/agents/bee-build.md.tmpl:15 states the finish verb with only a report flag and no `--id`, which has no --id and no .bee/bin/ prefix and therefore refuses; packages/bee/prompts/worker-cell.md:49 carries the correct full command. Fix it so the worker is told ONCE, runnably, and last. In the tmpl: stop stating the command. That file's job is to say what the agent is FOR, so it should name the step in words and point at the one home that owns the command — it must not carry a second, competing spelling (one-fact-one-home). In worker-cell.md: keep the command as the single home, and make it the LAST thing the prompt asks of the worker. Today the Result form block comes after it, so the final instruction a worker reads is to emit a JSON block, and it emits that block and stops. Reorder so the cap is terminal and the status token plus JSON reads as the ECHO of having capped, not as the finishing act. Do not weaken or drop anything the existing text requires: the proof line, the red-refuses-the-cap rule, the mistakes answer, and the report keys all stay exactly as they are — this cell changes WHERE and HOW MANY TIMES the instruction appears, never what it demands. Then run the regen chain, because these files render into trees the release manifest hashes: bee dev regen (render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write, in that order), and commit the refreshed manifest and rendered agent file with the change. Prove it by composing what a worker would actually receive rather than by reading either file alone: run `.bee/bin/bee dispatch prepare --runtime claude --kind cell` for a real claimed cell and assert the returned task text contains exactly one `cells finish` command and that it carries both `--id` and the `.bee/bin/` prefix.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee onboard && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "The composed worker task text contains exactly one cells finish command",
        "That command carries --id and the .bee/bin/ prefix, so it runs as written",
        "The cap is the last thing the prompt asks of the worker",
        "Nothing the existing instruction demands is weakened or dropped",
        "The rendered trees and the release manifest match the source"
      ],
      "artifacts": [
        {"path": "packages/bee/agents/bee-build.md.tmpl", "substantive": "names the step and points at its one home; carries no competing command spelling"},
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "the single home for the command, reordered so the cap is terminal"},
        {"path": ".claude/agents/bee-build.md", "substantive": "regenerated from the template"}
      ],
      "key_links": ["the proof is taken from the COMPOSED dispatch text, not from either file read alone"],
      "prohibitions": [
        "No second spelling of the finish command anywhere in the composed text",
        "No weakening of the proof line, the red rule, the mistakes answer or the report keys",
        "No change to the runner: it still does not cap for the worker"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "wci-2",
    "feature": "herding-caps-on-clean-report",
    "title": "Dispatch a real worker and watch it cap itself",
    "lane": "high-risk",
    "role": "test",
    "status": "open",
    "deps": ["wci-1"],
    "decisions": ["7152ebab-5933-4af4-ae9e-2480656c49e7"],
    "files": [".bee/verify/verify-app/features/cells-and-proof.md"],
    "read_first": [
      ".bee/verify/verify-app/features/cells-and-proof.md",
      ".bee/verify/verify-app/features/README.md",
      "docs/history/herding-caps-on-clean-report/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [".bee/verify/verify-app/features/cells-and-proof.md"],
    "action": "The claim is that a worker will now take the step. Only a worker taking it proves that, so drive one. Rebuild and install the binary at .bee/bin/bee first; a stale vendored copy makes the run worthless. Against a launched control-bee sandbox: create one trivial cell, claim it, dispatch a real execution worker for it through the door, and let it run to completion. Then read the cell WITHOUT capping anything by hand and record what you find: status, trace.outcome, trace.capped_at and the recorded proof line. A cell that comes back capped by its own worker is the result this feature exists for. If it comes back claimed, that is a real finding and NOT a failure of this cell — record exactly what the worker did and did not do, quote the finish instruction it actually received from the composed task text, and report it rather than capping by hand to make the run look clean. Either outcome gets written into the verify-app feature file that owns cells and proof as a new sub-feature, following that file's existing four-H2 contract, with the exact commands and assertions. Record the proof line as green:live.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee onboard",
    "must_haves": {
      "truths": [
        "A real execution worker was dispatched for a real claimed cell",
        "The cell's post-run state is read and recorded without any manual cap",
        "The finish instruction the worker actually received is quoted from the composed text",
        "The outcome is reported honestly whether or not the worker capped"
      ],
      "artifacts": [
        {"path": ".bee/verify/verify-app/features/cells-and-proof.md", "substantive": "a sub-feature covering the self-cap drive, with commands, assertions and what was observed"}
      ],
      "key_links": ["the drive runs the rebuilt binary installed at .bee/bin/bee"],
      "prohibitions": [
        "Never cap the drive's cell by hand to make the result look clean",
        "No claim of green without the fresh command output beside it"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| # | Dimension | Scenario | Pass when |
|---|---|---|---|
| 1 | Happy path | Compose a real cell dispatch text | exactly one `cells finish` command appears |
| 2 | Boundary | That command's shape | carries `--id` and the `.bee/bin/` prefix |
| 3 | Ordering | Position of the cap instruction | it is the last thing asked of the worker |
| 4 | Regression | The instruction's demands | proof line, red rule, mistakes answer and report keys all still required |
| 5 | Regression | Rendered trees | regen clean, `release-manifest --check` clean |
| 6 | Live | A real dispatched worker | the cell is capped by its own worker, with no manual step |
| 7 | Live, negative | The worker still does not cap | reported honestly with the received instruction quoted, never hand-capped |

## Open Questions

- Whether a single runnable instruction is SUFFICIENT is exactly what `wci-2` measures. If a worker still skips it with a runnable command in front of it, the runner route returns — and `hat-synthesis.md` records the four blockers it must solve first.

<!-- bee:not-a-deferral: This section lists what this feature deliberately does not change, so a reader knows the boundary. Each line is a superseded decision or a recorded blocker, not work postponed. -->
## Out of scope

- The runner capping on a worker's behalf. Superseded by `7152ebab`; its blockers are recorded in `hat-synthesis.md`.
- `trace.capped_by`. It belongs with the auto-cap, which is not being built.
- Anything the finish instruction demands. This feature moves the instruction; it does not soften it.
<!-- /bee:not-a-deferral -->
