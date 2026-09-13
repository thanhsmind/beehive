# Codex workflow compliance review

Review: `codex-workflow-20260913`. Requested by the user on 2026-09-13.

## Scope and verdict

The main workflow ran, but the records do not support full compliance. Seven findings remain. No demonstrated production failure, credential disclosure, or P1 blocker was established.

Frozen scope: `642631a59d84639922a70848c52f723d9c9e4f53` through `0d35307804be0fd2d34adb5be379816c68802478`, feature `codex-parity-completion`, cells cpc-1 through cpc-4. This review checks workflow application. It does not certify every Codex capability or audit all repository code.

The leader also checked the current authoritative workflow record to test claims about missing approval. That record is supporting context outside the frozen Git range. External evidence paths are identified as such; their bytes are not preserved by the reviewed commit.

## Verified steps

| Step | Evidence | Result |
|---|---|---|
| Scope and initial gates | CONTEXT.md; decisions ee16df03 and e0dcdfd9 | Scope and bypass approval recorded before first claim |
| Plan consult | advisor-synthesis.md; lane advisor_ref; wave ledger | Five perspectives recorded; model diversity explicitly unverified |
| Worktree and execution workers | Wave ledger; four cell traces; worktree merge commits | Worktree execution and delegated workers recorded |
| Leader acceptance | Revision reports and blocked attempts | Incomplete worker results were rejected and repaired |
| Final commit shape | e21176cd, bc490660, f6a6157a, 5dd8e83b | One final source commit per cell, with cell trailers |
| Final proof | Four judge records; final-green-suite.log; main-final-suite.log | Specific checks and successful suite outputs retained |
| Live proof | Canary and sandbox paths in verification-summary.md | Installed writes and restricted CLI execution proved; unsupported paths disclosed |
| Knowledge sync | Lane last_scribing_run, 2026-09-13T04:29:50Z | hook-runtime sync recorded |
| Merge and acceptance | 4c49458a, fc104c5e; decision 70d8bdab; usage closed_at | Merge preceded user testing; user acceptance preceded closure |
| Compound | Lane summary and deferred queue | Five capture stubs remain; this deferral is permitted |

Fresh leader check: `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` exited 0 during this review. This is unit/integration proof, not a new authenticated live Codex run.

Fresh range check: `git diff --check 642631a59d84639922a70848c52f723d9c9e4f53 0d353078 -- packages/bee-rs skills docs/06-runtime-integration.md README.md` exited 2 and named `session_close/tests.rs:1916`.

## Spec findings

### [P2] F1: Live probes crossed the real-home configuration boundary

Axis: spec. Autofix class: manual.

**Summary and current behavior:** The plan prohibited real-home changes. A retained decision records project-trust and mise configuration side effects.

**Evidence:** `CONTEXT.md:11`; `plan.md:78`; `.bee/decisions.jsonl:2862` (4725ef61); `.bee/cells/archive/codex-parity-completion/cpc-1.json:136`: “Codex PATH wrapper modified mise config”.

**Failure scenario:** A probe resolves `codex` through the user's PATH wrapper or uses the real Codex home. It changes settings outside the disposable test repository. A later isolated run does not undo that earlier side effect.

**Proposed fix:** Determine the exact affected entries from existing non-secret evidence. Restore only entries proven to belong to the probe, with appropriate authorization. Use the direct executable and a private test home for future probes.

**Acceptance:** Record the affected paths, the limited restoration result, and a test that rejects unintended real-home writes. Do not infer restoration or credential exposure from the current record. The extent of any remaining side effect is unverified.

## Standards findings

### [P2] F2: cpc-3 lacks retained red-before-green proof

Axis: standards. Autofix class: gated_auto.

**Summary and current behavior:** The transcript repair has successful tests, but its cap does not link a failed pre-fix run.

**Evidence:** `cpc-3-revision-2.md:3` requires RED tests before repair. `.bee/cells/archive/codex-parity-completion/cpc-3.json:86` and `:112` record green results. A search of the retained feature logs found no failed session_close or activity regression run.

**Failure scenario:** A later reviewer can see that the final tests pass, but cannot verify that those tests detected the original normalization and Claude transcript defects.

**Proposed fix:** Recover the original failing output if available. Otherwise run the relevant tests against a saved pre-fix tree and the accepted tree. Label that result as retrospective regression validation.

**Acceptance:** Preserve both outputs, exact test names, and commit identities. Do not claim that a new run proves the original red-before-green sequence occurred.

### [P2] F3: Temporary sign-in cleanup remains unresolved

Axis: standards. Autofix class: manual.

**Summary and current behavior:** The feature closed with a temporary sign-in copy still present. This was disclosed, but the cleanup lifecycle remains incomplete.

**Evidence:** `verification-summary.md:42-46`; decision 481db8ec at `.bee/decisions.jsonl:2874`; lane summary at `.bee/lanes/codex-parity-completion.json:13`. The reviewer checked file metadata only and confirmed mode 0600.

**Failure scenario:** The temporary test credential remains usable on disk after the tests finish. No unauthorized access or disclosure was established.

**Proposed fix:** Assign an owner and use a permitted cleanup path. Preserve transcripts and non-secret evidence.

**Acceptance:** Confirm that the exact temporary copy is absent without reading its contents. Do not bypass the physical-worktree guard.

### [P3] F4: Whitespace proof did not establish the committed range

Axis: standards. Autofix class: gated_auto.

**Summary and current behavior:** The cap names plain `git diff --check`. The committed source range still contains a whitespace error.

**Evidence:** `.bee/cells/archive/codex-parity-completion/cpc-4.json:92`; `verification-summary.md:22`; the fresh explicit range check reports a new blank line at `packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs:1916`.

**Failure scenario:** A check after commit can examine an empty working diff and miss committed defects. This is a formatting/proof-scope defect, not a failed live sandbox test.

**Proposed fix:** Check the explicit cell or feature range. Preserve raw logs as evidence and state any log exclusions. Correct the source whitespace in a separate change.

**Acceptance:** An explicit source/authored-document range check exits 0, with its scope recorded.

### [P3] F5: One judge pointer names the wrong summary line

Axis: standards. Autofix class: gated_auto.

**Summary and current behavior:** The cpc-1 judge points to summary line 22 for deny/allow proof. That line describes the whitespace check.

**Evidence:** `reports/cpc-1-judge.json:8`; `verification-summary.md:22` and `:26`. The same judge also points to the actual canary report, which supported the verdict during review.

**Failure scenario:** A reader following the summary anchor reaches unrelated evidence.

**Proposed fix:** Append a correction through the supported record path. Cite the actual canary and matching summary location; preserve the historical judgment.

**Acceptance:** Every corrected anchor leads directly to the claimed output.

### [P3] F6: An evidence file was written before reservation

Axis: standards. Autofix class: gated_auto.

**Summary and current behavior:** cpc-2 explicitly records an evidence log created before its path was reserved.

**Evidence:** `.bee/cells/archive/codex-parity-completion/cpc-2.json:144-147`: “Log captured before reservation”. No actual overwrite or reservation conflict was established.

**Failure scenario:** A concurrent worker can own the intended output path before the writer acquires its reservation.

**Proposed fix:** Reserve evidence output paths before starting commands that write them.

**Acceptance:** A conflicting reservation refuses the output step before it creates or truncates a file.

### [P3] F7: Refreshed onboarding metadata retains an obsolete runtime claim

Axis: standards. Autofix class: gated_auto.

**Summary and current behavior:** The refreshed onboarding record still describes prompt-only Codex role enforcement and all-null model defaults.

**Evidence:** `.bee/onboarding.json:58-60`. Its hook hash and update timestamp changed in the reviewed range, but the old note remained. `docs/06-runtime-integration.md` and the delivered dispatch path describe configured transports and restricted CLI execution.

**Failure scenario:** A later agent reads the refreshed onboarding record and applies the old runtime contract.

**Proposed fix:** Correct the owning renderer and regenerate the metadata. Do not edit CLI-owned JSON by hand.

**Acceptance:** Regenerated metadata states supported behavior and capability limits without claiming effective-model proof.

## Reviewer coverage and calibration

Four configured review-tier CLI sessions ran with requested model `gpt-5.6-sol` and a read-only sandbox. All completed. The effective backend model was not independently proved. Same-source repeats are duplicates, not independent evidence of greater impact.

| Lens | Dispatch | Lead disposition |
|---|---|---|
| code-quality | 9b1ba8f3-35ca-4cd7-a581-89180e6a3ba9 | Cleanup retained; missing-plan-approval claim checked against authoritative state |
| architecture | f16de9db-b773-4f5b-8284-e15719160ebe | Reservation issue retained; plan and recovery claims qualified below |
| security | ba7a99c9-7a95-4f2a-a5de-1b0fbafd9766 | Global side effects, cleanup, metadata and pointer retained; P1 labels rejected |
| test-coverage | 961c2337-f6e8-4428-8a3b-361e90e0fc98 | RED gap, range-check gap and pointer retained |

The security reviewer called whitespace a P1 and described the recorded real-home changes as P1. The first compared different commands. The second did not establish a current authorization exposure, data loss, or credential disclosure. F1 is P2; F4 is P3. No finding was hidden or repaired during this review.

## Dismissed and qualified

- Missing approval for the final plan hash — dismissed: `.bee/runtime/workflows/wf-07e08dde/state.json`, read by the leader, records plan_rev 1 and shape/execution approvals at 2026-09-13T04:36:33.458Z under full bypass. The post-merge plan diff adds only a documented annotation around the completed regeneration paragraph. The frozen Git projection alone omitted that gate detail. No source change was authorized retroactively by this annotation.
- Stale cpc-2 plan dependency — qualified: decision a95e7d19 explicitly changes execution order while retaining the live-proof integration requirement. The original plan remains the historical proposal; the decision is the recorded amendment.
- Leader recovery as an unrecorded source takeover — dismissed in that form: decision 32b3ac45 records a named method deviation within the already-approved scope. It is a departure from the normal worker path, not evidence of a new self-approved product scope. Decision cfab5be3 explicitly avoids retrospective user attribution.
- Source before initial gate, judge after merge, or acceptance after closure — dismissed: recorded timestamps show the required order.
- Zero native subagent usage means no workers — dismissed: the wave ledger records external execution workers and feature worktree paths.
- Multiple final commits per cell or broken report hashes — dismissed: consolidation produced four final cell commits, and the backup branch retains original report commits. Scoped product comparisons matched.
- Missing Compound blocks close — dismissed: bee-capturing explicitly permits queued Compound; five stubs remain visible.
- Missing Windows/PreCompact/privacy/reservation live tests proves false coverage — dismissed: those limits are disclosed; this review does not certify full runtime support.
- Successful full suite proves original RED sequence — rejected: final correctness proof does not establish historical test order.

## Limits and next action

This is a report-only workflow audit of already-merged work. It grants no merge approval and changes no feature gate. The normal merge-approval question does not apply to this audit; any repair needs its own approved scope. The existing user acceptance is retained, not repeated.

Raw live evidence remains partly in temporary directories. Missing archived dispatch payloads and reservation history limit reconstruction. The user conversation is not in the frozen commit; the leader has the current acceptance exchange as additional context.

Prioritize F1: establish and resolve the exact real-home side effects. Track F2-F7 separately; do not rewrite history to make the original run appear fully compliant.

Applied principles: verify-before-reporting rejected the false missing-approval and P1 whitespace claims; red-before-green distinguished a passing suite from missing historical proof; smallest-honest-shape kept this a workflow audit without source repairs.

Captured: verified workflow findings and dismissed claims are recorded in this report and review `codex-workflow-20260913`.

7 finding(s) — P1 0, P2 3, P3 4 · axis: spec 1, standards 6.
