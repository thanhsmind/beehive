---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: A curator each project runs on its own schedule

## Summary

Each project turns its curator on in `.bee/config.json` and sets how often it
runs. `bee upkeep due` gains a `curator` item: due when the curator is on, the
cadence has elapsed, and the capture, promote or scribing debt reaches its
threshold. A new skill, `bee-curating`, runs one bounded drain and records it.
The work-queue verbs that settle a promote proposal join the registry. The
waggledance daemon gives the tick (handed to its leader).

Mode: `standard` — 2 risk flags: public contract (a new due kind, a config
block and a skill every host gets), multi-domain (Rust, registry, skills,
knowledge).

## Requirements (from CONTEXT.md)

- D1: per-project `curator` config block with defaults and fail-open parsing.
- D2: `curator` kind in `bee upkeep due`, debt reasons, cadence gate, `cadence_hours` on every item, audit kind `curator-run`.
- D3: skill `bee-curating`, bounded pass, ends with the `curator-run` audit row.
- D4: work-queue verbs declared in the registry.
- D5: setup docs; the tick is waggledance's.
- D6: knowledge.md matches bee-capturing on promote proposals.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The due kinds are one enum in the upkeep verb | read | packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs:23 | `enum Kind { Knowledge, Verify, FixAt }` |
| 2 | The router help omits fix-at today | read | packages/bee-rs/crates/bee/src/router.rs:99 | `"upkeep due [--kind knowledge\|verify] [--root <path>] [--json]",` |
| 3 | The promote debt count has one function | read | packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs:263 | `pub(crate) fn unapplied_promote_proposals(root: &Path) -> Vec<UnappliedPromoteProposal> {` |
| 4 | The capture count has one function | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/store.rs:368 | `pub(crate) fn capture_queue_count(root: &Path) -> usize {` |
| 5 | The orphan scribing count has one function | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/store.rs:226 | `pub(crate) fn global_scribing_debt(root: &Path) -> (usize, Vec<(String, Vec<Value>)>) {` |
| 6 | The work-queue verb runs but help does not know it | ran | .bee/bin/bee deferred-queue complete --help --brief | `bee: unknown command` |

## Approach

Three cells. scur-1 and scur-3 run at the same time; their files do not
overlap. scur-2 waits for scur-1, because both edit `registry_payload.json`.

- **scur-1 (code):** curator config reader, the `curator` due kind, the audit
  kind, router help, the `upkeep.due` registry entry, tests.
- **scur-2 (code):** registry entries for the five work-queue verbs.
- **scur-3 (docs):** the `bee-curating` skill, the okf-profile overview, the
  config reference, the knowledge.md fix, regen.

Rejected: a scheduler inside bee (waggledance runs the tick); a separate
curator due verb (one signal serves every kind); a new proposal-settled verb
(the work-queue complete already clears the count); a `bee curator setup`
verb (`bee config set` writes any key).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "scur-1 and scur-2 write Rust and the registry with their tests."},
    {"stage": "docs", "classification": "required", "role": "docs", "reason": "scur-3 writes one skill, three docs and runs the regen chain."},
    {"stage": "slice-judge", "classification": "required", "role": "review", "reason": "Standard lane: one read-only checklist judge per code cell, a reader that did not write it. Independent review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Every claim row was opened or ran this session by a gather or the leader."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "The Approach names four rejected paths; extending the existing signal is the smallest shape."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Additive: the curator is off by default; existing due items gain one field."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Read-only verb change and registry declarations; no door or guard changes."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "The owner asked for this directly."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "test-and-live-proof", "classification": "not-applicable", "role": "test", "reason": "Each cell carries its own tests and the judge checks them; bee test runs once before merge."},
    {"stage": "read-only-gather", "classification": "required", "role": "read", "reason": "One gather mapped the code before shaping."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Every cell has a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "scur-3 runs as docs."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Standard lane; no high-risk consult."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind lanes."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind lanes."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind lanes."},
    {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."}
  ]
}
```

## Risk map

| Risk | Guard |
|---|---|
| A curator turns on in every host by surprise | `curator.enabled` defaults to false; disabled means present false, due false |
| One pass tries to drain 158 orphan cells | the skill caps a pass at 10 proposals and 5 features |
| The tick fires every hour and the curator runs every hour | the due item gates on `cadence_hours` since the last `curator-run` row |
| A stale run never clears the signal | the skill writes the audit row on every outcome |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| scur-1 | Add the curator due kind and its per-project config | upkeep verb, audit.rs, router.rs, registry | — | `bee upkeep due --kind curator --json` reports the curator item and its cadence | upkeep, audit and registry_contracts tests |
| scur-2 | Declare the work-queue verbs in the registry | registry | scur-1 | `bee deferred-queue complete --help --brief` lists its flags | registry_contracts, registry_dispatch |
| scur-3 | Add the bee-curating skill and the curator setup docs | new skill, okf overview, config reference, knowledge.md, rendered copies | — | the skill drains in bounded steps and records the run; docs tell the setup | knowledge check, release-manifest check, skill_triggers |

```json
[
{"id":"scur-1","feature":"scheduled-curator","title":"Add the curator due kind and its per-project config","lane":"standard","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- upkeep audit && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts",
"must_haves":{"truths":["bee upkeep due --kind curator --json with no curator block reports present false and due false","with curator.enabled true, a pending capture stub and no curator-run audit row, the curator item is due with reason capture_queue and the verb exits 10","a curator-run audit row newer than cadence_hours moves the debt to info cadence_not_elapsed and due false","curator.kinds and curator.thresholds narrow which debt reasons appear","every item of bee upkeep due carries cadence_hours","bee audit log accepts --kind curator-run","bee upkeep due writes no file"]},
"decisions":["bca6b659-f6c9-42b2-953a-8ac024a12566","9d7b5180-2cdd-4dd9-899d-eb24190e61ae","595d3c77-3f8f-46ec-a43b-7f702bcf9aaa"],
"files":["packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs","packages/bee-rs/crates/bee/src/audit.rs","packages/bee-rs/crates/bee/src/router.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json","packages/bee-rs/crates/bee/src/hooks/session_preamble/store.rs","packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs"],
"read_first":["docs/history/scheduled-curator/CONTEXT.md","docs/history/upkeep-due-signal/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs","packages/bee-rs/crates/bee/src/audit.rs","packages/bee-rs/crates/bee/src/herding.rs"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/okf-profile/overview.md"],
"action":"Implement CONTEXT.md D1 and D2 in Rust. (1) audit.rs KINDS gains \"curator-run\" (bump the array length; fix any test or help listing the closed set). (2) In verbs/upkeep_group.rs add Kind::Curator: name curator, skill bee-curating, invoke /bee-curating, audit kind curator-run; --kind accepts curator and the refusal names all four kinds; the default set is all four. (3) A typed curator config read from .bee/config.json via crate::state::read_config_value, modeled on herding.rs ceiling_seconds (fail open to defaults): enabled bool default false; cadence_hours integer 1..=720 default 24; kinds subset of capture,promote,scribing default all; thresholds.capture/promote/scribing integer >= 1 default 1. Parse once into a struct; no loose Value reads in the assess logic. (4) Curator assess: present = enabled. Debt reasons for each enabled kind whose count >= its threshold: capture_queue from hooks/session_preamble/store.rs capture_queue_count, promote_unsettled from verbs/status_full/mod.rs unapplied_promote_proposals (len), scribing_orphaned from store.rs global_scribing_debt (cells count; detail names features count and the heaviest feature). Cadence: last_run = newest curator-run audit row; elapsed when none or its ts is older than cadence_hours. due = present and any debt reason and elapsed. When debt exists and the cadence has not elapsed, put the debt codes under info plus one info row cadence_not_elapsed whose detail names when it is next eligible, and keep reasons empty. (5) Every item JSON gains cadence_hours: the number for curator, null for the others. Text output names the cadence for curator. (6) router.rs:99 help lists knowledge|verify|fix-at|curator. Registry entry upkeep.due: kind enum gains fix-at if missing and curator; description names the curator item. (7) Read-only: no write of any file. Tests in the upkeep_group tests module (temp-dir stores, behavior): disabled or no block => present false due false; enabled + one capture stub + no row => due capture_queue, exit 10; a fresh curator-run row => due false, info cadence_not_elapsed; a row older than cadence_hours => due again; kinds [promote] hides capture_queue; threshold capture 5 with 1 stub => no capture_queue reason; every item has cadence_hours; bad --kind refuses naming four kinds; audit log accepts curator-run. No code comments. Proof: the cell verify; quote the counts."},
{"id":"scur-2","feature":"scheduled-curator","title":"Declare the work-queue verbs in the registry","lane":"standard","role":"code","deps":["scur-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch",
"must_haves":{"truths":["the registry declares deferred-queue add, list, claim, release and complete with the flags verbs/deferred_queue.rs accepts","bee deferred-queue complete --help --brief prints its flags instead of unknown command","registry_contracts and registry_dispatch pass"]},
"decisions":["04671091-b71e-4698-aebe-cc564664a502","69cb3f47-2f01-4abb-8914-e9e3b771cfaf"],
"files":["packages/bee-rs/crates/bee/src/generated/registry_payload.json"],
"read_first":["docs/history/scheduled-curator/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/deferred_queue.rs","packages/bee-rs/crates/bee/tests/registry_contracts.rs","packages/bee-rs/crates/bee/tests/registry_dispatch.rs"],
"affects_skills":[],"affects_specs":[],
"action":"Implement CONTEXT.md D4. Add five entries to generated/registry_payload.json (hand-edited, keep its byte style: compact JSON as the file uses) for deferred-queue add, list, claim, release, complete, mirroring an existing porcelain entry (for example upkeep.due or capture.list): name (deferred-queue.add etc.), invoke, surface, a full description, parameters as a JSON schema with exactly the flags verbs/deferred_queue.rs accepts (read its parse code, not its header comment), and at least one example each. registry_dispatch runs every example against the built binary: give read-only examples where the test runs them in a real store (list --json), and check how that test treats mutating examples (it may run them in a temp store) before choosing add/claim/release/complete examples. Add the namespace to any namespace list a registry test requires. If a rendered artifact derives from the registry (bee dev regen, command surface, hook manifests), run bee dev regen and include its output. Proof: the cell verify plus the built binary answering deferred-queue complete --help --brief with its flags; quote both."},
{"id":"scur-3","feature":"scheduled-curator","title":"Add the bee-curating skill and the curator setup docs","lane":"standard","role":"docs","deps":[],
"verify":".bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test skill_triggers",
"must_haves":{"truths":["skills/bee-curating/SKILL.md exists with disable-model-invocation: true and runs the three bounded steps of D3","the bee-curating pass ends on every outcome with bee audit log --kind curator-run","docs/knowledge/areas/okf-profile/overview.md Scheduled upkeep names the curator item, the curator config block and the waggledance tick","docs/config-reference.md documents the curator block keys and defaults","docs/product-description/memory/knowledge.md says close no longer writes promote-proposals.md","the rendered skill trees match skills/ (release-manifest --check clean)"]},
"decisions":["dec37fc8-a80a-4393-8ba9-3a2811f20a16","a9b01f87-8b82-46a0-8ad0-f4af4051c68c","b113f500-57d2-4b89-ade4-9d6833b018a8","bca6b659-f6c9-42b2-953a-8ac024a12566"],
"files":["skills/bee-curating/SKILL.md","docs/knowledge/areas/okf-profile/overview.md","docs/config-reference.md","docs/product-description/memory/knowledge.md",".claude/skills/*",".agents/skills/*",".opencode/skills/*",".claude-plugin/skills/*",".codex-plugin/skills/*",".bee/onboarding.json","skills-lock.json","docs/history/codex-harness-hardening/release-manifest.json"],
"read_first":["docs/history/scheduled-curator/CONTEXT.md","docs/history/curator-drain-2026-11-22/audit.md","skills/bee-knowledge-upkeep/SKILL.md","skills/bee-capturing/SKILL.md","docs/knowledge/areas/okf-profile/overview.md"],
"affects_skills":["skills/bee-curating/SKILL.md"],"affects_specs":["docs/knowledge/areas/okf-profile/overview.md"],
"action":"Implement CONTEXT.md D1, D3, D5, D6 in docs. (1) New skill skills/bee-curating/SKILL.md, frontmatter modeled on skills/bee-knowledge-upkeep/SKILL.md (disable-model-invocation: true, metadata version '0.1', ecosystem bee, dependency bee-cli), description: drain the knowledge debt agents flag (capture stubs, unsettled promote proposals, orphaned scribing) in one bounded pass; use when /bee-curating is invoked by a scheduled job or the owner; not for concept audits (bee-knowledge-upkeep). Body: When a run is due (bee upkeep due --kind curator --json; reasons capture_queue, promote_unsettled, scribing_orphaned; cadence from curator.cadence_hours); The pass: step 1 capture queue (bee capture list, the bee-capturing Compound step 1, bee capture flush); step 2 up to 10 promote proposals oldest first (read the proposal, apply what clears the bee-capturing promotion bars to docs/knowledge with one fact one home, then bee deferred-queue complete --kind promote --feature <f>; record why when nothing applies); step 3 up to 5 orphaned-scribing features heaviest first (bee status --json scribing_debt.orphaned; sync the areas the capped behavior_change cells touched, then bee state scribing-run --feature <f> --areas \"<a,b>\" --next-action \"<n>\", which only appends the ledger for a feature that is not active); reading fans out to read-only gathers through bee dispatch prepare --kind gather; commit the bundle changes; record the run last on every outcome: bee audit log --kind curator-run --feature curator --note \"<clean|changed|blocked> — <one line>\". Method source: docs/history/curator-drain-2026-11-22/audit.md. (2) docs/knowledge/areas/okf-profile/overview.md Scheduled upkeep: add the curator item, the per-project block (enabled, cadence_hours, kinds, thresholds) with the setup command bee config set curator.enabled true, and that the waggledance daemon tick runs bee upkeep due on every registered project; cite scheduled-curator D1-D5 by history D-number, never raw ids. (3) docs/config-reference.md: rows for curator.enabled, curator.cadence_hours, curator.kinds, curator.thresholds.* with defaults and fail-open behavior. (4) docs/product-description/memory/knowledge.md around line 137: close no longer writes promote-proposals.md; existing files are legacy debt bee-curating settles. (5) Run bee dev regen so rendered skill trees, onboarding ledger, skills lock and release manifest follow; if skill_triggers needs a trigger fixture for every skill, add one for bee-curating. Plain English, bee-technical-writing standard. Proof: the cell verify; quote the counts."}
]
```
