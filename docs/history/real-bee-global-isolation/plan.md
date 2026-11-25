---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: real-bee-global-isolation

## Summary

Make the `real_bee` test helper in `pi_plugin_contracts.rs` isolate the
user-global config, so the release suite does not go red on a host whose
global config routes `team.claude` roles to herding agents.

Mode: `small` — 0 risk flags: none (one test file changes; no product code)
Why this is the least workflow that protects the work: one test helper, one
implementer and one independent tester.

Playbook: `skills/bee-planning/playbooks/bugfix.md`, situation `skills/bee-planning/playbooks/bugfix/bug-fix.md`.

## Requirements (from CONTEXT.md)

- D1 (`20732f48`): `real_bee` sets `BEEHIVE_CONFIG_DIR` to an empty folder.
- C1 (`77dff9a0`): a contract test proves the isolation against a present global config.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The non-test build reads BEEHIVE_CONFIG_DIR first | read | packages/bee-rs/crates/bee/src/config_layers.rs:78 | `if let Some(dir) = env_nonempty("BEEHIVE_CONFIG_DIR") {` |
| 2 | real_bee does not isolate the global config | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:4188 | `fn real_bee(bin: &Path) -> Command {` |
| 3 | The release suite failed on that test | ran | `bash scripts/release.sh 2.123.0` | `default dispatch prepare must select Claude Agent tool` |

## Cells (current slice)

```json
[
  {
    "id": "rbgi-1",
    "feature": "real-bee-global-isolation",
    "title": "Isolate the user-global config in the real_bee test helper",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "20732f48-e507-4ec4-b460-c64499870a95"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/real-bee-global-isolation/CONTEXT.md",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/src/config_layers.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs, change the real_bee helper so every bee child it starts gets BEEHIVE_CONFIG_DIR set to an empty folder that the helper creates (for example a folder next to the copied binary, created with create_dir_all). Keep the two env_remove calls. Follow the same pattern as opencode_plugin_contracts.rs, which sets BEEHIVE_CONFIG_DIR to an empty dir. Write no code comments. Run only the cell verify filter, never the full suite. Bee commands you may run: none besides the cell cap your brief names.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts -- real_bee injected_dispatch_guidance",
    "must_haves": {
      "truths": [
        "real_bee sets BEEHIVE_CONFIG_DIR to an existing empty folder for each child",
        "injected_dispatch_guidance_extracts_and_executes_pi_runtime_herding passes on a host whose global config routes team.claude to herding agents"
      ],
      "prohibitions": [
        "No edit outside packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
        "No code comments"
      ]
    },
    "behavior_change": false
  },
  {
    "id": "rbgi-1t",
    "feature": "real-bee-global-isolation",
    "title": "Prove real_bee children ignore a user-global team config",
    "lane": "small",
    "role": "test",
    "deps": [
      "rbgi-1"
    ],
    "decisions": [
      "20732f48-e507-4ec4-b460-c64499870a95",
      "77dff9a0-f52d-4ab5-bbd5-4dedef88eaf4"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/real-bee-global-isolation/CONTEXT.md",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Verify rbgi-1 independently, then add one test to packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs whose name starts with real_bee_ (C1). The test writes a user-global config.json that sets team.claude.extraction to kind herding with an agent, under a temp folder, and starts a fixture bee through real_bee with XDG_CONFIG_HOME and HOME pointed at that temp folder (so the global layer would load if real_bee did not isolate it). It runs dispatch prepare --runtime claude --kind gather --role extraction --json in a fixture whose tracked config sets team.claude.extraction model haiku, and asserts tool Agent and payload.model haiku. Watch it fail with the BEEHIVE_CONFIG_DIR line of real_bee removed, then restore it. Write no code comments. Run only the cell verify filter, never the full suite. Bee commands you may run: none besides the cell cap your brief names.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts -- real_bee injected_dispatch_guidance",
    "must_haves": {
      "truths": [
        "a real_bee_ test sets a herding team.claude config in a user-global folder and still gets tool Agent",
        "the new test fails when real_bee does not set BEEHIVE_CONFIG_DIR"
      ],
      "prohibitions": [
        "No edit outside packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
        "No code comments"
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
      "reason": "No release in this lane; release-2-123-0 runs after the merge."
    },
    {
      "stage": "planning",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "The plan is this one page and needs no planning dispatch."
    },
    {
      "stage": "implementation",
      "classification": "required",
      "role": "code",
      "reason": "rbgi-1 changes the real_bee test helper."
    },
    {
      "stage": "test-and-live-proof",
      "classification": "required",
      "role": "test",
      "reason": "rbgi-1t independently verifies the fix and adds the C1 contract test."
    },
    {
      "stage": "documentation-and-capture",
      "classification": "not-applicable",
      "role": "docs",
      "reason": "No documentation changes ride this fix."
    },
    {
      "stage": "read-only-gather",
      "classification": "not-applicable",
      "role": "read",
      "reason": "Nothing to survey; the preconditions are the script's own."
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
      "reason": "No high-risk consult is owed for a test-helper fix."
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
      "reason": "No competing designs; there is one sanctioned command."
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
      "reason": "No rendered UI changes ride this fix."
    },
    {
      "stage": "hat-facts-gaps",
      "classification": "not-applicable",
      "role": "hat-facts-gaps",
      "reason": "Small lane: no hat wave."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Small lane: no hat wave."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Small lane: no hat wave."
    },
    {
      "stage": "hat-alternatives",
      "classification": "not-applicable",
      "role": "hat-alternatives",
      "reason": "Small lane: no hat wave."
    },
    {
      "stage": "hat-user-impact",
      "classification": "not-applicable",
      "role": "hat-user-impact",
      "reason": "Small lane: no hat wave."
    },
    {
      "stage": "writing",
      "classification": "not-applicable",
      "role": "writer",
      "reason": "No prose is written for this fix."
    }
  ]
}
```

## Proof

- The cell filter green for both cells; the full suite once via `bee test` before merge.
