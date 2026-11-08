---
type: bee.delivery
title: verification-ships-to-hosts — delivery
description: "Delivery record proposed by bee knowledge promote for work item verification-ships-to-hosts: 4 capped cell(s), 9 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: verification-ships-to-hosts-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/verification-ships-to-hosts/CONTEXT.md, docs/history/verification-ships-to-hosts/plan.md]
  sources: [docs/history/verification-ships-to-hosts/CONTEXT.md, docs/history/verification-ships-to-hosts/plan.md, .bee/cells/archive/verification-ships-to-hosts/vsh-1.json, .bee/cells/archive/verification-ships-to-hosts/vsh-2.json, .bee/cells/archive/verification-ships-to-hosts/vsh-3.json, .bee/cells/archive/verification-ships-to-hosts/vsh-4.json]
---

# verification-ships-to-hosts — Delivery

## What shipped

- **vsh-1** — bee-verifying and bee-verify-upkeep land in the bee-* namespace, rendered into all five committed skill trees (12 file(s) changed)
- **vsh-2** — bee renders .bee/verify/ into all three runtime skill homes, copy-only: creates and updates, never removes (3 file(s) changed)
- **vsh-3** — onboard now offers bee-verifying to any repo declaring no test command, mutually exclusive with the retired-verify warnings (2 file(s) changed)
- **vsh-4** — bee-verifying gained the two-moment consent split, the append-never-replace composition rules, the bash-prefix reason, the .bee/config.json write path and the fast/full drive contract (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **vsh-1** — `bee dev regen && bee dev release-manifest --check`
- **vsh-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard`
- **vsh-3** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml notices`
- **vsh-4** — `bee dev regen && bee dev release-manifest --check`

## Deviations

- **vsh-1** — Committed .bee/onboarding.json, a file the cell did not name — the cell's own mandated `bee dev regen` rewrites its updated_at timestamp and the repo commits that file with every regen commit (8c898a37, a3aaa48b); reserved it under vsh-w1 before committing rather than leaving the worktree dirty — something else had to be fixed first
- **vsh-1** — Named THREE runtime skill homes in both skill bodies (.claude/skills, .agents/skills, .opencode/skills) where D3 names two — plan.md load-bearing claim 6 read templates.rs:317-321 and found repo-opencode is a third target, so writing D3's two verbatim would have shipped a false instruction — the plan was wrong about a fact
- **vsh-1** — Added a Helpers line stating a shipped script is invoked as `bash <path>` because the render strips the executable bit — plan.md claim 5 proves it by ran evidence, and the source generator's 'any script the skill ships is executable' would otherwise be a wrong instruction inside a rendered copy — found a better route
- **vsh-1** — Added a bee-idiom metadata block (version/ecosystem/dependencies) to both frontmatters, which the two pstack sources did not carry — every other bee-* skill has one, and skill-body wording is Agent's Discretion per CONTEXT.md — found a better route
- **vsh-1** — Capped with --sync-ack instead of repairing affects_skills — the cell predicted only the two SKILL.md paths while its own files list also named the three references/feature-map-example/ files, and `bee cells update` refuses inside a granted feature worktree (it reads the main checkout's control plane) — hit an unforeseen obstacle
- **vsh-1** — sync-ack: cell predicted only the two SKILL.md paths, but its own files list named the three references/feature-map-example/ files that shipped with bee-verifying; bee cells update refuses inside a granted feature worktree, so the prediction could not be repaired from here
- **vsh-2** — followed the plan
- **vsh-3** — Added five tests to notices.rs and isolated the existing stale_advisor_key_is_reported fixture by giving it a test command — the cell named no test work, but the retired-verify branch had zero coverage and the advisor test asserted a full notice list that the new notice legitimately joins — something else had to be fixed first
- **vsh-4** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work verification-ships-to-hosts` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/verification-ships-to-hosts/CONTEXT.md`, `docs/history/verification-ships-to-hosts/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
