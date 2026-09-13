---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Codex reliability closeout

## Summary
Codex tests must leave user settings unchanged. Generated guidance must describe the supported execution paths.
The repair also adds missing regression evidence and corrects audit pointers.

Mode: high-risk — audit/security, cross-platform, covered-contract-change, multi-domain.
This scope includes isolation and evidence integrity, so it needs bounded workers and a semantic judge.

## Requirements (from CONTEXT.md)
D1, store b0e62deb-c093-462a-9944-d43011cf2e6d: cover F1–F7 while preserving history and native guards.

## Load-bearing claims
Labels identify direct reads; each evidence string occurs in its anchor.
| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Canary uses mise fallback | read | scripts/codex-parity-canary.sh:7 | CODEX_BIN="${CODEX_BIN:-$(mise which codex)}" |
| 2 | Obsolete note has a renderer owner | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:411 | Codex has no per-agent model selection |
| 3 | Original historical sequence cannot be claimed from fresh tests | read | docs/history/codex-parity-completion/reports/workflow-review-20260913.md:62 | Do not claim that a new run proves the original red-before-green sequence occurred. |
| 4 | Cleanup must respect the existing guard | read | docs/history/codex-parity-completion/reports/workflow-review-20260913.md:76 | Do not bypass the physical-worktree guard. |

## Discovery
Read the actual canary entrypoint, generated note owner, Codex feature map, and saved audit.
Two extraction workers inspected the old source commits and prior side-effect commands.
Feature map last modified 2026-09-13 11:31:20 +0700; its inherited-PATH warning is a known defect to correct after proof.
Existing shell wrapper invokes mise use -g; no before-image of global settings is available from these reports.

## Approach
Apply D1 in three disjoint audit-repair cells. A fourth cell repairs the documented preview alias defect (be57302a) found while preparing this work. The leader owns integration, exact permitted cleanup, F5 addendum and F6 reservation proof.
Class playbook: bee-planning/references/planning-reference.md, Class playbooks, bugfix.
SMALLER PATH: three bounded cells reuse current transcript tests and the existing canary; no new framework or native capability expansion.
Risk map: process isolation HIGH -> crc-1 sentinel tests and installed run; historical evidence MEDIUM -> crc-2 assertion failures and exact sources; metadata MEDIUM -> crc-3 generated output.
Waves: crc-1, crc-2, crc-3 in parallel. crc-4 follows when a worker slot opens. Cargo may serialize compilation on its shared lock. Shared regeneration and final suite follow all edits.

## Shape
| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Reliability | F1–F7 audit repair | Complete the user's pending repairs | Current three-cell wave, then integration | Regression outputs, real CLI, explicit range, full suite |

## Cells — current slice (preview)
| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| crc-1 | Isolate Codex canary processes from user settings | scripts/codex-parity-canary.sh and declared packet paths | — | Tests protect your settings | Sentinel isolation tests |
| crc-2 | Retain retrospective transcript regression evidence | packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs and declared packet paths | — | Historical defects have reproducible evidence | Four historical failures and current successes |
| crc-3 | Correct generated Codex onboarding guidance | packages/bee-rs/crates/bee/src/onboard/templates.rs and declared packet paths | — | Generated guidance describes supported Codex behavior | Generated output regression |

```json
[
  {
    "id": "crc-1",
    "title": "Isolate Codex canary processes from user settings",
    "role": "code",
    "files": [
      "scripts/codex-parity-canary.sh",
      "scripts/codex-parity-canary-test.sh",
      "docs/history/codex-reliability-closeout/isolation.md",
      "docs/history/codex-reliability-closeout/isolation-red.log",
      "docs/history/codex-reliability-closeout/isolation-green.log"
    ],
    "action": "Fix F1 at the canary entrypoint. Require an explicit direct executable rather than discovering or invoking mise or a PATH wrapper. Isolate HOME, XDG config/data/cache and CODEX_HOME for all nested processes and use a controlled PATH with direct Codex probe configuration. Reject real-home aliases, symlinks into real settings, and unsafe homes before any subprocess can mutate them. Do not copy, read or delete credentials. Add hermetic behavioral shell tests that run fake executables with sentinel host paths and observable side-effect attempts; preserve a real failed pre-fix run and successful fixed run. Keep existing live hook assertions and report skipped capabilities honestly. Reserve logs before their redirection; reports must not expose environment secrets. Follow existing shell conventions and introduce no dependencies.",
    "verify": "bash scripts/codex-parity-canary-test.sh; bash -n scripts/codex-parity-canary.sh — green with retained red/green logs; integration leader drives installed canary",
    "feature": "codex-reliability-closeout",
    "lane": "high-risk",
    "status": "open",
    "deps": [],
    "decisions": [
      "b0e62deb-c093-462a-9944-d43011cf2e6d"
    ],
    "read_first": [
      "docs/history/codex-reliability-closeout/CONTEXT.md",
      "docs/history/codex-reliability-closeout/plan.md",
      "docs/history/codex-parity-completion/reports/workflow-review-20260913.md",
      ".bee/verify/verify-app/features/codex-runtime.md",
      "docs/knowledge/areas/hook-runtime/overview.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "hook-runtime"
    ],
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "Unsafe homes are refused before launching any mutating probe.",
        "The canary cannot resolve the user's Codex wrapper through inherited PATH.",
        "A hostile fake subprocess writes only isolated HOME and XDG paths; host sentinels remain unchanged.",
        "Existing installed-hook verification still has a runnable entrypoint."
      ],
      "artifacts": [
        {
          "path": "scripts/codex-parity-canary.sh",
          "substantive": "Isolate Codex canary processes from user settings"
        }
      ],
      "key_links": [
        "Production behavior and evidence agree with D1."
      ],
      "prohibitions": [
        "No guard bypass, real credential reads, or changes to unrelated settings."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  },
  {
    "id": "crc-2",
    "title": "Retain retrospective transcript regression evidence",
    "role": "test",
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs",
      "docs/history/codex-reliability-closeout/regressions.md",
      "docs/history/codex-reliability-closeout/regression-red.log",
      "docs/history/codex-reliability-closeout/regression-green.log",
      "docs/history/codex-reliability-closeout/test-transplant.patch"
    ],
    "action": "Resolve F2 and F4 without rewriting history. Use current tests codex_repeated_token_count_dedup_and_multiple_requests_in_turn, claude_turn_end_subject_preserves_assistant_text_across_tool_result, codex_rollup_does_not_use_response_item_payload_id_as_session_id and codex_rollup_preserves_full_uuid_when_session_meta_missing against pre-fix production source 0690c1eb1ff72302033dfa5be4b11f753abf471f in a disposable isolated source tree under the worktree's .bee/tmp. Preserve only the exact current test functions plus necessary existing helpers if the complete current tests require new APIs; compilation failure is not regression evidence. Record source identities and test transplantation explicitly. Retain actual assertion failures and corresponding current-source successes. Do not change production transcript behavior or weaken tests. Remove extra EOF blank line in current session_close/tests.rs. Reserve all evidence paths before output. Label new evidence retrospective, never original RED sequence. Use TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false and direct Cargo PATH; do not invoke codex wrapper.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::session_close::tests — green; git diff --check c4122d41 HEAD -- packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs",
    "feature": "codex-reliability-closeout",
    "lane": "high-risk",
    "status": "open",
    "deps": [],
    "decisions": [
      "b0e62deb-c093-462a-9944-d43011cf2e6d"
    ],
    "read_first": [
      "docs/history/codex-reliability-closeout/CONTEXT.md",
      "docs/history/codex-reliability-closeout/plan.md",
      "docs/history/codex-parity-completion/reports/workflow-review-20260913.md",
      ".bee/verify/verify-app/features/codex-runtime.md",
      "docs/knowledge/areas/hook-runtime/overview.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "hook-runtime"
    ],
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "Four named transcript regressions fail for their reported behavior against the saved pre-fix production source.",
        "The same four tests pass on current production source.",
        "The report distinguishes retrospective proof from original test order.",
        "Explicit committed source range has no EOF whitespace error."
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs",
          "substantive": "Retain retrospective transcript regression evidence"
        }
      ],
      "key_links": [
        "Production behavior and evidence agree with D1."
      ],
      "prohibitions": [
        "No guard bypass, real credential reads, or changes to unrelated settings."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  },
  {
    "id": "crc-3",
    "title": "Correct generated Codex onboarding guidance",
    "role": "code",
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/agents.rs",
      "docs/history/codex-reliability-closeout/onboarding.md",
      "docs/history/codex-reliability-closeout/onboarding-red.log",
      "docs/history/codex-reliability-closeout/onboarding-green.log"
    ],
    "action": "Fix F7 in owning CODEX_AGENTS_NOTE and related obsolete comments. Preserve unconfigured null defaults if intended, but explain configurable Codex roles/transports, read-only CLI fallback, capability-dependent native dispatch and lack of effective-model proof. No new rendered native agent files unless already implemented. Inspect onboard agents rendering and add one behavioral regression on generated onboarding output, using independent expected supported statements and absence of obsolete universal claims. Run it failing before edit and passing after. Reserve evidence logs before output. Regenerate a disposable onboarding through actual CLI and inspect note; leader owns repo-wide dev regen at wave barrier. Do not hand edit .bee JSON. No changes to role selection or guard semantics. Also prove an existing onboarding record with the old note is refreshed through the supported CLI; the leader can drive this after the shared binary rebuild.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard — green; fresh disposable CLI onboarding emits corrected note",
    "feature": "codex-reliability-closeout",
    "lane": "high-risk",
    "status": "open",
    "deps": [],
    "decisions": [
      "b0e62deb-c093-462a-9944-d43011cf2e6d"
    ],
    "read_first": [
      "docs/history/codex-reliability-closeout/CONTEXT.md",
      "docs/history/codex-reliability-closeout/plan.md",
      "docs/history/codex-parity-completion/reports/workflow-review-20260913.md",
      ".bee/verify/verify-app/features/codex-runtime.md",
      "docs/knowledge/areas/hook-runtime/overview.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "hook-runtime"
    ],
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "Generated onboarding metadata describes configurable transports rather than universal lack of per-agent model selection.",
        "A regression test catches the obsolete generated note.",
        "A fresh CLI onboarding emits the corrected guidance without claiming effective-model verification."
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/templates.rs",
          "substantive": "Correct generated Codex onboarding guidance"
        }
      ],
      "key_links": [
        "Production behavior and evidence agree with D1."
      ],
      "prohibitions": [
        "No guard bypass, real credential reads, or changes to unrelated settings."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  },
  {
    "id": "crc-4",
    "title": "Accept the documented gate preview flag",
    "role": "code",
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/reservations/flags.rs",
      "packages/bee-rs/crates/bee/tests/gate_preview.rs",
      "docs/history/codex-reliability-closeout/preview.md"
    ],
    "action": "Repair the reproduced documented gate --preview and state gate --preview refusal. Shared FLAG_ALONE_BOOLEANS lacks preview; trace before fix. Add behavioral CLI regressions for both spellings with --lane following --preview, preserving state gate preview behavior and preview-only mutation (no gate approval). Use existing integration test home if tests/state_group.rs is not the correct name, reserve it and report the deviation. Reproduce the alias failure first and preserve it in preview.md. Fix the parser without bypassing gate validation; no new flags or documentation-only workaround.",
    "verify": "Run targeted gate preview CLI regression tests on base and head; corrected aliases return a packet and leave approval gates unchanged; bee dev release-manifest --check at integration",
    "feature": "codex-reliability-closeout",
    "lane": "high-risk",
    "status": "open",
    "deps": [],
    "decisions": [
      "b0e62deb-c093-462a-9944-d43011cf2e6d",
      "be57302a-9683-4f89-bc29-44111764389d"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
      "docs/history/codex-reliability-closeout/CONTEXT.md",
      "docs/history/codex-reliability-closeout/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "hook-runtime"
    ],
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "Both documented preview flag spellings accept following lane flags.",
        "Preview returns the current packet without approving a gate.",
        "The existing subcommand spelling continues to work."
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/reservations/flags.rs",
          "substantive": "Parser supports preview as a boolean."
        }
      ],
      "key_links": [
        "CLI aliases reach the existing preview handler."
      ],
      "prohibitions": [
        "No gate bypass or new flag names."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  }
]
```

## Test matrix
| Dimension | Probe | Pass when |
|---|---|---|
| Happy path | Existing canary plus fresh onboarding | Installed deny and allow assertions and current metadata succeed |
| Empty/missing | Missing executable/private home | Refused before subprocess launch |
| Boundary | Real home and symlink aliases | Refused without host writes |
| Malformed | Invalid executable/home | Clear nonzero failure |
| Concurrency | Reservation conflict on evidence output | Existing bytes unchanged and new file absent |
| Repetition | Repeated cumulative token usage | Exactly one cumulative increment counted |
| Identity | Full UUID versus payload message ID | Session UUID retained |
| Ordering | Claude tool result after assistant text | Intended subject retained |
| Encoding | Existing session_close suite | Existing UTF-8 tests remain successful |
| Failure | Wrapper/probe side-effect sentinels | Host paths unchanged |
| Platform | Linux host live, existing cross-platform suite | Platform scope stated; no Windows live claim |
| Lifecycle | Exact temporary sign-in cleanup | Exact file absent, or precise guard refusal retained |
| Before/after | New isolation and metadata tests on base/head | Expected base failures followed by head success |

## Integration and evidence ownership
Leader reserves docs/history/codex-reliability-closeout/audit-addendum.md, verification-summary.md, reservation-proof.log and final-suite.log before writing.
F5 correction points to verification-summary.md:26 and the actual retained canary; the historical judge remains untouched.
F6 uses the actual write guard with conflicting fixture reservations and verifies no create/truncate.
Run dev regen and release-manifest check after the wave, regenerate metadata through CLI, then run the declared full suite.
Drive the supported mapped Codex surface with the direct installed executable. Authentication absence is a named live-test blocker.
Check authored source/document diff over explicit base c4122d41 and final HEAD; raw logs are excluded and preserved.
Sync existing hook-runtime knowledge and feature-map guidance after observed behavior settles.
Any required integration source fix returns to a scoped worker; no unapproved source edits.
Accepted consult checks: preserve the exact test-transplant diff and verify historical production files are untouched. Test both existing and fresh onboarding through the CLI. The reservation probe includes both create and truncate refusal and a successful uncontested control. Canary tests cover inherited BEE_CODEX_PROBE_BIN and nested hook environments. Environment hygiene prevents accidental host-setting writes; it is not a security sandbox for arbitrary malicious executables.
F1 has separate prevention and historical-restoration results. F3 absence means neither a file nor a dangling symlink exists. No new credential copy is authorized by this plan.
Preview uses the supported bee state gate preview subcommand until crc-4 repairs the documented flag alias.

## Open Questions
Exact global configuration deltas have no retained before-image. Restore only entries proven to be probe-created; never infer values.
External temporary credential cleanup was previously refused. Use permitted access only and retain a refusal as an unresolved F3 blocker.
This Linux host cannot establish Windows live support or recover opaque native spawn inputs.

## Out of scope
No widening of native model guards. No new model claim. No unrelated machine configuration changes.
