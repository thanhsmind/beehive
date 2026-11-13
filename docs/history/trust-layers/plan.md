---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Catch it higher — static checks, rule landing, counted corrections, prose load

## Summary

When bee is set up in a project, it now looks for the project's own lint
and type-check commands and offers to run them before the tests. A new
craft page tells an agent how to turn a repeated mistake into a lint rule
in the project's own tool. A correction from the owner, or a finding from
a review, can be recorded with its source, so the "seen twice" count
includes them. The feedback digest prints how large the always-read
instruction text is, so the owner can see it shrink as rules move into
checks. Nothing new refuses work, and bee writes no command on its own.

Mode: `standard` — 2 risk flags: public contracts (a new flag, new onboarding notices, a new digest field), multi-domain (onboarding, mailbox, feedback, doctrine).
Why this is the least workflow that protects the work: each part is small and additive, but the parts reach four areas and one always-loaded doctrine file, so each needs its own proof and one plan check.

## Requirements (from CONTEXT.md)

- D1: onboarding detects static checks and proposes them first in the `commands.test` list; propose-only; cases (a), (b), (c).
- D2: `expertise/static-rules.md`, routed from `expertise/INDEX.md` and from step 2 of the promotion tree; `FIX_AT_VALUES` unchanged.
- D3: `bee mailbox reflect --source self|user|review`; doctrine line for user corrections and accepted review findings.
- D4: `bee feedback digest` reports the prose load as a witness.
- C1, C2, C3: the three interface contracts and their test-name markers.
- D5, D6, D7: the plan-check rulings that refine D1, D3 and D4 (recorded rule for a list, marker-word match, run-before-offer; source on both structs and one doctrine home; top-level prose_load with null for absent and bee lines apart from project lines).

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Onboarding already detects commands from the project's own files in one function | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:88 | `pub fn detect_commands(root: &Path) -> Vec<Candidate> {` |
| 2 | bee knows three command keys and none is a static check | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:332 | `pub const COMMAND_KEYS: &[&str] = &["setup", "start", "test"];` |
| 3 | `commands.test` already takes a list with a static check first | read | packages/bee-rs/crates/bee/src/verbs/protocol.rs:963 | `{"commands":{"test":["cargo fmt --check","cargo test"]}}` |
| 4 | The command notice is propose-only | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:206 | `propose-only (decision D3)` |
| 5 | An existing offer goes quiet on a recorded refusal | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:367 | `first look through the project's recorded decisions for an earlier refusal` |
| 6 | The fix-at layer is a closed set of four words | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:628 | `pub(crate) const FIX_AT_VALUES: [&str; 4] = ["architecture", "check", "doctrine", "none"];` |
| 7 | The reflect verb checks its required parts in one place | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:864 | `[("--wrong", wrong), ("--better", better), ("--fix-at", fix_at)]` |
| 8 | The lesson key is built from the layer and the what only | read | packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:755 | `fn reflection_key(fix_at: Option<&str>, what: &str) -> Option<String> {` |
| 9 | Close files a backlog row whose layer starts with fix-at | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:418 | `let layer = format!("fix-at:{}", m.fix_at);` |
| 10 | The digest already carries one witness about reflections | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:905 | `how the caps ANSWERED the mistakes question` |
| 11 | The promotion tree has a step for mechanizable findings | read | skills/bee-capturing/references/promotion.md:123 | `2. Mechanizable? A grep/lint line in a verify command, a guard, a hook` |
| 12 | The leader doctrine names the reflect verb in its source file | read | packages/bee/LEADER.md:365 | `bee mailbox reflect --wrong "<what went wrong>"` |
| 13 | The knowledge index has a critical patterns section to count | read | docs/knowledge/index.md:19 | `## Critical patterns` |
| 14 | The craft pages ship to hosts through the release manifest | read | docs/history/codex-harness-hardening/release-manifest.json:16 | `"path": ".bee/expertise/apis.md",` |
| 15 | The notice treats only a string as a recorded command today, so a list reads as not recorded | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:214 | `.and_then(Value::as_str)` |
| 16 | A sibling rule in the same file already accepts a list | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:348 | `v.is_array()` |
| 17 | The digest's printed sentence has its own function, apart from the count verb's | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:1676 | `fn digest_summary_line` |
| 18 | The stored entry is a struct built by literals outside mailbox.rs | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:3531 | `let entry = mailbox::Entry {` |

## Discovery

The leader read `detect_commands` and `commands_notices` in full, the
reflection rules and "A mistake seen twice" in
`docs/knowledge/areas/human-mailbox/overview.md`, and the promotion tree in
`skills/bee-capturing/references/promotion.md`. `commands.verify` was
retired in 2.1.0 and `commands.test` takes a list (claim 3), so D1 adds no
key. The backlog of this repo already holds 260 `fix-at:check` and 31
`fix-at:architecture` rows (`rg -o` over `.bee/backlog.jsonl`), so the row
count D4 reports is real data on day one.

## Approach

Recommended path: three small code cells and one doctrine cell, each
additive. trl-1 adds a static-check detector beside `detect_commands` and
two notice arms (D1, C1). trl-2 adds `--source` to the reflect verb and
its letter render (D3, C2). trl-3 adds the `prose_load` object to the
digest (D4, C3). trl-4 writes the craft page, the pointers and the
doctrine lines, then runs the regen chain once (D2, D3). trl-5
consolidates knowledge.

Rejected: a `commands.check` key (reopens the verify/test split); new
fix-at values `type|lint|test` (a locked decision keeps four); a `fix_at`
field on review findings (a second counter); a hard prose budget (a guard
that cannot pass); writing a CI file into the host (the host owns its CI).

Risk map:
- An existing onboarding notice changes for a project with no static check — MEDIUM — the one fact: the new arms add text only when the detector returns a candidate — Lands in: trl-1 — proof: a test that asserts the no-candidate notice text equals today's string, for both the detected-commands arm and the ask-the-user arm.
- A list-valued `commands.test` changes what onboarding prints — MEDIUM — the one fact: today that config wrongly draws the not-recorded notice (claims 15, 16), and D5 makes the fix intended — Lands in: trl-1 — proof: a test that pins the new empty result for a list with nothing detected.
- A default reflect call writes a different row — MEDIUM — the one fact: `source` is stored only when it is `user` or `review` — Lands in: trl-2 — proof: a test that compares the stored entry of a call without `--source` to the entry shape before the change.
- The digest fails on a host with no knowledge bundle — LOW — Lands in: trl-3 — proof: a test on an empty repo root that returns zeros.
- A doctrine edit breaks a parity or pointer test — LOW — Lands in: trl-4 — proof: `rule_index_parity`, `pointer_integrity`, `instruction_laws`.

Waves: serial, trl-1 → trl-2 → trl-3 → trl-4 → trl-5. Reason: every cell's proof builds the one `bee` crate in the one worktree, so a sibling's half-written edit would break another cell's build (a single scarce resource). The product files of the cells are disjoint; the order is a build edge, not a data edge. After the merge the leader rebuilds and reinstalls `.bee/bin/bee` and checks a new string in the installed binary before calling the feature live.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"trl-1, trl-2 and trl-3 write Rust and its tests."},
    {"stage":"docs","classification":"required","role":"docs","reason":"trl-4 writes the craft page, the pointers and the doctrine lines, and runs the regen chain."},
    {"stage":"writing","classification":"required","role":"writer","reason":"trl-5 consolidates knowledge and writes delivery.md."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard with more than five product files: the three default seats run the plan check."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default seat; cites the SMALLER PATH check below."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default seat; the notices and the digest line are user-facing text."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Not high-risk; no hard-gate flag."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Not high-risk."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release runs through scripts/release.sh only when the owner asks."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each cell writes its own red-first tests."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader read the code during planning."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"slice-judge","classification":"required","role":"review","reason":"Standard lane: one read-only checklist judge covers the behavior-change cells trl-1, trl-2 and trl-3. Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

Epic map, one slice. Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied). Step 6 of that playbook (statechart) does not apply: no event-order flow is added.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Static layer for hosts | onboarding offers the project's static checks | D1 | 1 | `static_check` tests |
| Counted corrections | a reflection carries its source | D3 | 1 | `reflect_source` tests + registry contract test |
| Prose witness | the digest reports the prose load | D4 | 1 | `prose_load` tests |
| Rule landing | craft page + pointers + doctrine lines | D2, D3 | 1 | manifest check, onboard, parity and pointer tests |

SMALLER PATH: docs only (tell agents to add lint to `commands.test` by hand) is cheaper, but it leaves D1 in the prose layer, the layer this feature exists to shrink, and D3 and D4 need stored data. One merged code cell is cheaper in ceremony, but it puts three unrelated contracts in one brief. Dropping the regen from trl-4 is cheaper, but the vendored trees would then drift from `skills/`. PASS.

PLAN CHECK (three seats, one round). STRUCTURE — BLOCKERS, all closed in this draft: the recorded rule for a list-valued `commands.test` contradicted case (c) (closed by D5 and claims 15-16); trl-2 left out the other places that build the stored entry (files added, D6); the claim that the regen chain reads the command registry was unsupported (removed; the serial order rests on the shared build alone). WARNINGS applied: trl-3 anchored on the count verb (re-anchored on `build_digest` and `digest_summary_line`, `prose_load` at the top level, D7); absent sources read as null and bee's lines are apart from the project's (D7); the marker-word match, the lockfile runner and run-before-offer (D5); the reviewing-skill sentence dropped for one doctrine home (D6); the registry hand-edit rule and the worktree onboard note carried into the cells; trl-5 names its onboarding target. Dismissed: counting nested or tool-specific instruction files (out of D4's fixed list); a status or doctor surface for the digest line (not asked; named in delivery limits); a pointer from the backlog row text to the craft page (the promotion tree is where rows are taken); monorepo and further ecosystem detection (silence is safe; open choice). CELLS (reviewed: 5) — CRITICAL: none after the fixes above. MINOR: a name-filtered proof passes on zero matches, so each code cell must quote its matched test count.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| trl-1 | Offer a project's own static checks at onboarding | onboard/notices.rs, onboard/templates.rs | — | `bee onboard` in a project with a `lint` script and a recorded test command prints one notice that names `npm run lint` and says to offer it first in the test list | `cargo test … --bin bee -- static_check` |
| trl-2 | Record where a reflection came from | verbs/mailbox.rs, verbs/mailbox_digest.rs, cells/handlers_close.rs, drivers/close.rs, verbs/work.rs, generated/registry_payload.json | trl-1 | `bee mailbox reflect … --source user` stores the source and the letter says "your correction"; `--source boss` is refused with the three values | `cargo test … --bin bee -- reflect_source` and `--test registry_contracts` |
| trl-3 | Report the prose load in the feedback digest | verbs/feedback.rs | trl-2 | the sentence `bee feedback digest` prints also names the project's instruction lines, bee's lines, the critical pattern count and the fix-at rows filed | `cargo test … --bin bee -- prose_load` |
| trl-4 | Write the static-rules craft page and wire the doctrine | expertise/static-rules.md (new), expertise/INDEX.md, promotion.md, packages/bee/LEADER.md, regen outputs | trl-1, trl-2, trl-3 | the promotion tree and the expertise index point to a page that says how a check row becomes a lint rule; the leader doctrine says to record a user correction with its source | manifest check + onboard + parity and pointer tests |
| trl-5 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, delivery.md (new) | trl-1, trl-2, trl-3, trl-4 | the onboarding, mailbox and feedback concepts describe the new behavior | `.bee/bin/bee knowledge check` |

```json
[
  {
    "id": "trl-1",
    "feature": "trust-layers",
    "title": "Offer a project's own static checks at onboarding",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "420c8c96-ef15-4522-bc96-b7109044af2b",
      "7bd71a3f-3509-47a7-835c-a71c6e48598d",
      "4d39a59f-ae3f-4bc9-9d0d-cd5c19367e79"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/trust-layers/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/onboarding/"
    ],
    "action": "Implement docs/history/trust-layers/CONTEXT.md D1 as refined by D5, and contract C1. Anchors in onboard/notices.rs: `pub fn detect_commands(root: &Path) -> Vec<Candidate> {`, `pub fn commands_notices(repo_root: &Path, first_onboard: bool) -> Vec<String> {`, the predicate `let recorded = COMMAND_KEYS.iter().any(|key| {` inside it, and `fn declares_test(commands: Option<&serde_json::Map<String, Value>>) -> bool {`; in onboard/templates.rs: `pub const NO_TEST_VERIFICATION_OFFER` (the offer posture to follow). (1) Detector. Add a static-check detector beside detect_commands, with its own result type (do not reuse Candidate, whose key comes from COMMAND_KEYS): input the repo root; output at most one candidate per kind, the kinds being lint and type check; each candidate carries the command, the source file and a marker word. It reads the project's own root files only and runs no tool. Lint kind, first hit wins: a package.json script named lint; a Makefile target named lint; a composer.json script named lint; Cargo.toml (`cargo clippy -- -D warnings`, marker clippy); pyproject.toml holding a `[tool.ruff` table (`ruff check .`, marker ruff); go.mod (`go vet ./...`, marker vet). Type-check kind, first hit wins: a package.json script named typecheck or type-check; a Makefile target named typecheck; pyproject.toml holding a `[tool.mypy` table (`mypy .`, marker mypy). For a manifest script the marker word is the script name. For a package.json script the runner follows the lockfile at the root: pnpm-lock.yaml gives `pnpm run <name>`, yarn.lock gives `yarn <name>`, bun.lockb or bun.lock gives `bun run <name>`, otherwise `npm run <name>`. A malformed manifest yields no candidate from that file and never panics. (2) Recorded rule (D5). In commands_notices a test command counts as recorded when commands.test is a non-empty string or a non-empty list (the declares_test rule); setup and start keep their string rule. So a list-valued commands.test no longer draws the `No standard commands recorded` notice; this is an intended fix, and a test must pin it. (3) Notices. Case (a): no command recorded and at least one static check detected: the existing first notice (either arm) gains one sentence that lists the static checks with their sources and says to offer them as the first members of the `commands.test` list, before the tests. Case (b): a test command is recorded and it is not the sentinel none, and at least one detected check has its marker word in no commands.test entry: return one notice that lists those checks with their sources and says to offer adding them before the tests in the `commands.test` list. Both cases also say: run each proposed check once first and tell the user whether it is green or how many findings it reports; look through the project's recorded decisions for an earlier answer about that check and stay silent about a check the user already refused; record the user's answer for each check as a decision; never write an unconfirmed value. Case (c): commands.test is the sentinel none, or nothing is detected, or only setup or start is recorded: return exactly what the function returns today for that config (for a list-valued test with nothing detected that is now an empty list, per (2)). The Greenfield notice text does not change. Keep fixed notice text as constants in templates.rs. Red first, with test names that contain static_check: one candidate per kind and manifest beats convention; the lockfile picks the runner; a malformed package.json; case (a) for both arms; case (b) fires for a string and for a list that lacks the marker word and is silent when an entry contains it (for example `pnpm lint` or `cargo clippy --all-targets -- -D warnings`); a list-valued commands.test with nothing detected returns nothing; the none sentinel; and with no candidate the two existing arms equal today's exact strings. No code comments. The proof run must show the new tests ran: quote the count of tests the filter matched, which must be above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- static_check",
    "must_haves": {
      "truths": [
        "A project with a lint script and a recorded commands.test that lacks the word lint gets one notice naming the lint command and its source",
        "A project whose commands.test already has an entry containing the check's marker word gets no static-check notice",
        "A project with no command recorded and a static check found gets the existing notice plus the static checks, offered first in the commands.test list",
        "A list-valued commands.test counts as recorded and no longer draws the No standard commands recorded notice",
        "With no candidate detected, or commands.test set to none, a string-valued or empty config gets exactly the strings it got before",
        "Each static-check notice tells the agent to run the check once before offering it and to record the user's answer as a decision",
        "The detector reads files only and runs no tool"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/notices.rs",
          "substantive": "the static-check detector, the two notice arms and their static_check tests"
        }
      ],
      "key_links": [
        "commands_notices calls the static-check detector"
      ],
      "prohibitions": [
        "Never write a detected command to .bee/config.json",
        "No new commands key and no change to COMMAND_KEYS",
        "No code comments"
      ],
      "open_choices": [
        "Extra marker files beyond the listed ones"
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
  },
  {
    "id": "trl-2",
    "feature": "trust-layers",
    "title": "Record where a reflection came from",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "trl-1"
    ],
    "decisions": [
      "df297521-5dee-4a2f-a5cd-fe27b7440ba0",
      "8ddbe4ab-bf33-43ab-a031-ccba81bae765",
      "eb87cd73-2ca9-4b8f-a5fe-13af040488dc"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
      "packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/work.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/trust-layers/CONTEXT.md",
      "docs/knowledge/areas/human-mailbox/overview.md",
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
      "packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/human-mailbox/overview.md"
    ],
    "action": "Implement docs/history/trust-layers/CONTEXT.md D3 (the verb half) as refined by D6, and contract C2. Anchors in verbs/mailbox.rs: `pub(crate) const FIX_AT_VALUES: [&str; 4] = [\"architecture\", \"check\", \"doctrine\", \"none\"];`, the function `read_reflection` holding `let missing: Vec<&str> = [(\"--wrong\", wrong), (\"--better\", better), (\"--fix-at\", fix_at)]`, `pub(crate) fn reflection(at: &str, wrong: &str, better: &str, fix_at: &str) -> Self {`, the structs `Entry` and `LetterItem` with their `to_value`, and the frontmatter writer that uses `emit_opt`; in verbs/mailbox_digest.rs: `fn reflection_key(fix_at: Option<&str>, what: &str) -> Option<String> {`. Add an optional --source flag to bee mailbox reflect with the closed, lower-case, exact set self, user, review; default self. Shape: an optional source field on Entry and on LetterItem (the entry must hold every fact the letter prints). Entry and LetterItem are built by struct literals in other places too: `let entry = mailbox::Entry {` in verbs/cells/handlers_close.rs (two sites) and verbs/drivers/close.rs, and `Entry {` in a test helper in verbs/work.rs; give each of those sites an absent source and change nothing else there. Absent or self: the stored entry row carries no source key, and the letter item carries none, so both are byte-identical to before this change (the serializers must insert the key only when it is set, never as null). user or review: the entry row and the letter item carry source with that word, and the Mistakes & reflection bullet for that entry says `your correction` for user and `from a review` for review. Any other value, an empty value included: exit non-zero, append nothing, and name the three legal values in the error. --source together with --no-mistakes: refused, the same way --wrong with --no-mistakes is. The --json result of the verb carries source when it is set. A reflection read back from a cap trace has no source. reflection_key must not read source: two reflections with the same layer and the same first four words fold into one key whatever their sources. An entry read back with no source key is a self reflection. Register --source on the mailbox reflect verb in generated/registry_payload.json with a description of the three values, so bee mailbox reflect --help lists it and the CLI-shape guard accepts it. That file is one compact JSON line and is hand-edited: change it by a JSON round-trip that keeps its compact byte format and key order. The flag name source already exists in the registry vocabulary, so the pinned distinct-flag count in catalog.rs must not change; if a test says it does, stop and report. Do not change the --mistake path of the cap verb beyond the absent source. Red first, with test names that contain reflect_source: a default call stores no source key and its letter item has none; user and review are stored and rendered with the two phrases; an off-set value and an empty value refuse and append nothing; --source with --no-mistakes refuses; a user reflection and a self reflection with the same layer and what produce the same lesson key. No code comments. The proof run must show the new tests ran: quote the count of tests the filter matched, which must be above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- reflect_source && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts",
    "must_haves": {
      "truths": [
        "bee mailbox reflect with --source user stores source user on the entry and the letter bullet says your correction",
        "bee mailbox reflect without --source writes an entry row and a letter item with no source key",
        "A --source value outside self, user, review exits non-zero, appends nothing and lists the three values",
        "The lesson key of a reflection does not depend on its source",
        "The registry lists --source on mailbox reflect"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
          "substantive": "the --source flag, its validation, the stored field and reflect_source tests"
        }
      ],
      "key_links": [
        "generated/registry_payload.json registers --source on mailbox reflect"
      ],
      "prohibitions": [
        "No change to FIX_AT_VALUES",
        "No change to the stored shape of a default reflection or of the clean-run answer",
        "No code comments"
      ],
      "open_choices": []
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
  },
  {
    "id": "trl-3",
    "feature": "trust-layers",
    "title": "Report the prose load in the feedback digest",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "trl-2"
    ],
    "decisions": [
      "04b0108e-1397-4f27-bf03-18cfa9a4a299",
      "60f72e0f-2289-46b8-b39b-0485290a3edc",
      "c5fc83dd-c32c-4fc2-a04d-8862fe9459be"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs"
    ],
    "read_first": [
      "docs/history/trust-layers/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/human-mailbox/overview.md"
    ],
    "action": "Implement docs/history/trust-layers/CONTEXT.md D4 as refined by D7, and contract C3, in verbs/feedback.rs. Anchors: `fn build_digest` (it builds the digest object), `fn digest_summary_line` (the one printed sentence, where the mistakes witness already adds a clause), `fn collect_feedback(root: &Path) -> Option<CollectData> {` (it already walks .bee/backlog.jsonl row by row and reads each row's layer), and the test `the_digest_counts_clean_run_answers_against_real_reflections` (how a digest test builds its fixture). The functions collect_counts, counts_value and summary_line serve bee feedback count: do not change them, and the counts object stays byte-identical. Add one top-level key prose_load to the digest object. It holds: for each of AGENTS.md, CLAUDE.md and LEADER.md that exists at the repo root, its path and line count, and their sum as the project lines; the line count of .bee/LEADER.md as the bee lines; the number of critical patterns, counted as the top-level list lines (lines that start with `- `) under the `## Critical patterns` heading of docs/knowledge/index.md up to the next `## ` heading; and the number of rows filed in .bee/backlog.jsonl whose layer is exactly fix-at:check and the number whose layer is exactly fix-at:architecture, tallied from the raw rows in the existing backlog walk, before any row is dropped or folded. A file that is a symbolic link to another counted file is counted once. A source that is absent (no such file, no such section, no backlog) reads as null in the object, never as zero, following the existing witness, which uses null for not measured. An unreadable or malformed row is skipped. Nothing here fails or refuses the digest. Add one clause to the printed digest sentence that names, in plain words, the project lines, the bee lines, the critical pattern count and the two counts of rows filed; an absent source prints as not present, never as 0. It is a witness: it refuses nothing, drops nothing, and adds no score or threshold. The digest carries counts and the fixed file names only, never file content. Red first, with test names that contain prose_load: a fixture root with AGENTS.md, a .bee/LEADER.md, an index with three top-level critical pattern lines and one nested line, and a backlog with two check rows and one architecture row yields those exact numbers in the object and in the printed sentence; a CLAUDE.md that is a symbolic link to AGENTS.md is not counted twice (unix only: gate that test with cfg(unix)); an empty root yields nulls, the words not present, and a successful digest; the counts object of an empty root is unchanged. No code comments. The proof run must show the new tests ran: quote the count of tests the filter matched, which must be above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- prose_load",
    "must_haves": {
      "truths": [
        "The digest object holds a top-level prose_load with project lines, bee lines, the critical pattern count and the two fix-at row counts",
        "The printed digest sentence names project lines, bee lines, critical patterns and rows filed",
        "An absent source reads as null in the object and as not present in the sentence, and the digest still succeeds",
        "bee feedback count and the counts object are unchanged",
        "The digest holds no content of the instruction files"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
          "substantive": "the prose_load collection, its place in the digest object and printed line, and prose_load tests"
        }
      ],
      "key_links": [
        "build_digest puts prose_load on the digest object and digest_summary_line prints its clause"
      ],
      "prohibitions": [
        "No refusal, threshold or score from the prose load",
        "No code comments"
      ],
      "open_choices": [
        "The key names inside prose_load"
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
  },
  {
    "id": "trl-4",
    "feature": "trust-layers",
    "title": "Write the static-rules craft page and wire the doctrine",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [
      "trl-1",
      "trl-2",
      "trl-3"
    ],
    "decisions": [
      "135ea187-5a1c-42ad-ad8d-8584e92b4dc2",
      "df297521-5dee-4a2f-a5cd-fe27b7440ba0",
      "eb87cd73-2ca9-4b8f-a5fe-13af040488dc"
    ],
    "files": [
      "expertise/static-rules.md",
      "expertise/INDEX.md",
      "skills/bee-capturing/references/promotion.md",
      "packages/bee/LEADER.md",
      ".bee/LEADER.md",
      ".bee/expertise/*",
      ".bee/onboarding.json",
      ".claude/skills/*",
      ".agents/skills/*",
      ".claude-plugin/skills/*",
      ".codex-plugin/skills/*",
      ".opencode/skills/*",
      "skills-lock.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/trust-layers/CONTEXT.md",
      "expertise/INDEX.md",
      "expertise/tests.md",
      "skills/bee-capturing/references/promotion.md",
      "docs/knowledge/areas/human-mailbox/overview.md",
      "docs/knowledge/patterns/20260713-promote-an-order-to-the-always-loaded-layer.md"
    ],
    "affects_skills": [
      "skills/bee-capturing/references/promotion.md"
    ],
    "affects_specs": [],
    "action": "Implement docs/history/trust-layers/CONTEXT.md D2 and the doctrine half of D3. Load the bee-technical-writing skill first; the craft page is a how-to guide, one type per file. (1) Write expertise/static-rules.md in the voice and frontmatter shape of the other craft pages in expertise/ (read expertise/tests.md for the shape). Content, generic to any project, never about bee itself: when a mistake's fix-at layer is check, pick the highest of four homes that can hold the rule, in this order: a type the compiler enforces (make the wrong value impossible to build), a rule in the linter the project already runs, a test, a hook or guard. For the lint home, name where a project rule usually lives per ecosystem: ESLint (no-restricted-syntax, no-restricted-imports, or a local rule), Clippy (disallowed-methods and disallowed-types in clippy.toml), Ruff (banned-api under flake8-tidy-imports), golangci-lint (forbidigo, depguard), and a pattern rule tool such as ast-grep or Semgrep when the project's linter cannot express the rule. State the proof: show the new rule red on the past mistake, then green after the fix; a rule never seen red proves only that it loads. State where the rule must run: it has to be a member of the project's commands.test list, or CI never sees it. State that a tool the project does not already run is the owner's decision: ask before adding one. State how to stage the red proof: rebuild the past mistake as a small fixture or a scratch edit, show the rule red on it, then remove it so it does not stay in the tree. State the dead end: when the project has no commands.test list, or it is the sentinel none, take the question to the owner instead of writing a rule nothing runs. Add a one-line list of rule homes for other ecosystems (.NET banned-API analyzers, ErrorProne or ArchUnit for Java, detekt for Kotlin, RuboCop custom cops, PHPStan custom rules, SwiftLint custom_rules, Credo), and say that a project with no compiler-checked types starts at the lint home. State when not to write a rule: a one-off, or a rule with false positives that people will switch off. Keep it under 140 lines. (2) Add one row for static-rules.md to the Craft table of expertise/INDEX.md. (3) In skills/bee-capturing/references/promotion.md, step 2 of the Promotion Decision Tree (anchor: `2. Mechanizable? A grep/lint line in a verify command, a guard, a hook`), add one sentence that points to `.bee/expertise/static-rules.md` for which home to pick and how to prove it. (4) In packages/bee/LEADER.md, in the bullet that starts `Write a mistake down the MOMENT you notice it` (anchor: `bee mailbox reflect --wrong \"<what went wrong>\"`), add at most three lines: a correction from the user of how you worked (not a new instruction or a changed requirement) is a mistake too, recorded the same way with `--source user` the moment it is given, with the user's own words kept in `--wrong` so a repeat matches; an accepted P1 or P2 review finding takes `--source review`; the same key seen twice is a fix-first candidate whatever its source. (5) Run the regen chain: `.bee/bin/bee dev regen`, then `.bee/bin/bee dev release-manifest --write`, then `.bee/bin/bee onboard --repo-root . --json` and, when it reports changes_needed, `.bee/bin/bee onboard --repo-root . --apply --json`, so .bee/LEADER.md, .bee/expertise/ and the vendored skill trees match their sources. `.bee/bin/bee` in this worktree is the installed binary of the main checkout; the regen chain reads the source files of this worktree, so that binary is the right one to run. In a worktree the onboard report may still list runtime files such as .bee/reservations.json as changes_needed; that is expected, and no skill, expertise or leader file may remain in its plan after the apply. Never hand-edit a vendored tree. If a parity test pins a line count or a rule list of LEADER.md, keep the new lines inside the existing rule block and update the pinned value only where the test names it. Cite CONTEXT.md D-numbers in prose, never raw decision ids. One commit.",
    "verify": ".bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity --test instruction_laws",
    "must_haves": {
      "truths": [
        "expertise/static-rules.md states the four homes in order, the per-ecosystem rule homes, the red-then-green proof with its fixture removed, that a new tool needs the owner's yes, and when not to write a rule",
        "expertise/INDEX.md routes to static-rules.md",
        "Step 2 of the promotion tree points to the static-rules page",
        "The leader doctrine says a user correction is recorded as a reflection with --source user in the user's own words, and an accepted P1 or P2 review finding with --source review",
        "The vendored copies under .bee/ and the skill trees match their sources after the regen chain"
      ],
      "artifacts": [
        {
          "path": "expertise/static-rules.md",
          "substantive": "the how-to guide for turning a check row into a static rule"
        },
        {
          "path": ".bee/expertise/static-rules.md",
          "substantive": "the vendored copy, written by the regen chain"
        }
      ],
      "key_links": [
        "docs/history/codex-harness-hardening/release-manifest.json lists the new craft page so hosts receive it"
      ],
      "prohibitions": [
        "No hand edit of a vendored tree",
        "No change to FIX_AT_VALUES or to any refusal",
        "No more than three new lines in packages/bee/LEADER.md",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "trl-5",
    "feature": "trust-layers",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": [
      "trl-1",
      "trl-2",
      "trl-3",
      "trl-4"
    ],
    "decisions": [
      "420c8c96-ef15-4522-bc96-b7109044af2b",
      "135ea187-5a1c-42ad-ad8d-8584e92b4dc2",
      "df297521-5dee-4a2f-a5cd-fe27b7440ba0",
      "04b0108e-1397-4f27-bf03-18cfa9a4a299",
      "4d39a59f-ae3f-4bc9-9d0d-cd5c19367e79",
      "eb87cd73-2ca9-4b8f-a5fe-13af040488dc",
      "c5fc83dd-c32c-4fc2-a04d-8862fe9459be"
    ],
    "files": [
      "docs/knowledge/**",
      "docs/history/trust-layers/delivery.md"
    ],
    "read_first": [
      "docs/history/trust-layers/CONTEXT.md",
      "docs/history/trust-layers/plan.md",
      "docs/knowledge/areas/human-mailbox/overview.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the writer consolidation step on this feature: run `bee knowledge consolidate --feature trust-layers`. No onboarding concept documents the command notices today: add a short section on them and on the static-check offer (D1, D5), with the recorded rule for a list-valued commands.test, to docs/knowledge/areas/onboarding/host-project-artifacts.md, or to the onboarding concept you find closer after reading the area index; add the --source field (D3) and the prose-load witness (D4) to docs/knowledge/areas/human-mailbox/overview.md beside the reflection rules and 'A mistake seen twice'; name the static-rules craft page (D2) where the promotion tree is described. Dedupe and trim the deltas the cells wrote, anchor each concept to the code it describes, never restate code, and write docs/history/trust-layers/delivery.md listing the concepts changed and why, one line each, plus a Known limits section with these three: a repeat counts across distinct runs only, and only when the first four words of the what match; the prose-load line is printed by bee feedback digest and no host routine prints it yet; the installed .bee/bin/bee carries the feature only after a rebuild and reinstall. Cite CONTEXT.md D-numbers, never raw decision ids. Run `.bee/bin/bee knowledge index` when a concept's frontmatter changed, then `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "area and pattern concepts describe the feature's changes, anchored to code",
        "docs/history/trust-layers/delivery.md lists every concept the feature changed"
      ],
      "artifacts": [
        {
          "path": "docs/history/trust-layers/delivery.md",
          "substantive": "the feature's delivery record"
        }
      ],
      "key_links": [
        "the delivery lands in the same merge as the code"
      ],
      "prohibitions": [
        "No restating of code in area concepts",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

Standard: the triad, each row inside its cell.

| Case | Probe | Pass when |
|---|---|---|
| Happy path, D1 | repo with `package.json` script `lint`, `commands.test` = `npm test` | one notice names `npm run lint` and `package.json` |
| Edge, D1 | `commands.test` = `["npm run lint","npm test"]` | `commands_notices` returns nothing |
| Edge, D1 | `commands.test` = `pnpm lint && pnpm test`, lint script present | no static-check notice |
| Edge, D1 | `commands.test` = `none` | output equals today's |
| Error path, D1 | malformed `package.json` | no panic, no static-check notice from that file |
| Happy path, D3 | `reflect --wrong a --better b --fix-at check --source user` | entry has `source: user`; letter bullet says "your correction" |
| Edge, D3 | same call without `--source` | entry has no `source` key |
| Error path, D3 | `--source boss` | non-zero exit, nothing appended, error lists self, user, review |
| Happy path, D4 | fixture with two instruction files, three critical patterns, three fix-at rows | object and printed line carry those numbers |
| Edge, D4 | empty repo root | nulls, "not present" in the sentence, digest succeeds |
| Parity, D2 | regen chain, then manifest check and onboard | manifest check green; no skill, expertise or leader file left in the onboard plan |

## Open Questions

(none)

## Out of scope

- A CI workflow file written into a host project.
- A `fix_at` field on review findings.
- A hard budget or refusal on the prose load.
- A `--source` flag on `bee cells cap --mistake` (a cap is always the worker's own answer).
- A release; it runs through `scripts/release.sh` when the owner asks.
- A status or doctor line that shows the prose load in a host, and a pointer from the backlog row text to the craft page.
- Static-check detection below the repo root (monorepos) and for ecosystems past the listed markers.
