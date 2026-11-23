---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: parallel-session-unblock

## Summary

A worktree opens beside a live session when no companion is configured, and
the context mark refuses new work so bee hands the leader off through
waggledance.

Mode: `standard` — 0 risk flags: none (two refusal branches, one session field, one verb, doctrine text)
Why this is the least workflow that protects the work: two code cells, each with an independent tester, one docs cell; the owner set the behavior.

Playbook: `skills/bee-planning/playbooks/feature.md`, situation `skills/bee-planning/playbooks/feature/feature.md`.

## Requirements (from CONTEXT.md)

- D1 (`e37ebc32`): no dead-end refusal when the companion remedy is not configured.
- D2, D2a (`5380068f`): context mark door at clean-stop verbs; `bee state handoff send`.
- D3: doctrine text.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | worktree new refuses on a shared nested checkout with no config check | read | packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:800 | `if shared_nested_found {` |
| 2 | --with-companion needs config most projects lack | read | packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:818 | `"--with-companion requires commands.worktree_companion_start to be set in .bee/config.json.",` |
| 3 | feature start repeats the refusal | read | packages/bee-rs/crates/bee/src/verbs/feature_group.rs:297 | `match crate::nested_checkout::has_any_shared_nested_checkout(` |
| 4 | the write guard denies writes into a shared nested checkout | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:1274 | `pub(crate) fn is_shared_nested_checkout_target(` |
| 5 | the context mark is a prompt line only | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:353 | `fn compute_handoff_warning(config: &Map<String, Value>, payload: &Map<String, Value>) -> Option<String> {` |
| 6 | bee already runs the waggledance handoff | read | packages/bee-rs/crates/bee/src/verbs/handoff_door.rs:190 | `pub fn execute_handoff(` |

## Cells (current slice)

```json
[
 {
  "id": "psu-1",
  "feature": "parallel-session-unblock",
  "title": "Let a worktree open beside a live session when no companion is configured",
  "lane": "standard",
  "role": "code",
  "deps": [],
  "decisions": [
   "e37ebc32-aca4-4f0c-b902-c7a993bbb2db"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
   "packages/bee-rs/crates/bee/src/verbs/feature_group.rs",
   "packages/bee-rs/crates/bee/src/verbs/worktree/companion.rs"
  ],
  "read_first": [
   "docs/history/parallel-session-unblock/CONTEXT.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Implement CONTEXT.md D1. Add a small typed CompanionRemedy { Configured, NotConfigured } read once from the main root's config commands (worktree_companion_start and worktree_companion_mount both non-empty means Configured), next to the companion code in verbs/worktree/companion.rs. In verbs/worktree/handlers.rs at the shared_nested_found refusal and in verbs/feature_group.rs at the has_any_shared_nested_checkout Ok(true) arm: refuse exactly as today only when Configured; when NotConfigured create the worktree and print one stderr line starting 'warning: another session is live and this checkout holds a shared nested checkout;' that says the write guard keeps that checkout guarded while sessions are concurrent and that --with-companion becomes available once commands.worktree_companion_start and commands.worktree_companion_mount are set. The detection-error arm still fails closed. Put the decision in one shared helper both sites call so they cannot drift. No code comments.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree",
  "must_haves": {
   "why": "A refusal whose only remedy needs absent config pushes the owner to turn worktree_first off or kill a session.",
   "out_of_scope": [
    "doctrine text",
    "the write-guard nested-checkout deny"
   ],
   "truths": [
    "with another session live, a shared nested checkout and no companion config, bee worktree new and bee feature start create the worktree and print the warning line",
    "with companion config present the refusal and its --with-companion text are unchanged",
    "a detection error still fails closed"
   ],
   "prohibitions": [
    "No code comments",
    "No change to the write-guard deny for writes into the nested checkout"
   ]
  },
  "behavior_change": true
 },
 {
  "id": "psu-2",
  "feature": "parallel-session-unblock",
  "title": "Test the worktree guard with and without companion config",
  "lane": "standard",
  "role": "test",
  "deps": [
   "psu-1"
  ],
  "decisions": [
   "e37ebc32-aca4-4f0c-b902-c7a993bbb2db"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
   "packages/bee-rs/crates/bee/src/verbs/feature_group.rs"
  ],
  "read_first": [
   "docs/history/parallel-session-unblock/CONTEXT.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Without reading the psu-1 worker's notes, check the psu-1 commit against CONTEXT.md D1 and add behavior tests next to the existing nested-checkout and companion tests: a temp repo with a nested .git dir (not a registered submodule) and a second live session record. Cover worktree new and feature start: no companion config creates the worktree and prints the warning; both companion commands set keeps the refusal naming --with-companion; only one set counts as NotConfigured; a solo session never refuses. Fix any defect you find in psu-1's code and name it in the cap. No code comments.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree",
  "must_haves": {
   "why": "A code cell never certifies itself.",
   "out_of_scope": [
    "doctrine text"
   ],
   "truths": [
    "each D1 branch has a test that fails if the branch is removed"
   ],
   "prohibitions": [
    "No code comments"
   ]
  },
  "behavior_change": false,
  "tester_rule": true,
  "judge_obligation_ack": "Tester for psu-1."
 },
 {
  "id": "psu-3",
  "feature": "parallel-session-unblock",
  "title": "Refuse new work past the context mark and hand off through waggledance",
  "lane": "standard",
  "role": "code",
  "deps": [
   "psu-1"
  ],
  "decisions": [
   "5380068f-30cb-4724-b0f0-59e2b05e3443"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/handoff_door.rs",
   "packages/bee-rs/crates/bee/src/verbs/feature_group.rs",
   "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs",
   "packages/bee-rs/crates/bee/src/verbs/state_group/mod.rs",
   "packages/bee-rs/crates/bee/src/hooks/write_guard/mod.rs",
   "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
   "packages/bee-rs/crates/bee/src/verbs/dispatch/mod.rs",
   "packages/bee-rs/crates/bee/src/verbs/cells/mod.rs",
   "packages/bee-rs/crates/bee/src/audit.rs",
   "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
  ],
  "read_first": [
   "docs/history/parallel-session-unblock/CONTEXT.md",
   "docs/history/auto-handoff-on-due/CONTEXT.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Implement CONTEXT.md D2 and D2a. (a) The session record gains field context {tokens, measured_at}; the PreToolUse Bash hook path writes it for the payload session id only when the command runs a bee verb, measuring the payload transcript_path with devtools::statusline::context_and_turns; a write failure never blocks the tool call. (b) A pure fn context_mark_due(config, session_record) -> Option<MarkDue { tokens, mark }> using handoff_context_tokens (prompt_context DEFAULT_HANDOFF_MARK stays the single default; 0 is off). (c) Generalize handoff_door::compose_task_text and execute_handoff to take an enum HandoffReason { MergedFeature { merged }, ContextMark { tokens, mark } } and keep the feature-start merge path unchanged in behavior. (d) One shared gate fn used by dispatch prepare when --claim is passed, dispatch wave, cells claim, cells claim-next and feature start: when MarkDue, the door is Present, handoff_auto is not false and BEE_HERDING_WORKER is not 1, refuse with: [handoff_due] context <N>k is past the <M>k mark; run bee state handoff send --task \"<owner request and standing orders>\". feature start also accepts its existing --handoff-task for this case. Never gate cells cap, cells finish, worktree merge, or dispatch prepare without --claim. No door or handoff_auto false: no refusal. (e) New verb bee state handoff send --task <text>: request id bee-handoff-<session>-mark, task text names the current feature and its open or claimed cells; on Sent release the session, audit handoff-dispatched, fail with [handoff_sent] run <id>: this session is released; tell the owner in one line and stop. On Failed fail with [handoff_failed] <reason> and write nothing; with no door present refuse naming the missing door. Register the verb and its flags and regen registry_payload.json with bee dev regen. Add unit tests for context_mark_due and the reason-aware compose. Find the real dispatch and cells verb files with rg before editing; the files list names the likely homes. No code comments.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee handoff",
  "must_haves": {
   "why": "Leaders ignore the context mark line; the next clean stop must refuse new work and bee must hand off itself.",
   "out_of_scope": [
    "doctrine text",
    "the worktree guard"
   ],
   "truths": [
    "past the mark with a door, no agent flag lets dispatch prepare --claim, dispatch wave, cells claim, cells claim-next or feature start start new work in the same session",
    "cells cap, cells finish and worktree merge are never refused by the mark",
    "the merge-time handoff_due path behaves as before"
   ],
   "prohibitions": [
    "No code comments",
    "No refusal when no waggledance door exists",
    "No HANDOFF.json write on this path"
   ]
  },
  "behavior_change": true
 },
 {
  "id": "psu-4",
  "feature": "parallel-session-unblock",
  "title": "Test the context-mark door against a fake waggledance",
  "lane": "standard",
  "role": "test",
  "deps": [
   "psu-3"
  ],
  "decisions": [
   "5380068f-30cb-4724-b0f0-59e2b05e3443"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/feature_group.rs",
   "packages/bee-rs/crates/bee/src/verbs/handoff_door.rs",
   "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs"
  ],
  "read_first": [
   "docs/history/parallel-session-unblock/CONTEXT.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Without reading the psu-3 worker's notes, check the psu-3 commit against CONTEXT.md D2 and D2a and add behavior tests reusing the fake waggledance harness of the handoff_after_merge_door_with_waggledance_* tests. Cover: a session record at or past the mark refuses cells claim and feature start with [handoff_due] naming bee state handoff send; below the mark passes; mark 0 passes; no door passes; handoff_auto false passes; BEE_HERDING_WORKER=1 passes; cells cap is never refused; bee state handoff send dispatches with request id bee-handoff-<session>-mark, releases the session and writes the handoff-dispatched row; a failed dispatch writes nothing; the hook writes context tokens for a bee command and not for a non-bee command. Fix any defect in psu-3's code and name it in the cap. No code comments.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee handoff",
  "must_haves": {
   "why": "A code cell never certifies itself.",
   "out_of_scope": [
    "doctrine text"
   ],
   "truths": [
    "every D2 and D2a behavior has a test that fails if it is removed"
   ],
   "prohibitions": [
    "No code comments",
    "No real waggledance call from a test"
   ]
  },
  "behavior_change": false,
  "tester_rule": true,
  "judge_obligation_ack": "Tester for psu-3."
 },
 {
  "id": "psu-5",
  "feature": "parallel-session-unblock",
  "title": "Tell leaders the worktree guard has a way on and the mark hands off",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "e37ebc32-aca4-4f0c-b902-c7a993bbb2db",
   "5380068f-30cb-4724-b0f0-59e2b05e3443"
  ],
  "files": [
   "packages/bee/playbooks/handoff.md",
   "packages/bee/playbooks/multi-session.md",
   "docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md",
   "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md",
   "docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md",
   "docs/history/codex-harness-hardening/release-manifest.json"
  ],
  "read_first": [
   "docs/history/parallel-session-unblock/CONTEXT.md"
  ],
  "affects_skills": [
   "skills/bee-hive/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Write CONTEXT.md D3. In rule agents-context-handoff-65 (packages/bee/playbooks/handoff.md is the source; .bee/playbooks is a generated copy refreshed by bee dev regen): past the mark, the next claim, wave or feature start refuses [handoff_due]; the leader runs bee state handoff send --task \"<owner's request in their words; standing orders>\" at once, and on [handoff_sent] tells the owner in one line and stops. In packages/bee/playbooks/multi-session.md add: a worktree refusal for a shared nested checkout is never a reason to ask the owner to turn worktree_first off or to close another session; with no companion config bee opens the worktree and warns. Sync the three knowledge concepts with D1-D3, citing the history D-numbers of parallel-session-unblock, not raw decision ids. STE style, short. Run bee dev regen.",
  "verify": "/mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check && /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "why": "Doctrine must match the new behavior in every project on refresh.",
   "out_of_scope": [
    "code"
   ],
   "truths": [
    "no doctrine text tells a leader to ask the owner to turn worktree_first off or close a session for a nested-checkout refusal"
   ],
   "prohibitions": [
    "No hand edit of generated copies"
   ]
  },
  "behavior_change": false
 }
]
```

## Role assignments

```json
{
 "schema_version": "1.0",
 "runtime": "claude",
 "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
 "stages": [
  {
   "stage": "deployment",
   "classification": "not-applicable",
   "role": "deploy",
   "reason": "The release runs after merge as its own lane."
  },
  {
   "stage": "planning",
   "classification": "not-applicable",
   "role": "plan",
   "reason": "The leader wrote this plan from CONTEXT.md."
  },
  {
   "stage": "implementation",
   "classification": "required",
   "role": "code",
   "reason": "psu-1 and psu-3 write the guard and mark-door code."
  },
  {
   "stage": "test-and-live-proof",
   "classification": "required",
   "role": "test",
   "reason": "psu-2 and psu-4 test the code cells independently."
  },
  {
   "stage": "documentation-and-capture",
   "classification": "required",
   "role": "docs",
   "reason": "psu-5 updates doctrine and knowledge."
  },
  {
   "stage": "read-only-gather",
   "classification": "not-applicable",
   "role": "read",
   "reason": "The leader read the four refusal and mark sites; no gather needed."
  },
  {
   "stage": "fact-extraction",
   "classification": "not-applicable",
   "role": "extraction",
   "reason": "No narrow fact lookup is needed."
  },
  {
   "stage": "generation-fallback",
   "classification": "not-applicable",
   "role": "generation",
   "reason": "Every job here has a specific role."
  },
  {
   "stage": "independent-review",
   "classification": "not-applicable",
   "role": "review",
   "reason": "Independent review stays user-invoked and was not requested."
  },
  {
   "stage": "generic-advisor",
   "classification": "not-applicable",
   "role": "advisor",
   "reason": "No high-risk consult is owed for a publish."
  },
  {
   "stage": "supervision",
   "classification": "not-applicable",
   "role": "supervisor",
   "reason": "No unattended loop runs here."
  },
  {
   "stage": "blind-lane-1",
   "classification": "not-applicable",
   "role": "lane-1",
   "reason": "No competing designs; the owner set the behavior."
  },
  {
   "stage": "blind-lane-2",
   "classification": "not-applicable",
   "role": "lane-2",
   "reason": "No competing designs; the owner set the behavior."
  },
  {
   "stage": "blind-lane-3",
   "classification": "not-applicable",
   "role": "lane-3",
   "reason": "No competing designs; the owner set the behavior."
  },
  {
   "stage": "design-ui",
   "classification": "not-applicable",
   "role": "design",
   "reason": "No rendered UI."
  },
  {
   "stage": "hat-facts-gaps",
   "classification": "not-applicable",
   "role": "hat-facts-gaps",
   "reason": "Owner set the behavior in chat; one obvious shape, no hat wave."
  },
  {
   "stage": "hat-risks",
   "classification": "not-applicable",
   "role": "hat-risks",
   "reason": "Owner set the behavior in chat; one obvious shape, no hat wave."
  },
  {
   "stage": "hat-value",
   "classification": "not-applicable",
   "role": "hat-value",
   "reason": "Owner set the behavior in chat; one obvious shape, no hat wave."
  },
  {
   "stage": "hat-alternatives",
   "classification": "not-applicable",
   "role": "hat-alternatives",
   "reason": "Owner set the behavior in chat; one obvious shape, no hat wave."
  },
  {
   "stage": "hat-user-impact",
   "classification": "not-applicable",
   "role": "hat-user-impact",
   "reason": "Owner set the behavior in chat; one obvious shape, no hat wave."
  },
  {
   "stage": "writing",
   "classification": "not-applicable",
   "role": "writer",
   "reason": "Doctrine text rides psu-5 under the docs role."
  }
 ]
}
```

## Proof

- psu-2 and psu-4 behavior tests, green.
- `bee test` (full declared suite) once before `bee worktree merge`.
