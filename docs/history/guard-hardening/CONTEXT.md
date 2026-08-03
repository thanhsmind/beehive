# guard-hardening — CONTEXT

Close the enforcement gaps the prompt-diet audit surfaced (deterministic
backstop preference, prompt-writing-standard) and fix the recurring
user-reported pain: memory/scratchpad writes outside the project root
are denied by write-guard containment. User approved the feature
("ok làm tiếp đợt vá lỗ enforcement luôn") and separately asked to drop
the outside-project-root write rule entirely; E1 records the narrower
shape actually taken and why.

## Locked decisions

- E1 **Containment stays; harness-owned surfaces get a fail-closed
  allowlist.** Write-guard's out-of-root containment deny is a live
  safety boundary (it blocked cross-worktree writes and path escapes in
  the very session that shaped this feature). Instead of removing it,
  writes are additionally allowed when the resolved, canonical target
  is under (a) the harness memory root `<home>/.claude/projects/` or
  (b) the harness scratchpad root `<system-temp>/claude/`. Resolution
  happens before the check (no traversal/symlink escapes into the
  allowlist); anything else outside the root stays denied. If the user
  still wants the rule fully removed after seeing this, that is a new
  decision superseding E1.
- E2 **The CLI-owned direct-edit deny set extends** to `.bee/cells/*.json`,
  `.bee/lanes/*.json`, and `.bee/onboarding.json`. `.bee/config.json`
  and `.bee/decisions.jsonl` remain hand-writable (sanctioned agent
  surfaces: gate-bypass config edits, decision log merges).
- E3 **`grep`/`find` get a deterministic backstop** in this repo's
  `.claude/settings.json`: `permissions.deny` gains `Bash(grep:*)` and
  `Bash(find:*)`. Onboarding's merge preserves foreign top-level keys
  (proven by onboard test `repo_hooks_wires_both_projections_and_
  preserves_foreign_entries`), so the entries survive re-onboarding.
  CLAUDE.md keeps its rg/fd instruction — the deny only reaches
  prefix invocations; pipelines and subcommands stay guidance.
- E4 **No propagation machinery this feature.** A bee-managed
  `permissions` subtree (template + merge function) that would carry
  deny entries to every governed project is real new code — separate
  feature if wanted.
- E5 **Rules that stay markdown-only stay by necessity, recorded as
  such**: gate self-approval (actor identity is unknowable to the CLI),
  independent-review-never-automatic, claim-only-via-claim-next.
  Build-on-red is already CLI-enforced at `bee cells finish`.
- E6 **Proof.** Every guard change lands with tests beside the existing
  write-guard test coverage; the full declared suite is the cap gate.

## Out of scope

- Managed permissions propagation to governed projects (E4).
- Any relaxation of reservation/hold or gate-boundary enforcement.
- The Node-delegation gaps found during prompt-diet (chips filed:
  compounding-complete close, worktree-cwd test).
