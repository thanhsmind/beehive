---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Five checks from pstack, built into the bee binary

## Summary

bee gets five checks that pstack-craft-adoption ranked and left for later:
a `bee plan check` verb, an Upstream cells block in each worker prompt, a
patch-id stored at cap and read at merge, a dead-pointer warning at
`bee close`, and a rolling refill for `bee dispatch wave --run`. Every check
reports; none refuses.

Mode: `standard` — 2 risk flags: public contracts (a new verb, new JSON
fields, a new flag), covered contract change (the merge carried-proof check
and the worker prompt have tests that pin them).

## Requirements (from CONTEXT.md)

- D1 (`c9ae60cb`): `bee plan check`, four checks, exit 0 or 10, preview warnings.
- D2 (`03da9d9b`): Upstream cells block for capped deps; no deps renders as today.
- D3 (`77258729`): `trace.report.patch_id` at cap; merge uses it first; advisory.
- D4 (`682e3531`): dead proof pointer warnings and `dead_proof_pointers` at close.
- D5 (`806c6687`): `dispatch wave --run --rolling` refills herding slots.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The claims parse already refuses guessed rows, so D1 reuses it | read | packages/bee-rs/crates/bee/src/verbs/state_group/plan_claims.rs:144 | `if label == "guessed" {` |
| 2 | Gate preview has one body function where D1's warnings print | read | packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:487 | `pub(crate) fn run_gate_preview_body(root: &Path, flags: &Flags, _use_json: bool) -> R2<Out> {` |
| 3 | The cell prompt reads only the cell's own trace today | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:808 | `pub(crate) fn prior_round_event_lines(cell: &Value) -> Vec<String> {` |
| 4 | The prompt template has a conditional block D2 can copy | read | packages/bee/prompts/worker-cell.md:53 | `{{#if prior_rounds}}` |
| 5 | Merge computes a patch-id from the commit object | read | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:119 | `fn commit_patch_id(main_root: &Path, commit: &str) -> Option<String> {` |
| 6 | Close runs one feature proof check D4 sits beside | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2675 | `let proof = crate::verbs::cells::feature_proof_check(root, feature)?;` |
| 7 | The proof line parse splits three segments | read | packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:137 | `pub(crate) fn parse_tests_proof(s: &str) -> Option<(String, String, String)> {` |
| 8 | `--limit` truncates the first wave only | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4715 | `wave_ids.truncate(n);` |
| 9 | The herding half runs from one function D5 extends | read | packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs:19 | `pub(crate) fn run_wave_with(` |
| 10 | A recent small verb group is wired through verbs/mod.rs | read | packages/bee-rs/crates/bee/src/verbs/mod.rs:53 | `pub mod upkeep_group;` |

## Approach

Class playbook: `skills/bee-planning/playbooks/feature.md`. Step 1 (data
shape): each decision names its record first — the plan-check JSON (D1),
the Upstream block fields (D2), `trace.report.patch_id` (D3),
`dead_proof_pointers` (D4). Step 2 (walking skeleton): each cell runs its
check end to end through the real verb, no stub. Step 4 (user path): each
cell's proof drives the verb a user runs, in a temp store. Step 5: the
writer cell syncs the five knowledge concepts. Step 6 (statechart): the
rolling refill is an event-order loop, but no `statechart` skill is on this
host; skipped, and psrc-5's tests cover the refill order instead. Step 7:
no formal-verification skill on this host; skipped.

Four waves, split only where two cells write the same file:

- Wave 1, at the same time: psrc-1 (plan check: new verb, registry),
  psrc-3 (patch-id: cap and merge), psrc-4 (close warning, in a new module).
- Wave 2: psrc-2 (Upstream block) after psrc-1, both edit the registry.
- Wave 3: psrc-5 (rolling refill) after psrc-2, both edit prepare.rs.
- Wave 4: psrc-6 writer: knowledge sync, the bee-planning pre-flight line,
  delivery.md.

SMALLER PATH check: a cheaper shape is one cell per file group with no
writer cell. FAIL — five behaviors in public contracts need their knowledge
synced, and the plan-check verb is worth nothing if bee-planning never
tells the leader to run it.

Rejected: making the checks refuse (CONTEXT Out of scope: no new refusal
until a check has run clean on real plans); rolling refill for Agent-tool
items (bee cannot see when a native Agent call ends).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "psrc-1 to psrc-5 write Rust and their tests."},
    {"stage": "writing", "classification": "required", "role": "writer", "reason": "psrc-6 consolidates knowledge and writes delivery.md."},
    {"stage": "slice-judge", "classification": "required", "role": "review", "reason": "Standard lane: one read-only checklist judge covers the behavior-change cells. Independent review stays user-invoked."},
    {"stage": "read-only-gather", "classification": "required", "role": "read", "reason": "The 2026-11-14 gather confirmed the five surfaces and their anchors."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Every claim row was opened this session after a gather; the decisions are the owner's own ask."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "The Approach names the rejected paths and the SMALLER PATH check."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Every change is additive or advisory; no new refusal."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "No door or guard changes; the merge check stays advisory."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "The owner asked for this directly."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "psrc-6 runs as writer."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "test-and-live-proof", "classification": "not-applicable", "role": "test", "reason": "Each cell carries its own tests; bee test runs once before merge."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Every cell has a configured role."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Standard lane; no high-risk advisor consult."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind lanes."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind lanes."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind lanes."},
    {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."}
  ]
}
```

## Risk map

| Risk | Guard | Lands in |
|---|---|---|
| Plan check flags most old plans and is ignored | It reports only; exit 10 is a signal, never a refusal | psrc-1 |
| Upstream block changes prompts of cells with no deps | D2: byte-identical render without deps; a test pins it | psrc-2 |
| A stored patch-id hides a real code change | The stored id is compared with the branch set; a change in the patch reads stale | psrc-3 |
| The path heuristic flags command words as paths | Only slash-bearing tokens that are not URLs or flags; a test with a real proof line | psrc-4 |
| Rolling loop never ends | Ends when none ready and none live; bee never caps, so a failed cell does not unblock its dependants | psrc-5 |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| psrc-1 | Add bee plan check and print its findings in gate preview | new plan verb, plan_claims, plan_packets, registry | — | `bee plan check --json` lists findings and exits 10 on a plan with a missing section | plan_group tests, registry_contracts |
| psrc-3 | Store a patch-id at cap and read it at merge | cap handler, worktree phases, their tests | — | a capped cell's report carries patch_id; a rebased cell still reads carried at merge | patch_id and worktree tests |
| psrc-4 | Warn at bee close on proof lines that name missing paths | close.rs, new proof_pointers module | — | `bee close` prints one warning per missing path and JSON dead_proof_pointers | dead_proof tests |
| psrc-2 | Relay capped dependency outcomes into the worker prompt | prepare.rs, worker-cell.md, registry | psrc-1 | a cell with a capped dep gets an Upstream cells block | upstream tests, prompt render tests |
| psrc-5 | Refill wave slots as herding runs finish | wave_run.rs, prepare.rs, registry | psrc-2 | `dispatch wave --run --rolling --limit 2` starts the third cell when one of two ends | rolling and wave_run tests |
| psrc-6 | Sync knowledge and the planning pre-flight for the five checks | five knowledge concepts, bee-planning, delivery.md | all | knowledge docs name the five checks; bee-planning tells the leader to run bee plan check | knowledge check, release-manifest check |

```json
[
{"id":"psrc-1","feature":"pstack-rust-checks","title":"Add bee plan check and print its findings in gate preview","lane":"standard","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml plan_group && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml plan_claims && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts",
"must_haves":{"truths":["bee plan check --json on a standard plan with every section in order, a cited playbook, every D-number and matching evidence prints ok true and exits 0","a standard plan missing Risk map gets a sections finding and exit 10","a plan whose sections are out of order gets a sections finding","a plan with no skills/bee-planning/playbooks/<class>.md citation gets a playbook finding","a D-number of CONTEXT.md absent from plan.md gets a decisions finding naming it","a read row whose evidence is not a byte substring of the anchored line gets an evidence finding naming the row","bee gate --preview prints each finding as a warning line and still succeeds","bee plan check writes no file"],"prohibitions":["No new refusal anywhere","No code comments"]},
"decisions":["c9ae60cb-5d2b-4b6f-b9d2-3c52a14bbe18"],
"files":["packages/bee-rs/crates/bee/src/verbs/plan_group.rs","packages/bee-rs/crates/bee/src/verbs/mod.rs","packages/bee-rs/crates/bee/src/router.rs","packages/bee-rs/crates/bee/src/catalog.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json","packages/bee-rs/crates/bee/src/verbs/state_group/plan_claims.rs","packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs","packages/bee-rs/crates/bee/tests/registry_contracts.rs"],
"read_first":["docs/history/pstack-rust-checks/CONTEXT.md","docs/history/pstack-rust-checks/plan.md","packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs","packages/bee-rs/crates/bee/src/verbs/state_group/plan_claims.rs","packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs","docs/history/upkeep-due-signal/plan.md"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/workflow-state/gates.md"],
"action":"Implement CONTEXT.md D1. New verb group `bee plan check`, modeled on verbs/upkeep_group.rs (try_native wiring in verbs/mod.rs, the usage line in router.rs, the registry entry in generated/registry_payload.json with a full description; tests/registry_contracts.rs pins the shape). Flags: --feature <slug> (omitted: the calling session's bound lane, then the state record's feature; none resolves => typed refusal naming --feature), --json. Read docs/history/<feature>/plan.md (missing => typed error naming the path) and docs/history/<feature>/CONTEXT.md (missing => the decisions check reports one finding, not an error). mode = plan.md frontmatter `mode`. Route class = the lane record's route class for the feature (read it the way bee orient/route reads it); unknown class => the playbook check accepts any skills/bee-planning/playbooks/*.md citation. Checks, each producing findings {check, detail}: sections (standard|high-risk: '## Summary', '## Requirements', '## Load-bearing claims', '## Approach', '## Role assignments', '## Risk map', '## Cells' as heading prefixes, present and in that order; other modes: Summary, Load-bearing claims, Cells present); playbook; decisions (D-numbers from rows of the '## Locked Decisions' table whose first cell is D<n>; each must appear in plan.md as a whole word); evidence (parse the claims table with the existing plan_claims.rs parser — expose a pub(crate) row parse if needed; for label read: anchor 'path:N' or 'path:N-M' or 'path'; evidence cell with one outer backtick pair stripped; the evidence must be a byte substring of line N, of lines N..=M joined by newline, or of the whole file; path missing or line out of range => finding). Output JSON {schema:'bee-plan-check/v1', ok, feature, mode, findings:[...]}; text mode prints one line per finding then a summary line. Exit 0 when no finding, 10 when any. In plan_packets.rs run_gate_preview_body, run the same checks (one shared pure function, never shelling out) and print each finding as a 'warning: plan check: <check>: <detail>' line; it never changes the preview result. Read-only: no write. Tests (behavior, temp-dir repos; name them so the filter plan_group matches): clean plan exits 0; missing Risk map; out-of-order sections; missing playbook; missing D-number; evidence mismatch; evidence match on a range; preview prints the warning and still succeeds. No code comments. Proof: the verify command; quote the counts."},
{"id":"psrc-3","feature":"pstack-rust-checks","title":"Store a patch-id at cap and read it at merge","lane":"standard","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml patch_id && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::worktree",
"must_haves":{"truths":["a cap whose report commit resolves stores trace.report.patch_id equal to the stable patch-id of that commit","a cap whose commit does not resolve or is none stores no patch_id and still succeeds","at merge, a cell whose stored patch_id is in the branch patch-id set reads carried even when its commit object is gone","at merge, a cell whose stored patch_id is not in the branch set reads stale","a cell with no stored patch_id keeps the old compute-from-commit path","the merge preflight never refuses on this check","the three existing carried-proof tests in verbs/worktree/tests.rs stay green"],"prohibitions":["No new refusal","No code comments"]},
"decisions":["77258729-68bc-4667-9170-327760c0fa64"],
"files":["packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs","packages/bee-rs/crates/bee/src/verbs/cells/proof.rs","packages/bee-rs/crates/bee/src/verbs/cells/tests.rs","packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs","packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"],
"read_first":["docs/history/pstack-rust-checks/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs","packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs","packages/bee-rs/crates/bee/src/verbs/cells/proof.rs","docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],
"action":"Implement CONTEXT.md D3. (1) At cap (find where handlers_close.rs writes trace.report), when report.commit is a non-'none' value that resolves in the repo, compute its stable patch-id with the same command shape phases.rs:119 commit_patch_id uses (move that helper to a shared pub(crate) place both modules can call, never a second copy) and store it as trace.report.patch_id; failure to resolve stores nothing and never fails the cap. (2) In phases.rs split_carried_proof (around :178), for each flagged cell read trace.report.patch_id first; when present compare it with branch_patch_ids; when absent fall back to commit_patch_id(commit) as today. Found => carried, not found => stale. Keep the preflight advisory (phases.rs:672-686). Tests (behavior; names contain patch_id): cap stores patch_id; cap with commit none stores none; merge with stored patch_id whose commit object is unreachable (create the commit, record the id, rewrite history so the sha differs) reads carried; stored id not in branch reads stale; no stored id keeps the old path. Existing tests a_proof_taken_before_the_merge_base_is_named_and_the_merge_still_lands, a_proof_carried_across_a_rebase_is_named_and_the_merge_still_lands, a_proof_that_descends_from_the_merge_base_is_silent stay green unchanged. No code comments. Proof: the verify command; quote the counts."},
{"id":"psrc-4","feature":"pstack-rust-checks","title":"Warn at bee close on proof lines that name missing paths","lane":"standard","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml dead_proof",
"must_haves":{"truths":["bee close on a feature whose capped cell proof line names docs/missing/file.md prints one warning line naming the cell and that path","the close JSON carries dead_proof_pointers [{cell, path}]","a proof line naming only existing paths, URLs, flags and command words yields no warning and an empty dead_proof_pointers","the warning never changes whether close succeeds or refuses"],"prohibitions":["No new refusal","Do not edit generated/registry_payload.json","No code comments"]},
"decisions":["682e3531-bba9-4042-92cf-8accca31a67f"],
"files":["packages/bee-rs/crates/bee/src/verbs/drivers/close.rs","packages/bee-rs/crates/bee/src/verbs/drivers/proof_pointers.rs","packages/bee-rs/crates/bee/src/verbs/drivers/mod.rs"],
"read_first":["docs/history/pstack-rust-checks/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/drivers/close.rs","packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs","packages/bee-rs/crates/bee/src/verbs/cells/proof.rs","docs/knowledge/areas/workflow-state/merge-readiness.md"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/workflow-state/merge-readiness.md"],
"action":"Implement CONTEXT.md D4. New module verbs/drivers/proof_pointers.rs (declare it in drivers/mod.rs) with a pure function that takes a proof line (trace.report.tests, the '<command> — <result> — <scope reason>' string parsed by finish_support.rs parse_tests_proof) and returns candidate repo-relative paths: tokens split on whitespace, trimmed of surrounding quotes, backticks, commas, parentheses and a trailing colon or period; a candidate contains '/', does not start with http:// or https:// or '-', is not absolute, contains no '$', '*', '{' or '=' and is not a path:line form (strip a trailing ':N' or ':N-M' before checking). A second function checks each candidate under the repo root and returns the missing ones. In close.rs near the feature_proof_check call (:2675), for every capped cell of the feature collect missing paths; print 'warning: dead proof pointer: <cell>: <path>' per item in text mode and add dead_proof_pointers [{cell, path}] (always present, [] when none) to the JSON output. It never changes the close outcome. Unit tests live in proof_pointers.rs #[cfg(test)] (names contain dead_proof): tokenizer cases (URL, flag, path:line, glob, real path), the missing-path check in a temp dir, and one close-level test if the existing close test harness is reachable without editing drivers/tests.rs; otherwise test the collection function over a temp store. No code comments. Proof: the verify command; quote the counts."},
{"id":"psrc-2","feature":"pstack-rust-checks","title":"Relay capped dependency outcomes into the worker prompt","lane":"standard","role":"code","deps":["psrc-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml upstream && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml prior_round && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check",
"must_haves":{"truths":["dispatch prepare --kind cell for a cell with one capped dep renders an Upstream cells block naming the dep's id, title, outcome, commit, files and deviations","a dep that is not capped is listed by id with its status and no report fields","files past 20 render as a count line; each deviation is cut to 300 characters","a cell with no deps renders a prompt byte-identical to before this change","the holdout arm still renders the Upstream block"],"prohibitions":["No code comments"]},
"decisions":["03da9d9b-9d59-4bcc-93d4-2891538da070"],
"files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee/prompts/worker-cell.md","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json",".bee/prompts/worker-cell.md",".bee/onboarding.json","docs/history/codex-harness-hardening/release-manifest.json"],
"read_first":["docs/history/pstack-rust-checks/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee/prompts/worker-cell.md","packages/bee-rs/crates/bee/src/verbs/cells/proof.rs","docs/knowledge/areas/workflow-state/dispatch.md"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/workflow-state/dispatch.md"],
"action":"Implement CONTEXT.md D2. In prepare.rs kind-cell rendering (around :1454-1509, beside prior_round_event_lines :808), build upstream lines from the cell's deps: load each dep cell from the store; capped => '- <id> — <title>: outcome <outcome>; commit <commit>; files <comma list, first 20, then (+N more)>' plus one indented line per deviation cut to 300 characters; not capped => '- <id> — <status>'. Feed it to the template as `upstream` and add to packages/bee/prompts/worker-cell.md an `{{#if upstream}}` block titled 'Upstream cells' placed after the prior rounds block, mirroring that block's form. A cell with no deps must render byte-identically to before: pin with a test that compares against the render without the field. The block is not part of the knowledge holdout (both arms render it). Update the dispatch prepare description in generated/registry_payload.json to name the block in one sentence. Run `.bee/bin/bee dev regen` so the vendored prompt copy, the onboarding ledger and the release manifest follow. Tests (names contain upstream): capped dep renders all fields; uncapped dep renders status; truncation of files and deviations; no-deps byte-identical. No code comments. Proof: the verify command; quote the counts."},
{"id":"psrc-5","feature":"pstack-rust-checks","title":"Refill wave slots as herding runs finish","lane":"standard","role":"code","deps":["psrc-2"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml rolling && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml wave_run && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts",
"must_haves":{"truths":["dispatch wave --run --rolling --limit 2 over three independent ready cells keeps at most two runs live and starts the third when one ends","a cell whose dep is capped by a finished run is started in the same rolling call","the loop ends when no cell is ready and no run is live","native Agent-tool items are returned under native once and never refilled","--rolling without --run refuses naming --run","without --rolling the existing wave tests stay green unchanged"],"prohibitions":["bee never caps a cell here","No code comments"]},
"decisions":["806c6687-d629-419c-b643-14248562650e"],
"files":["packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json"],
"read_first":["docs/history/pstack-rust-checks/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","docs/knowledge/areas/bee-herding/waves-and-occupancy.md"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/bee-herding/waves-and-occupancy.md"],
"action":"Implement CONTEXT.md D5. Add flag --rolling (boolean) to dispatch wave (registry entry in generated/registry_payload.json, one sentence). --rolling without --run => typed refusal naming --run. With --run --rolling: run_dispatch_wave (prepare.rs:4596) and wave_run.rs run_wave_with (:19) become a loop: claim+prepare up to --limit (default: no cap) ready cells of the current wave exactly as today; start each herding item on its own thread; return native items under native once; when any run ends, record its row, recompute the schedule (compute_schedule, the same call as now), skip cells already claimed or capped, and claim+prepare+start the next ready herding cell until live < limit; end when no ready cell is left and no run is live. Use a channel from the worker threads to the loop; never busy-wait. Output keeps the {ran, native, skipped} shape with every ran row. Make the run-one-item step injectable the way run_wave_with already is so tests use a fake runner (names contain rolling): three independent cells with limit 2 => max two live at once, third starts after the first ends; a dep chain A->B where the fake run of A marks A capped => B starts; loop ends; native returned once; --rolling without --run refuses. Existing wave_run tests stay green unchanged. No code comments. Proof: the verify command; quote the counts."},
{"id":"psrc-6","feature":"pstack-rust-checks","title":"Sync knowledge and the planning pre-flight for the five checks","lane":"standard","role":"writer","deps":["psrc-1","psrc-2","psrc-3","psrc-4","psrc-5"],
"verify":".bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test skill_triggers",
"must_haves":{"truths":["docs/knowledge/areas/workflow-state/gates.md names bee plan check and its four checks","docs/knowledge/areas/workflow-state/dispatch.md names the Upstream cells block","docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md names the stored patch_id","docs/knowledge/areas/workflow-state/merge-readiness.md names dead_proof_pointers","docs/knowledge/areas/bee-herding/waves-and-occupancy.md names --rolling","skills/bee-planning/SKILL.md Gate section tells the leader to run bee plan check before presenting the gate","docs/history/pstack-rust-checks/delivery.md exists"],"prohibitions":["No code comments"]},
"decisions":["c9ae60cb-5d2b-4b6f-b9d2-3c52a14bbe18","03da9d9b-9d59-4bcc-93d4-2891538da070","77258729-68bc-4667-9170-327760c0fa64","682e3531-bba9-4042-92cf-8accca31a67f","806c6687-d629-419c-b643-14248562650e"],
"files":["docs/knowledge/areas/workflow-state/gates.md","docs/knowledge/areas/workflow-state/dispatch.md","docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md","docs/knowledge/areas/workflow-state/merge-readiness.md","docs/knowledge/areas/bee-herding/waves-and-occupancy.md","skills/bee-planning/SKILL.md","docs/history/pstack-rust-checks/delivery.md",".claude/skills/*",".agents/skills/*",".opencode/skills/*",".claude-plugin/skills/*",".codex-plugin/skills/*",".bee/onboarding.json","skills-lock.json","docs/history/codex-harness-hardening/release-manifest.json"],
"read_first":["docs/history/pstack-rust-checks/CONTEXT.md","docs/history/pstack-rust-checks/plan.md","skills/bee-planning/SKILL.md"],
"affects_skills":["skills/bee-planning/SKILL.md"],"affects_specs":["docs/knowledge/areas/workflow-state/gates.md","docs/knowledge/areas/workflow-state/dispatch.md","docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md","docs/knowledge/areas/workflow-state/merge-readiness.md","docs/knowledge/areas/bee-herding/waves-and-occupancy.md"],
"action":"Run `.bee/bin/bee knowledge consolidate --feature pstack-rust-checks`. Then sync the five knowledge concepts from the capped cells' commits (read each cell's report): one short paragraph each, citing the history D-numbers of docs/history/pstack-rust-checks/CONTEXT.md (D1..D5), never raw decision ids, with anchors to the new code. In skills/bee-planning/SKILL.md § Gate, in the 'Pre-flight the claims table yourself' paragraph, add one sentence: run `bee plan check` first; it checks sections, the cited playbook, D-number coverage and read-row evidence, and exits 10 with findings — fix them before the question. Run `.bee/bin/bee dev regen` so rendered skill trees follow. Write docs/history/pstack-rust-checks/delivery.md: what shipped per decision, the proofs each cell recorded, what was skipped. Plain global English, bee-technical-writing standard. Proof: the verify command; quote the results."}
]
```
