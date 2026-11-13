---
mode: small
---

# Plan: The release runs a live check of the built binary

## Summary

The release script gains one step. After the test suite is green and
before the release commit, it builds the bee binary, installs it into a
throwaway project, runs a short list of first-contact commands and stops
the release when one fails. The same check can be run by hand as one
command of the verify driver.

Mode: `small` — 1 risk flag: public contracts (the release script gains a step).
Why this is the least workflow that protects the work: two scripts and their docs form one change, and the check proves itself by running.

## Requirements (from CONTEXT.md)

- D1: a scripted live check inside the release script; red stops the release.
- D2: one `smoke` subcommand on the verify driver, private run id, fixed step list.
- D3: placed after the test gate and before the release commit; the test skip skips it too; checklist text unchanged.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The release script's test gate ends in one fail line, the place the smoke follows | read | scripts/release.sh:293 | `the declared test suite went RED` |
| 2 | The release commit block starts after the test gate | read | scripts/release.sh:301 | `SUBJECT="${COMMIT_SUBJECT:-Release $ARG_VERSION}"` |
| 3 | The verify driver dispatches subcommands in one case block | read | .bee/verify/verify-app/control-bee:362 | `build)    shift; cmd_build "$@" ;;` |
| 4 | The driver keeps a run pointer the smoke must not move | read | .bee/verify/verify-app/control-bee:46 | `RUN_POINTER="$VERIFY_HOME/current-run"` |
| 5 | A run id can be set from the environment, so a private run is possible | read | .bee/verify/verify-app/control-bee:54 | `if [ -n "${VERIFY_RUN:-}" ]; then printf '%s' "$VERIFY_RUN"; return; fi` |
| 6 | Neither the release script nor CI calls the driver today | ran | rg -n "verify-app" scripts/release.sh .github/workflows | `(no output)` |

## Discovery

The leader read the head and the test-gate part of `scripts/release.sh`, the
usage header and run-identity functions of the verify driver, and the Launch,
Doctor and Drive sections of the verify skill. The path floor rows cover
`.github/workflows/release*` only, so this change is not an owner-first path.

## Approach

One cell. The smoke lives on the driver (D2) so the sandbox and its
isolation check have one home; the release script calls it by its literal
path (D3). Rejected: a new script under `scripts/` (a second sandbox), a
separate skip flag (a second switch), a rule for agents (prose).

Risk map:
- The smoke dirties the tree mid-release — LOW — the driver writes only under its state folder and the build target — Lands in: rsm-1 — proof: the worktree is clean after a smoke run.
- A red smoke does not stop the release — MEDIUM — the one fact: the call ends in the script's own fail on a non-zero exit — Lands in: rsm-1 — proof: the smoke exits non-zero with an unusable state folder, and the call site is read at the leader check.

## Shape

One slice, one cell. Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied); step 6 does not apply.

SMALLER PATH: calling the driver's existing launch and doctor from the release script is cheaper, but it would move the agent's run pointer and drive no command; D2 asks for both. PASS.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| rsm-1 | Run a live smoke of the built binary inside the release script | the verify driver, its skill page, scripts/release.sh, LEADER.md, verify-pipeline knowledge, rendered copies | — | one command prints an ok line per step and a green last line; the release script runs it after the tests | the smoke run green, a forced red run, syntax checks |

```json
[
  {
    "id": "rsm-1",
    "feature": "release-smoke",
    "title": "Run a live smoke of the built binary inside the release script",
    "lane": "small",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "d29bbedb-b1a6-48fb-a35e-d660ccaaafbf"
    ],
    "files": [
      ".bee/verify/verify-app/control-bee",
      ".bee/verify/verify-app/SKILL.md",
      "scripts/release.sh",
      "LEADER.md",
      "docs/knowledge/areas/verify-pipeline/*",
      ".claude/skills/*",
      ".agents/skills/*",
      ".opencode/skills/*",
      ".claude-plugin/skills/*",
      ".codex-plugin/skills/*",
      ".bee/onboarding.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/release-smoke/CONTEXT.md",
      ".bee/verify/verify-app/control-bee",
      ".bee/verify/verify-app/SKILL.md",
      "scripts/release.sh",
      "docs/knowledge/patterns/20261112-verify-harness-sandbox-leaks-into-the-real-environment.md"
    ],
    "affects_skills": [
      ".bee/verify/verify-app/SKILL.md"
    ],
    "affects_specs": [
      "docs/knowledge/areas/verify-pipeline/overview.md"
    ],
    "action": "Implement docs/history/release-smoke/CONTEXT.md D1, D2 and D3. (1) Driver. In .bee/verify/verify-app/control-bee add one subcommand named smoke, wired in the case block beside the existing ones (anchor: the line that dispatches build to cmd_build) and listed in the usage header. Reuse the driver's own functions for build, launch, doctor, cli, host and cleanup; add no second sandbox logic. The smoke uses a run id of its own, made for this call, and must leave the current-run pointer file exactly as it found it (absent stays absent), so a verify run an agent has in flight is not disturbed. Steps, in order, each printing one line that starts with ok or FAIL and names the step: build the release binary; launch an onboarded sandbox; the driver's doctor; the binary's version equals the version in .claude-plugin/plugin.json; status as JSON exits 0 and parses; orient exits 0; the JSON help surface exits 0 and parses; one decision written with the decisions log verb and then found by the decisions search verb; a second onboard of the sandbox, run the way the driver's host door runs it, reports up_to_date. Remember that the driver's cli door always returns 0 and records the real exit code in a file: assert on the recorded exit code and output, never on the shell status. On the first failing step: print the FAIL line with the step name and the evidence folder, keep the sandbox, exit non-zero. On a clean run: remove the sandbox, keep the evidence, print one final line that says the smoke is green and how many steps ran, exit 0. A step you find is not deterministic on this tree (needs network, a live agent, or fails for a reason unrelated to the binary) is replaced by the nearest deterministic one, and you record that as a departure. (2) Release script. In scripts/release.sh, right after the test gate block (anchor: the fail line that says the declared test suite went RED) and before the release commit block (anchor: the line that sets SUBJECT), call the smoke through bash with the literal driver path, output to stderr like the test gate. A non-zero exit calls fail with a message that says the live smoke went RED and nothing was tagged or pushed. When tests are skipped (RUN_TESTS is 0) the smoke is skipped too and one log line says so; do not change the confirm checklist text or its item count, which scripts/tests/release-confirm-gate.sh pins. The idempotent branch for an already committed version stays untouched. Update the step list in the header of scripts/release.sh by editing existing lines so the header names the smoke; add no code comment anywhere else and write no comment in the new driver code. (3) Docs. In .bee/verify/verify-app/SKILL.md document the smoke subcommand in the same form as the other commands, by its literal path with the bash prefix, and say that the release script runs it. In LEADER.md (the project file at the repo root), section Release, name the live smoke between the test suite and the tag in one sentence. In docs/knowledge/areas/verify-pipeline/overview.md add the smoke and its place in the release, citing release-smoke D1, D2 and D3 by those names, anchored to the two scripts. Then run .bee/bin/bee dev regen so the rendered skill copies and the release manifest match the sources; never hand-edit a rendered copy. Proof, all three recorded: the smoke run green on this worktree with its step lines; the smoke run with VERIFY_HOME set to a path that cannot be created (for example a path under /proc) exiting non-zero; and bash -n on both scripts. Also run bash scripts/tests/release-confirm-gate.sh and report its result. One commit.",
    "verify": "bash .bee/verify/verify-app/control-bee smoke && bash -n scripts/release.sh && bash -n .bee/verify/verify-app/control-bee && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "The smoke subcommand of the verify driver builds the binary, launches an onboarded sandbox, runs the listed steps, prints one ok line per step and exits 0 on a clean run",
        "A failing step makes the smoke exit non-zero, name the step and the evidence folder, and keep the sandbox",
        "The smoke leaves the current-run pointer file as it found it",
        "scripts/release.sh calls the smoke after the test gate and before the release commit, and a non-zero exit stops the release through fail",
        "With tests skipped the release script skips the smoke and logs one line that says so",
        "The confirm checklist of the release script is unchanged"
      ],
      "artifacts": [
        {
          "path": ".bee/verify/verify-app/control-bee",
          "substantive": "the smoke subcommand, built on the driver's existing functions"
        },
        {
          "path": "scripts/release.sh",
          "substantive": "the smoke call between the test gate and the release commit"
        }
      ],
      "key_links": [
        "scripts/release.sh invokes .bee/verify/verify-app/control-bee smoke"
      ],
      "prohibitions": [
        "No new skip flag on the release script",
        "No change to the confirm checklist text or to scripts/tests/release-confirm-gate.sh",
        "No write into the real checkout's .bee state from the smoke",
        "No hand edit of a rendered skill copy",
        "No code comments in new code"
      ],
      "open_choices": [
        "The exact bee calls behind each step of the fixed list"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  }
]
```

## Test matrix

| Case | Probe | Pass when |
|---|---|---|
| Happy path | the smoke on the worktree | every step prints ok, exit 0, sandbox removed |
| Error path | the smoke with a state folder that cannot be created | exit non-zero |
| Edge | the smoke while a run pointer exists | the pointer file is byte-identical after |
| Wiring | read the release script | the call sits between the test gate and the commit and ends in fail |

## Open Questions

(none)

## Out of scope

- A rule that tells agents to run verify before a release.
- A CI job that runs the smoke.
- Driving the feature maps of the verify skill inside the release.
