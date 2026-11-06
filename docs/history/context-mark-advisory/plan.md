---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: context-mark-advisory

## Summary

Today the prompt hook tells a session at 150k tokens to stop, write a
handoff and leave. The doctrine says the same in nine places. After this
change the hook only says the size and "keep working". The rule says a
session finishes the task; a handoff happens only on the user's word or at
feature close, and then it joins the same waggledance run family through
`follow_up_of`.

Mode: `small` — one hook string and its tests, plus one rule rewritten in
every home. No public contract changes shape: the config key, its default
and its 0 switch stay.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`8921875b`): the mark is advisory; the user decides a switch.
- D2 (`48741dcf`): the hook line text, default kept.
- D3 (`a23d677e`): the rule text, with `follow_up_of`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The hook line is one format string | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:367 | `"context {n_k}k tokens is past the handoff mark ({t_k}k): finish this step,` |
| 2 | The default mark is 150000 and 0 turns the line off | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:354 | `if threshold == 0 {` |
| 3 | AGENTS.md renders from the block source | read | packages/bee/AGENTS.block.md:291 | `<!-- rule: agents-context-handoff-65 -->` |
| 4 | The rule index names the rule | read | docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:222 | `- `agents-context-handoff-65` (AGENTS.md § Care for the session):` |

## Approach

Two cells with disjoint files run in parallel. cma-1 changes the string
and its three tests. cma-2 rewrites the rule in every home, runs
`bee dev regen`, and updates the knowledge concept and the rule index.

Rejected: default 0 (D2 rationale).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| A doc home keeps the old stop wording | MEDIUM | cma-2 | `rg "past the handoff mark|65% context"` over the homes returns only history |
| A test still asserts the old text | LOW | cma-1 | prompt_context tests green |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"cma-1 changes one hook string and its tests."},
    {"stage":"test-and-live-proof","classification":"required","role":"code","reason":"cma-1 updates its tests red-first."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"cma-2 rewrites the rule in every home."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader already read every anchor."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Small lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Small lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Small lane."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Small lane."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"scripts/release.sh runs after merge, from main."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No extraction."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"No consult needed."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | hook line, rule text in every home | the whole ask fits one slice | a long session sees "keep working" | the waggledance handoff flag follow-up |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| cma-1 | Make the context mark line advisory | hooks/prompt_context.rs | — | past the mark the line says keep working | prompt_context tests green |
| cma-2 | Rewrite the context handoff rule as advisory in every home | doctrine, handbook, knowledge, AGENTS block, regen output | — | no home tells a session to stop at the mark | release-manifest check green |

```json
[
  {
    "id": "cma-1",
    "feature": "context-mark-advisory",
    "title": "Make the context mark line advisory",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["8921875b-7143-4620-a636-747e75a766de", "48741dcf-019c-4e33-8cc4-6ceec7e3bca4"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs"
    ],
    "read_first": [
      "docs/history/context-mark-advisory/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Contract decision 48741dcf. In compute_handoff_warning (prompt_context.rs), keep the threshold read, the 150000 default and the 0 switch unchanged. Replace the format string with exactly: `context {n_k}k tokens (mark {t_k}k): keep working; hand off only on the user's word or at feature close`. Update the three tests at transcript_past_handoff_mark_carries_warning_line, transcript_below_handoff_mark_adds_no_line and handoff_context_tokens_zero_adds_no_line_at_any_size: the past-mark test asserts `context 160k tokens (mark 150k): keep working; hand off only on the user's word or at feature close`; the two no-line tests assert stdout does not contain `(mark ` and does not contain `keep working; hand off`. Red first: change the expected text, see the test fail, then change the string.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prompt_context",
    "must_haves": {
      "truths": [
        "past the mark the hook adds the advisory line with the token count and the mark",
        "the line carries no stop order and no handoff steps",
        "below the mark and with handoff_context_tokens 0 the hook adds no line"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs", "substantive": "advisory line and its tests"}
      ],
      "key_links": ["compute_handoff_warning returns the advisory line"],
      "prohibitions": ["Do not change the default or the 0 switch", "No code comments"]
    },
    "behavior_change": true
  },
  {
    "id": "cma-2",
    "feature": "context-mark-advisory",
    "title": "Rewrite the context handoff rule as advisory in every home",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": ["8921875b-7143-4620-a636-747e75a766de", "a23d677e-69b2-4d7c-b1a3-06e4dd9068f8", "48741dcf-019c-4e33-8cc4-6ceec7e3bca4"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "AGENTS.md",
      "skills/bee-hive/SKILL.md",
      "skills/bee-hive/references/go-mode.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/handbook/stages/swarming.md",
      "docs/handbook/architecture-map.md",
      "docs/02-architecture.md",
      "README.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md",
      "docs/history/handoff-self-relaunch/promote-proposals.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/context-mark-advisory/CONTEXT.md",
      "packages/bee/AGENTS.block.md"
    ],
    "affects_skills": ["skills/bee-hive/SKILL.md", "skills/bee-swarming/SKILL.md"],
    "affects_specs": [],
    "action": "Rewrite rule agents-context-handoff-65 per decision a23d677e in every home. New meaning, in short plain sentences: a session never interrupts work at a context mark; it finishes the task. The hook may show the context size against the mark (handoff_context_tokens, default 150000); that line is advice, not a stop. A handoff is written only when the user asks for a switch or at feature close. Then it goes through the host's leader door: waggledance_dispatch with leader: true and follow_up_of naming the current run, so the successor joins the same run family. Never a self-spawned Paseo agent, never tell the user to open a new session. Show the handoff and wait only when a gate or decision the user owns is pending, or when the host has no leader door. Homes: packages/bee/AGENTS.block.md (the § Care for the session bullet under the rule marker), then run `.bee/bin/bee dev regen` so AGENTS.md and the rendered skill copies follow; skills/bee-hive/SKILL.md line 111; skills/bee-hive/references/go-mode.md lines 117 and 124 (no mid-swarm stop; landing a slice before a user-asked switch stays); skills/bee-hive/references/routing-and-contracts.md lines 116 and 196; skills/bee-swarming/SKILL.md line 165 (keep the line 242 worker [HANDOFF] token: a worker window is out of scope); skills/bee-swarming/references/swarming-reference.md near line 746 and 780 (the HANDOFF.json schema stays); docs/handbook/stages/swarming.md near line 97; docs/handbook/architecture-map.md line 103 (swarming to paused on the user's word or feature close); docs/02-architecture.md line 244; README.md lines 100 and 508. Knowledge: in the rule index entry for agents-context-handoff-65 (router-triage-and-the-agents-md-duplication-boundary.md line 222), rewrite the spoken form and cite D1 and D3 of docs/history/context-mark-advisory; in docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md near line 115, replace the hook line with `context <N>k tokens (mark <T>k): keep working; hand off only on the user's word or at feature close` and say it is advisory. In docs/history/handoff-self-relaunch/promote-proposals.md, mark the old stop-at-mark proposal superseded by docs/history/context-mark-advisory D3 (decision a23d677e supersedes c9439edb). Reserve the release manifest before regen. List every file regen changed in the cap and name the rendered copies as a scope deviation.",
    "verify": ".bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "no home tells a session to stop or hand off at the context mark",
        "every home says a handoff happens only on the user's word or at feature close",
        "the handoff dispatch names waggledance_dispatch with leader: true and follow_up_of the current run"
      ],
      "artifacts": [
        {"path": "packages/bee/AGENTS.block.md", "substantive": "rewritten rule under the agents-context-handoff-65 marker"}
      ],
      "key_links": ["AGENTS.md matches the block after bee dev regen"],
      "prohibitions": ["Do not change the worker [HANDOFF] token", "Do not change the HANDOFF.json schema"]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| Happy | transcript past the mark | Pass when the hook line is the advisory text |
| Edge | below the mark; handoff_context_tokens 0 | Pass when no line is added |
| Docs | every rule home | Pass when no home orders a stop at the mark |

## Open Questions

(none)

## Out of scope

- The worker `[HANDOFF]` token.
- The waggledance handoff flag beyond `follow_up_of`.
