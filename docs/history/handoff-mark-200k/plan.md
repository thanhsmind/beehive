---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: handoff-mark-200k

## Summary

The default context mark at which a leader hands off falls from 300000 to
200000 tokens. One code cell changes the default, the tests that pin it,
the playbook sentence and the concept. One test cell checks the contract
and adds the missing case.

Mode: `small` — one default value in two product files, one flag
(`covered-contract-change`).

Playbook: `skills/bee-planning/playbooks/perf/perf-issue.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`7bc2853d`): the default mark is 200000; line text, clean-stop rule and the 0 switch stay.
- C1 (`03ca8fc1`): test contract handoff-mark-directive at the 200k mark.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The default lives in one function | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:355 | `_ => 300_000,` |
| 2 | A test pins the default at the mark | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:3333 | `fn handoff_mark_absent_key_at_exactly_300000_tokens_adds_the_directive_line() {` |
| 3 | The playbook names the default | read | packages/bee/playbooks/handoff.md:17 | `- The default context mark is 300000` |

## Approach

hm2-1 changes the value in `compute_handoff_warning`, moves the pinned
tests to the new mark, edits the playbook sentence and the concept, and
runs the regen. hm2-2 is the tester.

SMALLER PATH: a config-only change in this repo is cheaper, but it does
not reach other projects, and D1 names the default. PASS.

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| A pinned test keeps the old mark | LOW | hm2-1 | the handoff_mark filter is green |
| The rendered playbook drifts from its source | LOW | hm2-1 | release-manifest check green |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "cd477eb57b66a0cef7310533c504a43a20e8b19e131431e73112bda9a9f114c8",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"hm2-1 changes the default and its pins."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"hm2-2 checks the contract and adds the missing case."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"hm2-1 carries the two doc sentences."},
    {"stage":"consolidation","classification":"not-applicable","role":"writer","reason":"Small lane: no consolidation cell."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The leader already read every anchor."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Small lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Small lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Small lane."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Small lane."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"The next release publishes it."},
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

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| hm2-1 | Lower the default handoff mark to 200000 tokens | prompt_context.rs, handoff playbook, concept, regen output | — | the handoff line at 200k context | handoff_mark tests and manifest check green |
| hm2-2 | Test the handoff mark contract at 200000 tokens | prompt_context.rs | hm2-1 | a test for a configured mark | handoff_mark tests green |

```json
[
  {
    "id": "hm2-1",
    "feature": "handoff-mark-200k",
    "title": "Lower the default handoff mark to 200000 tokens",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "7bc2853d-0196-4d43-bd90-3ff09210f266"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee/playbooks/handoff.md",
      ".bee/playbooks/handoff.md",
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md",
      "docs/history/codex-harness-hardening/release-manifest.json",
      ".bee/onboarding.json"
    ],
    "read_first": [
      "docs/history/handoff-mark-200k/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"
    ],
    "action": "Per D1: in packages/bee-rs/crates/bee/src/hooks/prompt_context.rs, function compute_handoff_warning, the default of handoff_context_tokens becomes 200000 in both match arms (today: `Some(Value::Number(n)) => n.as_u64().unwrap_or(300_000),` and `_ => 300_000,`). Nothing else in that function changes. Move the existing tests that pin the old default to the new mark, with the same shape: find them with rg -n 'handoff_mark|mark 300k|300000' on that one file (today: transcript_past_handoff_mark_carries_warning_line expects 'context 310k tokens (mark 300k)'; handoff_mark_absent_key_at_exactly_300000_tokens_adds_the_directive_line; handoff_mark_absent_key_at_299999_tokens_adds_no_line; the zero-key loop). A test name that holds 300000 or 299999 takes the new number. Add no new test; hm2-2 owns new tests. In packages/bee/playbooks/handoff.md change 'The default context mark is 300000' to 200000 and nothing else. In docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md, find the 300000 mentions with rg -n, state the 200000 default in the handoff paragraph, and add 'handoff-mark-200k D1 (handoff_context_tokens defaults to 200000)' to the frontmatter decisions list; cite D-numbers, never raw decision ids; avoid the words deferred, until and later. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-rrt cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run that binary's dev regen (/home/thanhsmind/.cache/cargo-target-rrt/release/bee dev regen); never ./.bee/bin/bee for regen. Restore .claude/skills changes you did not cause. Workspace protocol: .bee/playbooks/handoff.md, .bee/onboarding.json and docs/history/codex-harness-hardening/release-manifest.json change only through bee dev regen; never edit them by hand. Then run the verify, and only the verify, never the full suite. Write no code comments. The only bee commands you may run are the ones this brief names.",
    "verify": "/home/thanhsmind/.cache/cargo-target-rrt/release/bee dev release-manifest --check && /home/thanhsmind/.cache/cargo-target-rrt/release/bee onboard --repo-root . --json && ./.bee/bin/bee knowledge check && CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-rrt PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- handoff_mark",
    "must_haves": {
      "why": "The leader context is re-read on every call; a reset at 200k cuts about 41 percent of leader tokens in the measured session (D1).",
      "out_of_scope": [
        "the directive line text",
        "the clean-stop rule",
        "new tests",
        "project config files"
      ],
      "truths": [
        "with handoff_context_tokens absent, a transcript at 200000 context tokens gets the directive line with the text 'mark 200k'",
        "with handoff_context_tokens absent, a transcript at 199999 context tokens gets no line",
        "handoff_context_tokens 0 gives no line at any size, and a configured number is used as before",
        "packages/bee/playbooks/handoff.md and its rendered copy .bee/playbooks/handoff.md name 200000 as the default mark",
        "the token economy concept states the 200000 default and cites handoff-mark-200k D1"
      ],
      "prohibitions": [
        "No change to the directive line wording other than the mark number it prints"
      ]
    },
    "judge_obligation_ack": "One default value changes in a hook; the tester cell hm2-2 runs in a separate dispatched worker and reads the change with no shared context.",
    "behavior_change": true
  },
  {
    "id": "hm2-2",
    "feature": "handoff-mark-200k",
    "title": "Test the handoff mark contract at 200000 tokens",
    "lane": "small",
    "role": "test",
    "deps": [
      "hm2-1"
    ],
    "decisions": [
      "03ca8fc1-695c-4f58-9560-7f951797da70"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs"
    ],
    "read_first": [
      "docs/history/handoff-mark-200k/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Tester for hm2-1, contract C1. Coverage judgment first: in the test module of packages/bee-rs/crates/bee/src/hooks/prompt_context.rs, find the mark tests with rg -n 'handoff_mark' on that one file and cite, by test name, which C1 clause each one proves (at 200000 adds the line with 'mark 200k'; 199999 adds none; 0 adds none at any size). Then author only the gap. The expected gap is the clause 'a configured number wins over the default': with the helper handoff_mark_warning, a config of 300000 at 250000 context tokens adds no line, and a config of 150000 at 160000 context tokens adds the line with 'mark 150k'. Name each new test so the name holds handoff_mark. Check that compute_handoff_warning holds no 300_000 literal. A red on an hm2-1 truth is a finding to report, not a fix to make in non-test code. Run the verify, and only the verify, never the full suite. Write no code comments. The only bee commands you may run are the ones this brief names.",
    "verify": "CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-rrt PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- handoff_mark",
    "must_haves": {
      "why": "A code cell never certifies itself; the clause that a configured mark wins has no test at the new default (C1).",
      "out_of_scope": [
        "non-test code",
        "tests outside the mark contract"
      ],
      "truths": [
        "every clause of contract handoff-mark-directive is proven by a named test in prompt_context.rs",
        "a test shows a configured mark above the default silences the line between the two, and a configured mark below the default fires the line with its own number"
      ],
      "prohibitions": [
        "No edit outside the test module"
      ]
    },
    "judge_obligation_ack": "One default value changes in a hook; the tester cell hm2-2 runs in a separate dispatched worker and reads the change with no shared context.",
    "behavior_change": false
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| Default mark | key absent, 200000 tokens | Pass when the line names 'mark 200k' |
| Below the mark | key absent, 199999 tokens | Pass when no line is added |
| Off switch | key 0, any size | Pass when no line is added |
| Configured mark | key 300000 at 250000; key 150000 at 160000 | Pass when the configured number decides |

## Open Questions

(none)

## Out of scope

- A hard stop at the mark; the line stays a directive at the next clean stop.
- The worker launch flags through paseo.
