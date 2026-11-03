---
artifact_contract: bee-plan/v2
mode: standard
---

# live-proof-evidence — plan

Route: class `feature` · lane `standard` · flags `public-contracts`,
`covered-contract-change` · product files 5.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

## Summary

A `green:live` cap must say where its evidence lives. `bee cells finish`
refuses a `green:live` proof line whose scope reason names no evidence
locator (a run URL, an absolute path, or a path containing `evidence/`),
and the refusal teaches the three shapes. `control-bee` stops writing
evidence under `/tmp` so the locator still resolves after a reboot. The
feature map, the doctrine rule, and the worker step name the rule where
they already describe the proof line. Locked decisions:
`docs/history/live-proof-evidence/CONTEXT.md` D1–D3.

## Discovery

One reality touch per surface, all taken before this plan:

- `parse_report_flag`'s `tests` arms read at `finish_support.rs:237-285`
  — the vocabulary arm is where the new arm slots in.
- `parse_tests_proof` read at `finish_support.rs:114-129` — returns the
  reason segment untouched, so the new check reads the tuple, not the
  raw string.
- The three fixtures that cap `green:live` with a bare reason read at
  `cells/tests.rs:6063`, `cells/tests.rs:10080`,
  `pi_plugin_contracts.rs:7642` and `:7664`.
- `control-bee`'s default read at `control-bee:43`; `evidence_dir()` at
  `:64` derives from it, so one line moves every path.
- Counted with `rg -l green:live .bee/cells/`: 39 caps; `rg -c
  bee-verify/evidence .bee/cells/`: 3 caps name the evidence dir.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The result vocabulary is closed on the write path and the reason segment is free text | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:87` | `pub(crate) const PROOF_RESULT_VALUES: [&str; 3] = [` … `"green:live",` … and the accepting arm `Some(_) => {}` at `:279` reads nothing from the reason |
| 2 | The vocabulary arm receives the parsed tuple, reason included | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:273` | `Some((_, result, _)) if !PROOF_RESULT_VALUES.contains(&result.as_str()) => {` |
| 3 | The read path must stay tolerant | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:104-112` | `the READ path — feature_proof_check (proof.rs) — calls this same function over already-capped cells, so a vocabulary check HERE would retroactively refuse` |
| 4 | Three existing fixtures cap `green:live` with a bare reason and will go red | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:7642` | `"tests": "manual inspection — green:live — checked",` (also `:7664` `parity file verified on disk`, `cells/tests.rs:10080` `live verification in sandbox`, `cells/tests.rs:6063` loop `touched close.rs`) |
| 5 | `VERIFY_HOME` has one default line and every path derives from it | read | `.bee/verify/verify-app/control-bee:43` | `VERIFY_HOME="${VERIFY_HOME:-${TMPDIR:-/tmp}/bee-verify}"`; `evidence_dir()` at `:64` is `printf '%s/evidence/%s' "$VERIFY_HOME" "$(run_id)"` |
| 6 | The SKILL.md paths table restates the `/tmp` default | read | `.bee/verify/verify-app/SKILL.md:66-68` | `\| Evidence \| \`${TMPDIR:-/tmp}/bee-verify/evidence/<run-id>\` \|` |
| 7 | The rendered runtime copies come from the regen chain, not by hand | read | `.bee/verify/verify-app/SKILL.md:26-29` | `bee renders 0644 copies of it into .claude/skills/verify-app/, .agents/skills/verify-app/ and .opencode/skills/verify-app/` … `after any edit here, re-run bee onboard --apply` |
| 8 | The doctrine home of the proof vocabulary is R16c-a | read | `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md:216` | `- \`green:live\` — the real product or command was driven and its observable` |
| 9 | The worker step says "evidence attached" and names no locator | read | `skills/bee-swarming/SKILL.md:197` | `drive its mapped feature and inspect the result, evidence attached,` |
| 10 | The AGENTS-block parity test pins only two tokens, so no AGENTS.md edit is owed | read | `packages/bee-rs/crates/bee/tests/verification_contract_parity.rs:307` | `for token in ["user-facing", "green:live"] {` |
| 11 | The feature map's cell file lists `cell-cap-green` as the sub-feature the rule extends | read | `.bee/verify/verify-app/features/cells-and-proof.md:15` | `- \`cell-cap-green\` caps the cell and stores the proof line on its trace.` |
| 12 | `control-bee` already carries `#` comment lines, so the comment ratchet allows edits but no new comment lines | read | `.bee/verify/verify-app/control-bee:33` | `#   VERIFY_HOME   root for runs and evidence (default ${TMPDIR:-/tmp}/bee-verify)` |

## Smaller path check

*Is there a cheaper shape that still honours every locked decision?*

- **Doctrine only, no check.** FAIL — that is today's state: the rule
  is prose in three places and 36 of 39 live caps ignore it.
- **Require the `control-bee` evidence dir only.** FAIL — D1 rejects
  it: release caps prove live through a CI run URL and must stay legal.
- **A new `evidence` key on `--report`.** FAIL — proof-strength D6 keeps
  the proof line at three segments, and the reason segment already is
  the place a worker writes where they looked. One token scan on it is
  the smallest honest shape.
- **Skip D2.** FAIL — a locator into `/tmp` is a locator into nothing
  after a reboot.

## Hat wave

Not run. Decision `d33c2f89`: clear ask, user-picked shape, two
disjoint cells (gates-and-delegation "Hat wave" threshold).

## Approach

Recommended path (D1, D2, D3): one guard arm in `parse_report_flag`,
one script default, five doc sites, one regen.

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| the new refusal arm | LOW | lpe-1 | red-first test: bare reason refused, each locator shape caps, `green:unit` bare reason still caps |
| existing fixtures going red | LOW | lpe-1 | the cells and pi-plugin suites green |
| script default | LOW | lpe-2 | `control-bee paths` prints the new root |
| rendered copies drifting | LOW | lpe-2 | `bee dev release-manifest --check` green after `bee dev regen` |

Waves: lpe-1 and lpe-2 touch disjoint files and run at the same time.
No serial edge.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"lpe-1 writes the refusal arm and its tests in one cell."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"lpe-2 edits the script default, the SKILL paths table, the feature map and two doctrine sentences, then runs the regen chain."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"The code cell owns its tests red-first; no separate test cell."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"This page is the plan; the leader wrote it."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery was five targeted reads by the leader."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left open."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked and was not requested."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Standard lane, no high-risk consult owed."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"No hat wave (decision d33c2f89)."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"No hat wave (decision d33c2f89)."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"No hat wave (decision d33c2f89)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"No hat wave (decision d33c2f89)."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"No hat wave (decision d33c2f89)."}
  ]
}
```

## Shape

One slice, two cells, one wave.

### Slice 1 — the check, the root, the words

- **lpe-1** — `verbs/cells/finish_support.rs`: a
  `proof_reason_names_evidence(reason: &str) -> bool` token scan beside
  `PROOF_RESULT_VALUES`, and one new arm in `parse_report_flag` after
  the vocabulary arm: `green:live` with no locator refuses, naming the
  three shapes and the `control-bee` evidence dir. Tests in
  `cells/tests.rs`, red-first: bare reason refused; a `https://` run
  URL caps; an absolute path caps; a relative `evidence/<run>` token
  caps; `green:unit` with a bare reason still caps. The four fixtures
  of claim 4 gain a locator.
- **lpe-2** — `control-bee:43` and its header line `:33` take the
  `${XDG_STATE_HOME:-$HOME/.local/state}/bee-verify` default (edit the
  existing comment line; add no comment line — claim 12). SKILL.md
  paths table `:66-68` follows. `features/cells-and-proof.md` gains
  sub-feature `cell-cap-live-evidence` and one drive bullet.
  R16c-a's `green:live` bullet gains the locator sentence citing
  decision `d56826ef`. `bee-swarming/SKILL.md:197` says "the evidence
  locator named in the scope reason". Then `bee dev regen`; commit the
  refreshed rendered copies and `release-manifest.json` with the rest.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| lpe-1 | Refuse a green:live cap whose reason names no evidence | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` | — | `bee cells finish` with `… — green:live — checked` stops and names the three locator shapes; the same cap with the evidence dir in the reason lands | cells and pi-plugin suites green, red-first on the new arm |
| lpe-2 | Keep verify-app evidence out of /tmp and name the rule in the map and doctrine | `.bee/verify/verify-app/control-bee`, `.bee/verify/verify-app/SKILL.md`, `.bee/verify/verify-app/features/cells-and-proof.md`, `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md`, `skills/bee-swarming/SKILL.md`, rendered copies under `.claude/`, `.agents/`, `.opencode/`, `docs/history/codex-harness-hardening/release-manifest.json` | — | `control-bee paths` prints a root under `~/.local/state`; the feature map tells a driver how to prove the refusal | `bee dev release-manifest --check` green, `control-bee paths` output |

```json
[
  {
    "id": "lpe-1",
    "feature": "live-proof-evidence",
    "title": "Refuse a green:live cap whose reason names no evidence",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["d56826ef-77c2-40bf-aa54-e03c0f21aca1"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/live-proof-evidence/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs add `pub(crate) fn proof_reason_names_evidence(reason: &str) -> bool` beside PROOF_RESULT_VALUES: true when any whitespace-delimited token of `reason` starts with `http://` or `https://`, or starts with `/`, or contains `evidence/`. In parse_report_flag, directly after the arm that refuses a result outside PROOF_RESULT_VALUES (finish_support.rs:273), add one arm: `Some((_, result, reason)) if result == \"green:live\" && !proof_reason_names_evidence(&reason)` returns Err(Fail::Thrown(...)) whose text starts `cells finish: --report key \"tests\" result \"green:live\" names no evidence locator in its scope reason — a live proof must say where it can be reopened: a run URL (http:// or https://), an absolute path, or a path containing evidence/ (control-bee writes to $VERIFY_HOME/evidence/<run-id>).` The read path (parse_tests_proof, proof.rs feature_proof_check) is NOT touched (CONTEXT.md D1, proof-strength D2). RED-FIRST in cells/tests.rs, beside the existing proof-strength tests near :6040: (a) `report_tests_green_live_without_evidence_locator_refuses` — reason `checked` refuses, error text contains `names no evidence locator`; run it and watch it fail before writing the arm; (b) one test that loops three reasons — `https://github.com/o/r/actions/runs/1`, `/home/u/.local/state/bee-verify/evidence/r1 snapshot ok`, `see evidence/20260922-1 out files` — and asserts each caps with the proof stored on trace.report.tests; (c) `green:unit — touched close.rs` still caps (regression pin). Then update the fixtures that now go red so they name a locator: cells/tests.rs:6063 loop (use a reason with an absolute path such as `touched close.rs, evidence /tmp/x/evidence/r1`), cells/tests.rs:10080, tests/pi_plugin_contracts.rs:7642 and :7664 — change the reason segment only; every assertion on verify_output stays. NO COMMENTS of any form in the code you add (no_code_comments is on; the ratchet reds the suite if the count rises). One commit, subject in imperative mood, trailer `cell: lpe-1`.",
    "must_haves": {
      "truths": [
        "a green:live proof line whose reason has no locator token is refused by bee cells finish, and the refusal names http(s)://, an absolute path, and evidence/",
        "a green:live proof line with a URL, an absolute path, or an evidence/ token caps and stores the line on trace.report.tests",
        "green:unit and green:static caps with a bare reason are unchanged",
        "parse_tests_proof and proof.rs are byte-identical to main",
        "the cells test module and tests/pi_plugin_contracts.rs are green under cargo test --release"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs", "substantive": "proof_reason_names_evidence and the new refusal arm in parse_report_flag"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "substantive": "the red-first refusal test, the three-locator cap test, the green:unit regression pin"}
      ],
      "key_links": [
        "docs/history/live-proof-evidence/CONTEXT.md D1 is the locked rule",
        "docs/history/proof-strength-and-expiry/CONTEXT.md D2 forbids a read-path check"
      ],
      "prohibitions": [
        "No change to parse_tests_proof, proof.rs, or any read path",
        "No new --report key and no fourth proof value",
        "No code comment of any form",
        "No edit outside the three listed files"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells::tests && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts"
  },
  {
    "id": "lpe-2",
    "feature": "live-proof-evidence",
    "title": "Keep verify-app evidence out of /tmp and name the rule in the map and doctrine",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["bb51c581-6ed5-45a5-ab7c-88541c0089ee", "d56826ef-77c2-40bf-aa54-e03c0f21aca1"],
    "files": [
      ".bee/verify/verify-app/control-bee",
      ".bee/verify/verify-app/SKILL.md",
      ".bee/verify/verify-app/features/cells-and-proof.md",
      "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md",
      "skills/bee-swarming/SKILL.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/live-proof-evidence/CONTEXT.md",
      ".bee/verify/verify-app/control-bee",
      ".bee/verify/verify-app/features/cells-and-proof.md",
      "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md"
    ],
    "affects_skills": [
      "skills/bee-swarming/SKILL.md"
    ],
    "affects_specs": [
      "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md"
    ],
    "action": "Five edits, then the regen chain. (1) .bee/verify/verify-app/control-bee:43 — change the default to `VERIFY_HOME=\"${VERIFY_HOME:-${XDG_STATE_HOME:-$HOME/.local/state}/bee-verify}\"`; edit the existing header line :33 to read `(default ${XDG_STATE_HOME:-$HOME/.local/state}/bee-verify)`. Edit existing comment lines only; add NO new comment line anywhere (the comment ratchet reds the suite if the count rises). (2) .bee/verify/verify-app/SKILL.md:66-68 — the three paths-table rows use `${XDG_STATE_HOME:-$HOME/.local/state}/bee-verify/...`; keep every other sentence. (3) .bee/verify/verify-app/features/cells-and-proof.md — under `## Sub-features` add `- \\`cell-cap-live-evidence\\` refuses a \\`green:live\\` cap whose scope reason names no evidence locator (a run URL, an absolute path, or a path containing \\`evidence/\\`); the refusal names the three shapes.` and under `## Driving it with control-bee` add two labeled bullets after the green cap bullet: one that caps demo-note-1 with `tests` = `test -f NOTE.md — green:live — checked` and expects a refusal whose error contains `names no evidence locator` with a non-zero .exit, and one that caps with the reason naming the run's evidence dir (`$VERIFY_HOME/evidence/<run-id>`) and expects status capped. Keep the file's four-section contract. (4) docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md — inside R16c-a, extend the `green:live` bullet (:216) with one sentence: the scope reason must name an evidence locator — a run URL, an absolute path, or a path containing `evidence/` — checked on the write path only (live-proof-evidence D1, decision `d56826ef`, 2026-09-22). One home, no second copy. (5) skills/bee-swarming/SKILL.md:197 — replace `evidence attached,` with `the evidence locator named in the scope reason,`. Do NOT edit AGENTS.md (claim 10: the parity test pins only `user-facing` and `green:live`, and R16c-a is the rule's home). Write every edited sentence through bee-technical-writing. Then run `.bee/bin/bee dev regen` (render-skill-trees, onboard --repo-root . --apply, release-manifest --write) so the .claude/.agents/.opencode copies of verify-app and bee-swarming match their sources, and commit the refreshed copies plus docs/history/codex-harness-hardening/release-manifest.json with the rest. Prove: `.bee/bin/bee dev release-manifest --check` green and `bash .bee/verify/verify-app/control-bee paths` printing a root under $HOME/.local/state. One commit, subject in imperative mood, trailer `cell: lpe-2`.",
    "must_haves": {
      "truths": [
        "control-bee's VERIFY_HOME default is ${XDG_STATE_HOME:-$HOME/.local/state}/bee-verify and TMPDIR appears nowhere in the script",
        "the SKILL.md paths table shows the same root",
        "features/cells-and-proof.md lists cell-cap-live-evidence and drives both the refusal and the accepted cap",
        "R16c-a's green:live bullet names the locator rule and cites decision d56826ef",
        "bee-swarming/SKILL.md step 5 names the evidence locator in the scope reason",
        "AGENTS.md is unchanged",
        "the rendered copies match their sources and bee dev release-manifest --check is green"
      ],
      "artifacts": [
        {"path": ".bee/verify/verify-app/control-bee", "substantive": "the VERIFY_HOME default line"},
        {"path": ".bee/verify/verify-app/features/cells-and-proof.md", "substantive": "the cell-cap-live-evidence sub-feature and its two drive bullets"},
        {"path": "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md", "substantive": "the locator sentence in R16c-a"}
      ],
      "key_links": [
        "docs/history/live-proof-evidence/CONTEXT.md D2 and D3 are the locked decisions",
        "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs (lpe-1) is the behaviour the words must match"
      ],
      "prohibitions": [
        "No edit to AGENTS.md",
        "No new comment line in control-bee",
        "No Rust source change",
        "No hand edit under .claude/, .agents/ or .opencode/ — those come from bee dev regen"
      ]
    },
    "verify": ".bee/bin/bee dev release-manifest --check && bash .bee/verify/verify-app/control-bee paths"
  }
]
```

## Test matrix

| Dimension | Case | Pass when |
|---|---|---|
| happy | `green:live` reason carries `https://…/runs/1` | cap lands, `trace.report.tests` equals the proof line |
| happy | `green:live` reason carries `/home/u/.local/state/bee-verify/evidence/r1` | cap lands |
| edge | `green:live` reason carries relative `evidence/20260922-1` | cap lands |
| edge | `green:unit — touched close.rs` | cap lands, unchanged |
| error | `green:live — checked` | refused, exit non-zero, text contains `names no evidence locator` |
| behavior-change | same `green:live — checked` cap on main vs head | main: caps. head: refused |
| regression | `control-bee paths` | prints a root under `$HOME/.local/state` |

## Test scoping

`commands.test` is the declared suite and CI runs it on every push. lpe-1
records the cells module plus the pi-plugin integration test; lpe-2
records the manifest check plus the script's own `paths` output.

## Open Questions

(none)
