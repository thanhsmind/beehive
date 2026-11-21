---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: leader-context-thrift

## Summary

Keep the leader's context small. Five code cells, each with its own
tester, and two docs cells: the handoff mark falls to 120k, the handoff
record carries six more fields, a cold-read probe checks the record, a
guard refuses a leader's unranged read of a long file, dispatch prepare
can leave the worker prompt in a file, and `bee report context` measures
the result.

Mode: `standard` — about 12 product files across hooks, the handoff
store, the dispatch driver and a new report verb.

Playbook: `skills/bee-planning/playbooks/feature/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`985cc944`), C1 (`38d13294`): default mark 120000.
- D2 (`6d462761`), C2 (`7f7519bd`): six handoff record fields.
- D3 (`4945350f`): cold-read probe before session release.
- D4 (`c1d2947a`), C3 (`6d4f74aa`): leader read cap.
- D5 (`1fef0cf9`), C4 (`e01214ca`): dispatch prepare --prompt-out.
- D6 (`2d8d37a5`), C5 (`70dda218`): bee report context.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The default mark lives in one function | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:354 | `Some(Value::Number(n)) => n.as_u64().unwrap_or(200_000),` |
| 2 | The handoff record is built in one writer | read | packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs:157 | `pub(crate) fn write_mailbox_handoff(` |
| 3 | The preamble renders the handoff block in one function | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs:179 | `pub fn handoff_block_lines(handoff: &Value, outcome: Option<&HandoffOutcome>) -> Vec<String> {` |
| 4 | Hooks tell a worker from a leader by one marker | read | packages/bee-rs/crates/bee/src/hooks/mod.rs:94 | `pub(crate) fn herding_worker_marker_set() -> bool {` |
| 5 | Dispatch prepare inlines the herding prompt as stdin | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2945 | `payload.insert("stdin".into(), Value::String(pane_prompt.clone()));` |
| 6 | The playbook names the default mark | read | packages/bee/playbooks/handoff.md:17 | `- The default context mark is 200000 (`handoff_context_tokens`; 0 turns` |

## Approach

Five independent parts, each a code cell and a tester cell, plus the
probe doctrine and one concept sync at the end. Cells that share the
flag catalog, the registry or the regen outputs serialize on those
files; the scheduler orders them.

SMALLER PATH: a config-only mark change in this repo is cheaper but does
not reach other projects and leaves the read and record parts undone;
the owner asked for all six. PASS.

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| The read cap refuses a read a leader needs | MEDIUM | lcx-6 | ranged reads pass; config 0 turns it off; C3 tests |
| A pinned test keeps the old mark | LOW | lcx-1 | handoff_mark filter green |
| New flags drift the catalog pin | LOW | lcx-3, lcx-8, lcx-10 | catalog test inside each verify filter or the end suite |
| The prompt file path leaks into another dispatch | LOW | lcx-8 | the path is per call and named by the caller |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "cd477eb57b66a0cef7310533c504a43a20e8b19e131431e73112bda9a9f114c8",
  "stages": [
    {
      "stage": "implementation",
      "classification": "required",
      "role": "code",
      "reason": "lcx-1, 3, 6, 8 and 10 write the code."
    },
    {
      "stage": "test-and-live-proof",
      "classification": "required",
      "role": "test",
      "reason": "lcx-2, 4, 7, 9 and 11 test each code cell with no shared context."
    },
    {
      "stage": "documentation-and-capture",
      "classification": "required",
      "role": "docs",
      "reason": "lcx-5 writes the probe doctrine and lcx-12 syncs the concept."
    },
    {
      "stage": "consolidation",
      "classification": "not-applicable",
      "role": "writer",
      "reason": "No consolidation cell: lcx-12 is the one concept sync."
    },
    {
      "stage": "read-only-gather",
      "classification": "required",
      "role": "read",
      "reason": "A haiku gather mapped the code before this plan (job-1795269218813-3413911-1)."
    },
    {
      "stage": "hat-facts-gaps",
      "classification": "not-applicable",
      "role": "hat-facts-gaps",
      "reason": "The measurements in CONTEXT.md are the facts; each part is a small independent change."
    },
    {
      "stage": "hat-alternatives",
      "classification": "not-applicable",
      "role": "hat-alternatives",
      "reason": "Alternatives are recorded per decision with their rejection reason."
    },
    {
      "stage": "hat-user-impact",
      "classification": "not-applicable",
      "role": "hat-user-impact",
      "reason": "The owner is the user of every part and asked for it."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Risks are listed in this plan; the read cap has an off switch."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Value is measured in CONTEXT.md and re-measured by lcx-10."
    },
    {
      "stage": "planning",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "The leader writes this plan."
    },
    {
      "stage": "deployment",
      "classification": "not-applicable",
      "role": "deploy",
      "reason": "The next release publishes it."
    },
    {
      "stage": "fact-extraction",
      "classification": "not-applicable",
      "role": "extraction",
      "reason": "No extraction."
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
      "reason": "No consult needed."
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
      "reason": "One clear design per part."
    },
    {
      "stage": "blind-lane-2",
      "classification": "not-applicable",
      "role": "lane-2",
      "reason": "One clear design per part."
    },
    {
      "stage": "blind-lane-3",
      "classification": "not-applicable",
      "role": "lane-3",
      "reason": "One clear design per part."
    },
    {
      "stage": "design-ui",
      "classification": "not-applicable",
      "role": "design",
      "reason": "No rendered UI."
    }
  ]
}
```

## Cells — current slice (preview)

| id | title | files | deps | role | proof filter |
|---|---|---|---|---|---|
| lcx-1 | Lower the default handoff mark to 120000 tokens | 7 files | — | code | handoff_mark |
| lcx-2 | Test the handoff mark contract at 120000 tokens | 1 files | lcx-1 | test | handoff_mark |
| lcx-3 | Carry verified, tree, key files, gotchas, standing orders and ruled-out paths on a handoff | 6 files | — | code | handoff |
| lcx-4 | Test the handoff record fields contract | 2 files | lcx-3 | test | handoff_record_fields |
| lcx-5 | Probe a handoff record with a cold read before the session ends | 6 files | lcx-1, lcx-3 | docs | no_shipped_command_spelling_is_refused_by_the_widened_guard |
| lcx-6 | Refuse a leader's unranged read of a long file | 3 files | — | code | write_guard |
| lcx-7 | Test the leader read cap contract | 1 files | lcx-6 | test | leader_read_cap |
| lcx-8 | Leave the worker prompt in a file with dispatch prepare --prompt-out | 6 files | lcx-3 | code | prepare |
| lcx-9 | Test the dispatch prompt-out contract | 1 files | lcx-8 | test | dispatch_prompt_out |
| lcx-10 | Measure leader context with bee report context | 7 files | lcx-8 | code | report_context |
| lcx-11 | Test the report context contract | 1 files | lcx-10 | test | report_context |
| lcx-12 | Sync the token economy concept with the leader context thrift | 1 files | lcx-1, lcx-3, lcx-5, lcx-6, lcx-8, lcx-10 | docs | knowledge check |

```json
[
  {
    "id": "lcx-1",
    "feature": "leader-context-thrift",
    "title": "Lower the default handoff mark to 120000 tokens",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "985cc944-c77e-4ba0-80b2-ccd8470d87c3"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee/playbooks/handoff.md",
      ".bee/playbooks/handoff.md",
      ".bee/config.json",
      ".bee/onboarding.json",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1: in packages/bee-rs/crates/bee/src/hooks/prompt_context.rs, function compute_handoff_warning, both default arms (`unwrap_or(200_000)` and `_ => 200_000`) become 120_000. Move the existing tests that pin the old default to the new mark with the same shape (rg -n 'handoff_mark|200000|199999|mark 200k|handoff_context_tokens' on that file): names holding 200000 or 199999 take 120000 and 119999; expected strings 'mark 200k' take 'mark 120k'; the configured-above test uses a value above 120000 and a context between; the configured-below test uses a value below 120000. Add no new test. In packages/bee/playbooks/handoff.md change 'The default context mark is 200000' to 120000 and nothing else. In .bee/config.json set handoff_context_tokens to 120000 and change nothing else in it. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run /home/thanhsmind/.cache/cargo-target-lcx/release/bee dev regen; never ./.bee/bin/bee for regen. After regen restore .claude/skills changes you did not cause (git restore .claude/skills and git clean -fd .claude/skills). Files that bee dev regen writes (.bee/playbooks/*, .bee/onboarding.json, release manifests, packages/bee-rs/crates/bee/src/generated/*) change only through regen. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "/home/thanhsmind/.cache/cargo-target-lcx/release/bee onboard --repo-root . --json && /home/thanhsmind/.cache/cargo-target-lcx/release/bee dev release-manifest --check && CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- handoff_mark",
    "must_haves": {
      "why": "Leader context is re-read on every call; a reset at 120k cuts up to 64 percent of leader input in the 207-session replay (D1).",
      "out_of_scope": [
        "the directive line text",
        "new tests",
        "the knowledge concept (lcx-12)"
      ],
      "truths": [
        "with handoff_context_tokens absent, a transcript at 120000 context tokens gets the directive line with the text 'mark 120k'",
        "with handoff_context_tokens absent, a transcript at 119999 context tokens gets no line",
        "packages/bee/playbooks/handoff.md and .bee/playbooks/handoff.md name 120000 as the default mark",
        ".bee/config.json holds handoff_context_tokens 120000"
      ],
      "prohibitions": [
        "No change to the directive wording other than the number"
      ]
    },
    "behavior_change": true,
    "judge_obligation_ack": "Each code cell has a separate dispatched tester cell that reads the change with no shared context."
  },
  {
    "id": "lcx-2",
    "feature": "leader-context-thrift",
    "title": "Test the handoff mark contract at 120000 tokens",
    "lane": "standard",
    "role": "test",
    "deps": [
      "lcx-1"
    ],
    "decisions": [
      "38d13294-b8e4-49f7-b7b6-dd5201e9081b"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Tester for lcx-1, contract C1. In the test module of packages/bee-rs/crates/bee/src/hooks/prompt_context.rs, find the mark tests (rg -n 'handoff_mark') and cite by name which C1 clause each proves; author only the gaps, each new test name holding handoff_mark. Check compute_handoff_warning holds no 200_000 literal. A red on an lcx-1 truth is a finding to report, not a fix in non-test code. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- handoff_mark",
    "must_haves": {
      "why": "A code cell never certifies itself (C1).",
      "out_of_scope": [
        "non-test code"
      ],
      "truths": [
        "every clause of contract handoff-mark-directive is proven by a named test in prompt_context.rs"
      ],
      "prohibitions": [
        "No edit outside the test module"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "lcx-3",
    "feature": "leader-context-thrift",
    "title": "Carry verified, tree, key files, gotchas, standing orders and ruled-out paths on a handoff",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "6d462761-ae60-4ec1-98ed-1c949cc5dbd7"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      ".bee/onboarding.json",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D2: `bee state handoff write` takes --verified, --tree, --key-files, --gotchas, --standing-orders, --ruled-out on both kinds (pause and planned-next). Find where the write flags are declared (rg -n 'previous-cell' packages/bee-rs/crates/bee/src, excluding generated/) and add the six there; regen refreshes generated/registry_payload.json. In verbs/workflow_store/handoff.rs, function write_mailbox_handoff, store each present flag under its snake_case key: key_files split on commas, verified, gotchas, standing_orders and ruled_out split on semicolons into trimmed non-empty string arrays, tree a string; an absent flag stores no key. The planned-next branch today copies only named fields: add the six there too. In hooks/session_preamble/render.rs, function handoff_block_lines, print each present field on its own line after next_action ('- Standing orders: …', '- Ruled out: …', '- Key files: …', '- Gotchas: …', '- Verified: …', '- Tree: …'), arrays joined with '; ', each line cut to 300 characters. Find the `bee state handoff show` implementation (rg -n 'handoff show' or the show handler next to write) and make it print the same fields. If catalog.rs pins a distinct flag count, move the pin to the new count. Tests for C2 belong to lcx-4; keep existing tests green. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run /home/thanhsmind/.cache/cargo-target-lcx/release/bee dev regen; never ./.bee/bin/bee for regen. After regen restore .claude/skills changes you did not cause (git restore .claude/skills and git clean -fd .claude/skills). Files that bee dev regen writes (.bee/playbooks/*, .bee/onboarding.json, release manifests, packages/bee-rs/crates/bee/src/generated/*) change only through regen. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "/home/thanhsmind/.cache/cargo-target-lcx/release/bee dev release-manifest --check && CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- handoff",
    "must_haves": {
      "why": "The schema names verified, tree, key_files and gotchas but the CLI has no flag, so leaders drop them; standing orders and ruled-out paths are what a handoff loses (D2).",
      "out_of_scope": [
        "the probe step (lcx-5)",
        "new C2 tests (lcx-4)"
      ],
      "truths": [
        "the handoff write verb accepts --verified, --tree, --key-files, --gotchas, --standing-orders and --ruled-out on both kinds",
        "each present flag is stored under its snake_case key, split as D2 says, and an absent flag stores no key",
        "the preamble HANDOFF block and bee state handoff show print each present field on its own line, at most 300 characters"
      ],
      "prohibitions": [
        "No change to the existing kind preconditions"
      ]
    },
    "behavior_change": true,
    "judge_obligation_ack": "Each code cell has a separate dispatched tester cell that reads the change with no shared context."
  },
  {
    "id": "lcx-4",
    "feature": "leader-context-thrift",
    "title": "Test the handoff record fields contract",
    "lane": "standard",
    "role": "test",
    "deps": [
      "lcx-3"
    ],
    "decisions": [
      "7f7519bd-ed63-4288-804d-90cdbcbaca37"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Tester for lcx-3, contract C2. Write tests whose names hold handoff_record_fields: a pause write and a planned-next write (reuse the existing planned-next fixtures in the handoff test module) with all six flags read back with every value split as D2 says; a write with none stores none of the six keys; handoff_block_lines prints a present field and omits an absent one, and cuts a 400-character value to 300. A red on an lcx-3 truth is a finding to report, not a fix in non-test code. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- handoff_record_fields",
    "must_haves": {
      "why": "A code cell never certifies itself (C2).",
      "out_of_scope": [
        "non-test code"
      ],
      "truths": [
        "every clause of contract handoff-record-fields is proven by a named test"
      ],
      "prohibitions": [
        "No edit outside test modules"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "lcx-5",
    "feature": "leader-context-thrift",
    "title": "Probe a handoff record with a cold read before the session ends",
    "lane": "standard",
    "role": "docs",
    "deps": [
      "lcx-1",
      "lcx-3"
    ],
    "decisions": [
      "4945350f-2de3-4e6d-bca6-5c514ca1b92d"
    ],
    "files": [
      "packages/bee/playbooks/handoff.md",
      ".bee/playbooks/handoff.md",
      "skills/bee-swarming/references/swarming-reference.md",
      ".bee/onboarding.json",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D3: in packages/bee/playbooks/handoff.md, inside the agents-context-handoff-65 rule block, add the cold-read probe step before 'Release the session after the handoff': one dispatch through `bee dispatch prepare --runtime claude --kind gather --role read --json` whose brief holds only the `bee state handoff show` output and asks for the next action and the ruled-out paths; a miss means rewrite the record and probe once more. Name the six handoff write flags (--verified, --tree, --key-files, --gotchas, --standing-orders, --ruled-out) where the playbook says what the task text names. In skills/bee-swarming/references/swarming-reference.md near line 712-719, add standing_orders and ruled_out to the schema line, replace the sentence that says the four newer fields have no flags and must ride --next-action with the six flag names, and add the probe step. Keep every rule marker. Write full valid command spellings (runtime flag present). Avoid the words deferred, until and later. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run /home/thanhsmind/.cache/cargo-target-lcx/release/bee dev regen; never ./.bee/bin/bee for regen. After regen restore .claude/skills changes you did not cause (git restore .claude/skills and git clean -fd .claude/skills). Files that bee dev regen writes (.bee/playbooks/*, .bee/onboarding.json, release manifests, packages/bee-rs/crates/bee/src/generated/*) change only through regen. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "/home/thanhsmind/.cache/cargo-target-lcx/release/bee onboard --repo-root . --json && /home/thanhsmind/.cache/cargo-target-lcx/release/bee dev release-manifest --check && ./.bee/bin/bee knowledge check && CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- no_shipped_command_spelling_is_refused_by_the_widened_guard",
    "must_haves": {
      "why": "A record a fresh cheap model cannot act on will not serve a fresh leader (D3).",
      "out_of_scope": [
        "code"
      ],
      "truths": [
        "packages/bee/playbooks/handoff.md states the cold-read probe through bee dispatch prepare with kind gather and role read before the session release",
        "swarming-reference.md lists the six handoff write flags and drops the sentence that sends the four fields through --next-action"
      ],
      "prohibitions": [
        "No rule marker removed or renamed"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "lcx-6",
    "feature": "leader-context-thrift",
    "title": "Refuse a leader's unranged read of a long file",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "c1d2947a-779a-4975-92aa-ff4299875d49"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/leader_read.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/mod.rs"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D4: add a leader read cap to the PreToolUse guard. Put the decision in a pure function in a new module hooks/write_guard/leader_read.rs (inputs: tool name, tool input JSON, worker flag, the file's line count, the limit; output: allow or a refusal text), and call it from hooks/write_guard/main.rs where Read tools are handled (is_read_tool, near line 260) and where Bash commands are handled. Worker detection uses crate::hooks::herding_worker_marker_set() or a set PASEO_AGENT_ID. The limit is config key leader_read_max_lines, default 400, 0 off; read it the way neighbouring guard keys are read. Refuse: a Read with no limit on a file with more than the limit lines; a Bash command whose whole text is one `cat <file>` (optionally after a cd … &&) of such a file. Pass: ranged Reads, files at or under the limit, missing files, workers, pipes or any other command. The refusal text names both remedies: 'read a range with offset and limit' and 'dispatch a read gather: .bee/bin/bee dispatch prepare --runtime claude --kind gather --role read --json'. Count lines cheaply (bytes read once, newline count). Tests for C3 belong to lcx-7; keep existing guard tests green. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- write_guard",
    "must_haves": {
      "why": "Leader file reads are about 40 percent of attributable leader re-read cost; the prose rule does not hold (D4).",
      "out_of_scope": [
        "worker read rules",
        "Grep and Glob"
      ],
      "truths": [
        "in a leader session a Read with no limit on a file longer than leader_read_max_lines is refused with both remedies named",
        "a ranged Read, a file at or under the limit, a worker session and leader_read_max_lines 0 all pass",
        "a bare cat of one long file is refused and a piped cat passes"
      ],
      "prohibitions": [
        "No refusal in a worker session"
      ]
    },
    "behavior_change": true,
    "judge_obligation_ack": "Each code cell has a separate dispatched tester cell that reads the change with no shared context."
  },
  {
    "id": "lcx-7",
    "feature": "leader-context-thrift",
    "title": "Test the leader read cap contract",
    "lane": "standard",
    "role": "test",
    "deps": [
      "lcx-6"
    ],
    "decisions": [
      "6d4f74aa-af76-4bc4-8027-f0cf626b8df9"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/leader_read.rs"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Tester for lcx-6, contract C3. Write tests whose names hold leader_read_cap in the test module of hooks/write_guard/leader_read.rs, covering every C3 clause: 401-line unranged Read refused with 'offset and limit' and 'bee dispatch prepare' in the text; 400 lines passes; ranged passes; worker passes; limit 0 passes every size; bare cat of 401 lines refused; `cat f | head` passes. A red on an lcx-6 truth is a finding to report, not a fix in non-test code. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- leader_read_cap",
    "must_haves": {
      "why": "A code cell never certifies itself (C3).",
      "out_of_scope": [
        "non-test code"
      ],
      "truths": [
        "every clause of contract leader-read-cap is proven by a named test"
      ],
      "prohibitions": [
        "No edit outside the test module"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "lcx-8",
    "feature": "leader-context-thrift",
    "title": "Leave the worker prompt in a file with dispatch prepare --prompt-out",
    "lane": "standard",
    "role": "code",
    "deps": [
      "lcx-3"
    ],
    "decisions": [
      "1fef0cf9-7feb-46ed-9f58-1d376cdaea44"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "skills/bee-swarming/SKILL.md",
      ".bee/onboarding.json",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D5: `bee dispatch prepare` takes --prompt-out <path>. Declare the flag where the prepare flags are declared (rg -n '\"brief-file\"' excluding generated/). In verbs/drivers/prepare.rs, on the herding and cli-exec arms that insert payload stdin (lines near 2945, 3030, 3151, 3175, 3221), when --prompt-out is set write the pane prompt to the path (create parent dirs), do not insert stdin, and make the returned command read `--task-file <path>` instead of `--task-file -`. prompt_sha256 stays the hash of the same prompt. The Agent arm and the session-model arm ignore the flag. Without the flag the output is byte-identical. If catalog.rs pins a distinct flag count, move the pin. In skills/bee-swarming/SKILL.md, where the dispatch door is described, tell the leader to pass --prompt-out .bee/tmp/prompts/<dispatch>.txt on herding dispatches so the prompt stays out of its context. Tests for C4 belong to lcx-9. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run /home/thanhsmind/.cache/cargo-target-lcx/release/bee dev regen; never ./.bee/bin/bee for regen. After regen restore .claude/skills changes you did not cause (git restore .claude/skills and git clean -fd .claude/skills). Files that bee dev regen writes (.bee/playbooks/*, .bee/onboarding.json, release manifests, packages/bee-rs/crates/bee/src/generated/*) change only through regen. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "/home/thanhsmind/.cache/cargo-target-lcx/release/bee dev release-manifest --check && CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- prepare",
    "must_haves": {
      "why": "The inlined worker prompt in dispatch prepare output is re-read on every later leader call (D5).",
      "out_of_scope": [
        "the Agent arm"
      ],
      "truths": [
        "with --prompt-out on a herding dispatch the prompt is written to the path, payload stdin is absent and the command reads --task-file with that path",
        "prompt_sha256 equals the sha256 of the written file",
        "without --prompt-out the output is unchanged"
      ],
      "prohibitions": [
        "No change to the output without the flag"
      ]
    },
    "behavior_change": true,
    "judge_obligation_ack": "Each code cell has a separate dispatched tester cell that reads the change with no shared context."
  },
  {
    "id": "lcx-9",
    "feature": "leader-context-thrift",
    "title": "Test the dispatch prompt-out contract",
    "lane": "standard",
    "role": "test",
    "deps": [
      "lcx-8"
    ],
    "decisions": [
      "e01214ca-67d3-40b6-93f2-58512c1a0a1c"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Tester for lcx-8, contract C4. In verbs/drivers/tests.rs (or the prepare test module the existing prepare tests live in), write tests whose names hold dispatch_prompt_out: the file holds the prompt and its sha256 equals prompt_sha256; payload stdin is absent; the command names --task-file with the path; without the flag the JSON equals the no-flag output of the same dispatch. A red on an lcx-8 truth is a finding to report. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- dispatch_prompt_out",
    "must_haves": {
      "why": "A code cell never certifies itself (C4).",
      "out_of_scope": [
        "non-test code"
      ],
      "truths": [
        "every clause of contract dispatch-prompt-out is proven by a named test"
      ],
      "prohibitions": [
        "No edit outside test modules"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "lcx-10",
    "feature": "leader-context-thrift",
    "title": "Measure leader context with bee report context",
    "lane": "standard",
    "role": "code",
    "deps": [
      "lcx-8"
    ],
    "decisions": [
      "2d8d37a5-92b4-42b1-9a0f-d874ac97e8ee"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/report_context.rs",
      "packages/bee-rs/crates/bee/src/verbs/report.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      ".bee/onboarding.json",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D6: add `bee report context` with --since <iso>, optional --projects-dir (default: the Claude projects directory for this repo, resolved the way report_usage.rs resolves transcripts) and --json. Model it on verbs/report_usage.rs (transcript resolution, aggregate of message.usage, dedupe by message id). Put the aggregation in a pure function over parsed transcripts so tests run on fixtures. Leader sessions = top-level *.jsonl in the projects dir that no .bee/mailbox/*/activity.json transcript_path names. Context per call = input_tokens + cache_creation_input_tokens + cache_read_input_tokens of each distinct assistant message. Print: sessions, calls, mean and p90 context per call, the share of summed context from calls above the handoff mark (handoff_context_tokens, default from the same source compute_handoff_warning uses), and for sessions whose first non-meta user prompt contains 'handoff' or 'follow-up', the count of Bash commands containing 'cells reopen' or 'mailbox reflect' in their first 40 calls. Register the verb wherever report usage is registered (rg -n 'report usage|\"usage\"' outside generated/). Tests for C5 belong to lcx-11. Build with CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx cargo build --release --manifest-path packages/bee-rs/Cargo.toml and run /home/thanhsmind/.cache/cargo-target-lcx/release/bee dev regen; never ./.bee/bin/bee for regen. After regen restore .claude/skills changes you did not cause (git restore .claude/skills and git clean -fd .claude/skills). Files that bee dev regen writes (.bee/playbooks/*, .bee/onboarding.json, release manifests, packages/bee-rs/crates/bee/src/generated/*) change only through regen. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "/home/thanhsmind/.cache/cargo-target-lcx/release/bee dev release-manifest --check && CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- report_context",
    "must_haves": {
      "why": "The other parts need a measured before and after, and the rework count answers whether an earlier handoff costs correctness (D6).",
      "out_of_scope": [
        "cost weighting by model price"
      ],
      "truths": [
        "the report context verb prints sessions, calls, mean and p90 context, the above-mark share and the handoff rework count, and --json emits the same numbers",
        "a transcript named by a mailbox job is not counted as a leader"
      ],
      "prohibitions": [
        "No estimate: every number comes from message.usage"
      ]
    },
    "behavior_change": true,
    "judge_obligation_ack": "Each code cell has a separate dispatched tester cell that reads the change with no shared context."
  },
  {
    "id": "lcx-11",
    "feature": "leader-context-thrift",
    "title": "Test the report context contract",
    "lane": "standard",
    "role": "test",
    "deps": [
      "lcx-10"
    ],
    "decisions": [
      "70dda218-7f66-4cbf-b212-f505a5e56c59"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/report_context.rs"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Tester for lcx-10, contract C5. In the test module of verbs/report_context.rs write tests whose names hold report_context, on fixtures written to a temp dir: two leader transcripts and one worker transcript named by a mailbox activity.json; only the two leaders count; the mean comes from message usage; calls above the mark land in the share; a cells reopen Bash command inside a handoff-started session's first 40 calls counts and one after call 40 does not. A red on an lcx-10 truth is a finding to report. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-lcx PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- report_context",
    "must_haves": {
      "why": "A code cell never certifies itself (C5).",
      "out_of_scope": [
        "non-test code"
      ],
      "truths": [
        "every clause of contract report-context is proven by a named test"
      ],
      "prohibitions": [
        "No edit outside the test module"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "lcx-12",
    "feature": "leader-context-thrift",
    "title": "Sync the token economy concept with the leader context thrift",
    "lane": "standard",
    "role": "docs",
    "deps": [
      "lcx-1",
      "lcx-3",
      "lcx-5",
      "lcx-6",
      "lcx-8",
      "lcx-10"
    ],
    "decisions": [
      "985cc944-c77e-4ba0-80b2-ccd8470d87c3",
      "6d462761-ae60-4ec1-98ed-1c949cc5dbd7",
      "4945350f-2de3-4e6d-bca6-5c514ca1b92d",
      "c1d2947a-779a-4975-92aa-ff4299875d49",
      "1fef0cf9-7feb-46ed-9f58-1d376cdaea44",
      "2d8d37a5-92b4-42b1-9a0f-d874ac97e8ee"
    ],
    "files": [
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md"
    ],
    "read_first": [
      "docs/history/leader-context-thrift/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 to D6: in docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md state the 120000 default mark (replace the 200000 statement), the six handoff record fields and the cold-read probe, the leader read cap and its config key, dispatch prepare --prompt-out, and bee report context, each in one short paragraph in the existing Behaviors section, with the measured numbers from CONTEXT.md Measurements where a paragraph needs a why. Add to the frontmatter decisions list one entry per decision in the form 'leader-context-thrift D<n> (<short text>)'; cite D-numbers, never raw decision ids, one D-number per line. Add the new code paths to anchors. Avoid the words deferred, until and later. Write no code comments. Run the verify, and only the verify, never the full suite. The only bee commands you may run are the ones this brief names. Read CONTEXT.md first.",
    "verify": "./.bee/bin/bee knowledge check",
    "must_haves": {
      "why": "docs/knowledge is the state layer; a concept that lags the code misleads the next reader.",
      "out_of_scope": [
        "code"
      ],
      "truths": [
        "the token economy concept states the 120000 default and cites leader-context-thrift D1 to D6",
        "the concept names leader_read_max_lines, --prompt-out, the six handoff fields, the probe and bee report context"
      ],
      "prohibitions": [
        "No raw decision ids in the concept"
      ]
    },
    "behavior_change": true
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| Default mark | key absent, 120000 tokens | Pass when the line names 'mark 120k' |
| Below the mark | key absent, 119999 tokens | Pass when no line is added |
| Record fields | six flags on pause and planned-next | Pass when show prints every value |
| Read cap | 401-line unranged Read in a leader | Pass when refused with both remedies |
| Read cap off | leader_read_max_lines 0 | Pass when every size passes |
| Prompt out | --prompt-out on herding | Pass when the file hash equals prompt_sha256 and stdin is absent |
| Report | two leaders and one claimed worker | Pass when only the two leaders count |

## Open Questions

(none)

## Out of scope

- The preamble byte budget, model roles, worker read rules.
