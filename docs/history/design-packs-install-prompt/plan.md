---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: design-packs-install-prompt

## Summary

A person who installs bee into a project with web pages or components gets no
design skills unless they already know the `--design-packs` flag. The
installer never asks. It prints a line written for an agent, and the help for
`bee onboard` does not name the flag at all.

After this work, the installer asks the person once which design packs they
want, when the project has interface files and no pick yet. With `--yes`, with
no terminal, or on a dry run, it prints one plain line that names the flag
instead. `bee onboard --help` lists `--design-packs`.

Mode: `standard` — 2 risk flags: public-contracts (onboard JSON field, help
surface), cross-platform (install.sh and install.ps1).
Why this is the least workflow that protects the work: two cells in a fixed
order, each with tests, because the installer change reads a field the first
cell adds.

## Requirements

No `CONTEXT.md` — the scope came from a reproduced bug and the user's pick of
option 1. The locked decision is `7cbfad00-3f49-41e5-a0db-2d203e595774`:

- D1: when onboarding reports a design offer and no `--design-packs` was given,
  an interactive install without `--yes` asks one question (a comma list of
  web, charts, motion, apple, figma, or none; Enter skips).
- D2: `--yes`, no terminal, or a dry run prints one plain line naming the
  `--design-packs` flag, and asks nothing.
- D3: onboarding exposes the offer as a boolean `design_offer` in its JSON
  payload, computed by the same predicate that emits the agent offer notice.
- D4: the onboard registry entry declares `design-packs`, so
  `bee onboard --help` lists it.
- D5 (carried, design-os-in-bee): bee never picks packs for the user.

## Load-bearing claims

Labels: `read` = the file was opened at that line and the bytes copied; `ran` =
the command was run in this session and its output copied; `guessed` = neither.
No `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | An install without the flag on a repo with UI files writes no design skill but does write the design agent | ran | `control-bee sh -- bash scripts/install.sh -d <target3> -y --runtime claude` then `ls .claude/skills \| rg design; ls .claude/agents` | no design skill listed; agents: `bee-build.md bee-design.md bee-extract.md bee-gather.md bee-review.md` |
| 2 | The installer prints the agent-facing offer to the human terminal | ran | same run as claim 1 | `notice: This project has files that draw a user interface. ... Offer it - say what each pack is for, then ask which ones fit` |
| 3 | The offer predicate is "no recorded pick and a UI file" | read | `packages/bee-rs/crates/bee/src/onboard/notices.rs:250-253` | `let picked = obj.and_then(\|o\| o.get("design")).and_then(\|d\| d.get("packs")).is_some();` / `if !picked && has_ui_file(repo_root) {` |
| 4 | The plan-mode JSON payload is built in one place and already carries notices | read | `packages/bee-rs/crates/bee/src/onboard/mod.rs:482-496` | `payload.insert("status".into(), json!(status));` … `payload.insert("notices".into(), build_notices(&computed.plan));` |
| 5 | The installer already reads one field out of the preview JSON through a helper | read | `scripts/install.sh:656-658, 667-668` | `plan_field() {` / `printf '%s' "$1" \| "$BEE_BIN" dev install-support field --key "$2"` / `PREVIEW_STATUS="$(plan_field "$PREVIEW_JSON" status)"` |
| 6 | `field` prints a boolean as `true` / `false` | read | `packages/bee-rs/crates/bee/src/devtools/install_support.rs:107-110` | `Value::String(s) => s,` / `other => other.to_string(),` |
| 7 | The installer has a TTY check and a `--yes` switch to reuse | read | `scripts/install.sh:114-121` | `can_prompt() { [ -r /dev/tty ] && [ -w /dev/tty ] && ( : < /dev/tty ) 2>/dev/null; }` / `if [ "$ASSUME_YES" -eq 1 ]; then return 0; fi` |
| 8 | The pick is passed to onboarding through `ONBOARD_FLAGS` | read | `scripts/install.sh:483-485` | `if [ -n "$DESIGN_PACKS" ]; then` / `ONBOARD_FLAGS+=("--design-packs" "$DESIGN_PACKS")` |
| 9 | The PowerShell installer builds the same flags and runs the preview without JSON | read | `scripts/install.ps1:547, 567` | `if ($DesignPacks) { $onboardFlags += @('--design-packs', $DesignPacks) }` / `try { & $beeBin onboard --repo-root $Directory @onboardFlags } finally { Pop-Location }` |
| 10 | The registry entry for onboard has no design-packs parameter | ran | `jq -r '.commands[] \| select(.name=="onboard") \| .parameters.properties \| keys[]' packages/bee-rs/crates/bee/src/generated/registry_payload.json` | `apply claude-md force-downgrade global-skills json no-claude-md no-statusline plugin-source repo-hooks repo-root runtime` |
| 11 | The binary already parses the flag | read | `packages/bee-rs/crates/bee/src/onboard/mod.rs:226` | `} else if arg == "--design-packs" \|\| arg.starts_with("--design-packs=") {` |
| 12 | Installer tests can run install.sh end to end against a sandbox | read | `packages/bee-rs/crates/bee/tests/installer_contracts.rs:628-633` | `fn install_sh_forwards_the_design_pick_to_onboarding()` … `run_install_sh(sandbox.path(), &["--design-packs", "web"]);` |
| 13 | With the flag, install works today — the bug is only the missing ask | ran | `control-bee sh -- bash scripts/install.sh -d <target2> -y --runtime claude --design-packs web` | `bee-design` / `bee-design-web` under `.claude/skills`, config `"packs": ["web"]` |

## Discovery

The repro ran in a verify sandbox: with the flag, both `install.sh` and
`bee onboard --apply` copy `bee-design` and the picked pack (claim 13).
Without it, nothing is copied and the human sees an agent instruction
(claims 1, 2). The offer predicate lives in one function (claim 3), so the
installer can read it from onboarding instead of copying the UI scan into two
shell languages.

## Approach

**Onboarding owns the question "should the user be offered design packs".**
dpi-1 lifts the predicate at `notices.rs:250-253` into one public function,
uses it for the notice, and writes its result as `design_offer` into the plan
payload (D3). One home for the fact; the installers only read it.

dpi-2 makes both installers act on it after the preview (D1, D2). When the
person answers with names, the installer adds `--design-packs <answer>` to the
onboarding flags and runs the preview again, so an unknown name is refused by
onboarding before anything is written (the existing refusal path). An empty
answer skips and changes nothing.

Rejected alternatives:

- *Port the UI-file scan into install.sh and install.ps1.* Three copies of one
  rule (claim 3), and they would drift.
- *Parse the notice text in the installers.* The text is prose for agents; a
  wording change would silently break the ask.
- *Only rewrite the notice for humans (option 2).* The user picked option 1.

Smaller path check: is there a cheaper shape that still honours D1–D4? One cell
over all six files would work, but the shell test depends on the Rust field
existing, and splitting keeps each proof narrow. Two cells is the floor.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| `design_offer` field | LOW — additive key in a JSON payload | dpi-1 | unit test: true for a repo with `index.html` and no pick, false after a pick or with no UI files |
| Registry param | LOW — additive optional param | dpi-1 | test that `bee onboard --help` output contains `--design-packs` |
| The prompt in install.sh | MEDIUM — a new read from the terminal could hang a non-interactive install | dpi-2 | tests: `--yes` and no-TTY runs print the line and never block; a pty run (`script`) answering `web` installs the pack |
| install.ps1 mirror | MEDIUM — no pwsh on this machine to run it | dpi-2 | static contract test that install.ps1 reads `design_offer`, gates the ask on `-Yes`/interactive, and stays pure ASCII; Windows CI runs the rest |

Waves: dpi-1, then dpi-2 — dpi-2's tests read the field dpi-1 adds.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "Rust and installer script changes with their tests"},
    {"stage": "tests", "classification": "not-applicable", "role": "test", "reason": "each code cell writes its own tests red-first"},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "the reads were done at plan time; evidence is claims 1-13"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no lookup remains"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "code covers it"},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "this plan"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "two small cells, thirteen recorded claims"},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "the onboarding design-packs concept needs the installer ask recorded", "reason": "capture runs at close"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "two serial cells"},
    {"stage": "design", "classification": "not-applicable", "role": "design", "reason": "no rendered UI changes"},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "no hat wave: a reproduced bug with a user-picked fix and recorded claims"},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "as above"},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "as above"},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "the smaller-path check is inline in Approach"},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "as above"},
    {"stage": "lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes"},
    {"stage": "lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "as above"},
    {"stage": "lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "as above"}
  ]
}
```

## Shape

One slice, two cells in order.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| dpi-1 | Report the design offer in onboarding JSON and list --design-packs in onboard help | `onboard/notices.rs`, `onboard/mod.rs`, `generated/registry_payload.json`, `onboard/tests.rs` | — | `bee onboard --help` names `--design-packs`; `bee onboard --json` says `"design_offer": true` on a UI repo with no pick | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml design_offer` — green:unit |
| dpi-2 | Ask for design packs in both installers when onboarding offers them | `scripts/install.sh`, `scripts/install.ps1`, `tests/installer_contracts.rs`, release manifest | dpi-1 | an interactive install asks which design packs to add; `-y` prints one plain line naming the flag | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test installer_contracts design` — green:integration |

```json
[
  {
    "id": "dpi-1",
    "feature": "design-packs-install-prompt",
    "title": "Report the design offer in onboarding JSON and list --design-packs in onboard help",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["7cbfad00-3f49-41e5-a0db-2d203e595774"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/mod.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/mod.rs",
      "docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Red first, and name EVERY new test with `design_offer` in it so one filter covers the cell. (1) In onboard/notices.rs, lift the predicate at lines 250-253 (`!picked && has_ui_file(repo_root)`, where picked means .bee/config.json has design.packs) into one `pub fn design_offer_due(repo_root: &Path) -> bool`, and make the existing notice code call it, so the DESIGN_PACKS_OFFER notice and the new field can never disagree. (2) In onboard/mod.rs, insert `design_offer` (a JSON boolean from design_offer_due) into the plan-mode payload beside `notices` (around line 496) and into the apply payload where it builds notices (around line 559). (3) In generated/registry_payload.json, add a `design-packs` string parameter to the `onboard` command's parameters.properties, described as: Design skill packs to install: a comma list of web, charts, motion, apple, figma, or none. Installs bee-design with the picked packs and records the pick; none removes them and stops the offer. Keep the file's existing formatting (one-line JSON) and check whether any test or `bee dev regen` derives or checks this file; if regen owns it, change the owner instead. Tests: design_offer is true for a repo with an index.html and no design pick; false for the same repo once .bee/config.json records design.packs (including an empty list or none); false for a repo with no UI file; and `bee onboard --help` output lists `--design-packs`. Do not change the DESIGN_PACKS_OFFER wording, the UI scan, or any existing assertion. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml design_offer",
    "must_haves": {
      "truths": [
        "bee onboard --json reports design_offer true on a repo with a UI file and no recorded pick",
        "design_offer is false once a pick is recorded, and false with no UI file",
        "bee onboard --help lists --design-packs"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/notices.rs", "substantive": "pub fn design_offer_due used by the notice; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "onboard parameters include design-packs"}
      ],
      "key_links": [
        "the notice and the payload field both call design_offer_due"
      ],
      "prohibitions": [
        "No change to DESIGN_PACKS_OFFER wording or the UI scan",
        "No change to scripts/install.sh or scripts/install.ps1",
        "No edit to any existing assertion",
        "No code comments"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "dpi-2",
    "feature": "design-packs-install-prompt",
    "title": "Ask for design packs in both installers when onboarding offers them",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["dpi-1"],
    "decisions": ["7cbfad00-3f49-41e5-a0db-2d203e595774"],
    "files": [
      "scripts/install.sh",
      "scripts/install.ps1",
      "packages/bee-rs/crates/bee/tests/installer_contracts.rs",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "scripts/install.sh",
      "scripts/install.ps1",
      "packages/bee-rs/crates/bee/tests/installer_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Red first, and name EVERY new test with `design` in it (the cell filter is `--test installer_contracts design`). dpi-1 adds a boolean `design_offer` to the `bee onboard --json` preview payload. In scripts/install.sh, right after the preview status check (around line 667-672) and BEFORE the dry-run exit: when DESIGN_PACKS is empty and `plan_field \"$PREVIEW_JSON\" design_offer` prints `true`: if ASSUME_YES is 0, DRY_RUN is 0 and can_prompt succeeds, print to /dev/tty one short plain question that names the five packs (web, charts, motion, apple, figma), says `none` stops the question, and that Enter skips; read the answer from /dev/tty. A non-empty answer sets DESIGN_PACKS, appends `--design-packs <answer>` to ONBOARD_FLAGS, and re-runs the preview with the same status check (onboarding refuses an unknown name before anything is written). Otherwise (--yes, no TTY, dry run, or Enter) log one plain line in the installer's `log` style, for a person, e.g. `design   this project has interface files; re-run with --design-packs web (or charts,motion,apple,figma, or none) to add design skills`. Never pick a pack for the user. Mirror the same behaviour in scripts/install.ps1: run the preview once with --json to read design_offer (keep the existing human preview output), ask with Read-Host only when not -Yes, not -DryRun and the console is interactive (reuse the test Confirm-Step already uses), otherwise Write-Host the plain line; keep install.ps1 pure ASCII. Tests in tests/installer_contracts.rs using run_install_sh: a sandbox host with an index.html and `-y` prints the plain line and installs no bee-design skill; with `--design-packs web` no line is printed; an interactive run under `script -qec` (skip the test when `script` is absent) answering `web` installs bee-design and bee-design-web and records the pick; a static test that install.ps1 reads design_offer and gates Read-Host on -Yes. Do not change other installer behaviour or any existing assertion. No code comments in the Rust test; shell and ps1 files keep their existing comment style. After the script edits, run the regen chain inside this cell (.bee/bin/bee dev regen) so the release manifest hashes the new install.sh and install.ps1.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test installer_contracts design && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "An interactive install on a UI repo with no pick asks once and installs the answered packs",
        "--yes, no TTY, or a dry run prints one plain line naming --design-packs and never blocks",
        "A given --design-packs flag means no question and no line"
      ],
      "artifacts": [
        {"path": "scripts/install.sh", "substantive": "reads design_offer from the preview, asks or prints the line"},
        {"path": "scripts/install.ps1", "substantive": "same behaviour, pure ASCII"}
      ],
      "key_links": [
        "both installers read design_offer from onboarding rather than scanning for UI files"
      ],
      "prohibitions": [
        "No pack chosen without the user's answer",
        "No read from the terminal under --yes, without a TTY, or on a dry run",
        "No change to onboarding Rust code",
        "No edit to any existing assertion"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  }
]
```

## Test matrix

| Case | Scenario | Pass when |
|---|---|---|
| Happy path (dpi-1) | UI repo, no pick, `bee onboard --json` | `"design_offer": true` |
| Edge (dpi-1) | pick recorded (`web`, or `none`) | `"design_offer": false` |
| Edge (dpi-1) | no UI file | `"design_offer": false` |
| Help (dpi-1) | `bee onboard --help` | output contains `--design-packs` |
| Happy path (dpi-2) | interactive install, answer `web` | `.claude/skills/bee-design-web` exists, config `design.packs == ["web"]` |
| Edge (dpi-2) | `-y` on UI repo | output has the plain `design` line, no `bee-design` skill, exit 0 |
| Edge (dpi-2) | `--design-packs web` given | no `design` line printed |
| Behaviour change | `-y` install on a UI repo, main vs this branch | main prints only the agent notice; this branch also prints the plain line |

## Open Questions

(none)

## Out of scope

- The agent-facing `notice:` text stays as is; agents still read it.
- `bee config get` missing from the binary, and the `team.opencode` design
  warning — separate issues.
