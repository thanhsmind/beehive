---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: Lean context

## Summary

Two cells, in parallel, with no shared file. lnc-1 makes the
prompt-context hook warn when the session context passes
`handoff_context_tokens` (D1). lnc-2 moves the doctrine handoff mark to
about 150k tokens or 65%, whichever comes first, adds the feature-close
handoff point, and runs the claude docs role on sonnet (D2).

Mode: `small`. Playbook: perf.
Deviation: no hat wave. Both cells answer a measured cost (CONTEXT.md,
Evidence) and each is small. The rule id `agents-context-handoff-65`
stays: renaming it would sweep every applied_at file for no behavior
gain. `packages/bee/prompts/worker-cell.md` keeps its 65% line: workers
are short-lived and a drivers test pins that line.
Deviation: `.bee/config.json` has no CLI verb for team roles, so lc-2
edits that one key by hand.

## Requirements (from CONTEXT.md)

- D1 (`9bf28c33`): hook warning line at `handoff_context_tokens`
  (default 150000, 0 off); doctrine mark 150k or 65%, plus feature close.
- D2 (`88109b52`): `team.claude.docs` on sonnet.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | a reader of the latest request context exists | ran | packages/bee-rs/crates/bee/src/devtools/statusline.rs:302 | `pub(crate) fn context_and_turns(transcript: &str) -> Option<(u64, u64)> {` |
| 2 | the hook assembles injected parts | ran | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:379 | `parts.push(plan.reminder_text.clone());` |
| 3 | the doctrine line | ran | packages/bee/AGENTS.block.md:290 | `- At roughly 65% context, write` |

## Cells (current slice)

```json
[
  {
    "id": "lnc-1",
    "feature": "lean-context",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["9bf28c33-73c8-4f61-9f04-c199dc58e748"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"],
    "title": "Warn in the prompt-context hook when the session context passes the handoff mark",
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee-rs/crates/bee/src/devtools/statusline.rs",
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"
    ],
    "read_first": [
      "docs/history/lean-context/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"
    ],
    "action": "On UserPromptSubmit, the prompt-context hook reads the payload's transcript_path and takes the latest request context (input + cache read + cache creation tokens, as statusline context_and_turns computes it). When that value is at or above handoff_context_tokens from .bee/config.json (integer; absent means 150000; 0 turns the check off), the injected context carries exactly one extra line: `context <N>k tokens is past the handoff mark (<T>k): finish this step, write a planned-next handoff (bee state handoff write --kind planned-next), and continue in a fresh session.` Below the mark, with no transcript_path, or with an unreadable or usage-free transcript, nothing is added and the hook never fails or slows the turn. Reuse context_and_turns rather than a second parser. Record the key and the line in the named knowledge concept. No code comments. One commit with the trailer `cell: lnc-1`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prompt_context statusline",
    "must_haves": {
      "truths": [
        "a transcript whose latest usage sums to 160000 makes the hook output carry the handoff line naming 160k and 150k",
        "a transcript at 140000 adds no line",
        "handoff_context_tokens 0 adds no line at any size",
        "a missing or unreadable transcript adds no line and the hook exits 0"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs", "substantive": "handoff mark warning line"}
      ],
      "key_links": ["a test fails before the change and passes after"],
      "prohibitions": ["Do not weaken an existing assertion", "No code comments", "Never fail the hook on a transcript read error"],
      "open_choices": ["where in the injected parts the line sits"]
    },
    "behavior_change": true,
    "judge_obligation_ack": "the hook only adds an advisory context line; no allow or deny path changes"
  },
  {
    "id": "lnc-2",
    "feature": "lean-context",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": ["9bf28c33-73c8-4f61-9f04-c199dc58e748", "88109b52-87d4-4ddb-8cb7-d4875024db75"],
    "affects_skills": [
      "skills/bee-hive/SKILL.md",
      "skills/bee-hive/references/go-mode.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"],
    "title": "Move the handoff mark to 150k tokens or 65% and run the docs role on sonnet",
    "files": [
      "packages/bee/AGENTS.block.md",
      "AGENTS.md",
      "skills/bee-hive/SKILL.md",
      "skills/bee-hive/references/go-mode.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      ".bee/config.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": ["docs/history/lean-context/CONTEXT.md"],
    "action": "Every doctrine site that names the 65%-context handoff (rule: agents-context-handoff-65) says the mark is about 150k tokens or 65% of the window, whichever comes first, and that every feature close is also a handoff point; the rule id is unchanged. The AGENTS.block.md line under Care for the session carries this, and AGENTS.md is regenerated from it with bee dev regen inside this cell, never hand-edited. The rule index spoken line says the same. In .bee/config.json, team.claude.docs.model becomes sonnet; no other role changes. packages/bee/prompts/worker-cell.md is out of scope. One commit with the trailer `cell: lnc-2`.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- agents_md doctrine parity && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "rg -n '65%' over the five skills and AGENTS.block.md shows every hit paired with 150k",
        "AGENTS.md matches bee dev regen output",
        "team.claude.docs.model is sonnet and every other role is unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee/AGENTS.block.md", "substantive": "150k or 65% handoff mark plus feature close"}
      ],
      "key_links": ["the doctrine parity tests stay green"],
      "prohibitions": ["Do not rename the rule id", "Do not hand-edit AGENTS.md", "Do not touch worker-cell.md"],
      "open_choices": []
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
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {
      "stage": "implementation",
      "classification": "required",
      "role": "code",
      "reason": "Cell lnc-1 changes the prompt-context hook."
    },
    {
      "stage": "deployment",
      "classification": "not-applicable",
      "role": "deploy",
      "reason": "No release rides this lane."
    },
    {
      "stage": "planning",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "The plan is this page."
    },
    {
      "stage": "test-and-live-proof",
      "classification": "not-applicable",
      "role": "test",
      "reason": "Cell lnc-1 carries its own red-then-green unit test."
    },
    {
      "stage": "documentation-and-capture",
      "classification": "required",
      "role": "docs",
      "reason": "Cell lnc-2 edits doctrine prose and the docs role."
    },
    {
      "stage": "read-only-gather",
      "classification": "not-applicable",
      "role": "read",
      "reason": "The leader traced each issue already."
    },
    {
      "stage": "fact-extraction",
      "classification": "not-applicable",
      "role": "extraction",
      "reason": "No narrow lookup is needed."
    },
    {
      "stage": "generation-fallback",
      "classification": "not-applicable",
      "role": "generation",
      "reason": "Every job has a specific role."
    },
    {
      "stage": "independent-review",
      "classification": "not-applicable",
      "role": "review",
      "reason": "Independent review stays user-invoked."
    },
    {
      "stage": "generic-advisor",
      "classification": "not-applicable",
      "role": "advisor",
      "reason": "Not high-risk."
    },
    {
      "stage": "supervision",
      "classification": "not-applicable",
      "role": "supervisor",
      "reason": "No unattended loop."
    },
    {
      "stage": "blind-lane-1",
      "classification": "not-applicable",
      "role": "lane-1",
      "reason": "No competing designs."
    },
    {
      "stage": "blind-lane-2",
      "classification": "not-applicable",
      "role": "lane-2",
      "reason": "No competing designs."
    },
    {
      "stage": "blind-lane-3",
      "classification": "not-applicable",
      "role": "lane-3",
      "reason": "No competing designs."
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
      "reason": "Hat wave skipped: each cell fixes a reported, traced bug with the fix shape in the report."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    },
    {
      "stage": "hat-alternatives",
      "classification": "not-applicable",
      "role": "hat-alternatives",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    },
    {
      "stage": "hat-user-impact",
      "classification": "not-applicable",
      "role": "hat-user-impact",
      "reason": "Hat wave skipped: see hat-facts-gaps."
    }
  ]
}
```
