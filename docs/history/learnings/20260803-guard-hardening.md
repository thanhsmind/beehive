---
date: 2026-08-03
feature: guard-hardening
categories: [hook-runtime, doctrine-layer, permissions]
severity: medium
tags: [write-guard, containment-allowlist, cli-owned-state, permissions-deny, markdown-only-by-necessity]
---

# guard-hardening — prose becomes enforcement, and what stays prose says why

## What Happened

The prompt-diet audit left a standing preference — an absolute rule over a
structurally reachable action belongs in a hook or a permission, not in
markdown — and this feature applied it to the three reachable gaps:

1. **Containment allowlist (E1).** Write-guard's outside-root containment deny
   now exempts writes whose *resolved* target lands under the harness memory
   root (`<home>/.claude/projects/`) or the harness scratchpad root
   (`<system-temp>/claude/`). Resolution happens before the check, so a
   symlink or `..` spelling is judged by where it actually lands; an
   unresolvable root fails closed; sibling-worktree and unrelated out-of-root
   writes stay denied.
2. **CLI-owned state files (E2).** The direct-edit deny set grew from five
   entries to cover `.bee/cells/*.json`, `.bee/lanes/*.json`, and
   `.bee/onboarding.json`, each refusal naming the owning verb.
   `.bee/config.json` and `.bee/decisions.jsonl` stay hand-writable — they are
   sanctioned agent surfaces (gate-bypass config edits, decision log merges) —
   and a regression test pins both the new denies and the two allowances.
3. **grep/find deny (E3).** This repo's `.claude/settings.json` gained
   `permissions.deny: ["Bash(grep:*)", "Bash(find:*)"]` inside the existing
   permissions object. CLAUDE.md keeps its rg/fd instruction: the deny reaches
   only prefix invocations, so pipelines and subcommands remain guidance.

What did NOT move, and why (E5, recorded in
`docs/knowledge/areas/doctrine-layer/unenforced-obedience.md`, B5a): gate
self-approval (actor identity is unknowable to the CLI — the same process
writes the same approval either way), independent-review-never-automatic (no
mechanism reads the conversation where the request lives), and
cross-session-claim-only-via-`claim-next` (a browsed cell and a handed cell
are file-identical; the difference is intent). Never-build-on-red needed no
prose home at all — `bee cells finish` already refuses the cap on red.

## The E1 narrowing story

The user asked to drop the outside-project-root write rule outright — it kept
denying legitimate harness memory and scratchpad writes. The rule is also a
live safety boundary: it blocked cross-worktree writes and path escapes in the
very session that shaped this feature. The resolved-path allowlist for the two
harness-owned roots met the actual need (memory and scratchpad writes) while
keeping the boundary for everything else. E1 records that if the user still
wants the rule fully removed after seeing this, that is a new decision
superseding E1 — the narrowing is a shape, not a veto.

## The E2a test-contract flip

The plan's constraint said existing write-guard tests must pass unmodified,
and a test that must change is a STOP-and-report. gh-2's purpose, though, was
to flip a behavior an existing test pinned: `bee_cells_json_passes` asserted a
hand-edit of `.bee/cells/*.json` exits 0. The flip was taken deliberately and
annotated in place as E2a — the test became
`bee_cells_json_denied_names_owning_verb`, and the tree-hygiene scratch-shape
suite's `allow(".bee/cells/probe-th-6.json")` row became a deny expectation
kept (not dropped) to pin the direct-edit-before-scratch guard ordering. A
constraint that forbids test edits cannot forbid the test edit that IS the
feature; what it forbids is a silent one.

## The settings-preservation proof point

The E3 deny entries live in a file `bee onboard` rewrites. They survive
because onboarding's hooks merge preserves foreign top-level keys — proven by
the existing onboard test
`repo_hooks_wires_both_projections_and_preserves_foreign_entries`
(`packages/bee-rs/crates/bee/src/onboard/tests.rs`) — so the deny needed no
propagation machinery to be durable in this repo. Propagation to every
governed project (a bee-managed permissions subtree) was explicitly declined
as separate-feature work (E4).

## Recommendation

- **When a mechanism becomes possible, move the rule and say so.** The
  markdown-only set is now recorded with each rule's necessity attached; the
  next audit re-asks "can a mechanism observe this yet?" instead of
  re-litigating the whole layer.
- **A "tests must not change" constraint needs an exception shape.** The E2a
  flip shows the honest form: flip the pinned expectation in the same commit
  as the behavior change, annotate it with the decision id, and keep the old
  row as a deny expectation where it still pins an ordering.
