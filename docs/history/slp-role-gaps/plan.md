---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: SLP role-interaction gaps

## Summary

Workers, advisors and reviewers tell the leader more, and more honestly.
A worker says if it agrees with its task, what it did not check, what it
could not prove, and what it found on the way; it can stop only the broken
part of a task. A worker objects once, then follows the ruling. Advisors
name how to prove them wrong. The leader judges rival designs without
knowing whose they are. Reviewers check the exact commit. A worker whose
work stopped moving is reported once. A worker that runs on a different
model than configured can be refused. Final reports say which layer
(source, build, install, live) was really checked.

Mode: `standard` — 3 risk flags: public-contracts, multi-domain, covered-contract-change
Why this is the least workflow that protects the work: the result shapes and the dissent verb are contracts other sessions and foreign agents read, so each change needs its own tests and one plan check, but no change touches auth or data loss.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1: worker results carry optional `reaction`, `skipped`, `unknowns`, `found`, on herding and native transports.
- D2: reaction `partial` with a `stopped` field: finish the safe part, name the stopped part with evidence.
- D3: after a verdict, a further dissent on the cell needs `--new-evidence`; records still append.
- D4: advisor, hat and lane answers carry recommendation, confidence 0-100, falsifier (not run).
- D5: synthesis scores proposals with lane and model names hidden, in shuffled order.
- D6: review brief withholds the leader's verdict; the reviewer checks HEAD and clean paths first.
- D7: frozen worktree fingerprint for `herding.stall_after_secs` (default 900) on a write job → `stalled` / `no_diff_progress`, one notice per stretch, observe-only.
- D8: final reports state source / artifact / installed / live separately.
- D9: `herding.paseo.on_model_mismatch` warn|refuse; envelope carries `model_mismatch`.

## Load-bearing claims

Labels are `read` (opened at that line) or `ran` (command output held); evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Herding results already parse optional fields leniently, the pattern D1/D2 copy | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:756 | `let leaning = obj.get("leaning").and_then(Value::as_str).map(str::to_string);` |
| 2 | The run envelope re-emits optional fields only when present | read | packages/bee-rs/crates/bee/src/herding/run.rs:5214 | `m.insert("leaning".into(), Value::String(leaning.clone()));` |
| 3 | The Paseo check already compares thinking, not only the model name (D9 needs no new detection) | read | packages/bee-rs/crates/bee/src/herding/run.rs:4073 | `let thinking_matches = match (cfg_thinking, info.thinking.as_deref()) {` |
| 4 | A live mismatch exists today: study workers ran with a confirmed mismatch | ran | .bee/bin/bee herding status | `status=working paseo_state=working model_mismatch=true read_only=enforced` |
| 5 | `record_dissent` is the one dissent writer, also called from the herding run path | read | packages/bee-rs/crates/bee/src/herding/run.rs:5014 | `match crate::verbs::cells::record_dissent(` |
| 6 | The native finish report refuses unknown keys, so D1 must widen its optional key list | read | packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:75 | `pub(crate) const REPORT_OPTIONAL_KEYS: [&str; 2] = ["mistakes", "baseline"];` |
| 7 | Stall today is a write-on-transition status word, which D7 reuses for its one notice | read | packages/bee-rs/crates/bee/src/herding.rs:856 | `mailbox::transition_status(bee_dir, job_id, "stalled");` |
| 8 | Herding config knobs are read by small pure readers beside the code that uses them | read | packages/bee-rs/crates/bee/src/herding.rs:680 | `pub(crate) fn ceiling_seconds(cfg: &Value) -> Option<u64> {` |
| 9 | The worker-to-leader note channel has a closed kind set | read | packages/bee-rs/crates/bee/src/herding/notify.rs:11 | `pub(crate) const KINDS: [&str; 5] = ["question", "blocked", "dependency", "progress", "reopen"];` |
| 10 | The advisor prompt asks for a digest only, no falsifier or confidence | read | packages/bee/prompts/advisor.md:16 | `Digest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.` |
| 11 | Convergence reads proposals by name today, the step D5 changes | read | skills/bee-hive/references/gates-and-delegation.md:177 | `Read every proposal whole, never as a pile of parts.` |
| 12 | The job record holds the worker's directory, so D7 can fingerprint it without new plumbing | read | packages/bee-rs/crates/bee/src/herding/run.rs:2588 | `"cwd": opts.cwd.display().to_string(),` |
| 13 | The job record says whether the job is read-only, which D7 needs to skip read-only jobs | read | packages/bee-rs/crates/bee/src/herding/run.rs:2603 | `"read_only": opts.read_only,` |
| 14 | A crate-level writer for leader-visible job messages already exists, so D7's notice needs no new channel | read | packages/bee-rs/crates/bee/src/herding/notify.rs:21 | `pub(crate) fn append_message(` |
| 15 | A Paseo stop command already exists, the one `herding interrupt` uses, so D9 refuse needs no new stop path | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:501 | `pub fn stop_argv(id: &str) -> Vec<String> {` |
| 16 | The registry payload is declared by hand in the same change as a served verb flag, not produced by regen | read | packages/bee-rs/crates/bee/src/verbs/cells/util.rs:92 | `// here and DECLARED in generated/registry_payload.json in the same` |

## Discovery

Seven read-only gathers mapped the five reference sets and checked 28
mechanisms against bee (PRESENT / PARTIAL / ABSENT with anchors); the nine
locked decisions are the ABSENT and PARTIAL ones the user asked for. Every
code anchor above was opened in this worktree at its line.

## Approach

Recommended path: extend the existing contracts in place — optional,
lenient fields on the result shapes (D1, D2), one extra guard inside the one
dissent writer (D3), one config reader plus a refusal arm at the existing
mismatch check (D9), one fingerprint beside the existing stall word (D7),
and prompt and doctrine edits at each fact's one home (D4, D5, D6, D8).

Rejected alternatives:
- A new result schema version — the lenient-optional pattern already ships two such extensions without a version bump.
- Making `refuse` the default for D9 — paseo-pi-hardening D4 chose warn on purpose, and the live mismatch would break every gather.
- Killing a stalled worker — paseo-pi-hardening D3 found long silent tool calls; D7 stays observe-only.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Native finish report keys | MEDIUM — every cap goes through it | rig-2 | finish tests: old reports still cap byte-identical; new keys stored; bad reaction refused |
| Dissent guard | MEDIUM — blocks a worker voice | rig-1 | dissent tests: first dissent, dissent after verdict refused, with `--new-evidence` accepted |
| Mismatch refuse | MEDIUM — stops a paid worker | rig-3 | run tests: absent config = warn only; refuse = typed refusal; envelope field in both |
| Fingerprint stall | LOW — observe-only | rig-5 | status tests: frozen write job → stalled/no_diff_progress once; read-only job never |
| Prompts and doctrine | LOW | rig-6, rig-7 | `bee dev regen` then `bee dev release-manifest --check` green at wave close |

Settled before the gate (plan-check findings, each answered here):

- D6 lands as a reviewer prompt step: compare HEAD with the frozen head, and check that the in-scope paths are clean. A mismatch is reported as `REVIEW_HEAD_MISMATCH` with the fix line "re-freeze the review head, or commit or drop the stray paths". No new verb.
- D7 reads `cwd` and `read_only` from the job record (rows 12, 13). The baseline lives on the job record as `diff_fingerprint` and `diff_fingerprint_since`, the reason as `stall_reason` (`activity` or `no_diff_progress`). One shared helper serves both stall sites in `herding.rs`. The stall is computed when status is read, so the notice fires on the first status read past the window, never on its own. A change of stall reason counts as a new stretch. `recovered` fires for both reasons.
- D7 notice channel: the job's messages file through `append_message` (row 14), kind `progress` (the closed kind set does not grow), text `bee: stalled — no_diff_progress for <N>s; observe only, nothing was stopped; check with bee herding status`. The leader sees it as an unread message on `bee herding status`. Named deviation from the CONTEXT wording "result inbox": the messages file is the leader-facing herding inbox that exists; a second inbox would be machinery D7 never asked for.
- D9 refuse acts only while the run loop is still polling. A mismatch first seen after the worker already finished keeps the result and warns, because refusing then throws away paid, finished work. Refuse stops the agent with the stop command `herding interrupt` uses (row 15), never archive. The refusal names configured and observed values and two fixes: point the agent's paseo block at the model that runs, or set `herding.paseo.on_model_mismatch` to `warn`. The envelope's `model_mismatch` is read from the job record. The docs say plainly that today's flash route would be refused under `refuse`.
- D1 and D2 on two transports: native `--report` matches `reaction` case-insensitively, stores it lowercase, and a refusal quotes the allowed set; native `partial` without `stopped` is refused with that fix. Herding stays lenient: a `partial` without `stopped` re-emits with `stopped_missing: true`, so the leader sees the gap. For the list fields, `[]` means "none" and an absent key means "no answer".
- D3 on the herding path: the result's dissent object may carry `new_evidence`. A refused transcription keeps today's `dissent_recorded: false` plus `dissent_error`, and the error names the answered verdict and the fix (re-record by hand with `--new-evidence`, or drop it).
- `record_dissent` keeps its signature. rig-1 adds the evidence-taking sibling it delegates to, so the dissent cell never touches `run.rs` and one writer remains.
- The registry payload is declared by hand (row 16), so rig-1 owns it; regen does not write it.

Waves: wave 1 = rig-1, rig-2, rig-5, rig-6, rig-7 in parallel (disjoint files). Wave 2 = rig-3, which calls rig-1's evidence-taking writer. The regen chain (`bee dev regen`) runs once at each wave barrier, because the skill and prompt cells all feed the same rendered skill trees.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "Rust changes in herding, cells finish and dissent"},
    {"stage": "tests", "classification": "conditional", "role": "test", "condition": "a code cell's worker asks for a test repair", "reason": "red-first tests ride each code cell"},
    {"stage": "doctrine", "classification": "required", "role": "docs", "reason": "prompts, skills references and knowledge concepts for D1-D8"},
    {"stage": "lookup", "classification": "conditional", "role": "read", "condition": "a worker needs a digest of a file outside its cell", "reason": "read-only gathers"},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "a narrow fact lookup during execution", "reason": "cheap reader"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "no free-form generation stage"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the slice judge over behavior_change cells, or the user invokes bee-reviewing", "reason": "verification of capped cells; independent review stays user-invoked"},
    {"stage": "advisor", "classification": "required", "role": "advisor", "reason": "the plan-step hat wave synthesis is recorded as the advisor ref"},
    {"stage": "plan-hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "five-seat wave is high-risk only"},
    {"stage": "plan-hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "five-seat wave is high-risk only"},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "the leader plans in-session"},
    {"stage": "supervise", "classification": "not-applicable", "role": "supervisor", "reason": "no herding cockpit"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release"},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes: precedent for every part"},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "no blind lanes"},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "no blind lanes"}
  ]
}
```

## Shape

One slice, epic map:

| Epic | Capability | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| Worker voice | D1, D2, D3 | the leader cannot see what a worker did not prove, and one false premise loses a whole cell | 1 | finish, mailbox and dissent tests |
| Advice quality | D4, D5, D6 | advice with no falsifier, anchored synthesis, reviewer seeing the leader's verdict | 1 | regen + release-manifest check |
| Worker health | D7, D9 | a stuck or wrong-model worker is silent today | 1 | herding status and run tests |
| Report honesty | D8 | a source-green change reads as live | 1 | regen + release-manifest check |

Current slice to prepare: all six cells below (the planned rig-4 merged into rig-3 after the plan check: both edited the run envelope).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| rig-1 | Refuse a repeat dissent after a verdict unless it brings new evidence | `packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs`, `packages/bee-rs/crates/bee/src/catalog.rs`, `packages/bee-rs/crates/bee/src/generated/registry_payload.json`, `docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md` | — | `bee cells dissent` on an answered cell refuses, quoting the verdict and naming `--new-evidence` | `cargo test -p bee --lib dissent` |
| rig-2 | Accept reaction, skipped, unknowns, found and stopped on a native finish report | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/trace.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs` | — | `bee cells show` carries the worker's reaction and unknowns | `cargo test -p bee --lib cells::` |
| rig-3 | Carry the new result fields, dissent evidence and the model mismatch on herding runs, with an opt-in refuse | `packages/bee-rs/crates/bee/src/herding/mailbox.rs`, `packages/bee-rs/crates/bee/src/herding/run.rs`, `packages/bee-rs/crates/bee/src/herding/paseo.rs`, `docs/knowledge/areas/bee-herding/the-paseo-channel.md`, `docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md` | rig-1 | `bee herding run --json` shows reaction, unknowns and `model_mismatch`; with refuse the run stops and names both models | `cargo test -p bee --lib herding::` |
| rig-5 | Report a write worker whose worktree stopped changing as stalled, once | `packages/bee-rs/crates/bee/src/herding.rs`, `docs/knowledge/areas/bee-herding/overview.md` | — | `bee herding status` shows `stalled` with `no_diff_progress`, and one unread note reaches the leader | `cargo test -p bee --lib herding::tests` |
| rig-6 | Teach workers the new result fields, partial stop, dissent once and layered done status | `skills/bee-swarming/references/swarming-reference.md`, `skills/bee-swarming/SKILL.md`, `packages/bee/prompts/worker-cell.md` | — | a dispatched worker's brief asks for reaction and unknowns in one short block | `bee dev release-manifest --check` |
| rig-7 | Require falsifiers from advisors, blind synthesis and an exact-commit reviewer | `packages/bee/prompts/advisor.md`, `packages/bee/prompts/reviewer.md`, `skills/bee-hive/references/gates-and-delegation.md`, `skills/bee-reviewing/SKILL.md` | — | an advisor answer ends with confidence and a falsifier | `bee dev release-manifest --check` |

```json
[
  {
    "id": "rig-1",
    "feature": "slp-role-gaps",
    "title": "Refuse a repeat dissent after a verdict unless it brings new evidence",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["b2baf5e0-f22a-44e9-ab4c-0cb232f41ddd"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs", "packages/bee-rs/crates/bee/src/catalog.rs", "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md"],
    "read_first": ["docs/history/slp-role-gaps/CONTEXT.md", "docs/history/slp-role-gaps/plan.md", "packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs", "docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dissent-and-the-verdict-duty.md"],
    "action": "Per D3 (store b2baf5e0): in dissent.rs, `pub(crate) fn record_dissent(` (the one dissent writer; also called from herding/run.rs `match crate::verbs::cells::record_dissent(` — do NOT edit run.rs) keeps its exact signature and delegates to a new sibling that also takes `new_evidence: Option<&str>`. Before the cells lock is taken (the existing refusal order comment: every refusal happens BEFORE mutate_cell), the sibling reads the cell's existing dissent entries; when any entry carries a non-empty `verdict` (DISSENT_VERDICT_KEY) and `new_evidence` is absent or blank, refuse with a message that quotes the newest answered verdict and its reason and names the fix `--new-evidence \"<what is new>\"`; the cell file stays byte-identical. With evidence, scan it with the same `assert_safe_decision_fields` call, and store it on the new appended entry as `new_evidence`. Entries still append, never overwrite. Wire a `--new-evidence` flag on `bee cells dissent` (the run_dissent handler in dissent.rs), add the flag to the catalog.rs flag table where `--alternative` is registered, and declare it in generated/registry_payload.json for the `cells dissent` verb by hand (the same both-directions rule recorded in verbs/cells/util.rs beside `\"dissent\" => run_dissent`). Add the dissent-once rule, the refusal and the flag to the dissent knowledge concept. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee dissent",
    "must_haves": {
      "truths": ["A first dissent on a cell records as before", "A dissent on a cell whose dissent already has a verdict is refused unless --new-evidence is given, and the cell file is byte-identical after the refusal", "A dissent with --new-evidence after a verdict appends an entry carrying new_evidence"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs", "substantive": "evidence-taking sibling writer, the after-verdict refusal, the --new-evidence flag, tests for all three truths"}],
      "key_links": ["record_dissent delegates to the sibling with no evidence", "bee cells dissent passes --new-evidence to the sibling", "registry_payload.json declares --new-evidence on cells dissent"],
      "prohibitions": ["No change to record_dissent's signature", "No edit to herding/run.rs", "No overwrite of an earlier dissent entry", "No code comments"]
    },
    "trace": {"behavior_change": true}
  },
  {
    "id": "rig-2",
    "feature": "slp-role-gaps",
    "title": "Accept reaction, skipped, unknowns, found and stopped on a native finish report",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["15f6804c-1faf-4273-875b-e3b2dfaf0d38", "6e81d8db-8e72-48ef-aa69-c465336421d3"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "packages/bee-rs/crates/bee/src/verbs/cells/trace.rs", "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"],
    "read_first": ["docs/history/slp-role-gaps/CONTEXT.md", "docs/history/slp-role-gaps/plan.md", "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 (store 15f6804c) and D2 (store 6e81d8db): widen `pub(crate) const REPORT_OPTIONAL_KEYS: [&str; 2] = [\"mistakes\", \"baseline\"];` in finish_support.rs with `reaction`, `skipped`, `unknowns`, `found`, `stopped`. Validate them where `baseline` is validated (`match map.get(\"baseline\")`): `reaction` is a string matched case-insensitively against confirm|partial|challenge|blocked and stored lowercase, else refuse quoting the allowed set; `skipped`, `unknowns`, `found` are arrays of non-empty strings (`[]` is legal and means none); `stopped` is a non-empty string; `reaction` partial without `stopped` is refused with the fix `add \"stopped\": \"<the part not done> — <evidence>\"`. In handlers_close.rs, store each present key on the cell trace beside the `baseline` insert (`if let Some(Value::String(b)) = report_value.get(\"baseline\")`); an absent key writes nothing, so an old report caps byte-identical. In trace.rs, add the five keys to the list reset on release beside `\"baseline\"`. Tests in cells/tests.rs: new keys stored; old report unchanged; bad reaction refused; partial without stopped refused; mixed-case reaction stored lowercase. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee cells::",
    "must_haves": {
      "truths": ["A finish report with reaction and the list fields caps and the trace holds them", "A report without the new keys caps exactly as before", "reaction outside the closed set, and partial without stopped, are refused by name"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs", "substantive": "five optional keys validated"}],
      "key_links": ["handlers_close.rs stores the validated keys on the trace", "trace.rs resets them on release"],
      "prohibitions": ["No new required report key", "No change to the proof-line tag set", "No code comments"]
    },
    "trace": {"behavior_change": true}
  },
  {
    "id": "rig-3",
    "feature": "slp-role-gaps",
    "title": "Carry the new result fields, dissent evidence and the model mismatch on herding runs, with an opt-in refuse",
    "lane": "standard",
    "role": "code",
    "deps": ["rig-1"],
    "decisions": ["15f6804c-1faf-4273-875b-e3b2dfaf0d38", "6e81d8db-8e72-48ef-aa69-c465336421d3", "b2baf5e0-f22a-44e9-ab4c-0cb232f41ddd", "0ccaa387-07db-4737-b53c-6491f3fb3fc9"],
    "files": ["packages/bee-rs/crates/bee/src/herding/mailbox.rs", "packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee-rs/crates/bee/src/herding/paseo.rs", "docs/knowledge/areas/bee-herding/the-paseo-channel.md", "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md"],
    "read_first": ["docs/history/slp-role-gaps/CONTEXT.md", "docs/history/slp-role-gaps/plan.md", "packages/bee-rs/crates/bee/src/herding/mailbox.rs", "docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md", "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md"],
    "action": "Three parts, one envelope. (1) Per D1 and D2: add optional `reaction`, `skipped`, `unknowns`, `found`, `stopped` to `pub(crate) struct MailboxResult` in mailbox.rs, parsed leniently exactly like `let leaning = obj.get(\"leaning\")...` (absent or wrong-typed reads as absent, never Malformed; reaction lowercased and kept only when in confirm|partial|challenge|blocked). Name them in the result schema block of `pub(crate) fn render_brief(` as optional, in ONE short block, without naming any bee command (the existing brief pins forbid `bee ` in the brief). In run.rs `fn result_envelope(` re-emit each only when present, like `m.insert(\"leaning\".into(), ...)`; when reaction is partial and stopped is absent, emit `stopped_missing: true`. (2) Per D3: add optional `new_evidence` to `MailboxDissent` and pass it from the transcription site `match crate::verbs::cells::record_dissent(` to rig-1's evidence-taking sibling; a refused transcription keeps `dissent_recorded: false` plus `dissent_error`. (3) Per D9 (store 0ccaa387) and the plan's settled list: add a config reader in paseo.rs for `herding.paseo.on_model_mismatch` (`warn` default when absent or unknown, `refuse`) in the style of `fn configured_command(`. Make `fn record_observed_model(` return whether a confirmed mismatch was seen. At the two in-loop call sites (inside the poll closures) with `refuse`, stop the agent with `paseo::stop_argv` through the same cli call `herding interrupt` uses in job_verbs.rs, and end the run with a typed outcome reason `model_mismatch` whose message names configured and observed model and thinking and two fixes (point the agent's paseo block at the model that runs, or set herding.paseo.on_model_mismatch to warn); exit non-zero. At the post-loop call site never refuse: keep the result and warn. In both modes the envelope carries `model_mismatch: {configured, observed}` read from the job record's paseo_model_observed / paseo_thinking_observed when paseo_model_mismatch is true. Update the two knowledge concepts: the new result fields and envelope keys in the run-verb concept, the refuse mode in the paseo-channel concept (state that today's flash route reports thinking off and would be refused). No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding::",
    "must_haves": {
      "truths": ["A herding result with the new fields round-trips into the run envelope", "Wrong-typed new fields never make a result Malformed", "With on_model_mismatch absent a mismatch only warns and the envelope carries model_mismatch", "With refuse a mismatch seen while polling stops the agent and ends the run with reason model_mismatch", "A herding dissent with new_evidence after a verdict is recorded with that evidence"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "envelope fields, refuse arm at in-loop sites, tests"}, {"path": "packages/bee-rs/crates/bee/src/herding/paseo.rs", "substantive": "on_model_mismatch reader"}],
      "key_links": ["render_brief names the optional fields", "parse_result_text fills them", "result_envelope emits them", "transcription passes new_evidence to the rig-1 sibling"],
      "prohibitions": ["No bee command named in the worker brief", "No refuse after the worker finished", "No archive on refuse", "No change when on_model_mismatch is absent beyond the new envelope field", "No code comments"]
    },
    "trace": {"behavior_change": true}
  },
  {
    "id": "rig-5",
    "feature": "slp-role-gaps",
    "title": "Report a write worker whose worktree stopped changing as stalled, once",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["8a826f2f-ae7a-4923-a2b0-a65d06445153"],
    "files": ["packages/bee-rs/crates/bee/src/herding.rs", "docs/knowledge/areas/bee-herding/overview.md"],
    "read_first": ["docs/history/slp-role-gaps/CONTEXT.md", "docs/history/slp-role-gaps/plan.md", "packages/bee-rs/crates/bee/src/herding.rs", "packages/bee-rs/crates/bee/src/herding/notify.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/overview.md"],
    "action": "Per D7 (store 8a826f2f) and the plan's settled list. Add a reader `stall_after_secs` for `herding.stall_after_secs` (default 900) in the style of `pub(crate) fn ceiling_seconds(cfg: &Value) -> Option<u64> {`. Stall is derived in two places in herding.rs, both at `mailbox::transition_status(bee_dir, ..., \"stalled\")` (in compute_job_status and in the status-list loop); add ONE shared helper used by both. For a job whose job record `read_only` is not true and whose status would otherwise be working: compute a fingerprint in the job record's `cwd` = `git rev-parse HEAD` plus a hash of `git diff HEAD` stdout; store `diff_fingerprint` and `diff_fingerprint_since` on the job record when the fingerprint changes; when it is unchanged for longer than stall_after_secs, the status is `stalled` and `stall_reason` is `no_diff_progress` (the existing activity stall writes `stall_reason: activity`). A change of reason is a new stretch. On the transition into a no_diff_progress stretch, append ONE message to the job's messages file with `crate::herding::notify::append_message` (kind `progress`, text `bee: stalled — no_diff_progress for <N>s; observe only, nothing was stopped; check with bee herding status`). Nothing is stopped or retried. `recovered` fires when the fingerprint moves. A git failure in cwd leaves status unchanged. Document the signal, the knob and the read-time nature (computed when status is read) in the herding overview concept. Tests: frozen past window stalls with the reason; moving fingerprint never stalls; read-only job never stalls by fingerprint; second read appends no second message; recovery. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding::tests",
    "must_haves": {
      "truths": ["A write job with a frozen worktree past the window reads stalled with stall_reason no_diff_progress", "A read-only job never stalls by fingerprint", "Exactly one message is appended per stalled stretch"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding.rs", "substantive": "shared stall helper, fingerprint, config reader, tests"}],
      "key_links": ["both stall sites call the shared helper", "the notice goes through append_message"],
      "prohibitions": ["Never stop, retry or archive a worker", "No new notify kind", "No edit to notify.rs or run.rs", "No code comments"]
    },
    "trace": {"behavior_change": true}
  },
  {
    "id": "rig-6",
    "feature": "slp-role-gaps",
    "title": "Teach workers the new result fields, partial stop, dissent once and layered done status",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["15f6804c-1faf-4273-875b-e3b2dfaf0d38", "6e81d8db-8e72-48ef-aa69-c465336421d3", "b2baf5e0-f22a-44e9-ab4c-0cb232f41ddd", "e11f3e7a-dda7-4bb5-823d-1eb8a3ba9df1"],
    "files": ["skills/bee-swarming/references/swarming-reference.md", "skills/bee-swarming/SKILL.md", "packages/bee/prompts/worker-cell.md"],
    "read_first": ["docs/history/slp-role-gaps/CONTEXT.md", "docs/history/slp-role-gaps/plan.md", "skills/bee-swarming/references/swarming-reference.md", "packages/bee/prompts/worker-cell.md"],
    "affects_skills": ["skills/bee-swarming/references/swarming-reference.md", "skills/bee-swarming/SKILL.md"],
    "affects_specs": [],
    "action": "Doctrine, one home each. In swarming-reference.md section `## Result Formats (expected back from workers)` (the one home of the Result form): keep the minimal JSON example as is and add a second, optional example with `reaction`, `skipped`, `unknowns`, `found`, `stopped`; one bullet each (D1: reaction is the stance toward the brief, closed set confirm|partial|challenge|blocked; `[]` means none, an absent key means no answer). Add D2: a worker that finds a false premise in part of its cell finishes the safe part, returns [DONE] with reaction partial and `stopped` naming the part and its evidence; [BLOCKED] stays for a cell with no safe part. Add D3: dissent once per point; after the verdict, follow it; a second dissent needs new evidence. Add D8 beside the `tests` proof-tag bullet: a final report for work that ships something runnable states source / artifact / installed / live, each with its verdict or `not checked`; omit the four layers when nothing runnable ships; give one example line. In SKILL.md add the leader-side line: a partial result's `stopped` part becomes a new cell or a dissent verdict before close, and the leader's own final report follows D8 — as a pointer to the Result Formats section, never a copy. In worker-cell.md add ONE short block after the Result form paragraph naming the optional keys, partial stop and dissent-once, pointing at nothing outside the prompt. Keep the prompt short. Do not run bee dev regen: the leader runs it once at the wave barrier.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test instruction_laws",
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": ["The Result form names the five optional keys with their meaning", "Partial stop, dissent once and the four-layer report each have exactly one home"],
      "artifacts": [{"path": "skills/bee-swarming/references/swarming-reference.md", "substantive": "optional result example, D2, D3, D8 rules"}],
      "key_links": ["SKILL.md points at the Result Formats section", "worker-cell.md names the optional keys"],
      "prohibitions": ["No second copy of the Result form", "No size budget language", "No change to the proof-line tag set"]
    },
    "trace": {"behavior_change": true}
  },
  {
    "id": "rig-7",
    "feature": "slp-role-gaps",
    "title": "Require falsifiers from advisors, blind synthesis and an exact-commit reviewer",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["2fb9b028-3ef9-4e91-bbba-b20e30155cae", "121947c8-f163-41e0-a151-a9bdfaeb7186", "1fa910ba-ec15-4382-bf91-6fe5a9724027"],
    "files": ["packages/bee/prompts/advisor.md", "packages/bee/prompts/reviewer.md", "skills/bee-hive/references/gates-and-delegation.md", "skills/bee-reviewing/SKILL.md"],
    "read_first": ["docs/history/slp-role-gaps/CONTEXT.md", "docs/history/slp-role-gaps/plan.md", "skills/bee-hive/references/gates-and-delegation.md", "packages/bee/prompts/advisor.md"],
    "affects_skills": ["skills/bee-hive/references/gates-and-delegation.md", "skills/bee-reviewing/SKILL.md"],
    "affects_specs": [],
    "action": "Per D4 (store 2fb9b028): after `Digest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.` in advisor.md, add that the answer ends with a recommendation, a confidence 0-100 (anchors: 90+ = checked in the repo, 50 = reasoned but unchecked, below 30 = a guess), and a falsifier: one concrete check that would prove the recommendation wrong, marked not run. In gates-and-delegation.md say the same once for hat seats (Hat wave, The moves) and blind lanes (move 1 Fan out), by pointer to the advisor prompt rule. Per D5 (store 121947c8): in Blind lanes Converge, Base step (`Read every proposal whole, never as a pile of parts.`), and in the hat-wave synthesis line, add: before scoring, relabel the proposals A, B, C in a shuffled order with lane and model names hidden; reveal names only after every proposal is scored; with one proposal, score it alone. Per D6 (store 1fa910ba): in reviewer.md and in bee-reviewing SKILL.md where the reviewer inputs are named (`gets the cumulative diff`), add: the review brief never carries the orchestrator's own verdict, leaning or expected findings; the reviewer first checks that HEAD equals the frozen review head and that the in-scope paths are clean, and on a mismatch stops and reports `REVIEW_HEAD_MISMATCH` with the fix line `re-freeze the review head, or commit or drop the stray paths`. Do not run bee dev regen: the leader runs it once at the wave barrier.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test instruction_laws",
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": ["The advisor prompt asks for recommendation, confidence and a falsifier", "Convergence hides lane and model names until scoring is done", "The reviewer checks the exact head and clean paths first"],
      "artifacts": [{"path": "packages/bee/prompts/advisor.md", "substantive": "falsifier and confidence rule"}, {"path": "skills/bee-hive/references/gates-and-delegation.md", "substantive": "anonymized shuffled scoring, falsifier pointer"}],
      "key_links": ["gates-and-delegation points at the advisor prompt rule instead of copying it", "bee-reviewing SKILL.md and reviewer.md say the same head check"],
      "prohibitions": ["No neutrality-lint breakage in lane briefs", "No new seat role", "No size budget language"]
    },
    "trace": {"behavior_change": true}
  }
]
```

## Test matrix

| Cell | Happy | Edge | Error |
|---|---|---|---|
| rig-1 | first dissent on a cell records. Pass when the cell trace holds one dissent entry | a dissent after a verdict with `--new-evidence` records and stores the evidence. Pass when the second entry carries `new_evidence` | a dissent after a verdict without it is refused and the cell file is byte-identical. Pass when the refusal names `--new-evidence` |
| rig-2 | a report with `reaction: "confirm"` and lists caps and stores them. Pass when the trace holds the fields | an old report with none of the keys caps byte-identical to main. Pass when the trace has no new keys | `reaction: "maybe"` or `partial` without `stopped` is refused by name. Pass when the refusal names the key |
| rig-3 | the new fields round-trip into the envelope; with config absent a mismatch warns, the run continues and the envelope carries `model_mismatch`. Pass when the outcome is unchanged and the fields are present | wrong-typed fields read as absent, never `Malformed`; `partial` without `stopped` re-emits `stopped_missing: true`; sentinel `-`/`auto` never refuses; a mismatch first seen after the worker finished keeps the result | `refuse` while polling: the stop command is called and the run ends with reason `model_mismatch` naming both values, exit non-zero; a dissent after a verdict with no `new_evidence` gives `dissent_recorded: false` naming the verdict; with `new_evidence` the entry records it |
| rig-5 | a frozen fingerprint past the window on a write job gives `stalled` with `stall_reason: no_diff_progress`. Pass when status reads so | a changing fingerprint never stalls; a read-only job never stalls by fingerprint; recovery prints `recovered` | the notice is appended once per stretch. Pass when a second status read appends no second message |
| rig-6, rig-7 | regen green | pointer check: each fact has one home | release-manifest check green |

## Open Questions

(none)

## Out of scope

- Per-role stall windows — backlog.
- Byte fingerprint of the installed binary — backlog.
- A bee-side frozen-HEAD verb for reviewers — D6 lands as a reviewer prompt step.
