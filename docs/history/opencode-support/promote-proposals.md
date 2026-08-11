promote proposal for work item "opencode-support" (docs/history/opencode-support/CONTEXT.md + docs/history/opencode-support/plan.md) — 15 capped cell(s): oc-1, oc-2, oc-3, oc-4, oc-5, oc-6, oc-7, oc-8, oc-9, oc-10, oc-11, oc-12, oc-13, oc-14, oc-15
anchor: history — docs/history/opencode-support/CONTEXT.md, docs/history/opencode-support/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/opencode-support/delivery.md

---
type: bee.delivery
title: opencode-support — delivery
description: "Delivery record proposed by bee knowledge promote for work item opencode-support: 15 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: opencode-support-delivery
  lifecycle: active
  areas: [hook-runtime, onboarding]
  required_context: [docs/history/opencode-support/CONTEXT.md, docs/history/opencode-support/plan.md]
  sources: [docs/history/opencode-support/CONTEXT.md, docs/history/opencode-support/plan.md, .bee/cells/oc-1.json, .bee/cells/oc-2.json, .bee/cells/oc-3.json, .bee/cells/oc-4.json, .bee/cells/oc-5.json, .bee/cells/oc-6.json, .bee/cells/oc-7.json, .bee/cells/oc-8.json, .bee/cells/oc-9.json, .bee/cells/oc-10.json, .bee/cells/oc-11.json, .bee/cells/oc-12.json, .bee/cells/oc-13.json, .bee/cells/oc-14.json, .bee/cells/oc-15.json]
---

# opencode-support — Delivery

## What shipped

- **oc-1** — Installed opencode-ai@1.18.16 via npm, confirmed no third-party provider auth but a live opencode/* free-model session works, and verified plugin/skill/agent on-disk layout via a scratch probe (1 file(s) changed)
- **oc-2** — Re-verified after remediation by oc-3 (apply_patch mapped) and oc-9 (registry gate holds it shut) (2 file(s) changed)
- **oc-3** — Mapped apply_patch to bee write-guard in bee-guard.ts; recorded the write-capable tool registry and corrected discovery.md wording (2 file(s) changed)
- **oc-4** — Taught both skill-render pipelines the opencode runtime: opencode joins the marker grammar in both render sites, skill_trees.rs's target-dir pick is now an exhaustive mapping that refuses unknown runtimes, and .opencode/skills/ is rendered in this checkout with its bee-render/2 sidecar. (6 file(s) changed)
- **oc-5** — opencode accepted in parse_runtime, merge-plugin-state's --opencode flag, and mod.rs's runtime-label parsing; hook_manifests' Runtime enum keeps a named R1 exclusion (4 file(s) changed)
- **oc-6** — Re-verified after remediation by oc-8 (exit-0 repair and ask honored per D6) and oc-9 (fixtures assert those paths) (2 file(s) changed)
- **oc-7** — Re-verified after remediation by oc-9 (skip fails closed, payloads asserted) and oc-10 (the gate's first catch closed) (2 file(s) changed)
- **oc-8** — Applied D6: exit-0 updatedInput repairs now land in output.args, permissionDecision ask throws with bee's reason, additionalContext is logged not dropped, unparseable exit-0 stdout throws fail-closed; chat.message's output.message.id dereference wrapped in try/catch (F6). Live-proved via direct plugin invocation against a stubbed bee binary (all 5 cases). oc-9's opencode_plugin_contracts suite still 4 passed, unmodified and unaffected. (2 file(s) changed)
- **oc-9** — Closed F1/F3/F4/F5 in the OpenCode parity suite: fail-by-default (opt-out env var) on missing node/opencode capability, exact per-row payload+D6 verdict assertions, binary-derived tool-registry coverage gate (caught real unmapped lsp tool by name), and a properly line-scoped named-gap check; plugin left unchanged, lsp/list recorded as named gaps in discovery.md (2 file(s) changed)
- **oc-10** — lsp and list mapped through write-guard; fixture rows added; registry gate green (3 file(s) changed)
- **oc-11** — models.opencode is now a real, resolvable config key (3 readers widened + tests); .opencode/agent/bee-{build,gather,extract,review}.md hand-authored and live-verified against opencode 1.18.16, each pinning a free-tier model with mode: subagent and write-denying permission for the three read-only agents. (8 file(s) changed)
- **oc-12** — proved E4 live: a bee-build subagent capped a real cell (lv-1) from inside a nested OpenCode task-tool dispatch, after naming two real gaps (reservation session-id mismatch on nested dispatch; --agent flag not reaching subagents) (1 file(s) changed)
- **oc-13** — bee onboard --apply now vendors .opencode/skills/ and .opencode/plugins/bee-guard.ts idempotently (live-proved by re-applying against this repo: up_to_date on the second pass, plus two new fixture tests); status_full's RUNTIMES/normalize_models/agent-drift check and plugin_distribution's --runtime opencode branch widen to opencode with correct semantics; docs/06-runtime-integration.md rewritten 2->3 runtimes, fixing the stale bee-render/1 string and the deleted-catalog.mjs pointer in passing. Found and fixed a real pre-existing gap: the committed .opencode/skills/ tree (from oc-4's interim regen path) had no .bee-skills-version.json stamp, which would have permanently blocked every future bee onboard --apply against this repo. (16 file(s) changed)
- **oc-14** — Release manifest covers .opencode/plugins/, onboarding renders the four .opencode/agent worker files, docs 01/02/06 name three runtimes with no surviving Node-era claim (13 file(s) changed)
- **oc-15** — Merged the twelve capped opencode-support behavior_change cells' evidence into the hook-runtime catalog-projections-and-activation.md concept (three-runtime doctrine, the plugin belt as a named R1 difference, throw-based blocking, exit-0 repair/ask honoring, the tool-registry coverage gate, fail-not-skip proof discipline, and Open Gaps for the model-guard claude-literal check, the CLI subagent-selection fallback, and nested-dispatch reservation identity) and, deviating to the better-fit onboarding/repo-local-guardrails.md (recorded reason: installer-entrypoints-and-source-staging.md's R20b/R23/R27 govern bootstrap staging this feature never touched), added the third-runtime guard-belt vendoring behavior and the third models config key (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **oc-1** — `opencode --version`
- **oc-2** — `rg -n "tool.execute.before" .opencode/plugins/bee-guard.ts`
- **oc-3** — `rg -n "apply_patch" .opencode/plugins/bee-guard.ts`
- **oc-4** — `rtk proxy cat .opencode/skills/.bee-render.json`
- **oc-5** — `rg -n "opencode" packages/bee-rs/crates/bee/src/devtools/plugin_distribution.rs packages/bee-rs/crates/bee/src/devtools/install_support.rs`
- **oc-6** — `rg -n "chat.message|tool.execute.after|session.idle" .opencode/plugins/bee-guard.ts`
- **oc-7** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts`
- **oc-8** — `rg -n "updatedInput|permissionDecision" .opencode/plugins/bee-guard.ts`
- **oc-9** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts`
- **oc-10** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts`
- **oc-11** — `rg -n "^model:|^mode:" .opencode/agent/bee-build.md .opencode/agent/bee-gather.md .opencode/agent/bee-extract.md .opencode/agent/bee-review.md`
- **oc-12** — `rg -n "capped|cells finish" docs/history/opencode-support/discovery.md`
- **oc-13** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **oc-14** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **oc-15** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work opencode-support` from 15 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/opencode-support/CONTEXT.md`, `docs/history/opencode-support/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "opencode-support" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T19:03:30.547Z), the work item declares no bee.areas.

area hook-runtime:
  - [oc-2] Re-verified after remediation by oc-3 (apply_patch mapped) and oc-9 (registry gate holds it shut) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-2.json)
  - [oc-3] Mapped apply_patch to bee write-guard in bee-guard.ts; recorded the write-capable tool registry and corrected discovery.md wording — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-3.json)
  - [oc-4] Taught both skill-render pipelines the opencode runtime: opencode joins the marker grammar in both render sites, skill_trees.rs's target-dir pick is now an exhaustive mapping that refuses unknown runtimes, and .opencode/skills/ is rendered in this checkout with its bee-render/2 sidecar. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/oc-4.json)
  - [oc-5] opencode accepted in parse_runtime, merge-plugin-state's --opencode flag, and mod.rs's runtime-label parsing; hook_manifests' Runtime enum keeps a named R1 exclusion — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/oc-5.json)
  - [oc-6] Re-verified after remediation by oc-8 (exit-0 repair and ask honored per D6) and oc-9 (fixtures assert those paths) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-6.json)
  - [oc-7] Re-verified after remediation by oc-9 (skip fails closed, payloads asserted) and oc-10 (the gate's first catch closed) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-7.json)
  - [oc-8] Applied D6: exit-0 updatedInput repairs now land in output.args, permissionDecision ask throws with bee's reason, additionalContext is logged not dropped, unparseable exit-0 stdout throws fail-closed; chat.message's output.message.id dereference wrapped in try/catch (F6). Live-proved via direct plugin invocation against a stubbed bee binary (all 5 cases). oc-9's opencode_plugin_contracts suite still 4 passed, unmodified and unaffected. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-8.json)
  - [oc-9] Closed F1/F3/F4/F5 in the OpenCode parity suite: fail-by-default (opt-out env var) on missing node/opencode capability, exact per-row payload+D6 verdict assertions, binary-derived tool-registry coverage gate (caught real unmapped lsp tool by name), and a properly line-scoped named-gap check; plugin left unchanged, lsp/list recorded as named gaps in discovery.md — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-9.json)
  - [oc-10] lsp and list mapped through write-guard; fixture rows added; registry gate green — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/oc-10.json)
  - [oc-11] models.opencode is now a real, resolvable config key (3 readers widened + tests); .opencode/agent/bee-{build,gather,extract,review}.md hand-authored and live-verified against opencode 1.18.16, each pinning a free-tier model with mode: subagent and write-denying permission for the three read-only agents. — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/oc-11.json)
  - [oc-13] bee onboard --apply now vendors .opencode/skills/ and .opencode/plugins/bee-guard.ts idempotently (live-proved by re-applying against this repo: up_to_date on the second pass, plus two new fixture tests); status_full's RUNTIMES/normalize_models/agent-drift check and plugin_distribution's --runtime opencode branch widen to opencode with correct semantics; docs/06-runtime-integration.md rewritten 2->3 runtimes, fixing the stale bee-render/1 string and the deleted-catalog.mjs pointer in passing. Found and fixed a real pre-existing gap: the committed .opencode/skills/ tree (from oc-4's interim regen path) had no .bee-skills-version.json stamp, which would have permanently blocked every future bee onboard --apply against this repo. — feature-wide sync per the scribing stamp, 16 file(s) changed (trace .bee/cells/oc-13.json)
  - [oc-14] Release manifest covers .opencode/plugins/, onboarding renders the four .opencode/agent worker files, docs 01/02/06 name three runtimes with no surviving Node-era claim — feature-wide sync per the scribing stamp, 13 file(s) changed (trace .bee/cells/oc-14.json)

area onboarding:
  - [oc-2] Re-verified after remediation by oc-3 (apply_patch mapped) and oc-9 (registry gate holds it shut) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-2.json)
  - [oc-3] Mapped apply_patch to bee write-guard in bee-guard.ts; recorded the write-capable tool registry and corrected discovery.md wording — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-3.json)
  - [oc-4] Taught both skill-render pipelines the opencode runtime: opencode joins the marker grammar in both render sites, skill_trees.rs's target-dir pick is now an exhaustive mapping that refuses unknown runtimes, and .opencode/skills/ is rendered in this checkout with its bee-render/2 sidecar. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/oc-4.json)
  - [oc-5] opencode accepted in parse_runtime, merge-plugin-state's --opencode flag, and mod.rs's runtime-label parsing; hook_manifests' Runtime enum keeps a named R1 exclusion — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/oc-5.json)
  - [oc-6] Re-verified after remediation by oc-8 (exit-0 repair and ask honored per D6) and oc-9 (fixtures assert those paths) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-6.json)
  - [oc-7] Re-verified after remediation by oc-9 (skip fails closed, payloads asserted) and oc-10 (the gate's first catch closed) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-7.json)
  - [oc-8] Applied D6: exit-0 updatedInput repairs now land in output.args, permissionDecision ask throws with bee's reason, additionalContext is logged not dropped, unparseable exit-0 stdout throws fail-closed; chat.message's output.message.id dereference wrapped in try/catch (F6). Live-proved via direct plugin invocation against a stubbed bee binary (all 5 cases). oc-9's opencode_plugin_contracts suite still 4 passed, unmodified and unaffected. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-8.json)
  - [oc-9] Closed F1/F3/F4/F5 in the OpenCode parity suite: fail-by-default (opt-out env var) on missing node/opencode capability, exact per-row payload+D6 verdict assertions, binary-derived tool-registry coverage gate (caught real unmapped lsp tool by name), and a properly line-scoped named-gap check; plugin left unchanged, lsp/list recorded as named gaps in discovery.md — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/oc-9.json)
  - [oc-10] lsp and list mapped through write-guard; fixture rows added; registry gate green — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/oc-10.json)
  - [oc-11] models.opencode is now a real, resolvable config key (3 readers widened + tests); .opencode/agent/bee-{build,gather,extract,review}.md hand-authored and live-verified against opencode 1.18.16, each pinning a free-tier model with mode: subagent and write-denying permission for the three read-only agents. — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/oc-11.json)
  - [oc-13] bee onboard --apply now vendors .opencode/skills/ and .opencode/plugins/bee-guard.ts idempotently (live-proved by re-applying against this repo: up_to_date on the second pass, plus two new fixture tests); status_full's RUNTIMES/normalize_models/agent-drift check and plugin_distribution's --runtime opencode branch widen to opencode with correct semantics; docs/06-runtime-integration.md rewritten 2->3 runtimes, fixing the stale bee-render/1 string and the deleted-catalog.mjs pointer in passing. Found and fixed a real pre-existing gap: the committed .opencode/skills/ tree (from oc-4's interim regen path) had no .bee-skills-version.json stamp, which would have permanently blocked every future bee onboard --apply against this repo. — feature-wide sync per the scribing stamp, 16 file(s) changed (trace .bee/cells/oc-13.json)
  - [oc-14] Release manifest covers .opencode/plugins/, onboarding renders the four .opencode/agent worker files, docs 01/02/06 name three runtimes with no surviving Node-era claim — feature-wide sync per the scribing stamp, 13 file(s) changed (trace .bee/cells/oc-14.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell oc-1 — save as docs/knowledge/patterns/opencode-support-oc-1-pitfall.md

---
type: bee.pattern
title: opencode-support cell oc-1 — pitfall candidate
description: "Pitfall candidate mined from cell oc-1's capped trace: f77f756b8e84"
timestamp: 2026-08-11
bee:
  id: opencode-support-oc-1-pitfall
  lifecycle: draft
  areas: [hook-runtime, onboarding]
  sources: [.bee/cells/oc-1.json]
  polarity: pitfall
---

# opencode-support cell oc-1 — pitfall candidate

## What the cell did

Installed opencode-ai@1.18.16 via npm, confirmed no third-party provider auth but a live opencode/* free-model session works, and verified plugin/skill/agent on-disk layout via a scratch probe

## Recorded evidence (verbatim from .bee/cells/oc-1.json)

- **failure_signature** — f77f756b8e84

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell oc-2 — save as docs/knowledge/patterns/opencode-support-oc-2-pitfall.md

---
type: bee.pattern
title: opencode-support cell oc-2 — pitfall candidate
description: "Pitfall candidate mined from cell oc-2's capped trace: apply_patch, a registered write-capable OpenCode tool, bypassed the guard through a TypeScript-side default allow"
timestamp: 2026-08-11
bee:
  id: opencode-support-oc-2-pitfall
  lifecycle: draft
  areas: [hook-runtime, onboarding]
  sources: [.bee/cells/oc-2.json]
  polarity: pitfall
---

# opencode-support cell oc-2 — pitfall candidate

## What the cell did

Re-verified after remediation by oc-3 (apply_patch mapped) and oc-9 (registry gate holds it shut)

## Recorded evidence (verbatim from .bee/cells/oc-2.json)

- **failure_signature** — apply_patch, a registered write-capable OpenCode tool, bypassed the guard through a TypeScript-side default allow

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell oc-6 — save as docs/knowledge/patterns/opencode-support-oc-6-pitfall.md

---
type: bee.pattern
title: opencode-support cell oc-6 — pitfall candidate
description: "Pitfall candidate mined from cell oc-6's capped trace: bee's exit-0 repair and ask verdicts were dropped, leaving the model-guard belt inert on OpenCode"
timestamp: 2026-08-11
bee:
  id: opencode-support-oc-6-pitfall
  lifecycle: draft
  areas: [hook-runtime, onboarding]
  sources: [.bee/cells/oc-6.json]
  polarity: pitfall
---

# opencode-support cell oc-6 — pitfall candidate

## What the cell did

Re-verified after remediation by oc-8 (exit-0 repair and ask honored per D6) and oc-9 (fixtures assert those paths)

## Recorded evidence (verbatim from .bee/cells/oc-6.json)

- **failure_signature** — bee's exit-0 repair and ask verdicts were dropped, leaving the model-guard belt inert on OpenCode

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell oc-7 — save as docs/knowledge/patterns/opencode-support-oc-7-pitfall.md

---
type: bee.pattern
title: opencode-support cell oc-7 — pitfall candidate
description: "Pitfall candidate mined from cell oc-7's capped trace: the suite could report green while proving nothing: invisible environment skip plus unasserted payloads"
timestamp: 2026-08-11
bee:
  id: opencode-support-oc-7-pitfall
  lifecycle: draft
  areas: [hook-runtime, onboarding]
  sources: [.bee/cells/oc-7.json]
  polarity: pitfall
---

# opencode-support cell oc-7 — pitfall candidate

## What the cell did

Re-verified after remediation by oc-9 (skip fails closed, payloads asserted) and oc-10 (the gate's first catch closed)

## Recorded evidence (verbatim from .bee/cells/oc-7.json)

- **failure_signature** — the suite could report green while proving nothing: invisible environment skip plus unasserted payloads

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell oc-10 — save as docs/knowledge/patterns/opencode-support-oc-10-pitfall.md

---
type: bee.pattern
title: opencode-support cell oc-10 — pitfall candidate
description: "Pitfall candidate mined from cell oc-10's capped trace: fc1850e0fd1b"
timestamp: 2026-08-11
bee:
  id: opencode-support-oc-10-pitfall
  lifecycle: draft
  areas: [hook-runtime, onboarding]
  sources: [.bee/cells/oc-10.json]
  polarity: pitfall
---

# opencode-support cell oc-10 — pitfall candidate

## What the cell did

lsp and list mapped through write-guard; fixture rows added; registry gate green

## Recorded evidence (verbatim from .bee/cells/oc-10.json)

- **failure_signature** — fc1850e0fd1b

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 15 capped cell(s) mined, 1 delivery draft, 24 area bullet(s), 5 pattern candidate(s), 0 file(s) written.